//! The layout file as entries, one per element it places: the unit its diff
//! and its merge work on. Layout never changes what a model means, so a
//! change here is reported apart from the model's.

use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

/// The path of one entry: `["arr", "lab\u{1}", "places", "LC-1"]`.
pub type EntryPath = Vec<String>;

const SEP: &str = "\u{1}";

/// Every entry of a layout file with its value.
pub fn entries(layout: Option<&Value>) -> BTreeMap<EntryPath, Value> {
    let mut out = BTreeMap::new();
    let Some(Value::Object(root)) = layout else { return out };
    let path = |parts: &[&str]| parts.iter().map(|s| s.to_string()).collect::<EntryPath>();
    let set = |out: &mut BTreeMap<EntryPath, Value>, prefix: &[&str], list: Option<&Value>| {
        for item in list.and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str) {
            let mut p = path(prefix);
            p.push(item.to_string());
            out.insert(p, json!(true));
        }
    };
    if let Some(Value::Object(views)) = root.get("views") {
        for (id, view) in views {
            out.insert(path(&["views", id, "present"]), json!(true));
            out.insert(path(&["views", id, "open"]), view.get("open").cloned().unwrap_or(Value::Null));
            set(&mut out, &["views", id, "folded"], view.get("folded"));
        }
    }
    for arrangement in root.get("arrangements").and_then(Value::as_array).into_iter().flatten() {
        let view = arrangement.get("view").and_then(Value::as_str).unwrap_or("");
        let open = arrangement.get("open").and_then(Value::as_str).unwrap_or("");
        let key = format!("{view}{SEP}{open}");
        out.insert(path(&["arr", &key, "present"]), json!(true));
        set(&mut out, &["arr", &key, "folded"], arrangement.get("folded"));
        for map in ["places", "edges", "ports", "order"] {
            if let Some(Value::Object(items)) = arrangement.get(map) {
                for (id, value) in items {
                    out.insert(path(&["arr", &key, map, id]), value.clone());
                }
            }
        }
    }
    if let Some(Value::Object(layers)) = root.get("layers") {
        if let Some(order) = layers.get("order") {
            out.insert(path(&["layers", "order"]), order.clone());
        }
        set(&mut out, &["layers", "hidden"], layers.get("hidden"));
        set(&mut out, &["layers", "locked"], layers.get("locked"));
    }
    if let Some(Value::Object(groups)) = root.get("groups") {
        for (view, list) in groups {
            for group in list.as_array().into_iter().flatten() {
                if let Some(id) = group.get("id").and_then(Value::as_str) {
                    out.insert(path(&["groups", view, id]), group.clone());
                }
            }
        }
    }
    set(&mut out, &["locked"], root.get("locked"));
    out
}

