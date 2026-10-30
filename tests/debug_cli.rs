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
    let out = list("examples/automotive/ev_range_simulation.arc");
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let listed: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    for key in ["machines", "scenarios", "chains"] {
        assert_eq!(listed[key], serde_json::json!([]), "{key}");
    }
}

#[test]
fn a_model_that_does_not_compile_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let model = dir.path().join("broken.arc");
    std::fs::write(&model, "model \"M\" {\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_arclang")).args(["debug", "--list"]).arg(&model).output().unwrap();
    assert!(!out.status.success());
}
