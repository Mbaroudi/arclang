//! Layout Strategy System for Context-Aware Diagram Generation
//! 
//! This module provides different layout strategies optimized for specific
//! Arcadia phases and diagram types.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::compiler::semantic_analyzer::{
    ArcadiaPhase, ElementClassification, ElementType, SemanticContext,
};

/// Layout strategy trait - all strategies must implement this
pub trait LayoutStrategy {
    /// Get the name of this strategy
    fn name(&self) -> &str;
    
    /// Configure ELK/Dagre with strategy-specific options
    fn configure(&self, semantic: &SemanticContext) -> LayoutConfig;
    
    /// Pre-process elements before layout (add strategy-specific properties)
    fn preprocess(&self, elements: Vec<ElementData>) -> Vec<ElementData>;
    
    /// Post-process SVG after layout (add visual enhancements)
    fn postprocess(&self, svg_data: Value) -> Value;
}

/// Layout configuration for ELK
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub algorithm: String,
    pub direction: String,
    pub options: HashMap<String, Value>,
}

/// Element data structure for layout processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementData {
    pub id: String,
    pub name: String,
    pub element_type: String,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub properties: HashMap<String, Value>,
    pub children: Vec<ElementData>,
}

impl ElementData {
    pub fn from_classification(elem: &ElementClassification) -> Self {
        ElementData {
            id: elem.id.clone(),
            name: elem.name.clone(),
            element_type: format!("{:?}", elem.element_type),
            x: None,
            y: None,
            width: None,
            height: None,
            properties: HashMap::new(),
            children: Vec::new(),
        }
    }
}

/// Swimlane Strategy - For Operational Diagrams with Actors
/// 
/// Layout Pattern:
/// ```text
/// ┌──────────┬──────────────┬──────────┐
/// │  Actor1  │   System     │  Actor2  │
/// │          │              │          │
/// │          │  Activity1   │          │
/// │          │      ↓       │          │
/// │          │  Activity2   │          │
/// └──────────┴──────────────┴──────────┘
/// ```
pub struct SwimlaneStrategy;

impl SwimlaneStrategy {
    pub fn new() -> Self {
        SwimlaneStrategy
    }
    
    /// Assign partition numbers to elements based on their type
    fn assign_partitions(&self, elements: &[ElementData]) -> HashMap<String, i32> {
        let mut partitions = HashMap::new();
        let mut next_partition = 0;
        
        // Actors get their own partitions
        for elem in elements {
            if elem.element_type == "Actor" {
                partitions.insert(elem.id.clone(), next_partition);
                next_partition += 2; // Leave gap for system partition
            }
        }
        
        // System elements go in partition 1 (middle)
        for elem in elements {
            if elem.element_type != "Actor" {
                partitions.insert(elem.id.clone(), 1);
            }
        }
        
        partitions
    }
}

impl LayoutStrategy for SwimlaneStrategy {
    fn name(&self) -> &str {
        "Swimlane"
    }
    
    fn configure(&self, _semantic: &SemanticContext) -> LayoutConfig {
        let mut options = HashMap::new();
        
        // Enable partitioning for swimlanes
        options.insert("elk.partitioning.activate".to_string(), json!(true));
        
        // Vertical layout (top to bottom)
        options.insert("elk.direction".to_string(), json!("DOWN"));
        
        // Spacing for swimlanes
        options.insert("elk.spacing.componentComponent".to_string(), json!(150));
        options.insert("elk.layered.spacing.nodeNodeBetweenLayers".to_string(), json!(80));
        options.insert("elk.spacing.nodeNode".to_string(), json!(60));
        
        // Port constraints
        options.insert("elk.portConstraints".to_string(), json!("FIXED_SIDE"));
        
        // Edge routing
        options.insert("elk.edgeRouting".to_string(), json!("ORTHOGONAL"));
        
        // Crossing minimization
        options.insert("elk.layered.crossingMinimization.strategy".to_string(), json!("LAYER_SWEEP"));
        
        LayoutConfig {
            algorithm: "layered".to_string(),
            direction: "DOWN".to_string(),
            options,
        }
    }
    
