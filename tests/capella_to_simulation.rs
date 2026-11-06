//! From an architecture authored in Capella to its simulation: Capella's
//! property values read back as quantities, `refine` adding to elements
//! imported from another file, the roll-up of the elements' executable
//! models (`Executable.<Configuration>`, relays for exchanges between
//! behavior components of different nodes), missions in files of their
//! own, and the Capella export Capella's validation accepts (component
//! exchanges between component ports, a root system). The run in
//! OpenModelica is `#[ignore]`d without Docker.

use arclang::compiler::modelica;
use arclang::{Compiler, CompilerConfig};
use std::path::{Path, PathBuf};
use std::process::Command;

const DEMO: &str = "tests/fixtures/capella-sim";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn work(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("arclang-capella-sim-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn compile(path: &Path) -> arclang::compiler::CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_file(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

#[test]
fn capella_property_values_come_back_as_quantities() {
    let dir = work("import");
    let out = dir.join("hoist.arc");
    let run = Command::new(env!("CARGO_BIN_EXE_arclang")).arg("import").arg(root().join(DEMO).join("capella/hoist.capella")).args(["-f", "capella", "-o"]).arg(&out).output().unwrap();
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    let text = std::fs::read_to_string(&out).unwrap();
    for wanted in [
        "energy: 60 Wh",
        "mass: 150 kg",
        "max_force: 6000 N",
        "efficiency: 80 %",
        // A requirement keeps the id the cases verify, and what satisfies it.
        "req \"REQ-HEAVY\" \"Heavy duty\"",
        "trace \"PN-BATT\" satisfies \"REQ-HEAVY\"",
        // Links and exchanges keep their properties and their carrying link.
        "protocol: \"CAN\"",
        "via: \"Sensor bus\" frequency: 50 Hz message_type: \"Height\"",
    ] {
        assert!(text.contains(wanted), "{wanted}");
    }
    assert!(!text.contains("pv_mass"), "a typed attribute keeps its name when its value is a quantity");
    // The import is the file the simulation refines.
    assert_eq!(text, std::fs::read_to_string(root().join(DEMO).join("hoist.arc")).unwrap(), "hoist.arc is what the import writes");
}

#[test]
fn refine_adds_to_an_element_declared_in_another_file() {
    let dir = work("refine");
    std::fs::write(
        dir.join("base.arc"),
        "model \"M\" {}\nphysical_architecture \"PA\" {\n    node \"Pump\" {\n        id: \"PN-PUMP\"\n        power: 2 kW\n        port \"inlet\" { id: \"p1\" }\n        port in \"cmd\" { id: \"p2\" }\n    }\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("sim.arc"),
        "import \"base.arc\"\ntype \"Fluid\" { physics: electrical }\nrefine \"PN-PUMP\" {\n    model: \"Lib.Pump\"\n    power: 3 kW\n    port \"inlet\" { is: \"Fluid\" }\n    port \"cmd\" { physics: signal }\n    port \"outlet\" { is: \"Fluid\" }\n}\n",
    )
    .unwrap();
    let result = compile(&dir.join("sim.arc"));
    let node = &result.ast.physical_architecture[0].nodes[0];
    assert_eq!(node.attributes.get("model").and_then(|v| v.as_string()), Some("Lib.Pump"));
    assert_eq!(node.attributes.get("power").map(|v| v.display()), Some("3 kW".to_string()), "a refinement overrides");
    let inlet = node.ports.iter().find(|p| p.name == "inlet").unwrap();
    assert_eq!(inlet.attributes.get("is").and_then(|v| v.as_string()), Some("Fluid"));
    assert!(node.component_ports.iter().find(|p| p.name == "cmd").unwrap().attributes.contains_key("physics"), "an oriented port is refined by name");
    assert!(node.ports.iter().any(|p| p.name == "outlet"), "a new port is added");
    std::fs::write(dir.join("bad.arc"), "import \"base.arc\"\nrefine \"PN-NONE\" { x: 1 }\n").unwrap();
    let error = Compiler::new(CompilerConfig::default()).compile_file(dir.join("bad.arc")).err().unwrap().to_string();
    assert!(error.contains("refine \"PN-NONE\": no physical node, behavior component or logical component"), "{error}");
}

#[test]
fn the_element_models_are_rolled_up_into_each_configuration() {
    let result = compile(&root().join(DEMO).join("simulation.arc"));
    let export = modelica::export(&result.ast).unwrap();
    assert_eq!(export.executables.get("CFG-LIGHT").map(String::as_str), Some("CargoHoist.Executable.LightDuty"));
    let text = &export.text;
    for wanted in [
        "extends CargoHoist.Systems.LightDuty(redeclare HoistPhysics.Pack battery,",
        "redeclare HoistPhysics.Winch winch(redeclare HoistPhysics.Drive winchDrive)",
        "redeclare HoistPhysics.Cabin cabin(redeclare HoistPhysics.HeightSensor heightSensor)",
        "extends CargoHoist.Physical.Controller(redeclare HoistPhysics.HeightHold heightHold);",
        "redeclare CargoHoist.Executable.Parts.Controller controller",
    ] {
        assert!(text.contains(wanted), "{wanted}");
    }
    // An exchange between behavior components of two nodes goes through relays of the nodes.
    assert!(text.contains("connect(heightSensor.height, heightSensor_height)"));
    assert!(text.contains("connect(cabin.heightSensor_height, controller.heightHold_height)"));
}

#[test]
fn a_mission_file_adds_its_values_to_the_sweep() {
    let result = compile(&root().join(DEMO).join("simulation.arc"));
    let export = modelica::export(&result.ast).unwrap();
    let test = result.ast.test_cases.iter().find(|t| t.id == "TC-ENVELOPE").unwrap();
    let spec = arclang::compiler::simulation::spec_of(&test.attributes).unwrap().unwrap();
    assert!(spec.model.is_empty() && spec.sweep.is_empty());
    let spec = arclang::compiler::simulation::effective(spec, &root().join(DEMO), &export, Some("CFG-HEAVY".into())).unwrap();
    assert_eq!(spec.model, "CargoHoist.Executable.HeavyDuty");
    assert_eq!(spec.sweep, vec!["\"PN-CABIN\".load = 0 kg .. 200 kg step 100 kg".to_string(), "\"PN-CABIN\".mass = 120 kg, 150 kg".to_string()]);
    let missing = arclang::compiler::simulation::spec_of(&test.attributes).unwrap().unwrap();
    let error = arclang::compiler::simulation::effective(missing, Path::new("/nowhere"), &export, Some("CFG-HEAVY".into())).err().unwrap();
    assert!(error.contains("mission 'missions/envelope.mission': cannot read"), "{error}");
}

#[test]
fn the_capella_export_is_what_capella_accepts_between_component_ports() {
    let dir = work("export");
    std::fs::write(
        dir.join("m.arc"),
        "model \"M\" {}\nsystem_analysis \"Needs\" { function \"Command\" { id: \"SF-CMD\" } }\nphysical_architecture \"PA\" {\n    node \"Box\" {\n        id: \"PN-BOX\"\n        port in \"cmd\" { id: \"p-in\" }\n        behavior_component \"Logic\" { id: \"BC-L\" port in \"cmd\" { id: \"p-bc\" } }\n    }\n    physical_link \"Into the logic\" { id: \"PL-IN\" from: \"PN-BOX.cmd\" to: \"BC-L.cmd\" }\n}\n",
    )
    .unwrap();
    let run = Command::new(env!("CARGO_BIN_EXE_arclang")).arg("export").arg(dir.join("m.arc")).args(["-f", "capella", "-o"]).arg(dir.join("m")).output().unwrap();
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    let capella = std::fs::read_to_string(dir.join("m.capella")).unwrap();
    assert!(capella.contains("fa:ComponentExchange\" id=\"PL-IN\""), "a link between component ports is a component exchange");
    assert!(!capella.contains("cs:PhysicalLink\" id=\"PL-IN\""));
    assert!(capella.contains("name=\"System\""), "the system analysis has its root system");
}

#[test]
fn a_copy_of_the_model_compiles_as_the_file_it_stands_for() {
    // An editor writes its buffer to a temporary file and names the real one
    // with --as: the imports resolve next to the real file.
    let dir = work("as");
    std::fs::copy(root().join(DEMO).join("simulation.arc"), dir.join("simulation.arc")).unwrap();
    let alone = Command::new(env!("CARGO_BIN_EXE_arclang")).arg("workbench").arg(dir.join("simulation.arc")).output().unwrap();
    assert!(String::from_utf8_lossy(&alone.stdout).contains("imported file not found: hoist.arc"), "a copy alone cannot resolve its imports");
    let run = Command::new(env!("CARGO_BIN_EXE_arclang")).arg("workbench").arg(dir.join("simulation.arc")).arg("--as").arg(root().join(DEMO).join("simulation.arc")).output().unwrap();
    let text = String::from_utf8_lossy(&run.stdout);
    let snapshot: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(snapshot["success"], true, "{text}");
    assert!(snapshot["diagrams"]["graphs"].as_array().unwrap().len() >= 7, "the diagrams of the whole model");
}

#[test]
#[ignore = "needs Docker and the OpenModelica image (arclang verify)"]
fn the_hoist_from_capella_flies_its_missions_in_openmodelica() {
    let dir = work("verify");
    let copy = Command::new("cp").args(["-R"]).arg(root().join(DEMO)).arg(&dir).output().unwrap();
    assert!(copy.status.success());
    let model = dir.join("capella-sim").join("simulation.arc");
    let run = Command::new(env!("CARGO_BIN_EXE_arclang")).arg("verify").arg(&model).output().unwrap();
    let text = String::from_utf8_lossy(&run.stdout);
    assert!(run.status.success(), "{text}{}", String::from_utf8_lossy(&run.stderr));
    for wanted in ["TC-LIGHT \"Light duty, small pack\": PASS", "TC-HEAVY \"Heavy duty, large pack\": PASS", "sweep: 6 points, 6 pass"] {
        assert!(text.contains(wanted), "{wanted}\n{text}");
    }
}
