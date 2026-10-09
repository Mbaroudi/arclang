//! Viewpoint diagram model — what a renderer draws, compiled from the AST.
//!
//! One [`Diagram`] per Arcadia viewpoint that the model populates:
//!
//! | kind  | Arcadia diagram                | content                                         |
//! |-------|--------------------------------|-------------------------------------------------|
//! | `oab` | Operational Architecture Blank | entities/actors, their activities, interactions |
//! | `sab` | System Architecture Blank      | the system and actors, functions, exchanges     |
//! | `lab` | Logical Architecture Blank     | components, allocated functions, both exchanges |
//! | `pab` | Physical Architecture Blank    | nodes, behaviour components, links, exchanges   |
//! | `msm` | Mode and State Machine         | modes, states and transitions (one per machine) |
//! | `es`  | Exchange Scenario              | lifelines and ordered messages (one per scenario)|
//! | `cap` | Missions and Capabilities      | missions, capabilities, what they involve       |
//! | `cdb` | Class Diagram Blank            | classes, enumerations, types, exchange items    |
//! | `pbs` | Product Breakdown Structure    | configuration items, from system down to items  |
//!
//! Unlike the flat `graph_model`, this model keeps the structure Arcadia
//! notation needs: nodes are a tree (functions sit *inside* the component
//! they are allocated to), every exchange names the ports it connects, and
//! functional chains resolve to the node and edge ids to highlight.
//!
//! The builder never invents model content. A reference that does not
//! resolve produces a diagnostic, not a guessed node or port; the only
//! derived elements are function ports for exchanges whose endpoints name no
//! port, and those are flagged `synthesized`.
//!
//! Output is deterministic (declaration order, sorted maps) and pinned by
//! golden files in `tests/fixtures/diagrams/`. Layout is not done here: see
//! [`elk`] for the ELK graph a layout engine consumes, and [`html`] for the
//! viewer that lays it out and draws it in Arcadia notation.

mod builder;
mod capabilities;
mod data;
mod elements;
pub mod elk;
mod functions;
pub mod html;
pub mod layout_file;
mod links;
mod logical;
mod operational;
mod physical;
mod product;
mod scenario;
mod state_machine;
mod system;

use super::ast::Model;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};

/// Version of the serialized diagram model. Bumped on breaking changes.
pub const SCHEMA_VERSION: &str = "1";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DiagramSet {
    pub schema_version: &'static str,
    pub diagrams: Vec<Diagram>,
    /// Model inconsistencies found while building the diagrams, each
    /// prefixed with the id of the diagram it concerns (`[lab] ...`,
    /// `[msm:OperatingModes] ...`).
    pub diagnostics: Vec<String>,
    /// Traces of the model, their ends resolved to drawn elements: how a
    /// reader goes from one view to another.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<Link>,
    /// How a reader arranged these diagrams, from the layout file kept next
    /// to the model. See [`layout_file`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<layout_file::LayoutFile>,
    /// File name the viewer proposes when the reader saves the layout.
    #[serde(skip)]
    pub layout_name: Option<String>,
}

pub use links::Link;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ViewKind {
    Oab,
    Sab,
    Lab,
    Pab,
    /// Mode and state machine. One diagram per machine, id `msm:<name>`.
    Msm,
    /// Exchange scenario (sequence diagram). One per scenario, id `es:<name>`.
    Es,
    /// Missions, capabilities and the elements they involve.
    Cap,
    /// Data model: classes, enumerations, data types and exchange items.
    Cdb,
    /// Product breakdown structure (EPBS).
    Pbs,
}

impl ViewKind {
    pub const ALL: [ViewKind; 9] = [
        ViewKind::Oab,
        ViewKind::Sab,
        ViewKind::Lab,
        ViewKind::Pab,
        ViewKind::Msm,
        ViewKind::Es,
        ViewKind::Cap,
        ViewKind::Cdb,
        ViewKind::Pbs,
    ];

