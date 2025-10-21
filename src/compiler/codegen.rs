use super::semantic::SemanticModel;
use super::CompilerConfig;
use super::CompilerError;

pub struct CodeGenerator<'a> {
    config: &'a CompilerConfig,
}

impl<'a> CodeGenerator<'a> {
    pub fn new(config: &'a CompilerConfig) -> Self {
        Self { config }
    }
    
    pub fn generate(&self, model: &SemanticModel) -> Result<String, CompilerError> {
        match self.config.target.as_str() {
            "json" => self.generate_json(model),
            "capella" => self.generate_capella(model),
            "markdown" => self.generate_markdown(model),
            "mermaid" => self.generate_mermaid(model),
            "terraform" => self.generate_terraform(model),
            _ => Err(CompilerError::Semantic(format!("Unknown target: {}", self.config.target))),
        }
    }
    
    fn generate_terraform(&self, model: &SemanticModel) -> Result<String, CompilerError> {
        use super::terraform_databricks_generator::{generate_terraform_databricks, TerraformConfig};
        let config = TerraformConfig::default();
        generate_terraform_databricks(model, &config)
    }
    
    fn generate_mermaid(&self, model: &SemanticModel) -> Result<String, CompilerError> {
        use super::mermaid_generator::generate_mermaid_flowchart;
        generate_mermaid_flowchart(model, "System Requirements", "elk")
    }
    
    fn generate_json(&self, model: &SemanticModel) -> Result<String, CompilerError> {
        let requirements: Vec<_> = model.requirements.iter().map(|r| {
            serde_json::json!({
                "id": r.id,
                "description": r.description,
                "priority": r.priority,
                "safety_level": r.safety_level,
            })
        }).collect();
        
        let components: Vec<_> = model.components.iter().map(|c| {
            serde_json::json!({
                "id": c.id,
                "name": c.name,
                "type": c.component_type,
                "level": c.level,
            })
        }).collect();
        
        let functions: Vec<_> = model.functions.iter().map(|f| {
            serde_json::json!({
                "id": f.id,
                "name": f.name,
                "inputs": f.inputs,
                "outputs": f.outputs,
            })
        }).collect();
        
        let traces: Vec<_> = model.traces.iter().map(|t| {
            serde_json::json!({
                "from": t.from,
                "to": t.to,
                "type": t.trace_type,
                "rationale": t.rationale,
            })
        }).collect();
        
        let metrics = model.compute_metrics();
        let metrics_json = serde_json::json!({
            "total_elements": metrics.total_elements,
            "requirements": metrics.requirements_count,
            "components": metrics.components_count,
            "functions": metrics.functions_count,
            "traces": metrics.traces_count,
            "traceability_coverage": metrics.traceability_coverage,
        });
        
        let json = serde_json::json!({
            "requirements": requirements,
            "components": components,
            "functions": functions,
            "traces": traces,
            "metrics": metrics_json
        });
        
        Ok(serde_json::to_string_pretty(&json)
            .map_err(|e| CompilerError::Semantic(e.to_string()))?)
    }
    
    fn generate_capella(&self, model: &SemanticModel) -> Result<String, CompilerError> {
        let mut xml = String::new();
        
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<capella:Project xmlns:capella=\"http://www.polarsys.org/capella/core/1.4.0\">\n");
        xml.push_str("  <ownedRequirements>\n");
        
        for req in &model.requirements {
            xml.push_str(&format!(
                "    <requirement id=\"{}\" name=\"{}\" description=\"{}\" priority=\"{}\" />\n",
                req.id, req.id, req.description, req.priority
            ));
        }
        
        xml.push_str("  </ownedRequirements>\n");
        xml.push_str("  <ownedLogicalComponents>\n");
        
        for comp in &model.components {
            xml.push_str(&format!(
                "    <component id=\"{}\" name=\"{}\" type=\"{}\" />\n",
                comp.id, comp.name, comp.component_type
            ));
        }
        
        xml.push_str("  </ownedLogicalComponents>\n");
        xml.push_str("  <ownedTraces>\n");
        
        for trace in &model.traces {
            xml.push_str(&format!(
                "    <trace from=\"{}\" to=\"{}\" type=\"{}\" />\n",
                trace.from, trace.to, trace.trace_type
            ));
        }
        
        xml.push_str("  </ownedTraces>\n");
        xml.push_str("</capella:Project>\n");
        
        Ok(xml)
    }
    
    fn generate_markdown(&self, model: &SemanticModel) -> Result<String, CompilerError> {
        let mut md = String::new();
        
        md.push_str("# ArcLang Model Report\n\n");
        
        // Metrics
        let metrics = model.compute_metrics();
        md.push_str("## Metrics\n\n");
        md.push_str(&format!("- Total Elements: {}\n", metrics.total_elements));
        md.push_str(&format!("- Requirements: {}\n", metrics.requirements_count));
        md.push_str(&format!("- Components: {}\n", metrics.components_count));
        md.push_str(&format!("- Functions: {}\n", metrics.functions_count));
        md.push_str(&format!("- Traces: {}\n", metrics.traces_count));
        md.push_str(&format!("- Traceability Coverage: {:.1}%\n\n", metrics.traceability_coverage));
        
        // Requirements
        md.push_str("## Requirements\n\n");
        for req in &model.requirements {
            md.push_str(&format!("### {}\n\n", req.id));
            md.push_str(&format!("- **Description**: {}\n", req.description));
            md.push_str(&format!("- **Priority**: {}\n", req.priority));
            if let Some(ref safety) = req.safety_level {
                md.push_str(&format!("- **Safety Level**: {}\n", safety));
            }
            
            let traces = model.get_traces_from(&req.id);
            if !traces.is_empty() {
                md.push_str("- **Traces to**:\n");
                for trace in traces {
                    md.push_str(&format!("  - {} ({})\n", trace.to, trace.trace_type));
                }
            }
            md.push_str("\n");
        }
        
        // Components
        md.push_str("## Components\n\n");
        for comp in &model.components {
            md.push_str(&format!("### {}\n\n", comp.name));
            md.push_str(&format!("- **Type**: {}\n", comp.component_type));
            md.push_str(&format!("- **Level**: {}\n", comp.level));
            
            let traces = model.get_traces_to(&comp.id);
            if !traces.is_empty() {
                md.push_str("- **Traced from**:\n");
                for trace in traces {
                    md.push_str(&format!("  - {} ({})\n", trace.from, trace.trace_type));
                }
            }
            md.push_str("\n");
        }
        
        // Traceability Matrix
        md.push_str("## Traceability Matrix\n\n");
        md.push_str("| From | To | Type | Rationale |\n");
        md.push_str("|------|----|----|--------|\n");
        for trace in &model.traces {
            let rationale = trace.rationale.as_deref().unwrap_or("-");
            md.push_str(&format!("| {} | {} | {} | {} |\n", 
                trace.from, trace.to, trace.trace_type, rationale));
        }
        
        Ok(md)
    }
}
