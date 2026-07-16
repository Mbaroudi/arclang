use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CapellaElementType {
    Actor,
    Component,
    SystemComponent,
    LogicalComponent,
    PhysicalComponent,
    NodeComponent,
    BehaviorComponent,
    
    Function,
    SystemFunction,
    LogicalFunction,
    PhysicalFunction,
    OperationalActivity,
    OperationalProcess,
    
    FunctionalExchange,
    ComponentExchange,
    PhysicalLink,
    PhysicalPath,
    
    Interface,
    ProvidedInterface,
    RequiredInterface,
    ImplementedInterface,
    UsedInterface,
    
    Port,
    FunctionPort,
    FunctionInputPort,
    FunctionOutputPort,
    ComponentPort,
    PhysicalPort,
    
    Capability,
    Mission,
    OperationalCapability,
    
    Scenario,
    FunctionalScenario,
    InterfaceScenario,
    CapabilityRealization,
    
    Requirement,
    StakeholderRequirement,
    SystemRequirement,
    SubsystemRequirement,
    SoftwareRequirement,
    HardwareRequirement,
    
    State,
    Mode,
    Region,
    Transition,
    
    Data,
    DataType,
    Class,
    Collection,
    Union,
    Enumeration,
    
    Package,
    Layer,
    ComponentPackage,
    FunctionPackage,
    InterfacePackage,
    DataPackage,
    
    Constraint,
    PropertyValue,
    PropertyValueGroup,
    
    ExchangeItem,
    ExchangeItemElement,
    
    OperationalEntity,
    OperationalActor,
    
    ConfigurationItem,
    
    Allocation,
    ComponentFunctionAllocation,
    PartDeploymentLink,
}

#[derive(Debug, Clone)]
pub struct ElementTypeMetadata {
    pub element_type: CapellaElementType,
    pub display_name: &'static str,
    pub description: &'static str,
    pub category: ElementCategory,
    pub shape: DiagramShape,
    pub default_color: &'static str,
    pub default_width: f64,
    pub default_height: f64,
    pub can_contain: Vec<CapellaElementType>,
    pub can_connect_to: Vec<CapellaElementType>,
    pub placement_strategy: PlacementStrategy,
    pub port_configuration: PortConfiguration,
    pub architectural_layer: ArchitecturalLayer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ElementCategory {
    Structural,
    Behavioral,
    DataFlow,
    Requirement,
    Allocation,
    Organizational,
}

#[derive(Debug, Clone)]
pub enum DiagramShape {
    Rectangle,
    RoundedRectangle,
    Hexagon,
    Ellipse,
    Diamond,
    Parallelogram,
    Trapezoid,
    Pentagon,
    Cloud,
    Cylinder,
    Note,
    Actor,
}

#[derive(Debug, Clone)]
pub enum PlacementStrategy {
    Hierarchical,
    Layered,
    ForceDirected,
    Orthogonal,
    Circular,
    Tree,
    Sequential,
    Matrix,
}

#[derive(Debug, Clone)]
pub enum PortConfiguration {
    None,
    InputOutput,
    FourSides,
    TopBottom,
    LeftRight,
    Surrounding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArchitecturalLayer {
    Operational,
    System,
    Logical,
    Physical,
    EPBS,
    CrossLayer,
}

#[derive(Debug, Clone)]
pub struct CapellaMetamodel {
    elements: HashMap<CapellaElementType, ElementTypeMetadata>,
}

impl CapellaMetamodel {
    pub fn new() -> Self {
        let mut metamodel = Self {
            elements: HashMap::new(),
        };
        metamodel.initialize();
        metamodel
    }
    
    fn initialize(&mut self) {
        self.register_actors();
        self.register_components();
        self.register_functions();
        self.register_exchanges();
        self.register_interfaces();
        self.register_ports();
        self.register_capabilities();
        self.register_requirements();
        self.register_states();
        self.register_data();
        self.register_organizational();
    }
    