    fn preprocess(&self, mut elements: Vec<ElementData>) -> Vec<ElementData> {
        let partitions = self.assign_partitions(&elements);
        
        // Assign partition to each element
        for elem in &mut elements {
            if let Some(&partition) = partitions.get(&elem.id) {
                elem.properties.insert(
                    "elk.partitioning.partition".to_string(),
                    json!(partition),
                );
            }
            
            // Set port side for actors (all on inner side)
            if elem.element_type == "Actor" {
                elem.properties.insert(
                    "elk.port.side".to_string(),
                    json!("EAST"), // Always point inward to system
                );
            }
        }
        
        elements
    }
    
    fn postprocess(&self, mut svg_data: Value) -> Value {
        // Add visual swimlane separators
        if let Some(obj) = svg_data.as_object_mut() {
            obj.insert("swimlanes_enabled".to_string(), json!(true));
        }
        svg_data
    }
}

/// Hierarchy Strategy - For Component Containment
/// 
/// Layout Pattern:
/// ```text
/// ┌─────────────────────────┐
/// │  Component A            │
/// │  ┌───────────────────┐  │
/// │  │  Sub-Component 1  │  │
/// │  └───────────────────┘  │
/// │  ┌───────────────────┐  │
/// │  │  Sub-Component 2  │  │
/// │  └───────────────────┘  │
/// └─────────────────────────┘
/// ```
pub struct HierarchyStrategy;

impl HierarchyStrategy {
    pub fn new() -> Self {
        HierarchyStrategy
    }
}

impl LayoutStrategy for HierarchyStrategy {
    fn name(&self) -> &str {
        "Hierarchy"
    }
    
    fn configure(&self, semantic: &SemanticContext) -> LayoutConfig {
        let mut options = HashMap::new();
        
        // Handle nested components
        options.insert("elk.hierarchyHandling".to_string(), json!("INCLUDE_CHILDREN"));
        
        // Direction based on phase
        let direction = if semantic.phase == ArcadiaPhase::Physical {
            "DOWN" // Physical diagrams vertical
        } else {
            "RIGHT" // Logical diagrams horizontal
        };
        options.insert("elk.direction".to_string(), json!(direction));
        
        // Padding for parent containers
        options.insert("elk.padding".to_string(), json!("[top=40,left=30,bottom=30,right=30]"));
        
        // Spacing
        options.insert("elk.spacing.nodeNode".to_string(), json!(60));
        options.insert("elk.spacing.componentComponent".to_string(), json!(100));
        options.insert("elk.layered.spacing.nodeNodeBetweenLayers".to_string(), json!(80));
        
        // Port positioning
        options.insert("elk.portConstraints".to_string(), json!("FIXED_SIDE"));
        
        // Edge routing
        options.insert("elk.edgeRouting".to_string(), json!("ORTHOGONAL"));
        
        // Node placement
        options.insert("elk.layered.nodePlacement.strategy".to_string(), json!("BRANDES_KOEPF"));
        
        LayoutConfig {
            algorithm: "layered".to_string(),
            direction: direction.to_string(),
            options,
        }
    }
    
    fn preprocess(&self, mut elements: Vec<ElementData>) -> Vec<ElementData> {
        // Set port sides: IN=left, OUT=right
        for elem in &mut elements {
            if elem.element_type == "Component" || elem.element_type == "Function" {
                // Components with interfaces
                elem.properties.insert(
                    "elk.portConstraints".to_string(),
                    json!("FIXED_SIDE"),
                );
            }
            
            // For physical nodes, ensure proper padding for children
            if elem.element_type == "PhysicalNode" {
                elem.properties.insert(
                    "elk.padding".to_string(),
                    json!("[top=50,left=40,bottom=40,right=40]"),
                );
            }
        }
        
        elements
    }
    
    fn postprocess(&self, svg_data: Value) -> Value {
        // Hierarchy strategy doesn't need post-processing
        svg_data
    }
}

