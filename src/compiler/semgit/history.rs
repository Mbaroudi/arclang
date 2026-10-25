//! The history of one element: the commits that changed it, found by the
//! semantic diff between successive revisions of the model, not by line blame.

use super::git::{self, Commit};
use super::rich::{self, ElementChange, RelationChange, Source};
use crate::compiler::elements;
use crate::compiler::semantic_diff;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct Entry {
    #[serde(flatten)]
    pub commit: Commit,
    /// `added`, `removed` or `modified`.
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub renamed: Option<rich::Transition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moved: Option<rich::Transition>,
    pub changes: Vec<crate::compiler::semantic_diff::FieldChange>,
    /// The relationships of the element that changed in this commit.
    pub relationships: Vec<RelationChange>,
}

#[derive(Debug, Clone, Serialize)]
pub struct History {
    pub model: String,
    pub element: String,
    pub id: String,
    pub name: String,
    pub entries: Vec<Entry>,
    /// Revisions that could not be compared (the model did not compile there).
    pub skipped: Vec<String>,
}

/// The id of the element `reference` names (an id or a unique name), looked
/// for in the model now, else in the history.
fn resolve(result: &crate::compiler::CompilationResult, reference: &str) -> Option<(String, String)> {
    let graph = elements::build(&result.ast, &result.semantic_model);
    let by_id = graph.elements.iter().find(|e| e.id == reference);
    let found = by_id.or_else(|| {
        let named: Vec<_> = graph.elements.iter().filter(|e| e.name == reference).collect();
        (named.len() == 1).then(|| named[0])
    });
    found.map(|e| (e.id.clone(), e.name.clone()))
}

fn touches(diff: &semantic_diff::DiffReport, id: &str) -> Option<(&'static str, Option<ElementChange>)> {
    let refs = |e: &semantic_diff::ElementRef, status: &'static str| ElementChange { id: e.id.clone(), uuid: e.uuid.clone(), name: e.name.clone(), kind: e.element_type.clone(), status, renamed: None, moved: None, changes: Vec::new() };
    if let Some(e) = diff.added.iter().find(|e| e.id == id) {
        return Some(("added", Some(refs(e, "added"))));
    }
    if let Some(e) = diff.removed.iter().find(|e| e.id == id) {
        return Some(("removed", Some(refs(e, "removed"))));
    }
    if let Some(m) = diff.modified.iter().find(|m| m.element.id == id) {
        let take = |field: &str| m.changes.iter().find(|c| c.field == field).map(|c| rich::Transition { from: c.old.clone(), to: c.new.clone() });
        let mut change = refs(&m.element, "modified");
        change.renamed = take("name");
        change.moved = take("owner");
        change.changes = m.changes.iter().filter(|c| c.field != "name" && c.field != "owner").cloned().collect();
        return Some(("modified", Some(change)));
    }
    None
}

fn relationships_of(diff: &semantic_diff::DiffReport, id: &str) -> Vec<RelationChange> {
    let all = rich_relations(diff);
    all.into_iter().filter(|r| r.source == id || r.target == id).collect()
}

fn rich_relations(diff: &semantic_diff::DiffReport) -> Vec<RelationChange> {
    rich::relation_changes_of(diff)
}

