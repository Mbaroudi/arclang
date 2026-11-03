//! Embedded C: the generated behavior and glue compile with GCC and Clang
//! under `-std=c99 -Wall -Wextra -Werror -pedantic`, the SIL runs reproduce
//! the model-level runs of real examples, a failing case and a departing C
//! are detected, and the bindings and executed cases are checked.
//!
//! The C compilers are found as `ARCLANG_TEST_GCC` / `ARCLANG_TEST_CLANG`,
//! else the first GCC (`gcc-15` .. `gcc-11`, `gcc` when it is GCC) and
//! `clang` on the PATH. A missing compiler fails the test: a test that
//! compiled nothing proves nothing.

use arclang::compiler::embedded::{self, binding, case, plan, reference, sil};
use arclang::{Compiler, CompilerConfig};
use std::path::{Path, PathBuf};
use std::process::Command;

fn version(compiler: &str) -> Option<String> {
    let out = Command::new(compiler).arg("--version").output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).to_string())
}

fn gcc() -> String {
    if let Ok(cc) = std::env::var("ARCLANG_TEST_GCC") {
        return cc;
    }
    let mut candidates: Vec<String> = (11..=15).rev().map(|n| format!("gcc-{n}")).collect();
    candidates.push("gcc".to_string());
    candidates
        .into_iter()
        .find(|c| version(c).is_some_and(|v| !v.to_lowercase().contains("clang")))
        .expect("no GCC found (gcc-15 .. gcc-11, or gcc that is GCC): install one or set ARCLANG_TEST_GCC")
}

fn clang() -> String {
    let cc = std::env::var("ARCLANG_TEST_CLANG").unwrap_or_else(|_| "clang".to_string());
    assert!(version(&cc).is_some_and(|v| v.to_lowercase().contains("clang")), "no clang found: install it or set ARCLANG_TEST_CLANG");
    cc
}

fn compilers() -> Vec<String> {
    vec![gcc(), clang()]
}

fn example(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples").join(path)
}

const ACC: &str = "automotive/adaptive-cruise/adaptive_cruise_control.arc";
const REMOTE_START: &str = "automotive/remote-start/remote_start_architecture.arc";
const CHARGER: &str = "energy/battery-charger/battery_charger.arc";

fn compile_file(path: &Path) -> arclang::compiler::CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_file(path).unwrap_or_else(|e| panic!("{} compiles: {e}", path.display()))
}

fn compile_text(text: &str) -> Result<arclang::compiler::CompilationResult, String> {
    Compiler::new(CompilerConfig::default()).compile_string(text).map_err(|e| e.to_string())
}

