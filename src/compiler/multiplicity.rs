//! Multiplicity of an element in its owner: how many of it there are.
//!
//! Written `multiplicity: 4` for an exact count, or as text for a range:
//! `"0..1"`, `"1..4"`, `"1..*"`, `"*"` (any number, zero included).

use super::ast::AttributeValue;
use std::fmt;

/// Element kinds that can state a multiplicity: those exported as a part
/// definition and a part usage, the usage carrying the multiplicity.
pub const KINDS: &[&str] = &["LogicalComponent", "PhysicalNode"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Multiplicity {
    pub lower: u64,
    /// `None`: no upper bound (`*`).
    pub upper: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MultiplicityError {
    /// Not of the form `n`, `n..m`, `n..*` or `*`.
    Malformed(String),
    /// The lower bound exceeds the upper bound.
    Empty { lower: u64, upper: u64 },
}

impl fmt::Display for MultiplicityError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MultiplicityError::Malformed(text) => write!(
                f,
                "'{}' is not a multiplicity (expected a count such as 4, or a range such as \"0..1\", \"1..*\" or \"*\")",
                text
            ),
            MultiplicityError::Empty { lower, upper } => {
                write!(f, "multiplicity {}..{} is empty: the lower bound exceeds the upper bound", lower, upper)
            }
        }
    }
}

impl Multiplicity {
    pub fn exactly(count: u64) -> Multiplicity {
        Multiplicity { lower: count, upper: Some(count) }
    }

    pub fn parse(text: &str) -> Result<Multiplicity, MultiplicityError> {
        let malformed = || MultiplicityError::Malformed(text.to_string());
        let bound = |part: &str| part.trim().parse::<u64>().map_err(|_| malformed());
        let text_trimmed = text.trim();
        let multiplicity = match text_trimmed.split_once("..") {
            None if text_trimmed == "*" => Multiplicity { lower: 0, upper: None },
            None => Multiplicity::exactly(bound(text_trimmed)?),
            Some((lower, upper)) if upper.trim() == "*" => Multiplicity { lower: bound(lower)?, upper: None },
            Some((lower, upper)) => Multiplicity { lower: bound(lower)?, upper: Some(bound(upper)?) },
        };
        match multiplicity.upper {
            Some(upper) if multiplicity.lower > upper => Err(MultiplicityError::Empty { lower: multiplicity.lower, upper }),
            _ => Ok(multiplicity),
        }
    }

    /// The multiplicity an attribute value states.
    pub fn from_value(value: &AttributeValue) -> Result<Multiplicity, MultiplicityError> {
        match value {
            AttributeValue::Number(n) if n.fract() == 0.0 && *n >= 0.0 && *n < 1e15 => Ok(Multiplicity::exactly(*n as u64)),
            AttributeValue::String(text) => Multiplicity::parse(text),
            other => Err(MultiplicityError::Malformed(other.display())),
        }
    }

    /// The multiplicity an element of `kind` states with valid syntax, when
    /// its kind can state one.
    pub fn of(kind: &str, attributes: &std::collections::HashMap<String, AttributeValue>) -> Option<Multiplicity> {
        if !KINDS.contains(&kind) {
            return None;
        }
        attributes.get("multiplicity").and_then(|value| Multiplicity::from_value(value).ok())
    }

    /// `4`, `0..1`, `1..*`, `*`.
    pub fn canonical(&self) -> String {
        match (self.lower, self.upper) {
            (0, None) => "*".to_string(),
            (lower, None) => format!("{}..*", lower),
            (lower, Some(upper)) if lower == upper => lower.to_string(),
            (lower, Some(upper)) => format!("{}..{}", lower, upper),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_counts_and_ranges() {
        assert_eq!(Multiplicity::parse("4"), Ok(Multiplicity { lower: 4, upper: Some(4) }));
        assert_eq!(Multiplicity::parse(" 0..1 "), Ok(Multiplicity { lower: 0, upper: Some(1) }));
        assert_eq!(Multiplicity::parse("1 .. *"), Ok(Multiplicity { lower: 1, upper: None }));
        assert_eq!(Multiplicity::parse("*"), Ok(Multiplicity { lower: 0, upper: None }));
        assert_eq!(Multiplicity::from_value(&AttributeValue::Number(2.0)), Ok(Multiplicity::exactly(2)));
    }

    #[test]
    fn refuses_what_is_not_a_multiplicity() {
        for text in ["", "many", "-1", "1..", "..4", "1.5", "1..2..3", "*..4"] {
            assert!(matches!(Multiplicity::parse(text), Err(MultiplicityError::Malformed(_))), "{text}");
        }
        assert_eq!(Multiplicity::parse("4..2"), Err(MultiplicityError::Empty { lower: 4, upper: 2 }));
        assert!(Multiplicity::from_value(&AttributeValue::Number(1.5)).is_err());
        assert!(Multiplicity::from_value(&AttributeValue::Number(-1.0)).is_err());
    }

    #[test]
    fn canonical_form_reads_back_the_same() {
        for text in ["4", "0..1", "2..8", "1..*", "*"] {
            let multiplicity = Multiplicity::parse(text).unwrap();
            assert_eq!(multiplicity.canonical(), text);
            assert_eq!(Multiplicity::parse(&multiplicity.canonical()), Ok(multiplicity));
        }
        assert_eq!(Multiplicity::parse("0..*").unwrap().canonical(), "*");
    }
}
