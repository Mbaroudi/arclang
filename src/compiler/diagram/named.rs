//! Diagrams the author declared: `diagram "Name" { view: lab shows: [...] }`.
//!
//! A view draws a whole level of the model. A declared diagram is cut from
//! that view: the elements it names, what holds them, and the exchanges
//! between what is drawn, as a Capella diagram shows what was put on it.
//! With `context: true`, the elements an exchange ties to a named one are
//! drawn too, as a contextual diagram.

use super::{Diagram, Node, ViewKind};
use crate::compiler::ast::{DiagramDecl, Model};
use std::collections::HashSet;

/// One (diagram, diagnostics) per declaration, in declared order. A
/// declaration that names no view, or a view the model does not draw,
/// yields diagnostics and no diagram.
pub(super) fn build(model: &Model, views: &[Diagram]) -> Vec<(Option<Diagram>, Vec<String>)> {
    model.diagrams.iter().map(|decl| one(decl, views)).collect()
}

fn one(decl: &DiagramDecl, views: &[Diagram]) -> (Option<Diagram>, Vec<String>) {
    let id = format!("{}:{}", decl.view, decl.name);
    let scope = format!("[{id}] ");
    let Some(kind) = ViewKind::parse(&decl.view) else {
        return (
            None,
            vec![format!("{scope}'{}' is not a view: use one of oab, sab, lab, pab, cap, cdb, pbs", decl.view)],
        );
    };
    let Some(source) = views.iter().find(|view| view.kind == kind && view.id == kind.id()) else {
        return (None, vec![format!("{scope}the model draws no {} view to cut this diagram from", kind.label())]);
    };

    let mut diagnostics = Vec::new();
    let mut wanted: HashSet<String> = HashSet::new();
    for reference in &decl.shows {
        let found = matching(&source.nodes, reference);
        if found.is_empty() {
            diagnostics.push(format!("{scope}'{reference}' is not drawn in the {} view", kind.label()));
        }
        wanted.extend(found);
    }
    if decl.context {
        let neighbours: Vec<String> = source
            .edges
            .iter()
            .filter_map(|edge| match (wanted.contains(&edge.source), wanted.contains(&edge.target)) {
                (true, false) => Some(edge.target.clone()),
                (false, true) => Some(edge.source.clone()),
                _ => None,
            })
            .collect();
        wanted.extend(neighbours);
    }
    if wanted.is_empty() {
        diagnostics.push(format!("{scope}nothing to draw: `shows` names no drawn element"));
        return (None, diagnostics);
    }

    let nodes: Vec<Node> = source.nodes.iter().filter_map(|node| cut(node, &wanted)).collect();
    let mut present: HashSet<String> = HashSet::new();
    for node in &nodes {
        ids_of(node, &mut present);
    }
    let edges = source
        .edges
        .iter()
        .filter(|edge| present.contains(&edge.source) && present.contains(&edge.target))
        .cloned()
        .collect();
    let chains = source
        .chains
        .iter()
        .filter(|chain| chain.nodes.iter().all(|node| present.contains(node)))
        .cloned()
        .collect();
    let diagram = Diagram {
        id,
        kind,
        title: decl.name.clone(),
        nodes,
        edges,
        chains,
    };
    (Some(diagram), diagnostics)
}

/// Ids of the nodes answering to `reference`, by id then by name, at any depth.
fn matching(nodes: &[Node], reference: &str) -> Vec<String> {
    let mut found = Vec::new();
    for node in nodes {
        if node.id == reference || node.name == reference {
            found.push(node.id.clone());
        }
        found.extend(matching(&node.children, reference));
    }
    found
}

/// The node with only what leads to a wanted element: a wanted node whole,
/// a container holding one with just that branch, nothing otherwise.
fn cut(node: &Node, wanted: &HashSet<String>) -> Option<Node> {
    if wanted.contains(&node.id) {
        return Some(node.clone());
    }
    let children: Vec<Node> = node.children.iter().filter_map(|child| cut(child, wanted)).collect();
    if children.is_empty() {
        return None;
    }
    let mut kept = node.clone();
    kept.children = children;
    Some(kept)
}

fn ids_of(node: &Node, into: &mut HashSet<String>) {
    into.insert(node.id.clone());
    for port in &node.ports {
        into.insert(port.id.clone());
    }
    for child in &node.children {
        ids_of(child, into);
    }
}
