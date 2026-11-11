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

/// The VHDL of an example beside its C: (the directory kept alive, the model, the C, the VHDL, the file of its first block).
fn built_logic(example: &str) -> (tempfile::TempDir, PathBuf, PathBuf, PathBuf, PathBuf) {
    let (dir, model, header) = built(example);
    let logic = dir.path().join("vhdl");
    assert!(arclang(&["export", model.to_str().unwrap(), "-f", "vhdl", "-o", logic.to_str().unwrap()]).status.success());
    let block = std::fs::read_dir(&logic).unwrap().flatten().map(|e| e.path()).find(|p| p.extension().is_some_and(|e| e == "vhd") && !p.to_string_lossy().ends_with("_pkg.vhd") && !p.to_string_lossy().ends_with("_wb.vhd")).unwrap();
    let c = header.parent().unwrap().to_path_buf();
    (dir, model, c, logic, block)
}

fn prove_logic(model: &Path, c: &Path, logic: &Path) -> Output {
    arclang(&["drift", model.to_str().unwrap(), c.to_str().unwrap(), "--hdl", logic.to_str().unwrap(), "--prove", "--timeout", "1200"])
}

const FILTER: &str = "            r_v_filtered := fx_resize(fx_align(r_v_filtered, 2) + fx_align(fx_resize(signed'(x\"40000000\") * fx_resize(fx_align(r_v_raw, 0) - fx_align(r_v_filtered, 0), 0), 30), 0), 2);";

#[test]
#[ignore = "needs GHDL, Yosys and SymbiYosys: tools/embedded/formal/run.sh cargo test --test equivalence -- --include-ignored"]
fn a_vhdl_written_another_way_that_does_the_same_is_proven_equivalent() {
    let (_dir, model, c, logic, block) = built_logic(SPEED_MONITOR);
    // The two terms of the sum exchanged: other text, same logic.
    edit(&block, FILTER, "            r_v_filtered := fx_resize(fx_align(fx_resize(signed'(x\"40000000\") * fx_resize(fx_align(r_v_raw, 0) - fx_align(r_v_filtered, 0), 0), 30), 0) + fx_align(r_v_filtered, 2), 2);");
    let proven = prove_logic(&model, &c, &logic);
    let said = text(&proven);
    // The file differs, and drift says where...
    assert!(said.contains("speed_monitor_wheel_speed_filter.vhd:109: 1 line(s) of the file as built where the model generates 1"), "{said}");
    // ...and the proof says the logic does the same: the command succeeds.
    assert!(said.contains("Proven equivalent") && said.contains("entity speed_monitor_wheel_speed_filter: from reset and for any inputs"), "{said}");
    assert!(proven.status.success(), "{said}");
}

#[test]
#[ignore = "needs GHDL, Yosys and SymbiYosys: tools/embedded/formal/run.sh cargo test --test equivalence -- --include-ignored"]
fn a_vhdl_that_does_something_else_is_refuted_with_the_clock_and_the_outputs_that_show_it() {
    let (_dir, model, c, logic, block) = built_logic(SPEED_MONITOR);
    // The coefficient of the filter halved.
    edit(&block, "signed'(x\"40000000\") * fx_resize(fx_align(r_v_raw, 0)", "signed'(x\"20000000\") * fx_resize(fx_align(r_v_raw, 0)");
    let refuted = prove_logic(&model, &c, &logic);
    let said = text(&refuted);
    // Found by simulating the two side by side, before any proof is asked.
    assert!(said.contains("✗ Not equivalent") && said.contains("clock(s) after reset, on a run of random inputs simulated on both"), "{said}");
    assert!(said.contains("o_v_filtered = 0b") && said.contains("for the model"), "{said}");
    assert!(!refuted.status.success(), "{said}");
}

#[test]
#[ignore = "needs GHDL, Yosys and SymbiYosys: tools/embedded/formal/run.sh cargo test --test equivalence -- --include-ignored"]
fn a_vhdl_with_other_ports_or_that_is_not_vhdl_is_neither_proven_nor_refuted() {
    let (_dir, model, c, logic, block) = built_logic(SPEED_MONITOR);
    let original = std::fs::read_to_string(&block).unwrap();
    // One more port: another interface.
    edit(&block, "    rst", "    spare : in std_logic;\n    rst");
    let said = text(&prove_logic(&model, &c, &logic));
    assert!(said.contains("neither proven nor refuted") && said.contains("has other ports than the model's"), "{said}");
    // A statement GHDL refuses.
    std::fs::write(&block, original.replacen("r_v_filtered := fx_resize(", "r_v_filtered := fx_resiz(", 1)).unwrap();
    let asked = prove_logic(&model, &c, &logic);
    let said = text(&asked);
    assert!(said.contains("neither proven nor refuted") && said.contains("GHDL does not accept the VHDL as built"), "{said}");
    assert!(!asked.status.success(), "{said}");
}
