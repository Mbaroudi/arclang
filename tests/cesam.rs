//! The CESAM framework view of a model (`compiler::cesam`, table kind
//! `cesam_grid`): every element placed in its vision x aspect cell, the
//! refinements (requirement blocks, machine kinds and names, scenario
//! participants), the maturity of each view, and the CESAM column of the
//! metamodel.

use arclang::compiler::metamodel::Metamodel;
use arclang::compiler::tables::{self, Options, Table, TableSpec};
use arclang::compiler::{Compiler, CompilerConfig};

const MODEL: &str = r#"
model Door {}

requirements stakeholder {
    req "STK-1" "Open the door in one gesture" { priority: High }
}
requirements system {
    req "SYS-1" "Unlock within 200 ms" { priority: High }
}
requirements technical {
    req "TEC-1" "Latch rated 10000 cycles" { priority: Medium }
}

operational_analysis "Door Operations" {
    actor "User" { description: "Opens the door" }
}

system_analysis "Door System" {
    mission "Give access" { id: "MIS-1" }
    capability "Open" { id: "CAP-1" mission: "MIS-1" }
    function "Unlock" { id: "SF-1" }
    function "Release latch" { id: "SF-2" }
}

logical_architecture "Door Logical" {
    component "Controller" { id: "LC-1" function "Decide" { id: "LF-1" realizes: "SF-1" } }
    component "Latch" { id: "LC-2" function "Move" { id: "LF-2" realizes: "SF-2" } }
}

state_machine "Door lifecycle" {
    initial: "Production"
    state Production {}
    state Operation {}
    transition Production -> Operation { trigger: "delivered" }
}

state_machine "Door modes" {
    initial: "Locked"
    mode Locked {}
    mode Unlocked {}
    transition Locked -> Unlocked { trigger: "SF-1" }
}

scenario "Unlock sequence" {
    participants: ["LC-1", "LC-2"]
    message "LC-1" -> "LC-2" "release"
}
"#;

fn grid(source: &str) -> Table {
    let result = Compiler::new(CompilerConfig::default()).compile_string(source).expect("compiles");
    let spec = TableSpec { kind: "cesam_grid".to_string(), ..TableSpec::default() };
    tables::compute(&result, &spec, &Options::default()).expect("cesam_grid computes")
}

fn names(table: &Table, vision: &str, aspect: &str) -> Vec<String> {
    let row = table.rows.iter().find(|r| r.id == vision).unwrap_or_else(|| panic!("no row {vision}"));
    row.cells.get(aspect).map(|c| c.links.iter().map(|l| l.name.clone()).collect()).unwrap_or_default()
}

#[test]
fn the_grid_has_one_row_per_vision_and_one_column_per_aspect() {
    let table = grid(MODEL);
    let rows: Vec<&str> = table.rows.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(rows, ["operational", "functional", "constructional"]);
    let columns: Vec<&str> = table.columns.iter().map(|c| c.key.as_str()).collect();
    assert_eq!(columns, ["vision", "requirements", "states", "structure", "interactions", "scenarios", "interfaces"]);
    assert!(tables::KINDS.contains(&"cesam_grid"));
}

#[test]
fn requirements_take_the_vision_of_their_block() {
    let table = grid(MODEL);
    assert!(names(&table, "operational", "requirements").iter().any(|n| n.contains("STK-1") || n.contains("one gesture")));
    assert!(names(&table, "functional", "requirements").iter().any(|n| n.contains("SYS-1") || n.contains("200 ms")));
    assert!(names(&table, "constructional", "requirements").iter().any(|n| n.contains("TEC-1") || n.contains("10000")));
}

#[test]
fn structure_splits_use_cases_functions_and_components() {
    let table = grid(MODEL);
    let operational = names(&table, "operational", "structure");
    assert!(operational.contains(&"User".to_string()) && operational.contains(&"Give access".to_string()) && operational.contains(&"Open".to_string()));
    let functional = names(&table, "functional", "structure");
    assert!(functional.contains(&"Unlock".to_string()) && functional.contains(&"Decide".to_string()));
    let constructional = names(&table, "constructional", "structure");
    assert!(constructional.contains(&"Controller".to_string()) && constructional.contains(&"Latch".to_string()));
}

#[test]
fn a_lifecycle_is_operational_and_modes_functional() {
    let table = grid(MODEL);
    assert!(names(&table, "operational", "states").contains(&"Door lifecycle".to_string()));
    assert!(names(&table, "operational", "states").contains(&"Production".to_string()));
    assert!(names(&table, "functional", "states").contains(&"Door modes".to_string()));
    assert!(names(&table, "constructional", "states").is_empty());
}

#[test]
fn a_scenario_between_components_is_constructional() {
    let table = grid(MODEL);
    let scenarios = names(&table, "constructional", "scenarios");
    assert!(scenarios.contains(&"Unlock sequence".to_string()), "{scenarios:?}");
    assert!(scenarios.contains(&"release".to_string()), "the message follows its scenario: {scenarios:?}");
}

#[test]
fn an_empty_view_is_a_gap_and_says_which_capella_diagram_fills_it() {
    let table = grid(MODEL);
    let constructional = table.rows.iter().find(|r| r.id == "constructional").unwrap();
    assert_eq!(constructional.cells["states"].status.as_deref(), Some("gap"));
    assert_eq!(constructional.status, "gap");
    assert!(constructional.issues.iter().any(|i| i.starts_with("Technical configurations") && i.contains("Mode State Machine")));
    let functional = table.rows.iter().find(|r| r.id == "functional").unwrap();
    assert_eq!(functional.cells["structure"].status.as_deref(), Some("ok"));
}

#[test]
fn a_scope_keeps_one_level() {
    let result = Compiler::new(CompilerConfig::default()).compile_string(MODEL).unwrap();
    let spec = TableSpec { kind: "cesam_grid".to_string(), scope: Some("LA".to_string()), ..TableSpec::default() };
    let table = tables::compute(&result, &spec, &Options::default()).unwrap();
    assert_eq!(names(&table, "functional", "structure"), ["Decide", "Move"]);
    assert!(names(&table, "operational", "structure").is_empty());
}

#[test]
fn every_active_example_computes_its_grid() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/automotive/adaptive-cruise/adaptive_cruise_control.arc");
    let table = grid(&std::fs::read_to_string(root).unwrap());
    assert!(names(&table, "functional", "states").contains(&"ACC operating modes".to_string()));
    assert!(names(&table, "constructional", "scenarios").contains(&"Cut-in vehicle".to_string()));
    assert!(!names(&table, "functional", "interactions").is_empty());
}

#[test]
fn the_metamodel_names_the_cesam_view_of_each_placed_kind() {
    let metamodel = Metamodel::current();
    let note = |k: &str| metamodel.kind(k).and_then(|k| k.cesam.clone()).unwrap_or_default();
    assert!(note("SystemFunction").starts_with("Functional vision · Static elements"));
    assert!(note("LogicalComponent").starts_with("Constructional vision"));
    assert!(note("Requirement").contains("by its `vision:`"));
    assert!(metamodel.kind("Trace").unwrap().cesam.is_none());
    assert!(metamodel.to_markdown().contains("## CESAM views"));
}
