//! The rich diff as text, Markdown, HTML or JSON.

use super::rich::{ElementChange, RelationChange, RichDiff};
use crate::compiler::impact::whatif::CoverageDelta;
use crate::compiler::semantic_diff::FieldChange;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Text,
    Json,
    Md,
    Html,
}

pub fn render(diff: &RichDiff, format: Format) -> String {
    match format {
        Format::Text => text(diff),
        Format::Json => format!("{}\n", serde_json::to_string_pretty(diff).unwrap_or_default()),
        Format::Md => markdown(diff),
        Format::Html => html(diff),
    }
}

fn quote(text: &str) -> String {
    format!("\"{text}\"")
}

fn field_line(c: &FieldChange) -> String {
    match (c.old.is_empty(), c.new.is_empty()) {
        (true, _) => format!("{} = {}", c.field, quote(&c.new)),
        (_, true) => format!("{} removed (was {})", c.field, quote(&c.old)),
        _ => format!("{}: {} -> {}", c.field, quote(&c.old), quote(&c.new)),
    }
}

fn element_headline(e: &ElementChange) -> String {
    let mark = match e.status {
        "added" => "+ added   ",
        "removed" => "- removed ",
        _ if e.renamed.is_some() && e.changes.is_empty() && e.moved.is_none() => "~ renamed ",
        _ if e.moved.is_some() && e.changes.is_empty() && e.renamed.is_none() => "~ moved   ",
        _ => "~ modified",
    };
    format!("{mark} {} '{}' [{}]", e.kind, e.name, e.id)
}

fn element_details(e: &ElementChange) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(r) = &e.renamed {
        lines.push(format!("renamed: {} -> {}", quote(&r.from), quote(&r.to)));
    }
    if let Some(m) = &e.moved {
        lines.push(format!("moved: {} -> {}", if m.from.is_empty() { "(model)" } else { &m.from }, if m.to.is_empty() { "(model)" } else { &m.to }));
    }
    lines.extend(e.changes.iter().map(field_line));
    lines
}

fn relation_line(r: &RelationChange) -> String {
    let name = if r.name.is_empty() { String::new() } else { format!(" '{}'", r.name) };
    let ends = format!("{} -> {}", r.source, r.target);
    match r.status {
        "retargeted" => {
            let was = r.was.as_ref().map(|w| format!(" (was {} -> {})", w.from, w.to)).unwrap_or_default();
            format!("~ retargeted {} {}{name}: {ends}{was}", r.category, r.kind)
        }
        "added" => format!("+ {} {}{name}: {ends}", r.category, r.kind),
        "removed" => format!("- {} {}{name}: {ends}", r.category, r.kind),
        _ => format!("~ {} {}{name}: {ends}", r.category, r.kind),
    }
}

fn coverage_line(d: &CoverageDelta) -> String {
    format!("{} '{}': {} -> {}", d.requirement, d.name, d.before.status, d.after.status)
}

fn gate_line(d: &RichDiff) -> String {
    let c = &d.consequences;
    let verdict = |g: &crate::compiler::impact::GateState| if g.passed { "PASS" } else { "FAIL" };
    format!("{} ({}) -> {} ({} blocker(s) -> {})", verdict(&c.gate_before), c.gate_before.standard, verdict(&c.gate_after), c.gate_before.blockers.len(), c.gate_after.blockers.len())
}

pub fn headline(d: &RichDiff) -> String {
    let s = &d.summary;
    format!(
        "{} added, {} removed, {} modified ({} renamed, {} moved); {} relationship(s) added, {} removed, {} changed",
        s.elements_added, s.elements_removed, s.elements_modified, s.renamed, s.moved, s.relationships_added, s.relationships_removed, s.relationships_changed
    )
}

