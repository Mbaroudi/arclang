//! Logical Architecture Blank: components (nested), the functions allocated
//! to them, component exchanges between component ports and functional
//! exchanges between the allocated functions.

use super::builder::{
    identity, properties, string_list, text, view_title, Builder, Endpoint, NodeRef,
};
use super::functions::{
    add_flow_ports, add_functional_exchanges, dedupe, FunctionTable, Placement, UNSPECIFIED_ITEM,
};
use super::{Diagram, EdgeKind, NodeKind, PortDirection, ViewKind};
use crate::compiler::ast::{self, InterfaceDefinition, LogicalComponent, Model};
use std::collections::BTreeMap;

const COMPONENT_PROPERTIES: [&str; 4] = ["description", "type", "safety_level", "asil"];

pub(super) fn build(model: &Model) -> Option<(Diagram, Vec<String>)> {
    let mut builder = Builder::new(ViewKind::Lab);
    let mut functions = FunctionTable::of(model);

    for la in &model.logical_architecture {
        for component in &la.components {
            add_component(&mut builder, &mut functions, None, component);
        }
    }
    if builder.is_empty() {
        return None;
    }

    for la in &model.logical_architecture {
        for exchange in &la.component_exchanges {
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
            check_direction(
                &mut builder,
                &what,
                &source,
                PortDirection::In,
                "leaves through input port",
            );
            check_direction(
                &mut builder,
                &what,
                &target,
                PortDirection::Out,
                "arrives on output port",
            );
            let item = Some(exchange.exchange_item.clone())
                .filter(|i| !i.is_empty() && i != UNSPECIFIED_ITEM);
            builder.add_edge(
                EdgeKind::ComponentExchange,
                &source,
                &target,
                &name,
                item,
                BTreeMap::new(),
            );
        }
        for interface in &la.interfaces {
            let what = format!("interface '{}'", interface.name);
            let Some((source, target)) =
                builder.resolve_pair(&what, &interface.from, &interface.to, "component")
            else {
                continue;
            };
            builder.add_edge(
                EdgeKind::ComponentExchange,
                &source,
                &target,
                &interface.name,
                None,
                properties(&interface.attributes, &["description", "protocol"]),
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
    for la in &model.logical_architecture {
        for chain in &la.functional_chains {
            builder.add_chain(&chain.id, &chain.name, &chain.involves);
        }
    }
    builder.finish(view_title(
        ViewKind::Lab,
        model.logical_architecture.iter().map(|la| la.name.clone()),
    ))
}

fn add_component(
    builder: &mut Builder,
    functions: &mut FunctionTable<'_>,
    parent: Option<NodeRef>,
    component: &LogicalComponent,
) {
    let node = builder.add_node(
        parent,
        identity(&component.attributes, &component.name),
        &component.name,
        NodeKind::LogicalComponent,
        properties(&component.attributes, &COMPONENT_PROPERTIES),
    );
    for interface in &component.interfaces_in {
        add_interface_port(builder, node, interface, PortDirection::In);
    }
    for interface in &component.interfaces_out {
        add_interface_port(builder, node, interface, PortDirection::Out);
    }
    for port in &component.ports {
        let direction = match port.direction {
            ast::PortDirection::In => PortDirection::In,
            ast::PortDirection::Out => PortDirection::Out,
            ast::PortDirection::InOut => PortDirection::InOut,
        };
        let interface = Some(port.interface_type.clone()).filter(|t| !t.is_empty());
        builder.add_port(node, &port.name, &port.name, direction, interface, None);
    }

    let allocated = dedupe(
        component
            .allocated_functions
            .iter()
            .cloned()
            .chain(string_list(&component.attributes, "allocated_function"))
            .chain(string_list(&component.attributes, "allocated_functions")),
    );
    for reference in &allocated {
        match functions.place(builder, node, reference) {
            Placement::Drawn => {}
            Placement::Unknown => builder.report(format!(
                "component '{}': allocated function '{reference}' is not declared in the system analysis",
                component.name
            )),
            Placement::Ambiguous => builder.report(format!(
                "component '{}': allocated function '{reference}' names several functions — \
                 reference it by a unique id",
                component.name
            )),
        }
    }
    // Functions declared inline belong to this component alone: qualify
    // their identity so `Init` in two components stays two elements.
    let component_id = builder.node(node).id.clone();
    for function in &component.functions {
        let qualified = format!("{component_id}.{}", function.name);
        let function_node = builder.add_node(
            Some(node),
            identity(&function.attributes, &qualified),
            &function.name,
            NodeKind::Function,
            properties(
                &function.attributes,
                &["description", "safety_level", "asil", "latency"],
            ),
        );
        add_flow_ports(builder, function_node, &function.attributes);
    }
    for sub in &component.sub_components {
        add_component(builder, functions, Some(node), sub);
    }
}

fn add_interface_port(
    builder: &mut Builder,
    node: NodeRef,
    interface: &InterfaceDefinition,
    direction: PortDirection,
) {
    let carried = text(&interface.attributes, "name").map(str::to_string);
    let protocol = interface
        .protocol
        .clone()
        .or_else(|| text(&interface.attributes, "protocol").map(str::to_string));
    builder.add_port(
        node,
        &interface.name,
        &interface.name,
        direction,
        carried,
        protocol,
    );
}

/// Report an exchange bound to a port of the wrong orientation.
fn check_direction(
    builder: &mut Builder,
    what: &str,
    endpoint: &Endpoint,
    wrong: PortDirection,
    problem: &str,
) {
    let Some(port_id) = &endpoint.port else {
        return;
    };
    let node = builder.node(endpoint.node);
    if let Some(port) = node
        .ports
        .iter()
        .find(|p| &p.id == port_id && p.direction == wrong)
    {
        let message = format!("{what}: {problem} '{}' of '{}'", port.name, node.name);
        builder.report(message);
    }
}
