//! Adapter between simple SemanticModel and enhanced SemanticModel
//! This bridges the gap for 7D Intelligence integration

use super::semantic::{SemanticModel, ComponentInfo, FunctionInfo};
use super::semantic_enhanced::{EnhancedSemanticModel, SemanticElement, SemanticRelationship, RelationshipType, PortDirection, PortSide, SemanticPort};
use super::capella_metamodel::{CapellaMetamodel, CapellaElementType, ElementTypeMetadata, ArchitecturalLayer, ElementCategory, DiagramShape, PlacementStrategy, PortConfiguration};
use std::collections::HashMap;

pub struct SemanticAdapter;

impl SemanticAdapter {
    /// Convert simple SemanticModel to EnhancedSemanticModel for 7D Intelligence
    pub fn enhance(model: &SemanticModel, dimension: &str) -> EnhancedSemanticModel {
        let metamodel = CapellaMetamodel::new();
        let mut elements = Vec::new();
        let mut relationships = Vec::new();
        
        // Convert components to semantic elements
        for (idx, comp) in model.components.iter().enumerate() {
            let element_type = Self::infer_capella_type(comp, dimension);
            let layer = Self::infer_layer(dimension);
            
            let element = SemanticElement {
                id: comp.id.clone(),
                name: comp.name.clone(),
                element_type,
                layer,
                parent_id: None,
                children: Vec::new(),
                attributes: Self::extract_attributes(comp),
                metadata: metamodel.get_metadata(&element_type).cloned().unwrap_or_else(|| {
                    eprintln!("Warning: No metadata for {:?}, using default", element_type);
                    ElementTypeMetadata {
                        element_type,
                        display_name: "Unknown",
                        description: "Unknown element type",
                        category: ElementCategory::Structural,
                        shape: DiagramShape::Rectangle,
                        default_color: "#CCCCCC",
                        default_width: 200.0,
                        default_height: 150.0,
                        can_contain: vec![],
                        can_connect_to: vec![],
                        placement_strategy: PlacementStrategy::Hierarchical,
                        port_configuration: PortConfiguration::None,
                        architectural_layer: layer,
                    }
                }),
                ports: Self::create_ports(&comp.id, element_type),
                allocated_to: Vec::new(),
                allocated_from: Vec::new(),
            };
            
            elements.push(element);
        }
        
        // Convert functions to semantic elements
        for func in &model.functions {
            let element_type = Self::infer_function_type(func, dimension);
            let layer = Self::infer_layer(dimension);
            
            let element = SemanticElement {
                id: func.id.clone(),
                name: func.name.clone(),
                element_type,
                layer,
                parent_id: None,
                children: Vec::new(),
                attributes: HashMap::new(),
                metadata: metamodel.get_metadata(&element_type).cloned().unwrap_or_else(|| {
                    eprintln!("Warning: No metadata for {:?}, using default", element_type);
                    ElementTypeMetadata {
                        element_type,
                        display_name: "Unknown",
                        description: "Unknown element type",
                        category: ElementCategory::Behavioral,
                        shape: DiagramShape::RoundedRectangle,
                        default_color: "#CCCCCC",
                        default_width: 200.0,
                        default_height: 120.0,
                        can_contain: vec![],
                        can_connect_to: vec![],
                        placement_strategy: PlacementStrategy::Layered,
                        port_configuration: PortConfiguration::InputOutput,
                        architectural_layer: layer,
                    }
                }),
                ports: Self::create_ports(&func.id, element_type),
                allocated_to: Vec::new(),
                allocated_from: Vec::new(),
            };
            
            elements.push(element);
        }
        
        // Convert traces to relationships
        for trace in &model.traces {
            let relationship = SemanticRelationship {
                id: format!("rel_{}_{}", trace.from, trace.to),
                source_id: trace.from.clone(),
                target_id: trace.to.clone(),
                relationship_type: Self::infer_relationship_type(&trace.trace_type),
                exchange_items: Vec::new(),
                attributes: HashMap::new(),
            };
            
            relationships.push(relationship);
        }
        
        EnhancedSemanticModel {
            elements,
            relationships,
            patterns: Vec::new(),
            layers: vec![],
            metamodel,
        }
    }
    
