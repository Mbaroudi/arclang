//! The logical and physical architectures and the EPBS. A component is a
//! `part def` of its level; a component held by another is a `part` usage
//! in its holder's definition, so `'p_LC-0'.'p_LC-1'.'radar'` is a port a
//! connection can end on. A physical node holds its behaviour and hardware
//! components the same way, with its oriented ports and `#PhysicalPort`s.

use super::levels::{self, component_ports, connection, flow, functions, performs, port_defs, register_functions, register_part, usage, Function};
use super::{id_of, text, Exporter};
use crate::compiler::ast::*;

pub(super) fn register(e: &mut Exporter<'_>) {
    let model = e.model;
    for la in &model.logical_architecture {
        let package = levels::la_level(la).qualified();
        let mut all: Vec<&LogicalFunction> = la.functions.iter().collect();
        collect_component_functions(&la.components, &mut all);
        register_functions(e, &package, &all);
        register_components(e, &package, &la.components, None);
        for realization in &la.capability_realizations {
            e.register(&package, id_of(&realization.attributes, &realization.id), &[&realization.name]);
        }
        for chain in &la.functional_chains {
            e.register(&package, &chain.id, &[&chain.name]);
        }
    }
    for pa in &model.physical_architecture {
        let package = levels::pa_level(pa).qualified();
        let mut all: Vec<&LogicalFunction> = pa.functions.iter().collect();
        for node in &pa.nodes {
            all.extend(node.functions.iter());
            all.extend(node.behavior_components.iter().flat_map(|b| b.functions.iter()));
        }
        register_functions(e, &package, &all);
        for node in &pa.nodes {
            let id = id_of(&node.attributes, &node.id);
            let path = usage("p", id);
            let mut ports: Vec<&str> = node.component_ports.iter().map(|p| p.name.as_str()).collect();
            ports.extend(node.ports.iter().map(|p| p.name.as_str()));
            register_part(e, &package, id, &node.name, &ports, &path);
            for behaviour in &node.behavior_components {
                let ports: Vec<&str> = behaviour.ports.iter().map(|p| p.name.as_str()).collect();
                register_part(e, &package, &behaviour.id, &behaviour.name, &ports, &format!("{path}.{}", usage("p", &behaviour.id)));
            }
            for hardware in &node.hardware_components {
                register_part(e, &package, &hardware.id, &hardware.name, &[], &format!("{path}.{}", usage("p", &hardware.id)));
            }
        }
        for (index, link) in pa.links.iter().enumerate() {
            e.register(&package, &link_id(link, index), &[&link.name]);
        }
        for path in &pa.paths {
            e.register(&package, &path.id, &[&path.name]);
        }
        for realization in &pa.capability_realizations {
            e.register(&package, id_of(&realization.attributes, &realization.id), &[&realization.name]);
        }
        for chain in &pa.functional_chains {
            e.register(&package, &chain.id, &[&chain.name]);
        }
    }
    for epbs in &model.epbs {
        let package = levels::epbs_level(epbs).qualified();
        for system in &epbs.systems {
            e.register(&package, id_of(&system.attributes, &system.name), &[&system.name]);
            for subsystem in &system.subsystems {
                e.register(&package, id_of(&subsystem.attributes, &subsystem.name), &[&subsystem.name]);
                for item in &subsystem.items {
                    e.register(&package, id_of(&item.attributes, &item.name), &[&item.name]);
                }
            }
        }
    }
}

fn link_id(link: &PhysicalLink, index: usize) -> String {
    let id = id_of(&link.attributes, &link.name);
    if id.is_empty() { format!("link-{index}") } else { id.to_string() }
}

fn collect_component_functions<'a>(components: &'a [LogicalComponent], out: &mut Vec<&'a LogicalFunction>) {
    for component in components {
        out.extend(component.functions.iter());
        collect_component_functions(&component.sub_components, out);
    }
}

