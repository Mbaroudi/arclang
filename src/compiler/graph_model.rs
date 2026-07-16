//! Graph model - Shared graph data structures for diagram generators
//!
//! Provides the layered graph representation (nodes, edges, layers) built
//! from a `SemanticModel`. Used by the v2 generators (`arcviz_elk_static`,
//! `arcviz_explorer`) as the common intermediate structure before layout.

use super::semantic::SemanticModel;
use super::CompilerError;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Graph node for layout engines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub layer: String,
    pub stereotype: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asil: Option<String>,
    pub interfaces_in: Vec<InterfacePort>,
    pub interfaces_out: Vec<InterfacePort>,
    pub functions: Vec<String>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfacePort {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

/// Graph edge for layout engines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub label: String,
}

/// Complete graph structure
#[derive(Debug, Serialize, Deserialize)]
pub struct DagreGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub layers: Vec<LayerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerInfo {
    pub name: String,
    pub color: String,
    pub description: String,
}

impl DagreGraph {
    /// Build graph from semantic model
    pub fn from_model(model: &SemanticModel) -> Result<Self, CompilerError> {
        // Dynamically extract layers from components
        let mut layer_set = std::collections::HashSet::new();
        for comp in &model.components {
            if !comp.level.is_empty() {
                layer_set.insert(comp.level.clone());
            }
        }

        // Create layer configs with colors
        let layer_colors: HashMap<&str, (&str, &str)> = [
            ("User", ("#E1F5FE", "User Interface Layer")),
            ("Connectivity", ("#FFF3E0", "Connectivity & Communication Layer")),
            ("Control", ("#F3E5F5", "Control & Logic Layer")),
            ("Vehicle", ("#E8F5E9", "Vehicle Systems Layer")),
            ("Physical", ("#FCE4EC", "Physical Hardware Layer")),
            ("Logical", ("#E3F2FD", "Logical Architecture Layer")),
            ("Application", ("#FFF9C4", "Application Layer")),
            ("Service", ("#E0F2F1", "Service Layer")),
        ].iter().cloned().collect();

        let mut layers = Vec::new();
        for layer_name in layer_set {
            let (color, desc) = layer_colors.get(layer_name.as_str())
                .unwrap_or(&("#EFEBE9", "Architecture Layer"));
            layers.push(LayerInfo {
                name: layer_name.clone(),
                color: color.to_string(),
                description: desc.to_string(),
            });
        }

        // Sort layers for consistent ordering
        layers.sort_by(|a, b| a.name.cmp(&b.name));

        let mut nodes = Vec::new();

        // Extract nodes from model
        for comp in &model.components {
            let layer = comp.level.clone();

            let interfaces_in: Vec<InterfacePort> = comp.interfaces_in.iter()
                .map(|iface| InterfacePort {
                    name: iface.name.clone(),
                    protocol: iface.protocol.clone(),
                    format: iface.format.clone(),
                })
                .collect();

            let interfaces_out: Vec<InterfacePort> = comp.interfaces_out.iter()
                .map(|iface| InterfacePort {
                    name: iface.name.clone(),
                    protocol: iface.protocol.clone(),
                    format: iface.format.clone(),
                })
                .collect();

            let functions: Vec<String> = comp.functions.clone();

            // Calculate node dimensions based on content
            let width = calculate_node_width(&comp.name, &functions);
            let height = calculate_node_height(&interfaces_in, &interfaces_out, &functions);

            nodes.push(GraphNode {
                id: comp.id.clone(),
                label: comp.name.clone(),
                layer,
                stereotype: infer_stereotype(&comp.name),
                safety_level: comp.safety_level.clone(),
                asil: comp.asil.clone(),
                interfaces_in,
                interfaces_out,
                functions,
                width,
                height,
            });
        }

        // Extract edges, resolving endpoints to NODE ids: interfaces may
        // reference components by name or dotted port path (Comp.Port), and a
        // renderer crashes on edges pointing at nonexistent nodes.
        let node_ids: std::collections::HashSet<&str> =
            nodes.iter().map(|n| n.id.as_str()).collect();
        let name_to_id: HashMap<&str, &str> = nodes
            .iter()
            .map(|n| (n.label.as_str(), n.id.as_str()))
            .collect();
        let resolve = |endpoint: &str| -> Option<String> {
            if node_ids.contains(endpoint) {
                return Some(endpoint.to_string());
            }
            if let Some(id) = name_to_id.get(endpoint) {
                return Some((*id).to_string());
            }
            let root = endpoint.split('.').next().unwrap_or(endpoint);
            if node_ids.contains(root) {
                return Some(root.to_string());
            }
            name_to_id.get(root).map(|id| (*id).to_string())
        };

        let mut edges = Vec::new();
        for interface in &model.interfaces {
            // Edges between non-component elements (e.g. function-to-function
            // exchanges) don't belong on this component-level view.
            if let (Some(source), Some(target)) =
                (resolve(&interface.from), resolve(&interface.to))
            {
                edges.push(GraphEdge {
                    source,
                    target,
                    label: interface.name.clone(),
                });
            }
        }

        Ok(DagreGraph {
            nodes,
            edges,
            layers,
        })
    }
}

fn infer_stereotype(name: &str) -> String {
    if name.contains("Database") || name.contains("Storage") || name.contains("Warehouse") {
        "<<datastore>>".to_string()
    } else if name.contains("Orchestrator") || name.contains("Engine") || name.contains("ETL") {
        "<<process>>".to_string()
    } else if name.contains("API") || name.contains("Gateway") {
        "<<interface>>".to_string()
    } else if name.contains("Validator") || name.contains("Resolver") || name.contains("Monitor") {
        "<<service>>".to_string()
    } else if name.contains("Registry") || name.contains("Catalog") {
        "<<registry>>".to_string()
    } else {
        "<<component>>".to_string()
    }
}

fn calculate_node_width(name: &str, functions: &[String]) -> u32 {
    let name_width = name.len() as u32 * 7;
    let max_func_width = functions.iter()
        .map(|f| f.len() as u32 * 5)
        .max()
        .unwrap_or(0);

    let content_width = name_width.max(max_func_width);
    (content_width + 60).max(220).min(360)
}

fn calculate_node_height(interfaces_in: &[InterfacePort], interfaces_out: &[InterfacePort], functions: &[String]) -> u32 {
    let header_height = 50;
    let func_height = (functions.len().min(4) as u32) * 18 + 20;
    let max_ports = interfaces_in.len().max(interfaces_out.len()).min(3);
    let port_height = if max_ports > 0 {
        (max_ports as u32) * 30 + 20
    } else {
        10
    };

    (header_height + func_height + port_height).max(180).min(450)
}
