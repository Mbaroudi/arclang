//! The validation campaign: five systems from five domains, each with its
//! architecture in ArcLang, its physics in Modelica and its requirements
//! verified by simulation, one configuration meant to pass, one meant to
//! fail, and a sweep over the conditions the requirement names.
//!
//! The first tests need no tool: every system compiles, exports, and its
//! physics extends the system model of each configuration it simulates.
//! The last runs OpenModelica for real (Docker or ARCLANG_OMC) and checks
//! every verdict; it is ignored by default:
//! `cargo test --release --test simulation_campaign -- --ignored --nocapture`.

use arclang::compiler::modelica;
use arclang::compiler::simulation::{run, tool::ToolConfig};
use arclang::compiler::{Compiler, CompilerConfig};
use std::path::{Path, PathBuf};

struct System {
    model: &'static str,
    physics: &'static str,
    package: &'static str,
    /// Each test case and the verdict the physics gives it.
    verdicts: &'static [(&'static str, &'static str)],
}

const CAMPAIGN: [System; 5] = [
    System {
        model: "examples/automotive/ev_range_simulation.arc",
        physics: "physics/EVPhysics.mo",
        package: "EVRangeSimulation",
        verdicts: &[("TC-RANGE-STD", "fail"), ("TC-RANGE-LR", "pass"), ("TC-RANGE-ENVELOPE", "pass")],
    },
    System {
        model: "examples/automotive/battery_fast_charge.arc",
        physics: "physics/FastChargePhysics.mo",
        package: "BatteryFastCharge",
        verdicts: &[("TC-CHARGE-AIR", "fail"), ("TC-CHARGE-CHILLED", "pass"), ("TC-HOT-ENVELOPE", "pass")],
    },
    System {
        model: "examples/aerospace/drone_endurance.arc",
        physics: "physics/DronePhysics.mo",
        package: "DroneEndurance",
        verdicts: &[("TC-HOVER-STD", "fail"), ("TC-HOVER-EXT", "pass"), ("TC-ENVELOPE", "pass")],
    },
    System {
        model: "examples/building/heat_pump_heating.arc",
        physics: "physics/HeatingPhysics.mo",
        package: "HeatPumpHeating",
        verdicts: &[("TC-DAY-COMPACT", "fail"), ("TC-DAY-STANDARD", "pass"), ("TC-CLIMATE", "pass")],
    },
    System {
        model: "examples/aerospace/aircraft_electrical_network.arc",
        physics: "physics/ElectricalNetworkPhysics.mo",
        package: "AircraftElectricalNetwork",
        verdicts: &[("TC-LOSS-25", "fail"), ("TC-LOSS-40", "pass"), ("TC-LOSS-ENVELOPE", "fail")],
    },
];

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn compile(path: &Path) -> arclang::CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_file(path).unwrap_or_else(|e| panic!("{} compiles: {e}", path.display()))
}

/// The system copied to a fresh directory with its physics: no cached verdict there.
fn fresh(system: &System) -> (tempfile::TempDir, PathBuf) {
    let source = root().join(system.model);
    let dir = tempfile::tempdir().unwrap();
    let model = dir.path().join(source.file_name().unwrap());
    std::fs::copy(&source, &model).unwrap();
    std::fs::create_dir_all(dir.path().join("physics")).unwrap();
    std::fs::copy(source.parent().unwrap().join(system.physics), dir.path().join(system.physics)).unwrap();
    (dir, model)
}

#[test]
fn every_system_compiles_and_exports_with_a_system_model_per_configuration() {
    for system in &CAMPAIGN {
        let (_dir, model) = fresh(system);
        let result = compile(&model);
        let export = modelica::export(&result.ast).unwrap_or_else(|e| panic!("{}: {e}", system.model));
        assert_eq!(export.file_name, format!("{}.mo", system.package), "{}", system.model);
        assert!(export.notes.is_empty(), "{}: nothing left out of the export: {:?}", system.model, export.notes);
        assert_eq!(export.systems.len(), result.ast.configurations.len(), "{}", system.model);
        // Two configurations: one meant to pass, one meant to fail.
        assert_eq!(result.ast.configurations.len(), 2, "{}", system.model);
    }
}

