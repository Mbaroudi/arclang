//! Defects of 4.0.0 fixed in 4.0.1, each pinned through the real binary.

use std::path::Path;
use std::process::{Command, Output};

const BASE: &str = r#"model BrakeAssist {
  version: "1.0.0"

  operational_analysis "Driving" {
    actor "Driver" { id: "ACT-DRIVER" description: "Drives the vehicle" }
  }

  requirements safety {
    req "REQ-001" "Braking reaction" { description: "Command braking within 100 ms" }
  }

  architecture logical {
    component "Brake controller" {
      id: "LC-BRAKE"
      function "Command braking" { id: "LF-CMD" latency: 40 ms }
    }
    component "Radar" {
      id: "LC-RADAR"
      function "Detect obstacle" { id: "LF-DET" latency: 20 ms }
    }
  }
}

test_case "TC-001" {
  verifies: ["REQ-001"]
  method: "test"
  description: "HIL measurement"
}

safety_analysis {
  hazard "Late braking" {
    description: "Braking is commanded too late"
    severity: "S2"
    exposure: "E4"
    controllability: "C3"
    asil: "ASIL-C"
    mitigated_by: ["REQ-001"]
  }
}

trace "LC-BRAKE" satisfies "REQ-001" { rationale: "Controller issues the command" }
trace "LC-RADAR" satisfies "REQ-001" { rationale: "Radar provides the detection" }
"#;

fn arclang(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_arclang")).args(args).output().expect("arclang runs")
}

fn text(output: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr))
}

fn write(directory: &Path, name: &str, source: &str) -> String {
    let path = directory.join(name);
    std::fs::write(&path, source).unwrap();
    path.to_string_lossy().into_owned()
}

/// Diff of BASE against BASE with `from` replaced by `to`.
fn diff_after(from: &str, to: &str) -> (Output, serde_json::Value) {
    assert!(BASE.contains(from), "the base model has no `{from}`");
    let directory = tempfile::tempdir().unwrap();
    let old = write(directory.path(), "old.arc", BASE);
    let new = write(directory.path(), "new.arc", &BASE.replace(from, to));
    let human = arclang(&["diff", &old, &new]);
    let json = arclang(&["diff", &old, &new, "--json"]);
    let report = serde_json::from_slice(&json.stdout).unwrap_or_else(|e| panic!("diff --json: {e}\n{}", text(&json)));
    (human, report)
}

fn changed_fields(report: &serde_json::Value, id: &str) -> Vec<String> {
    report["modified"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["id"] == id)
        .flat_map(|entry| entry["changes"].as_array().unwrap().iter())
        .map(|change| change["field"].as_str().unwrap().to_string())
        .collect()
}

// ---- Parser: headerless file ---------------------------------------------

#[test]
fn a_headerless_file_accepts_every_layer_after_requirements() {
    let directory = tempfile::tempdir().unwrap();
    let path = write(
        directory.path(),
        "headerless.arc",
        r#"requirements safety {
  req "REQ-001" "Braking reaction" { description: "Command braking within 100 ms" }
}

operational_analysis "Driving" {
  actor "Driver" { id: "ACT-DRIVER" }
}

system_analysis "Vehicle" {
  requirement "REQ-002" { description: "Detect an obstacle at 150 m" }
}

architecture logical {
  component "Brake controller" { id: "LC-BRAKE" }
}

epbs "Product" {
  system "Brake system" { }
}

trace "LC-BRAKE" satisfies "REQ-001"
trace "LC-BRAKE" satisfies "REQ-002"
"#,
    );

    let output = arclang(&["check", &path]);
    assert!(output.status.success(), "{}", text(&output));

    // The blocks are kept, not merely skipped: each one shows in the model.
    let json = directory.path().join("headerless.json").to_string_lossy().into_owned();
    let build = arclang(&["build", &path, "-o", &json]);
    assert!(build.status.success(), "{}", text(&build));
    assert!(text(&build).contains("Requirements: 2"), "{}", text(&build));
    let model = std::fs::read_to_string(&json).unwrap();
    for kept in ["Driver", "REQ-002", "Brake system"] {
        assert!(model.contains(kept), "`{kept}` is missing from the compiled model");
    }
}

// ---- build ------------------------------------------------------------------

