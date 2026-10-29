//! The rich semantic diff: elements grouped by layer and kind, relationships,
//! and the consequences of the change on the gate, the coverage, the
//! constraints and the budgets.

use super::git::{self, Scratch};
use super::layout_diff::{self, LayoutChange};
use crate::compiler::diagram::layout_file::sidecar_path;
use crate::compiler::impact::graph::layer_code;
use crate::compiler::impact::whatif::{self, BudgetState, ConstraintState, Coverage, CoverageCounts, CoverageDelta, VerdictState};
use crate::compiler::impact::{GateState, Options};
use crate::compiler::metamodel::Metamodel;
use crate::compiler::semantic_diff::{self, DiffReport, FieldChange};
use crate::compiler::CompilationResult;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Where a version of a model comes from.
#[derive(Debug, Clone)]
pub enum Source {
    /// The file as committed at a revision (`HEAD`, `main`, a tag, a hash).
    Git(String),
    /// The file on disk.
    Worktree,
    /// Another file.
    File(PathBuf),
}

impl Source {
    pub fn parse(text: &str) -> Source {
        if text.eq_ignore_ascii_case("WORKTREE") {
            Source::Worktree
        } else {
            Source::Git(text.to_string())
        }
    }

    pub fn label(&self) -> String {
        match self {
            Source::Git(rev) => rev.clone(),
            Source::Worktree => "WORKTREE".to_string(),
            Source::File(path) => path.display().to_string(),
        }

    }
}

/// A version of a model, compiled, with its layout file.
pub struct Loaded {
    pub label: String,
    pub commit: Option<String>,
    pub result: CompilationResult,
    /// The model file as it is in this version (the text, imports not included).
    pub text: String,
    pub layout: Option<Value>,
    /// Where it was compiled; a scratch directory for a revision.
    pub path: PathBuf,
    _scratch: Option<Scratch>,
}

fn compile(path: &Path) -> Result<CompilationResult, String> {
    crate::Compiler::new(crate::CompilerConfig::default()).compile_file(path).map_err(|e| e.to_string())
}

fn read_layout(model: &Path) -> Option<Value> {
    let text = std::fs::read_to_string(sidecar_path(model)).ok()?;
    serde_json::from_str(&text).ok()
}

