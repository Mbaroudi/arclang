//! The levels of the V. A verification case says the level it belongs to
//! (`level:` component, subsystem, system or validation); a requirement
//! says the levels at which it is to be verified (`verification_levels:`,
//! its verification plan). A requirement is verified when a case of each
//! planned level verifies it: the gate, the coverage table, the element
//! graph and the requirements view say the same.

use arclang::compiler::diagram::{build_diagrams, ViewKind};
use arclang::compiler::elements;
use arclang::compiler::production_gate::{run_gate, Severity};
use arclang::compiler::tables::{self, TableSpec};
use arclang::compiler::verification::{level_of, planned_levels, requirement_status, Standing};
use arclang::compiler::{CompilationResult, Compiler, CompilerConfig};
use std::collections::BTreeMap;

const MODEL: &str = r#"
model "V" {}

system_analysis "Needs" {
    requirement "Stops in time" { id: "REQ-BOTH" description: "Planned at two levels, verified at both" verification_levels: ["system", "validation"] }
    requirement "Detects" { id: "REQ-HALF" description: "Planned at two levels, verified at one" verification_levels: ["subsystem", "system"] }
    requirement "Warns" { id: "REQ-UNSTATED" description: "Planned at one level, its case states none" verification_levels: ["component"] }
    requirement "Logs" { id: "REQ-FREE" description: "No plan: any case verifies it" }
}

logical_architecture "LA" {
    component "Unit" { id: "LC-1" }
}

trace "LC-1" satisfies "REQ-BOTH"
trace "LC-1" satisfies "REQ-HALF"
trace "LC-1" satisfies "REQ-UNSTATED"
trace "LC-1" satisfies "REQ-FREE"

test_case "Bench" { id: "TC-SYS" verifies: ["REQ-BOTH", "REQ-HALF", "REQ-FREE"] method: "test" level: "System" verdict: "pass" }
test_case "Track day" { id: "TC-VAL" verifies: ["REQ-BOTH"] method: "demonstration" level: "validation" verdict: "pass" }
test_case "Review" { id: "TC-ANY" verifies: ["REQ-UNSTATED"] method: "inspection" verdict: "pass" }

table "Coverage" { kind: requirements_coverage columns: ["id", "levels", "verdict", "status"] }
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
fn a_case_says_its_level_and_a_requirement_its_plan() {
    let result = compile(MODEL);
    let case = |id: &str| result.ast.test_cases.iter().find(|t| t.id == id).unwrap();

    assert_eq!(level_of(case("TC-SYS")), Some("system"), "the level is read whatever its case");
    assert_eq!(level_of(case("TC-VAL")), Some("validation"));
    assert_eq!(level_of(case("TC-ANY")), None);
    assert_eq!(planned_levels(&result.ast, "REQ-BOTH"), ["system", "validation"]);
    assert_eq!(planned_levels(&result.ast, "REQ-HALF"), ["subsystem", "system"]);
    assert!(planned_levels(&result.ast, "REQ-FREE").is_empty());
}

#[test]
fn a_requirement_is_verified_when_each_planned_level_has_a_case() {
    let result = compile(MODEL);
    let status = |id: &str| requirement_status(&result.ast, &result.semantic_model, id);

    assert!(status("REQ-BOTH").unverified_levels.is_empty());
    assert_eq!(status("REQ-BOTH").standing(), Standing::Pass);

    assert_eq!(status("REQ-HALF").unverified_levels, ["subsystem"]);
    assert_eq!(status("REQ-HALF").standing(), Standing::Unverified, "a passing case at another level does not stand for the missing one");
    assert!(!status("REQ-HALF").verified());

    assert_eq!(status("REQ-UNSTATED").unverified_levels, ["component"], "a case that states no level is at none");
    assert_eq!(status("REQ-UNSTATED").standing(), Standing::Unverified);

    assert_eq!(status("REQ-FREE").standing(), Standing::Pass, "without a plan nothing changes");
}

#[test]
fn the_gate_blocks_on_a_planned_level_no_case_verifies_at() {
    let result = compile(MODEL);
    let gate = run_gate(&result.ast, &result.semantic_model, "ISO26262");
    let blockers: Vec<&str> = gate.findings.iter().filter(|f| matches!(f.severity, Severity::Blocker) && f.check == "requirements.verification").map(|f| f.message.as_str()).collect();

    assert!(
        blockers.contains(&"requirement 'REQ-HALF' is to be verified at subsystem level, which no verification case does (test_case level: \"subsystem\")"),
        "got {blockers:#?}"
    );
    assert!(blockers.iter().any(|b| b.starts_with("requirement 'REQ-UNSTATED' is to be verified at component level")), "got {blockers:#?}");
    assert!(!blockers.iter().any(|b| b.contains("REQ-BOTH") || b.contains("REQ-FREE")), "got {blockers:#?}");
    assert_eq!(gate.requirements_verified, 2, "REQ-BOTH and REQ-FREE");
}

