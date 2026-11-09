//! CESAM (CESAMES Systems Architecting Method) as a view of an ArcLang model.
//!
//! CESAM and Arcadia hold the same content, organized differently: an
//! Arcadia layer (OA, SA, LA, PA) is one abstraction level covering several
//! perspectives, a CESAM vision is one perspective covering several levels.
//! CESAM studies any system under three visions, each seen through six
//! aspects — the CESAM grid:
//!
//! | vision         | question | requirements | states | structure | interactions | scenarios | interfaces |
//! |----------------|----------|--------------|--------|-----------|--------------|-----------|------------|
//! | operational    | why?     | stakeholder  | lifecycle | stakeholders, use cases | environment | operational | operational |
//! | functional     | what?    | functional   | functional modes | functional breakdown | functional architecture | functional | functional |
//! | constructional | how?     | technical    | technical configurations | product breakdown | constructional architecture | constructional | constructional |
//!
//! Nothing here is stored: every element and exchange of the element graph
//! is placed in a cell from its kind, refined by what it is about (the
//! participants of a scenario, the block of a requirement, the `vision:` it
//! states). The mapping to Capella diagrams follows the CESAMES webinar
//! "CESAM + Capella" (2023).

use super::ast::{AttributeValue, Model};
use super::elements::{ElementGraph, ElementRecord, RelationshipRecord};
use serde::Serialize;
use std::collections::HashMap;

/// A CESAM vision: the perspective a cell of the grid takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Vision {
    Operational,
    Functional,
    Constructional,
}

/// A CESAM aspect: what a cell of the grid looks at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Aspect {
    Requirements,
    States,
    Structure,
    Interactions,
    Scenarios,
    Interfaces,
}

pub const VISIONS: [Vision; 3] = [Vision::Operational, Vision::Functional, Vision::Constructional];
pub const ASPECTS: [Aspect; 6] = [Aspect::Requirements, Aspect::States, Aspect::Structure, Aspect::Interactions, Aspect::Scenarios, Aspect::Interfaces];

impl Vision {
    pub fn key(self) -> &'static str {
        match self {
            Vision::Operational => "operational",
            Vision::Functional => "functional",
            Vision::Constructional => "constructional",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Vision::Operational => "Operational vision",
            Vision::Functional => "Functional vision",
            Vision::Constructional => "Constructional vision",
        }
    }
    pub fn question(self) -> &'static str {
        match self {
            Vision::Operational => "Why? The services the system gives its environment",
            Vision::Functional => "What? The functions the system performs",
            Vision::Constructional => "How? The concrete resources the system is made of",
        }
    }
    /// The vision a written value names (`vision: functional`), ignoring
    /// case; the French names are accepted too.
    pub fn parse(text: &str) -> Option<Vision> {
        match text.trim().to_ascii_lowercase().as_str() {
            "operational" | "operationnelle" | "operationnel" | "op" | "why" => Some(Vision::Operational),
            "functional" | "fonctionnelle" | "fonctionnel" | "func" | "what" => Some(Vision::Functional),
            "constructional" | "organique" | "physical" | "technical" | "how" => Some(Vision::Constructional),
            _ => None,
        }
    }
}

impl Aspect {
    pub fn key(self) -> &'static str {
        match self {
            Aspect::Requirements => "requirements",
            Aspect::States => "states",
            Aspect::Structure => "structure",
            Aspect::Interactions => "interactions",
            Aspect::Scenarios => "scenarios",
            Aspect::Interfaces => "interfaces",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Aspect::Requirements => "Expected properties",
            Aspect::States => "States",
            Aspect::Structure => "Static elements",
            Aspect::Interactions => "Interactions",
            Aspect::Scenarios => "Scenarios",
            Aspect::Interfaces => "Interfaces (flows)",
        }
    }
}

/// One cell of the CESAM grid: the CESAM view it is, and the Capella
/// diagrams that draw it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct View {
    pub vision: Vision,
    pub aspect: Aspect,
    /// The CESAM name of the view.
    pub name: &'static str,
    /// The Capella diagrams the view maps to (CESAMES, "CESAM + Capella").
    pub capella: &'static str,
}

