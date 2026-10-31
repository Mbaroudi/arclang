//! Links between elements across views, from the traces of the model.
//!
//! A view shows one Arcadia level. What ties the levels together — this
//! function realizes that activity, this component satisfies that
//! requirement — is written as traces. They are exported next to the
//! diagrams so a viewer can take the reader from an element to the ones it
//! is traced to, in whichever view draws them.

use super::{Diagram, Edge, EdgeKind, Node, NodeKind};
use crate::compiler::ast::Model;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// One trace, its ends resolved to the ids the diagrams use.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Link {
    /// The trace type as written: `realizes`, `satisfies`, `refines`...
    pub kind: String,
    /// Id of a drawn element, or the reference as written when no view
    /// draws it (a requirement) or several drawn elements answer to it.
    pub source: String,
    pub target: String,
}

/// Every trace of the model, in declared order. Two traces of one type
/// between the same elements are one link.
pub(super) fn cross_links(model: &Model, diagrams: &[Diagram]) -> Vec<Link> {
    let drawn = Drawn::of(diagrams, undrawn_names(model));
    let traces = model.traces.iter().chain(
        model
            .operational_analysis
            .iter()
            .flat_map(|oa| oa.traces.iter()),
    );
    let mut seen: HashSet<Link> = HashSet::new();
    traces
        .map(|trace| Link {
            kind: trace.trace_type.clone(),
            source: drawn.resolve(&trace.from),
            target: drawn.resolve(&trace.to),
        })
        .filter(|link| seen.insert(link.clone()))
        .collect()
}

/// Draw on each diagram the traces whose two ends it draws: a dashed line
/// labelled by the trace type, which the editor selects, reconnects and
/// deletes like any other line. A scenario draws lifelines, not elements:
/// it gets none.
pub(super) fn draw_traces(model: &Model, diagrams: &mut [Diagram]) {
    let traces: Vec<_> = model
        .traces
        .iter()
        .chain(model.operational_analysis.iter().flat_map(|oa| oa.traces.iter()))
        .collect();
    if traces.is_empty() {
        return;
    }
    let undrawn = undrawn_names(model);
    for diagram in diagrams.iter_mut() {
        let drawn = Drawn::of(std::slice::from_ref(diagram), undrawn.clone());
        let mut taken: HashSet<String> = diagram.edges.iter().map(|e| e.id.clone()).collect();
        let mut stack: Vec<&Node> = diagram.nodes.iter().collect();
        while let Some(node) = stack.pop() {
            stack.extend(node.children.iter());
            taken.insert(node.id.clone());
            taken.extend(node.ports.iter().map(|p| p.id.clone()));
        }
        let mut added: Vec<Edge> = Vec::new();
        // A view may draw a relationship itself (a test case's `verifies:`): a trace saying it again is not drawn twice.
        let mut seen: HashSet<(String, String, String)> = diagram
            .edges
            .iter()
            .filter(|e| e.kind == EdgeKind::Trace)
            .map(|e| (e.source.clone(), e.label.clone(), e.target.clone()))
            .collect();
        for trace in &traces {
            let (source, target) = (drawn.resolve(&trace.from), drawn.resolve(&trace.to));
            if !drawn.ids.contains(source.as_str()) || !drawn.ids.contains(target.as_str()) || source == target {
                continue;
            }
            if !seen.insert((source.clone(), trace.trace_type.clone(), target.clone())) {
                continue;
            }
            let base = format!("trc:{source} {} {target}", trace.trace_type);
            let id = (1..).map(|n| if n == 1 { base.clone() } else { format!("{base}#{n}") }).find(|id| !taken.contains(id)).expect("an unused id");
            taken.insert(id.clone());
            added.push(Edge {
                id,
                kind: EdgeKind::Trace,
                source,
                source_port: None,
                target,
                target_port: None,
                label: trace.trace_type.clone(),
                exchange_item: None,
                properties: BTreeMap::from([("trace".to_string(), trace.trace_type.clone())]),
            });
        }
        diagram.edges.extend(added);
    }
}

/// Words of a note on lines of about `width` characters.
fn wrapped(text: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(line) if line.chars().count() + 1 + word.chars().count() <= width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_string()),
        }
    }
    lines
}