#[test]
fn build_writes_json_into_the_json_file() {
    let directory = tempfile::tempdir().unwrap();
    let path = write(directory.path(), "brake.arc", BASE);

    let output = arclang(&["build", &path]);

    assert!(output.status.success(), "{}", text(&output));
    let written = std::fs::read_to_string(directory.path().join("brake.json")).unwrap();
    let model: serde_json::Value = serde_json::from_str(&written).expect("brake.json holds JSON");
    assert!(model.is_object());
    for expected in ["REQ-001", "LC-BRAKE", "Late braking"] {
        assert!(written.contains(expected), "`{expected}` is missing from brake.json");
    }
    assert!(text(&output).contains("Requirements: 1"), "{}", text(&output));
}

#[test]
fn build_target_capella_writes_xml_into_a_capella_file() {
    let directory = tempfile::tempdir().unwrap();
    let path = write(directory.path(), "brake.arc", BASE);

    let output = arclang(&["build", &path, "--target", "capella"]);

    assert!(output.status.success(), "{}", text(&output));
    let written = std::fs::read_to_string(directory.path().join("brake.capella")).unwrap();
    assert!(written.trim_start().starts_with("<?xml"), "{}", &written[..written.len().min(200)]);
    assert!(!directory.path().join("brake.json").exists());
}

#[test]
fn build_refuses_an_unknown_target() {
    let directory = tempfile::tempdir().unwrap();
    let path = write(directory.path(), "brake.arc", BASE);

    let output = arclang(&["build", &path, "--target", "cobol"]);

    assert!(!output.status.success());
    assert!(text(&output).contains("cobol"), "{}", text(&output));
    assert!(!directory.path().join("brake.json").exists());
}

// ---- Traceability warnings ----------------------------------------------------

#[test]
fn check_raises_no_traceability_warning_on_a_traced_model() {
    let directory = tempfile::tempdir().unwrap();
    let path = write(directory.path(), "brake.arc", BASE);

    let output = arclang(&["check", &path]);

    assert!(output.status.success(), "{}", text(&output));
    assert!(!text(&output).contains("Traceability warnings"), "{}", text(&output));
    let validate = arclang(&["trace", &path, "--validate"]);
    assert!(text(&validate).contains("All elements properly traced"), "{}", text(&validate));
}

#[test]
fn check_names_the_requirement_and_the_component_that_no_trace_touches() {
    let directory = tempfile::tempdir().unwrap();
    let untraced = BASE.replace(
        "trace \"LC-RADAR\" satisfies \"REQ-001\" { rationale: \"Radar provides the detection\" }",
        "requirements extra { req \"REQ-009\" \"Orphan\" { description: \"Nothing satisfies this\" } }",
    );
    let path = write(directory.path(), "brake.arc", &untraced);

    let output = arclang(&["check", &path]);
    let report = text(&output);

    assert!(report.contains("Requirement REQ-009 has no trace"), "{report}");
    assert!(report.contains("Component LC-RADAR has no trace"), "{report}");
    assert!(!report.contains("REQ-001 has no"), "{report}");
    assert!(!report.contains("LC-BRAKE has no"), "{report}");
    assert!(!report.contains("ACT-DRIVER"), "an actor is not an architecture component: {report}");
}

#[test]
fn a_trace_from_a_function_covers_its_component() {
    let directory = tempfile::tempdir().unwrap();
    let by_function = BASE.replace("trace \"LC-RADAR\" satisfies", "trace \"LF-DET\" satisfies");
    let path = write(directory.path(), "brake.arc", &by_function);

    let output = arclang(&["check", &path]);

    assert!(!text(&output).contains("Traceability warnings"), "{}", text(&output));
}

#[test]
fn a_physical_node_is_not_reported_as_an_untraced_component() {
    let directory = tempfile::tempdir().unwrap();
    let deployed = format!(
        "{BASE}\narchitecture physical {{\n  node \"Brake ECU\" {{\n    id: \"PN-ECU\"\n    deploys \"LC-BRAKE\"\n    deploys \"LC-RADAR\"\n  }}\n}}\n"
    );
    let path = write(directory.path(), "brake.arc", &deployed);

    let output = arclang(&["check", &path]);

    assert!(output.status.success(), "{}", text(&output));
    assert!(!text(&output).contains("PN-ECU has no trace"), "{}", text(&output));
}

// ---- diff ---------------------------------------------------------------------

