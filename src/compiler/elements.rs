//! The model as a flat graph of identified elements and relationships.
//!
//! Every construct of a compiled model becomes either an [`ElementRecord`]
//! (something that exists: a component, a function, a requirement, a type)
//! or a [`RelationshipRecord`] (something that links two elements: a trace,
//! an exchange, a deployment, a typing). Each has a deterministic UUID, a
//! kind from the metamodel, an owner, and its typed attributes.
//!
//! This is the representation the Systems Modeling API serves. It is built
//! from the AST after type resolution, so attributes are EFFECTIVE values.
//! A relationship whose end cannot be resolved to an element is never
//! dropped silently: it is listed in [`ElementGraph::unresolved`].

use super::ast::*;
use super::identity::element_uuid;
use super::semantic::SemanticModel;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct ElementRecord {
    pub uuid: String,
    /// The ArcLang identifier (`id:` or the name).
    pub id: String,
    pub kind: &'static str,
    pub name: String,
    /// UUID of the owning element; `None` only for the model root.
    pub owner: Option<String>,
    pub attributes: HashMap<String, AttributeValue>,
    /// Kind-specific data that is not an attribute (a constraint's verdict,
    /// a port's direction...). Sorted for deterministic output.
    pub extra: BTreeMap<&'static str, Value>,
}

#[derive(Debug, Clone)]
pub struct RelationshipRecord {
    pub uuid: String,
    pub kind: &'static str,
    pub name: Option<String>,
    pub source: String,
    pub target: String,
    pub attributes: HashMap<String, AttributeValue>,
    pub extra: BTreeMap<&'static str, Value>,
}

#[derive(Debug, Clone, Default)]
pub struct ElementGraph {
    pub elements: Vec<ElementRecord>,
    pub relationships: Vec<RelationshipRecord>,
    /// Relationships declared in the model whose ends do not resolve.
    pub unresolved: Vec<String>,
}

/// JSON form of an attribute value: quantities keep their unit AND carry
/// their canonical SI value, so API clients never re-parse strings.
pub fn attribute_json(value: &AttributeValue) -> Value {
    match value {
        AttributeValue::String(text) => json!(text),
        AttributeValue::Number(n) => json!(n),
        AttributeValue::Boolean(b) => json!(b),
        AttributeValue::Quantity(q) => json!({
            "@type": "Quantity",
            "value": q.value,
            "unit": q.unit,
            "dimension": q.dimension().label(),
            "canonicalValue": q.canonical(),
            "canonicalUnit": q.dimension().canonical_unit(),
        }),
        AttributeValue::List(items) => Value::Array(items.iter().map(attribute_json).collect()),
        AttributeValue::Map(map) => attributes_json(map),
    }
}

/// Attributes as a JSON object with sorted keys.
pub fn attributes_json(attributes: &HashMap<String, AttributeValue>) -> Value {
    let sorted: BTreeMap<&String, &AttributeValue> = attributes.iter().collect();
    let mut object = Map::new();
    for (key, value) in sorted {
        object.insert(key.clone(), attribute_json(value));
    }
    Value::Object(object)
}

struct Builder {
    graph: ElementGraph,
    used: HashSet<String>,
    /// id, name and `Owner.port` → element uuid (first declaration wins).
    lookup: HashMap<String, String>,
    types: HashMap<String, String>,
}

fn direction_text(direction: &PortDirection) -> &'static str {
    match direction {
        PortDirection::In => "in",
        PortDirection::Out => "out",
        PortDirection::InOut => "inout",
    }
}

/// Owner of a capability: the capability it is declared inside, else `root`.
fn capability_owner(builder: &Builder, parent: Option<&str>, root: &str) -> String {
    parent
        .and_then(|parent| builder.resolve(parent))
        .unwrap_or_else(|| root.to_string())
}

fn empty() -> HashMap<String, AttributeValue> {
    HashMap::new()
}

impl Builder {
    fn unique_uuid(&mut self, namespace: &str, kind: &str, id: &str) -> String {
        let mut uuid = element_uuid(namespace, id);
        if self.used.contains(&uuid) {
            // Two different elements share an id: qualify by kind.
            uuid = element_uuid(kind, id);
        }
        let mut n = 2;
        while self.used.contains(&uuid) {
            uuid = element_uuid(kind, &format!("{}#{}", id, n));
            n += 1;
        }
        self.used.insert(uuid.clone());
        uuid
    }

