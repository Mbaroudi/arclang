//! The proof that the C as built does what the model's C does (`arclang
//! drift --prove`). The proofs need CBMC and Bitwuzla: they are ignored by
//! default and run in ArcLang's formal image:
//!   tools/embedded/formal/run.sh cargo test --test equivalence -- --include-ignored

use std::path::{Path, PathBuf};
use std::process::Output;

fn arclang(args: &[&str]) -> Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_arclang")).args(args).output().expect("arclang runs")
}

fn text(output: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr))
}

/// The C of an example in a fresh directory: (the directory kept alive, the model, the header of its machines).
fn built(example: &str) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let model = Path::new(env!("CARGO_MANIFEST_DIR")).join(example);
    let c = dir.path().join("c");
    assert!(arclang(&["export", model.to_str().unwrap(), "-f", "c", "-o", c.to_str().unwrap()]).status.success());
    let header = std::fs::read_dir(&c).unwrap().flatten().map(|e| e.path()).find(|p| p.extension().is_some_and(|e| e == "h") && std::fs::read_to_string(p).unwrap().contains("/* state_machine \"")).unwrap();
    (dir, model, header)
}

fn edit(file: &Path, from: &str, to: &str) {
    let before = std::fs::read_to_string(file).unwrap();
    assert!(before.contains(from), "the generated C has `{from}`");
    std::fs::write(file, before.replacen(from, to, 1)).unwrap();
}

fn prove(model: &Path, header: &Path) -> Output {
    // A slower machine than the one these were measured on must still end.
    arclang(&["drift", model.to_str().unwrap(), header.to_str().unwrap(), "--prove", "--timeout", "1200"])
}

const SPEED_MONITOR: &str = "examples/automotive/speed-monitor/speed_monitor.arc";

#[test]
#[ignore = "needs CBMC and Bitwuzla: tools/embedded/formal/run.sh cargo test --test equivalence -- --include-ignored"]
fn a_code_written_another_way_that_does_the_same_is_proven_equivalent() {
    let (_dir, model, header) = built(SPEED_MONITOR);
    let code = header.with_extension("c");
    // A temporary for the difference, and the comparison turned round: other text, same step.
    edit(&code, "    st->v_filtered = (st->v_filtered + (0.25 * (in->v_raw - st->v_filtered)));", "    { double delta = in->v_raw - st->v_filtered; st->v_filtered = st->v_filtered + (0.25 * delta); }");
    edit(&code, "if ((in->v_speed > st->v_on_above)) {", "if (st->v_on_above < in->v_speed) {");
    let proven = prove(&model, &header);
    let said = text(&proven);
    // The text differs, and drift says where...
    assert!(said.contains("the guard of transition Normal -> Warning is `on_above < speed` in the code"), "{said}");
    // ...and the proof says it does the same: the command succeeds.
    assert!(said.contains("Proven equivalent") && said.contains("19 fields compared"), "{said}");
    assert!(proven.status.success(), "{said}");
}

#[test]
#[ignore = "needs CBMC and Bitwuzla: tools/embedded/formal/run.sh cargo test --test equivalence -- --include-ignored"]
fn a_change_of_behaviour_is_refuted_with_the_state_and_the_inputs_that_show_it() {
    let (_dir, model, header) = built(SPEED_MONITOR);
    // The warning now comes on at the threshold, not above it.
    edit(&header.with_extension("c"), "if ((in->v_speed > st->v_on_above)) {", "if ((in->v_speed >= st->v_on_above)) {");
    let refuted = prove(&model, &header);
    let said = text(&refuted);
    assert!(!refuted.status.success(), "{said}");
    assert!(said.contains("Not equivalent"), "{said}");
    // The witness: the speed at the threshold.
    let value = |field: &str| said.lines().find_map(|line| line.trim().strip_prefix(&format!("{field} = ")).map(|v| v.parse::<f64>().unwrap())).unwrap_or_else(|| panic!("{field} in\n{said}"));
    assert_eq!(value("inputs.overspeed_warning.v_speed"), value("state.overspeed_warning.v_on_above"), "{said}");
}

#[test]
#[ignore = "needs CBMC and Bitwuzla: tools/embedded/formal/run.sh cargo test --test equivalence -- --include-ignored"]
fn the_code_the_model_generates_is_proven_for_machines_with_events_regions_and_blocks() {
    // A block that computes, a machine with sixteen events a step, one with regions, a PID and timed transitions.
    for example in [SPEED_MONITOR, "examples/rail/level-crossing/level_crossing_protection.arc", "examples/energy/battery-charger/battery_charger.arc"] {
        let (_dir, model, header) = built(example);
        let proven = prove(&model, &header);
        assert!(proven.status.success() && text(&proven).contains("Proven equivalent"), "{example}: {}", text(&proven));
    }
}

#[test]
#[ignore = "needs CBMC and Bitwuzla: tools/embedded/formal/run.sh cargo test --test equivalence -- --include-ignored"]
fn a_proof_that_does_not_end_in_time_is_neither_a_proof_nor_a_refutation() {
    // The power window (regions, histories, every pseudo-state, sixteen events a
    // step) is beyond a short limit: the answer is "neither", never a wrong one.
    let (_dir, model, header) = built("examples/automotive/power-window/power_window.arc");
    let output = arclang(&["drift", model.to_str().unwrap(), header.to_str().unwrap(), "--prove", "--timeout", "10"]);
    let said = text(&output);
    assert!(!said.contains("Not equivalent"), "{said}");
    assert!(said.contains("neither proven nor refuted") || said.contains("Proven equivalent"), "{said}");
    // The code is the model's, byte for byte: no drift, whatever the proof says.
    assert!(output.status.success(), "{said}");
}

#[test]
fn without_the_tools_the_command_says_where_they_are() {
    let (_dir, model, header) = built(SPEED_MONITOR);
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_arclang"))
        .args(["drift", model.to_str().unwrap(), header.to_str().unwrap(), "--prove"])
        .env("ARCLANG_CBMC", "/nonexistent/cbmc")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(text(&output).contains("cannot run '/nonexistent/cbmc'") && text(&output).contains("tools/embedded/formal/run.sh"), "{}", text(&output));
}

#[test]
fn a_code_with_another_interface_is_not_compared() {
    let (_dir, model, header) = built(SPEED_MONITOR);
    edit(&header, "    double v_filtered; /* variable \"filtered\" [km/h] */", "    double v_filtered; /* variable \"filtered\" [km/h] */\n    double v_extra;");
    let output = prove(&model, &header);
    assert!(!output.status.success());
    assert!(text(&output).contains("do not have the same structures"), "{}", text(&output));
}
