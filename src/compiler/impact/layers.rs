//! What ties the layers of a model together, as one document.
//!
//! The views get it link by link (`diagram::layers`). A client that draws
//! the model itself, or walks it without drawing, asks for the whole: each
//! element with its layer, and every link that joins two layers or an
//! element to what justifies it (a realization, an allocation, a
//! deployment, a satisfaction, a verification), read from the end that
//! depends on the other, with the words of the way back.
//!
//! It is read from the impact graph: the links are those `arclang impact`
//! follows, and nothing is guessed.

use super::graph::{ImpactGraph, LinkKind};
use crate::compiler::tables::Context;
use serde_json::{json, Value};

/// The kinds of link that join layers, or an element to what justifies it.
pub const JOINING: [LinkKind; 5] = [LinkKind::Realization, LinkKind::Allocation, LinkKind::Deployment, LinkKind::Satisfies, LinkKind::Verifies];

/// The elements of a compiled model with their layer, and the links between them.
pub fn document(result: &crate::CompilationResult) -> Value {
    let context = Context::new(result, "ISO26262");
    let graph = ImpactGraph::build(&context);
    let elements: Vec<Value> = graph
        .nodes
        .iter()
        .filter(|node| node.element.is_some())
        .map(|node| json!({ "@id": node.uuid, "id": node.id, "name": node.name, "kind": node.kind, "layer": node.layer }))
        .collect();
    let mut links = Vec::new();
    for (from, edges) in graph.edges.iter().enumerate() {
        let source = graph.node(from);
        if source.element.is_none() {
            continue;
        }
        for edge in edges.iter().filter(|edge| !edge.downstream && JOINING.contains(&edge.kind)) {
            let target = graph.node(edge.to);
            if target.element.is_none() {
                continue;
            }
            let back = graph.edges[edge.to].iter().find(|other| other.to == from && other.kind == edge.kind).map(|other| other.label.clone());
            links.push(json!({
                "kind": edge.kind,
                "label": edge.label,
                "back": back,
                "source": { "@id": source.uuid },
                "target": { "@id": target.uuid },
                "sourceLayer": source.layer,
                "targetLayer": target.layer,
            }));
        }
    }
    json!({ "elements": elements, "links": links })
}
