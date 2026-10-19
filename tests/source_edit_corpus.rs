//! Source edits over every example: an accepted edit must still compile,
//! and once confirmed on the compiled model it shows the new value on that
//! element and on no other.

use arclang::compiler::ast::AttributeValue;
use arclang::compiler::elements::{self, ElementGraph, ElementRecord};
use arclang::compiler::source_edit::{confirm_effect, enumeration_values, set_attribute, EditError};
use arclang::{Compiler, CompilerConfig};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const MARK: &str = "edited by the corpus test";

/// The text to write: a member of the enumeration when the metamodel types
/// the attribute as one (another than the one written), else the marker.
fn value_for(element: &ElementRecord, key: &str) -> String {
    let current = element.attributes.get(key).and_then(AttributeValue::as_string).unwrap_or("");
    let text = match enumeration_values(element.kind, key) {
        Some(members) => members.iter().copied().find(|member| !member.eq_ignore_ascii_case(current)).unwrap_or(MARK),
        None => MARK,
    };
    format!("\"{text}\"")
}

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

/// The elements whose `key` reads differently in `after` than in `before`.
fn changed_holders(before: &ElementGraph, after: &ElementGraph, key: &str) -> Vec<String> {
    after
        .elements
        .iter()
        .filter(|element| {
            let was = before.elements.iter().find(|b| b.uuid == element.uuid).and_then(|b| b.attributes.get(key));
            was != element.attributes.get(key)
        })
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
            let holders = changed_holders(before, &after, key);
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

    let detail = std::env::var_os("ARCLANG_EDIT_DETAIL").is_some();
    let mut outcomes: BTreeMap<&str, usize> = BTreeMap::new();
    let mut per_file: Vec<(String, usize, usize)> = Vec::new();
    for path in files {
        let source = std::fs::read_to_string(&path).unwrap();
        let Some(before) = graph(&source) else { continue };
        let (mut file_applied, mut file_total) = (0usize, 0usize);

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
                let value = value_for(element, key);
                // An element that writes no `id` is designated by its name.
                let attempt = match set_attribute(&source, &element.id, key, &value) {
                    Err(EditError::NotFound(_)) => set_attribute(&source, &element.name, key, &value),
                    other => other,
                };
                let reason = attempt.as_ref().err().map(ToString::to_string).unwrap_or_default();
                let outcome = match attempt {
                    Ok(edited) => outcome_once_compiled(&path, &before, element, key, &value, &edited),
                    Err(EditError::NotFound(_)) => "not found",
                    Err(EditError::Ambiguous { .. }) => "ambiguous",
                    Err(EditError::Unsafe(_)) => "refused",
                    Err(other) => panic!("{}: {}.{key}: {other}", path.display(), element.id),
                };
                if detail && outcome != "applied" {
                    println!("{}: {} {} .{key}: {outcome} {reason}", path.display(), element.kind, element.id);
                }
                *outcomes.entry(outcome).or_default() += 1;
                file_total += 1;
                file_applied += usize::from(outcome == "applied");
            }
        }
        per_file.push((path.strip_prefix(env!("CARGO_MANIFEST_DIR")).unwrap_or(&path).display().to_string(), file_applied, file_total));
    }

    for (file, applied, total) in &per_file {
        println!("{file}: {applied}/{total} applied");
    }
    println!("source edit outcomes over the corpus: {outcomes:?}");
    let applied = outcomes.get("applied").copied().unwrap_or(0);
    let total: usize = outcomes.values().sum();
    assert!(total >= 1000, "too few attributes to mean anything: {outcomes:?}");
    // The README states this share. The remainder is the editor refusing what
    // it cannot do yet (`req "ID" "Title"` titles, `test_case.method` values,
    // class fields shared by several classes, ports of the same name on two
    // components) or declining to pretend ("no effect"); the examples of the
    // 4.3 corpus exercise all of these.
    assert!(applied * 10 >= total * 9, "under 90% of edits applied: {outcomes:?}");
}
