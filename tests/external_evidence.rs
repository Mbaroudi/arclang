//! A requirement can be verified by evidence obtained outside the compiler
//! (a circuit simulated, a structure solved): a case names `evidence:`, the
//! tool records its verdict beside the model, and the compiler reads it back
//! checked against the model as it is. What changed since, or is not there,
//! or says nothing it was obtained on: the requirement is not verified.

use arclang::compiler::elements;
use arclang::compiler::evidence;
use arclang::compiler::production_gate::run_gate;
use arclang::compiler::suspect::fingerprint;
use arclang::compiler::verification::{requirement_status, Standing};
use arclang::compiler::{Compiler, CompilerConfig};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const MODEL: &str = r#"model Arm { name: "Arm" }
requirements r {
    requirement "Strength" { id: "REQ-ARM" description: "The arm carries 50 N" }
}
physical_architecture "P" {
    node "Arm" { id: "PN-ARM" description: "A printed arm" }
}
trace "PN-ARM" satisfies "REQ-ARM"
test_case "Arm strength" { id: "TC-ARM" verifies: ["REQ-ARM"] method: "analysis" evidence: "engineering" }
"#;

struct Bench {
    dir: tempfile::TempDir,
    model: PathBuf,
}

fn bench() -> Bench {
    let dir = tempfile::tempdir().unwrap();
    let model = dir.path().join("arm.arc");
    std::fs::write(&model, MODEL).unwrap();
    std::fs::write(dir.path().join("bracket.py"), "part = 1\n").unwrap();
    Bench { dir, model }
}

fn compile(model: &Path) -> arclang::CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_file(model).expect("the model compiles")
}

fn standing(model: &Path) -> (Standing, String) {
    let result = compile(model);
    let status = requirement_status(&result.ast, &result.semantic_model, "REQ-ARM");
    let reason = result.semantic_model.simulations.get("TC-ARM").map(|s| s.reason.clone()).unwrap_or_default();
    (status.standing(), reason)
}

fn sha(path: &Path) -> String {
    Sha256::digest(std::fs::read(path).unwrap()).iter().map(|b| format!("{b:02x}")).collect()
}

/// What a tool leaves beside the model: the verdict, on the arm as it is and on its script.
fn record(bench: &Bench, verdict: &str, with_subject: bool, with_input: bool) {
    let result = compile(&bench.model);
    let graph = elements::build(&result.ast, &result.semantic_model);
    let arm = graph.elements.iter().find(|e| e.id == "PN-ARM").unwrap();
    let path = evidence::file(&bench.model, "TC-ARM");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let subjects = if with_subject { vec![json!({ "element": "PN-ARM", "fingerprint": fingerprint(arm) })] } else { vec![] };
    let inputs = if with_input { vec![json!({ "path": "bracket.py", "sha256": sha(&bench.dir.path().join("bracket.py")) })] } else { vec![] };
    let text = json!({ "case": "TC-ARM", "tool": "arclang-server engineering model", "verdict": verdict, "reason": if verdict == "fail" { "stress 54.9 MPa above 50 MPa" } else { "" }, "evidence": "stress 39.0 MPa", "subjects": subjects, "inputs": inputs });
    std::fs::write(path, text.to_string()).unwrap();
}

#[test]
fn a_case_that_names_evidence_nobody_recorded_leaves_its_requirement_not_verified() {
    let bench = bench();
    let (standing, reason) = standing(&bench.model);
    assert_eq!(standing, Standing::Inconclusive);
    assert!(reason.contains("no evidence recorded"), "{reason}");
}

#[test]
fn evidence_that_still_holds_verifies_the_requirement_and_one_that_fails_fails_it() {
    let bench = bench();
    record(&bench, "pass", true, true);
    assert_eq!(standing(&bench.model).0, Standing::Pass);
    record(&bench, "fail", true, true);
    let (standing, _) = standing(&bench.model);
    assert_eq!(standing, Standing::Fail);
    let result = compile(&bench.model);
    assert!(result.warnings.iter().any(|w| w.contains("Arm strength") && w.contains("stress 54.9 MPa")), "{:?}", result.warnings);
}