#[test]
fn the_coverage_table_shows_each_planned_level_and_where_it_stands() {
    let result = compile(MODEL);
    let table = tables::compute(&result, &TableSpec::of(&result.ast.tables[0]), &tables::Options::default()).expect("the table computes");
    let cells: BTreeMap<&str, (String, String)> = table.rows.iter().map(|row| (row.id.as_str(), (row.cells["levels"].text.clone(), row.status.clone()))).collect();

    assert_eq!(cells["REQ-BOTH"], ("system: pass, validation: pass".to_string(), "ok".to_string()));
    assert_eq!(cells["REQ-HALF"], ("subsystem: no case, system: pass".to_string(), "gap".to_string()));
    assert_eq!(cells["REQ-UNSTATED"], ("component: no case".to_string(), "gap".to_string()));
    assert_eq!(cells["REQ-FREE"], (String::new(), "ok".to_string()));
    let half = table.rows.iter().find(|row| row.id == "REQ-HALF").unwrap();
    assert!(half.issues.iter().any(|issue| issue.contains("subsystem level")), "the row says what is missing: {:?}", half.issues);
}

#[test]
fn the_element_graph_carries_the_level_of_a_case_and_of_each_verification() {
    let result = compile(MODEL);
    let graph = elements::build(&result.ast, &result.semantic_model);

    let case = graph.elements.iter().find(|e| e.id == "TC-SYS").unwrap();
    assert_eq!(case.extra.get("level").and_then(|v| v.as_str()), Some("system"));
    let unstated = graph.elements.iter().find(|e| e.id == "TC-ANY").unwrap();
    assert!(!unstated.extra.contains_key("level"), "no level is invented");
    let verification = graph.relationships.iter().find(|r| r.kind == "Verification" && r.source == case.uuid).unwrap();
    assert_eq!(verification.extra.get("level").and_then(|v| v.as_str()), Some("system"));
}

#[test]
fn the_requirements_view_shows_the_level_of_a_case() {
    let result = compile(MODEL);
    let set = build_diagrams(&result.ast);
    let view = set.diagrams.iter().find(|d| d.kind == ViewKind::Req).expect("the requirements view");
    let case = view.nodes.iter().find(|n| n.id == "TC-VAL").expect("the case is drawn");

    assert!(case.compartment.contains(&"level: validation".to_string()), "got {:?}", case.compartment);
    let unstated = view.nodes.iter().find(|n| n.id == "TC-ANY").unwrap();
    assert!(!unstated.compartment.iter().any(|line| line.starts_with("level:")));
}

#[test]
fn a_level_that_is_none_of_the_v_is_refused() {
    let case = error_of(r#"system_analysis "SA" { requirement "R" { id: "REQ-1" } }
test_case "T" { verifies: ["REQ-1"] level: "unit" }"#);
    assert!(case.contains("level") && case.contains("component, subsystem, system, validation"), "got {case}");

    let plan = error_of(r#"system_analysis "SA" { requirement "R" { id: "REQ-1" verification_levels: ["system", "factory"] } }"#);
    assert!(plan.contains("factory") && plan.contains("component, subsystem, system, validation"), "got {plan}");
}

#[test]
fn a_level_planned_twice_is_refused() {
    let plan = error_of(r#"system_analysis "SA" { requirement "R" { id: "REQ-1" verification_levels: ["system", "System"] } }"#);
    assert!(plan.contains("REQ-1") && plan.contains("system") && plan.contains("twice"), "got {plan}");
}

#[test]
fn a_failing_case_at_a_planned_level_is_a_failure_not_a_gap() {
    let result = compile(
        r#"system_analysis "SA" { requirement "R" { id: "REQ-1" verification_levels: ["system"] } }
test_case "T" { id: "TC-1" verifies: ["REQ-1"] level: "system" verdict: "fail" }"#,
    );
    let status = requirement_status(&result.ast, &result.semantic_model, "REQ-1");

    assert!(status.unverified_levels.is_empty(), "a case verifies at that level; that it fails is said by its verdict");
    assert_eq!(status.standing(), Standing::Fail);
}

#[test]
fn the_levels_survive_a_round_trip_through_sysml_v2() {
    use std::process::Command;
    let dir = std::env::temp_dir().join(format!("arclang-vlevels-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (model, sysml, back) = (dir.join("v.arc"), dir.join("v.sysml"), dir.join("back.arc"));
    std::fs::write(&model, MODEL).unwrap();
    let run = |args: &[&str]| {
        let out = Command::new(env!("CARGO_BIN_EXE_arclang")).args(args).output().expect("arclang runs");
        assert!(out.status.success(), "arclang {args:?}: {}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    };
    run(&["export", model.to_str().unwrap(), "-f", "sys-ml", "-o", sysml.to_str().unwrap()]);
    run(&["import", sysml.to_str().unwrap(), "-f", "sysml", "-o", back.to_str().unwrap()]);

    let result = Compiler::new(CompilerConfig::default()).compile_file(&back).expect("the model read back compiles");
    let case = result.ast.test_cases.iter().find(|t| t.id == "TC-VAL").expect("the case is read back");
    assert_eq!(level_of(case), Some("validation"));
    assert_eq!(planned_levels(&result.ast, "REQ-HALF"), ["subsystem", "system"]);
    assert_eq!(requirement_status(&result.ast, &result.semantic_model, "REQ-HALF").unverified_levels, ["subsystem"]);
    std::fs::remove_dir_all(&dir).ok();
}
