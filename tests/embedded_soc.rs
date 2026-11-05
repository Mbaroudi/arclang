//! Target packages and processor in the loop: the compiler checks a
//! binding against its target's channels, `export -f zephyr` writes a
//! Zephyr application, the processor-in-the-loop link (arclang_pil.c) is
//! bit for bit, and, in the SoC image (tools/embedded/soc), `arclang pil`
//! builds for the LiteX VexRiscv SoC, runs in Renode and reproduces the
//! model step by step.
//!
//! The tests that need west, the Zephyr SDK and Renode are `#[ignore]`d on
//! a bare host and run in the image: `tools/embedded/soc/run.sh cargo test
//! --test embedded_soc -- --include-ignored` (CI job "Embedded SoC").

use arclang::compiler::embedded::{binding, case, pil, plan, reference, sil, target, zephyr};
use arclang::{Compiler, CompilerConfig};
use std::path::{Path, PathBuf};
use std::process::Command;

const ACC: &str = "automotive/adaptive-cruise/adaptive_cruise_control.arc";
const REMOTE_START: &str = "automotive/remote-start/remote_start_architecture.arc";
const CHARGER: &str = "energy/battery-charger/battery_charger.arc";
const WINDOW: &str = "automotive/power-window/power_window.arc";
const LITEX: &str = "zephyr/litex_vexriscv";

fn example(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples").join(path)
}

fn compile_file(path: &Path) -> arclang::compiler::CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_file(path).unwrap_or_else(|e| panic!("{} compiles: {e}", path.display()))
}

fn compile_text(text: &str) -> Result<arclang::compiler::CompilationResult, String> {
    Compiler::new(CompilerConfig::default()).compile_string(text).map_err(|e| e.to_string())
}