/// The view of each cell of the grid.
pub fn view(vision: Vision, aspect: Aspect) -> View {
    use Aspect::*;
    use Vision::*;
    let (name, capella) = match (vision, aspect) {
        (Operational, Requirements) => ("Stakeholder requirements", "requirements (stakeholder)"),
        (Operational, States) => ("Lifecycle", "[M&S] Mode State Machine"),
        (Operational, Structure) => ("Stakeholders and use cases", "[OEBD] Operational Entity Breakdown, [CSA] Contextual System Actors, [MCB] Mission Capabilities Blank"),
        (Operational, Interactions) => ("Environment diagram", "[OAB] Operational Architecture Blank, [SAB] System Architecture Blank"),
        (Operational, Scenarios) => ("Operational scenarios", "[ES] Exchange Scenario"),
        (Operational, Interfaces) => ("Operational interfaces", "[CDB] Class Diagram Blank"),
        (Functional, Requirements) => ("Functional requirements", "requirements (system)"),
        (Functional, States) => ("Functional modes", "[M&S] Mode State Machine"),
        (Functional, Structure) => ("Functional breakdown", "[SFBD] System, [LFBD] Logical, [PFBD] Physical Function Breakdown"),
        (Functional, Interactions) => ("Functional interactions", "[SDFB] System, [LDFB] Logical, [PDFB] Physical Data Flow Blank"),
        (Functional, Scenarios) => ("Functional scenarios", "[FS] Functional Scenario, functional chains"),
        (Functional, Interfaces) => ("Functional interfaces", "[CDB] Class Diagram Blank"),
        (Constructional, Requirements) => ("Technical requirements", "requirements (technical)"),
        (Constructional, States) => ("Technical configurations", "[M&S] Mode State Machine"),
        (Constructional, Structure) => ("Product breakdown", "[LCBD] Logical, [PCBD] Physical Component Breakdown, EPBS"),
        (Constructional, Interactions) => ("Constructional interactions", "[LAB] Logical, [PAB] Physical Architecture Blank"),
        (Constructional, Scenarios) => ("Constructional scenarios", "[ES] Exchange Scenario"),
        (Constructional, Interfaces) => ("Constructional interfaces", "[CDB] Class Diagram Blank"),
    };
    View { vision, aspect, name, capella }
}

/// Where an element or an exchange sits in the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Placement {
    pub vision: Vision,
    pub aspect: Aspect,
}

fn at(vision: Vision, aspect: Aspect) -> Option<Placement> {
    Some(Placement { vision, aspect })
}

/// The vision kinds of a function: performing it is functional.
fn is_function_kind(kind: &str) -> bool {
    matches!(kind, "SystemFunction" | "LogicalFunction" | "PhysicalFunction" | "OperationalActivity" | "FunctionalChain")
}

/// Kinds that are concrete resources of the solution.
fn is_constructional_kind(kind: &str) -> bool {
    matches!(
        kind,
        "LogicalComponent" | "PhysicalNode" | "BehaviorComponent" | "HardwareComponent" | "EpbsSystem" | "EpbsSubsystem" | "EpbsItem"
    )
}

/// The cell a kind sits in when nothing else is known about the element,
/// and whether the element itself can move it (`refined`).
pub fn kind_placement(kind: &str) -> Option<(Placement, bool)> {
    use Aspect::*;
    use Vision::*;
    let fixed = |v, a| at(v, a).map(|p| (p, false));
    let refined = |v, a| at(v, a).map(|p| (p, true));
    match kind {
        "Concern" => fixed(Operational, Requirements),
        "Stakeholder" => fixed(Operational, Structure),
        "Requirement" => refined(Functional, Requirements),
        "Actor" | "OperationalEntity" | "SystemActor" | "OperationalCapability" | "Mission" | "Capability" | "CapabilityRealization" => fixed(Operational, Structure),
        "SystemComponent" | "OperationalExchange" => fixed(Operational, Interactions),
        "OperationalActivity" => fixed(Operational, Interactions),
        "OperationalProcess" => fixed(Operational, Scenarios),
        "SystemFunction" | "LogicalFunction" | "PhysicalFunction" => fixed(Functional, Structure),
        "FunctionalExchange" => fixed(Functional, Interactions),
        "FunctionalChain" | "ControlNode" | "SequenceLink" => fixed(Functional, Scenarios),
        "FunctionPort" => fixed(Functional, Interfaces),
        "LogicalComponent" | "PhysicalNode" | "BehaviorComponent" | "HardwareComponent" | "EpbsSystem" | "EpbsSubsystem" | "EpbsItem" => {
            fixed(Constructional, Structure)
        }
        "ComponentExchange" | "PhysicalLink" | "PhysicalExchange" | "Deployment" | "PhysicalPath" => fixed(Constructional, Interactions),
        "ComponentPort" | "PhysicalPort" | "LogicalInterface" => fixed(Constructional, Interfaces),
        "ExchangeItem" | "Class" | "Association" | "Enumeration" | "DataType" => refined(Functional, Interfaces),
        "StateMachine" | "State" | "Transition" => refined(Functional, States),
        "Scenario" | "Fragment" | "StateFragment" | "Message" => refined(Operational, Scenarios),
        _ => None,
    }
}

