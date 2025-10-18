use super::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SemanticModel {
    pub requirements: Vec<RequirementInfo>,
    pub components: Vec<ComponentInfo>,
    pub functions: Vec<FunctionInfo>,
    pub traces: Vec<TraceInfo>,
    pub all_elements: HashMap<String, ElementInfo>,
}

#[derive(Debug, Clone)]
pub struct RequirementInfo {
    pub id: String,
    pub description: String,
    pub priority: String,
    pub category: Option<String>,
    pub safety_level: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ComponentInfo {
    pub id: String,
    pub name: String,
    pub component_type: String,
    pub level: String,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub id: String,
    pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TraceInfo {
    pub from: String,
    pub to: String,
    pub trace_type: String,
    pub rationale: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ElementInfo {
    pub id: String,
    pub name: String,
    pub element_type: String,
}

pub struct SemanticAnalyzer;

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    pub fn analyze(&self, ast: &Model) -> Result<SemanticModel, String> {
        let mut requirements = Vec::new();
        let mut components = Vec::new();
        let mut functions = Vec::new();
        let mut traces = Vec::new();
        let mut all_elements = HashMap::new();
        
        // Collect requirements from system analysis
        for sa in &ast.system_analysis {
            for req in &sa.requirements {
                let description = req.attributes.get("description")
                    .and_then(|v| v.as_string())
                    .unwrap_or("")
                    .to_string();
                
                let priority = req.attributes.get("priority")
                    .and_then(|v| v.as_string())
                    .unwrap_or("Medium")
                    .to_string();
                
                let category = req.attributes.get("category")
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_string());
                
                let safety_level = req.attributes.get("safety_level")
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_string());
                
                requirements.push(RequirementInfo {
                    id: req.id.clone(),
                    description,
                    priority,
                    category,
                    safety_level,
                });
                
                all_elements.insert(req.id.clone(), ElementInfo {
                    id: req.id.clone(),
                    name: req.id.clone(),
                    element_type: "Requirement".to_string(),
                });
            }
        }
        
        // Collect components from logical architecture
        for la in &ast.logical_architecture {
            for comp in &la.components {
                let comp_id = comp.attributes.get("id")
                    .and_then(|v| v.as_string())
                    .unwrap_or(&comp.name)
                    .to_string();
                
                let comp_type = comp.attributes.get("type")
                    .and_then(|v| v.as_string())
                    .unwrap_or("Logical")
                    .to_string();
                
                components.push(ComponentInfo {
                    id: comp_id.clone(),
                    name: comp.name.clone(),
                    component_type: comp_type,
                    level: "Logical".to_string(),
                });
                
                all_elements.insert(comp_id.clone(), ElementInfo {
                    id: comp_id.clone(),
                    name: comp.name.clone(),
                    element_type: "Component".to_string(),
                });
                
                // Collect functions from components
                for func in &comp.functions {
                    let func_id = func.attributes.get("id")
                        .and_then(|v| v.as_string())
                        .unwrap_or(&func.name)
                        .to_string();
                    
                    let inputs = func.attributes.get("inputs")
                        .and_then(|v| {
                            if let AttributeValue::List(list) = v {
                                Some(list.iter()
                                    .filter_map(|v| v.as_string().map(|s| s.to_string()))
                                    .collect())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();
                    
                    let outputs = func.attributes.get("outputs")
                        .and_then(|v| {
                            if let AttributeValue::List(list) = v {
                                Some(list.iter()
                                    .filter_map(|v| v.as_string().map(|s| s.to_string()))
                                    .collect())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();
                    
                    functions.push(FunctionInfo {
                        id: func_id.clone(),
                        name: func.name.clone(),
                        inputs,
                        outputs,
                    });
                    
                    all_elements.insert(func_id.clone(), ElementInfo {
                        id: func_id.clone(),
                        name: func.name.clone(),
                        element_type: "Function".to_string(),
                    });
                }
            }
        }
        
        // Collect traces
        for trace in &ast.traces {
            let rationale = trace.attributes.get("rationale")
                .and_then(|v| v.as_string())
                .map(|s| s.to_string());
            
            traces.push(TraceInfo {
                from: trace.from.clone(),
                to: trace.to.clone(),
                trace_type: trace.trace_type.clone(),
                rationale,
            });
        }
        
        // Validate traceability
        self.validate_traces(&traces, &all_elements)?;
        
        Ok(SemanticModel {
            requirements,
            components,
            functions,
            traces,
            all_elements,
        })
    }
    
    fn validate_traces(
        &self,
        traces: &[TraceInfo],
        elements: &HashMap<String, ElementInfo>,
    ) -> Result<(), String> {
        for trace in traces {
            if !elements.contains_key(&trace.from) {
                return Err(format!("Trace references unknown element: {}", trace.from));
            }
            if !elements.contains_key(&trace.to) {
                return Err(format!("Trace references unknown element: {}", trace.to));
            }
        }
        Ok(())
    }
}

impl SemanticModel {
    pub fn get_requirement(&self, id: &str) -> Option<&RequirementInfo> {
        self.requirements.iter().find(|r| r.id == id)
    }
    
    pub fn get_component(&self, id: &str) -> Option<&ComponentInfo> {
        self.components.iter().find(|c| c.id == id)
    }
    
    pub fn get_traces_from(&self, element_id: &str) -> Vec<&TraceInfo> {
        self.traces.iter()
            .filter(|t| t.from == element_id)
            .collect()
    }
    
    pub fn get_traces_to(&self, element_id: &str) -> Vec<&TraceInfo> {
        self.traces.iter()
            .filter(|t| t.to == element_id)
            .collect()
    }
    
    pub fn validate_traceability(&self) -> Vec<String> {
        let mut issues = Vec::new();
        
        // Check for requirements without downstream traces
        for req in &self.requirements {
            if self.get_traces_from(&req.id).is_empty() {
                issues.push(format!("Requirement {} has no downstream traces", req.id));
            }
        }
        
        // Check for components without upstream traces
        for comp in &self.components {
            if self.get_traces_to(&comp.id).is_empty() {
                issues.push(format!("Component {} has no upstream traces", comp.id));
            }
        }
        
        issues
    }
    
    pub fn compute_metrics(&self) -> ModelMetrics {
        let total_elements = self.requirements.len() + self.components.len() + self.functions.len();
        
        let traced_requirements = self.requirements.iter()
            .filter(|r| !self.get_traces_from(&r.id).is_empty())
            .count();
        
        let traceability_coverage = if !self.requirements.is_empty() {
            (traced_requirements as f64 / self.requirements.len() as f64) * 100.0
        } else {
            0.0
        };
        
        ModelMetrics {
            total_elements,
            requirements_count: self.requirements.len(),
            components_count: self.components.len(),
            functions_count: self.functions.len(),
            traces_count: self.traces.len(),
            traceability_coverage,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModelMetrics {
    pub total_elements: usize,
    pub requirements_count: usize,
    pub components_count: usize,
    pub functions_count: usize,
    pub traces_count: usize,
    pub traceability_coverage: f64,
}
