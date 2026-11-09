//! Where the verification of a requirement stands is computed once
//! (`verification::requirement_status`) and read by the production gate,
//! the coverage table, the impact analysis and the requirements view: on
//! one model they say the same of every requirement. A case that says no
//! verdict verifies nothing.

use arclang::compiler::diagram::{build_diagrams, judge_requirements, ViewKind};
use arclang::compiler::impact::{self, whatif, Analysis};
use arclang::compiler::production_gate::{run_gate, GateReport, Severity};
use arclang::compiler::tables::{self, TableSpec};
use arclang::compiler::verification::{requirement_status, Standing};
use arclang::compiler::{Compiler, CompilerConfig};
use std::collections::BTreeMap;

const MODEL: &str = r#"
model "Status" {}

system_analysis "Needs" {
    requirement "Passes" { id: "REQ-PASS" description: "Its case passes" }
    requirement "Fails" { id: "REQ-FAIL" description: "Its case fails" }
    requirement "Waits" { id: "REQ-PENDING" description: "Its case says no verdict" }
    requirement "Half" { id: "REQ-MIXED" description: "One case passes, another says nothing" }
    requirement "Unknown" { id: "REQ-INC" description: "Its simulation was never run" }
    requirement "Alone" { id: "REQ-NOCASE" description: "No case verifies it" }
    requirement "Floating" { id: "REQ-UNSAT" description: "Its case passes, nothing satisfies it" }
    requirement "Split" { id: "REQ-WORST" description: "One case passes, another fails" }
}

logical_architecture "LA" {
    component "Unit" { id: "LC-1" }
}

trace "LC-1" satisfies "REQ-PASS"
trace "LC-1" satisfies "REQ-FAIL"
trace "LC-1" satisfies "REQ-PENDING"
trace "LC-1" satisfies "REQ-MIXED"
trace "LC-1" satisfies "REQ-INC"
trace "LC-1" satisfies "REQ-NOCASE"
trace "LC-1" satisfies "REQ-WORST"

test_case "Pass" { id: "TC-PASS" verifies: ["REQ-PASS", "REQ-MIXED", "REQ-UNSAT", "REQ-WORST"] method: "test" verdict: "pass" }
test_case "Fail" { id: "TC-FAIL" verifies: ["REQ-FAIL", "REQ-WORST"] method: "test" verdict: "fail" }
test_case "Demo" { id: "TC-DEMO" verifies: ["REQ-PENDING", "REQ-MIXED"] method: "demonstration" }
test_case "Run" {
    id: "TC-RUN"
    verifies: ["REQ-INC"]
    method: "analysis"
    simulate: { model: "P.M" stop_time: 1 h }
    expects: ["final(a.b) >= 2 km"]
}

table "Coverage" { kind: requirements_coverage columns: ["id", "verified_by", "verdict", "gate", "status"] }
"#;

/// Each requirement, where its verification stands, and whether something satisfies it.
const EXPECTED: [(&str, Standing, bool); 8] = [
    ("REQ-PASS", Standing::Pass, true),
    ("REQ-FAIL", Standing::Fail, true),
    ("REQ-PENDING", Standing::Pending, true),
    ("REQ-MIXED", Standing::Pending, true),
    ("REQ-INC", Standing::Inconclusive, true),
    ("REQ-NOCASE", Standing::Unverified, true),
    ("REQ-UNSAT", Standing::Pass, false),
    ("REQ-WORST", Standing::Fail, true),
];

fn compile(text: &str) -> arclang::CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_string(text).unwrap_or_else(|e| panic!("{e}"))
}

fn gate(result: &arclang::CompilationResult) -> GateReport {
    run_gate(&result.ast, &result.semantic_model, "ISO26262")
}

fn blockers<'g>(gate: &'g GateReport, requirement: &str) -> Vec<&'g str> {
    let quoted = format!("'{requirement}'");
    gate.findings.iter().filter(|f| matches!(f.severity, Severity::Blocker) && f.message.contains(&quoted)).map(|f| f.message.as_str()).collect()
}

