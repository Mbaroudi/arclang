//! Models of several files: a file imported along two paths is read once,
//! and two declarations of one identity name both places, file and line:
//! an error for requirements, a warning for other elements.

use arclang::compiler::{Compiler, CompilerConfig};
use std::path::Path;

fn write(root: &Path, files: &[(&str, &str)]) {
    for (name, text) in files {
        std::fs::write(root.join(name), text).unwrap();
    }
}

fn compile(path: &Path) -> Result<arclang::CompilationResult, String> {
    Compiler::new(CompilerConfig::default()).compile_file(path).map_err(|e| e.to_string())
}

const BASE: &str = "requirements base {\n    req \"REQ-BASE\" \"Base\" { description: \"Shared\" }\n}\n";

#[test]
fn a_file_imported_along_two_paths_is_read_once() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write(root, &[
        ("main.arc", "model Diamond {}\nimport \"left.arc\"\nimport \"right.arc\"\n"),
        ("left.arc", "import \"base.arc\"\nlogical_architecture \"L\" { component \"Left\" { id: \"LC-L\" } }\n"),
        ("right.arc", "import \"base.arc\"\nlogical_architecture \"R\" { component \"Right\" { id: \"LC-R\" } }\n"),
        ("base.arc", BASE),
    ]);
    let result = compile(&root.join("main.arc")).unwrap();
    assert_eq!(result.semantic_model.requirements.len(), 1, "{:?}", result.semantic_model.requirements.iter().map(|r| &r.id).collect::<Vec<_>>());
}

#[test]
fn two_requirements_of_one_id_in_two_files_are_an_error_naming_both() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write(root, &[
        ("main.arc", "model Twice {}\nimport \"base.arc\"\n\nrequirements extra {\n    req \"REQ-BASE\" \"Again\" { description: \"Clash\" }\n}\n"),
        ("base.arc", BASE),
    ]);
    let error = compile(&root.join("main.arc")).err().unwrap_or_default();
    assert!(error.contains("requirement 'REQ-BASE' is declared twice"), "{error}");
    assert!(error.contains("main.arc:5") && error.contains("base.arc:2"), "{error}");
}

#[test]
fn two_requirements_of_one_id_in_one_file_are_an_error() {
    let source = "model Twice {}\nrequirements r {\n    req \"REQ-1\" \"A\" { description: \"a\" }\n    req \"REQ-1\" \"B\" { description: \"b\" }\n}\n";
    let error = Compiler::new(CompilerConfig::default()).compile_string(source).err().map(|e| e.to_string()).unwrap_or_default();
    assert!(error.contains("requirement 'REQ-1' is declared twice: line 3 and line 4"), "{error}");
}

#[test]
fn two_requirements_of_one_written_id_are_an_error_whatever_their_names() {
    // A requirement copied with its `id:` left as it was: one case and one trace would count for both.
    let source = "model Twice {}\nsystem_analysis \"Needs\" {\n    requirement \"AC charge\" { id: \"REQ-CHG\" description: \"a\" }\n    requirement \"DC charge\" { id: \"REQ-CHG\" description: \"b\" }\n}\n";
    let error = Compiler::new(CompilerConfig::default()).compile_string(source).err().map(|e| e.to_string()).unwrap_or_default();
    assert!(error.contains("requirement 'REQ-CHG' is declared twice: line 3 and line 4"), "{error}");
}

/// A duplicate component identity stays a warning (the gate blocks it, and
/// `integration_tests` holds that it compiles); the warning names both places.
#[test]
fn two_components_of_one_id_in_two_files_are_reported_with_both_places() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write(root, &[
        ("main.arc", "model Twice {}\nimport \"other.arc\"\nlogical_architecture \"L\" {\n    component \"A\" { id: \"LC-1\" }\n}\n"),
        ("other.arc", "logical_architecture \"M\" {\n    component \"B\" { id: \"LC-1\" }\n}\n"),
    ]);
    let result = compile(&root.join("main.arc")).unwrap();
    let warning = result.warnings.iter().find(|w| w.starts_with("duplicate element id 'LC-1'")).unwrap_or_else(|| panic!("{:?}", result.warnings));
    assert!(warning.contains("main.arc:4") && warning.contains("other.arc:2"), "{warning}");
}

#[test]
fn the_workbench_reads_a_model_of_several_files() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write(root, &[("main.arc", "model Two {}\nimport \"base.arc\"\n"), ("base.arc", BASE)]);
    let options = arclang::compiler::workbench::SnapshotOptions { gate_standard: "ISO26262", exports: false, explorer_html: false };
    let json: serde_json::Value = serde_json::from_str(&arclang::compiler::workbench::snapshot_file_json(&root.join("main.arc"), &options)).unwrap();
    assert_eq!(json["success"], true, "{json}");
    assert_eq!(json["stats"]["requirements"], 1);
}

/// Everything an imported file declares is part of the model: its rules
/// judge the model, its notes, diagrams and associations are there.
#[test]
fn the_rules_notes_diagrams_and_associations_of_an_imported_file_are_kept() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write(root, &[
        ("main.arc", "model Project {}\nimport \"shared.arc\"\n\nphysical_architecture \"Hardware\" {\n    node \"Battery\" { id: \"PN-BATT\" mass: 450 kg }\n    node \"Cable\" { id: \"PN-CABLE\" }\n}\n"),
        (
            "shared.arc",
            concat!(
                "rule \"Every node declares its mass\" {\n    id: \"RULE-MASS\"\n    kind: \"PhysicalNode\"\n    require: \"has(mass)\"\n    severity: \"blocker\"\n}\n",
                "note \"The cable is supplied by the customer\" { id: \"N-1\" about: [\"PN-CABLE\"] }\n",
                "diagram \"Power\" {\n  view: pab\n  shows: [\"PN-BATT\", \"PN-CABLE\"]\n}\n",
                "class \"Pack\" { id: \"CL-PACK\" cells: \"Cell\" }\nclass \"Cell\" { id: \"CL-CELL\" label: \"String\" }\n",
                "association \"holds\" { id: \"AS-1\" from: \"CL-PACK\" to: \"CL-CELL\" role: \"cells\" kind: \"composition\" multiplicity: \"1..*\" }\n",
            ),
        ),
    ]);
    let result = compile(&root.join("main.arc")).unwrap();
    let ast = &result.ast;
    assert_eq!((ast.rules.len(), ast.notes.len(), ast.diagrams.len(), ast.associations.len()), (1, 1, 1, 1));
    // The imported rule judges the model, and blocks the gate.
    let broken: Vec<&str> = result.warnings.iter().map(String::as_str).filter(|w| w.starts_with("rule '")).collect();
    assert_eq!(broken.len(), 1, "{:?}", result.warnings);
    assert!(broken[0].starts_with("rule 'RULE-MASS' (blocker): 'PN-CABLE'"), "{broken:?}");
    let report = arclang::compiler::production_gate::run_gate(&result.ast, &result.semantic_model, "ISO26262");
    assert!(report.findings.iter().any(|f| f.check == "rule.RULE-MASS"), "{:?}", report.findings.iter().map(|f| &f.check).collect::<Vec<_>>());
}
