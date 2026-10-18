//! The Arcadia levels as packages: `#OA package 'Operational Analysis'`,
//! `#SA`, `#LA`, `#PA`, `#EPBS`. Every element is a usage where it sits
//! (definitions are kept for types): a function is an `action` with its
//! ports as directed item parameters and its sub-functions nested, a
//! `perform action` inside the part that holds it, a `perform` of the level's
//! action in the part it is allocated to; a component is a `part` with its
//! ports and the components it holds; exchanges are flows and connections
//! between feature chains (`'SF-1'.raw`, `'LC-1'.'LC-2'.radar`).

use super::{data, id_of, text, Exporter};
use crate::compiler::ast::*;
use std::collections::HashMap;

/// What the three function trees of ArcLang have in common.
pub(super) trait Function {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn ports(&self) -> &[FunctionPort];
    fn subs(&self) -> Vec<&Self>;
    fn attributes(&self) -> &HashMap<String, AttributeValue>;
    /// The written id, as the element graph knows it.
    fn written(&self) -> &str {
        let id = id_of(self.attributes(), self.id());
        if id.is_empty() { self.name() } else { id }
    }
}

impl Function for OperationalActivity {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { &self.name }
    fn ports(&self) -> &[FunctionPort] { &self.ports }
    fn subs(&self) -> Vec<&Self> { self.sub_activities.iter().collect() }
    fn attributes(&self) -> &HashMap<String, AttributeValue> { &self.attributes }
}

impl Function for SystemFunction {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { &self.name }
    fn ports(&self) -> &[FunctionPort] { &self.ports }
    fn subs(&self) -> Vec<&Self> { self.sub_functions.iter().collect() }
    fn attributes(&self) -> &HashMap<String, AttributeValue> { &self.attributes }
}

impl Function for LogicalFunction {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { &self.name }
    fn ports(&self) -> &[FunctionPort] { &self.ports }
    fn subs(&self) -> Vec<&Self> { self.sub_functions.iter().collect() }
    fn attributes(&self) -> &HashMap<String, AttributeValue> { &self.attributes }
}

pub(super) fn visit_function_tree<F: Function>(f: &F, visit: &mut impl FnMut(&F)) {
    visit(f);
    for sub in f.subs() {
        visit_function_tree(sub, visit);
    }
}

pub(super) fn visit_system_functions(functions: &[SystemFunction], visit: &mut impl FnMut(&SystemFunction)) {
    for f in functions {
        visit_function_tree(f, visit);
    }
}

/// Every activity of an operational analysis, those its entities hold included.
pub(super) fn visit_activities(oa: &OperationalAnalysis, visit: &mut impl FnMut(&OperationalActivity)) {
    for activity in oa.activities.iter().chain(oa.entities.iter().flat_map(|e| e.activities.iter())) {
        visit_function_tree(activity, visit);
    }
}

/// Every logical function: the level's and those its components hold.
pub(super) fn visit_logical_functions(la: &LogicalArchitecture, visit: &mut impl FnMut(&LogicalFunction)) {
    fn components(list: &[LogicalComponent], visit: &mut impl FnMut(&LogicalFunction)) {
        for c in list {
            for f in &c.functions {
                visit_function_tree(f, visit);
            }
            components(&c.sub_components, visit);
        }
    }
    for f in &la.functions {
        visit_function_tree(f, visit);
    }
    components(&la.components, visit);
}

/// A level package: its metadata, name and qualified name.
pub(super) struct Level {
    pub metadata: &'static str,
    pub id: String,
    pub name: String,
}

impl Level {
    pub fn new(metadata: &'static str, id: &str, name: &str, default: &str) -> Level {
        Level { metadata, id: id.to_string(), name: if name.is_empty() { default.to_string() } else { name.to_string() } }
    }
    pub fn qualified(&self) -> String {
        text::name(&self.name)
    }
    pub fn head(&self) -> String {
        format!("#{} package {}", self.metadata, text::declared(&self.id, &self.name))
    }
}

pub(super) fn oa_level(oa: &OperationalAnalysis) -> Level {
    Level::new("OA", &oa.id, &oa.name, "Operational Analysis")
}
pub(super) fn sa_level(sa: &SystemAnalysis) -> Level {
    Level::new("SA", &sa.id, &sa.name, "System Analysis")
}
pub(super) fn la_level(la: &LogicalArchitecture) -> Level {
    Level::new("LA", &la.id, &la.name, "Logical Architecture")
}
pub(super) fn pa_level(pa: &PhysicalArchitecture) -> Level {
    Level::new("PA", &pa.id, &pa.name, "Physical Architecture")
}
pub(super) fn epbs_level(epbs: &Epbs) -> Level {
    Level::new("EPBS", &epbs.id, &epbs.name, "EPBS")
}

