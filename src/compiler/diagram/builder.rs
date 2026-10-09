//! Shared construction state for one diagram: a node arena, the reference
//! index used to resolve exchange endpoints, and the diagnostics collected.

use super::{Chain, Diagram, Edge, EdgeKind, Node, NodeKind, Port, PortDirection, ViewKind};
use crate::compiler::ast::AttributeValue;
use crate::compiler::identity::element_uuid;
use std::collections::{BTreeMap, HashMap, HashSet};

/// Handle to a node in the arena.
pub(super) type NodeRef = usize;

struct Draft {
    node: Node,
    parent: Option<NodeRef>,
    /// Keys (id, name, qualified names) this node can be referenced by.
    keys: Vec<String>,
}

/// Which end of an exchange an endpoint is; decides between an input and an
/// output port that share a name.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Role {
    Source,
    Target,
}

/// What an exchange endpoint resolved to.
pub(super) struct Endpoint {
    pub node: NodeRef,
    pub port: Option<String>,
    /// The reference named the node itself (possibly qualified), not one of
    /// its ports.
    pub whole: bool,
}

/// Why a reference resolved to nothing.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Missing {
    Unknown,
    Ambiguous,
}

pub(super) struct Builder {
    view: ViewKind,
    /// Id of the diagram being built; prefixes its diagnostics.
    scope: String,
    drafts: Vec<Draft>,
    /// Reference key -> nodes answering to it (more than one = ambiguous).
    index: HashMap<String, Vec<NodeRef>>,
    /// Model identity -> first node drawn for it.
    node_by_element: HashMap<String, NodeRef>,
    /// Every node, port and edge id handed out: ids share one namespace,
    /// as a layout engine addresses all three by id.
    claimed: HashSet<String>,
    edges: Vec<Edge>,
    chains: Vec<Chain>,
    diagnostics: Vec<String>,
}

impl Builder {
    pub fn new(view: ViewKind) -> Self {
        Self::scoped(view, view.id().to_string())
    }

