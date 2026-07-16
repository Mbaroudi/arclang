use super::semantic::*;
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ELKJsonGenerator {
    pub config: ELKGlobalConfig,
}

#[derive(Debug, Clone)]
pub struct ELKGlobalConfig {
    pub default_direction: String,
    pub default_algorithm: String,
    pub port_constraints: String,
    pub edge_routing: String,
    pub hierarchy_handling: String,
    pub node_spacing: f64,
    pub layer_spacing: f64,
    pub port_spacing: f64,
    pub enable_interactive: bool,
}

impl Default for ELKGlobalConfig {
    fn default() -> Self {
        Self {
            default_direction: "RIGHT".to_string(),
            default_algorithm: "layered".to_string(),
            port_constraints: "FIXED_SIDE".to_string(),
            edge_routing: "ORTHOGONAL".to_string(),
            hierarchy_handling: "INCLUDE_CHILDREN".to_string(),
            node_spacing: 80.0,
            layer_spacing: 100.0,
            port_spacing: 20.0,
            enable_interactive: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayoutConfig {
    pub algorithm: String,
    pub direction: Option<String>,
    pub interactive: bool,
    pub aspect_ratio: Option<f64>,
    pub expand_nodes: Option<bool>,
    pub separate_connected_components: Option<bool>,
    pub port_constraints: Option<String>,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            algorithm: "layered".to_string(),
            direction: Some("RIGHT".to_string()),
            interactive: false,
            aspect_ratio: None,
            expand_nodes: None,
            separate_connected_components: None,
            port_constraints: Some("FIXED_SIDE".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayoutConstraints {
    pub layer_choice: Option<i32>,
    pub position_choice: Option<i32>,
}

impl ELKJsonGenerator {
    pub fn new() -> Self {
        Self {
            config: ELKGlobalConfig::default(),
        }
    }

    pub fn with_config(config: ELKGlobalConfig) -> Self {
        Self { config }
    }

    pub fn generate(&self, model: &SemanticModel) -> Value {
        let mut root_children = Vec::new();
        
        let components_by_level = self.group_by_level(model);
        
        for (level, components) in components_by_level {
            let level_config = self.get_level_layout_config(&level);
            let level_node = self.create_level_container(&level, &components, &level_config);
            root_children.push(level_node);
        }
        
        let edges = self.generate_edges(model);
        
        json!({
            "id": "root",
            "layoutOptions": {
                "elk.algorithm": self.config.default_algorithm,
                "elk.direction": self.config.default_direction,
                "elk.hierarchyHandling": self.config.hierarchy_handling,
                "elk.portConstraints": self.config.port_constraints,
                "elk.edgeRouting": self.config.edge_routing,
                "elk.spacing.nodeNode": "100",
                "elk.spacing.edgeNode": "50",
                "elk.spacing.edgeEdge": "30",
                "elk.layered.spacing.nodeNodeBetweenLayers": "150",
                "elk.layered.spacing.edgeNodeBetweenLayers": "60",
                "elk.layered.spacing.edgeEdgeBetweenLayers": "30",
                "elk.layered.nodePlacement.strategy": "NETWORK_SIMPLEX",
                "elk.layered.crossingMinimization.strategy": "LAYER_SWEEP",
                "elk.layered.considerModelOrder.strategy": "NODES_AND_EDGES",
                "elk.separateConnectedComponents": "false",
                "interactiveLayout": self.config.enable_interactive,
            },
            "children": root_children,
            "edges": edges,
        })
    }

    fn create_level_container(
        &self,
        level: &str,
        components: &[&ComponentInfo],
        config: &LayoutConfig,
    ) -> Value {
        let mut component_nodes = Vec::new();
        
        for comp in components {
            let node = self.create_component_node(comp, None, None);
            component_nodes.push(node);
        }
        
        let mut layout_options = json!({
            "elk.padding": "[top=60,left=40,bottom=40,right=40]",
            "elk.algorithm": config.algorithm,
            "elk.hierarchyHandling": "INCLUDE_CHILDREN",
            "elk.spacing.nodeNode": "100",
            "elk.spacing.edgeNode": "50",
            "elk.spacing.edgeEdge": "30",
            "elk.layered.spacing.nodeNodeBetweenLayers": "150",
        });
        
        if let Some(dir) = &config.direction {
            layout_options["elk.direction"] = json!(dir);
        }
        
        if config.interactive {
            layout_options["interactiveLayout"] = json!(true);
        }
        
        if let Some(sep) = config.separate_connected_components {
            layout_options["separateConnectedComponents"] = json!(sep);
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

    fn create_component_node(
        &self,
        comp: &ComponentInfo,
        layout_override: Option<&LayoutConfig>,
        constraints: Option<&LayoutConstraints>,
    ) -> Value {
        let mut ports = Vec::new();
        
        for (idx, interface) in comp.interfaces_in.iter().enumerate() {
            let side = self.determine_port_side(&interface.name, "in");
            ports.push(self.create_port_with_margins(
                &format!("{}_port_in_{}", comp.id, idx),
                &interface.name,
                &side,
                idx,
                Some(10.0),
            ));
        }
        
        for (idx, interface) in comp.interfaces_out.iter().enumerate() {
            let side = self.determine_port_side(&interface.name, "out");
            ports.push(self.create_port_with_margins(
                &format!("{}_port_out_{}", comp.id, idx),
                &interface.name,
                &side,
                idx,
                Some(10.0),
            ));
        }
        
        let mut layout_options = json!({
            "elk.portAlignment.default": "CENTER",
        });
        
        if let Some(config) = layout_override {
            layout_options["elk.algorithm"] = json!(config.algorithm);
            if let Some(dir) = &config.direction {
                layout_options["elk.direction"] = json!(dir);
            }
            if config.interactive {
                layout_options["interactiveLayout"] = json!(true);
            }
        }
        
        if let Some(c) = constraints {
            if let Some(layer) = c.layer_choice {
                layout_options["layering.layerChoiceConstraint"] = json!(layer);
            }
            if let Some(pos) = c.position_choice {
                layout_options["crossingMinimization.positionChoiceConstraint"] = json!(pos);
            }
        }
        
        let min_width = 200.0;
        let min_height = 150.0;
        
        let label_width = comp.name.len() as f64 * 8.0 + 40.0;
        let width = label_width.max(min_width);
        
        let max_ports = comp.interfaces_in.len().max(comp.interfaces_out.len());
        let port_height = (max_ports as f64 * 25.0 + 50.0).max(min_height);
        
        json!({
            "id": comp.id,
            "width": width,
            "height": port_height,
            "labels": [{"text": comp.name}],
            "ports": ports,
            "layoutOptions": layout_options,
        })
    }

    fn create_port_with_margins(
        &self,
        id: &str,
        label: &str,
        side: &str,
        index: usize,
        surrounding_margin: Option<f64>,
    ) -> Value {
        let mut layout_options = json!({});
        
        if let Some(margin) = surrounding_margin {
            layout_options["elk.port.borderOffset"] = json!(format!("{}", margin));
        }
        
        json!({
            "id": id,
            "properties": {
                "port.side": side,
                "port.index": index,
            },
            "width": 10,
            "height": 10,
            "labels": [{
                "text": label,
            }]
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
        
        // Add edges from traces (for component-to-component relationships)
        for trace in &model.traces {
            // Only create edges for traces between components in the same architecture
            if self.is_component_trace(&trace.from, &trace.to, model) {
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
    
    fn is_component_trace(&self, from: &str, to: &str, model: &SemanticModel) -> bool {
        let from_is_component = model.components.iter().any(|c| c.id == from);
        let to_is_component = model.components.iter().any(|c| c.id == to);
        from_is_component && to_is_component
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

    fn get_level_layout_config(&self, level: &str) -> LayoutConfig {
        match level {
            "OA" => LayoutConfig {
                algorithm: "layered".to_string(),
                direction: Some("DOWN".to_string()),
                interactive: false,
                aspect_ratio: None,
                expand_nodes: None,
                separate_connected_components: Some(false),
                port_constraints: Some("FIXED_SIDE".to_string()),
            },
            "SA" | "LA" => LayoutConfig {
                algorithm: "layered".to_string(),
                direction: Some("RIGHT".to_string()),
                interactive: false,
                aspect_ratio: None,
                expand_nodes: None,
                separate_connected_components: Some(false),
                port_constraints: Some("FIXED_SIDE".to_string()),
            },
            "PA" => LayoutConfig {
                algorithm: "layered".to_string(),
                direction: Some("DOWN".to_string()),
                interactive: false,
                aspect_ratio: None,
                expand_nodes: None,
                separate_connected_components: Some(false),
                port_constraints: Some("FIXED_SIDE".to_string()),
            },
            _ => LayoutConfig::default(),
        }
    }
}

pub fn create_port_with_north_south_support(
    id: &str,
    label: &str,
    side: &str,
    index: usize,
    port_surrounding_margin: Option<f64>,
) -> Value {
    let mut properties = json!({
        "port.side": side,
        "port.index": index,
        "port.borderOffset": -5.0,
    });
    
    if let Some(margin) = port_surrounding_margin {
        properties["portSurroundingMargin"] = json!(margin);
    }
    
    json!({
        "id": id,
        "properties": properties,
        "width": 10,
        "height": 10,
        "labels": [{
            "text": label,
            "layoutOptions": {
                "elk.label.side": side,
            }
        }]
    })
}

pub fn create_interactive_node_with_constraints(
    comp: &ComponentInfo,
    algorithm: &str,
    direction: Option<&str>,
    constraints: Option<LayoutConstraints>,
) -> Value {
    let mut layout_options = json!({
        "elk.algorithm": algorithm,
        "interactiveLayout": true,
    });
    
    if let Some(dir) = direction {
        layout_options["elk.direction"] = json!(dir);
    }
    
    if let Some(c) = constraints {
        if let Some(layer) = c.layer_choice {
            layout_options["layering.layerChoiceConstraint"] = json!(layer);
        }
        if let Some(pos) = c.position_choice {
            layout_options["crossingMinimization.positionChoiceConstraint"] = json!(pos);
        }
    }
    
    json!({
        "id": comp.id,
        "layoutOptions": layout_options,
        "labels": [{"text": comp.name}],
        "width": 220,
        "height": 180,
    })
}
