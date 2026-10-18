//! What any Capella element carries besides its structure: its name,
//! description, summary, review, progress status, and the attributes
//! Capella has no feature for, as property values of the element.

use super::context::derived;
use super::xml::El;
use crate::compiler::ast::AttributeValue;
use std::collections::HashMap;

/// Capella's progress status literals, in its order.
const STATUS: &[&str] = &["DRAFT", "TO_BE_REVIEWED", "TO_BE_DISCUSSED", "REWORK_NECESSARY", "UNDER_REWORK", "REVIEWED_OK"];

/// The id of a progress status literal.
fn status_id(literal: &str) -> String {
    derived(&format!("status:{literal}"))
}

/// The `ProgressStatus` enumeration of the project: Capella's literals,
/// then those the model uses besides.
pub(super) fn progress_status(project_id: &str, used: &[String]) -> El {
    let mut status = El::new("ownedEnumerationPropertyTypes", "capellacore:EnumerationPropertyType", &derived(&format!("{project_id}:ProgressStatus"))).attr("name", "ProgressStatus");
    let extra = used.iter().map(String::as_str).filter(|u| !STATUS.contains(u));
    for literal in STATUS.iter().copied().chain(extra) {
        status.push(El::new("ownedLiterals", "capellacore:EnumerationPropertyLiteral", &status_id(literal)).attr("name", literal));
    }
    status
}

/// Keys every element writes through its header, never as property values.
const HEADER_KEYS: &[&str] = &["id", "name", "description", "summary", "review", "status", "capella_status", "constraints", "available_in_states", "owner"];

/// The header of `el`: description, summary, review and status from the
/// attributes, then every attribute that is neither a header key nor one of
/// `structural` (what the caller writes as Capella structure) as a property
/// value of the element.
pub(super) fn apply(el: El, attributes: &HashMap<String, AttributeValue>, structural: &[&str]) -> El {
    // Viewpoint attributes are extensions of the element, not property values.
    let (el, extensions) = super::cyber::extend(el, attributes);
    let text = |key: &str| attributes.get(key).and_then(AttributeValue::as_string);
    let mut el = el.opt("description", text("description")).opt("summary", text("summary")).opt("review", text("review"));
    if let Some(status) = text("capella_status").or_else(|| text("status")) {
        el.set("status", &format!("#{}", status_id(status)));
    }
    let owner = el.id().to_string();
    // Constraints are Capella constraints of the element, their text an opaque expression.
    let constraints: Vec<String> = match attributes.get("constraints") {
        Some(AttributeValue::List(items)) => items.iter().filter_map(|i| i.as_string().map(str::to_string)).collect(),
        Some(AttributeValue::String(one)) => vec![one.clone()],
        _ => Vec::new(),
    };
    for (index, body) in constraints.iter().enumerate() {
        let id = derived(&format!("{owner}:constraint:{index}"));
        el.push(
            El::new("ownedConstraints", "capellacore:Constraint", &id)
                .with(El::new("ownedSpecification", "datavalue:OpaqueExpression", &derived(&format!("{id}:expression"))).with(El::text("bodies", body)).with(El::text("languages", "capella:linkedText"))),
        );
    }
    let mut keys: Vec<&String> = attributes.keys().filter(|k| !HEADER_KEYS.contains(&k.as_str()) && !structural.contains(&k.as_str()) && !extensions.contains(*k)).collect();
    keys.sort();
    for key in keys {
        if let Some(value) = property_value(&owner, key, &attributes[key]) {
            el.push(value);
        }
    }
    el
}

/// An attribute as a property value. A `pv_` key is a property value whose
/// name clashed with an ArcLang attribute on import: it goes back to its name.
fn property_value(owner: &str, key: &str, value: &AttributeValue) -> Option<El> {
    let name = key.strip_prefix("pv_").unwrap_or(key);
    let id = derived(&format!("{owner}:pv:{key}"));
    let el = |kind: &str, text: String| Some(El::new("ownedPropertyValues", kind, &id).attr("name", name).attr("value", &text));
    match value {
        // The parser keeps `true` and `false` as text.
        AttributeValue::String(text) if text == "true" || text == "false" => el("capellacore:BooleanPropertyValue", text.clone()),
        AttributeValue::String(text) => el("capellacore:StringPropertyValue", text.clone()),
        AttributeValue::Boolean(flag) => el("capellacore:BooleanPropertyValue", flag.to_string()),
        AttributeValue::Number(n) if n.fract() == 0.0 && n.abs() < 1e15 => el("capellacore:IntegerPropertyValue", format!("{}", *n as i64)),
        AttributeValue::Number(n) => el("capellacore:FloatPropertyValue", n.to_string()),
        other => el("capellacore:StringPropertyValue", other.display()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rendered(el: &El) -> String {
        let mut out = String::new();
        el.render(&mut out, 0);
        out
    }

    #[test]
    fn the_header_writes_description_status_and_the_other_attributes_as_property_values() {
        let attributes: HashMap<String, AttributeValue> = [
            ("id".to_string(), AttributeValue::String("f1".into())),
            ("description".to_string(), AttributeValue::String("Pumps".into())),
            ("status".to_string(), AttributeValue::String("DRAFT".into())),
            ("mass".to_string(), AttributeValue::Number(12.0)),
            ("pv_kind".to_string(), AttributeValue::String("hydraulic".into())),
            ("parent".to_string(), AttributeValue::String("x".into())),
        ]
        .into_iter()
        .collect();
        let out = rendered(&apply(El::new("ownedFunctions", "ctx:SystemFunction", "f1"), &attributes, &["parent"]));
        assert!(out.contains("description=\"Pumps\""), "{out}");
        assert!(out.contains(&format!("status=\"#{}\"", status_id("DRAFT"))), "{out}");
        assert!(out.contains("IntegerPropertyValue\" id=") && out.contains("name=\"mass\" value=\"12\""), "{out}");
        assert!(out.contains("name=\"kind\" value=\"hydraulic\""), "{out}");
        assert!(!out.contains("name=\"parent\""), "{out}");
    }
}