/// Port-Centric Strategy - For Data Flow Diagrams
/// 
/// Layout Pattern:
/// ```text
/// ┌─────┐        ┌─────┐        ┌─────┐
/// │  F1 │───────→│  F2 │───────→│  F3 │
/// └─────┘        └─────┘        └─────┘
///   │                              ↑
///   └──────────────────────────────┘
/// ```
pub struct PortCentricStrategy;

impl PortCentricStrategy {
    pub fn new() -> Self {
        PortCentricStrategy
    }
}

impl LayoutStrategy for PortCentricStrategy {
    fn name(&self) -> &str {
        "PortCentric"
    }
    
    fn configure(&self, _semantic: &SemanticContext) -> LayoutConfig {
        let mut options = HashMap::new();
        
        // Left-to-right flow
        options.insert("elk.direction".to_string(), json!("RIGHT"));
        
        // Port-to-port routing
        options.insert("elk.port.side".to_string(), json!("NORTH_SOUTH_EAST_WEST"));
        options.insert("elk.portConstraints".to_string(), json!("FIXED_SIDE"));
        
        // Edge optimization
        options.insert("elk.layered.crossingMinimization.strategy".to_string(), json!("LAYER_SWEEP"));
        options.insert("elk.edgeRouting".to_string(), json!("ORTHOGONAL"));
        
        // Spacing optimized for data flow
        options.insert("elk.layered.spacing.nodeNodeBetweenLayers".to_string(), json!(120));
        options.insert("elk.layered.spacing.edgeNodeBetweenLayers".to_string(), json!(40));
        options.insert("elk.spacing.nodeNode".to_string(), json!(50));
        
        // Node placement for clean flow
        options.insert("elk.layered.nodePlacement.strategy".to_string(), json!("BRANDES_KOEPF"));
        
        // Compaction for dense layout
        options.insert("elk.layered.compaction.postCompaction.strategy".to_string(), json!("EDGE_LENGTH"));
        
        LayoutConfig {
            algorithm: "layered".to_string(),
            direction: "RIGHT".to_string(),
            options,
        }
    }
    
    fn preprocess(&self, mut elements: Vec<ElementData>) -> Vec<ElementData> {
        // Assign edge priorities based on data criticality
        for elem in &mut elements {
            if elem.element_type == "Function" {
                // Input ports on left (WEST)
                elem.properties.insert(
                    "elk.port.side.input".to_string(),
                    json!("WEST"),
                );
                
                // Output ports on right (EAST)
                elem.properties.insert(
                    "elk.port.side.output".to_string(),
                    json!("EAST"),
                );
                
                // Consider model order for better flow
                elem.properties.insert(
                    "elk.layered.considerModelOrder.strategy".to_string(),
                    json!("PREFER_EDGES"),
                );
            }
        }
        
        elements
    }
    
    fn postprocess(&self, svg_data: Value) -> Value {
        // Port-centric strategy doesn't need post-processing
        svg_data
    }
}

/// Strategy Selector - Chooses the best strategy for a given semantic context
pub struct StrategySelector;

impl StrategySelector {
    pub fn new() -> Self {
        StrategySelector
    }
    
    /// Select the most appropriate layout strategy
    pub fn select(&self, semantic: &SemanticContext) -> Box<dyn LayoutStrategy> {
        // Operational phase with actors → Swimlane
        if semantic.phase == ArcadiaPhase::Operational && semantic.has_actors {
            return Box::new(SwimlaneStrategy::new());
        }
        
        // System phase with data flow → Port-Centric
        if semantic.phase == ArcadiaPhase::System && semantic.has_data_flow {
            return Box::new(PortCentricStrategy::new());
        }
        
        // Logical/Physical with hierarchy → Hierarchy
        if (semantic.phase == ArcadiaPhase::Logical || semantic.phase == ArcadiaPhase::Physical)
            && semantic.has_hierarchy
        {
            return Box::new(HierarchyStrategy::new());
        }
        
        // Physical phase → always Hierarchy (for ECU nesting)
        if semantic.phase == ArcadiaPhase::Physical {
            return Box::new(HierarchyStrategy::new());
        }
        
        // Default: Hierarchy
        Box::new(HierarchyStrategy::new())
    }
    
