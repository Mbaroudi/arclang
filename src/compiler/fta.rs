//! Fault trees, from the failure modes of the FMEA to the hazards they lead to.
//!
//! ```text
//! safety_analysis {
//!     hazard "Unintended steering" { id: "HAZ-1" asil: "ASIL-D" }
//!     hazard "Loss of steering assist" { id: "HAZ-2" asil: "ASIL-B" gate: "and" }
//!     fmea "Torque sensor stuck" { id: "FM-1" element: "LC-SENS" leads_to: ["HAZ-1"] fit: 4 }
//! }
//! ```
//!
//! Each hazard is the top event of a tree; its basic events are the failure
//! modes that `leads_to` it. One of them suffices (an OR gate, the default),
//! or all of them together (`gate: "and"`, a redundancy). Failure rates are
//! in FIT (failures per 10^9 hours), the unit of ISO 26262; a tree whose
//! every event has a rate gets one, over `mission_time` (1 h by default),
//! and is held to the budget of its hazard: `budget_fit` when written, else
//! the target of its integrity level (ISO 26262 PMHF, ARP 4761 for DO-178C,
//! IEC 61508 high demand). The compiler warns when a tree exceeds it.

use crate::compiler::ast::{AttributeValue, Model};
use crate::compiler::metamodel::Metamodel;
use crate::compiler::quantity::Dimension;
use crate::compiler::semantic::SemanticModel;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Gate {
    Or,
    And,
}

/// A failure mode of the FMEA, a basic event of a tree.
#[derive(Debug, Clone, Serialize)]
pub struct BasicEvent {
    pub id: String,
    pub name: String,
    /// The element that fails, as written.
    pub element: Option<String>,
    pub fit: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Budget {
    pub fit: f64,
    /// Where it comes from: `budget_fit`, or the target of `ASIL-D`...
    pub from: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FaultTree {
    pub hazard: String,
    pub name: String,
    pub gate: Gate,
    pub mission_hours: f64,
    pub events: Vec<BasicEvent>,
    /// Probability of the top event over the mission, when every event has a rate.
    pub probability: Option<f64>,
    /// That probability as a rate, in FIT.
    pub fit: Option<f64>,
    pub budget: Option<Budget>,
    /// Its integrity level as written (`ASIL-B`, `Catastrophic`, `SIL-3`), when it has one.
    pub level: Option<String>,
    /// Failure modes that cause the hazard on their own.
    pub single_points: Vec<String>,
}

impl FaultTree {
    /// `Some(true)` when within budget, `Some(false)` when beyond, `None` when it cannot say.
    pub fn within_budget(&self) -> Option<bool> {
        Some(self.fit? <= self.budget.as_ref()?.fit)
    }
}

fn id_of(attributes: &std::collections::HashMap<String, AttributeValue>, name: &str) -> String {
    attributes.get("id").and_then(|v| v.as_string()).filter(|s| !s.is_empty()).unwrap_or(name).to_string()
}

fn text<'a>(attributes: &'a std::collections::HashMap<String, AttributeValue>, key: &str) -> Option<&'a str> {
    attributes.get(key).and_then(|v| v.as_string())
}

fn names(value: Option<&AttributeValue>) -> Vec<String> {
    match value {
        Some(AttributeValue::List(items)) => items.iter().filter_map(|v| v.as_string()).map(str::to_string).collect(),
        Some(AttributeValue::String(one)) => vec![one.clone()],
        _ => Vec::new(),
    }
}

/// The integrity level of a hazard, normalized.
fn level_of(attributes: &std::collections::HashMap<String, AttributeValue>) -> Option<String> {
    let metamodel = Metamodel::current();
    [("asil", "Asil"), ("condition", "FailureCondition"), ("dal", "Dal"), ("sil", "Sil")]
        .iter()
        .find_map(|(key, enumeration)| text(attributes, key).and_then(|raw| metamodel.normalize_enum(enumeration, raw)))
        .map(str::to_string)
}

