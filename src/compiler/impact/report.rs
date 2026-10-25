//! An impact report as text (the terminal), Markdown, a standalone HTML
//! change-request page, tab-separated rows, and an Excel workbook written
//! by the tables' own minimal writer. The same report gives the same bytes.

use super::{ImpactReport, ImpactSeverity, Impacted};
use crate::compiler::tables::{self, Cell, Column, Row, Summary, Table, TableSpec};
use std::collections::BTreeMap;

fn seeds_text(report: &ImpactReport) -> String {
    if let Some(revision) = &report.since {
        return format!("the changes since {revision}");
    }
    if report.what_if.is_some() && report.seeds.is_empty() {
        return "the proposed change".to_string();
    }
    let names: Vec<String> = report.seeds.iter().map(|s| s.id.clone()).collect();
    if report.what_if.is_some() {
        format!("the proposed change ({})", names.join(", "))
    } else {
        names.join(", ")
    }
}

fn gate_word(passed: bool) -> &'static str {
    if passed {
        "PASS"
    } else {
        "FAIL"
    }
}

/// The one-line summary: counts and the gate.
pub fn headline(report: &ImpactReport) -> String {
    let s = &report.summary;
    let gate = match &report.what_if {
        Some(w) => format!("gate {} {} → {}", w.gate_after.standard, gate_word(w.gate_before.passed), gate_word(w.gate_after.passed)),
        None => format!("gate {} {}", report.gate.standard, gate_word(report.gate.passed)),
    };
    format!("{} impacted: {} blocker{}, {} major{}, {} minor{} · {}", s.total, s.blockers, plural(s.blockers), s.majors, plural(s.majors), s.minors, plural(s.minors), gate)
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

/// The terminal output: by distance, as `arclang impact` has always printed it.
pub fn text(report: &ImpactReport) -> String {
    let mut out = format!("Impact analysis for '{}':\n", seeds_text(report));
    out.push_str(&format!("  {}\n", headline(report)));
    if let Some(w) = &report.what_if {
        what_if_text(w, &mut out);
    }
    if report.impacted.is_empty() {
        out.push_str("  No connected elements — changing it affects nothing traced.\n");
        return out;
    }
    let mut ordered: Vec<&Impacted> = report.impacted.iter().collect();
    ordered.sort_by(|a, b| (a.distance, &a.layer, &a.id).cmp(&(b.distance, &b.layer, &b.id)));
    let mut current = 0;
    for item in ordered {
        if item.distance != current {
            current = item.distance;
            let label = if current == 1 { "directly affected" } else { "transitively affected" };
            out.push_str(&format!("\n  Distance {current} ({label}):\n"));
        }
        out.push_str(&format!(
            "    [{}] {} '{}' [{}] ({}, {}{}) — {}\n",
            item.severity.as_str(),
            item.kind,
            item.name,
            item.id,
            item.layer,
            item.category.as_str(),
            if item.removed { ", removed" } else { "" },
            if item.why.is_empty() { item.reason.clone() } else { item.why.clone() }
        ));
    }
    out.push_str(&format!("\n  Total affected elements: {}\n", report.impacted.len()));
    out
}

fn what_if_text(w: &super::WhatIf, out: &mut String) {
    for blocker in &w.new_blockers {
        out.push_str(&format!("  + new blocker: {blocker}\n"));
    }
    for blocker in &w.removed_blockers {
        out.push_str(&format!("  - blocker cleared: {blocker}\n"));
    }
    for c in w.constraints.iter().filter(|c| c.flipped) {
        out.push_str(&format!("  ~ constraint {} {} → {} ({} → {} against {})\n", c.name, holds(c.before), holds(c.after), c.left_before, c.left_after, c.right));
    }
    for b in w.budgets.iter().filter(|b| b.before_ms != b.after_ms) {
        out.push_str(&format!("  ~ budget {}: {} → {} for {}{}\n", b.name, ms(b.before_ms), ms(b.after_ms), ms(b.budget_ms), if b.ok_after { "" } else { " — EXCEEDED" }));
    }
    for c in &w.coverage {
        out.push_str(&format!("  ~ coverage {}: {} → {}\n", c.requirement, c.before.status, c.after.status));
    }
    for v in &w.verdicts {
        out.push_str(&format!("  ~ verdict {}: {} → {}{}\n", v.test, v.before, v.after, if v.stale { " (stale: to run again)" } else { "" }));
    }
}

fn holds(state: Option<bool>) -> &'static str {
    match state {
        Some(true) => "holds",
        Some(false) => "violated",
        None => "absent",
    }
}

fn ms(value: Option<f64>) -> String {
    value.map(|v| format!("{} ms", trim(v))).unwrap_or_else(|| "—".to_string())
}