// --- registration ---------------------------------------------------------

/// Register a function tree under `base`: each function by id and name,
/// its ports as `Owner.port`, its sub-functions under it.
pub(super) fn register_function<F: Function>(e: &mut Exporter<'_>, package: &str, base: &[String], f: &F) {
    let id = f.written().to_string();
    let mut segments = base.to_vec();
    segments.push(text::name(&id));
    e.register_at(package, segments.clone(), &id, &[f.id(), f.name()]);
    for port in f.ports() {
        let mut path = segments.clone();
        path.push(text::name(&port.name));
        let keys: Vec<String> = [id.as_str(), f.id(), f.name()].iter().filter(|o| !o.is_empty()).map(|o| format!("{o}.{}", port.name)).collect();
        let keys: Vec<&str> = keys.iter().map(String::as_str).collect();
        e.register_at(package, path, &port.id, &keys);
    }
    for sub in f.subs() {
        register_function(e, package, &segments, sub);
    }
}

/// Let `Owner.function` name each function a part holds.
pub(super) fn alias_held<F: Function>(e: &mut Exporter<'_>, owners: &[&str], functions: &[F]) {
    for f in functions {
        for owner in owners.iter().filter(|o| !o.is_empty()) {
            for name in [f.name(), f.written()] {
                e.alias(&format!("{owner}.{name}"), f.written());
            }
        }
    }
}

/// Register a part under `base` and its ports; the segments that lead to it.
pub(super) fn register_part(e: &mut Exporter<'_>, package: &str, base: &[String], id: &str, name: &str, ports: &[(&str, &str)]) -> Vec<String> {
    let mut segments = base.to_vec();
    segments.push(text::name(id));
    e.register_at(package, segments.clone(), id, &[name]);
    for (port_id, port) in ports {
        let mut path = segments.clone();
        path.push(text::name(port));
        let keys: Vec<String> = [id, name].iter().filter(|o| !o.is_empty()).map(|o| format!("{o}.{port}")).collect();
        let keys: Vec<&str> = keys.iter().map(String::as_str).collect();
        e.register_at(package, path, port_id, &keys);
    }
    segments
}

pub(super) fn ports_of(ports: &[ComponentPort]) -> Vec<(&str, &str)> {
    ports.iter().map(|p| (p.id.as_str(), p.name.as_str())).collect()
}

pub(super) fn register(e: &mut Exporter<'_>) {
    let model = e.model;
    for oa in &model.operational_analysis {
        let package = oa_level(oa).qualified();
        for activity in &oa.activities {
            register_function(e, &package, &[], activity);
        }
        for entity in &oa.entities {
            let id = id_of(&entity.attributes, &entity.id);
            let at = register_part(e, &package, &[], id, &entity.name, &[]);
            for activity in &entity.activities {
                register_function(e, &package, &at, activity);
            }
            alias_held(e, &[id, &entity.name], &entity.activities);
            let performed: Vec<&OperationalActivity> = oa.activities.iter().filter(|a| a.performed_by == id || a.performed_by == entity.name).collect();
            for activity in performed {
                for owner in [id, entity.name.as_str()] {
                    e.alias(&format!("{owner}.{}", activity.name), activity.written());
                }
            }
        }
        for actor in &oa.actors {
            let id = id_of(&actor.attributes, actor.id.as_deref().unwrap_or(&actor.name));
            register_part(e, &package, &[], id, &actor.name, &[]);
        }
        register_capabilities(e, &package, &oa.capabilities);
        for process in &oa.processes {
            e.register(&package, &process.id, &[&process.name]);
        }
    }
    for sa in &model.system_analysis {
        let package = sa_level(sa).qualified();
        for f in &sa.functions {
            register_function(e, &package, &[], f);
        }
        for component in &sa.components {
            let id = id_of(&component.attributes, &component.name);
            register_part(e, &package, &[], id, &component.name, &ports_of(&component.ports));
        }
        for actor in &sa.external_actors {
            let id = id_of(&actor.attributes, &actor.id);
            register_part(e, &package, &[], id, &actor.name, &ports_of(&actor.ports));
        }
        for mission in &sa.missions {
            e.register(&package, id_of(&mission.attributes, &mission.id), &[&mission.name]);
        }
        for capability in &sa.capabilities {
            e.register(&package, id_of(&capability.attributes, &capability.id), &[&capability.name]);
        }
        for chain in &sa.functional_chains {
            e.register(&package, &chain.id, &[&chain.name]);
        }
        for exchange in &sa.functional_exchanges {
            e.register(&package, &exchange.id, &[]);
        }
        for exchange in &sa.component_exchanges {
            e.register(&package, &exchange.id, &[]);
        }
        for requirement in &sa.requirements {
            let id = id_of(&requirement.attributes, &requirement.id);
            e.register(&package, id, &[&requirement.id]);
        }
    }
    super::architecture::register(e);
}

