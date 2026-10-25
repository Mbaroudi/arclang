//! The attributes of an element as typed SysML attributes: a quantity the
//! metamodel types is its ISQ value type with its unit, an enumeration a
//! `String` of its canonical literal, numbers `Real` or `Integer`, a flag
//! `Boolean`, a list a sequence `(a, b)`. What a key refers to is written as
//! a relationship by the element's writer, never as text here.

use super::text;
use crate::compiler::ast::AttributeValue;
use crate::compiler::metamodel::{AttrType, Metamodel};
use std::collections::{BTreeSet, HashMap};

/// Keys carried by the declaration itself (short name, name, doc) or by a
/// relationship: not attributes.
pub const NOT_ATTRIBUTES: &[&str] = &[
    "id", "name", "title", "description", "involves", "realizes", "realized_by", "allocated_functions", "allocated_component", "mission",
    "verifies", "mitigated_by", "participants", "from", "to", "owner", "applies_to", "members", "elements", "values", "required",
    "parent", "performed_by", "deploys", "satisfies", "refines", "implements", "is", "specializes", "includes", "extends", "exploits",
];

/// The attribute lines of an element of `kind`, and the units they use.
/// What it `inherits` from a type unchanged is the type's, not written;
/// what it changes is a redefinition (`attribute :>> rate = ...;`).
pub fn lines(metamodel: &Metamodel, kind: &str, attributes: &HashMap<String, AttributeValue>, skip: &[&str], inherits: Option<&HashMap<String, AttributeValue>>, units: &mut BTreeSet<&'static str>) -> Vec<String> {
    let mut keys: Vec<&String> = attributes.keys().filter(|k| !NOT_ATTRIBUTES.contains(&k.as_str()) && !skip.contains(&k.as_str())).collect();
    keys.sort();
    let mut out = Vec::new();
    for key in keys {
        let value = &attributes[key];
        match inherits.and_then(|i| i.get(key.as_str())) {
            Some(same) if same == value => continue,
            Some(_) => {
                let written = line(metamodel, kind, key, value, units);
                // A block (a location, a shape) redefines with its body.
                if written.contains('\n') {
                    out.push(written.replacen("attribute ", "attribute :>> ", 1));
                    continue;
                }
                let literal = written.split_once(" = ").map(|(_, l)| l).unwrap_or(";");
                out.push(format!("attribute :>> {} = {literal}", text::name(key)));
            }
            None => out.push(line(metamodel, kind, key, value, units)),
        }
    }
    out
}

fn line(metamodel: &Metamodel, kind: &str, key: &str, value: &AttributeValue, units: &mut BTreeSet<&'static str>) -> String {
    let declared = metamodel.kind(kind).and_then(|k| k.attributes.iter().find(|a| a.key == key)).map(|a| a.ty.clone());
    let name = text::name(key);
    let (ty, literal) = match (declared, value) {
        (Some(AttrType::Quantity(dimension)), value) => match value.as_quantity() {
            Some(Ok(quantity)) if quantity.dimension() == dimension => {
                units.insert(quantity.spec().sysml);
                (dimension.sysml_value_type().to_string(), text::quantity(&quantity))
            }
            // Not a quantity of its dimension: kept as written.
            _ => ("String".to_string(), text::string(&value.display())),
        },
        // A location, a position, an orientation, a shape, a route: nested
        // attributes, each typed by its quantity (`attribute lat : AngularMeasureValue = 48.85 ['°'];`).
        (Some(AttrType::Location | AttrType::Position | AttrType::Orientation | AttrType::Shape), AttributeValue::Map(_)) => return block(&name, value, units),
        (Some(AttrType::List(inner)), AttributeValue::List(items)) if *inner == AttrType::Location && items.iter().all(|i| matches!(i, AttributeValue::Map(_))) => return block(&name, value, units),
        (Some(AttrType::Enum(enumeration)), value) => {
            let raw = value.display();
            let canonical = metamodel.normalize_enum(enumeration, &raw).map(str::to_string).unwrap_or(raw);
            ("String".to_string(), text::string(&canonical))
        }
        (_, value) => scalar(value, units),
    };
    format!("attribute {name} : {ty} = {literal};")
}

