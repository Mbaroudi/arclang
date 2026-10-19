//! Source edits over every example: an accepted edit must still compile,
//! and once confirmed on the compiled model it shows the new value on that
//! element and on no other.

use arclang::compiler::ast::AttributeValue;
use arclang::compiler::elements::{self, ElementGraph, ElementRecord};
use arclang::compiler::source_edit::{confirm_effect, enumeration_values, set_attribute, EditError};
use arclang::compiler::workbench::written_designator;
use arclang::{Compiler, CompilerConfig};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const MARK: &str = "edited by the corpus test";

/// The value to write: another member of the enumeration when the metamodel
/// types the attribute as one, another element of the referenced kind for a
/// reference, another declared kind for a threat; else the marker. A value
/// the model could not hold says nothing about the editor.
fn value_for(graph: &ElementGraph, element: &ElementRecord, key: &str) -> String {
    let current = element.attributes.get(key).and_then(AttributeValue::as_string).unwrap_or("");
    if let Some(members) = enumeration_values(&element.kind, key) {
        return format!("\"{}\"", members.iter().copied().find(|member| !member.eq_ignore_ascii_case(current)).unwrap_or(MARK));
    }
    if let Some(target) = reference_kind(&element.kind, key) {
        // Another element like the one referenced now (same kind, same owner,
        // so the same level), else another of the kind the metamodel names.
        let now = graph.elements.iter().find(|e| e.id == current || e.name == current);
        let like_now = now.and_then(|now| graph.elements.iter().find(|e| e.kind == now.kind && e.owner == now.owner && e.uuid != now.uuid && e.uuid != element.uuid));
        let of_kind = || graph.elements.iter().find(|e| e.kind == target && e.id != current && e.name != current);
        if let Some(other) = like_now.or_else(of_kind) {
            return format!("\"{}\"", other.id);
        }
        // The only candidate: the same element, designated the other way.
        if let Some(same) = graph.elements.iter().find(|e| e.kind == target && (e.id == current || e.name == current)) {
            let other_way = if same.id == current { &same.name } else { &same.id };
            if other_way != current {
                return format!("\"{other_way}\"");
            }
        }
    }
    if element.kind == "Threat" && key == "kind" {
        let kinds: Vec<&str> = graph.elements.iter().filter(|e| e.kind == "Threat").filter_map(|e| e.attributes.get("kind").and_then(AttributeValue::as_string)).collect();
        if let Some(other) = kinds.into_iter().find(|kind| *kind != current) {
            return format!("\"{other}\"");
        }
    }
    format!("\"{MARK}\"")
}

/// The kind a reference attribute points to, from the metamodel.
fn reference_kind(kind: &str, key: &str) -> Option<&'static str> {
    use arclang::compiler::metamodel::{AttrType, Metamodel};
    let metamodel = Metamodel::current();
    let spec = metamodel.kind(kind)?.attributes.iter().find(|attribute| attribute.key == key)?;
    match spec.ty {
        AttrType::Reference(target) => Some(target),
        _ => None,
    }
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
                let value = value_for(&before, element, key);
                // An element that writes no `id` is designated by its name.
                // A name another layer also uses is designated inside its layer.
                // The element as the text knows it, the way `arclang edit` designates it.
                let designator = written_designator(&before, &source, &element.id);
                let attempt = match set_attribute(&source, &designator, key, &value) {
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
    // The README states this: every text attribute of every example edits.
    assert_eq!(applied, total, "not every edit applied: {outcomes:?}");
}
