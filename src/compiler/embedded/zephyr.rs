//! `arclang export -f zephyr`: a Zephyr application of the model for a
//! target package of runtime `zephyr`. The behavior and the glue are the
//! same files as `-f c`; the package gives `arclang_io_zephyr.c`, the mains,
//! the build files, the board's devicetree overlay, its drivers and its
//! emulator; ArcLang generates what ties them to the model:
//!
//! - `src/arclang_app.h`: the model's and the binding's types and calls
//!   under fixed names (`arclang_state`, `arclang_cycle()`...), so the
//!   package's mains know no model;
//! - `src/arclang_io_config.h`: what `arclang_io_zephyr.c` sets up for the
//!   binding (the CAN identifiers it receives, its watchdog);
//! - `arclang_app.cmake`: the name of the period macro, the default main;
//! - `src/arclang_pil.h/.c`: the processor-in-the-loop report of the
//!   outputs and, for the step() API, the reading of the inputs, over the
//!   package's `arclang_pil_putc()` / `arclang_pil_getc()`. Values cross
//!   the link bit for bit (a `Real` as the 16 hexadecimal digits of its
//!   IEEE 754 binary64), so no conversion to text can differ.

use super::binding::{Binding, Sink, Source};
use super::plan::Plan;
use super::target::Package;
use super::Files;
use crate::compiler::ast::Model;
use crate::compiler::execution::expr::{Ty, Value};
use std::fmt::Write as _;

/// The events a step takes at most (`<MODEL>_<MACHINE>_QUEUE_SIZE`, by default).
const QUEUE_SIZE: usize = 16;

/// An exported application: its files (path, text, executable) and notes.
pub struct App {
    pub files: Vec<(String, String, bool)>,
    pub notes: Vec<String>,
    /// The binding the controller runs, if any.
    pub binding: Option<String>,
}

/// The target of an export: the one given, else the binding's when it is
/// not the host.
pub fn choose_target(bindings: &[Binding], binding: Option<&Binding>, target: Option<&str>) -> Result<Package, String> {
    let name = match (target, binding) {
        (Some(t), _) => t.to_string(),
        (None, Some(b)) if b.target != super::target::HOST => b.target.clone(),
        (None, _) => match bindings.iter().find(|b| b.target != super::target::HOST) {
            Some(b) => b.target.clone(),
            None => return Err("no target: give one (--target zephyr/litex_vexriscv) or name it in the io_binding (target: \"zephyr/litex_vexriscv\")".to_string()),
        },
    };
    let package = super::target::find(&name)?;
    if package.runtime != "zephyr" {
        return Err(format!("target '{name}' is of runtime '{}': -f zephyr exports for a target of runtime zephyr", package.runtime));
    }
    if let Some(b) = binding {
        super::target::check(b, &package)?;
    }
    Ok(package)
}

/// The binding an export runs: the one named, else the only one, else none
/// (an application for the step() API only).
pub fn choose_binding<'a>(bindings: &'a [Binding], name: Option<&str>) -> Result<Option<&'a Binding>, String> {
    match name {
        Some(n) => bindings.iter().find(|b| b.is(n)).map(Some).ok_or_else(|| format!("no io_binding '{n}' (io_bindings: {})", bindings.iter().map(|b| format!("'{}'", b.name)).collect::<Vec<_>>().join(", "))),
        None if bindings.len() <= 1 => Ok(bindings.first()),
        None => Err(format!("the model has {} io_bindings: name the one to run (--binding \"{}\")", bindings.len(), bindings[0].name)),
    }
}

/// The bindings an export runs as tasks: the one named, else every binding
/// of the target (one task each), else none.
pub fn choose_bindings<'a>(bindings: &'a [Binding], name: Option<&str>, target: Option<&str>) -> Result<Vec<&'a Binding>, String> {
    match name {
        Some(_) => Ok(choose_binding(bindings, name)?.into_iter().collect()),
        None => {
            let package = match target {
                Some(t) => t.to_string(),
                None => match bindings.iter().map(|b| b.target.as_str()).collect::<std::collections::BTreeSet<_>>().len() {
                    0 | 1 => bindings.first().map(|b| b.target.clone()).unwrap_or_default(),
                    _ => return Err(format!("the io_bindings are deployed on several targets ({}): name the target (--target) or the binding (--binding)", bindings.iter().map(|b| format!("'{}' on {}", b.name, b.target)).collect::<Vec<_>>().join(", "))),
                },
            };
            Ok(bindings.iter().filter(|b| b.target == package).collect())
        }
    }
}

