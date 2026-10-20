use std::collections::HashMap;
use serde::{Serialize, Deserialize};
pub use super::constraint::{BinOp, Expr};
pub use super::quantity::{Quantity, QuantityError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    /// The header identifier: `model <id> { ... }`. The model's `name:`
    /// (else the header) is `attributes["name"]`.
    #[serde(default)]
    pub id: String,
    /// Model-level metadata (name, version, author, ...) from the model header
    /// and its `metadata` block.
    #[serde(default)]
    pub attributes: HashMap<String, AttributeValue>,
    /// `import "path.arc"` declarations, resolved relative to the importing
    /// file by `Compiler::compile_file` (a string-compiled model must not
    /// have any). Kept in declaration order.
    #[serde(default)]
    pub imports: Vec<String>,
    pub operational_analysis: Vec<OperationalAnalysis>,
    pub system_analysis: Vec<SystemAnalysis>,
    pub logical_architecture: Vec<LogicalArchitecture>,
    pub physical_architecture: Vec<PhysicalArchitecture>,
    pub epbs: Vec<Epbs>,
    pub safety_analysis: Vec<SafetyAnalysis>,
    /// Cybersecurity analyses: the threats and primary assets of a level
    /// (the `CybersecurityPkg` of the Capella Cybersecurity viewpoint).
    #[serde(default)]
    pub security_analysis: Vec<SecurityAnalysis>,
    /// The scales of the security needs and the threat kinds of the model
    /// (the `CybersecurityConfiguration`); at most one per model.
    #[serde(default)]
    pub security_scales: Option<SecurityScales>,
    pub traces: Vec<Trace>,
    pub state_machines: Vec<StateMachine>,
    pub scenarios: Vec<Scenario>,
    pub exchange_items: Vec<ExchangeItem>,
    pub data_types: Vec<DataType>,
    #[serde(default)]
    pub classes: Vec<ClassDef>,
    /// Associations between classes, declared beside them.
    #[serde(default)]
    pub associations: Vec<Association>,
    /// Verification cases tracing to requirements (V&V).
    #[serde(default)]
    pub test_cases: Vec<TestCase>,
    /// Product-line configurations: the variants each selects.
    #[serde(default)]
    pub configurations: Vec<Configuration>,
    /// Dimension-checked assertions over typed attributes.
    #[serde(default)]
    pub constraints: Vec<Constraint>,
    /// User-defined types (`type Name extends Base { ... }`).
    #[serde(default)]
    pub types: Vec<TypeDef>,
    /// Diagrams declared by the author: a named view of chosen elements,
    /// as Capella diagrams show what was put on them rather than a layer.
    #[serde(default)]
    pub diagrams: Vec<DiagramDecl>,
    /// Tables declared by the author (`table "Name" { kind: ... }`): live
    /// views computed by the compiler (see `compiler::tables`). // tables hook
    #[serde(default)]
    pub tables: Vec<super::tables::TableDecl>, // tables hook
}

/// `diagram "Name" { view: lab shows: [A, B] context: true }`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramDecl {
    pub name: String,
    /// The view it is cut from: `oab`, `sab`, `lab`, `pab`, `cap`, `cdb`, `pbs`.
    pub view: String,
    /// Elements drawn, by id or name; what holds them is drawn around them.
    pub shows: Vec<String>,
    /// Also draw the elements an exchange ties to a shown one.
    pub context: bool,
    pub description: Option<String>,
    pub attributes: HashMap<String, AttributeValue>,
}

impl Model {
    pub fn new() -> Self {
        Self {
            id: String::new(),
            attributes: HashMap::new(),
            imports: Vec::new(),
            operational_analysis: Vec::new(),
            system_analysis: Vec::new(),
            logical_architecture: Vec::new(),
            physical_architecture: Vec::new(),
            epbs: Vec::new(),
            safety_analysis: Vec::new(),
            security_analysis: Vec::new(),
            security_scales: None,
            traces: Vec::new(),
            state_machines: Vec::new(),
            scenarios: Vec::new(),
            exchange_items: Vec::new(),
            data_types: Vec::new(),
            classes: Vec::new(),
            associations: Vec::new(),
            test_cases: Vec::new(),
            configurations: Vec::new(),
            constraints: Vec::new(),
            types: Vec::new(),
            diagrams: Vec::new(),
            tables: Vec::new(), // tables hook
        }
    }
    
