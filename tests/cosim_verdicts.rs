//! The verdicts `arclang cosim` keeps beside the model (`<case>.cosim.json`)
//! are evidence of the SIL stage for `check`, the gate and the tables: the
//! same generated C, run as an FMU under OMSimulator. Nothing runs here:
//! the verdicts are written as the command writes them.

use arclang::compiler::embedded::cosim;
use arclang::compiler::simulation::cache;
use arclang::compiler::verification::{verdict_for, Verdict};
use arclang::{Compiler, CompilerConfig};
use std::path::{Path, PathBuf};

const SPEED: &str = "automotive/speed-monitor/speed_monitor.arc";

fn example(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples").join(path)
}

fn compile_file(path: &Path) -> arclang::compiler::CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_file(path).unwrap_or_else(|e| panic!("{} compiles: {e}", path.display()))
}

fn work(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("arclang-cosim-verdicts-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A verdict as `arclang cosim` caches it for `test`, with `verdict`.
fn cached(model: &Path, test: &str, verdict: &str) {
    let result = compile_file(model);
    let case = result.ast.test_cases.iter().find(|t| t.id == test).unwrap();
    let key = cosim::cache_key(&result.ast, case, model.parent().unwrap()).unwrap();
    let outcome = arclang::compiler::simulation::Outcome {
        test_case: test.to_string(),
        kind: "cosim".to_string(),
        name: case.name.clone(),
        verifies: case.verifies.clone(),
        verdict: verdict.to_string(),
        reason: if verdict == "pass" { String::new() } else { "co-simulation with OMSimulator v2.1.2: the OMSimulator run departs from the model at 20 ms".to_string() },
        key,
        model: "state_machine Overspeed warning".to_string(),
        tool: "arclang cosim (OMSimulator v2.1.2)".to_string(),
        ..Default::default()
    };
    cache::write_kind(&cache::dir(model), "cosim", &outcome).unwrap();
}

fn status<'a>(result: &'a arclang::compiler::CompilationResult, test: &str) -> &'a arclang::compiler::simulation::Status {
    result.semantic_model.simulations.get(test).unwrap_or_else(|| panic!("{test} has a status: {:?}", result.semantic_model.simulations.keys().collect::<Vec<_>>()))
}

