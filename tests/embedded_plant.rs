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

// ---- an FMU from another tool, FMI 2.0 and FMI 3.0 -----------------------------------

const ROOM_C: &str = include_str!("fixtures/fmu/room.c");

/// The room of `tests/fixtures/fmu/room.c` as an FMU of `version` (2 or 3),
/// its binary for this computer under the directory that version names it
/// (FMI 3.0 `aarch64-darwin`, FMI 2.0 `darwin64`, ...): `<dir>/room.fmu`.
fn room_fmu(version: u32, dir: &Path) -> PathBuf {
    use arclang::compiler::simulation::twin_step::host_platforms;
    let source = dir.join("room.c");
    std::fs::write(&source, ROOM_C).unwrap();
    let library = dir.join("Room.lib");
    let out = std::process::Command::new(cc()).args(["-std=c99", "-O2", "-fPIC", "-shared", "-fvisibility=hidden", "-Wall", "-Wextra", "-Werror"]).arg(format!("-DFMI_VERSION={version}")).arg(&source).arg("-o").arg(&library).arg("-lm").output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let platform = host_platforms()[if version == 3 { 0 } else { 1 }];
    let extension = if cfg!(target_os = "macos") { "dylib" } else { "so" };
    let description = if version == 3 {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<fmiModelDescription fmiVersion="3.0" modelName="Room" instantiationToken="{room-3}" generationTool="tests/fixtures/fmu/room.c">
  <CoSimulation modelIdentifier="Room" canHandleVariableCommunicationStepSize="true"/>
  <TypeDefinitions>
    <Float64Type name="Temperature" unit="K"/>
  </TypeDefinitions>
  <ModelVariables>
    <Float64 name="time" valueReference="0" causality="independent" variability="continuous"/>
    <Float64 name="heat" valueReference="1" causality="input" variability="continuous" start="0" unit="W"/>
    <Float64 name="T" valueReference="2" causality="output" variability="continuous" declaredType="Temperature"/>
    <Float64 name="der(heat)" valueReference="3" causality="input" variability="continuous" start="0" derivative="1"/>
    <Int32 name="mode" valueReference="4" causality="parameter" variability="fixed" start="1"/>
  </ModelVariables>
  <ModelStructure>
    <Output valueReference="2"/>
  </ModelStructure>
</fmiModelDescription>
"#
    } else {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<fmiModelDescription fmiVersion="2.0" modelName="Room" guid="{room-2}" generationTool="tests/fixtures/fmu/room.c">
  <CoSimulation modelIdentifier="Room" canHandleVariableCommunicationStepSize="true" canInterpolateInputs="true" maxOutputDerivativeOrder="0"/>
  <ModelVariables>
    <ScalarVariable name="heat" valueReference="1" causality="input" variability="continuous"><Real start="0" unit="W"/></ScalarVariable>
    <ScalarVariable name="T" valueReference="2" causality="output" variability="continuous" initial="calculated"><Real unit="K"/></ScalarVariable>
    <ScalarVariable name="mode" valueReference="4" causality="parameter" variability="fixed"><Integer start="1"/></ScalarVariable>
  </ModelVariables>
  <ModelStructure>
    <Outputs><Unknown index="2"/></Outputs>
  </ModelStructure>
</fmiModelDescription>
"#
    };
    let entries = vec![
        ("modelDescription.xml".to_string(), description.as_bytes().to_vec()),
        (format!("binaries/{platform}/Room.{extension}"), std::fs::read(&library).unwrap()),
        ("sources/room.c".to_string(), ROOM_C.as_bytes().to_vec()),
    ];
    let fmu = dir.join("room.fmu");
    std::fs::write(&fmu, arclang::compiler::tables::xlsx::zip(&entries)).unwrap();
    fmu
}

