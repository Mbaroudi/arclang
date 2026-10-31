//! Parametric: each constraint of the model bound to the values it reads
//! (SysML's parametric diagram).
//!
//! A constraint is a box holding its expression, the value of each side and
//! its verdict; each attribute it reads is a value box inside the element
//! that holds it (`mass = 450 kg` in the battery), tied to the constraint by
//! a binding line. An aggregate (`sum("FC-AEB", latency)`) binds the value of
//! every member it adds up, the line labelled by the function.

use super::elements::element_index;
use super::{Diagram, Edge, EdgeKind, Node, NodeKind, ViewKind};
use crate::compiler::ast::Model;
use crate::compiler::constraint::{self, Expr, Scope};
use crate::compiler::identity::element_uuid;
use std::collections::BTreeMap;

/// A value a constraint reads: the element, the attribute, and the
/// function it goes through (`sum`), if any.
struct Read {
    element: String,
    attribute: String,
    through: Option<String>,
}

pub(super) fn build(model: &Model) -> Option<(Diagram, Vec<String>)> {
    if model.constraints.is_empty() {
        return None;
    }
    let scope = Scope::from_model(model);
    let index = element_index(model);
    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut diagnostics = Vec::new();
    for item in &model.constraints {
        let id = if item.id.is_empty() { item.name.clone() } else { item.id.clone() };
        if nodes.iter().any(|n| n.id == id) {
            diagnostics.push(format!("[par] two constraints share the id '{id}' — give one its own id"));
            continue;
        }
        nodes.push(constraint_node(&id, &item.name, &item.expression, &scope));
        let mut reads = Vec::new();
        reads_of(&item.expression, &scope, None, &mut reads);
        for read in reads {
            let Some(owner) = owner_node(&mut nodes, &read.element, &index, &scope) else { continue };
            let value_id = format!("{owner}.{}", read.attribute);
            let holder = nodes.iter_mut().find(|n| n.id == owner).expect("the owner was just added");
            if !holder.children.iter().any(|c| c.id == value_id) {
                holder.children.push(value_node(&value_id, &read, &scope));
            }
            let label = read.through.clone().unwrap_or_default();
            let edge_id = format!("bnd:{value_id} {id}");
            if !edges.iter().any(|e| e.id == edge_id) {
                edges.push(Edge {
                    id: edge_id,
                    kind: EdgeKind::Binding,
                    source: value_id,
                    source_port: None,
                    target: id.clone(),
                    target_port: None,
                    label,
                    exchange_item: None,
                    properties: BTreeMap::new(),
                });
            }
        }
    }
    let diagram = Diagram {
        id: ViewKind::Par.id().to_string(),
        kind: ViewKind::Par,
        title: ViewKind::Par.label().to_string(),
        nodes,
        edges,
        chains: Vec::new(),
        fragments: Vec::new(),
        invariants: Vec::new(),
    };
    Some((diagram, diagnostics))
}

/// The box of a constraint: its expression, the value of each side, its verdict.
fn constraint_node(id: &str, name: &str, expression: &Expr, scope: &Scope) -> Node {
    let mut compartment = vec![expression.to_string()];
    let mut properties = BTreeMap::from([("expression".to_string(), expression.to_string())]);
    match constraint::check(expression, scope) {
        Ok(outcome) => {
            compartment.push(format!("{} against {}", readable(&outcome.left.to_string()), readable(&outcome.right.to_string())));
            properties.insert("verdict".to_string(), if outcome.satisfied { "pass" } else { "fail" }.to_string());
            properties.insert("left".to_string(), readable(&outcome.left.to_string()));
            properties.insert("right".to_string(), readable(&outcome.right.to_string()));
        }
        Err(reason) => {
            compartment.push(format!("cannot be evaluated: {reason}"));
            properties.insert("verdict".to_string(), "inconclusive".to_string());
        }
    }
    Node {
        id: id.to_string(),
        uuid: element_uuid("element", id),
        name: name.to_string(),
        kind: NodeKind::Constraint,
        ports: Vec::new(),
        children: Vec::new(),
        realizes: Vec::new(),
        compartment,
        properties,
    }
}

