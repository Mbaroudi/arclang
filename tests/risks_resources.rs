//! Risks, resources and what elicits a requirement.
//!
//! A `risk` is a risk of the project, not a hazard of the product: how
//! likely, how severe, what it bears on, what mitigates it; its level is
//! computed. A `resource` is something functions consume and produce. A
//! capability, a mission or a scenario `elicits` the requirements it gives
//! rise to. All are in the element graph and followed by the impact analysis.

use arclang::compiler::elements;
use arclang::compiler::impact::{self, Analysis};
use arclang::compiler::{CompilationResult, Compiler, CompilerConfig};

const MODEL: &str = r#"
model "Rest" {}

organization "Purchasing" { id: "ORG-BUY" }
resource "28 V bus power" { id: "RES-PWR" capacity: "1.2 kW" }
resource "Waste heat" { id: "RES-HEAT" }

risk "Cell supplier delay" {
    id: "RISK-CELL"
    likelihood: "medium"
    severity: "high"
    bears_on: ["LC-BAT"]
    mitigated_by: ["REQ-SECOND-SOURCE"]
    responsible: "ORG-BUY"
}
risk "Tooling late" { id: "RISK-TOOL" likelihood: "low" severity: "low" bears_on: ["LC-BAT"] status: "accepted" }

system_analysis "SA" {
    requirement "Second source" { id: "REQ-SECOND-SOURCE" description: "Two qualified cell suppliers" }
    requirement "Range" { id: "REQ-RANGE" description: "300 km" }
    capability "Drive a day without charging" { id: "CAP-DAY" }
    function "Convert power" { id: "SF-CONV" consumes: ["RES-PWR"] produces: ["Waste heat"] }
    function "Cool" { id: "SF-COOL" consumes: ["RES-HEAT"] }
}
logical_architecture "LA" {
    component "Battery" { id: "LC-BAT" }
}
trace "LC-BAT" satisfies "REQ-RANGE"
trace "CAP-DAY" elicits "REQ-RANGE"
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

fn relationships(result: &CompilationResult, kind: &str) -> Vec<(String, String)> {
    let graph = elements::build(&result.ast, &result.semantic_model);
    let id_of = |uuid: &str| graph.elements.iter().find(|e| e.uuid == uuid).map(|e| e.id.clone()).unwrap_or_else(|| "?".to_string());
    graph.relationships.iter().filter(|r| r.kind == kind).map(|r| (id_of(&r.source), id_of(&r.target))).collect()
}

fn reached_from(result: &CompilationResult, id: &str) -> Vec<String> {
    let analysis = Analysis::new(result, "ISO26262");
    let seeds = analysis.resolve_all(&[id.to_string()]).expect("the element is found");
    analysis.impacted(&seeds, &impact::Options::default()).into_iter().map(|item| item.id).collect()
}

#[test]
fn a_risk_is_an_element_with_a_computed_level() {
    let result = compile(MODEL);
    let graph = elements::build(&result.ast, &result.semantic_model);
    let risk = |id: &str| graph.elements.iter().find(|e| e.kind == "Risk" && e.id == id).unwrap_or_else(|| panic!("no risk {id}"));

    assert_eq!(risk("RISK-CELL").extra.get("level").and_then(|v| v.as_str()), Some("high"), "medium likelihood, high severity");
    assert_eq!(risk("RISK-TOOL").extra.get("level").and_then(|v| v.as_str()), Some("low"));
    assert_eq!(result.semantic_model.all_elements["RISK-CELL"].element_type, "Risk");
}

#[test]
fn a_risk_bears_on_elements_is_mitigated_and_someone_answers_for_it() {
    let result = compile(MODEL);

    assert_eq!(relationships(&result, "RiskSubject"), [("RISK-CELL".to_string(), "LC-BAT".to_string()), ("RISK-TOOL".to_string(), "LC-BAT".to_string())]);
    assert_eq!(relationships(&result, "RiskMitigation"), [("REQ-SECOND-SOURCE".to_string(), "RISK-CELL".to_string())]);
    assert!(relationships(&result, "Responsibility").contains(&("RISK-CELL".to_string(), "ORG-BUY".to_string())));
    let from_battery = reached_from(&result, "LC-BAT");
    assert!(from_battery.contains(&"RISK-CELL".to_string()), "a change of the battery reaches the risks bearing on it: {from_battery:?}");
}

#[test]
fn a_high_risk_left_open_with_no_mitigation_is_reported() {
    let result = compile(
        r#"logical_architecture "LA" { component "A" { id: "LC-A" } }
risk "Single supplier" { id: "RISK-1" likelihood: "high" severity: "high" bears_on: ["LC-A"] }
risk "Accepted" { id: "RISK-2" likelihood: "high" severity: "high" bears_on: ["LC-A"] status: "accepted" }"#,
    );

    assert!(result.warnings.iter().any(|w| w.contains("RISK-1") && w.contains("high") && w.contains("mitigat")), "{:?}", result.warnings);
    assert!(!result.warnings.iter().any(|w| w.contains("RISK-2")), "an accepted risk is a decision, not an omission: {:?}", result.warnings);
}

