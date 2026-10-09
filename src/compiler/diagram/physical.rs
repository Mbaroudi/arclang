//! Physical Architecture Blank: nodes containing the behaviour and hardware
//! components deployed on them, physical links and physical exchanges.

use super::builder::{identity, properties, text, view_title, Builder};
use super::functions::{add_functional_exchanges, FunctionTable, Placement};
use super::{Diagram, EdgeKind, NodeKind, PortDirection, ViewKind};
use crate::compiler::ast::{LogicalComponent, Model};
use std::collections::{BTreeMap, HashMap};

const NODE_PROPERTIES: [&str; 6] = [
    "description",
    "node_type",
    "processor",
    "memory",
    "safety_level",
    "asil",
];

pub(super) fn build(model: &Model) -> Option<(Diagram, Vec<String>)> {
    let mut builder = Builder::new(ViewKind::Pab);
    let mut functions = FunctionTable::of(model);
    let logical = logical_component_ids(model);

    for pa in &model.physical_architecture {
        for node in &pa.nodes {
            let handle = builder.add_node(
                None,
                identity(&node.attributes, &node.name),
                &node.name,
                NodeKind::PhysicalNode,
                properties(&node.attributes, &NODE_PROPERTIES),
            );
            let node_id = builder.node(handle).id.clone();
            for port in &node.ports {
                builder.add_port(
                    handle,
                    &port.name,
                    &port.name,
                    PortDirection::Undirected,
                    None,
                    text(&port.attributes, "protocol").map(str::to_string),
                );
            }
            for behavior in &node.behavior_components {
                let child = builder.add_node(
                    Some(handle),
                    &behavior.id,
                    &behavior.name,
                    NodeKind::BehaviorComponent,
                    BTreeMap::new(),
                );
                // The parser stores `allocated_component` and
                // `allocated_function(s)` in one list: tell them apart here.
                for reference in &behavior.allocated_functions {
                    if let Some(realized) = logical.get(reference.as_str()) {
                        builder.add_realization(child, (*realized).to_string());
                    } else {
                        match functions.place(&mut builder, child, reference) {
                            Placement::Drawn => {}
                            Placement::Unknown => builder.report(format!(
                                "behavior_component '{}': '{reference}' is neither a declared \
                                 logical component nor a function",
                                behavior.name
                            )),
                            Placement::Ambiguous => builder.report(format!(
                                "behavior_component '{}': '{reference}' names several functions — \
                                 reference it by a unique id",
                                behavior.name
                            )),
                        }
                    }
                }
            }
            for hardware in &node.hardware_components {
                let mut hardware_properties = BTreeMap::new();
                if !hardware.hw_type.is_empty() {
                    hardware_properties.insert("type".to_string(), hardware.hw_type.clone());
                }
                if let Some(specs) = &hardware.specs {
                    hardware_properties.insert("specs".to_string(), specs.clone());
                }
                builder.add_node(
                    Some(handle),
                    &hardware.id,
                    &hardware.name,
                    NodeKind::HardwareComponent,
                    hardware_properties,
                );
            }
            for deployment in &node.deployments {
                let child = builder.add_node(
                    Some(handle),
                    &format!("{node_id}/{}", deployment.component),
                    &deployment.component,
                    NodeKind::DeployedComponent,
                    properties(&deployment.attributes, &["description", "partition"]),
                );
                if let Some(realized) = logical.get(deployment.component.as_str()) {
                    builder.add_realization(child, (*realized).to_string());
                }
            }
        }
    }
    if builder.is_empty() {
        return None;
    }

    for pa in &model.physical_architecture {
        for link in &pa.links {
            let name = if link.name.is_empty() {
                format!("{} -> {}", link.from, link.to)
            } else {
                link.name.clone()
            };
            let what = format!("link '{name}'");
            let Some((source, target)) = builder.resolve_pair(&what, &link.from, &link.to, "node")
            else {
                continue;
            };
            let mut link_properties = properties(&link.attributes, &["bandwidth", "latency"]);
            if !link.protocol.is_empty() {
                link_properties.insert("protocol".to_string(), link.protocol.clone());
            }
            builder.add_edge(
                EdgeKind::PhysicalLink,
                &source,
                &target,
                &name,
                None,
                link_properties,
            );
        }
        for exchange in &pa.physical_exchanges {
            let name = exchange
                .label
                .clone()
                .unwrap_or_else(|| format!("{} -> {}", exchange.from, exchange.to));
            let what = format!("physical_exchange '{name}'");
            let Some((source, target)) =
                builder.resolve_pair(&what, &exchange.from, &exchange.to, "node or component")
            else {
                continue;
            };
            let mut exchange_properties = BTreeMap::new();
            if let Some(via) = &exchange.via {
                exchange_properties.insert("via".to_string(), via.clone());
            }
            if let Some(frequency) = &exchange.frequency {
                exchange_properties.insert("frequency".to_string(), frequency.clone());
            }
            let item = Some(exchange.message_type.clone()).filter(|m| !m.is_empty());
            builder.add_edge(
                EdgeKind::PhysicalExchange,
                &source,
                &target,
                &name,
                item,
                exchange_properties,
            );
        }
    }
    add_functional_exchanges(
        &mut builder,
        model
            .system_analysis
            .iter()
            .flat_map(|sa| sa.functional_exchanges.iter()),
        Some(&functions),
    );
    for pa in &model.physical_architecture {
        for path in &pa.paths {
            builder.add_chain(&path.id, &path.name, &path.involves);
        }
    }
    builder.finish(view_title(
        ViewKind::Pab,
        model.physical_architecture.iter().map(|pa| pa.name.clone()),
    ))
}

/// Logical components addressable by id or name -> their identity.
fn logical_component_ids(model: &Model) -> HashMap<&str, &str> {
    fn collect<'a>(component: &'a LogicalComponent, out: &mut HashMap<&'a str, &'a str>) {
        let id = identity(&component.attributes, &component.name);
        out.entry(id).or_insert(id);
        out.entry(component.name.as_str()).or_insert(id);
        for sub in &component.sub_components {
            collect(sub, out);
        }
    }
    let mut ids = HashMap::new();
    for la in &model.logical_architecture {
        for component in &la.components {
            collect(component, &mut ids);
        }
    }
    ids
}
