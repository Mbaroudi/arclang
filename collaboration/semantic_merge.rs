use super::*;
use std::collections::{HashMap, HashSet};

pub struct SemanticMerger {
    merge_rules: Vec<MergeRule>,
}

impl SemanticMerger {
    pub fn new() -> Self {
        Self {
            merge_rules: Self::default_merge_rules(),
        }
    }
    
    pub fn merge(
        &self,
        base: &ModelSnapshot,
        ours: &ModelSnapshot,
        theirs: &ModelSnapshot,
    ) -> Result<MergeResult, CollaborationError> {
        let mut conflicts = Vec::new();
        let mut merged_changes = Vec::new();
        let mut warnings = Vec::new();
        
        let all_elements = self.collect_all_element_ids(base, ours, theirs);
        
        for element_id in all_elements {
            let merge_outcome = self.merge_element(element_id, base, ours, theirs)?;
            
            match merge_outcome {
                ElementMergeOutcome::NoConflict(change) => {
                    if let Some(c) = change {
                        merged_changes.push(c);
                    }
                }
                ElementMergeOutcome::Conflict(conflict) => {
                    conflicts.push(conflict);
                }
                ElementMergeOutcome::Warning(msg) => {
                    warnings.push(msg);
                }
            }
        }
        
        self.validate_semantic_integrity(&merged_changes, &mut warnings)?;
        
        Ok(MergeResult {
            success: conflicts.is_empty(),
            conflicts,
            auto_resolved: Vec::new(),
            merged_changes,
            warnings,
        })
    }
    
    fn merge_element(
        &self,
        element_id: &str,
        base: &ModelSnapshot,
        ours: &ModelSnapshot,
        theirs: &ModelSnapshot,
    ) -> Result<ElementMergeOutcome, CollaborationError> {
        let base_elem = base.get_element(element_id);
        let ours_elem = ours.get_element(element_id);
        let theirs_elem = theirs.get_element(element_id);
        
        match (base_elem, ours_elem, theirs_elem) {
            (None, Some(o), None) => {
                Ok(ElementMergeOutcome::NoConflict(Some(ModelChange {
                    change_type: ChangeType::Added,
                    element_id: element_id.to_string(),
                    element_type: o.element_type.clone(),
                    old_value: None,
                    new_value: Some(o.to_json()),
                    file_path: o.file_path.clone(),
                    line_range: None,
                })))
            }
            (None, None, Some(t)) => {
                Ok(ElementMergeOutcome::NoConflict(Some(ModelChange {
                    change_type: ChangeType::Added,
                    element_id: element_id.to_string(),
                    element_type: t.element_type.clone(),
                    old_value: None,
                    new_value: Some(t.to_json()),
                    file_path: t.file_path.clone(),
                    line_range: None,
                })))
            }
            (None, Some(o), Some(t)) => {
                if o.to_json() == t.to_json() {
                    Ok(ElementMergeOutcome::NoConflict(Some(ModelChange {
                        change_type: ChangeType::Added,
                        element_id: element_id.to_string(),
                        element_type: o.element_type.clone(),
                        old_value: None,
                        new_value: Some(o.to_json()),
                        file_path: o.file_path.clone(),
                        line_range: None,
                    })))
                } else {
                    Ok(ElementMergeOutcome::Conflict(Conflict {
                        id: format!("CONF-{}", element_id),
                        conflict_type: ConflictType::DuplicateId,
                        element_id: element_id.to_string(),
                        element_type: o.element_type.clone(),
                        base_value: None,
                        ours_value: Some(o.to_json()),
                        theirs_value: Some(t.to_json()),
                        description: format!("Both branches added element '{}' with different content", element_id),
                        resolution: None,
                    }))
                }
            }
            (Some(b), None, Some(t)) => {
                if t.to_json() == b.to_json() {
                    Ok(ElementMergeOutcome::NoConflict(Some(ModelChange {
                        change_type: ChangeType::Deleted,
                        element_id: element_id.to_string(),
                        element_type: b.element_type.clone(),
                        old_value: Some(b.to_json()),
                        new_value: None,
                        file_path: b.file_path.clone(),
                        line_range: None,
                    })))
                } else {
                    Ok(ElementMergeOutcome::Conflict(Conflict {
                        id: format!("CONF-{}", element_id),
                        conflict_type: ConflictType::DeleteModify,
                        element_id: element_id.to_string(),
                        element_type: b.element_type.clone(),
                        base_value: Some(b.to_json()),
                        ours_value: None,
                        theirs_value: Some(t.to_json()),
                        description: format!("Element '{}' deleted in ours but modified in theirs", element_id),
                        resolution: None,
                    }))
                }
            }
            (Some(b), Some(o), None) => {
                if o.to_json() == b.to_json() {
                    Ok(ElementMergeOutcome::NoConflict(Some(ModelChange {
                        change_type: ChangeType::Deleted,
                        element_id: element_id.to_string(),
                        element_type: b.element_type.clone(),
                        old_value: Some(b.to_json()),
                        new_value: None,
                        file_path: b.file_path.clone(),
                        line_range: None,
                    })))
                } else {
                    Ok(ElementMergeOutcome::Conflict(Conflict {
                        id: format!("CONF-{}", element_id),
                        conflict_type: ConflictType::DeleteModify,
                        element_id: element_id.to_string(),
                        element_type: b.element_type.clone(),
                        base_value: Some(b.to_json()),
                        ours_value: Some(o.to_json()),
                        theirs_value: None,
                        description: format!("Element '{}' modified in ours but deleted in theirs", element_id),
                        resolution: None,
                    }))
                }
            }
            (Some(b), Some(o), Some(t)) => {
                if o.to_json() == t.to_json() {
                    Ok(ElementMergeOutcome::NoConflict(Some(ModelChange {
                        change_type: ChangeType::Modified,
                        element_id: element_id.to_string(),
                        element_type: o.element_type.clone(),
                        old_value: Some(b.to_json()),
                        new_value: Some(o.to_json()),
                        file_path: o.file_path.clone(),
                        line_range: None,
                    })))
                } else if o.to_json() == b.to_json() {
                    Ok(ElementMergeOutcome::NoConflict(Some(ModelChange {
                        change_type: ChangeType::Modified,
                        element_id: element_id.to_string(),
                        element_type: t.element_type.clone(),
                        old_value: Some(b.to_json()),
                        new_value: Some(t.to_json()),
                        file_path: t.file_path.clone(),
                        line_range: None,
                    })))
                } else if t.to_json() == b.to_json() {
                    Ok(ElementMergeOutcome::NoConflict(Some(ModelChange {
                        change_type: ChangeType::Modified,
                        element_id: element_id.to_string(),
                        element_type: o.element_type.clone(),
                        old_value: Some(b.to_json()),
                        new_value: Some(o.to_json()),
                        file_path: o.file_path.clone(),
                        line_range: None,
                    })))
                } else {
                    let semantic_merge = self.try_semantic_merge(b, o, t)?;
                    
                    if let Some(merged) = semantic_merge {
                        Ok(ElementMergeOutcome::NoConflict(Some(ModelChange {
                            change_type: ChangeType::Modified,
                            element_id: element_id.to_string(),
                            element_type: merged.element_type.clone(),
                            old_value: Some(b.to_json()),
                            new_value: Some(merged.to_json()),
                            file_path: merged.file_path.clone(),
                            line_range: None,
                        })))
                    } else {
                        Ok(ElementMergeOutcome::Conflict(Conflict {
                            id: format!("CONF-{}", element_id),
                            conflict_type: ConflictType::SemanticConflict,
                            element_id: element_id.to_string(),
                            element_type: b.element_type.clone(),
                            base_value: Some(b.to_json()),
                            ours_value: Some(o.to_json()),
                            theirs_value: Some(t.to_json()),
                            description: format!("Element '{}' modified differently in both branches", element_id),
                            resolution: None,
                        }))
                    }
                }
            }
            _ => Ok(ElementMergeOutcome::NoConflict(None)),
        }
    }
    
