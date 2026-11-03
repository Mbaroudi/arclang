//! Timeslices and snapshots (SysML v2 portions of an occurrence): the
//! stretches and moments of an element's life, each with the values the
//! element has then. A timeslice may say when it starts and ends (`start:`,
//! `end:`), a snapshot when it is (`at:`), as times; a snapshot of a
//! timeslice is within it; a value the element declares as a quantity keeps
//! its dimension in a snapshot. A simulated test case checks snapshots
//! (`snapshots: ["LC-DRONE"]`): each value of each one is an expectation on
//! the result at its time, within its tolerance
//! (`at(40 s, "LC-DRONE".altitude) == 120 m within 1 %`).

use super::ast::{AttributeValue, Model, Occurrence, OccurrenceKind};
use super::quantity::Dimension;
use std::collections::{HashMap, HashSet};

/// Keys of a timeslice or snapshot that say when, not what the element is then.
pub const TIME_KEYS: [&str; 3] = ["start", "end", "at"];

/// How far a simulated value may be from a snapshot's and still reach it,
/// when the snapshot says nothing (`tolerance: 0.5 m`, `tolerance: 2 %`).
pub const DEFAULT_TOLERANCE: &str = "1 %";

/// Keys of a snapshot that are no value of the element.
fn is_value_key(key: &str) -> bool {
    !TIME_KEYS.contains(&key) && !matches!(key, "id" | "description" | "tolerance")
}

/// Every element with timeslices or snapshots: its id, its attributes and them.
pub fn owners(model: &Model) -> Vec<(&str, &HashMap<String, AttributeValue>, &[Occurrence])> {
    let mut out: Vec<(&str, &HashMap<String, AttributeValue>, &[Occurrence])> = Vec::new();
    fn components<'m>(list: &'m [super::ast::LogicalComponent], out: &mut Vec<(&'m str, &'m HashMap<String, AttributeValue>, &'m [Occurrence])>) {
        for component in list {
            if !component.occurrences.is_empty() {
                out.push((&component.id, &component.attributes, &component.occurrences));
            }
            components(&component.sub_components, out);
        }
    }
    for level in &model.logical_architecture {
        components(&level.components, &mut out);
    }
    for level in &model.physical_architecture {
        for node in level.nodes.iter().filter(|n| !n.occurrences.is_empty()) {
            out.push((&node.id, &node.attributes, &node.occurrences));
        }
    }
    out
}

/// The id of a timeslice or snapshot: its `id:`, else `Owner.Name`.
pub fn id_of(owner: &str, occurrence: &Occurrence) -> String {
    occurrence.attributes.get("id").and_then(|v| v.as_string()).map(str::to_string).unwrap_or_else(|| format!("{owner}.{}", occurrence.name))
}

/// A time attribute in seconds; an error when it is not a time.
fn time(occurrence: &Occurrence, key: &str) -> Result<Option<f64>, String> {
    let Some(value) = occurrence.attributes.get(key) else { return Ok(None) };
    match value.as_quantity() {
        Some(Ok(quantity)) if quantity.dimension() == Dimension::Time => Ok(Some(quantity.canonical())),
        _ => Err(format!("{} '{}': {key}: {} is not a time (write `{key}: 30 s`)", occurrence.kind.keyword(), occurrence.name, value.display())),
    }
}

fn dimension(value: &AttributeValue) -> Option<Dimension> {
    match value {
        AttributeValue::Quantity(quantity) => Some(quantity.dimension()),
        _ => None,
    }
}

/// The snapshots `written` stands for: one snapshot, the snapshots of a
/// timeslice, or every snapshot of an element; each with its element's id.
fn snapshots_of<'m>(model: &'m Model, written: &str) -> Option<Vec<(&'m str, String, &'m Occurrence)>> {
    fn all<'m>(owner: &'m str, prefix: &str, occurrences: &'m [Occurrence], out: &mut Vec<(&'m str, String, &'m Occurrence)>, only: Option<&str>) {
        for occurrence in occurrences {
            let id = id_of(prefix, occurrence);
            let wanted = only.is_none_or(|o| o == id || o.starts_with(&format!("{id}.")));
            match occurrence.kind {
                OccurrenceKind::Snapshot if wanted && only.is_none_or(|o| o == id) => out.push((owner, id, occurrence)),
                OccurrenceKind::Timeslice if wanted => all(owner, &id, &occurrence.snapshots, out, only.filter(|o| *o != id)),
                _ => {}
            }
        }
    }
    let owners = owners(model);
    let (owner, _, occurrences) = owners.iter().find(|(id, _, _)| *id == written || written.starts_with(&format!("{id}.")))?;
    let mut out = Vec::new();
    all(owner, owner, occurrences, &mut out, (written != *owner).then_some(written));
    (!out.is_empty()).then_some(out)
}

