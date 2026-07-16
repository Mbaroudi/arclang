use super::semantic::*;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Complete ELK generator supporting all algorithms and Capella-style diagrams
#[derive(Debug, Clone)]
pub struct ElkCompleteGenerator {
    pub config: ElkConfig,
}

#[derive(Debug, Clone)]
pub struct ElkConfig {
    /// Root algorithm: layered, stress, mrtree, radial, force, disco, box, fixed
    pub algorithm: String,
    /// Layout direction: RIGHT, DOWN, LEFT, UP
    pub direction: String,
    /// Hierarchy handling: INCLUDE_CHILDREN, SEPARATE_CHILDREN
    pub hierarchy_handling: String,
    /// Edge routing: ORTHOGONAL, POLYLINE, SPLINES, UNDEFINED
    pub edge_routing: String,
    /// Port constraints: FIXED_SIDE, FIXED_ORDER, FIXED_RATIO, FREE
    pub port_constraints: String,
    /// Enable interactive layout
    pub interactive: bool,
    /// Spacing configuration
    pub spacing: SpacingConfig,
    /// Algorithm-specific options
    pub algorithm_options: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct SpacingConfig {
    pub node_node: f64,
    pub edge_node: f64,
    pub edge_edge: f64,
    pub node_node_between_layers: f64,
    pub edge_node_between_layers: f64,
    pub edge_edge_between_layers: f64,
    pub component_spacing: f64,
}

impl Default for SpacingConfig {
    fn default() -> Self {
        Self {
            node_node: 80.0,
            edge_node: 40.0,
            edge_edge: 20.0,
            node_node_between_layers: 100.0,
            edge_node_between_layers: 40.0,
            edge_edge_between_layers: 20.0,
            component_spacing: 50.0,
        }
    }
}

impl Default for ElkConfig {
    fn default() -> Self {
        Self {
            algorithm: "layered".to_string(),
            direction: "RIGHT".to_string(),
            hierarchy_handling: "INCLUDE_CHILDREN".to_string(),
            edge_routing: "ORTHOGONAL".to_string(),
            port_constraints: "FIXED_SIDE".to_string(),
            interactive: false,
            spacing: SpacingConfig::default(),
            algorithm_options: HashMap::new(),
        }
    }
}

impl ElkCompleteGenerator {
    pub fn new() -> Self {
        Self {
            config: ElkConfig::default(),
        }
    }
    
    pub fn with_algorithm(mut self, algorithm: &str) -> Self {
        self.config.algorithm = algorithm.to_string();
        
        // Set algorithm-specific defaults
        match algorithm {
            "stress" => {
                self.config.algorithm_options.insert(
                    "elk.stress.desiredEdgeLength".to_string(),
                    json!("100")
                );
                self.config.algorithm_options.insert(
                    "elk.stress.epsilon".to_string(),
                    json!("0.0001")
                );
            }
            "force" => {
                self.config.algorithm_options.insert(
                    "elk.force.repulsion".to_string(),
                    json!("5.0")
                );
                self.config.algorithm_options.insert(
                    "elk.force.temperature".to_string(),
                    json!("0.001")
                );
            }
            "mrtree" => {
                self.config.algorithm_options.insert(
                    "elk.mrtree.searchDepth".to_string(),
                    json!("3")
                );
            }
            "radial" => {
                self.config.algorithm_options.insert(
                    "elk.radial.radius".to_string(),
                    json!("100")
                );
            }
            _ => {}
        }
        
        self
    }
    
    pub fn with_direction(mut self, direction: &str) -> Self {
        self.config.direction = direction.to_string();
        self
    }
    
    pub fn with_spacing(mut self, spacing: SpacingConfig) -> Self {
        self.config.spacing = spacing;
        self
    }
    
