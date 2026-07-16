//! Arcadia Rules Engine for Methodology Compliance
//! 
//! This module enforces phase-specific Arcadia methodology rules:
//! - Operational Analysis (OA): Actor positioning, activity containment
//! - System Analysis (SA): Function categorization, data flow
//! - Logical Architecture (LA): Interface notation, component colors, safety
//! - Physical Architecture (PA): ECU representation, deployment nesting

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::compiler::semantic_analyzer::{ArcadiaPhase, ElementType, SemanticContext};

/// Rule application result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleApplicationResult {
    pub rules_applied: usize,
    pub rules_passed: usize,
    pub rules_failed: usize,
    pub violations: Vec<RuleViolation>,
}

/// Rule violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleViolation {
    pub rule_name: String,
    pub element_id: String,
    pub severity: Severity,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Rule definition
pub struct ArcadiaRule {
    pub name: String,
    pub phase: ArcadiaPhase,
    pub description: String,
    pub severity: Severity,
    pub check_fn: fn(&Value, &SemanticContext) -> RuleCheckResult,
    pub apply_fn: fn(&mut Value, &SemanticContext),
}

/// Rule check result
pub struct RuleCheckResult {
    pub passed: bool,
    pub violations: Vec<(String, String)>,  // (element_id, message)
}

/// Arcadia Rules Engine
pub struct ArcadiaRulesEngine {
    rules: HashMap<ArcadiaPhase, Vec<ArcadiaRule>>,
}

impl ArcadiaRulesEngine {
    pub fn new() -> Self {
        let mut rules: HashMap<ArcadiaPhase, Vec<ArcadiaRule>> = HashMap::new();
        
        rules.insert(ArcadiaPhase::Operational, Self::operational_rules());
        rules.insert(ArcadiaPhase::System, Self::system_rules());
        rules.insert(ArcadiaPhase::Logical, Self::logical_rules());
        rules.insert(ArcadiaPhase::Physical, Self::physical_rules());
        
        ArcadiaRulesEngine { rules }
    }
    
    /// Apply all rules for a given phase
    pub fn apply(&self, diagram_data: &mut Value, semantic: &SemanticContext) -> RuleApplicationResult {
        let phase_rules = match self.rules.get(&semantic.phase) {
            Some(rules) => rules,
            None => return RuleApplicationResult {
                rules_applied: 0,
                rules_passed: 0,
                rules_failed: 0,
                violations: vec![],
            },
        };
        
        let mut violations = Vec::new();
        let mut rules_passed = 0;
        let mut rules_failed = 0;
        
        for rule in phase_rules {
            // Check rule
            let check_result = (rule.check_fn)(diagram_data, semantic);
            
            if check_result.passed {
                rules_passed += 1;
            } else {
                rules_failed += 1;
                for (elem_id, message) in check_result.violations {
                    violations.push(RuleViolation {
                        rule_name: rule.name.clone(),
                        element_id: elem_id,
                        severity: rule.severity.clone(),
                        message,
                    });
                }
            }
            
            // Apply rule transformations
            (rule.apply_fn)(diagram_data, semantic);
        }
        
        RuleApplicationResult {
            rules_applied: phase_rules.len(),
            rules_passed,
            rules_failed,
            violations,
        }
    }
    
