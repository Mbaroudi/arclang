use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    /// Model-level metadata (name, version, author, ...) from the model header
    /// and its `metadata` block.
    #[serde(default)]
    pub attributes: HashMap<String, AttributeValue>,
    pub operational_analysis: Vec<OperationalAnalysis>,
    pub system_analysis: Vec<SystemAnalysis>,
    pub logical_architecture: Vec<LogicalArchitecture>,
    pub physical_architecture: Vec<PhysicalArchitecture>,
    pub epbs: Vec<Epbs>,
    pub safety_analysis: Vec<SafetyAnalysis>,
    pub traces: Vec<Trace>,
    pub state_machines: Vec<StateMachine>,
    pub scenarios: Vec<Scenario>,
    pub exchange_items: Vec<ExchangeItem>,
    pub data_types: Vec<DataType>,
}

impl Model {
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
            operational_analysis: Vec::new(),
            system_analysis: Vec::new(),
            logical_architecture: Vec::new(),
            physical_architecture: Vec::new(),
            epbs: Vec::new(),
            safety_analysis: Vec::new(),
            traces: Vec::new(),
            state_machines: Vec::new(),
            scenarios: Vec::new(),
            exchange_items: Vec::new(),
            data_types: Vec::new(),
        }
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
    pub name: String,
    pub actors: Vec<Actor>,
    pub entities: Vec<OperationalEntity>,
    pub capabilities: Vec<OperationalCapability>,
    pub activities: Vec<OperationalActivity>,
    pub exchanges: Vec<OperationalExchange>,
    pub capability_associations: Vec<CapabilityAssociation>,
    pub traces: Vec<Trace>,
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
    pub name: String,
    pub requirements: Vec<Requirement>,
    pub functions: Vec<SystemFunction>,
    pub components: Vec<SystemComponent>,
    pub external_actors: Vec<ExternalActor>,
    pub functional_exchanges: Vec<FunctionalExchange>,
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
    pub attributes: HashMap<String, AttributeValue>,
}

/// Ordered set of references to functions and exchanges describing one
/// dataflow path, in the context of a capability (Arcadia: Functional Chain).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionalChain {
    pub id: String,
    pub name: String,
    /// Ordered references to the involved functions/exchanges.
    pub involves: Vec<String>,
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
    pub name: String,
    pub direction: PortDirection,
    pub port_type: PortType,
    pub data_type: String,
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
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionalExchange {
    pub from_port: String,
    pub to_port: String,
    pub data_type: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemComponent {
    pub name: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalArchitecture {
    pub name: String,
    pub components: Vec<LogicalComponent>,
    pub interfaces: Vec<LogicalInterface>,
    pub component_exchanges: Vec<ComponentExchange>,
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
    pub name: String,
    pub direction: PortDirection,
    pub interface_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentExchange {
    pub from_port: String,
    pub to_port: String,
    pub exchange_item: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceDefinition {
    pub name: String,
    pub protocol: Option<String>,
    pub format: Option<String>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalFunction {
    pub name: String,
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
    pub name: String,
    pub nodes: Vec<PhysicalNode>,
    pub links: Vec<PhysicalLink>,
    pub physical_exchanges: Vec<PhysicalExchange>,
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
    pub deployments: Vec<Deployment>,
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
    pub allocated_functions: Vec<String>,
    pub color: Option<String>,
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
    pub from: String,
    pub to: String,
    pub via: Option<String>,
    pub message_type: String,
    pub frequency: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Epbs {
    pub name: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub from: String,
    pub to: String,
    pub trace_type: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeValue {
    String(String),
    Number(f64),
    Boolean(bool),
    List(Vec<AttributeValue>),
    Map(HashMap<String, AttributeValue>),
}

impl AttributeValue {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            AttributeValue::String(s) => Some(s),
            _ => None,
        }
    }
    
    pub fn as_number(&self) -> Option<f64> {
        match self {
            AttributeValue::Number(n) => Some(*n),
            _ => None,
        }
    }
}

// Behavioral Models

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMachine {
    pub name: String,
    pub initial_state: String,
    pub states: Vec<State>,
    pub transitions: Vec<Transition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub name: String,
    pub entry_actions: Vec<String>,
    pub exit_actions: Vec<String>,
    pub internal_transitions: Vec<String>,
    pub sub_states: Vec<State>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub from: String,
    pub to: String,
    pub trigger: String,
    pub guard: Option<String>,
    pub action: Option<String>,
    pub timing: Option<String>,
    pub priority: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub participants: Vec<Participant>,
    pub messages: Vec<Message>,
    pub fragments: Vec<CombinedFragment>,
    pub timing_constraints: Vec<TimingConstraint>,
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
    pub from: String,
    pub to: String,
    pub label: String,
    pub message_type: MessageType,
    pub activation: bool,
    pub timing: Option<String>,
    pub params: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageType {
    Synchronous,
    Asynchronous,
    Return,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedFragment {
    pub fragment_type: FragmentType,
    pub label: String,
    pub condition: Option<String>,
    pub operands: Vec<FragmentOperand>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FragmentType {
    Par,
    Opt,
    Loop,
    Alt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentOperand {
    pub label: String,
    pub messages: Vec<Message>,
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
    pub name: String,
    pub stereotype: String,
    pub attributes: Vec<DataAttribute>,
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
    pub name: String,
    pub base_type: Option<String>,
    pub enumeration_values: Option<Vec<EnumValue>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumValue {
    pub name: String,
    pub value: Option<String>,
}
