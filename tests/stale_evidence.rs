//! A cached verdict of the embedded stages (the C, the logic) is evidence
//! only for what it ran with. When the physics of the plant the case closes
//! its loop on changes, or a machine wired upstream of the case's, the
//! verdict is stale: the gate reads the case as inconclusive until the stage
//! runs again. Nothing runs here: the verdicts are written to the cache as
//! the commands write them.

use arclang::compiler::ast::TestCase;
use arclang::compiler::embedded::{cosim, hdl, sil};
use arclang::compiler::simulation::expects::Evaluated;
use arclang::compiler::simulation::{cache, Outcome, Status};
use arclang::compiler::verification::{verdict_for, Verdict};
use arclang::compiler::CompilationResult;
use arclang::{Compiler, CompilerConfig};
use std::path::{Path, PathBuf};

const CHARGER: &str = "examples/energy/battery-charger";
const SPEED: &str = "examples/automotive/speed-monitor/speed_monitor.arc";
const HDL_TOOLS: &str = "ghdl 4.1.0";

fn compile(path: &Path) -> CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_file(path).unwrap_or_else(|e| panic!("{} compiles: {e}", path.display()))
}

fn work(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("arclang-stale-evidence-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The charger and the physics of its pack, copied: the model's path.
fn copy_of_the_charger(name: &str) -> PathBuf {
    let dir = work(name);
    let from = Path::new(env!("CARGO_MANIFEST_DIR")).join(CHARGER);
    std::fs::create_dir_all(dir.join("physics")).unwrap();
    std::fs::copy(from.join("battery_charger.arc"), dir.join("battery_charger.arc")).unwrap();
    std::fs::copy(from.join("physics/ChargerPhysics.mo"), dir.join("physics/ChargerPhysics.mo")).unwrap();
    dir.join("battery_charger.arc")
}

fn copy_of_the_speed_monitor(name: &str) -> PathBuf {
    let model = work(name).join("speed_monitor.arc");
    std::fs::copy(Path::new(env!("CARGO_MANIFEST_DIR")).join(SPEED), &model).unwrap();
    model
}

/// `from` becomes `to` in a file that holds it once.
fn edit(file: &Path, from: &str, to: &str) {
    let text = std::fs::read_to_string(file).unwrap();
    assert_eq!(text.matches(from).count(), 1, "{} holds '{from}' once", file.display());
    std::fs::write(file, text.replace(from, to)).unwrap();
}

fn case<'a>(result: &'a CompilationResult, id: &str) -> &'a TestCase {
    result.ast.test_cases.iter().find(|t| t.id == id).unwrap_or_else(|| panic!("the model has the case {id}"))
}

fn key(result: &CompilationResult, model: &Path, id: &str, kind: &str) -> String {
    let base = model.parent().unwrap();
    match kind {
        "sil" => sil::cache_key(&result.ast, case(result, id), base).unwrap(),
        "cosim" => cosim::cache_key(&result.ast, case(result, id), base).unwrap(),
        _ => hdl::cache_key(&result.ast, case(result, id), HDL_TOOLS, base).unwrap(),
    }
}

/// A verdict of a stage (`sil`, `cosim` or `hdl`) kept beside the model,
/// fresh, every expectation holding for every requirement of the case.
fn cached(model: &Path, id: &str, kind: &str, verdict: &str) {
    let result = compile(model);
    let test = case(&result, id);
    let values = test
        .verifies
        .iter()
        .map(|requirement| Evaluated {
            expect: format!("{requirement}: checked"),
            value: format!("{requirement} = as expected"),
            holds: true,
            margin: String::new(),
            requirement: Some(requirement.clone()),
            slack: None,
            slack_unit: None,
        })
        .collect();
    let outcome = Outcome {
        test_case: id.to_string(),
        kind: kind.to_string(),
        name: test.name.clone(),
        verifies: test.verifies.clone(),
        verdict: verdict.to_string(),
        reason: if verdict == "pass" { String::new() } else { format!("{kind}: no verdict") },
        values,
        agrees: true,
        key: key(&result, model, id, kind),
        model: "state_machine".to_string(),
        tool: if kind == "hdl" { HDL_TOOLS.to_string() } else { format!("arclang {kind}") },
        ..Default::default()
    };
    if kind == "sil" {
        cache::write(&cache::dir(model), &outcome).unwrap();
    } else {
        cache::write_kind(&cache::dir(model), kind, &outcome).unwrap();
    }
}