    /// A builder for one of several diagrams of a view (`msm:<machine>`).
    pub fn scoped(view: ViewKind, diagram_id: String) -> Self {
        Self {
            view,
            scope: diagram_id,
            drafts: Vec::new(),
            index: HashMap::new(),
            node_by_element: HashMap::new(),
            claimed: HashSet::new(),
            edges: Vec::new(),
            chains: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn report(&mut self, message: impl AsRef<str>) {
        self.diagnostics
            .push(format!("[{}] {}", self.scope, message.as_ref()));
    }

    pub fn is_empty(&self) -> bool {
        self.drafts.is_empty()
    }

    /// Reserve `wanted` as an id, suffixing it when already taken.
    fn claim(&mut self, wanted: &str) -> String {
        let id = (1..)
            .map(|n| {
                if n == 1 {
                    wanted.to_string()
                } else {
                    format!("{wanted}#{n}")
                }
            })
            .find(|candidate| !self.claimed.contains(candidate))
            .expect("an unused suffix always exists");
        self.claimed.insert(id.clone());
        id
    }

    /// Add a node. `element_id` is the model identity; when another node of
    /// this diagram already holds it, the new node gets a suffixed diagram
    /// id and the collision is reported (identity must be unique).
    pub fn add_node(
        &mut self,
        parent: Option<NodeRef>,
        element_id: &str,
        name: &str,
        kind: NodeKind,
        properties: BTreeMap<String, String>,
    ) -> NodeRef {
        let handle = self.drafts.len();
        let element_id = if element_id.is_empty() {
            name
        } else {
            element_id
        };
        if let Some(holder) = self.node_by_element.get(element_id).copied() {
            let other = self.drafts[holder].node.name.clone();
            self.report(format!(
                "identity collision: '{name}' and '{other}' share the id '{element_id}' — \
                 give one an explicit unique id"
            ));
        }
        self.node_by_element
            .entry(element_id.to_string())
            .or_insert(handle);
        let id = self.claim(element_id);

        let own_keys = distinct([element_id, name, id.as_str()]);
        let keys: Vec<String> = match parent {
            None => own_keys,
            Some(parent) => {
                let qualified: Vec<String> = self.drafts[parent]
                    .keys
                    .iter()
                    .flat_map(|outer| own_keys.iter().map(move |inner| format!("{outer}.{inner}")))
                    .collect();
                own_keys.into_iter().chain(qualified).collect()
            }
        };
        for key in &keys {
            let holders = self.index.entry(key.clone()).or_default();
            if !holders.contains(&handle) {
                holders.push(handle);
            }
        }

        self.drafts.push(Draft {
            node: Node {
                id,
                uuid: element_uuid("element", element_id),
                name: name.to_string(),
                kind,
                ports: Vec::new(),
                children: Vec::new(),
                realizes: Vec::new(),
                compartment: Vec::new(),
                properties,
            },
            parent,
            keys,
        });
        handle
    }

    pub fn node(&self, handle: NodeRef) -> &Node {
        &self.drafts[handle].node
    }

    /// Set the lines listed inside a node's box.
    pub fn set_compartment(&mut self, handle: NodeRef, lines: Vec<String>) {
        self.drafts[handle].node.compartment = lines;
    }

    pub fn add_realization(&mut self, handle: NodeRef, realized: String) {
        self.drafts[handle].node.realizes.push(realized);
    }

    /// Add a declared port to a node; the port id is `<node id>::<suffix>`.
    pub fn add_port(
        &mut self,
        handle: NodeRef,
        suffix: &str,
        name: &str,
        direction: PortDirection,
        interface: Option<String>,
        protocol: Option<String>,
    ) -> String {
        let wanted = format!("{}::{suffix}", self.drafts[handle].node.id);
        let id = self.claim(&wanted);
        self.drafts[handle].node.ports.push(Port {
            id: id.clone(),
            name: name.to_string(),
            direction,
            interface,
            protocol,
            synthesized: false,
        });
        id
    }

    /// Flag a port as derived from an exchange rather than declared.
    pub fn mark_synthesized(&mut self, handle: NodeRef, port_id: &str) {
        if let Some(port) = self.drafts[handle]
            .node
            .ports
            .iter_mut()
            .find(|p| p.id == port_id)
        {
            port.synthesized = true;
        }
    }

    /// Resolve a reference to exactly one node, by id, name or qualified
    /// name (`Parent.Child`). Ambiguity counts as unresolved.
    pub fn resolve(&self, reference: &str) -> Option<NodeRef> {
        match self.index.get(reference)?.as_slice() {
            [single] => Some(*single),
            _ => None,
        }
    }

    /// Like [`resolve`](Self::resolve), but says why nothing was found.
    pub fn find(&self, reference: &str) -> Result<NodeRef, Missing> {
        match self.index.get(reference).map(Vec::as_slice) {
            Some([single]) => Ok(*single),
            Some(_) => Err(Missing::Ambiguous),
            None => Err(Missing::Unknown),
        }
    }

    fn is_ambiguous(&self, reference: &str) -> bool {
        self.index
            .get(reference)
            .is_some_and(|nodes| nodes.len() > 1)
    }

    /// Resolve an exchange endpoint: a node, or `Node.Port`.
    ///
    /// When several ports of the node share the name, the one oriented for
    /// `role` wins. A port the node does not declare is reported and left
    /// unbound: the edge then attaches to the node itself. A dotted
    /// reference into a node that cannot carry ports (an entity, an actor)
    /// is unknown — it is never retargeted to the owner.
    pub fn resolve_endpoint(
        &mut self,
        what: &str,
        reference: &str,
        role: Role,
    ) -> Result<Endpoint, Missing> {
        if self.is_ambiguous(reference) {
            return Err(Missing::Ambiguous);
        }
        if let Some(node) = self.resolve(reference) {
            return Ok(Endpoint {
                node,
                port: None,
                whole: true,
            });
        }
        let Some((owner, port_name)) = reference.rsplit_once('.') else {
            return Err(Missing::Unknown);
        };
        if self.is_ambiguous(owner) {
            return Err(Missing::Ambiguous);
        }
        let Some(node) = self.resolve(owner) else {
            return Err(Missing::Unknown);
        };
        let owner_node = &self.drafts[node].node;
        if !carries_ports(owner_node.kind) {
            return Err(Missing::Unknown);
        }
        let fits = |port: &&Port| {
            !matches!(
                (role, port.direction),
                (Role::Source, PortDirection::In) | (Role::Target, PortDirection::Out)
            )
        };
        let named = || owner_node.ports.iter().filter(|p| p.name == port_name);
        let port = named()
            .find(fits)
            .or_else(|| named().next())
            .map(|p| p.id.clone());
        if port.is_none() {
            let owner_name = owner_node.name.clone();
            self.report(format!(
                "{what}: port '{port_name}' is not declared on '{owner_name}'"
            ));
        }
        Ok(Endpoint {
            node,
            port,
            whole: false,
        })
    }

    /// The endpoint standing for a whole node.
    pub fn whole(&self, node: NodeRef) -> Endpoint {
        Endpoint {
            node,
            port: None,
            whole: true,
        }
    }

    /// Report a reference that resolved to nothing. `expected` names what
    /// it should have been (`"component"`, `"function or actor"`).
    pub fn report_missing(
        &mut self,
        what: &str,
        reference: &str,
        missing: Missing,
        expected: &str,
    ) {
        match missing {
            Missing::Unknown => {
                self.report(format!(
                    "{what}: '{reference}' is not a declared {expected}"
                ));
            }
            Missing::Ambiguous => self.report(format!(
                "{what}: '{reference}' is ambiguous — qualify it (Parent.Name) or give the \
                 elements distinct ids"
            )),
        }
    }

    /// Resolve both ends of an exchange, reporting each one that is missing.
    pub fn resolve_pair(
        &mut self,
        what: &str,
        from: &str,
        to: &str,
        expected: &str,
    ) -> Option<(Endpoint, Endpoint)> {
        let source = self.resolve_endpoint(what, from, Role::Source);
        let target = self.resolve_endpoint(what, to, Role::Target);
        if let Err(missing) = &source {
            self.report_missing(what, from, *missing, expected);
        }
        if let Err(missing) = &target {
            self.report_missing(what, to, *missing, expected);
        }
        Some((source.ok()?, target.ok()?))
    }

    /// Add an edge named `name`; its id is `<kind prefix>:<name>`, suffixed
    /// when several exchanges of one kind share a name.
    pub fn add_edge(
        &mut self,
        kind: EdgeKind,
        source: &Endpoint,
        target: &Endpoint,
        name: &str,
        exchange_item: Option<String>,
        properties: BTreeMap<String, String>,
    ) -> String {
        let prefix = match kind {
            EdgeKind::Interaction => "int",
            EdgeKind::CommunicationMean => "cm",
            EdgeKind::FunctionalExchange => "fe",
            EdgeKind::ComponentExchange => "ce",
            EdgeKind::PhysicalLink => "pl",
            EdgeKind::PhysicalExchange => "px",
            EdgeKind::Transition => "tr",
            EdgeKind::Message => "msg",
            EdgeKind::Exploitation => "exp",
            EdgeKind::Realization => "rea",
            EdgeKind::Involvement => "inv",
            EdgeKind::CapabilityAssociation => "ca",
            EdgeKind::Generalization => "gen",
            EdgeKind::Association => "as",
            EdgeKind::ItemElement => "ie",
            EdgeKind::Breakdown => "bd",
        };
        let base_id = format!("{prefix}:{name}");
        let id = self.claim(&base_id);
        self.edges.push(Edge {
            id: id.clone(),
            kind,
            source: self.drafts[source.node].node.id.clone(),
            source_port: source.port.clone(),
            target: self.drafts[target.node].node.id.clone(),
            target_port: target.port.clone(),
            label: name.to_string(),
            exchange_item,
            properties,
        });
        id
    }

    /// Resolve a functional chain (or operational process, or physical
    /// path) against the nodes and edges built so far. Each chain is added
    /// by its home viewpoint only, so anything unresolved is reported.
    pub fn add_chain(&mut self, id: &str, name: &str, involves: &[String]) {
        let mut nodes: Vec<String> = Vec::new();
        let mut edges: Vec<String> = Vec::new();
        let mut unresolved: Vec<&str> = Vec::new();
        for reference in involves {
            if let Some(node) = self.resolve(reference) {
                nodes.push(self.drafts[node].node.id.clone());
            } else if let Some(edge) = self
                .edges
                .iter()
                .find(|e| &e.label == reference || &e.id == reference)
            {
                edges.push(edge.id.clone());
            } else {
                unresolved.push(reference);
            }
        }
        let missing = unresolved
            .iter()
            .map(|r| format!("'{r}'"))
            .collect::<Vec<_>>()
            .join(", ");
        if nodes.is_empty() && edges.is_empty() {
            self.report(format!(
                "chain '{name}': none of its elements ({missing}) is drawn in this view"
            ));
            return;
        }
        if !unresolved.is_empty() {
            self.report(format!("chain '{name}': {missing} not drawn in this view"));
        }
        for pair in nodes.windows(2) {
            let linking: Vec<String> = self
                .edges
                .iter()
                .filter(|e| e.source == pair[0] && e.target == pair[1])
                .map(|e| e.id.clone())
                .collect();
            for edge in linking {
                if !edges.contains(&edge) {
                    edges.push(edge);
                }
            }
        }
        let id = if id.is_empty() { name } else { id };
        self.chains.push(Chain {
            id: id.to_string(),
            name: name.to_string(),
            nodes,
            edges,
        });
    }

    /// Assemble the node tree. Returns `None` when the viewpoint is empty.
    pub fn finish(self, title: String) -> Option<(Diagram, Vec<String>)> {
        if self.drafts.is_empty() {
            return None;
        }
        Some(self.finish_scoped(title))
    }

    /// Change what an edge displays; its id keeps the name it was added
    /// under (a transition is identified by its ends, labelled by its
    /// trigger).
    pub fn set_edge_label(&mut self, edge_id: &str, label: String) {
        if let Some(edge) = self.edges.iter_mut().find(|edge| edge.id == edge_id) {
            edge.label = label;
        }
    }

    /// Assemble the diagram, empty or not, under the builder's scope id.
    pub fn finish_scoped(self, title: String) -> (Diagram, Vec<String>) {
        let id = self.scope.clone();
        let parents: Vec<Option<NodeRef>> = self.drafts.iter().map(|d| d.parent).collect();
        let mut nodes: Vec<Option<Node>> = self.drafts.into_iter().map(|d| Some(d.node)).collect();
        // Children were always added after their parent, so walking the
        // arena backwards attaches complete subtrees.
        for handle in (0..nodes.len()).rev() {
            if let Some(parent) = parents[handle] {
                let child = nodes[handle].take().expect("each node is attached once");
                nodes[parent]
                    .as_mut()
                    .expect("parent precedes child")
                    .children
                    .insert(0, child);
            }
        }
        let diagram = Diagram {
            id,
            kind: self.view,
            title,
            nodes: nodes.into_iter().flatten().collect(),
            edges: self.edges,
            chains: self.chains,
        };
        (diagram, self.diagnostics)
    }
}

/// Node kinds that own ports, i.e. that `Owner.Port` can address.
fn carries_ports(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::Function | NodeKind::LogicalComponent | NodeKind::PhysicalNode
    )
}

fn distinct<const N: usize>(keys: [&str; N]) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(N);
    for key in keys {
        if !key.is_empty() && !out.iter().any(|k| k == key) {
            out.push(key.to_string());
        }
    }
    out
}