pub fn history(model: &Path, reference: &str, limit: usize) -> Result<History, String> {
    let located = git::locate(model)?;
    let current = rich::load(model, &Source::Worktree).or_else(|_| rich::load(model, &Source::Git("HEAD".into())))?;
    let files = crate::Compiler::files_of(model).unwrap_or_default();
    let top = &located.top;
    let mut pathspecs: Vec<String> = files
        .iter()
        .filter_map(|f| f.strip_prefix(top).ok().map(|r| r.to_string_lossy().replace('\\', "/")))
        .collect();
    if pathspecs.is_empty() {
        pathspecs.push(located.relative.clone());
    }
    let (id, name) = resolve(&current.result, reference).unwrap_or_else(|| (reference.to_string(), reference.to_string()));

    // The commits that touched the model or what it imports, newest first.
    let mut args: Vec<String> = vec!["log".into(), "--format=%H%x1f%h%x1f%an%x1f%aI%x1f%s".into(), "HEAD".into(), "--".into()];
    args.extend(pathspecs);
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let log = git::run(top, &refs)?;
    let commits: Vec<Commit> = log
        .lines()
        .filter_map(|line| {
            let mut p = line.split('\u{1f}');
            Some(Commit { hash: p.next()?.into(), short: p.next()?.into(), author: p.next()?.into(), date: p.next()?.into(), subject: p.next().unwrap_or("").into() })
        })
        .collect();

    let mut entries = Vec::new();
    let mut skipped = Vec::new();
    let mut known = false;
    let mut newer: Option<rich::Loaded> = None;
    for (index, commit) in commits.iter().enumerate() {
        if entries.len() >= limit {
            break;
        }
        let new = match newer.take() {
            Some(loaded) => Some(loaded),
            None => rich::load(model, &Source::Git(commit.hash.clone())).map_err(|e| skipped.push(format!("{}: {e}", commit.short))).ok(),
        };
        let older = commits.get(index + 1).map(|c| (c, rich::load(model, &Source::Git(c.hash.clone()))));
        let (older_loaded, older_failed) = match older {
            Some((c, Ok(loaded))) => (Some(loaded), {
                let _ = c;
                None
            }),
            Some((c, Err(e))) => (None, Some(format!("{}: {e}", c.short))),
            None => (None, None),
        };
        if let Some(message) = older_failed {
            skipped.push(message);
        }
        if let Some(new) = &new {
            let diff = match &older_loaded {
                Some(old) => semantic_diff::diff_compiled(&old.result, &new.result),
                None => {
                    // The first revision of the model: everything in it is new.
                    let empty = crate::Compiler::new(crate::CompilerConfig::default()).compile_string("model Empty {\n}\n").map_err(|e| e.to_string())?;
                    semantic_diff::diff_compiled(&empty, &new.result)
                }
            };
            if let Some((status, change)) = touches(&diff, &id) {
                known = true;
                let change = change.expect("a change");
                entries.push(Entry { commit: commit.clone(), status, renamed: change.renamed, moved: change.moved, changes: change.changes, relationships: relationships_of(&diff, &id) });
            } else {
                let rels = relationships_of(&diff, &id);
                if !rels.is_empty() {
                    known = true;
                    entries.push(Entry { commit: commit.clone(), status: "modified", renamed: None, moved: None, changes: Vec::new(), relationships: rels });
                }
            }
        }
        newer = older_loaded;
    }
    if !known && entries.is_empty() && resolve(&current.result, reference).is_none() && commits.is_empty() {
        return Err(format!("no commit of {} is known", model.display()));
    }
    Ok(History { model: model.display().to_string(), element: reference.to_string(), id, name, entries, skipped })
}

pub fn text(h: &History) -> String {
    let mut out = format!("History of '{}' [{}] in {}\n", h.name, h.id, h.model);
    if h.entries.is_empty() {
        out.push_str("  No commit changed this element.\n");
    }
    for e in &h.entries {
        out.push_str(&format!("\n{} {}  {}  {}\n  {}\n", e.commit.short, e.commit.date.get(..10).unwrap_or(&e.commit.date), e.commit.author, e.status, e.commit.subject));
        if let Some(r) = &e.renamed {
            out.push_str(&format!("      renamed: \"{}\" -> \"{}\"\n", r.from, r.to));
        }
        if let Some(m) = &e.moved {
            out.push_str(&format!("      moved: {} -> {}\n", m.from, m.to));
        }
        for c in &e.changes {
            out.push_str(&format!("      {}: \"{}\" -> \"{}\"\n", c.field, c.old, c.new));
        }
        for r in &e.relationships {
            out.push_str(&format!("      {} {} {}: {} -> {}\n", r.status, r.category, r.kind, r.source, r.target));
        }
    }
    for s in &h.skipped {
        out.push_str(&format!("\n  (skipped: {s})\n"));
    }
    out
}
