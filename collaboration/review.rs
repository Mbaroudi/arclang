use super::*;
use std::collections::HashMap;

pub struct ReviewManager {
    reviews: HashMap<String, ReviewRequest>,
}

impl ReviewManager {
    pub fn new() -> Self {
        Self {
            reviews: HashMap::new(),
        }
    }
    
    pub fn create_review(&mut self, review: ReviewRequest) -> Result<String, CollaborationError> {
        let review_id = review.id.clone();
        
        if self.reviews.contains_key(&review_id) {
            return Err(CollaborationError::InvalidChangeSet(
                "Review ID already exists".to_string()
            ));
        }
        
        self.reviews.insert(review_id.clone(), review);
        
        Ok(review_id)
    }
    
    pub fn get_review(&self, review_id: &str) -> Option<&ReviewRequest> {
        self.reviews.get(review_id)
    }
    
    pub fn get_review_mut(&mut self, review_id: &str) -> Option<&mut ReviewRequest> {
        self.reviews.get_mut(review_id)
    }
    
    pub fn add_comment(&mut self, review_id: &str, comment: ReviewComment) -> Result<(), CollaborationError> {
        let review = self.reviews.get_mut(review_id)
            .ok_or_else(|| CollaborationError::InvalidChangeSet("Review not found".to_string()))?;
        
        review.comments.push(comment);
        review.updated_at = Utc::now();
        
        if review.status == ReviewStatus::Pending {
            review.status = ReviewStatus::InReview;
        }
        
        Ok(())
    }
    
    pub fn add_approval(&mut self, review_id: &str, approval: Approval) -> Result<(), CollaborationError> {
        let review = self.reviews.get_mut(review_id)
            .ok_or_else(|| CollaborationError::InvalidChangeSet("Review not found".to_string()))?;
        
        if !review.reviewers.contains(&approval.reviewer) {
            return Err(CollaborationError::PermissionDenied(
                format!("{} is not a reviewer for this changeset", approval.reviewer)
            ));
        }
        
        review.approvals.retain(|a| a.reviewer != approval.reviewer);
        review.approvals.push(approval.clone());
        review.updated_at = Utc::now();
        
        if approval.approved {
            self.check_and_update_status(review_id)?;
        } else {
            if let Some(rev) = self.reviews.get_mut(review_id) {
                rev.status = ReviewStatus::ChangesRequested;
            }
        }
        
        Ok(())
    }
    
    fn check_and_update_status(&mut self, review_id: &str) -> Result<(), CollaborationError> {
        let review = self.reviews.get_mut(review_id)
            .ok_or_else(|| CollaborationError::InvalidChangeSet("Review not found".to_string()))?;
        
        let required_approvals = review.reviewers.len();
        let approved_count = review.approvals.iter()
            .filter(|a| a.approved)
            .count();
        
        if approved_count >= required_approvals {
            review.status = ReviewStatus::Approved;
        }
        
        Ok(())
    }
    
    pub fn get_pending_reviews(&self, reviewer: &str) -> Vec<&ReviewRequest> {
        self.reviews.values()
            .filter(|r| {
                r.reviewers.contains(&reviewer.to_string()) &&
                matches!(r.status, ReviewStatus::Pending | ReviewStatus::InReview)
            })
            .collect()
    }
    
    pub fn get_reviews_by_author(&self, author: &str) -> Vec<&ReviewRequest> {
        self.reviews.values()
            .filter(|r| r.author == author)
            .collect()
    }
    
    pub fn reject_review(&mut self, review_id: &str, reviewer: &str, reason: String) -> Result<(), CollaborationError> {
        let approval = Approval {
            reviewer: reviewer.to_string(),
            timestamp: Utc::now(),
            approved: false,
            comment: Some(reason),
        };
        
        self.add_approval(review_id, approval)?;
        
        if let Some(review) = self.reviews.get_mut(review_id) {
            review.status = ReviewStatus::Rejected;
        }
        
        Ok(())
    }
}

pub struct ReviewAnalyzer;

