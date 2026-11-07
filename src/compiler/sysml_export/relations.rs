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

/// `#Trace dependency realizes from A to B;`: a trace, told by its metadata
/// from the reference attribute of the same key.
fn trace_dependency(e: &mut Exporter<'_>, key: &str, from: &str, to: &str) {
    match (e.reference(from), e.reference(to)) {
        (Some(source), Some(target)) => e.line(&format!("#Trace dependency {} from {source} to {target};", text::name(key))),
        _ => e.line(&format!("// trace {from} {key} {to}: an end is not exported")),
    }
}

/// `allocate` of the usage of `from` to the usage of `to`.
pub(super) fn allocation(e: &mut Exporter<'_>, from: &str, to: &str) {
    match (e.feature(from), e.feature(to)) {
        (Some(source), Some(target)) => e.line(&format!("allocate {source} to {target};")),
        // An end no feature chain reaches (a variant): a dependency, by name.
        _ => trace_dependency(e, "allocates", from, to),
    }
}

/// An allocation, typed by its allocation definition when it `is:` one
/// (`allocation : 'Software deployment' allocate a to b;`).
fn typed_allocation(e: &mut Exporter<'_>, trace: &Trace) {
    let typing = e.typing(&trace.attributes);
    if typing.is_empty() {
        return allocation(e, &trace.from, &trace.to);
    }
    match (e.feature(&trace.from), e.feature(&trace.to)) {
        (Some(source), Some(target)) => e.line(&format!("allocation{typing} allocate {source} to {target};")),
        _ => trace_dependency(e, "allocates", &trace.from, &trace.to),
    }
}

/// A requirement: its id as short name, its title as name, its text as doc;
/// what it is about as its `subject`, the concerns it addresses `frame`d,
/// its constraints `require`d and `assume`d, each its expression as text.
pub(super) fn requirement(e: &mut Exporter<'_>, requirement: &Requirement) {
    let attributes = &requirement.attributes;
    let id = id_of(attributes, &requirement.id).to_string();
    let title = attributes.get("title").and_then(|t| t.as_string()).unwrap_or(&id).to_string();
    e.block(&format!("requirement {}", text::declared(&id, &title)), |e| {
        e.header("Requirement", attributes, &["title", "frames", "subject", "require", "assume"]);
        if let Some(subject) = attributes.get("subject").and_then(|s| s.as_string()) {
            match e.reference(subject) {
                Some(target) => e.line(&format!("subject = {target};")),
                None => e.line(&format!("// subject {subject}: not exported")),
            }
        }
        for concern in list(attributes, "frames") {
            match e.reference(&concern) {
                Some(target) => e.line(&format!("frame {target};")),
                None => e.line(&format!("// frame {concern}: not exported")),
            }
        }
        for key in ["require", "assume"] {
            for constraint in list(attributes, key) {
                e.block(&format!("{key} constraint"), |e| e.line(&format!("attribute expression : String = {};", text::string(&constraint))));
            }
        }
    });
}

/// A concern: its id as short name, its name as name, its text as doc.
/// A stakeholder (CESAM stakeholder analysis): a part definition, its
/// attributes kept by the header.
pub(super) fn stakeholder(e: &mut Exporter<'_>, stakeholder: &Stakeholder) {
    let id = id_of(&stakeholder.attributes, &stakeholder.id).to_string();
    let name = stakeholder.attributes.get("name").and_then(|t| t.as_string()).unwrap_or(&stakeholder.id).to_string();
    e.block(&format!("part def {}", text::declared(&id, &name)), |e| {
        e.header("Stakeholder", &stakeholder.attributes, &[]);
    });
}

pub(super) fn concern(e: &mut Exporter<'_>, concern: &Concern) {
    let id = id_of(&concern.attributes, &concern.id).to_string();
    let name = concern.attributes.get("name").and_then(|t| t.as_string()).unwrap_or(&concern.id).to_string();
    e.block(&format!("concern {}", text::declared(&id, &name)), |e| {
        e.header("Concern", &concern.attributes, &["parent"]);
        if let Some(parent) = concern.attributes.get("parent").and_then(|p| p.as_string()) {
            dependency(e, "parent", &id, parent);
        }
    });
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
    for config in &model.configurations {
        e.register("", &config.id, &[&config.name]);
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
    // A configuration of the product line: what it selects, by dependency.
    for config in &model.configurations {
        e.block(&format!("#Configuration part {}", text::declared(&config.id, &config.name)), |e| e.header("Configuration", &config.attributes, &["selects"]));
        for selected in &config.selects {
            dependency(e, "selects", &config.id, selected);
        }
    }
    // A note: a comment about the elements it names (its id, when written, names it).
    for note in &model.notes {
        let about: Vec<String> = note.about.iter().filter_map(|r| e.reference(r)).collect();
        if about.is_empty() {
            continue;
        }
        let name = if note.id.is_empty() { String::new() } else { format!(" {}", text::name(&note.id)) };
        e.line(&format!("comment{name} about {} {}", about.join(", "), text::doc(&note.name).trim_start_matches("doc ")));
    }
    for test in &model.test_cases {
        let id = if test.id.is_empty() { test.name.clone() } else { test.id.clone() };
        let verified: Vec<String> = test.verifies.iter().filter_map(|r| e.reference(r)).collect();
        e.block(&format!("verification {}", text::declared(&id, &test.name)), |e| {
            e.header("TestCase", &test.attributes, &["method", "asserts"]);
            if !test.method.is_empty() {
                e.line(&format!("attribute method : String = {};", text::string(&test.method)));
            }
            e.block("objective", |e| {
                for requirement in verified {
                    e.line(&format!("verify {requirement};"));
                }
            });
            // An analysis: the constraints whose evaluation is its verdict.
            for asserted in crate::compiler::verification::asserted(test) {
                let id = model.constraints.iter().find(|c| c.id == asserted || c.name == asserted).map_or(asserted.clone(), |c| c.id.clone());
                e.line(&format!("assert {};", text::name(&id)));
            }
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
                _ => trace_dependency(e, "satisfies", &trace.from, &trace.to),
            },
            "allocates" => typed_allocation(e, trace),
            kind => trace_dependency(e, kind, &trace.from, &trace.to),
        }
    }
}
