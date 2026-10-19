//! ReqIF exchange with requirements tools: what a file written by DOORS,
//! Polarion or ReqIF Studio brings into ArcLang, and what ArcLang gives
//! back, keeping the foreign identity so the tool sees an update.

use arclang::compiler::reqif::{generate_reqif, generate_reqif_with_notes, import_reqif};
use arclang::compiler::{CompilationResult, Compiler, CompilerConfig};
use std::path::Path;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/reqif").join(name)).unwrap()
}

fn compile(source: &str) -> CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_string(source).unwrap_or_else(|e| panic!("{e}\n{source}"))
}

fn imported(name: &str) -> (String, CompilationResult) {
    let source = import_reqif(&fixture(name)).unwrap_or_else(|e| panic!("{e}"));
    let result = compile(&source);
    (source, result)
}

/// The SPEC-OBJECT identifiers of a ReqIF text, in order.
fn object_identifiers(xml: &str) -> Vec<String> {
    xml.split("<SPEC-OBJECT ").skip(1).filter_map(|s| s.split("IDENTIFIER=\"").nth(1)).map(|s| s.split('"').next().unwrap().to_string()).collect()
}

#[test]
fn xhtml_text_is_read_whatever_comes_first_in_the_value() {
    let (_, result) = imported("xhtml_def_first.reqif");
    assert_eq!(result.semantic_model.requirements[0].description, "Brake within 100 ms");
}

#[test]
fn reqif_standard_attribute_names_are_understood() {
    let (source, result) = imported("foreignid.reqif");
    let requirement = &result.semantic_model.requirements[0];
    assert_eq!(requirement.id, "SYS-42");
    assert!(source.contains("req \"SYS-42\" \"Braking\""), "{source}");
}

#[test]
fn enumeration_values_are_imported_by_name() {
    let (_, result) = imported("enum.reqif");
    assert_eq!(result.semantic_model.requirements[0].priority, "High");
}

#[test]
fn escaped_characters_and_backslashes_come_through() {
    let (_, lt) = imported("lt.reqif");
    assert_eq!(lt.semantic_model.requirements[0].description, "A < B and C");
    let (_, bs) = imported("bs.reqif");
    assert_eq!(bs.semantic_model.requirements[0].description, "Path C:\\new \"quoted\" & done");
}

#[test]
fn a_reqif_studio_file_keeps_its_relations_hierarchy_and_specification() {
    let xml = fixture("ReqIF_Studio_01.reqif");
    let (source, result) = imported("ReqIF_Studio_01.reqif");
    assert_eq!(result.semantic_model.requirements.len(), object_identifiers(&xml).len());
    let relations = xml.matches("<SPEC-RELATION ").count();
    let traces = result.ast.traces.iter().filter(|t| t.trace_type == "refines").count();
    assert_eq!(traces, relations, "{source}");
    assert!(source.contains("    parent: \""), "nesting is kept:\n{source}");
    assert!(source.contains("system_analysis \"...Anonymized...\""), "{source}");
    // Texts come in.
    assert!(result.semantic_model.requirements.iter().filter(|r| !r.description.is_empty()).count() > 100);
}

#[test]
fn a_polarion_file_comes_back_with_its_identifiers() {
    let xml = fixture("Polarion_01.reqif");
    let (_, result) = imported("Polarion_01.reqif");
    let back = generate_reqif(&result.semantic_model, &result.ast);
    let original = object_identifiers(&xml);
    let exported = object_identifiers(&back);
    assert_eq!(exported.len(), original.len());
    assert!(original.iter().all(|id| exported.contains(id)), "identifiers lost");
    assert!(back.contains("LONG-NAME=\"Software Requirements Specifications\""), "the specification keeps its name");
}

#[test]
fn a_doors_module_imports_with_its_texts() {
    let (_, result) = imported("Doors_10.reqif");
    assert!(result.semantic_model.requirements.iter().any(|r| !r.description.is_empty()));
}

const MODEL: &str = r#"
system_analysis "Braking needs" {
    req "REQ-1" "Stop" { description: "Stop within 40 m" verification_method: "test" }
    req "REQ-2" "Detect" { description: "Detect a pedestrian at 60 m" priority: "High" }
}
logical_architecture "LA" {
    component "Brake controller" { id: "LC-BRAKE" }
}
trace "REQ-2" refines "REQ-1"
trace "LC-BRAKE" satisfies "REQ-1"
"#;

#[test]
fn an_arclang_model_goes_through_reqif_and_back_unchanged() {
    let result = compile(MODEL);
    let (xml, notes) = generate_reqif_with_notes(&result.semantic_model, &result.ast);
    assert!(xml.contains("LONG-NAME=\"Braking needs\""), "{xml}");
    assert!(!xml.contains("THE-VALUE=\"Medium\""), "no priority the model does not give:\n{xml}");
    assert!(xml.contains("THE-VALUE=\"test\""), "other attributes are exported:\n{xml}");
    assert!(notes.iter().any(|n| n.contains("1 trace") && n.contains("satisfies")), "{notes:?}");
    let source = import_reqif(&xml).unwrap();
    let back = compile(&source);
    assert!(source.contains("system_analysis \"Braking needs\""), "{source}");
    assert!(source.contains("verification_method: \"test\""), "{source}");
    assert!(back.ast.traces.iter().any(|t| t.from == "REQ-2" && t.to == "REQ-1" && t.trace_type == "refines"), "{source}");
    // Identity: the second export is the first.
    assert_eq!(object_identifiers(&generate_reqif(&back.semantic_model, &back.ast)), object_identifiers(&xml));
}
