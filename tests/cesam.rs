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

// ---- Stakeholders, machine roles, state relations, methodology rules ----

use arclang::compiler::{cesam_rules, elements, rules};

fn compile(source: &str) -> Result<arclang::CompilationResult, String> {
    Compiler::new(CompilerConfig::default()).compile_string(source).map_err(|e| e.to_string())
}

fn example() -> arclang::CompilationResult {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/cesam/smart-lock/smart_lock.arc");
    Compiler::new(CompilerConfig::default()).compile_file(&path).expect("the CESAM example compiles")
}

#[test]
fn a_stakeholder_is_an_element_and_has_the_concerns_naming_it() {
    let result = example();
    let graph = elements::build(&result.ast, &result.semantic_model);
    let resident = graph.elements.iter().find(|e| e.kind == "Stakeholder" && e.id == "STK-RES").expect("stakeholder in the graph");
    assert_eq!(resident.name, "Resident");
    let needs: Vec<&str> = graph
        .relationships
        .iter()
        .filter(|r| r.kind == "Stakeholding" && r.target == resident.uuid)
        .filter_map(|r| graph.elements.iter().find(|e| e.uuid == r.source).map(|e| e.id.as_str()))
        .collect();
    assert_eq!(needs, ["NEED-1", "NEED-2", "NEED-3"]);
    assert!(graph.unresolved.is_empty(), "{:?}", graph.unresolved);
}

#[test]
fn the_cesam_example_fills_every_view_and_holds_every_rule() {
    let result = example();
    assert!(cesam_rules::declared(&result.ast));
    let advisories = cesam_rules::advisories(&result.ast, &result.semantic_model);
    assert!(advisories.is_empty(), "{advisories:?}");
    let spec = TableSpec { kind: "cesam_grid".to_string(), ..TableSpec::default() };
    let table = tables::compute(&result, &spec, &Options::default()).unwrap();
    for row in &table.rows {
        for (key, cell) in &row.cells {
            assert_ne!(cell.status.as_deref(), Some("gap"), "{} {key} is empty", row.id);
        }
    }
    assert_eq!(names(&table, "operational", "states")[0], "Lock lifecycle");
    assert_eq!(names(&table, "constructional", "states")[0], "Lock configurations");
    assert!(names(&table, "operational", "interfaces").contains(&"Access request".to_string()), "typed by operational exchanges");
}

#[test]
fn a_machine_kind_outside_its_enumeration_is_an_error() {
    let error = compile("state_machine \"M\" { kind: \"phases\" initial: \"A\" state A {} }").unwrap_err();
    assert!(error.contains("kind 'phases' is not one of lifecycle, functional_modes, technical_configurations"), "{error}");
    let error = compile("scenario \"S\" { vision: \"sideways\" }").unwrap_err();
    assert!(error.contains("vision 'sideways'"), "{error}");
}

#[test]
fn a_relation_to_no_state_is_an_error_and_to_a_state_of_another_role_a_warning() {
    let modes = "state_machine \"Modes\" { kind: \"functional_modes\" initial: \"On\" mode On {} }\n";
    let error = compile(&format!("{modes}state_machine \"Configs\" {{ kind: \"technical_configurations\" initial: \"C\" state C {{ realizes: [\"Modes.Off\"] }} }}")).unwrap_err();
    assert!(error.contains("realizes 'Modes.Off' is not a declared state"), "{error}");
    let lifecycle = "state_machine \"Life\" { kind: \"lifecycle\" initial: \"Use\" state Use {} }\n";
    let result = compile(&format!("{modes}{lifecycle}state_machine \"Configs\" {{ kind: \"technical_configurations\" initial: \"C\" state C {{ realizes: [\"Life.Use\"] }} }}")).unwrap();
    let warning = result.warnings.iter().find(|w| w.starts_with("cesam: ")).expect("a cesam warning");
    assert!(warning.contains("is a lifecycle phase, expected a functional mode"), "{warning}");
    assert_eq!(rules::classify_warning(warning).rule, "cesam.role");
    let error = compile(&format!("{lifecycle}state_machine \"L2\" {{ kind: \"lifecycle\" initial: \"P\" state P {{ context: [\"Nobody\"] }} }}")).unwrap_err();
    assert!(error.contains("context 'Nobody' is not a declared element"), "{error}");
}

#[test]
fn the_methodology_rules_name_what_cesam_expects() {
    let source = r#"
model M { method: "cesam" }
requirements stakeholder {
    stakeholder "Owner" { id: "STK-1" }
    stakeholder "Neighbour" { id: "STK-2" }
    concern "Quiet" { id: "NEED-1" stakeholders: ["STK-1"] }
}
system_analysis "S" {
    mission "Mow" { id: "MIS-1" }
    mission "Guard" { id: "MIS-2" }
    capability "Cut grass" { id: "UC-1" mission: "MIS-1" }
}
requirements { req "R-1" "Loose" { priority: Low } }
state_machine "Life" { kind: "lifecycle" initial: "Use" state Use {} }
state_machine "Modes" { kind: "functional_modes" initial: "On" mode On {} }
state_machine "Configs" { kind: "technical_configurations" initial: "C" state C {} }
"#;
    let result = compile(source).unwrap();
    let advisories = cesam_rules::advisories(&result.ast, &result.semantic_model);
    let rule = |id: &str| advisories.iter().filter(|a| a.rule == id).filter_map(|a| a.element.clone()).collect::<Vec<_>>();
    assert_eq!(rule("cesam.stakeholder-need"), ["STK-2"]);
    assert_eq!(rule("cesam.need-requirement"), ["NEED-1"]);
    assert_eq!(rule("cesam.mission-use-case"), ["MIS-2"]);
    assert_eq!(rule("cesam.use-case-scenario"), ["UC-1"]);
    assert_eq!(rule("cesam.lifecycle-context"), ["Life.Use"]);
    assert_eq!(rule("cesam.mode-configuration"), ["Modes.On"]);
    assert_eq!(rule("cesam.requirement-vision"), ["R-1"]);
    assert!(advisories.iter().any(|a| a.rule == "cesam.view-empty" && a.message.contains("Product breakdown")));
    // A model declaring the method reports them with its findings, and the gate warns.
    let findings = rules::findings(&result);
    assert!(findings.iter().any(|f| f.rule == "cesam.mission-use-case"));
    for advisory in &advisories {
        assert!(rules::rule(advisory.rule).is_some(), "{} is in the catalog", advisory.rule);
    }
    let gate = arclang::compiler::production_gate::run_gate(&result.ast, &result.semantic_model, "ISO26262");
    assert!(gate.findings.iter().any(|f| f.check == "methodology.cesam"));
}