/// The target rate of a hazard, from its integrity level.
fn budget_of(attributes: &std::collections::HashMap<String, AttributeValue>) -> Result<Option<Budget>, String> {
    if let Some(value) = attributes.get("budget_fit") {
        let fit = value.as_number().filter(|f| *f > 0.0).ok_or_else(|| format!("budget_fit must be a positive number of FIT, got {}", value.display()))?;
        return Ok(Some(Budget { fit, from: "budget_fit".to_string() }));
    }
    let metamodel = Metamodel::current();
    let level = |key: &str, enumeration: &str| text(attributes, key).and_then(|raw| metamodel.normalize_enum(enumeration, raw));
    let target = |fit: f64, from: &str| Some(Budget { fit, from: from.to_string() });
    Ok(match level("asil", "Asil") {
        Some(asil @ "ASIL-D") => target(10.0, asil),
        Some(asil @ ("ASIL-C" | "ASIL-B")) => target(100.0, asil),
        _ => match level("condition", "FailureCondition").or_else(|| level("dal", "Dal")) {
            Some(c @ ("Catastrophic" | "DAL-A")) => target(1.0, c),
            Some(c @ ("Hazardous" | "DAL-B")) => target(100.0, c),
            Some(c @ ("Major" | "DAL-C")) => target(10_000.0, c),
            Some(c @ ("Minor" | "DAL-D")) => target(1_000_000.0, c),
            _ => match level("sil", "Sil") {
                Some(s @ "SIL-4") => target(10.0, s),
                Some(s @ "SIL-3") => target(100.0, s),
                Some(s @ "SIL-2") => target(1_000.0, s),
                Some(s @ "SIL-1") => target(10_000.0, s),
                _ => None,
            },
        },
    })
}

fn mission_hours(attributes: &std::collections::HashMap<String, AttributeValue>) -> Result<f64, String> {
    let Some(value) = attributes.get("mission_time") else { return Ok(1.0) };
    match value.as_quantity() {
        Some(Ok(q)) if q.dimension() == Dimension::Time && q.canonical() > 0.0 => Ok(q.canonical() / 3600.0),
        _ => Err(format!("mission_time must be a positive duration (`10 h`), got {}", value.display())),
    }
}

/// Probability that the top event happens within `hours`, from the rates (FIT) of its events.
fn top_probability(gate: Gate, fits: &[f64], hours: f64) -> f64 {
    let failed = |fit: f64| 1.0 - (-fit * 1e-9 * hours).exp();
    match gate {
        Gate::Or => 1.0 - fits.iter().map(|f| 1.0 - failed(*f)).product::<f64>(),
        Gate::And => fits.iter().map(|f| failed(*f)).product(),
    }
}