    pub fn generate(&self, model: &SemanticModel) -> Value {
        let mut root_children = Vec::new();
        
        // Group components by level
        let components_by_level = self.group_by_level(model);
        
        // Create level containers with appropriate algorithms
        for (level, components) in components_by_level {
            let level_algorithm = self.get_level_algorithm(&level);
            let level_node = self.create_level_container(&level, &components, &level_algorithm);
            root_children.push(level_node);
        }
        
        // Generate edges
        let edges = self.generate_edges(model);
        
        // Build root layout options
        let mut root_layout_options = json!({
            "elk.algorithm": format!("elk.{}", self.config.algorithm),
            "elk.direction": self.config.direction,
            "elk.hierarchyHandling": self.config.hierarchy_handling,
            "elk.portConstraints": self.config.port_constraints,
            "elk.edgeRouting": self.config.edge_routing,
            "elk.spacing.nodeNode": self.config.spacing.node_node.to_string(),
            "elk.spacing.edgeNode": self.config.spacing.edge_node.to_string(),
            "elk.spacing.edgeEdge": self.config.spacing.edge_edge.to_string(),
            "elk.separateConnectedComponents": "false",
        });
        
        // Add layered-specific options if using layered algorithm
        if self.config.algorithm == "layered" {
            root_layout_options["elk.layered.spacing.nodeNodeBetweenLayers"] = 
                json!(self.config.spacing.node_node_between_layers.to_string());
            root_layout_options["elk.layered.spacing.edgeNodeBetweenLayers"] = 
                json!(self.config.spacing.edge_node_between_layers.to_string());
            root_layout_options["elk.layered.spacing.edgeEdgeBetweenLayers"] = 
                json!(self.config.spacing.edge_edge_between_layers.to_string());
            root_layout_options["elk.layered.nodePlacement.strategy"] = json!("NETWORK_SIMPLEX");
            root_layout_options["elk.layered.crossingMinimization.strategy"] = json!("LAYER_SWEEP");
            root_layout_options["elk.layered.cycleBreaking.strategy"] = json!("GREEDY");
            root_layout_options["elk.layered.layering.strategy"] = json!("NETWORK_SIMPLEX");
        }
        
        // Add algorithm-specific options
        for (key, value) in &self.config.algorithm_options {
            root_layout_options[key] = value.clone();
        }
        
        if self.config.interactive {
            root_layout_options["interactiveLayout"] = json!(true);
        }
        
        json!({
            "id": "root",
            "layoutOptions": root_layout_options,
            "children": root_children,
            "edges": edges,
        })
    }
    
    fn create_level_container(
        &self,
        level: &str,
        components: &[&ComponentInfo],
        algorithm: &str,
    ) -> Value {
        let mut component_nodes = Vec::new();
        
        for comp in components {
            let node = self.create_component_node(comp);
            component_nodes.push(node);
        }
        
        let mut layout_options = json!({
            "elk.padding": "[top=60,left=40,bottom=40,right=40]",
            "elk.algorithm": format!("elk.{}", algorithm),
            "elk.hierarchyHandling": "INCLUDE_CHILDREN",
            "elk.spacing.nodeNode": self.config.spacing.node_node.to_string(),
            "elk.spacing.edgeNode": self.config.spacing.edge_node.to_string(),
            "elk.spacing.edgeEdge": self.config.spacing.edge_edge.to_string(),
        });
        
        // Add algorithm-specific options for the level
        if algorithm == "layered" {
            layout_options["elk.direction"] = json!(self.config.direction);
            layout_options["elk.layered.spacing.nodeNodeBetweenLayers"] = 
                json!(self.config.spacing.node_node_between_layers.to_string());
        } else if algorithm == "stress" {
            layout_options["elk.stress.desiredEdgeLength"] = json!("100");
        }
        
        json!({
            "id": format!("level_{}", level),
            "layoutOptions": layout_options,
            "labels": [{
                "text": format!("{} Level", level),
                "layoutOptions": {
                    "elk.label.anchor": "TOP_CENTER"
                }
            }],
            "children": component_nodes,
        })
    }
    
    fn create_component_node(&self, comp: &ComponentInfo) -> Value {
        let mut ports = Vec::new();
        
        // Create input ports
        for (idx, interface) in comp.interfaces_in.iter().enumerate() {
            let side = self.determine_port_side(&interface.name, "in");
            ports.push(json!({
                "id": format!("{}_port_in_{}", comp.id, idx),
                "properties": {
                    "port.side": side,
                    "port.index": idx,
                },
                "width": 10,
                "height": 10,
                "labels": [{
                    "text": interface.name,
                }]
            }));
        }
        
        // Create output ports
        for (idx, interface) in comp.interfaces_out.iter().enumerate() {
            let side = self.determine_port_side(&interface.name, "out");
            ports.push(json!({
                "id": format!("{}_port_out_{}", comp.id, idx),
                "properties": {
                    "port.side": side,
                    "port.index": idx,
                },
                "width": 10,
                "height": 10,
                "labels": [{
                    "text": interface.name,
                }]
            }));
        }
        
        let min_width = 220.0;
        let min_height = 160.0;
        
        let label_width = comp.name.len() as f64 * 8.0 + 40.0;
        let width = label_width.max(min_width);
        
        let max_ports = comp.interfaces_in.len().max(comp.interfaces_out.len());
        let port_height = (max_ports as f64 * 30.0 + 60.0).max(min_height);
        
        json!({
            "id": comp.id,
            "width": width,
            "height": port_height,
            "labels": [{"text": comp.name}],
            "ports": ports,
            "layoutOptions": {
                "elk.portAlignment.default": "CENTER",
                "elk.portConstraints": self.config.port_constraints,
            }
        })
    }
    
