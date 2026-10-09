//! User-defined types and specialization.
//!
//! ```text
//! type "ECU" {
//!     required: ["ram"]
//!     voltage: 12 V
//!     port in power { interface: "Power12V" }
//! }
//! type "Safety ECU" extends "ECU" { safety_level: "ASIL-D" }
//!
//! component "Brake ECU" { is: "Safety ECU"  ram: 128 MB }
//! ```
//!
//! A `type` is a reusable definition: typed attributes, ports, and the list
//! of attributes its instances MUST provide. `extends` specializes another
//! type; `is:` types any model element. This module resolves both, before
//! semantic analysis, so every later stage (validation, gate, constraints,
//! exports) sees the EFFECTIVE attributes of each element.
//!
//! Rules — all violations are compile ERRORS:
//! - a type is declared once; `extends` and `is:` name declared types;
//!   specialization is acyclic;
//! - a redefinition conforms: an attribute that is a quantity in the type
//!   stays a quantity of the same dimension in the specialization/instance;
//! - every `required` attribute is provided (declared or inherited);
//! - ports are inherited by components only.
//!
//! Types live in their own namespace: a type and an element may share a name.

use super::ast::*;
use std::collections::{BTreeMap, HashMap};

/// Attribute keys that belong to the declaration itself and never flow
/// from a type to its specializations or instances.
const NOT_INHERITED: [&str; 5] = ["id", "name", "description", "required", "is"];

/// A type with everything it inherits folded in.
#[derive(Debug, Clone, Default)]
pub struct EffectiveType {
    pub attributes: HashMap<String, AttributeValue>,
    pub ports: Vec<ComponentPort>,
    pub required: Vec<String>,
    /// Base types, nearest first.
    pub ancestors: Vec<String>,
}

/// One element typed by `is:`.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeUse {
    pub type_name: String,
    pub kind: &'static str,
    pub element: String,
}

fn conforms(context: &str, key: &str, inherited: &AttributeValue, own: &AttributeValue) -> Result<(), String> {
    let Some(Ok(expected)) = inherited.as_quantity() else { return Ok(()) };
    match own.as_quantity() {
        Some(Ok(actual)) if actual.dimension() == expected.dimension() => Ok(()),
        Some(Ok(actual)) => Err(format!(
            "{}: redefinition of '{}' changes its dimension — inherited '{}' is a {}, '{}' is a {}",
            context,
            key,
            expected,
            expected.dimension().label(),
            actual,
            actual.dimension().label()
        )),
        _ => Err(format!(
            "{}: redefinition of '{}' must stay a {} quantity (inherited '{}'), got '{}'",
            context,
            key,
            expected.dimension().label(),
            expected,
            own.display()
        )),
    }
}

/// Fold every type's inheritance chain. Errors on duplicates, unknown
/// bases, cycles and non-conforming redefinitions.
pub fn effective_types(types: &[TypeDef]) -> Result<BTreeMap<String, EffectiveType>, String> {
    let mut declared: HashMap<&str, &TypeDef> = HashMap::new();
    for ty in types {
        if declared.insert(ty.name.as_str(), ty).is_some() {
            return Err(format!("type '{}' is declared more than once", ty.name));
        }
    }

    fn fold<'t>(
        name: &'t str,
        declared: &HashMap<&'t str, &'t TypeDef>,
        done: &mut BTreeMap<String, EffectiveType>,
        visiting: &mut Vec<&'t str>,
    ) -> Result<(), String> {
        if done.contains_key(name) {
            return Ok(());
        }
        if visiting.contains(&name) {
            let mut cycle: Vec<&str> = visiting.clone();
            cycle.push(name);
            return Err(format!("cyclic type specialization: {}", cycle.join(" -> ")));
        }
        let ty = declared[name];
        let mut effective = EffectiveType::default();
        if let Some(base) = &ty.extends {
            let Some((&base_name, _)) = declared.get_key_value(base.as_str()) else {
                return Err(format!("type '{}' extends unknown type '{}'", ty.name, base));
            };
            visiting.push(name);
            fold(base_name, declared, done, visiting)?;
            visiting.pop();
            let inherited = &done[base_name];
            effective.attributes = inherited.attributes.clone();
            effective.ports = inherited.ports.clone();
            effective.required = inherited.required.clone();
            effective.ancestors = std::iter::once(base_name.to_string()).chain(inherited.ancestors.iter().cloned()).collect();
        }
        let context = format!("type '{}'", ty.name);
        for (key, value) in &ty.attributes {
            if NOT_INHERITED.contains(&key.as_str()) {
                continue;
            }
            if let Some(inherited) = effective.attributes.get(key) {
                conforms(&context, key, inherited, value)?;
            }
            effective.attributes.insert(key.clone(), value.clone());
        }
        for port in &ty.ports {
            match effective.ports.iter_mut().find(|p| p.name == port.name) {
                Some(existing) => *existing = port.clone(),
                None => effective.ports.push(port.clone()),
            }
        }
        for key in &ty.required {
            if !effective.required.contains(key) {
                effective.required.push(key.clone());
            }
        }
        done.insert(name.to_string(), effective);
        Ok(())
    }

    let mut done = BTreeMap::new();
    for ty in types {
        fold(ty.name.as_str(), &declared, &mut done, &mut Vec::new())?;
    }
    Ok(done)
}

