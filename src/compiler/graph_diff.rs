//! Diff of two element graphs.
//!
//! The semantic model keeps a handful of fields per element. The element
//! graph holds every element with its effective attributes and every
//! relationship, so comparing two graphs is what makes "No semantic changes."
//! true: a changed latency, a removed test case, a reworded rationale are all
//! in there.

use super::ast::AttributeValue;
use super::elements::{attribute_json, ElementGraph, ElementRecord, RelationshipRecord};
use super::semantic_diff::{
    DiffReport, ElementRef, FieldChange, ModifiedElement, ModifiedRelationship, ModifiedTrace, RelationshipRef,
    TraceRef,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// Data a relationship repeats from the element it comes from: reporting it
/// twice would only add noise.
const MIRRORED_EXTRA: [(&str, &str); 1] = [("Verification", "method")];

/// How a value reads in a report: text as is, a quantity as `40 ms`.
fn render(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(text) => text.clone(),
        // serde_json prints 40.0; the model says 40.
        Value::Number(number) => number.as_f64().map(|n| n.to_string()).unwrap_or_else(|| number.to_string()),
        Value::Object(object) if object.get("@type").and_then(Value::as_str) == Some("Quantity") => {
            format!("{} {}", render(&object["value"]), object["unit"].as_str().unwrap_or(""))
        }
        Value::Array(items) => format!("[{}]", items.iter().map(render).collect::<Vec<_>>().join(", ")),
        other => other.to_string(),
    }
}

/// Attributes and kind-specific data of a record as one sorted field map.
fn fields(
    attributes: &HashMap<String, AttributeValue>,
    extra: &BTreeMap<&'static str, Value>,
) -> BTreeMap<String, String> {
    let declared = attributes.iter().map(|(key, value)| (key.clone(), render(&attribute_json(value))));
    let derived = extra.iter().map(|(key, value)| (key.to_string(), render(value)));
    // A declared attribute wins over derived data of the same name.
    derived.chain(declared).collect()
}

fn field_changes(old: &BTreeMap<String, String>, new: &BTreeMap<String, String>) -> Vec<FieldChange> {
    let keys: BTreeSet<&String> = old.keys().chain(new.keys()).collect();
    keys.into_iter()
        .filter_map(|key| {
            let before = old.get(key).map(String::as_str).unwrap_or("");
            let after = new.get(key).map(String::as_str).unwrap_or("");
            (before != after).then(|| FieldChange {
                field: key.clone(),
                old: before.to_string(),
                new: after.to_string(),
            })
        })
        .collect()
}

/// A graph indexed for comparison. Elements are matched by uuid, which is
/// derived from the id and stays distinct when two elements share an id (a
/// compile warning, but both must still be compared). The model root is set
/// apart so that renaming the model does not make every element look moved.
struct Indexed<'a> {
    elements: BTreeMap<&'a str, &'a ElementRecord>,
    labels: HashMap<&'a str, &'a str>,
    root: Option<&'a ElementRecord>,
}

impl<'a> Indexed<'a> {
    fn new(graph: &'a ElementGraph) -> Self {
        let root = graph.elements.iter().find(|element| element.owner.is_none());
        let elements = graph
            .elements
            .iter()
            .filter(|element| element.owner.is_some())
            .map(|element| (element.uuid.as_str(), element))
            .collect();
        let labels = graph.elements.iter().map(|element| (element.uuid.as_str(), element.id.as_str())).collect();
        Self { elements, labels, root }
    }