    fn element(
        &mut self,
        kind: &'static str,
        id: &str,
        name: &str,
        owner: &str,
        attributes: &HashMap<String, AttributeValue>,
    ) -> String {
        let id = attributes.get("id").and_then(|v| v.as_string()).unwrap_or(id);
        let id = if id.is_empty() { name } else { id };
        let uuid = self.unique_uuid("element", kind, id);
        for key in [id, name] {
            if !key.is_empty() {
                self.lookup.entry(key.to_string()).or_insert_with(|| uuid.clone());
            }
        }
        self.graph.elements.push(ElementRecord {
            uuid: uuid.clone(),
            id: id.to_string(),
            kind,
            name: if name.is_empty() { id.to_string() } else { name.to_string() },
            owner: Some(owner.to_string()),
            attributes: attributes.clone(),
            // A stated multiplicity is also served parsed: clients need
            // bounds, not text.
            extra: super::multiplicity::Multiplicity::of(kind, attributes)
                .map(|multiplicity| ("multiplicity", json!({ "lower": multiplicity.lower, "upper": multiplicity.upper })))
                .into_iter()
                .collect(),
        });
        uuid
    }

    fn set_extra(&mut self, key: &'static str, value: Value) {
        if let Some(last) = self.graph.elements.last_mut() {
            last.extra.insert(key, value);
        }
    }

