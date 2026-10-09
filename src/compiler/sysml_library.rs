//! The SysML v2 standard-library elements the export refers to: what each
//! written name designates (`String` → `ScalarValues::String`, a `DataType`;
//! `[kg]` → `SI::kilogram`, an `AttributeUsage`).
//!
//! The table is not written by hand. `spec/sysml_library_index.json` is
//! produced by the OMG pilot implementation from a package that uses every
//! library name the exporter can write (`sysmlv2_generator::library_probe`,
//! `tools/sysml_library_index.py`). A test fails when the exporter can
//! write a name the index does not hold.

use serde::Deserialize;
use std::sync::OnceLock;

/// How a name is used where it is written; the same spelling can designate
/// different elements in different positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// After `:` or `:>`.
    Type,
    /// In an expression: a unit, a prefix.
    Value,
    /// After `:>>`.
    Redefinition,
    /// Called in an expression.
    Function,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryElement {
    pub qualified_name: String,
    pub metaclass: String,
    pub name: Option<String>,
    pub short_name: Option<String>,
    pub roles: Vec<Role>,
}

#[derive(Deserialize)]
struct Index {
    elements: Vec<LibraryElement>,
}

/// Every indexed library element, sorted by qualified name.
pub fn elements() -> &'static [LibraryElement] {
    static INDEX: OnceLock<Vec<LibraryElement>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let index: Index = serde_json::from_str(include_str!("../../spec/sysml_library_index.json"))
            .expect("spec/sysml_library_index.json is generated and well-formed");
        index.elements
    })
}

/// The library element a written name designates in a given position: by
/// qualified name, by name or by short name (`kg`).
pub fn lookup(written: &str, role: Role) -> Option<&'static LibraryElement> {
    elements().iter().find(|element| {
        element.roles.contains(&role)
            && (element.qualified_name == written
                || element.name.as_deref() == Some(written)
                || element.short_name.as_deref() == Some(written))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_designate_what_the_pilot_says() {
        let string = lookup("String", Role::Type).unwrap();
        assert_eq!((string.qualified_name.as_str(), string.metaclass.as_str()), ("ScalarValues::String", "DataType"));
        assert_eq!(lookup("kg", Role::Value).unwrap().qualified_name, "SI::kilogram");
        assert_eq!(lookup("bit/s", Role::Value).unwrap().qualified_name, "SI::'bit per second'");
        assert_eq!(lookup("DataFunctions::max", Role::Function).unwrap().metaclass, "Function");
        assert_eq!(
            lookup("unitConversion", Role::Redefinition).unwrap().qualified_name,
            "MeasurementReferences::MeasurementUnit::unitConversion"
        );
    }

    #[test]
    fn a_name_only_resolves_in_the_position_it_is_indexed_for() {
        // `min` is the minute as a value and never a type.
        assert_eq!(lookup("min", Role::Value).unwrap().qualified_name, "SI::minute");
        assert_eq!(lookup("min", Role::Type), None);
        assert_eq!(lookup("NoSuchThing", Role::Type), None);
    }
}