#[test]
fn the_standing_of_a_requirement_is_the_worst_of_its_cases() {
    let result = compile(MODEL);
    for (id, standing, satisfied) in EXPECTED {
        let status = requirement_status(&result.ast, &result.semantic_model, id);
        assert_eq!(status.standing(), standing, "{id}");
        assert_eq!(status.satisfied(), satisfied, "{id}");
        assert_eq!(status.verified(), standing == Standing::Pass, "{id}");
    }
    let mixed = requirement_status(&result.ast, &result.semantic_model, "REQ-MIXED");
    let cases: Vec<&str> = mixed.cases.iter().map(|(case, _)| case.id.as_str()).collect();
    assert_eq!(cases, ["TC-PASS", "TC-DEMO"], "every case that verifies it, in the order of the model");
    assert_eq!(mixed.satisfied_by, ["LC-1"]);
}

#[test]
fn a_case_that_says_no_verdict_verifies_nothing_for_the_gate() {
    let result = compile(MODEL);
    let gate = gate(&result);
    assert_eq!((gate.requirements_total, gate.requirements_satisfied, gate.requirements_verified), (8, 7, 2));
    let split = blockers(&gate, "REQ-WORST");
    assert!(matches!(split.as_slice(), [one] if one.contains("verification cases fail")), "a passing case does not hide one that fails: {split:?}");
    assert!(!gate.passed);
    let pending = blockers(&gate, "REQ-PENDING");
    assert!(matches!(pending.as_slice(), [one] if one.contains("have no verdict (Demo)")), "{pending:?}");
    let mixed = blockers(&gate, "REQ-MIXED");
    assert!(matches!(mixed.as_slice(), [one] if one.contains("have no verdict (Demo)")), "a passing case does not hide one that says nothing: {mixed:?}");

    // The same model with the verdicts recorded: the gate counts them.
    let recorded = MODEL.replace("method: \"demonstration\" }", "method: \"demonstration\" verdict: \"pass\" }");
    let gate = self::gate(&compile(&recorded));
    assert_eq!(gate.requirements_verified, 4);
    assert!(blockers(&gate, "REQ-PENDING").is_empty() && blockers(&gate, "REQ-MIXED").is_empty());
}

#[test]
fn the_gate_the_table_the_impact_analysis_and_the_view_agree_on_every_requirement() {
    let result = compile(MODEL);
    let gate = gate(&result);

    let decl = result.ast.tables.iter().find(|t| t.name == "Coverage").unwrap();
    let table = tables::compute(&result, &TableSpec::of(decl), &tables::Options::default()).unwrap();
    let cell = |id: &str, key: &str| {
        let row = table.rows.iter().find(|r| r.id == id).unwrap_or_else(|| panic!("no row {id}"));
        let cell = &row.cells[key];
        (cell.status.clone().unwrap_or_default(), cell.text.clone())
    };

    let analysis = Analysis::new(&result, "ISO26262");
    let coverage: BTreeMap<String, whatif::Coverage> = whatif::coverage(&analysis).into_iter().map(|(id, (_, coverage))| (id, coverage)).collect();

    let mut set = build_diagrams(&result.ast);
    judge_requirements(&mut set, &result.ast, &result.semantic_model);
    let view = set.diagrams.iter().find(|d| d.kind == ViewKind::Req).expect("a requirements view");
    let drawn = |id: &str| view.nodes.iter().find(|n| n.id == id).and_then(|n| n.properties.get("verdict").cloned()).unwrap_or_else(|| panic!("{id} carries no verdict"));

    for (id, standing, satisfied) in EXPECTED {
        let verified = standing == Standing::Pass;
        // The gate blocks on a requirement exactly when it is not both satisfied and verified.
        assert_eq!(blockers(&gate, id).is_empty(), satisfied && verified, "gate, {id}: {:?}", blockers(&gate, id));
        // The table: its verdict is the standing, its row is covered exactly when the gate has nothing to say.
        let shown = if standing == Standing::Unverified { "none" } else { standing.word() };
        assert_eq!(cell(id, "verdict").1, shown, "table verdict, {id}");
        assert_eq!(cell(id, "status").0 == "ok", satisfied && verified, "table status, {id}: {:?}", cell(id, "status"));
        assert_eq!(cell(id, "gate").1 == "pass", satisfied && verified, "table gate, {id}: {:?}", cell(id, "gate"));
        // The impact analysis.
        let expected = match (satisfied, standing) {
            (false, _) => "unsatisfied",
            (true, Standing::Unverified) => "unverified",
            (true, Standing::Fail) => "failing",
            (true, Standing::Inconclusive) => "inconclusive",
            (true, Standing::Pending) => "pending",
            (true, Standing::Pass) => "covered",
        };
        assert_eq!(coverage[id].status, expected, "impact, {id}");
        assert_eq!(coverage[id].verified, verified, "impact, {id}");
        // The requirements view.
        assert_eq!(drawn(id), standing.word(), "view, {id}");
    }

    // And they count the same.
    let ok_rows = table.rows.iter().filter(|r| r.status == "ok").count();
    let both = EXPECTED.iter().filter(|(_, standing, satisfied)| *satisfied && *standing == Standing::Pass).count();
    assert_eq!(ok_rows, both);
    let edit = arclang::compiler::workbench::EditRequest::Set { element: "LC-1".to_string(), key: "description".to_string(), value: "\"The unit\"".to_string() };
    let report = impact::what_if(None, MODEL, vec![edit], &impact::Options::default()).unwrap();
    let counts = report.what_if.unwrap().coverage_after;
    assert_eq!((counts.requirements, counts.satisfied, counts.verified), (gate.requirements_total, gate.requirements_satisfied, gate.requirements_verified));
}

