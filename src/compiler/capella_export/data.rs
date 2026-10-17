//! The data model: data types (numeric, string, boolean, physical
//! quantities with their units), enumerations, classes with their fields
//! and generalization, exchange items with their elements, and the
//! interfaces of each level with the exchange items allocated to them.
//!
//! The data of the model goes to the data package of the system analysis,
//! where Capella's own levels see it.

use super::context::{derived, id_of, list, text, Context};
use super::header::apply;
use super::xml::El;
use crate::compiler::ast::{Association, AssociationKind, LogicalInterface, Model};

/// `min..max` (or one number, `*` unbounded) as Capella's two cardinalities.
fn cardinalities(owner: &str, multiplicity: Option<&str>) -> [El; 2] {
    let (min, max) = match multiplicity.map(str::trim) {
        Some(m) => match m.split_once("..") {
            Some((min, max)) => (min.trim().to_string(), max.trim().to_string()),
            None => (m.to_string(), m.to_string()),
        },
        None => ("1".to_string(), "1".to_string()),
    };
    let value = |v: &str| if v == "*" { "-1".to_string() } else { v.to_string() };
    [
        El::new("ownedMinCard", "datavalue:LiteralNumericValue", &derived(&format!("{owner}:min"))).attr("value", &value(&min)),
        El::new("ownedMaxCard", "datavalue:LiteralNumericValue", &derived(&format!("{owner}:max"))).attr("value", &value(&max)),
    ]
}

fn aggregation(kind: AssociationKind) -> Option<&'static str> {
    match kind {
        AssociationKind::Composition => Some("COMPOSITION"),
        AssociationKind::Aggregation => Some("AGGREGATION"),
        AssociationKind::Association => None,
    }
}

fn field_id(class: &str, field: &str) -> String {
    derived(&format!("{class}:field:{field}"))
}

/// An association: its navigable end is the field `role` of the `from`
/// class; without a role, both ends are members of the association.
fn association(context: &Context, a: &Association) -> Option<El> {
    let (from, to) = (context.resolve_type(&a.from)?.to_string(), context.resolve_type(&a.to)?.to_string());
    let mut el = apply(El::new("ownedAssociations", "information:Association", &a.id).attr("name", &a.name), &a.attributes, &["from", "to", "role", "kind", "multiplicity"]);
    let back = El::new("ownedMembers", "information:Property", &derived(&format!("{}:source-end", a.id))).attr("name", "").attr("abstractType", &format!("#{from}"));
    let [min, max] = cardinalities(&format!("{}:source-end", a.id), None);
    el.push(back.with(min).with(max));
    match &a.role {
        Some(role) => el.set("navigableMembers", &format!("#{}", field_id(&from, role))),
        None => {
            let end_id = derived(&format!("{}:target-end", a.id));
            let mut end = El::new("ownedMembers", "information:Property", &end_id).attr("name", "").attr("abstractType", &format!("#{to}"));
            if let Some(kind) = aggregation(a.kind) {
                end.set("aggregationKind", kind);
            }
            let [min, max] = cardinalities(&end_id, a.multiplicity.as_deref());
            el.push(end.with(min).with(max));
        }
    }
    Some(el)
}

pub(super) fn register(context: &mut Context, model: &Model) {
    for t in &model.data_types {
        context.register_type(&t.id, &t.name);
    }
    for c in &model.classes {
        context.register_type(&c.id, &c.name);
    }
    for item in &model.exchange_items {
        context.register(&item.id, &item.id);
        context.register(&item.name, &item.id);
    }
    for la in &model.logical_architecture {
        for (index, interface) in la.interfaces.iter().enumerate() {
            let id = interface_id(interface, index);
            context.register(&id, &id);
            context.register(&interface.name, &id);
        }
    }
}

fn interface_id(interface: &LogicalInterface, index: usize) -> String {
    id_of(&interface.attributes, &format!("interface:{index}:{}", interface.name))
}

