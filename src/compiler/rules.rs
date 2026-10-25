//! The catalogue of what the compiler checks, as a validation tool lists
//! its rules: an identifier, a severity, a title, what the rule means and
//! how to answer it.
//!
//! The checks themselves live where they are computed (the parser, the
//! metamodel check, the traceability validation, the methodology lints,
//! the production gate) and print the lines they always printed. This
//! module classifies those lines: each becomes a [`Finding`] that names its
//! rule, the element concerned when the line names one, and a hint at the
//! edit that answers it. A line the catalogue does not know is still a
//! finding, under a fallback rule, so nothing is lost on the way.

use super::CompilationResult;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Stops the production gate.
    Error,
    /// Worth fixing before a release.
    Warning,
    /// Advisory: the Arcadia method expects something else.
    Info,
}

#[derive(Debug, Clone, Serialize)]
pub struct Rule {
    pub id: &'static str,
    pub severity: Severity,
    pub title: &'static str,
    pub doc: &'static str,
    /// How to answer the rule, in prose.
    pub fix: &'static str,
}

/// The edit that answers a finding, when one can be named.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FixHint {
    /// Declare a trace from (or to) the element.
    Trace { element: String },
    /// Write or correct one attribute of the element.
    Set { element: String, key: String },
    /// Give the element an explicit, unique id.
    Identify { element: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub rule: &'static str,
    pub severity: Severity,
    pub message: String,
    /// The element the finding is about, by id or name as the line wrote it.
    pub element: Option<String>,
    pub fix: Option<FixHint>,
}

macro_rules! rule {
    ($id:expr, $severity:expr, $title:expr, $doc:expr, $fix:expr) => {
        Rule { id: $id, severity: $severity, title: $title, doc: $doc, fix: $fix }
    };
}

const RULES: &[Rule] = &[
    rule!("syntax.unmodeled", Severity::Warning, "Block not represented in the model",
        "A block the parser accepts but the compiled model does not hold (a `dataflow`, for instance): its contents are ignored.",
        "Model the content with constructs the metamodel holds, or remove the block."),
    rule!("syntax.attribute-not-stored", Severity::Warning, "Attribute not stored",
        "An attribute written on a construct that does not keep it: it is read and dropped.",
        "Move the information to an attribute the kind declares (see `arclang metamodel`), or remove it."),
    rule!("metamodel.type", Severity::Error, "Typed attribute violated",
        "A declared attribute does not fit its type: a quantity of the wrong dimension, an unknown enumeration value, a list where one value is expected. The production gate blocks on it.",
        "Write a value of the declared type; `arclang metamodel` lists the type and the accepted values."),
    rule!("metamodel.unit", Severity::Warning, "Quantity without a unit",
        "A bare number where the metamodel expects a quantity: the unit is assumed, the model should state it.",
        "Write the unit with the number: `latency: 25 ms`."),
    rule!("metamodel.reference", Severity::Warning, "Reference to an element of another kind",
        "A reference attribute names an element whose kind is not the one expected there.",
        "Point the attribute at an element of the expected kind."),
    rule!("identity.duplicate", Severity::Error, "Two elements share one identity",
        "Two declarations resolve to the same id: traces and exchanges could not tell them apart.",
        "Give one of them an explicit, unique `id`."),
    rule!("spatial.length", Severity::Error, "Link shorter than the ground it covers",
        "A physical link's `length` is shorter than its `route`, or than the distance between its located ends: the medium cannot be that short.",
        "Correct the `length`, the `route` or the `location` of an end."),
    rule!("spatial.max-length", Severity::Error, "Link beyond its maximum length",
        "A physical link is longer than its `max_length` (a cable or bus limit): by its length, its route or the distance between its ends.",
        "Shorten the route, move an end, or choose a medium that reaches that far."),
    rule!("spatial.latency", Severity::Error, "Latency faster than light",
        "A physical link's `latency` is shorter than the time light takes to cover its length: no medium carries a signal that fast.",
        "Correct the `latency` or the length of the link."),
    rule!("constraint.violated", Severity::Warning, "Constraint violated",
        "A declared `constraint` compares as false on the model's values. The production gate blocks on it.",
        "Change the values the constraint compares, or the constraint, so that the model holds what it claims."),
    rule!("trace.requirement", Severity::Warning, "Requirement without a trace",
        "No element satisfies, refines or verifies the requirement, in either direction: it is not covered.",
        "Declare `trace \"<element>\" satisfies \"<requirement>\"`, or a test case that `verifies:` it."),
    rule!("trace.component", Severity::Warning, "Component without a trace",
        "No trace names the component: nothing says what it realizes or satisfies.",
        "Declare what the component satisfies or realizes with a `trace`."),
    rule!("method.layer-order", Severity::Info, "Layer without the one it refines",
        "Arcadia builds each level on the previous one: a physical architecture realizes a logical one, which allocates system functions, which answer operational needs.",
        "Add the missing level, or keep the model as a single-level study if that is what it is."),
    rule!("method.component-functions", Severity::Info, "Logical component without a function",
        "Arcadia expects a logical component to realize behaviour: at least one function is allocated to it.",
        "Declare the functions the component performs inside it."),
    rule!("method.exchange-transport", Severity::Info, "Exchange between nodes without a physical link",
        "A behavioural exchange between components deployed on different nodes must be carried by a physical link or exchange.",
        "Declare the physical link (or physical exchange) between the two nodes."),
    rule!("method.other", Severity::Info, "Methodology advisory",
        "An Arcadia advisory the catalogue does not classify further.", "Read the advisory."),
    rule!("compile.other", Severity::Warning, "Compiler warning",
        "A warning the catalogue does not classify further.", "Read the warning."),
];

