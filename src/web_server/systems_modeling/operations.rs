//! What a commit request asks for, change by change: read from the JSON,
//! applied to the model text, and checked on the compiled result.
//!
//! Supported: renaming an element that keeps its identity; setting and
//! removing attributes of an element; creating an
//! element of a kind listed in [`CREATABLE`]; deleting an element (with what
//! it contains); creating and deleting a trace. Everything else is refused
//! with the reason.

use super::*;
use crate::compiler::elements::ElementGraph;
use crate::compiler::source_edit::{self, Container, EditError};
use std::collections::HashSet;

fn refused(description: String) -> Response {
    error(StatusCode::UNPROCESSABLE_ENTITY, description)
}

/// Kinds that can be created: the layer block that holds them when the
/// model owns them, their keyword, and the kinds they may be declared in.
const CREATABLE: &[(&str, Option<&str>, &str, &[&str])] = &[
    ("Actor", Some("operational_analysis"), "actor", &[]),
    ("Requirement", Some("system_analysis"), "requirement", &[]),
    ("SystemFunction", Some("system_analysis"), "function", &[]),
    ("LogicalComponent", Some("logical_architecture"), "component", &["LogicalComponent"]),
    ("LogicalFunction", None, "function", &["LogicalComponent"]),
    ("PhysicalNode", Some("physical_architecture"), "node", &[]),
];

/// Where a created element is declared.
enum Place {
    Layer { keyword: &'static str, name: Option<String> },
    /// Inside an element, by each way the text may know it.
    Inside(Vec<String>),
}

pub(super) enum Operation {
    /// Set (`Some`, as ArcLang source) or remove (`None`) one attribute.
    Attribute { designators: Vec<String>, key: String, value: Option<String> },
    Create { kind: String, keyword: &'static str, place: Place, name: String, owner: String, attributes: Vec<(String, String)> },
    Rename { identity: String, designators: Vec<String>, name: String },
    Delete { identity: String, designators: Vec<String> },
    CreateTrace { kind: String, from: String, to: String, source: String, target: String },
    DeleteTrace { identity: String, kind: String, from: Vec<String>, to: Vec<String> },
}

/// A JSON value as ArcLang source: the inverse of `elements::attribute_json`.
/// The result is not trusted as is; `source_edit` parses it as one value.
fn arclang_value(value: &Value) -> Result<String, String> {
    match value {
        Value::String(text) => Ok(source_edit::string_literal(text)),
        Value::Number(number) => Ok(number.to_string()),
        Value::Bool(flag) => Ok(flag.to_string()),
        Value::Null => Err("null is not a value".to_string()),
        Value::Array(items) => {
            let items: Result<Vec<String>, String> = items.iter().map(arclang_value).collect();
            Ok(format!("[{}]", items?.join(", ")))
        }
        Value::Object(object) => match (object.get("value"), object.get("unit")) {
            (Some(Value::Number(number)), Some(Value::String(unit))) => Ok(format!("{} {}", number, unit)),
            (_, Some(_)) => Err("a quantity is {\"value\": <number>, \"unit\": \"<symbol>\"}".to_string()),
            _ => {
                let mut entries = Vec::new();
                for (key, item) in object.iter().filter(|(key, _)| !key.starts_with('@')) {
                    entries.push(format!("{}: {}", key, arclang_value(item)?));
                }
                Ok(format!("{{ {} }}", entries.join(" ")))
            }
        },
    }
}

fn is_relationship(record: &Value) -> bool {
    record.get("relatedElement").is_some()
}

/// How the text may know an element: its identifier, then its name.
fn designators(record: &Value) -> Vec<String> {
    ["shortName", "name"].iter().filter_map(|key| record[*key].as_str().map(str::to_string)).collect()
}

fn end<'v>(head: &'v View, payload: &Value, role: &str) -> Result<&'v Value, Response> {
    let identity = match &payload[role] {
        Value::Array(items) => items.first().and_then(|item| item["@id"].as_str()),
        other => other["@id"].as_str(),
    };
    identity
        .and_then(|identity| head.record(identity))
        .filter(|record| !is_relationship(record))
        .ok_or_else(|| bad_request(format!("a trace needs a '{}' that is an element of the head commit", role)))
}

