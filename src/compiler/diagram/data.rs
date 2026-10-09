//! Data model: classes with their fields, enumerations with their values,
//! data types, and the exchange items that group them.

use super::builder::{properties, Builder, Missing, NodeRef};
use super::{Diagram, EdgeKind, NodeKind, ViewKind};
use crate::compiler::ast::{DataAttribute, ExchangeItem, Model};
use std::collections::BTreeMap;

pub(super) fn build(model: &Model) -> Option<(Diagram, Vec<String>)> {
    let mut builder = Builder::new(ViewKind::Cdb);
    // What owns typed fields, kept by handle: looking an owner up again by
    // name could miss it when two elements share that name.
    let mut owners: Vec<(NodeRef, &str, &[DataAttribute])> = Vec::new();
    let mut items: Vec<(NodeRef, &ExchangeItem)> = Vec::new();

    for class in &model.classes {
        let node = builder.add_node(
            None,
            &class.id,
            &class.name,
            NodeKind::Class,
            properties(&class.attributes, &["description"]),
        );
        builder.set_compartment(node, class.fields.iter().map(field_line).collect());
        owners.push((node, &class.name, &class.fields));
    }
    for data_type in &model.data_types {
        let (kind, lines) = match &data_type.enumeration_values {
            Some(values) => (
                NodeKind::Enumeration,
                values
                    .iter()
                    .map(|value| match &value.value {
                        Some(literal) => format!("{} = {literal}", value.name),
                        None => value.name.clone(),
                    })
                    .collect(),
            ),
            None => (
                NodeKind::DataType,
                data_type
                    .base_type
                    .iter()
                    .map(|base| format!("base : {base}"))
                    .chain(data_type.unit.iter().map(|unit| format!("unit : {unit}")))
                    .collect(),
            ),
        };
        let node = builder.add_node(None, &data_type.id, &data_type.name, kind, BTreeMap::new());
        builder.set_compartment(node, lines);
    }
    for item in &model.exchange_items {
        let mut item_properties = BTreeMap::new();
        if !item.stereotype.is_empty() {
            item_properties.insert("mechanism".to_string(), item.stereotype.clone());
        }
        let node = builder.add_node(
            None,
            &item.id,
            &item.name,
            NodeKind::ExchangeItem,
            item_properties,
        );
        builder.set_compartment(node, item.attributes.iter().map(field_line).collect());
        owners.push((node, &item.name, &item.attributes));
        items.push((node, item));
    }
    if builder.is_empty() {
        return None;
    }

    // A field whose type is a declared data element is an association.
    // `float` or `String` name no element and stay text in the box; a type
    // that several elements answer to is reported, not guessed.
    for (owner, owner_name, fields) in owners {
        let owner_id = builder.node(owner).id.clone();
        for field in fields {
            let what = format!("field '{owner_name}.{}'", field.name);
            match builder.find(&field.attr_type) {
                Ok(target) => {
                    let (source, target) = (builder.whole(owner), builder.whole(target));
                    let edge = builder.add_edge(
                        EdgeKind::Association,
                        &source,
                        &target,
                        &format!("{owner_id}.{}", field.name),
                        None,
                        BTreeMap::new(),
                    );
                    builder.set_edge_label(&edge, field.name.clone());
                }
                Err(Missing::Ambiguous) => {
                    builder.report_missing(
                        &what,
                        &field.attr_type,
                        Missing::Ambiguous,
                        "data element",
                    );
                }
                Err(Missing::Unknown) => {}
            }
        }
    }
    for (node, item) in items {
        let what = format!("exchange_item '{}'", item.name);
        for element in &item.elements {
            match builder.find(element) {
                Ok(target) => {
                    let (source, target) = (builder.whole(node), builder.whole(target));
                    let edge = builder.add_edge(
                        EdgeKind::ItemElement,
                        &source,
                        &target,
                        &format!("{} groups {element}", item.name),
                        None,
                        BTreeMap::new(),
                    );
                    builder.set_edge_label(&edge, String::new());
                }
                Err(missing) => builder.report_missing(
                    &what,
                    element,
                    missing,
                    "class, enumeration or data type",
                ),
            }
        }
    }
    builder.finish("Data model".to_string())
}

/// A field in UML attribute notation: `name : Type = default`.
fn field_line(field: &DataAttribute) -> String {
    match &field.default_value {
        Some(default) => format!("{} : {} = {default}", field.name, field.attr_type),
        None => format!("{} : {}", field.name, field.attr_type),
    }
}
