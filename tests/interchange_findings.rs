//! What a model says is what leaves it, and what comes back: findings of the
//! review of 2026-11-22 on the exchange formats.

use arclang::compiler::{CompilationResult, Compiler, CompilerConfig};

fn compile(source: &str) -> CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_string(source).unwrap_or_else(|e| panic!("{e}\n{source}"))
}

fn through_sysml(source: &str) -> (String, String) {
    let result = compile(source);
    let text = arclang::compiler::sysmlv2_generator::generate_sysmlv2(&result.semantic_model, &result.ast);
    let back = arclang::compiler::sysml_import::import_text(&text).unwrap_or_else(|f| panic!("{}\n{text}", f.message)).source;
    (text, back)
}

#[test]
fn the_json_of_a_model_is_the_same_bytes_on_every_run() {
    let source = "model \"M\" { id: \"M\" }\nlogical_architecture \"LA\" {\n  component \"A\" { id: \"LC-A\" description: \"a\" mass: \"3 kg\" power: \"5 W\" owner: \"x\" status: \"draft\" zeta: \"1\" alpha: \"2\" }\n}\n";
    let first = compile(source).ast.to_json().unwrap();
    for _ in 0..8 {
        assert_eq!(compile(source).ast.to_json().unwrap(), first);
    }
    // The keys of an object are in alphabetical order.
    let (alpha, zeta) = (first.find("\"alpha\"").unwrap(), first.find("\"zeta\"").unwrap());
    assert!(alpha < zeta, "{first}");
}

#[test]
fn the_description_of_a_type_goes_through_sysml_v2_and_back() {
    let (text, back) = through_sysml("model \"M\" { id: \"M\" }\ntype \"Speed\" { description: \"Vehicle speed over ground\" required: [\"mass\"] }\nlogical_architecture \"LA\" {\n  component \"A\" { id: \"LC-A\" is: \"Speed\" mass: \"3 kg\" }\n}\n");
    assert!(text.contains("doc /* Vehicle speed over ground */"), "{text}");
    assert!(back.contains("description: \"Vehicle speed over ground\""), "{back}");
    assert!(back.contains("required: [\"mass\"]"), "{back}");
}

#[test]
fn a_function_made_of_typed_functions_comes_back_from_sysml_v2_with_them() {
    let (text, back) = through_sysml(
        "model \"M\" { id: \"M\" }\ntype \"Measure\" { description: \"Reads\" }\ntype \"Compute\" { description: \"Computes\" }\nlogical_architecture \"LA\" {\n  component \"Controller\" {\n    id: \"LC-CTL\"\n    function \"Control speed\" {\n      id: \"LF-CTL\"\n      function \"Measure speed\" { id: \"LF-MEAS\" is: \"Measure\" }\n      function \"Compute torque\" { id: \"LF-COMP\" is: \"Compute\" }\n    }\n  }\n}\n",
    );
    assert!(back.contains("id: \"LF-CTL\""), "the function that holds them is kept:\n{back}\n{text}");
    let at = |id: &str| back.find(&format!("id: \"{id}\"")).unwrap_or_else(|| panic!("{id}\n{back}"));
    assert!(at("LF-CTL") < at("LF-MEAS") && at("LF-CTL") < at("LF-COMP"), "{back}");
    assert!(!back.contains("functional_chain"), "{back}");
}

#[test]
fn two_names_that_become_one_identifier_are_told_not_generated() {
    let result = compile("model \"M\" { id: \"M\" }\nclass \"Wheel Speed\" {\n  id: \"C1\"\n  frontLeft: \"Real\"\n  front_left: \"Real\"\n}\nclass \"Wheel_Speed\" { id: \"C2\" }\n");
    let c = arclang::compiler::c_header_generator::clashes(&result.ast);
    assert!(c.iter().any(|l| l.contains("class 'Wheel Speed'") && l.contains("class 'Wheel_Speed'") && l.contains("`wheel_speed_t`")), "{c:?}");
    assert!(c.iter().any(|l| l.contains("field 'frontLeft'") && l.contains("field 'front_left'") && l.contains("`front_left`") && l.contains("in class 'Wheel Speed'")), "{c:?}");
    let p = arclang::compiler::proto_generator::clashes(&result.semantic_model, &result.ast);
    assert!(p.iter().any(|l| l.contains("`WheelSpeed`")), "{p:?}");
    // A model without such names has none.
    let fine = compile("model \"M\" { id: \"M\" }\nclass \"Wheel Speed\" {\n  id: \"C1\"\n  frontLeft: \"Real\"\n}\n");
    assert!(arclang::compiler::c_header_generator::clashes(&fine.ast).is_empty());
    assert!(arclang::compiler::proto_generator::clashes(&fine.semantic_model, &fine.ast).is_empty());
}

#[test]
fn an_id_capella_cannot_hold_is_written_as_one_it_can_everywhere() {
    let result = compile("model \"M\" { id: \"M\" }\nlogical_architecture \"LA\" {\n  component \"LC 1\" { id: \"LC 1\" description: \"see LC 1\" }\n  component \"B\" { id: \"LC#2\" }\n  capability \"Cap\" { id: \"CAP-1\" involves: [\"LC 1\", \"LC#2\"] }\n}\n");
    let exported = arclang::compiler::capella_export::export(&result.ast, "m");
    let capella = &exported.capella;
    for line in capella.lines() {
        for reference in line.split('"').filter(|v| v.starts_with('#')) {
            assert!(!reference[1..].contains('#') && reference.split(' ').all(|r| r.starts_with('#')), "a reference Capella cannot follow: {line}");
        }
    }
    assert!(!capella.contains("id=\"LC 1\"") && !capella.contains("id=\"LC#2\""), "{capella}");
    // Its name and its description are what was written.
    assert!(capella.contains("name=\"LC 1\"") && capella.contains("description=\"see LC 1\""), "{capella}");
    assert_eq!(capella.matches("CapabilityRealizationInvolvement\"").count(), 2, "{capella}");
    assert_eq!(exported.notes.iter().filter(|n| n.contains("a Capella id holds no space")).count(), 2, "{:?}", exported.notes);
    // A model whose ids are fine is written as before, without a note.
    let fine = compile("model \"M\" { id: \"M\" }\nlogical_architecture \"LA\" {\n  component \"A\" { id: \"LC-1\" }\n}\n");
    let exported = arclang::compiler::capella_export::export(&fine.ast, "m");
    assert!(exported.capella.contains("id=\"LC-1\"") && exported.notes.is_empty(), "{:?}", exported.notes);
}
