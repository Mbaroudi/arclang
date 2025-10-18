pub mod git_integration;
pub mod semantic_merge;
pub mod conflict_resolution;
pub mod change_tracking;
pub mod review;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationConfig {
    pub repository_path: String,
    pub remote_url: Option<String>,
    pub branch_strategy: BranchStrategy,
    pub merge_strategy: MergeStrategy,
    pub review_required: bool,
    pub auto_sync: bool,
    pub conflict_resolution: ConflictResolutionPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BranchStrategy {
    GitFlow,
    TrunkBased,
    FeatureBranch,
    ReleaseFlow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeStrategy {
    SemanticMerge,
    ThreeWayMerge,
    FastForward,
    Rebase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolutionPolicy {
    Manual,
    AutoResolveNonSemantic,
    AutoResolveAll,
    PreferBase,
    PreferTheirs,
    PreferOurs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSet {
    pub id: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub changes: Vec<ModelChange>,
    pub affected_elements: Vec<String>,
    pub semantic_diff: SemanticDiff,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelChange {
    pub change_type: ChangeType,
    pub element_id: String,
    pub element_type: ElementType,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub file_path: String,
    pub line_range: Option<(usize, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
    Moved,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ElementType {
    Requirement,
    Component,
    Function,
    Interface,
    Capability,
    Actor,
    Scenario,
    DataType,
    Constraint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticDiff {
    pub added_requirements: Vec<String>,
    pub modified_requirements: Vec<String>,
    pub deleted_requirements: Vec<String>,
    pub added_components: Vec<String>,
    pub modified_components: Vec<String>,
    pub deleted_components: Vec<String>,
    pub added_relationships: Vec<RelationshipChange>,
    pub deleted_relationships: Vec<RelationshipChange>,
    pub integrity_impact: IntegrityImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipChange {
    pub relationship_type: String,
    pub source_id: String,
    pub target_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityImpact {
    pub breaks_traceability: bool,
    pub affects_safety_requirements: bool,
    pub impacts_interfaces: Vec<String>,
    pub orphaned_elements: Vec<String>,
    pub severity: ImpactSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImpactSeverity {
    None,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    pub success: bool,
    pub conflicts: Vec<Conflict>,
    pub auto_resolved: Vec<AutoResolution>,
    pub merged_changes: Vec<ModelChange>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub id: String,
    pub conflict_type: ConflictType,
    pub element_id: String,
    pub element_type: ElementType,
    pub base_value: Option<serde_json::Value>,
    pub ours_value: Option<serde_json::Value>,
    pub theirs_value: Option<serde_json::Value>,
    pub description: String,
    pub resolution: Option<ConflictResolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    TextualConflict,
    SemanticConflict,
    StructuralConflict,
    TraceabilityConflict,
    DuplicateId,
    DeleteModify,
    MoveMove,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub resolved_by: String,
    pub resolved_at: DateTime<Utc>,
    pub resolution_strategy: ResolutionStrategy,
    pub resolved_value: serde_json::Value,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    KeepOurs,
    KeepTheirs,
    KeepBoth,
    Manual,
    Merge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoResolution {
    pub element_id: String,
    pub conflict_type: ConflictType,
    pub strategy: ResolutionStrategy,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRequest {
    pub id: String,
    pub changeset_id: String,
    pub author: String,
    pub reviewers: Vec<String>,
    pub status: ReviewStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub comments: Vec<ReviewComment>,
    pub approvals: Vec<Approval>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReviewStatus {
    Pending,
    InReview,
    ChangesRequested,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewComment {
    pub id: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub element_id: Option<String>,
    pub comment_text: String,
    pub comment_type: CommentType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommentType {
    General,
    Question,
    Suggestion,
    Issue,
    Blocker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Approval {
    pub reviewer: String,
    pub timestamp: DateTime<Utc>,
    pub approved: bool,
    pub comment: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum CollaborationError {
    #[error("Git error: {0}")]
    GitError(String),
    
    #[error("Merge conflict: {0}")]
    MergeConflict(String),
    
    #[error("Semantic conflict: {0}")]
    SemanticConflict(String),
    
    #[error("Traceability broken: {0}")]
    TraceabilityBroken(String),
    
    #[error("Review required but not completed")]
    ReviewRequired,
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Invalid changeset: {0}")]
    InvalidChangeSet(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub struct CollaborationManager {
    config: CollaborationConfig,
    git: git_integration::GitManager,
    semantic_merge: semantic_merge::SemanticMerger,
    conflict_resolver: conflict_resolution::ConflictResolver,
    change_tracker: change_tracking::ChangeTracker,
}

impl CollaborationManager {
    pub fn new(config: CollaborationConfig) -> Result<Self, CollaborationError> {
        let git = git_integration::GitManager::new(&config.repository_path)?;
        let semantic_merge = semantic_merge::SemanticMerger::new();
        let conflict_resolver = conflict_resolution::ConflictResolver::new(config.conflict_resolution.clone());
        let change_tracker = change_tracking::ChangeTracker::new();
        
        Ok(Self {
            config,
            git,
            semantic_merge,
            conflict_resolver,
            change_tracker,
        })
    }
    
    pub fn commit(&mut self, message: &str, author: &str) -> Result<ChangeSet, CollaborationError> {
        let changes = self.change_tracker.get_pending_changes()?;
        
        if changes.is_empty() {
            return Err(CollaborationError::InvalidChangeSet("No changes to commit".to_string()));
        }
        
        let semantic_diff = self.compute_semantic_diff(&changes)?;
        
        self.validate_changes(&semantic_diff)?;
        
        let commit_id = self.git.commit(message, author)?;
        
        let changeset = ChangeSet {
            id: commit_id,
            author: author.to_string(),
            timestamp: Utc::now(),
            message: message.to_string(),
            changes,
            affected_elements: semantic_diff.affected_element_ids(),
            semantic_diff,
        };
        
        self.change_tracker.clear_pending_changes();
        
        Ok(changeset)
    }
    
    pub fn merge(&mut self, branch: &str) -> Result<MergeResult, CollaborationError> {
        let base_model = self.git.get_common_ancestor(branch)?;
        let ours_model = self.git.get_current_model()?;
        let theirs_model = self.git.get_branch_model(branch)?;
        
        let merge_result = self.semantic_merge.merge(&base_model, &ours_model, &theirs_model)?;
        
        if !merge_result.conflicts.is_empty() {
            if self.config.conflict_resolution != ConflictResolutionPolicy::Manual {
                let resolved = self.conflict_resolver.auto_resolve(&merge_result.conflicts)?;
                return Ok(MergeResult {
                    success: resolved.unresolved.is_empty(),
                    conflicts: resolved.unresolved,
                    auto_resolved: resolved.resolved,
                    merged_changes: merge_result.merged_changes,
                    warnings: merge_result.warnings,
                });
            }
        }
        
        Ok(merge_result)
    }
    
    pub fn create_review_request(&self, changeset_id: &str, reviewers: Vec<String>) -> Result<ReviewRequest, CollaborationError> {
        let changeset = self.git.get_changeset(changeset_id)?;
        
        let review = ReviewRequest {
            id: format!("RR-{}", uuid::Uuid::new_v4()),
            changeset_id: changeset_id.to_string(),
            author: changeset.author.clone(),
            reviewers,
            status: ReviewStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            comments: Vec::new(),
            approvals: Vec::new(),
        };
        
        Ok(review)
    }
    
    fn compute_semantic_diff(&self, changes: &[ModelChange]) -> Result<SemanticDiff, CollaborationError> {
        let mut diff = SemanticDiff {
            added_requirements: Vec::new(),
            modified_requirements: Vec::new(),
            deleted_requirements: Vec::new(),
            added_components: Vec::new(),
            modified_components: Vec::new(),
            deleted_components: Vec::new(),
            added_relationships: Vec::new(),
            deleted_relationships: Vec::new(),
            integrity_impact: IntegrityImpact {
                breaks_traceability: false,
                affects_safety_requirements: false,
                impacts_interfaces: Vec::new(),
                orphaned_elements: Vec::new(),
                severity: ImpactSeverity::None,
            },
        };
        
        for change in changes {
            match (&change.change_type, &change.element_type) {
                (ChangeType::Added, ElementType::Requirement) => {
                    diff.added_requirements.push(change.element_id.clone());
                }
                (ChangeType::Modified, ElementType::Requirement) => {
                    diff.modified_requirements.push(change.element_id.clone());
                }
                (ChangeType::Deleted, ElementType::Requirement) => {
                    diff.deleted_requirements.push(change.element_id.clone());
                }
                (ChangeType::Added, ElementType::Component) => {
                    diff.added_components.push(change.element_id.clone());
                }
                (ChangeType::Modified, ElementType::Component) => {
                    diff.modified_components.push(change.element_id.clone());
                }
                (ChangeType::Deleted, ElementType::Component) => {
                    diff.deleted_components.push(change.element_id.clone());
                }
                _ => {}
            }
        }
        
        Ok(diff)
    }
    
    fn validate_changes(&self, diff: &SemanticDiff) -> Result<(), CollaborationError> {
        if diff.integrity_impact.breaks_traceability {
            return Err(CollaborationError::TraceabilityBroken(
                "Changes would break traceability links".to_string()
            ));
        }
        
        Ok(())
    }
}

impl SemanticDiff {
    fn affected_element_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();
        ids.extend(self.added_requirements.clone());
        ids.extend(self.modified_requirements.clone());
        ids.extend(self.deleted_requirements.clone());
        ids.extend(self.added_components.clone());
        ids.extend(self.modified_components.clone());
        ids.extend(self.deleted_components.clone());
        ids
    }
}