    /// Merge an imported model into this one: every collection is appended
    /// in import order; model-header attributes of the ROOT file win (an
    /// imported fragment only fills keys the root did not set). Identity
    /// collisions across files surface through the semantic analyzer's
    /// duplicate-id warnings, exactly as within a single file.
    pub fn merge(&mut self, other: Model) {
        for (key, value) in other.attributes {
            self.attributes.entry(key).or_insert(value);
        }
        self.operational_analysis.extend(other.operational_analysis);
        self.system_analysis.extend(other.system_analysis);
        self.logical_architecture.extend(other.logical_architecture);
        self.physical_architecture.extend(other.physical_architecture);
        self.epbs.extend(other.epbs);
        self.safety_analysis.extend(other.safety_analysis);
        self.security_analysis.extend(other.security_analysis);
        if self.security_scales.is_none() {
            self.security_scales = other.security_scales;
        }
        self.traces.extend(other.traces);
        self.state_machines.extend(other.state_machines);
        self.scenarios.extend(other.scenarios);
        self.exchange_items.extend(other.exchange_items);
        self.data_types.extend(other.data_types);
        self.classes.extend(other.classes);
        self.test_cases.extend(other.test_cases);
        self.configurations.extend(other.configurations);
        self.constraints.extend(other.constraints);
        self.types.extend(other.types);
        self.tables.extend(other.tables); // tables hook
    }

    /// Export the model to JSON string for diagram rendering
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
    
    /// Export the model to JSON value for programmatic access
    pub fn to_json_value(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }
    
