//! What ties the layers together, given to the views whole.
//!
//! A view draws one Arcadia level, and the traces say only part of what
//! joins the levels: a function also `realizes:` another by an attribute, a
//! node `deploys` a component, a test case verifies a requirement. The
//! impact graph already holds all of it, typed and oriented. Here each drawn
//! element is given its layer and the links of the set are completed from
//! that graph, so that a viewer can go up and down the layers along the
//! same links `arclang impact` follows.
//!
//! Nothing is guessed: an end that several elements answer to keeps the
//! reference as written and gets no layer or identity.

use super::links::stands_for_an_element;
use super::{DiagramSet, Link, Node, NodeKind};
use crate::compiler::impact::graph::{layer_code, ImpactGraph};
use crate::compiler::impact::layers::JOINING;
use crate::compiler::metamodel::Layer;
use crate::compiler::tables::Context;
use std::collections::{HashMap, HashSet};

pub(super) fn complete(set: &mut DiagramSet, context: &Context) {
    let graph = ImpactGraph::build(context);
    let elements = Elements::of(context);

    let mut drawn: Drawn = Drawn::default();
    for diagram in &mut set.diagrams {
        for node in &mut diagram.nodes {
            mark(node, None, context, &elements, &mut drawn);
        }
    }

    for link in &mut set.links {
        let source = drawn.uuid_of.get(&link.source).cloned().or_else(|| elements.named(&link.source));
        let target = drawn.uuid_of.get(&link.target).cloned().or_else(|| elements.named(&link.target));
        link.source_layer = source.as_deref().and_then(|uuid| layer_of(context, uuid));
        link.target_layer = target.as_deref().and_then(|uuid| layer_of(context, uuid));
        if let (Some(from), Some(to)) = (source.as_deref().and_then(|u| graph.by_uuid(u)), target.as_deref().and_then(|u| graph.by_uuid(u))) {
            link.back = graph.edges[to].iter().find(|edge| edge.to == from && graph.edges[from].iter().any(|forward| forward.to == to && forward.kind == edge.kind && forward.label == link.kind)).map(|edge| edge.label.clone());
        }
        link.source_uuid = source;
        link.target_uuid = target;
    }

    let mut said: HashSet<(String, String, String)> = HashSet::new();
    for link in &set.links {
        said.insert((link.kind.clone(), link.source.clone(), link.target.clone()));
        if let (Some(source), Some(target)) = (&link.source_uuid, &link.target_uuid) {
            said.insert((link.kind.clone(), source.clone(), target.clone()));
        }
    }
    for (from, edges) in graph.edges.iter().enumerate() {
        let source = graph.node(from);
        let Some(source_element) = source.element else { continue };
        // Each link is held both ways: the way read from the end that depends on the other is the one kept.
        for edge in edges.iter().filter(|edge| !edge.downstream && JOINING.contains(&edge.kind)) {
            let target = graph.node(edge.to);
            let Some(target_element) = target.element else { continue };
            // A function inside the component performing it: the view draws that by nesting.
            if context.owners(source_element).contains(&target_element) || context.owners(target_element).contains(&source_element) {
                continue;
            }
            if !said.insert((edge.label.clone(), source.uuid.clone(), target.uuid.clone())) {
                continue;
            }
            let ends = (drawn.id_of.get(&source.uuid).unwrap_or(&source.id).clone(), drawn.id_of.get(&target.uuid).unwrap_or(&target.id).clone());
            if !said.insert((edge.label.clone(), ends.0.clone(), ends.1.clone())) {
                continue;
            }
            let back = graph.edges[edge.to].iter().find(|other| other.to == from && other.kind == edge.kind).map(|other| other.label.clone());
            set.links.push(Link {
                kind: edge.label.clone(),
                source: ends.0,
                target: ends.1,
                back,
                source_layer: layer_of(context, &source.uuid),
                target_layer: layer_of(context, &target.uuid),
                source_uuid: Some(source.uuid.clone()),
                target_uuid: Some(target.uuid.clone()),
            });
        }
    }
}

/// The drawn elements: the identity behind each id, and the id each
/// identity is first drawn under.
#[derive(Default)]
struct Drawn {
    uuid_of: HashMap<String, String>,
    id_of: HashMap<String, String>,
}