fn work(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("arclang-soc-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The charger with its binding changed by `edit`.
fn charger_with(edit: impl Fn(String) -> String) -> String {
    edit(std::fs::read_to_string(example(CHARGER)).unwrap())
}

// ---- target packages ------------------------------------------------------------------

#[test]
fn the_builtin_targets_are_packages_of_data() {
    let names = target::names();
    assert!(names.contains(&"host".to_string()) && names.contains(&LITEX.to_string()), "{names:?}");
    let litex = target::find(LITEX).unwrap();
    assert_eq!(litex.runtime, "zephyr");
    assert_eq!(litex.board.as_deref(), Some("litex_vexriscv"));
    assert!(litex.channels.contains_key("can_rx") && !litex.channels.contains_key("dac"));
    assert!(litex.build.is_some() && litex.emulator.is_some());
    let files: Vec<String> = litex.files().unwrap().into_iter().map(|(to, _, _)| to).collect();
    for f in ["CMakeLists.txt", "src/arclang_io_zephyr.c", "boards/litex_vexriscv.overlay", "arclang_module/drivers/can_litex_sja1000.c", "renode/arclang_litex_vexriscv.repl", "renode/pil_bench.py", "run.sh"] {
        assert!(files.contains(&f.to_string()), "{f} in {files:?}");
    }
    let e = target::find("zephyr/nucleo_f429zi").unwrap_err();
    assert!(e.contains("is no target package (known: host, zephyr/litex_vexriscv") && e.contains("ARCLANG_TARGET_PATH"), "{e}");
}

#[test]
fn a_package_on_the_target_path_is_found_without_changing_the_compiler() {
    let dir = work("target-path");
    let package = dir.join("linux/rt_demo");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        package.join("target.json"),
        r#"{ "name": "linux/rt_demo", "summary": "a demo", "runtime": "zephyr", "channels": { "gpio_in": { "channels": "0-3" }, "gpio_out": { "channels": "0-3" } },
            "files": [ { "from": "notes.txt", "to": "NOTES.txt" } ] }"#,
    )
    .unwrap();
    std::fs::write(package.join("notes.txt"), "hello").unwrap();
    // One process-wide variable: this test is the only one that sets it.
    std::env::set_var("ARCLANG_TARGET_PATH", &dir);
    let found = target::find("linux/rt_demo");
    let names = target::names();
    std::env::remove_var("ARCLANG_TARGET_PATH");
    let found = found.unwrap();
    assert_eq!(found.files().unwrap(), vec![("NOTES.txt".to_string(), "hello".to_string(), false)]);
    assert!(names.contains(&"linux/rt_demo".to_string()), "{names:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

fn binding_error(edit: impl Fn(String) -> String) -> String {
    match compile_text(&charger_with(edit)) {
        Ok(_) => panic!("the binding should be refused"),
        Err(e) => e,
    }
}

#[test]
fn the_compiler_checks_every_bound_channel_against_the_target() {
    // The example deploys on the LiteX target and compiles.
    let result = compile_file(&example(CHARGER));
    let plan = plan::plan(&result.ast).unwrap();
    let bindings = binding::resolve_all(&result.ast, &plan).unwrap();
    assert_eq!(bindings[0].target, LITEX);
    // A channel the target does not have.
    let e = binding_error(|t| t.replace("\"FanOn -> pwm 0 600\"", "\"FanOn -> dac 0 600\""));
    assert!(e.contains("'FanOn -> dac 0 600' cannot be deployed: target 'zephyr/litex_vexriscv' has no dac (it has gpio_in 0-31"), "{e}");
    // A channel number out of its range.
    let e = binding_error(|t| t.replace("\"Charging -> gpio 20\"", "\"Charging -> gpio 40\""));
    assert!(e.contains("'Charging -> gpio 40' cannot be deployed: target 'zephyr/litex_vexriscv' has gpio_out 0-31, not 40"), "{e}");
    let e = binding_error(|t| t.replace("\"fault <- uart 0 0x46\"", "\"fault <- uart 2 0x46\""));
    assert!(e.contains("has uart_rx 0-1, not 2"), "{e}");
    // An unknown target.
    let e = binding_error(|t| t.replace("target: \"zephyr/litex_vexriscv\"", "target: \"zephyr/nope\""));
    assert!(e.contains("io_binding 'Charger board': target: 'zephyr/nope' is no target package"), "{e}");
    let e = binding_error(|t| t.replace("target: \"zephyr/litex_vexriscv\"", "target: 3"));
    assert!(e.contains("target is the name of a target package"), "{e}");
    // The host takes the DAC; no target is the host.
    compile_text(&charger_with(|t| t.replace("target: \"zephyr/litex_vexriscv\"", "target: \"host\"").replace("\"FanOn -> pwm 0 600\"", "\"FanOn -> dac 0 600\""))).unwrap();
    let result = compile_text(&charger_with(|t| t.replace("    target: \"zephyr/litex_vexriscv\"\n", ""))).unwrap();
    let plan = plan::plan(&result.ast).unwrap();
    assert_eq!(binding::resolve_all(&result.ast, &plan).unwrap()[0].target, target::HOST);
}

// ---- the Zephyr application -----------------------------------------------------------

#[test]
fn export_writes_a_zephyr_application_tied_to_the_model() {
    let result = compile_file(&example(CHARGER));
    let app = zephyr::export(&result.ast, None, None).unwrap();
    let file = |name: &str| app.files.iter().find(|(n, _, _)| n == name).map(|(_, t, _)| t.clone()).unwrap_or_else(|| panic!("{name} in the export"));
    for name in ["CMakeLists.txt", "prj.conf", "arclang_app.cmake", "src/model/battery_charger.c", "src/model/battery_charger.h", "src/arclang_io.h", "src/glue/battery_charger_charger_board_io.c", "src/arclang_pil.c", "src/main.c", "src/pil_io.c", "src/pil_direct.c"] {
        file(name);
    }
    let header = file("src/arclang_app.h");
    assert!(header.contains("#define arclang_cycle(glue, state, inputs, outputs) battery_charger_charger_board_cycle("), "{header}");
    let config = file("src/arclang_io_config.h");
    assert!(config.contains("#define ARCLANG_IO_CAN_RX_IDS {0x210UL, 0x211UL}") && config.contains("#define ARCLANG_IO_WATCHDOG 1") && config.contains("ARCLANG_IO_WATCHDOG_TIMEOUT_MS 100U"), "{config}");
    assert!(file("arclang_app.cmake").contains("set(ARCLANG_PERIOD_MACRO BATTERY_CHARGER_PERIOD_US)"));
    assert!(app.files.iter().any(|(n, _, executable)| n == "run.sh" && *executable));
    assert!(app.notes.iter().any(|n| n.contains("adc: a register bank of this target")), "{:?}", app.notes);
    // The behavior is that of `export -f c` (the same header), but for the
    // machine the programmable logic computes: called through its driver.
    let (c_files, _) = arclang::compiler::embedded::export(&result.ast).unwrap();
    assert_eq!(c_files.iter().find(|(n, _)| n == "battery_charger.h").unwrap().1, file("src/model/battery_charger.h"));
    assert!(file("src/model/battery_charger.c").contains("battery_charger_charger_hw_step(") && !c_files.iter().find(|(n, _)| n == "battery_charger.c").unwrap().1.contains("_hw_step("));
    // Deterministic.
    let again = zephyr::export(&result.ast, None, None).unwrap();
    assert_eq!(zephyr::key_files(&app), zephyr::key_files(&again));
    // Without a binding: an application for the step() API only.
    let unbound = compile_file(&example(REMOTE_START));
    let e = zephyr::export(&unbound.ast, None, None).err().unwrap();
    assert!(e.contains("no target: give one (--target zephyr/litex_vexriscv)"), "{e}");
    let app = zephyr::export(&unbound.ast, Some(LITEX), None).unwrap();
    assert!(app.files.iter().any(|(n, t, _)| n == "arclang_app.cmake" && t.contains("set(ARCLANG_MAIN pil_direct")));
    assert!(!app.files.iter().any(|(n, _, _)| n.starts_with("src/glue/")));
    // The host is no Zephyr target.
    let e = zephyr::export(&result.ast, Some("host"), None).err().unwrap();
    assert!(e.contains("-f zephyr exports for a target of runtime zephyr"), "{e}");
}

#[test]
fn the_cli_exports_a_zephyr_application() {
    let dir = work("cli");
    let out = Command::new(env!("CARGO_BIN_EXE_arclang")).arg("export").arg(example(CHARGER)).args(["-f", "zephyr", "-o"]).arg(&dir).output().unwrap();
    assert!(out.status.success(), "{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    for file in ["CMakeLists.txt", "boards/litex_vexriscv.overlay", "src/arclang_io_zephyr.c", "renode/ArcLangSJA1000.cs", "arclang_module/zephyr/module.yml"] {
        assert!(dir.join(file).is_file(), "{file}");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert!(std::fs::metadata(dir.join("run.sh")).unwrap().permissions().mode() & 0o111 != 0);
    }
    let out = Command::new(env!("CARGO_BIN_EXE_arclang")).arg("export").arg(example(CHARGER)).args(["-f", "c", "--target", LITEX, "-o"]).arg(&dir).output().unwrap();
    assert!(!out.status.success() && String::from_utf8_lossy(&out.stderr).contains("--target and --binding apply to the Zephyr export"));
    let _ = std::fs::remove_dir_all(&dir);
}

// ---- the processor-in-the-loop link, on the host --------------------------------------

/// arclang_pil.c compiled on the host with a link over stdin/stdout, fed
/// with the lines `arclang pil` sends: its reports decode to the SIL's
/// values, bit for bit. What runs on the target is this same file.
#[test]
fn the_pil_link_carries_every_value_bit_for_bit() {
    let cc = std::env::var("ARCLANG_TEST_GCC").unwrap_or_else(|_| "gcc".to_string());
    for path in [CHARGER, WINDOW] {
        let result = compile_file(&example(path));
        let plan = plan::plan(&result.ast).unwrap();
        let bindings = binding::resolve_all(&result.ast, &plan).unwrap();
        let app = zephyr::app(&plan, None, &target::find(LITEX).unwrap()).unwrap();
        let dir = work(&format!("link-{}", path.replace('/', "_")));
        for (name, text, _) in &app.files {
            if name.starts_with("src/") {
                let file = dir.join(Path::new(name).file_name().unwrap());
                std::fs::write(file, text).unwrap();
            }
        }
        std::fs::write(
            dir.join("host_link.c"),
            "#include <stdio.h>\n#include <stdlib.h>\n#include \"arclang_pil.h\"\n\nvoid arclang_pil_putc(char c)\n{\n    (void)putchar(c);\n}\n\nint arclang_pil_getc(void)\n{\n    int c = getchar();\n    if (c == EOF) {\n        exit(0);\n    }\n    return c;\n}\n\nint main(void)\n{\n    static arclang_state s;\n    static arclang_inputs i;\n    static arclang_outputs o;\n    uint32_t k = 0U;\n    arclang_model_init(&s);\n    for (;;) {\n        if (!arclang_pil_read(&i)) {\n            return 3;\n        }\n        arclang_model_step(&s, &i, &o);\n        arclang_pil_report(k, 0U, 0, &o);\n        k++;\n    }\n}\n",
        )
        .unwrap();
        let binary = dir.join("link");
        let model_c = format!("{}.c", plan.ident);
        let out = Command::new(&cc).args(sil::STRICT_FLAGS).arg("-DARCLANG_DIRECT=1").arg("-I").arg(&dir).arg(dir.join(&model_c)).arg(dir.join("arclang_pil.c")).arg(dir.join("host_link.c")).arg("-o").arg(&binary).output().unwrap();
        assert!(out.status.success(), "{path}: {}", String::from_utf8_lossy(&out.stderr));
        for test in sil::cases(&result.ast) {
            let case = case::read(test, &plan, &bindings).unwrap().unwrap();
            if case.binding.is_some() {
                continue;
            }
            let vectors = case.vectors(&plan);
            let model = reference::run(&plan, &vectors, case.step_us);
            let stdin: String = (0..case.steps).map(|k| zephyr::direct_line(&plan, &vectors, k) + "\n").collect();
            let run = Command::new(&binary).stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).spawn().unwrap();
            use std::io::Write;
            run.stdin.as_ref().unwrap().write_all(stdin.as_bytes()).unwrap();
            let out = run.wait_with_output().unwrap();
            let text = String::from_utf8_lossy(&out.stdout);
            let rows: Vec<Vec<f64>> = text.lines().map(|l| l.strip_prefix("R ").unwrap().split(',').skip(3).map(|t| zephyr::decode(t).unwrap()).collect()).collect();
            assert_eq!(rows.len(), model.rows.len(), "{path} {}", test.id);
            for (k, (a, b)) in model.rows.iter().zip(&rows).enumerate() {
                for (n, (x, y)) in a.iter().zip(b).enumerate() {
                    assert_eq!(x.to_bits(), y.to_bits(), "{path} {} step {k} {}: model {x}, link {y}", test.id, model.names[n]);
                }
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
    // The tokens decode exactly.
    assert_eq!(zephyr::decode("x400921fb54442d18").unwrap(), std::f64::consts::PI);
    assert_eq!(zephyr::decode("x3fc00000").unwrap(), 1.5);
    assert_eq!(zephyr::decode("-42").unwrap(), -42.0);
    assert!(zephyr::decode("x12").is_err());
}

// ---- processor in the loop, in the SoC image ------------------------------------------

fn run_pil(path: &Path, extra: &[&str]) -> (bool, serde_json::Value, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_arclang")).arg("pil").arg(path).args(extra).arg("--json").output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("{e}: {stdout}{}", String::from_utf8_lossy(&out.stderr)));
    (out.status.success(), json, String::from_utf8_lossy(&out.stderr).to_string())
}

/// The charger (through its binding and through step()), the power window
/// (through its door module and through step()),
/// the cruise control and the remote start, built by the Zephyr SDK for the
/// LiteX VexRiscv SoC and run in Renode: every case reproduces the model,
/// largest difference 0; through the I/O, the period is kept on the
/// target's virtual time; the verdicts are cached as `pil`.
#[test]
#[ignore = "needs west, the Zephyr SDK and Renode: run in tools/embedded/soc (CI job Embedded SoC)"]
fn pil_on_the_litex_soc_in_renode_reproduces_the_model() {
    use arclang::compiler::verification::{verdict_for, Verdict};
    let dir = work("pil");
    for (path, cases) in [(CHARGER, 3), (WINDOW, 3), (ACC, 2), (REMOTE_START, 2)] {
        let model = dir.join(Path::new(path).file_name().unwrap());
        std::fs::copy(example(path), &model).unwrap();
        let (ok, json, stderr) = run_pil(&model, &["--target", LITEX]);
        let outcomes = json.as_array().unwrap();
        assert_eq!(outcomes.len(), cases, "{path}");
        for o in outcomes {
            assert!(o["verdict"] == "pass" && o["agrees"] == true, "{path}: {o}");
            assert_eq!(o["back_to_back"]["max_difference"], 0.0, "{path}: {o}");
            assert!(o["compiler"].as_str().unwrap().contains("riscv64-zephyr-elf-gcc") && o["compiler"].as_str().unwrap().contains("Renode"), "{o}");
            if o["binding"].is_string() {
                let timing = o["timing"].as_str().unwrap();
                assert!(timing.contains("one every 10000 us of the target's virtual time"), "{path}: {timing}");
            }
        }
        assert!(ok, "{path}: {stderr}");
        let cache = model.parent().unwrap().join(".arclang/sim-cache");
        for o in outcomes {
            let file = cache.join(format!("{}.pil.json", o["test_case"].as_str().unwrap()));
            let cached: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
            assert!(cached["kind"] == "pil" && cached["target"] == LITEX && cached["verdict"] == "pass", "{cached}");
        }
    }
    // The cached verdict stands for the case (no SIL run here), and goes stale with the C.
    let model = dir.join("battery_charger.arc");
    let result = compile_file(&model);
    let test = result.ast.test_cases.iter().find(|t| t.id == "TC-CHG-003").unwrap().clone();
    assert_eq!(verdict_for(&test, "SYS-CHG-004", &result.semantic_model), Verdict::Pass);
    let text = std::fs::read_to_string(&model).unwrap().replace("\"target := 2 A\"", "\"target := 3 A\"");
    std::fs::write(&model, text).unwrap();
    let result = compile_file(&model);
    assert!(result.warnings.iter().any(|w| w.contains("its PIL verdict on zephyr/litex_vexriscv is stale")), "{:?}", result.warnings);
    let _ = std::fs::remove_dir_all(&dir);
}

/// A wrong expectation fails on the target as on the model, and the
/// failure is the case's verdict.
#[test]
#[ignore = "needs west, the Zephyr SDK and Renode: run in tools/embedded/soc (CI job Embedded SoC)"]
fn a_failing_case_fails_in_pil_too() {
    use arclang::compiler::verification::{verdict_for, Verdict};
    let dir = work("pil-fail");
    let model = dir.join("charger.arc");
    std::fs::write(&model, charger_with(|t| t.replace("\"at(60 ms, \\\"Charger\\\".FanOn) == 1\"", "\"at(60 ms, \\\"Charger\\\".FanOn) == 0\""))).unwrap();
    let (ok, json, _) = run_pil(&model, &["--test", "TC-CHG-002"]);
    assert!(!ok);
    let o = &json.as_array().unwrap()[0];
    assert!(o["verdict"] == "fail" && o["model_verdict"] == "fail" && o["c_verdict"] == "fail" && o["agrees"] == true, "{o}");
    let result = compile_file(&model);
    let test = result.ast.test_cases.iter().find(|t| t.id == "TC-CHG-002").unwrap().clone();
    assert!(matches!(verdict_for(&test, "SYS-CHG-003", &result.semantic_model), Verdict::Fail(_)));
    assert!(result.warnings.iter().any(|w| w.contains("fails in PIL on zephyr/litex_vexriscv")), "{:?}", result.warnings);
    let _ = std::fs::remove_dir_all(&dir);
}

/// The options of a run reach the outcome: one case, a target that cannot run.
#[test]
fn pil_refuses_what_it_cannot_run() {
    let result = compile_file(&example(CHARGER));
    let options = pil::Options { target: Some("host".to_string()), work: work("pil-host"), only: None };
    let e = pil::run(&result.ast, &options).err().unwrap();
    assert!(e.contains("target 'host' has no build and emulator for processor in the loop"), "{e}");
    let options = pil::Options { target: None, work: work("pil-none"), only: Some("TC-NOPE".to_string()) };
    let e = pil::run(&result.ast, &options).err().unwrap();
    assert!(e.contains("no test_case 'TC-NOPE' that executes"), "{e}");
}

/// The charger's machine is allocated to the programmable logic: the Zephyr
/// application calls the block through its generated driver, and builds for
/// the LiteX VexRiscv SoC (the emulated SoC has no such block, so it is
/// built, not run: PIL runs the machine's software C).
#[test]
#[ignore = "needs west and the Zephyr SDK: run in tools/embedded/soc (CI job Embedded SoC)"]
fn the_zephyr_application_calling_the_hardware_block_builds_for_the_soc() {
    let dir = work("hw-app");
    let app_dir = dir.join("app");
    let out = Command::new(env!("CARGO_BIN_EXE_arclang")).arg("export").arg(example(CHARGER)).args(["-f", "zephyr", "-o"]).arg(&app_dir).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let c = std::fs::read_to_string(app_dir.join("src/model/battery_charger.c")).unwrap();
    assert!(c.contains("battery_charger_charger_hw_step("), "the step calls the block");
    let build = Command::new("west").args(["build", "-p", "auto", "-b", "litex_vexriscv", "-d"]).arg(dir.join("build")).arg(&app_dir).output().unwrap();
    assert!(build.status.success(), "{}{}", String::from_utf8_lossy(&build.stdout), String::from_utf8_lossy(&build.stderr));
    assert!(dir.join("build/zephyr/zephyr.elf").is_file());
    let _ = std::fs::remove_dir_all(&dir);
}