fn status(model: &Path, id: &str) -> Status {
    compile(model).semantic_model.simulations.get(id).unwrap_or_else(|| panic!("{id} has a status")).clone()
}

fn rung<'a>(status: &'a Status, stage: &str) -> &'a arclang::compiler::simulation::Rung {
    status.rungs.iter().find(|r| r.stage == stage).unwrap_or_else(|| panic!("a {stage} rung in {:?}", status.rungs))
}

/// What a case says of a requirement it verifies.
fn verified(model: &Path, id: &str, requirement: &str) -> Verdict {
    let result = compile(model);
    verdict_for(case(&result, id), requirement, &result.semantic_model)
}

const LOOP: &str = "TC-CHG-HIL";

#[test]
fn a_sil_verdict_of_a_case_in_the_loop_is_stale_once_the_physics_of_its_plant_changes() {
    let model = copy_of_the_charger("sil-plant");
    cached(&model, LOOP, "sil", "pass");
    let before = status(&model, LOOP);
    assert_eq!(before.verdict, "pass", "{before:?}");
    assert_eq!(verified(&model, LOOP, "SYS-CHG-005"), Verdict::Pass);

    // The pack is given four times the internal resistance: the loop the C ran in is no longer this one.
    edit(&model.parent().unwrap().join("physics/ChargerPhysics.mo"), "= 0.08 \"Internal resistance\"", "= 0.32 \"Internal resistance\"");
    let after = status(&model, LOOP);
    assert_eq!(after.verdict, "inconclusive", "{after:?}");
    assert_eq!(rung(&after, "sil").state, "stale", "{:?}", after.rungs);
    assert!(after.reason.contains("the plant"), "the reason names the plant: {}", after.reason);
    let result = compile(&model);
    assert!(result.warnings.iter().any(|w| w.contains("is inconclusive: stale: the generated C, the case or the plant it closes its loop on changed since its SIL run")), "{:?}", result.warnings);
    assert_ne!(verified(&model, LOOP, "SYS-CHG-005"), Verdict::Pass);
}

#[test]
fn a_case_without_a_plant_keeps_its_sil_verdict_when_the_physics_changes() {
    // The other cases of the charger replay inputs: the pack's physics is not theirs.
    let model = copy_of_the_charger("sil-no-plant");
    cached(&model, "TC-CHG-001", "sil", "pass");
    edit(&model.parent().unwrap().join("physics/ChargerPhysics.mo"), "= 0.08 \"Internal resistance\"", "= 0.32 \"Internal resistance\"");
    let after = status(&model, "TC-CHG-001");
    assert_eq!(after.verdict, "pass", "{after:?}");
}

#[test]
fn the_logic_verdict_of_a_case_in_the_loop_follows_the_physics_of_its_plant() {
    // `arclang hdl` replays the vectors of the loop: other physics, other vectors.
    let model = copy_of_the_charger("hdl-plant");
    let before = key(&compile(&model), &model, LOOP, "hdl");
    edit(&model.parent().unwrap().join("physics/ChargerPhysics.mo"), "= 0.08 \"Internal resistance\"", "= 0.32 \"Internal resistance\"");
    assert_ne!(key(&compile(&model), &model, LOOP, "hdl"), before);
}

#[test]
fn a_plant_file_that_is_gone_is_another_key_not_an_error() {
    let model = copy_of_the_charger("plant-gone");
    let before = key(&compile(&model), &model, LOOP, "sil");
    std::fs::remove_file(model.parent().unwrap().join("physics/ChargerPhysics.mo")).unwrap();
    assert_ne!(key(&compile(&model), &model, LOOP, "sil"), before);
}

