//! The plant in the loop (`execute: { plant: { ... } }`): the machine's
//! inputs come from a model of what it controls, the model's outputs go
//! back to it, a step at a time. These tests close the loop on a plant
//! written in Rust (no OpenModelica): the model's own loop, the generated
//! C in the loop (`arclang sil`), and the vectors the loop leaves for the
//! batch stages. The charger closed on its Modelica pack
//! (`examples/energy/battery-charger`, case TC-CHG-HIL) runs where
//! OpenModelica is: `cargo test --release --test embedded_plant --
//! --include-ignored`.

use arclang::compiler::embedded::plant::{self, Lag, Plants};
use arclang::compiler::embedded::{binding, plan, reference, sil};
use arclang::{Compiler, CompilerConfig};
use std::path::{Path, PathBuf};

const HEATER: &str = r#"
model Heater { metadata { version: "1.0.0" } }
requirements system {
    req "SYS-H-1" "The room warms" { description: "The heater shall bring the room near its setpoint and ease off" }
}
state_machine "Heater" {
    variables: ["in temperature: Real [K] range 0 K .. 400 K", "out power: Real [W] = 0 W", "const setpoint: Real [K] = 80 K", "const k_p: Real [W/K] = 10 W/K"]
    equations: ["power = max(0 W, min(100 W, k_p * (setpoint - temperature)))"]
    initial: "On"
    state On {}
}
test_case "TC-LOOP" {
    verifies: ["SYS-H-1"]
    execute: {
        machine: "Heater"
        step: 100 ms
        stop_time: 2 s
        plant: { fmu: "room.fmu" inputs: ["heat <- power"] outputs: ["temperature <- T"] }
    }
    expects: ["SYS-H-1: final(\"Heater\".temperature) > 50 K", "SYS-H-1: final(\"Heater\".power) < 100 W", "at(0 s, \"Heater\".power) == 100 W"]
}
"#;

fn work(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("arclang-plant-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn cc() -> String {
    std::env::var("CC").ok().filter(|c| !c.trim().is_empty()).unwrap_or_else(|| "cc".to_string())
}

#[test]
fn the_c_in_the_loop_reproduces_the_model_in_the_loop() {
    let ast = Compiler::new(CompilerConfig::default()).compile_string(HEATER).expect("compiles").ast;
    let mut plants = Plants::none();
    plants.inject("room.fmu", Box::new(Lag::new(0.5, 1)));
    let options = sil::Options { compiler: cc(), work: work("c-loop"), only: Some("TC-LOOP".into()) };
    let outcomes = sil::run_with(&ast, &options, &mut plants).unwrap();
    assert_eq!(outcomes.len(), 1);
    let o = &outcomes[0];
    assert_eq!(o.verdict, "pass", "{}", o.reason);
    assert!(o.compiler.contains("in the loop with the plant room.fmu"), "{}", o.compiler);
    let b2b = o.back_to_back.as_ref().unwrap();
    assert_eq!(b2b.first_mismatch, None, "the C's loop is the model's loop");
    assert_eq!(b2b.steps, 21);
    for j in &o.expectations {
        assert!(j.model.as_ref().unwrap().holds && j.c.as_ref().unwrap().holds, "{}: {:?} / {:?}", j.expect, j.model, j.c);
    }
}

#[test]
fn without_a_plant_a_case_with_one_says_what_is_missing() {
    let ast = Compiler::new(CompilerConfig::default()).compile_string(HEATER).expect("compiles").ast;
    let options = sil::Options { compiler: cc(), work: work("no-plant"), only: Some("TC-LOOP".into()) };
    let outcomes = sil::run(&ast, &options).unwrap();
    assert_eq!(outcomes[0].verdict, "error");
    assert!(outcomes[0].reason.contains("the plant needs the model's directory"), "{}", outcomes[0].reason);
}

#[test]
fn the_loop_leaves_vectors_a_batch_stage_replays() {
    let ast = Compiler::new(CompilerConfig::default()).compile_string(HEATER).expect("compiles").ast;
    let p = plan::plan(&ast).unwrap();
    let bindings = binding::resolve_all(&ast, &p).unwrap();
    let case = arclang::compiler::embedded::case::read(&ast.test_cases[0], &p, &bindings).unwrap().unwrap();
    let mut plants = Plants::none();
    plants.inject("room.fmu", Box::new(Lag::new(0.5, 1)));
    let (vectors, table) = plant::model_loop(&ast, &p, &case, &mut plants).unwrap();
    // The same case, the inputs as a list: what the loop gave at each step.
    let replay = reference::run(&p, &vectors, case.step_us);
    assert_eq!(replay.rows, table.rows);
    let stdin = sil::direct_stdin(&p, &vectors);
    assert_eq!(stdin.lines().count(), 22, "the steps, then one line a step");
}

// ---- with OpenModelica (Docker or a local omc) ----------------------------------------

fn example(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples").join(path)
}

#[test]
#[ignore]
fn the_charger_closed_on_its_pack_reaches_absorption() {
    let path = example("energy/battery-charger/battery_charger.arc");
    let result = Compiler::new(CompilerConfig::default()).compile_file(&path).unwrap();
    let mut plants = Plants::at(path.parent().unwrap());
    let options = sil::Options { compiler: cc(), work: work("charger-loop"), only: Some("TC-CHG-HIL".into()) };
    let outcomes = sil::run_with(&result.ast, &options, &mut plants).unwrap();
    let o = &outcomes[0];
    assert_eq!(o.verdict, "pass", "{}", o.reason);
    assert!(o.compiler.contains("ChargerPhysics.Battery"), "{}", o.compiler);
    let cached = Plants::cache_dir(path.parent().unwrap());
    assert!(std::fs::read_dir(&cached).unwrap().flatten().any(|e| e.path().extension().is_some_and(|x| x == "fmu")), "the pack's FMU is kept beside the model");
}
