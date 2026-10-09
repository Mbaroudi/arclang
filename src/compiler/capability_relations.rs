//! Relations between capabilities of one Arcadia level: extend, include and
//! generalization. They are written as attributes of the source capability
//! (`extends: [...]`, `includes: [...]`, `specializes: [...]`) at the
//! operational, system and logical levels alike.

use super::ast::AttributeValue;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityRelation {
    /// The source adds behaviour to the target under some condition.
    Extends,
    /// The source always incorporates the target.
    Includes,
    /// The source is a special case of the target.
    Specializes,
}

impl CapabilityRelation {
    pub const ALL: [CapabilityRelation; 3] = [
        CapabilityRelation::Extends,
        CapabilityRelation::Includes,
        CapabilityRelation::Specializes,
    ];

    /// The attribute that declares this relation.
    pub fn key(self) -> &'static str {
        match self {
            CapabilityRelation::Extends => "extends",
            CapabilityRelation::Includes => "includes",
            CapabilityRelation::Specializes => "specializes",
        }
    }
}

/// The relations a capability declares, in a fixed order: by relation, then
/// in the order the targets were written.
pub fn declared_relations(
    attributes: &HashMap<String, AttributeValue>,
) -> Vec<(CapabilityRelation, &str)> {
    CapabilityRelation::ALL
        .into_iter()
        .flat_map(|relation| {
            let targets: Vec<&str> = match attributes.get(relation.key()) {
                Some(AttributeValue::List(items)) => {
                    items.iter().filter_map(|item| item.as_string()).collect()
                }
                Some(AttributeValue::String(single)) => vec![single.as_str()],
                _ => Vec::new(),
            };
            targets.into_iter().map(move |target| (relation, target))
        })
        .collect()
}

impl CapabilityRelation {
    /// Name of the relationship in the typed element graph.
    pub fn relationship(self) -> &'static str {
        match self {
            CapabilityRelation::Extends => "CapabilityExtend",
            CapabilityRelation::Includes => "CapabilityInclude",
            CapabilityRelation::Specializes => "CapabilityGeneralization",
        }
    }
}

/// The id of the capability a relation names, among the capabilities of
/// one level given as `(id, name)`: by id, else by a name only one of them
/// carries.
pub fn resolve_in_level<'a>(level: &[(&'a str, &'a str)], reference: &str) -> Option<&'a str> {
    if let Some((id, _)) = level.iter().find(|(id, _)| *id == reference) {
        return Some(id);
    }
    let mut named = level.iter().filter(|(_, name)| *name == reference);
    match (named.next(), named.next()) {
        (Some((id, _)), None) => Some(id),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(items: &[&str]) -> AttributeValue {
        AttributeValue::List(
            items
                .iter()
                .map(|item| AttributeValue::String(item.to_string()))
                .collect(),
        )
    }

    #[test]
    fn relations_come_out_by_kind_then_in_written_order() {
        let attributes = HashMap::from([
            ("specializes".to_string(), list(&["Base"])),
            ("extends".to_string(), list(&["B", "A"])),
            (
                "includes".to_string(),
                AttributeValue::String("Single".to_string()),
            ),
            ("involves".to_string(), list(&["Not a relation"])),
        ]);

        assert_eq!(
            declared_relations(&attributes),
            vec![
                (CapabilityRelation::Extends, "B"),
                (CapabilityRelation::Extends, "A"),
                (CapabilityRelation::Includes, "Single"),
                (CapabilityRelation::Specializes, "Base"),
            ]
        );
    }

    #[test]
    fn capability_without_relations_declares_none() {
        assert!(declared_relations(&HashMap::new()).is_empty());
    }

    #[test]
    fn relation_target_resolves_by_id_then_by_unshared_name() {
        let level = [("CAP-1", "Brake"), ("CAP-2", "Warn"), ("CAP-3", "Warn")];

        assert_eq!(resolve_in_level(&level, "CAP-2"), Some("CAP-2"));
        assert_eq!(resolve_in_level(&level, "Brake"), Some("CAP-1"));
        assert_eq!(
            resolve_in_level(&level, "Warn"),
            None,
            "a shared name names no one"
        );
        assert_eq!(resolve_in_level(&level, "Ghost"), None);
    }
}