/// Names of the types an element declares with `is:` — one name, or a
/// list for multiple typing (`is: ["ECU", "Redundant"]`).
pub fn declared_types(attributes: &HashMap<String, AttributeValue>) -> Result<Vec<&str>, String> {
    match attributes.get("is") {
        None => Ok(Vec::new()),
        Some(AttributeValue::String(name)) => Ok(vec![name.as_str()]),
        Some(AttributeValue::List(items)) => {
            let mut names = Vec::with_capacity(items.len());
            for item in items {
                let name = item
                    .as_string()
                    .ok_or_else(|| format!("`is:` lists type names, got '{}'", item.display()))?;
                if names.contains(&name) {
                    return Err(format!("`is:` names type '{}' twice", name));
                }
                names.push(name);
            }
            if names.is_empty() {
                return Err("`is:` lists no type".to_string());
            }
            Ok(names)
        }
        Some(other) => Err(format!("`is:` names a type or a list of types, got '{}'", other.display())),
    }
}

/// The union of several types, as inherited by one element. An attribute
/// (or port) that two of the types define DIFFERENTLY is reported in
/// `conflicts`: the element must redefine it to resolve the ambiguity.
#[derive(Debug, Clone, Default)]
pub struct Inherited {
    pub attributes: HashMap<String, AttributeValue>,
    pub ports: Vec<ComponentPort>,
    pub required: Vec<String>,
    /// attribute key → the two types that disagree on it.
    pub conflicts: BTreeMap<String, (String, String)>,
    /// port name → the two types that disagree on it.
    pub port_conflicts: BTreeMap<String, (String, String)>,
}

fn same_port(a: &ComponentPort, b: &ComponentPort) -> bool {
    a.direction == b.direction && a.interface_type == b.interface_type
}

/// Combine the effective types named by `is:`. `Err` names an unknown type.
pub fn inherit(types: &BTreeMap<String, EffectiveType>, names: &[&str]) -> Result<Inherited, String> {
    let mut inherited = Inherited::default();
    let mut origin: HashMap<String, &str> = HashMap::new();
    let mut port_origin: HashMap<String, &str> = HashMap::new();
    for &name in names {
        let ty = types.get(name).ok_or_else(|| name.to_string())?;
        // Deterministic conflict reporting: attribute maps are unordered.
        let mut keys: Vec<&String> = ty.attributes.keys().collect();
        keys.sort();
        for key in keys {
            let value = &ty.attributes[key];
            match inherited.attributes.get(key) {
                Some(existing) if existing != value => {
                    inherited
                        .conflicts
                        .entry(key.clone())
                        .or_insert_with(|| (origin[key].to_string(), name.to_string()));
                }
                Some(_) => {}
                None => {
                    inherited.attributes.insert(key.clone(), value.clone());
                    origin.insert(key.clone(), name);
                }
            }
        }
        for port in &ty.ports {
            match inherited.ports.iter().find(|p| p.name == port.name) {
                Some(existing) if !same_port(existing, port) => {
                    inherited
                        .port_conflicts
                        .entry(port.name.clone())
                        .or_insert_with(|| (port_origin[&port.name].to_string(), name.to_string()));
                }
                Some(_) => {}
                None => {
                    inherited.ports.push(port.clone());
                    port_origin.insert(port.name.clone(), name);
                }
            }
        }
        for key in &ty.required {
            if !inherited.required.contains(key) {
                inherited.required.push(key.clone());
            }
        }
    }
    Ok(inherited)
}

