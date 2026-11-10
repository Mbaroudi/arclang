//! The realization ladder: for each requirement, what each stage says
//! (simulation, SIL, PIL, HDL, SoC, board, twin) from the verdicts cached
//! beside the model, fresh or stale, and what to run next. Nothing is run
//! here: the caches are written by hand with the keys the compiler
//! computes, as `arclang sil`, `pil` or `twin` would write them.

use arclang::compiler::realization;
use arclang::compiler::simulation::{cache, Outcome};
use arclang::{Compiler, CompilerConfig};
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// A copy of an example in a directory of its own, with no cache.
fn copy_of(example: &str, name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("arclang-realization-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let source = root().join(example);
    for entry in std::fs::read_dir(source.parent().unwrap()).unwrap().flatten() {
        let path = entry.path();
        if path.is_file() {
            std::fs::copy(&path, dir.join(path.file_name().unwrap())).unwrap();
        } else if path.is_dir() && !path.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.')) {
            copy_dir(&path, &dir.join(path.file_name().unwrap()));
        }
    }
    dir.join(source.file_name().unwrap())
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap().flatten() {
        let path = entry.path();
        if path.is_file() {
            std::fs::copy(&path, to.join(path.file_name().unwrap())).unwrap();
        } else if path.is_dir() {
            copy_dir(&path, &to.join(path.file_name().unwrap()));
        }
    }
}

fn compile(path: &Path) -> arclang::compiler::CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_file(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn stage<'a>(ladder: &'a realization::Ladder, name: &str) -> &'a realization::Stage {
    ladder.stages.iter().find(|s| s.stage == name).unwrap_or_else(|| panic!("no stage {name}"))
}

#[test]
fn a_model_never_run_has_every_stage_to_climb_and_says_which_first() {
    let model = copy_of("examples/automotive/power-window/power_window.arc", "fresh");
    let result = compile(&model);
    let ladders = realization::ladders(&result, Some(&model));
    let ladder = ladders.iter().find(|l| l.requirement == "SYS-PW-001").expect("SYS-PW-001");
    // The window has no simulated case and no twin; its machines run in C, one in logic.
    assert_eq!(stage(ladder, "simulation").state, "n/a");
    assert_eq!(stage(ladder, "twin").state, "n/a");
    for name in ["sil", "pil", "hdl", "soc", "board"] {
        assert_eq!(stage(ladder, name).state, "missing", "{name}");
    }
    assert!(ladder.next.starts_with("arclang sil ") && ladder.next.contains("--test TC-PW-00"), "{}", ladder.next);
    assert!(realization::line(ladder).contains("SIL ○ · PIL ○ · HDL ○ · SoC ○ · board ○ · twin –"), "{}", realization::line(ladder));
    assert_eq!(ladder.reached(), None);
}

#[test]
fn cached_verdicts_climb_the_ladder_and_a_changed_model_makes_them_stale() {
    let model = copy_of("examples/automotive/power-window/power_window.arc", "cached");
    let result = compile(&model);
    let dir = cache::dir(&model);
    let cases: Vec<_> = result.ast.test_cases.iter().filter(|t| t.verifies.iter().any(|r| r == "SYS-PW-001")).cloned().collect();
    assert!(!cases.is_empty());
    // SIL, as `arclang sil` keeps it: a fresh pass of every case of the requirement.
    for test in &cases {
        let key = arclang::compiler::embedded::sil::cache_key(&result.ast, test, model.parent().unwrap()).unwrap();
        let outcome = Outcome { test_case: test.id.clone(), kind: "sil".into(), name: test.name.clone(), verifies: test.verifies.clone(), verdict: "pass".into(), key, tool: "arclang sil (cc)".into(), ..Default::default() };
        cache::write(&dir, &outcome).unwrap();
    }
    // PIL on the first case, with a key of another C: stale.
    let first = &cases[0];
    let pil = Outcome { test_case: first.id.clone(), kind: "pil".into(), name: first.name.clone(), verifies: first.verifies.clone(), verdict: "pass".into(), key: "0000".into(), target: "zephyr/litex_vexriscv".into(), tool: "arclang pil (riscv64-zephyr-elf-gcc)".into(), ..Default::default() };
    cache::write_kind(&dir, "pil", &pil).unwrap();
    let result = compile(&model);
    let ladders = realization::ladders(&result, Some(&model));
    let ladder = ladders.iter().find(|l| l.requirement == "SYS-PW-001").unwrap();
    assert_eq!(stage(ladder, "sil").state, "pass");
    assert_eq!(stage(ladder, "pil").state, "stale");
    assert_eq!(stage(ladder, "pil").target, "zephyr/litex_vexriscv");
    assert_eq!(ladder.reached(), Some("sil"));
    assert!(ladder.next.starts_with("arclang pil "), "{}", ladder.next);
    let line = realization::line(ladder);
    assert!(line.contains("SIL ✓ · PIL ~ · HDL ○"), "{line}");
    // The rungs are on the case's status, kept apart from its folded verdict.
    let status = &result.semantic_model.simulations[&first.id];
    assert_eq!(status.rungs.iter().map(|r| (r.stage.as_str(), r.state.as_str())).collect::<Vec<_>>(), vec![("sil", "fresh"), ("pil", "stale")]);
    // A case changes (it runs longer): its SIL verdict no longer holds, and the requirement's stage with it.
    let text = std::fs::read_to_string(&model).unwrap();
    assert!(text.contains("stop_time: 150 ms"));
    std::fs::write(&model, text.replacen("stop_time: 150 ms", "stop_time: 160 ms", 1)).unwrap();
    let result = compile(&model);
    let ladders = realization::ladders(&result, Some(&model));
    let ladder = ladders.iter().find(|l| l.requirement == "SYS-PW-001").unwrap();
    assert_eq!(stage(ladder, "sil").state, "stale");
    assert!(ladder.next.starts_with("arclang sil "), "{}", ladder.next);
}

