//! Data flow between machines (docs/embedded-c.md, "Data flow between
//! machines"): a machine with equations and no state is a data-flow block;
//! an input `from "Machine".variable` reads another machine every step,
//! after it. The model, the C and the fixed point read the chain alike.
//! The VHDL of the speed monitor's block runs where GHDL is (the FPGA
//! image): `cargo test --release --test dataflow -- --include-ignored`.

use arclang::compiler::embedded::{binding, case, glue, hdl, plan, reference, sil};
use arclang::{Compiler, CompilerConfig};
use std::path::{Path, PathBuf};

const MONITOR: &str = "automotive/speed-monitor/speed_monitor.arc";

fn example(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples").join(path)
}

fn text() -> String {
    std::fs::read_to_string(example(MONITOR)).unwrap()
}

fn compile(text: &str) -> Result<arclang::compiler::CompilationResult, String> {
    Compiler::new(CompilerConfig::default()).compile_string(text).map_err(|e| e.to_string())
}

fn work(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("arclang-dataflow-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn cc() -> String {
    std::env::var("CC").ok().filter(|c| !c.trim().is_empty()).unwrap_or_else(|| "cc".to_string())
}

#[test]
fn a_machine_with_equations_and_no_state_is_a_block_stepped_in_one_state() {
    let ast = compile(&text()).unwrap().ast;
    let p = plan::plan(&ast).unwrap();
    let block = &p.machines[0];
    assert_eq!(block.name, "Wheel speed filter");
    assert!(block.dataflow);
    assert_eq!(block.states.iter().map(|s| s.path.as_str()).collect::<Vec<_>>(), vec![plan::FLOW_STATE]);
    assert!(!p.machines[1].dataflow);
    // Without equations, still nothing to generate.
    let empty = "model E { metadata { version: \"1.0.0\" } }\nrequirements system { req \"R-1\" \"R\" { description: \"R\" } }\nstate_machine \"Empty\" { variables: [\"in x: Real\"] }\ntest_case \"T\" { verifies: [\"R-1\"] execute: { stop_time: 10 ms } expects: [\"R-1: final(\\\"Empty\\\".x) == 0\"] }\n";
    let e = compile(empty).map(|_| ()).unwrap_err();
    assert!(e.contains("has no state and no equation: nothing to generate"), "{e}");
    // The state's name is taken.
    let taken = text().replace("\"out speed: Real [m/s]", "\"Flow: Real = 0\",\n        \"out speed: Real [m/s]");
    let e = compile(&taken).map(|_| ()).unwrap_err();
    assert!(e.contains("variable 'Flow' has the name of the state a data-flow block is stepped in"), "{e}");
}

#[test]
fn a_wire_reads_the_other_machine_after_its_step_converted_to_the_inputs_unit() {
    let ast = compile(&text()).unwrap().ast;
    let p = plan::plan(&ast).unwrap();
    let warning = &p.machines[1];
    assert_eq!(warning.wires.len(), 1);
    let w = &warning.wires[0];
    assert_eq!((w.machine, p.machines[w.machine].program().variables[w.variable].name.as_str()), (0, "speed"));
    // The range of what it reads, in its own unit: the fixed point follows the chain.
    let (lo, hi) = warning.program().variables[w.input].range.unwrap();
    assert!(lo == 0.0 && (hi - 252.0).abs() < 1e-9, "{lo} .. {hi}");
    // Written after its reader, the block still steps first.
    let swapped = {
        let t = text();
        let start = t.find("// The data-flow block").unwrap();
        let end = t.find("state_machine \"Overspeed warning\"").unwrap();
        let block = t[start..end].to_string();
        let rest = t.replacen(&block, "", 1);
        let at = rest.find("test_case \"TC-SPD-001\"").unwrap();
        format!("{}{}{}", &rest[..at], block, &rest[at..])
    };
    let p2 = plan::plan(&compile(&swapped).unwrap().ast).unwrap();
    assert_eq!(p2.machines.iter().map(|m| m.name.as_str()).collect::<Vec<_>>(), vec!["Wheel speed filter", "Overspeed warning"]);
    // The model: 140 km/h, 0.25 of it in the first step, read in km/h by the warning in that same step.
    let bindings = binding::resolve_all(&ast, &p).unwrap();
    let c = case::read(&ast.test_cases[0], &p, &bindings).unwrap().unwrap();
    let vectors = c.vectors(&p);
    let table = reference::run(&p, &vectors, c.step_us);
    let column = table.names.iter().position(|n| n == "overspeed_warning.speed").unwrap();
    assert!((table.rows[0][column] - 35.0).abs() < 1e-9 && (table.rows[1][column] - 61.25).abs() < 1e-9, "{:?}", &table.rows[..2]);
    // What a stage running the warning alone replays: the values the wire gave.
    let replay = reference::wired(&p, &vectors, c.step_us);
    let speed = w.input;
    assert!((replay.values[1][speed][1].number() - 61.25).abs() < 1e-9);
    assert_eq!(vectors.values[1][speed][1].number(), 0.0, "the case's own vectors are left as they are");
}

#[test]
fn the_c_steps_the_chain_as_the_model_does() {
    let result = Compiler::new(CompilerConfig::default()).compile_file(&example(MONITOR)).unwrap();
    let p = plan::plan(&result.ast).unwrap();
    let generated = arclang::compiler::embedded::cgen::generate(&p, 10_000);
    assert!(generated.source.contains("speed_monitor_overspeed_warning_inputs wired = inputs->overspeed_warning;"), "{}", generated.source);
    assert!(generated.source.contains("wired.v_speed = (outputs->wheel_speed_filter.v_speed * 3.5999999999999996);"), "{}", generated.source);
    assert!(generated.source.contains("overspeed_warning_run(&state->overspeed_warning, &wired, &outputs->overspeed_warning);"));
    let options = sil::Options { compiler: cc(), work: work("sil"), only: None };
    let outcomes = sil::run(&result.ast, &options).unwrap();
    assert_eq!(outcomes.len(), 3);
    for o in &outcomes {
        assert_eq!(o.verdict, "pass", "{}: {}", o.test_case, o.reason);
        assert_eq!(o.back_to_back.as_ref().unwrap().first_mismatch, None, "{}", o.test_case);
    }
}

#[test]
fn what_cannot_be_wired_is_said() {
    let bad = |from: &str, to: &str| compile(&text().replace(from, to)).map(|_| ()).unwrap_err();
    let wire = "\"in speed: Real [km/h] from \\\"Wheel speed filter\\\".speed\"";
    let e = bad(wire, "\"in speed: Real [km/h] from \\\"Wheel filter\\\".speed\"");
    assert!(e.contains("reads from 'Wheel filter', which is no state_machine of the model"), "{e}");
    let e = bad(wire, "\"in speed: Real [km/h] from \\\"Wheel speed filter\\\".velocity\"");
    assert!(e.contains("reads 'velocity', which is no variable of state_machine 'Wheel speed filter'"), "{e}");
    let e = bad(wire, "\"in speed: Real [km/h] from \\\"Overspeed warning\\\".on_above\"");
    assert!(e.contains("reads from its own machine"), "{e}");
    let extra = |line: &str| bad("\"const on_above", &format!("{line},\n        \"const on_above"));
    let e = extra("\"in amps: Real [A] from \\\"Wheel speed filter\\\".speed\"");
    assert!(e.contains("'speed' is a speed, 'amps' a") && e.contains("current"), "{e}");
    let e = extra("\"in count: Integer [km/h] from \\\"Wheel speed filter\\\".speed\"");
    assert!(e.contains("a Real read into Integer 'count'"), "{e}");
    let e = extra("\"lag: Real [km/h] from \\\"Wheel speed filter\\\".speed\"");
    assert!(e.contains("only an input (in) reads another machine's variable"), "{e}");
    // A loop: the block reads the warning back.
    let e = bad("\"in raw: Real [km/h] range 0 km/h .. 300 km/h\"", "\"in raw: Real [km/h] from \\\"Overspeed warning\\\".speed\"");
    assert!(e.contains("make a loop"), "{e}");
    // A case does not set what a wire sets.
    let e = bad("inputs: [\"0 s: \\\"Wheel speed filter\\\".raw = 140 km/h\"]", "inputs: [\"0 s: speed = 140 km/h\"]");
    assert!(e.contains("'speed' is read from state_machine 'Wheel speed filter'"), "{e}");
}

const BINDING: &str = "\nio_binding \"Dashboard\" {\n    machine: \"Overspeed warning\"\n    target: \"zephyr/litex_vexriscv\"\n    period: 10 ms\n    outputs: [\"Warning -> gpio 20\"]\n}\n";

#[test]
fn a_task_steps_the_blocks_its_machine_reads_before_it() {
    let ast = compile(&(text() + BINDING)).unwrap().ast;
    let p = plan::plan(&ast).unwrap();
    let bindings = binding::resolve_all(&ast, &p).unwrap();
    let g = glue::generate(&p, &bindings[0]);
    let block = g.source.find("speed_monitor_wheel_speed_filter_step(state, inputs, outputs);").expect(&g.source);
    let machine = g.source.find("speed_monitor_overspeed_warning_step(state, inputs, outputs);").unwrap();
    assert!(block < machine, "{}", g.source);
    // A block run by a task of its own: the wire would cross two tasks.
    let two = text() + BINDING + "\nio_binding \"Sensor\" {\n    machine: \"Wheel speed filter\"\n    target: \"zephyr/litex_vexriscv\"\n    period: 5 ms\n    outputs: [\"Flow -> gpio 21\"]\n}\n";
    let e = compile(&two).map(|_| ()).unwrap_err();
    assert!(e.contains("which io_binding 'Sensor' runs as a task of its own: a wire between two tasks needs a rate transition"), "{e}");
}

// ---- in the FPGA image (GHDL) ------------------------------------------------------------

fn assert_bit_exact(outcomes: &[hdl::Outcome]) {
    assert!(!outcomes.is_empty());
    for o in outcomes {
        assert_eq!(o.verdict, "pass", "{}: {}", o.test_case, o.reason);
        let b = o.bit_exact.as_ref().expect("compared");
        assert_eq!(b.first_mismatch, None, "{}", o.test_case);
    }
}

#[test]
#[ignore]
fn the_block_runs_in_ghdl_bit_exact_with_its_fixed_point_c() {
    let ast = Compiler::new(CompilerConfig::default()).compile_file(&example(MONITOR)).unwrap().ast;
    let options = hdl::Options { work: work("hdl"), only: None, synth: true, compiler: "gcc".into(), as_if: None, machine: None };
    let (outcomes, synthesis) = hdl::run(&ast, &options).unwrap();
    assert_eq!(outcomes.iter().map(|o| o.test_case.as_str()).collect::<Vec<_>>(), vec!["TC-SPD-003"], "the block's case; the warning is software");
    assert_bit_exact(&outcomes);
    assert!(synthesis.iter().all(|s| s.as_ref().is_ok_and(|s| s.fmax_mhz > s.clock_mhz)), "{synthesis:?}");
}

#[test]
#[ignore]
fn a_block_with_a_wired_input_replays_what_the_wire_gave_it() {
    // Both in the logic: the warning's speed is the filter's, as the model's run gave it.
    let both = text().replace("owner: \"PC-SPD-WARN\"", "owner: \"PC-SPD-FILTER\"");
    let ast = compile(&both).unwrap().ast;
    let options = hdl::Options { work: work("hdl-wired"), only: None, synth: false, compiler: "gcc".into(), as_if: None, machine: None };
    let (outcomes, _) = hdl::run(&ast, &options).unwrap();
    assert_eq!(outcomes.len(), 3);
    assert_bit_exact(&outcomes);
}