    fn register_actors(&mut self) {
        self.elements.insert(
            CapellaElementType::Actor,
            ElementTypeMetadata {
                element_type: CapellaElementType::Actor,
                display_name: "Actor",
                description: "External entity that interacts with the system",
                category: ElementCategory::Structural,
                shape: DiagramShape::Actor,
                default_color: "#FFE4B5",
                default_width: 180.0,
                default_height: 160.0,
                can_contain: vec![],
                can_connect_to: vec![
                    CapellaElementType::Component,
                    CapellaElementType::SystemComponent,
                    CapellaElementType::Actor,
                ],
                placement_strategy: PlacementStrategy::Layered,
                port_configuration: PortConfiguration::FourSides,
                architectural_layer: ArchitecturalLayer::CrossLayer,
            }
        );
        
        self.elements.insert(
            CapellaElementType::OperationalActor,
            ElementTypeMetadata {
                element_type: CapellaElementType::OperationalActor,
                display_name: "Operational Actor",
                description: "Actor in operational analysis",
                category: ElementCategory::Structural,
                shape: DiagramShape::Actor,
                default_color: "#E6F3FF",
                default_width: 180.0,
                default_height: 160.0,
                can_contain: vec![CapellaElementType::OperationalActivity],
                can_connect_to: vec![
                    CapellaElementType::OperationalEntity,
                    CapellaElementType::OperationalActor,
                ],
                placement_strategy: PlacementStrategy::Layered,
                port_configuration: PortConfiguration::FourSides,
                architectural_layer: ArchitecturalLayer::Operational,
            }
        );
        
        self.elements.insert(
            CapellaElementType::OperationalEntity,
            ElementTypeMetadata {
                element_type: CapellaElementType::OperationalEntity,
                display_name: "Operational Entity",
                description: "Entity participating in operational scenarios",
                category: ElementCategory::Structural,
                shape: DiagramShape::RoundedRectangle,
                default_color: "#E6F3FF",
                default_width: 200.0,
                default_height: 160.0,
                can_contain: vec![CapellaElementType::OperationalActivity],
                can_connect_to: vec![
                    CapellaElementType::OperationalEntity,
                    CapellaElementType::OperationalActor,
                ],
                placement_strategy: PlacementStrategy::Layered,
                port_configuration: PortConfiguration::FourSides,
                architectural_layer: ArchitecturalLayer::Operational,
            }
        );
    }
    
