//! The picture of a model that a pull-request report shows twice, before and
//! after: its elements by layer, the ones the change touched marked. No
//! layout engine and no font file: boxes in rows, text the viewer draws.
//! A drawing of the architecture itself is `arclang diagram`; this one is a
//! map of what changed.

use crate::compiler::impact::graph::Node;
use crate::compiler::impact::Analysis;
use crate::compiler::semgit::rich::RichDiff;
use crate::compiler::CompilationResult;
use std::collections::HashMap;

const LAYERS: [(&str, &str); 5] = [("OA", "Operational Analysis"), ("SA", "System Analysis"), ("LA", "Logical Architecture"), ("PA", "Physical Architecture"), ("EPBS", "EPBS")];
const BOX_W: usize = 168;
const BOX_H: usize = 40;
const GAP: usize = 10;
const COLUMNS: usize = 5;
const WIDTH: usize = 24 + COLUMNS * (BOX_W + GAP);
/// Boxes drawn at most; the changed ones first.
const MAX_BOXES: usize = 90;

/// Which picture: the model before the change, or after.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Before,
    After,
}

/// Ports, requirements and the like are not boxes of an architecture map.
fn is_drawn(kind: &str) -> bool {
    !(kind.ends_with("Port") || matches!(kind, "Requirement" | "Mission" | "Constraint" | "Capability" | "Scenario" | "TestCase"))
}

fn esc(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn clip(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_string()
    } else {
        format!("{}...", text.chars().take(max - 3).collect::<String>())
    }
}

/// The status of each element the diff names, by id.
fn statuses(diff: &RichDiff, side: Side) -> HashMap<String, &'static str> {
    let mut map = HashMap::new();
    for e in diff.layers.iter().flat_map(|l| &l.kinds).flat_map(|k| &k.elements) {
        let status = match (e.status, side) {
            ("added", Side::After) => "added",
            ("removed", Side::Before) => "removed",
            ("modified", _) => "modified",
            _ => continue,
        };
        map.insert(e.id.clone(), status);
    }
    map
}

fn colours(status: &str) -> (&'static str, &'static str, &'static str) {
    match status {
        "added" => ("#e6f4ea", "#1a7f37", ""),
        "removed" => ("#fdecea", "#c62828", "5 3"),
        "modified" => ("#fff4e0", "#b26a00", ""),
        _ => ("#f6f8fa", "#8c959f", ""),
    }
}

fn glyph(status: &str) -> &'static str {
    match status {
        "added" => "+",
        "removed" => "-",
        "modified" => "~",
        _ => "",
    }
}