#[test]
fn check_prints_the_ladder_and_its_json_carries_it() {
    let model = copy_of("examples/automotive/power-window/power_window.arc", "check");
    let out = Command::new(env!("CARGO_BIN_EXE_arclang")).arg("check").arg(&model).output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("Realization (what each stage says"), "{text}");
    assert!(text.contains("SYS-PW-001"), "{text}");
    assert!(text.contains("next: arclang sil"), "{text}");
    let out = Command::new(env!("CARGO_BIN_EXE_arclang")).arg("check").arg("--json").arg(&model).output().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let ladders = json["realization"].as_array().expect("realization");
    assert!(ladders.iter().any(|l| l["requirement"] == "SYS-PW-001" && l["stages"].as_array().unwrap().len() == 7), "{json}");
}

#[test]
fn a_twin_is_the_last_rung_of_what_it_verifies() {
    let model = copy_of("examples/building/heat-pump/heat_pump_heating.arc", "twin");
    let result = compile(&model);
    let twin = result.ast.twins.iter().find(|t| t.id == "TW-JANUARY").expect("TW-JANUARY").clone();
    let ladders = realization::ladders(&result, Some(&model));
    let ladder = ladders.iter().find(|l| l.requirement == "REQ-COMFORT").expect("REQ-COMFORT");
    assert_eq!(stage(ladder, "twin").state, "missing");
    // The twin ran: its verdict is cached under the key of the model, its sources and its measurements.
    let dir = cache::dir(&model);
    let key = arclang::compiler::simulation::twin::cache_key(&result, &model, &twin).unwrap();
    let outcome = Outcome { test_case: twin.id.clone(), kind: "twin".into(), name: twin.name.clone(), verifies: twin.verifies.clone(), verdict: "pass".into(), key, target: "data/house-2026-01-14.csv".into(), tool: "openmodelica".into(), ..Default::default() };
    cache::write_kind(&dir, "twin", &outcome).unwrap();
    let result = compile(&model);
    let ladders = realization::ladders(&result, Some(&model));
    let ladder = ladders.iter().find(|l| l.requirement == "REQ-COMFORT").unwrap();
    assert_eq!(stage(ladder, "twin").state, "pass");
    assert_eq!(stage(ladder, "twin").target, "data/house-2026-01-14.csv");
    // The measurements change: the twin's verdict is stale.
    let data = model.parent().unwrap().join("data/house-2026-01-14.csv");
    let text = std::fs::read_to_string(&data).unwrap();
    std::fs::write(&data, format!("{text}\n")).unwrap();
    let result = compile(&model);
    let ladders = realization::ladders(&result, Some(&model));
    let ladder = ladders.iter().find(|l| l.requirement == "REQ-COMFORT").unwrap();
    assert_eq!(stage(ladder, "twin").state, "stale");
    assert_eq!(stage(ladder, "twin").cases[0].state, "stale");
    assert!(stage(ladder, "twin").cases[0].reason.contains("arclang twin") && stage(ladder, "twin").cases[0].reason.contains("--twin TW-JANUARY"), "{}", stage(ladder, "twin").cases[0].reason);
    // The simulation of the design day comes before the twin on the ladder: it is what to run first.
    assert!(ladder.next.starts_with("arclang verify "), "{}", ladder.next);
}

