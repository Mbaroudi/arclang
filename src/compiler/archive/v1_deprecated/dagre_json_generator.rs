use super::semantic::*;
use serde_json::{json, Value};

pub struct DagreJsonGenerator;

impl DagreJsonGenerator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn generate(&self, model: &SemanticModel) -> Value {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        
        // Add all components as nodes
        for comp in &model.components {
            nodes.push(json!({
                "id": comp.id,
                "label": comp.name,
            }));
        }
        
        // Add edges from interfaces
        for interface in &model.interfaces {
            edges.push(json!({
                "from": interface.from,
                "to": interface.to,
                "label": interface.name,
            }));
        }
        
        // Add edges from traces (component-to-component only)
        for trace in &model.traces {
            let from_is_component = model.components.iter().any(|c| c.id == trace.from);
            let to_is_component = model.components.iter().any(|c| c.id == trace.to);
            
            if from_is_component && to_is_component {
                edges.push(json!({
                    "from": trace.from,
                    "to": trace.to,
                    "label": trace.trace_type,
                }));
            }
        }
        
        json!({
            "nodes": nodes,
            "edges": edges,
        })
    }
}