/// An SVG map of the model. `diff` marks what the change touched.
pub fn render(result: &CompilationResult, diff: &RichDiff, side: Side, caption: &str, standard: &str) -> String {
    let analysis = Analysis::new(result, standard);
    let marks = statuses(diff, side);
    let nodes: Vec<&Node> = analysis.graph.nodes.iter().filter(|n| n.element.is_some() && LAYERS.iter().any(|(code, _)| *code == n.layer) && is_drawn(&n.kind)).collect();
    // The changed first, so that a cap never hides them.
    let mut chosen: Vec<&Node> = nodes.iter().filter(|n| marks.contains_key(&n.id)).copied().collect();
    let room = MAX_BOXES.saturating_sub(chosen.len());
    chosen.extend(nodes.iter().filter(|n| !marks.contains_key(&n.id)).take(room).copied());
    let hidden = nodes.len() - chosen.len();

    let mut body = String::new();
    let mut y = 64;
    for (code, label) in LAYERS {
        let mut in_layer: Vec<&Node> = chosen.iter().filter(|n| n.layer == code).copied().collect();
        if in_layer.is_empty() {
            continue;
        }
        in_layer.sort_by(|a, b| (!marks.contains_key(&a.id), &a.kind, &a.id).cmp(&(!marks.contains_key(&b.id), &b.kind, &b.id)));
        body.push_str(&format!("<text x=\"12\" y=\"{}\" class=\"layer\">{} ({})</text>\n", y + 4, esc(label), code));
        y += 14;
        for (i, node) in in_layer.iter().enumerate() {
            let status = marks.get(&node.id).copied().unwrap_or("");
            let (fill, stroke, dash) = colours(status);
            let x = 12 + (i % COLUMNS) * (BOX_W + GAP);
            let top = y + (i / COLUMNS) * (BOX_H + GAP);
            let dash_attr = if dash.is_empty() { String::new() } else { format!(" stroke-dasharray=\"{dash}\"") };
            let strike = if status == "removed" { " text-decoration=\"line-through\"" } else { "" };
            body.push_str(&format!(
                "<g><title>{} {} [{}]{}</title><rect x=\"{x}\" y=\"{top}\" width=\"{BOX_W}\" height=\"{BOX_H}\" rx=\"5\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"{}\"{dash_attr}/>\
                 <text x=\"{}\" y=\"{}\" class=\"name\"{strike}>{}{}</text><text x=\"{}\" y=\"{}\" class=\"kind\">{}</text></g>\n",
                esc(&node.kind),
                esc(&node.name),
                esc(&node.id),
                if status.is_empty() { String::new() } else { format!(" ({status})") },
                if status.is_empty() { 1 } else { 2 },
                x + 8,
                top + 17,
                if status.is_empty() { String::new() } else { format!("{} ", glyph(status)) },
                esc(&clip(&node.name, 22)),
                x + 8,
                top + 32,
                esc(&clip(&node.kind, 26)),
            ));
        }
        y += in_layer.len().div_ceil(COLUMNS) * (BOX_H + GAP) + 18;
    }
    if chosen.is_empty() {
        body.push_str("<text x=\"12\" y=\"84\" class=\"kind\">This version of the model has no element in the Arcadia layers.</text>\n");
        y += 40;
    }
    if hidden > 0 {
        body.push_str(&format!("<text x=\"12\" y=\"{y}\" class=\"kind\">... and {hidden} more element(s), unchanged, not drawn.</text>\n"));
        y += 18;
    }
    let legend = [("added", "added"), ("modified", "modified"), ("removed", "removed"), ("", "unchanged")];
    let mut legend_svg = String::new();
    for (i, (status, label)) in legend.iter().enumerate() {
        let (fill, stroke, dash) = colours(status);
        let x = 12 + i * 120;
        let dash_attr = if dash.is_empty() { String::new() } else { format!(" stroke-dasharray=\"{dash}\"") };
        legend_svg.push_str(&format!("<rect x=\"{x}\" y=\"{}\" width=\"14\" height=\"14\" rx=\"3\" fill=\"{fill}\" stroke=\"{stroke}\"{dash_attr}/><text x=\"{}\" y=\"{}\" class=\"kind\">{label}</text>", y + 4, x + 20, y + 15));
    }
    let height = y + 30;
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{WIDTH}\" height=\"{height}\" viewBox=\"0 0 {WIDTH} {height}\" role=\"img\" aria-label=\"{}\">\n\
         <style>text{{font-family:system-ui,-apple-system,Segoe UI,sans-serif}}.title{{font-size:14px;font-weight:600;fill:#1f2328}}.layer{{font-size:12px;font-weight:600;fill:#57606a}}.name{{font-size:12px;fill:#1f2328}}.kind{{font-size:10px;fill:#57606a}}</style>\n\
         <rect width=\"100%\" height=\"100%\" fill=\"#ffffff\"/>\n<text x=\"12\" y=\"28\" class=\"title\">{}</text>\n<text x=\"12\" y=\"46\" class=\"kind\">Elements by Arcadia layer; what the change touched is marked.</text>\n{body}{legend_svg}\n</svg>\n",
        esc(caption),
        esc(caption),
    )
}