/// The application of the model for `target` (else the bindings'): every
/// binding of the target as a task, or the one named.
pub fn export(ast: &Model, target: Option<&str>, binding: Option<&str>) -> Result<App, String> {
    let plan = super::plan::plan(ast)?;
    let bindings = super::binding::resolve_all(ast, &plan)?;
    let chosen = choose_bindings(&bindings, binding, target)?;
    let package = choose_target(&bindings, chosen.first().copied(), target)?;
    let allocations = super::alloc::allocations(ast, &plan)?;
    if allocations.is_empty() {
        return app_of(&plan, &chosen, &package);
    }
    let period = chosen.first().map(|b| b.period_us).unwrap_or(super::case::DEFAULT_STEP_US);
    let mut built = hw_app_of(&plan, &allocations, &chosen, &package, period)?;
    for (k, a) in allocations.iter().enumerate() {
        let block = super::vhdl::entity(&plan, &plan.machines[a.machine]);
        built.notes.push(format!("state_machine '{}' is computed by the programmable logic ('{}'): src/model/{block}_hw.h drives block {block}_wb at {up}_HW_BASE (0x{base:08X} by default: where the SoC maps its Wishbone region; -D{up}_HW_BASE=... for another), the VHDL in hw/; the emulated SoC has no such block, so arclang pil runs the machine's software C instead (arclang soc runs it with the block)", plan.machines[a.machine].name, a.node, up = block.to_uppercase(), base = hw_base(k)));
    }
    Ok(built)
}

/// Where the SoC maps the k-th block's Wishbone region.
pub fn hw_base(k: usize) -> u32 {
    0xB000_0000u32 + 0x1000 * k as u32
}

/// The application of a plan whose allocated machines the programmable
/// logic computes: those called through their drivers (the VHDL in `hw/`),
/// in fixed point at `period_us`.
pub fn hw_app(plan: &Plan, allocations: &[super::alloc::Allocation], binding: Option<&Binding>, package: &Package, period_us: u64) -> Result<App, String> {
    let bindings: Vec<&Binding> = binding.into_iter().collect();
    hw_app_of(plan, allocations, &bindings, package, period_us)
}

/// As `hw_app`, for several bindings (a task each).
pub fn hw_app_of(plan: &Plan, allocations: &[super::alloc::Allocation], bindings: &[&Binding], package: &Package, period_us: u64) -> Result<App, String> {
    let mut hw = plan.clone();
    let mut extra = Vec::new();
    let mut defines = Vec::new();
    for (k, a) in allocations.iter().enumerate() {
        let fx = super::fixed::machine(&plan.machines[a.machine], period_us)?;
        let block = super::vhdl::entity(plan, &plan.machines[a.machine]);
        for (name, text) in super::vhdl::files(plan, &plan.machines[a.machine], &fx) {
            let dir = if name.ends_with("_hw.h") { "src/model" } else { "hw" };
            extra.push((format!("{dir}/{name}"), text, false));
        }
        defines.push(format!("if(NOT DEFINED {up}_HW_BASE)\n  set({up}_HW_BASE 0x{base:08X})\nendif()\ntarget_compile_definitions(app PRIVATE {up}_HW_BASE=${{{up}_HW_BASE}})\n", up = block.to_uppercase(), base = hw_base(k)));
        hw.machines[a.machine].hardware = Some(block);
    }
    let mut built = app_of(&hw, bindings, package)?;
    for (name, text, _) in built.files.iter_mut() {
        if name == "arclang_app.cmake" {
            text.push_str("# The blocks of the programmable logic: the addresses of their registers.\n");
            for d in &defines {
                text.push_str(d);
            }
        }
    }
    built.files.extend(extra);
    Ok(built)
}

