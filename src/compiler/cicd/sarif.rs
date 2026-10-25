//! SARIF 2.1.0: the diagnostics of the compiler and the blockers of the
//! gate, as a static-analysis tool reports them, for GitHub code scanning
//! and Azure DevOps. One run, one result per diagnostic, one rule per rule
//! id with its meaning and how to answer it.

use super::diag::{Diagnostic, Level, ModelRun, Precision};
use crate::compiler::rules;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const SCHEMA: &str = "https://json.schemastore.org/sarif-2.1.0.json";
pub const INFORMATION_URI: &str = "https://github.com/Mbaroudi/arclang";

/// What the tool knows of a rule: its title, its meaning, how to answer it.
struct RuleInfo {
    title: String,
    doc: String,
    fix: String,
    level: Level,
}

fn gate_rule(check: &str) -> RuleInfo {
    let (title, doc, fix) = match check {
        "requirements.presence" => ("No requirement", "The production gate needs requirements to measure the coverage of.", "Declare the requirements the design answers."),
        "requirements.satisfaction" => ("Requirement not satisfied", "No element satisfies the requirement: the design does not answer it.", "Declare `trace \"<element>\" satisfies \"<requirement>\"`."),
        "requirements.verification" => ("Requirement not verified", "No passing verification case covers the requirement, or a case fails or is inconclusive.", "Declare a test case that `verifies:` the requirement and make it pass (`arclang verify`)."),
        "verification.dangling" => ("Verification case without a requirement", "A test case verifies nothing that exists.", "Point `verifies:` at an existing requirement."),
        "safety.hara" => ("Incomplete hazard analysis", "A hazard misses what the standard requires to rate it (severity, exposure, controllability).", "Complete the hazard's attributes."),
        "safety.asil" => ("ASIL inconsistent", "The integrity level of an element does not follow from its hazard, or a decomposition is inconsistent.", "Correct the `asil` or the hazard rating."),
        "safety.dal" => ("DAL inconsistent", "The design assurance level does not follow from the failure condition.", "Correct the `dal` or the failure condition."),
        "safety.mitigation" => ("Hazard without a mitigation", "A hazard is not mitigated by any element.", "Declare the element that mitigates it."),
        "timing.latency" => ("Latency not stated", "A function on a timed chain has no latency.", "State the function's latency with a unit: `latency: 20 ms`."),
        "timing.budget" => ("Latency budget exceeded", "The latencies on a functional chain sum to more than its budget.", "Reduce a latency, or revise the budget with the people who own it."),
        "timing.units" => ("Time without a unit", "A latency is a bare number.", "Write the unit: `latency: 25 ms`."),
        "icd.completeness" => ("Interface description incomplete", "An exchange misses what the interface control document needs (carrying link, frequency, message type).", "Complete the exchange's attributes."),
        "constraints.violated" => ("Constraint violated", "A declared constraint is false on the model's values.", "Change the values or the constraint so that the model holds what it claims."),
        "metamodel.types" => ("Typed attribute violated", "An attribute does not fit its declared type.", "Write a value of the declared type (`arclang metamodel`)."),
        "methodology" => ("Methodology advisory", "The Arcadia method expects something else here.", "Read the advisory."),
        _ => ("Production gate finding", "A check of the production gate.", "Read the finding."),
    };
    RuleInfo { title: title.into(), doc: doc.into(), fix: fix.into(), level: Level::Error }
}

fn rule_info(id: &str) -> RuleInfo {
    if let Some(check) = id.strip_prefix("gate.") {
        return gate_rule(check);
    }
    match id {
        "compile.error" => RuleInfo { title: "The model does not compile".into(), doc: "The compiler stops at a syntax or semantic error: nothing else about the model can be said.".into(), fix: "Fix the error at the location given.".into(), level: Level::Error },
        "verification.failed" => RuleInfo { title: "Simulation verdict: fail".into(), doc: "A test case that simulates ran and an expectation does not hold.".into(), fix: "Correct the model or the expectation; run `arclang verify`.".into(), level: Level::Error },
        "verification.inconclusive" => RuleInfo { title: "Simulation verdict: inconclusive".into(), doc: "A test case that simulates could not be run (no OpenModelica, a stale cache): it is not verified.".into(), fix: "Run `arclang verify` where OpenModelica or Docker is available.".into(), level: Level::Warning },
        other => match rules::rule(other) {
            Some(rule) => RuleInfo { title: rule.title.into(), doc: rule.doc.into(), fix: rule.fix.into(), level: super::diag::Level::from_rule(rule.severity) },
            None => RuleInfo { title: other.into(), doc: String::new(), fix: String::new(), level: Level::Warning },
        },
    }
}