    fn determine_port_side(&self, interface_name: &str, direction: &str) -> String {
        let name_lower = interface_name.to_lowercase();
        
        if name_lower.contains("top") || name_lower.contains("north") {
            return "NORTH".to_string();
        }
        if name_lower.contains("bottom") || name_lower.contains("south") {
            return "SOUTH".to_string();
        }
        
        match direction {
            "in" => "WEST".to_string(),
            "out" => "EAST".to_string(),
            _ => "WEST".to_string(),
        }
    }
    
    fn generate_edges(&self, model: &SemanticModel) -> Vec<Value> {
        let mut edges = Vec::new();
        
        // Add edges from interfaces
        for interface in &model.interfaces {
            edges.push(json!({
                "id": format!("edge_{}_{}", interface.from, interface.to),
                "sources": [interface.from],
                "targets": [interface.to],
                "labels": [{
                    "text": interface.name
                }],
            }));
        }
        
        // Add edges from traces (component-to-component only)
        for trace in &model.traces {
            let from_is_component = model.components.iter().any(|c| c.id == trace.from);
            let to_is_component = model.components.iter().any(|c| c.id == trace.to);
            
            if from_is_component && to_is_component {
                edges.push(json!({
                    "id": format!("edge_trace_{}_{}", trace.from, trace.to),
                    "sources": [&trace.from],
                    "targets": [&trace.to],
                    "labels": [{
                        "text": trace.trace_type.clone()
                    }],
                }));
            }
        }
        
        edges
    }
    
    fn group_by_level<'a>(&self, model: &'a SemanticModel) -> HashMap<String, Vec<&'a ComponentInfo>> {
        let mut levels: HashMap<String, Vec<&'a ComponentInfo>> = HashMap::new();
        
        for comp in &model.components {
            levels
                .entry(comp.level.clone())
                .or_insert_with(Vec::new)
                .push(comp);
        }
        
        levels
    }
    
    fn get_level_algorithm(&self, level: &str) -> String {
        match level {
            "OA" => "layered".to_string(),  // Operational Analysis: hierarchical
            "SA" => "layered".to_string(),  // System Analysis: hierarchical
            "LA" => "layered".to_string(),  // Logical Architecture: hierarchical
            "PA" => "layered".to_string(),  // Physical Architecture: hierarchical
            _ => self.config.algorithm.clone(),
        }
    }
}

/// Builder for creating specific algorithm configurations
pub struct ElkAlgorithmBuilder;

impl ElkAlgorithmBuilder {
    /// Create layered (Sugiyama) configuration - best for hierarchical MBSE
    pub fn layered() -> ElkConfig {
        let mut config = ElkConfig::default();
        config.algorithm = "layered".to_string();
        config.spacing = SpacingConfig {
            node_node: 100.0,
            edge_node: 50.0,
            edge_edge: 30.0,
            node_node_between_layers: 150.0,
            edge_node_between_layers: 60.0,
            edge_edge_between_layers: 30.0,
            component_spacing: 60.0,
        };
        config
    }
    
    /// Create stress-based configuration - good for symmetric layouts
    pub fn stress() -> ElkConfig {
        let mut config = ElkConfig::default();
        config.algorithm = "stress".to_string();
        config.algorithm_options.insert(
            "elk.stress.desiredEdgeLength".to_string(),
            json!("100")
        );
        config
    }
    
    /// Create force-directed configuration - organic layouts
    pub fn force() -> ElkConfig {
        let mut config = ElkConfig::default();
        config.algorithm = "force".to_string();
        config.algorithm_options.insert(
            "elk.force.repulsion".to_string(),
            json!("5.0")
        );
        config
    }
    
    /// Create radial configuration - circular layouts
    pub fn radial() -> ElkConfig {
        let mut config = ElkConfig::default();
        config.algorithm = "radial".to_string();
        config
    }
    
    /// Create multi-root tree configuration
    pub fn mrtree() -> ElkConfig {
        let mut config = ElkConfig::default();
        config.algorithm = "mrtree".to_string();
        config
    }
    
    /// Create disco configuration - for disconnected graphs
    pub fn disco() -> ElkConfig {
        let mut config = ElkConfig::default();
        config.algorithm = "disco".to_string();
        config
    }
}
