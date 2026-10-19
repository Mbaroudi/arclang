//! The logical and physical architectures and the EPBS, as usages: a
//! component is a `part` of its level, the components it holds `part`s
//! inside it, so `'LC-1'.'LC-2'.radar` is a port a connection can end on;
//! a function declared in a component is a `perform action` there. A
//! physical node holds its behaviour and hardware components the same way,
//! with its oriented ports and its `#PhysicalPort`s.

use super::levels::{self, alias_held, component_ports, connection, flow, function, function_keys, performs, port_defs, ports_of, register_function, register_part};
use super::{id_of, text, Exporter};
use crate::compiler::ast::*;

pub(super) fn register(e: &mut Exporter<'_>) {
    let model = e.model;
    for la in &model.logical_architecture {
        let package = levels::la_level(la).qualified();
        for f in &la.functions {
            register_function(e, &package, &[], f);
        }
        register_components(e, &package, &la.components, &[]);
        for exchange in la.functional_exchanges.iter().map(|x| &x.id).chain(la.component_exchanges.iter().map(|x| &x.id)) {
            e.register(&package, exchange, &[]);
        }
        for realization in &la.capability_realizations {
            e.register(&package, id_of(&realization.attributes, &realization.id), &[&realization.name]);
        }
        for chain in &la.functional_chains {
            e.register(&package, &chain.id, &[&chain.name]);
        }
    }
    for pa in &model.physical_architecture {
        let package = levels::pa_level(pa).qualified();
        for f in &pa.functions {
            register_function(e, &package, &[], f);
        }
        for node in &pa.nodes {
            let id = id_of(&node.attributes, &node.id);
            let mut ports = ports_of(&node.component_ports);
            ports.extend(node.ports.iter().map(|p| (id_of(&p.attributes, ""), p.name.as_str())));
            let at = register_part(e, &package, &[], id, &node.name, &ports);
            for f in &node.functions {
                register_function(e, &package, &at, f);
            }
            levels::alias_held(e, &[id, &node.name], &node.functions);
            for behaviour in &node.behavior_components {
                let inside = register_part(e, &package, &at, &behaviour.id, &behaviour.name, &ports_of(&behaviour.ports));
                for f in &behaviour.functions {
                    register_function(e, &package, &inside, f);
                }
                levels::alias_held(e, &[&behaviour.id, &behaviour.name], &behaviour.functions);
            }
            for hardware in &node.hardware_components {
                register_part(e, &package, &at, &hardware.id, &hardware.name, &[]);
            }
        }
        for link in pa.links.iter().filter(|l| l.name != l.from) {
            let id = id_of(&link.attributes, &link.name);
            e.register(&package, id, &[&link.name]);
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
        for exchange in pa.physical_exchanges.iter().map(|x| &x.id).chain(pa.functional_exchanges.iter().map(|x| &x.id)) {
            e.register(&package, exchange, &[]);
        }
    }
    for epbs in &model.epbs {
        let package = levels::epbs_level(epbs).qualified();
        for system in &epbs.systems {
            let at = register_part(e, &package, &[], id_of(&system.attributes, &system.name), &system.name, &[]);
            for subsystem in &system.subsystems {
                let inside = register_part(e, &package, &at, id_of(&subsystem.attributes, &subsystem.name), &subsystem.name, &[]);
                for item in &subsystem.items {
                    register_part(e, &package, &inside, id_of(&item.attributes, &item.name), &item.name, &[]);
                }
            }
        }
    }
}


fn register_components(e: &mut Exporter<'_>, package: &str, components: &[LogicalComponent], base: &[String]) {
    for component in components {
        let id = id_of(&component.attributes, &component.id);
        let mut ports = ports_of(&component.ports);
        ports.extend(component.interfaces_in.iter().chain(component.interfaces_out.iter()).map(|i| ("", i.name.as_str())));
        let at = register_part(e, package, base, id, &component.name, &ports);
        levels::register_sub_ports(e, package, &at, &[id, &component.name], &component.ports);
        for f in &component.functions {
            register_function(e, package, &at, f);
        }
        alias_held(e, &[id, &component.name], &component.functions);
        register_components(e, package, &component.sub_components, &at);
    }
}

pub(super) fn write(e: &mut Exporter<'_>) {
    let model = e.model;
    for group in levels::grouped(&model.logical_architecture, levels::la_level) {
        levels::in_package(e, levels::la_level(group[0]), |e| group.iter().for_each(|la| logical(e, la)));
    }
    for group in levels::grouped(&model.physical_architecture, levels::pa_level) {
        levels::in_package(e, levels::pa_level(group[0]), |e| group.iter().for_each(|pa| physical(e, pa)));
    }
    for group in levels::grouped(&model.epbs, levels::epbs_level) {
        levels::in_package(e, levels::epbs_level(group[0]), |e| group.iter().for_each(|b| breakdown(e, b)));
    }
}

fn logical(e: &mut Exporter<'_>, la: &LogicalArchitecture) {
    {
        let e = &mut *e;
        e.header("LogicalArchitecture", &la.attributes, &[]);
        let mut interfaces: Vec<String> = Vec::new();
        collect_interfaces(&la.components, &mut interfaces);
        if has_offered_interfaces(&la.components) {
            interfaces.push(String::new());
        }
        port_defs(e, interfaces);
        for f in &la.functions {
            function(e, "LogicalFunction", f, "action");
        }
        for component in &la.components {
            logical_component(e, component);
        }
        for exchange in &la.functional_exchanges {
            flow(e, &exchange.id, exchange.label.as_deref().unwrap_or(""), &exchange.data_type, &exchange.from_port, &exchange.to_port);
        }
        for exchange in &la.component_exchanges {
            connection(e, "", &exchange.id, exchange.label.as_deref().unwrap_or(""), &exchange.from_port, &exchange.to_port, &exchange.attributes, "ComponentExchange");
        }
        for interface in &la.interfaces {
            let id = id_of(&interface.attributes, &interface.name).to_string();
            connection(e, "#Interface ", &id, &interface.name, &interface.from, &interface.to, &interface.attributes, "LogicalInterface");
        }
        realizations(e, &la.capability_realizations);
        let mut all: Vec<&LogicalFunction> = la.functions.iter().collect();
        collect_component_functions(&la.components, &mut all);
        let keys = function_keys(&all);
        for chain in &la.functional_chains {
            levels::chain(e, "FunctionalChain", chain, &keys);
        }
    }
}

fn collect_interfaces(components: &[LogicalComponent], out: &mut Vec<String>) {
    for component in components {
        out.extend(component.ports.iter().map(|p| p.interface_type.clone()));
        collect_interfaces(&component.sub_components, out);
    }
}

fn has_offered_interfaces(components: &[LogicalComponent]) -> bool {
    components.iter().any(|c| !c.interfaces_in.is_empty() || !c.interfaces_out.is_empty() || has_offered_interfaces(&c.sub_components))
}

fn collect_component_functions<'a>(components: &'a [LogicalComponent], out: &mut Vec<&'a LogicalFunction>) {
    for component in components {
        out.extend(component.functions.iter());
        collect_component_functions(&component.sub_components, out);
    }
}

