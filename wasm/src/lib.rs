//! ArcLang in the browser: the SAME compiler core as the CLI, compiled to
//! WebAssembly. The model never leaves the user's machine — compilation,
//! validation, the production gate and the explorer rendering all run
//! client-side. There is no server-side compile endpoint at all.
//!
//! The workbench logic (snapshot of a model, edits of its text) lives in
//! `arclang::compiler::workbench`, shared with `arclang workbench` and
//! `arclang edit`: this crate only binds it.

use arclang::compiler::workbench::{self, compile_text, SnapshotOptions};
use serde_json::json;
use wasm_bindgen::prelude::*;

/// Compile ArcLang source; returns a JSON string:
/// `{success, errors, warnings, stats, explorer_html, diagrams, document,
/// elements, findings, gate, sysmlv2, reqif, c_headers, proto}`.
#[wasm_bindgen]
pub fn compile(source: &str) -> String {
    workbench::snapshot_json(source, &SnapshotOptions::full())
}

/// Format ArcLang source (whitespace only, comments kept); returns a JSON
/// string: `{success, formatted}` or `{success: false, error}`.
#[wasm_bindgen]
pub fn format(source: &str) -> String {
    let output = match arclang::compiler::format::format_source(source) {
        Ok(formatted) => serde_json::json!({ "success": true, "formatted": formatted }),
        Err(error) => serde_json::json!({ "success": false, "error": error }),
    };
    output.to_string()
}

/// Import a SysML v2 text (any model of the textual notation) as ArcLang
/// source, as `arclang import -f sysml` does; returns a JSON string:
/// `{success, source, counts, notes}`, or `{success: false, error, rejected}`
/// with the ArcLang that did not compile when there is one.
#[wasm_bindgen]
pub fn import_sysml(text: &str) -> String {
    let output = match arclang::compiler::sysml_import::import_text(text) {
        Ok(imported) => json!({ "success": true, "source": imported.source, "counts": imported.counts, "notes": imported.notes }),
        Err(failure) => json!({ "success": false, "error": failure.message, "rejected": failure.rejected }),
    };
    output.to_string()
}

/// Version of the underlying compiler crate.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// An Arcadia transition (`oa-to-sa`, `sa-to-la`, `la-to-pa`) applied to
/// the source; returns `{success, source, created, container,
/// container_created}` or `{success: false, error}`. With `plan_only`,
/// returns the plan of every step instead: what each would create.
#[wasm_bindgen]
pub fn transition(source: &str, step: &str, into: &str, plan_only: bool) -> String {
    use arclang::compiler::transition::{self, Step};
    if plan_only {
        return match compile_text(source) {
            Ok(compiled) => {
                let plans: Vec<transition::Plan> = Step::ALL.iter().map(|s| transition::plan(*s, &compiled.ast)).collect();
                json!({ "success": true, "plans": plans }).to_string()
            }
            Err(error) => json!({ "success": false, "error": error }).to_string(),
        };
    }
    let Some(step) = Step::parse(step) else {
        return json!({ "success": false, "error": format!("unknown step '{step}': use oa-to-sa, sa-to-la or la-to-pa") }).to_string();
    };
    let into = if into.trim().is_empty() { None } else { Some(into.trim()) };
    match transition::apply(step, source, into) {
        Ok(done) => json!({
            "success": true,
            "source": done.source,
            "created": done.created,
            "container": done.container,
            "container_created": done.container_created,
        })
        .to_string(),
        Err(error) => json!({ "success": false, "error": error }).to_string(),
    }
}

/// The catalogue of validation rules as JSON: what `findings` name.
#[wasm_bindgen]
pub fn rules() -> String {
    serde_json::to_string(arclang::compiler::rules::catalogue()).unwrap_or_default()
}

/// The language metamodel as JSON (element kinds, typed attributes,
/// enumerations, units, trace rules) — same data as `GET /api/metamodel`.
#[wasm_bindgen]
pub fn metamodel() -> String {
    arclang::compiler::metamodel::Metamodel::current().to_json()
}

/// Apply one edit to ArcLang source; returns a JSON string:
/// `{success, source}` with the edited text, or `{success: false, error}`.
/// The edit is refused unless the edited model compiles and shows exactly
/// what was asked: comments, order and every other declaration are kept.
#[wasm_bindgen]
pub fn edit(source: &str, request: &str) -> String {
    let value = match workbench::edit_json(source, request) {
        Ok(edited) => json!({ "success": true, "source": edited }),
        Err(error) => json!({ "success": false, "error": error }),
    };
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use arclang::compiler::{Compiler, CompilerConfig};

    #[test]
    fn compile_and_edit_go_through_the_shared_workbench() {
        let model = "model M { name: \"M\" }\nlogical_architecture LA {\n    component \"Controller\" { id: \"LC-001\" }\n}\n";
        let output: serde_json::Value = serde_json::from_str(&compile(model)).unwrap();
        assert_eq!(output["success"], true);
        assert!(output["diagrams"]["graphs"].as_array().unwrap().len() >= 1);
        let edited: serde_json::Value = serde_json::from_str(&edit(model, r#"{"op":"set","element":"LC-001","key":"description","value":"\"Controls\""}"#)).unwrap();
        assert_eq!(edited["success"], true, "{edited}");
        assert!(edited["source"].as_str().unwrap().contains("description: \"Controls\""));
        let refused: serde_json::Value = serde_json::from_str(&edit(model, r#"{"op":"nope"}"#)).unwrap();
        assert_eq!(refused["success"], false);
    }

    #[test]
    fn any_sysml_v2_text_comes_back_as_arclang_that_compiles() {
        let text = "package Garage {\n    part def Car { port fuel; }\n    part car : Car;\n    requirement def <'R1'> Range { doc /* The car shall drive 500 km. */ }\n}\n";
        let output: serde_json::Value = serde_json::from_str(&import_sysml(text)).unwrap();
        assert_eq!(output["success"], true, "{output}");
        let source = output["source"].as_str().unwrap();
        assert!(Compiler::new(CompilerConfig::default()).compile_string(source).is_ok(), "{source}");
        assert!(output["counts"].as_object().is_some_and(|c| !c.is_empty()), "{output}");
    }

    #[test]
    fn text_that_is_not_sysml_is_an_error_with_a_message() {
        let output: serde_json::Value = serde_json::from_str(&import_sysml("package { part")).unwrap();
        assert_eq!(output["success"], false);
        assert!(!output["error"].as_str().unwrap().is_empty());
    }
}