/// The fault tree of every hazard, or what makes them wrong.
pub fn trees(ast: &Model) -> Result<Vec<FaultTree>, Vec<String>> {
    let mut errors = Vec::new();
    let hazards: Vec<_> = ast.safety_analysis.iter().flat_map(|s| &s.hazards).collect();
    let entries: Vec<_> = ast.safety_analysis.iter().flat_map(|s| &s.fmea).collect();
    let hazard_named = |reference: &str| hazards.iter().find(|h| id_of(&h.attributes, &h.name) == reference || h.name == reference).map(|h| id_of(&h.attributes, &h.name));
    let mut events_of: Vec<(String, BasicEvent)> = Vec::new();
    for entry in &entries {
        let id = id_of(&entry.attributes, &entry.name);
        let fit = match entry.attributes.get("fit") {
            None => None,
            Some(value) => match value.as_number().filter(|f| *f >= 0.0) {
                Some(fit) => Some(fit),
                None => {
                    errors.push(format!("fmea '{}': fit must be a number of failures per 10^9 hours, got {}", entry.name, value.display()));
                    None
                }
            },
        };
        let event = BasicEvent { id: id.clone(), name: entry.name.clone(), element: text(&entry.attributes, "element").map(str::to_string), fit };
        for reference in names(entry.attributes.get("leads_to")) {
            match hazard_named(&reference) {
                Some(hazard) => events_of.push((hazard, event.clone())),
                None => errors.push(format!("fmea '{}': leads_to '{reference}' names no hazard of the safety analysis", entry.name)),
            }
        }
    }
    let mut out = Vec::new();
    for hazard in &hazards {
        let id = id_of(&hazard.attributes, &hazard.name);
        let gate = match text(&hazard.attributes, "gate").map(str::to_ascii_lowercase).as_deref() {
            None | Some("or") => Gate::Or,
            Some("and") => Gate::And,
            Some(other) => {
                errors.push(format!("hazard '{}': gate '{other}' is not or, and", hazard.name));
                Gate::Or
            }
        };
        let budget = budget_of(&hazard.attributes).unwrap_or_else(|e| {
            errors.push(format!("hazard '{}': {e}", hazard.name));
            None
        });
        let hours = mission_hours(&hazard.attributes).unwrap_or_else(|e| {
            errors.push(format!("hazard '{}': {e}", hazard.name));
            1.0
        });
        let events: Vec<BasicEvent> = events_of.iter().filter(|(h, _)| h == &id).map(|(_, e)| e.clone()).collect();
        let fits: Option<Vec<f64>> = events.iter().map(|e| e.fit).collect();
        let probability = fits.filter(|f| !f.is_empty()).map(|f| top_probability(gate, &f, hours));
        let single_points = match gate {
            Gate::Or => events.iter().map(|e| e.id.clone()).collect(),
            Gate::And if events.len() == 1 => vec![events[0].id.clone()],
            Gate::And => Vec::new(),
        };
        out.push(FaultTree { hazard: id, name: hazard.name.clone(), gate, mission_hours: hours, fit: probability.map(|p| p / hours * 1e9), probability, budget, level: level_of(&hazard.attributes), events, single_points });
    }
    if errors.is_empty() { Ok(out) } else { Err(errors) }
}

/// Errors for references and values that are wrong; a warning for every
/// tree beyond its budget, and for an element that fails yet is not declared.
pub fn check(ast: &Model, semantic: &SemanticModel) -> Result<Vec<String>, String> {
    let trees = trees(ast).map_err(|errors| errors.join("\n"))?;
    let mut errors = Vec::new();
    for entry in ast.safety_analysis.iter().flat_map(|s| &s.fmea) {
        if let Some(element) = text(&entry.attributes, "element") {
            let known = semantic.all_elements.contains_key(element) || semantic.all_elements.values().any(|e| e.name == element);
            if !known {
                errors.push(format!("fmea '{}': element '{element}' names no element — declare it or fix the reference", entry.name));
            }
        }
    }
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    Ok(trees
        .iter()
        .filter(|tree| tree.within_budget() == Some(false))
        .map(|tree| {
            let budget = tree.budget.as_ref().expect("a verdict has a budget");
            format!("safety: hazard '{}': its fault tree fails at {} FIT, beyond the {} FIT of {}", tree.name, fit_text(tree.fit.unwrap_or_default()), fit_text(budget.fit), budget.from)
        })
        .collect())
}

/// A rate in FIT, with the digits that matter.
pub fn fit_text(fit: f64) -> String {
    if fit == 0.0 {
        "0".to_string()
    } else if fit >= 100.0 {
        format!("{fit:.0}")
    } else if fit >= 1.0 {
        format!("{fit:.2}").trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        format!("{fit:.3e}")
    }
}

fn verdict(tree: &FaultTree) -> String {
    match (tree.within_budget(), &tree.budget, tree.fit) {
        (Some(true), Some(b), _) => format!("within its budget ({} FIT, {})", fit_text(b.fit), b.from),
        (Some(false), Some(b), _) => format!("BEYOND its budget ({} FIT, {})", fit_text(b.fit), b.from),
        (None, Some(b), None) if !tree.events.is_empty() => format!("not computed: a failure mode has no fit (budget {} FIT, {})", fit_text(b.fit), b.from),
        (None, None, Some(_)) => match &tree.level {
            Some(level) => format!("no budget: {level} sets no rate target (write budget_fit to hold it to one)"),
            None => "no budget: no integrity level nor budget_fit".to_string(),
        },
        _ if tree.events.is_empty() => "no failure mode leads to it".to_string(),
        _ => "not computed".to_string(),
    }
}

