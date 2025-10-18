use super::semantic::SemanticModel;
use super::CompilerError;
use std::collections::{HashMap, HashSet};

pub struct MermaidGenerator {
    layout: String,
}

impl MermaidGenerator {
    pub fn new() -> Self {
        Self {
            layout: "elk".to_string(),
        }
    }
    
    pub fn with_layout(mut self, layout: String) -> Self {
        self.layout = layout;
        self
    }
    
    pub fn generate(&self, model: &SemanticModel, title: &str) -> Result<String, CompilerError> {
        let mut mermaid = String::new();
        
        // Header
        mermaid.push_str("---\n");
        mermaid.push_str(&format!("config:\n  layout: {}\n", self.layout));
        mermaid.push_str(&format!("title: {}\n", title));
        mermaid.push_str("---\n");
        mermaid.push_str("flowchart TD\n");
        
        // Group requirements by category
        let categories = self.group_by_category(model);
        
        // Generate subgraphs for each category
        for (i, (category, reqs)) in categories.iter().enumerate() {
            mermaid.push_str(&format!(" subgraph subGraph{}[\"{}\"]", i, category));
            mermaid.push_str("\n");
            
            for req in reqs {
                let node_id = &req.id;
                let name = req.id.clone();
                let desc = req.description.replace('\n', "<br>");
                
                mermaid.push_str(&format!(
                    "        {}[\"{}\"]\n",
                    node_id,
                    format!("{}<br>{}", name, desc)
                ));
            }
            
            mermaid.push_str("  end\n");
        }
        
        // Generate trace relationships
        mermaid.push_str(&self.generate_relationships(model));
        
        // Generate styling
        mermaid.push_str(&self.generate_styles(&categories));
        
        Ok(mermaid)
    }
    
    fn group_by_category<'a>(&self, model: &'a SemanticModel) -> Vec<(String, Vec<&'a super::semantic::RequirementInfo>)> {
        let mut categories: HashMap<String, Vec<&super::semantic::RequirementInfo>> = HashMap::new();
        
        for req in &model.requirements {
            let category = req.category.clone().unwrap_or_else(|| self.extract_category(&req.id));
            categories.entry(category).or_insert_with(Vec::new).push(req);
        }
        
        // Sort categories
        let mut sorted_categories: Vec<_> = categories.into_iter().collect();
        sorted_categories.sort_by(|a, b| a.0.cmp(&b.0));
        
        sorted_categories
    }
    
    fn extract_category(&self, id: &str) -> String {
        // Automotive/Aerospace patterns
        if id.contains("SYS-") || id.starts_with("SYS") {
            "System Requirements".to_string()
        } else if id.contains("SW-") || id.starts_with("SW") {
            "Software Requirements".to_string()
        } else if id.contains("HW-") || id.starts_with("HW") {
            "Hardware Requirements".to_string()
        } else if id.contains("SAFE-") || id.starts_with("SAFE") {
            "Safety Requirements".to_string()
        }
        // Analytics patterns (business)
        else if id.starts_with("ANLX00") && id.ends_with("1") || id.ends_with("2") {
            "Executive Analytics".to_string()
        } else if id.starts_with("ANLX00") && id.ends_with("3") || id.ends_with("4") {
            "Market Analytics".to_string()
        } else if id.starts_with("ANLX00") && id.ends_with("5") || id.ends_with("6") {
            "Merchant Analytics".to_string()
        } else if id.starts_with("ANLX00") && id.ends_with("7") || id.ends_with("8") {
            "Client Analytics".to_string()
        } else if id.starts_with("ANLX00") && id.ends_with("9") || id.starts_with("ANLX010") {
            "Financial Analytics".to_string()
        } else if id.starts_with("ANLX011") || id.starts_with("ANLX012") {
            "Compliance Analytics".to_string()
        } else if id.starts_with("ANLX013") || id.starts_with("ANLX014") {
            "Data Quality Analytics".to_string()
        } else if id.starts_with("ADV") {
            "Advanced Analytics".to_string()
        } else if id.starts_with("ARCH") || id.starts_with("GOV") {
            "Implementation".to_string()
        } else if id.starts_with("BR-") {
            "Business Requirements".to_string()
        } else {
            "Requirements".to_string()
        }
    }
    
    fn generate_relationships(&self, model: &SemanticModel) -> String {
        let mut relationships = String::new();
        let mut added: HashSet<(String, String)> = HashSet::new();
        
        for trace in &model.traces {
            if trace.trace_type == "implements" {
                let key = (trace.from.clone(), trace.to.clone());
                if !added.contains(&key) {
                    relationships.push_str(&format!(
                        "    {} --> {}\n",
                        trace.from, trace.to
                    ));
                    added.insert(key);
                }
            }
        }
        
        relationships
    }
    
    fn generate_styles(&self, categories: &[(String, Vec<&super::semantic::RequirementInfo>)]) -> String {
        let mut styles = String::new();
        
        let color_map = [
            // Business/Analytics colors
            ("Executive Analytics", "#1F77B4", "executive"),
            ("Market Analytics", "#FF7F0E", "market"),
            ("Merchant Analytics", "#2CA02C", "merchant"),
            ("Client Analytics", "#D62728", "client"),
            ("Financial Analytics", "#9467BD", "financial"),
            ("Compliance Analytics", "#8C564B", "compliance"),
            ("Data Quality Analytics", "#E377C2", "quality"),
            ("Advanced Analytics", "#7F7F7F", "advanced"),
            ("Implementation", "#BCBD22", "architecture"),
            ("Business Requirements", "#17BECF", "business"),
            // Automotive/Aerospace colors
            ("System Requirements", "#1F77B4", "system"),
            ("Software Requirements", "#FF7F0E", "software"),
            ("Hardware Requirements", "#2CA02C", "hardware"),
            ("Safety Requirements", "#D62728", "safety"),
            ("Functional Safety", "#9467BD", "functional_safety"),
            ("Performance", "#FF7F0E", "performance"),
            ("Safety Override", "#D62728", "safety_override"),
            ("Operational Range", "#8C564B", "operational"),
            ("Requirements", "#7F7F7F", "requirements"),
        ];
        
        // Apply styles to nodes
        for (category, reqs) in categories {
            if let Some((_, _, class_name)) = color_map.iter().find(|(cat, _, _)| cat == category) {
                for req in reqs {
                    styles.push_str(&format!("     {}:::{}\n", req.id, class_name));
                }
            }
        }
        
        // Define classes
        for (_, color, class_name) in &color_map {
            styles.push_str(&format!(
                "    classDef {} fill:{},color:white\n",
                class_name, color
            ));
        }
        
        styles
    }
}

// Convenience function
pub fn generate_mermaid_flowchart(
    model: &SemanticModel,
    title: &str,
    layout: &str,
) -> Result<String, CompilerError> {
    MermaidGenerator::new()
        .with_layout(layout.to_string())
        .generate(model, title)
}
