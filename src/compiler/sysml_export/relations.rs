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
/// from the reference attribute of the same key. One that says something
/// carries the metadata in its body, with what it says.
fn trace_dependency(e: &mut Exporter<'_>, key: &str, from: &str, to: &str, said: &str) {
    let prefix = if said == ";" { "#Trace " } else { "" };
    match (e.reference(from), e.reference(to)) {
        (Some(source), Some(target)) => e.line(&format!("{prefix}dependency {} from {source} to {target}{said}", text::name(key))),
        _ => e.line(&format!("// trace {from} {key} {to}: an end is not exported")),
    }
}

/// What a trace says besides its ends is not said by its ends.
const ENDS_OF_A_TRACE: [&str; 2] = ["is", "trace_type"];

/// The keys the traces of the model say something under, with the type of
/// what they say: the attributes of the `Trace` metadata definition.
pub(super) fn said_by_traces(model: &Model) -> Vec<(String, String)> {
    let mut keys: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    let mut units = std::collections::BTreeSet::new();
    for trace in model.traces.iter().chain(model.operational_analysis.iter().flat_map(|oa| oa.traces.iter())) {
        for (key, value) in &trace.attributes {
            if !ENDS_OF_A_TRACE.contains(&key.as_str()) {
                keys.entry(key.clone()).or_insert_with(|| super::attributes::scalar(value, &mut units).0);
            }
        }
    }
    keys.into_iter().collect()
}

/// What a trace says besides its ends (why it holds, the change it was
/// reviewed against): `{ @Trace { rationale = "..."; } }`, an annotation a
/// relationship can carry, or `;` when it says nothing.
fn said(e: &mut Exporter<'_>, trace: &Trace) -> String {
    // What its type gives it unchanged is the type's, not the trace's.
    let given = e.inherited(&trace.attributes).map(|i| i.attributes).unwrap_or_default();
    let mut keys: Vec<&String> = trace.attributes.iter().filter(|(k, v)| !ENDS_OF_A_TRACE.contains(&k.as_str()) && given.get(*k) != Some(*v)).map(|(k, _)| k).collect();
    if keys.is_empty() {
        return ";".to_string();
    }
    keys.sort();
    let values: Vec<String> = keys.iter().map(|key| format!("{} = {};", text::name(key), super::attributes::scalar(&trace.attributes[*key], &mut e.units).1)).collect();
    format!(" {{ @Trace {{ {} }} }}", values.join(" "))
}

/// `allocate` of the usage of `from` to the usage of `to`.
pub(super) fn allocation(e: &mut Exporter<'_>, from: &str, to: &str) {
    allocation_saying(e, from, to, ";")
}

fn allocation_saying(e: &mut Exporter<'_>, from: &str, to: &str, said: &str) {
    match (e.feature(from), e.feature(to)) {
        (Some(source), Some(target)) => e.line(&format!("allocate {source} to {target}{said}")),
        // An end no feature chain reaches (a variant): a dependency, by name.
        _ => trace_dependency(e, "allocates", from, to, said),
    }
}

/// An allocation, typed by its allocation definition when it `is:` one
/// (`allocation : 'Software deployment' allocate a to b;`).
fn typed_allocation(e: &mut Exporter<'_>, trace: &Trace) {
    let typing = e.typing(&trace.attributes);
    let said = said(e, trace);
    if typing.is_empty() {
        return allocation_saying(e, &trace.from, &trace.to, &said);
    }
    match (e.feature(&trace.from), e.feature(&trace.to)) {
        (Some(source), Some(target)) => e.line(&format!("allocation{typing} allocate {source} to {target}{said}")),
        _ => trace_dependency(e, "allocates", &trace.from, &trace.to, &said),
    }
}

/// A requirement: its id as short name, its title as name, its text as doc;
/// what it is about as the default of its `subject`, the concerns it addresses `frame`d,
/// its constraints `require`d and `assume`d, each its expression as text.
/// One written by its name with an `id` (`requirement "Hover endurance" { id: "REQ-1" }`)
/// carries `#Named`: its name is its name, where the name of another is its title.
/// The configurations it applies to are dependencies named `applies_to`.
pub(super) fn requirement(e: &mut Exporter<'_>, requirement: &Requirement) {
    let attributes = &requirement.attributes;
    let id = id_of(attributes, &requirement.id).to_string();
    let is_named = requirement.id != id;
    let title = attributes.get("title").and_then(|t| t.as_string());
    let display = if is_named { requirement.id.as_str() } else { title.unwrap_or(&id) };
    let prefix = if is_named { "#Named " } else { "" };
    e.block(&format!("{prefix}requirement {}", text::declared(&id, display)), |e| {
        e.header("Requirement", attributes, &["title", "frames", "subject", "require", "assume"]);
        if let Some(title) = title.filter(|_| is_named) {
            e.line(&format!("attribute title : String = {};", text::string(title)));
        }
        if let Some(subject) = attributes.get("subject").and_then(|s| s.as_string()) {
            match e.reference(subject) {
                // A default, not a binding: `satisfy R by X` binds the subject of R, and a
                // subject already bound cannot be ("Cannot override a binding feature value").
                Some(target) => e.line(&format!("subject default = {target};")),
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
    for configuration in list(attributes, "applies_to") {
        dependency(e, "applies_to", &id, &configuration);
    }
}

/// A stakeholder (CESAM stakeholder analysis): a part definition carrying
/// `#Stakeholder`, which tells it from a component, its attributes kept by
/// the header.
pub(super) fn stakeholder(e: &mut Exporter<'_>, stakeholder: &Stakeholder) {
    let id = id_of(&stakeholder.attributes, &stakeholder.id).to_string();
    let name = stakeholder.attributes.get("name").and_then(|t| t.as_string()).unwrap_or(&stakeholder.id).to_string();
    e.block(&format!("#Stakeholder part def {}", text::declared(&id, &name)), |e| {
        e.header("Stakeholder", &stakeholder.attributes, &[]);
    });
}

/// A concern: its id as short name, its name as name, its text as doc.
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
            "satisfies" => {
                let said = said(e, trace);
                match (is_requirement(model, &trace.to).then(|| e.reference(&trace.to)).flatten(), e.feature(&trace.from)) {
                    (Some(requirement), Some(by)) => e.line(&format!("satisfy {requirement} by {by}{said}")),
                    _ => trace_dependency(e, "satisfies", &trace.from, &trace.to, &said),
                }
            }
            "allocates" => typed_allocation(e, trace),
            kind => {
                let said = said(e, trace);
                trace_dependency(e, kind, &trace.from, &trace.to, &said)
            }
        }
    }
}