/// The application of a plan, one binding (or none), for a package.
pub fn app(plan: &Plan, binding: Option<&Binding>, package: &Package) -> Result<App, String> {
    let bindings: Vec<&Binding> = binding.into_iter().collect();
    app_of(plan, &bindings, package)
}

/// The application of a plan for a package: none of its bindings (the
/// step() API alone), one (the controller's loop, `main.c`), or several
/// (a task each at its period and priority, `tasks.c`).
pub fn app_of(plan: &Plan, bindings: &[&Binding], package: &Package) -> Result<App, String> {
    let binding = bindings.first().copied();
    let period = binding.map(|b| b.period_us).unwrap_or(super::case::DEFAULT_STEP_US);
    let mut files: Vec<(String, String, bool)> = Vec::new();
    // Several bindings: each machine at its binding's period (a task of its own);
    // one: the model's period is the binding's, as `export -f c` generates it.
    let periods: Vec<(usize, u64)> = if bindings.len() > 1 { bindings.iter().map(|b| (b.machine, b.period_us)).collect() } else { Vec::new() };
    let generated = super::cgen::generate_for(plan, period, &periods);
    files.push((format!("src/model/{}", generated.header_name), generated.header, false));
    files.push((format!("src/model/{}", generated.source_name), generated.source, false));
    files.push(("src/arclang_io.h".to_string(), super::sil::IO_HEADER.to_string(), false));
    for b in bindings {
        let glue = super::glue::generate(plan, b);
        files.push((format!("src/glue/{}", glue.header_name), glue.header, false));
        files.push((format!("src/glue/{}", glue.source_name), glue.source, false));
    }
    files.push(("src/arclang_app.h".to_string(), app_header(plan, bindings, period), false));
    files.push(("src/arclang_io_config.h".to_string(), io_config(bindings), false));
    files.push(("arclang_app.cmake".to_string(), app_cmake(plan, bindings, package), false));
    if bindings.len() > 1 {
        files.push(("src/tasks.c".to_string(), tasks_main(plan, bindings), false));
    }
    let (header, source) = pil_files(plan);
    files.push(("src/arclang_pil.h".to_string(), header, false));
    files.push(("src/arclang_pil.c".to_string(), source, false));
    files.extend(package.files()?);
    let mut notes = vec![format!("target {}: {}", package.name, package.summary)];
    match bindings {
        [] => notes.push("no io_binding: the application runs the step() API in processor in the loop only (ARCLANG_MAIN=pil_direct)".to_string()),
        [b] => notes.push(format!("the controller runs io_binding '{}' every {} us (src/main.c)", b.name, b.period_us)),
        many => {
            let owned: Vec<Binding> = many.iter().map(|b| (*b).clone()).collect();
            let priorities = super::binding::priorities(&owned);
            for (b, p) in many.iter().zip(priorities) {
                notes.push(format!("task '{}' runs state_machine '{}' every {} us at priority {p} (src/tasks.c)", b.name, plan.machines[b.machine].name, b.period_us));
            }
        }
    }
    for (kind, channel) in &package.channels {
        if let Some(note) = &channel.note {
            if bindings.iter().any(|b| uses(Some(b), kind)) {
                notes.push(format!("{kind}: {note}"));
            }
        }
    }
    Ok(App { files, notes, binding: binding.map(|b| b.name.clone()) })
}

