//! The requirements view: requirements, their derivations, what satisfies
//! them and the test cases that verify them, judged by the compiler.

use arclang::compiler::diagram::{build_diagrams, judge_requirements, Diagram, EdgeKind, NodeKind, ViewKind};
use arclang::compiler::{elements, source_edit, suspect, Compiler, CompilerConfig};

const MODEL: &str = r#"
model "Pack" {}

system_analysis "Needs" {
    requirement "Range" { id: "REQ-RANGE" description: "The car drives 300 km on one charge" }
    requirement "Cell temperature" { id: "REQ-CELL" description: "Cells stay under 55 °C" }
    requirement "Charge" { id: "REQ-CHARGE" description: "Charges in 30 min" parent: "REQ-RANGE" }
    requirement "Cooling" { id: "REQ-COOL" description: "The loop removes 5 kW" }
}

physical_architecture "Hardware" {
    node "Battery" { id: "PN-BATT" mass: 450 kg }
}

test_case "Range drive" { id: "TC-RANGE" verifies: ["REQ-RANGE", "REQ-CELL"] method: "test" verdict: "pass" }
test_case "Bench charge" { id: "TC-CHARGE" verifies: ["Charge"] method: "test" verdict: "fail" }

trace "REQ-CELL" derives "REQ-RANGE"
trace "REQ-COOL" refines "REQ-CELL"
trace "PN-BATT" satisfies "REQ-RANGE"
"#;

fn compile(text: &str) -> arclang::CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_string(text).unwrap_or_else(|e| panic!("{e}"))
}

fn view(result: &arclang::CompilationResult) -> Diagram {
    let mut set = build_diagrams(&result.ast);
    judge_requirements(&mut set, &result.ast, &result.semantic_model);
    set.diagrams.into_iter().find(|d| d.kind == ViewKind::Req).expect("a requirements view")
}

fn edges(diagram: &Diagram) -> Vec<(String, String, String)> {
    let mut out: Vec<_> = diagram.edges.iter().map(|e| (e.source.clone(), e.label.clone(), e.target.clone())).collect();
    out.sort();
    out
}

#[test]
fn the_view_draws_requirements_their_derivations_what_satisfies_and_what_verifies_them() {
    let result = compile(MODEL);
    let diagram = view(&result);
    assert_eq!(diagram.title, "Requirements");
    let kind_of = |id: &str| diagram.nodes.iter().find(|n| n.id == id).map(|n| n.kind);
    for id in ["REQ-RANGE", "REQ-CELL", "REQ-CHARGE", "REQ-COOL"] {
        assert_eq!(kind_of(id), Some(NodeKind::Requirement), "{id}");
    }
    assert_eq!(kind_of("TC-RANGE"), Some(NodeKind::TestCase));
    assert_eq!(kind_of("PN-BATT"), Some(NodeKind::PhysicalNode), "the element that satisfies a requirement");
    let range = diagram.nodes.iter().find(|n| n.id == "REQ-RANGE").unwrap();
    assert_eq!(range.name, "Range");
    assert_eq!(range.compartment, ["id: REQ-RANGE", "The car drives 300 km on one charge"]);
    let uuid = elements::build(&result.ast, &result.semantic_model).elements.iter().find(|e| e.id == "REQ-RANGE").unwrap().uuid.clone();
    assert_eq!(range.uuid, uuid, "the box is the element of the model");

    let drawn = edges(&diagram);
    let has = |s: &str, l: &str, t: &str| drawn.contains(&(s.to_string(), l.to_string(), t.to_string()));
    assert!(has("REQ-CELL", "derives", "REQ-RANGE"), "{drawn:?}");
    assert!(has("REQ-COOL", "refines", "REQ-CELL"), "{drawn:?}");
    assert!(has("PN-BATT", "satisfies", "REQ-RANGE"), "{drawn:?}");
    assert!(has("TC-CHARGE", "verifies", "REQ-CHARGE"), "verifies: by the requirement's name: {drawn:?}");
    assert!(has("REQ-RANGE", "", "REQ-CHARGE"), "parent: is containment: {drawn:?}");
    assert_eq!(diagram.edges.iter().find(|e| e.target == "REQ-CHARGE" && e.label.is_empty()).unwrap().kind, EdgeKind::Containment);
    assert_eq!(diagram.edges.len(), 7, "{drawn:?}");
}

