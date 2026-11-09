//! `requirements_coverage`: per requirement, its text, what satisfies it,
//! the verification cases with their computed verdict, and what the
//! production gate says about it.

use super::context::{shown, Context};
use super::{label_of, Cell, Column, Link, Row, TableSpec};
use crate::compiler::verification::{self, Standing, Verdict};
use std::collections::{BTreeMap, BTreeSet};

pub const DEFAULT_COLUMNS: [&str; 8] = ["id", "name", "text", "priority", "satisfied_by", "verified_by", "verdict", "status"];
/// Shown by default when a verification case has evidence (a simulation).
const EVIDENCE: &str = "evidence";
const COMPUTED: [(&str, &str); 13] = [
    ("id", "text"),
    ("name", "text"),
    ("text", "text"),
    ("priority", "text"),
    ("safety_level", "text"),
    ("satisfied_by", "links"),
    ("verified_by", "links"),
    ("verdict", "status"),
    ("evidence", "text"),
    ("refined_by", "links"),
    ("mitigates", "links"),
    ("gate", "status"),
    ("status", "status"),
];

fn column(key: &str, ty: &str) -> Column {
    let label = match key {
        "id" => "Req id".to_string(),
        "name" => "Name".to_string(),
        "text" => "Requirement text".to_string(),
        "verdict" => "Verdict".to_string(),
        "evidence" => "Key values".to_string(),
        "gate" => "Gate".to_string(),
        other => label_of(other),
    };
    Column { key: key.to_string(), label, ty: ty.to_string() }
}

pub fn compute(context: &Context, spec: &TableSpec) -> Result<(Vec<Column>, Vec<Column>, Vec<Row>), String> {
    context.check_scope(spec.scope.as_deref())?;
    let requirements: Vec<usize> = (0..context.graph.elements.len())
        .filter(|&i| context.element(i).kind == spec.row_type.as_deref().unwrap_or("Requirement"))
        .filter(|&i| context.in_scope(i, spec.scope.as_deref()))
        .collect();
    // Every attribute some requirement writes is a column one may pick.
    let mut extra: BTreeSet<String> = BTreeSet::new();
    for &r in &requirements {
        extra.extend(context.element(r).attributes.keys().filter(|k| !matches!(k.as_str(), "id" | "title" | "description" | "text")).cloned());
    }
    let mut available: Vec<Column> = COMPUTED.iter().map(|(k, t)| column(k, t)).collect();
    available.extend(extra.iter().filter(|k| !COMPUTED.iter().any(|(c, _)| c == k)).map(|k| column(k, "text")));
    let keys: Vec<String> = if spec.columns.is_empty() {
        let mut keys: Vec<String> = DEFAULT_COLUMNS.iter().map(|s| s.to_string()).collect();
        if !context.evidence.is_empty() {
            keys.insert(keys.iter().position(|k| k == "verdict").map(|i| i + 1).unwrap_or(keys.len()), EVIDENCE.to_string());
        }
        keys
    } else {
        spec.columns.clone()
    };
    let columns: Vec<Column> = keys.iter().map(|k| available.iter().find(|c| &c.key == k).cloned().unwrap_or_else(|| column(k, "text"))).collect();
    let rows = requirements.iter().map(|&r| row(context, r, &keys)).collect();
    // A requirement under another (`parent:`) is a row under its row.
    let parent_of = |row: &Row| context.index_of_uuid(&row.uuid).and_then(|i| context.element(i).attributes.get("parent").and_then(|p| p.as_string()).map(str::to_string));
    Ok((columns, available, super::nest(rows, parent_of)))
}

/// The sources of the traces of `kind` ending on `target`, in graph order.
fn traced_to(context: &Context, target: &str, kind: &str) -> Vec<usize> {
    let mut out: Vec<usize> = context
        .relationships("Trace")
        .filter(|t| t.target == target && t.extra.get("traceKind").and_then(|k| k.as_str()) == Some(kind))
        .filter_map(|t| context.index_of_uuid(&t.source))
        .collect();
    out.dedup();
    out
}

