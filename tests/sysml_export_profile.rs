//! What the SysML v2 export writes of the Arcadia profile beyond the golden
//! file: the root package is the model, traces carry their metadata, and an
//! attribute is written once.

use arclang::compiler::{sysmlv2_generator, Compiler, CompilerConfig};
use std::path::Path;

fn exported(source: &str) -> String {
    let result = Compiler::new(CompilerConfig::default()).compile_string(source).expect("the model compiles");
    sysmlv2_generator::generate_sysmlv2(&result.semantic_model, &result.ast)
}

fn flagship() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/complete_emergency_braking_simple.arc");
    exported(&std::fs::read_to_string(path).unwrap())
}

#[test]
fn the_root_package_has_the_header_id_as_short_name() {
    assert!(flagship().contains("package <EmergencyBrakingSystem> 'Automotive Emergency Braking System (AEB)' {"));
}

#[test]
fn a_model_without_header_or_name_is_the_package_model() {
    let text = exported("system_analysis \"SA\" {\n  requirement \"REQ-1\" { description: \"x\" }\n}\n");
    assert!(text.contains("\npackage Model {"), "{text}");
}

#[test]
fn a_trace_is_a_dependency_with_the_trace_metadata() {
    let text = flagship();
    assert!(text.contains("#Trace dependency realizes from "), "trace realizes");
    assert!(!text.lines().any(|l| l.trim_start().starts_with("dependency realizes from 'SF-")), "a realizes trace is not an attribute");
}

#[test]
fn a_test_method_is_written_once() {
    let text = flagship();
    let block = &text[text.find("verification <'TC-001'>").unwrap()..];
    let block = &block[..block.find("objective").unwrap()];
    assert_eq!(block.matches("attribute method").count(), 1, "{block}");
}

#[test]
fn a_class_association_is_a_connection_def_with_its_role() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/capella/in_flight_entertainment.arc");
    let text = exported(&std::fs::read_to_string(path).unwrap());
    let at = text.find("#Association connection def <'0945715c-8c26-495b-ab84-8dd5f86ce7cb'> has {").expect("association exported");
    let block = &text[at..at + text[at..].find("\n    }").unwrap()];
    for expected in ["end source : '8a3c3a57", "end target : '4bb53659", "attribute role : String = \"field_3\";", "attribute kind : String = \"composition\";", "attribute multiplicity : String = \"1..*\";"] {
        assert!(block.contains(expected), "`{expected}` in {block}");
    }
}

#[test]
fn a_trace_can_name_a_scenario_by_its_id() {
    let source = "scenario \"FS\" {\n  id: \"SC-1\"\n}\nscenario \"ES\" {\n  id: \"SC-2\"\n}\ntrace \"SC-2\" realizes \"SC-1\"\n";
    Compiler::new(CompilerConfig::default()).compile_string(source).expect("a scenario is known by its id");
}

fn ife() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/capella/in_flight_entertainment.arc");
    exported(&std::fs::read_to_string(path).unwrap())
}

#[test]
fn a_chain_with_sequence_links_has_its_control_nodes_and_successions() {
    let text = ife();
    for expected in [
        "fork <'f99cae18-4781-46b0-b6c1-307c24b0679e'> 'AND 1';",
        "join <'800e54fc-f4d1-438d-ac58-2321d92c7b17'> 'AND 2';",
        "#Iterate merge <'3ed2161f-c41d-4298-8f35-3cf3391e4461'> 'ITERATE 1';",
        "succession <'8784bebf-d0a9-4952-bc51-00085d10d15d'> first step2 then 'f99cae18-4781-46b0-b6c1-307c24b0679e';",
        "succession <'e88f1849-7afe-41f1-b4aa-da09f06d8d2a'> first 'f99cae18-4781-46b0-b6c1-307c24b0679e' then step5 {\n                attribute condition : String = \"Condition A\";",
    ] {
        assert!(text.contains(expected), "`{expected}`");
    }
    assert!(!text.contains("then action step2 :> '5158c762-8f3c-4947-98f4-4cf085492bf5'.'95ea7294-402d-4783-b6e5-2e405c9f4ccb'.'fd6de739"), "the links order the steps");
}

#[test]
fn a_state_fragment_is_an_invariant_on_its_lifeline() {
    let text = ife();
    let expected = "#Invariant state 'f91174fd-7900-492c-ae4d-f62f7a4f31e3' :> '3e1b1f3a-e2b0-48fa-b669-7f61f0369e14'.'39eb29ec-2085-47fe-b3ea-55027fc82faf' {\n                    attribute start : Integer = 0;\n                }";
    assert!(text.contains(expected), "an instant on its state");
    assert!(!text.contains("attribute target : String"), "every fragment names a declared state or function");
}
