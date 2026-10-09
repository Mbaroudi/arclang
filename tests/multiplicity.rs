//! Multiplicity of components and nodes: typed by the metamodel, served
//! parsed by the API graph, written on the usage in the SysML v2 export.

use arclang::compiler::elements;
use arclang::compiler::metamodel::{AttrType, Metamodel};
use arclang::compiler::multiplicity::KINDS;
use arclang::compiler::production_gate::run_gate;
use arclang::compiler::sysmlv2_generator::generate_sysmlv2;
use arclang::{CompilationResult, Compiler, CompilerConfig};
use serde_json::json;

fn compile(source: &str) -> CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_string(source).expect("the model compiles")
}

const MODEL: &str = r#"
model Fleet {}
logical_architecture "Logical" {
    component "Sensor" {
        id: "LC-SNS"
        multiplicity: 2
        component "Lens" { id: "LC-LNS" multiplicity: "1..4" }
    }
    component "Backup" { id: "LC-BKP" multiplicity: "0..1" }
    component "Probe" { id: "LC-PRB" multiplicity: "*" }
    component "Hub" { id: "LC-HUB" }
}
physical_architecture "Hardware" {
    node "ECU" { id: "PN-ECU" multiplicity: "2..*" }
}
"#;

#[test]
fn the_metamodel_types_multiplicity_on_exactly_the_kinds_that_export_it() {
    let metamodel = Metamodel::current();
    let mut typed: Vec<&str> = metamodel
        .kinds
        .iter()
        .filter(|kind| kind.attributes.iter().any(|a| a.key == "multiplicity" && a.ty == AttrType::Multiplicity))
        .map(|kind| kind.name)
        .collect();
    typed.sort();
    let mut expected = KINDS.to_vec();
    expected.sort();
    assert_eq!(typed, expected);
}

#[test]
fn a_valid_multiplicity_raises_nothing_and_is_served_with_its_bounds() {
    let result = compile(MODEL);
    assert!(result.warnings.iter().all(|w| !w.contains("multiplicity")), "{:?}", result.warnings);

    let graph = elements::build(&result.ast, &result.semantic_model);
    let bounds = |id: &str| graph.elements.iter().find(|e| e.id == id).unwrap().extra.get("multiplicity").cloned();
    assert_eq!(bounds("LC-SNS"), Some(json!({ "lower": 2, "upper": 2 })));
    assert_eq!(bounds("LC-LNS"), Some(json!({ "lower": 1, "upper": 4 })));
    assert_eq!(bounds("LC-BKP"), Some(json!({ "lower": 0, "upper": 1 })));
    assert_eq!(bounds("LC-PRB"), Some(json!({ "lower": 0, "upper": null })));
    assert_eq!(bounds("PN-ECU"), Some(json!({ "lower": 2, "upper": null })));
    assert_eq!(bounds("LC-HUB"), None, "no multiplicity stated, none invented");
}

#[test]
fn the_sysml_export_writes_the_multiplicity_on_the_usage_not_as_an_attribute() {
    let result = compile(MODEL);
    let sysml = generate_sysmlv2(&result.semantic_model, &result.ast);

    assert!(sysml.contains("    part p_LC_SNS : Sensor [2];"), "{sysml}");
    assert!(sysml.contains("        part p_LC_LNS : Lens [1..4];"), "{sysml}");
    assert!(sysml.contains("    part p_LC_BKP : Backup [0..1];"), "{sysml}");
    assert!(sysml.contains("    part p_LC_PRB : Probe [*];"), "{sysml}");
    assert!(sysml.contains("    part p_PN_ECU : ECU [2..*];"), "{sysml}");
    assert!(sysml.contains("    part p_LC_HUB : Hub;"), "{sysml}");
    assert!(!sysml.contains("attribute multiplicity"), "{sysml}");
}

#[test]
fn an_invalid_multiplicity_is_a_warning_a_gate_blocker_and_is_not_exported_as_one() {
    let source = "model M {}\nlogical_architecture \"L\" {\n  component \"A\" { id: \"LC-A\" multiplicity: \"4..2\" }\n  component \"B\" { id: \"LC-B\" multiplicity: \"many\" }\n  component \"C\" { id: \"LC-C\" multiplicity: 1.5 }\n}\n";
    let result = compile(source);

    let warnings: Vec<&String> = result.warnings.iter().filter(|w| w.contains("multiplicity")).collect();
    assert_eq!(warnings.len(), 3, "{:?}", result.warnings);
    assert!(warnings.iter().all(|w| w.starts_with("metamodel:")), "{warnings:?}");
    assert!(warnings.iter().any(|w| w.contains("lower bound exceeds the upper bound")), "{warnings:?}");

    let gate = run_gate(&result.ast, &result.semantic_model, "iso26262");
    let blockers = gate.findings.iter().filter(|f| f.message.contains("multiplicity")).count();
    assert_eq!(blockers, 3, "{:?}", gate.findings.iter().map(|f| &f.message).collect::<Vec<_>>());

    // Nothing is guessed: the usage has no multiplicity, the text is kept.
    let sysml = generate_sysmlv2(&result.semantic_model, &result.ast);
    assert!(sysml.contains("    part p_LC_A : A;"), "{sysml}");
    assert!(sysml.contains("attribute multiplicity : String = \"4..2\";"), "{sysml}");
    let graph = elements::build(&result.ast, &result.semantic_model);
    assert!(graph.elements.iter().all(|e| !e.extra.contains_key("multiplicity")));
}

#[test]
fn a_kind_that_does_not_export_a_usage_keeps_multiplicity_as_a_plain_attribute() {
    let result = compile("model M {}\nsystem_analysis \"S\" {\n  function \"F\" { id: \"SF-1\" multiplicity: \"1..*\" }\n}\n");
    assert!(result.warnings.iter().all(|w| !w.contains("multiplicity")), "{:?}", result.warnings);
    let sysml = generate_sysmlv2(&result.semantic_model, &result.ast);
    assert!(sysml.contains("attribute multiplicity : String = \"1..*\";"), "{sysml}");
}