    /// Short identifier, also the diagram id and the CLI `--view` value.
    pub fn id(self) -> &'static str {
        match self {
            ViewKind::Oab => "oab",
            ViewKind::Sab => "sab",
            ViewKind::Lab => "lab",
            ViewKind::Pab => "pab",
            ViewKind::Msm => "msm",
            ViewKind::Es => "es",
            ViewKind::Cap => "cap",
            ViewKind::Cdb => "cdb",
            ViewKind::Pbs => "pbs",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ViewKind::Oab => "Operational Architecture",
            ViewKind::Sab => "System Architecture",
            ViewKind::Lab => "Logical Architecture",
            ViewKind::Pab => "Physical Architecture",
            ViewKind::Msm => "Modes and States",
            ViewKind::Es => "Scenarios",
            ViewKind::Cap => "Missions and Capabilities",
            ViewKind::Cdb => "Data Model",
            ViewKind::Pbs => "Product Breakdown",
        }
    }

    pub fn parse(text: &str) -> Option<ViewKind> {
        ViewKind::ALL
            .into_iter()
            .find(|kind| kind.id().eq_ignore_ascii_case(text))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Diagram {
    /// The view id (`lab`), or `<view>:<name>` for views drawn once per
    /// machine or scenario (`msm:OperatingModes`).
    pub id: String,
    pub kind: ViewKind,
    pub title: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub chains: Vec<Chain>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    OperationalActor,
    OperationalEntity,
    OperationalActivity,
    /// An ordered set of activities, the operational counterpart of a
    /// functional chain.
    OperationalProcess,
    System,
    SystemActor,
    SystemComponent,
    Function,
    LogicalComponent,
    PhysicalNode,
    BehaviorComponent,
    HardwareComponent,
    DeployedComponent,
    State,
    Mode,
    /// Pseudo-state marking where a machine starts.
    InitialState,
    /// A participant of a scenario.
    Lifeline,
    Mission,
    OperationalCapability,
    Capability,
    CapabilityRealization,
    FunctionalChain,
    Class,
    Enumeration,
    DataType,
    ExchangeItem,
    /// An element of the product breakdown: system, subsystem or item.
    ConfigurationItem,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Node {
    /// Unique within the diagram. Equals the model element id unless two
    /// elements share one (then suffixed, and a diagnostic is emitted).
    pub id: String,
    /// Stable identity of the model element (same as the semantic model).
    /// Two nodes reported as an identity collision share it: that is the
    /// model's actual state until one is given an explicit id.
    pub uuid: String,
    pub name: String,
    pub kind: NodeKind,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ports: Vec<Port>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Node>,
    /// Ids of the higher-level elements this node realizes (PA -> LA).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub realizes: Vec<String>,
    /// Lines listed inside the box: a class's fields, an enumeration's
    /// values.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub compartment: Vec<String>,
    /// Display attributes (description, safety level, ...), sorted by key.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PortDirection {
    In,
    Out,
    InOut,
    /// Physical ports are not oriented (Arcadia).
    Undirected,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Port {
    pub id: String,
    pub name: String,
    pub direction: PortDirection,
    /// Interface (component port) or data type (function port) carried.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    /// True when the model declares no such port and it was derived from an
    /// exchange endpoint.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub synthesized: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    Interaction,
    CommunicationMean,
    FunctionalExchange,
    ComponentExchange,
    PhysicalLink,
    PhysicalExchange,
    Transition,
    /// A scenario message. Messages are ordered: their position in
    /// `Diagram::edges` is their position in time.
    Message,
    /// A mission exploits a capability.
    Exploitation,
    /// A capability realizes a higher-level one.
    Realization,
    /// A capability involves a function, an actor or a functional chain.
    Involvement,
    /// Include or extend between capabilities of one level.
    CapabilityAssociation,
    /// A capability is a special case of another of the same level.
    Generalization,
    /// A field typed by another data element.
    Association,
    /// An exchange item groups a data element.
    ItemElement,
    /// A configuration item is made of another.
    Breakdown,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Edge {
    pub id: String,
    pub kind: EdgeKind,
    /// Node id. When `source_port` is set, the port belongs to this node.
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_port: Option<String>,
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_port: Option<String>,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange_item: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, String>,
}

/// A functional chain / operational process, resolved to diagram ids.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Chain {
    pub id: String,
    pub name: String,
    /// Involved nodes, in chain order.
    pub nodes: Vec<String>,
    /// Involved edges: those named by the chain, plus the exchanges linking
    /// consecutive chain nodes.
    pub edges: Vec<String>,
}

/// Build every viewpoint diagram the model populates.
pub fn build_diagrams(model: &Model) -> DiagramSet {
    let built = [
        operational::build(model),
        system::build(model),
        logical::build(model),
        physical::build(model),
    ]
    .into_iter()
    .flatten()
    .map(|(diagram, diagnostics)| (Some(diagram), diagnostics))
    .chain(
        [
            capabilities::build(model),
            data::build(model),
            product::build(model),
        ]
        .into_iter()
        .flatten()
        .map(|(diagram, diagnostics)| (Some(diagram), diagnostics)),
    )
    .chain(state_machine::build(model))
    .chain(scenario::build(model));

    let mut diagrams: Vec<Diagram> = Vec::new();
    let mut diagnostics: Vec<String> = Vec::new();
    for (diagram, found) in built {
        let Some(mut diagram) = diagram else {
            diagnostics.extend(found);
            continue;
        };
        // Diagram ids address tabs, caches and exported files: two machines
        // or scenarios sharing a name must not share one.
        let wanted = diagram.id.clone();
        if diagrams.iter().any(|other| other.id == wanted) {
            diagram.id = (2..)
                .map(|n| format!("{wanted}#{n}"))
                .find(|candidate| diagrams.iter().all(|other| &other.id != candidate))
                .expect("an unused suffix always exists");
            diagnostics.push(format!(
                "[{}] '{}' is declared more than once — rename one so each has its own identity",
                diagram.id, diagram.title
            ));
        }
        let (old_scope, new_scope) = (format!("[{wanted}] "), format!("[{}] ", diagram.id));
        diagnostics.extend(
            found
                .into_iter()
                .map(|line| match line.strip_prefix(&old_scope) {
                    Some(rest) if old_scope != new_scope => format!("{new_scope}{rest}"),
                    _ => line,
                }),
        );
        diagrams.push(diagram);
    }

    let links = links::cross_links(model, &diagrams);
    DiagramSet {
        schema_version: SCHEMA_VERSION,
        diagrams,
        diagnostics,
        links,
        layout: None,
        layout_name: None,
    }
}

impl Diagram {
    /// Structural invariants every diagram must satisfy; a non-empty result
    /// is a bug in the builder, not in the user's model.
    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();
        // Nodes, ports and edges share one id namespace: a layout engine
        // addresses all three by id.
        let mut all_ids: HashSet<&str> = HashSet::new();
        let mut node_ids = HashSet::new();
        let mut port_owner: HashMap<&str, &str> = HashMap::new();
        let mut stack: Vec<&Node> = self.nodes.iter().collect();
        while let Some(node) = stack.pop() {
            if node.id.is_empty() || !all_ids.insert(node.id.as_str()) {
                problems.push(format!("empty or duplicate node id '{}'", node.id));
            }
            node_ids.insert(node.id.as_str());
            for port in &node.ports {
                if port.id.is_empty() || !all_ids.insert(port.id.as_str()) {
                    problems.push(format!("empty or duplicate port id '{}'", port.id));
                }
                port_owner.insert(port.id.as_str(), node.id.as_str());
            }
            stack.extend(node.children.iter());
        }

        let mut edge_ids = HashSet::new();
        for edge in &self.edges {
            if edge.id.is_empty() || !all_ids.insert(edge.id.as_str()) {
                problems.push(format!("empty or duplicate edge id '{}'", edge.id));
            }
            edge_ids.insert(edge.id.as_str());
            for (role, node, port) in [
                ("source", &edge.source, &edge.source_port),
                ("target", &edge.target, &edge.target_port),
            ] {
                if !node_ids.contains(node.as_str()) {
                    problems.push(format!(
                        "edge '{}': {role} node '{node}' does not exist",
                        edge.id
                    ));
                }
                if let Some(port) = port {
                    if port_owner.get(port.as_str()) != Some(&node.as_str()) {
                        problems.push(format!(
                            "edge '{}': {role} port '{port}' does not belong to node '{node}'",
                            edge.id
                        ));
                    }
                }
            }
        }

        for chain in &self.chains {
            for node in &chain.nodes {
                if !node_ids.contains(node.as_str()) {
                    problems.push(format!(
                        "chain '{}': node '{node}' does not exist",
                        chain.id
                    ));
                }
            }
            for edge in &chain.edges {
                if !edge_ids.contains(edge.as_str()) {
                    problems.push(format!(
                        "chain '{}': edge '{edge}' does not exist",
                        chain.id
                    ));
                }
            }
        }
        problems
    }
}
