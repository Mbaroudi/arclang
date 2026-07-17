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

use super::ast::{Model, StateKind};
use super::semantic::SemanticModel;
use super::CompilerError;
use super::graph_model::{DagreGraph, GraphNode, GraphEdge, LayerInfo};
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
    #[serde(default)]
    pub missions: Vec<MissionDetail>,
    #[serde(default)]
    pub capabilities: Vec<CapabilityDetail>,
    #[serde(default)]
    pub functional_chains: Vec<ChainDetail>,
    #[serde(default)]
    pub state_machines: Vec<StateMachineDetail>,
    #[serde(default)]
    pub scenario_details: Vec<ScenarioDetail>,
    #[serde(default)]
    pub data_model: DataModelDetail,
    pub diagram: DiagramData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MissionDetail {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CapabilityDetail {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub mission: Option<String>,
    pub realizes: Option<String>,
    pub involves: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainDetail {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub involves: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateMachineDetail {
    pub name: String,
    pub initial: String,
    pub states: Vec<StateDetail>,
    pub transitions: Vec<TransitionDetail>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateDetail {
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransitionDetail {
    pub from: String,
    pub to: String,
    pub trigger: String,
    pub guard: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScenarioDetail {
    pub name: String,
    pub participants: Vec<String>,
    pub messages: Vec<MessageDetail>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageDetail {
    pub from: String,
    pub to: String,
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DataModelDetail {
    pub classes: Vec<ClassDetail>,
    pub enumerations: Vec<EnumDetail>,
    pub data_types: Vec<TypeDetail>,
    pub exchange_items: Vec<ExchangeItemDetail>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassDetail {
    pub name: String,
    pub fields: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnumDetail {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TypeDetail {
    pub name: String,
    pub base: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExchangeItemDetail {
    pub name: String,
    pub mechanism: String,
    pub elements: Vec<String>,
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
    /// Human-readable names resolved from the element registry.
    #[serde(default)]
    pub from_name: String,
    #[serde(default)]
    pub to_name: String,
    pub trace_type: String,
    pub rationale: Option<String>,
}

impl ArchitectureDocument {
    pub fn from_model(model: &SemanticModel, ast: &Model) -> Result<Self, CompilerError> {
        let display_name = |reference: &str| -> String {
            model
                .all_elements
                .get(reference)
                .map(|e| e.name.clone())
                .unwrap_or_else(|| reference.to_string())
        };

        let missions = model
            .missions
            .iter()
            .map(|m| MissionDetail { id: m.id.clone(), name: m.name.clone() })
            .collect();
        let capabilities = model
            .capabilities
            .iter()
            .map(|c| CapabilityDetail {
                id: c.id.clone(),
                name: c.name.clone(),
                kind: c.kind.clone(),
                mission: c.mission.as_deref().map(display_name),
                realizes: c.realizes.as_deref().map(display_name),
                involves: c.involves.iter().map(|i| display_name(i)).collect(),
            })
            .collect();
        // System-function latencies: shown on chain steps so the timing
        // budget of a chain is readable directly in the dossier.
        let mut sf_latency: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        for sa in &ast.system_analysis {
            for func in &sa.functions {
                if let Some(lat) = func.attributes.get("latency").and_then(|v| v.as_string()) {
                    sf_latency.insert(func.id.clone(), lat.to_string());
                    sf_latency.insert(func.name.clone(), lat.to_string());
                }
            }
        }
        let mut functional_chains: Vec<ChainDetail> = model
            .functional_chains
            .iter()
            .map(|chain| ChainDetail {
                id: chain.id.clone(),
                name: chain.name.clone(),
                kind: "Functional Chain".to_string(),
                involves: chain
                    .involves
                    .iter()
                    .map(|i| {
                        let shown = display_name(i);
                        match sf_latency.get(i).or_else(|| sf_latency.get(&shown)) {
                            Some(lat) => format!("{shown} \u{2022} {lat}"),
                            None => shown,
                        }
                    })
                    .collect(),
            })
            .collect();
        for oa in &ast.operational_analysis {
            for process in &oa.processes {
                functional_chains.push(ChainDetail {
                    id: process.id.clone(),
                    name: process.name.clone(),
                    kind: "Operational Process".to_string(),
                    involves: process.involves.iter().map(|i| display_name(i)).collect(),
                });
            }
        }
        let state_machines = ast
            .state_machines
            .iter()
            .map(|machine| StateMachineDetail {
                name: machine.name.clone(),
                initial: machine.initial_state.clone(),
                states: machine
                    .states
                    .iter()
                    .map(|s| StateDetail {
                        name: s.name.clone(),
                        kind: if s.kind == StateKind::Mode { "mode" } else { "state" }.to_string(),
                    })
                    .collect(),
                transitions: machine
                    .transitions
                    .iter()
                    .map(|t| TransitionDetail {
                        from: t.from.clone(),
                        to: t.to.clone(),
                        trigger: t.trigger.clone(),
                        guard: t.guard.clone(),
                    })
                    .collect(),
            })
            .collect();
        let scenario_details = ast
            .scenarios
            .iter()
            .map(|scenario| ScenarioDetail {
                name: scenario.name.clone(),
                participants: scenario.participants.iter().map(|p| display_name(&p.id)).collect(),
                messages: scenario
                    .messages
                    .iter()
                    .map(|m| MessageDetail {
                        from: display_name(&m.from),
                        to: display_name(&m.to),
                        label: m.label.clone(),
                    })
                    .collect(),
            })
            .collect();
        let data_model = DataModelDetail {
            classes: ast
                .classes
                .iter()
                .map(|c| ClassDetail {
                    name: c.name.clone(),
                    fields: c.fields.iter().map(|f| format!("{}: {}", f.name, f.attr_type)).collect(),
                })
                .collect(),
            enumerations: ast
                .data_types
                .iter()
                .filter(|d| d.enumeration_values.is_some())
                .map(|d| EnumDetail {
                    name: d.name.clone(),
                    values: d
                        .enumeration_values
                        .as_ref()
                        .map(|vs| vs.iter().map(|v| v.name.clone()).collect())
                        .unwrap_or_default(),
                })
                .collect(),
            data_types: ast
                .data_types
                .iter()
                .filter(|d| d.enumeration_values.is_none())
                .map(|d| TypeDetail { name: d.name.clone(), base: d.base_type.clone() })
                .collect(),
            exchange_items: ast
                .exchange_items
                .iter()
                .map(|item| ExchangeItemDetail {
                    name: item.name.clone(),
                    mechanism: item.stereotype.clone(),
                    elements: item.elements.iter().map(|e| display_name(e)).collect(),
                })
                .collect(),
        };

        // The title is the model's declared name — never guessed from content.
        let title = model
            .name
            .clone()
            .unwrap_or_else(|| "System Architecture".to_string());
        
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
                from_name: display_name(&trace.from),
                to_name: display_name(&trace.to),
                from: trace.from.clone(),
                to: trace.to.clone(),
                trace_type: trace.trace_type.clone(),
                rationale: trace.rationale.clone(),
            })
            .collect();
        
        // Generate diagram data, then enrich nodes with AST-level detail the
        // semantic model flattens away: function latencies and the logical
        // components deployed on each physical node (deployment view).
        let mut latency_of: std::collections::HashMap<(String, String), String> =
            std::collections::HashMap::new();
        fn walk_latencies(
            comp: &crate::compiler::ast::LogicalComponent,
            map: &mut std::collections::HashMap<(String, String), String>,
        ) {
            for f in &comp.functions {
                if let Some(lat) = f.attributes.get("latency").and_then(|v| v.as_string()) {
                    if !comp.id.is_empty() {
                        map.insert((comp.id.clone(), f.name.clone()), lat.to_string());
                    }
                    map.insert((comp.name.clone(), f.name.clone()), lat.to_string());
                }
            }
            for sub in &comp.sub_components {
                walk_latencies(sub, map);
            }
        }
        for la in &ast.logical_architecture {
            for comp in &la.components {
                walk_latencies(comp, &mut latency_of);
            }
        }
        let mut deployed_on: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for pa in &ast.physical_architecture {
            for node in &pa.nodes {
                for dep in &node.deployments {
                    let shown = display_name(&dep.component);
                    if !node.id.is_empty() {
                        deployed_on.entry(node.id.clone()).or_default().push(shown.clone());
                    }
                    deployed_on.entry(node.name.clone()).or_default().push(shown);
                }
            }
        }

        let dagre_graph = DagreGraph::from_model(model)?;
        let mut diagram_nodes = dagre_graph.nodes;
        for node in &mut diagram_nodes {
            node.functions = node
                .functions
                .iter()
                .map(|f| {
                    latency_of
                        .get(&(node.id.clone(), f.clone()))
                        .or_else(|| latency_of.get(&(node.label.clone(), f.clone())))
                        .map(|lat| format!("{f}  \u{2022} {lat}"))
                        .unwrap_or_else(|| f.clone())
                })
                .collect();
            if let Some(dep) = deployed_on
                .get(&node.id)
                .or_else(|| deployed_on.get(&node.label))
            {
                node.deployed = dep.clone();
            }
            let lines = (node.functions.len() + node.deployed.len()) as u32;
            node.height = node.height.max(64 + 18 * lines + 14);
        }
        let diagram = DiagramData {
            nodes: diagram_nodes,
            edges: dagre_graph.edges,
            layers: dagre_graph.layers,
        };
        
        Ok(ArchitectureDocument {
            metadata,
            missions,
            capabilities,
            functional_chains,
            state_machines,
            scenario_details,
            data_model,
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

pub fn generate_explorer_html(model: &SemanticModel, ast: &Model) -> Result<(String, String), CompilerError> {
    let doc = ArchitectureDocument::from_model(model, ast)?;
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