/// The data package of the model, or `None` when it declares no data.
pub(super) fn package(context: &Context, model: &Model, level_id: &str) -> Option<El> {
    if model.data_types.is_empty() && model.classes.is_empty() && model.exchange_items.is_empty() && model.associations.is_empty() {
        return None;
    }
    let mut pkg = El::new("ownedDataPkg", "information:DataPkg", &derived(&format!("{level_id}:data"))).attr("name", "Data");
    let mut units: Vec<(String, String)> = Vec::new();
    for t in &model.data_types {
        if let Some(values) = &t.enumeration_values {
            let mut el = El::new("ownedDataTypes", "datatype:Enumeration", &t.id).attr("name", &t.name);
            for value in values {
                el.push(El::new("ownedLiterals", "datavalue:EnumerationLiteral", &derived(&format!("{}:literal:{}", t.id, value.name))).attr("name", &value.name));
            }
            pkg.push(el);
            continue;
        }
        let base = t.base_type.as_deref().unwrap_or("String");
        let unit = t.unit.clone();
        let el = match (base, &unit) {
            (_, Some(unit)) => {
                let unit_id = derived(&format!("unit:{unit}"));
                if !units.iter().any(|(id, _)| *id == unit_id) {
                    units.push((unit_id.clone(), unit.clone()));
                }
                El::new("ownedDataTypes", "datatype:PhysicalQuantity", &t.id).attr("name", &t.name).attr("unit", &format!("#{unit_id}"))
            }
            ("Real" | "Float" | "Double", None) => El::new("ownedDataTypes", "datatype:NumericType", &t.id).attr("name", &t.name).attr("kind", "FLOAT"),
            ("Integer" | "Int", None) => El::new("ownedDataTypes", "datatype:NumericType", &t.id).attr("name", &t.name),
            ("Boolean" | "Bool", None) => El::new("ownedDataTypes", "datatype:BooleanType", &t.id).attr("name", &t.name),
            _ => El::new("ownedDataTypes", "datatype:StringType", &t.id).attr("name", &t.name),
        };
        pkg.push(el);
    }
    for c in &model.classes {
        let mut el = apply(El::new("ownedClasses", "information:Class", &c.id).attr("name", &c.name), &c.attributes, &field_keys(c));
        if let Some(base) = c.extends.as_deref().and_then(|b| context.resolve_type(b)) {
            el.push(
                El::new("ownedGeneralizations", "capellacore:Generalization", &derived(&format!("{}->{base}:generalization", c.id)))
                    .attr("super", &format!("#{base}"))
                    .attr("sub", &format!("#{}", c.id)),
            );
        }
        for field in &c.fields {
            // A field that is the navigable end of an association takes its kind and multiplicity.
            let role = model.associations.iter().find(|a| a.role.as_deref() == Some(field.name.as_str()) && context.resolve_type(&a.from) == Some(c.id.as_str()));
            let id = field_id(&c.id, &field.name);
            let mut property = El::new("ownedFeatures", "information:Property", &id).attr("name", &field.name);
            match role.and_then(|a| aggregation(a.kind)) {
                Some(kind) => property.set("aggregationKind", kind),
                None if role.is_none() => property.set("aggregationKind", "COMPOSITION"),
                None => {}
            }
            let [min, max] = cardinalities(&id, role.and_then(|a| a.multiplicity.as_deref()));
            let property = property.with(min).with(max);
            let property = match context.resolve_type(&field.attr_type) {
                Some(t) => property.attr("abstractType", &format!("#{t}")),
                None => property,
            };
            el.push(property);
        }
        pkg.push(el);
    }
    for item in &model.exchange_items {
        let mechanism = match item.stereotype.as_str() {
            "DATA" => "FLOW",
            other => other,
        };
        let mut el = El::new("ownedExchangeItems", "information:ExchangeItem", &item.id).attr("name", &item.name).attr("exchangeMechanism", mechanism);
        for (index, element) in item.elements.iter().enumerate() {
            if let Some(t) = context.resolve_type(element).or_else(|| context.resolve(element)) {
                let id = derived(&format!("{}:element:{index}", item.id));
                let [min, max] = cardinalities(&id, None);
                el.push(El::new("ownedElements", "information:ExchangeItemElement", &id).attr("abstractType", &format!("#{t}")).attr("direction", "UNSET").with(min).with(max));
            }
        }
        pkg.push(el);
    }
    for a in &model.associations {
        if let Some(el) = association(context, a) {
            pkg.push(el);
        }
    }
    for (id, name) in units {
        pkg.push(El::new("ownedUnits", "information:Unit", &id).attr("name", &name));
    }
    Some(pkg)
}

/// The class attributes that are fields, not header attributes.
fn field_keys(c: &crate::compiler::ast::ClassDef) -> Vec<&str> {
    let mut keys: Vec<&str> = c.fields.iter().map(|f| f.name.as_str()).collect();
    keys.extend(["extends", "items"]);
    keys
}

/// The interface package of a level (`system`, `logical` or `physical`), or
/// `None` when the model declares no interface there.
pub(super) fn interfaces(context: &Context, model: &Model, level: &str, level_id: &str) -> Option<El> {
    let mut pkg = El::new("ownedInterfacePkg", "cs:InterfacePkg", &derived(&format!("{level_id}:interfaces"))).attr("name", "Interfaces");
    for la in &model.logical_architecture {
        for (index, interface) in la.interfaces.iter().enumerate() {
            if text(&interface.attributes, "level").unwrap_or("logical") != level {
                continue;
            }
            let id = interface_id(interface, index);
            let mut el = apply(El::new("ownedInterfaces", "cs:Interface", &id).attr("name", &interface.name), &interface.attributes, &["level", "exchange_items", "from", "to"]);
            for item in list(&interface.attributes, "exchange_items") {
                if let Some(target) = context.resolve(&item) {
                    el.push(El::new("ownedExchangeItemAllocations", "cs:ExchangeItemAllocation", &derived(&format!("{id}->{target}:allocation"))).attr("allocatedItem", &format!("#{target}")));
                }
            }
            pkg.push(el);
        }
    }
    (!pkg.is_empty()).then_some(pkg)
}
