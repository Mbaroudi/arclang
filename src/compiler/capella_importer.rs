use super::ast::*;
use super::CompilerError;
use std::collections::HashMap;
use std::path::Path;

pub struct CapellaImporter;

impl CapellaImporter {
    pub fn new() -> Self {
        Self
    }
    
    pub fn import_file<P: AsRef<Path>>(&self, path: P) -> Result<Model, CompilerError> {
        let xml_content = std::fs::read_to_string(path)?;
        self.import_string(&xml_content)
    }
    
    pub fn import_string(&self, xml_content: &str) -> Result<Model, CompilerError> {
        use quick_xml::Reader;
        use quick_xml::events::Event;
        
        let mut reader = Reader::from_str(xml_content);
        reader.trim_text(true);
        
        let mut requirements_map = HashMap::new();
        let mut components_map = HashMap::new();
        let mut traces = Vec::new();
        
        let mut buf = Vec::new();
        let mut current_section = Section::None;
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    match e.name().as_ref() {
                        b"ownedRequirements" => {
                            current_section = Section::Requirements;
                        }
                        b"ownedLogicalComponents" => {
                            current_section = Section::Components;
                        }
                        b"ownedTraces" => {
                            current_section = Section::Traces;
                        }
                        b"requirement" => {
                            if let Section::Requirements = current_section {
                                if let Some(req) = self.parse_requirement(e.attributes())? {
                                    requirements_map.insert(req.id.clone(), req);
                                }
                            }
                        }
                        b"component" => {
                            if let Section::Components = current_section {
                                if let Some(comp) = self.parse_component(e.attributes())? {
                                    components_map.insert(comp.name.clone(), comp);
                                }
                            }
                        }
                        b"trace" => {
                            if let Section::Traces = current_section {
                                if let Some(trace) = self.parse_trace(e.attributes())? {
                                    traces.push(trace);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(CompilerError::Semantic(format!("XML parse error: {}", e))),
                _ => {}
            }
            buf.clear();
        }
        
        let system_analysis = if !requirements_map.is_empty() {
            let mut requirements = Vec::new();
            for (_, req) in requirements_map {
                requirements.push(req);
            }
            
            vec![SystemAnalysis {
                name: "Imported System".to_string(),
                requirements,
                functions: Vec::new(),
                components: Vec::new(),
            }]
        } else {
            Vec::new()
        };
        
        let logical_architecture = if !components_map.is_empty() {
            let mut components = Vec::new();
            for (_, comp) in components_map {
                components.push(comp);
            }
            
            vec![LogicalArchitecture {
                name: "Imported Architecture".to_string(),
                components,
                interfaces: Vec::new(),
            }]
        } else {
            Vec::new()
        };
        
        Ok(Model {
            operational_analysis: Vec::new(),
            system_analysis,
            logical_architecture,
            physical_architecture: Vec::new(),
            epbs: Vec::new(),
            safety_analysis: Vec::new(),
            traces,
        })
    }
    
    fn parse_requirement(&self, attrs: quick_xml::events::attributes::Attributes) 
        -> Result<Option<Requirement>, CompilerError> {
        let mut id = String::new();
        let mut description = String::new();
        let mut priority = String::new();
        let mut safety_level = None;
        
        for attr in attrs {
            let attr = attr.map_err(|e| CompilerError::Semantic(format!("Attribute error: {}", e)))?;
            let key = std::str::from_utf8(attr.key.as_ref())
                .map_err(|e| CompilerError::Semantic(format!("UTF-8 error: {}", e)))?;
            let value = attr.unescape_value()
                .map_err(|e| CompilerError::Semantic(format!("XML unescape error: {}", e)))?;
            
            match key {
                "id" => id = value.to_string(),
                "description" => description = value.to_string(),
                "priority" => priority = value.to_string(),
                "safety_level" | "safetyLevel" => safety_level = Some(value.to_string()),
                _ => {}
            }
        }
        
        if id.is_empty() {
            return Ok(None);
        }
        
        let mut attributes = HashMap::new();
        attributes.insert("description".to_string(), AttributeValue::String(description));
        attributes.insert("priority".to_string(), AttributeValue::String(priority));
        if let Some(level) = safety_level {
            attributes.insert("safety_level".to_string(), AttributeValue::String(level));
        }
        
        Ok(Some(Requirement {
            id,
            attributes,
        }))
    }
    
    fn parse_component(&self, attrs: quick_xml::events::attributes::Attributes) 
        -> Result<Option<LogicalComponent>, CompilerError> {
        let mut id = String::new();
        let mut name = String::new();
        let mut comp_type = String::new();
        
        for attr in attrs {
            let attr = attr.map_err(|e| CompilerError::Semantic(format!("Attribute error: {}", e)))?;
            let key = std::str::from_utf8(attr.key.as_ref())
                .map_err(|e| CompilerError::Semantic(format!("UTF-8 error: {}", e)))?;
            let value = attr.unescape_value()
                .map_err(|e| CompilerError::Semantic(format!("XML unescape error: {}", e)))?;
            
            match key {
                "id" => id = value.to_string(),
                "name" => name = value.to_string(),
                "type" => comp_type = value.to_string(),
                _ => {}
            }
        }
        
        if name.is_empty() {
            name = id.clone();
        }
        
        if name.is_empty() {
            return Ok(None);
        }
        
        let mut attributes = HashMap::new();
        if !id.is_empty() {
            attributes.insert("id".to_string(), AttributeValue::String(id));
        }
        if !comp_type.is_empty() {
            attributes.insert("type".to_string(), AttributeValue::String(comp_type));
        }
        
        Ok(Some(LogicalComponent {
            name,
            functions: Vec::new(),
            attributes,
        }))
    }
    
    fn parse_trace(&self, attrs: quick_xml::events::attributes::Attributes) 
        -> Result<Option<Trace>, CompilerError> {
        let mut from = String::new();
        let mut to = String::new();
        let mut trace_type = String::new();
        let mut rationale = None;
        
        for attr in attrs {
            let attr = attr.map_err(|e| CompilerError::Semantic(format!("Attribute error: {}", e)))?;
            let key = std::str::from_utf8(attr.key.as_ref())
                .map_err(|e| CompilerError::Semantic(format!("UTF-8 error: {}", e)))?;
            let value = attr.unescape_value()
                .map_err(|e| CompilerError::Semantic(format!("XML unescape error: {}", e)))?;
            
            match key {
                "from" => from = value.to_string(),
                "to" => to = value.to_string(),
                "type" => trace_type = value.to_string(),
                "rationale" => rationale = Some(value.to_string()),
                _ => {}
            }
        }
        
        if from.is_empty() || to.is_empty() {
            return Ok(None);
        }
        
        let mut attributes = HashMap::new();
        if let Some(r) = rationale {
            attributes.insert("rationale".to_string(), AttributeValue::String(r));
        }
        
        Ok(Some(Trace {
            from,
            to,
            trace_type,
            attributes,
        }))
    }
}

#[derive(Debug)]
enum Section {
    None,
    Requirements,
    Components,
    Traces,
}

pub struct ArcCodeGenerator;

impl ArcCodeGenerator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn generate(&self, model: &Model) -> Result<String, CompilerError> {
        let mut arc_code = String::new();
        
        // Generate System Analysis
        for sa in &model.system_analysis {
            arc_code.push_str(&format!("system_analysis \"{}\" {{\n", sa.name));
            
            for req in &sa.requirements {
                arc_code.push_str(&format!("    requirement \"{}\" {{\n", req.id));
                
                for (key, value) in &req.attributes {
                    match value {
                        AttributeValue::String(s) => {
                            arc_code.push_str(&format!("        {}: \"{}\"\n", key, s));
                        }
                        AttributeValue::Number(n) => {
                            arc_code.push_str(&format!("        {}: {}\n", key, n));
                        }
                        AttributeValue::List(items) => {
                            arc_code.push_str(&format!("        {}: [", key));
                            for (i, item) in items.iter().enumerate() {
                                if i > 0 {
                                    arc_code.push_str(", ");
                                }
                                if let AttributeValue::String(s) = item {
                                    arc_code.push_str(&format!("\"{}\"", s));
                                }
                            }
                            arc_code.push_str("]\n");
                        }
                        AttributeValue::Boolean(b) => {
                            arc_code.push_str(&format!("        {}: {}\n", key, b));
                        }
                    }
                }
                
                arc_code.push_str("    }\n\n");
            }
            
            arc_code.push_str("}\n\n");
        }
        
        // Generate Logical Architecture
        for la in &model.logical_architecture {
            arc_code.push_str(&format!("logical_architecture \"{}\" {{\n", la.name));
            
            for comp in &la.components {
                arc_code.push_str(&format!("    component \"{}\" {{\n", comp.name));
                
                for (key, value) in &comp.attributes {
                    match value {
                        AttributeValue::String(s) => {
                            arc_code.push_str(&format!("        {}: \"{}\"\n", key, s));
                        }
                        _ => {}
                    }
                }
                
                for func in &comp.functions {
                    arc_code.push_str(&format!("\n        function \"{}\" {{\n", func.name));
                    
                    for (key, value) in &func.attributes {
                        match value {
                            AttributeValue::String(s) => {
                                arc_code.push_str(&format!("            {}: \"{}\"\n", key, s));
                            }
                            AttributeValue::List(items) => {
                                arc_code.push_str(&format!("            {}: [", key));
                                for (i, item) in items.iter().enumerate() {
                                    if i > 0 {
                                        arc_code.push_str(", ");
                                    }
                                    if let AttributeValue::String(s) = item {
                                        arc_code.push_str(&format!("\"{}\"", s));
                                    }
                                }
                                arc_code.push_str("]\n");
                            }
                            AttributeValue::Number(n) => {
                                arc_code.push_str(&format!("            {}: {}\n", key, n));
                            }
                            AttributeValue::Boolean(b) => {
                                arc_code.push_str(&format!("            {}: {}\n", key, b));
                            }
                        }
                    }
                    
                    arc_code.push_str("        }\n");
                }
                
                arc_code.push_str("    }\n\n");
            }
            
            arc_code.push_str("}\n\n");
        }
        
        // Generate Traces
        for trace in &model.traces {
            arc_code.push_str(&format!(
                "trace \"{}\" {} \"{}\" {{\n",
                trace.from, trace.trace_type, trace.to
            ));
            
            for (key, value) in &trace.attributes {
                if let AttributeValue::String(s) = value {
                    arc_code.push_str(&format!("    {}: \"{}\"\n", key, s));
                }
            }
            
            arc_code.push_str("}\n\n");
        }
        
        Ok(arc_code)
    }
}