    fn register_components(&mut self) {
        self.elements.insert(
            CapellaElementType::Component,
            ElementTypeMetadata {
                element_type: CapellaElementType::Component,
                display_name: "Component",
                description: "Generic architectural component",
                category: ElementCategory::Structural,
                shape: DiagramShape::RoundedRectangle,
                default_color: "#B3E5FC",
                default_width: 220.0,
                default_height: 160.0,
                can_contain: vec![
                    CapellaElementType::Component,
                    CapellaElementType::Function,
                    CapellaElementType::Port,
                ],
                can_connect_to: vec![
                    CapellaElementType::Component,
                    CapellaElementType::Actor,
                ],
                placement_strategy: PlacementStrategy::Hierarchical,
                port_configuration: PortConfiguration::FourSides,
                architectural_layer: ArchitecturalLayer::CrossLayer,
            }
        );
        
        self.elements.insert(
            CapellaElementType::SystemComponent,
            ElementTypeMetadata {
                element_type: CapellaElementType::SystemComponent,
                display_name: "System Component",
                description: "Component at system architecture level",
                category: ElementCategory::Structural,
                shape: DiagramShape::RoundedRectangle,
                default_color: "#FFE082",
                default_width: 220.0,
                default_height: 160.0,
                can_contain: vec![
                    CapellaElementType::SystemComponent,
                    CapellaElementType::SystemFunction,
                    CapellaElementType::ComponentPort,
                ],
                can_connect_to: vec![
                    CapellaElementType::SystemComponent,
                    CapellaElementType::Actor,
                ],
                placement_strategy: PlacementStrategy::Hierarchical,
                port_configuration: PortConfiguration::FourSides,
                architectural_layer: ArchitecturalLayer::System,
            }
        );
        
        self.elements.insert(
            CapellaElementType::LogicalComponent,
            ElementTypeMetadata {
                element_type: CapellaElementType::LogicalComponent,
                display_name: "Logical Component",
                description: "Component at logical architecture level",
                category: ElementCategory::Structural,
                shape: DiagramShape::RoundedRectangle,
                default_color: "#C5E1A5",
                default_width: 220.0,
                default_height: 160.0,
                can_contain: vec![
                    CapellaElementType::LogicalComponent,
                    CapellaElementType::LogicalFunction,
                    CapellaElementType::ComponentPort,
                ],
                can_connect_to: vec![
                    CapellaElementType::LogicalComponent,
                    CapellaElementType::Actor,
                ],
                placement_strategy: PlacementStrategy::Hierarchical,
                port_configuration: PortConfiguration::FourSides,
                architectural_layer: ArchitecturalLayer::Logical,
            }
        );
        
        self.elements.insert(
            CapellaElementType::PhysicalComponent,
            ElementTypeMetadata {
                element_type: CapellaElementType::PhysicalComponent,
                display_name: "Physical Component",
                description: "Component at physical architecture level",
                category: ElementCategory::Structural,
                shape: DiagramShape::RoundedRectangle,
                default_color: "#CE93D8",
                default_width: 220.0,
                default_height: 160.0,
                can_contain: vec![
                    CapellaElementType::PhysicalComponent,
                    CapellaElementType::PhysicalFunction,
                    CapellaElementType::PhysicalPort,
                ],
                can_connect_to: vec![
                    CapellaElementType::PhysicalComponent,
                    CapellaElementType::NodeComponent,
                ],
                placement_strategy: PlacementStrategy::Hierarchical,
                port_configuration: PortConfiguration::FourSides,
                architectural_layer: ArchitecturalLayer::Physical,
            }
        );
        
        self.elements.insert(
            CapellaElementType::NodeComponent,
            ElementTypeMetadata {
                element_type: CapellaElementType::NodeComponent,
                display_name: "Node Component",
                description: "Deployment node for physical components",
                category: ElementCategory::Structural,
                shape: DiagramShape::Cylinder,
                default_color: "#B39DDB",
                default_width: 200.0,
                default_height: 180.0,
                can_contain: vec![CapellaElementType::PhysicalComponent],
                can_connect_to: vec![CapellaElementType::NodeComponent],
                placement_strategy: PlacementStrategy::Hierarchical,
                port_configuration: PortConfiguration::FourSides,
                architectural_layer: ArchitecturalLayer::Physical,
            }
        );
        
        self.elements.insert(
            CapellaElementType::BehaviorComponent,
            ElementTypeMetadata {
                element_type: CapellaElementType::BehaviorComponent,
                display_name: "Behavior Component",
                description: "Component with explicit behavior definition",
                category: ElementCategory::Behavioral,
                shape: DiagramShape::RoundedRectangle,
                default_color: "#90CAF9",
                default_width: 220.0,
                default_height: 160.0,
                can_contain: vec![
                    CapellaElementType::State,
                    CapellaElementType::Mode,
                    CapellaElementType::Function,
                ],
                can_connect_to: vec![CapellaElementType::BehaviorComponent],
                placement_strategy: PlacementStrategy::Hierarchical,
                port_configuration: PortConfiguration::FourSides,
                architectural_layer: ArchitecturalLayer::CrossLayer,
            }
        );
    }
    