impl ReviewAnalyzer {
    pub fn analyze_changeset(changeset: &ChangeSet) -> ReviewAnalysis {
        let mut analysis = ReviewAnalysis {
            complexity_score: 0.0,
            risk_areas: Vec::new(),
            suggested_reviewers: Vec::new(),
            estimated_review_time_minutes: 0,
            focus_areas: Vec::new(),
        };
        
        analysis.complexity_score = Self::calculate_complexity(changeset);
        analysis.risk_areas = Self::identify_risk_areas(changeset);
        analysis.estimated_review_time_minutes = Self::estimate_review_time(changeset);
        analysis.focus_areas = Self::identify_focus_areas(changeset);
        
        analysis
    }
    
    fn calculate_complexity(changeset: &ChangeSet) -> f64 {
        let mut score = 0.0;
        
        score += changeset.changes.len() as f64 * 0.5;
        
        let deletions = changeset.changes.iter()
            .filter(|c| c.change_type == ChangeType::Deleted)
            .count();
        score += deletions as f64 * 2.0;
        
        let requirement_changes = changeset.changes.iter()
            .filter(|c| c.element_type == ElementType::Requirement)
            .count();
        score += requirement_changes as f64 * 1.5;
        
        score
    }
    
    fn identify_risk_areas(changeset: &ChangeSet) -> Vec<RiskArea> {
        let mut areas = Vec::new();
        
        if changeset.semantic_diff.integrity_impact.breaks_traceability {
            areas.push(RiskArea {
                category: RiskCategory::Traceability,
                description: "Changes may break traceability links".to_string(),
                severity: RiskSeverity::High,
                affected_elements: changeset.semantic_diff.integrity_impact.orphaned_elements.clone(),
            });
        }
        
        if changeset.semantic_diff.integrity_impact.affects_safety_requirements {
            areas.push(RiskArea {
                category: RiskCategory::Safety,
                description: "Changes affect safety-critical requirements".to_string(),
                severity: RiskSeverity::Critical,
                affected_elements: changeset.semantic_diff.modified_requirements.clone(),
            });
        }
        
        if !changeset.semantic_diff.integrity_impact.impacts_interfaces.is_empty() {
            areas.push(RiskArea {
                category: RiskCategory::Integration,
                description: "Changes impact component interfaces".to_string(),
                severity: RiskSeverity::Medium,
                affected_elements: changeset.semantic_diff.integrity_impact.impacts_interfaces.clone(),
            });
        }
        
        areas
    }
    
    fn estimate_review_time(changeset: &ChangeSet) -> u32 {
        let base_time = 10;
        let per_change_time = 2;
        let per_requirement_time = 5;
        
        let change_time = changeset.changes.len() as u32 * per_change_time;
        let requirement_time = (changeset.semantic_diff.added_requirements.len() + 
                               changeset.semantic_diff.modified_requirements.len()) as u32 * per_requirement_time;
        
        base_time + change_time + requirement_time
    }
    
    fn identify_focus_areas(changeset: &ChangeSet) -> Vec<String> {
        let mut areas = Vec::new();
        
        if !changeset.semantic_diff.added_requirements.is_empty() {
            areas.push(format!("Review {} new requirements for completeness", 
                changeset.semantic_diff.added_requirements.len()));
        }
        
        if !changeset.semantic_diff.modified_requirements.is_empty() {
            areas.push(format!("Verify {} requirement modifications maintain traceability", 
                changeset.semantic_diff.modified_requirements.len()));
        }
        
        if !changeset.semantic_diff.deleted_requirements.is_empty() {
            areas.push(format!("Validate {} requirement deletions and impact", 
                changeset.semantic_diff.deleted_requirements.len()));
        }
        
        if !changeset.semantic_diff.added_relationships.is_empty() {
            areas.push("Check new relationships for correctness".to_string());
        }
        
        areas
    }
}