/// The CESAM column of `spec/METAMODEL.md` for a kind.
pub fn kind_note(kind: &str) -> Option<String> {
    let (placement, refined) = kind_placement(kind)?;
    let cell = view(placement.vision, placement.aspect);
    let base = format!("{} · {} ({})", placement.vision.label(), placement.aspect.label(), cell.name);
    if !refined {
        return Some(base);
    }
    let how = match kind {
        "Requirement" => "by its `vision:`, else its block (`requirements stakeholder|functional|technical`), else what satisfies it",
        "StateMachine" | "State" | "Transition" => "by the machine's `kind:` (`lifecycle`, `functional_modes`, `technical_configurations`) or `vision:`",
        "Scenario" | "Fragment" | "StateFragment" | "Message" => "by the scenario's `vision:`, else its participants (functions: functional; components: constructional)",
        _ => "by its `vision:`, else the exchanges it types",
    };
    Some(format!("{base} by default; placed {how}"))
}

/// Places every element and exchange of a model in the grid.
pub struct Classifier<'a> {
    graph: &'a ElementGraph,
    by_uuid: HashMap<&'a str, usize>,
    by_ref: HashMap<&'a str, usize>,
    /// Requirement id -> the subtype of the block declaring it (`stakeholder`).
    requirement_block: HashMap<String, String>,
    /// Requirement uuid -> kinds of what satisfies it.
    satisfiers: HashMap<&'a str, Vec<&'static str>>,
    /// Exchange item / class name -> visions of the exchanges it types.
    typing: HashMap<String, Vec<Vision>>,
}

fn text_attr<'b>(attributes: &'b HashMap<String, AttributeValue>, key: &str) -> Option<&'b str> {
    attributes.get(key).and_then(|v| v.as_string())
}

impl<'a> Classifier<'a> {
    pub fn new(graph: &'a ElementGraph, ast: &Model) -> Classifier<'a> {
        let by_uuid = graph.elements.iter().enumerate().map(|(i, e)| (e.uuid.as_str(), i)).collect();
        let mut by_ref: HashMap<&str, usize> = HashMap::new();
        for (i, e) in graph.elements.iter().enumerate() {
            by_ref.entry(e.id.as_str()).or_insert(i);
            by_ref.entry(e.name.as_str()).or_insert(i);
        }
        let mut requirement_block = HashMap::new();
        for sa in &ast.system_analysis {
            let subtype = sa.name.strip_suffix(" Requirements").unwrap_or("").to_ascii_lowercase();
            if subtype.is_empty() {
                continue;
            }
            for requirement in &sa.requirements {
                requirement_block.insert(requirement.id.clone(), subtype.clone());
            }
        }
        let mut classifier = Classifier { graph, by_uuid, by_ref, requirement_block, satisfiers: HashMap::new(), typing: HashMap::new() };
        let mut satisfiers: HashMap<&str, Vec<&'static str>> = HashMap::new();
        let mut typing: HashMap<String, Vec<Vision>> = HashMap::new();
        for relationship in &graph.relationships {
            if relationship.kind == "Trace" {
                let kind = relationship.extra.get("traceKind").and_then(|v| v.as_str()).unwrap_or("");
                if kind.eq_ignore_ascii_case("satisfies") {
                    if let Some(source) = classifier.record(&relationship.source) {
                        satisfiers.entry(relationship.target.as_str()).or_default().push(source.kind);
                    }
                }
            }
            let data_type = relationship.extra.get("dataType").and_then(|v| v.as_str()).or_else(|| text_attr(&relationship.attributes, "data_type"));
            if let (Some(data_type), Some((placement, _))) = (data_type, kind_placement(relationship.kind)) {
                typing.entry(data_type.to_string()).or_default().push(placement.vision);
            }
        }
        classifier.satisfiers = satisfiers;
        classifier.typing = typing;
        classifier
    }

    fn record(&self, uuid: &str) -> Option<&'a ElementRecord> {
        self.by_uuid.get(uuid).map(|&i| &self.graph.elements[i])
    }