    fn infer_capella_type(comp: &ComponentInfo, dimension: &str) -> CapellaElementType {
        let comp_type = &comp.component_type;
        
        match dimension {
            "operational" => {
                if comp_type.contains("Actor") {
                    CapellaElementType::OperationalActor
                } else if comp_type.contains("Entity") {
                    CapellaElementType::OperationalEntity
                } else {
                    CapellaElementType::OperationalActivity
                }
            },
            "system" => {
                if comp_type.contains("Actor") {
                    CapellaElementType::Actor
                } else {
                    CapellaElementType::SystemComponent
                }
            },
            "logical" => CapellaElementType::LogicalComponent,
            "physical" => CapellaElementType::PhysicalComponent,
            _ => CapellaElementType::Component,
        }
    }
    
    fn infer_function_type(_func: &FunctionInfo, dimension: &str) -> CapellaElementType {
        match dimension {
            "operational" => CapellaElementType::OperationalActivity,
            "system" => CapellaElementType::SystemFunction,
            "logical" => CapellaElementType::LogicalFunction,
            "physical" => CapellaElementType::PhysicalFunction,
            _ => CapellaElementType::Function,
        }
    }
    
    fn infer_layer(dimension: &str) -> ArchitecturalLayer {
        match dimension {
            "operational" => ArchitecturalLayer::Operational,
            "system" => ArchitecturalLayer::System,
            "logical" => ArchitecturalLayer::Logical,
            "physical" => ArchitecturalLayer::Physical,
            "epbs" => ArchitecturalLayer::EPBS,
            _ => ArchitecturalLayer::Logical,
        }
    }
    
    fn infer_relationship_type(trace_type: &str) -> RelationshipType {
        match trace_type {
            "satisfies" => RelationshipType::Traces,
            "implements" => RelationshipType::Implements,
            "realizes" => RelationshipType::Realizes,
            "allocates" => RelationshipType::Allocates,
            "contains" => RelationshipType::Contains,
            _ => RelationshipType::Connects,
        }
    }
    
    fn extract_attributes(comp: &ComponentInfo) -> HashMap<String, String> {
        let mut attrs = HashMap::new();
        
        attrs.insert("component_type".to_string(), comp.component_type.clone());
        attrs.insert("level".to_string(), comp.level.clone());
        
        if let Some(safety) = &comp.safety_level {
            attrs.insert("safety_level".to_string(), safety.clone());
        }
        
        if let Some(asil) = &comp.asil {
            attrs.insert("asil".to_string(), asil.clone());
        }
        
        attrs
    }
    
    fn create_ports(element_id: &str, element_type: CapellaElementType) -> Vec<SemanticPort> {
        match element_type {
            CapellaElementType::LogicalComponent |
            CapellaElementType::PhysicalComponent |
            CapellaElementType::SystemComponent => {
                vec![
                    SemanticPort {
                        id: format!("{}_in", element_id),
                        name: "IN".to_string(),
                        port_type: CapellaElementType::ComponentPort,
                        direction: PortDirection::In,
                        side: PortSide::West,
                    },
                    SemanticPort {
                        id: format!("{}_out", element_id),
                        name: "OUT".to_string(),
                        port_type: CapellaElementType::ComponentPort,
                        direction: PortDirection::Out,
                        side: PortSide::East,
                    },
                ]
            },
            CapellaElementType::SystemFunction |
            CapellaElementType::LogicalFunction => {
                vec![
                    SemanticPort {
                        id: format!("{}_in", element_id),
                        name: "IN".to_string(),
                        port_type: CapellaElementType::FunctionInputPort,
                        direction: PortDirection::In,
                        side: PortSide::West,
                    },
                    SemanticPort {
                        id: format!("{}_out", element_id),
                        name: "OUT".to_string(),
                        port_type: CapellaElementType::FunctionOutputPort,
                        direction: PortDirection::Out,
                        side: PortSide::East,
                    },
                ]
            },
            _ => Vec::new(),
        }
    }
}
