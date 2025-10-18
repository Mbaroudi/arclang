use super::CompilerError;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PlantUMLComponent {
    pub id: String,
    pub name: String,
    pub component_type: String,
    pub parent: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PlantUMLRelation {
    pub from: String,
    pub to: String,
    pub relation_type: String,
}

#[derive(Debug)]
pub struct PlantUMLModel {
    pub diagram_type: String,
    pub components: Vec<PlantUMLComponent>,
    pub relations: Vec<PlantUMLRelation>,
    pub packages: HashMap<String, Vec<String>>,
}

pub struct PlantUMLImporter;

impl PlantUMLImporter {
    pub fn new() -> Self {
        Self
    }
    
    pub fn import(&self, content: &str) -> Result<PlantUMLModel, CompilerError> {
        let mut diagram_type = "component".to_string();
        let mut components = Vec::new();
        let mut relations = Vec::new();
        let mut packages: HashMap<String, Vec<String>> = HashMap::new();
        let mut current_package: Option<String> = None;
        let mut component_id_counter = 1;
        
        for line in content.lines() {
            let trimmed = line.trim();
            
            if trimmed.starts_with("@startuml") {
                continue;
            } else if trimmed.starts_with("@enduml") {
                break;
            } else if trimmed.contains("start") || trimmed.contains("stop") {
                diagram_type = "activity".to_string();
            } else if trimmed.starts_with("package ") || trimmed.starts_with("node ") || 
                      trimmed.starts_with("cloud") || trimmed.starts_with("database ") ||
                      trimmed.starts_with("folder ") || trimmed.starts_with("frame ") {
                let package_name = self.extract_package_name(trimmed);
                current_package = Some(package_name.clone());
                packages.entry(package_name).or_insert_with(Vec::new);
            } else if trimmed == "}" {
                current_package = None;
            } else if trimmed.starts_with(':') && (trimmed.ends_with(';') || trimmed.contains(';')) {
                if let Some(activity) = self.parse_activity(trimmed, &mut component_id_counter) {
                    components.push(activity);
                }
            } else if trimmed.contains("[") && trimmed.contains("]") {
                if let Some(comp) = self.parse_component(trimmed, &current_package, &mut component_id_counter) {
                    if let Some(ref pkg) = current_package {
                        if let Some(components_list) = packages.get_mut(pkg) {
                            components_list.push(comp.id.clone());
                        }
                    }
                    components.push(comp);
                }
            } else if trimmed.contains("-->") || trimmed.contains("->") {
                if let Some(rel) = self.parse_relation(trimmed) {
                    relations.push(rel);
                }
            }
        }
        
        Ok(PlantUMLModel {
            diagram_type,
            components,
            relations,
            packages,
        })
    }
    
    fn extract_package_name(&self, line: &str) -> String {
        if let Some(start) = line.find('"') {
            if let Some(end) = line[start + 1..].find('"') {
                return line[start + 1..start + 1 + end].to_string();
            }
        }
        
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            parts[1].trim_matches('"').to_string()
        } else {
            "Unknown".to_string()
        }
    }
    
    fn parse_component(&self, line: &str, parent: &Option<String>, counter: &mut i32) -> Option<PlantUMLComponent> {
        let start = line.find('[')?;
        let end = line[start..].find(']')?;
        let name = line[start + 1..start + end].trim().to_string();
        
        let component_type = if line.contains("HTTP") || line.contains("FTP") {
            "Interface"
        } else if parent.is_some() {
            "Logical"
        } else {
            "Logical"
        };
        
        let id = format!("LC-{:03}", counter);
        *counter += 1;
        
        Some(PlantUMLComponent {
            id,
            name,
            component_type: component_type.to_string(),
            parent: parent.clone(),
        })
    }
    
    fn parse_activity(&self, line: &str, counter: &mut i32) -> Option<PlantUMLComponent> {
        if line.trim().starts_with(':') {
            let content = line.trim()
                .trim_start_matches(':')
                .trim_end_matches(';')
                .trim();
            
            let name = content
                .replace("**", "")
                .replace("\\n", " ")
                .trim()
                .to_string();
            
            let id = format!("ACT-{:03}", counter);
            *counter += 1;
            
            Some(PlantUMLComponent {
                id,
                name,
                component_type: "Activity".to_string(),
                parent: None,
            })
        } else {
            None
        }
    }
    
    fn parse_relation(&self, line: &str) -> Option<PlantUMLRelation> {
        let arrow = if line.contains("-->") {
            "-->"
        } else if line.contains("->") {
            "->"
        } else {
            return None;
        };
        
        let parts: Vec<&str> = line.split(arrow).collect();
        if parts.len() != 2 {
            return None;
        }
        
        let from = self.extract_component_name(parts[0].trim());
        let to = self.extract_component_name(parts[1].trim());
        
        Some(PlantUMLRelation {
            from,
            to,
            relation_type: "implements".to_string(),
        })
    }
    
    fn extract_component_name(&self, text: &str) -> String {
        if let Some(start) = text.find('[') {
            if let Some(end) = text[start..].find(']') {
                return text[start + 1..start + end].trim().to_string();
            }
        }
        text.trim().to_string()
    }
}

