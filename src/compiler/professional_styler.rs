//! Professional Styling System for Capella-Quality Diagrams
//! 
//! This module applies professional visual styling:
//! - Capella color scheme by stereotype
//! - Safety indicators (ASIL borders)
//! - Depth effects (shadows, gradients)
//! - Legends and annotations
//! - Typography and iconography

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::compiler::semantic_analyzer::{ArcadiaPhase, ElementType, ElementStereotype, SemanticContext};

/// Styling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleConfig {
    pub theme: Theme,
    pub enable_shadows: bool,
    pub enable_gradients: bool,
    pub enable_legend: bool,
    pub enable_grid: bool,
    pub color_blind_safe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Theme {
    Capella,
    CapellaLight,
    HighContrast,
    Monochrome,
}

impl Default for StyleConfig {
    fn default() -> Self {
        StyleConfig {
            theme: Theme::Capella,
            enable_shadows: true,
            enable_gradients: true,
            enable_legend: true,
            enable_grid: false,
            color_blind_safe: false,
        }
    }
}

/// Color scheme definition
#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub actor: String,
    pub sensor: String,
    pub controller: String,
    pub actuator: String,
    pub function: String,
    pub component: String,
    pub ecu: String,
    pub generic: String,
    pub background: String,
    pub grid: String,
    pub text: String,
}

impl ColorScheme {
    /// Capella standard color scheme
    pub fn capella() -> Self {
        ColorScheme {
            actor: "#E8F4F8".to_string(),
            sensor: "#70AD47".to_string(),
            controller: "#6495ED".to_string(),
            actuator: "#ED7D31".to_string(),
            function: "#70AD47".to_string(),
            component: "#BFBFBF".to_string(),
            ecu: "#FFE699".to_string(),
            generic: "#BFBFBF".to_string(),
            background: "#FFFFFF".to_string(),
            grid: "#E0E0E0".to_string(),
            text: "#000000".to_string(),
        }
    }
    
    /// High contrast color scheme (accessibility)
    pub fn high_contrast() -> Self {
        ColorScheme {
            actor: "#FFFF00".to_string(),
            sensor: "#00FF00".to_string(),
            controller: "#0000FF".to_string(),
            actuator: "#FF0000".to_string(),
            function: "#00FFFF".to_string(),
            component: "#FFFFFF".to_string(),
            ecu: "#FFAA00".to_string(),
            generic: "#CCCCCC".to_string(),
            background: "#000000".to_string(),
            grid: "#333333".to_string(),
            text: "#FFFFFF".to_string(),
        }
    }
    
    /// Color-blind safe scheme (deuteranopia-friendly)
    pub fn color_blind_safe() -> Self {
        ColorScheme {
            actor: "#E8F4F8".to_string(),
            sensor: "#0173B2".to_string(),      // Blue
            controller: "#DE8F05".to_string(),  // Orange
            actuator: "#CC78BC".to_string(),    // Purple
            function: "#0173B2".to_string(),
            component: "#BFBFBF".to_string(),
            ecu: "#FFE699".to_string(),
            generic: "#949494".to_string(),
            background: "#FFFFFF".to_string(),
            grid: "#E0E0E0".to_string(),
            text: "#000000".to_string(),
        }
    }
}

/// Professional Styler
pub struct ProfessionalStyler {
    config: StyleConfig,
    color_scheme: ColorScheme,
}

impl ProfessionalStyler {
    pub fn new(config: StyleConfig) -> Self {
        let color_scheme = match config.theme {
            Theme::Capella | Theme::CapellaLight => {
                if config.color_blind_safe {
                    ColorScheme::color_blind_safe()
                } else {
                    ColorScheme::capella()
                }
            },
            Theme::HighContrast => ColorScheme::high_contrast(),
            Theme::Monochrome => ColorScheme::capella(), // TODO: implement monochrome
        };
        
        ProfessionalStyler {
            config,
            color_scheme,
        }
    }
    