struct Resolver<'t> {
    types: &'t BTreeMap<String, EffectiveType>,
    uses: Vec<TypeUse>,
}

impl Resolver<'_> {
    fn apply(
        &mut self,
        kind: &'static str,
        label: &str,
        attributes: &mut HashMap<String, AttributeValue>,
        ports: Option<&mut Vec<ComponentPort>>,
    ) -> Result<(), String> {
        let context = format!("{} '{}'", kind, label);
        let type_names: Vec<String> = declared_types(attributes)
            .map_err(|reason| format!("{}: {}", context, reason))?
            .into_iter()
            .map(str::to_string)
            .collect();
        if type_names.is_empty() {
            return Ok(());
        }
        let names: Vec<&str> = type_names.iter().map(String::as_str).collect();
        let inherited = inherit(self.types, &names)
            .map_err(|unknown| format!("{} is typed by unknown type '{}'", context, unknown))?;
        let described = type_names.join("' + '");

        // Two types disagreeing on an attribute is an ambiguity only the
        // element can resolve, by redefining it.
        for (key, (first, second)) in &inherited.conflicts {
            if !attributes.contains_key(key) {
                return Err(format!(
                    "{}: types '{}' and '{}' both define '{}' with different values — redefine '{}' on the element to resolve the ambiguity",
                    context, first, second, key, key
                ));
            }
        }
        for (key, value) in &inherited.attributes {
            match attributes.get(key) {
                Some(own) => conforms(&context, key, value, own)?,
                None => {
                    attributes.insert(key.clone(), value.clone());
                }
            }
        }
        let missing: Vec<&str> = inherited
            .required
            .iter()
            .map(String::as_str)
            .filter(|key| !attributes.contains_key(*key))
            .collect();
        if !missing.is_empty() {
            return Err(format!(
                "{} is a '{}' but does not provide required attribute(s): {}",
                context,
                described,
                missing.join(", ")
            ));
        }
        match ports {
            Some(ports) => {
                for (name, (first, second)) in &inherited.port_conflicts {
                    if !ports.iter().any(|p| &p.name == name) {
                        return Err(format!(
                            "{}: types '{}' and '{}' both declare port '{}' differently — declare '{}' on the component to resolve the ambiguity",
                            context, first, second, name, name
                        ));
                    }
                }
                for port in &inherited.ports {
                    if !ports.iter().any(|p| p.name == port.name) {
                        ports.push(port.clone());
                    }
                }
            }
            None if !inherited.ports.is_empty() => {
                return Err(format!(
                    "{} cannot be a '{}': that type declares ports, which only components carry",
                    context, described
                ));
            }
            None => {}
        }
        for type_name in type_names {
            self.uses.push(TypeUse { type_name, kind, element: label.to_string() });
        }
        Ok(())
    }

    fn activity(&mut self, activity: &mut OperationalActivity) -> Result<(), String> {
        let label = activity.id.clone();
        self.apply("OperationalActivity", &label, &mut activity.attributes, None)?;
        activity.sub_activities.iter_mut().try_for_each(|sub| self.activity(sub))
    }

    fn function(&mut self, function: &mut SystemFunction) -> Result<(), String> {
        let label = function.id.clone();
        self.apply("SystemFunction", &label, &mut function.attributes, None)?;
        function.sub_functions.iter_mut().try_for_each(|sub| self.function(sub))
    }

    fn component(&mut self, component: &mut LogicalComponent) -> Result<(), String> {
        let label = component.id.clone();
        self.apply("LogicalComponent", &label, &mut component.attributes, Some(&mut component.ports))?;
        for function in &mut component.functions {
            let label = function.name.clone();
            self.apply("LogicalFunction", &label, &mut function.attributes, None)?;
        }
        component.sub_components.iter_mut().try_for_each(|sub| self.component(sub))
    }

    fn model(&mut self, ast: &mut Model) -> Result<(), String> {
        for oa in &mut ast.operational_analysis {
            for actor in &mut oa.actors {
                let label = actor.name.clone();
                self.apply("Actor", &label, &mut actor.attributes, None)?;
            }
            for entity in &mut oa.entities {
                let label = entity.id.clone();
                self.apply("OperationalEntity", &label, &mut entity.attributes, None)?;
                entity.activities.iter_mut().try_for_each(|a| self.activity(a))?;
            }
            for capability in &mut oa.capabilities {
                let label = capability.id.clone();
                self.apply("OperationalCapability", &label, &mut capability.attributes, None)?;
            }
            oa.activities.iter_mut().try_for_each(|a| self.activity(a))?;
            for process in &mut oa.processes {
                let label = process.id.clone();
                self.apply("OperationalProcess", &label, &mut process.attributes, None)?;
            }
        }
        for sa in &mut ast.system_analysis {
            for requirement in &mut sa.requirements {
                let label = requirement.id.clone();
                self.apply("Requirement", &label, &mut requirement.attributes, None)?;
            }
            sa.functions.iter_mut().try_for_each(|f| self.function(f))?;
            for component in &mut sa.components {
                let label = component.name.clone();
                self.apply("SystemComponent", &label, &mut component.attributes, None)?;
            }
            for actor in &mut sa.external_actors {
                let label = actor.id.clone();
                self.apply("SystemActor", &label, &mut actor.attributes, None)?;
            }
            for mission in &mut sa.missions {
                let label = mission.id.clone();
                self.apply("Mission", &label, &mut mission.attributes, None)?;
            }
            for capability in &mut sa.capabilities {
                let label = capability.id.clone();
                self.apply("Capability", &label, &mut capability.attributes, None)?;
            }
            for chain in &mut sa.functional_chains {
                let label = chain.id.clone();
                self.apply("FunctionalChain", &label, &mut chain.attributes, None)?;
            }
        }
        for la in &mut ast.logical_architecture {
            la.components.iter_mut().try_for_each(|c| self.component(c))?;
            for interface in &mut la.interfaces {
                let label = interface.name.clone();
                self.apply("LogicalInterface", &label, &mut interface.attributes, None)?;
            }
            for chain in &mut la.functional_chains {
                let label = chain.id.clone();
                self.apply("FunctionalChain", &label, &mut chain.attributes, None)?;
            }
        }
        for pa in &mut ast.physical_architecture {
            for node in &mut pa.nodes {
                let label = node.id.clone();
                self.apply("PhysicalNode", &label, &mut node.attributes, None)?;
            }
            for link in &mut pa.links {
                let label = link.name.clone();
                self.apply("PhysicalLink", &label, &mut link.attributes, None)?;
                // Fields the parser derived from attributes follow inheritance.
                if link.protocol == "Unknown" {
                    if let Some(protocol) = link.attributes.get("protocol").and_then(|v| v.as_string()) {
                        link.protocol = protocol.to_string();
                    }
                }
                if link.bandwidth.is_none() {
                    link.bandwidth = link.attributes.get("bandwidth").map(|v| v.display());
                }
            }
            for path in &mut pa.paths {
                let label = path.name.clone();
                self.apply("PhysicalPath", &label, &mut path.attributes, None)?;
            }
        }
        for epbs in &mut ast.epbs {
            for system in &mut epbs.systems {
                let label = system.name.clone();
                self.apply("EpbsSystem", &label, &mut system.attributes, None)?;
                for subsystem in &mut system.subsystems {
                    let label = subsystem.name.clone();
                    self.apply("EpbsSubsystem", &label, &mut subsystem.attributes, None)?;
                    for item in &mut subsystem.items {
                        let label = item.name.clone();
                        self.apply("EpbsItem", &label, &mut item.attributes, None)?;
                    }
                }
            }
        }
        for safety in &mut ast.safety_analysis {
            for hazard in &mut safety.hazards {
                let label = hazard.name.clone();
                self.apply("Hazard", &label, &mut hazard.attributes, None)?;
            }
        }
        for test_case in &mut ast.test_cases {
            let label = test_case.name.clone();
            self.apply("TestCase", &label, &mut test_case.attributes, None)?;
        }
        Ok(())
    }
}

