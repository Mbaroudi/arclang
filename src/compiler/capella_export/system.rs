//! The system analysis: functions with their ports and exchanges, the
//! system and its actors with their ports, allocations and exchanges,
//! functional chains, missions and capabilities.

use super::context::{derived, id_of, list, port_id, Context};
use super::header::apply;
use super::relations;
use super::xml::El;
use crate::compiler::ast::{ComponentPort, FunctionPort, PortDirection, SystemAnalysis, SystemFunction};

/// The id of the system analysis.
fn level_id(sa: &SystemAnalysis) -> String {
    if !sa.id.is_empty() {
        return sa.id.clone();
    }
    derived(&format!("system-analysis:{}", sa.name))
}

fn register_function(context: &mut Context, function: &SystemFunction) {
    context.functions.insert(function.id.clone());
    context.register_function_name(&function.name, &function.id);
    context.register(&function.id, &function.id);
    context.register(&function.name, &function.id);
    for port in &function.ports {
        let id = port_id(&function.id, &port.name, &port.id);
        context.register(&format!("{}.{}", function.id, port.name), &id);
        context.register(&format!("{}.{}", function.name, port.name), &id);
    }
    for sub in &function.sub_functions {
        register_function(context, sub);
    }
}

fn register_component(context: &mut Context, id: &str, name: &str, ports: &[ComponentPort]) {
    context.register(id, id);
    context.register(name, id);
    for port in ports {
        let pid = port_id(id, &port.name, &port.id);
        context.register(&format!("{id}.{}", port.name), &pid);
        context.register(&format!("{name}.{}", port.name), &pid);
    }
}

/// Record what the system analysis declares.
pub(super) fn register(context: &mut Context, sa: &SystemAnalysis) {
    for function in &sa.functions {
        register_function(context, function);
    }
    for component in &sa.components {
        let id = id_of(&component.attributes, &format!("system-component:{}", component.name));
        register_component(context, &id, &component.name, &component.ports);
    }
    for actor in &sa.external_actors {
        register_component(context, &actor.id, &actor.name, &actor.ports);
    }
    for exchange in &sa.functional_exchanges {
        context.register(&exchange.id, &exchange.id);
    }
    for exchange in &sa.component_exchanges {
        context.register(&exchange.id, &exchange.id);
    }
    for chain in &sa.functional_chains {
        context.chains.insert(chain.id.clone());
        context.register(&chain.id, &chain.id);
        context.register(&chain.name, &chain.id);
    }
    for mission in &sa.missions {
        context.register(&mission.id, &mission.id);
        context.register(&mission.name, &mission.id);
    }
    for capability in &sa.capabilities {
        context.register(&capability.id, &capability.id);
        context.register(&capability.name, &capability.id);
    }
}

fn function_port(owner: &str, port: &FunctionPort) -> El {
    let (tag, kind) = match port.direction {
        PortDirection::Out => ("outputs", "fa:FunctionOutputPort"),
        _ => ("inputs", "fa:FunctionInputPort"),
    };
    El::new(tag, kind, &port_id(owner, &port.name, &port.id)).attr("name", &port.name)
}

fn function(function: &SystemFunction) -> El {
    let mut el = apply(El::new("ownedFunctions", "ctx:SystemFunction", &function.id).attr("name", &function.name), &function.attributes, &["kind"]);
    for port in &function.ports {
        el.push(function_port(&function.id, port));
    }
    for sub in &function.sub_functions {
        el.push(self::function(sub));
    }
    el
}

pub(super) fn component_port(owner: &str, port: &ComponentPort) -> El {
    let orientation = match port.direction {
        PortDirection::In => "IN",
        PortDirection::Out => "OUT",
        PortDirection::InOut => "INOUT",
    };
    El::new("ownedFeatures", "fa:ComponentPort", &port_id(owner, &port.name, &port.id)).attr("name", &port.name).attr("orientation", orientation).attr("kind", "FLOW")
}