fn register_components(e: &mut Exporter<'_>, package: &str, components: &[LogicalComponent], holder: Option<&str>) {
    for component in components {
        let id = id_of(&component.attributes, &component.id).to_string();
        let path = match holder {
            Some(holder) => format!("{holder}.{}", usage("p", &id)),
            None => usage("p", &id),
        };
        let ports: Vec<&str> = component.ports.iter().map(|p| p.name.as_str()).collect();
        register_part(e, package, &id, &component.name, &ports, &path);
        register_components(e, package, &component.sub_components, Some(&path));
    }
}

pub(super) fn write(e: &mut Exporter<'_>) {
    let model = e.model;
    for la in &model.logical_architecture {
        logical(e, la);
    }
    for pa in &model.physical_architecture {
        physical(e, pa);
    }
    for epbs in &model.epbs {
        breakdown(e, epbs);
    }
}

fn logical(e: &mut Exporter<'_>, la: &LogicalArchitecture) {
    let level = levels::la_level(la);
    e.package(&format!("#LA package {}", text::declared(&level.id, &level.name)), &level.qualified(), |e| {
        e.header("LogicalArchitecture", &la.attributes, &[]);
        let mut interfaces: Vec<String> = Vec::new();
        collect_interfaces(&la.components, &mut interfaces);
        port_defs(e, interfaces);
        let mut all: Vec<&LogicalFunction> = la.functions.iter().collect();
        collect_component_functions(&la.components, &mut all);
        functions(e, "LogicalFunction", &all);
        components(e, &la.components);
        for component in &la.components {
            let id = id_of(&component.attributes, &component.id).to_string();
            e.line(&format!("part {} : {};", usage("p", &id), text::name(&id)));
        }
        for exchange in &la.functional_exchanges {
            flow(e, &exchange.id, exchange.label.as_deref().unwrap_or(""), &exchange.data_type, &exchange.from_port, &exchange.to_port);
        }
        for exchange in &la.component_exchanges {
            connection(e, "", &exchange.id, exchange.label.as_deref().unwrap_or(""), &exchange.from_port, &exchange.to_port, &exchange.attributes, "ComponentExchange");
        }
        for interface in &la.interfaces {
            let id = id_of(&interface.attributes, &interface.name).to_string();
            connection(e, "", &id, &interface.name, &interface.from, &interface.to, &interface.attributes, "LogicalInterface");
        }
        realizations(e, &la.capability_realizations);
        for chain in &la.functional_chains {
            levels::chain(e, "FunctionalChain", chain);
        }
    });
}

fn collect_interfaces(components: &[LogicalComponent], out: &mut Vec<String>) {
    for component in components {
        out.extend(component.ports.iter().map(|p| p.interface_type.clone()));
        collect_interfaces(&component.sub_components, out);
    }
}

/// Every component of the tree as a part def; a held one is a part of its holder.
fn components(e: &mut Exporter<'_>, list: &[LogicalComponent]) {
    for component in list {
        let id = id_of(&component.attributes, &component.id).to_string();
        let actor = super::flag(&component.attributes, "actor") || component.component_type.eq_ignore_ascii_case("actor");
        let metadata = if actor { "#Actor " } else { "" };
        let mut performed: Vec<String> = component.allocated_functions.clone();
        performed.extend(component.functions.iter().map(|f| f.written().to_string()));
        e.block(&format!("{metadata}part def {}", text::declared(&id, &component.name)), |e| {
            e.header("LogicalComponent", &component.attributes, &["actor"]);
            if !component.component_type.is_empty() && !actor {
                e.line(&format!("attribute component_type : String = {};", text::string(&component.component_type)));
            }
            component_ports(e, &component.ports);
            for sub in &component.sub_components {
                let sub_id = id_of(&sub.attributes, &sub.id).to_string();
                e.line(&format!("part {} : {};", usage("p", &sub_id), text::name(&sub_id)));
            }
            performs(e, &performed);
        });
        components(e, &component.sub_components);
    }
}