#[derive(Debug, Clone)]
pub struct ReviewAnalysis {
    pub complexity_score: f64,
    pub risk_areas: Vec<RiskArea>,
    pub suggested_reviewers: Vec<String>,
    pub estimated_review_time_minutes: u32,
    pub focus_areas: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RiskArea {
    pub category: RiskCategory,
    pub description: String,
    pub severity: RiskSeverity,
    pub affected_elements: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum RiskCategory {
    Safety,
    Traceability,
    Integration,
    Compliance,
    Performance,
}

#[derive(Debug, Clone)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

pub fn generate_review_report(review: &ReviewRequest) -> String {
    let mut report = String::new();
    
    report.push_str("Code Review Report\n");
    report.push_str("==================\n\n");
    
    report.push_str(&format!("Review ID: {}\n", review.id));
    report.push_str(&format!("Changeset: {}\n", review.changeset_id));
    report.push_str(&format!("Author: {}\n", review.author));
    report.push_str(&format!("Status: {:?}\n", review.status));
    report.push_str(&format!("Created: {}\n", review.created_at.format("%Y-%m-%d %H:%M:%S")));
    report.push_str(&format!("Updated: {}\n\n", review.updated_at.format("%Y-%m-%d %H:%M:%S")));
    
    report.push_str("Reviewers:\n");
    for reviewer in &review.reviewers {
        let approval = review.approvals.iter().find(|a| a.reviewer == *reviewer);
        let status = if let Some(a) = approval {
            if a.approved { "✓ Approved" } else { "✗ Changes Requested" }
        } else {
            "⧗ Pending"
        };
        report.push_str(&format!("  {} - {}\n", reviewer, status));
    }
    report.push_str("\n");
    
    if !review.comments.is_empty() {
        report.push_str("Comments:\n");
        report.push_str("---------\n");
        for comment in &review.comments {
            report.push_str(&format!("[{}] {} at {}\n", 
                match comment.comment_type {
                    CommentType::General => "General",
                    CommentType::Question => "Question",
                    CommentType::Suggestion => "Suggestion",
                    CommentType::Issue => "Issue",
                    CommentType::Blocker => "BLOCKER",
                },
                comment.author,
                comment.timestamp.format("%Y-%m-%d %H:%M")
            ));
            if let Some(elem_id) = &comment.element_id {
                report.push_str(&format!("  Element: {}\n", elem_id));
            }
            report.push_str(&format!("  {}\n\n", comment.comment_text));
        }
    }
    
    report
}

pub fn export_review_to_markdown(review: &ReviewRequest, analysis: &ReviewAnalysis) -> String {
    let mut md = String::new();
    
    md.push_str(&format!("# Code Review: {}\n\n", review.id));
    
    md.push_str("## Summary\n\n");
    md.push_str(&format!("- **Changeset**: {}\n", review.changeset_id));
    md.push_str(&format!("- **Author**: {}\n", review.author));
    md.push_str(&format!("- **Status**: {:?}\n", review.status));
    md.push_str(&format!("- **Complexity Score**: {:.1}\n", analysis.complexity_score));
    md.push_str(&format!("- **Estimated Review Time**: {} minutes\n\n", analysis.estimated_review_time_minutes));
    
    if !analysis.risk_areas.is_empty() {
        md.push_str("## Risk Areas\n\n");
        for risk in &analysis.risk_areas {
            md.push_str(&format!("- **{:?}** ({:?}): {}\n", 
                risk.category, risk.severity, risk.description));
        }
        md.push_str("\n");
    }
    
    if !analysis.focus_areas.is_empty() {
        md.push_str("## Focus Areas\n\n");
        for area in &analysis.focus_areas {
            md.push_str(&format!("- [ ] {}\n", area));
        }
        md.push_str("\n");
    }
    
    md.push_str("## Reviewers\n\n");
    for reviewer in &review.reviewers {
        let approval = review.approvals.iter().find(|a| a.reviewer == *reviewer);
        md.push_str(&format!("- **{}**: ", reviewer));
        if let Some(a) = approval {
            md.push_str(if a.approved { "✅ Approved" } else { "❌ Changes Requested" });
            if let Some(comment) = &a.comment {
                md.push_str(&format!(" - _{}_", comment));
            }
        } else {
            md.push_str("⏳ Pending");
        }
        md.push_str("\n");
    }
    
    md
}