/// A block of nested attributes: a map, one attribute per key; a list of
/// maps, one block per item, named `point_1`, `point_2`... in order.
fn block(name: &str, value: &AttributeValue, units: &mut BTreeSet<&'static str>) -> String {
    let mut body = Vec::new();
    match value {
        AttributeValue::Map(map) => {
            // The order a reader expects (lat, lon, alt; x, y, z; yaw, pitch, roll), then the rest by name.
            const ORDER: [&str; 9] = ["lat", "lon", "alt", "x", "y", "z", "yaw", "pitch", "roll"];
            let rank = |k: &str| ORDER.iter().position(|o| *o == k).unwrap_or(ORDER.len());
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_by(|a, b| rank(a).cmp(&rank(b)).then(a.cmp(b)));
            for key in keys {
                let item = &map[key];
                let inner = text::name(key);
                body.push(match item {
                    AttributeValue::Map(_) => block(&inner, item, units),
                    AttributeValue::List(items) if items.iter().any(|i| matches!(i, AttributeValue::Map(_))) => block(&inner, item, units),
                    _ => {
                        let (ty, literal) = scalar(item, units);
                        format!("attribute {inner} : {ty} = {literal};")
                    }
                });
            }
        }
        AttributeValue::List(items) => {
            for (i, item) in items.iter().enumerate() {
                body.push(block(&format!("point_{}", i + 1), item, units));
            }
        }
        other => {
            let (ty, literal) = scalar(other, units);
            return format!("attribute {name} : {ty} = {literal};");
        }
    }
    let indented: Vec<String> = body.iter().flat_map(|b| b.lines().map(|l| format!("    {l}")).collect::<Vec<_>>()).collect();
    format!("attribute {name} {{\n{}\n}}", indented.join("\n"))
}

/// The type and literal of a value the metamodel does not type.
fn scalar(value: &AttributeValue, units: &mut BTreeSet<&'static str>) -> (String, String) {
    match value {
        AttributeValue::Integer(i) => ("Integer".to_string(), i.to_string()),
        AttributeValue::Number(n) => ("Real".to_string(), text::number(*n)),
        AttributeValue::Boolean(b) => ("Boolean".to_string(), b.to_string()),
        // The parser keeps `true` and `false` as text.
        AttributeValue::String(s) if s == "true" || s == "false" => ("Boolean".to_string(), s.clone()),
        AttributeValue::String(s) => ("String".to_string(), text::string(s)),
        AttributeValue::Quantity(q) => {
            units.insert(q.spec().sysml);
            (q.dimension().sysml_value_type().to_string(), text::quantity(q))
        }
        AttributeValue::List(items) => {
            let typed: Vec<(String, String)> = items.iter().map(|i| scalar(i, units)).collect();
            let ty = match typed.first() {
                Some((first, _)) if typed.iter().all(|(t, _)| t == first) => first.clone(),
                _ => "Anything".to_string(),
            };
            let literal = match typed.len() {
                0 => "()".to_string(),
                // A sequence of one is written with a trailing comma.
                1 => format!("({},)", typed[0].1),
                _ => format!("({})", typed.iter().map(|(_, l)| l.as_str()).collect::<Vec<_>>().join(", ")),
            };
            // Several values: the attribute is multi-valued.
            (format!("{} [*]", if ty == "Anything" { "String" } else { &ty }), literal)
        }
        AttributeValue::Map(_) => ("String".to_string(), text::string(&value.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn written(pairs: &[(&str, AttributeValue)]) -> Vec<String> {
        let attributes: HashMap<String, AttributeValue> = pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect();
        lines(&Metamodel::current(), "SystemFunction", &attributes, &[], None, &mut BTreeSet::new())
    }

    #[test]
    fn values_are_typed_and_lists_are_sequences() {
        let out = written(&[
            ("count", AttributeValue::Integer(3)),
            ("ratio", AttributeValue::Number(800.0)),
            ("tags", AttributeValue::List(vec![AttributeValue::String("a".into()), AttributeValue::String("b".into())])),
            ("one", AttributeValue::List(vec![AttributeValue::Integer(1)])),
            ("flag", AttributeValue::String("true".into())),
        ]);
        assert_eq!(
            out,
            [
                "attribute count : Integer = 3;",
                "attribute flag : Boolean = true;",
                "attribute one : Integer [*] = (1,);",
                "attribute ratio : Real = 800.0;",
                "attribute tags : String [*] = (\"a\", \"b\");",
            ]
        );
    }

    #[test]
    fn references_and_identity_are_not_attributes() {
        let out = written(&[("id", AttributeValue::String("F1".into())), ("involves", AttributeValue::List(vec![])), ("description", AttributeValue::String("d".into()))]);
        assert!(out.is_empty(), "{out:?}");
    }
}