fn register_capabilities(e: &mut Exporter<'_>, package: &str, capabilities: &[OperationalCapability]) {
    for capability in capabilities {
        e.register(package, id_of(&capability.attributes, &capability.id), &[&capability.name]);
        register_capabilities(e, package, &capability.children);
    }
}

// --- writing --------------------------------------------------------------

pub(super) fn write(e: &mut Exporter<'_>) {
    let model = e.model;
    for oa in &model.operational_analysis {
        operational(e, oa);
    }
    for sa in &model.system_analysis {
        system(e, sa);
    }
    super::architecture::write(e);
}

/// A function as an action usage (`keyword` is `action`, or `perform
/// action` inside the part that holds it): its ports as directed items,
/// its sub-functions nested.
pub(super) fn function<F: Function>(e: &mut Exporter<'_>, kind: &str, f: &F, keyword: &str) {
    e.block(&format!("{keyword} {}", text::declared(f.written(), f.name())), |e| {
        e.header(kind, f.attributes(), &["inputs", "outputs"]);
        for port in f.ports() {
            let ty = data::type_reference(e, &port.data_type);
            let short = if port.id.is_empty() { String::new() } else { format!("{} ", text::short(&port.id)) };
            e.line(&format!("{} item {short}{} : {ty};", data::direction(&port.direction), text::name(&port.name)));
        }
        for sub in f.subs() {
            function(e, kind, sub, "action");
        }
    });
}

/// `perform` of each function a part is allocated, by reference.
pub(super) fn performs(e: &mut Exporter<'_>, functions: &[String]) {
    for f in functions {
        match e.feature(f) {
            Some(action) => e.line(&format!("perform {action};")),
            None => e.line(&format!("// performs {f}: not exported")),
        }
    }
}

/// Ports of a part: `in port 'radar' : 'Frame port';`, typed by the port
/// definition of what it carries.
pub(super) fn component_ports(e: &mut Exporter<'_>, ports: &[ComponentPort]) {
    for port in ports {
        let short = if port.id.is_empty() { String::new() } else { format!("{} ", text::short(&port.id)) };
        let def = port_def(&port.interface_type);
        e.line(&format!("{} port {short}{} : {def};", data::direction(&port.direction), text::name(&port.name)));
    }
}

/// The port definition of a port carrying `interface`: one per interface, in the level.
pub(super) fn port_def(interface: &str) -> String {
    if interface.is_empty() { "Port".to_string() } else { text::name(&format!("{interface} port")) }
}

/// The port definitions a level's ports use: the types of its ports.
pub(super) fn port_defs(e: &mut Exporter<'_>, interfaces: impl IntoIterator<Item = String>) {
    let mut seen: Vec<String> = interfaces.into_iter().collect();
    seen.sort();
    seen.dedup();
    for interface in seen {
        let head = format!("port def {}", port_def(&interface));
        if interface.is_empty() {
            e.line(&format!("{head};"));
        } else {
            let ty = data::type_reference(e, &interface);
            e.block(&head, |e| e.line(&format!("inout item carried : {ty};")));
        }
    }
}

fn connector_head(id: &str, label: &str) -> String {
    match (id.is_empty(), label.is_empty()) {
        (true, true) => String::new(),
        (true, false) => format!("{} ", text::name(label)),
        (false, _) => format!("{} ", text::declared(id, if label.is_empty() { id } else { label })),
    }
}

