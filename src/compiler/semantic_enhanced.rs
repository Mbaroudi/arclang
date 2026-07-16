use super::ast::*;
use super::capella_metamodel::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EnhancedSemanticModel {
    pub elements: Vec<SemanticElement>,
    pub relationships: Vec<SemanticRelationship>,
    pub patterns: Vec<ArchitecturalPattern>,
    pub layers: Vec<ArchitecturalLayerInfo>,
    pub metamodel: CapellaMetamodel,
}

#[derive(Debug, Clone)]
pub struct SemanticElement {
    pub id: String,
    pub name: String,
    pub element_type: CapellaElementType,
    pub layer: ArchitecturalLayer,
    pub parent_id: Option<String>,
    pub children: Vec<String>,
    pub attributes: HashMap<String, String>,
    pub metadata: ElementTypeMetadata,
    pub ports: Vec<SemanticPort>,
    pub allocated_to: Vec<String>,
    pub allocated_from: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SemanticPort {
    pub id: String,
    pub name: String,
    pub port_type: CapellaElementType,
    pub direction: PortDirection,
    pub side: PortSide,
}

#[derive(Debug, Clone)]
pub enum PortDirection {
    In,
    Out,
    InOut,
}

#[derive(Debug, Clone)]
pub enum PortSide {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Clone)]
pub struct SemanticRelationship {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relationship_type: RelationshipType,
    pub exchange_items: Vec<String>,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipType {
    Contains,
    Connects,
    Allocates,
    Traces,
    Realizes,
    Extends,
    Uses,
    Implements,
    Composes,
    Aggregates,
    FunctionalExchange,
    ComponentExchange,
    PhysicalLink,
}

#[derive(Debug, Clone)]
pub struct ArchitecturalPattern {
    pub pattern_type: PatternType,
    pub elements: Vec<String>,
    pub description: String,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub enum PatternType {
    LayeredArchitecture,
    MicroserviceArchitecture,
    PipelinePattern,
    PublishSubscribe,
    RequestResponse,
    MasterSlave,
    Redundancy,
    Monitoring,
    ControlLoop,
    DataFlow,
    EventDriven,
}

#[derive(Debug, Clone)]
pub struct ArchitecturalLayerInfo {
    pub layer: ArchitecturalLayer,
    pub elements: Vec<String>,
    pub dependencies: Vec<String>,
}

impl EnhancedSemanticModel {
    pub fn get_element(&self, id: &str) -> Option<&SemanticElement> {
        self.elements.iter().find(|e| e.id == id)
    }
    
    pub fn get_children(&self, parent_id: &str) -> Vec<&SemanticElement> {
        self.elements.iter()
            .filter(|e| e.parent_id.as_ref() == Some(&parent_id.to_string()))
            .collect()
    }
    
    pub fn get_relationships_from(&self, element_id: &str) -> Vec<&SemanticRelationship> {
        self.relationships.iter()
            .filter(|r| r.source_id == element_id)
            .collect()
    }
    
    pub fn get_relationships_to(&self, element_id: &str) -> Vec<&SemanticRelationship> {
        self.relationships.iter()
            .filter(|r| r.target_id == element_id)
            .collect()
    }
    
    pub fn get_layer_elements(&self, layer: &ArchitecturalLayer) -> Vec<&SemanticElement> {
        self.elements.iter()
            .filter(|e| e.layer == *layer)
            .collect()
    }
    
    pub fn get_element_type_count(&self, element_type: &CapellaElementType) -> usize {
        self.elements.iter()
            .filter(|e| e.element_type == *element_type)
            .count()
    }
}