/// A change whose identity is not in the head commit: a creation.
fn creation(payload: &Map<String, Value>, head: &View) -> Result<Operation, Response> {
    let payload_value = Value::Object(payload.clone());
    let Some(kind) = payload.get("@type").and_then(Value::as_str) else {
        return Err(bad_request("creating an element needs its '@type' (an ArcLang kind) and its 'name'".to_string()));
    };
    if kind == "Trace" {
        let trace_kind = payload.get("traceKind").and_then(Value::as_str).ok_or_else(|| {
            bad_request("a trace needs a 'traceKind' (satisfies, implements, refines...)".to_string())
        })?;
        let (source, target) = (end(head, &payload_value, "source")?, end(head, &payload_value, "target")?);
        return Ok(Operation::CreateTrace {
            kind: trace_kind.to_string(),
            from: designators(source).into_iter().next().unwrap_or_default(),
            to: designators(target).into_iter().next().unwrap_or_default(),
            source: source["@id"].as_str().unwrap_or_default().to_string(),
            target: target["@id"].as_str().unwrap_or_default().to_string(),
        });
    }
    let Some((_, layer, keyword, parents)) = CREATABLE.iter().find(|(creatable, ..)| *creatable == kind) else {
        let kinds: Vec<&str> = CREATABLE.iter().map(|(creatable, ..)| *creatable).collect();
        return Err(refused(format!(
            "creating a {} is not supported through the API (supported: {}, Trace)",
            kind,
            kinds.join(", ")
        )));
    };
    let Some(name) = payload.get("name").and_then(Value::as_str).filter(|name| !name.trim().is_empty()) else {
        return Err(bad_request(format!("creating a {} needs a 'name'", kind)));
    };
    for key in payload.keys() {
        if !matches!(key.as_str(), "@type" | "@id" | "elementId" | "name" | "declaredName" | "owner" | "attributes" | "arclang:container") {
            return Err(refused(format!("'{}' cannot be given when creating a {}", key, kind)));
        }
    }

    let root = head.roots.first().map(|&index| &head.records[index]);
    let owner = match payload.get("owner").and_then(|owner| owner["@id"].as_str()) {
        Some(identity) => head.record(identity).ok_or_else(|| bad_request(format!("owner '{}' is not in the head commit", identity)))?,
        None => root.ok_or_else(|| refused("the model has no root element".to_string()))?,
    };
    let owner_kind = owner["@type"].as_str().unwrap_or_default();
    let owned_by_model = Some(owner) == root;
    let place = match (owned_by_model, layer) {
        (true, Some(keyword)) => Place::Layer {
            keyword,
            name: payload.get("arclang:container").and_then(Value::as_str).map(str::to_string),
        },
        (false, _) if parents.contains(&owner_kind) => Place::Inside(designators(owner)),
        _ => {
            return Err(refused(format!("a {} cannot be created inside a {}", kind, owner_kind)));
        }
    };

    let mut attributes = Vec::new();
    match payload.get("attributes") {
        None => {}
        Some(Value::Object(given)) => {
            for (key, value) in given {
                let source = arclang_value(value).map_err(|reason| bad_request(format!("attribute '{}': {}", key, reason)))?;
                attributes.push((key.clone(), source));
            }
        }
        Some(_) => return Err(bad_request("'attributes' must be an object".to_string())),
    }
    Ok(Operation::Create {
        kind: kind.to_string(),
        keyword,
        place,
        name: name.to_string(),
        owner: owner["@id"].as_str().unwrap_or_default().to_string(),
        attributes,
    })
}

fn deletion(identity: &str, record: &Value, head: &View) -> Result<Operation, Response> {
    if !is_relationship(record) {
        if record["owner"].is_null() {
            return Err(refused("the model itself cannot be deleted".to_string()));
        }
        return Ok(Operation::Delete { identity: identity.to_string(), designators: designators(record) });
    }
    if record["@type"] != "Trace" {
        return Err(refused(format!(
            "deleting a {} is not supported through the API; only traces among relationships",
            record["@type"].as_str().unwrap_or("relationship")
        )));
    }
    let ends = |role: &str| -> Vec<String> {
        record[role][0]["@id"].as_str().and_then(|id| head.record(id)).map(designators).unwrap_or_default()
    };
    Ok(Operation::DeleteTrace {
        identity: identity.to_string(),
        kind: record["traceKind"].as_str().unwrap_or_default().to_string(),
        from: ends("source"),
        to: ends("target"),
    })
}