/// `src/tasks.c`: one Zephyr thread per binding, each its glue, its clock
/// (`arclang_io_task_wait_period`) and its priority; the machines share
/// the model's state and I/O structures but each thread touches its own
/// machine's members only.
fn tasks_main(plan: &Plan, bindings: &[&Binding]) -> String {
    let owned: Vec<Binding> = bindings.iter().map(|b| (*b).clone()).collect();
    let priorities = super::binding::priorities(&owned);
    let mut out = banner("The controller as tasks: one thread per io_binding, at its period and priority.", plan);
    let _ = writeln!(out, "#include <zephyr/kernel.h>\n#include <zephyr/sys/printk.h>\n#include <string.h>\n\n#include \"arclang_app.h\"\n");
    let _ = writeln!(out, "static arclang_state state;\nstatic arclang_inputs inputs;\nstatic arclang_outputs outputs;\nstatic bool io_up;\n");
    for (k, (b, p)) in bindings.iter().zip(&priorities).enumerate() {
        let g = &b.ident;
        let _ = writeln!(out, "/* task {k}: io_binding \"{}\" on state_machine \"{}\", every {} us, priority {p} */\nstatic {g}_glue glue_{k};\n\nstatic void task_{k}(void *a, void *b, void *c)\n{{\n\t(void)a;\n\t(void)b;\n\t(void)c;\n\t{g}_glue_init(&glue_{k});\n\tfor (;;) {{\n\t\t(void)arclang_io_task_wait_period({k}U, {}UL);\n\t\tif (io_up) {{\n\t\t\t(void){g}_cycle(&glue_{k}, &state, &inputs, &outputs);\n\t\t}}\n\t}}\n}}\n", super::cgen::comment(&b.name), super::cgen::comment(&plan.machines[b.machine].name), b.period_us, b.period_us);
        // Zephyr: a lower number is a higher priority; the binding's 1 is the kernel's 0.
        let _ = writeln!(out, "K_THREAD_DEFINE(arclang_task_{k}, ARCLANG_TASK_STACK, task_{k}, NULL, NULL, NULL, {}, 0, 0);\n", p.saturating_sub(1));
    }
    let _ = writeln!(out, "int main(void)\n{{\n\t(void)memset(&inputs, 0, sizeof inputs);\n\t(void)memset(&outputs, 0, sizeof outputs);\n\tarclang_model_init(&state);\n\tif (arclang_io_init() != ARCLANG_IO_OK) {{\n\t\tprintk(\"arclang: the I/O did not come up; no cycle runs\\n\");\n\t\treturn 0;\n\t}}\n\tio_up = true;\n\treturn 0;\n}}");
    out
}

fn uses(binding: Option<&Binding>, kind: &str) -> bool {
    let Some(b) = binding else { return false };
    b.inputs.iter().any(|i| {
        kind == match i.source {
            Source::GpioLevel(_) | Source::GpioEdge(..) => "gpio_in",
            Source::Adc { .. } => "adc",
            Source::Can(_) => "can_rx",
            Source::Uart { .. } => "uart_rx",
        }
    }) || b.outputs.iter().any(|o| {
        kind == match o.sink {
            Sink::Gpio(_) => "gpio_out",
            Sink::Pwm(..) => "pwm",
            Sink::Dac(..) => "dac",
            Sink::Can(_) => "can_tx",
            Sink::Uart(..) => "uart_tx",
        }
    }) || (kind == "watchdog" && b.watchdog)
}

fn banner(what: &str, plan: &Plan) -> String {
    format!("/*\n * {what}\n * Generated by ArcLang from model \"{}\". Do not edit: regenerate.\n */\n", super::cgen::comment(&plan.model))
}