    fn owner(&self, element: &ElementRecord) -> Option<&'a ElementRecord> {
        element.owner.as_deref().and_then(|o| self.record(o))
    }

    /// The vision an element states with `vision:`.
    fn stated(attributes: &HashMap<String, AttributeValue>) -> Option<Vision> {
        text_attr(attributes, "vision").and_then(Vision::parse)
    }

    /// Where an element of the graph sits; `None` for what CESAM does not
    /// place (the model, types, traces, tests, safety analyses...).
    pub fn element(&self, element: &ElementRecord) -> Option<Placement> {
        let (default, refined) = kind_placement(element.kind)?;
        if !refined {
            return Some(default);
        }
        let vision = match element.kind {
            "Requirement" => self.requirement_vision(element),
            "StateMachine" => Some(machine_vision(&element.attributes, &element.name)),
            "State" => self.machine_of(element).map(|m| machine_vision(&m.attributes, &m.name)),
            "Scenario" => Some(self.scenario_vision(element)),
            "Fragment" | "StateFragment" => self.scenario_of(element).map(|s| self.scenario_vision(s)),
            _ => Self::stated(&element.attributes).or_else(|| self.typed_vision(element)),
        };
        Some(Placement { vision: vision.unwrap_or(default.vision), aspect: default.aspect })
    }

    /// Where an exchange (a relationship of the graph) sits.
    pub fn relationship(&self, relationship: &RelationshipRecord) -> Option<Placement> {
        let (default, refined) = kind_placement(relationship.kind)?;
        if !refined {
            return Some(default);
        }
        let vision = match relationship.kind {
            "Transition" => self.record(&relationship.source).and_then(|s| self.machine_of(s)).map(|m| machine_vision(&m.attributes, &m.name)),
            "Message" => {
                let scenario = relationship.extra.get("scenario").and_then(|v| v.as_str());
                scenario.and_then(|name| self.graph.elements.iter().find(|e| e.kind == "Scenario" && e.name == name)).map(|s| self.scenario_vision(s))
            }
            _ => Self::stated(&relationship.attributes),
        };
        Some(Placement { vision: vision.unwrap_or(default.vision), aspect: default.aspect })
    }

    /// Whether anything gives a requirement its vision (else it defaults).
    pub fn requirement_vision_known(&self, element: &ElementRecord) -> bool {
        self.requirement_vision(element).is_some()
    }

    fn requirement_vision(&self, element: &ElementRecord) -> Option<Vision> {
        if let Some(stated) = Self::stated(&element.attributes) {
            return Some(stated);
        }
        if let Some(block) = self.requirement_block.get(&element.id) {
            match block.as_str() {
                "stakeholder" | "operational" | "need" | "needs" | "mission" | "customer" => return Some(Vision::Operational),
                "functional" | "system" | "function" => return Some(Vision::Functional),
                "technical" | "constructional" | "design" | "physical" | "hardware" | "software" | "interface" => return Some(Vision::Constructional),
                _ => {}
            }
        }
        let kinds = self.satisfiers.get(element.uuid.as_str())?;
        if kinds.iter().any(|k| is_function_kind(k)) {
            Some(Vision::Functional)
        } else if kinds.iter().any(|k| is_constructional_kind(k)) {
            Some(Vision::Constructional)
        } else if kinds.iter().any(|k| matches!(*k, "Mission" | "Capability" | "OperationalCapability" | "Actor" | "SystemActor")) {
            Some(Vision::Operational)
        } else {
            None
        }
    }

    fn machine_of(&self, state: &ElementRecord) -> Option<&'a ElementRecord> {
        let mut current = self.owner(state)?;
        for _ in 0..64 {
            if current.kind == "StateMachine" {
                return Some(current);
            }
            current = self.owner(current)?;
        }
        None
    }

    fn scenario_of(&self, element: &ElementRecord) -> Option<&'a ElementRecord> {
        let mut current = self.owner(element)?;
        for _ in 0..64 {
            if current.kind == "Scenario" {
                return Some(current);
            }
            current = self.owner(current)?;
        }
        None
    }

    /// A scenario between functions is functional, between components
    /// constructional, between the system and its environment operational.
    fn scenario_vision(&self, scenario: &ElementRecord) -> Vision {
        if let Some(stated) = Self::stated(&scenario.attributes) {
            return stated;
        }
        let participants: Vec<&str> = scenario
            .extra
            .get("participants")
            .and_then(|v| v.as_array())
            .map(|items| items.iter().filter_map(|p| p.as_str()).collect())
            .unwrap_or_default();
        let kinds: Vec<&str> = participants.iter().filter_map(|p| self.by_ref.get(p)).map(|&i| self.graph.elements[i].kind).collect();
        if kinds.iter().any(|k| is_function_kind(k)) {
            Vision::Functional
        } else if kinds.iter().any(|k| is_constructional_kind(k)) {
            Vision::Constructional
        } else {
            Vision::Operational
        }
    }

    /// An exchange item or class takes the vision of the exchanges it types.
    fn typed_vision(&self, element: &ElementRecord) -> Option<Vision> {
        let visions = self.typing.get(&element.name).or_else(|| self.typing.get(&element.id))?;
        visions.iter().min().copied()
    }
}

