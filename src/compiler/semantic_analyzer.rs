//! Semantic Analyzer for ArcLang Models
//! Extracts MBSE-specific intelligence to enable context-aware diagram generation.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::compiler::ast;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ArcadiaPhase {
    Operational,  // OA
    System,       // SA
    Logical,      // LA
    Physical,     // PA
    EPBS,         // EPBS
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ElementType {
    Actor,
    Activity,
    Function,
    Component,
    Interface,
    PhysicalNode,
    BehaviorComponent,
    HardwareComponent,
    Requirement,
    Capability,
    Exchange,
    Deployment,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ElementStereotype {
    Sensor,
    Controller,
    Actuator,
    Human,
    System,
    Hardware,
    Software,
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementClassification {
    pub id: String,
    pub name: String,
    pub element_type: ElementType,
    pub stereotype: ElementStereotype,
    pub safety_level: Option<String>,
    pub parent_id: Option<String>,
    pub contains: Vec<String>,
    pub interfaces_in: Vec<String>,
    pub interfaces_out: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipAnalysis {
    pub containment: Vec<(String, String)>,  // (parent_id, child_id)
    pub connections: Vec<(String, String, String)>,  // (from_id, to_id, type)
    pub allocations: Vec<(String, String)>,  // (function_id, component_id)
    pub traces: Vec<(String, String)>,  // (requirement_id, element_id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    pub total_elements: usize,
    pub depth: usize,  // Max nesting depth
    pub branching_factor: f32,  // Avg connections per node
    pub has_cycles: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecommendedStrategy {
    Swimlane,      // For operational diagrams with actors
    Hierarchy,     // For component containment
    PortCentric,   // For data flow diagrams
    Layer,         // For multi-layer views
    Context,       // For context diagrams
    Tree,          // For hierarchical breakdowns
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticContext {
    pub phase: ArcadiaPhase,
    pub diagram_type: String,
    pub elements: Vec<ElementClassification>,
    pub relationships: RelationshipAnalysis,
    pub complexity: ComplexityMetrics,
    pub recommended_strategy: RecommendedStrategy,
    pub has_actors: bool,
    pub has_hierarchy: bool,
    pub has_data_flow: bool,
    pub has_safety_critical: bool,
}

impl SemanticContext {
    /// Derive the semantic context from the canonical `SemanticModel`
    /// produced by the main compilation pipeline.
    ///
    /// This is the single-derivation entry point: it replaces re-analyzing
    /// the AST (`SemanticAnalyzer::analyze`) on generator paths, so the
    /// already-computed canonical layer (stable UUIDs included) is the only
    /// source of semantic truth.
    pub fn from_model(model: &crate::compiler::semantic::SemanticModel) -> SemanticContext {
        let analyzer = SemanticAnalyzer::new();

        // --- Phase detection ---------------------------------------------
        // Equivalent to the AST section-presence check (OA > SA > LA > PA):
        // the canonical model tags every component with the level it came
        // from ("Operational" / "System" / "Logical" / "Physical" or a
        // custom layer), and system-analysis content also surfaces as
        // requirements / SystemFunction elements.
        let has_level = |level: &str| model.components.iter().any(|c| c.level == level);
        let has_element_type = |element_type: &str| {
            model.all_elements.values().any(|e| e.element_type == element_type)
        };

        let phase = if has_level("Operational") || has_element_type("Actor") || has_element_type("Entity") {
            ArcadiaPhase::Operational
        } else if has_level("System")
            || !model.requirements.is_empty()
            || has_element_type("SystemFunction")
            || has_element_type("SystemComponent")
        {
            ArcadiaPhase::System
        } else if has_level("Logical") || model.components.iter().any(|c| c.level != "Physical") {
            ArcadiaPhase::Logical
        } else if has_level("Physical") {
            ArcadiaPhase::Physical
        } else {
            ArcadiaPhase::System
        };

        let diagram_type = match phase {
            ArcadiaPhase::Operational => "operational",
            ArcadiaPhase::System => "functional",
            ArcadiaPhase::Logical => "component",
            ArcadiaPhase::Physical => "physical",
            ArcadiaPhase::EPBS => "component",
        }
        .to_string();

        // --- Element classification --------------------------------------
        // Owner map: allocated function id -> owning component id, so
        // function allocation is represented as containment.
        let mut function_owner: HashMap<&str, &str> = HashMap::new();
        for component in &model.components {
            for function_id in &component.functions {
                function_owner.insert(function_id.as_str(), component.id.as_str());
            }
        }

        let mut elements = Vec::new();
        let mut classified: std::collections::HashSet<&str> = std::collections::HashSet::new();

        for component in &model.components {
            let registry_type = model
                .all_elements
                .get(&component.id)
                .map(|e| e.element_type.as_str())
                .unwrap_or("Component");

            let (element_type, stereotype) = match registry_type {
                "Actor" => (ElementType::Actor, ElementStereotype::Human),
                "Entity" => {
                    let stereotype = if component.component_type == "Actor" {
                        ElementStereotype::Human
                    } else {
                        ElementStereotype::System
                    };
                    (ElementType::Actor, stereotype)
                }
                "Activity" | "OperationalActivity" => {
                    (ElementType::Activity, ElementStereotype::Generic)
                }
                _ if component.level == "Physical" => {
                    (ElementType::PhysicalNode, ElementStereotype::Hardware)
                }
                _ => (
                    ElementType::Component,
                    Self::stereotype_from_keywords(&component.component_type),
                ),
            };

            classified.insert(component.id.as_str());
            elements.push(ElementClassification {
                id: component.id.clone(),
                name: component.name.clone(),
                element_type,
                stereotype,
                safety_level: component
                    .safety_level
                    .clone()
                    .or_else(|| component.asil.clone()),
                parent_id: None,
                contains: component.functions.clone(),
                interfaces_in: component.interfaces_in.iter().map(|i| i.name.clone()).collect(),
                interfaces_out: component.interfaces_out.iter().map(|i| i.name.clone()).collect(),
            });
        }

        // Functions (logical component functions and operational activities)
        for function in &model.functions {
            if !classified.insert(function.id.as_str()) {
                continue;
            }
            elements.push(ElementClassification {
                id: function.id.clone(),
                name: function.name.clone(),
                element_type: ElementType::Function,
                stereotype: analyzer.infer_function_stereotype(&function.name),
                safety_level: None,
                parent_id: function_owner.get(function.id.as_str()).map(|s| s.to_string()),
                contains: Vec::new(),
                interfaces_in: function.inputs.clone(),
                interfaces_out: function.outputs.clone(),
            });
        }

        // System functions only exist in the element registry
        for element in model.all_elements.values() {
            if element.element_type == "SystemFunction" && classified.insert(element.id.as_str()) {
                elements.push(ElementClassification {
                    id: element.id.clone(),
                    name: element.name.clone(),
                    element_type: ElementType::Function,
                    stereotype: analyzer.infer_function_stereotype(&element.name),
                    safety_level: None,
                    parent_id: None,
                    contains: Vec::new(),
                    interfaces_in: Vec::new(),
                    interfaces_out: Vec::new(),
                });
            }
        }

        // Keep classification order deterministic regardless of HashMap iteration
        elements.sort_by(|a, b| a.id.cmp(&b.id));

        // --- Relationships -------------------------------------------------
        // Containment + port-name matching (same as the AST derivation), then
        // the explicit exchanges carried by the canonical model.
        let mut relationships = analyzer.analyze_relationships(&elements);
        for interface in &model.interfaces {
            let from = Self::resolve_endpoint(&interface.from, model);
            let to = Self::resolve_endpoint(&interface.to, model);
            let connection = (from, to, "interface".to_string());
            if !relationships.connections.contains(&connection) {
                relationships.connections.push(connection);
            }
        }

        let complexity = analyzer.assess_complexity(&elements, &relationships);

        let has_actors = elements.iter().any(|e| e.element_type == ElementType::Actor);
        let has_hierarchy = elements.iter().any(|e| !e.contains.is_empty());
        let has_data_flow = !relationships.connections.is_empty();
        let has_safety_critical = elements.iter().any(|e| e.safety_level.is_some());

        let recommended_strategy =
            analyzer.select_strategy(&phase, has_actors, has_hierarchy, has_data_flow);

        SemanticContext {
            phase,
            diagram_type,
            elements,
            relationships,
            complexity,
            recommended_strategy,
            has_actors,
            has_hierarchy,
            has_data_flow,
            has_safety_critical,
        }
    }

    /// Resolve an exchange endpoint ("Component", "Component.Port",
    /// "COMP-ID", ...) to a canonical element id, best effort: exact id,
    /// then element name, then legacy `PREFIX_...` port convention.
    pub fn resolve_endpoint(
        endpoint: &str,
        model: &crate::compiler::semantic::SemanticModel,
    ) -> String {
        let root = endpoint.split('.').next().unwrap_or(endpoint);

        if model.all_elements.contains_key(root) {
            return root.to_string();
        }
        if let Some(element) = model.all_elements.values().find(|e| e.name == root) {
            return element.id.clone();
        }
        let underscore_root = root.split('_').next().unwrap_or(root);
        if model.all_elements.contains_key(underscore_root) {
            return underscore_root.to_string();
        }
        root.to_string()
    }

    /// Infer a component stereotype from a free-form type string
    /// (e.g. the canonical `component_type`).
    fn stereotype_from_keywords(type_name: &str) -> ElementStereotype {
        let lower = type_name.to_lowercase();
        if lower.contains("sensor") {
            ElementStereotype::Sensor
        } else if lower.contains("controller") {
            ElementStereotype::Controller
        } else if lower.contains("actuator") {
            ElementStereotype::Actuator
        } else {
            ElementStereotype::Generic
        }
    }
}

pub struct SemanticAnalyzer;

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer
    }
    
    /// Main entry point: Analyze ArcLang model
    pub fn analyze(&self, model: &ast::Model) -> SemanticContext {
        // Detect Arcadia phase
        let phase = self.detect_phase(model);
        
        // Classify all elements
        let elements = self.classify_elements(model);
        
        // Analyze relationships
        let relationships = self.analyze_relationships(&elements);
        
        // Calculate complexity
        let complexity = self.assess_complexity(&elements, &relationships);
        
        // Detect key characteristics
        let has_actors = elements.iter().any(|e| e.element_type == ElementType::Actor);
        let has_hierarchy = elements.iter().any(|e| !e.contains.is_empty());
        let has_data_flow = !relationships.connections.is_empty();
        let has_safety_critical = elements.iter().any(|e| e.safety_level.is_some());
        
        // Recommend strategy
        let recommended_strategy = self.select_strategy(
            &phase,
            has_actors,
            has_hierarchy,
            has_data_flow,
        );
        
        SemanticContext {
            phase,
            diagram_type: self.infer_diagram_type(model),
            elements,
            relationships,
            complexity,
            recommended_strategy,
            has_actors,
            has_hierarchy,
            has_data_flow,
            has_safety_critical,
        }
    }
    
    /// Detect which Arcadia phase this model represents
    fn detect_phase(&self, model: &ast::Model) -> ArcadiaPhase {
        // Check for operational analysis elements
        if !model.operational_analysis.is_empty() {
            return ArcadiaPhase::Operational;
        }
        
        // Check for system analysis elements
        if !model.system_analysis.is_empty() {
            return ArcadiaPhase::System;
        }
        
        // Check for logical architecture elements
        if !model.logical_architecture.is_empty() {
            return ArcadiaPhase::Logical;
        }
        
        // Check for physical architecture elements
        if !model.physical_architecture.is_empty() {
            return ArcadiaPhase::Physical;
        }
        
        // Default to System Analysis
        ArcadiaPhase::System
    }
    
    /// Infer diagram type from model structure
    fn infer_diagram_type(&self, model: &ast::Model) -> String {
        if !model.operational_analysis.is_empty() {
            "operational".to_string()
        } else if !model.system_analysis.is_empty() {
            "functional".to_string()
        } else if !model.logical_architecture.is_empty() {
            "component".to_string()
        } else if !model.physical_architecture.is_empty() {
            "physical".to_string()
        } else {
            "component".to_string()
        }
    }
    
    /// Classify all elements by type and stereotype
    fn classify_elements(&self, model: &ast::Model) -> Vec<ElementClassification> {
        let mut elements = Vec::new();
        
        // Classify operational analysis elements - actors
        for oa in &model.operational_analysis {
            for actor in &oa.actors {
                let actor_id = actor.id.clone().unwrap_or_else(|| format!("ACTOR-{}", actor.name));
                elements.push(ElementClassification {
                    id: actor_id,
                    name: actor.name.clone(),
                    element_type: ElementType::Actor,
                    stereotype: ElementStereotype::Human,  // Default to human
                    safety_level: None,
                    parent_id: None,
                    contains: Vec::new(),
                    interfaces_in: Vec::new(),
                    interfaces_out: Vec::new(),
                });
            }
            
            // Activities
            for activity in &oa.activities {
                elements.push(ElementClassification {
                    id: activity.id.clone(),
                    name: activity.name.clone(),
                    element_type: ElementType::Activity,
                    stereotype: ElementStereotype::Generic,
                    safety_level: None,
                    parent_id: None,
                    contains: Vec::new(),
                    interfaces_in: Vec::new(),
                    interfaces_out: Vec::new(),
                });
            }
        }
        
        // Classify system analysis elements - functions
        for sa in &model.system_analysis {
            for function in &sa.functions {
                elements.push(ElementClassification {
                    id: function.id.clone(),
                    name: function.name.clone(),
                    element_type: ElementType::Function,
                    stereotype: self.infer_function_stereotype(&function.name),
                    safety_level: None,
                    parent_id: None,
                    contains: Vec::new(),
                    interfaces_in: Vec::new(),
                    interfaces_out: Vec::new(),
                });
            }
        }
        
        // Classify logical architecture elements - components
        for la in &model.logical_architecture {
            for component in &la.components {
                let interfaces_in: Vec<String> = component.interfaces_in
                    .iter()
                    .map(|i| i.name.clone())
                    .collect();
                    
                let interfaces_out: Vec<String> = component.interfaces_out
                    .iter()
                    .map(|i| i.name.clone())
                    .collect();
                
                elements.push(ElementClassification {
                    id: component.id.clone(),
                    name: component.name.clone(),
                    element_type: ElementType::Component,
                    stereotype: self.infer_component_stereotype(&component.attributes),
                    safety_level: self.extract_safety_level(&component.attributes),
                    parent_id: None,
                    contains: Vec::new(),
                    interfaces_in,
                    interfaces_out,
                });
            }
        }
        
        // Classify physical architecture elements - nodes
        for pa in &model.physical_architecture {
            for node in &pa.nodes {
                let mut child_ids = Vec::new();
                
                // Add behavior components
                for bc in &node.behavior_components {
                    child_ids.push(bc.id.clone());
                    elements.push(ElementClassification {
                        id: bc.id.clone(),
                        name: bc.name.clone(),
                        element_type: ElementType::BehaviorComponent,
                        stereotype: ElementStereotype::Software,
                        safety_level: None,
                        parent_id: Some(node.id.clone()),
                        contains: Vec::new(),
                        interfaces_in: Vec::new(),
                        interfaces_out: Vec::new(),
                    });
                }
                
                // Add hardware components
                for hc in &node.hardware_components {
                    child_ids.push(hc.id.clone());
                    elements.push(ElementClassification {
                        id: hc.id.clone(),
                        name: hc.name.clone(),
                        element_type: ElementType::HardwareComponent,
                        stereotype: ElementStereotype::Hardware,
                        safety_level: None,
                        parent_id: Some(node.id.clone()),
                        contains: Vec::new(),
                        interfaces_in: Vec::new(),
                        interfaces_out: Vec::new(),
                    });
                }
                
                // Add the node itself
                elements.push(ElementClassification {
                    id: node.id.clone(),
                    name: node.name.clone(),
                    element_type: ElementType::PhysicalNode,
                    stereotype: ElementStereotype::Hardware,
                    safety_level: None,
                    parent_id: None,
                    contains: child_ids,
                    interfaces_in: Vec::new(),
                    interfaces_out: Vec::new(),
                });
            }
        }
        
        elements
    }
    
    /// Extract safety level from attributes
    fn extract_safety_level(&self, attributes: &HashMap<String, ast::AttributeValue>) -> Option<String> {
        attributes.get("safety_level").and_then(|v| {
            if let ast::AttributeValue::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        })
    }
    
    /// Infer component stereotype from attributes
    fn infer_component_stereotype(&self, attributes: &HashMap<String, ast::AttributeValue>) -> ElementStereotype {
        // Check for explicit stereotype
        if let Some(ast::AttributeValue::String(stereotype)) = attributes.get("stereotype") {
            let stereo_lower = stereotype.to_lowercase().replace("<", "").replace(">", "");
            return match stereo_lower.as_str() {
                "sensor" => ElementStereotype::Sensor,
                "controller" => ElementStereotype::Controller,
                "actuator" => ElementStereotype::Actuator,
                _ => ElementStereotype::Generic,
            };
        }
        
        ElementStereotype::Generic
    }
    
    /// Infer function stereotype from name
    fn infer_function_stereotype(&self, name: &str) -> ElementStereotype {
        let name_lower = name.to_lowercase();
        
        if name_lower.contains("detect") || name_lower.contains("sense") || name_lower.contains("measure") {
            ElementStereotype::Sensor
        } else if name_lower.contains("control") || name_lower.contains("calculate") || name_lower.contains("decide") {
            ElementStereotype::Controller
        } else if name_lower.contains("apply") || name_lower.contains("actuate") || name_lower.contains("move") {
            ElementStereotype::Actuator
        } else {
            ElementStereotype::Generic
        }
    }
    
    /// Analyze relationships between elements
    fn analyze_relationships(&self, elements: &[ElementClassification]) -> RelationshipAnalysis {
        let mut containment = Vec::new();
        let mut connections = Vec::new();
        let allocations = Vec::new();
        let traces = Vec::new();
        
        // Build containment relationships
        for elem in elements {
            if let Some(parent_id) = &elem.parent_id {
                containment.push((parent_id.clone(), elem.id.clone()));
            }
            
            // Build interface connections
            for interface_out in &elem.interfaces_out {
                // Find potential targets
                for target in elements {
                    if target.interfaces_in.contains(interface_out) {
                        connections.push((
                            elem.id.clone(),
                            target.id.clone(),
                            "interface".to_string(),
                        ));
                    }
                }
            }
        }
        
        RelationshipAnalysis {
            containment,
            connections,
            allocations,
            traces,
        }
    }
    
    /// Calculate complexity metrics
    fn assess_complexity(
        &self,
        elements: &[ElementClassification],
        relationships: &RelationshipAnalysis,
    ) -> ComplexityMetrics {
        let total_elements = elements.len();
        
        // Calculate max depth
        let depth = self.calculate_max_depth(elements, relationships);
        
        // Calculate branching factor (avg connections per node)
        let total_connections = relationships.connections.len();
        let branching_factor = if total_elements > 0 {
            total_connections as f32 / total_elements as f32
        } else {
            0.0
        };
        
        // Detect cycles (simplified)
        let has_cycles = false;
        
        ComplexityMetrics {
            total_elements,
            depth,
            branching_factor,
            has_cycles,
        }
    }
    
    /// Calculate maximum nesting depth
    fn calculate_max_depth(
        &self,
        elements: &[ElementClassification],
        relationships: &RelationshipAnalysis,
    ) -> usize {
        let mut max_depth = 0;
        
        // For each element, calculate its depth
        for elem in elements {
            let depth = self.calculate_element_depth(&elem.id, relationships, 0);
            if depth > max_depth {
                max_depth = depth;
            }
        }
        
        max_depth
    }
    
    /// Calculate depth of a specific element
    fn calculate_element_depth(
        &self,
        elem_id: &str,
        relationships: &RelationshipAnalysis,
        current_depth: usize,
    ) -> usize {
        // Find parent
        let parent = relationships.containment
            .iter()
            .find(|(_, child)| child == elem_id)
            .map(|(parent, _)| parent);
        
        if let Some(parent_id) = parent {
            self.calculate_element_depth(parent_id, relationships, current_depth + 1)
        } else {
            current_depth
        }
    }
    
    /// Select best layout strategy based on model characteristics
    fn select_strategy(
        &self,
        phase: &ArcadiaPhase,
        has_actors: bool,
        has_hierarchy: bool,
        has_data_flow: bool,
    ) -> RecommendedStrategy {
        // Operational phase with actors → Swimlane
        if *phase == ArcadiaPhase::Operational && has_actors {
            return RecommendedStrategy::Swimlane;
        }
        
        // System phase with data flow → Port-Centric
        if *phase == ArcadiaPhase::System && has_data_flow {
            return RecommendedStrategy::PortCentric;
        }
        
        // Logical/Physical with hierarchy → Hierarchy
        if (*phase == ArcadiaPhase::Logical || *phase == ArcadiaPhase::Physical) && has_hierarchy {
            return RecommendedStrategy::Hierarchy;
        }
        
        // Physical phase → always use Hierarchy (for ECU nesting)
        if *phase == ArcadiaPhase::Physical {
            return RecommendedStrategy::Hierarchy;
        }
        
        // Default: Hierarchy
        RecommendedStrategy::Hierarchy
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_phase_logical() {
        let analyzer = SemanticAnalyzer::new();
        let mut model = ast::Model::new();
        
        model.logical_architecture.push(ast::LogicalArchitecture {
            capability_realizations: Vec::new(),
            functional_chains: Vec::new(),
            name: "Test LA".to_string(),
            components: vec![],
            component_exchanges: vec![],
            interfaces: vec![],
            unallocated_functions: vec![],
        });
        
        let phase = analyzer.detect_phase(&model);
        assert_eq!(phase, ArcadiaPhase::Logical);
    }
    
    #[test]
    fn test_infer_stereotype_sensor() {
        let analyzer = SemanticAnalyzer::new();
        let stereotype = analyzer.infer_function_stereotype("DetectObstacle");
        assert_eq!(stereotype, ElementStereotype::Sensor);
    }
    
    #[test]
    fn test_infer_stereotype_controller() {
        let analyzer = SemanticAnalyzer::new();
        let stereotype = analyzer.infer_function_stereotype("CalculateBraking");
        assert_eq!(stereotype, ElementStereotype::Controller);
    }
    
    #[test]
    fn test_infer_stereotype_actuator() {
        let analyzer = SemanticAnalyzer::new();
        let stereotype = analyzer.infer_function_stereotype("ApplyBrakes");
        assert_eq!(stereotype, ElementStereotype::Actuator);
    }
    
    #[test]
    fn test_strategy_selection_swimlane() {
        let analyzer = SemanticAnalyzer::new();
        let strategy = analyzer.select_strategy(
            &ArcadiaPhase::Operational,
            true,  // has actors
            false,
            false,
        );
        assert_eq!(strategy, RecommendedStrategy::Swimlane);
    }
    
    #[test]
    fn from_model_classifies_actor_and_logical_component() {
        use crate::compiler::semantic::{ComponentInfo, ElementInfo, SemanticModel};

        let mut model = SemanticModel::default();

        model.components.push(ComponentInfo {
            id: "ACT-001".to_string(),
            name: "Driver".to_string(),
            component_type: "Actor".to_string(),
            level: "Operational".to_string(),
            safety_level: None,
            asil: None,
            interfaces_in: Vec::new(),
            interfaces_out: Vec::new(),
            functions: Vec::new(),
        });
        model.all_elements.insert(
            "ACT-001".to_string(),
            ElementInfo::new("ACT-001", "Driver", "Actor"),
        );

        model.components.push(ComponentInfo {
            id: "LC-001".to_string(),
            name: "Radar Sensor".to_string(),
            component_type: "Sensor".to_string(),
            level: "Logical".to_string(),
            safety_level: Some("ASIL_B".to_string()),
            asil: None,
            interfaces_in: Vec::new(),
            interfaces_out: Vec::new(),
            functions: vec!["LF-001".to_string()],
        });
        model.all_elements.insert(
            "LC-001".to_string(),
            ElementInfo::new("LC-001", "Radar Sensor", "Component"),
        );

        let context = SemanticContext::from_model(&model);

        // Operational content wins phase detection (same priority as the AST path)
        assert_eq!(context.phase, ArcadiaPhase::Operational);
        assert_eq!(context.diagram_type, "operational");
        assert!(context.has_actors);
        assert!(context.has_hierarchy);
        assert!(context.has_safety_critical);
        assert_eq!(context.recommended_strategy, RecommendedStrategy::Swimlane);

        let actor = context.elements.iter().find(|e| e.id == "ACT-001").unwrap();
        assert_eq!(actor.element_type, ElementType::Actor);
        assert_eq!(actor.stereotype, ElementStereotype::Human);

        let component = context.elements.iter().find(|e| e.id == "LC-001").unwrap();
        assert_eq!(component.element_type, ElementType::Component);
        assert_eq!(component.stereotype, ElementStereotype::Sensor);
        assert_eq!(component.safety_level, Some("ASIL_B".to_string()));
        assert_eq!(component.contains, vec!["LF-001".to_string()]);
    }

    #[test]
    fn from_model_detects_logical_phase_without_operational_content() {
        use crate::compiler::semantic::{ComponentInfo, ElementInfo, InterfaceInfo, SemanticModel};

        let mut model = SemanticModel::default();
        for (id, name) in [("LC-001", "Radar"), ("LC-002", "Fusion")] {
            model.components.push(ComponentInfo {
                id: id.to_string(),
                name: name.to_string(),
                component_type: "Logical".to_string(),
                level: "Logical".to_string(),
                safety_level: None,
                asil: None,
                interfaces_in: Vec::new(),
                interfaces_out: Vec::new(),
                functions: Vec::new(),
            });
            model.all_elements.insert(
                id.to_string(),
                ElementInfo::new(id, name, "Component"),
            );
        }
        model.interfaces.push(InterfaceInfo {
            name: "detections".to_string(),
            from: "Radar.out".to_string(),
            to: "Fusion.in".to_string(),
        });

        let context = SemanticContext::from_model(&model);

        assert_eq!(context.phase, ArcadiaPhase::Logical);
        assert!(context.has_data_flow);
        // Endpoints are resolved to canonical element ids
        assert!(context
            .relationships
            .connections
            .contains(&("LC-001".to_string(), "LC-002".to_string(), "interface".to_string())));
    }

    #[test]
    fn test_strategy_selection_hierarchy() {
        let analyzer = SemanticAnalyzer::new();
        let strategy = analyzer.select_strategy(
            &ArcadiaPhase::Logical,
            false,
            true,  // has hierarchy
            false,
        );
        assert_eq!(strategy, RecommendedStrategy::Hierarchy);
    }
}