    /// Get strategy by name (for manual override)
    pub fn get_strategy(&self, name: &str) -> Option<Box<dyn LayoutStrategy>> {
        match name.to_lowercase().as_str() {
            "swimlane" => Some(Box::new(SwimlaneStrategy::new())),
            "hierarchy" => Some(Box::new(HierarchyStrategy::new())),
            "portcentric" | "port-centric" => Some(Box::new(PortCentricStrategy::new())),
            _ => None,
        }
    }
}

impl Default for StrategySelector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::semantic_analyzer::{
        ArcadiaPhase, ComplexityMetrics, RelationshipAnalysis, RecommendedStrategy,
    };
    
    fn create_test_semantic_context(
        phase: ArcadiaPhase,
        has_actors: bool,
        has_hierarchy: bool,
        has_data_flow: bool,
    ) -> SemanticContext {
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
                total_elements: 0,
                depth: 0,
                branching_factor: 0.0,
                has_cycles: false,
            },
            recommended_strategy: RecommendedStrategy::Hierarchy,
            has_actors,
            has_hierarchy,
            has_data_flow,
            has_safety_critical: false,
        }
    }
    
    #[test]
    fn test_strategy_selector_swimlane() {
        let selector = StrategySelector::new();
        let semantic = create_test_semantic_context(
            ArcadiaPhase::Operational,
            true,  // has actors
            false,
            false,
        );
        
        let strategy = selector.select(&semantic);
        assert_eq!(strategy.name(), "Swimlane");
    }
    
    #[test]
    fn test_strategy_selector_hierarchy() {
        let selector = StrategySelector::new();
        let semantic = create_test_semantic_context(
            ArcadiaPhase::Logical,
            false,
            true,  // has hierarchy
            false,
        );
        
        let strategy = selector.select(&semantic);
        assert_eq!(strategy.name(), "Hierarchy");
    }
    
    #[test]
    fn test_strategy_selector_port_centric() {
        let selector = StrategySelector::new();
        let semantic = create_test_semantic_context(
            ArcadiaPhase::System,
            false,
            false,
            true,  // has data flow
        );
        
        let strategy = selector.select(&semantic);
        assert_eq!(strategy.name(), "PortCentric");
    }
    
    #[test]
    fn test_swimlane_config() {
        let strategy = SwimlaneStrategy::new();
        let semantic = create_test_semantic_context(
            ArcadiaPhase::Operational,
            true,
            false,
            false,
        );
        
        let config = strategy.configure(&semantic);
        assert_eq!(config.algorithm, "layered");
        assert_eq!(config.direction, "DOWN");
        assert!(config.options.contains_key("elk.partitioning.activate"));
    }
    
    #[test]
    fn test_hierarchy_config() {
        let strategy = HierarchyStrategy::new();
        let semantic = create_test_semantic_context(
            ArcadiaPhase::Logical,
            false,
            true,
            false,
        );
        
        let config = strategy.configure(&semantic);
        assert_eq!(config.algorithm, "layered");
        assert_eq!(config.direction, "RIGHT");
        assert!(config.options.contains_key("elk.hierarchyHandling"));
    }
    
    #[test]
    fn test_port_centric_config() {
        let strategy = PortCentricStrategy::new();
        let semantic = create_test_semantic_context(
            ArcadiaPhase::System,
            false,
            false,
            true,
        );
        
        let config = strategy.configure(&semantic);
        assert_eq!(config.algorithm, "layered");
        assert_eq!(config.direction, "RIGHT");
        assert!(config.options.contains_key("elk.layered.crossingMinimization.strategy"));
    }
    
    #[test]
    fn test_get_strategy_by_name() {
        let selector = StrategySelector::new();
        
        assert!(selector.get_strategy("swimlane").is_some());
        assert!(selector.get_strategy("hierarchy").is_some());
        assert!(selector.get_strategy("portcentric").is_some());
        assert!(selector.get_strategy("invalid").is_none());
    }
}