    fn register_functions(&mut self) {
        self.elements.insert(
            CapellaElementType::Function,
            ElementTypeMetadata {
                element_type: CapellaElementType::Function,
                display_name: "Function",
                description: "Generic functional element",
                category: ElementCategory::Behavioral,
                shape: DiagramShape::RoundedRectangle,
                default_color: "#FFE082",
                default_width: 200.0,
                default_height: 120.0,
                can_contain: vec![
                    CapellaElementType::Function,
                    CapellaElementType::FunctionPort,
                ],
                can_connect_to: vec![CapellaElementType::Function],
                placement_strategy: PlacementStrategy::Layered,
                port_configuration: PortConfiguration::InputOutput,
                architectural_layer: ArchitecturalLayer::CrossLayer,
            }
        );
        
        self.elements.insert(
            CapellaElementType::SystemFunction,
            ElementTypeMetadata {
                element_type: CapellaElementType::SystemFunction,
                display_name: "System Function",
                description: "Function at system level",
                category: ElementCategory::Behavioral,
                shape: DiagramShape::RoundedRectangle,
                default_color: "#FFF59D",
                default_width: 200.0,
                default_height: 120.0,
                can_contain: vec![
                    CapellaElementType::SystemFunction,
                    CapellaElementType::FunctionInputPort,
                    CapellaElementType::FunctionOutputPort,
                ],
                can_connect_to: vec![CapellaElementType::SystemFunction],
                placement_strategy: PlacementStrategy::Layered,
                port_configuration: PortConfiguration::InputOutput,
                architectural_layer: ArchitecturalLayer::System,
            }
        );
        
        self.elements.insert(
            CapellaElementType::LogicalFunction,
            ElementTypeMetadata {
                element_type: CapellaElementType::LogicalFunction,
                display_name: "Logical Function",
                description: "Function at logical level",
                category: ElementCategory::Behavioral,
                shape: DiagramShape::RoundedRectangle,
                default_color: "#DCEDC8",
                default_width: 200.0,
                default_height: 120.0,
                can_contain: vec![
                    CapellaElementType::LogicalFunction,
                    CapellaElementType::FunctionInputPort,
                    CapellaElementType::FunctionOutputPort,
                ],
                can_connect_to: vec![CapellaElementType::LogicalFunction],
                placement_strategy: PlacementStrategy::Layered,
                port_configuration: PortConfiguration::InputOutput,
                architectural_layer: ArchitecturalLayer::Logical,
            }
        );
        
        self.elements.insert(
            CapellaElementType::PhysicalFunction,
            ElementTypeMetadata {
                element_type: CapellaElementType::PhysicalFunction,
                display_name: "Physical Function",
                description: "Function at physical level",
                category: ElementCategory::Behavioral,
                shape: DiagramShape::RoundedRectangle,
                default_color: "#E1BEE7",
                default_width: 200.0,
                default_height: 120.0,
                can_contain: vec![
                    CapellaElementType::PhysicalFunction,
                    CapellaElementType::FunctionInputPort,
                    CapellaElementType::FunctionOutputPort,
                ],
                can_connect_to: vec![CapellaElementType::PhysicalFunction],
                placement_strategy: PlacementStrategy::Layered,
                port_configuration: PortConfiguration::InputOutput,
                architectural_layer: ArchitecturalLayer::Physical,
            }
        );
        
        self.elements.insert(
            CapellaElementType::OperationalActivity,
            ElementTypeMetadata {
                element_type: CapellaElementType::OperationalActivity,
                display_name: "Operational Activity",
                description: "Activity in operational analysis",
                category: ElementCategory::Behavioral,
                shape: DiagramShape::RoundedRectangle,
                default_color: "#E3F2FD",
                default_width: 200.0,
                default_height: 120.0,
                can_contain: vec![CapellaElementType::OperationalActivity],
                can_connect_to: vec![CapellaElementType::OperationalActivity],
                placement_strategy: PlacementStrategy::Sequential,
                port_configuration: PortConfiguration::InputOutput,
                architectural_layer: ArchitecturalLayer::Operational,
            }
        );
        
        self.elements.insert(
            CapellaElementType::OperationalProcess,
            ElementTypeMetadata {
                element_type: CapellaElementType::OperationalProcess,
                display_name: "Operational Process",
                description: "Process containing operational activities",
                category: ElementCategory::Behavioral,
                shape: DiagramShape::Parallelogram,
                default_color: "#BBDEFB",
                default_width: 240.0,
                default_height: 140.0,
                can_contain: vec![CapellaElementType::OperationalActivity],
                can_connect_to: vec![CapellaElementType::OperationalProcess],
                placement_strategy: PlacementStrategy::Sequential,
                port_configuration: PortConfiguration::InputOutput,
                architectural_layer: ArchitecturalLayer::Operational,
            }
        );
    }
    
