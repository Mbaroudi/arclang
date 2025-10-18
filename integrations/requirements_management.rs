use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RMConfig {
    pub system: RMSystem,
    pub connection: RMConnectionConfig,
    pub sync_policy: RMSyncPolicy,
    pub mapping: RMMapping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RMSystem {
    DOORS,
    DOORSNext,
    Polarion,
    Jama,
    JIRA,
    AzureDevOps,
    Codebeamer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RMConnectionConfig {
    pub server_url: String,
    pub authentication: RMAuthentication,
    pub timeout_seconds: u64,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RMAuthentication {
    OAuth2 {
        client_id: String,
        client_secret: String,
        token_url: String,
    },
    BasicAuth {
        username: String,
        password: String,
    },
    APIToken {
        token: String,
    },
    PAT {
        personal_access_token: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RMSyncPolicy {
    pub mode: RMSyncMode,
    pub frequency: RMSyncFrequency,
    pub conflict_resolution: RMConflictResolution,
    pub auto_create_links: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RMSyncMode {
    Bidirectional,
    ArcLangToRM,
    RMToArcLang,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RMSyncFrequency {
    OnCommit,
    OnCompile,
    Scheduled { cron: String },
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RMConflictResolution {
    Manual,
    RMWins,
    ArcLangWins,
    LastModifiedWins,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RMMapping {
    pub requirement_type_mappings: HashMap<String, String>,
    pub attribute_mappings: HashMap<String, String>,
    pub status_mappings: HashMap<String, String>,
    pub priority_mappings: HashMap<String, String>,
}

#[async_trait]
pub trait RequirementsConnector: Send + Sync {
    fn name(&self) -> &str;
    
    async fn connect(&mut self, config: &RMConfig) -> Result<(), RMError>;
    
    async fn disconnect(&mut self) -> Result<(), RMError>;
    
    async fn fetch_baseline(&self) -> Result<RMBaseline, RMError>;
    
    async fn fetch_requirement(&self, req_id: &str) -> Result<Requirement, RMError>;
    
    async fn fetch_module(&self, module_id: &str) -> Result<RequirementModule, RMError>;
    
    async fn create_requirement(&self, req: &Requirement) -> Result<String, RMError>;
    
    async fn update_requirement(&self, req_id: &str, changes: &RequirementChanges) -> Result<(), RMError>;
    
    async fn delete_requirement(&self, req_id: &str) -> Result<(), RMError>;
    
    async fn create_trace_link(&self, link: &TraceLink) -> Result<String, RMError>;
    
    async fn delete_trace_link(&self, link_id: &str) -> Result<(), RMError>;
    
    async fn query_requirements(&self, filter: &RequirementFilter) -> Result<Vec<Requirement>, RMError>;
    
    async fn generate_traceability_matrix(&self, from: &str, to: &str) -> Result<TraceabilityMatrix, RMError>;
    
    async fn get_coverage_report(&self) -> Result<CoverageReport, RMError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RMBaseline {
    pub timestamp: DateTime<Utc>,
    pub system: String,
    pub project: String,
    pub modules: Vec<RequirementModule>,
    pub requirements: HashMap<String, Requirement>,
    pub trace_links: Vec<TraceLink>,
    pub metadata: RMMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RMMetadata {
    pub system_version: String,
    pub baseline_name: String,
    pub created_by: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementModule {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<String>,
    pub requirements: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: String,
    pub external_id: Option<String>,
    pub title: String,
    pub text: String,
    pub requirement_type: RequirementType,
    pub status: RequirementStatus,
    pub priority: RequirementPriority,
    pub rationale: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub verification_method: Option<VerificationMethod>,
    pub verification_status: Option<VerificationStatus>,
    pub compliance: Vec<ComplianceTag>,
    pub custom_attributes: HashMap<String, AttributeValue>,
    pub parent_id: Option<String>,
    pub children_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub created_by: String,
    pub modified_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RequirementType {
    Stakeholder,
    System,
    Functional,
    NonFunctional,
    Performance,
    Safety,
    Security,
    Interface,
    Constraint,
    Regulatory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RequirementStatus {
    Draft,
    UnderReview,
    Approved,
    Rejected,
    Obsolete,
    Implemented,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RequirementPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationMethod {
    Test,
    Inspection,
    Analysis,
    Demonstration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerificationStatus {
    NotStarted,
    InProgress,
    Passed,
    Failed,
    PartiallyPassed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceTag {
    pub standard: String,
    pub section: String,
    pub level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceLink {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub link_type: TraceLinkType,
    pub rationale: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TraceLinkType {
    Satisfies,
    DerivedFrom,
    Refines,
    AllocatedTo,
    VerifiedBy,
    Traces,
    Implements,
    DependsOn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Date(DateTime<Utc>),
    List(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementChanges {
    pub title: Option<String>,
    pub text: Option<String>,
    pub status: Option<RequirementStatus>,
    pub priority: Option<RequirementPriority>,
    pub rationale: Option<String>,
    pub verification_method: Option<VerificationMethod>,
    pub custom_attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementFilter {
    pub requirement_type: Option<RequirementType>,
    pub status: Option<RequirementStatus>,
    pub priority: Option<RequirementPriority>,
    pub text_contains: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub modified_after: Option<DateTime<Utc>>,
    pub module_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceabilityMatrix {
    pub source_type: String,
    pub target_type: String,
    pub rows: Vec<MatrixRow>,
    pub columns: Vec<MatrixColumn>,
    pub cells: Vec<MatrixCell>,
    pub coverage_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixRow {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixColumn {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixCell {
    pub row_id: String,
    pub column_id: String,
    pub has_link: bool,
    pub link_type: Option<TraceLinkType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub total_requirements: usize,
    pub requirements_with_traces: usize,
    pub requirements_verified: usize,
    pub requirements_implemented: usize,
    pub coverage_by_type: HashMap<RequirementType, CoverageStats>,
    pub gaps: Vec<CoverageGap>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageStats {
    pub total: usize,
    pub with_traces: usize,
    pub verified: usize,
    pub coverage_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageGap {
    pub requirement_id: String,
    pub gap_type: GapType,
    pub severity: GapSeverity,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GapType {
    NoTraceToDesign,
    NoVerification,
    NoImplementation,
    OrphanRequirement,
    BrokenTraceLink,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GapSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RMDelta {
    pub added_requirements: Vec<Requirement>,
    pub modified_requirements: Vec<RequirementDiff>,
    pub deleted_requirements: Vec<String>,
    pub added_trace_links: Vec<TraceLink>,
    pub deleted_trace_links: Vec<String>,
    pub change_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementDiff {
    pub requirement_id: String,
    pub changes: Vec<AttributeChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeChange {
    pub attribute: String,
    pub old_value: Option<AttributeValue>,
    pub new_value: Option<AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RMSyncResult {
    pub success: bool,
    pub requirements_created: Vec<String>,
    pub requirements_updated: Vec<String>,
    pub requirements_failed: Vec<(String, String)>,
    pub trace_links_created: Vec<String>,
    pub trace_links_failed: Vec<(String, String)>,
    pub sync_timestamp: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum RMError {
    #[error("Connection failed: {0}")]
    ConnectionError(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
    
    #[error("Requirement not found: {0}")]
    RequirementNotFound(String),
    
    #[error("Module not found: {0}")]
    ModuleNotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Conflict detected: {0}")]
    ConflictError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("API error: {0}")]
    APIError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub struct RMIntegrationManager {
    connectors: HashMap<RMSystem, Box<dyn RequirementsConnector>>,
    config: RMConfig,
}

impl RMIntegrationManager {
    pub fn new(config: RMConfig) -> Self {
        Self {
            connectors: HashMap::new(),
            config,
        }
    }
    
    pub fn register_connector(&mut self, system: RMSystem, connector: Box<dyn RequirementsConnector>) {
        self.connectors.insert(system, connector);
    }
    
    pub async fn sync_to_rm(&self, delta: &RMDelta) -> Result<RMSyncResult, RMError> {
        let connector = self.connectors
            .get(&self.config.system)
            .ok_or_else(|| RMError::ConnectionError("Connector not found".to_string()))?;
        
        let mut result = RMSyncResult {
            success: true,
            requirements_created: Vec::new(),
            requirements_updated: Vec::new(),
            requirements_failed: Vec::new(),
            trace_links_created: Vec::new(),
            trace_links_failed: Vec::new(),
            sync_timestamp: Utc::now(),
        };
        
        for req in &delta.added_requirements {
            match connector.create_requirement(req).await {
                Ok(id) => result.requirements_created.push(id),
                Err(e) => {
                    result.requirements_failed.push((req.id.clone(), e.to_string()));
                    result.success = false;
                }
            }
        }
        
        for req_diff in &delta.modified_requirements {
            let changes = self.build_changes(req_diff);
            match connector.update_requirement(&req_diff.requirement_id, &changes).await {
                Ok(_) => result.requirements_updated.push(req_diff.requirement_id.clone()),
                Err(e) => {
                    result.requirements_failed.push((req_diff.requirement_id.clone(), e.to_string()));
                    result.success = false;
                }
            }
        }
        
        if self.config.sync_policy.auto_create_links {
            for link in &delta.added_trace_links {
                match connector.create_trace_link(link).await {
                    Ok(id) => result.trace_links_created.push(id),
                    Err(e) => {
                        result.trace_links_failed.push((link.id.clone(), e.to_string()));
                    }
                }
            }
        }
        
        Ok(result)
    }
    
    pub async fn sync_from_rm(&self) -> Result<RMBaseline, RMError> {
        let connector = self.connectors
            .get(&self.config.system)
            .ok_or_else(|| RMError::ConnectionError("Connector not found".to_string()))?;
        
        connector.fetch_baseline().await
    }
    
    pub async fn generate_traceability_report(&self, from: &str, to: &str) -> Result<TraceabilityMatrix, RMError> {
        let connector = self.connectors
            .get(&self.config.system)
            .ok_or_else(|| RMError::ConnectionError("Connector not found".to_string()))?;
        
        connector.generate_traceability_matrix(from, to).await
    }
    
    pub async fn get_coverage_report(&self) -> Result<CoverageReport, RMError> {
        let connector = self.connectors
            .get(&self.config.system)
            .ok_or_else(|| RMError::ConnectionError("Connector not found".to_string()))?;
        
        connector.get_coverage_report().await
    }
    
    fn build_changes(&self, diff: &RequirementDiff) -> RequirementChanges {
        let mut changes = RequirementChanges {
            title: None,
            text: None,
            status: None,
            priority: None,
            rationale: None,
            verification_method: None,
            custom_attributes: HashMap::new(),
        };
        
        for change in &diff.changes {
            match change.attribute.as_str() {
                "title" => {
                    if let Some(AttributeValue::String(s)) = &change.new_value {
                        changes.title = Some(s.clone());
                    }
                }
                "text" => {
                    if let Some(AttributeValue::String(s)) = &change.new_value {
                        changes.text = Some(s.clone());
                    }
                }
                "rationale" => {
                    if let Some(AttributeValue::String(s)) = &change.new_value {
                        changes.rationale = Some(s.clone());
                    }
                }
                _ => {
                    if let Some(val) = &change.new_value {
                        changes.custom_attributes.insert(change.attribute.clone(), val.clone());
                    }
                }
            }
        }
        
        changes
    }
}

impl RMDelta {
    pub fn is_empty(&self) -> bool {
        self.added_requirements.is_empty() &&
        self.modified_requirements.is_empty() &&
        self.deleted_requirements.is_empty() &&
        self.added_trace_links.is_empty() &&
        self.deleted_trace_links.is_empty()
    }
    
    pub fn requirement_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();
        ids.extend(self.added_requirements.iter().map(|r| r.id.clone()));
        ids.extend(self.modified_requirements.iter().map(|r| r.requirement_id.clone()));
        ids.extend(self.deleted_requirements.clone());
        ids
    }
}

pub mod delta_computer {
    use super::*;
    use crate::compiler::semantic::SemanticModel;
    
    pub struct RMDeltaComputer {
        current_model: SemanticModel,
        baseline: Option<RMBaseline>,
    }
    
    impl RMDeltaComputer {
        pub fn new(model: SemanticModel, baseline: Option<RMBaseline>) -> Self {
            Self {
                current_model: model,
                baseline,
            }
        }
        
        pub fn compute_delta(&self) -> Result<RMDelta, RMError> {
            let mut delta = RMDelta {
                added_requirements: Vec::new(),
                modified_requirements: Vec::new(),
                deleted_requirements: Vec::new(),
                added_trace_links: Vec::new(),
                deleted_trace_links: Vec::new(),
                change_summary: String::new(),
            };
            
            if let Some(baseline) = &self.baseline {
                self.detect_added_requirements(&mut delta, baseline);
                self.detect_modified_requirements(&mut delta, baseline);
                self.detect_deleted_requirements(&mut delta, baseline);
                self.detect_trace_link_changes(&mut delta, baseline);
            } else {
                self.create_initial_sync(&mut delta);
            }
            
            delta.change_summary = self.generate_summary(&delta);
            
            Ok(delta)
        }
        
        fn detect_added_requirements(&self, delta: &mut RMDelta, baseline: &RMBaseline) {
            for req in self.current_model.requirements() {
                if !baseline.requirements.contains_key(&req.id) {
                    delta.added_requirements.push(self.convert_to_rm_requirement(req));
                }
            }
        }
        
        fn detect_modified_requirements(&self, delta: &mut RMDelta, baseline: &RMBaseline) {
            for req in self.current_model.requirements() {
                if let Some(baseline_req) = baseline.requirements.get(&req.id) {
                    if let Some(diff) = self.compute_requirement_diff(req, baseline_req) {
                        delta.modified_requirements.push(diff);
                    }
                }
            }
        }
        
        fn detect_deleted_requirements(&self, delta: &mut RMDelta, baseline: &RMBaseline) {
            for (req_id, _) in &baseline.requirements {
                if !self.current_model.has_requirement(req_id) {
                    delta.deleted_requirements.push(req_id.clone());
                }
            }
        }
        
        fn detect_trace_link_changes(&self, delta: &mut RMDelta, baseline: &RMBaseline) {
            let current_links = self.extract_trace_links();
            
            for link in &current_links {
                if !baseline.trace_links.iter().any(|bl| bl.id == link.id) {
                    delta.added_trace_links.push(link.clone());
                }
            }
            
            for baseline_link in &baseline.trace_links {
                if !current_links.iter().any(|cl| cl.id == baseline_link.id) {
                    delta.deleted_trace_links.push(baseline_link.id.clone());
                }
            }
        }
        
        fn create_initial_sync(&self, delta: &mut RMDelta) {
            for req in self.current_model.requirements() {
                delta.added_requirements.push(self.convert_to_rm_requirement(req));
            }
            
            delta.added_trace_links = self.extract_trace_links();
        }
        
        fn compute_requirement_diff(&self, current: &RequirementDecl, baseline: &Requirement) 
            -> Option<RequirementDiff> {
            let mut changes = Vec::new();
            
            if current.title != baseline.title {
                changes.push(AttributeChange {
                    attribute: "title".to_string(),
                    old_value: Some(AttributeValue::String(baseline.title.clone())),
                    new_value: Some(AttributeValue::String(current.title.clone())),
                });
            }
            
            if current.text != baseline.text {
                changes.push(AttributeChange {
                    attribute: "text".to_string(),
                    old_value: Some(AttributeValue::String(baseline.text.clone())),
                    new_value: Some(AttributeValue::String(current.text.clone())),
                });
            }
            
            if let Some(rationale) = &current.rationale {
                if Some(rationale) != baseline.rationale.as_ref() {
                    changes.push(AttributeChange {
                        attribute: "rationale".to_string(),
                        old_value: baseline.rationale.as_ref().map(|r| AttributeValue::String(r.clone())),
                        new_value: Some(AttributeValue::String(rationale.clone())),
                    });
                }
            }
            
            if changes.is_empty() {
                None
            } else {
                Some(RequirementDiff {
                    requirement_id: current.id.clone(),
                    changes,
                })
            }
        }
        
        fn extract_trace_links(&self) -> Vec<TraceLink> {
            let mut links = Vec::new();
            
            for req in self.current_model.requirements() {
                for trace in &req.traces.satisfies {
                    links.push(TraceLink {
                        id: format!("{}-satisfies-{}", req.id, trace),
                        source_id: req.id.clone(),
                        target_id: trace.clone(),
                        link_type: TraceLinkType::Satisfies,
                        rationale: None,
                        created_at: Utc::now(),
                        created_by: "arclang".to_string(),
                    });
                }
                
                for trace in &req.traces.verified_by {
                    links.push(TraceLink {
                        id: format!("{}-verifiedby-{}", req.id, trace),
                        source_id: req.id.clone(),
                        target_id: trace.clone(),
                        link_type: TraceLinkType::VerifiedBy,
                        rationale: None,
                        created_at: Utc::now(),
                        created_by: "arclang".to_string(),
                    });
                }
            }
            
            links
        }
        
        fn convert_to_rm_requirement(&self, req: &RequirementDecl) -> Requirement {
            Requirement {
                id: req.id.clone(),
                external_id: req.plm_metadata.as_ref().and_then(|p| p.external_id.clone()),
                title: req.title.clone(),
                text: req.text.clone(),
                requirement_type: self.map_requirement_type(&req.req_type),
                status: self.map_requirement_status(&req.status),
                priority: self.map_priority(&req.priority),
                rationale: req.rationale.clone(),
                acceptance_criteria: None,
                verification_method: req.verification.as_ref().map(|v| self.map_verification_method(&v.method)),
                verification_status: None,
                compliance: req.compliance.iter().map(|c| ComplianceTag {
                    standard: c.standard.clone(),
                    section: c.section.clone(),
                    level: c.level.clone(),
                }).collect(),
                custom_attributes: HashMap::new(),
                parent_id: None,
                children_ids: Vec::new(),
                created_at: Utc::now(),
                modified_at: Utc::now(),
                created_by: "arclang".to_string(),
                modified_by: "arclang".to_string(),
            }
        }
        
        fn map_requirement_type(&self, req_type: &str) -> RequirementType {
            match req_type.to_lowercase().as_str() {
                "functional" => RequirementType::Functional,
                "performance" => RequirementType::Performance,
                "safety" => RequirementType::Safety,
                "security" => RequirementType::Security,
                "interface" => RequirementType::Interface,
                _ => RequirementType::System,
            }
        }
        
        fn map_requirement_status(&self, status: &str) -> RequirementStatus {
            match status.to_lowercase().as_str() {
                "draft" => RequirementStatus::Draft,
                "approved" => RequirementStatus::Approved,
                "rejected" => RequirementStatus::Rejected,
                "obsolete" => RequirementStatus::Obsolete,
                _ => RequirementStatus::UnderReview,
            }
        }
        
        fn map_priority(&self, priority: &str) -> RequirementPriority {
            match priority.to_lowercase().as_str() {
                "critical" => RequirementPriority::Critical,
                "high" => RequirementPriority::High,
                "low" => RequirementPriority::Low,
                _ => RequirementPriority::Medium,
            }
        }
        
        fn map_verification_method(&self, method: &str) -> VerificationMethod {
            match method.to_lowercase().as_str() {
                "test" => VerificationMethod::Test,
                "inspection" => VerificationMethod::Inspection,
                "analysis" => VerificationMethod::Analysis,
                _ => VerificationMethod::Demonstration,
            }
        }
        
        fn generate_summary(&self, delta: &RMDelta) -> String {
            let mut summary = String::new();
            
            if !delta.added_requirements.is_empty() {
                summary.push_str(&format!("Added {} requirements\n", delta.added_requirements.len()));
            }
            
            if !delta.modified_requirements.is_empty() {
                summary.push_str(&format!("Modified {} requirements\n", delta.modified_requirements.len()));
            }
            
            if !delta.deleted_requirements.is_empty() {
                summary.push_str(&format!("Deleted {} requirements\n", delta.deleted_requirements.len()));
            }
            
            if !delta.added_trace_links.is_empty() {
                summary.push_str(&format!("Added {} trace links\n", delta.added_trace_links.len()));
            }
            
            summary
        }
    }
}
