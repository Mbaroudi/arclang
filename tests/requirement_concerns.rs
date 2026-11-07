//! Concerns, framed concerns, a requirement's subject and its constraints
//! (SysML v2 `concern`, `frame`, `subject`, `require`, `assume`).

use arclang::compiler::{Compiler, CompilerConfig};

const MODEL: &str = r#"
model "Car" {}

system_analysis "Needs" {
    concern "Driver safety" { id: "CON-SAFETY" stakeholders: ["Driver"] description: "The driver is not harmed" }
    requirement "Mass" {
        id: "REQ-MASS"
        description: "The car weighs at most 1200 kg"
        frames: ["CON-SAFETY"]
        subject: "LC-CAR"
        require: ["mass <= 1200 kg"]
        assume: ["ambient.T >= -20 °C"]
    }
}

logical_architecture "Logic" {
    component "Car" { id: "LC-CAR" }
}
"#;

fn compile(text: &str) -> Result<arclang::CompilationResult, String> {
    Compiler::new(CompilerConfig::default()).compile_string(text).map_err(|e| e.to_string())
}

#[test]
fn a_requirement_frames_a_concern_of_its_subject_with_constraints() {
    let result = compile(MODEL).unwrap();
    let concern = &result.ast.system_analysis[0].concerns[0];
    assert_eq!(concern.id, "Driver safety");
    assert_eq!(concern.attributes["id"].as_string(), Some("CON-SAFETY"));
}