/// A flow between two features (`'SF-1'.raw`); between two whole functions,
/// which a flow cannot end on, a connection.
pub(super) fn flow(e: &mut Exporter<'_>, id: &str, label: &str, item: &str, from: &str, to: &str) {
    let (Some(source), Some(target)) = (e.feature(from), e.feature(to)) else {
        e.line(&format!("// flow {} from {from} to {to}: an end is not exported", text::name(label)));
        return;
    };
    let declared = connector_head(id, label);
    if !source.contains('.') || !target.contains('.') {
        e.line(&format!("connection {declared}connect {source} to {target};"));
        if !item.is_empty() && !id.is_empty() {
            super::relations::dependency(e, "exchange_item", id, item);
        }
        return;
    }
    let of = if item.is_empty() { String::new() } else { format!("of {} ", data::type_reference(e, item)) };
    e.line(&format!("flow {declared}{of}from {source} to {target};"));
}

/// A connection between two parts or their ports.
pub(super) fn connection(e: &mut Exporter<'_>, metadata: &str, id: &str, label: &str, from: &str, to: &str, attributes: &HashMap<String, AttributeValue>, kind: &str) {
    let (Some(source), Some(target)) = (e.feature(from), e.feature(to)) else {
        e.line(&format!("// connection {} from {from} to {to}: an end is not exported", text::name(label)));
        return;
    };
    let head = format!("{metadata}connection {}connect {source} to {target}", connector_head(id, label));
    e.block(&head, |e| e.header(kind, attributes, &["from_port", "to_port", "exchange_item", "via"]));
}

/// A functional chain as an action of steps in sequence, each step a
/// subset of the function it involves; what it involves that is not a
/// function (an exchange) is a dependency.
pub(super) fn chain(e: &mut Exporter<'_>, kind: &str, chain: &FunctionalChain, functions: &[String]) {
    let mut others: Vec<String> = Vec::new();
    e.block(&format!("#Chain action {}", text::declared(&chain.id, &chain.name)), |e| {
        e.header(kind, &chain.attributes, &[]);
        let mut first = true;
        for (n, step) in chain.involves.iter().enumerate() {
            match e.feature(step).filter(|_| functions.iter().any(|f| f == step)) {
                Some(function) => {
                    let then = if first { "" } else { "then " };
                    e.line(&format!("{then}action step{} :> {function};", n + 1));
                    first = false;
                }
                None => others.push(step.clone()),
            }
        }
    });
    for other in others {
        super::relations::dependency(e, "involves", &chain.id, &other);
    }
}

/// The ids and names of every function of a tree, what a chain's steps name.
pub(super) fn function_keys<F: Function>(list: &[&F]) -> Vec<String> {
    let mut keys = Vec::new();
    for f in list {
        visit_function_tree(*f, &mut |f: &F| keys.extend([f.written().to_string(), f.id().to_string(), f.name().to_string()]));
    }
    keys
}

/// Missions and capabilities as use cases with their kind.
pub(super) fn use_case(e: &mut Exporter<'_>, metadata: &str, kind: &str, id: &str, name: &str, attributes: &HashMap<String, AttributeValue>) {
    e.block(&format!("{metadata} use case {}", text::declared(id, name)), |e| e.header(kind, attributes, &[]));
}

fn operational(e: &mut Exporter<'_>, oa: &OperationalAnalysis) {
    let level = oa_level(oa);
    e.package(&level.head(), &level.qualified(), |e| {
        e.header("OperationalAnalysis", &oa.attributes, &[]);
        for activity in &oa.activities {
            function(e, "OperationalActivity", activity, "action");
        }
        for entity in &oa.entities {
            let id = id_of(&entity.attributes, &entity.id).to_string();
            let allocated: Vec<String> = oa.activities.iter().filter(|a| a.performed_by == id || a.performed_by == entity.name).map(|a| a.written().to_string()).collect();
            e.block(&format!("#Entity part {}", text::declared(&id, &entity.name)), |e| {
                e.header("OperationalEntity", &entity.attributes, &[]);
                for activity in &entity.activities {
                    function(e, "OperationalActivity", activity, "perform action");
                }
                performs(e, &allocated);
            });
        }
        for actor in &oa.actors {
            let id = id_of(&actor.attributes, actor.id.as_deref().unwrap_or(&actor.name)).to_string();
            let allocated: Vec<String> = oa.activities.iter().filter(|a| a.performed_by == id || a.performed_by == actor.name).map(|a| a.written().to_string()).collect();
            e.block(&format!("#Actor part {}", text::declared(&id, &actor.name)), |e| {
                e.header("Actor", &actor.attributes, &[]);
                performs(e, &allocated);
            });
        }
        capabilities(e, &oa.capabilities);
        for exchange in &oa.exchanges {
            let id = id_of(&exchange.attributes, "").to_string();
            flow(e, &id, exchange.label.as_deref().unwrap_or(""), &exchange.data_type, &exchange.from, &exchange.to);
        }
        for mean in &oa.communication_means {
            let id = id_of(&mean.attributes, "").to_string();
            connection(e, "", &id, mean.label.as_deref().unwrap_or(""), &mean.from, &mean.to, &mean.attributes, "CommunicationMean");
        }
        let mut activities: Vec<&OperationalActivity> = oa.activities.iter().collect();
        activities.extend(oa.entities.iter().flat_map(|en| en.activities.iter()));
        let keys = function_keys(&activities);
        for process in &oa.processes {
            chain(e, "OperationalProcess", process, &keys);
        }
    });
}