/// The expectations a test case's `snapshots:` stand for, on a simulation
/// result: every value of every snapshot, at its time.
pub fn expectations(model: &Model, attributes: &HashMap<String, AttributeValue>) -> Result<Vec<String>, String> {
    let written: Vec<String> = match attributes.get("snapshots") {
        Some(AttributeValue::List(items)) => items.iter().filter_map(|i| i.as_string()).map(str::to_string).collect(),
        Some(AttributeValue::String(one)) => vec![one.clone()],
        _ => return Ok(Vec::new()),
    };
    let mut out = Vec::new();
    for reference in &written {
        let snapshots = snapshots_of(model, reference).ok_or_else(|| format!("snapshots: '{reference}' is no element with snapshots, no timeslice and no snapshot"))?;
        for (owner, id, snapshot) in snapshots {
            let at = snapshot.attributes.get("at").ok_or_else(|| format!("snapshots: '{id}' says no time: give it `at:` to check it on a simulation"))?;
            let mut keys: Vec<&String> = snapshot.attributes.keys().filter(|k| is_value_key(k)).collect();
            let tolerance = snapshot.attributes.get("tolerance").map(|t| t.display()).unwrap_or_else(|| DEFAULT_TOLERANCE.to_string());
            keys.sort();
            if keys.is_empty() {
                return Err(format!("snapshots: '{id}' gives no value to check"));
            }
            for key in keys {
                let value = &snapshot.attributes[key];
                if !matches!(value, AttributeValue::Quantity(_) | AttributeValue::Number(_) | AttributeValue::Integer(_)) {
                    return Err(format!("snapshots: '{id}': {key} is {}, not a number a simulation gives", value.display()));
                }
                out.push(format!("at({}, \"{owner}\".{key}) == {} within {tolerance}", at.display(), value.display()));
            }
        }
    }
    Ok(out)
}

/// What is wrong with the timeslices and snapshots of the model.
pub fn check(model: &Model) -> Vec<String> {
    let mut errors = Vec::new();
    for case in model.test_cases.iter().filter(|c| c.attributes.contains_key("snapshots")) {
        if !case.attributes.contains_key("simulate") {
            errors.push(format!("test_case '{}': snapshots: are checked on a simulation: give it simulate: {{ ... }}", case.name));
        }
        match expectations(model, &case.attributes) {
            Ok(expects) => {
                for expect in expects {
                    if let Err(error) = super::simulation::expects::check_syntax(&expect) {
                        errors.push(format!("test_case '{}': {error}", case.name));
                    }
                }
            }
            Err(error) => errors.push(format!("test_case '{}': {error}", case.name)),
        }
    }
    for (owner, attributes, occurrences) in owners(model) {
        let mut names: HashSet<&str> = HashSet::new();
        for occurrence in occurrences {
            if !names.insert(&occurrence.name) {
                errors.push(format!("'{owner}': two timeslices or snapshots are named '{}'", occurrence.name));
            }
            check_one(owner, attributes, occurrence, None, &mut errors);
        }
    }
    errors
}

fn check_one(owner: &str, attributes: &HashMap<String, AttributeValue>, occurrence: &Occurrence, within: Option<(Option<f64>, Option<f64>, &str)>, errors: &mut Vec<String>) {
    let mut times = |key: &str| match time(occurrence, key) {
        Ok(t) => t,
        Err(e) => {
            errors.push(format!("'{owner}': {e}"));
            None
        }
    };
    let (start, end, at) = (times("start"), times("end"), times("at"));
    let what = format!("'{owner}': {} '{}'", occurrence.kind.keyword(), occurrence.name);
    match occurrence.kind {
        OccurrenceKind::Timeslice => {
            if occurrence.attributes.contains_key("at") {
                errors.push(format!("{what}: a timeslice has a start and an end, not an `at`"));
            }
            if let (Some(s), Some(e)) = (start, end) {
                if s > e {
                    errors.push(format!("{what}: it ends before it starts"));
                }
            }
        }
        OccurrenceKind::Snapshot => {
            if occurrence.attributes.contains_key("start") || occurrence.attributes.contains_key("end") {
                errors.push(format!("{what}: a snapshot is one moment: say when with `at`, not `start`/`end`"));
            }
            if let (Some(at), Some((from, to, slice))) = (at, within) {
                if from.is_some_and(|f| at < f) || to.is_some_and(|t| at > t) {
                    let written = occurrence.attributes.get("at").map(|v| v.display()).unwrap_or_default();
                    errors.push(format!("{what}: at {written} is outside its timeslice '{slice}'"));
                }
            }
        }
    }
    // A value of the element keeps its dimension in a snapshot of it.
    if let Some(tolerance) = occurrence.attributes.get("tolerance") {
        let is_percent = matches!(tolerance.as_quantity(), Some(Ok(q)) if q.spec().symbol == "%");
        let is_amount = matches!(tolerance, AttributeValue::Quantity(_) | AttributeValue::Number(_) | AttributeValue::Integer(_));
        if occurrence.kind == OccurrenceKind::Timeslice || !(is_percent || is_amount) {
            errors.push(format!("{what}: tolerance is how far a simulated value may be from a snapshot's: a percentage or an amount, on a snapshot"));
        }
    }
    for (key, value) in occurrence.attributes.iter().filter(|(k, _)| is_value_key(k)) {
        if let (Some(declared), Some(Ok(quantity))) = (attributes.get(key).and_then(dimension), value.as_quantity()) {
            if quantity.dimension() != declared {
                errors.push(format!("{what}: {key} is a {} of '{owner}', not a {}", declared.label(), quantity.dimension().label()));
            }
        }
    }
    let mut names: HashSet<&str> = HashSet::new();
    for snapshot in &occurrence.snapshots {
        if !names.insert(&snapshot.name) {
            errors.push(format!("{what}: two snapshots are named '{}'", snapshot.name));
        }
        check_one(owner, attributes, snapshot, Some((start, end, &occurrence.name)), errors);
    }
}
