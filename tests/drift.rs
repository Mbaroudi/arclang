//! Drift: the code as built against the model as designed. A hand edit of
//! the generated C is found, said in the terms of the model, with its line.

use std::path::{Path, PathBuf};
use std::process::Output;

fn arclang(args: &[&str]) -> Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_arclang")).args(args).output().expect("arclang runs")
}

fn text(output: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr))
}

/// The C of an example in a fresh directory: (the directory kept alive, the model, the C).
fn built(example: &str) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let model = Path::new(env!("CARGO_MANIFEST_DIR")).join(example);
    let c = dir.path().join("c");
    let exported = arclang(&["export", model.to_str().unwrap(), "-f", "c", "-o", c.to_str().unwrap()]);
    assert!(exported.status.success(), "{}", text(&exported));
    (dir, model, c)
}

fn edit(file: &Path, from: &str, to: &str) {
    let before = std::fs::read_to_string(file).unwrap();
    assert!(before.contains(from), "the generated C has `{from}`");
    std::fs::write(file, before.replacen(from, to, 1)).unwrap();
}

const SPEED_MONITOR: &str = "examples/automotive/speed-monitor/speed_monitor.arc";

#[test]
fn the_code_the_model_generates_has_not_drifted() {
    let (_dir, model, c) = built(SPEED_MONITOR);
    let same = arclang(&["drift", model.to_str().unwrap(), c.to_str().unwrap()]);
    assert!(same.status.success(), "{}", text(&same));
    assert!(text(&same).contains("No drift: the code is, byte for byte, the code the model generates"), "{}", text(&same));
}

#[test]
fn hand_edits_of_the_code_are_said_in_the_terms_of_the_model() {
    let (_dir, model, c) = built(SPEED_MONITOR);
    let code = c.join("speed_monitor.c");
    // A guard, an initial value and the statement of an equation.
    edit(&code, "(in->v_speed > st->v_on_above)", "(in->v_speed >= st->v_on_above)");
    edit(&code, "state->overspeed_warning.v_on_above = 130.0;", "state->overspeed_warning.v_on_above = 135.0;");
    edit(&code, "(0.25 * (in->v_raw - st->v_filtered))", "(0.3 * (in->v_raw - st->v_filtered))");
    let drifted = arclang(&["drift", model.to_str().unwrap(), c.to_str().unwrap()]);
    let said = text(&drifted);
    assert!(!drifted.status.success(), "{said}");
    assert!(said.contains("the guard of transition Normal -> Warning is `speed >= on_above` in the code; the model says `speed > on_above` (speed_monitor.c:"), "{said}");
    assert!(said.contains("variable 'on_above' starts at `135.0` in the code; the model says `130.0` (speed_monitor.c:"), "{said}");
    assert!(said.contains("equation `filtered = filtered + 0.25 * (raw - filtered)` is computed in the code as `st->v_filtered = (st->v_filtered + (0.3 * (in->v_raw - st->v_filtered)));`"), "{said}");
    assert!(said.contains("3 difference(s) in its machines, 0 line(s) they do not explain"), "{said}");

    // The same, as JSON: each finding with its machine, its file and its line.
    let json = arclang(&["drift", model.to_str().unwrap(), c.to_str().unwrap(), "--json"]);
    let value: serde_json::Value = serde_json::from_slice(&json.stdout).expect("JSON on the standard output");
    assert_eq!(value["identical"], false);
    let findings = value["findings"].as_array().unwrap();
    assert_eq!(findings.len(), 3, "{value}");
    assert!(findings.iter().all(|f| f["file"] == "speed_monitor.c" && f["line"].as_u64().is_some_and(|line| line > 0)), "{value}");
    assert!(findings.iter().any(|f| f["machine"] == "Wheel speed filter") && findings.iter().any(|f| f["machine"] == "Overspeed warning"), "{value}");
}

#[test]
fn a_comment_that_lies_and_a_helper_edited_by_hand_are_found_too() {
    let (_dir, model, c) = built(SPEED_MONITOR);
    let code = c.join("speed_monitor.c");
    // The comment of an equation says another limit than the statement under it uses.
    edit(&code, "/* speed = saturate(filtered, 0 km/h, 250 km/h) */", "/* speed = saturate(filtered, 0 km/h, 200 km/h) */");
    // A function the machines call, changed: no machine says it, the line does.
    edit(&code, "        r = lo;", "        r = hi;");
    let drifted = arclang(&["drift", model.to_str().unwrap(), c.to_str().unwrap()]);
    let said = text(&drifted);
    assert!(!drifted.status.success(), "{said}");
    assert!(said.contains("the comment of equation `speed = saturate(filtered, 0 km/h, 250 km/h)` says `speed = saturate(filtered, 0 km/h, 200 km/h)` in the code: the statement under it is the model's"), "{said}");
    assert!(said.contains("1 line(s) of the code are not what its machines generate") && said.contains("r = hi;"), "{said}");
}

#[test]
fn a_change_of_structure_is_a_transition_or_a_state_of_the_model() {
    // A machine with regions and timed transitions: its time, edited in the code.
    let (_dir, model, c) = built("examples/energy/battery-charger/battery_charger.arc");
    let code = c.join("battery_charger.c");
    edit(&code, "((200000UL + ", "((500000UL + ");
    let drifted = arclang(&["drift", model.to_str().unwrap(), c.join("battery_charger.h").to_str().unwrap()]);
    let said = text(&drifted);
    assert!(!drifted.status.success(), "{said}");
    assert!(said.contains("the time of transition Charging.Precharge -> Charging.Bulk is `after 500000 us` in the code; the model says `after 200000 us`"), "{said}");
}

#[test]
fn the_code_of_another_model_is_refused() {
    let (_dir, _, c) = built(SPEED_MONITOR);
    let other = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/energy/battery-charger/battery_charger.arc");
    let refused = arclang(&["drift", other.to_str().unwrap(), c.to_str().unwrap()]);
    assert!(!refused.status.success());
    assert!(text(&refused).contains("this code is that of another model"), "{}", text(&refused));
}

#[test]
fn every_example_is_without_drift_from_its_own_code() {
    for example in ["examples/automotive/power-window/power_window.arc", "examples/capella/in-flight-entertainment/in_flight_entertainment.arc", "examples/automotive/body-controller/body_controller.arc"] {
        let (_dir, model, c) = built(example);
        let header = std::fs::read_dir(&c).unwrap().flatten().map(|e| e.path()).find(|p| p.extension().is_some_and(|e| e == "h") && std::fs::read_to_string(p).unwrap().contains("/* state_machine \"")).unwrap();
        let same = arclang(&["drift", model.to_str().unwrap(), header.to_str().unwrap()]);
        assert!(same.status.success(), "{example}: {}", text(&same));
    }
}
