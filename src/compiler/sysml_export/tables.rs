//! Declared tables as SysML v2 views rendered as element tables: a
//! `view` with `render Views::asElementTable;`, its kind and parameters as
//! attributes (`attribute 'table' : String = "requirements_coverage";`).
//! What the table shows is computed from the model, so only what the
//! author declared is written; the import reads it back as the same
//! `table` declaration.

use super::{text, Exporter};
use crate::compiler::ast::AttributeValue;

/// The attributes written for a table, in this order; the others follow, sorted.
const ORDER: [&str; 8] = ["kind", "scope", "row_type", "rows", "columns", "kinds", "filter", "gaps_only"];

fn value(value: &AttributeValue) -> Option<(&'static str, String)> {
    Some(match value {
        AttributeValue::Boolean(flag) => ("Boolean", flag.to_string()),
        AttributeValue::List(items) => ("String", text::string(&items.iter().filter_map(|v| v.as_string()).collect::<Vec<_>>().join(", "))),
        AttributeValue::String(s) => ("String", text::string(s)),
        AttributeValue::Integer(i) => ("Integer", i.to_string()),
        AttributeValue::Number(n) => ("Real", text::number(*n)),
        _ => return None,
    })
}

pub(super) fn write(e: &mut Exporter<'_>) {
    let model = e.model;
    for table in &model.tables {
        e.block(&format!("view {}", text::declared(&table.id, &table.name)), |e| {
            if let Some(description) = table.attributes.get("description").and_then(|v| v.as_string()) {
                e.line(&text::doc(description));
            }
            let mut keys: Vec<&String> = table.attributes.keys().filter(|k| !matches!(k.as_str(), "id" | "description")).collect();
            keys.sort_by_key(|k| (ORDER.iter().position(|o| o == k).unwrap_or(ORDER.len()), k.to_string()));
            for key in keys {
                let Some((ty, literal)) = value(&table.attributes[key]) else { continue };
                // `kind` is written `table`: the attribute that says the view is one.
                let written = if key == "kind" { "table" } else { key.as_str() };
                e.line(&format!("attribute {} : {ty} = {literal};", text::name(written)));
            }
            e.line("render Views::asElementTable;");
        });
    }
}
