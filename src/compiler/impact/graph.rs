//! The impact graph: the element graph of a compiled model with every link
//! a change travels along, typed and oriented.
//!
//! Each link is stored twice, once per way, with the words that explain it
//! that way (`satisfies` / `satisfied by`). One of its ends depends on the
//! other (a component on the requirement it satisfies, an exchange on its
//! ends, a diagram on what it shows): going from the end depended on to the
//! dependent is going *downstream*. Exchanges, which are relationships of
//! the element graph, become nodes here, so that an interface change is an
//! impacted item of its own and the path through it can be read.

use crate::compiler::ast::{AttributeValue, Model};
use crate::compiler::constraint::Expr;
use crate::compiler::metamodel::Layer;
use crate::compiler::tables::{self, Context, TableSpec};
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// The kinds of link a change propagates along (`--kinds`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkKind {
    /// `realizes`, `implements`, `refines` between layers; a behavior
    /// component's `allocated_component`.
    Realization,
    /// A function performed by a component, an `allocates` trace.
    Allocation,
    /// A component deployed on a node.
    Deployment,
    /// Functional, component, operational and physical exchanges, links, interfaces.
    Exchange,
    /// The ports of an element, the link carrying an exchange.
    Interface,
    /// What a functional chain, a physical path or a capability involves.
    Chain,
    Satisfies,
    /// Verification cases (`verifies`, `asserts`) and their verdicts.
    Verifies,
    /// Hazards and their mitigations, threats and assets.
    Safety,
    /// Owner and owned elements.
    Containment,
    /// `is:` users of a type, specializations.
    Typing,
    /// Constraints over an element's attributes.
    Constraint,
    /// Declared diagrams and tables that show the element.
    View,
    /// Scenario messages and state transitions.
    Behavior,
    /// Any other reference an attribute makes (`mission:`, `parent:`...).
    Reference,
}

impl LinkKind {
    pub const ALL: [LinkKind; 15] = [
        LinkKind::Realization,
        LinkKind::Allocation,
        LinkKind::Deployment,
        LinkKind::Exchange,
        LinkKind::Interface,
        LinkKind::Chain,
        LinkKind::Satisfies,
        LinkKind::Verifies,
        LinkKind::Safety,
        LinkKind::Containment,
        LinkKind::Typing,
        LinkKind::Constraint,
        LinkKind::View,
        LinkKind::Behavior,
        LinkKind::Reference,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            LinkKind::Realization => "realization",
            LinkKind::Allocation => "allocation",
            LinkKind::Deployment => "deployment",
            LinkKind::Exchange => "exchange",
            LinkKind::Interface => "interface",
            LinkKind::Chain => "chain",
            LinkKind::Satisfies => "satisfies",
            LinkKind::Verifies => "verifies",
            LinkKind::Safety => "safety",
            LinkKind::Containment => "containment",
            LinkKind::Typing => "typing",
            LinkKind::Constraint => "constraint",
            LinkKind::View => "view",
            LinkKind::Behavior => "behavior",
            LinkKind::Reference => "reference",
        }
    }

    pub fn parse(text: &str) -> Option<LinkKind> {
        let key = text.trim().to_ascii_lowercase();
        let key = match key.as_str() {
            "realizes" | "implements" | "refines" => "realization",
            "allocations" | "allocates" => "allocation",
            "deployments" | "deploys" => "deployment",
            "exchanges" => "exchange",
            "ports" | "interfaces" | "icd" => "interface",
            "chains" | "functional_chain" | "functional_chains" => "chain",
            "satisfy" => "satisfies",
            "verification" | "tests" | "verify" => "verifies",
            "hazards" | "mitigations" => "safety",
            "owner" | "children" | "ownership" => "containment",
            "types" | "type" | "is" => "typing",
            "constraints" | "budget" | "budgets" => "constraint",
            "views" | "diagrams" | "tables" => "view",
            "scenarios" | "messages" | "modes" => "behavior",
            "references" => "reference",
            other => other,
        };
        LinkKind::ALL.iter().copied().find(|k| k.as_str() == key)
    }
}

