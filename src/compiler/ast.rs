use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Model {
    pub operational_analysis: Vec<OperationalAnalysis>,
    pub system_analysis: Vec<SystemAnalysis>,
    pub logical_architecture: Vec<LogicalArchitecture>,
    pub physical_architecture: Vec<PhysicalArchitecture>,
    pub epbs: Vec<Epbs>,
    pub safety_analysis: Vec<SafetyAnalysis>,
    pub traces: Vec<Trace>,
}

impl Model {
    pub fn new() -> Self {
        Self {
            operational_analysis: Vec::new(),
            system_analysis: Vec::new(),
            logical_architecture: Vec::new(),
            physical_architecture: Vec::new(),
            epbs: Vec::new(),
            safety_analysis: Vec::new(),
            traces: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OperationalAnalysis {
    pub name: String,
    pub actors: Vec<Actor>,
    pub capabilities: Vec<OperationalCapability>,
    pub activities: Vec<OperationalActivity>,
}

#[derive(Debug, Clone)]
pub struct Actor {
    pub name: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone)]
pub struct OperationalCapability {
    pub name: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone)]
pub struct OperationalActivity {
    pub name: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone)]
pub struct SystemAnalysis {
    pub name: String,
    pub requirements: Vec<Requirement>,
    pub functions: Vec<SystemFunction>,
    pub components: Vec<SystemComponent>,
}

#[derive(Debug, Clone)]
pub struct Requirement {
    pub id: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone)]
pub struct SystemFunction {
    pub name: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone)]
pub struct SystemComponent {
    pub name: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone)]
pub struct LogicalArchitecture {
    pub name: String,
    pub components: Vec<LogicalComponent>,
    pub interfaces: Vec<LogicalInterface>,
}

#[derive(Debug, Clone)]
pub struct LogicalComponent {
    pub name: String,
    pub functions: Vec<LogicalFunction>,
    pub interfaces_in: Vec<InterfaceDefinition>,
    pub interfaces_out: Vec<InterfaceDefinition>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone)]
pub struct InterfaceDefinition {
    pub name: String,
    pub protocol: Option<String>,
    pub format: Option<String>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone)]
pub struct LogicalFunction {
    pub name: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone)]
pub struct LogicalInterface {
    pub name: String,
    pub from: String,
    pub to: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone)]
pub struct PhysicalArchitecture {
    pub name: String,
    pub nodes: Vec<PhysicalNode>,
    pub links: Vec<PhysicalLink>,
}

#[derive(Debug, Clone)]
pub struct PhysicalNode {
    pub name: String,
    pub deployments: Vec<Deployment>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone)]
pub struct Deployment {
    pub component: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone)]
pub struct PhysicalLink {
    pub name: String,
    pub connections: Vec<String>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone)]
pub struct Epbs {
    pub name: String,
    pub systems: Vec<EpbsSystem>,
}

#[derive(Debug, Clone)]
pub struct EpbsSystem {
    pub name: String,
    pub subsystems: Vec<EpbsSubsystem>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone)]
pub struct EpbsSubsystem {
    pub name: String,
    pub items: Vec<EpbsItem>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone)]
pub struct EpbsItem {
    pub name: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone)]
pub struct SafetyAnalysis {
    pub hazards: Vec<Hazard>,
    pub fmea: Vec<FmeaEntry>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone)]
pub struct Hazard {
    pub name: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone)]
pub struct FmeaEntry {
    pub name: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone)]
pub struct Trace {
    pub from: String,
    pub to: String,
    pub trace_type: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone)]
pub enum AttributeValue {
    String(String),
    Number(f64),
    Boolean(bool),
    List(Vec<AttributeValue>),
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