fn app_header(plan: &Plan, bindings: &[&Binding], period: u64) -> String {
    let p = &plan.ident;
    let mut out = banner("The model and the binding under the names the target's mains use.", plan);
    let _ = writeln!(out, "#ifndef ARCLANG_APP_H\n#define ARCLANG_APP_H\n\n#include \"arclang_io.h\"\n#include \"{p}.h\"\n");
    let _ = writeln!(out, "typedef {p}_state arclang_state;\ntypedef {p}_inputs arclang_inputs;\ntypedef {p}_outputs arclang_outputs;\n#define arclang_model_init(state) {p}_init(state)\n#define arclang_model_step(state, inputs, outputs) {p}_step((state), (inputs), (outputs))\n");
    match bindings {
        [b] => {
            let g = &b.ident;
            let _ = writeln!(out, "#define ARCLANG_HAS_BINDING 1\n#define ARCLANG_TASKS 1\n#ifndef ARCLANG_DIRECT\n#include \"{g}_io.h\"\ntypedef {g}_glue arclang_glue;\n#define arclang_glue_init(glue) {g}_glue_init(glue)\n#define arclang_cycle(glue, state, inputs, outputs) {g}_cycle((glue), (state), (inputs), (outputs))\n#define ARCLANG_PERIOD_US {}_PERIOD_US\n#endif\n", g.to_uppercase());
        }
        [] => {
            let _ = writeln!(out, "#define ARCLANG_HAS_BINDING 0\n#define ARCLANG_TASKS 0\n#define ARCLANG_PERIOD_US {period}UL\n");
        }
        many => {
            // Several: the tasks of src/tasks.c; the first binding's period stays the model's.
            let _ = writeln!(out, "#define ARCLANG_HAS_BINDING 1\n#define ARCLANG_TASKS {}\n#define ARCLANG_TASK_STACK 2048\n#ifndef ARCLANG_DIRECT", many.len());
            for b in many {
                let _ = writeln!(out, "#include \"{}_io.h\"", b.ident);
            }
            let _ = writeln!(out, "#define ARCLANG_PERIOD_US {}_PERIOD_US\n#endif\n", many[0].ident.to_uppercase());
        }
    }
    let _ = writeln!(out, "#endif /* ARCLANG_APP_H */");
    out
}

fn io_config(bindings: &[&Binding]) -> String {
    let mut ids: Vec<u32> = bindings.iter().flat_map(|b| b.inputs.iter().filter_map(|i| if let Source::Can(id) = i.source { Some(id) } else { None })).collect();
    ids.sort();
    ids.dedup();
    let watchdog = bindings.iter().any(|b| b.watchdog);
    // Ten periods without a kick (the longest period of the bindings that kick it), at least 10 ms.
    let timeout_ms = bindings.iter().filter(|b| b.watchdog).map(|b| (b.period_us * 10).div_ceil(1000).max(10)).max().unwrap_or(0);
    let mut out = String::from("/* What arclang_io_zephyr.c sets up for the binding. Generated by ArcLang. */\n#ifndef ARCLANG_IO_CONFIG_H\n#define ARCLANG_IO_CONFIG_H\n\n");
    let _ = writeln!(out, "#ifdef ARCLANG_DIRECT\n#define ARCLANG_IO_WATCHDOG 0\n#define ARCLANG_IO_CAN_RX_COUNT 0\n#else");
    let _ = writeln!(out, "#define ARCLANG_IO_WATCHDOG {}\n#define ARCLANG_IO_WATCHDOG_TIMEOUT_MS {timeout_ms}U", watchdog as u8);
    let _ = writeln!(out, "#define ARCLANG_IO_CAN_RX_COUNT {}", ids.len());
    if !ids.is_empty() {
        let _ = writeln!(out, "#define ARCLANG_IO_CAN_RX_IDS {{{}}}", ids.iter().map(|id| format!("0x{id:X}UL")).collect::<Vec<_>>().join(", "));
    }
    let _ = writeln!(out, "#endif\n\n#endif /* ARCLANG_IO_CONFIG_H */");
    out
}

fn app_cmake(plan: &Plan, bindings: &[&Binding], package: &Package) -> String {
    let main = match bindings.len() {
        0 => "pil_direct",
        1 => "main",
        _ => "tasks",
    };
    format!(
        "# Generated by ArcLang for target {}: what CMakeLists.txt needs of the model.\nset(ARCLANG_PERIOD_MACRO {})\nif(NOT DEFINED ARCLANG_MAIN)\n  set(ARCLANG_MAIN {main} CACHE STRING \"main, tasks, pil_io or pil_direct\")\nendif()\n",
        package.name,
        super::cgen::period_macro(plan)
    )
}