/// The trees as text, for the terminal.
pub fn text_report(trees: &[FaultTree]) -> String {
    let mut out = String::new();
    for tree in trees {
        let rate = tree.fit.map(|f| format!(", {} FIT", fit_text(f))).unwrap_or_default();
        out.push_str(&format!("{} [{}]{rate}: {}\n", tree.name, tree.hazard, verdict(tree)));
        if tree.events.is_empty() {
            continue;
        }
        out.push_str(&format!("  {} gate{}\n", if tree.gate == Gate::Or { "OR" } else { "AND" }, if tree.mission_hours != 1.0 { format!(" over {} h", tree.mission_hours) } else { String::new() }));
        for (n, event) in tree.events.iter().enumerate() {
            let branch = if n + 1 == tree.events.len() { "└─" } else { "├─" };
            let fit = event.fit.map(|f| format!("{} FIT", fit_text(f))).unwrap_or_else(|| "no fit".to_string());
            let element = event.element.as_deref().map(|e| format!(" of {e}")).unwrap_or_default();
            let single = if tree.single_points.contains(&event.id) { "  single point of failure" } else { "" };
            out.push_str(&format!("  {branch} {} [{}]{element}: {fit}{single}\n", event.name, event.id));
        }
    }
    out
}

fn mermaid_label(text: &str) -> String {
    text.replace('"', "'").replace(['[', ']', '{', '}'], " ")
}