    /// The id of the element a uuid names; the model root reads as empty so
    /// that renaming the model moves nothing.
    fn label(&self, uuid: &str) -> &'a str {
        match self.root {
            Some(root) if root.uuid == uuid => "",
            _ => self.labels.get(uuid).copied().unwrap_or(""),
        }
    }

    fn element_fields(&self, element: &ElementRecord) -> BTreeMap<String, String> {
        let mut all = fields(&element.attributes, &element.extra);
        all.remove("id");
        all.insert("name".to_string(), element.name.clone());
        let owner = element.owner.as_deref().map(|uuid| self.label(uuid)).unwrap_or("");
        if !owner.is_empty() {
            all.insert("owner".to_string(), owner.to_string());
        }
        all
    }

    /// Relationships grouped by identity: kind, ends, name and trace kind.
    fn relationships(&self, graph: &'a ElementGraph) -> BTreeMap<RelationshipKey, Vec<&'a RelationshipRecord>> {
        let mut grouped: BTreeMap<RelationshipKey, Vec<&RelationshipRecord>> = BTreeMap::new();
        for relationship in &graph.relationships {
            let key = RelationshipKey {
                kind: relationship.kind,
                source: self.label(&relationship.source).to_string(),
                target: self.label(&relationship.target).to_string(),
                name: relationship.name.clone().unwrap_or_default(),
                trace_kind: relationship.extra.get("traceKind").map(render).unwrap_or_default(),
            };
            grouped.entry(key).or_default().push(relationship);
        }
        grouped
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RelationshipKey {
    kind: &'static str,
    source: String,
    target: String,
    name: String,
    trace_kind: String,
}

impl RelationshipKey {
    fn as_ref(&self) -> RelationshipRef {
        RelationshipRef {
            kind: self.kind.to_string(),
            name: self.name.clone(),
            source: self.source.clone(),
            target: self.target.clone(),
        }
    }
}

fn relationship_fields(relationship: &RelationshipRecord) -> BTreeMap<String, String> {
    let mut all = fields(&relationship.attributes, &relationship.extra);
    all.remove("traceKind");
    for (kind, key) in MIRRORED_EXTRA {
        if relationship.kind == kind {
            all.remove(key);
        }
    }
    all
}

fn element_ref(element: &ElementRecord) -> ElementRef {
    ElementRef {
        id: element.id.clone(),
        name: element.name.clone(),
        element_type: element.kind.to_string(),
        uuid: element.uuid.clone(),
    }
}

fn diff_elements(report: &mut DiffReport, old: &Indexed, new: &Indexed) {
    for (key, element) in &new.elements {
        if !old.elements.contains_key(key) {
            report.added.push(element_ref(element));
        }
    }
    for (key, element) in &old.elements {
        if !new.elements.contains_key(key) {
            report.removed.push(element_ref(element));
        }
    }
    let common = old.elements.iter().filter_map(|(key, before)| Some((*before, *new.elements.get(key)?)));
    for (before, after) in common.chain(old.root.zip(new.root)) {
        let changes = field_changes(&old.element_fields(before), &new.element_fields(after));
        if !changes.is_empty() {
            report.modified.push(ModifiedElement { element: element_ref(after), changes });
        }
    }
}

fn diff_relationships(report: &mut DiffReport, old: &Indexed, new: &Indexed, graphs: (&ElementGraph, &ElementGraph)) {
    let before = old.relationships(graphs.0);
    let after = new.relationships(graphs.1);
    let keys: BTreeSet<&RelationshipKey> = before.keys().chain(after.keys()).collect();

    for key in keys {
        let empty = Vec::new();
        let (old_records, new_records) = (before.get(key).unwrap_or(&empty), after.get(key).unwrap_or(&empty));
        let is_trace = key.kind == "Trace";
        // Relationships of one identity are paired by content, not by the
        // order they are declared in: swapping two of them is no change.
        let sorted = |records: &[&RelationshipRecord]| {
            let mut all: Vec<_> = records.iter().map(|record| relationship_fields(record)).collect();
            all.sort();
            all
        };
        let (old_fields, new_fields) = (sorted(old_records), sorted(new_records));
        for index in 0..old_fields.len().max(new_fields.len()) {
            match (old_fields.get(index), new_fields.get(index)) {
                (Some(old_record), Some(new_record)) => {
                    let changes = field_changes(old_record, new_record);
                    if changes.is_empty() {
                        continue;
                    }
                    if is_trace {
                        report.traces_modified.push(ModifiedTrace { trace: trace_ref(key), changes });
                    } else {
                        report.relationships_modified.push(ModifiedRelationship { relationship: key.as_ref(), changes });
                    }
                }
                // Added and removed traces are reported from the semantic
                // model (see `diff_compiled`).
                (None, Some(_)) if !is_trace => report.relationships_added.push(key.as_ref()),
                (Some(_), None) if !is_trace => report.relationships_removed.push(key.as_ref()),
                _ => {}
            }
        }
    }
}

fn trace_ref(key: &RelationshipKey) -> TraceRef {
    TraceRef {
        from: key.source.clone(),
        trace_type: key.trace_kind.clone(),
        to: key.target.clone(),
    }
}

/// Complete `report` (the trace comparison) with everything the two element
/// graphs show.
pub fn complete(mut report: DiffReport, old: &ElementGraph, new: &ElementGraph) -> DiffReport {
    let (old_index, new_index) = (Indexed::new(old), Indexed::new(new));
    diff_elements(&mut report, &old_index, &new_index);
    diff_relationships(&mut report, &old_index, &new_index, (old, new));
    report.added.sort_by(|a, b| (&a.element_type, &a.id).cmp(&(&b.element_type, &b.id)));
    report.removed.sort_by(|a, b| (&a.element_type, &a.id).cmp(&(&b.element_type, &b.id)));
    report.modified.sort_by(|a, b| (&a.element.id, &a.element.uuid).cmp(&(&b.element.id, &b.element.uuid)));
    report
}
