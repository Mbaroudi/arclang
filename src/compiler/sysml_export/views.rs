//! Tags and declared diagrams: each tag a metadata definition of the
//! `Tags` package (`@Safety;` in the elements carrying it), each declared
//! diagram a `view` exposing what it shows (`expose X;`, `expose X::**;`
//! for `under`) and filtering by its tags (`filter @Safety;`).

use super::{text, Exporter};
use crate::compiler::ast::AttributeValue;
use std::collections::HashMap;
use crate::compiler::diagram::tags;

/// Every tag the model's elements carry, sorted: whatever the element, as
/// the header writes `@Tag;` for any of them.
fn all_tags(e: &Exporter<'_>) -> Vec<String> {
    fn walk(value: &serde_json::Value, found: &mut std::collections::BTreeSet<String>) {
        match value {
            serde_json::Value::Object(object) => {
                if let Some(attributes) = object.get("attributes") {
                    if let Ok(attributes) = serde_json::from_value::<HashMap<String, AttributeValue>>(attributes.clone()) {
                        found.extend(tags::tags_of(&attributes));
                        found.extend(tags::texts(&attributes, "tagged"));
                    }
                }
                object.values().for_each(|v| walk(v, found));
            }
            serde_json::Value::Array(items) => items.iter().for_each(|v| walk(v, found)),
            _ => {}
        }
    }
    let mut found = std::collections::BTreeSet::new();
    if let Ok(model) = serde_json::to_value(e.model) {
        walk(&model, &mut found);
    }
    found.into_iter().collect()
}

/// The `Tags` package, when the model tags anything.
pub(super) fn tags(e: &mut Exporter<'_>) {
    let tags = all_tags(e);
    if tags.is_empty() {
        return;
    }
    e.block("package Tags", |e| {
        for tag in &tags {
            e.line(&format!("metadata def {};", text::name(tag)));
        }
    });
    e.line("private import Tags::*;");
}

pub(super) fn write(e: &mut Exporter<'_>) {
    let model = e.model;
    for diagram in &model.diagrams {
        // A view whose scope is not exported would show more than the diagram: said, not written.
        if let Some(under) = diagram.attributes.get("under").and_then(|v| v.as_string()).filter(|u| e.reference(u).is_none()) {
            e.line(&format!("// view {} left out: under '{under}' is no exported element", text::name(&diagram.name)));
            continue;
        }
        let missing: Vec<&String> = diagram.shows.iter().filter(|s| e.reference(s).is_none()).collect();
        if !missing.is_empty() && missing.len() == diagram.shows.len() {
            e.line(&format!("// view {} left out: it shows no exported element", text::name(&diagram.name)));
            continue;
        }
        for shown in &missing {
            e.line(&format!("// view {}: '{shown}' is no exported element", text::name(&diagram.name)));
        }
        e.block(&format!("view {}", text::name(&diagram.name)), |e| {
            e.line(&format!("attribute 'view' : String = {};", text::string(&diagram.view)));
            if let Some(configuration) = diagram.attributes.get("configuration").and_then(|v| v.as_string()) {
                e.line(&format!("attribute configuration : String = {};", text::string(configuration)));
            }
            if diagram.context {
                e.line("attribute context : Boolean = true;");
            }
            if let Some(under) = diagram.attributes.get("under").and_then(|v| v.as_string()).and_then(|u| e.reference(u)) {
                e.line(&format!("expose {under}::**;"));
            }
            for shown in &diagram.shows {
                if let Some(reference) = e.reference(shown) {
                    e.line(&format!("expose {reference};"));
                }
            }
            let tagged: Vec<String> = tags::texts(&diagram.attributes, "tagged").iter().map(|t| format!("@{}", text::name(t))).collect();
            if !tagged.is_empty() {
                e.line(&format!("filter {};", tagged.join(" or ")));
            }
        });
    }
}
