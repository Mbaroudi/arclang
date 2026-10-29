//! Notes on the diagrams: `note "Text" { about: ["LC-A"] }`, declared at
//! the top of a file like traces.
//!
//! A note says something about elements; every element it names must be
//! one the model declares, by id or by name. A note about nothing is drawn
//! nowhere: the compiler says so rather than keep it silently.

use crate::compiler::ast::Model;
use crate::compiler::semantic::{ElementInfo, SemanticModel};

/// The id a note is known by: its written id, else its text.
pub fn id_of(note: &crate::compiler::ast::Note) -> String {
    if note.id.is_empty() { note.name.clone() } else { note.id.clone() }
}

/// Errors for references that name nothing, warnings for notes about nothing;
/// each note joins the elements of the model.
pub fn check(ast: &Model, semantic: &mut SemanticModel) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    for note in &ast.notes {
        if note.about.is_empty() {
            warnings.push(format!("note '{}': it is about no element, so no diagram draws it — add about: [\"<id>\"]", note.name));
        }
        for reference in &note.about {
            let known = semantic.all_elements.contains_key(reference) || semantic.all_elements.values().any(|e| &e.name == reference);
            if !known {
                errors.push(format!("note '{}': about '{reference}' names no element — declare it or fix the reference", note.name));
            }
        }
    }
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    for note in &ast.notes {
        let id = id_of(note);
        semantic.all_elements.entry(id.clone()).or_insert_with(|| ElementInfo::new(id, note.name.clone(), "Note"));
    }
    Ok(warnings)
}

#[cfg(test)]
mod tests {
    use crate::compiler::{Compiler, CompilerConfig};

    const MODEL: &str = "model T { name: \"t\" }\nlogical_architecture \"L\" {\n    component \"Camera\" { id: \"LC-CAM\" }\n}\nnote \"Runs at 30 Hz\" { about: [\"LC-CAM\"] }\nnote \"By name too\" { id: \"N-2\" about: [\"Camera\"] }\n";

    #[test]
    fn a_note_names_declared_elements_and_joins_the_model() {
        let result = Compiler::new(CompilerConfig::default()).compile_string(MODEL).expect("compiles");
        assert_eq!(result.ast.notes.len(), 2);
        assert_eq!(result.ast.notes[0].about, ["LC-CAM"]);
        assert_eq!(result.semantic_model.all_elements["N-2"].element_type, "Note");
        assert!(result.semantic_model.all_elements.contains_key("Runs at 30 Hz"));
        let error = Compiler::new(CompilerConfig::default()).compile_string(&MODEL.replace("[\"Camera\"]", "[\"Nobody\"]")).unwrap_err().to_string();
        assert!(error.contains("about 'Nobody' names no element"), "{error}");
        let alone = Compiler::new(CompilerConfig::default()).compile_string(&MODEL.replace(" about: [\"Camera\"]", "")).expect("compiles");
        assert!(alone.warnings.iter().any(|w| w.contains("about no element")), "{:?}", alone.warnings);
    }
}

#[cfg(test)]
mod edits {
    use crate::compiler::workbench::edit_json;

    const MODEL: &str = "model T { name: \"t\" }\nlogical_architecture \"L\" {\n    component \"Camera\" { id: \"LC-CAM\" }\n}\n";

    #[test]
    fn a_note_is_added_renamed_and_removed_as_any_declaration() {
        let added = edit_json(MODEL, r#"{"op":"add_raw","declaration":"note \"Runs at 30 Hz\" { about: [\"LC-CAM\"] }","top_level":true}"#).expect("added");
        assert!(added.ends_with("note \"Runs at 30 Hz\" { about: [\"LC-CAM\"] }\n"), "{added}");
        let renamed = edit_json(&added, r#"{"op":"rename","element":"Runs at 30 Hz","name":"Runs at 60 Hz"}"#).expect("renamed");
        // Renamed, it keeps its identity, as any element that wrote no id.
        assert!(renamed.contains("note \"Runs at 60 Hz\" { about: [\"LC-CAM\"] id: \"Runs at 30 Hz\" }"), "{renamed}");
        let removed = edit_json(&renamed, r#"{"op":"remove","element":"Runs at 30 Hz"}"#).expect("removed");
        assert_eq!(removed.trim_end(), MODEL.trim_end());
        let refused = edit_json(MODEL, r#"{"op":"add_raw","declaration":"note \"x\" { about: [\"Nobody\"] }","top_level":true}"#).unwrap_err();
        assert!(refused.contains("names no element"), "{refused}");
    }
}
