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

/// The points of variation of a model, its configurations, and for each
/// configuration what it leaves out.
///
/// A configuration is one product: it selects one variant of every
/// variation (`arclang::compiler::variability`). What it leaves out is the
/// variants it does not select, and everything those hold: a client that
/// shows a product dims exactly those elements. Elements that only link to a
/// left-out variant (a node that deploys it, a function it realizes) are not
/// in the list: only ownership is followed, nothing is guessed.
pub fn variants(result: &crate::CompilationResult) -> Value {
    let context = Context::new(result, "ISO26262");
    let by_written_id = |id: &str| context.graph.elements.iter().position(|element| element.id == id && element.kind == "LogicalComponent");
    let variations: Vec<Value> = crate::compiler::variability::variations(&result.ast)
        .iter()
        .map(|variation| {
            let held: Vec<Value> = variation
                .variants
                .iter()
                .map(|(id, name)| json!({ "id": id, "name": name, "@id": by_written_id(id).map(|index| context.element(index).uuid.clone()) }))
                .collect();
            let name = by_written_id(&variation.id).map(|index| context.element(index).name.clone()).unwrap_or_else(|| variation.id.clone());
            json!({ "id": variation.id, "name": name, "@id": by_written_id(&variation.id).map(|index| context.element(index).uuid.clone()), "variants": held })
        })
        .collect();
    let configurations: Vec<Value> = result
        .ast
        .configurations
        .iter()
        .map(|configuration| {
            let left_out: Vec<usize> = crate::compiler::variability::excluded_by(&result.ast, &configuration.id).iter().filter_map(|id| by_written_id(id)).collect();
            let excluded: Vec<&str> = (0..context.graph.elements.len())
                .filter(|&index| left_out.contains(&index) || context.owners(index).iter().any(|owner| left_out.contains(owner)))
                .map(|index| context.element(index).uuid.as_str())
                .collect();
            json!({ "id": configuration.id, "name": configuration.name, "selects": configuration.selects, "excluded": excluded })
        })
        .collect();
    json!({ "variations": variations, "configurations": configurations })
}
