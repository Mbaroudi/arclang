//! `refine "<id>" { ... }`: what a model adds to an element declared
//! elsewhere, most often in a file imported from another tool (a Capella
//! project read by `arclang import -f capella`) and never edited by hand.
//! The architecture stays where it is authored; what the simulation needs
//! (the physics of a port, a parameter's typed value, the executable model
//! of an element) is written beside it, in a file of its own, and survives
//! the next import.
//!
//! ```text
//! import "drone_from_capella.arc"
//!
//! refine "PN-BATT" {
//!     model: "DronePhysics.Pack"
//!     port "p" { is: "Power plus" }
//! }
//! ```
//!
//! The body is a node's: attributes, ports, behavior components. The target
//! is a physical node, a behavior component or a logical component, by id
//! (else by name, when one element only has it). Attributes are set
//! (overriding what the element had), a port of the same name gets the
//! port's attributes, any other port is added; behavior components and
//! deployments are added to a node.

use crate::compiler::ast::{AttributeValue, BehaviorComponent, ComponentPort, LogicalComponent, Model, PhysicalNode, PhysicalPort};
use std::collections::HashMap;

/// One `refine` block: its target as written and its body, read as a node's.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Refinement {
    pub target: String,
    pub body: PhysicalNode,
}

fn merge_attributes(into: &mut HashMap<String, AttributeValue>, from: &HashMap<String, AttributeValue>) {
    for (k, v) in from {
        into.insert(k.clone(), v.clone());
    }
}

fn merge_component_ports(into: &mut Vec<ComponentPort>, from: &[ComponentPort]) {
    for port in from {
        match into.iter_mut().find(|p| p.name == port.name) {
            Some(existing) => {
                merge_attributes(&mut existing.attributes, &port.attributes);
                if !port.interface_type.is_empty() {
                    existing.interface_type = port.interface_type.clone();
                }
            }
            None => into.push(port.clone()),
        }
    }
}

fn merge_physical_ports(into: &mut Vec<PhysicalPort>, from: &[PhysicalPort]) {
    for port in from {
        match into.iter_mut().find(|p| p.name == port.name) {
            Some(existing) => merge_attributes(&mut existing.attributes, &port.attributes),
            None => into.push(port.clone()),
        }
    }
}

/// The body's ports, whichever kind they were read as: a port refined on a
/// node may be written either way, and matched by name in both lists.
fn refine_node(node: &mut PhysicalNode, body: &PhysicalNode) {
    merge_attributes(&mut node.attributes, &body.attributes);
    // A port written without a direction is read as a physical port: refine
    // the oriented port of that name when the node has one.
    for port in &body.ports {
        if let Some(existing) = node.component_ports.iter_mut().find(|p| p.name == port.name) {
            merge_attributes(&mut existing.attributes, &port.attributes);
        } else {
            merge_physical_ports(&mut node.ports, std::slice::from_ref(port));
        }
    }
    merge_component_ports(&mut node.component_ports, &body.component_ports);
    node.behavior_components.extend(body.behavior_components.iter().cloned());
    node.deployments.extend(body.deployments.iter().cloned());
}

fn refine_behavior(component: &mut BehaviorComponent, body: &PhysicalNode) {
    merge_attributes(&mut component.attributes, &body.attributes);
    let as_component: Vec<ComponentPort> = body
        .ports
        .iter()
        .map(|p| ComponentPort { id: String::new(), name: p.name.clone(), direction: crate::compiler::ast::PortDirection::InOut, interface_type: String::new(), attributes: p.attributes.clone(), ports: Vec::new() })
        .chain(body.component_ports.iter().cloned())
        .collect();
    merge_component_ports(&mut component.ports, &as_component);
}

fn refine_logical(component: &mut LogicalComponent, body: &PhysicalNode) {
    merge_attributes(&mut component.attributes, &body.attributes);
    merge_component_ports(&mut component.ports, &body.component_ports);
}

/// Where a target is: (kind, path of indices).
enum Found {
    Node(usize, usize),
    Behavior(usize, usize, usize),
    Logical(usize, Vec<usize>),
}

fn find_logical(components: &[LogicalComponent], wanted: &dyn Fn(&str, &str) -> bool, path: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
    for (k, c) in components.iter().enumerate() {
        path.push(k);
        if wanted(&c.id, &c.name) {
            out.push(path.clone());
        }
        find_logical(&c.sub_components, wanted, path, out);
        path.pop();
    }
}

fn locate(model: &Model, target: &str) -> Vec<Found> {
    let mut found = Vec::new();
    for by_id in [true, false] {
        let wanted = |id: &str, name: &str| if by_id { id == target } else { name == target };
        for (a, pa) in model.physical_architecture.iter().enumerate() {
            for (n, node) in pa.nodes.iter().enumerate() {
                if wanted(&node.id, &node.name) {
                    found.push(Found::Node(a, n));
                }
                for (b, bc) in node.behavior_components.iter().enumerate() {
                    if wanted(&bc.id, &bc.name) {
                        found.push(Found::Behavior(a, n, b));
                    }
                }
            }
        }
        for (a, la) in model.logical_architecture.iter().enumerate() {
            let mut paths = Vec::new();
            find_logical(&la.components, &wanted, &mut Vec::new(), &mut paths);
            found.extend(paths.into_iter().map(|p| Found::Logical(a, p)));
        }
        if !found.is_empty() {
            break;
        }
    }
    found
}

fn logical_at<'m>(components: &'m mut [LogicalComponent], path: &[usize]) -> &'m mut LogicalComponent {
    let c = &mut components[path[0]];
    if path.len() == 1 {
        c
    } else {
        logical_at(&mut c.sub_components, &path[1..])
    }
}

/// Apply every `refine` of the model, in order. An error names a target
/// that is not found, or a name that several elements have.
pub fn apply(model: &mut Model) -> Result<(), String> {
    for r in std::mem::take(&mut model.refinements) {
        let found = locate(model, &r.target);
        let place = match found.len() {
            0 => return Err(format!("refine \"{}\": no physical node, behavior component or logical component has this id or name", r.target)),
            1 => found.into_iter().next().expect("one"),
            n => return Err(format!("refine \"{}\": {n} elements have this name; refine one by its id", r.target)),
        };
        match place {
            Found::Node(a, n) => refine_node(&mut model.physical_architecture[a].nodes[n], &r.body),
            Found::Behavior(a, n, b) => refine_behavior(&mut model.physical_architecture[a].nodes[n].behavior_components[b], &r.body),
            Found::Logical(a, path) => refine_logical(logical_at(&mut model.logical_architecture[a].components, &path), &r.body),
        }
    }
    Ok(())
}