    /// Operational Analysis Rules
    fn operational_rules() -> Vec<ArcadiaRule> {
        vec![
            ArcadiaRule {
                name: "OA-01-ActorsBoundary".to_string(),
                phase: ArcadiaPhase::Operational,
                description: "Actors must be positioned at system boundaries".to_string(),
                severity: Severity::Error,
                check_fn: |diagram_data, semantic| {
                    let mut violations = Vec::new();
                    
                    // Check if actors exist
                    let has_actors = semantic.elements.iter()
                        .any(|e| e.element_type == ElementType::Actor);
                    
                    if !has_actors && semantic.phase == ArcadiaPhase::Operational {
                        violations.push(("system".to_string(), 
                            "Operational diagrams should contain actors".to_string()));
                    }
                    
                    RuleCheckResult {
                        passed: violations.is_empty(),
                        violations,
                    }
                },
                apply_fn: |diagram_data, semantic| {
                    // Ensure actors are at boundaries (x=0 or x=max)
                    if let Some(nodes) = diagram_data.get_mut("nodes").and_then(|n| n.as_array_mut()) {
                        for node in nodes {
                            if let Some(node_type) = node.get("type").and_then(|t| t.as_str()) {
                                if node_type == "actor" {
                                    node["position"] = json!("boundary");
                                    node["boundary_style"] = json!("external");
                                }
                            }
                        }
                    }
                },
            },
            ArcadiaRule {
                name: "OA-02-ActivityContainment".to_string(),
                phase: ArcadiaPhase::Operational,
                description: "Activities must be inside system boundary".to_string(),
                severity: Severity::Error,
                check_fn: |_diagram_data, semantic| {
                    let activities = semantic.elements.iter()
                        .filter(|e| e.element_type == ElementType::Activity)
                        .count();
                    
                    RuleCheckResult {
                        passed: activities > 0 || semantic.phase != ArcadiaPhase::Operational,
                        violations: if activities == 0 && semantic.phase == ArcadiaPhase::Operational {
                            vec![("system".to_string(), "No activities defined in operational analysis".to_string())]
                        } else {
                            vec![]
                        },
                    }
                },
                apply_fn: |diagram_data, _semantic| {
                    // Mark activities as internal
                    if let Some(nodes) = diagram_data.get_mut("nodes").and_then(|n| n.as_array_mut()) {
                        for node in nodes {
                            if let Some(node_type) = node.get("type").and_then(|t| t.as_str()) {
                                if node_type == "activity" {
                                    node["containment"] = json!("system_boundary");
                                }
                            }
                        }
                    }
                },
            },
            ArcadiaRule {
                name: "OA-03-SwimlaneLayout".to_string(),
                phase: ArcadiaPhase::Operational,
                description: "Use swimlane layout for operational diagrams".to_string(),
                severity: Severity::Warning,
                check_fn: |_diagram_data, _semantic| {
                    RuleCheckResult {
                        passed: true,
                        violations: vec![],
                    }
                },
                apply_fn: |diagram_data, _semantic| {
                    // Set layout hint
                    if let Some(obj) = diagram_data.as_object_mut() {
                        obj.insert("layout_strategy".to_string(), json!("swimlane"));
                        obj.insert("partition_by".to_string(), json!("actor"));
                    }
                },
            },
        ]
    }
    
    /// System Analysis Rules
    fn system_rules() -> Vec<ArcadiaRule> {
        vec![
            ArcadiaRule {
                name: "SA-01-FunctionCategorization".to_string(),
                phase: ArcadiaPhase::System,
                description: "Functions must be categorized (Environmental/System/Management)".to_string(),
                severity: Severity::Warning,
                check_fn: |_diagram_data, semantic| {
                    let functions = semantic.elements.iter()
                        .filter(|e| e.element_type == ElementType::Function)
                        .count();
                    
                    RuleCheckResult {
                        passed: functions > 0,
                        violations: if functions == 0 {
                            vec![("system".to_string(), "System analysis should define functions".to_string())]
                        } else {
                            vec![]
                        },
                    }
                },
                apply_fn: |diagram_data, _semantic| {
                    // Add category metadata
                    if let Some(nodes) = diagram_data.get_mut("nodes").and_then(|n| n.as_array_mut()) {
                        for node in nodes {
                            if let Some(node_type) = node.get("type").and_then(|t| t.as_str()) {
                                if node_type == "function" {
                                    if !node.get("category").is_some() {
                                        node["category"] = json!("System");
                                    }
                                }
                            }
                        }
                    }
                },
            },
            ArcadiaRule {
                name: "SA-02-DataFlowDirection".to_string(),
                phase: ArcadiaPhase::System,
                description: "Data flow must be clearly directional".to_string(),
                severity: Severity::Info,
                check_fn: |_diagram_data, _semantic| {
                    RuleCheckResult {
                        passed: true,
                        violations: vec![],
                    }
                },
                apply_fn: |diagram_data, _semantic| {
                    // Ensure edges have direction indicators
                    if let Some(edges) = diagram_data.get_mut("edges").and_then(|e| e.as_array_mut()) {
                        for edge in edges {
                            if !edge.get("direction").is_some() {
                                edge["direction"] = json!("forward");
                            }
                            edge["arrow"] = json!(true);
                        }
                    }
                },
            },
        ]
    }
    
