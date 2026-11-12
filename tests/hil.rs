//! Hardware in the loop without hardware: the direct harness built on this
//! computer stands for the board (`arclang hil --command`), the same
//! protocol on its stdin and stdout. The smart lock's bench model
//! (examples/cesam/smart-lock/uno/smart_lock_bench.arc, the lock imported
//! plus one executed case) runs through it, its verdict lands in the cache
//! as the board stage, and the coverage table and the realization ladder
//! read it.

use arclang::compiler::realization;
use arclang::compiler::simulation::cache;
use std::path::{Path, PathBuf};
use std::process::Command;

const LOCK: &str = "examples/cesam/smart-lock/smart_lock.arc";
const BENCH: &str = "examples/cesam/smart-lock/uno/smart_lock_bench.arc";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn work(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("arclang-hil-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("uno")).unwrap();
    dir
}

fn cc() -> String {
    std::env::var("CC").ok().filter(|c| !c.trim().is_empty()).unwrap_or_else(|| "cc".to_string())
}

/// A copy of the lock and its bench model, with no cache: the bench imports
/// `../smart_lock.arc`, so both go in, one level apart.
fn bench_copy(dir: &Path) -> PathBuf {
    std::fs::copy(root().join(LOCK), dir.join("smart_lock.arc")).unwrap();
    let bench = dir.join("uno/smart_lock_bench.arc");
    std::fs::copy(root().join(BENCH), &bench).unwrap();
    bench
}

/// The lock's C exported from the bench model, its harness compiled: the
/// binary that stands for the board.
fn harness(dir: &Path, model: &Path) -> PathBuf {
    let exe = env!("CARGO_BIN_EXE_arclang");
    let out = Command::new(exe).arg("export").arg(model).args(["-f", "c", "-o"]).arg(dir.join("c")).output().unwrap();
    assert!(out.status.success(), "{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    assert!(dir.join("c/host/harness.c").is_file(), "export -f c writes host/harness.c");
    let binary = dir.join("harness");
    let build = Command::new(cc())
        .args(["-std=c99", "-Wall", "-Wextra", "-Werror", "-pedantic", "-I"])
        .arg(dir.join("c"))
        .arg(dir.join("c/smart_door_lock.c"))
        .arg(dir.join("c/host/harness.c"))
        .arg("-o")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(build.status.success(), "{}", String::from_utf8_lossy(&build.stderr));
    binary
}

/// A board: says it is ready, then runs the harness once.
fn board(binary: &Path, ident: &str) -> String {
    format!("printf '# arclang hil ready {ident} test-board\\n'; exec {}", binary.display())
}

#[test]
fn the_case_runs_on_a_program_standing_for_the_board_and_climbs_the_ladder() {
    let dir = work("board");
    let model = bench_copy(&dir);
    let binary = harness(&dir, &model);
    let exe = env!("CARGO_BIN_EXE_arclang");
    // SIL first, as the ladder climbs: the board's verdict is then added to the SIL evidence.
    let out = Command::new(exe).arg("sil").arg(&model).args(["--cc", &cc()]).output().unwrap();
    assert!(out.status.success(), "{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    let out = Command::new(exe).arg("hil").arg(&model).args(["--command", &board(&binary, "smart_door_lock")]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(out.status.success(), "{stdout}{}", String::from_utf8_lossy(&out.stderr));
    assert!(stdout.contains("HIL ") && stdout.contains("test-board on program"), "{stdout}");
    assert!(stdout.contains("✓ pass  TC-7"), "{stdout}");
    assert!(stdout.contains("verifies FUN-1, TEC-1 · machine Lock modes"), "{stdout}");
    assert!(stdout.contains("model: pass · board: pass · the board run reproduces the model"), "{stdout}");
    assert!(stdout.contains("back to back: 31 steps × 8 outputs, largest difference 0"), "{stdout}");
    assert!(stdout.contains("31 steps answered by the board in"), "{stdout}");
    assert!(stdout.contains("ok   FUN-1: at(30 ms, \"Lock modes\".Unlocked) == 1   model 1 · board 1"), "{stdout}");
    assert!(stdout.contains("1 verdict(s) cached in"), "{stdout}");
    // The verdict, as cached: kind hil, the board as target, the key of the generated C and the case.
    let cached = cache::read_kind(&cache::dir(&model), "TC-7", "hil").expect("TC-7.hil.json");
    assert_eq!((cached.kind.as_str(), cached.target.as_str(), cached.verdict.as_str()), ("hil", "test-board", "pass"));
    assert!(cached.tool.starts_with("arclang hil (on the board test-board on program"), "{}", cached.tool);
    assert!(cached.log.contains("31 steps x 8 outputs, largest difference 0"), "{}", cached.log);
    // The coverage table reads it: the case passes, with the board as evidence.
    let out = Command::new(exe).arg("table").arg(&model).args(["--kind", "requirements_coverage", "-f", "json"]).output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(out.status.success(), "{text}{}", String::from_utf8_lossy(&out.stderr));
    assert!(text.contains("board test-board"), "the table names the board: {text}");
    // The realization ladder: the board stage of FUN-1 passes, it is the stage reached.
    let result = arclang::Compiler::new(arclang::CompilerConfig::default()).compile_file(&model).unwrap();
    let ladders = realization::ladders(&result, Some(&model));
    let ladder = ladders.iter().find(|l| l.requirement == "FUN-1").expect("FUN-1");
    let stage = ladder.stages.iter().find(|s| s.stage == "board").unwrap();
    assert_eq!(stage.state, "pass", "{:?}", ladder.stages);
    assert_eq!(stage.target, "test-board");
    assert_eq!(ladder.reached(), Some("board"));
    assert!(realization::line(ladder).contains("board ✓"), "{}", realization::line(ladder));
    // JSON output lists the outcome; a second run on the same program is as good.
    let out = Command::new(exe).arg("hil").arg(&model).args(["--command", &board(&binary, "smart_door_lock"), "--json"]).output().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| panic!("{e}: {}", String::from_utf8_lossy(&out.stdout)));
    assert_eq!(json[0]["test_case"], "TC-7");
    assert_eq!(json[0]["verdict"], "pass");
    assert_eq!(json[0]["compiler"], "test-board on program `".to_string() + &board(&binary, "smart_door_lock") + "`");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_board_holding_another_model_or_answering_wrongly_is_an_error_and_the_command_fails() {
    let dir = work("wrong");
    let model = bench_copy(&dir);
    let binary = harness(&dir, &model);
    let exe = env!("CARGO_BIN_EXE_arclang");
    // Another model on the board.
    let out = Command::new(exe).arg("hil").arg(&model).args(["--command", &board(&binary, "battery_charger")]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(!out.status.success());
    assert!(stdout.contains("! ERROR") && stdout.contains("holds the model 'battery_charger', not 'smart_door_lock'"), "{stdout}");
    // The error is cached too, as the board's verdict.
    let cached = cache::read_kind(&cache::dir(&model), "TC-7", "hil").expect("TC-7.hil.json");
    assert_eq!(cached.verdict, "error");
    assert!(cached.reason.starts_with("on the board"), "{}", cached.reason);
    // A board that never says it is ready.
    let out = Command::new(exe).arg("hil").arg(&model).args(["--command", "cat > /dev/null", "--timeout", "1"]).output().unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(stderr.contains("did not say `# arclang hil ready"), "{stderr}");
    // A board that answers too few rows.
    let short = "printf '# arclang hil ready smart_door_lock short\\n'; printf '0,0,0,1,0,0,1,0\\n'; cat > /dev/null";
    let out = Command::new(exe).arg("hil").arg(&model).args(["--command", short, "--timeout", "1"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(!out.status.success());
    assert!(stdout.contains("! ERROR") && stdout.contains("the board gave 1 of 31 rows"), "{stdout}");
    // Neither a port nor a program.
    let out = Command::new(exe).arg("hil").arg(&model).output().unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("--port"), "{}", String::from_utf8_lossy(&out.stderr));
    let _ = std::fs::remove_dir_all(&dir);
}
