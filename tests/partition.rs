//! HW/SW partitioning (`arclang partition`): the entries built from the
//! two runs' JSON (no tools needed here), the cache beside the model and
//! the table kind `partition` that reads it, the command's dry run. The
//! measurement itself (Renode, Yosys, nextpnr) runs where the tools are:
//! `arclang partition examples/energy/battery-charger/battery_charger.arc`
//! on a host with Docker.

use arclang::compiler::embedded::{binding, plan};
use arclang::compiler::partition::{self, Entry};
use arclang::{Compiler, CompilerConfig};
use std::path::{Path, PathBuf};
use std::process::Command;

const CHARGER: &str = "energy/battery-charger/battery_charger.arc";

fn example(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples").join(path)
}

fn work(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("arclang-partition-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn pil_json() -> serde_json::Value {
    serde_json::json!([
        { "test_case": "TC-CHG-001", "machine": "Charger", "binding": null, "compiler": "zephyr/litex_vexriscv; gcc 14", "verdict": "pass", "timing": "step() took at most 38.2 us of the target's virtual time", "footprint": { "text": 4210, "data": 16, "bss": 256, "tool": "riscv64-zephyr-elf-size" } },
        { "test_case": "TC-CHG-003", "machine": "Charger", "binding": null, "compiler": "zephyr/litex_vexriscv; gcc 14", "verdict": "pass", "timing": "step() took at most 41.0 us of the target's virtual time" },
        { "test_case": "TC-CHG-002", "machine": "Charger", "binding": "Charger board", "compiler": "zephyr/litex_vexriscv; gcc 14", "verdict": "pass", "timing": "41 cycles, one every 10000 us of the target's virtual time (the first at 0 us)" }
    ])
}

fn hdl_json(timing_met: bool) -> serde_json::Value {
    serde_json::json!({ "cases": [], "synthesis": [
        { "block": "battery_charger_charger_wb", "node": "as if on LFE5U-25F-6BG381C", "device": "LFE5U-25F-6BG381C (nextpnr-ecp5 --25k --package CABGA381 --speed 6)", "clock_mhz": 25.0, "fmax_mhz": 32.63, "timing_met": timing_met, "luts": 3436, "ffs": 1347, "dsps": 16, "brams": 0, "ios": 81, "bitstream": "x.bit", "notes": [] }
    ] })
}

#[test]
fn the_entries_read_both_runs_and_say_what_fits() {
    let result = Compiler::new(CompilerConfig::default()).compile_file(&example(CHARGER)).unwrap();
    let p = plan::plan(&result.ast).unwrap();
    let bindings = binding::resolve_all(&result.ast, &p).unwrap();
    let pil = pil_json();
    let hdl = hdl_json(true);
    let entries = partition::from_runs(&result.ast, &p, &bindings, Ok(&pil), Ok(&hdl), "zephyr/litex_vexriscv", "LFE5U-25F-6BG381C", 25e6, None).unwrap();
    assert_eq!(entries.len(), 1);
    let e = &entries[0];
    assert_eq!((e.machine.as_str(), e.period_us, e.now.as_str(), e.node.as_deref()), ("Charger", 10_000, "logic", Some("Charger fabric")));
    let s = e.software.as_ref().unwrap();
    assert_eq!((s.step_us, s.cases, s.text), (41.0, 2, Some(4210)), "the worst step over the cases through step(); the footprint of the build");
    assert_eq!((e.logic.as_ref().unwrap().luts, e.logic.as_ref().unwrap().device.as_str()), (3436, "LFE5U-25F-6BG381C"), "the wrapper's synthesis, the part as asked");
    assert_eq!((e.fits_software(), e.fits_logic()), (Some(true), Some(true)));
    assert_eq!(e.advice(), "fits both; now logic");
    assert!(partition::lines(e)[1].contains("41.0 us (244× of margin), 4.1 KiB text"), "{:?}", partition::lines(e));
    // A side that did not run says why.
    let none = partition::from_runs(&result.ast, &p, &bindings, Err("no renode".to_string()), Ok(&hdl_json(false)), "t", "LFE5U-25F-6BG381C", 25e6, None).unwrap();
    assert_eq!(none[0].software_error.as_deref(), Some("no renode"));
    assert_eq!(none[0].advice(), "the logic misses timing; the processor is not measured; now logic");
}

#[test]
fn the_cache_is_read_by_the_compiler_and_the_table_and_goes_stale_with_the_code() {
    let dir = work("cache");
    let model = dir.join("battery_charger.arc");
    std::fs::copy(example(CHARGER), &model).unwrap();
    let _ = std::fs::create_dir_all(dir.join("physics"));
    std::fs::copy(example("energy/battery-charger/physics/ChargerPhysics.mo"), dir.join("physics/ChargerPhysics.mo")).unwrap();
    let result = Compiler::new(CompilerConfig::default()).compile_file(&model).unwrap();
    assert!(result.partition.is_empty(), "nothing measured yet");
    let p = plan::plan(&result.ast).unwrap();
    let bindings = binding::resolve_all(&result.ast, &p).unwrap();
    let (pil, hdl) = (pil_json(), hdl_json(true));
    let entries: Vec<Entry> = partition::from_runs(&result.ast, &p, &bindings, Ok(&pil), Ok(&hdl), "zephyr/litex_vexriscv", "LFE5U-25F-6BG381C", 25e6, None).unwrap();
    partition::write(&model, &entries).unwrap();
    let again = Compiler::new(CompilerConfig::default()).compile_file(&model).unwrap();
    assert_eq!(again.partition.len(), 1);
    assert!(again.partition[0].1, "fresh: the code did not change");
    // The table reads it.
    let exe = env!("CARGO_BIN_EXE_arclang");
    let out = Command::new(exe).arg("table").arg(&model).args(["--kind", "partition", "-f", "csv"]).output().unwrap();
    let csv = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{csv}{}", String::from_utf8_lossy(&out.stderr));
    assert!(csv.contains("Charger") && csv.contains("fits") && csv.contains("41"), "{csv}");
    // The machine's code changes: the measurement no longer holds.
    let text = std::fs::read_to_string(&model).unwrap().replace("\"const i_bulk: Real [A] = 16 A\"", "\"const i_bulk: Real [A] = 15 A\"");
    std::fs::write(&model, text).unwrap();
    let changed = Compiler::new(CompilerConfig::default()).compile_file(&model).unwrap();
    assert!(!changed.partition[0].1, "stale: the generated C changed");
    let out = Command::new(exe).arg("table").arg(&model).args(["--kind", "partition", "-f", "csv"]).output().unwrap();
    assert!(String::from_utf8_lossy(&out.stdout).contains("stale: run arclang partition"), "{}", String::from_utf8_lossy(&out.stdout));
}

#[test]
fn a_dry_run_names_the_two_runs_from_the_model() {
    let exe = env!("CARGO_BIN_EXE_arclang");
    let out = Command::new(exe).arg("partition").arg(example(CHARGER)).arg("--dry-run").output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{text}{}", String::from_utf8_lossy(&out.stderr));
    assert!(text.contains("arclang pil") && text.contains("--json --target zephyr/litex_vexriscv"), "{text}");
    assert!(text.contains("arclang hdl") && text.contains("--synth --json --as-if LFE5U-25F-6BG381C@25"), "{text}");
    let out = Command::new(exe).arg("partition").arg(example(CHARGER)).args(["--dry-run", "--device", "LFE5U-85F-8BG756C", "--clock", "50", "--machine", "Charger"]).output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("--as-if LFE5U-85F-8BG756C@50 --machine Charger"), "{text}");
    let out = Command::new(exe).arg("partition").arg(example(CHARGER)).args(["--dry-run", "--device", "XC7A35T"]).output().unwrap();
    assert!(!out.status.success() && String::from_utf8_lossy(&out.stderr).contains("no Lattice ECP5 part"));
}