/// A component, the functions it holds and the components inside it. A
/// variation holds nothing but its variants (SysML v2): what it has of its
/// own (attributes, ports, functions) is a definition typing it.
fn logical_component(e: &mut Exporter<'_>, component: &LogicalComponent) {
    let id = id_of(&component.attributes, &component.id).to_string();
    let actor = super::flag(&component.attributes, "actor") || component.component_type.eq_ignore_ascii_case("actor");
    let metadata = if actor { "#Actor " } else { "" };
    let (bounds, stated) = super::multiplicity("LogicalComponent", &component.attributes);
    let typing = e.typing(&component.attributes);
    let skip = ["actor", "variation", "variant", stated.unwrap_or("")];
    if crate::compiler::variability::is_variation(&component.attributes) {
        let mut typing = typing;
        if has_own_features(component, &skip) {
            let definition = text::name(&format!("{} variation", component.name));
            let specializes = typing.replacen(" : ", " :> ", 1);
            e.block(&format!("{metadata}part def {definition}{specializes}"), |e| own_features(e, component, &skip));
            typing = format!(" : {definition}");
        }
        e.block(&format!("variation part {}{typing}{bounds}", text::declared(&id, &component.name)), |e| {
            for sub in &component.sub_components {
                logical_component(e, sub);
            }
        });
        return;
    }
    let variant = if crate::compiler::variability::is_variant(&component.attributes) { "variant " } else { "" };
    e.block(&format!("{metadata}{variant}part {}{typing}{bounds}", text::declared(&id, &component.name)), |e| {
        own_features(e, component, &skip);
        for sub in &component.sub_components {
            logical_component(e, sub);
        }
    });
}

