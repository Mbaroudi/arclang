//! `elements`: the elements of any kind, one row each, the columns picked
//! among the attributes the metamodel gives that kind (and those written).

use super::context::{shown, Context};
use super::{label_of, Cell, Column, Row, TableSpec};
use crate::compiler::ast::AttributeValue;
use crate::compiler::metamodel::AttrType;
use std::collections::{BTreeMap, BTreeSet};

const FIXED: [&str; 4] = ["id", "name", "kind", "owner"];
/// How many attribute columns a table shows when it names none.
const DEFAULT_ATTRIBUTE_COLUMNS: usize = 6;

fn kinds_of(spec: &TableSpec) -> Vec<String> {
    spec.row_type.as_deref().map(|t| t.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()).unwrap_or_default()
}

pub fn compute(context: &Context, spec: &TableSpec) -> Result<(Vec<Column>, Vec<Column>, Vec<Row>), String> {
    context.check_scope(spec.scope.as_deref())?;
    let kinds = kinds_of(spec);
    for kind in &kinds {
        if context.metamodel.kind(kind).is_none() {
            return Err(format!("row type '{kind}' is no kind of the metamodel (arclang metamodel lists them)"));
        }
    }
    let rows_of: Vec<usize> = (0..context.graph.elements.len())
        .filter(|&i| {
            let e = context.element(i);
            e.kind != "Model" && (kinds.is_empty() || kinds.iter().any(|k| k == e.kind)) && context.in_scope(i, spec.scope.as_deref())
        })
        .collect();
    // The metamodel's attributes of the row kinds first, in its order; then what is written.
    let mut declared: Vec<String> = Vec::new();
    let row_kinds: BTreeSet<&str> = rows_of.iter().map(|&i| context.element(i).kind).collect();
    for kind in &row_kinds {
        for attribute in context.metamodel.kind(kind).map(|k| k.attributes.as_slice()).unwrap_or_default() {
            if attribute.key != "id" && !declared.iter().any(|d| d == attribute.key) {
                declared.push(attribute.key.to_string());
            }
        }
    }
    let mut written: BTreeSet<String> = BTreeSet::new();
    for &i in &rows_of {
        written.extend(context.element(i).attributes.keys().filter(|k| k.as_str() != "id").cloned());
    }
    let mut available: Vec<Column> = FIXED.iter().map(|k| column(k)).collect();
    available.extend(declared.iter().map(|k| column(k)));
    available.extend(written.iter().filter(|k| !declared.contains(k) && !FIXED.contains(&k.as_str())).map(|k| column(k)));

    let keys: Vec<String> = if spec.columns.is_empty() {
        let mut keys: Vec<String> = FIXED.iter().map(|s| s.to_string()).collect();
        if row_kinds.len() == 1 {
            keys.retain(|k| k != "kind");
        }
        // An owner column only when some row has an owner other than the model.
        if !rows_of.iter().any(|&i| context.owners(i).first().is_some_and(|&o| context.element(o).kind != "Model")) {
            keys.retain(|k| k != "owner");
        }
        let used: Vec<String> = declared.iter().chain(written.iter().filter(|k| !declared.contains(k))).filter(|k| written.contains(*k)).cloned().collect();
        keys.extend(used.into_iter().filter(|k| !FIXED.contains(&k.as_str())).take(DEFAULT_ATTRIBUTE_COLUMNS));
        keys
    } else {
        spec.columns.clone()
    };
    let columns: Vec<Column> = keys.iter().map(|k| available.iter().find(|c| &c.key == k).cloned().unwrap_or_else(|| column(k))).collect();
    let rows = rows_of.iter().map(|&i| row(context, i, &keys)).collect();
    Ok((columns, available, rows))
}

fn column(key: &str) -> Column {
    let ty = if key == "owner" { "links" } else { "text" };
    Column { key: key.to_string(), label: if key == "id" { "Id".to_string() } else { label_of(key) }, ty: ty.to_string() }
}

fn row(context: &Context, index: usize, keys: &[String]) -> Row {
    let element = context.element(index);
    let mut cells = BTreeMap::new();
    for key in keys {
        let cell = match key.as_str() {
            "id" => Cell::text(element.id.clone()),
            "name" => Cell::text(element.name.clone()).editable(context.rename_edit(index)),
            "kind" => Cell::text(element.kind),
            "owner" => Cell::links(context.owners(index).first().filter(|&&o| context.element(o).kind != "Model").map(|&o| vec![context.link(o)]).unwrap_or_default()),
            other => attribute_cell(context, index, other),
        };
        cells.insert(key.clone(), cell);
    }
    Row { id: element.id.clone(), uuid: element.uuid.clone(), kind: element.kind.to_string(), name: element.name.clone(), status: "ok".to_string(), issues: Vec::new(), cells }
}

/// An attribute: its text, with links when it names elements.
fn attribute_cell(context: &Context, index: usize, key: &str) -> Cell {
    let element = context.element(index);
    let Some(value) = element.attributes.get(key) else {
        return Cell::text("").editable(context.attribute_edit(index, key));
    };
    let reference = context.metamodel.kind(element.kind).and_then(|k| k.attributes.iter().find(|a| a.key == key)).is_some_and(|a| matches!(&a.ty, AttrType::Reference(_)) || matches!(&a.ty, AttrType::List(inner) if matches!(**inner, AttrType::Reference(_))));
    let edit = context.attribute_edit(index, key);
    if reference {
        let names: Vec<String> = match value {
            AttributeValue::List(items) => items.iter().filter_map(|v| v.as_string()).map(str::to_string).collect(),
            other => other.as_string().map(|s| vec![s.to_string()]).unwrap_or_default(),
        };
        let links: Vec<_> = names.iter().filter_map(|n| context.resolve(n)).map(|i| context.link(i)).collect();
        if links.len() == names.len() && !links.is_empty() {
            return Cell { text: shown(value), ..Cell::links(links) }.editable(edit);
        }
    }
    Cell::text(shown(value)).editable(edit)
}