#[test]
fn a_requirement_is_not_verified_in_a_configuration_no_case_runs_in() {
    let model = r#"
model "Line" {}
system_analysis "Needs" {
    requirement "Range" { id: "REQ-RANGE" applies_to: ["City", "Touring"] description: "Drives 300 km" }
}
logical_architecture "LA" {
    component "Pack" { id: "LC-PACK" }
}
configuration "City" { id: "CFG-CITY" }
configuration "Touring" { id: "CFG-TOURING" }
trace "LC-PACK" satisfies "REQ-RANGE"
test_case "City drive" {
    id: "TC-CITY"
    verifies: ["REQ-RANGE"]
    method: "analysis"
    simulate: { model: "P.M" stop_time: 1 h configuration: "CFG-CITY" }
    expects: ["final(a.b) >= 300 km"]
}
"#;
    let result = compile(model);
    let status = requirement_status(&result.ast, &result.semantic_model, "REQ-RANGE");
    assert_eq!(status.uncovered, ["CFG-TOURING"]);
    assert_eq!(status.standing(), Standing::Inconclusive, "the case that exists was never run");
    assert!(!status.verified());
    let gate = gate(&result);
    assert!(blockers(&gate, "REQ-RANGE").iter().any(|b| b.contains("applies to configuration Touring (CFG-TOURING), which no verification case runs in")), "{:?}", blockers(&gate, "REQ-RANGE"));
}

/// A requirement is referred to by its id, or by its name when one
/// requirement alone carries it: the gate reads a case that names its
/// requirement by its name as every other reader does.
#[test]
fn a_case_naming_its_requirement_by_its_name_verifies_it_for_the_gate_too() {
    let model = r#"
model "Named" {}
system_analysis "Needs" {
    requirement "Charge" { id: "REQ-CHARGE" description: "Charges in 30 min" }
}
logical_architecture "LA" {
    component "Charger" { id: "LC-CHG" }
}
trace "LC-CHG" satisfies "REQ-CHARGE"
test_case "Fast charge" { id: "TC-CHG" verifies: ["Charge"] method: "test" verdict: "fail" }
table "Coverage" { kind: requirements_coverage columns: ["id", "verdict", "status"] }
"#;
    let unknown = |gate: &GateReport| gate.findings.iter().filter(|f| f.message.contains("unknown requirement")).map(|f| f.message.clone()).collect::<Vec<_>>();
    let result = compile(model);
    let status = requirement_status(&result.ast, &result.semantic_model, "REQ-CHARGE");
    assert_eq!(status.standing(), Standing::Fail);
    let gate = gate(&result);
    assert!(blockers(&gate, "REQ-CHARGE").iter().any(|b| b.contains("verification cases fail")), "{:?}", blockers(&gate, "REQ-CHARGE"));
    assert_eq!(unknown(&gate), Vec::<String>::new());
    let decl = result.ast.tables.iter().find(|t| t.name == "Coverage").unwrap();
    let table = tables::compute(&result, &TableSpec::of(decl), &tables::Options::default()).unwrap();
    assert_eq!((table.rows[0].cells["verdict"].text.as_str(), table.rows[0].status.as_str()), ("fail", "fail"));

    // The same case with a passing verdict: the requirement is verified, nothing blocks on the reference.
    let gate = self::gate(&compile(&model.replace("verdict: \"fail\"", "verdict: \"pass\"")));
    assert_eq!((gate.requirements_verified, gate.requirements_total), (1, 1));
    assert_eq!(unknown(&gate), Vec::<String>::new());
    assert_eq!(blockers(&gate, "REQ-CHARGE"), Vec::<&str>::new());
}