    fn register_exchanges(&mut self) {
        self.elements.insert(
            CapellaElementType::FunctionalExchange,
            ElementTypeMetadata {
                element_type: CapellaElementType::FunctionalExchange,
                display_name: "Functional Exchange",
                description: "Data or control flow between functions",
                category: ElementCategory::DataFlow,
                shape: DiagramShape::Diamond,
                default_color: "#81C784",
                default_width: 80.0,
                default_height: 80.0,
                can_contain: vec![CapellaElementType::ExchangeItem],
                can_connect_to: vec![
                    CapellaElementType::Function,
                    CapellaElementType::FunctionPort,
                ],
                placement_strategy: PlacementStrategy::Orthogonal,
                port_configuration: PortConfiguration::None,
                architectural_layer: ArchitecturalLayer::CrossLayer,
            }
        );
        
        self.elements.insert(
            CapellaElementType::ComponentExchange,
            ElementTypeMetadata {
                element_type: CapellaElementType::ComponentExchange,
                display_name: "Component Exchange",
                description: "Connection between components",
                category: ElementCategory::DataFlow,
                shape: DiagramShape::Diamond,
                default_color: "#64B5F6",
                default_width: 80.0,
                default_height: 80.0,
                can_contain: vec![CapellaElementType::ExchangeItem],
                can_connect_to: vec![
                    CapellaElementType::Component,
                    CapellaElementType::ComponentPort,
                ],
                placement_strategy: PlacementStrategy::Orthogonal,
                port_configuration: PortConfiguration::None,
                architectural_layer: ArchitecturalLayer::CrossLayer,
            }
        );
        
        self.elements.insert(
            CapellaElementType::PhysicalLink,
            ElementTypeMetadata {
                element_type: CapellaElementType::PhysicalLink,
                display_name: "Physical Link",
                description: "Physical connection between components",
                category: ElementCategory::DataFlow,
                shape: DiagramShape::Diamond,
                default_color: "#BA68C8",
                default_width: 80.0,
                default_height: 80.0,
                can_contain: vec![],
                can_connect_to: vec![
                    CapellaElementType::PhysicalComponent,
                    CapellaElementType::PhysicalPort,
                ],
                placement_strategy: PlacementStrategy::Orthogonal,
                port_configuration: PortConfiguration::None,
                architectural_layer: ArchitecturalLayer::Physical,
            }
        );
        
        self.elements.insert(
            CapellaElementType::PhysicalPath,
            ElementTypeMetadata {
                element_type: CapellaElementType::PhysicalPath,
                display_name: "Physical Path",
                description: "Path through physical links",
                category: ElementCategory::DataFlow,
                shape: DiagramShape::Parallelogram,
                default_color: "#AB47BC",
                default_width: 160.0,
                default_height: 80.0,
                can_contain: vec![CapellaElementType::PhysicalLink],
                can_connect_to: vec![CapellaElementType::PhysicalComponent],
                placement_strategy: PlacementStrategy::Orthogonal,
                port_configuration: PortConfiguration::None,
                architectural_layer: ArchitecturalLayer::Physical,
            }
        );
    }
    
    fn register_interfaces(&mut self) {
        self.elements.insert(
            CapellaElementType::Interface,
            ElementTypeMetadata {
                element_type: CapellaElementType::Interface,
                display_name: "Interface",
                description: "Contract between components",
                category: ElementCategory::Structural,
                shape: DiagramShape::Ellipse,
                default_color: "#4FC3F7",
                default_width: 140.0,
                default_height: 80.0,
                can_contain: vec![
                    CapellaElementType::ExchangeItem,
                    CapellaElementType::FunctionalExchange,
                ],
                can_connect_to: vec![
                    CapellaElementType::Component,
                    CapellaElementType::Port,
                ],
                placement_strategy: PlacementStrategy::Orthogonal,
                port_configuration: PortConfiguration::None,
                architectural_layer: ArchitecturalLayer::CrossLayer,
            }
        );
    }
    