/// The vision of a state machine: its `vision:`, else its `kind:`, else its
/// name (`Lifecycle`, `Technical configurations`), else functional modes.
pub fn machine_vision(attributes: &HashMap<String, AttributeValue>, name: &str) -> Vision {
    if let Some(stated) = text_attr(attributes, "vision").and_then(Vision::parse) {
        return stated;
    }
    if let Some(kind) = text_attr(attributes, "kind") {
        if let Some(vision) = machine_kind_vision(kind) {
            return vision;
        }
    }
    let lower = name.to_ascii_lowercase();
    if lower.contains("lifecycle") || lower.contains("life cycle") || lower.contains("cycle de vie") || lower.contains("phases") {
        Vision::Operational
    } else if lower.contains("configuration") {
        Vision::Constructional
    } else {
        Vision::Functional
    }
}

/// The values a state machine's `kind:` takes, and the vision of each.
pub const MACHINE_KINDS: [&str; 3] = ["lifecycle", "functional_modes", "technical_configurations"];

pub fn machine_kind_vision(kind: &str) -> Option<Vision> {
    match kind.trim().to_ascii_lowercase().replace([' ', '-'], "_").as_str() {
        "lifecycle" | "life_cycle" => Some(Vision::Operational),
        "functional_modes" | "modes" => Some(Vision::Functional),
        "technical_configurations" | "configurations" => Some(Vision::Constructional),
        _ => None,
    }
}

/// The `## CESAM views` section of `spec/METAMODEL.md`: every cell of the
/// grid, its view and the Capella diagrams drawing it.
pub fn grid_markdown() -> String {
    let mut out = String::from("## CESAM views\n\n");
    out.push_str("CESAM (CESAMES Systems Architecting Method) studies a system under three visions, each seen through six aspects. ");
    out.push_str("Every element and exchange of a model is placed in one cell (the `CESAM:` line of each kind above); ");
    out.push_str("`arclang table --kind cesam_grid` computes the grid of a model, the elements of each view and how mature it is.\n\n");
    out.push_str("| Vision | Aspect | CESAM view | Capella diagrams |\n|---|---|---|---|\n");
    for vision in VISIONS {
        for aspect in ASPECTS {
            let cell = view(vision, aspect);
            out.push_str(&format!("| {} | {} | {} | {} |\n", vision.label(), aspect.label(), cell.name, cell.capella));
        }
    }
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_cell_has_a_view() {
        for vision in VISIONS {
            for aspect in ASPECTS {
                let cell = view(vision, aspect);
                assert!(!cell.name.is_empty() && !cell.capella.is_empty());
            }
        }
    }

    #[test]
    fn kinds_land_in_the_cell_of_their_vision() {
        let cell = |k| kind_placement(k).map(|(p, _)| (p.vision, p.aspect));
        assert_eq!(cell("SystemFunction"), Some((Vision::Functional, Aspect::Structure)));
        assert_eq!(cell("LogicalComponent"), Some((Vision::Constructional, Aspect::Structure)));
        assert_eq!(cell("Mission"), Some((Vision::Operational, Aspect::Structure)));
        assert_eq!(cell("ComponentExchange"), Some((Vision::Constructional, Aspect::Interactions)));
        assert_eq!(cell("Trace"), None);
    }

    #[test]
    fn a_machine_kind_or_name_gives_its_vision() {
        let none = HashMap::new();
        assert_eq!(machine_vision(&none, "System lifecycle"), Vision::Operational);
        assert_eq!(machine_vision(&none, "Technical configurations"), Vision::Constructional);
        assert_eq!(machine_vision(&none, "Door"), Vision::Functional);
        let mut stated = HashMap::new();
        stated.insert("kind".to_string(), AttributeValue::String("lifecycle".into()));
        assert_eq!(machine_vision(&stated, "Door"), Vision::Operational);
    }

    #[test]
    fn visions_parse_in_english_and_french() {
        assert_eq!(Vision::parse("Fonctionnelle"), Some(Vision::Functional));
        assert_eq!(Vision::parse("organique"), Some(Vision::Constructional));
        assert_eq!(Vision::parse("nope"), None);
    }
}

