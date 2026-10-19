//! Tags and declared diagrams: each tag a metadata definition of the
//! `Tags` package (`@Safety;` in the elements carrying it), each declared
//! diagram a `view` exposing what it shows (`expose X;`, `expose X::**;`
//! for `under`) and filtering by its tags (`filter @Safety;`).

use super::{text, Exporter};
use crate::compiler::ast::AttributeValue;
use crate::compiler::diagram::tags;

/// Every tag the model's elements carry, sorted.
fn all_tags(e: &Exporter<'_>) -> Vec<String> {
    let mut found = std::collections::BTreeSet::new();
    let model = e.model;
    let mut every = |attributes: &std::collections::HashMap<String, AttributeValue>| found.extend(tags::tags_of(attributes));
    fn components(list: &[crate::compiler::ast::LogicalComponent], every: &mut dyn FnMut(&std::collections::HashMap<String, AttributeValue>)) {
        for c in list {
            every(&c.attributes);
            for f in &c.functions {
                every(&f.attributes);
            }
            components(&c.sub_components, every);
        }
    }
    for la in &model.logical_architecture {
        components(&la.components, &mut every);
        la.functions.iter().for_each(|f| every(&f.attributes));
    }
    for sa in &model.system_analysis {
        sa.functions.iter().for_each(|f| every(&f.attributes));
        sa.components.iter().for_each(|c| every(&c.attributes));
    }
    for pa in &model.physical_architecture {
        pa.nodes.iter().for_each(|n| every(&n.attributes));
    }
    for diagram in &model.diagrams {
        if let Some(AttributeValue::List(items)) = diagram.attributes.get("tagged") {
            found.extend(items.iter().filter_map(|v| v.as_string()).map(str::to_string));
        }
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
            let tagged: Vec<String> = match diagram.attributes.get("tagged") {
                Some(AttributeValue::List(items)) => items.iter().filter_map(|v| v.as_string()).map(|t| format!("@{}", text::name(t))).collect(),
                _ => Vec::new(),
            };
            if !tagged.is_empty() {
                e.line(&format!("filter {};", tagged.join(" or ")));
            }
        });
    }
}