fn value_node(id: &str, read: &Read, scope: &Scope) -> Node {
    let written = scope.written(&read.element, &read.attribute).unwrap_or_else(|| "?".to_string());
    Node {
        id: id.to_string(),
        uuid: element_uuid("value", id),
        name: read.attribute.clone(),
        kind: NodeKind::ValueProperty,
        ports: Vec::new(),
        children: Vec::new(),
        realizes: Vec::new(),
        compartment: vec![format!("= {written}")],
        properties: BTreeMap::from([("value".to_string(), written)]),
    }
}

/// The node of the element holding a value, added once; its id.
fn owner_node(nodes: &mut Vec<Node>, element: &str, index: &std::collections::HashMap<&str, super::elements::ElementRef<'_>>, scope: &Scope) -> Option<String> {
    let (id, name, kind) = match index.get(element) {
        Some(found) => (found.id.to_string(), found.name.to_string(), found.kind),
        None if scope.members(element).is_some() => (element.to_string(), element.to_string(), NodeKind::FunctionalChain),
        None if scope.knows(element) => (element.to_string(), element.to_string(), NodeKind::ConfigurationItem),
        None => return None,
    };
    if !nodes.iter().any(|n| n.id == id) {
        nodes.push(Node {
            id: id.clone(),
            uuid: element_uuid("element", &id),
            name,
            kind,
            ports: Vec::new(),
            children: Vec::new(),
            realizes: Vec::new(),
            compartment: Vec::new(),
            properties: BTreeMap::new(),
        });
    }
    Some(id)
}

/// The values an expression reads, in order.
fn reads_of(expr: &Expr, scope: &Scope, through: Option<&str>, out: &mut Vec<Read>) {
    match expr {
        Expr::Attr { element, attribute } => out.push(Read { element: element.clone(), attribute: attribute.clone(), through: through.map(str::to_string) }),
        Expr::Call { function, args } => {
            if let [Expr::Ref(collection), Expr::Ref(attribute)] = args.as_slice() {
                for member in scope.operand_members(collection).unwrap_or_default() {
                    if scope.attribute(member, attribute).is_ok() {
                        out.push(Read { element: member.to_string(), attribute: attribute.clone(), through: Some(function.clone()) });
                    }
                }
            }
        }
        Expr::Neg(inner) => reads_of(inner, scope, through, out),
        Expr::Binary { lhs, rhs, .. } => {
            reads_of(lhs, scope, through, out);
            reads_of(rhs, scope, through, out);
        }
        Expr::Number(_) | Expr::Quantity(_) | Expr::Ref(_) => {}
    }
}

/// A computed value in a unit a reader would say: `500000 W` reads `500 kW`.
fn readable(value: &str) -> String {
    const PREFIXED: [&str; 6] = ["W", "V", "A", "J", "Hz", "Wh"];
    let Some((number, unit)) = value.split_once(' ') else { return value.to_string() };
    let Ok(n) = number.parse::<f64>() else { return value.to_string() };
    if !PREFIXED.contains(&unit) {
        return value.to_string();
    }
    let (scaled, prefix) = match n.abs() {
        a if a >= 1e9 => (n / 1e9, "G"),
        a if a >= 1e6 => (n / 1e6, "M"),
        a if a >= 1e3 => (n / 1e3, "k"),
        _ => return value.to_string(),
    };
    let rounded = (scaled * 1e3).round() / 1e3;
    format!("{rounded} {prefix}{unit}")
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_large_value_reads_with_a_prefix() {
        assert_eq!(super::readable("500000 W"), "500 kW");
        assert_eq!(super::readable("1500000 Wh"), "1.5 MWh");
        assert_eq!(super::readable("50 ms"), "50 ms");
        assert_eq!(super::readable("300000 m"), "300000 m");
    }
}