fn realizations(e: &mut Exporter<'_>, list: &[Capability]) {
    for realization in list {
        let id = id_of(&realization.attributes, &realization.id).to_string();
        levels::use_case(e, "#Capability", "CapabilityRealization", &id, &realization.name, &realization.attributes);
        for involved in &realization.involves {
            super::relations::dependency(e, "involves", &id, involved);
        }
        for (key, target) in [("realizes", &realization.realizes), ("parent", &realization.parent)] {
            if let Some(target) = target {
                super::relations::dependency(e, key, &id, target);
            }
        }
    }
}

fn physical(e: &mut Exporter<'_>, pa: &PhysicalArchitecture) {
    let level = levels::pa_level(pa);
    e.package(&format!("#PA package {}", text::declared(&level.id, &level.name)), &level.qualified(), |e| {
        e.header("PhysicalArchitecture", &pa.attributes, &[]);
        let interfaces = pa.nodes.iter().flat_map(|n| n.component_ports.iter().chain(n.behavior_components.iter().flat_map(|b| b.ports.iter()))).map(|p| p.interface_type.clone());
        port_defs(e, interfaces.collect::<Vec<_>>());
        if pa.nodes.iter().any(|n| !n.ports.is_empty()) {
            e.line("#PhysicalPort port def PhysicalPort;");
        }
        let mut all: Vec<&LogicalFunction> = pa.functions.iter().collect();
        for node in &pa.nodes {
            all.extend(node.functions.iter());
            all.extend(node.behavior_components.iter().flat_map(|b| b.functions.iter()));
        }
        functions(e, "PhysicalFunction", &all);
        for node in &pa.nodes {
            node_defs(e, node);
        }
        for node in &pa.nodes {
            let id = id_of(&node.attributes, &node.id).to_string();
            e.line(&format!("part {} : {};", usage("p", &id), text::name(&id)));
        }
        for node in &pa.nodes {
            let node_id = id_of(&node.attributes, &node.id).to_string();
            for deployment in &node.deployments {
                super::relations::allocation(e, &deployment.component, &node_id);
            }
        }
        for (index, link) in pa.links.iter().enumerate() {
            let id = link_id(link, index);
            let mut attributes = link.attributes.clone();
            if !link.protocol.is_empty() {
                attributes.entry("protocol".into()).or_insert(AttributeValue::String(link.protocol.clone()));
            }
            if let Some(bandwidth) = &link.bandwidth {
                attributes.entry("bandwidth".into()).or_insert(AttributeValue::String(bandwidth.clone()));
            }
            connection(e, "#Link ", &id, &link.name, &link.from, &link.to, &attributes, "PhysicalLink");
        }
        for exchange in &pa.physical_exchanges {
            let mut attributes = exchange.attributes.clone();
            if !exchange.message_type.is_empty() {
                attributes.entry("message_type".into()).or_insert(AttributeValue::String(exchange.message_type.clone()));
            }
            if let Some(frequency) = &exchange.frequency {
                attributes.entry("frequency".into()).or_insert(AttributeValue::String(frequency.clone()));
            }
            connection(e, "", &exchange.id, exchange.label.as_deref().unwrap_or(""), &exchange.from, &exchange.to, &attributes, "PhysicalExchange");
            if let Some(via) = &exchange.via {
                super::relations::dependency(e, "via", &exchange.id, via);
            }
        }
        for exchange in &pa.functional_exchanges {
            flow(e, &exchange.id, exchange.label.as_deref().unwrap_or(""), &exchange.data_type, &exchange.from_port, &exchange.to_port);
        }
        for path in &pa.paths {
            e.block(&format!("#Path part def {}", text::declared(&path.id, &path.name)), |e| e.header("PhysicalPath", &path.attributes, &[]));
            for involved in &path.involves {
                super::relations::dependency(e, "involves", &path.id, involved);
            }
        }
        realizations(e, &pa.capability_realizations);
        for chain in &pa.functional_chains {
            levels::chain(e, "FunctionalChain", chain);
        }
    });
}

