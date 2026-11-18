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

/// Where the located elements of a model are: the GeoJSON `arclang export
/// -f geojson` writes (a point for each element with a `location`, a line
/// for each physical link, along its route or straight between its located
/// ends), with the notes on what could not be drawn. Elements without a
/// location are not in it: a map never guesses where something is.
pub fn space(result: &crate::CompilationResult) -> Value {
    let (text, notes) = crate::compiler::spatial::geojson::export(&result.ast);
    json!({ "collection": serde_json::from_str::<Value>(&text).unwrap_or(Value::Null), "notes": notes })
}

/// The C a model's state machines become, and where each machine is in it.
///
/// What `arclang export -f c` generates for the behavior of the model: the
/// source and the header of the machines, with for each machine (an element
/// of the model, by its identity) the lines of its own code, the lines of its
/// step function, and the lines of each transition. The generated text is
/// served as it is written; the lines are those of that text. `null` when the
/// model has no machine to generate.
pub fn code(result: &crate::CompilationResult) -> Value {
    let Ok(plan) = crate::compiler::embedded::plan::plan(&result.ast) else { return Value::Null };
    let Ok((files, notes)) = crate::compiler::embedded::export(&result.ast) else { return Value::Null };
    let source_name = format!("{}.c", plan.ident);
    let header_name = format!("{}.h", plan.ident);
    let text_of = |name: &str| files.iter().find(|(file, _)| file == name).map(|(_, text)| text.as_str()).unwrap_or("");
    let source = text_of(&source_name);
    let lines: Vec<&str> = source.lines().collect();
    let context = Context::new(result, "ISO26262");

    // A machine's own code runs from its marker to the next marker, or to the first function of the model.
    let marker = |name: &str| format!("/* ---- state_machine \"{}\" ---- */", crate::compiler::embedded::cgen::comment(name));
    let starts: Vec<(usize, &str)> = plan.machines.iter().filter_map(|m| lines.iter().position(|l| *l == marker(&m.name)).map(|at| (at, m.name.as_str()))).collect();
    let public_start = lines.iter().position(|l| l.starts_with(&format!("void {}_init(", plan.ident))).unwrap_or(lines.len());
    let function = |name: &str| -> Option<Value> {
        let start = lines.iter().position(|l| l.starts_with(&format!("void {name}(")))?;
        let end = lines[start..].iter().position(|l| *l == "}").map(|at| start + at)?;
        Some(json!({ "file": source_name, "from": start + 1, "to": end + 1 }))
    };
    let machines: Vec<Value> = plan
        .machines
        .iter()
        .filter_map(|m| {
            let &(from, _) = starts.iter().find(|(_, name)| *name == m.name)?;
            let next = starts.iter().map(|(at, _)| *at).filter(|at| *at > from).min().unwrap_or(public_start);
            // The section ends with its last line of code, not with the blank line before the next.
            let mut to = next;
            while to > from && lines[to - 1].trim().is_empty() {
                to -= 1;
            }
            let transitions: Vec<Value> = (from..to)
                .filter(|&at| lines[at].starts_with("/* transition "))
                .map(|at| {
                    let end = lines[at..to].iter().position(|l| *l == "}").map(|rest| at + rest).unwrap_or(at);
                    json!({ "label": lines[at].trim_start_matches("/* ").trim_end_matches(" */"), "from": at + 1, "to": end + 1 })
                })
                .collect();
            let uuid = context.graph.elements.iter().find(|e| e.kind == "StateMachine" && (e.id == m.id || e.name == m.name)).map(|e| e.uuid.clone());
            Some(json!({
                "@id": uuid,
                "name": m.name,
                "section": { "file": source_name, "from": from + 1, "to": to },
                "step": function(&format!("{}_{}_step", plan.ident, m.ident)),
                "transitions": transitions,
            }))
        })
        .collect();
    let others: Vec<&str> = files.iter().map(|(name, _)| name.as_str()).filter(|name| *name != source_name && *name != header_name).collect();
    json!({
        "files": [
            { "name": source_name, "language": "c", "text": source },
            { "name": header_name, "language": "c", "text": text_of(&header_name) },
        ],
        "other_files": others,
        "machines": machines,
        "notes": notes,
    })
}
