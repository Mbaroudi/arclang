//! ArcViz Explorer - Interactive Architecture Document
//! 
//! A comprehensive, explorable architecture view that reflects the complete
//! Capella-style architecture document with all details from the .arc file.
//! 
//! Features:
//! - Complete requirements traceability
//! - Expandable component hierarchies
//! - Interactive architecture diagram
//! - Smooth scrolling navigation
//! - Export as self-contained HTML
//! - No external dependencies needed

use super::semantic::SemanticModel;
use super::CompilerError;
use super::arcviz_d3::{DagreGraph, GraphNode, GraphEdge, LayerInfo};
use serde::{Serialize, Deserialize};

/// Complete architecture document structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ArchitectureDocument {
    pub metadata: DocumentMetadata,
    pub requirements: Vec<RequirementSection>,
    pub components: Vec<ComponentDetail>,
    pub interfaces: Vec<InterfaceDetail>,
    pub functions: Vec<FunctionDetail>,
    pub traces: Vec<TraceDetail>,
    pub diagram: DiagramData,
}

/// Diagram data for D3.js visualization
#[derive(Debug, Serialize, Deserialize)]
pub struct DiagramData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub layers: Vec<LayerInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: String,
    pub version: String,
    pub description: String,
    pub total_requirements: usize,
    pub total_components: usize,
    pub total_interfaces: usize,
    pub total_functions: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequirementSection {
    pub category: String,
    pub requirements: Vec<RequirementDetail>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequirementDetail {
    pub id: String,
    pub description: String,
    pub priority: String,
    pub category: Option<String>,
    pub safety_level: Option<String>,
    pub traces_to: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentDetail {
    pub id: String,
    pub name: String,
    pub layer: String,
    pub component_type: String,
    pub safety_level: Option<String>,
    pub asil: Option<String>,
    pub interfaces_in: Vec<InterfacePortDetail>,
    pub interfaces_out: Vec<InterfacePortDetail>,
    pub functions: Vec<String>,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InterfacePortDetail {
    pub name: String,
    pub protocol: Option<String>,
    pub format: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InterfaceDetail {
    pub name: String,
    pub from: String,
    pub to: String,
    pub from_component: String,
    pub to_component: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionDetail {
    pub id: String,
    pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub component: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraceDetail {
    pub from: String,
    pub to: String,
    pub trace_type: String,
    pub rationale: Option<String>,
}

impl ArchitectureDocument {
    pub fn from_model(model: &SemanticModel) -> Result<Self, CompilerError> {
        // Extract metadata from model or use defaults
        // Try to infer title from the architecture domain or system name
        let title = if !model.components.is_empty() {
            // Check component IDs and names for domain keywords
            let comp_ids: Vec<String> = model.components.iter().map(|c| c.id.clone()).collect();
            let comp_names: Vec<String> = model.components.iter().map(|c| c.name.clone()).collect();
            
            // Check for automotive/vehicle domain
            if comp_ids.iter().any(|id| id.contains("VHC") || id.contains("CTRL")) ||
               comp_names.iter().any(|n| n.contains("Engine") || n.contains("Battery") || 
                                      n.contains("Vehicle") || n.contains("ECU") ||
                                      n.contains("Telematics") || n.contains("Powertrain")) {
                "Vehicle Remote Start System Architecture".to_string()
            }
            // Check for data platform domain
            else if comp_ids.iter().any(|id| id.contains("MIG") || id.contains("ANLZ") || id.contains("TGT")) ||
                    comp_names.iter().any(|n| n.contains("Migration") || n.contains("Analytics") || 
                                           n.contains("Data") || n.contains("Databricks")) {
                "Data Platform Architecture".to_string()
            }
            // Generic fallback
            else {
                "System Architecture".to_string()
            }
        } else {
            "Architecture Document".to_string()
        };
        
        let description = format!("Complete architecture with {} components, {} interfaces, and {} requirements",
            model.components.len(),
            model.interfaces.len(),
            model.requirements.len()
        );
        
        let metadata = DocumentMetadata {
            title,
            version: "1.0.0".to_string(),
            description,
            total_requirements: model.requirements.len(),
            total_components: model.components.len(),
            total_interfaces: model.interfaces.len(),
            total_functions: model.functions.len(),
        };
        
        // Group requirements by category
        let mut req_by_category: std::collections::HashMap<String, Vec<RequirementDetail>> = 
            std::collections::HashMap::new();
        
        for req in &model.requirements {
            let category = req.category.clone().unwrap_or_else(|| "General".to_string());
            
            // Find traces from this requirement
            let traces_to: Vec<String> = model.traces.iter()
                .filter(|t| t.from == req.id)
                .map(|t| t.to.clone())
                .collect();
            
            let req_detail = RequirementDetail {
                id: req.id.clone(),
                description: req.description.clone(),
                priority: req.priority.clone(),
                category: req.category.clone(),
                safety_level: req.safety_level.clone(),
                traces_to,
            };
            
            req_by_category.entry(category.clone())
                .or_insert_with(Vec::new)
                .push(req_detail);
        }
        
        let requirements: Vec<RequirementSection> = req_by_category.into_iter()
            .map(|(category, reqs)| RequirementSection { category, requirements: reqs })
            .collect();
        
        // Extract component details
        let components: Vec<ComponentDetail> = model.components.iter()
            .map(|comp| {
                // Use the actual layer from component metadata, fallback to inferred if empty
                let layer = if !comp.level.is_empty() {
                    comp.level.clone()
                } else {
                    infer_layer(&comp.id)
                };
                
                let interfaces_in: Vec<InterfacePortDetail> = comp.interfaces_in.iter()
                    .map(|iface| InterfacePortDetail {
                        name: iface.name.clone(),
                        protocol: iface.protocol.clone(),
                        format: iface.format.clone(),
                    })
                    .collect();
                
                let interfaces_out: Vec<InterfacePortDetail> = comp.interfaces_out.iter()
                    .map(|iface| InterfacePortDetail {
                        name: iface.name.clone(),
                        protocol: iface.protocol.clone(),
                        format: iface.format.clone(),
                    })
                    .collect();
                
                ComponentDetail {
                    id: comp.id.clone(),
                    name: comp.name.clone(),
                    layer,
                    component_type: comp.component_type.clone(),
                    safety_level: comp.safety_level.clone(),
                    asil: comp.asil.clone(),
                    interfaces_in,
                    interfaces_out,
                    functions: comp.functions.clone(),
                    description: format!("Component in {} layer", comp.level),
                }
            })
            .collect();
        
        // Extract interface details with component names
        let interfaces: Vec<InterfaceDetail> = model.interfaces.iter()
            .map(|iface| {
                let from_comp = model.components.iter()
                    .find(|c| c.id == iface.from)
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| iface.from.clone());
                
                let to_comp = model.components.iter()
                    .find(|c| c.id == iface.to)
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| iface.to.clone());
                
                InterfaceDetail {
                    name: iface.name.clone(),
                    from: iface.from.clone(),
                    to: iface.to.clone(),
                    from_component: from_comp,
                    to_component: to_comp,
                }
            })
            .collect();
        
        // Extract function details with component association
        let functions: Vec<FunctionDetail> = model.functions.iter()
            .map(|func| {
                let component = model.components.iter()
                    .find(|c| c.functions.contains(&func.name))
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                
                FunctionDetail {
                    id: func.id.clone(),
                    name: func.name.clone(),
                    inputs: func.inputs.clone(),
                    outputs: func.outputs.clone(),
                    component,
                }
            })
            .collect();
        
        // Extract trace details
        let traces: Vec<TraceDetail> = model.traces.iter()
            .map(|trace| TraceDetail {
                from: trace.from.clone(),
                to: trace.to.clone(),
                trace_type: trace.trace_type.clone(),
                rationale: trace.rationale.clone(),
            })
            .collect();
        
        // Generate diagram data
        let dagre_graph = DagreGraph::from_model(model)?;
        let diagram = DiagramData {
            nodes: dagre_graph.nodes,
            edges: dagre_graph.edges,
            layers: dagre_graph.layers,
        };
        
        Ok(ArchitectureDocument {
            metadata,
            requirements,
            components,
            interfaces,
            functions,
            traces,
            diagram,
        })
    }
    
    pub fn generate_html(&self) -> Result<String, CompilerError> {
        let data_json = serde_json::to_string_pretty(self)
            .map_err(|e| CompilerError::Semantic(format!("JSON error: {}", e)))?;
        
        // Write JSON to separate file for embedding
        std::fs::write("arch_data.json", &data_json)
            .map_err(|e| CompilerError::Io(e))?;
        
        Ok(generate_html_template())
    }
}

fn infer_layer(comp_id: &str) -> String {
    if comp_id.starts_with("LA-SRC") { "Source".to_string() }
    else if comp_id.starts_with("LA-MIG") { "Migration".to_string() }
    else if comp_id.starts_with("LA-TGT") { "Target".to_string() }
    else if comp_id.starts_with("LA-PROC") { "Processing".to_string() }
    else if comp_id.starts_with("LA-GOV") { "Governance".to_string() }
    else if comp_id.starts_with("LA-INT") { "Integration".to_string() }
    else if comp_id.starts_with("LA-ANLZ") { "Analytics".to_string() }
    else if comp_id.starts_with("LA-MON") { "Monitoring".to_string() }
    else { "Other".to_string() }
}

fn generate_html_template() -> String {
    include_str!("arcviz_explorer_template.html").to_string()
}

pub fn generate_explorer_html(model: &SemanticModel) -> Result<(String, String), CompilerError> {
    let doc = ArchitectureDocument::from_model(model)?;
    let json = serde_json::to_string_pretty(&doc)
        .map_err(|e| CompilerError::Semantic(format!("JSON error: {}", e)))?;
    
    // Embed JSON data into HTML template
    let template = generate_html_template();
    let html = template.replace(
        "/*ARCH_DATA_PLACEHOLDER*/null/*END_ARCH_DATA_PLACEHOLDER*/",
        &json
    );
    
    Ok((html, json))
}