pub fn text(d: &RichDiff) -> String {
    let mut out = format!("Semantic diff (by stable identity): {} -> {}\n  Model: {}\n", d.from, d.to, d.model);
    if !d.model_changed {
        out.push_str("  No semantic changes.\n");
    }
    for layer in &d.layers {
        let title = if layer.label == layer.layer { layer.label.clone() } else { format!("{} ({})", layer.label, layer.layer) };
        out.push_str(&format!("\n{title}: +{} -{} ~{}\n", layer.counts.added, layer.counts.removed, layer.counts.modified));
        for group in &layer.kinds {
            for e in &group.elements {
                out.push_str(&format!("  {}\n", element_headline(e)));
                for line in element_details(e) {
                    out.push_str(&format!("      {line}\n"));
                }
            }
        }
    }
    if !d.relationships.is_empty() {
        out.push_str("\nRelationships:\n");
        for r in &d.relationships {
            out.push_str(&format!("  {}\n", relation_line(r)));
            for c in &r.changes {
                out.push_str(&format!("      {}\n", field_line(c)));
            }
        }
    }
    if d.model_changed {
        let c = &d.consequences;
        out.push_str("\nConsequences:\n");
        out.push_str(&format!("  Production gate: {}\n", gate_line(d)));
        for b in &c.new_blockers {
            out.push_str(&format!("    + blocker: {b}\n"));
        }
        for b in &c.removed_blockers {
            out.push_str(&format!("    - blocker resolved: {b}\n"));
        }
        for x in &c.newly_unsatisfied {
            out.push_str(&format!("  Newly unsatisfied: {}\n", coverage_line(x)));
        }
        for x in &c.newly_unverified {
            out.push_str(&format!("  Newly unverified: {}\n", coverage_line(x)));
        }
        out.push_str(&format!(
            "  Coverage: {}/{} satisfied, {}/{} verified (was {}/{}, {}/{})\n",
            c.coverage_after.satisfied, c.coverage_after.requirements, c.coverage_after.verified, c.coverage_after.requirements,
            c.coverage_before.satisfied, c.coverage_before.requirements, c.coverage_before.verified, c.coverage_before.requirements
        ));
        for k in &c.constraints {
            out.push_str(&format!("  Constraint {} flips: {} -> {} ({})\n", k.id, truth(k.before), truth(k.after), k.expression));
        }
        for b in &c.budgets {
            out.push_str(&format!("  Budget {}: {} ms -> {} ms (limit {} ms){}\n", b.name, ms(b.before_ms), ms(b.after_ms), ms(b.budget_ms), if b.ok_before && !b.ok_after { ", EXCEEDED" } else { "" }));
        }
        for v in &c.verdicts {
            out.push_str(&format!("  Verification {}: {} -> {}{}\n", v.test, v.before, v.after, if v.stale { " (stale: to redo)" } else { "" }));
        }
        out.push_str(&format!("  Impact: {} element(s) reached, {} blocker(s)\n", c.impacted, c.impacted_blockers));
    }
    if !d.layout.is_empty() {
        out.push_str(&format!("\nLayout{} ({} entr{}, not a model change):\n", if d.layout_only { " only" } else { "" }, d.layout.len(), if d.layout.len() == 1 { "y" } else { "ies" }));
        for l in d.layout.iter().take(30) {
            out.push_str(&format!("  {} {} {}{}\n", match l.status { "added" => "+", "removed" => "-", _ => "~" }, l.what, l.element, if l.view.is_empty() { String::new() } else { format!(" in {}", l.view) }));
        }
        if d.layout.len() > 30 {
            out.push_str(&format!("  ... and {} more\n", d.layout.len() - 30));
        }
    }
    out.push_str(&format!("\n  Total: {}\n", headline(d)));
    out
}

fn truth(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "holds",
        Some(false) => "violated",
        None => "absent",
    }
}

fn ms(value: Option<f64>) -> String {
    value.map(|v| format!("{v:.0}")).unwrap_or_else(|| "-".to_string())
}

pub fn markdown(d: &RichDiff) -> String {
    let mut out = format!("# Semantic diff: `{}` -> `{}`\n\nModel: `{}`\n\n**{}**\n", d.from, d.to, d.model, headline(d));
    if !d.model_changed {
        out.push_str("\nNo semantic changes.\n");
    }
    for layer in &d.layers {
        out.push_str(&format!("\n## {} ({}): +{} -{} ~{}\n", layer.label, layer.layer, layer.counts.added, layer.counts.removed, layer.counts.modified));
        for group in &layer.kinds {
            out.push_str(&format!("\n### {}\n\n", group.kind));
            for e in &group.elements {
                out.push_str(&format!("- **{}** `{}` {}\n", e.status, e.id, e.name));
                for line in element_details(e) {
                    out.push_str(&format!("  - {line}\n"));
                }
            }
        }
    }
    if !d.relationships.is_empty() {
        out.push_str("\n## Relationships\n\n");
        for r in &d.relationships {
            out.push_str(&format!("- {}\n", relation_line(r)));
            for c in &r.changes {
                out.push_str(&format!("  - {}\n", field_line(c)));
            }
        }
    }
    if d.model_changed {
        let c = &d.consequences;
        out.push_str(&format!("\n## Consequences\n\n- Production gate: **{}**\n", gate_line(d)));
        for b in &c.new_blockers {
            out.push_str(&format!("  - new blocker: {b}\n"));
        }
        for b in &c.removed_blockers {
            out.push_str(&format!("  - blocker resolved: {b}\n"));
        }
        for x in &c.newly_unsatisfied {
            out.push_str(&format!("- Newly unsatisfied: {}\n", coverage_line(x)));
        }
        for x in &c.newly_unverified {
            out.push_str(&format!("- Newly unverified: {}\n", coverage_line(x)));
        }
        out.push_str(&format!(
            "- Coverage: {}/{} satisfied, {}/{} verified (was {}/{}, {}/{})\n",
            c.coverage_after.satisfied, c.coverage_after.requirements, c.coverage_after.verified, c.coverage_after.requirements,
            c.coverage_before.satisfied, c.coverage_before.requirements, c.coverage_before.verified, c.coverage_before.requirements
        ));
        for k in &c.constraints {
            out.push_str(&format!("- Constraint `{}` flips: {} -> {}\n", k.id, truth(k.before), truth(k.after)));
        }
        for b in &c.budgets {
            out.push_str(&format!("- Budget `{}`: {} ms -> {} ms (limit {} ms)\n", b.name, ms(b.before_ms), ms(b.after_ms), ms(b.budget_ms)));
        }
        out.push_str(&format!("- Impact: {} element(s) reached, {} blocker(s)\n", c.impacted, c.impacted_blockers));
    }
    if !d.layout.is_empty() {
        out.push_str(&format!("\n## Layout{} (not a model change)\n\n{} entr{} changed.\n", if d.layout_only { " only" } else { "" }, d.layout.len(), if d.layout.len() == 1 { "y" } else { "ies" }));
    }
    out
}

