//! Semantic model diff — compares two models by STABLE IDENTITY, not text.
//!
//! Because every element's UUID is deterministic from its id, two versions
//! of a model can be compared structurally: moving a block or reformatting
//! a file produces an EMPTY diff; renaming an element (same id, new name)
//! is a modification, not a remove+add. This is what turns a model review
//! into a reviewable change list instead of a textual diff of `.arc` files.

use super::semantic::SemanticModel;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ElementRef {
    pub id: String,
    pub name: String,
    pub element_type: String,
    pub uuid: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FieldChange {
    pub field: String,
    pub old: String,
    pub new: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModifiedElement {
    #[serde(flatten)]
    pub element: ElementRef,
    pub changes: Vec<FieldChange>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TraceRef {
    pub from: String,
    pub trace_type: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct DiffReport {
    pub added: Vec<ElementRef>,
    pub removed: Vec<ElementRef>,
    pub modified: Vec<ModifiedElement>,
    pub traces_added: Vec<TraceRef>,
    pub traces_removed: Vec<TraceRef>,
}

impl DiffReport {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty()
            && self.removed.is_empty()
            && self.modified.is_empty()
            && self.traces_added.is_empty()
            && self.traces_removed.is_empty()
    }
}

fn push_change(changes: &mut Vec<FieldChange>, field: &str, old: &str, new: &str) {
    if old != new {
        changes.push(FieldChange {
            field: field.to_string(),
            old: old.to_string(),
            new: new.to_string(),
        });
    }
}

/// Field-level changes for one element id present in both models.
fn element_changes(old: &SemanticModel, new: &SemanticModel, id: &str) -> Vec<FieldChange> {
    let mut changes = Vec::new();

    let old_info = &old.all_elements[id];
    let new_info = &new.all_elements[id];
    push_change(&mut changes, "name", &old_info.name, &new_info.name);
    push_change(&mut changes, "type", &old_info.element_type, &new_info.element_type);

    let old_req = old.requirements.iter().find(|r| r.id == id);
    let new_req = new.requirements.iter().find(|r| r.id == id);
    if let (Some(o), Some(n)) = (old_req, new_req) {
        push_change(&mut changes, "description", &o.description, &n.description);
        push_change(&mut changes, "priority", &o.priority, &n.priority);
        push_change(
            &mut changes,
            "safety_level",
            o.safety_level.as_deref().unwrap_or(""),
            n.safety_level.as_deref().unwrap_or(""),
        );
        push_change(
            &mut changes,
            "category",
            o.category.as_deref().unwrap_or(""),
            n.category.as_deref().unwrap_or(""),
        );
    }

    let old_comp = old.components.iter().find(|c| c.id == id);
    let new_comp = new.components.iter().find(|c| c.id == id);
    if let (Some(o), Some(n)) = (old_comp, new_comp) {
        push_change(
            &mut changes,
            "safety_level",
            o.safety_level.as_deref().unwrap_or(""),
            n.safety_level.as_deref().unwrap_or(""),
        );
        push_change(&mut changes, "level", &o.level, &n.level);
        let old_functions = o.functions.join(", ");
        let new_functions = n.functions.join(", ");
        push_change(&mut changes, "functions", &old_functions, &new_functions);
    }

    changes
}

pub fn diff_models(old: &SemanticModel, new: &SemanticModel) -> DiffReport {
    let mut report = DiffReport::default();

    let as_ref = |model: &SemanticModel, id: &str| -> ElementRef {
        let info = &model.all_elements[id];
        ElementRef {
            id: info.id.clone(),
            name: info.name.clone(),
            element_type: info.element_type.clone(),
            uuid: info.uuid.clone(),
        }
    };

    // Deterministic ordering: sort ids so the report is stable.
    let mut old_ids: Vec<&String> = old.all_elements.keys().collect();
    old_ids.sort();
    let mut new_ids: Vec<&String> = new.all_elements.keys().collect();
    new_ids.sort();

    for id in &new_ids {
        if !old.all_elements.contains_key(*id) {
            report.added.push(as_ref(new, id));
        }
    }
    for id in &old_ids {
        if !new.all_elements.contains_key(*id) {
            report.removed.push(as_ref(old, id));
        }
    }
    for id in &old_ids {
        if new.all_elements.contains_key(*id) {
            let changes = element_changes(old, new, id);
            if !changes.is_empty() {
                report.modified.push(ModifiedElement {
                    element: as_ref(new, id),
                    changes,
                });
            }
        }
    }

    // Traces by identity (from, type, to); rationale changes are ignored —
    // they don't alter the traceability graph.
    let key = |t: &super::semantic::TraceInfo| (t.from.clone(), t.trace_type.clone(), t.to.clone());
    let old_traces: std::collections::HashSet<_> = old.traces.iter().map(key).collect();
    let new_traces: std::collections::HashSet<_> = new.traces.iter().map(key).collect();
    let mut traces_added: Vec<_> = new_traces.difference(&old_traces).cloned().collect();
    traces_added.sort();
    let mut traces_removed: Vec<_> = old_traces.difference(&new_traces).cloned().collect();
    traces_removed.sort();
    report.traces_added = traces_added
        .into_iter()
        .map(|(from, trace_type, to)| TraceRef { from, trace_type, to })
        .collect();
    report.traces_removed = traces_removed
        .into_iter()
        .map(|(from, trace_type, to)| TraceRef { from, trace_type, to })
        .collect();

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{Compiler, CompilerConfig};

    fn compile(source: &str) -> SemanticModel {
        Compiler::new(CompilerConfig::default())
            .compile_string(source)
            .expect("compiles")
            .semantic_model
    }

    const BASE: &str = r#"
model Demo {
}

requirements {
  req "REQ-001" "Range" { description: "Detect at 150 m" priority: "High" }
  req "REQ-002" "Latency" { description: "React in 100 ms" priority: "High" }
}

architecture logical {
  component "Controller" { id: "LC-001" function "compute" }
}

trace "LC-001" satisfies "REQ-001" { rationale: "x" }
"#;

    #[test]
    fn reordering_blocks_is_an_empty_diff() {
        let reordered = r#"
model Demo {
}

architecture logical {
  component "Controller" { id: "LC-001" function "compute" }
}

requirements {
  req "REQ-002" "Latency" { description: "React in 100 ms" priority: "High" }
  req "REQ-001" "Range" { description: "Detect at 150 m" priority: "High" }
}

trace "LC-001" satisfies "REQ-001" { rationale: "reworded rationale" }
"#;
        let report = diff_models(&compile(BASE), &compile(reordered));
        assert!(report.is_empty(), "moving blocks must not be a change: {report:?}");
    }

    #[test]
    fn rename_is_a_modification_not_remove_add() {
        let renamed = BASE.replace(
            "component \"Controller\" { id: \"LC-001\"",
            "component \"Brake Controller\" { id: \"LC-001\"",
        );
        let report = diff_models(&compile(BASE), &compile(&renamed));
        assert!(report.added.is_empty() && report.removed.is_empty());
        assert_eq!(report.modified.len(), 1);
        let modified = &report.modified[0];
        assert_eq!(modified.element.id, "LC-001");
        assert_eq!(modified.changes[0].field, "name");
        assert_eq!(modified.changes[0].new, "Brake Controller");
    }

    #[test]
    fn added_requirement_and_trace_are_reported() {
        let extended = BASE.replace(
            "trace \"LC-001\" satisfies \"REQ-001\" { rationale: \"x\" }",
            "trace \"LC-001\" satisfies \"REQ-001\" { rationale: \"x\" }\ntrace \"LC-001\" satisfies \"REQ-002\" { rationale: \"y\" }",
        );
        let report = diff_models(&compile(BASE), &compile(&extended));
        assert!(report.added.is_empty() && report.removed.is_empty() && report.modified.is_empty());
        assert_eq!(report.traces_added.len(), 1);
        assert_eq!(report.traces_added[0].to, "REQ-002");
    }

    #[test]
    fn description_change_is_field_level() {
        let changed = BASE.replace("Detect at 150 m", "Detect at 200 m");
        let report = diff_models(&compile(BASE), &compile(&changed));
        assert_eq!(report.modified.len(), 1);
        let change = &report.modified[0].changes[0];
        assert_eq!(change.field, "description");
        assert_eq!(change.old, "Detect at 150 m");
        assert_eq!(change.new, "Detect at 200 m");
    }
}
