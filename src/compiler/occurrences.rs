//! Timeslices and snapshots (SysML v2 portions of an occurrence): the
//! stretches and moments of an element's life, each with the values the
//! element has then. A timeslice may say when it starts and ends (`start:`,
//! `end:`), a snapshot when it is (`at:`), as times; a snapshot of a
//! timeslice is within it; a value the element declares as a quantity keeps
//! its dimension in a snapshot.

use super::ast::{AttributeValue, Model, Occurrence, OccurrenceKind};
use super::quantity::Dimension;
use std::collections::{HashMap, HashSet};

/// Keys of a timeslice or snapshot that say when, not what the element is then.
pub const TIME_KEYS: [&str; 3] = ["start", "end", "at"];

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

/// What is wrong with the timeslices and snapshots of the model.
pub fn check(model: &Model) -> Vec<String> {
    let mut errors = Vec::new();
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
    for (key, value) in occurrence.attributes.iter().filter(|(k, _)| !TIME_KEYS.contains(&k.as_str()) && !matches!(k.as_str(), "id" | "description")) {
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
