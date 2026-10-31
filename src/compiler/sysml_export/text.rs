//! Names and literals of the SysML v2 textual notation, as the profile
//! writes them (`spec/SYSML_PROFILE.md`): a declared name is the ArcLang
//! name as written, quoted when it is not a basic identifier or is a
//! reserved word; a short name is the element's id; references go by short
//! name, the one name that is unique.

use crate::compiler::quantity::Quantity;

/// Reserved words of the SysML v2 textual notation: a name spelled like
/// one is only usable quoted.
const KEYWORDS: &[&str] = &[
    "about", "abstract", "accept", "action", "actor", "after", "alias", "all", "allocate", "allocation", "analysis", "and", "as",
    "assert", "assign", "assume", "at", "attribute", "bind", "binding", "by", "calc", "case", "comment", "concern", "connect",
    "connection", "constant", "constraint", "crosses", "decide", "def", "default", "defined", "dependency", "derived", "do", "doc",
    "else", "end", "entry", "enum", "event", "exhibit", "exit", "expose", "false", "feature", "filter", "first", "flow", "for",
    "fork", "frame", "from", "hastype", "if", "implies", "import", "in", "include", "individual", "inout", "interface", "istype",
    "item", "join", "language", "library", "locale", "loop", "merge", "message", "meta", "metadata", "nonunique", "not", "null",
    "objective", "occurrence", "of", "or", "ordered", "out", "package", "parallel", "part", "perform", "port", "private",
    "protected", "public", "redefines", "ref", "references", "render", "rendering", "rep", "require", "requirement", "return",
    "satisfy", "send", "snapshot", "specializes", "stakeholder", "standard", "state", "subject", "subsets", "succession",
    "terminate", "then", "timeslice", "to", "transition", "true", "until", "use", "variant", "variation", "verification", "verify",
    "via", "view", "viewpoint", "when", "while", "xor",
];

/// A name as SysML writes it: as is when it is a basic identifier that is
/// not reserved, else in the unrestricted form `'...'`.
pub fn name(raw: &str) -> String {
    let mut chars = raw.chars();
    let basic = match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => chars.all(|c| c.is_ascii_alphanumeric() || c == '_'),
        _ => false,
    };
    if basic && !KEYWORDS.contains(&raw) {
        raw.to_string()
    } else {
        format!("'{}'", raw.replace('\\', "\\\\").replace('\'', "\\'"))
    }
}

/// `<'LC-001'>`: the short name of an element of this id.
pub fn short(id: &str) -> String {
    format!("<{}>", name(id))
}

/// A short name and a name, as a declaration writes them: `<'LC-1'> 'Radar'`.
/// An element whose id is its name is written by its name alone: the name
/// is what a reader takes for the id when there is no short name.
pub fn declared(id: &str, display: &str) -> String {
    if id.is_empty() || id == display {
        return name(display);
    }
    format!("{} {}", short(id), name(display))
}

pub fn string(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

/// `doc /* ... */`, the end marker of the comment made harmless.
pub fn doc(text: &str) -> String {
    format!("doc /* {} */", text.replace("*/", "* /"))
}

pub fn number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        // A decimal stays decimal: `800.0` is a Real, `800` an Integer.
        format!("{}.0", n as i64)
    } else {
        n.to_string()
    }
}

pub fn quantity(q: &Quantity) -> String {
    // SysML's °C is a temperature difference: a level in °C is written in K.
    let value = if q.spec().offset != 0.0 { q.canonical() } else { q.value };
    let value = if value.fract() == 0.0 && value.abs() < 1e15 { format!("{}", value as i64) } else { format!("{}", (value * 1e9).round() / 1e9) };
    format!("{value} [{}]", q.spec().sysml)
}

/// The SysML scalar type an ArcLang data type name stands for.
pub fn scalar_type(name: &str) -> Option<&'static str> {
    match name.to_ascii_lowercase().as_str() {
        "string" | "str" | "text" => Some("String"),
        "int" | "integer" | "int32" | "int64" | "uint8" | "uint16" | "uint32" | "uint64" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" => Some("Integer"),
        "float" | "double" | "real" | "f32" | "f64" | "number" => Some("Real"),
        "bool" | "boolean" => Some("Boolean"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_quoted_only_when_they_must_be() {
        assert_eq!(name("Radar"), "Radar");
        assert_eq!(name("Radar Sensor"), "'Radar Sensor'");
        assert_eq!(name("LC-001"), "'LC-001'");
        assert_eq!(name("part"), "'part'");
        assert_eq!(name("it's"), "'it\\'s'");
    }

    #[test]
    fn a_declaration_has_its_id_as_short_name() {
        assert_eq!(declared("LC-1", "Radar Sensor"), "<'LC-1'> 'Radar Sensor'");
        assert_eq!(declared("Radar", "Radar"), "Radar");
        assert_eq!(declared("", "Radar"), "Radar");
    }

    #[test]
    fn a_decimal_keeps_its_point() {
        assert_eq!(number(800.0), "800.0");
        assert_eq!(number(0.5), "0.5");
    }
}
