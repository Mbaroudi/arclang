//! Who answers for an element, and which documents describe it.
//!
//! `organization "Powertrain team" { id: "ORG-PT" parent: "ORG-ENG" }` and
//! `document "Interface control document" { id: "DOC-ICD" reference: "ICD-0042" }`
//! are declared at the top of a file like notes and test cases. Any element
//! then names the organization responsible for it (`responsible: "ORG-PT"`)
//! and the documents that describe it (`documented_by: ["DOC-ICD"]`), by id
//! or by name.
//!
//! An organization or a document declared twice, or an organization part of
//! one that is not declared or of itself, is an error. A `responsible:` or a
//! `documented_by:` that names nothing declared is the model's own text: it
//! is kept, makes no relationship, and the compiler says so
//! (`metamodel_check`), since a model may have used those words before they
//! had a meaning.

use crate::compiler::ast::Model;
use crate::compiler::semantic::{ElementInfo, SemanticModel};

/// The organization a reference names, by id or by name: its id.
pub fn organization<'m>(ast: &'m Model, reference: &str) -> Option<&'m str> {
    ast.organizations.iter().find(|o| o.id == reference).or_else(|| ast.organizations.iter().find(|o| o.name == reference)).map(|o| o.id.as_str())
}

/// The document a reference names, by id or by name: its id.
pub fn document<'m>(ast: &'m Model, reference: &str) -> Option<&'m str> {
    ast.documents.iter().find(|d| d.id == reference).or_else(|| ast.documents.iter().find(|d| d.name == reference)).map(|d| d.id.as_str())
}

/// Organizations and documents join the elements of the model; one declared
/// twice, one whose id another element has, an organization part of nothing
/// declared or of itself are errors.
pub fn check(ast: &Model, semantic: &mut SemanticModel) -> Result<(), String> {
    let mut errors = Vec::new();
    let declared = ast.organizations.iter().map(|o| ("organization", "Organization", &o.id, &o.name)).chain(ast.documents.iter().map(|d| ("document", "Document", &d.id, &d.name)));
    for (word, kind, id, name) in declared {
        match semantic.all_elements.get(id) {
            Some(other) if other.element_type == kind => errors.push(format!("{word} '{id}' is declared more than once")),
            Some(other) => errors.push(format!("{word} '{name}': its id '{id}' is already that of a {}", other.element_type)),
            None => {
                semantic.all_elements.insert(id.clone(), ElementInfo::new(id.clone(), name.clone(), kind));
            }
        }
    }
    for organization in &ast.organizations {
        let Some(parent) = organization.attributes.get("parent").and_then(|v| v.as_string()) else { continue };
        if self::organization(ast, parent).is_none() {
            errors.push(format!("organization '{}' is part of '{parent}', which is no organization the model declares", organization.id));
            continue;
        }
        // Up the organizations it is part of: coming back to it is a loop.
        let mut seen = vec![organization.id.as_str()];
        let mut current = self::organization(ast, parent);
        while let Some(id) = current {
            if seen.contains(&id) {
                errors.push(format!("organization '{}' is part of itself (through {})", organization.id, seen[1..].join(", ")));
                break;
            }
            seen.push(id);
            current = ast.organizations.iter().find(|o| o.id == id).and_then(|o| o.attributes.get("parent")).and_then(|v| v.as_string()).and_then(|p| self::organization(ast, p));
        }
    }
    if errors.is_empty() { Ok(()) } else { Err(errors.join("\n")) }
}