#[test]
fn diff_of_a_model_with_itself_is_empty() {
    let directory = tempfile::tempdir().unwrap();
    let old = write(directory.path(), "old.arc", BASE);
    let new = write(directory.path(), "new.arc", BASE);

    let output = arclang(&["diff", &old, &new]);

    assert!(output.status.success(), "{}", text(&output));
    assert!(text(&output).contains("No semantic changes."));
}

#[test]
fn diff_reports_a_changed_quantity() {
    let (human, report) = diff_after("latency: 40 ms", "latency: 45 ms");

    assert_eq!(human.status.code(), Some(1), "{}", text(&human));
    assert_eq!(changed_fields(&report, "LF-CMD"), ["latency"]);
    assert!(text(&human).contains("latency: \"40 ms\" -> \"45 ms\""), "{}", text(&human));
}

#[test]
fn diff_reports_an_added_attribute() {
    let (human, report) = diff_after("id: \"LC-BRAKE\"", "id: \"LC-BRAKE\" owner: \"Chassis team\"");

    assert_eq!(human.status.code(), Some(1));
    assert_eq!(changed_fields(&report, "LC-BRAKE"), ["owner"]);
}

#[test]
fn diff_reports_an_actor_description() {
    let (human, report) = diff_after("Drives the vehicle", "Supervises the vehicle");

    assert_eq!(human.status.code(), Some(1));
    assert_eq!(changed_fields(&report, "ACT-DRIVER"), ["description"]);
}

#[test]
fn diff_reports_a_changed_test_method() {
    let (human, report) = diff_after("method: \"test\"", "method: \"analysis\"");

    assert_eq!(human.status.code(), Some(1));
    assert_eq!(changed_fields(&report, "TC-001"), ["method"]);
}

#[test]
fn diff_reports_a_removed_test_case() {
    let test_case = "test_case \"TC-001\" {\n  verifies: [\"REQ-001\"]\n  method: \"test\"\n  description: \"HIL measurement\"\n}\n";
    let (human, report) = diff_after(test_case, "");

    assert_eq!(human.status.code(), Some(1));
    let removed: Vec<_> = report["removed"].as_array().unwrap().iter().map(|e| e["id"].clone()).collect();
    assert_eq!(removed, ["TC-001"]);
    let relationships = report["relationships_removed"].as_array().unwrap();
    assert_eq!(relationships.len(), 1, "{report}");
    assert_eq!(relationships[0]["kind"], "Verification");
    assert_eq!(relationships[0]["source"], "TC-001");
    assert_eq!(relationships[0]["target"], "REQ-001");
}

#[test]
fn diff_reports_a_changed_hazard_severity() {
    let (human, report) = diff_after("severity: \"S2\"", "severity: \"S3\"");

    assert_eq!(human.status.code(), Some(1));
    assert_eq!(changed_fields(&report, "Late braking"), ["severity"]);
}

#[test]
fn diff_reports_a_reworded_rationale_as_a_modified_trace() {
    let (human, report) = diff_after("Controller issues the command", "Controller issues the command in time");

    assert_eq!(human.status.code(), Some(1), "{}", text(&human));
    assert!(report["traces_added"].as_array().unwrap().is_empty());
    assert!(report["traces_removed"].as_array().unwrap().is_empty());
    let modified = report["traces_modified"].as_array().unwrap();
    assert_eq!(modified.len(), 1, "{report}");
    assert_eq!(modified[0]["from"], "LC-BRAKE");
    assert_eq!(modified[0]["to"], "REQ-001");
    assert_eq!(modified[0]["changes"][0]["field"], "rationale");
    assert_eq!(modified[0]["changes"][0]["new"], "Controller issues the command in time");
}

#[test]
fn diff_ignores_layout_and_block_order() {
    let directory = tempfile::tempdir().unwrap();
    let old = write(directory.path(), "old.arc", BASE);
    let (head, tail) = BASE.split_once("test_case").unwrap();
    let (test_case, rest) = tail.split_once("safety_analysis").unwrap();
    let reordered = format!("{head}safety_analysis{rest}\n\ntest_case{test_case}").replace("  ", "    ");
    let new = write(directory.path(), "new.arc", &reordered);

    let output = arclang(&["diff", &old, &new]);

    assert!(output.status.success(), "{}", text(&output));
    assert!(text(&output).contains("No semantic changes."), "{}", text(&output));
}