/// The operations one `DataVersion` of the request asks for.
pub(super) fn operations(change: &Value, head: &View) -> Result<Vec<Operation>, Response> {
    let identity = change["identity"]["@id"].as_str();
    let existing = identity.and_then(|identity| head.record(identity).map(|record| (identity, record)));
    let payload = match (&change["payload"], existing) {
        (Value::Object(payload), _) => payload,
        (Value::Null, Some((identity, record))) => return Ok(vec![deletion(identity, record, head)?]),
        (Value::Null, None) => return Err(bad_request("a deletion needs the identity of an element of the head commit".to_string())),
        _ => return Err(bad_request("the payload of a change must be an object, or null to delete".to_string())),
    };
    let Some((identity, record)) = existing else {
        return Ok(vec![creation(payload, head)?]);
    };
    if is_relationship(record) {
        return Err(refused(format!("'{}' is a relationship; a relationship can be created or deleted, not changed", identity)));
    }
    let mut operations = Vec::new();
    let mut new_name: Option<&str> = None;
    for (key, value) in payload {
        let differs = record.get(key) != Some(value);
        match (key.as_str(), value.as_str(), differs) {
            ("attributes", _, _) | (_, _, false) => {}
            ("name" | "declaredName", Some(name), true) if new_name.is_none() || new_name == Some(name) => new_name = Some(name),
            _ => {
                return Err(refused(format!(
                    "only the name and the attributes of an existing element can be changed: '{}' of element '{}' differs from the head commit",
                    key, identity
                )));
            }
        }
    }
    if let Some(name) = new_name {
        operations.push(Operation::Rename { identity: identity.to_string(), designators: designators(record), name: name.to_string() });
    }
    let attributes = match payload.get("attributes") {
        None => return Ok(operations),
        Some(Value::Object(attributes)) => attributes,
        Some(_) => return Err(bad_request(format!("'attributes' of '{}' must be an object", identity))),
    };
    for (key, value) in attributes {
        let current = record["attributes"].get(key);
        let value = match value {
            Value::Null if current.is_none() => continue,
            Value::Null => None,
            same if Some(same) == current => continue,
            changed => Some(arclang_value(changed).map_err(|reason| {
                bad_request(format!("attribute '{}' of '{}': {}", key, identity, reason))
            })?),
        };
        operations.push(Operation::Attribute { designators: designators(record), key: key.clone(), value });
    }
    Ok(operations)
}

/// Try an edit with each way the text may know an element.
fn with_each(designators: &[String], edit: impl Fn(&str) -> Result<String, EditError>) -> Result<String, EditError> {
    let mut outcome = Err(EditError::NotFound(designators.join(" / ")));
    for designator in designators {
        outcome = edit(designator);
        if !matches!(outcome, Err(EditError::NotFound(_))) {
            break;
        }
    }
    outcome
}

impl Operation {
    /// How the operation reads in a commit message and in an error.
    pub(super) fn label(&self) -> String {
        let first = |designators: &[String]| designators.first().cloned().unwrap_or_default();
        match self {
            Operation::Attribute { designators, key, value: Some(_) } => format!("set {}.{}", first(designators), key),
            Operation::Attribute { designators, key, value: None } => format!("unset {}.{}", first(designators), key),
            Operation::Create { kind, name, .. } => format!("add {} '{}'", kind, name),
            Operation::Rename { designators, name, .. } => format!("rename {} to '{}'", first(designators), name),
            Operation::Delete { designators, .. } => format!("remove {}", first(designators)),
            Operation::CreateTrace { kind, from, to, .. } => format!("add trace {} {} {}", from, kind, to),
            Operation::DeleteTrace { kind, from, to, .. } => format!("remove trace {} {} {}", first(from), kind, first(to)),
        }
    }

    /// The model text with this operation applied.
    pub(super) fn apply(&self, source: &str) -> Result<String, EditError> {
        match self {
            Operation::Attribute { designators, key, value: Some(value) } => {
                with_each(designators, |designator| source_edit::set_attribute(source, designator, key, value))
            }
            Operation::Attribute { designators, key, value: None } => {
                with_each(designators, |designator| source_edit::remove_attribute(source, designator, key))
            }
            Operation::Create { keyword, place: Place::Layer { keyword: layer, name: layer_name }, name, attributes, .. } => {
                let container = Container::Layer { keyword: layer, name: layer_name.as_deref() };
                source_edit::add_element(source, container, keyword, name, attributes)
            }
            Operation::Create { keyword, place: Place::Inside(owners), name, attributes, .. } => with_each(owners, |owner| {
                source_edit::add_element(source, Container::Element(owner), keyword, name, attributes)
            }),
            Operation::Rename { designators, name, .. } => {
                with_each(designators, |designator| source_edit::rename_element(source, designator, name))
            }
            Operation::Delete { designators, .. } => {
                with_each(designators, |designator| source_edit::remove_element(source, designator))
            }
            Operation::CreateTrace { kind, from, to, .. } => source_edit::add_trace(source, from, kind, to),
            Operation::DeleteTrace { kind, from, to, .. } => {
                let from: Vec<&str> = from.iter().map(String::as_str).collect();
                let to: Vec<&str> = to.iter().map(String::as_str).collect();
                source_edit::remove_trace(source, &from, kind, &to)
            }
        }
    }
}

fn label_of(record: &Value) -> String {
    format!(
        "{} '{}'",
        record["@type"].as_str().unwrap_or("element"),
        record["shortName"].as_str().or(record["name"].as_str()).unwrap_or("?")
    )
}