/// Resolve `extends` and `is:` across the model, folding inherited
/// attributes and ports into each typed element. Returns every use.
pub fn resolve(ast: &mut Model) -> Result<Vec<TypeUse>, String> {
    let types = effective_types(&ast.types)?;
    let mut resolver = Resolver { types: &types, uses: Vec::new() };
    resolver.model(ast)?;
    Ok(resolver.uses)
}

#[cfg(test)]
mod tests {
    use crate::compiler::{CompilationResult, Compiler, CompilerConfig};

    fn compile(source: &str) -> Result<CompilationResult, String> {
        Compiler::new(CompilerConfig::default()).compile_string(source).map_err(|e| e.to_string())
    }

    const TYPES: &str = r#"
model T {}
type "ECU" {
    required: ["ram"]
    voltage: 12 V
    supplier: "Tier-1"
    port in power { interface: "Power12V" }
}
type "Safety ECU" extends "ECU" {
    safety_level: "ASIL-D"
    voltage: 24 V
    port out diagnostics { interface: "UDS" }
}
"#;

    #[test]
    fn instance_inherits_attributes_and_ports_and_may_redefine() {
        let result = compile(&format!(
            r#"{TYPES}
architecture logical {{
    component "Brake ECU" {{ id: "LC-1" is: "Safety ECU" ram: 128 MB supplier: "In-house" port out torque {{ interface: "Torque" }} }}
}}
"#
        ))
        .unwrap();
        let component = &result.ast.logical_architecture[0].components[0];
        assert_eq!(component.attributes["voltage"].display(), "24 V", "nearest definition wins");
        assert_eq!(component.attributes["supplier"].display(), "In-house", "own value wins");
        assert_eq!(component.attributes["safety_level"].display(), "ASIL-D");
        let ports: Vec<&str> = component.ports.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(ports, ["torque", "power", "diagnostics"]);
        // Inherited attributes reach the canonical model.
        assert_eq!(result.semantic_model.components[0].safety_level.as_deref(), Some("ASIL-D"));
        let info = result.semantic_model.types.iter().find(|t| t.name == "Safety ECU").unwrap();
        assert_eq!(info.extends.as_deref(), Some("ECU"));
        assert_eq!(info.instances, ["LC-1"]);
        assert_eq!(info.required, ["ram"]);
    }