    fn register_ports(&mut self) {
        self.elements.insert(
            CapellaElementType::FunctionInputPort,
            ElementTypeMetadata {
                element_type: CapellaElementType::FunctionInputPort,
                display_name: "Input Port",
                description: "Function input port",
                category: ElementCategory::Structural,
                shape: DiagramShape::Rectangle,
                default_color: "#66BB6A",
                default_width: 12.0,
                default_height: 12.0,
                can_contain: vec![],
                can_connect_to: vec![
                    CapellaElementType::FunctionOutputPort,
                    CapellaElementType::FunctionalExchange,
                ],
                placement_strategy: PlacementStrategy::Orthogonal,
                port_configuration: PortConfiguration::None,
                architectural_layer: ArchitecturalLayer::CrossLayer,
            }
        );
        
        self.elements.insert(
            CapellaElementType::FunctionOutputPort,
            ElementTypeMetadata {
                element_type: CapellaElementType::FunctionOutputPort,
                display_name: "Output Port",
                description: "Function output port",
                category: ElementCategory::Structural,
                shape: DiagramShape::Rectangle,
                default_color: "#FFA726",
                default_width: 12.0,
                default_height: 12.0,
                can_contain: vec![],
                can_connect_to: vec![
                    CapellaElementType::FunctionInputPort,
                    CapellaElementType::FunctionalExchange,
                ],
                placement_strategy: PlacementStrategy::Orthogonal,
                port_configuration: PortConfiguration::None,
                architectural_layer: ArchitecturalLayer::CrossLayer,
            }
        );
        
        self.elements.insert(
            CapellaElementType::ComponentPort,
            ElementTypeMetadata {
                element_type: CapellaElementType::ComponentPort,
                display_name: "Component Port",
                description: "Component interaction point",
                category: ElementCategory::Structural,
                shape: DiagramShape::Rectangle,
                default_color: "#42A5F5",
                default_width: 12.0,
                default_height: 12.0,
                can_contain: vec![],
                can_connect_to: vec![
                    CapellaElementType::ComponentPort,
                    CapellaElementType::ComponentExchange,
                ],
                placement_strategy: PlacementStrategy::Orthogonal,
                port_configuration: PortConfiguration::None,
                architectural_layer: ArchitecturalLayer::CrossLayer,
            }
        );
        
        self.elements.insert(
            CapellaElementType::PhysicalPort,
            ElementTypeMetadata {
                element_type: CapellaElementType::PhysicalPort,
                display_name: "Physical Port",
                description: "Physical connection point",
                category: ElementCategory::Structural,
                shape: DiagramShape::Rectangle,
                default_color: "#AB47BC",
                default_width: 12.0,
                default_height: 12.0,
                can_contain: vec![],
                can_connect_to: vec![
                    CapellaElementType::PhysicalPort,
                    CapellaElementType::PhysicalLink,
                ],
                placement_strategy: PlacementStrategy::Orthogonal,
                port_configuration: PortConfiguration::None,
                architectural_layer: ArchitecturalLayer::Physical,
            }
        );
    }
    