#[test]
fn a_recorded_verdict_counts_for_the_configuration_it_names() {
    let model = r#"
model "Line" {}
system_analysis "Needs" {
    requirement "Range" { id: "REQ-RANGE" applies_to: ["City", "Touring"] description: "Drives 300 km" }
}
logical_architecture "LA" {
    component "Pack" { id: "LC-PACK" }
}
configuration "City" { id: "CFG-CITY" }
configuration "Touring" { id: "CFG-TOURING" }
trace "LC-PACK" satisfies "REQ-RANGE"
test_case "City drive" { id: "TC-CITY" verifies: ["REQ-RANGE"] method: "test" verdict: "pass" configuration: "CFG-CITY" }
"#;
    let result = compile(model);
    let status = requirement_status(&result.ast, &result.semantic_model, "REQ-RANGE");
    assert_eq!(status.uncovered, ["CFG-TOURING"]);
    assert!(!status.verified());
    let gate = gate(&result);
    assert!(blockers(&gate, "REQ-RANGE").iter().any(|b| b.contains("applies to configuration Touring (CFG-TOURING), which no verification case runs in")), "{:?}", blockers(&gate, "REQ-RANGE"));

    // A second drive, in the other configuration (named by its name): verified in both.
    let both = format!("{model}test_case \"Touring drive\" {{ id: \"TC-TOURING\" verifies: [\"REQ-RANGE\"] method: \"test\" verdict: \"pass\" configuration: \"Touring\" }}\n");
    let result = compile(&both);
    let status = requirement_status(&result.ast, &result.semantic_model, "REQ-RANGE");
    assert_eq!(status.uncovered, Vec::<String>::new());
    assert!(status.verified());

    // Said in no configuration, the verdict stands for every one.
    let result = compile(&model.replace(" configuration: \"CFG-CITY\"", ""));
    assert!(requirement_status(&result.ast, &result.semantic_model, "REQ-RANGE").verified());
}

#[test]
fn a_case_cannot_say_two_configurations() {
    let model = r#"
model "Line" {}
system_analysis "Needs" {
    requirement "Range" { id: "REQ-RANGE" description: "Drives 300 km" }
}
configuration "City" { id: "CFG-CITY" }
configuration "Touring" { id: "CFG-TOURING" }
test_case "Drive" {
    id: "TC-DRIVE"
    verifies: ["REQ-RANGE"]
    method: "analysis"
    configuration: "CFG-TOURING"
    simulate: { model: "P.M" stop_time: 1 h configuration: "CFG-CITY" }
    expects: ["final(a.b) >= 300 km"]
}
"#;
    let error = Compiler::new(CompilerConfig::default()).compile_string(model).err().map(|e| e.to_string()).unwrap_or_default();
    assert!(error.contains("test_case 'Drive' names configuration 'CFG-TOURING' and simulates in 'CFG-CITY': say it once"), "{error}");
    // An unknown one is refused, as in `applies_to:`.
    let unknown = model.replace("configuration: \"CFG-TOURING\"\n    simulate: { model: \"P.M\" stop_time: 1 h configuration: \"CFG-CITY\" }", "configuration: \"CFG-SPORT\"\n    simulate: { model: \"P.M\" stop_time: 1 h }");
    let error = Compiler::new(CompilerConfig::default()).compile_string(&unknown).err().map(|e| e.to_string()).unwrap_or_default();
    assert!(error.contains("test_case 'Drive' runs in 'CFG-SPORT', which is no configuration"), "{error}");
}