/// One way of a link.
#[derive(Debug, Clone)]
pub struct Edge {
    pub to: usize,
    pub kind: LinkKind,
    /// The words of the link read this way: `satisfies`, `verified by`.
    pub label: String,
    /// Whether this way goes from the end depended on to the dependent.
    pub downstream: bool,
}

/// An element of the model, an exchange or a declared diagram.
#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub uuid: String,
    pub name: String,
    pub kind: String,
    /// `OA`, `SA`, `LA`, `PA`, `EPBS`, `Data` or `Transverse`.
    pub layer: String,
    /// The index in the element graph, for an element.
    pub element: Option<usize>,
    /// `ASIL-D`, `DAL-A`, `SIL 3`: the integrity level it carries.
    pub integrity: Option<String>,
    /// For a functional chain: its latency budget in milliseconds.
    pub budget_ms: Option<f64>,
}

pub struct ImpactGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Vec<Edge>>,
    by_id: HashMap<String, usize>,
    by_uuid: HashMap<String, usize>,
    by_name: HashMap<String, Vec<usize>>,
    of_element: HashMap<usize, usize>,
}

pub fn layer_code(layer: Layer) -> &'static str {
    match layer {
        Layer::Operational => "OA",
        Layer::System => "SA",
        Layer::Logical => "LA",
        Layer::Physical => "PA",
        Layer::Epbs => "EPBS",
        Layer::Data => "Data",
        Layer::Transverse => "Transverse",
    }
}

fn view_layer(view: &str) -> &'static str {
    match view {
        "oab" | "oes" | "ocb" => "OA",
        "sab" | "cap" | "sfbd" => "SA",
        "lab" => "LA",
        "pab" => "PA",
        "pbs" => "EPBS",
        "cdb" => "Data",
        _ => "Transverse",
    }
}

/// The integrity level an element states: an ASIL (A to D), a DAL (A to C), a SIL.
pub fn integrity_of(attributes: &HashMap<String, AttributeValue>) -> Option<String> {
    for key in ["asil", "safety_level", "integrity_level", "dal", "sil"] {
        let Some(text) = attributes.get(key).and_then(|v| v.as_string()) else { continue };
        let upper = text.trim().to_ascii_uppercase();
        let critical = ["ASIL-A", "ASIL-B", "ASIL-C", "ASIL-D", "ASIL A", "ASIL B", "ASIL C", "ASIL D", "DAL-A", "DAL-B", "DAL-C", "DAL A", "DAL B", "DAL C", "SIL 1", "SIL 2", "SIL 3", "SIL 4", "SIL-1", "SIL-2", "SIL-3", "SIL-4"];
        if critical.iter().any(|c| upper.starts_with(c)) {
            return Some(text.trim().to_string());
        }
    }
    None
}

/// A time attribute in milliseconds: a quantity, else a bare number taken as ms.
pub fn millis(value: Option<&AttributeValue>) -> Option<f64> {
    match value? {
        AttributeValue::Quantity(q) if q.dimension() == crate::compiler::quantity::Dimension::Time => Some(q.canonical() * 1000.0),
        AttributeValue::Number(n) => Some(*n),
        AttributeValue::Integer(i) => Some(*i as f64),
        AttributeValue::String(text) => crate::compiler::production_gate::parse_millis(text),
        _ => None,
    }
}

const COMPONENT_KINDS: [&str; 7] = ["LogicalComponent", "SystemComponent", "PhysicalNode", "BehaviorComponent", "HardwareComponent", "OperationalEntity", "SystemActor"];
const EXCHANGE_KINDS: [&str; 6] = ["FunctionalExchange", "ComponentExchange", "OperationalExchange", "PhysicalExchange", "PhysicalLink", "LogicalInterface"];