    fn register_capabilities(&mut self) {
        self.elements.insert(
            CapellaElementType::Capability,
            ElementTypeMetadata {
                element_type: CapellaElementType::Capability,
                display_name: "Capability",
                description: "System capability",
                category: ElementCategory::Organizational,
                shape: DiagramShape::Hexagon,
                default_color: "#FFD54F",
                default_width: 200.0,
                default_height: 100.0,
                can_contain: vec![CapellaElementType::Scenario],
                can_connect_to: vec![
                    CapellaElementType::Component,
                    CapellaElementType::Function,
                ],
                placement_strategy: PlacementStrategy::Tree,
                port_configuration: PortConfiguration::None,
                architectural_layer: ArchitecturalLayer::CrossLayer,
            }
        );
        
        self.elements.insert(
            CapellaElementType::Mission,
            ElementTypeMetadata {
                element_type: CapellaElementType::Mission,
                display_name: "Mission",
                description: "High-level mission statement",
                category: ElementCategory::Organizational,
                shape: DiagramShape::Hexagon,
                default_color: "#FFB74D",
                default_width: 220.0,
                default_height: 110.0,
                can_contain: vec![CapellaElementType::Capability],
                can_connect_to: vec![],
                placement_strategy: PlacementStrategy::Tree,
                port_configuration: PortConfiguration::None,
                architectural_layer: ArchitecturalLayer::CrossLayer,
            }
        );
    }
    
    fn register_requirements(&mut self) {
        let requirement_types = vec![
            (CapellaElementType::Requirement, "Requirement", "Generic requirement", ArchitecturalLayer::CrossLayer),
            (CapellaElementType::StakeholderRequirement, "Stakeholder Requirement", "Stakeholder need", ArchitecturalLayer::Operational),
            (CapellaElementType::SystemRequirement, "System Requirement", "System-level requirement", ArchitecturalLayer::System),
            (CapellaElementType::SubsystemRequirement, "Subsystem Requirement", "Subsystem requirement", ArchitecturalLayer::Logical),
        ];
        
        for (req_type, display_name, description, layer) in requirement_types {
            self.elements.insert(
                req_type,
                ElementTypeMetadata {
                    element_type: req_type.clone(),
                    display_name,
                    description,
                    category: ElementCategory::Requirement,
                    shape: DiagramShape::Note,
                    default_color: "#FFF9C4",
                    default_width: 180.0,
                    default_height: 100.0,
                    can_contain: vec![],
                    can_connect_to: vec![],
                    placement_strategy: PlacementStrategy::Matrix,
                    port_configuration: PortConfiguration::None,
                    architectural_layer: layer,
                }
            );
        }
    }
    
    fn register_states(&mut self) {
        self.elements.insert(
            CapellaElementType::State,
            ElementTypeMetadata {
                element_type: CapellaElementType::State,
                display_name: "State",
                description: "Component or function state",
                category: ElementCategory::Behavioral,
                shape: DiagramShape::RoundedRectangle,
                default_color: "#AED581",
                default_width: 140.0,
                default_height: 80.0,
                can_contain: vec![CapellaElementType::State],
                can_connect_to: vec![CapellaElementType::State],
                placement_strategy: PlacementStrategy::ForceDirected,
                port_configuration: PortConfiguration::None,
                architectural_layer: ArchitecturalLayer::CrossLayer,
            }
        );
        
        self.elements.insert(
            CapellaElementType::Mode,
            ElementTypeMetadata {
                element_type: CapellaElementType::Mode,
                display_name: "Mode",
                description: "Operational mode",
                category: ElementCategory::Behavioral,
                shape: DiagramShape::Hexagon,
                default_color: "#A5D6A7",
                default_width: 160.0,
                default_height: 90.0,
                can_contain: vec![CapellaElementType::State],
                can_connect_to: vec![CapellaElementType::Mode],
                placement_strategy: PlacementStrategy::ForceDirected,
                port_configuration: PortConfiguration::None,
                architectural_layer: ArchitecturalLayer::CrossLayer,
            }
        );
    }
    