/// Two elements sharing an id is a compile warning, not an error: the diff
/// must still compare both, not only the one that shadows the other.
const SHARED_ID: &str = r#"model Shared {
  version: "1.0.0"

  requirements safety {
    req "X1" "Reaction" { description: "React in time" }
  }

  architecture logical {
    component "First" {
      id: "LC-1"
      function "Measure" { id: "DUP" latency: 10 ms }
    }
    component "Second" {
      id: "LC-2"
      function "Decide" { id: "DUP" latency: 20 ms }
    }
  }
}
"#;

fn shared_id_diff(from: &str, to: &str) -> serde_json::Value {
    assert!(SHARED_ID.contains(from));
    let directory = tempfile::tempdir().unwrap();
    let old = write(directory.path(), "old.arc", SHARED_ID);
    let new = write(directory.path(), "new.arc", &SHARED_ID.replace(from, to));
    let output = arclang(&["diff", &old, &new, "--json"]);
    assert_eq!(output.status.code(), Some(1), "{}", text(&output));
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn diff_compares_both_elements_that_share_an_id() {
    for (from, to, name) in [("latency: 10 ms", "latency: 11 ms", "Measure"), ("latency: 20 ms", "latency: 21 ms", "Decide")] {
        let report = shared_id_diff(from, to);
        let modified = report["modified"].as_array().unwrap();
        assert_eq!(modified.len(), 1, "{report}");
        assert_eq!(modified[0]["name"], name, "{report}");
        assert_eq!(modified[0]["changes"][0]["field"], "latency");
    }
}

#[test]
fn diff_reports_an_added_element_whose_id_another_kind_already_uses() {
    let report = shared_id_diff("id: \"LC-2\"", "id: \"X1\"");

    let added: Vec<_> = report["added"].as_array().unwrap().iter().map(|e| e["name"].clone()).collect();
    assert!(added.contains(&serde_json::json!("Second")), "{report}");
    let removed: Vec<_> = report["removed"].as_array().unwrap().iter().map(|e| e["id"].clone()).collect();
    assert!(removed.contains(&serde_json::json!("LC-2")), "{report}");
}

#[test]
fn diff_ignores_the_order_of_two_exchanges_between_the_same_ports() {
    let model = |first: &str, second: &str| {
        format!(
            r#"model Links {{
  version: "1.0.0"
  architecture logical {{
    component "Radar" {{ id: "LC-RADAR" port out objects }}
    component "Brake controller" {{ id: "LC-BRAKE" port in objects }}
    component_exchange "Objects" {{ from_port: "LC-RADAR.objects" to_port: "LC-BRAKE.objects" exchange_item: "{first}" }}
    component_exchange "Objects" {{ from_port: "LC-RADAR.objects" to_port: "LC-BRAKE.objects" exchange_item: "{second}" }}
  }}
}}
"#
        )
    };
    let directory = tempfile::tempdir().unwrap();
    let old = write(directory.path(), "old.arc", &model("Track", "Status"));
    let swapped = write(directory.path(), "swapped.arc", &model("Status", "Track"));
    let changed = write(directory.path(), "changed.arc", &model("Track", "Heartbeat"));

    let same = arclang(&["diff", &old, &swapped]);
    assert!(same.status.success(), "{}", text(&same));

    let different = arclang(&["diff", &old, &changed]);
    assert_eq!(different.status.code(), Some(1), "{}", text(&different));
    assert!(text(&different).contains("Status") && text(&different).contains("Heartbeat"), "{}", text(&different));
}

// ---- diagram ------------------------------------------------------------------

#[test]
fn diagram_plant_uml_writes_a_plantuml_document() {
    let directory = tempfile::tempdir().unwrap();
    let path = write(directory.path(), "brake.arc", BASE);
    let target = directory.path().join("brake.puml");

    let output = arclang(&["diagram", &path, "-f", "plant-uml", "-o", &target.to_string_lossy()]);

    assert!(output.status.success(), "{}", text(&output));
    let written = std::fs::read_to_string(&target).unwrap();
    assert!(written.contains("@startuml") && written.contains("@enduml"), "{written}");
    assert!(written.contains("Brake controller"), "{written}");
}

#[test]
fn diagram_help_lists_only_formats_that_exist() {
    let help = text(&arclang(&["diagram", "--help"]));

    assert!(help.contains("plant-uml"), "{help}");
    for absent in ["graphviz", "  svg"] {
        assert!(!help.contains(absent), "`{absent}` is listed but was never implemented:\n{help}");
    }
}
