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

pub struct EnhancedSemanticAnalyzer {
    metamodel: CapellaMetamodel,
}

impl EnhancedSemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            metamodel: CapellaMetamodel::new(),
        }
    }
    
    pub fn analyze(&self, ast: &Model) -> Result<EnhancedSemanticModel, String> {
        let mut elements = Vec::new();
        let mut relationships = Vec::new();
        
        self.analyze_operational_analysis(&ast.operational_analysis, &mut elements, &mut relationships)?;
        
        self.analyze_system_analysis(&ast.system_analysis, &mut elements, &mut relationships)?;
        
        self.analyze_logical_architecture(&ast.logical_architecture, &mut elements, &mut relationships)?;
        
        self.analyze_physical_architecture(&ast.physical_architecture, &mut elements, &mut relationships)?;
        
        self.analyze_traces(&ast.traces, &mut relationships)?;
        
        let patterns = self.detect_patterns(&elements, &relationships);
        
        let layers = self.organize_by_layers(&elements);
        
        Ok(EnhancedSemanticModel {
            elements,
            relationships,
            patterns,
            layers,
            metamodel: self.metamodel.clone(),
        })
    }
    
    fn analyze_operational_analysis(
        &self,
        operational_analyses: &[OperationalAnalysis],
        elements: &mut Vec<SemanticElement>,
        relationships: &mut Vec<SemanticRelationship>,
    ) -> Result<(), String> {
        for oa in operational_analyses {
            for actor in &oa.actors {
                let metadata = self.metamodel.get_metadata(&CapellaElementType::OperationalActor)
                    .ok_or("No metadata for OperationalActor")?
                    .clone();
                
                let actor_id = actor.attributes.get("id")
                    .and_then(|v| v.as_string())
                    .unwrap_or(&actor.name)
                    .to_string();
                
                elements.push(SemanticElement {
                    id: actor_id.clone(),
                    name: actor.name.clone(),
                    element_type: CapellaElementType::OperationalActor,
                    layer: ArchitecturalLayer::Operational,
                    parent_id: None,
                    children: Vec::new(),
                    attributes: self.extract_attributes(&actor.attributes),
                    metadata,
                    ports: Vec::new(),
                    allocated_to: Vec::new(),
                    allocated_from: Vec::new(),
                });
            }
            
            for entity in &oa.entities {
                let element_type = match entity.entity_type {
                    super::ast::EntityType::Actor => CapellaElementType::OperationalActor,
                    _ => CapellaElementType::OperationalEntity,
                };
                
                let metadata = self.metamodel.get_metadata(&element_type)
                    .ok_or_else(|| format!("No metadata for {:?}", element_type))?
                    .clone();
                
                let entity_id = entity.attributes.get("id")
                    .and_then(|v| v.as_string())
                    .unwrap_or(&entity.name)
                    .to_string();
                
                elements.push(SemanticElement {
                    id: entity_id.clone(),
                    name: entity.name.clone(),
                    element_type,
                    layer: ArchitecturalLayer::Operational,
                    parent_id: None,
                    children: Vec::new(),
                    attributes: self.extract_attributes(&entity.attributes),
                    metadata,
                    ports: Vec::new(),
                    allocated_to: Vec::new(),
                    allocated_from: Vec::new(),
                });
            }
            
            for activity in &oa.activities {
                let activity_id = activity.attributes.get("id")
                    .and_then(|v| v.as_string())
                    .unwrap_or(&activity.name)
                    .to_string();
                
                let activity_metadata = self.metamodel
                    .get_metadata(&CapellaElementType::OperationalActivity)
                    .ok_or("No metadata for OperationalActivity")?
                    .clone();
                
                elements.push(SemanticElement {
                    id: activity_id.clone(),
                    name: activity.name.clone(),
                    element_type: CapellaElementType::OperationalActivity,
                    layer: ArchitecturalLayer::Operational,
                    parent_id: None,
                    children: Vec::new(),
                    attributes: self.extract_attributes(&activity.attributes),
                    metadata: activity_metadata,
                    ports: self.create_function_ports(&activity.name),
                    allocated_to: Vec::new(),
                    allocated_from: Vec::new(),
                });
            }
        }
        
        Ok(())
    }
    
    fn analyze_system_analysis(
        &self,
        system_analyses: &[SystemAnalysis],
        elements: &mut Vec<SemanticElement>,
        relationships: &mut Vec<SemanticRelationship>,
    ) -> Result<(), String> {
        for sa in system_analyses {
            for req in &sa.requirements {
                let req_type = match req.attributes.get("type").and_then(|v| v.as_string()) {
                    Some("stakeholder") => CapellaElementType::StakeholderRequirement,
                    Some("system") => CapellaElementType::SystemRequirement,
                    _ => CapellaElementType::Requirement,
                };
                
                let metadata = self.metamodel.get_metadata(&req_type)
                    .ok_or_else(|| format!("No metadata for {:?}", req_type))?
                    .clone();
                
                elements.push(SemanticElement {
                    id: req.id.clone(),
                    name: req.id.clone(),
                    element_type: req_type,
                    layer: ArchitecturalLayer::System,
                    parent_id: None,
                    children: Vec::new(),
                    attributes: self.extract_attributes(&req.attributes),
                    metadata,
                    ports: Vec::new(),
                    allocated_to: Vec::new(),
                    allocated_from: Vec::new(),
                });
            }
        }
        
        Ok(())
    }
    
    fn analyze_logical_architecture(
        &self,
        logical_architectures: &[LogicalArchitecture],
        elements: &mut Vec<SemanticElement>,
        relationships: &mut Vec<SemanticRelationship>,
    ) -> Result<(), String> {
        for la in logical_architectures {
            for comp in &la.components {
                let comp_id = comp.attributes.get("id")
                    .and_then(|v| v.as_string())
                    .unwrap_or(&comp.name)
                    .to_string();
                
                let metadata = self.metamodel.get_metadata(&CapellaElementType::LogicalComponent)
                    .ok_or("No metadata for LogicalComponent")?
                    .clone();
                
                let mut ports = Vec::new();
                
                for (idx, iface_in) in comp.interfaces_in.iter().enumerate() {
                    ports.push(SemanticPort {
                        id: format!("{}_in_{}", comp_id, idx),
                        name: iface_in.name.clone(),
                        port_type: CapellaElementType::ComponentPort,
                        direction: PortDirection::In,
                        side: PortSide::West,
                    });
                }
                
                for (idx, iface_out) in comp.interfaces_out.iter().enumerate() {
                    ports.push(SemanticPort {
                        id: format!("{}_out_{}", comp_id, idx),
                        name: iface_out.name.clone(),
                        port_type: CapellaElementType::ComponentPort,
                        direction: PortDirection::Out,
                        side: PortSide::East,
                    });
                }
                
                elements.push(SemanticElement {
                    id: comp_id.clone(),
                    name: comp.name.clone(),
                    element_type: CapellaElementType::LogicalComponent,
                    layer: ArchitecturalLayer::Logical,
                    parent_id: None,
                    children: Vec::new(),
                    attributes: self.extract_attributes(&comp.attributes),
                    metadata,
                    ports,
                    allocated_to: Vec::new(),
                    allocated_from: Vec::new(),
                });
                
                for func in &comp.functions {
                    let func_id = func.attributes.get("id")
                        .and_then(|v| v.as_string())
                        .unwrap_or(&func.name)
                        .to_string();
                    
                    let func_metadata = self.metamodel
                        .get_metadata(&CapellaElementType::LogicalFunction)
                        .ok_or("No metadata for LogicalFunction")?
                        .clone();
                    
                    elements.push(SemanticElement {
                        id: func_id.clone(),
                        name: func.name.clone(),
                        element_type: CapellaElementType::LogicalFunction,
                        layer: ArchitecturalLayer::Logical,
                        parent_id: Some(comp_id.clone()),
                        children: Vec::new(),
                        attributes: self.extract_attributes(&func.attributes),
                        metadata: func_metadata,
                        ports: self.create_function_ports(&func.name),
                        allocated_to: Vec::new(),
                        allocated_from: Vec::new(),
                    });
                    
                    relationships.push(SemanticRelationship {
                        id: format!("allocates_{}_{}", comp_id, func_id),
                        source_id: comp_id.clone(),
                        target_id: func_id.clone(),
                        relationship_type: RelationshipType::Allocates,
                        exchange_items: Vec::new(),
                        attributes: HashMap::new(),
                    });
                }
            }
            
            for interface in &la.interfaces {
                relationships.push(SemanticRelationship {
                    id: format!("interface_{}_{}", interface.from, interface.to),
                    source_id: interface.from.clone(),
                    target_id: interface.to.clone(),
                    relationship_type: RelationshipType::ComponentExchange,
                    exchange_items: vec![interface.name.clone()],
                    attributes: HashMap::new(),
                });
            }
        }
        
        Ok(())
    }
    
    fn analyze_physical_architecture(
        &self,
        physical_architectures: &[PhysicalArchitecture],
        elements: &mut Vec<SemanticElement>,
        relationships: &mut Vec<SemanticRelationship>,
    ) -> Result<(), String> {
        for pa in physical_architectures {
            for node in &pa.nodes {
                let node_id = node.attributes.get("id")
                    .and_then(|v| v.as_string())
                    .unwrap_or(&node.name)
                    .to_string();
                
                let metadata = self.metamodel.get_metadata(&CapellaElementType::NodeComponent)
                    .ok_or("No metadata for NodeComponent")?
                    .clone();
                
                elements.push(SemanticElement {
                    id: node_id.clone(),
                    name: node.name.clone(),
                    element_type: CapellaElementType::NodeComponent,
                    layer: ArchitecturalLayer::Physical,
                    parent_id: None,
                    children: Vec::new(),
                    attributes: self.extract_attributes(&node.attributes),
                    metadata,
                    ports: Vec::new(),
                    allocated_to: Vec::new(),
                    allocated_from: Vec::new(),
                });
            }
        }
        
        Ok(())
    }
    
    fn analyze_traces(
        &self,
        traces: &[Trace],
        relationships: &mut Vec<SemanticRelationship>,
    ) -> Result<(), String> {
        for trace in traces {
            relationships.push(SemanticRelationship {
                id: format!("trace_{}_{}", trace.from, trace.to),
                source_id: trace.from.clone(),
                target_id: trace.to.clone(),
                relationship_type: RelationshipType::Traces,
                exchange_items: Vec::new(),
                attributes: self.extract_attributes(&trace.attributes),
            });
        }
        
        Ok(())
    }
    
    fn create_function_ports(&self, function_name: &str) -> Vec<SemanticPort> {
        vec![
            SemanticPort {
                id: format!("{}_in", function_name),
                name: "input".to_string(),
                port_type: CapellaElementType::FunctionInputPort,
                direction: PortDirection::In,
                side: PortSide::West,
            },
            SemanticPort {
                id: format!("{}_out", function_name),
                name: "output".to_string(),
                port_type: CapellaElementType::FunctionOutputPort,
                direction: PortDirection::Out,
                side: PortSide::East,
            },
        ]
    }
    
    fn extract_attributes(&self, attrs: &HashMap<String, AttributeValue>) -> HashMap<String, String> {
        attrs.iter()
            .filter_map(|(k, v)| {
                v.as_string().map(|s| (k.clone(), s.to_string()))
            })
            .collect()
    }
    
    fn detect_patterns(
        &self,
        elements: &[SemanticElement],
        relationships: &[SemanticRelationship],
    ) -> Vec<ArchitecturalPattern> {
        let mut patterns = Vec::new();
        
        if self.has_layered_structure(elements) {
            patterns.push(ArchitecturalPattern {
                pattern_type: PatternType::LayeredArchitecture,
                elements: elements.iter().map(|e| e.id.clone()).collect(),
                description: "Detected layered architecture with clear separation between operational, system, logical, and physical layers".to_string(),
                confidence: 0.9,
            });
        }
        
        if self.has_dataflow_pattern(relationships) {
            patterns.push(ArchitecturalPattern {
                pattern_type: PatternType::DataFlow,
                elements: self.find_dataflow_elements(relationships),
                description: "Detected data flow pattern with functional exchanges".to_string(),
                confidence: 0.85,
            });
        }
        
        if self.has_control_loop_pattern(relationships) {
            patterns.push(ArchitecturalPattern {
                pattern_type: PatternType::ControlLoop,
                elements: self.find_control_loop_elements(elements, relationships),
                description: "Detected control loop pattern with feedback".to_string(),
                confidence: 0.8,
            });
        }
        
        patterns
    }
    
    fn has_layered_structure(&self, elements: &[SemanticElement]) -> bool {
        let layers: Vec<ArchitecturalLayer> = elements.iter()
            .map(|e| e.layer.clone())
            .collect();
        
        let has_operational = layers.contains(&ArchitecturalLayer::Operational);
        let has_system = layers.contains(&ArchitecturalLayer::System);
        let has_logical = layers.contains(&ArchitecturalLayer::Logical);
        
        has_operational && has_system && has_logical
    }
    
    fn has_dataflow_pattern(&self, relationships: &[SemanticRelationship]) -> bool {
        relationships.iter().any(|r| {
            matches!(r.relationship_type, 
                RelationshipType::FunctionalExchange | 
                RelationshipType::ComponentExchange)
        })
    }
    
    fn has_control_loop_pattern(&self, relationships: &[SemanticRelationship]) -> bool {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        
        for rel in relationships {
            graph.entry(rel.source_id.clone())
                .or_insert_with(Vec::new)
                .push(rel.target_id.clone());
        }
        
        for (node, targets) in &graph {
            for target in targets {
                if let Some(target_edges) = graph.get(target) {
                    if target_edges.contains(node) {
                        return true;
                    }
                }
            }
        }
        
        false
    }
    
    fn find_dataflow_elements(&self, relationships: &[SemanticRelationship]) -> Vec<String> {
        relationships.iter()
            .filter(|r| matches!(r.relationship_type, 
                RelationshipType::FunctionalExchange | 
                RelationshipType::ComponentExchange))
            .flat_map(|r| vec![r.source_id.clone(), r.target_id.clone()])
            .collect()
    }
    
    fn find_control_loop_elements(
        &self,
        _elements: &[SemanticElement],
        relationships: &[SemanticRelationship],
    ) -> Vec<String> {
        let mut loop_elements = Vec::new();
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        
        for rel in relationships {
            graph.entry(rel.source_id.clone())
                .or_insert_with(Vec::new)
                .push(rel.target_id.clone());
        }
        
        for (node, targets) in &graph {
            for target in targets {
                if let Some(target_edges) = graph.get(target) {
                    if target_edges.contains(node) {
                        loop_elements.push(node.clone());
                        loop_elements.push(target.clone());
                    }
                }
            }
        }
        
        loop_elements
    }
    
    fn organize_by_layers(&self, elements: &[SemanticElement]) -> Vec<ArchitecturalLayerInfo> {
        let mut layers_map: HashMap<ArchitecturalLayer, Vec<String>> = HashMap::new();
        
        for element in elements {
            layers_map.entry(element.layer.clone())
                .or_insert_with(Vec::new)
                .push(element.id.clone());
        }
        
        layers_map.into_iter()
            .map(|(layer, elements)| ArchitecturalLayerInfo {
                layer,
                elements,
                dependencies: Vec::new(),
            })
            .collect()
    }
}

impl Default for EnhancedSemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
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