pub struct PlantUMLArcLangGenerator;

impl PlantUMLArcLangGenerator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn generate(&self, model: &PlantUMLModel) -> Result<String, CompilerError> {
        let mut output = String::new();
        
        output.push_str("// Generated from PlantUML diagram\n");
        output.push_str(&format!("// Diagram type: {}\n\n", model.diagram_type));
        
        if model.diagram_type == "activity" {
            self.generate_operational_analysis(&mut output, model)?;
        } else {
            self.generate_logical_architecture(&mut output, model)?;
        }
        
        if !model.relations.is_empty() {
            output.push_str("\n// Component relationships\n");
            for rel in &model.relations {
                output.push_str(&format!(
                    "trace \"{}\" implements \"{}\" {{\n",
                    self.find_component_id(&model.components, &rel.from),
                    self.find_component_id(&model.components, &rel.to)
                ));
                output.push_str("    rationale: \"Dependency from PlantUML diagram\"\n");
                output.push_str("}\n\n");
            }
        }
        
        Ok(output)
    }
    
    fn generate_operational_analysis(&self, output: &mut String, model: &PlantUMLModel) -> Result<(), CompilerError> {
        output.push_str("operational_analysis \"Process Flow\" {\n");
        
        for comp in &model.components {
            if comp.component_type == "Activity" {
                output.push_str(&format!("    actor \"{}\" {{\n", comp.name));
                output.push_str(&format!("        id: \"{}\"\n", comp.id));
                output.push_str(&format!("        description: \"{}\"\n", comp.name));
                output.push_str("        concerns: [\"Process execution\"]\n");
                output.push_str("    }\n\n");
            }
        }
        
        output.push_str("}\n\n");
        Ok(())
    }
    
    fn generate_logical_architecture(&self, output: &mut String, model: &PlantUMLModel) -> Result<(), CompilerError> {
        output.push_str("logical_architecture \"System Architecture\" {\n");
        
        for comp in &model.components {
            let category = comp.parent.as_ref()
                .map(|p| format!("        category: \"{}\"\n", p))
                .unwrap_or_default();
            
            output.push_str(&format!("    component \"{}\" {{\n", comp.name));
            output.push_str(&format!("        id: \"{}\"\n", comp.id));
            output.push_str(&format!("        type: \"{}\"\n", comp.component_type));
            output.push_str(&category);
            output.push_str("    }\n\n");
        }
        
        output.push_str("}\n");
        Ok(())
    }
    
    fn find_component_id(&self, components: &[PlantUMLComponent], name: &str) -> String {
        components.iter()
            .find(|c| c.name == name)
            .map(|c| c.id.clone())
            .unwrap_or_else(|| name.to_string())
    }
}

pub fn import_plantuml(content: &str) -> Result<String, CompilerError> {
    let importer = PlantUMLImporter::new();
    let model = importer.import(content)?;
    
    let generator = PlantUMLArcLangGenerator::new();
    generator.generate(&model)
}