/// The references an attribute makes, by key: the kind of link, the words
/// read from the element, the words read back, and whether the element
/// holding the attribute is the dependent end.
fn attribute_link(key: &str) -> Option<(LinkKind, &'static str, &'static str, bool)> {
    Some(match key {
        "realizes" | "implements" | "refines" => (LinkKind::Realization, "realizes", "realized by", true),
        "allocated_component" => (LinkKind::Realization, "realizes", "realized by", true),
        "allocated_to" | "allocated_function" | "allocated_functions" => (LinkKind::Allocation, "allocated to", "allocates", true),
        "mission" => (LinkKind::Reference, "contributes to", "contributed to by", true),
        "parent" => (LinkKind::Containment, "under", "holds", true),
        "capability" => (LinkKind::Chain, "exemplifies", "exemplified by", true),
        "via" => (LinkKind::Interface, "carried by", "carries", true),
        "asserts" => (LinkKind::Verifies, "asserts", "asserted by", true),
        "elements" => (LinkKind::Reference, "groups", "grouped in", true),
        "selects" => (LinkKind::Reference, "selects", "selected by", true),
        "participants" => (LinkKind::Behavior, "involves", "plays in", true),
        _ => return None,
    })
}

fn texts(value: &AttributeValue) -> Vec<String> {
    match value {
        AttributeValue::String(one) => vec![one.clone()],
        AttributeValue::List(items) => items.iter().filter_map(|v| v.as_string()).map(str::to_string).collect(),
        _ => Vec::new(),
    }
}

/// The element names an expression mentions: `"FC-001".latency_budget`, `sum("FC-001", latency)`.
fn mentioned(expr: &Expr, into: &mut Vec<String>) {
    match expr {
        Expr::Attr { element, .. } => into.push(element.clone()),
        Expr::Ref(name) => into.push(name.clone()),
        Expr::Call { args, .. } => args.iter().for_each(|a| mentioned(a, into)),
        Expr::Neg(inner) => mentioned(inner, into),
        Expr::Binary { lhs, rhs, .. } => {
            mentioned(lhs, into);
            mentioned(rhs, into);
        }
        Expr::Number(_) | Expr::Quantity(_) => {}
    }
}

impl ImpactGraph {
    pub fn node(&self, index: usize) -> &Node {
        &self.nodes[index]
    }

    pub fn of_element(&self, element: usize) -> Option<usize> {
        self.of_element.get(&element).copied()
    }

    pub fn by_uuid(&self, uuid: &str) -> Option<usize> {
        self.by_uuid.get(uuid).copied()
    }

    /// The node a reference names: an id, a dotted port path, a unique name,
    /// an exchange or a declared diagram by its name.
    pub fn resolve(&self, context: &Context, reference: &str) -> Option<usize> {
        let reference = reference.trim();
        if let Some(&node) = self.by_id.get(reference) {
            return Some(node);
        }
        if let Some(node) = context.resolve(reference).and_then(|e| self.of_element(e)) {
            return Some(node);
        }
        self.by_name.get(reference).and_then(|all| all.first().copied())
    }

    /// The graph of a compiled model, every kind of link included.
    pub fn build(context: &Context) -> ImpactGraph {
        let mut graph = ImpactGraph { nodes: Vec::new(), edges: Vec::new(), by_id: HashMap::new(), by_uuid: HashMap::new(), by_name: HashMap::new(), of_element: HashMap::new() };
        graph.add_elements(context);
        graph.add_containment(context);
        graph.add_relationships(context);
        graph.add_attribute_links(context);
        graph.add_constraints(context);
        graph.add_views(context);
        graph.sort_edges();
        graph
    }

    fn push_node(&mut self, node: Node) -> usize {
        let index = self.nodes.len();
        self.by_id.entry(node.id.clone()).or_insert(index);
        self.by_uuid.insert(node.uuid.clone(), index);
        self.by_name.entry(node.name.clone()).or_default().push(index);
        if let Some(element) = node.element {
            self.of_element.insert(element, index);
        }
        self.nodes.push(node);
        self.edges.push(Vec::new());
        index
    }

