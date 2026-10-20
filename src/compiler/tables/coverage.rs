//! `requirements_coverage`: per requirement, its text, what satisfies it,
//! the verification cases with their computed verdict, and what the
//! production gate says about it.

use super::context::{shown, Context};
use super::{label_of, Cell, Column, Link, Row, TableSpec};
use std::collections::{BTreeMap, BTreeSet};

pub const DEFAULT_COLUMNS: [&str; 8] = ["id", "name", "text", "priority", "satisfied_by", "verified_by", "verdict", "status"];
const COMPUTED: [(&str, &str); 12] = [
    ("id", "text"),
    ("name", "text"),
    ("text", "text"),
    ("priority", "text"),
    ("safety_level", "text"),
    ("satisfied_by", "links"),
    ("verified_by", "links"),
    ("verdict", "status"),
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
    let keys: Vec<String> = if spec.columns.is_empty() { DEFAULT_COLUMNS.iter().map(|s| s.to_string()).collect() } else { spec.columns.clone() };
    let columns: Vec<Column> = keys.iter().map(|k| available.iter().find(|c| &c.key == k).cloned().unwrap_or_else(|| column(k, "text"))).collect();
    let rows = requirements.iter().map(|&r| row(context, r, &keys)).collect();
    Ok((columns, available, rows))
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
    let satisfied = traced_to(context, &uuid, "satisfies");
    let mut tests: Vec<usize> = context.relationships("Verification").filter(|v| v.target == uuid).filter_map(|v| context.index_of_uuid(&v.source)).collect();
    tests.extend(traced_to(context, &uuid, "verifies"));
    let mut seen = BTreeSet::new();
    tests.retain(|t| seen.insert(*t));
    let test_links: Vec<Link> = tests
        .iter()
        .map(|&t| {
            let mut link = context.link(t);
            link.verdict = Some(context.verdicts.get(&link.id).map(|(v, _)| v.clone()).unwrap_or_else(|| "pending".to_string()));
            link
        })
        .collect();
    let verdicts: Vec<&str> = test_links.iter().filter_map(|l| l.verdict.as_deref()).collect();
    let (verdict_text, verdict_status) = if verdicts.is_empty() {
        ("none", "gap")
    } else if verdicts.contains(&"fail") {
        ("fail", "fail")
    } else if verdicts.iter().all(|v| *v == "pass") {
        ("pass", "ok")
    } else {
        ("pending", "warn")
    };
    let blockers = context.blockers_of(index);

    let mut issues = Vec::new();
    if satisfied.is_empty() {
        issues.push("satisfied by no element".to_string());
    }
    if tests.is_empty() {
        issues.push("verified by no test case".to_string());
    }
    for link in test_links.iter().filter(|l| l.verdict.as_deref() == Some("fail")) {
        let why = context.verdicts.get(&link.id).and_then(|(_, why)| why.clone()).map(|w| format!(": {w}")).unwrap_or_default();
        issues.push(format!("{} fails{why}", link.name));
    }
    for blocker in &blockers {
        if !issues.iter().any(|i| blocker.contains(i.as_str())) && !blocker.contains("satisfied by no") && !blocker.contains("no verification case") && !blocker.contains("cases fail") {
            issues.push(blocker.clone());
        }
    }
    let status = if verdict_status == "fail" {
        "fail"
    } else if satisfied.is_empty() || tests.is_empty() || !blockers.is_empty() {
        "gap"
    } else if verdict_status == "warn" {
        "warn"
    } else {
        "ok"
    };
    let status_text = match status {
        "fail" => "Failing",
        "gap" if satisfied.is_empty() && tests.is_empty() => "Not covered",
        "gap" if satisfied.is_empty() => "Not satisfied",
        "gap" if tests.is_empty() => "Not verified",
        "gap" => "Gate blocked",
        "warn" => "Verdict pending",
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
    Row { id: element.id.clone(), uuid, kind: element.kind.to_string(), name: element.name.clone(), status: status.to_string(), issues, cells }
}