    /// Export the model to compact JSON (no pretty printing)
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalAnalysis {
    /// `id:` of the level block when written (Capella's architecture id).
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub attributes: HashMap<String, AttributeValue>,
    pub actors: Vec<Actor>,
    pub entities: Vec<OperationalEntity>,
    pub capabilities: Vec<OperationalCapability>,
    pub activities: Vec<OperationalActivity>,
    pub exchanges: Vec<OperationalExchange>,
    pub capability_associations: Vec<CapabilityAssociation>,
    pub traces: Vec<Trace>,
    /// Ordered activity/interaction paths fulfilling an operational capability.
    #[serde(default)]
    pub processes: Vec<FunctionalChain>,
    /// Supports linking operational entities, followed by interactions.
    #[serde(default)]
    pub communication_means: Vec<OperationalExchange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalEntity {
    pub id: String,
    pub name: String,
    pub entity_type: EntityType,
    pub activities: Vec<OperationalActivity>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub entity_type: EntityType,
    pub icon: String,
    pub description: Option<String>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntityType {
    Actor,
    System,
    Environment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    pub name: String,
    pub id: Option<String>,
    pub icon: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalCapability {
    pub id: String,
    pub name: String,
    pub level: CapabilityLevel,
    pub color: Option<String>,
    pub stereotype: Option<String>,
    pub children: Vec<OperationalCapability>,
    /// Id of the capability this one is declared inside, when nested.
    #[serde(default)]
    pub parent: Option<String>,
    /// Elements (actors, entities, activities, processes) involved.
    #[serde(default)]
    pub involves: Vec<String>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CapabilityLevel {
    Mission,
    Capability,
    SubCapability,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalActivity {
    pub id: String,
    pub name: String,
    pub performed_by: String,
    pub category: String,
    pub icon: String,
    pub color: String,
    pub sub_activities: Vec<OperationalActivity>,
    /// Ports of the activity (Capella gives activities inputs and outputs, as functions).
    #[serde(default)]
    pub ports: Vec<FunctionPort>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalExchange {
    pub from: String,
    pub to: String,
    pub data_type: String,
    pub label: Option<String>,
    pub protocol: Option<String>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityAssociation {
    pub from: String,
    pub to: String,
    pub association_type: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemAnalysis {
    /// `id:` of the level block when written (Capella's architecture id).
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub attributes: HashMap<String, AttributeValue>,
    pub requirements: Vec<Requirement>,
    pub functions: Vec<SystemFunction>,
    pub components: Vec<SystemComponent>,
    pub external_actors: Vec<ExternalActor>,
    pub functional_exchanges: Vec<FunctionalExchange>,
    /// Exchanges between the ports of the system, its components and its actors.
    #[serde(default)]
    pub component_exchanges: Vec<ComponentExchange>,
    #[serde(default)]
    pub missions: Vec<Mission>,
    #[serde(default)]
    pub capabilities: Vec<Capability>,
    #[serde(default)]
    pub functional_chains: Vec<FunctionalChain>,
}

/// High-level goal the system contributes to (Arcadia: System Mission).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    pub id: String,
    pub name: String,
    pub attributes: HashMap<String, AttributeValue>,
}

/// Expected ability of the system to supply a service fulfilling missions.
/// Also used for LA/PA CapabilityRealization (with `realizes` set).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub id: String,
    pub name: String,
    /// Elements (functions, actors, chains) involved in this capability.
    pub involves: Vec<String>,
    /// For realizations: the higher-level capability being realized.
    pub realizes: Option<String>,
    /// The mission this capability contributes to, when declared.
    pub mission: Option<String>,
    /// Id of the capability this one is declared inside, when nested.
    #[serde(default)]
    pub parent: Option<String>,
    pub attributes: HashMap<String, AttributeValue>,
}

/// Ordered set of references to functions and exchanges describing one
/// dataflow path, in the context of a capability (Arcadia: Functional Chain).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionalChain {
    pub id: String,
    pub name: String,
    /// Ordered references to the involved functions, exchanges and chains
    /// (a chain involving another chain references it, as Capella does).
    pub involves: Vec<String>,
    /// Control nodes branching the chain: AND, OR, ITERATE.
    #[serde(default)]
    pub controls: Vec<ControlNode>,
    /// Sequence links between steps and control nodes; without any, the
    /// chain is the plain order of `involves`.
    #[serde(default)]
    pub sequences: Vec<SequenceLink>,
    pub attributes: HashMap<String, AttributeValue>,
}

/// A control node of a functional chain (Capella: ControlNode).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlNode {
    /// `id:` when written, else `<chain id>.<name>`.
    pub id: String,
    pub name: String,
    pub kind: ControlKind,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ControlKind {
    And,
    Or,
    Iterate,
}

impl ControlKind {
    pub fn parse(text: &str) -> Option<ControlKind> {
        match text {
            "and" | "AND" => Some(ControlKind::And),
            "or" | "OR" => Some(ControlKind::Or),
            "iterate" | "ITERATE" => Some(ControlKind::Iterate),
            _ => None,
        }
    }

    pub fn keyword(self) -> &'static str {
        match self {
            ControlKind::And => "and",
            ControlKind::Or => "or",
            ControlKind::Iterate => "iterate",
        }
    }
}

/// A sequence link of a functional chain: `sequence "A" -> "B" { condition: "..." }`,
/// each end a step of the chain or one of its control nodes (Capella: SequenceLink).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceLink {
    /// `id:` when written, else `<chain id>.<from>-><to>`.
    pub id: String,
    pub from: String,
    pub to: String,
    pub condition: Option<String>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemFunction {
    pub id: String,
    pub name: String,
    pub category: FunctionCategory,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub ports: Vec<FunctionPort>,
    pub sub_functions: Vec<SystemFunction>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FunctionCategory {
    Environmental,
    System,
    Management,
    Control,
    Interaction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionPort {
    /// `id:` when written (Capella keeps ids on ports), else empty: the
    /// port is then known as `Owner.name`.
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub direction: PortDirection,
    pub port_type: PortType,
    pub data_type: String,
    #[serde(default)]
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PortDirection {
    In,
    Out,
    InOut,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PortType {
    Data,
    Control,
    Event,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalActor {
    pub id: String,
    pub name: String,
    pub color: String,
    #[serde(default)]
    pub ports: Vec<ComponentPort>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionalExchange {
    /// The written `id`, else the label, else `from -> to`.
    #[serde(default)]
    pub id: String,
    pub from_port: String,
    pub to_port: String,
    pub data_type: String,
    pub label: Option<String>,
    /// Every attribute written (exchange items, categories, description...).
    #[serde(default)]
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemComponent {
    pub name: String,
    #[serde(default)]
    pub ports: Vec<ComponentPort>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalArchitecture {
    /// `id:` of the level block when written (Capella's architecture id).
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub attributes: HashMap<String, AttributeValue>,
    pub components: Vec<LogicalComponent>,
    /// Functions of the level that no component performs (a breakdown's parents).
    #[serde(default)]
    pub functions: Vec<LogicalFunction>,
    pub interfaces: Vec<LogicalInterface>,
    pub component_exchanges: Vec<ComponentExchange>,
    /// Exchanges between the functions of this level.
    #[serde(default)]
    pub functional_exchanges: Vec<FunctionalExchange>,
    pub unallocated_functions: Vec<String>,
    #[serde(default)]
    pub capability_realizations: Vec<Capability>,
    #[serde(default)]
    pub functional_chains: Vec<FunctionalChain>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalComponent {
    pub id: String,
    pub name: String,
    pub component_type: String,
    pub color: Option<String>,
    pub sub_components: Vec<LogicalComponent>,
    pub allocated_functions: Vec<String>,
    pub ports: Vec<ComponentPort>,
    pub functions: Vec<LogicalFunction>,
    pub interfaces_in: Vec<InterfaceDefinition>,
    pub interfaces_out: Vec<InterfaceDefinition>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentPort {
    /// `id:` when written, else empty (the port is known as `Owner.name`).
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub direction: PortDirection,
    pub interface_type: String,
    #[serde(default)]
    pub attributes: HashMap<String, AttributeValue>,
    /// Ports this port holds (`port inout "nuts" { port in "lugNut" }`),
    /// named `Owner.port.sub` by an exchange.
    #[serde(default)]
    pub ports: Vec<ComponentPort>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentExchange {
    /// The written `id`, else the label, else `from -> to`.
    #[serde(default)]
    pub id: String,
    pub from_port: String,
    pub to_port: String,
    pub exchange_item: String,
    pub label: Option<String>,
    /// Every attribute written (exchange items, categories, description...).
    #[serde(default)]
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceDefinition {
    pub name: String,
    pub protocol: Option<String>,
    pub format: Option<String>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A function of the logical or physical level: declared inside the
/// component that performs it, with its own ports and sub-functions, and
/// `realizes:` the function of the level above.
pub struct LogicalFunction {
    /// The written `id`, else the name.
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub ports: Vec<FunctionPort>,
    #[serde(default)]
    pub sub_functions: Vec<LogicalFunction>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalInterface {
    pub name: String,
    pub from: String,
    pub to: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalArchitecture {
    /// `id:` of the level block when written (Capella's architecture id).
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub attributes: HashMap<String, AttributeValue>,
    pub nodes: Vec<PhysicalNode>,
    /// Functions of the level that no behaviour component performs.
    #[serde(default)]
    pub functions: Vec<LogicalFunction>,
    pub links: Vec<PhysicalLink>,
    pub physical_exchanges: Vec<PhysicalExchange>,
    /// Exchanges between the functions of this level.
    #[serde(default)]
    pub functional_exchanges: Vec<FunctionalExchange>,
    /// Ordered physical functions realizing one dataflow path.
    #[serde(default)]
    pub functional_chains: Vec<FunctionalChain>,
    /// Realizations of the logical capability realizations by the physical components.
    #[serde(default)]
    pub capability_realizations: Vec<Capability>,
    /// Ordered sets of physical links routing behavioural exchanges (Arcadia).
    #[serde(default)]
    pub paths: Vec<PhysicalPath>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalPath {
    #[serde(default)]
    pub id: String,
    pub name: String,
    /// Ordered references to the physical links forming the path.
    pub involves: Vec<String>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalNode {
    pub id: String,
    pub name: String,
    pub node_type: NodeType,
    pub color: Option<String>,
    pub processor: Option<String>,
    pub memory: Option<String>,
    pub behavior_components: Vec<BehaviorComponent>,
    pub hardware_components: Vec<HardwareComponent>,
    /// The physical functions the node itself performs.
    #[serde(default)]
    pub functions: Vec<LogicalFunction>,
    pub deployments: Vec<Deployment>,
    /// Oriented component ports of the node itself (`port in|out "name"`):
    /// a node, an actor node above all, may exchange data as a component
    /// does, so a `physical_exchange` may end on `Node.port`.
    #[serde(default)]
    pub component_ports: Vec<ComponentPort>,
    /// Physical ports are NOT oriented (Arcadia).
    #[serde(default)]
    pub ports: Vec<PhysicalPort>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalPort {
    pub name: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeType {
    Hardware,
    Software,
    SystemOfSystems,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorComponent {
    pub id: String,
    pub name: String,
    /// The logical component this behaviour realizes (`allocated_component:`).
    #[serde(default)]
    pub allocated_component: Option<String>,
    /// The functions it performs by name (`allocated_functions:`), the legacy
    /// form; functions declared inside it are in `functions`.
    pub allocated_functions: Vec<String>,
    pub color: Option<String>,
    /// The physical functions the component performs, declared inside it.
    #[serde(default)]
    pub functions: Vec<LogicalFunction>,
    /// Oriented ports, as a logical component has them.
    #[serde(default)]
    pub ports: Vec<ComponentPort>,
    #[serde(default)]
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareComponent {
    pub id: String,
    pub name: String,
    pub hw_type: String,
    pub specs: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub component: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalLink {
    #[serde(default)]
    pub name: String,
    pub from: String,
    pub to: String,
    pub protocol: String,
    pub bandwidth: Option<String>,
    pub color: Option<String>,
    pub connections: Vec<String>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalExchange {
    /// The written `id`, else the label, else `from -> to`.
    #[serde(default)]
    pub id: String,
    pub from: String,
    pub to: String,
    pub via: Option<String>,
    pub message_type: String,
    pub frequency: Option<String>,
    pub label: Option<String>,
    /// Every attribute written (exchange items, categories, description...).
    #[serde(default)]
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Epbs {
    /// `id:` of the block when written (Capella's EPBS architecture id).
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub attributes: HashMap<String, AttributeValue>,
    pub systems: Vec<EpbsSystem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpbsSystem {
    pub name: String,
    pub subsystems: Vec<EpbsSubsystem>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpbsSubsystem {
    pub name: String,
    pub items: Vec<EpbsItem>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpbsItem {
    pub name: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyAnalysis {
    pub hazards: Vec<Hazard>,
    pub fmea: Vec<FmeaEntry>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hazard {
    pub name: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FmeaEntry {
    pub name: String,
    pub attributes: HashMap<String, AttributeValue>,
}

/// `security_analysis { level: "sa" threat … asset … }`: the threats and
/// primary assets analysed at one level. `level` names the architecture
/// (`oa`, `sa`, `la`, `pa`) whose cybersecurity package holds them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAnalysis {
    /// Written id of the package; empty when not written.
    #[serde(default)]
    pub id: String,
    pub threats: Vec<Threat>,
    pub assets: Vec<PrimaryAsset>,
    pub attributes: HashMap<String, AttributeValue>,
}

impl SecurityAnalysis {
    /// The architecture level (`oa`, `sa`, `la`, `pa`) when written.
    pub fn level(&self) -> Option<&str> {
        self.attributes.get("level").and_then(|v| v.as_string())
    }
}

/// `threat "Name" { kind: level: applies_to: involves: … }`: a threat is a
/// capability of an attacker (Capella: an AbstractCapability), so it may
/// own scenarios (`owner:` of a scenario) and realize a threat of the
/// level above through an ordinary `realizes` trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Threat {
    /// The id, written or defaulted to the name.
    pub id: String,
    pub name: String,
    /// A threat kind of the model's `security_scales.threat_kinds`.
    pub kind: Option<String>,
    /// Primary assets the threat applies to (ThreatApplication).
    pub applies_to: Vec<String>,
    /// Components, entities or actors involved (ThreatInvolvement).
    pub involves: Vec<String>,
    pub attributes: HashMap<String, AttributeValue>,
}

/// What a primary asset is: a service, a piece of information or a
/// business concern (Capella's three PrimaryAsset metaclasses).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetKind {
    Functional,
    Information,
    Enterprise,
}

impl AssetKind {
    pub fn parse(text: &str) -> Option<AssetKind> {
        match text.to_ascii_lowercase().as_str() {
            "functional" => Some(AssetKind::Functional),
            "information" => Some(AssetKind::Information),
            "enterprise" => Some(AssetKind::Enterprise),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AssetKind::Functional => "functional",
            AssetKind::Information => "information",
            AssetKind::Enterprise => "enterprise",
        }
    }
}

/// `asset "Name" { kind: "functional" members: [...] }`: a primary asset
/// to protect; its members are the functions, exchange items or classes
/// that make it up (PrimaryAssetMember).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimaryAsset {
    /// The id, written or defaulted to the name.
    pub id: String,
    pub name: String,
    pub kind: AssetKind,
    pub members: Vec<String>,
    pub attributes: HashMap<String, AttributeValue>,
}

/// `security_scales { confidentiality: ["0", …] … threat_kinds: [...] }`:
/// the ordered literals of each security need and the kinds a threat may
/// have. Every entry is a list of names; `id` is the configuration's.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScales {
    #[serde(default)]
    pub id: String,
    pub attributes: HashMap<String, AttributeValue>,
}

impl SecurityScales {
    /// The security needs a scale may be declared for, in Capella's order.
    pub const NEEDS: [&'static str; 4] = ["confidentiality", "integrity", "availability", "traceability"];

    /// The literal names of a scale (`confidentiality`, …, `threat_kinds`).
    pub fn scale(&self, key: &str) -> Option<Vec<String>> {
        match self.attributes.get(key)? {
            AttributeValue::List(items) => Some(items.iter().map(|v| v.display()).collect()),
            single => Some(vec![single.display()]),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub from: String,
    pub to: String,
    pub trace_type: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttributeValue {
    String(String),
    /// A number written with a decimal point (`800.0`): a real value, kept
    /// apart from [`AttributeValue::Integer`] so a model round-trips as written.
    Number(f64),
    /// A number written without a decimal point (`800`).
    Integer(i64),
    Boolean(bool),
    /// A number with a unit (`latency: 25 ms`). Typed at parse time: the
    /// unit is always a known symbol, so the dimension is never in doubt.
    Quantity(Quantity),
    List(Vec<AttributeValue>),
    Map(HashMap<String, AttributeValue>),
}

/// A decimal number as it is written in a model: a whole value keeps its
/// decimal point (`800.0`), so it stays a decimal when read back.
pub fn decimal_literal(n: f64) -> String {
    if n.is_finite() && n.fract() == 0.0 && n.abs() < 1e15 { format!("{n:.1}") } else { n.to_string() }
}

impl AttributeValue {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            AttributeValue::String(s) => Some(s),
            _ => None,
        }
    }
    
    /// The numeric value, whether written as an integer or a decimal.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            AttributeValue::Number(n) => Some(*n),
            AttributeValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Same as [`AttributeValue::as_number`].
    pub fn as_f64(&self) -> Option<f64> {
        self.as_number()
    }

    /// The value when it was written as an integer.
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            AttributeValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// The value as a typed quantity. Accepts the typed literal form and,
    /// for backward compatibility, the legacy string form (`"25 ms"`).
    /// A bare number is NOT a quantity (its unit is unknown) and yields
    /// `Some(Err(MissingUnit))` so callers can report it precisely.
    pub fn as_quantity(&self) -> Option<Result<Quantity, QuantityError>> {
        match self {
            AttributeValue::Quantity(q) => Some(Ok(q.clone())),
            AttributeValue::String(s) => Some(Quantity::parse(s)),
            AttributeValue::Number(n) => Some(Err(QuantityError::MissingUnit(*n))),
            AttributeValue::Integer(i) => Some(Err(QuantityError::MissingUnit(*i as f64))),
            _ => None,
        }
    }

    /// Human-readable rendering used by exporters and the explorer.
    pub fn display(&self) -> String {
        match self {
            AttributeValue::String(s) => s.clone(),
            AttributeValue::Number(n) => decimal_literal(*n),
            AttributeValue::Integer(i) => i.to_string(),
            AttributeValue::Boolean(b) => b.to_string(),
            AttributeValue::Quantity(q) => q.to_string(),
            AttributeValue::List(items) => {
                format!("[{}]", items.iter().map(|v| v.display()).collect::<Vec<_>>().join(", "))
            }
            AttributeValue::Map(map) => {
                let mut keys: Vec<_> = map.keys().collect();
                keys.sort();
                format!(
                    "{{{}}}",
                    keys.iter().map(|k| format!("{}: {}", k, map[*k].display())).collect::<Vec<_>>().join(", ")
                )
            }
        }
    }
}

// Behavioral Models

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMachine {
    /// `id:` when written, else empty: a machine is known by its name.
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub attributes: HashMap<String, AttributeValue>,
    pub initial_state: String,
    /// Every state of the machine, at the top level of any region.
    pub states: Vec<State>,
    /// Every transition of the machine, whatever region declares it.
    pub transitions: Vec<Transition>,
    /// The top-level regions written as `region` blocks (a machine with
    /// several, each with its own initial state); their states and
    /// transitions are in the flat lists too.
    #[serde(default)]
    pub regions: Vec<Region>,
}

/// A top-level region of a state machine written explicitly: `region "Name"
/// { initial: "A" state "A" {} ... }` (Capella: Region).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    /// `id:` when written, else `<machine>.<name>`.
    pub id: String,
    pub name: String,
    pub initial: String,
    pub states: Vec<State>,
    pub transitions: Vec<Transition>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// The written `id`, when the state has one; a trace names it by it.
    #[serde(default)]
    pub id: String,
    pub name: String,
    /// Arcadia distinguishes Mode (chosen behaviour) from State (undergone).
    #[serde(default)]
    pub kind: StateKind,
    pub entry_actions: Vec<String>,
    pub exit_actions: Vec<String>,
    pub internal_transitions: Vec<String>,
    pub sub_states: Vec<State>,
    /// The transitions declared in the region this state opens; they are
    /// also in the machine's flat list, which every check and view reads.
    #[serde(default)]
    pub transitions: Vec<Transition>,
    pub color: Option<String>,
    /// The initial state of the region this state opens (`initial:`), when any.
    #[serde(default)]
    pub initial: String,
    /// Every attribute written (`kind:` for a pseudo-state, property values...).
    #[serde(default)]
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum StateKind {
    #[default]
    State,
    Mode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    /// The written `id`, when the transition has one.
    #[serde(default)]
    pub id: String,
    pub from: String,
    pub to: String,
    pub trigger: String,
    pub guard: Option<String>,
    pub action: Option<String>,
    pub timing: Option<String>,
    pub priority: Option<String>,
    /// Every attribute written (`events:`, property values...).
    #[serde(default)]
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    /// `id:` when written, else empty: a scenario is known by its name.
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub attributes: HashMap<String, AttributeValue>,
    pub participants: Vec<Participant>,
    pub messages: Vec<Message>,
    pub fragments: Vec<CombinedFragment>,
    /// The states, modes and functions asserted on lifelines, in order.
    #[serde(default)]
    pub invariants: Vec<StateInvariant>,
    pub timing_constraints: Vec<TimingConstraint>,
}

/// A state, mode or function asserted on a lifeline between messages
/// (Capella: StateFragment): `state "Lifeline" "Armed" { ... }`. It spans the
/// messages written in its body; without any, it is an instant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateInvariant {
    /// `id:` when written, else `<scenario>.<lifeline>.<name>#<n>`.
    pub id: String,
    pub lifeline: String,
    /// The state, mode or function, by id or unambiguous name.
    pub name: String,
    pub kind: InvariantKind,
    /// First and last message of the span, as indexes into the scenario's
    /// messages (`end < start` for an instant).
    pub start: usize,
    pub end: usize,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InvariantKind {
    State,
    Function,
}

impl InvariantKind {
    pub fn keyword(self) -> &'static str {
        match self {
            InvariantKind::State => "state",
            InvariantKind::Function => "function",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: String,
    pub name: String,
    pub participant_type: ParticipantType,
    pub lifeline_color: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParticipantType {
    Actor,
    Component,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// `id:` when written, else empty.
    #[serde(default)]
    pub id: String,
    pub from: String,
    pub to: String,
    pub label: String,
    pub message_type: MessageType,
    pub activation: bool,
    pub timing: Option<String>,
    pub params: Option<String>,
    /// Every attribute written: `kind:` (Capella's message kind), `exchange:`
    /// (the functional or component exchange the message carries)...
    #[serde(default)]
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageType {
    Synchronous,
    Asynchronous,
    Return,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FragmentType {
    Par,
    Opt,
    Loop,
    Alt,
    Seq,
    Critical,
}

impl FragmentType {
    pub fn parse(text: &str) -> Option<FragmentType> {
        match text {
            "par" => Some(FragmentType::Par),
            "opt" => Some(FragmentType::Opt),
            "loop" => Some(FragmentType::Loop),
            "alt" => Some(FragmentType::Alt),
            "seq" => Some(FragmentType::Seq),
            "critical" => Some(FragmentType::Critical),
            _ => None,
        }
    }

    pub fn keyword(self) -> &'static str {
        match self {
            FragmentType::Par => "par",
            FragmentType::Opt => "opt",
            FragmentType::Loop => "loop",
            FragmentType::Alt => "alt",
            FragmentType::Seq => "seq",
            FragmentType::Critical => "critical",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedFragment {
    pub fragment_type: FragmentType,
    pub label: String,
    pub condition: Option<String>,
    /// The operands, in order; a fragment written without `operand` blocks
    /// has one, unlabelled, holding its body.
    pub operands: Vec<FragmentOperand>,
    /// First and last message of the fragment, as indexes into the
    /// scenario's messages (`end < start` when it holds none).
    #[serde(default)]
    pub start: usize,
    #[serde(default)]
    pub end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentOperand {
    pub label: String,
    pub messages: Vec<Message>,
    /// First and last message of the operand, as indexes into the
    /// scenario's messages (`end < start` when it holds none).
    #[serde(default)]
    pub start: usize,
    #[serde(default)]
    pub end: usize,
    /// Fragments nested in this operand.
    #[serde(default)]
    pub fragments: Vec<CombinedFragment>,
    /// The invariants written in this operand, as indexes into the
    /// scenario's `invariants` (an invariant outside every operand is at
    /// the scenario's top level).
    #[serde(default)]
    pub invariants: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingConstraint {
    pub from_message: String,
    pub to_message: String,
    pub max_duration: String,
    pub requirement: Option<String>,
}

// Data Model

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeItem {
    /// Stable identity (defaults to the name).
    #[serde(default)]
    pub id: String,
    pub name: String,
    /// Exchange mechanism: EVENT | FLOW | OPERATION | DATA | SHARED_DATA.
    pub stereotype: String,
    pub attributes: Vec<DataAttribute>,
    /// References to the data elements (classes) grouped by this item.
    #[serde(default)]
    pub elements: Vec<String>,
    /// Every attribute written (status, description...); `attributes` are
    /// the item's fields.
    #[serde(default)]
    pub properties: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAttribute {
    pub name: String,
    pub attr_type: String,
    pub default_value: Option<String>,
    pub enumeration: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataType {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub base_type: Option<String>,
    /// Unit symbol of the values (`m/s`), when declared.
    #[serde(default)]
    pub unit: Option<String>,
    pub enumeration_values: Option<Vec<EnumValue>>,
    /// Every attribute written (status, unit_name, description...).
    #[serde(default)]
    pub attributes: HashMap<String, AttributeValue>,
}

/// A verification case: how one or more requirements are verified.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    #[serde(default)]
    pub id: String,
    pub name: String,
    /// Requirements verified by this case.
    pub verifies: Vec<String>,
    /// Verification method: test | analysis | inspection | demonstration.
    pub method: String,
    pub attributes: HashMap<String, AttributeValue>,
}

/// `configuration "City" { selects: ["LC-4CYL", "LC-MANUAL"] }`: one
/// product of a product line, the variant it selects in each variation
/// (a component `variation: true` holding components `variant: true`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub id: String,
    pub name: String,
    pub selects: Vec<String>,
    pub attributes: HashMap<String, AttributeValue>,
}

/// A reusable definition: typed attributes, ports, and the attributes its
/// instances must provide. Elements reference it with `is: "Name"`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDef {
    pub name: String,
    /// The type this one specializes.
    pub extends: Option<String>,
    /// Attributes declared by this type itself (not the inherited ones).
    pub attributes: HashMap<String, AttributeValue>,
    #[serde(default)]
    pub ports: Vec<ComponentPort>,
    /// Attributes every instance must provide.
    #[serde(default)]
    pub required: Vec<String>,
}

/// A named assertion over typed model attributes (Arcadia: Constraint).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    #[serde(default)]
    pub id: String,
    pub name: String,
    /// The asserted comparison.
    pub expression: Expr,
    pub attributes: HashMap<String, AttributeValue>,
}

/// A structured data element produced/used by functions (Arcadia: Class).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassDef {
    #[serde(default)]
    pub id: String,
    pub name: String,
    /// The class this one specializes (`extends: Base`), when any.
    #[serde(default)]
    pub extends: Option<String>,
    /// Fields: name -> type.
    pub fields: Vec<DataAttribute>,
    pub attributes: HashMap<String, AttributeValue>,
}

/// An association between two classes, as an element of its own (Arcadia:
/// Association). `from` owns the navigable end, `role` names the field on
/// it typed by `to`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Association {
    /// `id:` when written, else `<name>:<from>-><to>`.
    pub id: String,
    pub name: String,
    pub from: String,
    pub to: String,
    pub role: Option<String>,
    pub kind: AssociationKind,
    pub multiplicity: Option<String>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssociationKind {
    Association,
    Aggregation,
    Composition,
}

impl AssociationKind {
    pub fn parse(text: &str) -> Option<AssociationKind> {
        match text {
            "association" | "ASSOCIATION" => Some(AssociationKind::Association),
            "aggregation" | "AGGREGATION" => Some(AssociationKind::Aggregation),
            "composition" | "COMPOSITION" => Some(AssociationKind::Composition),
            _ => None,
        }
    }

    pub fn keyword(self) -> &'static str {
        match self {
            AssociationKind::Association => "association",
            AssociationKind::Aggregation => "aggregation",
            AssociationKind::Composition => "composition",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumValue {
    pub name: String,
    pub value: Option<String>,
}