    /// A link between two nodes, both ways: `source_depends` says the
    /// source is the end that depends on the other.
    fn link(&mut self, source: usize, target: usize, kind: LinkKind, forward: &str, backward: &str, source_depends: bool) {
        if source == target {
            return;
        }
        if self.edges[source].iter().any(|e| e.to == target && e.kind == kind) {
            return;
        }
        self.edges[source].push(Edge { to: target, kind, label: forward.to_string(), downstream: !source_depends });
        self.edges[target].push(Edge { to: source, kind, label: backward.to_string(), downstream: source_depends });
    }

    fn add_elements(&mut self, context: &Context) {
        for (index, element) in context.graph.elements.iter().enumerate() {
            if element.kind == "Model" {
                continue;
            }
            let budget_ms = if element.kind == "FunctionalChain" { millis(element.attributes.get("latency_budget")) } else { None };
            self.push_node(Node {
                id: element.id.clone(),
                uuid: element.uuid.clone(),
                name: element.name.clone(),
                kind: element.kind.to_string(),
                layer: layer_code(context.layer_of(index)).to_string(),
                element: Some(index),
                integrity: integrity_of(&element.attributes),
                budget_ms,
            });
        }
    }

    fn add_containment(&mut self, context: &Context) {
        for (index, element) in context.graph.elements.iter().enumerate() {
            let (Some(child), Some(owner)) = (self.of_element(index), element.owner.as_deref().and_then(|u| context.index_of_uuid(u)).and_then(|o| self.of_element(o))) else { continue };
            let owner_kind = self.nodes[owner].kind.clone();
            let child_kind = element.kind;
            if child_kind.ends_with("Port") {
                self.link(owner, child, LinkKind::Interface, "has port", "port of", false);
            } else if (child_kind.ends_with("Function") || child_kind == "OperationalActivity") && COMPONENT_KINDS.contains(&owner_kind.as_str()) {
                self.link(owner, child, LinkKind::Allocation, "performs", "performed by", false);
            } else {
                self.link(owner, child, LinkKind::Containment, "contains", "part of", false);
            }
        }
    }

    /// The owner of a port end, when the end is a port.
    fn port_owner(&self, context: &Context, node: usize) -> Option<usize> {
        let element = self.nodes[node].element?;
        if !context.element(element).kind.ends_with("Port") {
            return None;
        }
        let owner = context.element(element).owner.as_deref().and_then(|u| context.index_of_uuid(u))?;
        let owner_node = self.of_element(owner)?;
        if self.nodes[owner_node].kind.ends_with("Port") {
            return self.port_owner(context, owner_node).or(Some(owner_node));
        }
        Some(owner_node)
    }