    fn try_semantic_merge(
        &self,
        base: &ModelElement,
        ours: &ModelElement,
        theirs: &ModelElement,
    ) -> Result<Option<ModelElement>, CollaborationError> {
        for rule in &self.merge_rules {
            if rule.applies_to(&base.element_type) {
                if let Some(merged) = rule.merge(base, ours, theirs)? {
                    return Ok(Some(merged));
                }
            }
        }
        
        Ok(None)
    }
    
    fn collect_all_element_ids(
        &self,
        base: &ModelSnapshot,
        ours: &ModelSnapshot,
        theirs: &ModelSnapshot,
    ) -> Vec<String> {
        let mut ids = HashSet::new();
        
        for id in base.element_ids() {
            ids.insert(id.clone());
        }
        
        for id in ours.element_ids() {
            ids.insert(id.clone());
        }
        
        for id in theirs.element_ids() {
            ids.insert(id.clone());
        }
        
        ids.into_iter().collect()
    }
    
    fn validate_semantic_integrity(
        &self,
        merged_changes: &[ModelChange],
        warnings: &mut Vec<String>,
    ) -> Result<(), CollaborationError> {
        let mut element_refs = HashSet::new();
        let mut defined_elements = HashSet::new();
        
        for change in merged_changes {
            match change.change_type {
                ChangeType::Added | ChangeType::Modified => {
                    defined_elements.insert(change.element_id.clone());
                    
                    if let Some(new_val) = &change.new_value {
                        let refs = self.extract_references(new_val);
                        element_refs.extend(refs);
                    }
                }
                ChangeType::Deleted => {}
                _ => {}
            }
        }
        
        for ref_id in element_refs {
            if !defined_elements.contains(&ref_id) {
                warnings.push(format!("Warning: Reference to undefined element '{}'", ref_id));
            }
        }
        
        Ok(())
    }
    