#[test]
fn a_plant_that_reads_the_models_own_modelica_follows_it() {
    // The clock of the fabric is in the model's Modelica export, not in the C
    // of the case: the key follows it only when the plant reads the export.
    let follows_the_clock = |model: &Path| {
        let before = key(&compile(model), model, LOOP, "sil");
        edit(model, "clock: 25 MHz", "clock: 50 MHz");
        key(&compile(model), model, LOOP, "sil") != before
    };
    let alone = copy_of_the_charger("plant-alone");
    assert!(!follows_the_clock(&alone), "a plant of its own does not read the export");

    let reading = copy_of_the_charger("plant-reads-export");
    let physics = reading.parent().unwrap().join("physics/ChargerPhysics.mo");
    let text = std::fs::read_to_string(&physics).unwrap();
    std::fs::write(&physics, text.replace("  model Battery", "  model Fabric\n    extends BatteryCharger.Physical.ChargerFabric;\n  end Fabric;\n  model Battery")).unwrap();
    assert!(std::fs::read_to_string(&physics).unwrap().contains("BatteryCharger.Physical.ChargerFabric"));
    assert!(follows_the_clock(&reading), "a plant that extends the model's export follows it");
}

/// The warning's case: its machine reads the filtered speed through a wire.
const WIRED: &str = "TC-SPD-001";
/// A case on the filter alone.
const UPSTREAM: &str = "TC-SPD-003";

#[test]
fn the_logic_verdict_of_a_wired_machine_follows_the_machine_upstream() {
    // `arclang hdl` feeds a wired input the values the model's run of the
    // upstream machine gave it: another filter, other inputs.
    let model = copy_of_the_speed_monitor("hdl-upstream-key");
    let before = compile(&model);
    let (wired, upstream) = (key(&before, &model, WIRED, "hdl"), key(&before, &model, UPSTREAM, "hdl"));
    edit(&model, "filtered + 0.25 * (raw - filtered)", "filtered + 0.5 * (raw - filtered)");
    let after = compile(&model);
    assert_ne!(key(&after, &model, WIRED, "hdl"), wired);
    assert_ne!(key(&after, &model, UPSTREAM, "hdl"), upstream, "the filter's own cases too");
}

#[test]
fn the_logic_verdict_of_an_upstream_machine_does_not_follow_the_machine_it_feeds() {
    let model = copy_of_the_speed_monitor("hdl-downstream-key");
    let before = key(&compile(&model), &model, UPSTREAM, "hdl");
    edit(&model, "const on_above: Real [km/h] = 130 km/h", "const on_above: Real [km/h] = 131 km/h");
    assert_eq!(key(&compile(&model), &model, UPSTREAM, "hdl"), before);
}

#[test]
fn a_logic_pass_does_not_stand_for_a_wired_case_once_the_machine_upstream_changes() {
    let model = copy_of_the_speed_monitor("hdl-upstream");
    cached(&model, WIRED, "sil", "pass");
    cached(&model, WIRED, "hdl", "pass");
    let before = status(&model, WIRED);
    assert_eq!(before.verdict, "pass", "{before:?}");

    edit(&model, "filtered + 0.25 * (raw - filtered)", "filtered + 0.5 * (raw - filtered)");
    let after = status(&model, WIRED);
    assert_eq!(rung(&after, "sil").state, "stale", "{:?}", after.rungs);
    assert_eq!(rung(&after, "hdl").state, "stale", "{:?}", after.rungs);
    assert_eq!(after.verdict, "inconclusive", "{after:?}");
}

#[test]
fn a_fresh_earlier_stage_that_concludes_nothing_is_not_replaced_by_a_later_pass() {
    // Only a stage never run, or stale, is stood for by a later one.
    let model = copy_of_the_speed_monitor("fresh-inconclusive");
    cached(&model, WIRED, "sil", "inconclusive");
    cached(&model, WIRED, "hdl", "pass");
    let read = status(&model, WIRED);
    assert_eq!(rung(&read, "sil").state, "fresh", "{:?}", read.rungs);
    assert_eq!(read.verdict, "inconclusive", "{read:?}");
}