#[test]
fn a_model_not_following_cesam_gets_no_cesam_findings() {
    let result = compile("requirements { req \"R-1\" \"Loose\" { priority: Low } }").unwrap();
    assert!(!cesam_rules::declared(&result.ast));
    assert!(rules::findings(&result).iter().all(|f| !f.rule.starts_with("cesam.")));
}

#[test]
fn stakeholders_are_exported_to_sysml_v2() {
    let result = example();
    let sysml = arclang::compiler::sysmlv2_generator::generate_sysmlv2(&result.semantic_model, &result.ast);
    assert!(sysml.contains("part def") && sysml.contains("Resident"), "stakeholder part def exported");
}

#[test]
fn the_cesam_diagrams_are_the_viewpoint_diagrams_titled_and_ordered_by_the_grid() {
    let result = example();
    let mut set = arclang::compiler::diagram::build_diagrams(&result.ast);
    let graph = elements::build(&result.ast, &result.semantic_model);
    arclang::compiler::cesam::arrange_diagrams(&mut set, &result.ast, &graph);
    let ids: Vec<&str> = set.diagrams.iter().map(|d| d.id.as_str()).collect();
    let at = |id: &str| ids.iter().position(|i| *i == id).unwrap_or_else(|| panic!("{id} missing from {ids:?}"));
    assert!(at("msm:Lock lifecycle") < at("msm:Lock modes") && at("msm:Lock modes") < at("msm:Lock configurations"));
    assert!(at("es:Resident comes home") < at("es:Credential check") && at("es:Credential check") < at("es:Unlock on the boards"));
    assert!(at("cap") < at("lab"));
    assert!(set.diagrams.iter().all(|d| !d.id.starts_with("gv") && !d.id.starts_with("par")));
    let lifecycle = set.diagrams.iter().find(|d| d.id == "msm:Lock lifecycle").unwrap();
    assert_eq!(lifecycle.title, "Lifecycle — Lock lifecycle (Operational vision)");
}

// ---- Levels, the system of interest, the template, the SysML v2 views ----

#[test]
fn a_cell_counts_its_elements_per_arcadia_level() {
    let table = grid(MODEL);
    let functional = table.rows.iter().find(|r| r.id == "functional").unwrap();
    let text = &functional.cells["structure"].text;
    assert!(text.starts_with("4 (SA 2, LA 2) · "), "{text}");
}

#[test]
fn a_component_as_the_system_of_interest_keeps_what_it_holds_and_what_crosses_its_boundary() {
    let result = example();
    let spec = TableSpec { kind: "cesam_grid".to_string(), scope: Some("LC-2".to_string()), ..TableSpec::default() };
    let table = tables::compute(&result, &spec, &Options::default()).unwrap();
    assert_eq!(names(&table, "functional", "structure"), ["Check credential", "Store codes", "Publish state"]);
    let interactions = names(&table, "constructional", "interactions");
    assert!(interactions.contains(&"Credential".to_string()) && interactions.contains(&"Bolt command".to_string()), "{interactions:?}");
    assert!(names(&table, "operational", "structure").is_empty());
}

#[test]
fn the_cesam_template_compiles_passes_the_gate_and_holds_every_rule() {
    let source = include_str!("../src/cli/templates/cesam.arc").replace("{{NAME}}", "Door").replace("{{FILE}}", "door.arc");
    let result = compile(&source).expect("the template compiles");
    assert!(cesam_rules::advisories(&result.ast, &result.semantic_model).is_empty());
    let gate = arclang::compiler::production_gate::run_gate(&result.ast, &result.semantic_model, "ISO26262");
    assert!(gate.passed, "{:?}", gate.findings);
}

#[test]
fn a_model_following_cesam_exports_its_views_to_sysml_v2_and_they_read_back() {
    let result = example();
    let sysml = arclang::compiler::sysmlv2_generator::generate_sysmlv2(&result.semantic_model, &result.ast);
    assert!(sysml.contains("view 'CESAM Lifecycle' {"), "{sysml}");
    assert!(sysml.contains("attribute vision : String = \"constructional\";"));
    assert!(sysml.contains("expose Behaviors::'Lock lifecycle';"));
    arclang::compiler::sysml_records::from_text(&sysml).expect("the export reads back");
    let plain = compile(MODEL).unwrap();
    let sysml = arclang::compiler::sysmlv2_generator::generate_sysmlv2(&plain.semantic_model, &plain.ast);
    assert!(!sysml.contains("view 'CESAM"), "a model not following CESAM gets no CESAM views");
}
