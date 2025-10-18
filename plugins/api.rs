use super::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SemanticModel {
    pub requirements: Vec<Requirement>,
    pub components: Vec<Component>,
    pub functions: Vec<Function>,
    pub interfaces: Vec<Interface>,
    pub traces: Vec<TraceLink>,
    pub metadata: ModelMetadata,
}

impl SemanticModel {
    pub fn new() -> Self {
        Self {
            requirements: Vec::new(),
            components: Vec::new(),
            functions: Vec::new(),
            interfaces: Vec::new(),
            traces: Vec::new(),
            metadata: ModelMetadata::default(),
        }
    }
    
    pub fn get_requirement(&self, id: &str) -> Option<&Requirement> {
        self.requirements.iter().find(|r| r.id == id)
    }
    
    pub fn get_component(&self, id: &str) -> Option<&Component> {
        self.components.iter().find(|c| c.id == id)
    }
    
    pub fn get_function(&self, id: &str) -> Option<&Function> {
        self.functions.iter().find(|f| f.id == id)
    }
    
    pub fn get_interface(&self, id: &str) -> Option<&Interface> {
        self.interfaces.iter().find(|i| i.id == id)
    }
    
    pub fn get_traces_from(&self, element_id: &str) -> Vec<&TraceLink> {
        self.traces.iter()
            .filter(|t| t.from == element_id)
            .collect()
    }
    
    pub fn get_traces_to(&self, element_id: &str) -> Vec<&TraceLink> {
        self.traces.iter()
            .filter(|t| t.to == element_id)
            .collect()
    }
    
    pub fn query_by_type(&self, element_type: ElementType) -> Vec<ModelElement> {
        let mut results = Vec::new();
        
        match element_type {
            ElementType::Requirement => {
                results.extend(self.requirements.iter().map(|r| ModelElement::Requirement(r.clone())));
            }
            ElementType::Component => {
                results.extend(self.components.iter().map(|c| ModelElement::Component(c.clone())));
            }
            ElementType::Function => {
                results.extend(self.functions.iter().map(|f| ModelElement::Function(f.clone())));
            }
            ElementType::Interface => {
                results.extend(self.interfaces.iter().map(|i| ModelElement::Interface(i.clone())));
            }
        }
        
        results
    }
    
    pub fn query_by_level(&self, level: ArcadiaLevel) -> Vec<ModelElement> {
        let mut results = Vec::new();
        
        for req in &self.requirements {
            if req.level == level {
                results.push(ModelElement::Requirement(req.clone()));
            }
        }
        
        for comp in &self.components {
            if comp.level == level {
                results.push(ModelElement::Component(comp.clone()));
            }
        }
        
        for func in &self.functions {
            if func.level == level {
                results.push(ModelElement::Function(func.clone()));
            }
        }
        
        results
    }
    
    pub fn validate_traceability(&self) -> Vec<TraceabilityIssue> {
        let mut issues = Vec::new();
        
        for req in &self.requirements {
            if self.get_traces_from(&req.id).is_empty() {
                issues.push(TraceabilityIssue {
                    element_id: req.id.clone(),
                    issue_type: IssueType::MissingDownstreamTrace,
                    description: format!("Requirement {} has no downstream traces", req.id),
                });
            }
        }
        
        for comp in &self.components {
            if self.get_traces_to(&comp.id).is_empty() {
                issues.push(TraceabilityIssue {
                    element_id: comp.id.clone(),
                    issue_type: IssueType::MissingUpstreamTrace,
                    description: format!("Component {} has no upstream traces", comp.id),
                });
            }
        }
        
        issues
    }
    