#[test]
fn realize_plans_the_climb_in_the_order_of_the_stages_and_names_the_gaps() {
    let model = copy_of("examples/automotive/power-window/power_window.arc", "plan");
    // A requirement nothing verifies: a gap, not a step.
    let text = std::fs::read_to_string(&model).unwrap();
    std::fs::write(&model, text.replacen("requirements system {", "requirements system {\n    req \"SYS-PW-009\" \"Documented behaviour\" { description: \"The behaviour is documented\" }", 1)).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_arclang")).arg("realize").arg(&model).arg("--dry-run").arg("--json").output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let steps = json["plan"]["steps"].as_array().unwrap();
    let stages: Vec<&str> = steps.iter().map(|s| s["stage"].as_str().unwrap()).collect();
    // Every executed case once per stage, SIL before PIL before HDL...
    assert!(stages.windows(2).all(|w| realization::STAGES.iter().position(|s| *s == w[0]) <= realization::STAGES.iter().position(|s| *s == w[1])), "{stages:?}");
    assert_eq!(stages.iter().filter(|s| **s == "sil").count(), 3, "{stages:?}");
    assert!(stages.contains(&"pil") && stages.contains(&"hdl") && stages.contains(&"soc") && !stages.contains(&"board"), "the board needs --board: {stages:?}");
    let first = &steps[0];
    assert_eq!(first["args"][0], "sil");
    assert_eq!(first["args"][1].as_str().unwrap(), model.display().to_string());
    assert_eq!(first["args"][2], "--test");
    assert!(first["requirements"].as_array().unwrap().len() >= 2, "one step climbs every requirement its case verifies");
    let gaps = json["plan"]["gaps"].as_array().unwrap();
    assert_eq!(gaps.len(), 1);
    assert_eq!(gaps[0]["requirement"], "SYS-PW-009");
    assert!(gaps[0]["hint"].as_str().unwrap().contains("test_case"));
    // Up to SIL, for one requirement: its cases, nothing further.
    let out = Command::new(env!("CARGO_BIN_EXE_arclang")).args(["realize", "--dry-run", "--json", "--requirement", "SYS-PW-003", "--up-to", "sil"]).arg(&model).output().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let steps = json["plan"]["steps"].as_array().unwrap();
    assert!(steps.iter().all(|s| s["stage"] == "sil" && s["requirements"] == serde_json::json!(["SYS-PW-003"])), "{steps:?}");
    assert_eq!(steps.len(), 2, "TC-PW-001 and TC-PW-003 verify SYS-PW-003");
    // The text form says the same.
    let out = Command::new(env!("CARGO_BIN_EXE_arclang")).args(["realize", "--dry-run"]).arg(&model).output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("Plan (") && text.contains("SIL (missing) for") && text.contains("SYS-PW-009 \"Documented behaviour\": no case simulates"), "{text}");
    let out = Command::new(env!("CARGO_BIN_EXE_arclang")).args(["realize", "--dry-run", "--up-to", "orbit"]).arg(&model).output().unwrap();
    assert!(!out.status.success() && String::from_utf8_lossy(&out.stderr).contains("unknown stage 'orbit'"));
}

#[test]
fn realize_climbs_to_sil_and_stops_where_the_evidence_fails() {
    // SIL needs a C compiler, as the SIL tests do; the climb runs it as this binary.
    let model = copy_of("examples/automotive/power-window/power_window.arc", "climb");
    let out = Command::new(env!("CARGO_BIN_EXE_arclang")).args(["realize", "--up-to", "sil", "--requirement", "SYS-PW-002"]).arg(&model).output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{text}{}", String::from_utf8_lossy(&out.stderr));
    assert!(text.contains("→ SIL (missing): arclang sil") && text.contains("SYS-PW-002 SIL (TC-PW-001): ✓ pass") && text.contains("Complete up to SIL: every stage that applies, up to it, is fresh evidence."), "{text}");
    // The ladder after: SIL fresh for every case of the requirement.
    let result = compile(&model);
    let ladders = realization::ladders(&result, Some(&model));
    assert_eq!(stage(ladders.iter().find(|l| l.requirement == "SYS-PW-002").unwrap(), "sil").state, "pass");
    // A case made to fail: the climb reports the failure and does not go past it.
    let text = std::fs::read_to_string(&model).unwrap();
    // The expectation is written inside a string list: its quotes are escaped in the file.
    assert!(text.contains(r#"final(\"Window\".pinches) == 3"#));
    std::fs::write(&model, text.replacen(r#"final(\"Window\".pinches) == 3"#, r#"final(\"Window\".pinches) == 7"#, 1)).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_arclang")).args(["realize", "--up-to", "pil", "--requirement", "SYS-PW-002"]).arg(&model).output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(!out.status.success());
    assert!(text.contains("SYS-PW-002 SIL (TC-PW-001): ✗ fail") && text.contains("the climb of SYS-PW-002 stops here") && !text.contains("→ PIL"), "{text}");
}
