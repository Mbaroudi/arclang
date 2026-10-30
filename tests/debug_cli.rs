//! `arclang debug --list` from the command line: what VS Code asks before
//! offering *Run…* or answering F5.

use std::path::Path;
use std::process::Command;

fn list(model: &str) -> std::process::Output {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(model);
    Command::new(env!("CARGO_BIN_EXE_arclang")).args(["debug", "--list"]).arg(path).output().expect("arclang runs")
}

#[test]
fn a_model_with_nothing_to_run_lists_nothing_and_still_compiles() {
    let dir = tempfile::tempdir().unwrap();
    let model = dir.path().join("hardware.arc");
    std::fs::write(&model, "model \"Hardware only\" {}\nphysical_architecture \"HW\" {\n    node \"Battery pack\" { id: \"PN-BATT\" voltage: 400 V }\n}\n").unwrap();
    let out = list(model.to_str().unwrap());
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let listed: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    for key in ["machines", "scenarios", "chains"] {
        assert_eq!(listed[key], serde_json::json!([]), "{key}");
    }
}

#[test]
fn the_ev_range_example_runs_its_drive_modes_trip_and_speed_loop() {
    let out = list("examples/automotive/ev_range_simulation.arc");
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let listed: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(listed["machines"], serde_json::json!(["Drive modes"]));
    assert_eq!(listed["scenarios"], serde_json::json!(["Winter highway trip"]));
    assert_eq!(listed["chains"], serde_json::json!(["Speed control loop"]));
}

#[test]
fn a_model_that_does_not_compile_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let model = dir.path().join("broken.arc");
    std::fs::write(&model, "model \"M\" {\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_arclang")).args(["debug", "--list"]).arg(&model).output().unwrap();
    assert!(!out.status.success());
}