/// A layout file from entries (the inverse of [`entries`]).
pub fn build(entries: &BTreeMap<EntryPath, Value>) -> Value {
    let mut views: Map<String, Value> = Map::new();
    let mut arrangements: BTreeMap<String, Map<String, Value>> = BTreeMap::new();
    let mut layers: Map<String, Value> = Map::new();
    let mut groups: Map<String, Value> = Map::new();
    let mut locked: Vec<Value> = Vec::new();
    let push = |object: &mut Map<String, Value>, key: &str, item: &str| {
        let list = object.entry(key.to_string()).or_insert_with(|| json!([]));
        if let Value::Array(items) = list {
            items.push(json!(item));
        }
    };
    for (path, value) in entries {
        let p: Vec<&str> = path.iter().map(String::as_str).collect();
        match p.as_slice() {
            ["views", id, "present"] => {
                views.entry(id.to_string()).or_insert_with(|| json!({"folded": [], "open": null}));
            }
            ["views", id, "open"] => {
                if let Value::Object(view) = views.entry(id.to_string()).or_insert_with(|| json!({"folded": [], "open": null})) {
                    view.insert("open".into(), value.clone());
                }
            }
            ["views", id, "folded", item] => {
                if let Value::Object(view) = views.entry(id.to_string()).or_insert_with(|| json!({"folded": [], "open": null})) {
                    push(view, "folded", item);
                }
            }
            ["arr", key, rest @ ..] => {
                let (view, open) = key.split_once(SEP).unwrap_or((key, ""));
                let arrangement = arrangements.entry(key.to_string()).or_insert_with(|| {
                    let mut m = Map::new();
                    m.insert("view".into(), json!(view));
                    m.insert("open".into(), if open.is_empty() { Value::Null } else { json!(open) });
                    m.insert("folded".into(), json!([]));
                    m.insert("places".into(), json!({}));
                    m
                });
                match rest {
                    ["present"] => {}
                    ["folded", item] => push(arrangement, "folded", item),
                    [map, id] => {
                        if let Value::Object(items) = arrangement.entry(map.to_string()).or_insert_with(|| json!({})) {
                            items.insert(id.to_string(), value.clone());
                        }
                    }
                    _ => {}
                }
            }
            ["layers", "order"] => {
                layers.insert("order".into(), value.clone());
            }
            ["layers", which, item] => push(&mut layers, which, item),
            ["groups", view, _id] => {
                if let Value::Array(list) = groups.entry(view.to_string()).or_insert_with(|| json!([])) {
                    list.push(value.clone());
                }
            }
            ["locked", item] => locked.push(json!(item)),
            _ => {}
        }
    }
    let mut root = Map::new();
    root.insert("arclang_layout".into(), json!("1"));
    root.insert("views".into(), Value::Object(views));
    root.insert("arrangements".into(), Value::Array(arrangements.into_values().map(Value::Object).collect()));
    if !layers.is_empty() {
        root.insert("layers".into(), Value::Object(layers));
    }
    if !groups.is_empty() {
        root.insert("groups".into(), Value::Object(groups));
    }
    if !locked.is_empty() {
        root.insert("locked".into(), Value::Array(locked));
    }
    Value::Object(root)
}

/// One entry of the layout that differs between two versions.
#[derive(Debug, Clone, Serialize)]
pub struct LayoutChange {
    /// `added`, `removed` or `changed`.
    pub status: &'static str,
    /// What the entry places: `places`, `edges`, `ports`, `order`, `folded`, `view`, `layers`, `groups`, `locked`.
    pub what: String,
    /// The view the entry belongs to, when it has one.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub view: String,
    /// The element (or edge, port, group) the entry is about.
    pub element: String,
}

fn describe(path: &EntryPath) -> Option<(String, String, String)> {
    let p: Vec<&str> = path.iter().map(String::as_str).collect();
    Some(match p.as_slice() {
        ["views", id, "present"] => ("view".into(), id.to_string(), String::new()),
        ["views", id, "open"] => ("view".into(), id.to_string(), "open container".into()),
        ["views", id, "folded", item] => ("folded".into(), id.to_string(), item.to_string()),
        ["arr", _, "present"] => return None,
        ["arr", key, "folded", item] => ("folded".into(), key.replace(SEP, "/").trim_end_matches('/').to_string(), item.to_string()),
        ["arr", key, map, id] => (map.to_string(), key.replace(SEP, "/").trim_end_matches('/').to_string(), id.to_string()),
        ["layers", which, ..] => ("layers".into(), String::new(), which.to_string()),
        ["groups", view, id] => ("groups".into(), view.to_string(), id.to_string()),
        ["locked", item] => ("locked".into(), String::new(), item.to_string()),
        _ => return None,
    })
}

/// What differs between two layout files, entry by entry.
pub fn diff(old: Option<&Value>, new: Option<&Value>) -> Vec<LayoutChange> {
    let (before, after) = (entries(old), entries(new));
    let mut out = Vec::new();
    for (path, value) in &after {
        let status = match before.get(path) {
            None => "added",
            Some(was) if was != value => "changed",
            _ => continue,
        };
        if let Some((what, view, element)) = describe(path) {
            out.push(LayoutChange { status, what, view, element });
        }
    }
    for path in before.keys().filter(|p| !after.contains_key(*p)) {
        if let Some((what, view, element)) = describe(path) {
            out.push(LayoutChange { status: "removed", what, view, element });
        }
    }
    out
}
