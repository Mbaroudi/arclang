use super::CompilerError;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MermaidNode {
    pub id: String,
    pub label: String,
    pub category: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MermaidEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug)]
pub struct MermaidModel {
    pub title: String,
    pub nodes: Vec<MermaidNode>,
    pub edges: Vec<MermaidEdge>,
    pub subgraphs: HashMap<String, Vec<String>>,
}

pub struct MermaidImporter;

impl MermaidImporter {
    pub fn new() -> Self {
        Self
    }
    
    pub fn import(&self, content: &str) -> Result<MermaidModel, CompilerError> {
        let mut title = "Imported Model".to_string();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut subgraphs: HashMap<String, Vec<String>> = HashMap::new();
        let mut current_subgraph: Option<String> = None;
        
        for line in content.lines() {
            let trimmed = line.trim();
            
            if trimmed.starts_with("title:") {
                title = trimmed.trim_start_matches("title:").trim().to_string();
            }
            else if trimmed.starts_with("subgraph ") {
                let parts: Vec<&str> = trimmed.split('[').collect();
                if parts.len() >= 2 {
                    let category = parts[1]
                        .trim_end_matches(']')
                        .trim_end_matches('"')
                        .trim_start_matches('"')
                        .to_string();
                    current_subgraph = Some(category.clone());
                    subgraphs.entry(category).or_insert_with(Vec::new);
                }
            }
            else if trimmed == "end" {
                current_subgraph = None;
            }
            else if trimmed.contains("[\"") && trimmed.contains("\"]") {
                let node = self.parse_node(trimmed)?;
                
                if let Some(ref category) = current_subgraph {
                    if let Some(nodes_list) = subgraphs.get_mut(category) {
                        nodes_list.push(node.id.clone());
                    }
                }
                
                let mut node_with_category = node;
                node_with_category.category = current_subgraph.clone();
                nodes.push(node_with_category);
            }
            else if trimmed.contains("-->") {
                if let Some(edge) = self.parse_edge(trimmed) {
                    edges.push(edge);
                }
            }
        }
        
        Ok(MermaidModel {
            title,
            nodes,
            edges,
            subgraphs,
        })
    }
    
    fn parse_node(&self, line: &str) -> Result<MermaidNode, CompilerError> {
        let parts: Vec<&str> = line.split('[').collect();
        if parts.len() < 2 {
            return Err(CompilerError::Parse(format!("Invalid node format: {}", line)));
        }
        
        let id = parts[0].trim().to_string();
        
        let label_part = parts[1].trim_end_matches(']');
        let label_content = label_part
            .trim_start_matches('"')
            .trim_end_matches('"');
        
        let label_parts: Vec<&str> = label_content.split("<br>").collect();
        let label = if label_parts.len() > 1 {
            label_parts[1].to_string()
        } else {
            label_parts[0].to_string()
        };
        
        Ok(MermaidNode {
            id,
            label,
            category: None,
        })
    }
    
    fn parse_edge(&self, line: &str) -> Option<MermaidEdge> {
        let parts: Vec<&str> = line.split("-->").collect();
        if parts.len() == 2 {
            Some(MermaidEdge {
                from: parts[0].trim().to_string(),
                to: parts[1].trim().to_string(),
            })
        } else {
            None
        }
    }
}

pub struct ArcLangGenerator;

impl ArcLangGenerator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn generate(&self, model: &MermaidModel) -> Result<String, CompilerError> {
        let mut output = String::new();
        
        output.push_str(&format!("// Generated from Mermaid diagram: {}\n", model.title));
        output.push_str("// This file was automatically generated from a Mermaid flowchart\n\n");
        
        output.push_str(&format!("system_analysis \"{}\" {{\n", model.title));
        
        for node in &model.nodes {
            let category = node.category.as_ref()
                .map(|c| format!("        category: \"{}\"\n", c))
                .unwrap_or_default();
            
            output.push_str(&format!("    requirement \"{}\" {{\n", node.id));
            output.push_str(&format!("        id: \"{}\"\n", node.id));
            output.push_str(&format!("        description: \"{}\"\n", node.label));
            output.push_str(&category);
            output.push_str("        priority: \"Medium\"\n");
            output.push_str("    }\n\n");
        }
        
        output.push_str("}\n\n");
        
        if !model.edges.is_empty() {
            output.push_str("// Traceability relationships\n");
            for edge in &model.edges {
                output.push_str(&format!(
                    "trace \"{}\" implements \"{}\" {{\n",
                    edge.from, edge.to
                ));
                output.push_str("    rationale: \"Dependency from Mermaid diagram\"\n");
                output.push_str("}\n\n");
            }
        }
        
        Ok(output)
    }
}

pub fn import_mermaid(content: &str) -> Result<String, CompilerError> {
    let importer = MermaidImporter::new();
    let model = importer.import(content)?;
    
    let generator = ArcLangGenerator::new();
    generator.generate(&model)
}
