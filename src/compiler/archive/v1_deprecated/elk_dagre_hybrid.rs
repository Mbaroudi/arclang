use super::semantic::*;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Hybrid layout generator that uses Dagre for node placement and ELK for edge routing
pub struct ElkDagreHybridGenerator {
    pub use_dagre_positions: bool,
    pub use_elk_routing: bool,
}

impl Default for ElkDagreHybridGenerator {
    fn default() -> Self {
        Self {
            use_dagre_positions: true,
            use_elk_routing: true,
        }
    }
}

impl ElkDagreHybridGenerator {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn generate(&self, model: &SemanticModel) -> Value {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        
        // Generate nodes with proper sizing
        for comp in &model.components {
            let width = 220.0;
            let height = 160.0;
            
            nodes.push(json!({
                "id": comp.id,
                "label": comp.name,
                "width": width,
                "height": height,
                "layoutOptions": {
                    "elk.portConstraints": "FIXED_SIDE",
                    "elk.padding": "[top=10,left=10,bottom=10,right=10]",
                }
            }));
        }
        
        // Generate edges from interfaces
        for interface in &model.interfaces {
            edges.push(json!({
                "id": format!("edge_{}_{}", interface.from, interface.to),
                "source": interface.from,
                "target": interface.to,
                "label": interface.name,
            }));
        }
        
        // Generate edges from traces (component-to-component only)
        for trace in &model.traces {
            let from_is_component = model.components.iter().any(|c| c.id == trace.from);
            let to_is_component = model.components.iter().any(|c| c.id == trace.to);
            
            if from_is_component && to_is_component {
                edges.push(json!({
                    "id": format!("edge_trace_{}_{}", trace.from, trace.to),
                    "source": trace.from,
                    "target": trace.to,
                    "label": trace.trace_type,
                }));
            }
        }
        
        json!({
            "nodes": nodes,
            "edges": edges,
            "dagreConfig": {
                "rankdir": "LR",
                "nodesep": 250,
                "edgesep": 120,
                "ranksep": 350,
                "marginx": 100,
                "marginy": 100
            },
            "elkConfig": {
                "algorithm": "layered",
                "direction": "RIGHT",
                "edgeRouting": "ORTHOGONAL",
                "spacing": {
                    "nodeNode": 250,
                    "edgeNode": 120,
                    "edgeEdge": 80,
                    "nodeNodeBetweenLayers": 350
                },
                "layered": {
                    "nodePlacement": {
                        "strategy": "NETWORK_SIMPLEX",
                        "favorStraightEdges": true
                    },
                    "crossingMinimization": {
                        "strategy": "LAYER_SWEEP",
                        "greedySwitch": {
                            "type": "TWO_SIDED"
                        }
                    },
                    "edgeRouting": {
                        "sloppySplineRouting": false,
                        "preferStraightEdges": true
                    }
                },
                "portConstraints": "FIXED_SIDE",
                "separateConnectedComponents": false
            }
        })
    }
}
