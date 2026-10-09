//! The percent sign is a unit like any other: `load: 40 %`.

use arclang::compiler::ast::AttributeValue;
use arclang::compiler::elements;
use arclang::compiler::format::format_source;
use arclang::compiler::source_edit::{confirm_effect, set_attribute};
use arclang::compiler::sysmlv2_generator::generate_sysmlv2;
use arclang::{CompilationResult, Compiler, CompilerConfig};

const MODEL: &str = r#"model Bus {}
physical_architecture "Hardware" {
    node "Gateway" {
        id: "PN-GW"
        cpu_load: 40 %
        margin: 12.5%
        legacy: 30 percent
    }
}
constraint "Headroom" {
    id: "CST-1"
    assert: "PN-GW".cpu_load + "PN-GW".margin <= 60 %
}
"#;

fn compile(source: &str) -> CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_string(source).expect("the model compiles")
}

fn ratio(result: &CompilationResult, key: &str) -> (f64, f64) {
    let graph = elements::build(&result.ast, &result.semantic_model);
    let node = graph.elements.iter().find(|e| e.id == "PN-GW").unwrap();
    match &node.attributes[key] {
        AttributeValue::Quantity(quantity) => (quantity.value, quantity.canonical()),
        other => panic!("{key} is not a quantity: {other:?}"),
    }
}

#[test]
fn a_percentage_is_a_ratio_quantity_however_it_is_spelled() {
    let result = compile(MODEL);

    assert_eq!(ratio(&result, "cpu_load").0, 40.0);
    assert_eq!(ratio(&result, "margin").0, 12.5);
    // `%` and `percent` are the same unit: same canonical value for the same number.
    let spelled = compile(&MODEL.replace("cpu_load: 40 %", "cpu_load: 40 percent"));
    assert_eq!(ratio(&result, "cpu_load"), ratio(&spelled, "cpu_load"));
}

#[test]
fn percentages_are_usable_in_constraints_and_exported_with_the_sysml_unit() {
    let result = compile(MODEL);

    let verdict = &result.semantic_model.constraints[0];
    assert!(verdict.satisfied, "52.5 % <= 60 %: {verdict:?}");
    let sysml = generate_sysmlv2(&result.semantic_model, &result.ast);
    assert!(sysml.contains("attribute cpu_load : DimensionOneValue = 40 ['%'];"), "{sysml}");
    assert!(sysml.contains("<= 60 ['%']"), "{sysml}");
}

#[test]
fn the_formatter_and_the_editor_handle_the_percent_sign() {
    let formatted = format_source(MODEL).unwrap();
    assert!(formatted.contains("        cpu_load: 40 %\n        margin: 12.5%\n"), "{formatted}");

    let edited = set_attribute(MODEL, "PN-GW", "cpu_load", "55 %").unwrap();
    assert_eq!(edited, MODEL.replace("cpu_load: 40 %", "cpu_load: 55 %"));
    let result = compile(&edited);
    let graph = elements::build(&result.ast, &result.semantic_model);
    assert_eq!(confirm_effect(&graph, "PN-GW", "cpu_load", "55 %"), Ok(()));
}

#[test]
fn a_percent_sign_without_a_number_is_still_an_error() {
    let outcome = Compiler::new(CompilerConfig::default()).compile_string("model M {}\nphysical_architecture \"H\" {\n  node \"N\" { load: % }\n}\n");
    assert!(outcome.is_err());
}
