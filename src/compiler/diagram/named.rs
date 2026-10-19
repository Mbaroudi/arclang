//! Diagrams the author declared: `diagram "Name" { view: lab shows: [...] }`.
//!
//! A view draws a whole level of the model. A declared diagram is cut from
//! that view: the elements it names, what holds them, and the exchanges
//! between what is drawn, as a Capella diagram shows what was put on it.
//! With `context: true`, the elements an exchange ties to a named one are
//! drawn too, as a contextual diagram.

use super::{Diagram, Edge, Node, ViewKind};
use crate::compiler::ast::{DiagramDecl, Model};
use std::collections::HashSet;

/// One (diagram, diagnostics) per declaration, in declared order. A
/// declaration that names no view, or a view the model does not draw,
/// yields diagnostics and no diagram.
pub(super) fn build(model: &Model, views: &[Diagram]) -> Vec<(Option<Diagram>, Vec<String>)> {
    model.diagrams.iter().map(|decl| one(model, decl, views)).collect()
}

fn one(model: &Model, decl: &DiagramDecl, views: &[Diagram]) -> (Option<Diagram>, Vec<String>) {
    // `configuration: "CFG-1"`: the view of one product, without the
    // variants it does not select.
    let configuration = decl.attributes.get("configuration").and_then(|v| v.as_string());
    if let Some(name) = configuration {
        if !model.configurations.iter().any(|c| c.id == name || c.name == name) {
            return (None, vec![format!("[{}:{}] '{name}' is not a declared configuration", decl.view, decl.name)]);
        }
    }
    let excluded = configuration.map(|c| crate::compiler::variability::excluded_by(model, c)).unwrap_or_default();
    let (diagram, diagnostics) = cut_view(decl, views, configuration.is_some());
    (diagram.map(|d| without(d, &excluded)), diagnostics)
}

/// A diagram without the nodes `excluded` names (by id or name), nor the
/// edges that ended on them.
fn without(mut diagram: Diagram, excluded: &HashSet<String>) -> Diagram {
    if excluded.is_empty() {
        return diagram;
    }
    fn prune(nodes: Vec<Node>, excluded: &HashSet<String>) -> Vec<Node> {
        nodes
            .into_iter()
            .filter(|n| !excluded.contains(&n.id) && !excluded.contains(&n.name))
            .map(|mut n| {
                n.children = prune(std::mem::take(&mut n.children), excluded);
                n
            })
            .collect()
    }
    diagram.nodes = prune(std::mem::take(&mut diagram.nodes), excluded);
    let mut present = HashSet::new();
    for node in &diagram.nodes {
        ids_of(node, &mut present);
    }
    diagram.edges.retain(|e| present.contains(&e.source) && present.contains(&e.target));
    diagram
}

fn cut_view(decl: &DiagramDecl, views: &[Diagram], whole_when_empty: bool) -> (Option<Diagram>, Vec<String>) {
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
    // A configuration with nothing named shows its whole view.
    if whole_when_empty && decl.shows.is_empty() {
        wanted.extend(source.nodes.iter().map(|n| n.id.clone()));
    }
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
    let edges: Vec<Edge> = source
        .edges
        .iter()
        .filter(|edge| present.contains(&edge.source) && present.contains(&edge.target))
        .cloned()
        .collect();
    // A chain or path lights what the diagram shows of it, as Capella does:
    // its steps and links kept here, nothing it would have to invent.
    let kept_edges: HashSet<&str> = edges.iter().map(|edge| edge.id.as_str()).collect();
    let chains = source
        .chains
        .iter()
        .filter_map(|chain| {
            let mut chain = chain.clone();
            chain.nodes.retain(|node| present.contains(node));
            chain.edges.retain(|edge| kept_edges.contains(edge.as_str()));
            (!chain.nodes.is_empty() || !chain.edges.is_empty()).then_some(chain)
        })
        .collect();
    let diagram = Diagram {
        id,
        kind,
        title: decl.name.clone(),
        nodes,
        edges,
        chains,
        fragments: Vec::new(),
        invariants: Vec::new(),
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