    #[test]
    fn missing_required_attribute_is_an_error() {
        let error = compile(&format!(
            "{TYPES}\narchitecture logical {{ component \"Brake ECU\" {{ id: \"LC-1\" is: \"Safety ECU\" }} }}"
        ))
        .unwrap_err();
        assert!(error.contains("LogicalComponent 'LC-1' is a 'Safety ECU' but does not provide required attribute(s): ram"), "{error}");
    }

    #[test]
    fn redefinition_must_keep_the_dimension() {
        let error = compile(&format!(
            "{TYPES}\narchitecture logical {{ component \"X\" {{ id: \"LC-1\" is: \"ECU\" ram: 1 MB voltage: 5 A }} }}"
        ))
        .unwrap_err();
        assert!(error.contains("redefinition of 'voltage' changes its dimension") && error.contains("voltage") && error.contains("electric current"), "{error}");

        let error = compile("model T {}\ntype \"A\" { period: 10 ms }\ntype \"B\" extends \"A\" { period: \"fast\" }").unwrap_err();
        assert!(error.contains("type 'B': redefinition of 'period' must stay a time quantity"), "{error}");
    }

    #[test]
    fn unknown_duplicate_and_cyclic_types_are_errors() {
        let error = compile("model T {}\narchitecture logical { component \"X\" { is: \"Ghost\" } }").unwrap_err();
        assert!(error.contains("is typed by unknown type 'Ghost'"), "{error}");
        let error = compile("model T {}\ntype \"A\" extends \"Ghost\" { }").unwrap_err();
        assert!(error.contains("type 'A' extends unknown type 'Ghost'"), "{error}");
        let error = compile("model T {}\ntype \"A\" { }\ntype \"A\" { }").unwrap_err();
        assert!(error.contains("type 'A' is declared more than once"), "{error}");
        let error = compile("model T {}\ntype \"A\" extends \"B\" { }\ntype \"B\" extends \"A\" { }").unwrap_err();
        assert!(error.contains("cyclic type specialization: A -> B -> A"), "{error}");
    }