    fn add_relationships(&mut self, context: &Context) {
        for relationship in &context.graph.relationships {
            let (Some(source), Some(target)) = (self.by_uuid(&relationship.source), self.by_uuid(&relationship.target)) else { continue };
            let kind = relationship.kind;
            if EXCHANGE_KINDS.contains(&kind) {
                self.add_exchange(context, relationship, source, target);
                continue;
            }
            match kind {
                "Trace" => {
                    let trace = relationship.extra.get("traceKind").and_then(|v| v.as_str()).unwrap_or("traces").to_string();
                    let (link, backward) = match trace.as_str() {
                        "satisfies" => (LinkKind::Satisfies, "satisfied by".to_string()),
                        "realizes" => (LinkKind::Realization, "realized by".to_string()),
                        "refines" => (LinkKind::Realization, "refined by".to_string()),
                        "implements" => (LinkKind::Realization, "implemented by".to_string()),
                        "allocates" => (LinkKind::Allocation, "allocated by".to_string()),
                        "verifies" => (LinkKind::Verifies, "verified by".to_string()),
                        "validates" => (LinkKind::Verifies, "validated by".to_string()),
                        other => (LinkKind::Reference, format!("{other} (back)")),
                    };
                    self.link(source, target, link, &trace, &backward, true);
                }
                "Verification" => self.link(source, target, LinkKind::Verifies, "verifies", "verified by", true),
                "Realization" => self.link(source, target, LinkKind::Realization, "realizes", "realized by", true),
                "Contribution" => self.link(source, target, LinkKind::Reference, "contributes to", "contributed to by", true),
                "Deployment" => self.link(source, target, LinkKind::Deployment, "deployed on", "hosts", true),
                "Involvement" => self.link(source, target, LinkKind::Chain, "involves", "involved in", true),
                "SequenceLink" => self.link(source, target, LinkKind::Chain, "precedes", "follows", false),
                "Mitigation" => self.link(source, target, LinkKind::Safety, "mitigates", "mitigated by", false),
                "ThreatApplication" => self.link(source, target, LinkKind::Safety, "threatens", "threatened by", true),
                "ThreatInvolvement" => self.link(source, target, LinkKind::Safety, "involves", "involved in threat", true),
                "AssetMember" => self.link(source, target, LinkKind::Safety, "holds", "member of asset", true),
                "Typing" => self.link(source, target, LinkKind::Typing, "is a", "type of", true),
                "Specialization" => self.link(source, target, LinkKind::Typing, "specializes", "specialized by", true),
                "Message" => {
                    let label = relationship.name.clone().unwrap_or_default();
                    self.link(source, target, LinkKind::Behavior, &format!("sends '{label}' to"), &format!("receives '{label}' from"), false);
                }
                "Transition" => self.link(source, target, LinkKind::Behavior, "transitions to", "transitions from", false),
                "Association" => self.link(source, target, LinkKind::Reference, "associated with", "associated with", true),
                other => {
                    let forward = other.to_ascii_lowercase();
                    self.link(source, target, LinkKind::Reference, &forward, &format!("{forward} (back)"), true);
                }
            }
        }
    }

    /// An exchange as a node between its ends (and the owners of port ends).
    fn add_exchange(&mut self, context: &Context, relationship: &crate::compiler::elements::RelationshipRecord, source: usize, target: usize) {
        let kind = relationship.kind;
        let name = relationship.name.clone().unwrap_or_else(|| format!("{} → {}", self.nodes[source].name, self.nodes[target].name));
        let layer = context.metamodel.kind(kind).map(|k| layer_code(k.layer).to_string()).unwrap_or_else(|| self.nodes[source].layer.clone());
        let id = if self.by_id.contains_key(&name) { format!("{kind}:{name}") } else { name.clone() };
        let node = self.push_node(Node { id, uuid: relationship.uuid.clone(), name, kind: kind.to_string(), layer, element: None, integrity: integrity_of(&relationship.attributes), budget_ms: None });
        let (send, sent, receive, received) = if kind == "PhysicalLink" { ("linked by", "links", "linked by", "links") } else { ("sends", "sent by", "receives", "received by") };
        let mut ends = vec![(source, true), (target, false)];
        if let Some(owner) = self.port_owner(context, source) {
            ends.push((owner, true));
        }
        if let Some(owner) = self.port_owner(context, target) {
            ends.push((owner, false));
        }
        for (end, is_source) in ends {
            if is_source {
                self.link(end, node, LinkKind::Exchange, send, sent, false);
            } else {
                self.link(end, node, LinkKind::Exchange, receive, received, false);
            }
        }
        if let Some(via) = relationship.extra.get("via").and_then(|v| v.as_str()) {
            if let Some(link) = self.by_name.get(via).and_then(|all| all.iter().copied().find(|&n| self.nodes[n].kind == "PhysicalLink")) {
                self.link(node, link, LinkKind::Interface, "carried by", "carries", true);
            }
        }
    }