/// A safety report in Markdown: the hazards and their budgets, the FMEA by
/// risk, each fault tree (a Mermaid diagram) and its verdict, the single
/// points of failure, the mitigations.
pub fn markdown_report(ast: &Model, trees: &[FaultTree], standard: &str) -> String {
    let title = ast.attributes.get("name").and_then(|v| v.as_string()).unwrap_or("the model");
    let mut out = format!("# Safety report: {title}\n\nStandard: {standard}. Rates in FIT (failures per 10^9 hours). Generated by `arclang safety --report` from the model; every figure below is computed from it.\n\n");
    let beyond = trees.iter().filter(|t| t.within_budget() == Some(false)).count();
    let unknown = trees.iter().filter(|t| t.budget.is_some() && t.within_budget().is_none()).count();
    let untouched = trees.iter().filter(|t| t.events.is_empty()).count();
    out.push_str(&format!("**Summary**: {} hazards, {} within budget, {beyond} beyond, {unknown} not computed, {untouched} with no failure mode leading to it.\n\n", trees.len(), trees.iter().filter(|t| t.within_budget() == Some(true)).count()));

    out.push_str("## Hazards\n\n| Hazard | Level | Gate | Failure modes | Rate (FIT) | Budget (FIT) | Verdict |\n|---|---|---|---|---|---|---|\n");
    for tree in trees {
        let budget = tree.budget.as_ref().map(|b| format!("{} ({})", fit_text(b.fit), b.from)).unwrap_or_else(|| "—".to_string());
        let level = tree.level.clone().unwrap_or_else(|| "—".to_string());
        let rate = tree.fit.map(fit_text).unwrap_or_else(|| "—".to_string());
        let verdict = match (tree.within_budget(), &tree.budget, tree.events.is_empty()) {
            (Some(true), _, _) => "within",
            (Some(false), _, _) => "**beyond**",
            (None, _, true) => "no failure mode",
            (None, None, false) => "no rate target",
            (None, Some(_), false) => "not computed",
        };
        out.push_str(&format!("| {} `{}` | {level} | {} | {} | {rate} | {budget} | {verdict} |\n", tree.name, tree.hazard, if tree.gate == Gate::Or { "OR" } else { "AND" }, tree.events.len()));
    }

    let mut entries: Vec<_> = ast.safety_analysis.iter().flat_map(|s| &s.fmea).collect();
    if !entries.is_empty() {
        entries.sort_by(|a, b| {
            let rpn = |e: &&crate::compiler::ast::FmeaEntry| e.attributes.get("rpn").and_then(|v| v.as_number()).unwrap_or(-1.0);
            rpn(b).partial_cmp(&rpn(a)).unwrap_or(std::cmp::Ordering::Equal)
        });
        out.push_str("\n## FMEA, by risk priority\n\n| Failure mode | Element | RPN | FIT | Leads to | Effect |\n|---|---|---|---|---|---|\n");
        for entry in entries {
            let cell = |key: &str| entry.attributes.get(key).map(|v| v.as_string().map(str::to_string).unwrap_or_else(|| v.display())).unwrap_or_else(|| "—".to_string());
            out.push_str(&format!("| {} `{}` | {} | {} | {} | {} | {} |\n", entry.name, id_of(&entry.attributes, &entry.name), cell("element"), cell("rpn"), cell("fit"), names(entry.attributes.get("leads_to")).join(", "), cell("effect").replace('|', "/")));
        }
    }

    out.push_str("\n## Fault trees\n");
    for tree in trees {
        out.push_str(&format!("\n### {} `{}`\n\n{}.\n", tree.name, tree.hazard, verdict(tree)));
        if tree.events.is_empty() {
            continue;
        }
        let rate = tree.fit.map(|f| format!("<br/>{} FIT", fit_text(f))).unwrap_or_default();
        out.push_str(&format!("\n```mermaid\nflowchart TD\n  top[\"{}{rate}\"]\n  gate{{{}}}\n  top --- gate\n", mermaid_label(&tree.name), if tree.gate == Gate::Or { "OR" } else { "AND" }));
        for (n, event) in tree.events.iter().enumerate() {
            let fit = event.fit.map(|f| format!("<br/>{} FIT", fit_text(f))).unwrap_or_default();
            out.push_str(&format!("  e{n}((\"{}{fit}\"))\n  gate --- e{n}\n", mermaid_label(&event.name)));
        }
        out.push_str("```\n\n");
        // The same tree as a list, for a preview that draws no Mermaid.
        for event in &tree.events {
            let fit = event.fit.map(|f| format!("{} FIT", fit_text(f))).unwrap_or_else(|| "no fit".to_string());
            let element = event.element.as_deref().map(|e| format!(" of `{e}`")).unwrap_or_default();
            out.push_str(&format!("- {} `{}`{element}: {fit}\n", event.name, event.id));
        }
        if !tree.single_points.is_empty() {
            out.push_str(&format!("\nSingle points of failure: {}.\n", tree.single_points.iter().map(|s| format!("`{s}`")).collect::<Vec<_>>().join(", ")));
        }
    }

    let hazards: Vec<_> = ast.safety_analysis.iter().flat_map(|s| &s.hazards).collect();
    out.push_str("\n## Mitigations\n\n| Hazard | Mitigated by |\n|---|---|\n");
    for hazard in hazards {
        let by = names(hazard.attributes.get("mitigated_by"));
        out.push_str(&format!("| {} | {} |\n", hazard.name, if by.is_empty() { "**none**".to_string() } else { by.join(", ") }));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{Compiler, CompilerConfig};

    const MODEL: &str = r#"model T { name: "Steering" }
logical_architecture "L" {
    component "Torque sensor" { id: "LC-SENS" }
    component "Controller" { id: "LC-CTL" }
}
safety_analysis {
    hazard "Unintended steering" { id: "HAZ-1" asil: "ASIL-D" }
    hazard "Loss of assist" { id: "HAZ-2" asil: "ASIL-B" gate: "and" mission_time: 10 h }
    hazard "Noise" { id: "HAZ-3" }
    fmea "Sensor stuck" { id: "FM-1" element: "LC-SENS" leads_to: ["HAZ-1", "HAZ-2"] fit: 4 rpn: 90 }
    fmea "Controller reset" { id: "FM-2" element: "Controller" leads_to: ["Unintended steering", "HAZ-2"] fit: 3 rpn: 40 }
}
"#;

    fn compiled(source: &str) -> crate::CompilationResult {
        Compiler::new(CompilerConfig::default()).compile_string(source).expect("compiles")
    }

    #[test]
    fn a_tree_per_hazard_with_its_rate_budget_and_single_points() {
        let result = compiled(MODEL);
        let trees = trees(&result.ast).expect("trees");
        let one = &trees[0];
        assert_eq!((one.hazard.as_str(), one.gate, one.events.len()), ("HAZ-1", Gate::Or, 2));
        assert!((one.fit.unwrap() - 7.0).abs() < 1e-6, "{:?}", one.fit);
        assert_eq!(one.budget.as_ref().map(|b| (b.fit, b.from.as_str())), Some((10.0, "ASIL-D")));
        assert_eq!(one.within_budget(), Some(true));
        assert_eq!(one.single_points, ["FM-1", "FM-2"]);
        // Both must fail within ten hours: a product of probabilities.
        let two = &trees[1];
        assert_eq!(two.gate, Gate::And);
        assert!(two.single_points.is_empty());
        let expected = (1.0 - (-4e-9 * 10.0f64).exp()) * (1.0 - (-3e-9 * 10.0f64).exp());
        assert!((two.probability.unwrap() - expected).abs() < 1e-24, "{:?}", two.probability);
        assert!(trees[2].events.is_empty() && trees[2].budget.is_none());
        assert!(result.warnings.iter().all(|w| !w.starts_with("safety:")), "{:?}", result.warnings);
    }

    #[test]
    fn a_tree_beyond_its_budget_is_a_warning_and_wrong_references_are_errors() {
        let beyond = compiled(&MODEL.replace("fit: 3 rpn", "fit: 30 rpn"));
        assert!(beyond.warnings.iter().any(|w| w.contains("'Unintended steering'") && w.contains("34 FIT") && w.contains("10 FIT of ASIL-D")), "{:?}", beyond.warnings);
        let compile = |source: String| Compiler::new(CompilerConfig::default()).compile_string(&source).unwrap_err().to_string();
        assert!(compile(MODEL.replace("[\"HAZ-1\", \"HAZ-2\"]", "[\"HAZ-9\"]")).contains("leads_to 'HAZ-9' names no hazard"));
        assert!(compile(MODEL.replace("element: \"LC-SENS\"", "element: \"LC-NONE\"")).contains("element 'LC-NONE' names no element"));
        assert!(compile(MODEL.replace("gate: \"and\"", "gate: \"xor\"")).contains("gate 'xor' is not or, and"));
        assert!(compile(MODEL.replace("fit: 4", "fit: \"low\"")).contains("fit must be a number"));
    }

    #[test]
    fn the_reports_say_what_was_computed() {
        let result = compiled(MODEL);
        let trees = trees(&result.ast).unwrap();
        let text = text_report(&trees);
        assert!(text.contains("Unintended steering [HAZ-1], 7 FIT: within its budget (10 FIT, ASIL-D)"), "{text}");
        assert!(text.contains("├─ Sensor stuck [FM-1] of LC-SENS: 4 FIT  single point of failure"), "{text}");
        assert!(text.contains("Noise [HAZ-3]: no failure mode leads to it"), "{text}");
        let markdown = markdown_report(&result.ast, &trees, "ISO26262");
        assert!(markdown.starts_with("# Safety report: Steering"), "{markdown}");
        assert!(markdown.contains("```mermaid\nflowchart TD\n  top[\"Unintended steering<br/>7 FIT\"]"), "{markdown}");
        assert!(markdown.contains("| Sensor stuck `FM-1` | LC-SENS | 90 | 4 |"), "{markdown}");
        assert!(markdown.contains("| Noise | **none** |"), "{markdown}");
        let unrated = compiled(&MODEL.replace("asil: \"ASIL-B\" gate", "asil: \"ASIL-A\" gate"));
        let text = text_report(&super::trees(&unrated.ast).unwrap());
        assert!(text.contains("no budget: ASIL-A sets no rate target"), "{text}");
    }
}
