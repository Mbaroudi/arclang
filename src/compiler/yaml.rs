//! A JSON value written as block YAML (`arclang export -f yaml`).
//!
//! Mappings and sequences are written in block style; a scalar is written as
//! its JSON, which YAML 1.2 reads as the same value whatever the text holds.

use serde_json::Value;

pub fn to_yaml(value: &Value) -> String {
    let mut out = String::new();
    match value {
        Value::Object(map) if !map.is_empty() => mapping(map, 0, &mut out),
        Value::Array(items) if !items.is_empty() => sequence(items, 0, &mut out),
        scalar => {
            out.push_str(&scalar.to_string());
            out.push('\n');
        }
    }
    out
}

fn mapping(map: &serde_json::Map<String, Value>, depth: usize, out: &mut String) {
    for (key, value) in map {
        out.push_str(&"  ".repeat(depth));
        out.push_str(&Value::String(key.clone()).to_string());
        out.push(':');
        nested(value, depth, out);
    }
}

fn sequence(items: &[Value], depth: usize, out: &mut String) {
    for item in items {
        out.push_str(&"  ".repeat(depth));
        out.push('-');
        nested(item, depth, out);
    }
}

/// What follows a key or a dash: a scalar on the line, a collection below it.
fn nested(value: &Value, depth: usize, out: &mut String) {
    match value {
        Value::Object(map) if !map.is_empty() => {
            out.push('\n');
            mapping(map, depth + 1, out);
        }
        Value::Array(items) if !items.is_empty() => {
            out.push('\n');
            sequence(items, depth + 1, out);
        }
        scalar => {
            out.push(' ');
            out.push_str(&scalar.to_string());
            out.push('\n');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::to_yaml;
    use serde_json::json;

    #[test]
    fn collections_are_blocks_and_scalars_keep_their_text() {
        let value = json!({ "name": "a: b # c", "empty": [], "list": [1, { "deep": [true, null] }, "x\ny"], "n": -1.5 });
        let text = to_yaml(&value);
        assert_eq!(text, "\"empty\": []\n\"list\":\n  - 1\n  -\n    \"deep\":\n      - true\n      - null\n  - \"x\\ny\"\n\"n\": -1.5\n\"name\": \"a: b # c\"\n");
    }
}