/// The tokens of one step's inputs for `arclang_pil_read()`, per machine:
/// the number of events then their codes, the conditions (0/1), the `in`
/// variables (an Integer in decimal, a Real or a Float as `x` and the
/// hexadecimal digits of its bits).
pub fn direct_line(plan: &Plan, vectors: &super::reference::Vectors, k: usize) -> String {
    let mut tokens = Vec::new();
    for (n, m) in plan.machines.iter().enumerate() {
        if !m.events.is_empty() {
            tokens.push(vectors.queue[n][k].len().to_string());
            tokens.extend(vectors.queue[n][k].iter().map(|e| e.to_string()));
        }
        tokens.extend((0..m.conditions.len()).map(|c| (vectors.conditions[n][c][k] as u8).to_string()));
        tokens.extend(m.program().inputs().map(|(i, _)| match vectors.values[n][i][k] {
            Value::Bool(b) => (b as u8).to_string(),
            Value::Int(v) => v.to_string(),
            Value::Real(r) => format!("x{:016x}", r.to_bits()),
            Value::Float(f) => format!("x{:08x}", f.to_bits()),
        }));
    }
    tokens.join(" ")
}

/// A reported value back to a number: decimal, or `x` and the bits of a
/// Real (16 digits) or a Float (8).
pub fn decode(token: &str) -> Result<f64, String> {
    match token.strip_prefix('x') {
        Some(hex) if hex.len() == 16 => u64::from_str_radix(hex, 16).map(f64::from_bits).map_err(|_| format!("'{token}' is no binary64")),
        Some(hex) if hex.len() == 8 => u32::from_str_radix(hex, 16).map(|b| f32::from_bits(b) as f64).map_err(|_| format!("'{token}' is no binary32")),
        Some(_) => Err(format!("'{token}' is no value")),
        None => token.parse::<f64>().map_err(|_| format!("'{token}' is no value")),
    }
}

