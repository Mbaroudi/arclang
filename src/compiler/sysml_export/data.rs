//! The data model, in its own package `Data`: enumerations as `enum def`,
//! data types as `attribute def` specializing their scalar base, exchange
//! items as `#ExchangeItem item def`, classes as `#Class item def` with
//! their fields and base, and an `item def` for every type a port or an
//! exchange names that the model does not declare.

use super::{id_of, text, Exporter};
use crate::compiler::ast::{Association, AssociationKind, DataAttribute, PortDirection};
use std::collections::BTreeSet;

pub(super) const PACKAGE: &str = "Data";

/// The item carried by a port or an exchange that names no type.
pub(super) const UNTYPED: &str = "Data";

pub(super) fn register(e: &mut Exporter<'_>) {
    let model = e.model;
    for data_type in &model.data_types {
        e.register(PACKAGE, id_of(&data_type.attributes, &data_type.id), &[&data_type.name]);
    }
    for item in &model.exchange_items {
        e.register(PACKAGE, if item.id.is_empty() { &item.name } else { &item.id }, &[&item.name]);
    }
    for class in &model.classes {
        e.register(PACKAGE, id_of(&class.attributes, &class.id), &[&class.name]);
    }
    let names: Vec<String> = undeclared(e).into_iter().collect();
    for name in &names {
        e.register(PACKAGE, name, &[]);
    }
    e.undeclared = names;
}

/// Types named by ports and exchanges that nothing declares.
fn undeclared(e: &Exporter<'_>) -> BTreeSet<String> {
    let model = e.model;
    let mut named: BTreeSet<String> = BTreeSet::new();
    let mut add = |t: &str| {
        let t = if t.is_empty() { UNTYPED } else { t };
        if text::scalar_type(t).is_none() {
            named.insert(t.to_string());
        }
    };
    for sa in &model.system_analysis {
        super::levels::visit_system_functions(&sa.functions, &mut |f| f.ports.iter().for_each(|p| add(&p.data_type)));
        for exchange in &sa.functional_exchanges {
            if !exchange.data_type.is_empty() {
                add(&exchange.data_type);
            }
        }
    }
    for oa in &model.operational_analysis {
        super::levels::visit_activities(oa, &mut |a| a.ports.iter().for_each(|p| add(&p.data_type)));
        oa.exchanges.iter().filter(|x| !x.data_type.is_empty()).for_each(|x| add(&x.data_type));
    }
    for la in &model.logical_architecture {
        super::levels::visit_logical_functions(la, &mut |f| f.ports.iter().for_each(|p| add(&p.data_type)));
        la.functional_exchanges.iter().filter(|x| !x.data_type.is_empty()).for_each(|x| add(&x.data_type));
    }
    for pa in &model.physical_architecture {
        pa.functional_exchanges.iter().filter(|x| !x.data_type.is_empty()).for_each(|x| add(&x.data_type));
        for f in &pa.functions {
            super::levels::visit_function_tree(f, &mut |f| f.ports.iter().for_each(|p| add(&p.data_type)));
        }
    }
    // What component ports carry: their port definitions name it.
    let mut interface = |t: &str| {
        if !t.is_empty() {
            add(t);
        }
    };
    for sa in &model.system_analysis {
        sa.components.iter().flat_map(|c| c.ports.iter()).chain(sa.external_actors.iter().flat_map(|a| a.ports.iter())).for_each(|p| interface(&p.interface_type));
    }
    // A port and the ports it holds.
    fn port(p: &crate::compiler::ast::ComponentPort, interface: &mut impl FnMut(&str)) {
        interface(&p.interface_type);
        p.ports.iter().for_each(|sub| port(sub, interface));
    }
    fn logical(list: &[crate::compiler::ast::LogicalComponent], interface: &mut impl FnMut(&str)) {
        for c in list {
            c.ports.iter().for_each(|p| port(p, interface));
            logical(&c.sub_components, interface);
        }
    }
    for la in &model.logical_architecture {
        logical(&la.components, &mut interface);
    }
    // The ports of a type are written in `Types`, with their port definitions.
    for ty in &model.types {
        ty.ports.iter().for_each(|p| port(p, &mut interface));
    }
    for pa in &model.physical_architecture {
        for node in &pa.nodes {
            node.component_ports.iter().chain(node.behavior_components.iter().flat_map(|b| b.ports.iter())).for_each(|p| interface(&p.interface_type));
        }
    }
    for item in &model.exchange_items {
        item.attributes.iter().for_each(|f| add(&f.attr_type));
        item.elements.iter().for_each(|t| add(t));
    }
    for class in &model.classes {
        class.fields.iter().for_each(|f| add(&f.attr_type));
    }
    named.retain(|n| !e.targets.contains_key(n));
    named
}

/// The SysML type of a field or a port type named `name`.
pub(super) fn type_reference(e: &Exporter<'_>, name: &str) -> String {
    let name = if name.is_empty() { UNTYPED } else { name };
    match text::scalar_type(name) {
        Some(scalar) => scalar.to_string(),
        None => e.reference(name).unwrap_or_else(|| text::name(name)),
    }
}

pub(super) fn direction(direction: &PortDirection) -> &'static str {
    match direction {
        PortDirection::In => "in",
        PortDirection::Out => "out",
        PortDirection::InOut => "inout",
    }
}

