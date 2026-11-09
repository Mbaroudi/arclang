//! What is written must be read: an attribute spelled almost as one the
//! compiler knows is reported (the element is free to carry attributes of
//! its own, so the compiler cannot refuse it), and an attribute written
//! twice in one block is an error instead of the last one winning.

use arclang::compiler::{Compiler, CompilerConfig};

const MODEL: &str = r#"model "Keys" {}

system_analysis "Needs" {
    requirement "Range" { id: "REQ-RANGE" description: "Drives 300 km" }
}

physical_architecture "Hardware" {
    node "Battery" { id: "PN-BATT" mass: 450 kg cell_count: 96 }
}

trace "PN-BATT" satisfies "REQ-RANGE"

test_case "Range test" { id: "TC-RANGE" verifies: ["REQ-RANGE"] method: "test" verdict: "pass" }
"#;

fn compile(text: &str) -> Result<arclang::CompilationResult, String> {
    Compiler::new(CompilerConfig::default()).compile_string(text).map_err(|e| e.to_string())
}

fn near_misses(text: &str) -> Vec<String> {
    compile(text).unwrap().warnings.into_iter().filter(|w| w.contains("did you mean")).collect()
}

#[test]
fn attributes_of_the_model_s_own_are_not_reported() {
    // `mass` and `cell_count` are the author's: nothing to say.
    assert_eq!(near_misses(MODEL), Vec::<String>::new());
}

#[test]
fn a_verdict_misspelled_is_reported_instead_of_being_read_as_nothing() {
    let text = MODEL.replace("verdict: \"pass\"", "verdit: \"fail\"");
    assert_eq!(near_misses(&text), ["metamodel: TestCase 'Range test'.verdit: not an attribute the compiler reads: did you mean 'verdict'?"]);
}

#[test]
fn a_letter_swapped_dropped_or_added_is_a_near_miss() {
    for (written, meant) in [("verifeis: [\"REQ-RANGE\"]", "verifies"), ("mehtod: \"test\"", "method"), ("expect: [\"x > 1\"]", "expects"), ("asserts_: [\"C-1\"]", "asserts")] {
        let text = MODEL.replace("method: \"test\"", &format!("method: \"test\" {written}"));
        let key = written.split(':').next().unwrap();
        assert_eq!(near_misses(&text), [format!("metamodel: TestCase 'Range test'.{key}: not an attribute the compiler reads: did you mean '{meant}'?")], "{written}");
    }
    let text = MODEL.replace("description: \"Drives 300 km\"", "description: \"Drives 300 km\" applys_to: [\"CFG-A\"]");
    assert_eq!(near_misses(&text), ["metamodel: Requirement 'Range'.applys_to: not an attribute the compiler reads: did you mean 'applies_to'?"]);
}

#[test]
fn an_attribute_written_twice_in_one_block_is_an_error() {
    let text = MODEL.replace("verdict: \"pass\"", "verdict: \"fail\" verdict: \"pass\"");
    let error = compile(&text).err().unwrap_or_default();
    assert!(error.contains("'verdict' is written twice"), "{error}");
    // In a nested block too.
    let text = MODEL.replace("mass: 450 kg", "limits: { low: 1 low: 2 }");
    let error = compile(&text).err().unwrap_or_default();
    assert!(error.contains("'low' is written twice"), "{error}");
}

#[test]
fn the_gate_of_a_fault_tree_is_one_of_its_two_values() {
    // The metamodel says so (editors offer the two values), and the compiler refuses another.
    let metamodel = arclang::compiler::metamodel::Metamodel::current();
    assert_eq!(metamodel.enumeration("FaultGate").map(|e| e.values), Some(&["or", "and"][..]));
    let hazard = |gate: &str| format!("{MODEL}\nsafety_analysis {{\n    hazard \"Loss of drive\" {{ id: \"HAZ-1\" gate: \"{gate}\" mitigated_by: [\"REQ-RANGE\"] }}\n}}\n");
    let error = compile(&hazard("xor")).err().unwrap_or_default();
    assert!(error.contains("gate 'xor' is not or, and"), "{error}");
    for valid in ["or", "and", "AND"] {
        let about_the_gate: Vec<String> = compile(&hazard(valid)).unwrap().warnings.into_iter().filter(|w| w.contains("'Loss of drive'.gate")).collect();
        assert_eq!(about_the_gate, Vec::<String>::new(), "{valid}");
    }
}
