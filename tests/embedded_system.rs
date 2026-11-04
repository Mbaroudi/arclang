//! The processor and the programmable logic together (`arclang soc`): the
//! SoC's description (each block where its driver looks for it), the
//! application's link to a bench that comes after the boot, what is
//! refused; and, in the system image (tools/embedded/system), the SoC
//! simulated cycle by cycle in Verilator, Zephyr on its VexRiscv calling
//! each block through its driver, compared word for word with the
//! fixed-point C and within the tolerance of the model, and the same SoC
//! built into one bitstream for the ULX3S meeting timing.
//!
//! The tests that need LiteX, Verilator, Zephyr, GHDL, Yosys and nextpnr
//! are `#[ignore]`d on a bare host and run in the image:
//! `tools/embedded/system/run.sh cargo test --release --test embedded_system
//! -- --include-ignored` (CI job "Embedded system").

use arclang::compiler::embedded::{alloc, hdl, plan, system, zephyr};
use arclang::{Compiler, CompilerConfig};
use std::path::{Path, PathBuf};
use std::process::Command;

const CHARGER: &str = "energy/battery-charger/battery_charger.arc";
const WINDOW: &str = "automotive/power-window/power_window.arc";

fn example(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples").join(path)
}

fn compile_file(path: &Path) -> arclang::compiler::CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_file(path).unwrap_or_else(|e| panic!("{} compiles: {e}", path.display()))
}