#[test]
fn every_physics_package_extends_the_system_of_each_configuration_it_simulates() {
    for system in &CAMPAIGN {
        let (dir, model) = fresh(system);
        let result = compile(&model);
        let export = modelica::export(&result.ast).unwrap();
        let physics = std::fs::read_to_string(dir.path().join(system.physics)).unwrap();
        for class in export.systems.values() {
            assert!(physics.contains(&format!("extends {class}(")), "{}: no physics extends {class}", system.physics);
        }
        assert!(physics.contains("annotation(uses(Modelica(version=\"4.1.0\")))"), "{}: pins the Modelica library", system.physics);
    }
}

#[test]
fn every_case_is_simulated_and_every_requirement_is_verified_by_one() {
    for system in &CAMPAIGN {
        let (_dir, model) = fresh(system);
        let result = compile(&model);
        let simulated = &result.semantic_model.simulations;
        let mut ids: Vec<&str> = simulated.keys().map(String::as_str).collect();
        ids.sort_unstable();
        let mut expected: Vec<&str> = system.verdicts.iter().map(|(id, _)| *id).collect();
        expected.sort_unstable();
        assert_eq!(ids, expected, "{}", system.model);
        // Never simulated here: inconclusive, never a pass.
        assert!(simulated.values().all(|s| s.verdict == "inconclusive"), "{}", system.model);
        let verified: std::collections::HashSet<&str> = result.ast.test_cases.iter().flat_map(|t| t.verifies.iter().map(String::as_str)).collect();
        for requirement in result.ast.system_analysis.iter().flat_map(|sa| sa.requirements.iter()) {
            // The id written in the block (`id: "REQ-RANGE"`), the name otherwise.
            let id = match requirement.attributes.get("id") {
                Some(arclang::compiler::ast::AttributeValue::String(id)) => id.as_str(),
                _ => requirement.id.as_str(),
            };
            assert!(verified.contains(id), "{}: {id} is verified by no test case", system.model);
        }
    }
}

#[test]
#[ignore = "runs OpenModelica (Docker or ARCLANG_OMC): cargo test --release --test simulation_campaign -- --ignored --nocapture"]
fn openmodelica_gives_every_case_of_the_campaign_its_verdict() {
    let config = ToolConfig::from_env();
    for system in &CAMPAIGN {
        let (_dir, model) = fresh(system);
        let outcomes = run::verify(&compile(&model), &model, None, &config).unwrap();
        println!("{}", system.model);
        for outcome in &outcomes {
            println!("  {} {}: {}", outcome.test_case, outcome.verdict, outcome.reason);
            assert!(outcome.seconds > 0.0, "{} was simulated", outcome.test_case);
        }
        for (id, verdict) in system.verdicts {
            let outcome = outcomes.iter().find(|o| o.test_case == *id).unwrap_or_else(|| panic!("{id} ran"));
            assert_eq!(outcome.verdict, *verdict, "{id}: {}", outcome.reason);
            // A pass holds every expectation, at every point of a sweep.
            if *verdict == "pass" {
                assert!(outcome.values.iter().all(|v| v.holds) && outcome.points.iter().all(|p| p.holds && p.error.is_empty()), "{id}");
            }
        }
        // The verdicts are cached: the gate reads them without a tool.
        let cached = compile(&model);
        for (id, verdict) in system.verdicts {
            assert_eq!(cached.semantic_model.simulations[*id].verdict, *verdict, "{id} cached");
        }
    }
}

#[test]
#[ignore = "runs OpenModelica (Docker or ARCLANG_OMC): cargo test --release --test simulation_campaign -- --ignored --nocapture"]
fn a_sweep_finds_the_corner_where_the_design_is_not_enough() {
    let system = &CAMPAIGN[4];
    let (_dir, model) = fresh(system);
    let outcomes = run::verify(&compile(&model), &model, Some("TC-LOSS-ENVELOPE"), &ToolConfig::from_env()).unwrap();
    let envelope = &outcomes[0];
    assert_eq!(envelope.points.len(), 9);
    let failing: Vec<&str> = envelope.points.iter().filter(|p| !p.holds).map(|p| p.label.as_str()).collect();
    assert_eq!(failing.len(), 5, "{failing:?}");
    assert!(envelope.reason.starts_with("5 of 9 points fail; first at temperature = 253.15 K, power = 550 W"), "{}", envelope.reason);
    // Warm and light, the 40 Ah battery is enough.
    assert!(envelope.points.iter().any(|p| p.holds && p.label == "temperature = 293.15 K, power = 550 W"));
}