    /// Logical Architecture Rules
    fn logical_rules() -> Vec<ArcadiaRule> {
        vec![
            ArcadiaRule {
                name: "LA-01-InterfaceNotation".to_string(),
                phase: ArcadiaPhase::Logical,
                description: "Components must show interface notation (lollipop/socket)".to_string(),
                severity: Severity::Error,
                check_fn: |_diagram_data, semantic| {
                    let has_interfaces = semantic.elements.iter()
                        .any(|e| !e.interfaces_in.is_empty() || !e.interfaces_out.is_empty());
                    
                    RuleCheckResult {
                        passed: has_interfaces,
                        violations: if !has_interfaces {
                            vec![("system".to_string(), "Logical components should define interfaces".to_string())]
                        } else {
                            vec![]
                        },
                    }
                },
                apply_fn: |diagram_data, semantic| {
                    // Add interface visual properties
                    if let Some(nodes) = diagram_data.get_mut("nodes").and_then(|n| n.as_array_mut()) {
                        for node in nodes {
                            if let Some(node_id) = node.get("id").and_then(|i| i.as_str()) {
                                // Find element in semantic context
                                if let Some(elem) = semantic.elements.iter().find(|e| e.id == node_id) {
                                    if !elem.interfaces_out.is_empty() {
                                        node["interface_provided"] = json!(true);
                                        node["interface_style"] = json!("lollipop");
                                    }
                                    if !elem.interfaces_in.is_empty() {
                                        node["interface_required"] = json!(true);
                                        node["interface_style_in"] = json!("socket");
                                    }
                                }
                            }
                        }
                    }
                },
            },
            ArcadiaRule {
                name: "LA-02-ComponentColors".to_string(),
                phase: ArcadiaPhase::Logical,
                description: "Components use Capella color scheme by stereotype".to_string(),
                severity: Severity::Warning,
                check_fn: |_diagram_data, _semantic| {
                    RuleCheckResult {
                        passed: true,
                        violations: vec![],
                    }
                },
                apply_fn: |diagram_data, semantic| {
                    // Apply Capella color scheme
                    let color_map: HashMap<&str, &str> = [
                        ("Sensor", "#70AD47"),
                        ("Controller", "#6495ED"),
                        ("Actuator", "#ED7D31"),
                        ("Generic", "#BFBFBF"),
                    ].iter().cloned().collect();
                    
                    if let Some(nodes) = diagram_data.get_mut("nodes").and_then(|n| n.as_array_mut()) {
                        for node in nodes {
                            if let Some(node_id) = node.get("id").and_then(|i| i.as_str()) {
                                if let Some(elem) = semantic.elements.iter().find(|e| e.id == node_id) {
                                    let stereotype_str = format!("{:?}", elem.stereotype);
                                    if let Some(&color) = color_map.get(stereotype_str.as_str()) {
                                        node["fill"] = json!(color);
                                    }
                                }
                            }
                        }
                    }
                },
            },
            ArcadiaRule {
                name: "LA-03-SafetyBorders".to_string(),
                phase: ArcadiaPhase::Logical,
                description: "Safety-critical components must have visual borders".to_string(),
                severity: Severity::Error,
                check_fn: |_diagram_data, semantic| {
                    let has_safety_critical = semantic.elements.iter()
                        .any(|e| e.safety_level.is_some());
                    
                    RuleCheckResult {
                        passed: !semantic.has_safety_critical || has_safety_critical,
                        violations: vec![],
                    }
                },
                apply_fn: |diagram_data, semantic| {
                    // Apply safety borders
                    let safety_styles: HashMap<&str, (&str, u32)> = [
                        ("ASIL_D", ("#8B0000", 6)),
                        ("ASIL_C", ("#CC0000", 4)),
                        ("ASIL_B", ("#FF6B6B", 3)),
                        ("ASIL_A", ("#FFA500", 2)),
                    ].iter().cloned().collect();
                    
                    if let Some(nodes) = diagram_data.get_mut("nodes").and_then(|n| n.as_array_mut()) {
                        for node in nodes {
                            if let Some(node_id) = node.get("id").and_then(|i| i.as_str()) {
                                if let Some(elem) = semantic.elements.iter().find(|e| e.id == node_id) {
                                    if let Some(ref safety_level) = elem.safety_level {
                                        if let Some(&(color, width)) = safety_styles.get(safety_level.as_str()) {
                                            node["stroke"] = json!(color);
                                            node["stroke_width"] = json!(width);
                                            node["safety_indicator"] = json!(true);
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
            },
        ]
    }
    
    /// Physical Architecture Rules
    fn physical_rules() -> Vec<ArcadiaRule> {
        vec![
            ArcadiaRule {
                name: "PA-01-ECURepresentation".to_string(),
                phase: ArcadiaPhase::Physical,
                description: "ECUs must use 3D representation with golden color".to_string(),
                severity: Severity::Warning,
                check_fn: |_diagram_data, semantic| {
                    let has_nodes = semantic.elements.iter()
                        .any(|e| e.element_type == ElementType::PhysicalNode);
                    
                    RuleCheckResult {
                        passed: has_nodes,
                        violations: if !has_nodes {
                            vec![("system".to_string(), "Physical architecture should define nodes/ECUs".to_string())]
                        } else {
                            vec![]
                        },
                    }
                },
                apply_fn: |diagram_data, _semantic| {
                    // Apply ECU styling
                    if let Some(nodes) = diagram_data.get_mut("nodes").and_then(|n| n.as_array_mut()) {
                        for node in nodes {
                            if let Some(node_type) = node.get("type").and_then(|t| t.as_str()) {
                                if node_type == "physical_node" || node_type == "ecu" {
                                    node["fill"] = json!("#FFE699");
                                    node["style"] = json!("3d");
                                    node["shadow"] = json!(true);
                                    node["ecu_indicator"] = json!(true);
                                }
                            }
                        }
                    }
                },
            },
            ArcadiaRule {
                name: "PA-02-NestedDeployment".to_string(),
                phase: ArcadiaPhase::Physical,
                description: "Behavior components must nest inside physical nodes".to_string(),
                severity: Severity::Error,
                check_fn: |_diagram_data, semantic| {
                    let has_hierarchy = semantic.elements.iter()
                        .any(|e| !e.contains.is_empty());
                    
                    RuleCheckResult {
                        passed: has_hierarchy,
                        violations: if !has_hierarchy {
                            vec![("system".to_string(), "Physical nodes should contain behavior components".to_string())]
                        } else {
                            vec![]
                        },
                    }
                },
                apply_fn: |diagram_data, semantic| {
                    // Ensure proper nesting
                    if let Some(nodes) = diagram_data.get_mut("nodes").and_then(|n| n.as_array_mut()) {
                        for node in nodes {
                            if let Some(node_id) = node.get("id").and_then(|i| i.as_str()) {
                                if let Some(elem) = semantic.elements.iter().find(|e| e.id == node_id) {
                                    if !elem.contains.is_empty() {
                                        node["nested_components"] = json!(elem.contains);
                                        node["show_allocation"] = json!(true);
                                    }
                                }
                            }
                        }
                    }
                },
            },
            ArcadiaRule {
                name: "PA-03-ShowSpecs".to_string(),
                phase: ArcadiaPhase::Physical,
                description: "Physical nodes should display processor/memory specs".to_string(),
                severity: Severity::Info,
                check_fn: |_diagram_data, _semantic| {
                    RuleCheckResult {
                        passed: true,
                        violations: vec![],
                    }
                },
                apply_fn: |diagram_data, _semantic| {
                    // Enable spec display
                    if let Some(nodes) = diagram_data.get_mut("nodes").and_then(|n| n.as_array_mut()) {
                        for node in nodes {
                            if let Some(node_type) = node.get("type").and_then(|t| t.as_str()) {
                                if node_type == "physical_node" || node_type == "ecu" {
                                    node["show_specs"] = json!(true);
                                }
                            }
                        }
                    }
                },
            },
        ]
    }
}

impl Default for ArcadiaRulesEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::semantic_analyzer::{
        ComplexityMetrics, ElementClassification, ElementStereotype,
        RelationshipAnalysis, RecommendedStrategy,
    };
    
    fn create_test_semantic(phase: ArcadiaPhase) -> SemanticContext {
        SemanticContext {
            phase,
            diagram_type: "test".to_string(),
            elements: vec![],
            relationships: RelationshipAnalysis {
                containment: vec![],
                connections: vec![],
                allocations: vec![],
                traces: vec![],
            },
            complexity: ComplexityMetrics {
                total_elements: 3,
                depth: 1,
                branching_factor: 1.5,
                has_cycles: false,
            },
            recommended_strategy: RecommendedStrategy::Hierarchy,
            has_actors: false,
            has_hierarchy: false,
            has_data_flow: false,
            has_safety_critical: false,
        }
    }
    
    #[test]
    fn test_arcadia_rules_engine_creation() {
        let engine = ArcadiaRulesEngine::new();
        
        assert!(engine.rules.contains_key(&ArcadiaPhase::Operational));
        assert!(engine.rules.contains_key(&ArcadiaPhase::System));
        assert!(engine.rules.contains_key(&ArcadiaPhase::Logical));
        assert!(engine.rules.contains_key(&ArcadiaPhase::Physical));
    }
    
    #[test]
    fn test_operational_rules_count() {
        let rules = ArcadiaRulesEngine::operational_rules();
        assert_eq!(rules.len(), 3);
        assert_eq!(rules[0].name, "OA-01-ActorsBoundary");
    }
    
    #[test]
    fn test_logical_rules_apply() {
        let engine = ArcadiaRulesEngine::new();
        let mut semantic = create_test_semantic(ArcadiaPhase::Logical);
        
        semantic.elements.push(ElementClassification {
            id: "comp1".to_string(),
            name: "Component1".to_string(),
            element_type: ElementType::Component,
            stereotype: ElementStereotype::Sensor,
            safety_level: Some("ASIL_D".to_string()),
            parent_id: None,
            contains: vec![],
            interfaces_in: vec!["IInterface".to_string()],
            interfaces_out: vec!["OInterface".to_string()],
        });
        
        let mut diagram_data = json!({
            "nodes": [
                {"id": "comp1", "type": "component"}
            ]
        });
        
        let result = engine.apply(&mut diagram_data, &semantic);
        
        assert!(result.rules_applied > 0);
        
        // Check that color was applied
        assert_eq!(diagram_data["nodes"][0]["fill"], json!("#70AD47"));
        
        // Check that safety border was applied
        assert_eq!(diagram_data["nodes"][0]["stroke"], json!("#8B0000"));
        assert_eq!(diagram_data["nodes"][0]["stroke_width"], json!(6));
    }
    
    #[test]
    fn test_physical_rules_apply() {
        let engine = ArcadiaRulesEngine::new();
        let mut semantic = create_test_semantic(ArcadiaPhase::Physical);
        semantic.has_hierarchy = true;
        
        semantic.elements.push(ElementClassification {
            id: "ecu1".to_string(),
            name: "ECU_Control".to_string(),
            element_type: ElementType::PhysicalNode,
            stereotype: ElementStereotype::Hardware,
            safety_level: None,
            parent_id: None,
            contains: vec!["bc1".to_string()],
            interfaces_in: vec![],
            interfaces_out: vec![],
        });
        
        let mut diagram_data = json!({
            "nodes": [
                {"id": "ecu1", "type": "physical_node"}
            ]
        });
        
        let result = engine.apply(&mut diagram_data, &semantic);
        
        assert!(result.rules_applied > 0);
        
        // Check ECU styling
        assert_eq!(diagram_data["nodes"][0]["fill"], json!("#FFE699"));
        assert_eq!(diagram_data["nodes"][0]["style"], json!("3d"));
        
        // Check nested components
        assert_eq!(diagram_data["nodes"][0]["nested_components"][0], json!("bc1"));
    }
    
    #[test]
    fn test_rule_violation_reporting() {
        let engine = ArcadiaRulesEngine::new();
        let semantic = create_test_semantic(ArcadiaPhase::Operational);
        
        let mut diagram_data = json!({
            "nodes": []
        });
        
        let result = engine.apply(&mut diagram_data, &semantic);
        
        // Should have violations for missing actors/activities
        assert!(result.rules_failed > 0);
        assert!(!result.violations.is_empty());
    }
}