/// Give a node and what it holds their layer, and note where each is drawn.
fn mark(node: &mut Node, owner: Option<usize>, context: &Context, elements: &Elements, drawn: &mut Drawn) {
    // A lifeline is not an element, the participant it stands for is one.
    let element = if stands_for_an_element(node.kind) || node.kind == NodeKind::Lifeline { elements.drawn_as(node, owner, context) } else { None };
    if let Some(index) = element {
        let uuid = &context.element(index).uuid;
        node.layer = Some(layer_code(context.layer_of(index)));
        drawn.uuid_of.entry(node.id.clone()).or_insert_with(|| uuid.clone());
        // A link names an element by the id its own layer draws it under, not by a lifeline or a SysML v2 shape.
        if !matches!(node.kind, NodeKind::Lifeline | NodeKind::SysmlPart | NodeKind::SysmlAction | NodeKind::SysmlDefinition) {
            drawn.id_of.entry(uuid.clone()).or_insert_with(|| node.id.clone());
        }
    } else if node.kind == NodeKind::System {
        // The box of the system is the system analysis itself.
        node.layer = Some(layer_code(Layer::System));
    } else if node.kind == NodeKind::ValueProperty {
        // A value drawn on its own is a property of the element that holds it.
        node.layer = owner.map(|index| layer_code(context.layer_of(index)));
    }
    for child in &mut node.children {
        mark(child, element.or(owner), context, elements, drawn);
    }
}

/// Whether an element of this kind is drawn as a node of that kind: a
/// `LogicalFunction` as a `function`, a `PhysicalNode` as a `physical_node`.
fn drawn_as_kind(element_kind: &str, node_kind: &str) -> bool {
    let mut snake = String::new();
    for (at, letter) in element_kind.chars().enumerate() {
        if letter.is_ascii_uppercase() && at > 0 {
            snake.push('_');
        }
        snake.push(letter.to_ascii_lowercase());
    }
    snake == node_kind
        || snake.ends_with(&format!("_{node_kind}"))
        || node_kind.ends_with(&format!("_{snake}"))
        // A mode is a state said to be one; the product breakdown draws every EPBS kind as an item.
        || (element_kind == "State" && node_kind == "mode")
        || (element_kind.starts_with("Epbs") && node_kind == "configuration_item")
        // A SysML v2 view and a scenario draw an element of any kind under their own shape: its id says which.
        || matches!(node_kind, "sysml_part" | "sysml_action" | "sysml_definition" | "lifeline")
}

fn layer_of(context: &Context, uuid: &str) -> Option<&'static str> {
    context.index_of_uuid(uuid).map(|index| layer_code(context.layer_of(index)))
}

/// The elements of the model by what a trace or a view may call them.
struct Elements<'a> {
    by_id: HashMap<&'a str, Vec<usize>>,
    by_name: HashMap<&'a str, Vec<usize>>,
    context: &'a Context<'a>,
}

impl<'a> Elements<'a> {
    fn of(context: &'a Context<'a>) -> Self {
        let mut elements = Elements { by_id: HashMap::new(), by_name: HashMap::new(), context };
        for (index, element) in context.graph.elements.iter().enumerate() {
            elements.by_id.entry(&element.id).or_default().push(index);
            elements.by_name.entry(&element.name).or_default().push(index);
        }
        elements
    }

    /// The identity of the one element a reference names: by id, else by name.
    fn named(&self, reference: &str) -> Option<String> {
        for known in [&self.by_id, &self.by_name] {
            match known.get(reference).map(Vec::as_slice) {
                Some([one]) => return Some(self.context.element(*one).uuid.clone()),
                Some(_) => return None,
                None => {}
            }
        }
        None
    }

    /// The element a node draws. The views and the element graph give the
    /// same identity to an element that states its id; one that does not
    /// (a function named inside its component, a name two kinds share) is
    /// found by its id or name among the elements of the node's kind, under
    /// the element its container draws when several answer.
    fn drawn_as(&self, node: &Node, owner: Option<usize>, context: &Context) -> Option<usize> {
        let kind = serde_json::to_value(node.kind).ok()?.as_str()?.to_string();
        let of_kind = |index: &usize| drawn_as_kind(context.element(*index).kind, &kind);
        if let Some(index) = context.index_of_uuid(&node.uuid).filter(of_kind) {
            return Some(index);
        }
        let short = node.id.rsplit_once('.').map(|(_, last)| last);
        let mut candidates: Vec<usize> = Vec::new();
        for key in [Some(node.id.as_str()), short, Some(node.name.as_str())].into_iter().flatten() {
            for known in [&self.by_id, &self.by_name] {
                for index in known.get(key).into_iter().flatten().filter(|index| of_kind(index)) {
                    if !candidates.contains(index) {
                        candidates.push(*index);
                    }
                }
            }
        }
        if let (Some(owner), true) = (owner, candidates.len() > 1) {
            candidates.retain(|index| context.owners(*index).contains(&owner));
        }
        match candidates.as_slice() {
            [one] => Some(*one),
            _ => None,
        }
    }
}
