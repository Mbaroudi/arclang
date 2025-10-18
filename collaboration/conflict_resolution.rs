use super::*;

pub struct ConflictResolver {
    policy: ConflictResolutionPolicy,
}

impl ConflictResolver {
    pub fn new(policy: ConflictResolutionPolicy) -> Self {
        Self { policy }
    }
    
    pub fn auto_resolve(&self, conflicts: &[Conflict]) -> Result<ResolutionResult, CollaborationError> {
        let mut resolved = Vec::new();
        let mut unresolved = Vec::new();
        
        for conflict in conflicts {
            match self.try_auto_resolve(conflict)? {
                Some(resolution) => {
                    resolved.push(AutoResolution {
                        element_id: conflict.element_id.clone(),
                        conflict_type: conflict.conflict_type.clone(),
                        strategy: resolution.resolution_strategy.clone(),
                        confidence: self.calculate_confidence(&conflict.conflict_type, &resolution.resolution_strategy),
                    });
                }
                None => {
                    unresolved.push(conflict.clone());
                }
            }
        }
        
        Ok(ResolutionResult {
            resolved,
            unresolved,
        })
    }
    
    fn try_auto_resolve(&self, conflict: &Conflict) -> Result<Option<ConflictResolution>, CollaborationError> {
        match (&self.policy, &conflict.conflict_type) {
            (ConflictResolutionPolicy::AutoResolveNonSemantic, ConflictType::TextualConflict) => {
                self.resolve_textual_conflict(conflict)
            }
            (ConflictResolutionPolicy::AutoResolveAll, _) => {
                self.resolve_by_heuristic(conflict)
            }
            (ConflictResolutionPolicy::PreferOurs, _) => {
                Ok(Some(ConflictResolution {
                    resolved_by: "auto".to_string(),
                    resolved_at: Utc::now(),
                    resolution_strategy: ResolutionStrategy::KeepOurs,
                    resolved_value: conflict.ours_value.clone().unwrap_or(serde_json::Value::Null),
                    rationale: "Policy: Prefer ours".to_string(),
                }))
            }
            (ConflictResolutionPolicy::PreferTheirs, _) => {
                Ok(Some(ConflictResolution {
                    resolved_by: "auto".to_string(),
                    resolved_at: Utc::now(),
                    resolution_strategy: ResolutionStrategy::KeepTheirs,
                    resolved_value: conflict.theirs_value.clone().unwrap_or(serde_json::Value::Null),
                    rationale: "Policy: Prefer theirs".to_string(),
                }))
            }
            (ConflictResolutionPolicy::PreferBase, _) => {
                Ok(Some(ConflictResolution {
                    resolved_by: "auto".to_string(),
                    resolved_at: Utc::now(),
                    resolution_strategy: ResolutionStrategy::KeepOurs,
                    resolved_value: conflict.base_value.clone().unwrap_or(serde_json::Value::Null),
                    rationale: "Policy: Prefer base".to_string(),
                }))
            }
            _ => Ok(None),
        }
    }
    
    fn resolve_textual_conflict(&self, conflict: &Conflict) -> Result<Option<ConflictResolution>, CollaborationError> {
        if conflict.element_type == ElementType::Requirement {
            return Ok(None);
        }
        
        let ours = conflict.ours_value.as_ref()
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let theirs = conflict.theirs_value.as_ref()
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        if ours.len() > theirs.len() {
            Ok(Some(ConflictResolution {
                resolved_by: "auto".to_string(),
                resolved_at: Utc::now(),
                resolution_strategy: ResolutionStrategy::KeepOurs,
                resolved_value: conflict.ours_value.clone().unwrap(),
                rationale: "Selected longer text variant".to_string(),
            }))
        } else {
            Ok(Some(ConflictResolution {
                resolved_by: "auto".to_string(),
                resolved_at: Utc::now(),
                resolution_strategy: ResolutionStrategy::KeepTheirs,
                resolved_value: conflict.theirs_value.clone().unwrap(),
                rationale: "Selected longer text variant".to_string(),
            }))
        }
    }
    
    fn resolve_by_heuristic(&self, conflict: &Conflict) -> Result<Option<ConflictResolution>, CollaborationError> {
        match conflict.conflict_type {
            ConflictType::DeleteModify => {
                Ok(Some(ConflictResolution {
                    resolved_by: "auto".to_string(),
                    resolved_at: Utc::now(),
                    resolution_strategy: ResolutionStrategy::KeepTheirs,
                    resolved_value: if conflict.ours_value.is_none() {
                        conflict.theirs_value.clone().unwrap()
                    } else {
                        conflict.ours_value.clone().unwrap()
                    },
                    rationale: "Prefer modification over deletion".to_string(),
                }))
            }
            ConflictType::DuplicateId => {
                Ok(None)
            }
            ConflictType::SemanticConflict => {
                self.resolve_semantic_conflict(conflict)
            }
            ConflictType::TraceabilityConflict => {
                self.resolve_traceability_conflict(conflict)
            }
            _ => Ok(None),
        }
    }
    
    fn resolve_semantic_conflict(&self, conflict: &Conflict) -> Result<Option<ConflictResolution>, CollaborationError> {
        if let (Some(ours), Some(theirs)) = (&conflict.ours_value, &conflict.theirs_value) {
            if let (Some(ours_obj), Some(theirs_obj)) = (ours.as_object(), theirs.as_object()) {
                let merged = self.merge_objects(ours_obj, theirs_obj)?;
                
                return Ok(Some(ConflictResolution {
                    resolved_by: "auto".to_string(),
                    resolved_at: Utc::now(),
                    resolution_strategy: ResolutionStrategy::Merge,
                    resolved_value: serde_json::Value::Object(merged),
                    rationale: "Merged non-conflicting properties".to_string(),
                }));
            }
        }
        
        Ok(None)
    }
    