fn trim(v: f64) -> String {
    if v.fract().abs() < 1e-9 {
        format!("{}", v as i64)
    } else {
        format!("{v:.1}")
    }
}

fn status_of(severity: ImpactSeverity) -> &'static str {
    match severity {
        ImpactSeverity::Blocker => "fail",
        ImpactSeverity::Major => "warn",
        ImpactSeverity::Minor => "ok",
    }
}

fn table(name: &str, columns: &[(&str, &str)], rows: Vec<Row>) -> Table {
    let columns: Vec<Column> = columns.iter().map(|(key, label)| Column { key: key.to_string(), label: label.to_string(), ty: "text".to_string() }).collect();
    let summary = Summary { rows: rows.len(), ok: 0, warn: 0, gap: 0, fail: 0, gate_standard: String::new(), gate_passed: true, gate_blockers: 0 };
    Table { name: name.to_string(), id: name.to_string(), uuid: None, kind: "impact".to_string(), declared: false, spec: TableSpec::default(), available_columns: columns.clone(), columns, rows, matrix: None, summary }
}

fn row(id: &str, status: &str, cells: Vec<(&str, Cell)>) -> Row {
    Row { id: id.to_string(), uuid: String::new(), kind: String::new(), name: id.to_string(), status: status.to_string(), issues: Vec::new(), cells: cells.into_iter().map(|(k, c)| (k.to_string(), c)).collect::<BTreeMap<_, _>>() }
}

const IMPACT_COLUMNS: [(&str, &str); 9] = [("id", "Element"), ("name", "Name"), ("kind", "Kind"), ("layer", "Layer"), ("category", "Category"), ("severity", "Severity"), ("distance", "Distance"), ("reason", "Reason"), ("why", "Why")];

fn impact_row(item: &Impacted) -> Row {
    let status = status_of(item.severity);
    row(
        &item.id,
        status,
        vec![
            ("id", Cell::text(&item.id)),
            ("name", Cell::text(&item.name)),
            ("kind", Cell::text(&item.kind)),
            ("layer", Cell::text(&item.layer)),
            ("category", Cell::text(item.category.as_str())),
            ("severity", Cell::text(item.severity.as_str()).with_status(status)),
            ("distance", Cell::text(item.distance.to_string())),
            ("reason", Cell::text(if item.removed { format!("removed by the change; {}", item.reason) } else { item.reason.clone() })),
            ("why", Cell::text(&item.why)),
        ],
    )
}

fn sorted(report: &ImpactReport) -> Vec<&Impacted> {
    let mut all: Vec<&Impacted> = report.impacted.iter().collect();
    all.sort_by(|a, b| (a.severity, a.distance, &a.id).cmp(&(b.severity, b.distance, &b.id)));
    all
}

