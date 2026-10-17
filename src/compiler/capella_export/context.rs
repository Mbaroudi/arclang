//! What the export knows while it writes: the Capella id of every ArcLang
//! reference (an id, a name, `Owner.port`), and the ids it has to make up
//! for the structure Capella needs and ArcLang does not write (packages,
//! parts, realizations, involvements).

use crate::compiler::ast::AttributeValue;
use std::collections::HashMap;
use uuid::Uuid;

/// The namespace of the ids the export makes up: the same model gives the
/// same ids, so an export is stable.
const NAMESPACE: Uuid = Uuid::from_bytes([0x6b, 0x1e, 0x2c, 0x4d, 0x8a, 0x3f, 0x4e, 0x51, 0x9c, 0x07, 0x2d, 0x6e, 0x5a, 0x41, 0x7b, 0x93]);

/// A made-up id, from what identifies the element in the model.
pub(super) fn derived(seed: &str) -> String {
    Uuid::new_v5(&NAMESPACE, seed.as_bytes()).to_string()
}

#[derive(Default)]
pub(super) struct Context {
    /// ArcLang reference -> Capella id.
    refs: HashMap<String, String>,
    /// Data types and classes by id and by name: a field or an element names a type.
    types: HashMap<String, String>,
    /// Capella ids of the functions of every level.
    pub(super) functions: std::collections::HashSet<String>,
    /// What could not be written, for the report.
    pub(super) notes: Vec<String>,
}

impl Context {
    /// Record that `reference` (an id or a name) is the element `id`. A name
    /// two elements share names neither: references by name must be unique.
    pub(super) fn register(&mut self, reference: &str, id: &str) {
        if reference.is_empty() {
            return;
        }
        match self.refs.get(reference) {
            Some(existing) if existing != id => {
                self.refs.insert(reference.to_string(), String::new());
            }
            Some(_) => {}
            None => {
                self.refs.insert(reference.to_string(), id.to_string());
            }
        }
    }
    /// The Capella id an ArcLang reference names, when it names one.
    pub(super) fn resolve(&self, reference: &str) -> Option<&str> {
        self.refs.get(reference).map(String::as_str).filter(|id| !id.is_empty())
    }
    /// Record a type (data type, enumeration, class) under its id and name.
    pub(super) fn register_type(&mut self, id: &str, name: &str) {
        self.types.insert(id.to_string(), id.to_string());
        self.types.entry(name.to_string()).or_insert_with(|| id.to_string());
        self.register(id, id);
        self.register(name, id);
    }
    /// The type a field, an element or a generalization names.
    pub(super) fn resolve_type(&self, reference: &str) -> Option<&str> {
        self.types.get(reference).map(String::as_str)
    }
    pub(super) fn note(&mut self, text: String) {
        if !self.notes.contains(&text) {
            self.notes.push(text);
        }
    }
}

/// The id of a port of `owner` (a function or a component): `Owner.port` names it.
pub(super) fn port_id(owner: &str, port: &str) -> String {
    derived(&format!("port:{owner}.{port}"))
}

/// A text attribute.
pub(super) fn text<'a>(attributes: &'a HashMap<String, AttributeValue>, key: &str) -> Option<&'a str> {
    attributes.get(key).and_then(AttributeValue::as_string)
}

/// A list attribute, as the texts it holds (a single text is a list of one).
pub(super) fn list(attributes: &HashMap<String, AttributeValue>, key: &str) -> Vec<String> {
    match attributes.get(key) {
        Some(AttributeValue::List(items)) => items.iter().filter_map(|i| i.as_string().map(str::to_string)).collect(),
        Some(AttributeValue::String(one)) => vec![one.clone()],
        _ => Vec::new(),
    }
}

/// The Capella id of an element: its `id:`, else one made up from `seed`.
pub(super) fn id_of(attributes: &HashMap<String, AttributeValue>, seed: &str) -> String {
    text(attributes, "id").map(str::to_string).unwrap_or_else(|| derived(seed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn made_up_ids_are_stable_uuids() {
        assert_eq!(derived("a"), derived("a"));
        assert_ne!(derived("a"), derived("b"));
        assert!(Uuid::parse_str(&derived("a")).is_ok());
    }

    #[test]
    fn a_name_two_elements_share_names_neither() {
        let mut context = Context::default();
        context.register("Pump", "1");
        context.register("Pump", "2");
        context.register("p-1", "1");
        assert_eq!(context.resolve("Pump"), None);
        assert_eq!(context.resolve("p-1"), Some("1"));
    }
}
