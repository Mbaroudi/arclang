//! Requirements: the requirements of the model, how they derive from and
//! refine one another, the elements that satisfy them and the test cases
//! that verify them (SysML's requirement diagram, Capella's requirements
//! viewpoint).
//!
//! A requirement is a box holding its id and its text; `parent:` is drawn
//! as containment. The traces between two requirements (`derives`,
//! `refines`) and from an element to a requirement (`satisfies`,
//! `implements`) are drawn like every trace a view can draw both ends of
//! (`links::draw_traces`); a test case's `verifies:` is a `verifies` line
//! from the case to each requirement.

use super::builder::identity;
use super::elements::element_index;
use super::{Diagram, Edge, EdgeKind, Node, NodeKind, ViewKind};
use crate::compiler::ast::{AttributeValue, Model, Requirement, TestCase};
use crate::compiler::identity::element_uuid;
use crate::compiler::semantic::SemanticModel;
use std::collections::{BTreeMap, HashMap, HashSet};

/// Characters of a requirement's text on one line of its box.
const TEXT_WIDTH: usize = 40;
/// Lines of text a box shows; the rest is in its properties.
const TEXT_LINES: usize = 4;

/// Trace types from an element to the requirement it answers.
const SATISFYING: [&str; 2] = ["satisfies", "implements"];

pub(super) fn build(model: &Model) -> Option<(Diagram, Vec<String>)> {
    let requirements: Vec<&Requirement> = model.system_analysis.iter().flat_map(|sa| &sa.requirements).collect();
    if requirements.is_empty() {
        return None;
    }
    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut diagnostics: Vec<String> = Vec::new();
    // Every way a requirement is written (id, name, title), to its node id.
    let mut written: HashMap<&str, &str> = HashMap::new();
    for requirement in &requirements {
        let id = identity(&requirement.attributes, &requirement.id);
        if nodes.iter().any(|n| n.id == id) {
            diagnostics.push(format!("[req] two requirements share the id '{id}' — give one its own id"));
            continue;
        }
        written.insert(id, id);
        written.entry(&requirement.id).or_insert(id);
        for key in ["name", "title"] {
            if let Some(name) = requirement.attributes.get(key).and_then(|v| v.as_string()) {
                written.entry(name).or_insert(id);
            }
        }
        nodes.push(requirement_node(requirement, id, title_of(requirement)));
    }
    for requirement in &requirements {
        let id = identity(&requirement.attributes, &requirement.id);
        let Some(parent) = requirement.attributes.get("parent").and_then(|v| v.as_string()) else { continue };
        match written.get(parent) {
            Some(parent) => edges.push(edge(EdgeKind::Containment, "ct", parent, id, "", BTreeMap::new())),
            None => diagnostics.push(format!("[req] '{id}': parent '{parent}' is not a requirement")),
        }
    }
    for case in &model.test_cases {
        let id = if case.id.is_empty() { case.name.as_str() } else { case.id.as_str() };
        let targets: Vec<&str> = case.verifies.iter().filter_map(|r| written.get(r.as_str()).copied()).collect();
        if targets.is_empty() || nodes.iter().any(|n| n.id == id) {
            continue;
        }
        nodes.push(case_node(case, id));
        for target in targets {
            let properties = BTreeMap::from([("trace".to_string(), "verifies".to_string())]);
            edges.push(edge(EdgeKind::Trace, "vfy", id, target, "verifies", properties));
        }
    }
    // The elements a trace says satisfy a requirement, once each.
    let index = element_index(model);
    let mut added: HashSet<String> = nodes.iter().map(|n| n.id.clone()).collect();
    for trace in &model.traces {
        if !SATISFYING.contains(&trace.trace_type.as_str()) || !written.contains_key(trace.to.as_str()) {
            continue;
        }
        let Some(element) = index.get(trace.from.as_str()) else { continue };
        if !added.insert(element.id.to_string()) {
            continue;
        }
        nodes.push(Node {
            id: element.id.to_string(),
            uuid: element_uuid("element", element.id),
            name: element.name.to_string(),
            kind: element.kind,
            ports: Vec::new(),
            children: Vec::new(),
            realizes: Vec::new(),
            compartment: Vec::new(),
            properties: BTreeMap::new(),
        });
    }
    let diagram = Diagram {
        id: ViewKind::Req.id().to_string(),
        kind: ViewKind::Req,
        title: ViewKind::Req.label().to_string(),
        nodes,
        edges,
        chains: Vec::new(),
        fragments: Vec::new(),
        invariants: Vec::new(),
    };
    Some((diagram, diagnostics))
}

/// The name a requirement is shown by: its `title`, its `name`, else the name it is declared with.
fn title_of(requirement: &Requirement) -> &str {
    ["title", "name"]
        .iter()
        .find_map(|key| requirement.attributes.get(*key).and_then(|v| v.as_string()))
        .unwrap_or(&requirement.id)
}

fn requirement_node(requirement: &Requirement, id: &str, title: &str) -> Node {
    let mut compartment = vec![format!("id: {id}")];
    let text = ["text", "description"].iter().find_map(|key| requirement.attributes.get(*key).and_then(|v| v.as_string()));
    if let Some(text) = text {
        let mut lines = wrapped(text, TEXT_WIDTH);
        if lines.len() > TEXT_LINES {
            lines.truncate(TEXT_LINES);
            if let Some(last) = lines.last_mut() {
                last.push('…');
            }
        }
        compartment.extend(lines);
    }
    Node {
        id: id.to_string(),
        uuid: element_uuid("element", id),
        name: title.to_string(),
        kind: NodeKind::Requirement,
        ports: Vec::new(),
        children: Vec::new(),
        realizes: Vec::new(),
        compartment,
        properties: properties(&requirement.attributes),
    }
}