/// Whether a component has more than its identity and its subcomponents.
fn has_own_features(component: &LogicalComponent, skip: &[&str]) -> bool {
    let identity = ["id", "name", "is"];
    component.attributes.keys().any(|k| !identity.contains(&k.as_str()) && !skip.contains(&k.as_str()))
        || (!component.component_type.is_empty() && component.component_type != "Logical")
        || !component.ports.is_empty()
        || !component.interfaces_in.is_empty()
        || !component.interfaces_out.is_empty()
        || !component.functions.is_empty()
        || !component.allocated_functions.is_empty()
}

/// What a component has of its own: header, ports, offered interfaces, functions.
fn own_features(e: &mut Exporter<'_>, component: &LogicalComponent, skip: &[&str]) {
    let actor = super::flag(&component.attributes, "actor") || component.component_type.eq_ignore_ascii_case("actor");
    e.header("LogicalComponent", &component.attributes, skip);
    // `Logical` is what the parser gives a component that states no type.
    if !component.component_type.is_empty() && component.component_type != "Logical" && !actor {
        e.line(&format!("attribute component_type : String = {};", text::string(&component.component_type)));
    }
    levels::owned_ports(e, &component.ports, &component.attributes);
    // `interface_in` / `interface_out`: ports named by the interface they offer.
    for (direction, interfaces) in [("in", &component.interfaces_in), ("out", &component.interfaces_out)] {
        for interface in interfaces {
            let mut attributes = interface.attributes.clone();
            for (key, value) in [("protocol", &interface.protocol), ("format", &interface.format)] {
                if let Some(value) = value {
                    attributes.entry(key.to_string()).or_insert(AttributeValue::String(value.clone()));
                }
            }
            e.block(&format!("{direction} port {} : Port", text::name(&interface.name)), |e| e.header("InterfaceDefinition", &attributes, &[]));
        }
    }
    for f in &component.functions {
        function(e, "LogicalFunction", f, "perform action");
    }
    performs(e, &component.allocated_functions);
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
    {
        let e = &mut *e;
        e.header("PhysicalArchitecture", &pa.attributes, &[]);
        let interfaces = pa.nodes.iter().flat_map(|n| n.component_ports.iter().chain(n.behavior_components.iter().flat_map(|b| b.ports.iter()))).map(|p| p.interface_type.clone());
        port_defs(e, interfaces.collect::<Vec<_>>());
        for f in &pa.functions {
            function(e, "PhysicalFunction", f, "action");
        }
        for node in &pa.nodes {
            physical_node(e, node);
        }
        for node in &pa.nodes {
            let node_id = id_of(&node.attributes, &node.id).to_string();
            for deployment in &node.deployments {
                super::relations::allocation(e, &deployment.component, &node_id);
            }
            for behaviour in &node.behavior_components {
                if let Some(component) = &behaviour.allocated_component {
                    super::relations::dependency(e, "allocated_component", &behaviour.id, component);
                }
            }
        }
        for link in &pa.links {
            // `link "A" -> "B"` has no name of its own: the parser gives it its source.
            let name = if link.name == link.from { "" } else { link.name.as_str() };
            let id = id_of(&link.attributes, "").to_string();
            let mut attributes = link.attributes.clone();
            if !link.protocol.is_empty() {
                attributes.entry("protocol".into()).or_insert(AttributeValue::String(link.protocol.clone()));
            }
            if let Some(bandwidth) = &link.bandwidth {
                attributes.entry("bandwidth".into()).or_insert(AttributeValue::String(bandwidth.clone()));
            }
            connection(e, "#Link ", &id, name, &link.from, &link.to, &attributes, "PhysicalLink");
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
            e.block(&format!("#Path part {}", text::declared(&path.id, &path.name)), |e| e.header("PhysicalPath", &path.attributes, &[]));
            for involved in &path.involves {
                super::relations::dependency(e, "involves", &path.id, involved);
            }
        }
        realizations(e, &pa.capability_realizations);
        let mut all: Vec<&LogicalFunction> = pa.functions.iter().collect();
        for node in &pa.nodes {
            all.extend(node.functions.iter());
            all.extend(node.behavior_components.iter().flat_map(|b| b.functions.iter()));
        }
        let keys = function_keys(&all);
        for chain in &pa.functional_chains {
            levels::chain(e, "FunctionalChain", chain, &keys);
        }
    }
}