/// Every rule, in catalogue order. Gate checks are rules of their own,
/// named after the check (`requirements.satisfaction`, `timing.budget`...).
pub fn catalogue() -> &'static [Rule] {
    RULES
}

pub fn rule(id: &str) -> Option<&'static Rule> {
    RULES.iter().find(|rule| rule.id == id)
}

/// The text between two markers, trimmed; the whole rest when `close` is absent.
fn between<'a>(text: &'a str, open: &str, close: &str) -> Option<&'a str> {
    let start = text.find(open)? + open.len();
    let rest = &text[start..];
    let end = rest.find(close).unwrap_or(rest.len());
    Some(rest[..end].trim())
}

fn finding(rule_id: &'static str, message: &str, element: Option<&str>, fix: Option<FixHint>) -> Finding {
    let severity = rule(rule_id).map(|rule| rule.severity).unwrap_or(Severity::Warning);
    Finding { rule: rule_id, severity, message: message.to_string(), element: element.map(str::to_string), fix }
}

/// Classify one compiler warning (parser, metamodel check, constraints, identities).
pub fn classify_warning(message: &str) -> Finding {
    if let Some(rest) = message.strip_prefix("metamodel: ") {
        // `metamodel: Kind 'element'[.attribute]: text`
        let element = between(rest, "'", "'");
        let after_element = element.and_then(|e| rest.find(&format!("'{e}'")).map(|at| &rest[at + e.len() + 2..]));
        let attribute = after_element.and_then(|tail| tail.strip_prefix('.')).and_then(|tail| tail.split(':').next()).map(str::trim);
        let text = after_element.and_then(|tail| tail.find(": ").map(|at| &tail[at + 2..])).unwrap_or(rest);
        let rule_id = if text.contains("faster than light") {
            "spatial.latency"
        } else if text.contains("is shorter than its route") || text.contains("is shorter than the distance between its ends") {
            "spatial.length"
        } else if text.contains("is beyond its max_length") {
            "spatial.max-length"
        } else if text.contains("no unit") || text.contains("unit is assumed") || text.contains("without a unit") {
            "metamodel.unit"
        } else if text.contains("expected a") && text.contains(" is a ") && !text.contains("(e.g.") {
            // `'X' is a Kind, expected a Kind` names an element of the wrong
            // kind; the same words with an example are a quantity of the
            // wrong dimension.
            "metamodel.reference"
        } else {
            "metamodel.type"
        };
        let fix = match (element, attribute) {
            (Some(element), Some(key)) => Some(FixHint::Set { element: element.to_string(), key: key.to_string() }),
            _ => None,
        };
        return finding(rule_id, message, element, fix);
    }
    if message.starts_with("constraint: ") {
        let element = between(message, "constraint '", "'");
        return finding("constraint.violated", message, element, None);
    }
    if message.starts_with("duplicate element id ") {
        let element = between(message, "duplicate element id '", "'");
        return finding("identity.duplicate", message, element, element.map(|e| FixHint::Identify { element: e.to_string() }));
    }
    if message.contains("is not yet represented in the compiled model") {
        return finding("syntax.unmodeled", message, None, None);
    }
    if message.contains("is not yet stored in the model") {
        return finding("syntax.attribute-not-stored", message, None, None);
    }
    finding("compile.other", message, None, None)
}

/// Classify one line of `validate_traceability`.
pub fn classify_traceability(message: &str) -> Finding {
    let trace = |element: &str| Some(FixHint::Trace { element: element.to_string() });
    if let Some(element) = message.strip_prefix("Requirement ").and_then(|rest| rest.strip_suffix(" has no trace")) {
        return finding("trace.requirement", message, Some(element), trace(element));
    }
    if let Some(element) = message.strip_prefix("Component ").and_then(|rest| rest.strip_suffix(" has no trace")) {
        return finding("trace.component", message, Some(element), trace(element));
    }
    finding("compile.other", message, None, None)
}

/// Classify one Arcadia methodology advisory.
pub fn classify_lint(message: &str) -> Finding {
    if message.contains(" without a ") && message.contains("architecture") || message.contains("without an operational analysis") {
        return finding("method.layer-order", message, None, None);
    }
    if message.starts_with("logical component '") && message.contains("performs no functions") {
        let element = between(message, "logical component '", "'");
        return finding("method.component-functions", message, element, None);
    }
    if message.starts_with("component exchange '") && message.contains("crosses nodes") {
        let element = between(message, "component exchange '", "'");
        return finding("method.exchange-transport", message, element, None);
    }
    finding("method.other", message, None, None)
}