/// Check on the compiled result that the request did what it asked and
/// nothing else: every value shows, exactly the requested elements and
/// traces appeared or disappeared, and no relationship was left dangling.
pub(super) fn check_outcome(operations: &[Operation], head: &Snapshot, after: &Snapshot, graph: &ElementGraph) -> Result<(), Response> {
    let (before, now) = (&head.view, &after.view);
    let appeared = |trace: bool| -> Vec<&Value> {
        now.records
            .iter()
            .filter(|record| record["@id"].as_str().is_some_and(|id| before.record(id).is_none()))
            .filter(|record| if trace { record["@type"] == "Trace" } else { !is_relationship(record) })
            .collect()
    };
    let vanished = |trace: bool| -> Vec<&Value> {
        before
            .records
            .iter()
            .filter(|record| record["@id"].as_str().is_some_and(|id| now.record(id).is_none()))
            .filter(|record| if trace { record["@type"] == "Trace" } else { !is_relationship(record) })
            .collect()
    };

    let mut new_elements = appeared(false);
    let mut new_traces = appeared(true);
    // What may disappear: the deleted elements and what they contain.
    let mut removable: HashSet<&str> = HashSet::new();
    let mut removable_traces: HashSet<&str> = HashSet::new();
    for operation in operations {
        match operation {
            Operation::Attribute { designators, key, value: Some(value) } => {
                let designator = designators.first().map(String::as_str).unwrap_or_default();
                source_edit::confirm_effect(graph, designator, key, value).map_err(|reason| refused(reason.to_string()))?;
            }
            Operation::Attribute { value: None, .. } => {}
            Operation::Create { kind, name, owner, attributes, .. } => {
                let found = new_elements.iter().position(|record| {
                    record["@type"] == kind.as_str() && record["name"] == name.as_str() && record["owner"]["@id"] == owner.as_str()
                });
                let Some(found) = found else {
                    return Err(refused(format!(
                        "{}: once compiled, the model does not hold that {} under the requested owner",
                        operation.label(),
                        kind
                    )));
                };
                let created = new_elements.swap_remove(found);
                let designator = created["shortName"].as_str().unwrap_or(name);
                for (key, value) in attributes {
                    source_edit::confirm_effect(graph, designator, key, value).map_err(|reason| refused(reason.to_string()))?;
                }
            }
            Operation::Rename { identity, name, .. } => {
                // Same identity, new name. An element known by its name
                // (no `id`) would come back as another element.
                if now.record(identity).map(|record| &record["name"]) != Some(&json!(name)) {
                    return Err(refused(format!(
                        "{}: the element would not keep its identity; give it an `id` and refer to it by that id first",
                        operation.label()
                    )));
                }
            }
            Operation::Delete { identity, .. } => {
                if now.record(identity).is_some() {
                    return Err(refused(format!("{}: the element is still in the compiled model", operation.label())));
                }
                removable.insert(identity);
            }
            Operation::CreateTrace { kind, source, target, .. } => {
                let found = new_traces.iter().position(|record| {
                    record["traceKind"] == kind.as_str()
                        && record["source"][0]["@id"] == source.as_str()
                        && record["target"][0]["@id"] == target.as_str()
                });
                match found {
                    Some(found) => new_traces.swap_remove(found),
                    None => return Err(refused(format!("{}: once compiled, the model does not hold that trace", operation.label()))),
                };
            }
            Operation::DeleteTrace { identity, .. } => {
                if now.record(identity).is_some() {
                    return Err(refused(format!("{}: the trace is still in the compiled model", operation.label())));
                }
                removable_traces.insert(identity);
            }
        }
    }

    if let Some(extra) = new_elements.first().or(new_traces.first()) {
        return Err(refused(format!("the request would also create {}", label_of(extra))));
    }
    let inside_removed = |record: &Value| {
        let mut cursor = Some(record);
        while let Some(current) = cursor {
            if current["@id"].as_str().is_some_and(|id| removable.contains(id)) {
                return true;
            }
            cursor = current["owner"]["@id"].as_str().and_then(|owner| before.record(owner));
        }
        false
    };
    if let Some(lost) = vanished(false).into_iter().find(|record| !inside_removed(record)) {
        return Err(refused(format!("the request would also delete {}", label_of(lost))));
    }
    let lost_trace = vanished(true).into_iter().find(|record| !record["@id"].as_str().is_some_and(|id| removable_traces.contains(id)));
    if lost_trace.is_some() {
        return Err(refused("the request would also delete a trace it does not name".to_string()));
    }
    // A relationship whose end no longer exists is still written in the
    // text: the model would compile with a link silently gone.
    let dangling: Vec<&String> = after.unresolved.iter().filter(|entry| !head.unresolved.contains(entry)).collect();
    if !dangling.is_empty() {
        return Err(refused(format!(
            "the request would leave {} relationship(s) without an end: {}",
            dangling.len(),
            dangling.iter().map(|entry| entry.as_str()).collect::<Vec<_>>().join("; ")
        )));
    }
    Ok(())
}