fn work(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("arclang-system-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn options(name: &str) -> system::Options {
    system::Options { work: work(name), only: None, compiler: "gcc".into(), simulate: true, gate_level: false, bitstream: false, board: None, port: None }
}

// ---- on any host ----------------------------------------------------------------------

#[test]
fn the_soc_has_each_block_where_its_driver_looks_for_it() {
    let model = compile_file(&example(CHARGER));
    let p = plan::plan(&model.ast).unwrap();
    let allocations = alloc::allocations(&model.ast, &p).unwrap();
    let dir = work("config");
    let board = system::board_for(&allocations, None).unwrap();
    assert_eq!(board.name, "ulx3s", "the first board of the part");
    assert_eq!(system::board_for(&allocations, Some("colorlight_i5")).unwrap().loader, "colorlight-i5", "the same part on another board");
    let config: serde_json::Value = serde_json::from_str(&system::soc_config(&p, &allocations, &board, 25e6, 10_000, &dir, 4327).unwrap()).unwrap();
    assert_eq!((config["board"].as_str(), config["device"].as_str(), config["clock_hz"].as_u64()), (Some("ulx3s"), Some("LFE5U-25F-6BG381C"), Some(25_000_000)));
    assert_eq!(config["ram_bytes"].as_u64(), Some(system::RAM_BYTES as u64));
    let block = &config["blocks"][0];
    assert_eq!(block["name"].as_str(), Some("battery_charger_charger"));
    assert_eq!(block["base"].as_u64(), Some(zephyr::hw_base(0) as u64));
    // The application the SoC runs takes the same address by default.
    let package = arclang::compiler::embedded::target::find("zephyr/litex_vexriscv").unwrap();
    let app = zephyr::hw_app(&p, &allocations, None, &package, 10_000).unwrap();
    let cmake = &app.files.iter().find(|(n, _, _)| n == "arclang_app.cmake").unwrap().1;
    assert!(cmake.contains(&format!("set(BATTERY_CHARGER_CHARGER_HW_BASE 0x{:08X})", zephyr::hw_base(0))), "{cmake}");
    // The SoC script reads that description and nothing of the model.
    for wanted in ["cpu_type=\"vexriscv\"", "cpu_reset_address=MAIN_RAM", "integrated_rom_size=0", "self.bus.add_slave(block[\"name\"]", "serial2tcp", "radiona_ulx3s"] {
        assert!(system::SOC_SCRIPT.contains(wanted), "{wanted}");
    }
}

#[test]
fn a_bench_that_comes_after_the_boot_can_ask_whether_the_target_is_ready() {
    let model = compile_file(&example(CHARGER));
    let p = plan::plan(&model.ast).unwrap();
    let (_, source) = zephyr::pil_files(&p);
    assert!(source.contains("while (ok && (line[0] == '?') && (line[1] == '\\0'))") && source.contains("arclang_pil_puts(\"ARCLANG READY\\n\");"), "{source}");
}

#[test]
fn the_soc_refuses_what_it_cannot_build() {
    let software = compile_file(&example("automotive/adaptive-cruise/adaptive_cruise_control.arc"));
    assert!(system::run(&software.ast, &options("none")).unwrap_err().contains("no state_machine is allocated to programmable logic"));
    let charger = compile_file(&example(CHARGER));
    let board = system::Options { board: Some("arty".into()), ..options("board") };
    assert!(system::run(&charger.ast, &board).unwrap_err().contains("no board 'arty' (boards: ulx3s, colorlight_i5, colorlight_i9, ecp5_evn)"));
    let evn = system::Options { board: Some("ecp5_evn".into()), ..options("evn") };
    assert!(system::run(&charger.ast, &evn).unwrap_err().contains("board 'ecp5_evn' carries LFE5UM5G-85F-8BG381C, not node 'Charger fabric''s LFE5U-25F-6BG381C"));
}

/// The charger with its processor at 25 MHz and its block at 20 MHz.
fn two_clocks() -> String {
    let text = std::fs::read_to_string(example(CHARGER)).unwrap().replace("processor: \"VexRiscv (LiteX)\"", "processor: \"VexRiscv (LiteX)\"\n        clock: 25 MHz");
    let at = text.find("node \"Charger fabric\"").unwrap();
    let clock = at + text[at..].find("clock: 25 MHz").unwrap();
    format!("{}clock: 20 MHz{}", &text[..clock], &text[clock + "clock: 25 MHz".len()..])
}

#[test]
fn a_block_on_its_own_clock_is_a_clock_domain_of_the_soc() {
    let model = Compiler::new(CompilerConfig::default()).compile_string(&two_clocks()).unwrap();
    let p = plan::plan(&model.ast).unwrap();
    let allocations = alloc::allocations(&model.ast, &p).unwrap();
    assert_eq!(alloc::processor_clock(&model.ast).unwrap(), Some(25e6));
    let board = system::board_for(&allocations, None).unwrap();
    let config: serde_json::Value = serde_json::from_str(&system::soc_config(&p, &allocations, &board, 25e6, 10_000, &work("clocks"), 4327).unwrap()).unwrap();
    assert_eq!((config["clock_hz"].as_u64(), config["blocks"][0]["clock_hz"].as_u64()), (Some(25_000_000), Some(20_000_000)));
    for wanted in ["synth_ecp5 -top %s -noiopad", "MULT18X18D: only the combinational mode Yosys uses is modelled", "wishbone.ClockDomainCrossing(bus, port, cd_from=\"sys\", cd_to=domain)", "pll.create_clkout(cd, hz)", "sim.add_clocker(\"clk_\" + name, freq_hz=hz)"] {
        assert!(system::SOC_SCRIPT.contains(wanted), "{wanted}");
    }
}

// ---- in the system image --------------------------------------------------------------

fn assert_passes(outcomes: &[hdl::Outcome]) {
    assert!(!outcomes.is_empty());
    for o in outcomes {
        assert_eq!(o.verdict, "pass", "{}: {}", o.test_case, o.reason);
        let b = o.bit_exact.as_ref().unwrap();
        assert_eq!((b.max_difference, b.first_mismatch.as_deref()), (0.0, None), "{}: the SoC gives the fixed-point C's words", o.test_case);
        assert!(o.cycles.as_deref().is_some_and(|c| c.contains("in the simulated SoC")), "{:?}", o.cycles);
    }
}

#[test]
#[ignore = "needs LiteX, Verilator, Zephyr and GHDL of tools/embedded/system: run in its image (CI job Embedded system)"]
fn the_charger_runs_in_the_simulated_soc_bit_exact_with_the_fixed_point_c() {
    let model = compile_file(&example(CHARGER));
    let report = system::run(&model.ast, &options("charger")).unwrap();
    assert_eq!(report.outcomes.len(), 3);
    assert_passes(&report.outcomes);
}

#[test]
#[ignore = "needs LiteX, Verilator, Zephyr and GHDL of tools/embedded/system: run in its image (CI job Embedded system)"]
fn the_window_runs_in_the_simulated_soc_bit_exact_with_the_fixed_point_c() {
    let model = compile_file(&example(WINDOW));
    let report = system::run(&model.ast, &options("window")).unwrap();
    assert_eq!(report.outcomes.len(), 2);
    assert_passes(&report.outcomes);
}

#[test]
#[ignore = "needs LiteX, Verilator, Zephyr and GHDL of tools/embedded/system: run in its image (CI job Embedded system)"]
fn a_block_on_its_own_clock_runs_bit_exact_through_the_clock_domain_crossing() {
    let model = Compiler::new(CompilerConfig::default()).compile_string(&two_clocks()).unwrap();
    let report = system::run(&model.ast, &system::Options { only: Some("TC-CHG-002".into()), ..options("two-clocks") }).unwrap();
    assert_passes(&report.outcomes);
}

#[test]
#[ignore = "needs LiteX, Verilator, Zephyr, GHDL and Yosys of tools/embedded/system: run in its image (CI job Embedded system)"]
fn the_block_as_synthesised_for_the_ecp5_runs_bit_exact_at_gate_level() {
    // The charger's block uses 16 multipliers: their model is exercised.
    let model = compile_file(&example(CHARGER));
    let report = system::run(&model.ast, &system::Options { only: Some("TC-CHG-003".into()), gate_level: true, ..options("gate-level") }).unwrap();
    assert_eq!(report.outcomes.len(), 1);
    let o = &report.outcomes[0];
    assert_eq!(o.verdict, "pass", "{}", o.reason);
    assert_eq!(o.bit_exact.as_ref().unwrap().max_difference, 0.0);
    assert!(o.cycles.as_deref().is_some_and(|c| c.contains("gate level")), "{:?}", o.cycles);
}

#[test]
#[ignore = "needs LiteX, Zephyr, GHDL, Yosys and nextpnr-ecp5 of tools/embedded/system: run in its image (CI job Embedded system)"]
fn the_same_soc_is_one_bitstream_for_the_ulx3s_that_meets_timing() {
    let model = compile_file(&example(CHARGER));
    let options = system::Options { simulate: false, bitstream: true, board: Some("ulx3s".into()), ..options("bitstream") };
    let report = system::run(&model.ast, &options).unwrap();
    let b = report.bitstream.unwrap().unwrap();
    assert!(b.timing_met, "Fmax {} MHz for {} MHz", b.fmax_mhz, b.clock_mhz);
    assert!(Path::new(&b.bitstream).is_file() && b.flash.starts_with("openFPGALoader -b ulx3s "), "{b:?}");
    assert!(b.luts > 0 && b.brams >= (system::RAM_BYTES as u64) / 2048, "the RAM is block RAM: {b:?}");
    assert!(b.image_bytes > 0 && b.image_bytes < system::RAM_BYTES as u64);
    assert_eq!(b.blocks, vec!["battery_charger_charger_wb".to_string()]);
}

#[test]
#[ignore = "needs LiteX, Verilator, Zephyr and GHDL of tools/embedded/system: run in its image (CI job Embedded system)"]
fn the_cli_runs_the_soc_and_check_reads_its_verdicts() {
    let dir = work("cli");
    let model = dir.join("power_window.arc");
    std::fs::copy(example(WINDOW), &model).unwrap();
    let bin = env!("CARGO_BIN_EXE_arclang");
    let run = Command::new(bin).arg("soc").arg(&model).output().unwrap();
    let text = String::from_utf8_lossy(&run.stdout);
    assert!(run.status.success(), "{text}{}", String::from_utf8_lossy(&run.stderr));
    assert!(text.contains("SoC vs fixed-point C") && text.contains("largest difference 0 (bit-exact)"), "{text}");
    let cached = dir.join(".arclang/sim-cache/TC-PW-001.soc.json");
    let outcome: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&cached).unwrap()).unwrap();
    assert_eq!((outcome["kind"].as_str(), outcome["verdict"].as_str()), (Some("soc"), Some("pass")));
    let check = Command::new(bin).arg("check").arg(&model).output().unwrap();
    let said = format!("{}{}", String::from_utf8_lossy(&check.stdout), String::from_utf8_lossy(&check.stderr));
    assert!(!said.contains("SoC verdict is stale"), "{said}");
}