impl Level {
    fn from_rule(severity: rules::Severity) -> Level {
        match severity {
            rules::Severity::Error => Level::Error,
            rules::Severity::Warning => Level::Warning,
            rules::Severity::Info => Level::Note,
        }
    }
}

fn pascal(id: &str) -> String {
    id.split(['.', '-', '_']).map(|w| w.chars().next().map(|c| c.to_uppercase().collect::<String>() + &w[c.len_utf8()..]).unwrap_or_default()).collect()
}

fn fingerprint(d: &Diagnostic) -> String {
    let mut hash = Sha256::new();
    for part in [&d.rule, &d.file, d.element.as_deref().unwrap_or(""), &d.message] {
        hash.update((part.len() as u64).to_le_bytes());
        hash.update(part.as_bytes());
    }
    hash.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

fn rule_json(id: &str) -> Value {
    let info = rule_info(id);
    let mut rule = json!({
        "id": id,
        "name": pascal(id),
        "shortDescription": { "text": info.title },
        "fullDescription": { "text": if info.doc.is_empty() { info.title.clone() } else { info.doc.clone() } },
        "defaultConfiguration": { "level": info.level.sarif() },
        "helpUri": format!("{INFORMATION_URI}/blob/main/docs/CI_CD.md"),
        "properties": { "tags": if id.starts_with("gate.") { json!(["production-gate"]) } else { json!(["model"]) } },
    });
    if !info.fix.is_empty() {
        rule["help"] = json!({ "text": info.fix, "markdown": info.fix });
    }
    rule
}

fn result_json(d: &Diagnostic, rule_index: usize) -> Value {
    let mut region = Map::new();
    region.insert("startLine".into(), json!(d.line.max(1)));
    if let Some(column) = d.column {
        region.insert("startColumn".into(), json!(column.max(1)));
    }
    let mut location = json!({
        "physicalLocation": {
            "artifactLocation": { "uri": d.file, "uriBaseId": "%SRCROOT%" },
            "region": Value::Object(region),
        }
    });
    if let Some(element) = &d.element {
        location["logicalLocations"] = json!([{ "name": element, "kind": "element" }]);
    }
    let info = rule_info(&d.rule);
    let mut properties = json!({
        "locationPrecision": match d.precision { Precision::Exact => "exact", Precision::Element => "element", Precision::File => "file" },
    });
    if let Some(check) = &d.gate_check {
        properties["gateCheck"] = json!(check);
    }
    if let Some(fix) = &d.fix {
        properties["fixHint"] = serde_json::to_value(fix).unwrap_or(Value::Null);
    }
    let message = if info.fix.is_empty() || d.rule == "compile.error" { d.message.clone() } else { format!("{}. Fix: {}", d.message.trim_end_matches('.'), info.fix) };
    json!({
        "ruleId": d.rule,
        "ruleIndex": rule_index,
        "level": d.level.sarif(),
        "message": { "text": message },
        "locations": [location],
        "partialFingerprints": { "arclang/v1": fingerprint(d) },
        "properties": properties,
    })
}

/// A SARIF log of the examined models. `all_models` lists the models looked
/// at, so that a clean run still says what it covered.
pub fn log(runs: &[ModelRun], tool_version: &str) -> Value {
    let mut rule_ids: BTreeMap<String, usize> = BTreeMap::new();
    for d in runs.iter().flat_map(|r| &r.diagnostics) {
        rule_ids.entry(d.rule.clone()).or_insert(0);
    }
    for (index, (_, slot)) in rule_ids.iter_mut().enumerate() {
        *slot = index;
    }
    let rules: Vec<Value> = rule_ids.keys().map(|id| rule_json(id)).collect();
    let results: Vec<Value> = runs.iter().flat_map(|r| &r.diagnostics).map(|d| result_json(d, rule_ids[&d.rule])).collect();
    let artifacts: Vec<Value> = runs.iter().map(|r| json!({ "location": { "uri": r.model, "uriBaseId": "%SRCROOT%" }, "roles": ["analysisTarget"] })).collect();
    json!({
        "$schema": SCHEMA,
        "version": "2.1.0",
        "runs": [{
            "tool": { "driver": {
                "name": "ArcLang",
                "semanticVersion": tool_version,
                "version": tool_version,
                "informationUri": INFORMATION_URI,
                "rules": rules,
            }},
            "artifacts": artifacts,
            "results": results,
            "invocations": [{ "executionSuccessful": true }],
        }]
    })
}

pub fn text(runs: &[ModelRun], tool_version: &str) -> String {
    format!("{}\n", serde_json::to_string_pretty(&log(runs, tool_version)).unwrap_or_default())
}
