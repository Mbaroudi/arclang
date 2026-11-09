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
//!
//! A requirement's box also lists what it is about (`subject:`), the
//! concerns it frames and the constraints it requires and assumes; each
//! concern is a box of its own with a `frames` line to it, the subject a box
//! with a `subject` line to it and the requirements about it in its compartment.

use super::builder::identity;
use super::elements::element_index;
use super::{Diagram, Edge, EdgeKind, Node, NodeKind, ViewKind};
use crate::compiler::ast::{AttributeValue, Concern, Model, Requirement, TestCase};
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
    // A requirement derived from, or refining, another: the thread from the
    // stakeholders' needs down to the technical requirements.
    for trace in model.traces.iter().filter(|t| matches!(t.trace_type.as_str(), "derives" | "refines")) {
        let (Some(from), Some(to)) = (written.get(trace.from.as_str()), written.get(trace.to.as_str())) else { continue };
        let properties = BTreeMap::from([("trace".to_string(), trace.trace_type.clone())]);
        edges.push(edge(EdgeKind::Trace, "drv", from, to, &trace.trace_type, properties));
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
    let concerns: Vec<&Concern> = model.system_analysis.iter().flat_map(|sa| &sa.concerns).collect();
    let mut concern_ids: HashMap<&str, &str> = HashMap::new();
    for concern in &concerns {
        let id = identity(&concern.attributes, &concern.id);
        concern_ids.insert(id, id);
        concern_ids.entry(&concern.id).or_insert(id);
        if added.insert(id.to_string()) {
            nodes.push(concern_node(concern, id));
        }
    }
    // The subject of each requirement, and the requirements about it.
    let mut about: Vec<(String, Vec<String>)> = Vec::new();
    for requirement in &requirements {
        let id = identity(&requirement.attributes, &requirement.id);
        if let Some(AttributeValue::List(framed)) = requirement.attributes.get("frames") {
            for concern in framed.iter().filter_map(|c| c.as_string()) {
                match concern_ids.get(concern) {
                    Some(target) => {
                        let properties = BTreeMap::from([("trace".to_string(), "frames".to_string())]);
                        edges.push(edge(EdgeKind::Trace, "frm", id, target, "frames", properties));
                    }
                    None => diagnostics.push(format!("[req] '{id}': frames '{concern}', which is not a concern")),
                }
            }
        }
        let Some(subject) = requirement.attributes.get("subject").and_then(|v| v.as_string()) else { continue };
        let Some(element) = index.get(subject) else { continue };
        if added.insert(element.id.to_string()) {
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
        let properties = BTreeMap::from([("trace".to_string(), "subject".to_string())]);
        edges.push(edge(EdgeKind::Trace, "sbj", id, element.id, "subject", properties));
        match about.iter_mut().find(|(subject, _)| subject == element.id) {
            Some((_, ids)) => ids.push(id.to_string()),
            None => about.push((element.id.to_string(), vec![id.to_string()])),
        }
    }
    for (subject, ids) in about {
        if let Some(node) = nodes.iter_mut().find(|n| n.id == subject) {
            node.compartment.push("requirements".to_string());
            node.compartment.extend(ids);
        }
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

/// The requirements views: one per requirements block when there are
/// several (`req:Stakeholder Requirements`...), each with its requirements,
/// what satisfies and verifies them, the needs they frame, their subjects
/// and the requirements of the other blocks they derive from or into;
/// then every requirement at once (`req`).
pub(super) fn diagrams(model: &Model) -> Vec<(Diagram, Vec<String>)> {
    let Some((all, diagnostics)) = build(model) else { return Vec::new() };
    let blocks: Vec<(&str, Vec<&str>)> = model
        .system_analysis
        .iter()
        .filter(|sa| !sa.requirements.is_empty())
        .map(|sa| (sa.name.as_str(), sa.requirements.iter().map(|r| identity(&r.attributes, &r.id)).collect()))
        .collect();
    let mut out = Vec::new();
    if blocks.len() > 1 {
        for (name, ids) in &blocks {
            let own: HashSet<&str> = ids.iter().copied().collect();
            let mut kept: HashSet<String> = own.iter().map(|s| s.to_string()).collect();
            for edge in &all.edges {
                if own.contains(edge.source.as_str()) {
                    kept.insert(edge.target.clone());
                }
                if own.contains(edge.target.as_str()) {
                    kept.insert(edge.source.clone());
                }
            }
            let nodes: Vec<Node> = all.nodes.iter().filter(|n| kept.contains(&n.id)).cloned().collect();
            let edges: Vec<Edge> = all.edges.iter().filter(|e| kept.contains(&e.source) && kept.contains(&e.target) && (own.contains(e.source.as_str()) || own.contains(e.target.as_str()))).cloned().collect();
            let view = Diagram { id: format!("{}:{name}", ViewKind::Req.id()), title: name.to_string(), nodes, edges, ..all.clone() };
            out.push((view, Vec::new()));
        }
    }
    out.push((all, diagnostics));
    out
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
    if let Some(subject) = requirement.attributes.get("subject").and_then(|v| v.as_string()) {
        compartment.push(format!("subject: {subject}"));
    }
    for key in ["frames", "require", "assume"] {
        let items: Vec<String> = match requirement.attributes.get(key) {
            Some(AttributeValue::List(items)) => items.iter().map(|i| i.as_string().map(str::to_string).unwrap_or_else(|| i.display())).collect(),
            Some(AttributeValue::String(one)) => vec![one.clone()],
            _ => Vec::new(),
        };
        if !items.is_empty() {
            compartment.push(key.to_string());
            compartment.extend(items);
        }
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

fn concern_node(concern: &Concern, id: &str) -> Node {
    let mut compartment = vec![format!("id: {id}")];
    if let Some(AttributeValue::List(stakeholders)) = concern.attributes.get("stakeholders") {
        let names: Vec<String> = stakeholders.iter().map(|s| s.as_string().map(str::to_string).unwrap_or_else(|| s.display())).collect();
        if !names.is_empty() {
            compartment.push(format!("stakeholders: {}", names.join(", ")));
        }
    }
    if let Some(text) = concern.attributes.get("description").and_then(|v| v.as_string()) {
        compartment.extend(wrapped(text, TEXT_WIDTH).into_iter().take(TEXT_LINES));
    }
    let name = concern.attributes.get("name").and_then(|v| v.as_string()).unwrap_or(&concern.id);
    Node {
        id: id.to_string(),
        uuid: element_uuid("element", id),
        name: name.to_string(),
        kind: NodeKind::Concern,
        ports: Vec::new(),
        children: Vec::new(),
        realizes: Vec::new(),
        compartment,
        properties: properties(&concern.attributes),
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
    for diagram in set.diagrams.iter_mut().filter(|d| d.kind == ViewKind::Req) {
        judge_one(diagram, model, semantic);
    }
}

/// The verdicts, and the suspect links, on one requirements view.
fn judge_one(diagram: &mut Diagram, model: &Model, semantic: &SemanticModel) {
    use crate::compiler::verification::{requirement_status, verdict, verdict_for};
    let case_of = |id: &str| model.test_cases.iter().find(|c| c.id == id || (c.id.is_empty() && c.name == id));
    for edge in diagram.edges.iter_mut().filter(|e| e.id.starts_with("vfy:")) {
        let Some(case) = case_of(&edge.source) else { continue };
        // The reference as the case writes it: the verdict of an expectation naming its requirement is kept under it.
        let written = case.verifies.iter().find(|r| written_for(model, r) == edge.target).cloned().unwrap_or_else(|| edge.target.clone());
        edge.properties.insert("verdict".to_string(), verdict_for(case, &written, semantic).word().to_string());
    }
    for node in diagram.nodes.iter_mut() {
        let verdict = match node.kind {
            NodeKind::TestCase => match case_of(&node.id) {
                Some(case) => verdict(case, semantic).word(),
                None => continue,
            },
            // Where it stands for the gate, whatever the view shows of its cases.
            NodeKind::Requirement => requirement_status(model, semantic, &node.id).standing().word(),
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

/// In the architecture views, an element some requirement is about lists
/// those requirements (SysML v2's requirements compartment): in its box
/// when it holds no other box, else in its `requirements` property.
pub(super) fn show_on_subjects(model: &Model, diagrams: &mut [Diagram]) {
    let index = element_index(model);
    let mut about: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for requirement in model.system_analysis.iter().flat_map(|sa| &sa.requirements) {
        let Some(subject) = requirement.attributes.get("subject").and_then(|v| v.as_string()) else { continue };
        let Some(element) = index.get(subject) else { continue };
        about.entry(element.id).or_default().push(identity(&requirement.attributes, &requirement.id));
    }
    if about.is_empty() {
        return;
    }
    fn walk(nodes: &mut [Node], about: &BTreeMap<&str, Vec<&str>>) {
        for node in nodes {
            if let Some(ids) = about.get(node.id.as_str()) {
                node.properties.insert("requirements".to_string(), ids.join(", "));
                if node.children.is_empty() && node.compartment.is_empty() {
                    node.compartment.push("requirements".to_string());
                    node.compartment.extend(ids.iter().map(|id| id.to_string()));
                }
            }
            walk(&mut node.children, about);
        }
    }
    for diagram in diagrams.iter_mut().filter(|d| matches!(d.kind, ViewKind::Oab | ViewKind::Sab | ViewKind::Lab | ViewKind::Pab)) {
        walk(&mut diagram.nodes, &about);
    }
}
