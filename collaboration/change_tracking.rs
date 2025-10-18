use super::*;
use std::collections::HashMap;

pub struct ChangeTracker {
    pending_changes: Vec<ModelChange>,
    change_history: Vec<ChangeSet>,
}

impl ChangeTracker {
    pub fn new() -> Self {
        Self {
            pending_changes: Vec::new(),
            change_history: Vec::new(),
        }
    }
    
    pub fn track_change(&mut self, change: ModelChange) {
        self.pending_changes.push(change);
    }
    
    pub fn get_pending_changes(&self) -> Result<Vec<ModelChange>, CollaborationError> {
        Ok(self.pending_changes.clone())
    }
    
    pub fn clear_pending_changes(&mut self) {
        self.pending_changes.clear();
    }
    
    pub fn add_to_history(&mut self, changeset: ChangeSet) {
        self.change_history.push(changeset);
    }
    
    pub fn get_history(&self) -> &[ChangeSet] {
        &self.change_history
    }
    
    pub fn get_changes_for_element(&self, element_id: &str) -> Vec<&ModelChange> {
        self.pending_changes
            .iter()
            .filter(|c| c.element_id == element_id)
            .collect()
    }
    
    pub fn compute_impact(&self, changes: &[ModelChange]) -> ImpactAnalysis {
        let mut analysis = ImpactAnalysis {
            total_changes: changes.len(),
            affected_elements: HashSet::new(),
            affected_files: HashSet::new(),
            change_types: HashMap::new(),
            element_types: HashMap::new(),
            risk_level: RiskLevel::Low,
            recommendations: Vec::new(),
        };
        
        for change in changes {
            analysis.affected_elements.insert(change.element_id.clone());
            analysis.affected_files.insert(change.file_path.clone());
            
            *analysis.change_types.entry(change.change_type.clone()).or_insert(0) += 1;
            *analysis.element_types.entry(change.element_type.clone()).or_insert(0) += 1;
        }
        
        analysis.risk_level = self.assess_risk_level(changes);
        analysis.recommendations = self.generate_recommendations(changes);
        
        analysis
    }
    
    fn assess_risk_level(&self, changes: &[ModelChange]) -> RiskLevel {
        let has_deletions = changes.iter().any(|c| c.change_type == ChangeType::Deleted);
        let has_requirement_changes = changes.iter().any(|c| c.element_type == ElementType::Requirement);
        let has_interface_changes = changes.iter().any(|c| c.element_type == ElementType::Interface);
        
        if has_deletions && has_requirement_changes {
            return RiskLevel::High;
        }
        
        if has_interface_changes {
            return RiskLevel::Medium;
        }
        
        if changes.len() > 20 {
            return RiskLevel::Medium;
        }
        
        RiskLevel::Low
    }
    
    fn generate_recommendations(&self, changes: &[ModelChange]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        let deletions = changes.iter().filter(|c| c.change_type == ChangeType::Deleted).count();
        if deletions > 0 {
            recommendations.push(format!("{} elements will be deleted. Verify traceability impact.", deletions));
        }
        
        let req_changes = changes.iter().filter(|c| c.element_type == ElementType::Requirement).count();
        if req_changes > 5 {
            recommendations.push("Multiple requirements changed. Consider PLM sync.".to_string());
        }
        
        let interface_changes = changes.iter().filter(|c| c.element_type == ElementType::Interface).count();
        if interface_changes > 0 {
            recommendations.push("Interface changes detected. Update integration tests.".to_string());
        }
        
        recommendations
    }
}

use std::collections::HashSet;

pub struct ImpactAnalysis {
    pub total_changes: usize,
    pub affected_elements: HashSet<String>,
    pub affected_files: HashSet<String>,
    pub change_types: HashMap<ChangeType, usize>,
    pub element_types: HashMap<ElementType, usize>,
    pub risk_level: RiskLevel,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

pub fn generate_change_summary(changeset: &ChangeSet) -> String {
    let mut summary = String::new();
    
    summary.push_str(&format!("Changeset: {}\n", changeset.id));
    summary.push_str(&format!("Author: {}\n", changeset.author));
    summary.push_str(&format!("Date: {}\n", changeset.timestamp.format("%Y-%m-%d %H:%M:%S")));
    summary.push_str(&format!("Message: {}\n\n", changeset.message));
    
    summary.push_str("Changes:\n");
    summary.push_str(&format!("  Added Requirements: {}\n", changeset.semantic_diff.added_requirements.len()));
    summary.push_str(&format!("  Modified Requirements: {}\n", changeset.semantic_diff.modified_requirements.len()));
    summary.push_str(&format!("  Deleted Requirements: {}\n", changeset.semantic_diff.deleted_requirements.len()));
    summary.push_str(&format!("  Added Components: {}\n", changeset.semantic_diff.added_components.len()));
    summary.push_str(&format!("  Modified Components: {}\n", changeset.semantic_diff.modified_components.len()));
    summary.push_str(&format!("  Deleted Components: {}\n", changeset.semantic_diff.deleted_components.len()));
    
    summary.push_str(&format!("\nTotal Affected Elements: {}\n", changeset.affected_elements.len()));
    
    if changeset.semantic_diff.integrity_impact.breaks_traceability {
        summary.push_str("\n⚠️  WARNING: This change may break traceability!\n");
    }
    
    if changeset.semantic_diff.integrity_impact.affects_safety_requirements {
        summary.push_str("\n⚠️  WARNING: This change affects safety requirements!\n");
    }
    
    summary
}

pub fn export_changes_to_json(changeset: &ChangeSet) -> Result<String, CollaborationError> {
    serde_json::to_string_pretty(changeset)
        .map_err(|e| CollaborationError::SerializationError(e.to_string()))
}
