//! What ArcLang declares and SysML v2 has no notion of: the I/O binding of
//! a state machine, the twin of a model, a rule of the project. Each is a
//! `part` of the package `Extensions` carrying its metadata (`#IoBinding`,
//! `#Twin`, `#Rule`), and everything it says is an attribute, what it names
//! included (the machine of a binding, the requirements of a twin): another
//! tool reads a part with attributes and nothing it would take for
//! structure or for a trace; the import reads the declaration back as
//! written. Without them a case that executes on a binding no longer
//! compiles, a twin is no longer followed and a rule no longer blocks.

use super::{attributes, text, Exporter};
use crate::compiler::ast::AttributeValue;
use std::collections::HashMap;

/// The package the declarations are written in.
pub const PACKAGE: &str = "Extensions";

/// The metadata of each declaration, as the profile names them.
pub const METADATA: [&str; 3] = ["IoBinding", "Twin", "Rule"];

pub(super) fn write(e: &mut Exporter<'_>) {
    let model = e.model;
    if model.io_bindings.is_empty() && model.twins.is_empty() && model.rules.is_empty() {
        return;
    }
    e.block(&format!("package {PACKAGE}"), |e| {
        for binding in &model.io_bindings {
            declaration(e, "IoBinding", &binding.id, &binding.name, &binding.attributes);
        }
        for twin in &model.twins {
            declaration(e, "Twin", &twin.id, &twin.name, &twin.attributes);
        }
        for rule in &model.rules {
            let id = rule.attributes.get("id").and_then(|v| v.as_string()).unwrap_or("");
            declaration(e, "Rule", id, &rule.name, &rule.attributes);
        }
    });
}

fn declaration(e: &mut Exporter<'_>, metadata: &str, id: &str, name: &str, declared: &HashMap<String, AttributeValue>) {
    e.block(&format!("#{metadata} part {}", text::declared(id, name)), |e| {
        if let Some(description) = declared.get("description").and_then(|d| d.as_string()) {
            e.line(&text::doc(description));
        }
        let mut keys: Vec<&String> = declared.keys().filter(|k| !matches!(k.as_str(), "id" | "name" | "description")).collect();
        keys.sort();
        for key in keys {
            let line = attributes::declared(&e.metamodel, metadata, key, &declared[key], &mut e.units);
            e.line(&line);
        }
    });
}
