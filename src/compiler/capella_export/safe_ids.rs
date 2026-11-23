//! Ids Capella can hold. A reference in a Capella file is `#id`, and
//! several of them are separated by spaces: an id with a space or a `#` in
//! it cannot be referred to, and the file does not load. Such an id is
//! written as an id made up from it, everywhere the model names it.

use super::context::derived;
use crate::compiler::ast::Model;
use serde_json::Value;
use std::collections::BTreeMap;

/// Keys whose text is for people: never a reference, left as written.
const TEXT_KEYS: &[&str] = &["name", "description", "summary", "review", "label", "title", "text", "rationale"];

fn is_unsafe(id: &str) -> bool {
    id.chars().any(|c| c.is_whitespace() || c == '#')
}

fn id_text(value: &Value) -> Option<&str> {
    value.as_str().or_else(|| value.get("String").and_then(Value::as_str))
}

/// The ids to replace, each with whether the model writes it (`id: "..."`)
/// or it is the name of an element that has no id of its own.
fn collect(value: &Value, found: &mut BTreeMap<String, (String, bool)>) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if key == "id" {
                    if let Some(id) = id_text(value).filter(|id| is_unsafe(id)) {
                        let written = !value.is_string();
                        let entry = found.entry(id.to_string()).or_insert_with(|| (derived(&format!("id:{id}")), written));
                        entry.1 |= written;
                    }
                }
                collect(value, found);
            }
        }
        Value::Array(items) => items.iter().for_each(|item| collect(item, found)),
        _ => {}
    }
}

/// An id itself: always replaced.
fn rewrite_id(value: &mut Value, ids: &BTreeMap<String, (String, bool)>) {
    let text = match value {
        Value::String(text) => Some(text),
        Value::Object(map) => match map.get_mut("String") {
            Some(Value::String(text)) => Some(text),
            _ => None,
        },
        _ => None,
    };
    if let Some(text) = text {
        if let Some((safe, _)) = ids.get(text.as_str()) {
            *text = safe.clone();
        }
    }
}

/// What names an element by an id the model writes follows it. An element
/// without an id of its own is named by its name, which is kept.
fn rewrite(value: &mut Value, ids: &BTreeMap<String, (String, bool)>) {
    match value {
        Value::String(text) => {
            if let Some((safe, true)) = ids.get(text.as_str()) {
                *text = safe.clone();
            } else if let Some((raw, (safe, _))) = ids.iter().find(|(raw, (_, written))| *written && text.strip_prefix(raw.as_str()).is_some_and(|rest| rest.starts_with('.'))) {
                // `Owner.port`: the port of an element named by its id.
                *text = format!("{safe}{}", &text[raw.len()..]);
            }
        }
        Value::Object(map) => {
            for (key, value) in map.iter_mut() {
                if key == "id" {
                    rewrite_id(value, ids);
                } else if !TEXT_KEYS.contains(&key.as_str()) {
                    rewrite(value, ids);
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(|item| rewrite(item, ids)),
        _ => {}
    }
}

/// The model with ids Capella can hold, and a note for each id the model
/// writes that was replaced; `None` when every id can be written as it is.
pub(super) fn model_with_safe_ids(model: &Model) -> Option<(Model, Vec<String>)> {
    let mut json = serde_json::to_value(model).ok()?;
    let mut ids = BTreeMap::new();
    collect(&json, &mut ids);
    if ids.is_empty() {
        return None;
    }
    rewrite(&mut json, &ids);
    let safe: Model = serde_json::from_value(json).ok()?;
    let notes = ids.iter().filter(|(_, (_, written))| *written).map(|(raw, (safe, _))| format!("id '{raw}' is written as {safe}: a Capella id holds no space and no '#'")).collect();
    Some((safe, notes))
}