/// A node with its ports, the functions it holds, and its behaviour and hardware components.
fn physical_node(e: &mut Exporter<'_>, node: &PhysicalNode) {
    let id = id_of(&node.attributes, &node.id).to_string();
    let actor = super::flag(&node.attributes, "actor");
    let metadata = if actor { "#Node #Actor" } else { "#Node" };
    let (bounds, stated) = super::multiplicity("PhysicalNode", &node.attributes);
    let typing = e.typing(&node.attributes);
    e.block(&format!("{metadata} part {}{typing}{bounds}", text::declared(&id, &node.name)), |e| {
        e.header("PhysicalNode", &node.attributes, &["actor", stated.unwrap_or("")]);
        for (key, value) in [("processor", &node.processor), ("memory", &node.memory)] {
            if let Some(value) = value {
                e.line(&format!("attribute {key} : String = {};", text::string(value)));
            }
        }
        levels::owned_ports(e, &node.component_ports, &node.attributes);
        for port in &node.ports {
            let short = id_of(&port.attributes, "");
            let short = if short.is_empty() { String::new() } else { format!("{} ", text::short(short)) };
            e.line(&format!("#PhysicalPort port {short}{};", text::name(&port.name)));
        }
        for f in &node.functions {
            function(e, "PhysicalFunction", f, "perform action");
        }
        for behaviour in &node.behavior_components {
            e.block(&format!("#Behaviour part {}", text::declared(&behaviour.id, &behaviour.name)), |e| {
                e.header("BehaviorComponent", &behaviour.attributes, &[]);
                component_ports(e, &behaviour.ports);
                for f in &behaviour.functions {
                    function(e, "PhysicalFunction", f, "perform action");
                }
                performs(e, &behaviour.allocated_functions);
            });
        }
        for hardware in &node.hardware_components {
            e.block(&format!("#Hardware part {}", text::declared(&hardware.id, &hardware.name)), |e| {
                if !hardware.hw_type.is_empty() {
                    e.line(&format!("attribute hw_type : String = {};", text::string(&hardware.hw_type)));
                }
                if let Some(specs) = &hardware.specs {
                    e.line(&format!("attribute specs : String = {};", text::string(specs)));
                }
            });
        }
    });
}

fn breakdown(e: &mut Exporter<'_>, epbs: &Epbs) {
    {
        let e = &mut *e;
        e.header("Epbs", &epbs.attributes, &[]);
        for system in &epbs.systems {
            e.block(&format!("part {}", text::declared(id_of(&system.attributes, &system.name), &system.name)), |e| {
                e.header("EpbsSystem", &system.attributes, &[]);
                for subsystem in &system.subsystems {
                    e.block(&format!("part {}", text::declared(id_of(&subsystem.attributes, &subsystem.name), &subsystem.name)), |e| {
                        e.header("EpbsSubsystem", &subsystem.attributes, &[]);
                        for item in &subsystem.items {
                            e.block(&format!("part {}", text::declared(id_of(&item.attributes, &item.name), &item.name)), |e| e.header("EpbsItem", &item.attributes, &[]));
                        }
                    });
                }
            });
        }
    }
}