    fn resolve_traceability_conflict(&self, conflict: &Conflict) -> Result<Option<ConflictResolution>, CollaborationError> {
        if let (Some(ours), Some(theirs)) = (&conflict.ours_value, &conflict.theirs_value) {
            if let (Some(ours_arr), Some(theirs_arr)) = (
                ours.get("traces").and_then(|v| v.as_array()),
                theirs.get("traces").and_then(|v| v.as_array()),
            ) {
                let mut combined = std::collections::HashSet::new();
                
                for item in ours_arr {
                    if let Some(s) = item.as_str() {
                        combined.insert(s.to_string());
                    }
                }
                
                for item in theirs_arr {
                    if let Some(s) = item.as_str() {
                        combined.insert(s.to_string());
                    }
                }
                
                let merged_traces: Vec<serde_json::Value> = combined
                    .into_iter()
                    .map(serde_json::Value::String)
                    .collect();
                
                let mut merged_obj = ours.as_object().unwrap().clone();
                merged_obj.insert("traces".to_string(), serde_json::Value::Array(merged_traces));
                
                return Ok(Some(ConflictResolution {
                    resolved_by: "auto".to_string(),
                    resolved_at: Utc::now(),
                    resolution_strategy: ResolutionStrategy::KeepBoth,
                    resolved_value: serde_json::Value::Object(merged_obj),
                    rationale: "Merged traceability links from both branches".to_string(),
                }));
            }
        }
        
        Ok(None)
    }
    
    fn merge_objects(
        &self,
        ours: &serde_json::Map<String, serde_json::Value>,
        theirs: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<serde_json::Map<String, serde_json::Value>, CollaborationError> {
        let mut merged = ours.clone();
        
        for (key, theirs_val) in theirs {
            if !ours.contains_key(key) {
                merged.insert(key.clone(), theirs_val.clone());
            } else if ours.get(key) != Some(theirs_val) {
            }
        }
        
        Ok(merged)
    }
    
    fn calculate_confidence(&self, conflict_type: &ConflictType, strategy: &ResolutionStrategy) -> f64 {
        match (conflict_type, strategy) {
            (ConflictType::TextualConflict, ResolutionStrategy::KeepOurs | ResolutionStrategy::KeepTheirs) => 0.8,
            (ConflictType::TraceabilityConflict, ResolutionStrategy::KeepBoth) => 0.95,
            (ConflictType::DeleteModify, ResolutionStrategy::KeepTheirs) => 0.7,
            (ConflictType::SemanticConflict, ResolutionStrategy::Merge) => 0.6,
            _ => 0.5,
        }
    }
}

pub struct ResolutionResult {
    pub resolved: Vec<AutoResolution>,
    pub unresolved: Vec<Conflict>,
}

pub struct InteractiveResolver {
    ui_enabled: bool,
}

impl InteractiveResolver {
    pub fn new() -> Self {
        Self { ui_enabled: true }
    }
    
    pub fn resolve_interactively(&self, conflict: &Conflict) -> Result<ConflictResolution, CollaborationError> {
        println!("\n=== Conflict Resolution ===");
        println!("Element: {} ({:?})", conflict.element_id, conflict.element_type);
        println!("Conflict Type: {:?}", conflict.conflict_type);
        println!("Description: {}", conflict.description);
        println!("\nOptions:");
        println!("1. Keep ours");
        println!("2. Keep theirs");
        println!("3. Keep both");
        println!("4. Manual edit");
        println!("5. Skip");
        
        let strategy = ResolutionStrategy::Manual;
        let resolved_value = conflict.ours_value.clone().unwrap_or(serde_json::Value::Null);
        
        Ok(ConflictResolution {
            resolved_by: "user".to_string(),
            resolved_at: Utc::now(),
            resolution_strategy: strategy,
            resolved_value,
            rationale: "User selection".to_string(),
        })
    }
    
    pub fn show_diff(&self, conflict: &Conflict) {
        println!("\n--- Base ---");
        if let Some(base) = &conflict.base_value {
            println!("{}", serde_json::to_string_pretty(base).unwrap());
        } else {
            println!("(none)");
        }
        
        println!("\n--- Ours ---");
        if let Some(ours) = &conflict.ours_value {
            println!("{}", serde_json::to_string_pretty(ours).unwrap());
        } else {
            println!("(deleted)");
        }
        
        println!("\n--- Theirs ---");
        if let Some(theirs) = &conflict.theirs_value {
            println!("{}", serde_json::to_string_pretty(theirs).unwrap());
        } else {
            println!("(deleted)");
        }
    }
}

pub fn generate_conflict_report(conflicts: &[Conflict]) -> String {
    let mut report = String::new();
    
    report.push_str("Merge Conflict Report\n");
    report.push_str("=====================\n\n");
    
    report.push_str(&format!("Total Conflicts: {}\n\n", conflicts.len()));
    
    let mut by_type: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for conflict in conflicts {
        *by_type.entry(format!("{:?}", conflict.conflict_type)).or_insert(0) += 1;
    }
    
    report.push_str("Conflicts by Type:\n");
    for (conflict_type, count) in by_type {
        report.push_str(&format!("  {}: {}\n", conflict_type, count));
    }
    report.push_str("\n");
    
    report.push_str("Detailed Conflicts:\n");
    report.push_str("-------------------\n\n");
    
    for (i, conflict) in conflicts.iter().enumerate() {
        report.push_str(&format!("{}. {} ({:?})\n", i + 1, conflict.element_id, conflict.conflict_type));
        report.push_str(&format!("   {}\n", conflict.description));
        report.push_str(&format!("   Element Type: {:?}\n", conflict.element_type));
        report.push_str("\n");
    }
    
    report
}