#[test]
fn a_fresh_co_simulation_pass_stands_for_the_sil_verdict_of_a_case_never_run_in_sil() {
    let dir = work("pass");
    let model = dir.join("speed_monitor.arc");
    std::fs::copy(example(SPEED), &model).unwrap();
    // Never run: the case is inconclusive, its SIL rung missing.
    let before = compile_file(&model);
    let test = before.ast.test_cases.iter().find(|t| t.id == "TC-SPD-001").unwrap().clone();
    assert_eq!(status(&before, "TC-SPD-001").verdict, "inconclusive");
    assert!(matches!(verdict_for(&test, "SYS-SPD-001", &before.semantic_model), Verdict::Inconclusive(_)), "{:?}", verdict_for(&test, "SYS-SPD-001", &before.semantic_model));

    cached(&model, "TC-SPD-001", "pass");
    let result = compile_file(&model);
    let status = status(&result, "TC-SPD-001");
    assert_eq!(status.verdict, "pass", "{status:?}");
    assert_eq!(verdict_for(&test, "SYS-SPD-001", &result.semantic_model), Verdict::Pass);
    let sil: Vec<_> = status.rungs.iter().filter(|r| r.stage == "sil").collect();
    assert_eq!(sil.len(), 1, "one rung for the SIL stage: {:?}", status.rungs);
    assert!(sil[0].state == "fresh" && sil[0].verdict == "pass" && sil[0].target == "OMSimulator v2.1.2", "{:?}", sil[0]);
    assert!(!result.warnings.iter().any(|w| w.contains("co-simulation")), "{:?}", result.warnings);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_co_simulation_failure_is_the_case_s_verdict_and_a_stale_one_a_warning() {
    let dir = work("fail");
    let model = dir.join("speed_monitor.arc");
    std::fs::copy(example(SPEED), &model).unwrap();
    cached(&model, "TC-SPD-001", "fail");
    let result = compile_file(&model);
    let test = result.ast.test_cases.iter().find(|t| t.id == "TC-SPD-001").unwrap().clone();
    assert_eq!(status(&result, "TC-SPD-001").verdict, "fail");
    assert!(matches!(verdict_for(&test, "SYS-SPD-001", &result.semantic_model), Verdict::Fail(_)));
    assert!(result.warnings.iter().any(|w| w.contains("fails in co-simulation in OMSimulator v2.1.2")), "{:?}", result.warnings);

    // The case changes: the verdict no longer holds, the case is inconclusive again, and says why.
    let text = std::fs::read_to_string(&model).unwrap().replace("\"0 s: \\\"Wheel speed filter\\\".raw = 140 km/h\"]\n    }\n    expects: [\n        \"SYS-SPD-001: at(0 s", "\"0 s: \\\"Wheel speed filter\\\".raw = 150 km/h\"]\n    }\n    expects: [\n        \"SYS-SPD-001: at(0 s");
    assert_ne!(text, std::fs::read_to_string(&model).unwrap(), "the case was edited");
    std::fs::write(&model, text).unwrap();
    let result = compile_file(&model);
    assert_eq!(status(&result, "TC-SPD-001").verdict, "inconclusive");
    assert!(result.warnings.iter().any(|w| w.contains("its co-simulation verdict is stale") && w.contains("arclang cosim")), "{:?}", result.warnings);
    let _ = std::fs::remove_dir_all(&dir);
}

/// The browser has no file system: the workbench snapshot is given the
/// folder's files, and reads the verdicts from them as from the disk.
#[test]
fn the_snapshot_given_the_folder_s_files_reads_the_verdicts_kept_there() {
    use arclang::compiler::workbench::{snapshot, snapshot_with_files, SnapshotOptions};
    let dir = work("snapshot");
    let model = dir.join("speed_monitor.arc");
    std::fs::copy(example(SPEED), &model).unwrap();
    cached(&model, "TC-SPD-001", "pass");
    cached(&model, "TC-SPD-002", "pass");
    cached(&model, "TC-SPD-003", "pass");
    let text = std::fs::read_to_string(&model).unwrap();
    let options = SnapshotOptions::default();
    // Nothing given: the cases are never run, the gate wants them.
    let bare = snapshot(&text, &options);
    let gate = bare.gate.as_ref().unwrap();
    assert!(gate.requirements_verified < gate.requirements_total, "{}/{}", gate.requirements_verified, gate.requirements_total);
    // The folder's files given as the page holds them, by path relative to the model.
    let cache = dir.join(".arclang/sim-cache");
    let files: std::collections::HashMap<String, String> = std::fs::read_dir(&cache)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .map(|name| (format!(".arclang/sim-cache/{name}"), std::fs::read_to_string(cache.join(&name)).unwrap()))
        .collect();
    assert_eq!(files.len(), 3);
    let with = snapshot_with_files(&text, &options, files);
    let gate = with.gate.as_ref().unwrap();
    assert!(gate.requirements_verified > bare.gate.as_ref().unwrap().requirements_verified, "{}/{}", gate.requirements_verified, gate.requirements_total);
    let sil = with.realization.iter().flat_map(|l| l.stages.iter()).filter(|s| s.stage == "sil").collect::<Vec<_>>();
    assert!(!sil.is_empty() && sil.iter().all(|s| s.state == "pass" && s.target == "OMSimulator v2.1.2"), "{sil:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_key_follows_the_generated_c_the_case_and_the_plant_s_sources() {
    let dir = work("key");
    let model = dir.join("speed_monitor.arc");
    std::fs::copy(example(SPEED), &model).unwrap();
    let result = compile_file(&model);
    let case = result.ast.test_cases.iter().find(|t| t.id == "TC-SPD-001").unwrap();
    let key = cosim::cache_key(&result.ast, case, &dir).unwrap();
    assert_eq!(key, cosim::cache_key(&result.ast, case, &dir).unwrap(), "deterministic");
    let other = result.ast.test_cases.iter().find(|t| t.id == "TC-SPD-002").unwrap();
    assert_ne!(key, cosim::cache_key(&result.ast, other, &dir).unwrap(), "another case");
    // A plant's sources are part of it: the charger's cases read physics/ChargerPhysics.mo.
    let charger = example("energy/battery-charger/battery_charger.arc");
    let result = compile_file(&charger);
    let plant = result.ast.test_cases.iter().find(|t| matches!(t.attributes.get("execute"), Some(arclang::compiler::ast::AttributeValue::Map(m)) if m.contains_key("plant"))).expect("a case with a plant");
    let with = cosim::cache_key(&result.ast, plant, charger.parent().unwrap()).unwrap();
    let without = cosim::cache_key(&result.ast, plant, &dir).unwrap();
    assert_ne!(with, without, "the plant's sources read beside the model, or missing");
    let _ = std::fs::remove_dir_all(&dir);
}