    fn extract_references(&self, value: &serde_json::Value) -> Vec<String> {
        let mut refs = Vec::new();
        
        if let Some(obj) = value.as_object() {
            if let Some(traces) = obj.get("traces") {
                if let Some(arr) = traces.as_array() {
                    for item in arr {
                        if let Some(s) = item.as_str() {
                            refs.push(s.to_string());
                        }
                    }
                }
            }
        }
        
        refs
    }
    
    fn default_merge_rules() -> Vec<MergeRule> {
        vec![
            MergeRule::PropertyMerge,
            MergeRule::ListMerge,
            MergeRule::TraceabilityMerge,
        ]
    }
}

enum ElementMergeOutcome {
    NoConflict(Option<ModelChange>),
    Conflict(Conflict),
    Warning(String),
}

#[derive(Debug, Clone)]
pub struct ModelSnapshot {
    elements: HashMap<String, ModelElement>,
}

impl ModelSnapshot {
    pub fn get_element(&self, id: &str) -> Option<&ModelElement> {
        self.elements.get(id)
    }
    
    pub fn element_ids(&self) -> Vec<&String> {
        self.elements.keys().collect()
    }
}

#[derive(Debug, Clone)]
pub struct ModelElement {
    pub id: String,
    pub element_type: ElementType,
    pub properties: HashMap<String, serde_json::Value>,
    pub file_path: String,
}

impl ModelElement {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "type": format!("{:?}", self.element_type),
            "properties": self.properties,
        })
    }
}

enum MergeRule {
    PropertyMerge,
    ListMerge,
    TraceabilityMerge,
}

impl MergeRule {
    fn applies_to(&self, _element_type: &ElementType) -> bool {
        true
    }
    
    fn merge(
        &self,
        base: &ModelElement,
        ours: &ModelElement,
        theirs: &ModelElement,
    ) -> Result<Option<ModelElement>, CollaborationError> {
        match self {
            MergeRule::PropertyMerge => self.merge_properties(base, ours, theirs),
            MergeRule::ListMerge => self.merge_lists(base, ours, theirs),
            MergeRule::TraceabilityMerge => self.merge_traceability(base, ours, theirs),
        }
    }
    
    fn merge_properties(
        &self,
        base: &ModelElement,
        ours: &ModelElement,
        theirs: &ModelElement,
    ) -> Result<Option<ModelElement>, CollaborationError> {
        let mut merged_properties = base.properties.clone();
        
        for (key, ours_val) in &ours.properties {
            let base_val = base.properties.get(key);
            let theirs_val = theirs.properties.get(key);
            
            match (base_val, theirs_val) {
                (Some(b), Some(t)) if b == ours_val => {
                    merged_properties.insert(key.clone(), t.clone());
                }
                (Some(b), Some(t)) if b == t => {
                    merged_properties.insert(key.clone(), ours_val.clone());
                }
                (Some(_), Some(t)) if ours_val == t => {
                    merged_properties.insert(key.clone(), ours_val.clone());
                }
                (Some(_), Some(_)) => {
                    return Ok(None);
                }
                (None, None) | (Some(_), None) | (None, Some(_)) => {
                    merged_properties.insert(key.clone(), ours_val.clone());
                }
            }
        }
        
        for (key, theirs_val) in &theirs.properties {
            if !ours.properties.contains_key(key) && !base.properties.contains_key(key) {
                merged_properties.insert(key.clone(), theirs_val.clone());
            }
        }
        
        Ok(Some(ModelElement {
            id: base.id.clone(),
            element_type: base.element_type.clone(),
            properties: merged_properties,
            file_path: ours.file_path.clone(),
        }))
    }
    
    fn merge_lists(
        &self,
        _base: &ModelElement,
        _ours: &ModelElement,
        _theirs: &ModelElement,
    ) -> Result<Option<ModelElement>, CollaborationError> {
        Ok(None)
    }
    
    fn merge_traceability(
        &self,
        base: &ModelElement,
        ours: &ModelElement,
        theirs: &ModelElement,
    ) -> Result<Option<ModelElement>, CollaborationError> {
        let mut merged = base.clone();
        
        if let (Some(ours_traces), Some(theirs_traces)) = (
            ours.properties.get("traces"),
            theirs.properties.get("traces"),
        ) {
            if let (Some(ours_arr), Some(theirs_arr)) = (
                ours_traces.as_array(),
                theirs_traces.as_array(),
            ) {
                let mut combined: HashSet<String> = HashSet::new();
                
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
                    .map(|s| serde_json::Value::String(s))
                    .collect();
                
                merged.properties.insert("traces".to_string(), serde_json::Value::Array(merged_traces));
            }
        }
        
        Ok(Some(merged))
    }
}