#[test]
fn an_fmu_of_another_tool_is_stepped_fmi_2_and_fmi_3_alike() {
    use arclang::compiler::simulation::tool::ToolConfig;
    use arclang::compiler::simulation::twin_step::Model;
    for version in [2, 3] {
        let dir = work(&format!("room-{version}"));
        let fmu = room_fmu(version, &dir);
        let mut room = Model::from_fmu(&fmu, vec!["heat".into()], vec!["T".into()], None, &ToolConfig::default()).unwrap_or_else(|e| panic!("FMI {version}: {e}"));
        assert!(room.tool_text.contains("this computer's C compiler"), "FMI {version}: {}", room.tool_text);
        assert_eq!(room.units.get("T").map(String::as_str), Some("K"), "FMI {version}: {:?}", room.units);
        assert_eq!(room.init(0.0, &[0.0]).unwrap(), vec![0.0]);
        // The heat goes from 0 along a slope of 2 W/s over the step: the
        // room follows the ramp, not a held input (T = 1 + e^-2 at 1 s).
        let t = room.step(1.0, &[0.0], &[2.0]).unwrap()[0];
        assert!((t - (1.0 + (-2.0f64).exp())).abs() < 1e-12, "FMI {version}: {t}");
        // Held from there: 2 W for 1 s.
        let t2 = room.step(2.0, &[2.0], &[0.0]).unwrap()[0];
        assert!((t2 - (2.0 + (t - 2.0) * (-2.0f64).exp())).abs() < 1e-12, "FMI {version}: {t2}");
        // A variable that is not there, one that is not real: said.
        let e = Model::from_fmu(&fmu, vec!["heat".into()], vec!["nope".into()], None, &ToolConfig::default()).err().unwrap();
        assert!(e.contains("no such variable in the FMU: nope"), "FMI {version}: {e}");
        let e = Model::from_fmu(&fmu, vec!["mode".into()], vec!["T".into()], None, &ToolConfig::default()).err().unwrap();
        assert!(e.contains(if version == 3 { "not a Float64 variable: mode" } else { "not a Real variable: mode" }), "FMI {version}: {e}");
    }
}

#[test]
fn the_heater_closes_its_loop_on_an_fmi_3_room_as_on_the_same_room_in_rust() {
    let ast = Compiler::new(CompilerConfig::default()).compile_string(HEATER).expect("compiles").ast;
    let mut lag = Plants::none();
    lag.inject("room.fmu", Box::new(Lag::new(0.5, 1)));
    let p = plan::plan(&ast).unwrap();
    let bindings = binding::resolve_all(&ast, &p).unwrap();
    let case = arclang::compiler::embedded::case::read(&ast.test_cases[0], &p, &bindings).unwrap().unwrap();
    let (_, in_rust) = plant::model_loop(&ast, &p, &case, &mut lag).unwrap();
    for version in [2, 3] {
        let dir = work(&format!("heater-room-{version}"));
        room_fmu(version, &dir);
        let mut plants = Plants::at(&dir);
        let (_, table) = plant::model_loop(&ast, &p, &case, &mut plants).unwrap_or_else(|e| panic!("FMI {version}: {e}"));
        assert_eq!(table.rows.len(), in_rust.rows.len());
        for (k, (a, b)) in table.rows.iter().zip(&in_rust.rows).enumerate() {
            // The same lag, written two ways: equal to the last bits.
            assert!(a.iter().zip(b).all(|(x, y)| (x - y).abs() <= 1e-9 * (1.0 + y.abs())), "FMI {version}, step {k}: {a:?} against {b:?}");
        }
        let options = sil::Options { compiler: cc(), work: dir.join("sil"), only: Some("TC-LOOP".into()) };
        let outcomes = sil::run_with(&ast, &options, &mut plants).unwrap();
        assert_eq!(outcomes[0].verdict, "pass", "FMI {version}: {}", outcomes[0].reason);
        assert_eq!(outcomes[0].back_to_back.as_ref().unwrap().first_mismatch, None);
    }
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
