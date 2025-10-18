use super::semantic::SemanticModel;
use super::CompilerError;
use std::collections::HashMap;

pub struct PlantUMLGenerator {
    diagram_type: String,
}

impl PlantUMLGenerator {
    pub fn new() -> Self {
        Self {
            diagram_type: "component".to_string(),
        }
    }
    
    pub fn with_type(mut self, diagram_type: String) -> Self {
        self.diagram_type = diagram_type;
        self
    }
    
    pub fn generate(&self, model: &SemanticModel) -> Result<String, CompilerError> {
        let mut output = String::new();
        
        output.push_str("@startuml\n");
        output.push_str("!theme plain\n\n");
        
        if self.diagram_type == "component" {
            self.generate_component_diagram(&mut output, model)?;
        } else if self.diagram_type == "activity" {
            self.generate_activity_diagram(&mut output, model)?;
        }
        
        output.push_str("\n@enduml\n");
        
        Ok(output)
    }
    
    fn generate_component_diagram(&self, output: &mut String, model: &SemanticModel) -> Result<(), CompilerError> {
        let mut packages: HashMap<String, Vec<&super::semantic::ComponentInfo>> = HashMap::new();
        
        for comp in &model.components {
            let category = "System Components";
            packages.entry(category.to_string()).or_insert_with(Vec::new).push(comp);
        }
        
        for (package_name, components) in &packages {
            output.push_str(&format!("package \"{}\" {{\n", package_name));
            
            for comp in components {
                output.push_str(&format!("  [{}]\n", comp.name));
            }
            
            output.push_str("}\n\n");
        }
        
        for trace in &model.traces {
            if trace.trace_type == "implements" || trace.trace_type == "satisfies" {
                let from_name = self.find_component_name(&model.components, &trace.from);
                let to_name = self.find_component_name(&model.components, &trace.to);
                
                if !from_name.is_empty() && !to_name.is_empty() {
                    output.push_str(&format!("[{}] --> [{}]\n", from_name, to_name));
                }
            }
        }
        
        Ok(())
    }
    
    fn generate_activity_diagram(&self, output: &mut String, model: &SemanticModel) -> Result<(), CompilerError> {
        output.push_str("start\n\n");
        
        for req in &model.requirements {
            let desc = req.description.replace('\n', "\\n");
            output.push_str(&format!(":{};\n", desc));
        }
        
        output.push_str("\nstop\n");
        
        Ok(())
    }
    
    fn find_component_name(&self, components: &[super::semantic::ComponentInfo], id: &str) -> String {
        components.iter()
            .find(|c| c.id == id)
            .map(|c| c.name.clone())
            .unwrap_or_default()
    }
}

pub fn generate_plantuml_component(model: &SemanticModel) -> Result<String, CompilerError> {
    PlantUMLGenerator::new()
        .with_type("component".to_string())
        .generate(model)
}

pub fn generate_plantuml_activity(model: &SemanticModel) -> Result<String, CompilerError> {
    PlantUMLGenerator::new()
        .with_type("activity".to_string())
        .generate(model)
}
