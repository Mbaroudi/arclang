//! System Architecture Blank: the system with its functions, the external
//! actors, and the functional exchanges between function ports.

use super::builder::{identity, properties, view_title, Builder, NodeRef};
use super::functions::{add_function, add_functional_exchanges, UNSPECIFIED_ITEM};
use super::logical::check_direction;
use super::{Diagram, EdgeKind, NodeKind, PortDirection, ViewKind};
use crate::compiler::ast::{self, ComponentPort, Model};
use std::collections::BTreeMap;

/// The oriented ports of a system component or actor.
fn add_component_ports(builder: &mut Builder, node: NodeRef, ports: &[ComponentPort]) {
    for port in ports {
        let direction = match port.direction {
            ast::PortDirection::In => PortDirection::In,
            ast::PortDirection::Out => PortDirection::Out,
            ast::PortDirection::InOut => PortDirection::InOut,
        };
        let interface = Some(port.interface_type.clone()).filter(|t| !t.is_empty());
        builder.add_port(node, &port.name, &port.name, direction, interface, None);
    }
}

/// Identity of the box standing for the system under study.
const SYSTEM_ID: &str = "SA-System";

pub(super) fn build(model: &Model) -> Option<(Diagram, Vec<String>)> {
    let mut builder = Builder::new(ViewKind::Sab);

    let has_functions = model
        .system_analysis
        .iter()
        .any(|sa| !sa.functions.is_empty());
    if has_functions {
        let name = super::builder::text(&model.attributes, "name").unwrap_or("System");
        let system = builder.add_node(None, SYSTEM_ID, name, NodeKind::System, BTreeMap::new());
        for sa in &model.system_analysis {
            for function in &sa.functions {
                add_function(&mut builder, Some(system), function);
            }
        }
    }
    for sa in &model.system_analysis {
        for actor in &sa.external_actors {
            let node = builder.add_node(
                None,
                identity(&actor.attributes, &actor.id),
                &actor.name,
                NodeKind::SystemActor,
                properties(&actor.attributes, &["description", "type"]),
            );
            add_component_ports(&mut builder, node, &actor.ports);
        }
        for component in &sa.components {
            let node = builder.add_node(
                None,
                identity(&component.attributes, &component.name),
                &component.name,
                NodeKind::SystemComponent,
                properties(
                    &component.attributes,
                    &["description", "type", "safety_level", "asil"],
                ),
            );
            add_component_ports(&mut builder, node, &component.ports);
        }
    }
    if builder.is_empty() {
        return None;
    }

    // Component exchanges between the ports of the system, its components and its actors.
    for sa in &model.system_analysis {
        for exchange in &sa.component_exchanges {
            let name = exchange
                .label
                .clone()
                .unwrap_or_else(|| format!("{} -> {}", exchange.from_port, exchange.to_port));
            let what = format!("component_exchange '{name}'");
            let Some((source, target)) =
                builder.resolve_pair(&what, &exchange.from_port, &exchange.to_port, "component")
            else {
                continue;
            };
            check_direction(&mut builder, &what, &source, PortDirection::In, "leaves through input port");
            check_direction(&mut builder, &what, &target, PortDirection::Out, "arrives on output port");
            let item = Some(exchange.exchange_item.clone()).filter(|i| !i.is_empty() && i != UNSPECIFIED_ITEM);
            builder.add_edge(EdgeKind::ComponentExchange, &source, &target, &name, item, BTreeMap::new());
        }
    }

    add_functional_exchanges(
        &mut builder,
        model
            .system_analysis
            .iter()
            .flat_map(|sa| sa.functional_exchanges.iter()),
        None,
    );
    for sa in &model.system_analysis {
        for chain in &sa.functional_chains {
            builder.add_chain(&chain.id, &chain.name, &chain.involves);
        }
    }
    builder.finish(view_title(
        ViewKind::Sab,
        model.system_analysis.iter().map(|sa| sa.name.clone()),
    ))
}