/// Every finding of a compiled model: compiler warnings, traceability,
/// methodology. The production gate reports its own findings, each named
/// after its check.
pub fn findings(result: &CompilationResult) -> Vec<Finding> {
    let mut all: Vec<Finding> = result.warnings.iter().map(|line| classify_warning(line)).collect();
    all.extend(result.semantic_model.validate_traceability().iter().map(|line| classify_traceability(line)));
    all.extend(super::semantic::arcadia_methodology_lints(&result.ast).iter().map(|line| classify_lint(line)));
    all
}

/// The catalogue as Markdown, for the documentation and `arclang rules`.
pub fn to_markdown() -> String {
    let mut out = String::from("# ArcLang validation rules\n\nWhat the compiler checks, as `arclang check --lint --json` reports it: each finding names its rule.\n\n| Rule | Severity | Title | Meaning | How to answer |\n|---|---|---|---|---|\n");
    for rule in RULES {
        let severity = match rule.severity {
            Severity::Error => "error (gate blocker)",
            Severity::Warning => "warning",
            Severity::Info => "info",
        };
        out.push_str(&format!("| `{}` | {} | {} | {} | {} |\n", rule.id, severity, rule.title, rule.doc, rule.fix));
    }
    out.push_str("\nThe production gate (`arclang gate`) reports its own findings, each named after its check: `requirements.presence`, `requirements.satisfaction`, `requirements.verification`, `verification.dangling`, `safety.hara`, `safety.asil`, `safety.dal`, `safety.mitigation`, `timing.latency`, `timing.budget`, `timing.units`, `icd.completeness`, `constraints.violated`, `metamodel.types`, `methodology`.\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traceability_lines_name_the_element_and_a_trace_fix() {
        let finding = classify_traceability("Requirement REQ-001 has no trace");
        assert_eq!(finding.rule, "trace.requirement");
        assert_eq!(finding.severity, Severity::Warning);
        assert_eq!(finding.element.as_deref(), Some("REQ-001"));
        assert_eq!(finding.fix, Some(FixHint::Trace { element: "REQ-001".to_string() }));
        assert_eq!(classify_traceability("Component LC-7 has no trace").rule, "trace.component");
    }

    #[test]
    fn metamodel_lines_name_the_attribute_to_set() {
        let finding = classify_warning("metamodel: LogicalComponent 'LC-001'.asil: 'High' is not an Asil value (QM, ASIL-A, ASIL-B, ASIL-C, ASIL-D)");
        assert_eq!(finding.rule, "metamodel.type");
        assert_eq!(finding.severity, Severity::Error);
        assert_eq!(finding.element.as_deref(), Some("LC-001"));
        assert_eq!(finding.fix, Some(FixHint::Set { element: "LC-001".to_string(), key: "asil".to_string() }));
        let unit = classify_warning("metamodel: LogicalFunction 'F1'.latency: bare number 25 — the unit is assumed to be ms; write `25 ms`");
        assert_eq!(unit.rule, "metamodel.unit");
        let dimension = classify_warning("metamodel: LogicalFunction 'F1'.latency: '25 MHz' is a Frequency, expected a Time (e.g. `latency: 1 ms`)");
        assert_eq!(dimension.rule, "metamodel.type");
        let reference = classify_warning("metamodel: Trace 'T'.to: 'LC-1' is a LogicalComponent, expected a Requirement");
        assert_eq!(reference.rule, "metamodel.reference");
    }

    #[test]
    fn lints_and_unknown_lines_fall_in_their_families() {
        assert_eq!(classify_lint("physical architecture without a logical architecture: PA elements have nothing to realize").rule, "method.layer-order");
        assert_eq!(classify_lint("system analysis without an operational analysis: no operational need is captured").rule, "method.layer-order");
        let functions = classify_lint("logical component 'Gateway' performs no functions — Arcadia expects logical components to realize behavior");
        assert_eq!((functions.rule, functions.element.as_deref()), ("method.component-functions", Some("Gateway")));
        assert_eq!(classify_lint("something new").rule, "method.other");
        assert_eq!(classify_warning("something else").rule, "compile.other");
        assert_eq!(classify_warning("duplicate element id 'X': A 'a' and B 'b' share the same identity").fix, Some(FixHint::Identify { element: "X".to_string() }));
    }

    #[test]
    fn every_rule_is_documented_and_unique() {
        let mut ids: Vec<&str> = RULES.iter().map(|rule| rule.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), RULES.len());
        for rule in RULES {
            assert!(!rule.doc.is_empty() && !rule.fix.is_empty() && !rule.title.is_empty(), "{}", rule.id);
        }
        assert!(to_markdown().contains("| `trace.requirement` |"));
    }
}