fn work(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("arclang-embedded-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Every generated file of an example, written and compiled (each `.c` on
/// its own, with the strict flags) by every compiler.
fn compiles_strictly(path: &str) {
    let result = compile_file(&example(path));
    let (files, _) = embedded::export(&result.ast).unwrap();
    let dir = work(&format!("strict-{}", path.replace('/', "_")));
    for (name, text) in &files {
        let file = dir.join(name);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, text).unwrap();
    }
    std::fs::copy(dir.join("host/arclang_io_host.h"), dir.join("arclang_io_host.h")).unwrap();
    let sources: Vec<&String> = files.iter().map(|(n, _)| n).filter(|n| n.ends_with(".c")).collect();
    assert!(sources.len() >= 2, "{sources:?}");
    for cc in compilers() {
        for source in &sources {
            let out = Command::new(&cc).args(sil::STRICT_FLAGS).arg("-c").arg("-I").arg(&dir).arg(dir.join(source)).arg("-o").arg(dir.join("unit.o")).output().unwrap();
            assert!(out.status.success(), "{cc} rejects {source} of {path}:\n{}", String::from_utf8_lossy(&out.stderr));
            assert!(out.stderr.is_empty(), "{cc} warns on {source}: {}", String::from_utf8_lossy(&out.stderr));
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_generated_c_of_the_examples_compiles_with_gcc_and_clang_under_werror() {
    for path in [ACC, REMOTE_START, CHARGER] {
        compiles_strictly(path);
    }
}

#[test]
fn the_behavior_knows_no_board() {
    let result = compile_file(&example(CHARGER));
    let plan = plan::plan(&result.ast).unwrap();
    let generated = embedded::cgen::generate(&plan, 10_000);
    let bindings = binding::resolve_all(&result.ast, &plan).unwrap();
    let glue = embedded::glue::generate(&plan, &bindings[0]);
    for text in [&generated.header, &generated.source] {
        for forbidden in ["malloc", "free(", "#include <stdio.h>", "#include <stdlib.h>", "arclang_io", "volatile", "printf"] {
            assert!(!text.contains(forbidden), "the behavior mentions {forbidden}");
        }
        // The only file-scope objects are constant tables.
        for line in text.lines().filter(|l| l.starts_with("static ") && !l.contains('(')) {
            assert!(line.starts_with("static const "), "a file-scope variable: {line}");
        }
    }
    for forbidden in ["malloc", "#include <stdio.h>", "printf"] {
        assert!(!glue.source.contains(forbidden), "the glue mentions {forbidden}");
    }
    assert!(glue.source.contains("arclang_io_gpio_read(2U") && glue.source.contains("battery_charger_step(state, inputs, outputs)"));
    // Deterministic: the same model gives the same text.
    let again = embedded::cgen::generate(&plan::plan(&result.ast).unwrap(), 10_000);
    assert_eq!(generated.source, again.source);
    assert_eq!(generated.header, again.header);
}

fn run_sil(path: &Path, compiler: &str, name: &str) -> Vec<sil::Outcome> {
    let result = compile_file(path);
    let options = sil::Options { compiler: compiler.to_string(), work: work(name), only: None };
    let outcomes = sil::run(&result.ast, &options).unwrap();
    let _ = std::fs::remove_dir_all(&options.work);
    outcomes
}

/// The model-level verdict of each expectation, from the reference run alone.
fn model_verdicts(path: &Path) -> Vec<(String, Vec<bool>)> {
    let result = compile_file(path);
    let plan = plan::plan(&result.ast).unwrap();
    let bindings = binding::resolve_all(&result.ast, &plan).unwrap();
    sil::cases(&result.ast)
        .into_iter()
        .map(|test| {
            let case = case::read(test, &plan, &bindings).unwrap().unwrap();
            let table = reference::run(&plan, &case.vectors(&plan), case.step_us);
            let texts = arclang::compiler::simulation::expects_of(&test.attributes);
            let values = sil::judge(&texts, &reference::aliases(&plan), &table).unwrap();
            (test.id.clone(), values.iter().map(|v| v.holds).collect())
        })
        .collect()
}

#[test]
fn sil_reproduces_the_model_on_real_examples_with_both_compilers() {
    for path in [ACC, REMOTE_START, CHARGER] {
        let expected = model_verdicts(&example(path));
        assert!(expected.len() >= 2, "{path} has executed cases");
        for cc in compilers() {
            let outcomes = run_sil(&example(path), &cc, &format!("sil-{}", path.replace('/', "_")));
            assert_eq!(outcomes.len(), expected.len());
            for (outcome, (id, verdicts)) in outcomes.iter().zip(&expected) {
                assert_eq!(&outcome.test_case, id);
                assert_eq!(outcome.verdict, "pass", "{path} {id} with {cc}: {}", outcome.reason);
                assert!(outcome.agrees, "{path} {id} with {cc}: {}", outcome.reason);
                let b2b = outcome.back_to_back.as_ref().unwrap();
                assert!(b2b.first_mismatch.is_none() && b2b.max_difference == 0.0 && b2b.steps > 10, "{b2b:?}");
                let model: Vec<bool> = outcome.expectations.iter().map(|j| j.model.as_ref().unwrap().holds).collect();
                let c: Vec<bool> = outcome.expectations.iter().map(|j| j.c.as_ref().unwrap().holds).collect();
                assert_eq!(&model, verdicts);
                assert_eq!(&c, verdicts);
            }
        }
    }
}

#[test]
fn the_runs_through_the_io_glue_compare_the_bound_channels_too() {
    let outcomes = run_sil(&example(ACC), &clang(), "io-acc");
    let through = outcomes.iter().find(|o| o.binding.is_some()).expect("a case through the binding");
    assert_eq!(through.verdict, "pass", "{}", through.reason);
    // 6 states + 4 bound outputs + the watchdog kicks.
    assert_eq!(through.back_to_back.as_ref().unwrap().columns, 11);
}

const FAILING: &str = r#"
test_case "TC-ACC-SIL-WRONG" {
    verifies: ["SYS-ACC-005"]
    method: "test"
    execute: {
        machine: "ACC operating modes"
        stop_time: 200 ms
        inputs: ["0 s: main switch on", "10 ms: LF-022"]
    }
    expects: ["final(\"ACC operating modes\".Cruising) == 1", "final(\"ACC operating modes\".Standby) == 1"]
}
"#;

fn acc_with(extra: &str) -> String {
    format!("{}\n{extra}", std::fs::read_to_string(example(ACC)).unwrap())
}

#[test]
fn a_failing_case_fails_on_the_model_and_on_the_c() {
    let text = acc_with(FAILING);
    let result = compile_text(&text).unwrap();
    assert!(
        result.warnings.iter().any(|w| w.contains("test_case 'TC-ACC-SIL-WRONG' fails on the model") && w.contains("final(acc_operating_modes.Cruising) = 0")),
        "{:#?}",
        result.warnings
    );
    let dir = work("failing");
    for cc in compilers() {
        let options = sil::Options { compiler: cc.clone(), work: dir.join(&cc), only: Some("TC-ACC-SIL-WRONG".into()) };
        let outcomes = sil::run(&result.ast, &options).unwrap();
        let [outcome] = outcomes.as_slice() else { panic!("one case") };
        assert_eq!((outcome.verdict.as_str(), outcome.model_verdict.as_str(), outcome.c_verdict.as_str()), ("fail", "fail", "fail"), "{}", outcome.reason);
        assert!(outcome.agrees, "the C fails as the model does");
        let holds: Vec<bool> = outcome.expectations.iter().map(|j| j.c.as_ref().unwrap().holds).collect();
        assert_eq!(holds, [false, true]);
        assert!(outcome.reason.contains("on the C run as on the model"), "{}", outcome.reason);
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_c_that_departs_from_the_model_is_detected() {
    let result = compile_file(&example(REMOTE_START));
    let plan = plan::plan(&result.ast).unwrap();
    let test = sil::cases(&result.ast)[0];
    let case = case::read(test, &plan, &[]).unwrap().unwrap();
    let vectors = case.vectors(&plan);
    let model = reference::run(&plan, &vectors, case.step_us);
    let dir = work("tampered");
    let generated = embedded::cgen::generate(&plan, case.step_us);
    // Break the C: the transition Starting -> Running enters Inhibited instead.
    let running = plan.machines[0].state("Running").unwrap();
    let inhibited = plan.machines[0].state("Inhibited").unwrap();
    let fire = generated.source.split("/* transition Starting -> Running */").nth(1).unwrap().split('}').next().unwrap().to_string();
    let tampered = generated.source.replace(&fire, &fire.replace(&format!("_enter(st, in, out, {running}U)"), &format!("_enter(st, in, out, {inhibited}U)")));
    assert_ne!(tampered, generated.source);
    std::fs::write(dir.join(&generated.header_name), &generated.header).unwrap();
    std::fs::write(dir.join(&generated.source_name), &tampered).unwrap();
    std::fs::write(dir.join("sil_main.c"), sil::direct_harness(&plan)).unwrap();
    let binary = dir.join("sil");
    sil::compile(&gcc(), &[dir.join(&generated.source_name), dir.join("sil_main.c")], &dir, &[], &binary).unwrap();
    let stdout = sil::run_binary(&binary, &sil::direct_stdin(&plan, &vectors)).unwrap();
    let c = sil::parse_rows(&stdout, &model.names, case.step_us).unwrap();
    let b2b = sil::compare(&model, &c, 0.0);
    let mismatch = b2b.first_mismatch.expect("the departure is found");
    assert!(mismatch.contains("remote_start_sequence.Running: model 1, C 0") || mismatch.contains("remote_start_sequence.Inhibited: model 0, C 1"), "{mismatch}");
    // A tolerance does not hide a state: the difference is a whole 1.
    assert_eq!(b2b.max_difference, 1.0);
    // The untampered C agrees.
    std::fs::write(dir.join(&generated.source_name), &generated.source).unwrap();
    sil::compile(&gcc(), &[dir.join(&generated.source_name), dir.join("sil_main.c")], &dir, &[], &binary).unwrap();
    let stdout = sil::run_binary(&binary, &sil::direct_stdin(&plan, &vectors)).unwrap();
    assert!(sil::compare(&model, &sil::parse_rows(&stdout, &model.names, case.step_us).unwrap(), 0.0).first_mismatch.is_none());
    let _ = std::fs::remove_dir_all(&dir);
}

fn binding_error(binding: &str) -> String {
    let text = format!("{}\n{binding}", std::fs::read_to_string(example(REMOTE_START)).unwrap());
    compile_text(&text).err().unwrap_or_else(|| panic!("the binding is refused: {binding}"))
}

#[test]
fn the_compiler_checks_the_bindings() {
    let cases: [(&str, &str); 12] = [
        (r#"io_binding "B" { period: 10 ms inputs: ["LF-NOPE <- gpio 1 rising"] }"#, "'LF-NOPE' is no event or condition of state_machine 'Remote start sequence' (events: 'LF-RX'"),
        (r#"io_binding "B" { period: 10 ms inputs: ["LF-RX <- gpio 1"] }"#, "'LF-RX' is an event, an instant: bind it to an edge (`gpio 1 rising` or `gpio 1 falling`), not to a level"),
        (r#"io_binding "B" { period: 10 ms inputs: ["driver takes over <- gpio 1 rising"] }"#, "'driver takes over' is a condition, a level: bind it to `gpio 1`, not to an edge"),
        (r#"io_binding "B" { period: 10 ms inputs: ["driver takes over <- uart 0 0x41"] }"#, "a byte on uart 0 is an instant, bind it to an event"),
        (r#"io_binding "B" { period: 10 ms inputs: ["LF-RX <- can 0x10", "LF-RX <- can 0x11"] }"#, "'LF-RX' is bound twice"),
        (r#"io_binding "B" { period: 10 ms outputs: ["Running -> gpio 9", "Idle -> gpio 9"] }"#, "gpio 9 is written by two outputs"),
        (r#"io_binding "B" { period: 10 ms inputs: ["LF-RX <- gpio 9 rising"] outputs: ["Running -> gpio 9"] }"#, "gpio 9 is read by input 'LF-RX <- gpio 9 rising' and written by output"),
        (r#"io_binding "B" { period: 10 inputs: ["LF-RX <- gpio 1 rising"] }"#, "period is a time: `10 ms`"),
        (r#"io_binding "B" { machine: "Nope" period: 10 ms inputs: ["LF-RX <- gpio 1 rising"] }"#, "machine 'Nope' is no state_machine of the model"),
        (r#"io_binding "B" { period: 10 ms inputs: ["driver takes over <- adc 2 > 65535"] }"#, "can never hold"),
        (r#"io_binding "B" { period: 10 ms inputs: ["LF-RX <- can 0x20000000"] }"#, "a CAN identifier 0x20000000 is above 536870911"),
        (r#"io_binding "B" { period: 10 ms outputs: ["Flying -> gpio 3"] }"#, "'Flying' is no state or action of state_machine 'Remote start sequence' (states: 'Idle'"),
    ];
    for (binding, expected) in cases {
        let error = binding_error(binding);
        assert!(error.contains(expected), "{binding}\n  gives: {error}\n  wanted: {expected}");
    }
    // A binding that is right compiles, and warns of what it leaves unbound.
    let text = format!("{}\nio_binding \"B\" {{ period: 5 ms inputs: [\"LF-RX <- can 0x101\"] outputs: [\"Running -> gpio 9\"] }}", std::fs::read_to_string(example(REMOTE_START)).unwrap());
    let result = compile_text(&text).unwrap();
    assert!(result.warnings.iter().any(|w| w.contains("event 'LF-SIG' is bound to no channel: on the target it never occurs")), "{:#?}", result.warnings);
}

#[test]
fn the_compiler_checks_the_executed_cases() {
    let case = |execute: &str, expects: &str| {
        format!("test_case \"TC-X\" {{ verifies: [\"SYS-RS-001\"] execute: {{ {execute} }} expects: [{expects}] }}")
    };
    let ok_expect = r#""final(\"Remote start sequence\".Idle) == 1""#;
    let cases = [
        (case(r#"stop_time: 100 ms inputs: ["7 ms: LF-RX"]"#, ok_expect), "execute: input '7 ms: LF-RX' is not on the step grid (10000 us)"),
        (case(r#"stop_time: 100 ms inputs: ["10 ms: LF-NOPE"]"#, ok_expect), "'LF-NOPE' is no event, condition or input variable of state_machine 'Remote start sequence'"),
        (case(r#"stop_time: 100 ms inputs: ["200 ms: LF-RX"]"#, ok_expect), "comes after stop_time"),
        (case(r#"stop_time: 100 ms"#, r#""final(\"Remote start sequence\".Flying) == 1""#), "'remote_start_sequence.Flying' is no state, action or variable of the machines"),
        (case(r#"stop_time: 100 ms speed: 3"#, ok_expect), "execute: 'speed' is not one of machine, step, stop_time, inputs, binding, tolerance"),
        (case(r#"inputs: []"#, ok_expect), "execute: stop_time is required"),
        (case(r#"stop_time: 100 ms binding: "Nope""#, ok_expect), "binding 'Nope' is no io_binding of the model"),
        (case(r#"stop_time: 100 ms"#, ""), "execute: needs expects"),
    ];
    for (test, expected) in cases {
        let text = format!("{}\n{test}", std::fs::read_to_string(example(REMOTE_START)).unwrap());
        let error = compile_text(&text).err().unwrap_or_else(|| panic!("refused: {test}"));
        assert!(error.contains(expected), "{test}\n  gives: {error}\n  wanted: {expected}");
    }
}

#[test]
fn vectors_that_the_io_cannot_produce_are_reported() {
    // Two events in consecutive steps cannot both be rising edges of one pin.
    let text = format!(
        "{}\nio_binding \"B\" {{ period: 10 ms inputs: [\"LF-RX <- gpio 1 rising\"] }}\ntest_case \"TC-EDGE\" {{ verifies: [\"SYS-RS-001\"] execute: {{ binding: \"B\" stop_time: 50 ms inputs: [\"0 ms: LF-RX\", \"10 ms: LF-RX\"] }} expects: [\"final(\\\"Remote start sequence\\\".Authenticating) == 1\"] }}",
        std::fs::read_to_string(example(REMOTE_START)).unwrap()
    );
    let result = compile_text(&text).unwrap();
    let options = sil::Options { compiler: clang(), work: work("edge"), only: Some("TC-EDGE".into()) };
    let outcomes = sil::run(&result.ast, &options).unwrap();
    assert_eq!(outcomes[0].verdict, "error");
    assert!(outcomes[0].reason.contains("the vectors cannot be produced on gpio 1"), "{}", outcomes[0].reason);
    // The model-level run is judged all the same.
    assert_eq!(outcomes[0].model_verdict, "pass");
    let _ = std::fs::remove_dir_all(&options.work);
}

#[test]
fn a_transition_across_regions_is_refused() {
    let text = r#"model T { name: "t" }
state_machine "Two" {
  region "A" { initial: "A1" state A1 {} state A2 {} transition A1 -> A2 { trigger: "go" } }
  region "B" { initial: "B1" state B1 {} transition B1 -> A2 { trigger: "jump" } }
}
"#;
    let result = compile_text(text).unwrap();
    let error = plan::plan(&result.ast).unwrap_err();
    assert!(error.contains("transition B1 -> A2 goes from one region to another; such a transition is not generated"), "{error}");
}

#[test]
fn the_cli_exports_c_and_runs_sil() {
    let dir = work("cli");
    let exe = env!("CARGO_BIN_EXE_arclang");
    let out = Command::new(exe).args(["export"]).arg(example(CHARGER)).args(["-f", "c", "-o"]).arg(&dir).output().unwrap();
    assert!(out.status.success(), "{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    for file in ["battery_charger.h", "battery_charger.c", "arclang_io.h", "battery_charger_charger_board_io.h", "battery_charger_charger_board_io.c", "host/arclang_io_host.c", "host/arclang_io_host.h"] {
        assert!(dir.join(file).is_file(), "{file} written");
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("action(s) written in prose are reported (counted per step), not executed"), "{stdout}");
    let out = Command::new(exe).arg("sil").arg(example(CHARGER)).args(["--cc", &gcc(), "--json"]).output().unwrap();
    assert!(out.status.success(), "{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json.as_array().unwrap().len(), 3);
    assert!(json.as_array().unwrap().iter().all(|o| o["verdict"] == "pass" && o["agrees"] == true), "{json}");
    // A failing case makes the command fail.
    let model = dir.join("failing.arc");
    std::fs::write(&model, acc_with(FAILING)).unwrap();
    let out = Command::new(exe).arg("sil").arg(&model).args(["--cc", &clang(), "--test", "TC-ACC-SIL-WRONG"]).output().unwrap();
    assert!(!out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("✗ FAIL  TC-ACC-SIL-WRONG") && stdout.contains("model: fail · C: fail · the C reproduces the model"), "{stdout}");
    let _ = std::fs::remove_dir_all(&dir);
}

// ---- the action language -------------------------------------------------------------

/// A machine exercising every operator, type, unit conversion and block,
/// with inputs that hit the corners (saturation, division by zero, limits).
const ARITHMETIC: &str = r#"model Arith { name: "arith" }
requirements system { req "R-1" "Arithmetic" { description: "d" } }
state_machine "Calc" {
    variables: [
        "in speed: Real [km/h]",
        "in n: Integer",
        "in d: Integer",
        "in x: Float",
        "in on: Boolean",
        "out ms: Real [m/s]",
        "out big: Integer = 2147483000",
        "out q: Integer",
        "out neg: Integer",
        "out f: Float = 0.5",
        "out lim: Real",
        "out mx: Integer",
        "out ab: Real",
        "out held: Boolean",
        "out late: Integer = 7",
        "out acc: Real [m]",
        "out u: Real",
        "out count: Integer = 0",
        "out fast: Boolean"
    ]
    equations: [
        "ms = speed",
        "big = big + n * 1000",
        "q = n / d",
        "neg = -n - 2147483647",
        "f = f * 0.75 + x / 3",
        "lim = saturate(speed / 1 km/h * 0.1, -2.5, 2.5)",
        "mx = max(n, d) - min(n, 3) + abs(d)",
        "ab = abs(ms / 1 m/s - 20)",
        "held = delay(on)",
        "late = delay(n, 7)",
        "acc = integrator(ms, 1, 0 m, 1 m)",
        "u = pid(1 - lim, 2, 4 Hz, 0.01 s)",
        "fast = speed > 30 km/h and not (n == 0) or on"
    ]
    initial: "Slow"
    state Slow { entry: "count := count + 1" }
    state Fast { entry: ["count := count + 10", "beep"] do: "count := count + 1" }
    transition Slow -> Fast { guard: "ms >= 10 m/s and count < 5" action: "count := count * 2" }
    transition Fast -> Slow { trigger: "brake" guard: "speed < 20 km/h or on" }
}
test_case "TC-ARITH" {
    verifies: ["R-1"]
    execute: {
        machine: "Calc"
        stop_time: 300 ms
        inputs: [
            "0 s: speed = 10 km/h", "0 s: n = 3", "0 s: d = 0", "0 s: x = 1.5",
            "50 ms: speed = 54 km/h", "60 ms: d = -2", "80 ms: n = 2000000",
            "100 ms: on = true", "120 ms: x = -7.25", "150 ms: speed = 12 km/h",
            "160 ms: brake", "200 ms: n = 0", "200 ms: on = false"
        ]
    }
    expects: [
        "R-1: at(40 ms, \"Calc\".ms) == 10 km/h",
        "R-1: at(70 ms, \"Calc\".ms) == 15 m/s",
        "R-1: final(\"Calc\".big) == 2147483647",
        "R-1: at(40 ms, \"Calc\".q) == 0",
        "R-1: at(70 ms, \"Calc\".q) == -1",
        "R-1: at(80 ms, \"Calc\".neg) == -2147483648",
        "R-1: max(\"Calc\".lim) == 2.5",
        "R-1: max(\"Calc\".acc) == 1 m",
        "R-1: at(100 ms, \"Calc\".held) == 0",
        "R-1: at(110 ms, \"Calc\".held) == 1",
        "R-1: at(80 ms, \"Calc\".late) == 3",
        "R-1: at(90 ms, \"Calc\".late) == 2000000",
        "R-1: at(60 ms, \"Calc\".Fast) == 1",
        "R-1: at(170 ms, \"Calc\".Slow) == 1",
        "R-1: at(50 ms, \"Calc\".count) == 13",
        "R-1: at(50 ms, \"Calc\".action.beep) == 1"
    ]
}
"#;

#[test]
fn the_action_language_runs_alike_in_the_model_and_in_the_c() {
    let dir = work("arith-src");
    let model = dir.join("arith.arc");
    std::fs::write(&model, ARITHMETIC).unwrap();
    let expected = model_verdicts(&model);
    assert_eq!(expected[0].1.len(), 16);
    assert!(expected[0].1.iter().all(|v| *v), "every expectation holds on the model: {expected:?}");
    for cc in compilers() {
        let outcomes = run_sil(&model, &cc, "arith");
        let o = &outcomes[0];
        assert_eq!(o.verdict, "pass", "{cc}: {}", o.reason);
        let b2b = o.back_to_back.as_ref().unwrap();
        // Bit for bit: Integer saturation, Float, unit conversions, the blocks.
        assert!(b2b.first_mismatch.is_none() && b2b.max_difference == 0.0, "{cc}: {b2b:?}");
    }
    // The C: typed fields, saturating helpers, blocks, no contraction.
    let result = compile_file(&model);
    let plan = plan::plan(&result.ast).unwrap();
    let generated = embedded::cgen::generate(&plan, 10_000);
    for wanted in ["double v_speed; /* in variable \"speed\" [km/h]", "int32_t v_big;", "float v_f;", "bool v_on;", "static int32_t arc_add_i32", "static int32_t arc_div_i32", "#pragma STDC FP_CONTRACT OFF", "_pid_state blk_"] {
        assert!(generated.header.contains(wanted) || generated.source.contains(wanted), "{wanted}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

fn machine_error(variables: &str, rest: &str) -> String {
    let text = format!("model T {{ name: \"t\" }}\nstate_machine \"M\" {{\n  variables: [{variables}]\n  initial: \"A\"\n  state A {{}}\n  state B {{}}\n  {rest}\n}}\n");
    compile_text(&text).err().unwrap_or_else(|| panic!("refused: {text}"))
}

#[test]
fn the_compiler_checks_types_units_and_names_of_the_code() {
    let cases = [
        (r#""in speed: Real [km/h]""#, r#"transition A -> B { guard: "speed > 3 s" }"#, "guard \"speed > 3 s\": '>' between a speed and a time"),
        (r#""in speed: Real [km/h]""#, r#"transition A -> B { guard: "speed + 1 km/h" }"#, "guard \"speed + 1 km/h\" is a Real, not a Boolean"),
        (r#""in speed: Real [km/h]""#, r#"transition A -> B { guard: "speed > limt" }"#, "'limt' is no variable of the machine"),
        (r#""n: Integer", "in speed: Real""#, r#"transition A -> B { action: "n := speed" }"#, "a Real assigned to Integer 'n'"),
        (r#""in speed: Real""#, r#"transition A -> B { action: "speed := 1" }"#, "'speed' is an input (in)"),
        (r#""const k: Real = 2""#, r#"transition A -> B { action: "k := 3" }"#, "'k' is a constant"),
        (r#""on: Boolean""#, r#"transition A -> B { guard: "on + 1 > 0" }"#, "'+' between a Boolean and a Integer: it takes numbers"),
        (r#""x: Real [V]""#, r#"transition A -> B { action: "x := 3 A" }"#, "a electric current assigned to 'x', a voltage"),
        (r#""x: Complex""#, "", "'Complex' is not a type (Boolean, Integer, Real, Float)"),
        (r#""x: Real", "x: Integer""#, "", "variable 'x' is declared twice"),
        (r#""t: Real [°C]""#, "", "'°C' has an offset"),
        (r#""in u: Real", "y: Real""#, r#"transition A -> B { action: "y := delay(u)" }"#, "delay(...) keeps a state from step to step: it is written in equations:"),
    ];
    for (variables, rest, expected) in cases {
        let error = machine_error(variables, rest);
        assert!(error.contains(expected), "{variables} {rest}\n  gives: {error}\n  wanted: {expected}");
    }
    let equations = |eqs: &str| {
        let text = format!("model T {{ name: \"t\" }}\nstate_machine \"M\" {{\n  variables: [\"in e: Real [m]\", \"y: Real [N]\", \"z: Real\"]\n  equations: [{eqs}]\n  initial: \"A\"\n  state A {{}}\n}}\n");
        compile_text(&text).err().unwrap_or_default()
    };
    assert!(equations(r#""y = pid(e, 2 N / 1 m, 1 N / 1 m, 0 s)""#).contains("pid(...): ki*e*time is a"), "{}", equations(r#""y = pid(e, 2 N / 1 m, 1 N / 1 m, 0 s)""#));
    assert!(equations(r#""z = 1", "z = 2""#).contains("'z' is already computed by"));
    assert!(equations(r#""y = e""#).contains("a length assigned to 'y', a force"));
    assert!(equations(r#""z = delay(delay(e / 1 m))""#).contains("inside the arguments of a block"));
    // Prose stays prose: a machine with variables keeps its text guards and actions.
    let text = "model T { name: \"t\" }\nstate_machine \"M\" {\n  variables: [\"n: Integer\"]\n  initial: \"A\"\n  state A {}\n  state B {}\n  transition A -> B { trigger: \"go\" guard: \"lead vehicle in range\" action: \"close contactors\" }\n}\n";
    let result = compile_text(text).unwrap();
    let plan = plan::plan(&result.ast).unwrap();
    assert_eq!(plan.machines[0].conditions.len(), 1);
    assert!(plan.machines[0].program().guards.is_empty() && plan.machines[0].program().actions.is_empty());
}