pub(super) fn write(e: &mut Exporter<'_>) {
    let model = e.model;
    let extra = e.undeclared.clone();
    if model.data_types.is_empty() && model.exchange_items.is_empty() && model.classes.is_empty() && extra.is_empty() {
        return;
    }
    e.package(&format!("package {PACKAGE}"), PACKAGE, |e| {
        for data_type in &model.data_types {
            let id = id_of(&data_type.attributes, &data_type.id);
            match &data_type.enumeration_values {
                Some(values) => e.block(&format!("enum def {}", text::declared(id, &data_type.name)), |e| {
                    e.header("Enumeration", &data_type.attributes, &[]);
                    for value in values {
                        match &value.value {
                            Some(v) => e.line(&format!("{} {{ {} }}", text::name(&value.name), text::doc(v))),
                            None => e.line(&format!("{};", text::name(&value.name))),
                        }
                    }
                }),
                None => {
                    let base = data_type.base_type.as_deref().and_then(text::scalar_type).map(|b| format!(" :> {b}")).unwrap_or_default();
                    e.block(&format!("attribute def {}{base}", text::declared(id, &data_type.name)), |e| {
                        e.header("DataType", &data_type.attributes, &[]);
                        if let Some(base) = data_type.base_type.as_deref().filter(|b| text::scalar_type(b).is_none() && !data_type.attributes.contains_key("base")) {
                            e.line(&format!("attribute base_type : String = {};", text::string(base)));
                        }
                        if let Some(unit) = data_type.unit.as_ref().filter(|_| !data_type.attributes.contains_key("unit")) {
                            e.line(&format!("attribute unit : String = {};", text::string(unit)));
                        }
                    });
                }
            }
        }
        for item in &model.exchange_items {
            let id = if item.id.is_empty() { item.name.as_str() } else { item.id.as_str() };
            e.block(&format!("#ExchangeItem item def {}", text::declared(id, &item.name)), |e| {
                e.header("ExchangeItem", &item.properties, &[]);
                if !item.stereotype.is_empty() && !item.properties.contains_key("mechanism") && !item.properties.contains_key("stereotype") {
                    e.line(&format!("attribute stereotype : String = {};", text::string(&item.stereotype)));
                }
                fields(e, &item.attributes);
                for element in &item.elements {
                    // Not named like the type: the name would resolve to itself.
                    let ty = type_reference(e, element);
                    // A value type (an enumeration, a data type) is an attribute, an item a reference.
                    let is_value = model.data_types.iter().any(|d| id_of(&d.attributes, &d.id) == element || d.name == *element);
                    let keyword = if is_value { "attribute" } else { "ref item" };
                    e.line(&format!("{keyword} {} : {ty};", text::name(&format!("element_{element}"))));
                }
            });
        }
        for class in &model.classes {
            let id = id_of(&class.attributes, &class.id);
            let base = class.extends.as_deref().map(|b| format!(" :> {}", type_reference(e, b))).unwrap_or_default();
            e.block(&format!("#Class item def {}{base}", text::declared(id, &class.name)), |e| {
                // The fields are the class's attributes: written once, typed.
                let mut field_names: Vec<&str> = class.fields.iter().map(|f| f.name.as_str()).collect();
                field_names.push("extends"); // the `:>` of the declaration
                e.header("Class", &class.attributes, &field_names);
                fields(e, &class.fields);
            });
        }
        for association in &model.associations {
            association_def(e, association);
        }
        for name in extra {
            e.line(&format!("item def {};", text::name(&name)));
        }
    });
}

/// `#Association connection def <'id'> has { end source : A; end target : B; }`:
/// an association between classes, its role, kind and multiplicity as
/// attributes.
fn association_def(e: &mut Exporter<'_>, association: &Association) {
    let (source, target) = (type_reference(e, &association.from), type_reference(e, &association.to));
    e.block(&format!("#Association connection def {}", text::declared(&association.id, &association.name)), |e| {
        e.header("Association", &association.attributes, &["role", "kind", "multiplicity"]);
        e.line(&format!("end source : {source};"));
        e.line(&format!("end target : {target};"));
        if let Some(role) = &association.role {
            e.line(&format!("attribute role : String = {};", text::string(role)));
        }
        if association.kind != AssociationKind::Association {
            e.line(&format!("attribute kind : String = {};", text::string(association.kind.keyword())));
        }
        if let Some(multiplicity) = &association.multiplicity {
            e.line(&format!("attribute multiplicity : String = {};", text::string(multiplicity)));
        }
    });
}

fn fields(e: &mut Exporter<'_>, fields: &[DataAttribute]) {
    let model = e.model;
    for field in fields {
        let ty = type_reference(e, &field.attr_type);
        // A field typed by a class or an item is an item, by a value an attribute.
        let is_item = model.classes.iter().any(|c| id_of(&c.attributes, &c.id) == field.attr_type || c.name == field.attr_type)
            || model.exchange_items.iter().any(|i| i.id == field.attr_type || i.name == field.attr_type)
            || e.undeclared.iter().any(|u| *u == field.attr_type);
        let keyword = if is_item { "item" } else { "attribute" };
        match &field.default_value {
            Some(value) => e.line(&format!("{keyword} {} : {ty} default = {};", text::name(&field.name), text::string(value))),
            None => e.line(&format!("{keyword} {} : {ty};", text::name(&field.name))),
        }
    }
}