/// Display properties: the listed attribute keys that are present.
pub(super) fn properties(
    attributes: &HashMap<String, AttributeValue>,
    keys: &[&str],
) -> BTreeMap<String, String> {
    keys.iter()
        .filter_map(|key| {
            attributes
                .get(*key)
                .map(|value| ((*key).to_string(), value.display()))
        })
        .collect()
}

/// An attribute as a string, when present and textual.
pub(super) fn text<'a>(
    attributes: &'a HashMap<String, AttributeValue>,
    key: &str,
) -> Option<&'a str> {
    attributes.get(key).and_then(AttributeValue::as_string)
}

/// The model identity of an element: its explicit `id` attribute, else the
/// fallback (the same rule the semantic analyzer applies).
pub(super) fn identity<'a>(
    attributes: &'a HashMap<String, AttributeValue>,
    fallback: &'a str,
) -> &'a str {
    text(attributes, "id")
        .filter(|id| !id.is_empty())
        .unwrap_or(fallback)
}

/// Every attribute of an element as display properties, except its `id`.
pub(super) fn all_properties(
    attributes: &HashMap<String, AttributeValue>,
) -> BTreeMap<String, String> {
    attributes
        .iter()
        .filter(|(key, _)| key.as_str() != "id")
        .map(|(key, value)| (key.clone(), value.display()))
        .collect()
}

/// Strings of a list attribute (`inputs: ["a", "b"]`), in order.
pub(super) fn string_list(attributes: &HashMap<String, AttributeValue>, key: &str) -> Vec<String> {
    match attributes.get(key) {
        Some(AttributeValue::List(items)) => items
            .iter()
            .filter_map(|v| v.as_string().map(str::to_string))
            .collect(),
        Some(AttributeValue::String(single)) => vec![single.clone()],
        _ => Vec::new(),
    }
}

/// The model's display title for a viewpoint: the first block's `name`.
pub(super) fn view_title(view: ViewKind, block_names: impl Iterator<Item = String>) -> String {
    block_names
        .into_iter()
        .find(|name| !name.is_empty())
        .unwrap_or_else(|| view.label().to_string())
}