/// A node, its behaviour and hardware components as part defs; the node holds them.
fn node_defs(e: &mut Exporter<'_>, node: &PhysicalNode) {
    let id = id_of(&node.attributes, &node.id).to_string();
    for behaviour in &node.behavior_components {
        let mut performed = behaviour.allocated_functions.clone();
        performed.extend(behaviour.functions.iter().map(|f| f.written().to_string()));
        e.block(&format!("#Behaviour part def {}", text::declared(&behaviour.id, &behaviour.name)), |e| {
            e.header("BehaviorComponent", &behaviour.attributes, &[]);
            component_ports(e, &behaviour.ports);
            performs(e, &performed);
        });
        if let Some(component) = &behaviour.allocated_component {
            super::relations::dependency(e, "allocated_component", &behaviour.id, component);
        }
    }
    for hardware in &node.hardware_components {
        e.block(&format!("#Hardware part def {}", text::declared(&hardware.id, &hardware.name)), |e| {
            if !hardware.hw_type.is_empty() {
                e.line(&format!("attribute hw_type : String = {};", text::string(&hardware.hw_type)));
            }
            if let Some(specs) = &hardware.specs {
                e.line(&format!("attribute specs : String = {};", text::string(specs)));
            }
        });
    }
    let performed: Vec<String> = node.functions.iter().map(|f| f.written().to_string()).collect();
    let actor = super::flag(&node.attributes, "actor");
    let metadata = if actor { "#Node #Actor" } else { "#Node" };
    e.block(&format!("{metadata} part def {}", text::declared(&id, &node.name)), |e| {
        e.header("PhysicalNode", &node.attributes, &["actor"]);
        for (key, value) in [("processor", &node.processor), ("memory", &node.memory)] {
            if let Some(value) = value {
                e.line(&format!("attribute {key} : String = {};", text::string(value)));
            }
        }
        component_ports(e, &node.component_ports);
        for port in &node.ports {
            let short = id_of(&port.attributes, "");
            let short = if short.is_empty() { String::new() } else { format!("{} ", text::short(short)) };
            e.line(&format!("port {short}{} : PhysicalPort;", text::name(&port.name)));
        }
        for behaviour in &node.behavior_components {
            e.line(&format!("part {} : {};", usage("p", &behaviour.id), text::name(&behaviour.id)));
        }
        for hardware in &node.hardware_components {
            e.line(&format!("part {} : {};", usage("p", &hardware.id), text::name(&hardware.id)));
        }
        performs(e, &performed);
    });
}

fn breakdown(e: &mut Exporter<'_>, epbs: &Epbs) {
    let level = levels::epbs_level(epbs);
    e.package(&format!("#EPBS package {}", text::declared(&level.id, &level.name)), &level.qualified(), |e| {
        e.header("Epbs", &epbs.attributes, &[]);
        for system in &epbs.systems {
            for subsystem in &system.subsystems {
                for item in &subsystem.items {
                    let id = id_of(&item.attributes, &item.name);
                    e.block(&format!("part def {}", text::declared(id, &item.name)), |e| e.header("EpbsItem", &item.attributes, &[]));
                }
                let id = id_of(&subsystem.attributes, &subsystem.name).to_string();
                e.block(&format!("part def {}", text::declared(&id, &subsystem.name)), |e| {
                    e.header("EpbsSubsystem", &subsystem.attributes, &[]);
                    for item in &subsystem.items {
                        let item_id = id_of(&item.attributes, &item.name);
                        e.line(&format!("part {} : {};", usage("p", item_id), text::name(item_id)));
                    }
                });
            }
            let id = id_of(&system.attributes, &system.name).to_string();
            e.block(&format!("part def {}", text::declared(&id, &system.name)), |e| {
                e.header("EpbsSystem", &system.attributes, &[]);
                for subsystem in &system.subsystems {
                    let sub_id = id_of(&subsystem.attributes, &subsystem.name);
                    e.line(&format!("part {} : {};", usage("p", sub_id), text::name(sub_id)));
                }
            });
        }
    });
}
