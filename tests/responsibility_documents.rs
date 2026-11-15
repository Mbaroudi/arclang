//! Who is responsible for an element, and which documents describe it.
//! `organization` and `document` are elements of the model; any element
//! names the organization responsible for it (`responsible:`) and the
//! documents that describe it (`documented_by:`). Both are relationships of
//! the element graph, followed by the impact analysis and shown in tables.

use arclang::compiler::elements;
use arclang::compiler::impact::{self, Analysis};
use arclang::compiler::tables::{self, TableSpec};
use arclang::compiler::{CompilationResult, Compiler, CompilerConfig};

const MODEL: &str = r#"
model "Who" {}

organization "Engineering" { id: "ORG-ENG" }
organization "Powertrain team" { id: "ORG-PT" parent: "ORG-ENG" description: "Owns the traction chain" }
document "Battery interface control document" { id: "DOC-ICD" reference: "ICD-0042" version: "C" uri: "https://docs.example.org/icd-0042" }
document "Safety plan" { id: "DOC-SP" }

system_analysis "SA" {
    requirement "Range" { id: "REQ-RANGE" description: "300 km" responsible: "ORG-PT" documented_by: ["DOC-SP"] }
}
logical_architecture "LA" {
    component "Battery" { id: "LC-BAT" responsible: "Powertrain team" documented_by: ["DOC-ICD", "Safety plan"] }
    component "Cabin" { id: "LC-CAB" }
}
trace "LC-BAT" satisfies "REQ-RANGE"

table "Owners" { kind: elements row_type: "LogicalComponent" columns: ["id", "responsible", "documented_by"] }
"#;

fn compile(source: &str) -> CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_string(source).expect("test model compiles")
}

fn error_of(source: &str) -> String {
    match Compiler::new(CompilerConfig::default()).compile_string(source) {
        Ok(_) => panic!("the model must be refused"),
        Err(error) => error.to_string(),
    }
}

#[test]
fn organizations_and_documents_are_elements_of_the_model() {
    let result = compile(MODEL);

    assert_eq!(result.ast.organizations.iter().map(|o| o.id.as_str()).collect::<Vec<_>>(), ["ORG-ENG", "ORG-PT"]);
    assert_eq!(result.ast.documents.iter().map(|d| d.id.as_str()).collect::<Vec<_>>(), ["DOC-ICD", "DOC-SP"]);
    assert_eq!(result.semantic_model.all_elements["ORG-PT"].element_type, "Organization");
    assert_eq!(result.semantic_model.all_elements["DOC-ICD"].element_type, "Document");

    let graph = elements::build(&result.ast, &result.semantic_model);
    let kinds: Vec<(&str, &str)> = graph.elements.iter().filter(|e| matches!(e.kind, "Organization" | "Document")).map(|e| (e.kind, e.id.as_str())).collect();
    assert_eq!(kinds, [("Organization", "ORG-ENG"), ("Organization", "ORG-PT"), ("Document", "DOC-ICD"), ("Document", "DOC-SP")]);
    assert!(graph.unresolved.is_empty(), "{:?}", graph.unresolved);
}

#[test]
fn responsibility_and_documentation_are_relationships_by_id_or_by_name() {
    let result = compile(MODEL);
    let graph = elements::build(&result.ast, &result.semantic_model);
    let id_of = |uuid: &str| graph.elements.iter().find(|e| e.uuid == uuid).map(|e| e.id.as_str()).unwrap_or("?");
    let of = |kind: &str| -> Vec<(&str, &str)> { graph.relationships.iter().filter(|r| r.kind == kind).map(|r| (id_of(&r.source), id_of(&r.target))).collect() };

    assert_eq!(of("Responsibility"), [("REQ-RANGE", "ORG-PT"), ("LC-BAT", "ORG-PT")]);
    assert_eq!(of("Documentation"), [("REQ-RANGE", "DOC-SP"), ("LC-BAT", "DOC-ICD"), ("LC-BAT", "DOC-SP")]);
}