fn row(context: &Context, index: usize, keys: &[String]) -> Row {
    let element = context.element(index);
    let uuid = element.uuid.clone();
    // What satisfies it, what verifies it and where it stands: as the gate reads it.
    let status = verification::requirement_status(&context.result.ast, &context.result.semantic_model, &element.id);
    let standing = status.standing();
    let satisfied: Vec<usize> = status.satisfied_by.iter().filter_map(|s| context.resolve(s)).collect();
    let tests: Vec<(usize, &Verdict)> = status.cases.iter().filter_map(|(case, verdict)| context.resolve(&case.id).map(|index| (index, verdict))).collect();
    let test_links: Vec<Link> = tests
        .iter()
        .map(|&(index, verdict)| {
            let mut link = context.link(index);
            link.verdict = Some(verdict.word().to_string());
            link
        })
        .collect();
    let (verdict_text, verdict_status) = match standing {
        Standing::Unverified if tests.is_empty() => ("none", "gap"),
        Standing::Unverified => ("unverified", "gap"),
        Standing::Fail => ("fail", "fail"),
        Standing::Inconclusive => ("inconclusive", "warn"),
        Standing::Pending => ("pending", "warn"),
        Standing::Pass => ("pass", "ok"),
    };
    let blockers = context.blockers_of(index);

    let mut issues = Vec::new();
    if satisfied.is_empty() {
        issues.push("satisfied by no element".to_string());
    }
    if tests.is_empty() {
        issues.push("verified by no test case".to_string());
    }
    for (link, (_, verdict)) in test_links.iter().zip(&tests) {
        let why = verdict.why().map(|w| format!(": {w}")).unwrap_or_default();
        match verdict {
            Verdict::Fail(_) => issues.push(format!("{} fails{why}", link.name)),
            Verdict::Inconclusive(_) => issues.push(format!("{} is inconclusive{why}", link.name)),
            Verdict::Pending => issues.push(format!("{} has no verdict", link.name)),
            Verdict::Pass => {}
        }
    }
    // The gate's blockers that say something more than the lines above.
    const RESTATED: [&str; 5] = ["satisfied by no", "no verification case", "cases fail", "cases are inconclusive", "cases have no verdict"];
    for blocker in &blockers {
        if !issues.iter().any(|i| blocker.contains(i.as_str())) && !RESTATED.iter().any(|said| blocker.contains(said)) {
            issues.push(blocker.clone());
        }
    }
    // A row is covered exactly when the gate has nothing to say of it: `is:gap` lists what it still blocks on.
    let status = if standing == Standing::Fail {
        "fail"
    } else if satisfied.is_empty() || standing != Standing::Pass || !blockers.is_empty() {
        "gap"
    } else {
        "ok"
    };
    let status_text = match status {
        "fail" => "Failing",
        "gap" if satisfied.is_empty() && tests.is_empty() => "Not covered",
        "gap" if satisfied.is_empty() => "Not satisfied",
        "gap" if tests.is_empty() => "Not verified",
        "gap" if standing == Standing::Inconclusive => "Verdict inconclusive",
        "gap" if standing == Standing::Pending => "Verdict pending",
        "gap" => "Gate blocked",
        _ => "Covered",
    };

    let text_key = if element.attributes.contains_key("text") { "text" } else { "description" };
    let mut cells = BTreeMap::new();
    for key in keys {
        let cell = match key.as_str() {
            "id" => Cell::text(element.id.clone()),
            "name" => {
                let edit = if element.attributes.contains_key("title") { context.attribute_edit(index, "title") } else { context.rename_edit(index) };
                Cell::text(element.name.clone()).editable(edit)
            }
            "text" => Cell::text(element.attributes.get(text_key).map(shown).unwrap_or_default()).editable(context.attribute_edit(index, text_key)),
            "satisfied_by" => {
                let cell = Cell::links(satisfied.iter().map(|&s| context.link(s)).collect());
                if satisfied.is_empty() { cell.with_status("gap") } else { cell }
            }
            "verified_by" => {
                let cell = Cell::links(test_links.clone());
                if tests.is_empty() { cell.with_status("gap") } else { cell }
            }
            "verdict" => Cell::text(verdict_text).with_status(verdict_status),
            "evidence" => Cell::text(
                test_links.iter().filter_map(|l| context.evidence.get(&l.id).map(|e| format!("{}: {e}", l.id))).collect::<Vec<_>>().join(" | "),
            ),
            "refined_by" => Cell::links(traced_to(context, &uuid, "refines").iter().map(|&s| context.link(s)).collect()),
            "mitigates" => Cell::links(
                context.relationships("Mitigation").filter(|m| m.source == uuid).filter_map(|m| context.index_of_uuid(&m.target)).map(|h| context.link(h)).collect(),
            ),
            "gate" => {
                if blockers.is_empty() {
                    Cell::text("pass").with_status("ok")
                } else {
                    Cell::text(format!("{} blocker{}", blockers.len(), if blockers.len() > 1 { "s" } else { "" })).with_status("fail")
                }
            }
            "status" => Cell::text(status_text).with_status(status),
            other => Cell::text(element.attributes.get(other).map(shown).unwrap_or_default()).editable(context.attribute_edit(index, other)),
        };
        cells.insert(key.clone(), cell);
    }
    Row { id: element.id.clone(), uuid, kind: element.kind.to_string(), name: element.name.clone(), status: status.to_string(), issues, cells, parent: None, depth: 0 }
}