fn capabilities(e: &mut Exporter<'_>, list: &[OperationalCapability]) {
    for capability in list {
        let id = id_of(&capability.attributes, &capability.id).to_string();
        use_case(e, "#Capability", "OperationalCapability", &id, &capability.name, &capability.attributes);
        for involved in &capability.involves {
            super::relations::dependency(e, "involves", &id, involved);
        }
        if let Some(parent) = &capability.parent {
            super::relations::dependency(e, "parent", &id, parent);
        }
        capabilities(e, &capability.children);
    }
}

fn system(e: &mut Exporter<'_>, sa: &SystemAnalysis) {
    let level = sa_level(sa);
    e.package(&level.head(), &level.qualified(), |e| {
        e.header("SystemAnalysis", &sa.attributes, &[]);
        let interfaces = sa.components.iter().flat_map(|c| c.ports.iter()).chain(sa.external_actors.iter().flat_map(|a| a.ports.iter())).map(|p| p.interface_type.clone());
        port_defs(e, interfaces.collect::<Vec<_>>());
        for f in &sa.functions {
            function(e, "SystemFunction", f, "action");
        }
        for component in &sa.components {
            let id = id_of(&component.attributes, &component.name).to_string();
            let allocated = super::relations::list(&component.attributes, "allocated_functions");
            e.block(&format!("part {}", text::declared(&id, &component.name)), |e| {
                e.header("SystemComponent", &component.attributes, &[]);
                component_ports(e, &component.ports);
                performs(e, &allocated);
            });
        }
        for actor in &sa.external_actors {
            let id = id_of(&actor.attributes, &actor.id).to_string();
            let allocated = super::relations::list(&actor.attributes, "allocated_functions");
            e.block(&format!("#Actor part {}", text::declared(&id, &actor.name)), |e| {
                e.header("SystemActor", &actor.attributes, &[]);
                component_ports(e, &actor.ports);
                performs(e, &allocated);
            });
        }
        for exchange in &sa.functional_exchanges {
            flow(e, &exchange.id, exchange.label.as_deref().unwrap_or(""), &exchange.data_type, &exchange.from_port, &exchange.to_port);
        }
        for exchange in &sa.component_exchanges {
            connection(e, "", &exchange.id, exchange.label.as_deref().unwrap_or(""), &exchange.from_port, &exchange.to_port, &exchange.attributes, "ComponentExchange");
        }
        for mission in &sa.missions {
            use_case(e, "#Mission", "Mission", id_of(&mission.attributes, &mission.id), &mission.name, &mission.attributes);
        }
        for capability in &sa.capabilities {
            let id = id_of(&capability.attributes, &capability.id).to_string();
            use_case(e, "#Capability", "Capability", &id, &capability.name, &capability.attributes);
            for involved in &capability.involves {
                super::relations::dependency(e, "involves", &id, involved);
            }
            for (key, target) in [("realizes", &capability.realizes), ("mission", &capability.mission), ("parent", &capability.parent)] {
                if let Some(target) = target {
                    super::relations::dependency(e, key, &id, target);
                }
            }
        }
        let keys = function_keys(&sa.functions.iter().collect::<Vec<_>>());
        for chain_ in &sa.functional_chains {
            chain(e, "FunctionalChain", chain_, &keys);
        }
        for requirement in &sa.requirements {
            super::relations::requirement(e, requirement);
        }
    });
}