#[test]
fn a_model_that_names_no_one_compiles_as_before_and_warns_of_nothing() {
    let result = compile(r#"logical_architecture "LA" { component "A" { id: "LC-A" } }"#);

    assert!(result.ast.organizations.is_empty() && result.ast.documents.is_empty());
    assert!(!result.warnings.iter().any(|w| w.contains("responsible") || w.contains("document")), "{:?}", result.warnings);
}

#[test]
fn a_responsible_that_is_no_declared_organization_is_said_not_guessed() {
    let result = compile(r#"logical_architecture "LA" { component "A" { id: "LC-A" responsible: "Safety Team" documented_by: ["Plan 7"] } }"#);

    assert!(
        result.warnings.iter().any(|w| w.contains("LC-A") && w.contains("responsible") && w.contains("'Safety Team'") && w.contains("organization \"Safety Team\"")),
        "the warning says how to declare it: {:?}",
        result.warnings
    );
    assert!(result.warnings.iter().any(|w| w.contains("documented_by") && w.contains("'Plan 7'") && w.contains("document \"Plan 7\"")), "{:?}", result.warnings);
    let graph = elements::build(&result.ast, &result.semantic_model);
    assert!(!graph.relationships.iter().any(|r| matches!(r.kind, "Responsibility" | "Documentation")), "no relationship to nothing");
}

#[test]
fn an_organization_is_part_of_a_declared_organization_and_not_of_itself() {
    let unknown = error_of(r#"organization "Team" { id: "ORG-T" parent: "ORG-NONE" }"#);
    assert!(unknown.contains("ORG-T") && unknown.contains("'ORG-NONE'") && unknown.contains("no organization"), "got {unknown}");

    let cycle = error_of(
        r#"organization "A" { id: "ORG-A" parent: "ORG-B" }
organization "B" { id: "ORG-B" parent: "ORG-A" }"#,
    );
    assert!(cycle.contains("part of itself"), "got {cycle}");
}

#[test]
fn two_organizations_or_documents_of_one_id_are_refused() {
    let twice = error_of(
        r#"document "Plan" { id: "DOC-1" }
document "Other plan" { id: "DOC-1" }"#,
    );
    assert!(twice.contains("DOC-1") && twice.contains("more than once"), "got {twice}");
}

#[test]
fn a_change_of_a_document_reaches_what_it_documents_and_an_organization_what_it_answers_for() {
    let result = compile(MODEL);
    let analysis = Analysis::new(&result, "ISO26262");
    let reached_from = |id: &str| -> Vec<String> {
        let seeds = analysis.resolve_all(&[id.to_string()]).expect("the element is found");
        analysis.impacted(&seeds, &impact::Options::default()).into_iter().map(|item| item.id).collect()
    };

    let from_document = reached_from("DOC-ICD");
    assert!(from_document.contains(&"LC-BAT".to_string()), "what the document describes is impacted: {from_document:?}");
    assert!(!from_document.contains(&"LC-CAB".to_string()), "{from_document:?}");
    let from_organization = reached_from("ORG-PT");
    assert!(from_organization.contains(&"LC-BAT".to_string()) && from_organization.contains(&"REQ-RANGE".to_string()), "{from_organization:?}");
}

#[test]
fn a_table_shows_who_answers_for_each_element() {
    let result = compile(MODEL);
    let table = tables::compute(&result, &TableSpec::of(&result.ast.tables[0]), &tables::Options::default()).expect("the table computes");
    let battery = table.rows.iter().find(|row| row.id == "LC-BAT").expect("the battery row");

    assert_eq!(battery.cells["responsible"].text, "Powertrain team");
    assert!(battery.cells["documented_by"].text.contains("DOC-ICD"), "{}", battery.cells["documented_by"].text);
}

#[test]
fn organizations_and_documents_survive_a_round_trip_through_sysml_v2() {
    use std::process::Command;
    let dir = std::env::temp_dir().join(format!("arclang-respdoc-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (model, sysml, back) = (dir.join("w.arc"), dir.join("w.sysml"), dir.join("back.arc"));
    std::fs::write(&model, MODEL).unwrap();
    let run = |args: &[&str]| {
        let out = Command::new(env!("CARGO_BIN_EXE_arclang")).args(args).output().expect("arclang runs");
        assert!(out.status.success(), "arclang {args:?}: {}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    };
    run(&["export", model.to_str().unwrap(), "-f", "sys-ml", "-o", sysml.to_str().unwrap()]);
    run(&["import", sysml.to_str().unwrap(), "-f", "sysml", "-o", back.to_str().unwrap()]);

    let result = Compiler::new(CompilerConfig::default()).compile_file(&back).expect("the model read back compiles");
    assert_eq!(result.ast.organizations.iter().map(|o| o.id.as_str()).collect::<Vec<_>>(), ["ORG-ENG", "ORG-PT"]);
    assert_eq!(result.ast.documents.iter().map(|d| d.id.as_str()).collect::<Vec<_>>(), ["DOC-ICD", "DOC-SP"]);
    let graph = elements::build(&result.ast, &result.semantic_model);
    assert_eq!(graph.relationships.iter().filter(|r| r.kind == "Responsibility").count(), 2);
    assert_eq!(graph.relationships.iter().filter(|r| r.kind == "Documentation").count(), 3);
    let icd = result.ast.documents.iter().find(|d| d.id == "DOC-ICD").unwrap();
    assert_eq!(icd.attributes.get("reference").and_then(|v| v.as_string()), Some("ICD-0042"));
    std::fs::remove_dir_all(&dir).ok();
}