    #[test]
    fn typed_functions_feed_constraints_and_the_gate_and_ports_stay_on_components() {
        let result = compile(
            r#"
model T {}
type "Periodic task" { required: ["wcet"] period: 10 ms }
system_analysis "SA" {
    function "Sample" { id: "SF-1" is: "Periodic task" wcet: 2 ms }
    function "Filter" { id: "SF-2" is: "Periodic task" wcet: 3 ms period: 20 ms }
    functional_chain "Loop" { id: "FC-1" involves: ["SF-1", "SF-2"] }
}
constraint "Utilisation" { assert: "SF-1".wcet / "SF-1".period + "SF-2".wcet / "SF-2".period <= 0.5 }
"#,
        )
        .unwrap();
        assert!(result.semantic_model.constraints[0].satisfied, "0.2 + 0.15 <= 0.5");

        let error = compile(
            "model T {}\ntype \"Boxed\" { port in x }\nsystem_analysis \"SA\" { function \"F\" { id: \"SF-1\" is: \"Boxed\" } }",
        )
        .unwrap_err();
        assert!(error.contains("declares ports, which only components carry"), "{error}");
    }

    #[test]
    fn multiple_typing_merges_types_and_reports_ambiguities() {
        const MIXINS: &str = r#"
model T {}
type "ECU" { required: ["ram"] voltage: 12 V port in power { interface: "Power12V" } }
type "Redundant" { required: ["channels"] redundancy: "dual" voltage: 12 V }
type "High voltage" { voltage: 48 V port in power { interface: "Power48V" } }
"#;
        let result = compile(&format!(
            "{MIXINS}\narchitecture logical {{ component \"X\" {{ id: \"LC-1\" is: [\"ECU\", \"Redundant\"] ram: 1 MB channels: 2 }} }}"
        ))
        .unwrap();
        let component = &result.ast.logical_architecture[0].components[0];
        assert_eq!(component.attributes["redundancy"].display(), "dual");
        assert_eq!(component.attributes["voltage"].display(), "12 V", "agreeing types are not a conflict");
        assert_eq!(component.ports.len(), 1);
        for name in ["ECU", "Redundant"] {
            let info = result.semantic_model.types.iter().find(|t| t.name == name).unwrap();
            assert_eq!(info.instances, ["LC-1"]);
        }

        let error = compile(&format!(
            "{MIXINS}\narchitecture logical {{ component \"X\" {{ id: \"LC-1\" is: [\"ECU\", \"High voltage\"] ram: 1 MB }} }}"
        ))
        .unwrap_err();
        assert!(error.contains("types 'ECU' and 'High voltage' both define 'voltage' with different values"), "{error}");

        let error = compile(&format!(
            "{MIXINS}\narchitecture logical {{ component \"X\" {{ id: \"LC-1\" is: [\"ECU\", \"High voltage\"] ram: 1 MB voltage: 48 V }} }}"
        ))
        .unwrap_err();
        assert!(error.contains("both declare port 'power' differently"), "{error}");

        let resolved = compile(&format!(
            "{MIXINS}\narchitecture logical {{ component \"X\" {{ id: \"LC-1\" is: [\"ECU\", \"High voltage\"] ram: 1 MB voltage: 48 V port in power {{ interface: \"Power48V\" }} }} }}"
        ))
        .unwrap();
        assert_eq!(resolved.ast.logical_architecture[0].components[0].attributes["voltage"].display(), "48 V");

        let error = compile(&format!(
            "{MIXINS}\narchitecture logical {{ component \"X\" {{ id: \"LC-1\" is: [\"ECU\", \"Redundant\"] ram: 1 MB }} }}"
        ))
        .unwrap_err();
        assert!(error.contains("is a 'ECU' + 'Redundant' but does not provide required attribute(s): channels"), "{error}");

        let error = compile(&format!("{MIXINS}\narchitecture logical {{ component \"X\" {{ is: [\"ECU\", \"ECU\"] }} }}")).unwrap_err();
        assert!(error.contains("names type 'ECU' twice"), "{error}");
    }

    #[test]
    fn inherited_link_protocol_satisfies_the_gate() {
        let result = compile(
            r#"
model T {}
type "CAN FD bus" { protocol: "CAN FD" bandwidth: 2 Mbps }
architecture physical {
    node "A" { id: "PN-1" }
    node "B" { id: "PN-2" }
    link "Chassis" { from: "PN-1" to: "PN-2" is: "CAN FD bus" }
}
"#,
        )
        .unwrap();
        let link = &result.ast.physical_architecture[0].links[0];
        assert_eq!(link.protocol, "CAN FD");
        assert_eq!(link.bandwidth.as_deref(), Some("2 Mbps"));
    }
}
