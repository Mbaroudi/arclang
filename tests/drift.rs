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
    assert!(text(&same).contains("No drift: the machines' code is, byte for byte, the code the model generates"), "{}", text(&same));
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

#[test]
fn every_file_the_model_generates_is_compared_the_io_layer_and_the_vhdl_too() {
    let (dir, model, c) = built(SPEED_MONITOR);
    let hdl = dir.path().join("hdl");
    let exported = arclang(&["export", model.to_str().unwrap(), "-f", "vhdl", "-o", hdl.to_str().unwrap()]);
    assert!(exported.status.success(), "{}", text(&exported));

    // Untouched: the machines, the I/O layer, the host files (the harness among them) and the logic are the model's.
    let same = arclang(&["drift", model.to_str().unwrap(), c.to_str().unwrap(), "--hdl", hdl.to_str().unwrap()]);
    assert!(same.status.success(), "{}", text(&same));
    assert!(text(&same).contains("byte for byte, the code the model generates, and so are the 12 other file(s) compared"), "{}", text(&same));

    // A line of the host I/O and a line of the VHDL changed by hand, one more added.
    edit(&c.join("host/arclang_io_host.c"), "        gpio_out[k] = false;", "        gpio_out[k] = true;");
    edit(&hdl.join("speed_monitor_wheel_speed_filter.vhd"), "        ready <= '1';", "        ready <= '0';\n        -- patched on the bench");
    let drifted = arclang(&["drift", model.to_str().unwrap(), c.to_str().unwrap(), "--hdl", hdl.to_str().unwrap()]);
    let said = text(&drifted);
    assert!(!drifted.status.success(), "{said}");
    assert!(said.contains("The machines' code is, byte for byte, the code the model generates."), "{said}");
    assert!(said.contains("host/arclang_io_host.c:39: 1 line(s) of the file as built where the model generates 1") && said.contains("built:         gpio_out[k] = true;") && said.contains("model:         gpio_out[k] = false;"), "{said}");
    assert!(said.contains("speed_monitor_wheel_speed_filter.vhd:76: 2 line(s) of the file as built where the model generates 1") && said.contains("-- patched on the bench"), "{said}");
    assert!(said.contains("0 difference(s) in its machines, 0 line(s) they do not explain, 2 other file(s) that differ"), "{said}");

    // A file that is not there is said, and is not a drift.
    std::fs::remove_file(c.join("host/arclang_io_host.h")).unwrap();
    let json = arclang(&["drift", model.to_str().unwrap(), c.to_str().unwrap(), "--json"]);
    let value: serde_json::Value = serde_json::from_slice(&json.stdout).expect("JSON");
    let files = value["files"].as_array().unwrap();
    assert!(files.iter().any(|f| f["file"] == "host/arclang_io_host.h" && f["missing"] == true), "{value}");
    assert!(files.iter().any(|f| f["file"] == "host/arclang_io_host.c" && f["line"] == 39), "{value}");

    // A directory that holds none of the logic's files is refused.
    let wrong = arclang(&["drift", model.to_str().unwrap(), c.to_str().unwrap(), "--hdl", dir.path().to_str().unwrap()]);
    assert!(!wrong.status.success() && text(&wrong).contains("holds none of the files the model generates for the logic"), "{}", text(&wrong));
}

#[test]
fn every_line_the_machines_do_not_explain_is_listed() {
    let (_dir, model, c) = built(SPEED_MONITOR);
    let code = c.join("speed_monitor.c");
    // Eight statements of the helper and of the bookkeeping changed: none is a machine's.
    let before = std::fs::read_to_string(&code).unwrap();
    let after = before.replace("        r = lo;", "        r = hi;").replace("st->in_state[k]++;", "st->in_state[k] += 2U;").replace("st->in_state[s] = 0U;", "st->in_state[s] = 1U;").replace("    (void)s;", "    (void)s; (void)0;");
    let changed = before.lines().zip(after.lines()).filter(|(a, b)| a != b).count();
    assert!(changed > 5, "more lines than the earlier limit of five: {changed}");
    std::fs::write(&code, after).unwrap();
    let json = arclang(&["drift", model.to_str().unwrap(), c.to_str().unwrap(), "--json"]);
    let value: serde_json::Value = serde_json::from_slice(&json.stdout).expect("JSON");
    assert_eq!(value["unexplained_lines"], changed, "{value}");
    assert_eq!(value["unexplained"].as_array().unwrap().len(), changed, "{value}");
}