/// Load a model at a source. `model` is the file in the working tree.
pub fn load(model: &Path, source: &Source) -> Result<Loaded, String> {
    match source {
        Source::Worktree => {
            let result = compile(model).map_err(|e| format!("the model does not compile: {e}"))?;
            let text = std::fs::read_to_string(model).map_err(|e| format!("{}: {e}", model.display()))?;
            Ok(Loaded { label: "WORKTREE".into(), commit: None, result, text, layout: read_layout(model), path: model.to_path_buf(), _scratch: None })
        }
        Source::File(path) => {
            let result = compile(path).map_err(|e| format!("{}: {e}", path.display()))?;
            let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
            Ok(Loaded { label: path.display().to_string(), commit: None, result, text, layout: read_layout(path), path: path.clone(), _scratch: None })
        }
        Source::Git(revision) => {
            let located = git::locate(model)?;
            let commit = git::commit_of(&located.top, revision)?;
            let text = git::show(&located.top, &commit, &located.relative).ok_or_else(|| format!("{} does not exist at {revision}", located.relative))?;
            let scratch = Scratch::new("rev")?;
            git::export_tree(&located.top, &commit, &scratch.0)?;
            let path = scratch.0.join(&located.relative);
            let mut result = compile(&path).map_err(|e| format!("the model at {revision} does not compile: {}", e.replace(&scratch.0.display().to_string(), &format!("<{revision}>"))))?;
            // Its warnings and verdicts name the file the user knows, not the scratch copy.
            result.name_file_as(&path, &model.display().to_string());
            result.name_file_as(&scratch.0, &format!("<{revision}>"));
            let layout = read_layout(&path);
            Ok(Loaded { label: revision.clone(), commit: Some(commit), result, text, layout, path, _scratch: Some(scratch) })
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ElementChange {
    pub id: String,
    pub uuid: String,
    pub name: String,
    pub kind: String,
    /// `added`, `removed` or `modified` (a rename or a move alone is a modification).
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub renamed: Option<Transition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moved: Option<Transition>,
    pub changes: Vec<FieldChange>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Transition {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct KindGroup {
    pub kind: String,
    pub elements: Vec<ElementChange>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Counts {
    pub added: usize,
    pub removed: usize,
    pub modified: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct LayerGroup {
    pub layer: String,
    pub label: String,
    pub counts: Counts,
    pub kinds: Vec<KindGroup>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RelationChange {
    /// `added`, `removed`, `retargeted` or `modified`.
    pub status: &'static str,
    /// `trace`, `exchange`, `allocation`, `deployment` or `other`.
    pub category: &'static str,
    pub kind: String,
    pub name: String,
    pub source: String,
    pub target: String,
    /// For a retargeted relationship: the end it had.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub was: Option<Transition>,
    pub changes: Vec<FieldChange>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Consequences {
    pub gate_before: GateState,
    pub gate_after: GateState,
    pub new_blockers: Vec<String>,
    pub removed_blockers: Vec<String>,
    /// Requirements that were satisfied and no longer are.
    pub newly_unsatisfied: Vec<CoverageDelta>,
    /// Requirements that were verified and no longer are.
    pub newly_unverified: Vec<CoverageDelta>,
    pub coverage: Vec<CoverageDelta>,
    pub coverage_before: CoverageCounts,
    pub coverage_after: CoverageCounts,
    pub constraints: Vec<ConstraintState>,
    pub budgets: Vec<BudgetState>,
    pub verdicts: Vec<VerdictState>,
    pub impacted: usize,
    pub impacted_blockers: usize,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Summary {
    pub elements_added: usize,
    pub elements_removed: usize,
    pub elements_modified: usize,
    pub renamed: usize,
    pub moved: usize,
    pub relationships_added: usize,
    pub relationships_removed: usize,
    pub relationships_changed: usize,
    pub layout_entries: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RichDiff {
    pub model: String,
    pub from: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_commit: Option<String>,
    pub model_changed: bool,
    /// Only the layout file differs: nothing the model means has changed.
    pub layout_only: bool,
    pub summary: Summary,
    pub layers: Vec<LayerGroup>,
    pub relationships: Vec<RelationChange>,
    pub consequences: Consequences,
    pub layout: Vec<LayoutChange>,
}

const LAYER_ORDER: [&str; 7] = ["OA", "SA", "LA", "PA", "EPBS", "Data", "Transverse"];

fn layer_label(code: &str) -> &'static str {
    match code {
        "OA" => "Operational Analysis",
        "SA" => "System Analysis",
        "LA" => "Logical Architecture",
        "PA" => "Physical Architecture",
        "EPBS" => "EPBS",
        "Data" => "Data model",
        _ => "Transverse",
    }
}

fn layer_of(metamodel: &Metamodel, kind: &str) -> String {
    metamodel.kind(kind).map(|k| layer_code(k.layer).to_string()).unwrap_or_else(|| "Transverse".to_string())
}

fn group_elements(diff: &DiffReport, metamodel: &Metamodel) -> Vec<LayerGroup> {
    let mut by_layer: BTreeMap<usize, (Counts, BTreeMap<String, Vec<ElementChange>>)> = BTreeMap::new();
    let mut push = |kind: &str, change: ElementChange| {
        let layer = layer_of(metamodel, kind);
        let rank = LAYER_ORDER.iter().position(|l| *l == layer).unwrap_or(LAYER_ORDER.len() - 1);
        let entry = by_layer.entry(rank).or_default();
        match change.status {
            "added" => entry.0.added += 1,
            "removed" => entry.0.removed += 1,
            _ => entry.0.modified += 1,
        }
        entry.1.entry(kind.to_string()).or_default().push(change);
    };
    for e in &diff.added {
        push(&e.element_type, ElementChange { id: e.id.clone(), uuid: e.uuid.clone(), name: e.name.clone(), kind: e.element_type.clone(), status: "added", renamed: None, moved: None, changes: Vec::new() });
    }
    for e in &diff.removed {
        push(&e.element_type, ElementChange { id: e.id.clone(), uuid: e.uuid.clone(), name: e.name.clone(), kind: e.element_type.clone(), status: "removed", renamed: None, moved: None, changes: Vec::new() });
    }
    for m in &diff.modified {
        let take = |field: &str| m.changes.iter().find(|c| c.field == field).map(|c| Transition { from: c.old.clone(), to: c.new.clone() });
        let renamed = take("name");
        let moved = take("owner");
        let changes: Vec<FieldChange> = m.changes.iter().filter(|c| c.field != "name" && c.field != "owner").cloned().collect();
        push(&m.element.element_type, ElementChange { id: m.element.id.clone(), uuid: m.element.uuid.clone(), name: m.element.name.clone(), kind: m.element.element_type.clone(), status: "modified", renamed, moved, changes });
    }
    by_layer
        .into_iter()
        .map(|(rank, (counts, kinds))| {
            let layer = LAYER_ORDER[rank].to_string();
            LayerGroup {
                label: layer_label(&layer).to_string(),
                layer,
                counts,
                kinds: kinds.into_iter().map(|(kind, mut elements)| {
                    elements.sort_by(|a, b| a.id.cmp(&b.id));
                    KindGroup { kind, elements }
                }).collect(),
            }
        })
        .collect()
}

fn category(kind: &str) -> &'static str {
    let lower = kind.to_ascii_lowercase();
    if lower.contains("exchange") || lower.contains("link") || lower.contains("interface") {
        "exchange"
    } else if lower.contains("alloc") || lower == "performs" {
        "allocation"
    } else if lower.contains("deploy") {
        "deployment"
    } else {
        "other"
    }
}

pub fn relation_changes_of(diff: &DiffReport) -> Vec<RelationChange> {
    let mut out = Vec::new();
    let mut removed: Vec<RelationChange> = Vec::new();
    let mut added: Vec<RelationChange> = Vec::new();
    for t in &diff.traces_added {
        let category = if t.trace_type.to_ascii_lowercase().contains("alloc") { "allocation" } else { "trace" };
        added.push(RelationChange { status: "added", category, kind: t.trace_type.clone(), name: String::new(), source: t.from.clone(), target: t.to.clone(), was: None, changes: Vec::new() });
    }
    for t in &diff.traces_removed {
        let category = if t.trace_type.to_ascii_lowercase().contains("alloc") { "allocation" } else { "trace" };
        removed.push(RelationChange { status: "removed", category, kind: t.trace_type.clone(), name: String::new(), source: t.from.clone(), target: t.to.clone(), was: None, changes: Vec::new() });
    }
    for r in &diff.relationships_added {
        added.push(RelationChange { status: "added", category: category(&r.kind), kind: r.kind.clone(), name: r.name.clone(), source: r.source.clone(), target: r.target.clone(), was: None, changes: Vec::new() });
    }
    for r in &diff.relationships_removed {
        removed.push(RelationChange { status: "removed", category: category(&r.kind), kind: r.kind.clone(), name: r.name.clone(), source: r.source.clone(), target: r.target.clone(), was: None, changes: Vec::new() });
    }
    // A relationship of the same kind and name that kept one end and changed the other is retargeted.
    let mut retargeted = Vec::new();
    removed.retain(|gone| {
        let twin = added.iter().position(|new| {
            new.kind == gone.kind && new.name == gone.name && new.category == gone.category && ((new.source == gone.source) != (new.target == gone.target))
        });
        match twin {
            Some(at) => {
                let new = added.remove(at);
                let (from, to) = if new.source == gone.source { (gone.target.clone(), new.target.clone()) } else { (gone.source.clone(), new.source.clone()) };
                retargeted.push(RelationChange { status: "retargeted", was: Some(Transition { from, to }), ..new });
                false
            }
            None => true,
        }
    });
    out.extend(added);
    out.extend(removed);
    out.extend(retargeted);
    for t in &diff.traces_modified {
        out.push(RelationChange { status: "modified", category: "trace", kind: t.trace.trace_type.clone(), name: String::new(), source: t.trace.from.clone(), target: t.trace.to.clone(), was: None, changes: t.changes.clone() });
    }
    for r in &diff.relationships_modified {
        out.push(RelationChange { status: "modified", category: category(&r.relationship.kind), kind: r.relationship.kind.clone(), name: r.relationship.name.clone(), source: r.relationship.source.clone(), target: r.relationship.target.clone(), was: None, changes: r.changes.clone() });
    }
    out
}

fn consequences(old: &CompilationResult, new: &CompilationResult, diff: &DiffReport, options: &Options) -> Result<Consequences, String> {
    let report = whatif::compare(old, new, diff.clone(), Vec::new(), options)?;
    let w = report.what_if.ok_or("the comparison has no consequences")?;
    let verified = |c: &Coverage| c.status == "covered";
    let satisfied = |c: &Coverage| matches!(c.status.as_str(), "covered" | "unverified" | "failing");
    let newly_unsatisfied = w.coverage.iter().filter(|d| satisfied(&d.before) && !satisfied(&d.after) && d.after.status != "absent").cloned().collect();
    let newly_unverified = w.coverage.iter().filter(|d| verified(&d.before) && !verified(&d.after) && d.after.status != "absent").cloned().collect();
    Ok(Consequences {
        gate_before: w.gate_before,
        gate_after: w.gate_after,
        new_blockers: w.new_blockers,
        removed_blockers: w.removed_blockers,
        newly_unsatisfied,
        newly_unverified,
        coverage: w.coverage,
        coverage_before: w.coverage_before,
        coverage_after: w.coverage_after,
        constraints: w.constraints.into_iter().filter(|c| c.flipped).collect(),
        budgets: w.budgets.into_iter().filter(|b| b.before_ms != b.after_ms || b.ok_before != b.ok_after).collect(),
        verdicts: w.verdicts,
        impacted: report.summary.total,
        impacted_blockers: report.summary.blockers,
    })
}

/// The diff of two loaded versions of a model.
pub fn compare(model: &Path, old: &Loaded, new: &Loaded, gate_standard: &str) -> Result<RichDiff, String> {
    let diff = semantic_diff::diff_compiled(&old.result, &new.result);
    let metamodel = Metamodel::current();
    let options = Options { gate_standard: gate_standard.to_string(), ..Options::default() };
    let consequences = consequences(&old.result, &new.result, &diff, &options)?;
    let layers = group_elements(&diff, &metamodel);
    let relationships = relation_changes_of(&diff);
    let layout = layout_diff::diff(old.layout.as_ref(), new.layout.as_ref());
    let model_changed = !diff.is_empty();
    let summary = Summary {
        elements_added: diff.added.len(),
        elements_removed: diff.removed.len(),
        elements_modified: diff.modified.len(),
        renamed: diff.modified.iter().filter(|m| m.changes.iter().any(|c| c.field == "name")).count(),
        moved: diff.modified.iter().filter(|m| m.changes.iter().any(|c| c.field == "owner")).count(),
        relationships_added: relationships.iter().filter(|r| r.status == "added").count(),
        relationships_removed: relationships.iter().filter(|r| r.status == "removed").count(),
        relationships_changed: relationships.iter().filter(|r| r.status == "retargeted" || r.status == "modified").count(),
        layout_entries: layout.len(),
    };
    Ok(RichDiff {
        model: model.display().to_string(),
        from: old.label.clone(),
        to: new.label.clone(),
        from_commit: old.commit.clone(),
        to_commit: new.commit.clone(),
        model_changed,
        layout_only: !model_changed && !layout.is_empty(),
        summary,
        layers,
        relationships,
        consequences,
        layout,
    })
}

/// Load both sides and compare them.
pub fn diff_sources(model: &Path, from: &Source, to: &Source, gate_standard: &str) -> Result<RichDiff, String> {
    let old = load(model, from)?;
    let new = load(model, to)?;
    compare(model, &old, &new, gate_standard)
}

/// The sequence of a value in a [`Value`] map, for the other modules.
pub fn string_of(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}