    /// Main entry point: Apply all styling
    pub fn apply_styles(&self, diagram_data: &mut Value, semantic: &SemanticContext) {
        // Apply color coding
        self.apply_color_coding(diagram_data, semantic);
        
        // Apply safety indicators
        self.apply_safety_indicators(diagram_data, semantic);
        
        // Add depth effects
        if self.config.enable_shadows {
            self.add_depth_effects(diagram_data);
        }
        
        // Add legend
        if self.config.enable_legend {
            self.add_legend(diagram_data, semantic);
        }
        
        // Add grid
        if self.config.enable_grid {
            self.add_grid(diagram_data);
        }
        
        // Apply typography
        self.apply_typography(diagram_data);
    }
    
    /// Apply Capella color scheme
    fn apply_color_coding(&self, diagram_data: &mut Value, semantic: &SemanticContext) {
        if let Some(nodes) = diagram_data.get_mut("nodes").and_then(|n| n.as_array_mut()) {
            for node in nodes {
                if let Some(node_id) = node.get("id").and_then(|i| i.as_str()) {
                    // Find element in semantic context
                    if let Some(elem) = semantic.elements.iter().find(|e| e.id == node_id) {
                        let color = match elem.stereotype {
                            ElementStereotype::Sensor => &self.color_scheme.sensor,
                            ElementStereotype::Controller => &self.color_scheme.controller,
                            ElementStereotype::Actuator => &self.color_scheme.actuator,
                            ElementStereotype::Human => &self.color_scheme.actor,
                            ElementStereotype::Hardware => &self.color_scheme.ecu,
                            ElementStereotype::Software => &self.color_scheme.component,
                            ElementStereotype::Generic => &self.color_scheme.generic,
                            _ => &self.color_scheme.generic,
                        };
                        
                        node["fill"] = json!(color);
                        
                        // Add subtle gradient if enabled
                        if self.config.enable_gradients {
                            node["gradient"] = json!(true);
                            node["gradient_direction"] = json!("vertical");
                            node["gradient_opacity"] = json!(0.2);
                        }
                    }
                }
            }
        }
    }
    
