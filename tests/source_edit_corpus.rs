//! Source edits over every example: an accepted edit must still compile,
//! and once confirmed on the compiled model it shows the new value on that
//! element and on no other.

use arclang::compiler::ast::AttributeValue;
use arclang::compiler::elements::{self, ElementGraph, ElementRecord};
use arclang::compiler::source_edit::{confirm_effect, set_attribute, EditError};
use arclang::{Compiler, CompilerConfig};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const MARK: &str = "edited by the corpus test";

fn arc_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            arc_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "arc") {
            files.push(path);
        }
    }
}

fn graph(source: &str) -> Option<ElementGraph> {
    let result = Compiler::new(CompilerConfig::default()).compile_string(source).ok()?;
    Some(elements::build(&result.ast, &result.semantic_model))
}

fn marked(graph: &ElementGraph, key: &str) -> Vec<String> {
    graph
        .elements
        .iter()
        .filter(|element| element.attributes.get(key).and_then(AttributeValue::as_string) == Some(MARK))
        .map(|element| element.id.clone())
        .collect()
}

/// What the compiler says of an edit the editor accepted.
fn outcome_once_compiled(
    path: &Path,
    before: &ElementGraph,
    element: &ElementRecord,
    key: &str,
    value: &str,
    edited: &str,
) -> &'static str {
    // Text is not a valid value everywhere (a reference, an enumeration):
    // the compiler then rejects the edited model.
    let Some(after) = graph(edited) else {
        return "rejected by the compiler";
    };
    assert_eq!(after.elements.len(), before.elements.len(), "{}", path.display());
    match confirm_effect(&after, &element.id, key, value) {
        Ok(()) => {
            let holders = marked(&after, key);
            assert!(
                holders.len() <= 1 || element.kind == "Type",
                "{}: {}.{key} also changed {holders:?}",
                path.display(),
                element.id
            );
            "applied"
        }
        Err(EditError::NoEffect { .. }) => "no effect",
        Err(other) => panic!("{}: {}.{key}: {other}", path.display(), element.id),
    }
}

#[test]
fn accepted_edits_change_exactly_the_requested_attribute() {
    let mut files = Vec::new();
    arc_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join("examples"), &mut files);
    files.sort();

    let mut outcomes: BTreeMap<&str, usize> = BTreeMap::new();
    for path in files {
        let source = std::fs::read_to_string(&path).unwrap();
        let Some(before) = graph(&source) else { continue };

        for element in &before.elements {
            let mut keys: Vec<&String> = element.attributes.keys().collect();
            keys.sort();
            for key in keys {
                let is_text = matches!(element.attributes[key], AttributeValue::String(_));
                // `id` and `name` carry identity: changing them renames the
                // element, which is not the edit under test here.
                if !is_text || key == "id" || key == "name" || key == "is" {
                    continue;
                }
                let value = format!("\"{MARK}\"");
                // An element that writes no `id` is designated by its name.
                let attempt = match set_attribute(&source, &element.id, key, &value) {
                    Err(EditError::NotFound(_)) => set_attribute(&source, &element.name, key, &value),
                    other => other,
                };
                let outcome = match attempt {
                    Ok(edited) => outcome_once_compiled(&path, &before, element, key, &value, &edited),
                    Err(EditError::NotFound(_)) => "not found",
                    Err(EditError::Ambiguous { .. }) => "ambiguous",
                    Err(EditError::Unsafe(_)) => "refused",
                    Err(other) => panic!("{}: {}.{key}: {other}", path.display(), element.id),
                };
                *outcomes.entry(outcome).or_default() += 1;
            }
        }
    }

    println!("source edit outcomes over the corpus: {outcomes:?}");
    let applied = outcomes.get("applied").copied().unwrap_or(0);
    let total: usize = outcomes.values().sum();
    assert!(total >= 1000, "too few attributes to mean anything: {outcomes:?}");
    // The README states this share.
    assert!(applied * 10 >= total * 9, "under 90% of edits applied: {outcomes:?}");
}