fn esc(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

pub fn html(d: &RichDiff) -> String {
    let mut body = format!("<h1>Semantic diff <code>{}</code> &rarr; <code>{}</code></h1>\n<p class=\"meta\">Model <code>{}</code></p>\n<p class=\"sum\">{}</p>\n", esc(&d.from), esc(&d.to), esc(&d.model), esc(&headline(d)));
    if !d.model_changed {
        body.push_str("<p>No semantic changes.</p>\n");
    }
    for layer in &d.layers {
        body.push_str(&format!("<h2>{} <small>{}: +{} -{} ~{}</small></h2>\n<ul>\n", esc(&layer.label), esc(&layer.layer), layer.counts.added, layer.counts.removed, layer.counts.modified));
        for group in &layer.kinds {
            for e in &group.elements {
                body.push_str(&format!("<li class=\"{}\"><b>{}</b> {} <code>{}</code> {}", e.status, e.status, esc(&e.kind), esc(&e.id), esc(&e.name)));
                let details = element_details(e);
                if !details.is_empty() {
                    body.push_str("<ul>");
                    for line in details {
                        body.push_str(&format!("<li>{}</li>", esc(&line)));
                    }
                    body.push_str("</ul>");
                }
                body.push_str("</li>\n");
            }
        }
        body.push_str("</ul>\n");
    }
    if !d.relationships.is_empty() {
        body.push_str("<h2>Relationships</h2>\n<ul>\n");
        for r in &d.relationships {
            body.push_str(&format!("<li>{}</li>\n", esc(&relation_line(r))));
        }
        body.push_str("</ul>\n");
    }
    if d.model_changed {
        let c = &d.consequences;
        body.push_str(&format!("<h2>Consequences</h2>\n<ul>\n<li>Production gate: <b>{}</b></li>\n", esc(&gate_line(d))));
        for b in &c.new_blockers {
            body.push_str(&format!("<li class=\"removed\">new blocker: {}</li>\n", esc(b)));
        }
        for b in &c.removed_blockers {
            body.push_str(&format!("<li class=\"added\">blocker resolved: {}</li>\n", esc(b)));
        }
        for x in c.newly_unsatisfied.iter().chain(&c.newly_unverified) {
            body.push_str(&format!("<li class=\"removed\">{}</li>\n", esc(&coverage_line(x))));
        }
        body.push_str(&format!("<li>Impact: {} element(s) reached, {} blocker(s)</li>\n</ul>\n", c.impacted, c.impacted_blockers));
    }
    if !d.layout.is_empty() {
        body.push_str(&format!("<h2>Layout{}</h2>\n<p>{} entries changed (not a model change).</p>\n", if d.layout_only { " only" } else { "" }, d.layout.len()));
    }
    format!(
        "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><title>Semantic diff</title>\n<style>body{{font:14px/1.5 system-ui,sans-serif;max-width:60rem;margin:2rem auto;padding:0 1rem}}code{{background:#eee;padding:0 .25em;border-radius:3px}}small{{color:#666;font-weight:400}}li.added>b{{color:#1a7f37}}li.removed>b{{color:#c62828}}li.modified>b{{color:#b26a00}}li.added{{color:#1a7f37}}li.removed{{color:#c62828}}</style></head>\n<body>\n{body}</body></html>\n"
    )
}
