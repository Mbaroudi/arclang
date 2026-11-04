//! The hardware/software split: machines allocated to programmable logic,
//! their fixed-point formats, what the hardware refuses, the VHDL (the
//! same bytes for the same model), the register map and the driver, the
//! bit-exact fixed-point C reference and the Zephyr export calling the
//! block; and, in the FPGA image (tools/embedded/fpga), `arclang hdl` in
//! GHDL against the fixed-point C (difference 0) and the model (within the
//! quantisation tolerance), and synthesis for the ECP5 meeting timing.
//!
//! The tests that need GHDL, Yosys and nextpnr are `#[ignore]`d on a bare
//! host and run in the image: `tools/embedded/fpga/run.sh cargo test --test
//! embedded_fpga -- --include-ignored` (CI job "Embedded FPGA").

use arclang::compiler::embedded::{alloc, fixed, fxc, hdl, plan, regmap, sil, vhdl, zephyr};
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

fn compile_text(text: &str) -> Result<arclang::compiler::CompilationResult, String> {
    Compiler::new(CompilerConfig::default()).compile_string(text).map_err(|e| e.to_string())
}

fn work(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("arclang-fpga-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A machine with these variables and equations on a node of kind fpga.
fn on_fpga(variables: &[&str], equations: &[&str]) -> String {
    format!(
        "physical_architecture \"HW\" {{\n    node \"Fabric\" {{ kind: \"fpga\" deploys \"M\" }}\n}}\nstate_machine \"M\" {{\n    variables: [{}]\n    equations: [{}]\n    initial: \"A\"\n    state A {{}}\n}}\n",
        variables.iter().map(|v| format!("\"{v}\"")).collect::<Vec<_>>().join(", "),
        equations.iter().map(|v| format!("\"{v}\"")).collect::<Vec<_>>().join(", ")
    )
}

// ---- allocation -----------------------------------------------------------------------

#[test]
fn machines_are_allocated_to_programmable_logic_as_arcadia_allocates_behavior() {
    let charger = compile_file(&example(CHARGER));
    let p = plan::plan(&charger.ast).unwrap();
    let found = alloc::allocations(&charger.ast, &p).unwrap();
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!((p.machines[found[0].machine].name.as_str(), found[0].node.as_str()), ("Charger", "Charger fabric"), "by its owner, a behavior component of the node");
    let device = found[0].device().unwrap();
    assert_eq!((device.size.as_str(), device.package.as_str(), device.speed, found[0].clock_hz), ("25k", "CABGA381", 6, 25e6));
    let window = compile_file(&example(WINDOW));
    let p = plan::plan(&window.ast).unwrap();
    assert_eq!(alloc::allocations(&window.ast, &p).unwrap().len(), 1, "deployed by name");
    // A node that is not programmable logic allocates nothing.
    let text = on_fpga(&["in v: Real"], &[]).replace("kind: \"fpga\"", "processor: \"x\"");
    let model = compile_text(&text).unwrap();
    assert!(alloc::allocations(&model.ast, &plan::plan(&model.ast).unwrap()).unwrap().is_empty());
    // The part is read when synthesising (the VHDL itself is vendor neutral).
    let other = compile_text(&on_fpga(&["in v: Real range 0 .. 1"], &[]).replace("kind: \"fpga\"", "kind: \"fpga\" device: \"XC7A35T\"")).unwrap();
    let found = alloc::allocations(&other.ast, &plan::plan(&other.ast).unwrap()).unwrap();
    assert!(found[0].device().unwrap_err().contains("node 'Fabric': device 'XC7A35T' is no Lattice ECP5 part"));
}

// ---- fixed point ----------------------------------------------------------------------

#[test]
fn the_formats_of_the_charger_come_from_its_ranges() {
    let charger = compile_file(&example(CHARGER));
    let p = plan::plan(&charger.ast).unwrap();
    let m = &p.machines[0];
    let fx = fixed::machine(m, 10_000).unwrap();
    let format = |name: &str| {
        let i = m.variable(name).unwrap();
        (fx.vars[i].kind.label(), fx.vars[i].origin)
    };
    assert_eq!(format("voltage"), ("Q6.25".to_string(), "declared"));
    assert_eq!(format("current"), ("Q5.26".to_string(), "declared"));
    assert_eq!(format("filtered"), ("Q6.25".to_string(), "declared"), "fed back: declared");
    assert_eq!(format("charge"), ("Q4.27".to_string(), "declared"));
    assert_eq!(format("target"), ("Q5.26".to_string(), "derived"), "0, 2, 16, 4 A: below 2^5");
    assert_eq!(format("setpoint"), ("Q5.26".to_string(), "derived"), "the limits of the pid");
    assert_eq!(format("v_absorption"), ("Q6.25".to_string(), "constant"));
    assert_eq!(format("plugs"), ("int32".to_string(), "int32"));
    let target = m.variable("target").unwrap();
    assert_eq!(fx.vars[target].range, (0.0, 16.0));
    let error = m.variable("error").unwrap();
    assert_eq!(fx.vars[error].range, (-25.0, 41.0), "target - current");
}

#[test]
fn what_the_hardware_cannot_compute_is_refused_naming_the_construct() {
    let div = compile_text(&on_fpga(&["in a: Real range 1 .. 2", "in b: Real range 1 .. 2", "out y: Real"], &["y = a / b"])).unwrap_err();
    assert!(div.contains("'/' by an expression that is not a constant") && div.contains("allocated to programmable logic node 'Fabric'"), "{div}");
    let range = compile_text(&on_fpga(&["in a: Real", "out y: Real"], &["y = a"])).unwrap_err();
    assert!(range.contains("in variable 'a' (Real) has no range"), "{range}");
    let fed = compile_text(&on_fpga(&["in a: Real range 0 .. 1", "f: Real"], &["f = f + 0.5 * (a - f)"])).unwrap_err();
    assert!(fed.contains("'f' cannot be derived: it is fed back"), "{fed}");
    let pid = compile_text(&on_fpga(&["in e: Real range -1 .. 1", "out y: Real"], &["y = pid(e, 1, 1 Hz, 0 s)"])).unwrap_err();
    assert!(pid.contains("a pid without limits"), "{pid}");
    let wide = compile_text(&on_fpga(&["in a: Real range 0 .. 3000000000", "out y: Real"], &["y = a"])).unwrap_err();
    assert!(wide.contains("needs more than 31 integer bits"), "{wide}");
    // The same machine off the FPGA is software: nothing is asked of it.
    let soft = on_fpga(&["in a: Real", "in b: Real", "out y: Real"], &["y = a / b"]).replace("kind: \"fpga\" ", "");
    assert!(compile_text(&soft).is_ok());
    assert!(compile_text(&on_fpga(&["in a: Real range 0 .. 2", "out y: Real"], &["y = a / 4 + min(a, 1) - abs(a) * 3"])).is_ok(), "division by a constant, min, abs, *");
}

// ---- generated files ------------------------------------------------------------------

#[test]
fn the_vhdl_is_the_same_bytes_for_the_same_model_and_has_the_shape_described() {
    let a = hdl::export(&compile_file(&example(CHARGER)).ast).unwrap().0;
    let b = hdl::export(&compile_file(&example(CHARGER)).ast).unwrap().0;
    assert_eq!(a, b, "deterministic");
    let names: Vec<&str> = a.iter().map(|(n, _)| n.as_str()).collect();
    for f in ["arclang_fx_pkg.vhd", "battery_charger_charger.vhd", "battery_charger_charger_wb.vhd", "battery_charger_charger_regs.json", "battery_charger_charger_csr.csv", "battery_charger_charger_hw.h", "c/battery_charger_fx.c", "c/battery_charger_fx.h"] {
        assert!(names.contains(&f), "{f} in {names:?}");
    }
    let core = &a.iter().find(|(n, _)| n == "battery_charger_charger.vhd").unwrap().1;
    for wanted in ["entity battery_charger_charger is", "use ieee.numeric_std.all;", "rising_edge(clk)", "if rst = '1' then", "step  : in", "ready : out", "done  : out", "fx_resize(", "procedure exit_range", "o_v_setpoint : out signed(31 downto 0)"] {
        let found = core.contains(wanted) || core.replace("  ", " ").contains(&wanted.replace("  ", " "));
        assert!(found, "{wanted}");
    }
    for vendor in ["library ecp5", "library unisim", "EHXPLLL", "MULT18X18D", "altera"] {
        assert!(!core.to_lowercase().contains(&vendor.to_lowercase()), "no vendor primitive: {vendor}");
    }
    let map: serde_json::Value = serde_json::from_str(&a.iter().find(|(n, _)| n.ends_with("_regs.json")).unwrap().1).unwrap();
    assert_eq!(map["registers"][0]["name"], "CTRL");
    assert_eq!(map["registers"][1]["offset"], "0x004");
    let voltage = map["registers"].as_array().unwrap().iter().find(|r| r["name"] == "IN_V_VOLTAGE").unwrap();
    assert_eq!(voltage["format"], "Q6.25");
    let driver = &a.iter().find(|(n, _)| n.ends_with("_hw.h")).unwrap().1;
    assert!(driver.contains("static inline int battery_charger_charger_hw_step(") && driver.contains("arc_fx_from((double)in->v_voltage, 25)"), "{driver}");
}

#[test]
fn the_window_machine_has_every_pseudo_state_in_vhdl() {
    let model = compile_file(&example(WINDOW));
    let files = hdl::export(&model.ast).unwrap().0;
    let core = &files.iter().find(|(n, _)| n == "power_window_window.vhd").unwrap().1;
    for wanted in ["has_", "deep_", "hist_", "terminated := true;", "procedure enter_default", "-- choice"] {
        assert!(core.contains(wanted), "{wanted}");
    }
}

#[test]
fn the_fixed_point_reference_compiles_strictly_and_without_floating_point() {
    let model = compile_file(&example(CHARGER));
    let p = plan::plan(&model.ast).unwrap();
    let fx_plan = fxc::plan(&p, &[0], 10_000).unwrap();
    let generated = arclang::compiler::embedded::cgen::generate(&fx_plan, 10_000);
    assert!(!generated.source.contains("double") && !generated.header.contains("double"), "no floating point in the reference");
    let dir = work("fxc");
    std::fs::write(dir.join(&generated.header_name), &generated.header).unwrap();
    let source = dir.join(&generated.source_name);
    std::fs::write(&source, &generated.source).unwrap();
    let cc = std::env::var("CC").unwrap_or_else(|_| "cc".to_string());
    let out = Command::new(&cc).args(sil::STRICT_FLAGS).arg("-c").arg(&source).arg("-I").arg(&dir).arg("-DBATTERY_CHARGER_FX_PERIOD_US=10000UL").arg("-o").arg(dir.join("fx.o")).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let wrong = Command::new(&cc).args(sil::STRICT_FLAGS).arg("-c").arg(&source).arg("-I").arg(&dir).arg("-DBATTERY_CHARGER_FX_PERIOD_US=20000UL").arg("-o").arg(dir.join("fx2.o")).output().unwrap();
    assert!(!wrong.status.success() && String::from_utf8_lossy(&wrong.stderr).contains("generated for a period of 10000 us"), "another period is refused");
}

#[test]
fn a_model_without_allocation_generates_the_same_c_as_before() {
    // The C of a software-only machine does not depend on the new code.
    let model = compile_file(&example("automotive/adaptive-cruise/adaptive_cruise_control.arc"));
    let (files, _) = arclang::compiler::embedded::export(&model.ast).unwrap();
    let c = &files.iter().find(|(n, _)| n.ends_with(".c") && !n.contains('/') && !n.ends_with("_io.c")).unwrap().1;
    assert!(!c.contains("_hw.h") && !c.contains("arc_fx_"), "no hardware in software-only C");
    assert!(hdl::export(&model.ast).unwrap_err().contains("no state_machine is allocated to programmable logic"));
}

#[test]
fn the_zephyr_export_calls_the_block_through_its_driver() {
    let model = compile_file(&example(CHARGER));
    let app = zephyr::export(&model.ast, None, None).unwrap();
    let file = |name: &str| app.files.iter().find(|(n, _, _)| n == name).map(|(_, t, _)| t.clone()).unwrap_or_else(|| panic!("{name}"));
    let c = file("src/model/battery_charger.c");
    assert!(c.contains("#include \"battery_charger_charger_hw.h\"") && c.contains("(void)battery_charger_charger_hw_step(&inputs->charger, &outputs->charger);") && c.contains("battery_charger_charger_hw_reset();"), "{c}");
    assert!(!c.contains("static void charger_run("), "the software step of the machine is not generated");
    assert!(file("src/model/battery_charger_charger_hw.h").contains("BATTERY_CHARGER_CHARGER_HW_BASE"));
    assert!(file("arclang_app.cmake").contains("BATTERY_CHARGER_CHARGER_HW_BASE=${BATTERY_CHARGER_CHARGER_HW_BASE}"));
    assert!(app.files.iter().any(|(n, _, _)| n == "hw/battery_charger_charger_wb.vhd"));
    assert!(app.notes.iter().any(|n| n.contains("computed by the programmable logic")), "{:?}", app.notes);
}

#[test]
fn the_register_map_names_every_input_and_output() {
    let model = compile_file(&example(WINDOW));
    let p = plan::plan(&model.ast).unwrap();
    let m = &p.machines[0];
    let fx = fixed::machine(m, 10_000).unwrap();
    let map = regmap::map(&p, m, &fx);
    assert_eq!(map.find(&regmap::Reg::EvQueue(15)).map(|r| r.name.as_str()), Some("EV_QUEUE15"));
    assert_eq!(map.outputs().len(), regmap::words(m.states.len()) + m.actions.len() + fx.vars.len() + regmap::words(m.budgets.len()) + 1);
    assert!(vhdl::wrapper(&p, m, &fx, &map).contains("entity power_window_window_wb is"));
    let csv = regmap::csv(&map);
    assert!(csv.contains("csr_register,power_window_window_ctrl,0x00000000,1,rw"), "{csv}");
}

// ---- in the FPGA image ----------------------------------------------------------------

fn run_hdl(path: &str, synth: bool) -> (Vec<hdl::Outcome>, Vec<Result<hdl::Synthesis, String>>) {
    let model = compile_file(&example(path));
    let options = hdl::Options { work: work(&format!("hdl-{}", path.replace('/', "_"))), only: None, synth, compiler: "gcc".into() };
    hdl::run(&model.ast, &options).unwrap()
}

fn assert_passes(outcomes: &[hdl::Outcome]) {
    assert!(!outcomes.is_empty());
    for o in outcomes {
        assert_eq!(o.verdict, "pass", "{}: {}", o.test_case, o.reason);
        let b = o.bit_exact.as_ref().unwrap();
        assert_eq!((b.max_difference, b.first_mismatch.as_deref()), (0.0, None), "{}: bit-exact", o.test_case);
        let q = o.quantisation.as_ref().unwrap();
        assert!(q.first_mismatch.is_none() && q.max_difference <= q.tolerance, "{}: {q:?}", o.test_case);
    }
}

#[test]
#[ignore = "needs GHDL and gcc of tools/embedded/fpga: run in its image (CI job Embedded FPGA)"]
fn the_charger_controller_runs_in_ghdl_bit_exact_with_the_fixed_point_c() {
    let (outcomes, _) = run_hdl(CHARGER, false);
    assert_eq!(outcomes.len(), 3);
    assert_passes(&outcomes);
    let pid = outcomes.iter().find(|o| o.test_case == "TC-CHG-003").unwrap();
    let q = pid.quantisation.as_ref().unwrap();
    assert!(q.max_difference > 0.0 && q.max_difference < 1e-4, "the quantisation is seen, and small: {q:?}");
}

#[test]
#[ignore = "needs GHDL and gcc of tools/embedded/fpga: run in its image (CI job Embedded FPGA)"]
fn the_window_machine_runs_in_ghdl_bit_exact_with_the_fixed_point_c() {
    let (outcomes, _) = run_hdl(WINDOW, false);
    assert_eq!(outcomes.len(), 2);
    assert_passes(&outcomes);
}

#[test]
#[ignore = "needs GHDL, Yosys and nextpnr-ecp5 of tools/embedded/fpga: run in its image (CI job Embedded FPGA)"]
fn both_blocks_synthesise_for_the_ecp5_and_meet_timing() {
    for path in [CHARGER, WINDOW] {
        let model = compile_file(&example(path));
        let p = plan::plan(&model.ast).unwrap();
        let a = &alloc::allocations(&model.ast, &p).unwrap()[0];
        let s = hdl::synthesise(&p, a, 10_000, &work(&format!("synth-{}", path.replace('/', "_")))).unwrap();
        assert!(s.timing_met, "{path}: Fmax {} MHz for {} MHz", s.fmax_mhz, s.clock_mhz);
        assert!(s.luts > 0 && s.ffs > 0 && Path::new(&s.bitstream).is_file(), "{s:?}");
    }
}

#[test]
#[ignore = "needs GHDL and gcc of tools/embedded/fpga: run in its image (CI job Embedded FPGA)"]
fn the_cli_exports_vhdl_that_ghdl_analyses_and_runs_hdl() {
    let dir = work("cli");
    let bin = env!("CARGO_BIN_EXE_arclang");
    let out = Command::new(bin).args(["export"]).arg(example(WINDOW)).args(["-f", "vhdl", "-o"]).arg(&dir).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let analyse = Command::new("ghdl").args(["-a", "--std=08", "arclang_fx_pkg.vhd", "power_window_window.vhd", "power_window_window_wb.vhd"]).current_dir(&dir).output().unwrap();
    assert!(analyse.status.success(), "{}", String::from_utf8_lossy(&analyse.stderr));
    let synth = Command::new("ghdl").args(["--synth", "--std=08", "power_window_window_wb"]).current_dir(&dir).output().unwrap();
    assert!(synth.status.success(), "{}", String::from_utf8_lossy(&synth.stderr));
    // The verdicts are cached, kind hdl, where check reads them.
    let model_dir = work("cli-model");
    let model = model_dir.join("power_window.arc");
    std::fs::copy(example(WINDOW), &model).unwrap();
    let run = Command::new(bin).arg("hdl").arg(&model).output().unwrap();
    let text = String::from_utf8_lossy(&run.stdout);
    assert!(run.status.success(), "{text}{}", String::from_utf8_lossy(&run.stderr));
    assert!(text.contains("largest difference 0 (bit-exact)"), "{text}");
    let cached = model_dir.join(".arclang/sim-cache/TC-PW-001.hdl.json");
    let outcome: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&cached).unwrap()).unwrap();
    assert_eq!((outcome["kind"].as_str(), outcome["verdict"].as_str()), (Some("hdl"), Some("pass")));
    let check = Command::new(bin).arg("check").arg(&model).output().unwrap();
    let said = format!("{}{}", String::from_utf8_lossy(&check.stdout), String::from_utf8_lossy(&check.stderr));
    assert!(!said.contains("HDL verdict is stale"), "{said}");
}
