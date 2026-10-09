//! `arclang set` / `arclang unset`: change one attribute of one element in a
//! model file, keeping comments and layout. The edit is applied to the text
//! (see `compiler::source_edit`) and only accepted once the edited model
//! compiles and shows the requested value.

use super::CliError;
use crate::compiler::ast::AttributeValue;
use crate::compiler::elements::{self, ElementGraph, ElementRecord};
use crate::compiler::source_edit::{self, EditError};
use crate::{Compiler, CompilerConfig};
use std::path::Path;

fn compile(path: &Path) -> Result<ElementGraph, CliError> {
    let result = Compiler::new(CompilerConfig::default())
        .compile_file(path)
        .map_err(|e| CliError::Compilation(format!("{}: {e}", path.display())))?;
    Ok(elements::build(&result.ast, &result.semantic_model))
}

fn find<'g>(graph: &'g ElementGraph, element: &str) -> Option<&'g ElementRecord> {
    graph
        .elements
        .iter()
        .find(|record| record.id == element)
        .or_else(|| graph.elements.iter().find(|record| record.name == element))
}

fn shown(record: Option<&ElementRecord>, key: &str) -> String {
    record
        .and_then(|record| record.attributes.get(key))
        .map_or_else(|| "(not set)".to_string(), AttributeValue::display)
}

/// Apply the edit to the element designated by `element`. The text knows
/// an element by the `id` it writes, or by its name when it writes none;
/// the compiled model says which of the two this element goes by.
fn apply(
    source: &str,
    record: Option<&ElementRecord>,
    element: &str,
    key: &str,
    value: Option<&str>,
) -> Result<String, EditError> {
    let mut designators = vec![element];
    if let Some(record) = record {
        designators.extend([record.id.as_str(), record.name.as_str()]);
    }
    let mut outcome = Err(EditError::NotFound(element.to_string()));
    for designator in designators {
        outcome = match value {
            Some(value) => source_edit::set_attribute(source, designator, key, value),
            None => source_edit::remove_attribute(source, designator, key),
        };
        if !matches!(outcome, Err(EditError::NotFound(_))) {
            break;
        }
    }
    outcome
}

/// Rename `element`. Its identity must survive: the compiled model must
/// hold the same elements and relationships, one of them under a new name.
pub fn rename(input: &Path, element: &str, new_name: &str, write: bool) -> Result<(), CliError> {
    let refused = |reason: String| CliError::Compilation(reason);
    let source = std::fs::read_to_string(input)?;
    let before = compile(input)?;
    let record = find(&before, element).ok_or_else(|| refused(format!("no element '{element}' in this model")))?;

    let mut designators = vec![element, record.id.as_str(), record.name.as_str()];
    designators.dedup();
    let mut edited = Err(EditError::NotFound(element.to_string()));
    for designator in designators {
        edited = source_edit::rename_element(&source, designator, new_name);
        if !matches!(edited, Err(EditError::NotFound(_))) {
            break;
        }
    }
    let edited = edited.map_err(|e| refused(e.to_string()))?;

    let compiled = source_edit::compile_beside(input, &edited)
        .map_err(|e| refused(format!("{}: {e}", input.display())))?;
    let after = elements::build(&compiled.ast, &compiled.semantic_model);
    let identities = |graph: &ElementGraph| -> Vec<String> {
        let mut all: Vec<String> = graph.elements.iter().map(|e| e.uuid.clone()).collect();
        all.extend(graph.relationships.iter().map(|r| r.uuid.clone()));
        all.sort();
        all
    };
    let renamed = after.elements.iter().find(|e| e.uuid == record.uuid);
    if renamed.map(|e| e.name.as_str()) != Some(new_name) || identities(&before) != identities(&after) {
        return Err(refused(format!(
            "edit refused: renaming '{}' would change its identity or what refers to it; \
             give it an `id` and refer to it by that id first",
            record.name
        )));
    }
    if after.unresolved.len() > before.unresolved.len() {
        return Err(refused(format!("edit refused: renaming '{}' would leave a relationship without an end", record.name)));
    }

    if !write {
        print!("{edited}");
        return Ok(());
    }
    std::fs::write(input, &edited)?;
    println!("{}: '{}' -> '{new_name}'", record.id, record.name);
    Ok(())
}

/// `value` is `Some` to set the attribute, `None` to remove it.
pub fn run(input: &Path, element: &str, key: &str, value: Option<&str>, write: bool) -> Result<(), CliError> {
    let source = std::fs::read_to_string(input)?;
    let before = compile(input)?;
    let record = find(&before, element);

    let edited = apply(&source, record, element, key, value).map_err(|e| match (&e, record) {
        (EditError::NotFound(_), Some(_)) => CliError::Compilation(format!(
            "'{element}' is part of this model but is not declared in {}: edit the imported file that declares it",
            input.display()
        )),
        _ => CliError::Compilation(e.to_string()),
    })?;

    let compiled = source_edit::compile_beside(input, &edited)
        .map_err(|e| CliError::Compilation(format!("{}: {e}", input.display())))?;
    let after = elements::build(&compiled.ast, &compiled.semantic_model);
    if let Some(value) = value {
        source_edit::confirm_effect(&after, element, key, value)
            .map_err(|e| CliError::Compilation(e.to_string()))?;
    }

    if !write {
        print!("{edited}");
        return Ok(());
    }
    if edited == source {
        println!("{element}.{key} unchanged");
        return Ok(());
    }
    std::fs::write(input, &edited)?;
    println!(
        "{element}.{key}: {} -> {}",
        shown(record, key),
        shown(find(&after, element), key)
    );
    Ok(())
}
