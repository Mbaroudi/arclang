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

// ---- the conversions of the driver ----------------------------------------------------

#[test]
fn the_driver_converts_with_integers_only_and_gives_the_words_of_the_floating_point_formula() {
    // arc_fx_from / arc_fx_to read the IEEE 754 bits: no floating point on a
    // processor without an FPU. Checked against floor(ldexp(v, f) + 0.5)
    // (saturated) and ldexp(w, -f) on edge cases and random values.
    let dir = work("conversions");
    let harness = format!(
        "#include <math.h>\n#include <stdbool.h>\n#include <stdint.h>\n#include <stdio.h>\n#include <string.h>\n{}\n\
static int32_t reference(double v, int f) {{ double s; if (isnan(v)) return 0; s = floor(ldexp(v, f) + 0.5); if (s >= 2147483647.0) return INT32_MAX; if (s <= -2147483648.0) return INT32_MIN; return (int32_t)s; }}\n\
static uint64_t state = 88172645463325252ULL;\nstatic uint64_t next(void) {{ state ^= state << 13; state ^= state >> 7; state ^= state << 17; return state; }}\n\
int main(void) {{\n\
  const double edges[] = {{0.0, -0.0, 0.5, -0.5, 1.5, -1.5, 2.5, -2.5, 54.6, -54.6, 1e-300, -1e-300, 4.9e-324, 1e300, -1e300, 2147483647.0, 2147483647.5, -2147483648.0, -2147483648.5, 1.0/0.0, -1.0/0.0, 0.0/0.0}};\n\
  long bad = 0; long n = 0; int f; unsigned k;\n\
  for (f = 0; f <= 40; f++) for (k = 0; k < sizeof edges / sizeof edges[0]; k++) {{ double v = ldexp(edges[k], -(f % 7)); n++; if (arc_fx_from(v, f) != reference(v, f)) {{ bad++; if (bad < 5) printf(\"from %.17g %d: %d %d\\n\", v, f, arc_fx_from(v, f), reference(v, f)); }} }}\n\
  for (k = 0; k < 2000000U; k++) {{ uint64_t b = next(); double v; int32_t w; f = (int)(next() % 48U); b = (b & 0x800FFFFFFFFFFFFFULL) | ((uint64_t)(1023U - 40U + (unsigned)(next() % 80U)) << 52); memcpy(&v, &b, sizeof v);\n\
    n++; if (arc_fx_from(v, f) != reference(v, f)) {{ bad++; if (bad < 5) printf(\"from %.17g %d: %d %d\\n\", v, f, arc_fx_from(v, f), reference(v, f)); }}\n\
    w = (int32_t)next(); if (arc_fx_to(w, f) != ldexp((double)w, -f)) {{ bad++; if (bad < 5) printf(\"to %d %d\\n\", w, f); }} }}\n\
  if (arc_fx_to(INT32_MIN, 0) != -2147483648.0 || arc_fx_to(0, 5) != 0.0) bad++;\n\
  printf(\"%ld of %ld differ\\n\", bad, n); return bad != 0; }}\n",
        fxc::CONVERSIONS
    );
    std::fs::write(dir.join("conversions.c"), harness).unwrap();
    let cc = std::env::var("CC").unwrap_or_else(|_| "cc".to_string());
    let built = Command::new(&cc).args(["-std=c99", "-O2", "-Wall", "-Wextra", "-Werror", "conversions.c", "-o", "conversions", "-lm"]).current_dir(&dir).output().unwrap();
    assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
    let ran = Command::new(dir.join("conversions")).output().unwrap();
    assert!(ran.status.success(), "{}", String::from_utf8_lossy(&ran.stdout));
    // And the driver itself calls no floating-point function.
    let model = compile_file(&example(CHARGER));
    let p = plan::plan(&model.ast).unwrap();
    let fx = fixed::machine(&p.machines[0], 10_000).unwrap();
    let driver = regmap::driver(&p, &p.machines[0], &fx, &regmap::map(&p, &p.machines[0], &fx));
    assert!(!driver.contains("ldexp") && !driver.contains("floor(") && !driver.contains("<math.h>"), "{driver}");
}

// ---- narrow words ---------------------------------------------------------------------

/// The charger with its voltages resolved to 1 mV and its current to 1 mA.
fn narrow_charger() -> String {
    std::fs::read_to_string(example(CHARGER))
        .unwrap()
        .replace("\"in voltage: Real [V] range 0 V .. 60 V\"", "\"in voltage: Real [V] range 0 V .. 60 V resolution 1 mV\"")
        .replace("\"in current: Real [A] range -25 A .. 25 A\"", "\"in current: Real [A] range -25 A .. 25 A resolution 1 mA\"")
        .replace("\"filtered: Real [V] range 0 V .. 60 V = 0 V\"", "\"filtered: Real [V] range 0 V .. 60 V resolution 1 mV = 0 V\"")
}