    fn add_attribute_links(&mut self, context: &Context) {
        for (index, element) in context.graph.elements.iter().enumerate() {
            let Some(from) = self.of_element(index) else { continue };
            let mut keys: Vec<&String> = element.attributes.keys().collect();
            keys.sort();
            for key in keys {
                let Some((kind, forward, backward, holder_depends)) = attribute_link(key) else { continue };
                // These already are relationships of the element graph.
                if key == "realizes" && matches!(element.kind, "Capability" | "CapabilityRealization" | "OperationalCapability") {
                    continue;
                }
                if key == "mission" && element.kind == "Capability" {
                    continue;
                }
                for reference in texts(&element.attributes[key]) {
                    if let Some(to) = self.resolve(context, &reference) {
                        self.link(from, to, kind, forward, backward, holder_depends);
                    }
                }
            }
        }
    }

    fn add_constraints(&mut self, context: &Context) {
        for constraint in &context.result.ast.constraints {
            let designator = if constraint.id.is_empty() { &constraint.name } else { &constraint.id };
            let Some(node) = self.resolve(context, designator) else { continue };
            let mut names = Vec::new();
            mentioned(&constraint.expression, &mut names);
            for name in names {
                if let Some(target) = self.resolve(context, &name) {
                    if target != node && self.nodes[target].element.is_some() {
                        self.link(node, target, LinkKind::Constraint, "constrains", "constrained by", true);
                    }
                }
            }
        }
    }

    /// Declared diagrams (what they show and what it holds) and declared tables (their rows).
    fn add_views(&mut self, context: &Context) {
        let model: &Model = &context.result.ast;
        for diagram in &model.diagrams {
            let node = self.push_node(Node {
                id: format!("diagram:{}", diagram.name),
                uuid: crate::compiler::identity::element_uuid("diagram", &diagram.name),
                name: diagram.name.clone(),
                kind: "Diagram".to_string(),
                layer: view_layer(&diagram.view).to_string(),
                element: None,
                integrity: None,
                budget_ms: None,
            });
            let mut shown: BTreeSet<usize> = BTreeSet::new();
            for reference in &diagram.shows {
                if let Some(target) = self.resolve(context, reference) {
                    shown.insert(target);
                    if let Some(element) = self.nodes[target].element {
                        for (child, record) in context.graph.elements.iter().enumerate() {
                            if record.kind.ends_with("Port") {
                                continue;
                            }
                            if context.owners(child).contains(&element) {
                                if let Some(n) = self.of_element(child) {
                                    shown.insert(n);
                                }
                            }
                        }
                    }
                }
            }
            for target in shown {
                self.link(node, target, LinkKind::View, "shows", "shown in", true);
            }
        }
        for decl in &model.tables {
            let Some(node) = self.resolve(context, &decl.id).or_else(|| self.resolve(context, &decl.name)) else { continue };
            let Ok(table) = tables::compute_in(context, &TableSpec::of(decl)) else { continue };
            let mut listed: BTreeSet<usize> = BTreeSet::new();
            for row in &table.rows {
                listed.extend(self.by_uuid(&row.uuid));
                for cell in row.cells.values() {
                    listed.extend(cell.links.iter().filter_map(|l| self.by_uuid(&l.uuid)));
                }
            }
            if let Some(matrix) = &table.matrix {
                listed.extend(matrix.rows.iter().chain(matrix.columns.iter()).filter_map(|l| self.by_uuid(&l.uuid)));
            }
            for target in listed {
                self.link(node, target, LinkKind::View, "lists", "listed in", true);
            }
        }
    }

    /// The same model always gives the same paths: links in a fixed order.
    fn sort_edges(&mut self) {
        let keys: Vec<(String, String)> = self.nodes.iter().map(|n| (n.id.clone(), n.uuid.clone())).collect();
        for edges in &mut self.edges {
            edges.sort_by(|a, b| (a.kind, &keys[a.to], &a.label).cmp(&(b.kind, &keys[b.to], &b.label)));
        }
    }

    /// How many links of each kind the graph holds (both ways counted once).
    pub fn census(&self) -> BTreeMap<&'static str, usize> {
        let mut out = BTreeMap::new();
        for edges in &self.edges {
            for edge in edges.iter().filter(|e| e.downstream) {
                *out.entry(edge.kind.as_str()).or_insert(0) += 1;
            }
        }
        out
    }
}