/// A system component or actor, with its ports and allocated functions, and its part.
fn component(context: &Context, id: &str, name: &str, actor: bool, ports: &[ComponentPort], attributes: &std::collections::HashMap<String, crate::compiler::ast::AttributeValue>) -> (El, El) {
    let mut el = El::new("ownedSystemComponents", "ctx:SystemComponent", id).attr("name", name);
    if actor {
        el.set("actor", "true");
    }
    let mut el = apply(el, attributes, &["allocated_functions"]);
    for port in ports {
        el.push(component_port(id, port));
    }
    for function in list(attributes, "allocated_functions") {
        if let Some(target) = context.resolve(&function).filter(|id| context.functions.contains(*id)) {
            el.push(
                El::new("ownedFunctionalAllocation", "fa:ComponentFunctionalAllocation", &derived(&format!("{id}->{target}:allocation")))
                    .attr("targetElement", &format!("#{target}"))
                    .attr("sourceElement", &format!("#{id}")),
            );
        }
    }
    let part = El::new("ownedParts", "cs:Part", &derived(&format!("{id}:part"))).attr("name", name).attr("abstractType", &format!("#{id}"));
    (el, part)
}

/// An exchange between two ends that resolve, else `None` and a note.
fn ends(context: &mut Context, what: &str, label: &str, from: &str, to: &str) -> Option<(String, String)> {
    match (context.resolve(from), context.resolve(to)) {
        (Some(from), Some(to)) => Some((from.to_string(), to.to_string())),
        _ => {
            context.note(format!("{what} '{label}' left out: an end ('{from}' or '{to}') is not exported"));
            None
        }
    }
}