#[test]
fn a_resolution_gives_a_word_as_narrow_as_the_range_and_the_step_allow() {
    let model = compile_text(&narrow_charger()).unwrap();
    let p = plan::plan(&model.ast).unwrap();
    let fx = fixed::machine(&p.machines[0], 10_000).unwrap();
    let var = |name: &str| fx.vars[p.machines[0].program().variables.iter().position(|v| v.name == name).unwrap()].clone();
    // 60 V < 2^6, 1 mV > 2^-10: Q6.10 and a sign, 17 bits; 25 A < 2^5: 16 bits.
    assert_eq!((var("voltage").kind, var("voltage").width, var("voltage").label()), (fixed::Kind::Fixed(10), 17, "Q6.10 (17 bits)".to_string()));
    assert_eq!((var("current").kind, var("current").width), (fixed::Kind::Fixed(10), 16));
    assert_eq!(var("setpoint").width, 32, "without a resolution, 32 bits");
    assert_eq!(fixed::narrow(1 << 20, 17), (1 << 16) - 1, "a store saturates to the word");
    assert!(compile_text(&on_fpga(&["in v: Real range 0 .. 1000000 resolution 0.0000001", "out y: Real"], &["y = v"])).and_then(|m| {
        let p = plan::plan(&m.ast).unwrap();
        fixed::machine(&p.machines[0], 10_000)
    }).unwrap_err().contains("more than 32"));
    let fmt = compile_text(&on_fpga(&["in v: Real format Q6.9", "out y: Real range 0 .. 64 format Q6.9"], &["y = v"])).unwrap();
    let p = plan::plan(&fmt.ast).unwrap();
    assert_eq!(fixed::machine(&p.machines[0], 10_000).unwrap().vars[1].width, 16, "Q6.9: 16 bits");
}

#[test]
fn a_narrow_variable_is_held_in_its_width_in_the_vhdl_the_c_and_the_driver() {
    let model = compile_text(&narrow_charger()).unwrap();
    let p = plan::plan(&model.ast).unwrap();
    let m = &p.machines[0];
    let fx = fixed::machine(m, 10_000).unwrap();
    let core = vhdl::core(&p, m, &fx);
    assert!(core.contains("variable r_v_voltage : signed(16 downto 0);") && core.contains("variable r_v_filtered : signed(16 downto 0);"), "narrow storage");
    assert!(core.contains("r_v_filtered := resize(fx_narrow(") && core.contains("resize(r_v_filtered, 32)"), "saturated when stored, sign-extended when read");
    let fx_plan = fxc::plan(&p, &[0], 10_000).unwrap();
    let c = arclang::compiler::embedded::cgen::generate(&fx_plan, 10_000).source;
    assert!(c.contains("arc_fx_narrow(") && c.contains("static int32_t arc_fx_narrow(int32_t word, int bits)"), "the C reference saturates the same store");
    let driver = regmap::driver(&p, m, &fx, &regmap::map(&p, m, &fx));
    assert!(driver.contains("arc_fx_narrow(arc_fx_from((double)in->v_voltage, 10), 17)"), "the driver writes an input in its width");
}

#[test]
fn the_quantisation_bound_sees_a_fed_back_value_cancel() {
    // f = f + 0.2 * (v - f) carries 0.8 of f's error a step, not 1.2: the
    // bound stays near a few units of the format over many steps.
    let model = compile_file(&example(CHARGER));
    let p = plan::plan(&model.ast).unwrap();
    let fx = fixed::machine(&p.machines[0], 10_000).unwrap();
    let filtered = p.machines[0].program().variables.iter().position(|v| v.name == "filtered").unwrap();
    let ulp = 2f64.powi(-25);
    let bound = fixed::bounds(&fx, 500, &vec![0.0; fx.vars.len()])[filtered];
    assert!(bound < 16.0 * ulp, "{bound} for an ulp of {ulp}");
}

// ---- in the FPGA image ----------------------------------------------------------------

fn run_hdl(path: &str, synth: bool) -> (Vec<hdl::Outcome>, Vec<Result<hdl::Synthesis, String>>) {
    let model = compile_file(&example(path));
    let options = hdl::Options { work: work(&format!("hdl-{}", path.replace('/', "_"))), only: None, synth, compiler: "gcc".into(), as_if: None, machine: None };
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
    assert_eq!(outcomes.len(), 3);
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
#[ignore = "needs GHDL, Yosys and nextpnr-ecp5 of tools/embedded/fpga: run in its image (CI job Embedded FPGA)"]
fn narrow_words_stay_bit_exact_and_take_fewer_flip_flops() {
    let model = compile_text(&narrow_charger()).unwrap();
    let options = hdl::Options { work: work("narrow"), only: None, synth: false, compiler: "gcc".into(), as_if: None, machine: None };
    let (outcomes, _) = hdl::run(&model.ast, &options).unwrap();
    assert_eq!(outcomes.len(), 3);
    assert_passes(&outcomes);
    let p = plan::plan(&model.ast).unwrap();
    let a = &alloc::allocations(&model.ast, &p).unwrap()[0];
    let narrow = hdl::synthesise(&p, a, 10_000, &work("narrow-synth")).unwrap();
    let wide_model = compile_file(&example(CHARGER));
    let wp = plan::plan(&wide_model.ast).unwrap();
    let wide = hdl::synthesise(&wp, &alloc::allocations(&wide_model.ast, &wp).unwrap()[0], 10_000, &work("wide-synth")).unwrap();
    assert!(narrow.ffs < wide.ffs, "{} flip-flops narrow, {} at 32 bits", narrow.ffs, wide.ffs);
    assert!(narrow.timing_met, "Fmax {} MHz", narrow.fmax_mhz);
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