/// The report as tables: the summary, the impact set, then (for a what-if) its deltas.
pub fn tables(report: &ImpactReport) -> Vec<Table> {
    let mut out = Vec::new();
    let mut summary = vec![
        row("seed", "ok", vec![("key", Cell::text("Change")), ("value", Cell::text(seeds_text(report)))]),
        row("summary", if report.summary.blockers > 0 { "fail" } else { "ok" }, vec![("key", Cell::text("Impact")), ("value", Cell::text(headline(report)))]),
        row("depth", "ok", vec![("key", Cell::text("Depth")), ("value", Cell::text(report.depth.to_string()))]),
        row("kinds", "ok", vec![("key", Cell::text("Links followed")), ("value", Cell::text(report.kinds.join(", ")))]),
    ];
    for (layer, n) in &report.summary.by_layer {
        summary.push(row(layer, "ok", vec![("key", Cell::text(format!("Layer {layer}"))), ("value", Cell::text(n.to_string()))]));
    }
    out.push(table("Summary", &[("key", "Item"), ("value", "Value")], summary));
    out.push(table("Impact", &IMPACT_COLUMNS, sorted(report).into_iter().map(impact_row).collect()));
    let Some(w) = &report.what_if else { return out };
    let mut gate = Vec::new();
    for b in &w.new_blockers {
        gate.push(row(b, "fail", vec![("change", Cell::text("new blocker").with_status("fail")), ("message", Cell::text(b))]));
    }
    for b in &w.removed_blockers {
        gate.push(row(b, "ok", vec![("change", Cell::text("cleared").with_status("ok")), ("message", Cell::text(b))]));
    }
    gate.push(row("gate", if w.gate_after.passed { "ok" } else { "fail" }, vec![("change", Cell::text("gate")), ("message", Cell::text(format!("{} {} → {}", w.gate_after.standard, gate_word(w.gate_before.passed), gate_word(w.gate_after.passed))))]));
    out.push(table("Gate", &[("change", "Change"), ("message", "Blocker")], gate));
    let constraints: Vec<Row> = w
        .constraints
        .iter()
        .map(|c| {
            let status = if c.after == Some(false) { "fail" } else { "ok" };
            row(&c.id, status, vec![("id", Cell::text(&c.id)), ("name", Cell::text(&c.name)), ("before", Cell::text(holds(c.before))), ("after", Cell::text(holds(c.after)).with_status(status)), ("left", Cell::text(format!("{} → {}", c.left_before, c.left_after))), ("right", Cell::text(&c.right)), ("expression", Cell::text(&c.expression))])
        })
        .collect();
    if !constraints.is_empty() {
        out.push(table("Constraints", &[("id", "Constraint"), ("name", "Name"), ("before", "Before"), ("after", "After"), ("left", "Value"), ("right", "Bound"), ("expression", "Expression")], constraints));
    }
    let budgets: Vec<Row> = w
        .budgets
        .iter()
        .map(|b| {
            let status = if b.ok_after { "ok" } else { "fail" };
            row(&b.chain, status, vec![("chain", Cell::text(&b.chain)), ("name", Cell::text(&b.name)), ("budget", Cell::text(ms(b.budget_ms))), ("before", Cell::text(ms(b.before_ms))), ("after", Cell::text(ms(b.after_ms)).with_status(status)), ("missing", Cell::text(b.missing.join(", ")))])
        })
        .collect();
    if !budgets.is_empty() {
        out.push(table("Budgets", &[("chain", "Chain"), ("name", "Name"), ("budget", "Budget"), ("before", "Before"), ("after", "After"), ("missing", "Without latency")], budgets));
    }
    let coverage: Vec<Row> = w
        .coverage
        .iter()
        .map(|c| {
            let status = if c.after.status == "covered" { "ok" } else if c.after.status == "unverified" { "warn" } else { "fail" };
            row(&c.requirement, status, vec![("id", Cell::text(&c.requirement)), ("name", Cell::text(&c.name)), ("before", Cell::text(&c.before.status)), ("after", Cell::text(&c.after.status).with_status(status)), ("satisfied", Cell::text(format!("{} → {}", c.before.satisfied_by.join(", "), c.after.satisfied_by.join(", ")))), ("verified", Cell::text(c.after.verified_by.join(", ")))])
        })
        .collect();
    out.push(table("Coverage", &[("id", "Requirement"), ("name", "Name"), ("before", "Before"), ("after", "After"), ("satisfied", "Satisfied by"), ("verified", "Verified by")], coverage));
    let verdicts: Vec<Row> = w
        .verdicts
        .iter()
        .map(|v| {
            let status = if v.stale || v.after == "fail" { "warn" } else { "ok" };
            row(&v.test, status, vec![("id", Cell::text(&v.test)), ("name", Cell::text(&v.name)), ("before", Cell::text(&v.before)), ("after", Cell::text(&v.after).with_status(status)), ("stale", Cell::text(if v.stale { "to run again" } else { "" })), ("why", Cell::text(&v.why))])
        })
        .collect();
    if !verdicts.is_empty() {
        out.push(table("Verdicts", &[("id", "Test"), ("name", "Name"), ("before", "Before"), ("after", "After"), ("stale", "Stale"), ("why", "Why")], verdicts));
    }
    let mut changes = Vec::new();
    for e in &w.diff.added {
        changes.push(row(&e.id, "ok", vec![("change", Cell::text("added")), ("id", Cell::text(&e.id)), ("kind", Cell::text(&e.element_type)), ("detail", Cell::text(&e.name))]));
    }
    for e in &w.diff.removed {
        changes.push(row(&e.id, "fail", vec![("change", Cell::text("removed").with_status("fail")), ("id", Cell::text(&e.id)), ("kind", Cell::text(&e.element_type)), ("detail", Cell::text(&e.name))]));
    }
    for e in &w.diff.modified {
        let detail = e.changes.iter().map(|c| format!("{}: {} → {}", c.field, c.old, c.new)).collect::<Vec<_>>().join("; ");
        changes.push(row(&e.element.id, "warn", vec![("change", Cell::text("changed").with_status("warn")), ("id", Cell::text(&e.element.id)), ("kind", Cell::text(&e.element.element_type)), ("detail", Cell::text(detail))]));
    }
    for t in &w.diff.traces_removed {
        changes.push(row(&t.from, "fail", vec![("change", Cell::text("trace removed").with_status("fail")), ("id", Cell::text(&t.from)), ("kind", Cell::text("Trace")), ("detail", Cell::text(format!("{} {} {}", t.from, t.trace_type, t.to)))]));
    }
    for t in &w.diff.traces_added {
        changes.push(row(&t.from, "ok", vec![("change", Cell::text("trace added")), ("id", Cell::text(&t.from)), ("kind", Cell::text("Trace")), ("detail", Cell::text(format!("{} {} {}", t.from, t.trace_type, t.to)))]));
    }
    out.push(table("Changes", &[("change", "Change"), ("id", "Element"), ("kind", "Kind"), ("detail", "Detail")], changes));
    out
}