/// The CESAM view a viewpoint diagram draws: `oab`/`sab` the environment,
/// `cap` the use cases, `lab`/`pab` the constructional interactions, `pbs`
/// the product breakdown, `cdb` the interfaces, `req` the requirements of every vision (first), a
/// machine (`msm:<name>`) or a scenario (`es:<name>`) the states or the
/// scenarios of its own vision. `None` for the SysML v2 views and the
/// parametric one, which CESAM does not name.
pub fn diagram_view(id: &str, machine_vision: impl Fn(&str) -> Option<Vision>, scenario_vision: impl Fn(&str) -> Option<Vision>) -> Option<View> {
    use Aspect::*;
    use Vision::*;
    let (kind, name) = id.split_once(':').unwrap_or((id, ""));
    let (vision, aspect) = match kind {
        "oab" | "sab" => (Operational, Interactions),
        "cap" => (Operational, Structure),
        "lab" | "pab" => (Constructional, Interactions),
        "pbs" => (Constructional, Structure),
        "cdb" => (Functional, Interfaces),
        // One requirements block's view, in the vision of its block; every requirement at once, in the operational cell.
        "req" => {
            let block = name.to_ascii_lowercase();
            let vision = if block.starts_with("functional") || block.starts_with("system") {
                Functional
            } else if block.starts_with("technical") {
                Constructional
            } else {
                Operational
            };
            (vision, Requirements)
        }
        "msm" => (machine_vision(name).unwrap_or(Functional), States),
        "es" => (scenario_vision(name).unwrap_or(Operational), Scenarios),
        _ => return None,
    };
    Some(view(vision, aspect))
}

/// The CESAM view each diagram of a set draws, in the set's order: what
/// the workbench's CESAM grid opens from each of its cells.
pub fn diagram_cells(set: &super::diagram::DiagramSet, ast: &Model, graph: &ElementGraph) -> Vec<(String, View)> {
    let classifier = Classifier::new(graph, ast);
    let machine = |name: &str| ast.state_machines.iter().find(|m| m.name == name).map(|m| machine_vision(&m.attributes, &m.name));
    let scenario = |name: &str| graph.elements.iter().find(|e| e.kind == "Scenario" && e.name == name).and_then(|e| classifier.element(e)).map(|p| p.vision);
    set.diagrams.iter().filter_map(|d| diagram_view(&d.id, machine, scenario).map(|view| (d.id.clone(), view))).collect()
}

/// Keep the diagrams of a set that draw a CESAM view, in the order of the
/// grid, titled `<view> — <diagram>`; their diagnostics follow them.
pub fn arrange_diagrams(set: &mut super::diagram::DiagramSet, ast: &Model, graph: &ElementGraph) {
    let classifier = Classifier::new(graph, ast);
    let machine = |name: &str| ast.state_machines.iter().find(|m| m.name == name).map(|m| machine_vision(&m.attributes, &m.name));
    let scenario = |name: &str| graph.elements.iter().find(|e| e.kind == "Scenario" && e.name == name).and_then(|e| classifier.element(e)).map(|p| p.vision);
    // A view of each requirements block, when there are several, stands in its cell; not every requirement at once again.
    let by_block = set.diagrams.iter().any(|d| d.id.starts_with("req:"));
    let mut placed: Vec<(usize, super::diagram::Diagram)> = std::mem::take(&mut set.diagrams)
        .into_iter()
        .filter(|d| !(by_block && d.id == "req"))
        .filter_map(|mut diagram| {
            let cell = diagram_view(&diagram.id, machine, scenario)?;
            let rank = VISIONS.iter().position(|v| *v == cell.vision).unwrap_or(0) * ASPECTS.len() + ASPECTS.iter().position(|a| *a == cell.aspect).unwrap_or(0);
            diagram.title = if diagram.id == "req" {
                format!("Expected properties of the three visions — {}", diagram.title)
            } else {
                format!("{} — {} ({})", cell.name, diagram.title, cell.vision.label())
            };
            Some((rank, diagram))
        })
        .collect();
    placed.sort_by_key(|(rank, _)| *rank);
    set.diagrams = placed.into_iter().map(|(_, d)| d).collect();
}