#[test]
fn the_gate_counts_the_evidence_like_any_other_verdict() {
    let bench = bench();
    record(&bench, "pass", true, true);
    let result = compile(&bench.model);
    let report = run_gate(&result.ast, &result.semantic_model, "ISO26262");
    let text = format!("{report:?}");
    assert!(!text.contains("verification cases fail"), "{text}");
    // Stale evidence is a blocker of the gate, as a stale simulation is.
    std::fs::write(&bench.model, MODEL.replace("A printed arm", "A printed arm, thinner")).unwrap();
    let stale = compile(&bench.model);
    let report = run_gate(&stale.ast, &stale.semantic_model, "ISO26262");
    assert!(format!("{report:?}").contains("TC-ARM") || format!("{report:?}").contains("REQ-ARM"), "{report:?}");
}

#[test]
fn evidence_goes_stale_when_an_element_or_a_file_it_was_obtained_on_changes() {
    let bench = bench();
    record(&bench, "pass", true, true);
    // The element changed.
    std::fs::write(&bench.model, MODEL.replace("A printed arm", "A printed arm, thinner")).unwrap();
    let (state, reason) = standing(&bench.model);
    assert_eq!(state, Standing::Inconclusive);
    assert!(reason.contains("stale") && reason.contains("'PN-ARM' changed"), "{reason}");
    // The element is back as it was: the evidence stands again.
    std::fs::write(&bench.model, MODEL).unwrap();
    assert_eq!(standing(&bench.model).0, Standing::Pass);
    // The file changed.
    std::fs::write(bench.dir.path().join("bracket.py"), "part = 2\n").unwrap();
    let (state, reason) = standing(&bench.model);
    assert_eq!(state, Standing::Inconclusive);
    assert!(reason.contains("bracket.py") && reason.contains("changed"), "{reason}");
    // The file is gone.
    std::fs::remove_file(bench.dir.path().join("bracket.py")).unwrap();
    assert!(standing(&bench.model).1.contains("is not there"));
    // A comment in the model changes nothing it was obtained on.
    std::fs::write(bench.dir.path().join("bracket.py"), "part = 1\n").unwrap();
    std::fs::write(&bench.model, format!("// reviewed\n{MODEL}")).unwrap();
    assert_eq!(standing(&bench.model).0, Standing::Pass);
}

#[test]
fn evidence_that_names_nothing_it_was_obtained_on_is_not_taken() {
    let bench = bench();
    record(&bench, "pass", false, false);
    let (state, reason) = standing(&bench.model);
    assert_eq!(state, Standing::Inconclusive);
    assert!(reason.contains("could never go stale"), "{reason}");
}

#[test]
fn evidence_of_another_case_an_unreadable_file_or_a_made_up_verdict_is_inconclusive() {
    let bench = bench();
    let path = evidence::file(&bench.model, "TC-ARM");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    for (text, why) in [
        (r#"{ "case": "TC-OTHER", "verdict": "pass", "inputs": [] }"#.to_string(), "is that of case 'TC-OTHER'"),
        ("not json".to_string(), "does not read"),
        (json!({ "case": "TC-ARM", "verdict": "great", "inputs": [{ "path": "bracket.py", "sha256": "00" }] }).to_string(), "not a verdict"),
    ] {
        std::fs::write(&path, text).unwrap();
        let (state, reason) = standing(&bench.model);
        assert_eq!(state, Standing::Inconclusive);
        assert!(reason.contains(why), "{reason}");
    }
}

#[test]
fn a_file_outside_the_models_directory_is_refused_as_evidence() {
    let bench = bench();
    let path = evidence::file(&bench.model, "TC-ARM");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, json!({ "case": "TC-ARM", "verdict": "pass", "inputs": [{ "path": "../secret", "sha256": "00" }] }).to_string()).unwrap();
    assert!(standing(&bench.model).1.contains("outside the model's directory"));
}

#[test]
fn a_case_the_compiler_runs_itself_does_not_take_evidence_from_outside() {
    // `asserts:` is computed here: naming evidence as well changes nothing.
    let model = MODEL.replace("evidence: \"engineering\"", "asserts: [\"C1\"] evidence: \"engineering\"") + "constraint \"C1\" { id: \"C1\" assert: 1 <= 2 }\n";
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("m.arc");
    std::fs::write(&path, model).unwrap();
    let result = compile(&path);
    assert!(result.semantic_model.simulations.get("TC-ARM").is_none());
}