#[test]
fn each_requirement_and_case_carries_its_verdict() {
    let diagram = view(&compile(MODEL));
    let verdict = |id: &str| diagram.nodes.iter().find(|n| n.id == id).and_then(|n| n.properties.get("verdict").cloned());
    assert_eq!(verdict("TC-RANGE").as_deref(), Some("pass"));
    assert_eq!(verdict("TC-CHARGE").as_deref(), Some("fail"));
    assert_eq!(verdict("REQ-RANGE").as_deref(), Some("pass"));
    assert_eq!(verdict("REQ-CHARGE").as_deref(), Some("fail"));
    assert_eq!(verdict("REQ-COOL").as_deref(), Some("unverified"));
    assert_eq!(verdict("PN-BATT"), None);
    let line = diagram.edges.iter().find(|e| e.source == "TC-CHARGE").unwrap();
    assert_eq!(line.properties.get("verdict").map(String::as_str), Some("fail"));
}

#[test]
fn a_suspect_trace_or_verification_is_marked_on_its_line() {
    let result = compile(MODEL);
    let graph = elements::build(&result.ast, &result.semantic_model);
    let trace = result.ast.traces.iter().find(|t| t.trace_type == "satisfies").unwrap();
    let mut text = source_edit::set_trace_attribute(MODEL, "PN-BATT", "satisfies", "REQ-RANGE", "reviewed", &source_edit::string_literal(&suspect::stamp(&graph, trace).unwrap())).unwrap();
    let case = result.ast.test_cases.iter().find(|c| c.id == "TC-RANGE").unwrap();
    let stamps: Vec<String> = ["REQ-RANGE", "REQ-CELL"].iter().map(|r| source_edit::string_literal(&suspect::verification_stamp(&graph, case, r).unwrap())).collect();
    text = source_edit::set_attribute(&text, "TC-RANGE", "reviewed", &format!("[{}]", stamps.join(", "))).unwrap();
    let marked = |diagram: &Diagram| -> Vec<(String, String)> { diagram.edges.iter().filter(|e| e.properties.get("suspect").is_some()).map(|e| (e.source.clone(), e.target.clone())).collect() };
    assert!(marked(&view(&compile(&text))).is_empty(), "just reviewed");

    let changed = view(&compile(&text.replace("drives 300 km", "drives 350 km")));
    let mut found = marked(&changed);
    found.sort();
    assert_eq!(found, [("PN-BATT".to_string(), "REQ-RANGE".to_string()), ("TC-RANGE".to_string(), "REQ-RANGE".to_string())]);
    let line = changed.edges.iter().find(|e| e.source == "PN-BATT").unwrap();
    assert!(line.properties["suspect_reason"].starts_with("suspect: trace 'PN-BATT' satisfies 'REQ-RANGE'"), "{:?}", line.properties);
}

#[test]
fn a_derivation_links_two_requirements() {
    let result = compile(&format!("{MODEL}\ntrace \"PN-BATT\" derives \"REQ-RANGE\"\n"));
    assert!(result.warnings.iter().any(|w| w.contains("`derives` links a requirement to the requirement it is derived from")), "{:?}", result.warnings);
}

#[test]
fn a_model_without_requirements_has_no_requirements_view() {
    let result = compile("model \"M\" {}\nphysical_architecture \"P\" { node \"N\" { id: \"PN-N\" } }\n");
    assert!(build_diagrams(&result.ast).diagrams.iter().all(|d| d.kind != ViewKind::Req));
}