/// `arclang_pil.h` and `arclang_pil.c`.
pub fn pil_files(plan: &Plan) -> (String, String) {
    let mut header = banner("Processor in the loop: the report of the outputs, the reading of the inputs.", plan);
    let _ = writeln!(
        header,
        "#ifndef ARCLANG_PIL_H\n#define ARCLANG_PIL_H\n\n#include <stdbool.h>\n#include <stdint.h>\n#include \"arclang_app.h\"\n\n/* The link, given by the target (the console). getc blocks. */\nvoid arclang_pil_putc(char c);\nint arclang_pil_getc(void);\n\nvoid arclang_pil_puts(const char *text);\n\n/* `R <step>,<time>,<status>,<outputs...>` and a new line: the states and\n   actions (counts), the variables (Boolean, Integer in decimal; Real, Float\n   as x and the hexadecimal digits of their bits), the budgets. */\nvoid arclang_pil_report(uint32_t step, uint64_t time, int32_t status, const arclang_outputs *outputs);\n\n/* One line of inputs (arclang pil writes it); false when it is ill-formed. */\nbool arclang_pil_read(arclang_inputs *inputs);\n\n#endif /* ARCLANG_PIL_H */"
    );
    let p = &plan.ident;
    let mut tokens = 0usize;
    for m in &plan.machines {
        tokens += if m.events.is_empty() { 0 } else { 1 + QUEUE_SIZE } + m.conditions.len() + m.program().inputs().count();
    }
    let line = (tokens + 1) * 24 + 2;
    let mut source = banner("Processor in the loop: the report of the outputs, the reading of the inputs.", plan);
    let _ = writeln!(source, "#include <string.h>\n#include \"arclang_pil.h\"\n\n#define ARCLANG_PIL_LINE {line}U\n");
    let tys: Vec<Ty> = plan.machines.iter().flat_map(|m| m.program().variables.iter().map(|v| v.ty)).collect();
    let input_tys: Vec<Ty> = plan.machines.iter().flat_map(|m| m.program().inputs().map(|(_, v)| v.ty).collect::<Vec<_>>()).collect();
    let ints = plan.machines.iter().any(|m| !m.events.is_empty() || !m.conditions.is_empty()) || input_tys.iter().any(|t| matches!(t, Ty::Bool | Ty::Int));
    let bits = input_tys.iter().any(|t| matches!(t, Ty::Real | Ty::Float));
    source.push_str(PIL_OUTPUT);
    if tys.iter().any(|t| matches!(t, Ty::Real | Ty::Float)) {
        source.push_str(PIL_HEX);
    }
    if tys.contains(&Ty::Real) {
        source.push_str(PIL_REAL);
    }
    if tys.contains(&Ty::Float) {
        source.push_str(PIL_FLOAT);
    }
    source.push_str(PIL_INPUT);
    if ints || bits {
        source.push_str(PIL_TOKEN);
    }
    if ints {
        source.push_str(PIL_INT);
    }
    if bits {
        source.push_str(PIL_BITS);
    }
    let _ = writeln!(source, "void arclang_pil_report(uint32_t step, uint64_t time, int32_t status, const arclang_outputs *outputs)\n{{\n    arclang_pil_puts(\"R \");\n    put_u64(step);\n    arclang_pil_putc(',');\n    put_u64(time);\n    arclang_pil_putc(',');\n    put_i64(status);");
    for m in &plan.machines {
        for field in m.states.iter().map(|s| &s.ident).chain(m.actions.iter().map(|a| &a.ident)) {
            let _ = writeln!(source, "    arclang_pil_putc(',');\n    put_u64((uint64_t)outputs->{}.{field});", m.ident);
        }
        for (v, signal) in m.program().variables.iter().zip(&m.variables) {
            let put = match v.ty {
                Ty::Bool => format!("put_u64(outputs->{}.{} ? 1U : 0U)", m.ident, signal.ident),
                Ty::Int => format!("put_i64((int64_t)outputs->{}.{})", m.ident, signal.ident),
                Ty::Real => format!("put_real(outputs->{}.{})", m.ident, signal.ident),
                Ty::Float => format!("put_float(outputs->{}.{})", m.ident, signal.ident),
            };
            let _ = writeln!(source, "    arclang_pil_putc(',');\n    {put};");
        }
        for b in &m.budgets {
            let _ = writeln!(source, "    arclang_pil_putc(',');\n    put_u64(outputs->{}.{} ? 1U : 0U);", m.ident, b.ident);
        }
    }
    let _ = writeln!(source, "    arclang_pil_putc('\\n');\n}}\n");
    let _ = writeln!(source, "bool arclang_pil_read(arclang_inputs *inputs)\n{{\n    char line[ARCLANG_PIL_LINE];\n    const char *at = line;\n    int64_t v = 0;\n    uint64_t bits = 0U;\n    bool ok = read_line(line, sizeof line);\n    /* `?`: a bench that came after the boot asks again (a simulator, a board). */\n    while (ok && (line[0] == '?') && (line[1] == '\\0')) {{\n        arclang_pil_puts(\"ARCLANG READY\\n\");\n        ok = read_line(line, sizeof line);\n    }}");
    for m in &plan.machines {
        if !m.events.is_empty() {
            let t = format!("{p}_{}", m.ident);
            let _ = writeln!(
                source,
                "    inputs->{i}.queued = 0U;\n    ok = ok && read_int(&at, &v);\n    {{\n        int64_t n = v;\n        int64_t e;\n        for (e = 0; ok && (e < n); e++) {{\n            ok = read_int(&at, &v) && {t}_post(&inputs->{i}, (uint16_t)v);\n        }}\n    }}",
                i = m.ident
            );
        }
        for c in &m.conditions {
            let _ = writeln!(source, "    ok = ok && read_int(&at, &v);\n    inputs->{}.{} = (v != 0);", m.ident, c.ident);
        }
        for (i, v) in m.program().inputs() {
            let field = format!("inputs->{}.{}", m.ident, m.variables[i].ident);
            let read = match v.ty {
                Ty::Bool => format!("ok = ok && read_int(&at, &v);\n    {field} = (v != 0);"),
                Ty::Int => format!("ok = ok && read_int(&at, &v);\n    {field} = (int32_t)v;"),
                Ty::Real => format!("ok = ok && read_bits(&at, 16U, &bits);\n    {{\n        double d;\n        (void)memcpy(&d, &bits, sizeof d);\n        {field} = d;\n    }}"),
                Ty::Float => format!("ok = ok && read_bits(&at, 8U, &bits);\n    {{\n        uint32_t b32 = (uint32_t)bits;\n        float f;\n        (void)memcpy(&f, &b32, sizeof f);\n        {field} = f;\n    }}"),
            };
            let _ = writeln!(source, "    {read}");
        }
    }
    let _ = writeln!(source, "    (void)v;\n    (void)bits;\n    (void)at;\n    return ok;\n}}");
    (header, source)
}