/// The system analysis element.
pub(super) fn write(context: &mut Context, sa: &SystemAnalysis, above_root: Option<&str>) -> El {
    let id = level_id(sa);
    let mut level = El::new("ownedArchitectures", "ctx:SystemAnalysis", &id).attr("name", &sa.name);

    let root_id = derived(&format!("{id}:root-function"));
    let mut root = El::new("ownedSystemFunctions", "ctx:SystemFunction", &root_id).attr("name", "Root System Function");
    if let Some(above) = above_root {
        root.push(super::realization("ownedFunctionRealizations", "fa:FunctionRealization", &root_id, above));
    }
    for f in &sa.functions {
        root.push(function(f));
    }
    let mut exchange_ends: std::collections::HashMap<String, (String, String)> = std::collections::HashMap::new();
    for exchange in &sa.functional_exchanges {
        let label = exchange.label.clone().unwrap_or_else(|| exchange.id.clone());
        let owner = |port: &str| port.split_once('.').map(|(f, _)| f.to_string()).unwrap_or_else(|| port.to_string());
        if let (Some(from), Some(to)) = (context.resolve(&owner(&exchange.from_port)), context.resolve(&owner(&exchange.to_port))) {
            exchange_ends.insert(exchange.id.clone(), (from.to_string(), to.to_string()));
        }
        if let Some((source, target)) = ends(context, "functional exchange", &label, &exchange.from_port, &exchange.to_port) {
            let el = El::new("ownedFunctionalExchanges", "fa:FunctionalExchange", &exchange.id).attr("name", &label).attr("target", &format!("#{target}")).attr("source", &format!("#{source}"));
            root.push(relations::exchange(context, el, &exchange.attributes));
        }
    }
    for chain in &sa.functional_chains {
        let el = apply(El::new("ownedFunctionalChains", "fa:FunctionalChain", &chain.id).attr("name", &chain.name), &chain.attributes, &["involves"]);
        root.push(relations::chain_steps(context, el, chain, &exchange_ends));
    }
    super::functions::place_exchanges(&mut root);
    level.push(El::new("ownedFunctionPkg", "ctx:SystemFunctionPkg", &derived(&format!("{id}:functions"))).attr("name", "System Functions").with(root));

    let mut components = El::new("ownedSystemComponentPkg", "ctx:SystemComponentPkg", &derived(&format!("{id}:components"))).attr("name", "Structure");
    let mut parts = Vec::new();
    for c in &sa.components {
        let cid = id_of(&c.attributes, &format!("system-component:{}", c.name));
        let (el, part) = component(context, &cid, &c.name, false, &c.ports, &c.attributes);
        components.push(el);
        parts.push(part);
    }
    for actor in &sa.external_actors {
        let (el, part) = component(context, &actor.id, &actor.name, true, &actor.ports, &actor.attributes);
        components.push(el);
        parts.push(part);
    }
    for exchange in &sa.component_exchanges {
        let label = exchange.label.clone().unwrap_or_else(|| exchange.id.clone());
        if let Some((source, target)) = ends(context, "component exchange", &label, &exchange.from_port, &exchange.to_port) {
            let el = El::new("ownedComponentExchanges", "fa:ComponentExchange", &exchange.id).attr("name", &label).attr("target", &format!("#{target}")).attr("source", &format!("#{source}")).attr("kind", "FLOW");
            components.push(relations::exchange(context, el, &exchange.attributes));
        }
    }
    for part in parts {
        components.push(part);
    }
    level.push(components);

    let mut missions = El::new("ownedMissionPkg", "ctx:MissionPkg", &derived(&format!("{id}:missions"))).attr("name", "Missions");
    for mission in &sa.missions {
        let mut el = apply(El::new("ownedMissions", "ctx:Mission", &mission.id).attr("name", &mission.name), &mission.attributes, &["involves"]);
        for involved in list(&mission.attributes, "involves") {
            if let Some(target) = context.resolve(&involved) {
                el.push(El::new("ownedMissionInvolvements", "ctx:MissionInvolvement", &derived(&format!("{}->{target}:involvement", mission.id))).attr("involved", &format!("#{target}")));
            }
        }
        for capability in sa.capabilities.iter().filter(|c| c.mission.as_deref() == Some(mission.id.as_str()) || c.mission.as_deref() == Some(mission.name.as_str())) {
            el.push(El::new("ownedCapabilityExploitations", "ctx:CapabilityExploitation", &derived(&format!("{}->{}:exploitation", mission.id, capability.id))).attr("capability", &format!("#{}", capability.id)));
        }
        missions.push(el);
    }
    level.push(missions);

    let mut capabilities = El::new("ownedAbstractCapabilityPkg", "ctx:CapabilityPkg", &derived(&format!("{id}:capabilities"))).attr("name", "Capabilities");
    for capability in &sa.capabilities {
        let mut el = apply(
            El::new("ownedCapabilities", "ctx:Capability", &capability.id).attr("name", &capability.name),
            &capability.attributes,
            &["involves", "mission", "realizes", "parent", "extends", "includes", "specializes", "also_exploited_by"],
        );
        for involved in &capability.involves {
            let Some(target) = context.resolve(involved).map(str::to_string) else { continue };
            let is_function = sa.functions.iter().any(|f| contains_function(f, &target));
            let is_chain = sa.functional_chains.iter().any(|c| c.id == target);
            let (tag, kind) = if is_function {
                ("ownedAbstractFunctionAbstractCapabilityInvolvements", "interaction:AbstractFunctionAbstractCapabilityInvolvement")
            } else if is_chain {
                ("ownedFunctionalChainAbstractCapabilityInvolvements", "interaction:FunctionalChainAbstractCapabilityInvolvement")
            } else {
                ("ownedCapabilityInvolvements", "ctx:CapabilityInvolvement")
            };
            el.push(El::new(tag, kind, &derived(&format!("{}->{target}:involvement", capability.id))).attr("involved", &format!("#{target}")));
        }
        capabilities.push(relations::capability(context, el, &capability.id, &capability.attributes, capability.realizes.as_deref()));
    }
    level.push(capabilities);
    level
}

fn contains_function(function: &SystemFunction, id: &str) -> bool {
    function.id == id || function.sub_functions.iter().any(|f| contains_function(f, id))
}

/// An empty system analysis, for a model that declares none.
pub(super) fn empty(project_id: &str) -> El {
    // As Capella's own empty project: no package, no root function.
    El::new("ownedArchitectures", "ctx:SystemAnalysis", &derived(&format!("{project_id}:sa"))).attr("name", "System Analysis")
}