pub fn markdown(report: &ImpactReport) -> String {
    let mut out = format!("# Change impact: {}\n\n", seeds_text(report));
    out.push_str(&format!("{}  \nModel: `{}` · depth {} · links: {}\n\n", headline(report), report.model, report.depth, report.kinds.join(", ")));
    for t in tables(report).iter().skip(1) {
        out.push_str(&tables::export::markdown(t));
        out.push('\n');
    }
    out
}

/// Tab-separated rows of the impact set (what pastes as a table).
pub fn tsv(report: &ImpactReport) -> String {
    tables::export::tsv(&table("Impact", &IMPACT_COLUMNS, sorted(report).into_iter().map(impact_row).collect()))
}

pub fn xlsx(report: &ImpactReport) -> Vec<u8> {
    tables::xlsx::workbook(&tables(report))
}

fn esc(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

const STYLE: &str = r#":root{--bg:#fbfaf7;--fg:#1d2026;--muted:#5d6470;--line:#d9d6ce;--card:#ffffff;--blocker:#b42318;--major:#b54708;--minor:#35618f;--ok:#2e7d4f}
@media (prefers-color-scheme: dark){:root{--bg:#16181d;--fg:#e8e6e1;--muted:#a3a8b3;--line:#2f333b;--card:#1e2128;--blocker:#f97066;--major:#fdb022;--minor:#84adff;--ok:#6ce9a6}}
*{box-sizing:border-box}body{margin:0;padding:24px 16px 48px;background:var(--bg);color:var(--fg);font:14px/1.5 system-ui,-apple-system,"Segoe UI",sans-serif}
main{max-width:1180px;margin:0 auto}h1{font-size:22px;margin:0 0 4px}h2{font-size:15px;margin:28px 0 8px;text-transform:uppercase;letter-spacing:.06em;color:var(--muted)}
.lede{color:var(--muted);margin:0 0 16px}.chips{display:flex;flex-wrap:wrap;gap:8px;margin:12px 0}
.chip{border:1px solid var(--line);border-radius:999px;padding:3px 12px;background:var(--card);font-weight:600}
.chip.blocker{color:var(--blocker);border-color:currentColor}.chip.major{color:var(--major);border-color:currentColor}.chip.minor{color:var(--minor);border-color:currentColor}
table{border-collapse:collapse;width:100%;background:var(--card);border:1px solid var(--line)}th,td{text-align:left;padding:6px 10px;border-bottom:1px solid var(--line);vertical-align:top}
th{font-size:12px;color:var(--muted);font-weight:600}td.why{font-family:ui-monospace,Menlo,monospace;font-size:12px}
td.fail{color:var(--blocker);font-weight:600}td.warn{color:var(--major);font-weight:600}td.ok{color:var(--ok)}
.scroll{overflow-x:auto}"#;

pub fn html(report: &ImpactReport) -> String {
    let s = &report.summary;
    let mut body = format!(
        "<h1>Change impact: {}</h1><p class=\"lede\">{} · depth {} · links followed: {}</p><div class=\"chips\"><span class=\"chip blocker\">{} blocker{}</span><span class=\"chip major\">{} major{}</span><span class=\"chip minor\">{} minor{}</span><span class=\"chip\">{}</span></div>",
        esc(&seeds_text(report)),
        esc(&report.model),
        report.depth,
        esc(&report.kinds.join(", ")),
        s.blockers,
        plural(s.blockers),
        s.majors,
        plural(s.majors),
        s.minors,
        plural(s.minors),
        esc(headline(report).rsplit(" · ").next().unwrap_or(""))
    );
    for t in tables(report).iter().skip(1) {
        body.push_str(&format!("<h2>{}</h2><div class=\"scroll\"><table><thead><tr>", esc(&t.name)));
        for c in &t.columns {
            body.push_str(&format!("<th>{}</th>", esc(&c.label)));
        }
        body.push_str("</tr></thead><tbody>");
        for r in &t.rows {
            body.push_str("<tr>");
            for c in &t.columns {
                let cell = r.cells.get(&c.key);
                let class = cell.and_then(|x| x.status.clone()).unwrap_or_else(|| if c.key == "why" { "why".to_string() } else { String::new() });
                body.push_str(&format!("<td class=\"{}\">{}</td>", class, esc(&cell.map(|x| x.text.clone()).unwrap_or_default())));
            }
            body.push_str("</tr>");
        }
        body.push_str("</tbody></table></div>");
    }
    format!("<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>Change impact</title><style>{STYLE}</style></head><body><main>{body}</main></body></html>\n")
}