    fn register_data(&mut self) {
        self.elements.insert(
            CapellaElementType::ExchangeItem,
            ElementTypeMetadata {
                element_type: CapellaElementType::ExchangeItem,
                display_name: "Exchange Item",
                description: "Data exchanged between elements",
                category: ElementCategory::DataFlow,
                shape: DiagramShape::Rectangle,
                default_color: "#B2DFDB",
                default_width: 120.0,
                default_height: 60.0,
                can_contain: vec![CapellaElementType::ExchangeItemElement],
                can_connect_to: vec![],
                placement_strategy: PlacementStrategy::Matrix,
                port_configuration: PortConfiguration::None,
                architectural_layer: ArchitecturalLayer::CrossLayer,
            }
        );
        
        self.elements.insert(
            CapellaElementType::Class,
            ElementTypeMetadata {
                element_type: CapellaElementType::Class,
                display_name: "Class",
                description: "Data class definition",
                category: ElementCategory::DataFlow,
                shape: DiagramShape::Rectangle,
                default_color: "#80CBC4",
                default_width: 180.0,
                default_height: 120.0,
                can_contain: vec![],
                can_connect_to: vec![CapellaElementType::Class],
                placement_strategy: PlacementStrategy::Hierarchical,
                port_configuration: PortConfiguration::None,
                architectural_layer: ArchitecturalLayer::CrossLayer,
            }
        );
    }
    
    fn register_organizational(&mut self) {
        self.elements.insert(
            CapellaElementType::Package,
            ElementTypeMetadata {
                element_type: CapellaElementType::Package,
                display_name: "Package",
                description: "Organizational container",
                category: ElementCategory::Organizational,
                shape: DiagramShape::Rectangle,
                default_color: "#EEEEEE",
                default_width: 300.0,
                default_height: 250.0,
                can_contain: vec![
                    CapellaElementType::Component,
                    CapellaElementType::Function,
                    CapellaElementType::Package,
                ],
                can_connect_to: vec![],
                placement_strategy: PlacementStrategy::Hierarchical,
                port_configuration: PortConfiguration::None,
                architectural_layer: ArchitecturalLayer::CrossLayer,
            }
        );
    }
    
    pub fn get_metadata(&self, element_type: &CapellaElementType) -> Option<&ElementTypeMetadata> {
        self.elements.get(element_type)
    }
    
    pub fn get_all_types(&self) -> Vec<&CapellaElementType> {
        self.elements.keys().collect()
    }
    
    pub fn get_by_category(&self, category: ElementCategory) -> Vec<&ElementTypeMetadata> {
        self.elements.values()
            .filter(|meta| meta.category == category)
            .collect()
    }
    
    pub fn get_by_layer(&self, layer: ArchitecturalLayer) -> Vec<&ElementTypeMetadata> {
        self.elements.values()
            .filter(|meta| meta.architectural_layer == layer || meta.architectural_layer == ArchitecturalLayer::CrossLayer)
            .collect()
    }
    
    pub fn can_contain(&self, parent: &CapellaElementType, child: &CapellaElementType) -> bool {
        if let Some(metadata) = self.elements.get(parent) {
            metadata.can_contain.contains(child)
        } else {
            false
        }
    }
    
    pub fn can_connect(&self, source: &CapellaElementType, target: &CapellaElementType) -> bool {
        if let Some(metadata) = self.elements.get(source) {
            metadata.can_connect_to.contains(target)
        } else {
            false
        }
    }
    
    pub fn infer_element_type_from_string(&self, type_str: &str) -> Option<CapellaElementType> {
        let normalized = type_str.to_lowercase();
        
        match normalized.as_str() {
            "actor" => Some(CapellaElementType::Actor),
            "component" => Some(CapellaElementType::Component),
            "systemcomponent" | "system_component" => Some(CapellaElementType::SystemComponent),
            "logicalcomponent" | "logical_component" => Some(CapellaElementType::LogicalComponent),
            "physicalcomponent" | "physical_component" => Some(CapellaElementType::PhysicalComponent),
            "function" => Some(CapellaElementType::Function),
            "systemfunction" | "system_function" => Some(CapellaElementType::SystemFunction),
            "logicalfunction" | "logical_function" => Some(CapellaElementType::LogicalFunction),
            "physicalfunction" | "physical_function" => Some(CapellaElementType::PhysicalFunction),
            "interface" => Some(CapellaElementType::Interface),
            "requirement" => Some(CapellaElementType::Requirement),
            "capability" => Some(CapellaElementType::Capability),
            _ => None,
        }
    }
}

impl Default for CapellaMetamodel {
    fn default() -> Self {
        Self::new()
    }
}