#[test]
fn a_framed_concern_must_be_a_concern() {
    let error = compile(&MODEL.replace(r#"frames: ["CON-SAFETY"]"#, r#"frames: ["LC-CAR"]"#)).unwrap_err();
    assert!(error.contains("frames 'LC-CAR'"), "{error}");
    let error = compile(&MODEL.replace(r#"frames: ["CON-SAFETY"]"#, r#"frames: ["CON-NONE"]"#)).unwrap_err();
    assert!(error.contains("CON-NONE"), "{error}");
}

#[test]
fn the_subject_is_an_element_of_the_model() {
    let error = compile(&MODEL.replace(r#"subject: "LC-CAR""#, r#"subject: "LC-BUS""#)).unwrap_err();
    assert!(error.contains("subject 'LC-BUS'"), "{error}");
}

#[test]
fn a_constraint_is_a_comparison_with_consistent_units() {
    let error = compile(&MODEL.replace("mass <= 1200 kg", "mass")).unwrap_err();
    assert!(error.contains(r#"require "mass""#) && error.contains("comparison"), "{error}");
    let error = compile(&MODEL.replace("mass <= 1200 kg", "mass <= ")).unwrap_err();
    assert!(error.contains("REQ-MASS"), "{error}");
}

fn import_sysml(text: &str) -> String {
    let built = arclang::compiler::sysml_parser::records(text).unwrap_or_else(|e| panic!("the SysML does not parse: {e}\n{text}"));
    arclang::compiler::sysml_import::import_records(&built.records).unwrap_or_else(|f| panic!("{}", f.message)).source
}

#[test]
fn sysml_v2_says_them_natively_and_reads_them_back() {
    let result = compile(MODEL).unwrap();
    let text = arclang::compiler::sysmlv2_generator::generate_sysmlv2(&result.semantic_model, &result.ast);
    for line in ["concern <'CON-SAFETY'> 'Driver safety' {", "subject = Logic::'LC-CAR';", "frame 'CON-SAFETY';", "require constraint {", "assume constraint {"] {
        assert!(text.contains(line), "{line}\n{text}");
    }
    let back = compile(&import_sysml(&text)).unwrap();
    let requirement = &back.ast.system_analysis[0].requirements[0].attributes;
    assert_eq!(requirement["subject"].as_string(), Some("LC-CAR"));
    assert_eq!(requirement["frames"].display(), "[CON-SAFETY]");
    assert_eq!(requirement["require"].display(), "[mass <= 1200 kg]");
    assert_eq!(requirement["assume"].display(), "[ambient.T >= -20 °C]");
    assert_eq!(back.ast.system_analysis[0].concerns[0].attributes["stakeholders"].display(), "[Driver]");
    assert!(back.ast.constraints.is_empty(), "a requirement's constraint is not a constraint of the model");
}

#[test]
fn a_requirement_written_in_sysml_v2_brings_its_subject_concern_and_constraints() {
    let source = import_sysml(
        r#"package Car {
    private import ISQ::*;
    private import SI::*;
    part def Vehicle { attribute mass : MassValue; }
    part car : Vehicle;
    concern 'Driver safety' { doc /* The driver is not harmed */ }
    requirement <'REQ-MASS'> 'Mass' {
        subject vehicle : Vehicle = car;
        frame 'Driver safety';
        require constraint { vehicle.mass <= 1200 [kg] }
    }
}"#,
    );
    let back = compile(&source).unwrap_or_else(|e| panic!("{e}\n{source}"));
    let requirement = &back.ast.system_analysis[0].requirements[0].attributes;
    assert_eq!(requirement["require"].display(), "[vehicle.mass <= 1200 kg]", "{source}");
    assert_eq!(requirement["frames"].display(), "[Driver safety]");
    assert!(requirement.get("subject").is_some() && requirement.get("vehicle").is_none(), "{source}");
}

#[test]
fn the_element_graph_holds_the_concern_and_the_requirements_frame_and_subject() {
    let result = compile(MODEL).unwrap();
    let graph = arclang::compiler::elements::build(&result.ast, &result.semantic_model);
    let uuid = |id: &str| graph.elements.iter().find(|e| e.id == id).map(|e| e.uuid.clone()).unwrap_or_else(|| panic!("no {id}"));
    assert_eq!(graph.elements.iter().find(|e| e.id == "CON-SAFETY").unwrap().kind, "Concern");
    let has = |kind: &str, from: &str, to: &str| graph.relationships.iter().any(|r| r.kind == kind && r.source == uuid(from) && r.target == uuid(to));
    assert!(has("Frame", "REQ-MASS", "CON-SAFETY"));
    assert!(has("Subject", "REQ-MASS", "LC-CAR"));
    assert!(graph.unresolved.is_empty(), "{:?}", graph.unresolved);
}

#[test]
fn the_requirements_view_draws_the_concern_framed_and_the_subject() {
    use arclang::compiler::diagram::{build_diagrams, NodeKind, ViewKind};
    let result = compile(MODEL).unwrap();
    let set = build_diagrams(&result.ast);
    let view = set.diagrams.iter().find(|d| d.kind == ViewKind::Req).unwrap();
    let node = |id: &str| view.nodes.iter().find(|n| n.id == id).unwrap_or_else(|| panic!("no {id}"));
    let concern = node("CON-SAFETY");
    assert_eq!(concern.kind, NodeKind::Concern);
    assert_eq!(concern.name, "Driver safety");
    assert!(concern.compartment.contains(&"stakeholders: Driver".to_string()), "{:?}", concern.compartment);
    let graph = arclang::compiler::elements::build(&result.ast, &result.semantic_model);
    assert_eq!(concern.uuid, graph.elements.iter().find(|e| e.id == "CON-SAFETY").unwrap().uuid, "the box is the element of the model");
    let requirement = node("REQ-MASS");
    for line in ["subject: LC-CAR", "frames", "CON-SAFETY", "require", "mass <= 1200 kg", "assume", "ambient.T >= -20 °C"] {
        assert!(requirement.compartment.contains(&line.to_string()), "{line}: {:?}", requirement.compartment);
    }
    assert_eq!(node("LC-CAR").compartment, ["requirements", "REQ-MASS"]);
    let has = |s: &str, l: &str, t: &str| view.edges.iter().any(|e| e.source == s && e.label == l && e.target == t);
    assert!(has("REQ-MASS", "frames", "CON-SAFETY"));
    assert!(has("REQ-MASS", "subject", "LC-CAR"));
}

#[test]
fn an_architecture_view_lists_the_requirements_about_an_element() {
    use arclang::compiler::diagram::{build_diagrams, ViewKind};
    let result = compile(MODEL).unwrap();
    let set = build_diagrams(&result.ast);
    let logical = set.diagrams.iter().find(|d| d.kind == ViewKind::Lab).unwrap();
    let car = logical.nodes.iter().find(|n| n.id == "LC-CAR").unwrap();
    assert_eq!(car.compartment, ["requirements", "REQ-MASS"]);
    assert_eq!(car.properties["requirements"], "REQ-MASS");
}
