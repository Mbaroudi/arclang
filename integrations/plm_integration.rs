use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PLMConfig {
    pub system: PLMSystem,
    pub connection: ConnectionConfig,
    pub sync_policy: SyncPolicy,
    pub mapping: MappingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PLMSystem {
    Windchill,
    Teamcenter,
    ThreeDExperience,
    SAP,
    Autodesk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub url: String,
    pub authentication: AuthenticationMethod,
    pub timeout_seconds: u64,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthenticationMethod {
    OAuth2 {
        client_id: String,
        client_secret: String,
        token_url: String,
    },
    BasicAuth {
        username: String,
        password: String,
    },
    APIKey {
        key: String,
        header: String,
    },
    SAML {
        idp_url: String,
        sp_entity_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPolicy {
    pub mode: SyncMode,
    pub frequency: SyncFrequency,
    pub conflict_resolution: ConflictResolution,
    pub auto_create_eco: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMode {
    Bidirectional,
    ArcLangToPLM,
    PLMToArcLang,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncFrequency {
    OnCommit,
    OnCompile,
    Scheduled { cron: String },
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    Manual,
    PLMWins,
    ArcLangWins,
    LastModifiedWins,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingConfig {
    pub part_mappings: Vec<PartMapping>,
    pub bom_structure: BOMStructureMapping,
    pub attribute_mappings: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartMapping {
    pub arclang_type: String,
    pub plm_type: String,
    pub attribute_map: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BOMStructureMapping {
    pub structure_type: String,
    pub quantity_field: String,
    pub reference_designator_field: String,
}

#[async_trait]
pub trait PLMConnector: Send + Sync {
    fn name(&self) -> &str;
    
    async fn connect(&mut self, config: &PLMConfig) -> Result<(), PLMError>;
    
    async fn disconnect(&mut self) -> Result<(), PLMError>;
    
    async fn fetch_baseline(&self) -> Result<PLMBaseline, PLMError>;
    
    async fn fetch_part(&self, part_number: &str) -> Result<PLMPart, PLMError>;
    
    async fn fetch_bom(&self, parent_part: &str) -> Result<BOM, PLMError>;
    
    async fn push_changes(&self, delta: &PLMDelta) -> Result<PLMSyncResult, PLMError>;
    
    async fn create_part(&self, part: &PLMPart) -> Result<String, PLMError>;
    
    async fn update_part(&self, part_id: &str, changes: &PartChanges) -> Result<(), PLMError>;
    
    async fn create_eco(&self, request: &ChangeRequest) -> Result<String, PLMError>;
    
    async fn query_parts(&self, filter: &PartFilter) -> Result<Vec<PLMPart>, PLMError>;
    
    async fn check_out(&self, part_id: &str) -> Result<(), PLMError>;
    
    async fn check_in(&self, part_id: &str, comment: &str) -> Result<(), PLMError>;
    
    async fn get_lifecycle_state(&self, part_id: &str) -> Result<LifecycleState, PLMError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PLMBaseline {
    pub timestamp: DateTime<Utc>,
    pub model_hash: String,
    pub parts: HashMap<String, PLMPart>,
    pub boms: HashMap<String, BOM>,
    pub metadata: BaselineMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineMetadata {
    pub source_system: String,
    pub version: String,
    pub created_by: String,
    pub project: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PLMPart {
    pub id: String,
    pub part_number: String,
    pub revision: String,
    pub name: String,
    pub description: Option<String>,
    pub part_type: String,
    pub lifecycle_state: LifecycleState,
    pub manufacturer: Option<String>,
    pub supplier: Option<Supplier>,
    pub unit_cost: Option<f64>,
    pub lead_time_weeks: Option<u32>,
    pub weight_kg: Option<f64>,
    pub material: Option<String>,
    pub safety_level: Option<String>,
    pub custom_attributes: HashMap<String, AttributeValue>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub created_by: String,
    pub modified_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Supplier {
    pub name: String,
    pub supplier_id: String,
    pub contact_email: String,
    pub location: String,
    pub rating: Option<f32>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LifecycleState {
    InWork,
    UnderReview,
    Released,
    Obsolete,
    Frozen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BOM {
    pub parent_part: String,
    pub structure_type: String,
    pub items: Vec<BOMItem>,
    pub effectivity: Option<Effectivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BOMItem {
    pub item_number: u32,
    pub part_number: String,
    pub quantity: f64,
    pub unit: String,
    pub reference_designator: Option<String>,
    pub find_number: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effectivity {
    pub effectivity_type: EffectivityType,
    pub start: String,
    pub end: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectivityType {
    SerialNumber,
    Date,
    Unit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PLMDelta {
    pub added_parts: Vec<PLMPart>,
    pub modified_parts: Vec<PartDiff>,
    pub deleted_parts: Vec<String>,
    pub bom_changes: Vec<BOMChange>,
    pub eco_required: bool,
    pub change_summary: String,
    pub impact_analysis: ImpactAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartDiff {
    pub part_id: String,
    pub part_number: String,
    pub changes: Vec<AttributeChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeChange {
    pub attribute: String,
    pub old_value: Option<AttributeValue>,
    pub new_value: Option<AttributeValue>,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BOMChange {
    pub parent_part: String,
    pub change_type: BOMChangeType,
    pub item: BOMItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BOMChangeType {
    ItemAdded,
    ItemRemoved,
    QuantityChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    pub affected_parts: Vec<String>,
    pub affected_assemblies: Vec<String>,
    pub cost_impact: Option<f64>,
    pub schedule_impact_days: Option<u32>,
    pub safety_impact: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PLMSyncResult {
    pub success: bool,
    pub parts_created: Vec<String>,
    pub parts_updated: Vec<String>,
    pub parts_failed: Vec<(String, String)>,
    pub eco_id: Option<String>,
    pub sync_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRequest {
    pub title: String,
    pub description: String,
    pub reason: String,
    pub affected_items: Vec<String>,
    pub requester: String,
    pub priority: Priority,
    pub change_type: ECOChangeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ECOChangeType {
    Engineering,
    Manufacturing,
    Documentation,
    Safety,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartChanges {
    pub description: Option<String>,
    pub lifecycle_state: Option<LifecycleState>,
    pub supplier: Option<Supplier>,
    pub unit_cost: Option<f64>,
    pub custom_attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartFilter {
    pub part_type: Option<String>,
    pub lifecycle_state: Option<LifecycleState>,
    pub manufacturer: Option<String>,
    pub modified_after: Option<DateTime<Utc>>,
    pub name_contains: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum PLMError {
    #[error("Connection failed: {0}")]
    ConnectionError(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
    
    #[error("Part not found: {0}")]
    PartNotFound(String),
    
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

pub struct PLMIntegrationManager {
    connectors: HashMap<PLMSystem, Box<dyn PLMConnector>>,
    config: PLMConfig,
}

impl PLMIntegrationManager {
    pub fn new(config: PLMConfig) -> Self {
        Self {
            connectors: HashMap::new(),
            config,
        }
    }
    
    pub fn register_connector(&mut self, system: PLMSystem, connector: Box<dyn PLMConnector>) {
        self.connectors.insert(system, connector);
    }
    
    pub async fn sync_to_plm(&self, delta: &PLMDelta) -> Result<PLMSyncResult, PLMError> {
        let connector = self.connectors
            .get(&self.config.system)
            .ok_or_else(|| PLMError::ConnectionError("Connector not found".to_string()))?;
        
        if delta.eco_required && self.config.sync_policy.auto_create_eco {
            let eco_request = ChangeRequest {
                title: delta.change_summary.clone(),
                description: format!("Automated sync from ArcLang\n\n{}", delta.change_summary),
                reason: "Model synchronization".to_string(),
                affected_items: delta.affected_part_numbers(),
                requester: "arclang-system".to_string(),
                priority: self.determine_priority(delta),
                change_type: ECOChangeType::Engineering,
            };
            
            connector.create_eco(&eco_request).await?;
        }
        
        connector.push_changes(delta).await
    }
    
    pub async fn sync_from_plm(&self) -> Result<PLMBaseline, PLMError> {
        let connector = self.connectors
            .get(&self.config.system)
            .ok_or_else(|| PLMError::ConnectionError("Connector not found".to_string()))?;
        
        connector.fetch_baseline().await
    }
    
    fn determine_priority(&self, delta: &PLMDelta) -> Priority {
        if delta.impact_analysis.safety_impact {
            return Priority::Critical;
        }
        
        if let Some(cost) = delta.impact_analysis.cost_impact {
            if cost > 100000.0 {
                return Priority::High;
            }
        }
        
        Priority::Medium
    }
}

impl PLMDelta {
    pub fn affected_part_numbers(&self) -> Vec<String> {
        let mut parts = Vec::new();
        
        parts.extend(self.added_parts.iter().map(|p| p.part_number.clone()));
        parts.extend(self.modified_parts.iter().map(|p| p.part_number.clone()));
        parts.extend(self.deleted_parts.clone());
        
        parts
    }
    
    pub fn requires_eco(&self) -> bool {
        !self.deleted_parts.is_empty() ||
        self.impact_analysis.safety_impact ||
        self.has_significant_cost_change() ||
        self.has_supplier_change()
    }
    
    fn has_significant_cost_change(&self) -> bool {
        for part_diff in &self.modified_parts {
            for change in &part_diff.changes {
                if change.attribute == "unit_cost" {
                    if let (Some(AttributeValue::Number(old)), Some(AttributeValue::Number(new))) = 
                        (&change.old_value, &change.new_value) {
                        let change_pct = (new - old).abs() / old;
                        if change_pct > 0.1 {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
    
    fn has_supplier_change(&self) -> bool {
        self.modified_parts.iter().any(|p| {
            p.changes.iter().any(|c| c.attribute == "supplier")
        })
    }
}

pub mod delta_computer {
    use super::*;
    use crate::compiler::semantic::SemanticModel;
    
    pub struct DeltaComputer {
        current_model: SemanticModel,
        baseline: Option<PLMBaseline>,
    }
    
    impl DeltaComputer {
        pub fn new(model: SemanticModel, baseline: Option<PLMBaseline>) -> Self {
            Self {
                current_model: model,
                baseline,
            }
        }
        
        pub fn compute_delta(&self) -> Result<PLMDelta, PLMError> {
            let mut delta = PLMDelta {
                added_parts: Vec::new(),
                modified_parts: Vec::new(),
                deleted_parts: Vec::new(),
                bom_changes: Vec::new(),
                eco_required: false,
                change_summary: String::new(),
                impact_analysis: ImpactAnalysis {
                    affected_parts: Vec::new(),
                    affected_assemblies: Vec::new(),
                    cost_impact: None,
                    schedule_impact_days: None,
                    safety_impact: false,
                },
            };
            
            if let Some(baseline) = &self.baseline {
                self.detect_added_parts(&mut delta, baseline);
                self.detect_modified_parts(&mut delta, baseline);
                self.detect_deleted_parts(&mut delta, baseline);
                self.detect_bom_changes(&mut delta, baseline);
            } else {
                self.create_initial_sync(&mut delta);
            }
            
            self.analyze_impact(&mut delta);
            delta.eco_required = delta.requires_eco();
            delta.change_summary = self.generate_summary(&delta);
            
            Ok(delta)
        }
        
        fn detect_added_parts(&self, delta: &mut PLMDelta, baseline: &PLMBaseline) {
            for node in self.current_model.physical_nodes() {
                if let Some(plm_info) = &node.plm {
                    if !baseline.parts.contains_key(&plm_info.part_number) {
                        delta.added_parts.push(self.convert_to_plm_part(node));
                    }
                }
            }
        }
        
        fn detect_modified_parts(&self, delta: &mut PLMDelta, baseline: &PLMBaseline) {
            for node in self.current_model.physical_nodes() {
                if let Some(plm_info) = &node.plm {
                    if let Some(baseline_part) = baseline.parts.get(&plm_info.part_number) {
                        if let Some(diff) = self.compute_part_diff(node, baseline_part) {
                            delta.modified_parts.push(diff);
                        }
                    }
                }
            }
        }
        
        fn detect_deleted_parts(&self, delta: &mut PLMDelta, baseline: &PLMBaseline) {
            for (part_number, _) in &baseline.parts {
                if !self.current_model.has_part(part_number) {
                    delta.deleted_parts.push(part_number.clone());
                }
            }
        }
        
        fn detect_bom_changes(&self, delta: &mut PLMDelta, baseline: &PLMBaseline) {
            for bom in self.current_model.boms() {
                if let Some(baseline_bom) = baseline.boms.get(&bom.parent_part) {
                    self.compute_bom_diff(bom, baseline_bom, delta);
                }
            }
        }
        
        fn compute_part_diff(&self, node: &NodeDecl, baseline: &PLMPart) -> Option<PartDiff> {
            let mut changes = Vec::new();
            
            if let Some(plm_info) = &node.plm {
                if let Some(new_cost) = plm_info.cost {
                    if let Some(old_cost) = baseline.unit_cost {
                        if (new_cost - old_cost).abs() > 0.01 {
                            changes.push(AttributeChange {
                                attribute: "unit_cost".to_string(),
                                old_value: Some(AttributeValue::Number(old_cost)),
                                new_value: Some(AttributeValue::Number(new_cost)),
                                change_type: ChangeType::Modified,
                            });
                        }
                    }
                }
                
                if plm_info.manufacturer != baseline.manufacturer {
                    changes.push(AttributeChange {
                        attribute: "manufacturer".to_string(),
                        old_value: baseline.manufacturer.as_ref().map(|m| AttributeValue::String(m.clone())),
                        new_value: plm_info.manufacturer.as_ref().map(|m| AttributeValue::String(m.clone())),
                        change_type: ChangeType::Modified,
                    });
                }
            }
            
            if let Some(safety) = &node.safety {
                let new_safety = format!("{:?}", safety.asil);
                if Some(&new_safety) != baseline.safety_level.as_ref() {
                    changes.push(AttributeChange {
                        attribute: "safety_level".to_string(),
                        old_value: baseline.safety_level.as_ref().map(|s| AttributeValue::String(s.clone())),
                        new_value: Some(AttributeValue::String(new_safety)),
                        change_type: ChangeType::Modified,
                    });
                }
            }
            
            if changes.is_empty() {
                None
            } else {
                Some(PartDiff {
                    part_id: baseline.id.clone(),
                    part_number: baseline.part_number.clone(),
                    changes,
                })
            }
        }
        
        fn compute_bom_diff(&self, current: &BOM, baseline: &BOM, delta: &mut PLMDelta) {
            for current_item in &current.items {
                let baseline_item = baseline.items.iter()
                    .find(|i| i.part_number == current_item.part_number);
                
                match baseline_item {
                    None => {
                        delta.bom_changes.push(BOMChange {
                            parent_part: current.parent_part.clone(),
                            change_type: BOMChangeType::ItemAdded,
                            item: current_item.clone(),
                        });
                    }
                    Some(baseline_item) if baseline_item.quantity != current_item.quantity => {
                        delta.bom_changes.push(BOMChange {
                            parent_part: current.parent_part.clone(),
                            change_type: BOMChangeType::QuantityChanged,
                            item: current_item.clone(),
                        });
                    }
                    _ => {}
                }
            }
            
            for baseline_item in &baseline.items {
                if !current.items.iter().any(|i| i.part_number == baseline_item.part_number) {
                    delta.bom_changes.push(BOMChange {
                        parent_part: current.parent_part.clone(),
                        change_type: BOMChangeType::ItemRemoved,
                        item: baseline_item.clone(),
                    });
                }
            }
        }
        
        fn create_initial_sync(&self, delta: &mut PLMDelta) {
            for node in self.current_model.physical_nodes() {
                if node.plm.is_some() {
                    delta.added_parts.push(self.convert_to_plm_part(node));
                }
            }
        }
        
        fn analyze_impact(&self, delta: &mut PLMDelta) {
            delta.impact_analysis.affected_parts = delta.affected_part_numbers();
            
            delta.impact_analysis.safety_impact = delta.modified_parts.iter().any(|p| {
                p.changes.iter().any(|c| c.attribute == "safety_level")
            }) || delta.deleted_parts.iter().any(|part_num| {
                self.baseline.as_ref()
                    .and_then(|b| b.parts.get(part_num))
                    .and_then(|p| p.safety_level.as_ref())
                    .is_some()
            });
            
            let mut total_cost_impact = 0.0;
            for part_diff in &delta.modified_parts {
                for change in &part_diff.changes {
                    if change.attribute == "unit_cost" {
                        if let (Some(AttributeValue::Number(old)), Some(AttributeValue::Number(new))) = 
                            (&change.old_value, &change.new_value) {
                            total_cost_impact += new - old;
                        }
                    }
                }
            }
            
            if total_cost_impact.abs() > 0.01 {
                delta.impact_analysis.cost_impact = Some(total_cost_impact);
            }
        }
        
        fn generate_summary(&self, delta: &PLMDelta) -> String {
            let mut summary = String::new();
            
            if !delta.added_parts.is_empty() {
                summary.push_str(&format!("Added {} new parts\n", delta.added_parts.len()));
            }
            
            if !delta.modified_parts.is_empty() {
                summary.push_str(&format!("Modified {} parts\n", delta.modified_parts.len()));
            }
            
            if !delta.deleted_parts.is_empty() {
                summary.push_str(&format!("Deleted {} parts\n", delta.deleted_parts.len()));
            }
            
            if !delta.bom_changes.is_empty() {
                summary.push_str(&format!("BOM changes: {} items\n", delta.bom_changes.len()));
            }
            
            if delta.impact_analysis.safety_impact {
                summary.push_str("⚠️  Safety-related changes detected\n");
            }
            
            if let Some(cost_impact) = delta.impact_analysis.cost_impact {
                summary.push_str(&format!("Cost impact: ${:.2}\n", cost_impact));
            }
            
            summary
        }
        
        fn convert_to_plm_part(&self, node: &NodeDecl) -> PLMPart {
            let plm_info = node.plm.as_ref().unwrap();
            
            PLMPart {
                id: String::new(),
                part_number: plm_info.part_number.clone(),
                revision: plm_info.revision.clone().unwrap_or_else(|| "A".to_string()),
                name: node.name.clone(),
                description: node.description.clone(),
                part_type: "Electronic Assembly".to_string(),
                lifecycle_state: LifecycleState::InWork,
                manufacturer: plm_info.manufacturer.clone(),
                supplier: None,
                unit_cost: plm_info.cost,
                lead_time_weeks: plm_info.lead_time,
                weight_kg: None,
                material: None,
                safety_level: node.safety.as_ref().map(|s| format!("{:?}", s.asil)),
                custom_attributes: HashMap::new(),
                created_at: Utc::now(),
                modified_at: Utc::now(),
                created_by: "arclang".to_string(),
                modified_by: "arclang".to_string(),
            }
        }
    }
}