const PIL_OUTPUT: &str = r#"void arclang_pil_puts(const char *text)
{
    while (*text != '\0') {
        arclang_pil_putc(*text);
        text++;
    }
}

static void put_u64(uint64_t value)
{
    char digits[20];
    int n = 0;
    do {
        digits[n] = (char)('0' + (int)(value % 10U));
        n++;
        value /= 10U;
    } while (value != 0U);
    while (n > 0) {
        n--;
        arclang_pil_putc(digits[n]);
    }
}

static void put_i64(int64_t value)
{
    if (value < 0) {
        arclang_pil_putc('-');
        put_u64((uint64_t)(-(value + 1)) + 1U);
    } else {
        put_u64((uint64_t)value);
    }
}

"#;

const PIL_HEX: &str = r#"static void put_hex(uint64_t value, unsigned digits)
{
    static const char hex[] = "0123456789abcdef";
    unsigned k;
    arclang_pil_putc('x');
    for (k = digits; k > 0U; k--) {
        arclang_pil_putc(hex[(value >> (4U * (k - 1U))) & 0xFU]);
    }
}

"#;

const PIL_REAL: &str = r#"static void put_real(double value)
{
    uint64_t bits;
    (void)memcpy(&bits, &value, sizeof bits);
    put_hex(bits, 16U);
}

"#;

const PIL_FLOAT: &str = r#"static void put_float(float value)
{
    uint32_t bits;
    (void)memcpy(&bits, &value, sizeof bits);
    put_hex(bits, 8U);
}

"#;

const PIL_INPUT: &str = r#"static bool read_line(char *line, unsigned size)
{
    unsigned n = 0U;
    for (;;) {
        int c = arclang_pil_getc();
        if (c == '\n') {
            break;
        }
        if ((c != '\r') && (n + 1U < size)) {
            line[n] = (char)c;
            n++;
        }
    }
    line[n] = '\0';
    return n + 1U < size;
}

"#;

const PIL_TOKEN: &str = r#"static const char *token(const char **at)
{
    while (**at == ' ') {
        (*at)++;
    }
    return (**at == '\0') ? (const char *)0 : *at;
}

"#;

const PIL_INT: &str = r#"static bool read_int(const char **at, int64_t *value)
{
    const char *t = token(at);
    bool negative = false;
    uint64_t magnitude = 0U;
    bool any = false;
    if (t == (const char *)0) {
        return false;
    }
    if (*t == '-') {
        negative = true;
        t++;
    }
    while ((*t >= '0') && (*t <= '9')) {
        magnitude = (magnitude * 10U) + (uint64_t)(*t - '0');
        any = true;
        t++;
    }
    *at = t;
    *value = negative ? -(int64_t)magnitude : (int64_t)magnitude;
    return any && ((*t == ' ') || (*t == '\0'));
}

"#;

const PIL_BITS: &str = r#"static bool read_bits(const char **at, unsigned digits, uint64_t *bits)
{
    const char *t = token(at);
    unsigned k;
    uint64_t value = 0U;
    if ((t == (const char *)0) || (*t != 'x')) {
        return false;
    }
    t++;
    for (k = 0U; k < digits; k++) {
        char c = t[k];
        unsigned d;
        if ((c >= '0') && (c <= '9')) {
            d = (unsigned)(c - '0');
        } else if ((c >= 'a') && (c <= 'f')) {
            d = (unsigned)(c - 'a') + 10U;
        } else {
            return false;
        }
        value = (value << 4U) | d;
    }
    *at = t + digits;
    *bits = value;
    return (**at == ' ') || (**at == '\0');
}

"#;

/// The files of an application, as (path, text).
pub fn key_files(app: &App) -> Files {
    app.files.iter().map(|(p, t, _)| (p.clone(), t.clone())).collect()
}