fn case_node(case: &TestCase, id: &str) -> Node {
    let mut compartment = vec![format!("method: {}", case.method)];
    if case.attributes.contains_key("simulate") {
        compartment.push("by simulation".to_string());
    }
    let mut node_properties = properties(&case.attributes);
    node_properties.insert("method".to_string(), case.method.clone());
    Node {
        id: id.to_string(),
        uuid: element_uuid("element", id),
        name: case.name.clone(),
        kind: NodeKind::TestCase,
        ports: Vec::new(),
        children: Vec::new(),
        realizes: Vec::new(),
        compartment,
        properties: node_properties,
    }
}

/// Every attribute as a display property, except the id and the fingerprints of a review.
fn properties(attributes: &HashMap<String, AttributeValue>) -> BTreeMap<String, String> {
    attributes
        .iter()
        .filter(|(key, _)| !matches!(key.as_str(), "id" | "reviewed"))
        .map(|(key, value)| (key.clone(), value.display()))
        .collect()
}

fn edge(kind: EdgeKind, prefix: &str, source: &str, target: &str, label: &str, properties: BTreeMap<String, String>) -> Edge {
    Edge {
        id: format!("{prefix}:{source} {target}"),
        kind,
        source: source.to_string(),
        source_port: None,
        target: target.to_string(),
        target_port: None,
        label: label.to_string(),
        exchange_item: None,
        properties,
    }
}

/// Words of a text on lines of about `width` characters.
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

/// What the compiler knows of the requirements view once the model is
/// checked: the verdict of each test case, of each verification line and of
/// each requirement (`pass`, `fail`, `inconclusive`, `pending`, or
/// `unverified` for a requirement no case verifies), and the traces and
/// verifications that are suspect (`suspect: true`, with the line `check`
/// gives).
pub fn judge(set: &mut super::DiagramSet, model: &Model, semantic: &SemanticModel) {
    use crate::compiler::verification::{verdict, verdict_for, Verdict};
    let Some(diagram) = set.diagrams.iter_mut().find(|d| d.kind == ViewKind::Req) else { return };
    let word = |v: &Verdict| match v {
        Verdict::Pass => "pass",
        Verdict::Fail(_) => "fail",
        Verdict::Inconclusive(_) => "inconclusive",
        Verdict::Pending => "pending",
    };
    let case_of = |id: &str| model.test_cases.iter().find(|c| c.id == id || (c.id.is_empty() && c.name == id));
    let mut by_requirement: HashMap<String, Vec<&'static str>> = HashMap::new();
    for edge in diagram.edges.iter_mut().filter(|e| e.id.starts_with("vfy:")) {
        let Some(case) = case_of(&edge.source) else { continue };
        // The reference as the case writes it: the verdict of an expectation naming its requirement is kept under it.
        let written = case.verifies.iter().find(|r| written_for(model, r) == edge.target).cloned().unwrap_or_else(|| edge.target.clone());
        let verdict = word(&verdict_for(case, &written, semantic));
        edge.properties.insert("verdict".to_string(), verdict.to_string());
        by_requirement.entry(edge.target.clone()).or_default().push(verdict);
    }
    for node in diagram.nodes.iter_mut() {
        let verdict = match node.kind {
            NodeKind::TestCase => match case_of(&node.id) {
                Some(case) => word(&verdict(case, semantic)),
                None => continue,
            },
            NodeKind::Requirement => {
                let verdicts = by_requirement.get(&node.id).map(Vec::as_slice).unwrap_or_default();
                if verdicts.is_empty() {
                    "unverified"
                } else if verdicts.contains(&"fail") {
                    "fail"
                } else if verdicts.iter().all(|v| *v == "pass") {
                    "pass"
                } else if verdicts.contains(&"inconclusive") {
                    "inconclusive"
                } else {
                    "pending"
                }
            }
            _ => continue,
        };
        node.properties.insert("verdict".to_string(), verdict.to_string());
    }
    if !crate::compiler::suspect::watches(model) {
        return;
    }
    let graph = crate::compiler::elements::build(model, semantic);
    let Ok(suspects) = crate::compiler::suspect::suspects(model, &graph) else { return };
    let node_of = |written: &str| -> Option<String> {
        let element = crate::compiler::suspect::end(&graph, written)?;
        diagram_ids(&diagram.nodes, element.uuid.as_str())
    };
    let marks: Vec<(String, String, String, String)> = suspects
        .iter()
        .filter_map(|s| Some((node_of(&s.from)?, s.trace_type.clone(), node_of(&s.to)?, s.line())))
        .collect();
    for edge in diagram.edges.iter_mut() {
        if let Some((_, _, _, line)) = marks.iter().find(|(from, kind, to, _)| *from == edge.source && *kind == edge.label && *to == edge.target) {
            edge.properties.insert("suspect".to_string(), "true".to_string());
            edge.properties.insert("suspect_reason".to_string(), line.clone());
        }
    }
}

/// The id of the node standing for the element of this uuid.
fn diagram_ids(nodes: &[Node], uuid: &str) -> Option<String> {
    nodes.iter().find(|n| n.uuid == uuid).map(|n| n.id.clone())
}

/// The node id a requirement reference resolves to: the id of the requirement it names.
fn written_for(model: &Model, reference: &str) -> String {
    model
        .system_analysis
        .iter()
        .flat_map(|sa| &sa.requirements)
        .find(|r| {
            r.id == reference
                || identity(&r.attributes, &r.id) == reference
                || ["name", "title"].iter().any(|k| r.attributes.get(*k).and_then(|v| v.as_string()) == Some(reference))
        })
        .map(|r| identity(&r.attributes, &r.id).to_string())
        .unwrap_or_else(|| reference.to_string())
}
