//! Relationships: what SysML says natively (`satisfy`, `allocate`,
//! `verify`) is said so; any other reference of ArcLang (`involves`,
//! `realizes`, `mission`...) is a `dependency` named by its key, from the
//! element to what it names. Requirements, hazards, FMEA entries, test
//! cases and constraints are written here too, at the model level.

use super::{id_of, text, Exporter};
use crate::compiler::ast::*;
use std::collections::HashMap;

pub(super) const PACKAGE: &str = "Safety";

/// The text items of a list attribute (`allocated_functions: [A, B]`).
pub(super) fn list(attributes: &HashMap<String, AttributeValue>, key: &str) -> Vec<String> {
    match attributes.get(key) {
        Some(AttributeValue::List(items)) => items.iter().map(|i| i.display()).collect(),
        Some(AttributeValue::String(one)) if !one.is_empty() => vec![one.clone()],
        _ => Vec::new(),
    }
}

/// `dependency involves from 'CAP-1' to 'System Analysis'::'SF-1';`
pub(super) fn dependency(e: &mut Exporter<'_>, key: &str, from: &str, to: &str) {
    match (e.reference(from), e.reference(to)) {
        (Some(source), Some(target)) => e.line(&format!("dependency {} from {source} to {target};", text::name(key))),
        _ => e.line(&format!("// {from} {key} {to}: an end is not exported")),
    }
}

/// `allocate` of the usage of `from` to the usage of `to`.
pub(super) fn allocation(e: &mut Exporter<'_>, from: &str, to: &str) {
    match (e.feature(from), e.feature(to)) {
        (Some(source), Some(target)) => e.line(&format!("allocate {source} to {target};")),
        _ => e.line(&format!("// {from} allocated to {to}: an end is not exported")),
    }
}

/// A requirement: its id as short name, its title as name, its text as doc.
pub(super) fn requirement(e: &mut Exporter<'_>, requirement: &Requirement) {
    let id = id_of(&requirement.attributes, &requirement.id).to_string();
    let title = requirement.attributes.get("title").and_then(|t| t.as_string()).unwrap_or(&id).to_string();
    e.block(&format!("requirement {}", text::declared(&id, &title)), |e| e.header("Requirement", &requirement.attributes, &["title"]));
}

pub(super) fn register(e: &mut Exporter<'_>) {
    let model = e.model;
    for analysis in &model.safety_analysis {
        for hazard in &analysis.hazards {
            e.register(PACKAGE, id_of(&hazard.attributes, &hazard.name), &[&hazard.name]);
        }
        for entry in &analysis.fmea {
            e.register(PACKAGE, id_of(&entry.attributes, &entry.name), &[&entry.name]);
        }
    }
    for test in &model.test_cases {
        let id = if test.id.is_empty() { &test.name } else { &test.id };
        e.register("", id, &[&test.name]);
    }
}

pub(super) fn write(e: &mut Exporter<'_>) {
    let model = e.model;
    let safety = model.safety_analysis.iter().any(|s| !s.hazards.is_empty() || !s.fmea.is_empty());
    if safety {
        e.package(&format!("package {PACKAGE}"), PACKAGE, |e| {
            for analysis in &model.safety_analysis {
                for hazard in &analysis.hazards {
                    let id = id_of(&hazard.attributes, &hazard.name).to_string();
                    e.block(&format!("#Hazard requirement {}", text::declared(&id, &hazard.name)), |e| e.header("Hazard", &hazard.attributes, &[]));
                    for requirement in list(&hazard.attributes, "mitigated_by") {
                        dependency(e, "mitigated_by", &id, &requirement);
                    }
                }
                for entry in &analysis.fmea {
                    let id = id_of(&entry.attributes, &entry.name).to_string();
                    e.block(&format!("#Fmea requirement {}", text::declared(&id, &entry.name)), |e| e.header("FmeaEntry", &entry.attributes, &[]));
                }
            }
        });
    }
    for test in &model.test_cases {
        let id = if test.id.is_empty() { test.name.clone() } else { test.id.clone() };
        let verified: Vec<String> = test.verifies.iter().filter_map(|r| e.reference(r)).collect();
        e.block(&format!("verification {}", text::declared(&id, &test.name)), |e| {
            e.header("TestCase", &test.attributes, &[]);
            if !test.method.is_empty() {
                e.line(&format!("attribute method : String = {};", text::string(&test.method)));
            }
            e.block("objective", |e| {
                for requirement in verified {
                    e.line(&format!("verify {requirement};"));
                }
            });
        });
    }
    super::constraints::write(e);
    traces(e);
}

fn is_requirement(model: &Model, reference: &str) -> bool {
    model.system_analysis.iter().flat_map(|sa| sa.requirements.iter()).any(|r| r.id == reference || id_of(&r.attributes, &r.id) == reference)
}

fn traces(e: &mut Exporter<'_>) {
    let model = e.model;
    let all: Vec<&Trace> = model.traces.iter().chain(model.operational_analysis.iter().flat_map(|oa| oa.traces.iter())).collect();
    for trace in all {
        match trace.trace_type.as_str() {
            // A `satisfy` is of a requirement, by a feature; anything else
            // that "satisfies" is a dependency of that name.
            "satisfies" => match (is_requirement(model, &trace.to).then(|| e.reference(&trace.to)).flatten(), e.feature(&trace.from)) {
                (Some(requirement), Some(by)) => e.line(&format!("satisfy {requirement} by {by};")),
                _ => dependency(e, "satisfies", &trace.from, &trace.to),
            },
            "allocates" => allocation(e, &trace.from, &trace.to),
            kind => dependency(e, kind, &trace.from, &trace.to),
        }
    }
}