#[test]
fn a_risk_names_what_the_model_declares_and_values_of_its_scales() {
    let unknown = error_of(r#"risk "R" { id: "RISK-1" likelihood: "low" severity: "low" bears_on: ["LC-NONE"] }"#);
    assert!(unknown.contains("RISK-1") && unknown.contains("'LC-NONE'"), "got {unknown}");

    let scale = error_of(r#"risk "R" { id: "RISK-1" likelihood: "often" severity: "low" }"#);
    assert!(scale.contains("likelihood") && scale.contains("low, medium, high"), "got {scale}");

    let status = error_of(r#"risk "R" { id: "RISK-1" likelihood: "low" severity: "low" status: "gone" }"#);
    assert!(status.contains("status") && status.contains("open, mitigated, accepted, closed"), "got {status}");
}

#[test]
fn a_risk_that_says_no_likelihood_or_severity_has_no_level() {
    let result = compile(r#"risk "Unsized" { id: "RISK-1" }"#);
    let graph = elements::build(&result.ast, &result.semantic_model);
    let risk = graph.elements.iter().find(|e| e.id == "RISK-1").unwrap();

    assert!(!risk.extra.contains_key("level"), "a level is computed from what is said, never assumed");
    assert!(result.warnings.iter().any(|w| w.contains("RISK-1") && w.contains("likelihood") && w.contains("severity")), "{:?}", result.warnings);
}

#[test]
fn functions_consume_and_produce_resources() {
    let result = compile(MODEL);

    assert_eq!(relationships(&result, "Consumption"), [("SF-CONV".to_string(), "RES-PWR".to_string()), ("SF-COOL".to_string(), "RES-HEAT".to_string())]);
    assert_eq!(relationships(&result, "Production"), [("SF-CONV".to_string(), "RES-HEAT".to_string())]);
    let from_power = reached_from(&result, "RES-PWR");
    assert!(from_power.contains(&"SF-CONV".to_string()), "a change of a resource reaches what consumes it: {from_power:?}");
    let from_converter = reached_from(&result, "SF-CONV");
    assert!(from_converter.contains(&"RES-HEAT".to_string()) && from_converter.contains(&"SF-COOL".to_string()), "what a function produces, and who consumes that: {from_converter:?}");
}

#[test]
fn a_resource_nothing_uses_and_a_name_that_is_no_resource_are_said() {
    let result = compile(
        r#"resource "Hydraulic pressure" { id: "RES-HYD" }
system_analysis "SA" { function "Pump" { id: "SF-PUMP" consumes: ["Fuel"] } }"#,
    );

    assert!(result.warnings.iter().any(|w| w.contains("RES-HYD") && w.contains("no function")), "{:?}", result.warnings);
    assert!(result.warnings.iter().any(|w| w.contains("consumes") && w.contains("'Fuel'") && w.contains("resource \"Fuel\"")), "{:?}", result.warnings);
    assert!(relationships(&result, "Consumption").is_empty());
}

#[test]
fn a_capability_elicits_a_requirement() {
    let result = compile(MODEL);
    let graph = elements::build(&result.ast, &result.semantic_model);
    let trace = graph.relationships.iter().find(|r| r.kind == "Trace" && r.extra.get("traceKind").and_then(|k| k.as_str()) == Some("elicits")).expect("the trace is in the graph");
    let id_of = |uuid: &str| graph.elements.iter().find(|e| e.uuid == uuid).map(|e| e.id.as_str()).unwrap_or("?");

    assert_eq!((id_of(&trace.source), id_of(&trace.target)), ("CAP-DAY", "REQ-RANGE"));
    let from_capability = reached_from(&result, "CAP-DAY");
    assert!(from_capability.contains(&"REQ-RANGE".to_string()), "a change of the capability reaches the requirements it elicits: {from_capability:?}");
    assert!(!result.warnings.iter().any(|w| w.contains("elicits")), "{:?}", result.warnings);
}

#[test]
fn elicits_points_at_a_requirement() {
    let result = compile(
        r#"system_analysis "SA" { capability "C" { id: "CAP-1" } function "F" { id: "SF-1" } }
trace "CAP-1" elicits "SF-1""#,
    );
    assert!(result.warnings.iter().any(|w| w.contains("elicits") && w.contains("requirement")), "{:?}", result.warnings);
}

#[test]
fn risks_and_resources_survive_a_round_trip_through_sysml_v2() {
    use std::process::Command;
    let dir = std::env::temp_dir().join(format!("arclang-risks-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (model, sysml, back) = (dir.join("r.arc"), dir.join("r.sysml"), dir.join("back.arc"));
    std::fs::write(&model, MODEL).unwrap();
    let run = |args: &[&str]| {
        let out = Command::new(env!("CARGO_BIN_EXE_arclang")).args(args).output().expect("arclang runs");
        assert!(out.status.success(), "arclang {args:?}: {}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    };
    run(&["export", model.to_str().unwrap(), "-f", "sys-ml", "-o", sysml.to_str().unwrap()]);
    run(&["import", sysml.to_str().unwrap(), "-f", "sysml", "-o", back.to_str().unwrap()]);

    let result = Compiler::new(CompilerConfig::default()).compile_file(&back).expect("the model read back compiles");
    assert_eq!(result.ast.risks.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), ["RISK-CELL", "RISK-TOOL"]);
    assert_eq!(result.ast.resources.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), ["RES-PWR", "RES-HEAT"]);
    assert_eq!(relationships(&result, "RiskSubject").len(), 2);
    assert_eq!(relationships(&result, "RiskMitigation").len(), 1);
    assert_eq!(relationships(&result, "Consumption").len(), 2);
    assert_eq!(relationships(&result, "Production").len(), 1);
    let graph = elements::build(&result.ast, &result.semantic_model);
    assert!(graph.relationships.iter().any(|r| r.kind == "Trace" && r.extra.get("traceKind").and_then(|k| k.as_str()) == Some("elicits")));
    std::fs::remove_dir_all(&dir).ok();
}
