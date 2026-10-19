//! Typed allocations between levels: a `type` declaring `from_kind` and
//! `to_kind` is an allocation definition (SysML v2 `allocation def`); an
//! `allocates` trace `is:` it. The compiler checks the kinds of its ends and
//! the trace carries the definition's attributes.

use arclang::compiler::{Compiler, CompilerConfig};

const MODEL: &str = r#"
type "Software deployment" {
    from_kind: "PhysicalNode"
    to_kind: "LogicalComponent"
    partition: "ASIL-D partition"
}

logical_architecture "Logical" {
    component "Sensor fusion" { id: "LC-FUSION" }
}

physical_architecture "Physical" {
    node "ADAS ECU" { id: "PN-ADAS" }
}

trace "PN-ADAS" allocates "LC-FUSION" { is: "Software deployment" }
"#;

fn compile(source: &str) -> Result<arclang::compiler::CompilationResult, String> {
    Compiler::new(CompilerConfig::default()).compile_string(source).map_err(|e| e.to_string())
}

#[test]
fn a_typed_allocation_carries_its_definition() {
    let result = compile(MODEL).unwrap();
    let trace = &result.ast.traces[0];
    assert_eq!(trace.attributes.get("partition").and_then(|v| v.as_string()), Some("ASIL-D partition"));
    assert!(result.warnings.iter().all(|w| !w.contains("allocation")), "{:?}", result.warnings);
}

#[test]
fn an_allocation_whose_ends_are_not_of_its_kinds_is_an_error() {
    let error = compile(&MODEL.replace("trace \"PN-ADAS\" allocates \"LC-FUSION\"", "trace \"LC-FUSION\" allocates \"PN-ADAS\"")).err().unwrap_or_default();
    assert!(error.contains("allocation 'LC-FUSION' -> 'PN-ADAS' is a 'Software deployment': it goes from a PhysicalNode, not a LogicalComponent"), "{error}");
}

#[test]
fn a_typed_allocation_goes_through_sysml_v2_and_back() {
    let result = compile(MODEL).unwrap();
    let text = arclang::compiler::sysmlv2_generator::generate_sysmlv2(&result.semantic_model, &result.ast);
    assert!(text.contains("allocation def 'Software deployment'"), "{text}");
    assert!(text.contains(": Types::'Software deployment' allocate"), "{text}");
    let back = arclang::compiler::sysml_import::import_text(&text).unwrap_or_else(|f| panic!("{}\n{text}", f.message)).source;
    assert!(back.contains("type \"Software deployment\""), "{back}");
    assert!(back.contains("from_kind: \"PhysicalNode\""), "{back}");
    assert!(back.contains("trace \"PN-ADAS\" allocates \"LC-FUSION\" { is: \"Software deployment\" }"), "{back}");
}