/// Draw each note on every diagram that draws one of the elements it is
/// about: a box holding its text, tied to each of them by a dashed line.
pub(super) fn draw_notes(model: &Model, diagrams: &mut [Diagram]) {
    if model.notes.is_empty() {
        return;
    }
    let undrawn = undrawn_names(model);
    for diagram in diagrams.iter_mut() {
        let drawn = Drawn::of(std::slice::from_ref(diagram), undrawn.clone());
        let mut nodes: Vec<Node> = Vec::new();
        let mut edges: Vec<Edge> = Vec::new();
        for note in &model.notes {
            let id = crate::compiler::notes::id_of(note);
            let targets: Vec<String> = note.about.iter().map(|r| drawn.resolve(r)).filter(|t| drawn.ids.contains(t.as_str())).collect();
            if targets.is_empty() || drawn.ids.contains(id.as_str()) || nodes.iter().any(|n| n.id == id) {
                continue;
            }
            nodes.push(Node {
                id: id.clone(),
                uuid: crate::compiler::semantic::ElementInfo::new(id.clone(), note.name.clone(), "Note").uuid,
                name: String::new(),
                kind: NodeKind::Note,
                ports: Vec::new(),
                children: Vec::new(),
                realizes: Vec::new(),
                compartment: wrapped(&note.name, 36),
                properties: BTreeMap::from([("text".to_string(), note.name.clone())]),
            });
            for target in targets {
                edges.push(Edge {
                    id: format!("ann:{id} {target}"),
                    kind: EdgeKind::Annotation,
                    source: id.clone(),
                    source_port: None,
                    target,
                    target_port: None,
                    label: String::new(),
                    exchange_item: None,
                    properties: BTreeMap::new(),
                });
            }
        }
        diagram.nodes.extend(nodes);
        diagram.edges.extend(edges);
    }
}

/// The elements the diagrams draw, by id and by name.
struct Drawn<'a> {
    /// Ids and names of requirements, test cases and hazards: only the
    /// requirements view draws the first two.
    undrawn: HashSet<&'a str>,
    ids: HashSet<&'a str>,
    /// Ids of the requirements and test cases drawn.
    requirements: HashSet<&'a str>,
    ids_by_name: HashMap<&'a str, BTreeSet<&'a str>>,
}

impl<'a> Drawn<'a> {
    fn of(diagrams: &'a [Diagram], undrawn: HashSet<&'a str>) -> Self {
        let mut drawn = Drawn {
            undrawn,
            ids: HashSet::new(),
            requirements: HashSet::new(),
            ids_by_name: HashMap::new(),
        };
        let mut stack: Vec<&Node> = diagrams.iter().flat_map(|d| d.nodes.iter()).collect();
        while let Some(node) = stack.pop() {
            stack.extend(node.children.iter());
            if matches!(node.kind, NodeKind::Requirement | NodeKind::TestCase) {
                drawn.requirements.insert(&node.id);
            }
            if stands_for_an_element(node.kind) {
                drawn.ids.insert(&node.id);
                drawn
                    .ids_by_name
                    .entry(&node.name)
                    .or_default()
                    .insert(&node.id);
            }
        }
        drawn
    }

    /// The id a reference names: itself when it is an id, the element
    /// carrying that name when there is exactly one, else the reference
    /// unchanged. A name a requirement or test case also answers to is
    /// that requirement or case where it is drawn, else left as written:
    /// the trace may be about it.
    fn resolve(&self, reference: &str) -> String {
        if self.ids.contains(reference) {
            return reference.to_string();
        }
        if self.undrawn.contains(reference) {
            let drawn: Vec<&&str> = self.ids_by_name.get(reference).map(|ids| ids.iter().filter(|id| self.requirements.contains(**id)).collect()).unwrap_or_default();
            return match drawn.as_slice() {
                [single] => (**single).to_string(),
                _ => reference.to_string(),
            };
        }
        let named = self.ids_by_name.get(reference);
        match named.map(|ids| ids.iter().collect::<Vec<_>>()).as_deref() {
            Some([single]) => (**single).to_string(),
            _ => reference.to_string(),
        }
    }
}

/// Ids and names of the elements that have no diagram: requirements, test
/// cases and hazards.
fn undrawn_names(model: &Model) -> HashSet<&str> {
    let mut names: HashSet<&str> = HashSet::new();
    for requirement in model.system_analysis.iter().flat_map(|sa| &sa.requirements) {
        names.insert(&requirement.id);
        for key in ["id", "name", "title"] {
            names.extend(requirement.attributes.get(key).and_then(|v| v.as_string()));
        }
    }
    for case in &model.test_cases {
        names.insert(&case.id);
        names.insert(&case.name);
    }
    for hazard in model
        .safety_analysis
        .iter()
        .flat_map(|safety| &safety.hazards)
    {
        names.insert(&hazard.name);
        names.extend(hazard.attributes.get("id").and_then(|v| v.as_string()));
    }
    names
}

/// Nodes that are an element of the model, as opposed to a stand-in for one
/// (a lifeline, a component shown on the node it is deployed on) or a
/// notation mark.
fn stands_for_an_element(kind: NodeKind) -> bool {
    !matches!(
        kind,
        NodeKind::Lifeline | NodeKind::DeployedComponent | NodeKind::InitialState | NodeKind::Junction | NodeKind::ValueProperty
    )
}