    /// A port: addressable as `OwnerId.port` and `OwnerName.port`.
    fn port(&mut self, kind: &'static str, owner_uuid: &str, owner_keys: [&str; 2], name: &str, extra: Vec<(&'static str, Value)>) {
        let id = format!("{}.{}", owner_keys[0], name);
        let uuid = self.unique_uuid("element", kind, &id);
        for owner_key in owner_keys {
            if !owner_key.is_empty() {
                self.lookup.entry(format!("{}.{}", owner_key, name)).or_insert_with(|| uuid.clone());
            }
        }
        self.graph.elements.push(ElementRecord {
            uuid,
            id,
            kind,
            name: name.to_string(),
            owner: Some(owner_uuid.to_string()),
            attributes: empty(),
            extra: extra.into_iter().collect(),
        });
    }

    /// Resolve a reference: exact id/name/port path, else the owner of a
    /// dotted path (`Component.unknownPort` → the component).
    fn resolve(&self, reference: &str) -> Option<String> {
        if let Some(uuid) = self.lookup.get(reference) {
            return Some(uuid.clone());
        }
        let (owner, _) = reference.rsplit_once('.')?;
        self.lookup.get(owner).cloned()
    }

    #[allow(clippy::too_many_arguments)]
    fn relate(
        &mut self,
        kind: &'static str,
        name: Option<&str>,
        from: &str,
        to: &str,
        attributes: &HashMap<String, AttributeValue>,
        extra: Vec<(&'static str, Value)>,
    ) {
        match (self.resolve(from), self.resolve(to)) {
            (Some(source), Some(target)) => self.relate_resolved(kind, name, source, target, attributes, extra),
            (source, target) => {
                let mut missing = Vec::new();
                if source.is_none() {
                    missing.push(from);
                }
                if target.is_none() {
                    missing.push(to);
                }
                self.graph.unresolved.push(format!(
                    "{} {}{} -> {}: unresolved end(s): {}",
                    kind,
                    name.map(|n| format!("'{}' ", n)).unwrap_or_default(),
                    from,
                    to,
                    missing.join(", ")
                ));
            }
        }
    }

    fn relate_resolved(
        &mut self,
        kind: &'static str,
        name: Option<&str>,
        source: String,
        target: String,
        attributes: &HashMap<String, AttributeValue>,
        extra: Vec<(&'static str, Value)>,
    ) {
        let identity = format!("{}|{}|{}|{}", kind, source, target, name.unwrap_or(""));
        let uuid = self.unique_uuid("relationship", kind, &identity);
        self.graph.relationships.push(RelationshipRecord {
            uuid,
            kind,
            name: name.filter(|n| !n.is_empty()).map(str::to_string),
            source,
            target,
            attributes: attributes.clone(),
            extra: extra.into_iter().collect(),
        });
    }

    /// `involves: [...]` of a chain, path or capability, in order.
    fn involvements(&mut self, owner: &str, owner_label: &str, members: &[String]) {
        for (index, member) in members.iter().enumerate() {
            match self.resolve(member) {
                Some(target) => self.relate_resolved(
                    "Involvement",
                    None,
                    owner.to_string(),
                    target,
                    &empty(),
                    vec![("order", json!(index + 1))],
                ),
                // Chains also name exchanges, which are relationships, not
                // elements: record them rather than pretend they resolved.
                None => self.graph.unresolved.push(format!(
                    "Involvement '{}' -> {}: not an element (exchanges are relationships)",
                    owner_label, member
                )),
            }
        }
    }

    /// `extends:` / `includes:` / `specializes:` among the capabilities of
    /// one level, given as `(id, name, attributes)`.
    fn capability_relations(&mut self, level: &[(&str, &str, &HashMap<String, AttributeValue>)]) {
        let names: Vec<(&str, &str)> = level.iter().map(|(id, name, _)| (*id, *name)).collect();
        for (id, _, attributes) in level {
            for (relation, reference) in super::capability_relations::declared_relations(attributes) {
                match super::capability_relations::resolve_in_level(&names, reference) {
                    Some(target) => self.relate(relation.relationship(), None, id, target, &empty(), Vec::new()),
                    None => self.graph.unresolved.push(format!(
                        "{} '{}' -> {}: not a capability of the same level",
                        relation.relationship(), id, reference
                    )),
                }
            }
        }
    }

    fn typings(&mut self, element: &str, label: &str, attributes: &HashMap<String, AttributeValue>) {
        let Ok(names) = super::types::declared_types(attributes) else { return };
        for name in names {
            match self.types.get(name).cloned() {
                Some(target) => self.relate_resolved("Typing", None, element.to_string(), target, &empty(), Vec::new()),
                None => self.graph.unresolved.push(format!("Typing '{}' -> type '{}': unknown type", label, name)),
            }
        }
    }

    fn typed_element(
        &mut self,
        kind: &'static str,
        id: &str,
        name: &str,
        owner: &str,
        attributes: &HashMap<String, AttributeValue>,
    ) -> String {
        let uuid = self.element(kind, id, name, owner, attributes);
        self.typings(&uuid, if id.is_empty() { name } else { id }, attributes);
        uuid
    }

    fn activity(&mut self, activity: &OperationalActivity, owner: &str) {
        let uuid = self.typed_element("OperationalActivity", &activity.id, &activity.name, owner, &activity.attributes);
        for sub in &activity.sub_activities {
            self.activity(sub, &uuid);
        }
    }

    fn function(&mut self, function: &SystemFunction, owner: &str) {
        let uuid = self.typed_element("SystemFunction", &function.id, &function.name, owner, &function.attributes);
        let id = function.attributes.get("id").and_then(|v| v.as_string()).unwrap_or(&function.id).to_string();
        for port in &function.ports {
            self.port(
                "FunctionPort",
                &uuid,
                [&id, &function.name],
                &port.name,
                vec![("direction", json!(direction_text(&port.direction))), ("dataType", json!(port.data_type))],
            );
        }
        for sub in &function.sub_functions {
            self.function(sub, &uuid);
        }
    }

    fn component(&mut self, component: &LogicalComponent, owner: &str) {
        let uuid = self.typed_element("LogicalComponent", &component.id, &component.name, owner, &component.attributes);
        let id = component.attributes.get("id").and_then(|v| v.as_string()).unwrap_or(&component.id).to_string();
        for port in &component.ports {
            self.port(
                "ComponentPort",
                &uuid,
                [&id, &component.name],
                &port.name,
                vec![("direction", json!(direction_text(&port.direction))), ("interface", json!(port.interface_type))],
            );
        }
        for (interfaces, direction) in [(&component.interfaces_in, "in"), (&component.interfaces_out, "out")] {
            for interface in interfaces {
                self.port(
                    "ComponentPort",
                    &uuid,
                    [&id, &component.name],
                    &interface.name,
                    vec![("direction", json!(direction)), ("protocol", json!(interface.protocol))],
                );
            }
        }
        for function in &component.functions {
            self.typed_element("LogicalFunction", &function.name, &function.name, &uuid, &function.attributes);
        }
        for sub in &component.sub_components {
            self.component(sub, &uuid);
        }
    }

    fn state(&mut self, state: &State, machine: &str, owner: &str) {
        let id = format!("{}.{}", machine, state.name);
        let uuid = self.element("State", &id, &state.name, owner, &empty());
        self.set_extra("stateKind", json!(match state.kind { StateKind::Mode => "mode", StateKind::State => "state" }));
        for sub in &state.sub_states {
            self.state(sub, machine, &uuid);
        }
    }
}

/// Build the element graph of a compiled model.
pub fn build(ast: &Model, semantic: &SemanticModel) -> ElementGraph {
    let model_name = semantic.name.clone().unwrap_or_else(|| "Model".to_string());
    let mut builder = Builder {
        graph: ElementGraph::default(),
        used: HashSet::new(),
        lookup: HashMap::new(),
        types: HashMap::new(),
    };
    let root = builder.unique_uuid("model", "Model", &model_name);
    builder.graph.elements.push(ElementRecord {
        uuid: root.clone(),
        id: model_name.clone(),
        kind: "Model",
        name: model_name,
        owner: None,
        attributes: ast.attributes.clone(),
        extra: BTreeMap::new(),
    });

    // ---- Types (own namespace) -------------------------------------------
    for ty in &ast.types {
        let uuid = builder.unique_uuid("type", "Type", &ty.name);
        builder.types.insert(ty.name.clone(), uuid.clone());
        let mut extra = BTreeMap::new();
        extra.insert("required", json!(ty.required));
        extra.insert(
            "ports",
            Value::Array(
                ty.ports
                    .iter()
                    .map(|p| json!({"name": p.name, "direction": direction_text(&p.direction), "interface": p.interface_type}))
                    .collect(),
            ),
        );
        builder.graph.elements.push(ElementRecord {
            uuid,
            id: ty.name.clone(),
            kind: "Type",
            name: ty.name.clone(),
            owner: Some(root.clone()),
            attributes: ty.attributes.clone(),
            extra,
        });
    }
    for ty in &ast.types {
        if let (Some(base), Some(source)) = (&ty.extends, builder.types.get(&ty.name).cloned()) {
            match builder.types.get(base).cloned() {
                Some(target) => builder.relate_resolved("Specialization", None, source, target, &empty(), Vec::new()),
                None => builder.graph.unresolved.push(format!("Specialization '{}' -> '{}': unknown type", ty.name, base)),
            }
        }
    }

    // ---- Elements ---------------------------------------------------------
    for oa in &ast.operational_analysis {
        for actor in &oa.actors {
            let id = actor.id.clone().unwrap_or_else(|| format!("ACT-{}", actor.name.replace(' ', "-")));
            builder.typed_element("Actor", &id, &actor.name, &root, &actor.attributes);
        }
        for entity in &oa.entities {
            let uuid = builder.typed_element("OperationalEntity", &entity.id, &entity.name, &root, &entity.attributes);
            for activity in &entity.activities {
                builder.activity(activity, &uuid);
            }
        }
        for capability in &oa.capabilities {
            let owner = capability_owner(&builder, capability.parent.as_deref(), &root);
            builder.typed_element("OperationalCapability", &capability.id, &capability.name, &owner, &capability.attributes);
        }
        for activity in &oa.activities {
            builder.activity(activity, &root);
        }
        for process in &oa.processes {
            builder.typed_element("OperationalProcess", &process.id, &process.name, &root, &process.attributes);
        }
    }
    for sa in &ast.system_analysis {
        for requirement in &sa.requirements {
            let title = requirement
                .attributes
                .get("title")
                .or_else(|| requirement.attributes.get("name"))
                .and_then(|v| v.as_string())
                .unwrap_or(&requirement.id)
                .to_string();
            builder.typed_element("Requirement", &requirement.id, &title, &root, &requirement.attributes);
        }
        for function in &sa.functions {
            builder.function(function, &root);
        }
        for component in &sa.components {
            builder.typed_element("SystemComponent", "", &component.name, &root, &component.attributes);
        }
        for actor in &sa.external_actors {
            builder.typed_element("SystemActor", &actor.id, &actor.name, &root, &actor.attributes);
        }
        for mission in &sa.missions {
            builder.typed_element("Mission", &mission.id, &mission.name, &root, &mission.attributes);
        }
        for capability in &sa.capabilities {
            let owner = capability_owner(&builder, capability.parent.as_deref(), &root);
            builder.typed_element("Capability", &capability.id, &capability.name, &owner, &capability.attributes);
        }
        for chain in &sa.functional_chains {
            builder.typed_element("FunctionalChain", &chain.id, &chain.name, &root, &chain.attributes);
        }
    }
    for la in &ast.logical_architecture {
        for component in &la.components {
            builder.component(component, &root);
        }
        for capability in &la.capability_realizations {
            let owner = capability_owner(&builder, capability.parent.as_deref(), &root);
            builder.typed_element("CapabilityRealization", &capability.id, &capability.name, &owner, &capability.attributes);
        }
        for chain in &la.functional_chains {
            builder.typed_element("FunctionalChain", &chain.id, &chain.name, &root, &chain.attributes);
        }
    }
    for pa in &ast.physical_architecture {
        for node in &pa.nodes {
            let uuid = builder.typed_element("PhysicalNode", &node.id, &node.name, &root, &node.attributes);
            let id = node.attributes.get("id").and_then(|v| v.as_string()).unwrap_or(&node.id).to_string();
            for port in &node.ports {
                builder.port("PhysicalPort", &uuid, [&id, &node.name], &port.name, Vec::new());
            }
            for hardware in &node.hardware_components {
                builder.element("HardwareComponent", &hardware.id, &hardware.name, &uuid, &empty());
                builder.set_extra("hardwareType", json!(hardware.hw_type));
            }
            for behavior in &node.behavior_components {
                builder.element("BehaviorComponent", &behavior.id, &behavior.name, &uuid, &empty());
            }
        }
        for path in &pa.paths {
            builder.typed_element("PhysicalPath", &path.id, &path.name, &root, &path.attributes);
        }
    }
    for epbs in &ast.epbs {
        for system in &epbs.systems {
            let system_uuid = builder.typed_element("EpbsSystem", "", &system.name, &root, &system.attributes);
            for subsystem in &system.subsystems {
                let subsystem_uuid = builder.typed_element("EpbsSubsystem", "", &subsystem.name, &system_uuid, &subsystem.attributes);
                for item in &subsystem.items {
                    builder.typed_element("EpbsItem", "", &item.name, &subsystem_uuid, &item.attributes);
                }
            }
        }
    }
    for class in &ast.classes {
        builder.element("Class", &class.id, &class.name, &root, &class.attributes);
        builder.set_extra(
            "fields",
            Value::Array(class.fields.iter().map(|f| json!({"name": f.name, "type": f.attr_type})).collect()),
        );
    }
    for data_type in &ast.data_types {
        let kind = if data_type.enumeration_values.is_some() { "Enumeration" } else { "DataType" };
        builder.element(kind, &data_type.id, &data_type.name, &root, &empty());
        if let Some(values) = &data_type.enumeration_values {
            builder.set_extra("values", json!(values.iter().map(|v| v.name.clone()).collect::<Vec<_>>()));
        }
        if let Some(base) = &data_type.base_type {
            builder.set_extra("base", json!(base));
        }
        if let Some(unit) = &data_type.unit {
            builder.set_extra("unit", json!(unit));
        }
    }
    for item in &ast.exchange_items {
        builder.element("ExchangeItem", &item.id, &item.name, &root, &empty());
        builder.set_extra("mechanism", json!(item.stereotype));
    }
    for safety in &ast.safety_analysis {
        for hazard in &safety.hazards {
            builder.typed_element("Hazard", "", &hazard.name, &root, &hazard.attributes);
        }
        for entry in &safety.fmea {
            builder.element("FmeaEntry", "", &entry.name, &root, &entry.attributes);
        }
    }
    for test_case in &ast.test_cases {
        builder.typed_element("TestCase", &test_case.id, &test_case.name, &root, &test_case.attributes);
        builder.set_extra("method", json!(test_case.method));
    }
    for constraint in &ast.constraints {
        builder.element("Constraint", &constraint.id, &constraint.name, &root, &constraint.attributes);
        builder.set_extra("expression", json!(constraint.expression.to_string()));
        if let Some(verdict) = semantic.constraints.iter().find(|c| c.id == constraint.id) {
            builder.set_extra("satisfied", json!(verdict.satisfied));
            builder.set_extra("left", json!(verdict.left));
            builder.set_extra("right", json!(verdict.right));
        }
    }
    for machine in &ast.state_machines {
        let uuid = builder.element("StateMachine", "", &machine.name, &root, &empty());
        builder.set_extra("initial", json!(machine.initial_state));
        for state in &machine.states {
            builder.state(state, &machine.name, &uuid);
        }
    }
    for scenario in &ast.scenarios {
        builder.element("Scenario", "", &scenario.name, &root, &empty());
        builder.set_extra("participants", json!(scenario.participants.iter().map(|p| p.id.clone()).collect::<Vec<_>>()));
    }

    // ---- Relationships -----------------------------------------------------
    for oa in &ast.operational_analysis {
        for exchange in oa.exchanges.iter().chain(oa.communication_means.iter()) {
            builder.relate("OperationalExchange", exchange.label.as_deref(), &exchange.from, &exchange.to, &exchange.attributes, Vec::new());
        }
        for process in &oa.processes {
            if let Some(owner) = builder.resolve(&process.id) {
                builder.involvements(&owner, &process.id, &process.involves);
            }
        }
        for capability in &oa.capabilities {
            if let Some(owner) = builder.resolve(&capability.id) {
                builder.involvements(&owner, &capability.id, &capability.involves);
            }
        }
        let level: Vec<_> = oa.capabilities.iter().map(|c| (c.id.as_str(), c.name.as_str(), &c.attributes)).collect();
        builder.capability_relations(&level);
    }
    for sa in &ast.system_analysis {
        for exchange in &sa.functional_exchanges {
            builder.relate(
                "FunctionalExchange",
                exchange.label.as_deref(),
                &exchange.from_port,
                &exchange.to_port,
                &empty(),
                vec![("dataType", json!(exchange.data_type))],
            );
        }
        for capability in &sa.capabilities {
            if let Some(owner) = builder.resolve(&capability.id) {
                builder.involvements(&owner, &capability.id, &capability.involves);
                for (kind, reference) in [("Realization", &capability.realizes), ("Contribution", &capability.mission)] {
                    if let Some(reference) = reference {
                        builder.relate(kind, None, &capability.id, reference, &empty(), Vec::new());
                    }
                }
            }
        }
        for chain in &sa.functional_chains {
            if let Some(owner) = builder.resolve(&chain.id) {
                builder.involvements(&owner, &chain.id, &chain.involves);
            }
        }
        let level: Vec<_> = sa.capabilities.iter().map(|c| (c.id.as_str(), c.name.as_str(), &c.attributes)).collect();
        builder.capability_relations(&level);
    }
    for la in &ast.logical_architecture {
        let level: Vec<_> = la.capability_realizations.iter().map(|c| (c.id.as_str(), c.name.as_str(), &c.attributes)).collect();
        builder.capability_relations(&level);
        for interface in &la.interfaces {
            builder.relate("LogicalInterface", Some(&interface.name), &interface.from, &interface.to, &interface.attributes, Vec::new());
        }
        for exchange in &la.component_exchanges {
            builder.relate(
                "ComponentExchange",
                exchange.label.as_deref(),
                &exchange.from_port,
                &exchange.to_port,
                &empty(),
                vec![("exchangeItem", json!(exchange.exchange_item))],
            );
        }
        for capability in &la.capability_realizations {
            if let Some(owner) = builder.resolve(&capability.id) {
                builder.involvements(&owner, &capability.id, &capability.involves);
                if let Some(reference) = &capability.realizes {
                    builder.relate("Realization", None, &capability.id, reference, &empty(), Vec::new());
                }
            }
        }
        for chain in &la.functional_chains {
            if let Some(owner) = builder.resolve(&chain.id) {
                builder.involvements(&owner, &chain.id, &chain.involves);
            }
        }
    }
    for pa in &ast.physical_architecture {
        for node in &pa.nodes {
            for deployment in &node.deployments {
                builder.relate("Deployment", None, &deployment.component, &node.id, &deployment.attributes, Vec::new());
            }
        }
        for link in &pa.links {
            let mut attributes = link.attributes.clone();
            if let Some(bandwidth) = &link.bandwidth {
                attributes.entry("bandwidth".to_string()).or_insert_with(|| AttributeValue::String(bandwidth.clone()));
            }
            attributes.entry("protocol".to_string()).or_insert_with(|| AttributeValue::String(link.protocol.clone()));
            let before = builder.graph.relationships.len();
            builder.relate("PhysicalLink", Some(&link.name), &link.from, &link.to, &attributes, Vec::new());
            // A link is typed like an element; its typings hang off the link.
            if builder.graph.relationships.len() > before {
                let uuid = builder.graph.relationships[before].uuid.clone();
                builder.lookup.entry(link.name.clone()).or_insert_with(|| uuid.clone());
                builder.typings(&uuid, &link.name, &link.attributes);
            }
        }
        for exchange in &pa.physical_exchanges {
            let mut extra = vec![("messageType", json!(exchange.message_type))];
            if let Some(via) = &exchange.via {
                extra.push(("via", json!(via)));
            }
            if let Some(frequency) = &exchange.frequency {
                extra.push(("frequency", json!(frequency)));
            }
            builder.relate("PhysicalExchange", exchange.label.as_deref(), &exchange.from, &exchange.to, &empty(), extra);
        }
        for path in &pa.paths {
            if let Some(owner) = builder.resolve(&path.id).or_else(|| builder.resolve(&path.name)) {
                builder.involvements(&owner, &path.name, &path.involves);
            }
        }
    }
    for safety in &ast.safety_analysis {
        for hazard in &safety.hazards {
            if let Some(AttributeValue::List(items)) = hazard.attributes.get("mitigated_by") {
                for item in items {
                    if let Some(requirement) = item.as_string() {
                        builder.relate("Mitigation", None, requirement, &hazard.name, &empty(), Vec::new());
                    }
                }
            }
        }
    }
    for test_case in &ast.test_cases {
        for requirement in &test_case.verifies {
            builder.relate("Verification", None, &test_case.id, requirement, &empty(), vec![("method", json!(test_case.method))]);
        }
    }
    // Traces come from the semantic model: ends are already resolved to ids.
    for trace in &semantic.traces {
        let mut extra = vec![("traceKind", json!(trace.trace_type))];
        if let Some(rationale) = &trace.rationale {
            extra.push(("rationale", json!(rationale)));
        }
        builder.relate("Trace", None, &trace.from, &trace.to, &empty(), extra);
    }
    for machine in &ast.state_machines {
        for transition in &machine.transitions {
            let mut extra = vec![("trigger", json!(transition.trigger))];
            for (key, value) in [("guard", &transition.guard), ("action", &transition.action), ("timing", &transition.timing)] {
                if let Some(value) = value {
                    extra.push((key, json!(value)));
                }
            }
            builder.relate(
                "Transition",
                None,
                &format!("{}.{}", machine.name, transition.from),
                &format!("{}.{}", machine.name, transition.to),
                &empty(),
                extra,
            );
        }
    }
    for scenario in &ast.scenarios {
        for (index, message) in scenario.messages.iter().enumerate() {
            let mut extra = vec![("order", json!(index + 1)), ("scenario", json!(scenario.name))];
            if let Some(timing) = &message.timing {
                extra.push(("timing", json!(timing)));
            }
            builder.relate("Message", Some(&message.label), &message.from, &message.to, &empty(), extra);
        }
    }

    builder.graph
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{Compiler, CompilerConfig};

    fn graph(source: &str) -> ElementGraph {
        let result = Compiler::new(CompilerConfig::default()).compile_string(source).expect("compiles");
        build(&result.ast, &result.semantic_model)
    }

    const MODEL: &str = r#"
model Demo {}
type "ECU" { voltage: 12 V }
type "Safety ECU" extends "ECU" { safety_level: "ASIL-B" }
system_analysis "SA" {
    requirement "REQ-1" { description: "react fast" }
    function "Detect" { id: "SF-1" latency: 30 ms port out lane { data_type: "Lane" } }
    function "Decide" { id: "SF-2" latency: 10 ms port in lane { data_type: "Lane" } }
    functional_exchange "SF-1.lane" -> "SF-2.lane" { label: "lane" }
    functional_chain "Chain" { id: "FC-1" involves: ["SF-1", "SF-2"] }
}
architecture logical {
    component "Controller" { id: "LC-1" function "Plan" component "Core" { id: "LC-1-1" } }
}
architecture physical {
    node "Chassis ECU" { id: "PN-1" is: "Safety ECU" deploys "LC-1" }
    node "Camera ECU" { id: "PN-2" }
    link "CAN" { from: "PN-1" to: "PN-2" protocol: "CAN FD" }
}
constraint "Budget" { id: "CST-1" assert: sum("FC-1", latency) <= 50 ms }
test_case "TC-1" { verifies: ["REQ-1"] method: "test" }
trace "LC-1" satisfies "REQ-1"
"#;

    #[test]
    fn every_construct_is_an_element_or_a_relationship_with_a_unique_uuid() {
        let graph = graph(MODEL);
        let mut uuids = HashSet::new();
        for uuid in graph.elements.iter().map(|e| &e.uuid).chain(graph.relationships.iter().map(|r| &r.uuid)) {
            assert!(uuids.insert(uuid.clone()), "duplicate uuid {uuid}");
        }
        let kinds: Vec<&str> = graph.elements.iter().map(|e| e.kind).collect();
        for kind in ["Model", "Type", "Requirement", "SystemFunction", "FunctionPort", "FunctionalChain", "LogicalComponent", "LogicalFunction", "PhysicalNode", "Constraint", "TestCase"] {
            assert!(kinds.contains(&kind), "missing {kind} in {kinds:?}");
        }
        let relationships: Vec<&str> = graph.relationships.iter().map(|r| r.kind).collect();
        for kind in ["Specialization", "Typing", "FunctionalExchange", "Involvement", "Deployment", "PhysicalLink", "Verification", "Trace"] {
            assert!(relationships.contains(&kind), "missing {kind} in {relationships:?}");
        }
        assert!(graph.unresolved.is_empty(), "{:?}", graph.unresolved);
    }

    #[test]
    fn ownership_identity_and_effective_attributes() {
        let graph = graph(MODEL);
        let find = |id: &str| graph.elements.iter().find(|e| e.id == id).unwrap_or_else(|| panic!("no element {id}"));
        let root = &graph.elements[0];
        assert_eq!((root.kind, root.owner.as_ref()), ("Model", None));
        assert_eq!(find("LC-1").owner.as_ref(), Some(&root.uuid));
        assert_eq!(find("LC-1-1").owner.as_ref(), Some(&find("LC-1").uuid), "nested component is owned by its parent");
        assert_eq!(find("SF-1.lane").owner.as_ref(), Some(&find("SF-1").uuid));
        // Same identity as everywhere else in the toolchain.
        assert_eq!(find("LC-1").uuid, element_uuid("element", "LC-1"));
        // Inherited through `is:` — effective value, with its unit.
        let node = find("PN-1");
        assert_eq!(attribute_json(&node.attributes["voltage"])["canonicalValue"], json!(12.0));
        assert_eq!(node.attributes["safety_level"].display(), "ASIL-B");
        let constraint = find("CST-1");
        assert_eq!(constraint.extra["satisfied"], json!(true));
        assert_eq!(constraint.extra["left"], json!("40 ms"));
    }

    #[test]
    fn relationships_resolve_ports_and_keep_order() {
        let graph = graph(MODEL);
        let uuid_of = |id: &str| graph.elements.iter().find(|e| e.id == id).unwrap().uuid.clone();
        let exchange = graph.relationships.iter().find(|r| r.kind == "FunctionalExchange").unwrap();
        assert_eq!((exchange.source.clone(), exchange.target.clone()), (uuid_of("SF-1.lane"), uuid_of("SF-2.lane")));
        let orders: Vec<_> = graph
            .relationships
            .iter()
            .filter(|r| r.kind == "Involvement")
            .map(|r| (r.extra["order"].clone(), r.target.clone()))
            .collect();
        assert_eq!(orders, vec![(json!(1), uuid_of("SF-1")), (json!(2), uuid_of("SF-2"))]);
        let typing = graph.relationships.iter().find(|r| r.kind == "Typing").unwrap();
        assert_eq!(typing.source, uuid_of("PN-1"));
        let specialization = graph.relationships.iter().find(|r| r.kind == "Specialization").unwrap();
        let type_uuid = |name: &str| graph.elements.iter().find(|e| e.kind == "Type" && e.name == name).unwrap().uuid.clone();
        assert_eq!((specialization.source.clone(), specialization.target.clone()), (type_uuid("Safety ECU"), type_uuid("ECU")));
    }

    #[test]
    fn unresolved_ends_are_reported_not_dropped() {
        let graph = graph(
            r#"
model U {}
system_analysis "SA" {
    function "A" { id: "SF-1" }
    functional_exchange "SF-1" -> "SF-404" { label: "ghost" }
}
"#,
        );
        assert!(graph.relationships.iter().all(|r| r.kind != "FunctionalExchange"));
        assert_eq!(graph.unresolved.len(), 1);
        assert!(graph.unresolved[0].contains("FunctionalExchange 'ghost'") && graph.unresolved[0].contains("SF-404"), "{:?}", graph.unresolved);
    }

    #[test]
    fn graph_is_deterministic() {
        let render = |g: &ElementGraph| {
            g.elements
                .iter()
                .map(|e| format!("{}|{}|{}|{}", e.uuid, e.kind, e.name, attributes_json(&e.attributes)))
                .chain(g.relationships.iter().map(|r| format!("{}|{}|{}|{}", r.uuid, r.kind, r.source, r.target)))
                .collect::<Vec<_>>()
        };
        let first = render(&graph(MODEL));
        for _ in 0..4 {
            assert_eq!(first, render(&graph(MODEL)));
        }
    }
}