    pub fn compute_metrics(&self) -> ModelMetrics {
        let total_elements = self.requirements.len() 
            + self.components.len() 
            + self.functions.len() 
            + self.interfaces.len();
        
        let traceability_coverage = if !self.requirements.is_empty() {
            let traced_reqs = self.requirements.iter()
                .filter(|r| !self.get_traces_from(&r.id).is_empty())
                .count();
            (traced_reqs as f64 / self.requirements.len() as f64) * 100.0
        } else {
            0.0
        };
        
        ModelMetrics {
            total_elements,
            requirements_count: self.requirements.len(),
            components_count: self.components.len(),
            functions_count: self.functions.len(),
            interfaces_count: self.interfaces.len(),
            traces_count: self.traces.len(),
            traceability_coverage,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Requirement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub level: ArcadiaLevel,
    pub priority: RequirementPriority,
    pub status: RequirementStatus,
    pub safety_level: Option<SafetyLevel>,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Component {
    pub id: String,
    pub name: String,
    pub description: String,
    pub level: ArcadiaLevel,
    pub component_type: ComponentType,
    pub properties: HashMap<String, String>,
    pub ports: Vec<Port>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub id: String,
    pub name: String,
    pub description: String,
    pub level: ArcadiaLevel,
    pub inputs: Vec<FunctionParameter>,
    pub outputs: Vec<FunctionParameter>,
    pub behavior: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Interface {
    pub id: String,
    pub name: String,
    pub interface_type: InterfaceType,
    pub source_component: String,
    pub target_component: String,
    pub exchanges: Vec<Exchange>,
}

#[derive(Debug, Clone)]
pub struct TraceLink {
    pub from: String,
    pub to: String,
    pub trace_type: TraceType,
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ModelMetadata {
    pub version: String,
    pub project_name: String,
    pub author: String,
    pub created_at: String,
    pub modified_at: String,
}

#[derive(Debug, Clone)]
pub enum ModelElement {
    Requirement(Requirement),
    Component(Component),
    Function(Function),
    Interface(Interface),
}

impl ModelElement {
    pub fn id(&self) -> &str {
        match self {
            ModelElement::Requirement(r) => &r.id,
            ModelElement::Component(c) => &c.id,
            ModelElement::Function(f) => &f.id,
            ModelElement::Interface(i) => &i.id,
        }
    }
    
    pub fn name(&self) -> &str {
        match self {
            ModelElement::Requirement(r) => &r.name,
            ModelElement::Component(c) => &c.name,
            ModelElement::Function(f) => &f.name,
            ModelElement::Interface(i) => &i.name,
        }
    }
    
    pub fn element_type(&self) -> ElementType {
        match self {
            ModelElement::Requirement(_) => ElementType::Requirement,
            ModelElement::Component(_) => ElementType::Component,
            ModelElement::Function(_) => ElementType::Function,
            ModelElement::Interface(_) => ElementType::Interface,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ElementType {
    Requirement,
    Component,
    Function,
    Interface,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArcadiaLevel {
    OperationalAnalysis,
    SystemAnalysis,
    LogicalArchitecture,
    PhysicalArchitecture,
    EPBS,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RequirementPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RequirementStatus {
    Draft,
    UnderReview,
    Approved,
    Implemented,
    Verified,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SafetyLevel {
    ASIL_A,
    ASIL_B,
    ASIL_C,
    ASIL_D,
    DAL_A,
    DAL_B,
    DAL_C,
    DAL_D,
    DAL_E,
    SIL_1,
    SIL_2,
    SIL_3,
    SIL_4,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComponentType {
    Logical,
    Physical,
    Behavioral,
    Node,
}

#[derive(Debug, Clone)]
pub struct Port {
    pub id: String,
    pub name: String,
    pub direction: PortDirection,
    pub data_type: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortDirection {
    In,
    Out,
    InOut,
}

#[derive(Debug, Clone)]
pub struct FunctionParameter {
    pub name: String,
    pub data_type: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterfaceType {
    Data,
    Control,
    Physical,
}

#[derive(Debug, Clone)]
pub struct Exchange {
    pub id: String,
    pub name: String,
    pub exchange_type: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TraceType {
    Satisfies,
    Refines,
    DerivedFrom,
    Implements,
    Verifies,
    Allocates,
}

#[derive(Debug, Clone)]
pub struct TraceabilityIssue {
    pub element_id: String,
    pub issue_type: IssueType,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IssueType {
    MissingUpstreamTrace,
    MissingDownstreamTrace,
    BrokenTrace,
    InvalidTrace,
}

#[derive(Debug, Clone)]
pub struct ModelMetrics {
    pub total_elements: usize,
    pub requirements_count: usize,
    pub components_count: usize,
    pub functions_count: usize,
    pub interfaces_count: usize,
    pub traces_count: usize,
    pub traceability_coverage: f64,
}

pub struct QueryBuilder {
    filters: Vec<QueryFilter>,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }
    
    pub fn with_type(mut self, element_type: ElementType) -> Self {
        self.filters.push(QueryFilter::Type(element_type));
        self
    }
    
    pub fn with_level(mut self, level: ArcadiaLevel) -> Self {
        self.filters.push(QueryFilter::Level(level));
        self
    }
    
    pub fn with_name_contains(mut self, pattern: String) -> Self {
        self.filters.push(QueryFilter::NameContains(pattern));
        self
    }
    
    pub fn with_safety_level(mut self, safety_level: SafetyLevel) -> Self {
        self.filters.push(QueryFilter::SafetyLevel(safety_level));
        self
    }
    
    pub fn execute(&self, model: &SemanticModel) -> Vec<ModelElement> {
        let mut results = Vec::new();
        
        results.extend(model.requirements.iter().map(|r| ModelElement::Requirement(r.clone())));
        results.extend(model.components.iter().map(|c| ModelElement::Component(c.clone())));
        results.extend(model.functions.iter().map(|f| ModelElement::Function(f.clone())));
        results.extend(model.interfaces.iter().map(|i| ModelElement::Interface(i.clone())));
        
        for filter in &self.filters {
            results = self.apply_filter(results, filter);
        }
        
        results
    }
    
    fn apply_filter(&self, elements: Vec<ModelElement>, filter: &QueryFilter) -> Vec<ModelElement> {
        match filter {
            QueryFilter::Type(element_type) => {
                elements.into_iter()
                    .filter(|e| e.element_type() == *element_type)
                    .collect()
            }
            QueryFilter::Level(level) => {
                elements.into_iter()
                    .filter(|e| {
                        match e {
                            ModelElement::Requirement(r) => r.level == *level,
                            ModelElement::Component(c) => c.level == *level,
                            ModelElement::Function(f) => f.level == *level,
                            _ => false,
                        }
                    })
                    .collect()
            }
            QueryFilter::NameContains(pattern) => {
                elements.into_iter()
                    .filter(|e| e.name().contains(pattern))
                    .collect()
            }
            QueryFilter::SafetyLevel(safety_level) => {
                elements.into_iter()
                    .filter(|e| {
                        match e {
                            ModelElement::Requirement(r) => {
                                r.safety_level.as_ref() == Some(safety_level)
                            }
                            _ => false,
                        }
                    })
                    .collect()
            }
        }
    }
}

#[derive(Debug, Clone)]
enum QueryFilter {
    Type(ElementType),
    Level(ArcadiaLevel),
    NameContains(String),
    SafetyLevel(SafetyLevel),
}

pub struct ModelBuilder {
    model: SemanticModel,
}

impl ModelBuilder {
    pub fn new() -> Self {
        Self {
            model: SemanticModel::new(),
        }
    }
    
    pub fn add_requirement(mut self, requirement: Requirement) -> Self {
        self.model.requirements.push(requirement);
        self
    }
    
    pub fn add_component(mut self, component: Component) -> Self {
        self.model.components.push(component);
        self
    }
    
    pub fn add_function(mut self, function: Function) -> Self {
        self.model.functions.push(function);
        self
    }
    
    pub fn add_interface(mut self, interface: Interface) -> Self {
        self.model.interfaces.push(interface);
        self
    }
    
    pub fn add_trace(mut self, trace: TraceLink) -> Self {
        self.model.traces.push(trace);
        self
    }
    
    pub fn with_metadata(mut self, metadata: ModelMetadata) -> Self {
        self.model.metadata = metadata;
        self
    }
    
    pub fn build(self) -> SemanticModel {
        self.model
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_creation() {
        let model = ModelBuilder::new()
            .add_requirement(Requirement {
                id: "REQ-001".to_string(),
                name: "Test Requirement".to_string(),
                description: "A test requirement".to_string(),
                level: ArcadiaLevel::SystemAnalysis,
                priority: RequirementPriority::High,
                status: RequirementStatus::Approved,
                safety_level: Some(SafetyLevel::ASIL_B),
                attributes: HashMap::new(),
            })
            .build();
        
        assert_eq!(model.requirements.len(), 1);
        assert!(model.get_requirement("REQ-001").is_some());
    }
    
    #[test]
    fn test_query_builder() {
        let model = ModelBuilder::new()
            .add_requirement(Requirement {
                id: "REQ-001".to_string(),
                name: "Safety Requirement".to_string(),
                description: "Test".to_string(),
                level: ArcadiaLevel::SystemAnalysis,
                priority: RequirementPriority::Critical,
                status: RequirementStatus::Approved,
                safety_level: Some(SafetyLevel::ASIL_D),
                attributes: HashMap::new(),
            })
            .build();
        
        let results = QueryBuilder::new()
            .with_type(ElementType::Requirement)
            .with_level(ArcadiaLevel::SystemAnalysis)
            .execute(&model);
        
        assert_eq!(results.len(), 1);
    }
    
    #[test]
    fn test_traceability_validation() {
        let model = ModelBuilder::new()
            .add_requirement(Requirement {
                id: "REQ-001".to_string(),
                name: "Test".to_string(),
                description: "Test".to_string(),
                level: ArcadiaLevel::SystemAnalysis,
                priority: RequirementPriority::High,
                status: RequirementStatus::Approved,
                safety_level: None,
                attributes: HashMap::new(),
            })
            .build();
        
        let issues = model.validate_traceability();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].issue_type, IssueType::MissingDownstreamTrace);
    }
    
    #[test]
    fn test_metrics_computation() {
        let model = ModelBuilder::new()
            .add_requirement(Requirement {
                id: "REQ-001".to_string(),
                name: "Test".to_string(),
                description: "Test".to_string(),
                level: ArcadiaLevel::SystemAnalysis,
                priority: RequirementPriority::High,
                status: RequirementStatus::Approved,
                safety_level: None,
                attributes: HashMap::new(),
            })
            .add_component(Component {
                id: "COMP-001".to_string(),
                name: "Test Component".to_string(),
                description: "Test".to_string(),
                level: ArcadiaLevel::LogicalArchitecture,
                component_type: ComponentType::Logical,
                properties: HashMap::new(),
                ports: Vec::new(),
            })
            .add_trace(TraceLink {
                from: "REQ-001".to_string(),
                to: "COMP-001".to_string(),
                trace_type: TraceType::Satisfies,
                rationale: None,
            })
            .build();
        
        let metrics = model.compute_metrics();
        assert_eq!(metrics.total_elements, 2);
        assert_eq!(metrics.traces_count, 1);
        assert_eq!(metrics.traceability_coverage, 100.0);
    }
}