    /// Apply safety indicators
    fn apply_safety_indicators(&self, diagram_data: &mut Value, semantic: &SemanticContext) {
        let safety_styles: HashMap<&str, (&str, u32)> = [
            ("ASIL_D", ("#8B0000", 6)),
            ("ASIL_C", ("#CC0000", 4)),
            ("ASIL_B", ("#FF6B6B", 3)),
            ("ASIL_A", ("#FFA500", 2)),
            ("QM", ("#808080", 1)),
        ].iter().cloned().collect();
        
        if let Some(nodes) = diagram_data.get_mut("nodes").and_then(|n| n.as_array_mut()) {
            for node in nodes {
                if let Some(node_id) = node.get("id").and_then(|i| i.as_str()) {
                    if let Some(elem) = semantic.elements.iter().find(|e| e.id == node_id) {
                        if let Some(ref safety_level) = elem.safety_level {
                            if let Some(&(color, width)) = safety_styles.get(safety_level.as_str()) {
                                node["stroke"] = json!(color);
                                node["stroke_width"] = json!(width);
                                node["safety_badge"] = json!(safety_level);
                                
                                // Add corner badge
                                node["corner_badge"] = json!({
                                    "text": safety_level,
                                    "position": "top-right",
                                    "color": color
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// Add depth effects (shadows, 3D)
    fn add_depth_effects(&self, diagram_data: &mut Value) {
        if let Some(nodes) = diagram_data.get_mut("nodes").and_then(|n| n.as_array_mut()) {
            for node in nodes {
                // Add drop shadow
                node["shadow"] = json!({
                    "enabled": true,
                    "offset_x": 2,
                    "offset_y": 2,
                    "blur": 4,
                    "color": "rgba(0, 0, 0, 0.2)"
                });
                
                // Add 3D effect for ECUs
                if let Some(node_type) = node.get("type").and_then(|t| t.as_str()) {
                    if node_type == "physical_node" || node_type == "ecu" {
                        node["style_3d"] = json!(true);
                        node["extrusion_depth"] = json!(8);
                    }
                }
            }
        }
    }
    
    /// Add legend to diagram
    fn add_legend(&self, diagram_data: &mut Value, semantic: &SemanticContext) {
        let mut legend_items: Vec<Value> = Vec::new();
        
        // Collect unique stereotypes from semantic context
        let mut seen_stereotypes: HashMap<String, bool> = HashMap::new();
        
        for elem in &semantic.elements {
            let stereotype_key = format!("{:?}", elem.stereotype);
            if !seen_stereotypes.contains_key(&stereotype_key) {
                seen_stereotypes.insert(stereotype_key.clone(), true);
                
                let color = match elem.stereotype {
                    ElementStereotype::Sensor => &self.color_scheme.sensor,
                    ElementStereotype::Controller => &self.color_scheme.controller,
                    ElementStereotype::Actuator => &self.color_scheme.actuator,
                    ElementStereotype::Human => &self.color_scheme.actor,
                    ElementStereotype::Hardware => &self.color_scheme.ecu,
                    ElementStereotype::Software => &self.color_scheme.component,
                    _ => &self.color_scheme.generic,
                };
                
                legend_items.push(json!({
                    "label": stereotype_key,
                    "color": color,
                    "shape": "rectangle"
                }));
            }
        }
        
        // Add safety legend if applicable
        if semantic.has_safety_critical {
            legend_items.push(json!({
                "label": "Safety Critical",
                "color": "#8B0000",
                "shape": "border",
                "note": "Red border indicates ASIL-D"
            }));
        }
        
        if !legend_items.is_empty() {
            let legend = json!({
                "title": format!("{:?} - {}", semantic.phase, semantic.diagram_type),
                "items": legend_items,
                "position": "bottom-right",
                "background": "#FFFFFF",
                "border": true
            });
            
            if let Some(obj) = diagram_data.as_object_mut() {
                obj.insert("legend".to_string(), legend);
            }
        }
    }
    
    /// Add grid to diagram
    fn add_grid(&self, diagram_data: &mut Value) {
        if let Some(obj) = diagram_data.as_object_mut() {
            obj.insert("grid".to_string(), json!({
                "enabled": true,
                "size": 10,
                "color": self.color_scheme.grid,
                "style": "dotted"
            }));
        }
    }
    
    /// Apply professional typography
    fn apply_typography(&self, diagram_data: &mut Value) {
        let typography = json!({
            "title_font": "Arial, sans-serif",
            "title_size": 16,
            "title_weight": "bold",
            "label_font": "Arial, sans-serif",
            "label_size": 12,
            "label_weight": "normal",
            "annotation_font": "Arial, sans-serif",
            "annotation_size": 10,
            "annotation_style": "italic"
        });
        
        if let Some(obj) = diagram_data.as_object_mut() {
            obj.insert("typography".to_string(), typography);
        }
        
        // Apply to nodes
        if let Some(nodes) = diagram_data.get_mut("nodes").and_then(|n| n.as_array_mut()) {
            for node in nodes {
                node["font_family"] = json!("Arial, sans-serif");
                node["font_size"] = json!(12);
                node["text_color"] = json!(self.color_scheme.text);
            }
        }
    }
}

impl Default for ProfessionalStyler {
    fn default() -> Self {
        ProfessionalStyler::new(StyleConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::semantic_analyzer::{
        ComplexityMetrics, ElementClassification, RelationshipAnalysis, RecommendedStrategy,
    };
    
    fn create_test_semantic() -> SemanticContext {
        let mut semantic = SemanticContext {
            phase: ArcadiaPhase::Logical,
            diagram_type: "component".to_string(),
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
            has_hierarchy: true,
            has_data_flow: true,
            has_safety_critical: true,
        };
        
        semantic.elements.push(ElementClassification {
            id: "comp1".to_string(),
            name: "Radar".to_string(),
            element_type: ElementType::Component,
            stereotype: ElementStereotype::Sensor,
            safety_level: Some("ASIL_D".to_string()),
            parent_id: None,
            contains: vec![],
            interfaces_in: vec![],
            interfaces_out: vec!["IObstacleData".to_string()],
        });
        
        semantic.elements.push(ElementClassification {
            id: "comp2".to_string(),
            name: "Controller".to_string(),
            element_type: ElementType::Component,
            stereotype: ElementStereotype::Controller,
            safety_level: None,
            parent_id: None,
            contains: vec![],
            interfaces_in: vec!["IObstacleData".to_string()],
            interfaces_out: vec!["IBrakingCommand".to_string()],
        });
        
        semantic
    }
    
    #[test]
    fn test_color_scheme_capella() {
        let scheme = ColorScheme::capella();
        assert_eq!(scheme.sensor, "#70AD47");
        assert_eq!(scheme.controller, "#6495ED");
        assert_eq!(scheme.actuator, "#ED7D31");
    }
    
    #[test]
    fn test_professional_styler_creation() {
        let styler = ProfessionalStyler::default();
        assert_eq!(styler.config.theme, Theme::Capella);
        assert!(styler.config.enable_shadows);
    }
    
    #[test]
    fn test_apply_color_coding() {
        let styler = ProfessionalStyler::default();
        let semantic = create_test_semantic();
        
        let mut diagram_data = json!({
            "nodes": [
                {"id": "comp1", "type": "component"},
                {"id": "comp2", "type": "component"}
            ]
        });
        
        styler.apply_color_coding(&mut diagram_data, &semantic);
        
        // Sensor should be green
        assert_eq!(diagram_data["nodes"][0]["fill"], json!("#70AD47"));
        
        // Controller should be blue
        assert_eq!(diagram_data["nodes"][1]["fill"], json!("#6495ED"));
    }
    
    #[test]
    fn test_apply_safety_indicators() {
        let styler = ProfessionalStyler::default();
        let semantic = create_test_semantic();
        
        let mut diagram_data = json!({
            "nodes": [
                {"id": "comp1", "type": "component"}
            ]
        });
        
        styler.apply_safety_indicators(&mut diagram_data, &semantic);
        
        // ASIL-D component should have red border
        assert_eq!(diagram_data["nodes"][0]["stroke"], json!("#8B0000"));
        assert_eq!(diagram_data["nodes"][0]["stroke_width"], json!(6));
        assert_eq!(diagram_data["nodes"][0]["safety_badge"], json!("ASIL_D"));
    }
    
    #[test]
    fn test_add_legend() {
        let styler = ProfessionalStyler::default();
        let semantic = create_test_semantic();
        
        let mut diagram_data = json!({
            "nodes": []
        });
        
        styler.add_legend(&mut diagram_data, &semantic);
        
        assert!(diagram_data.get("legend").is_some());
        assert!(diagram_data["legend"]["items"].is_array());
        assert!(diagram_data["legend"]["items"].as_array().unwrap().len() > 0);
    }
    
    #[test]
    fn test_apply_all_styles() {
        let styler = ProfessionalStyler::default();
        let semantic = create_test_semantic();
        
        let mut diagram_data = json!({
            "nodes": [
                {"id": "comp1", "type": "component"}
            ]
        });
        
        styler.apply_styles(&mut diagram_data, &semantic);
        
        // Should have applied color
        assert!(diagram_data["nodes"][0].get("fill").is_some());
        
        // Should have applied shadow
        assert!(diagram_data["nodes"][0].get("shadow").is_some());
        
        // Should have legend
        assert!(diagram_data.get("legend").is_some());
        
        // Should have typography
        assert!(diagram_data.get("typography").is_some());
    }
    
    #[test]
    fn test_color_blind_safe_theme() {
        let config = StyleConfig {
            theme: Theme::Capella,
            color_blind_safe: true,
            ..Default::default()
        };
        
        let styler = ProfessionalStyler::new(config);
        
        // Should use color-blind safe scheme
        assert_eq!(styler.color_scheme.sensor, "#0173B2");
    }
}
