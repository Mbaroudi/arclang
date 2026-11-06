//! Software-in-the-loop, back to back: the generated C compiled on the host
//! and run with the input vectors of each `execute:` case, its outputs
//! compared with the model-level run step by step, and the case's
//! `expects:` judged on both.
//!
//! A case without `binding:` drives the `step()` API directly (layer 1). A
//! case with `binding:` drives the host I/O board instead (the vectors
//! become pin levels, converter values, CAN frames and UART bytes), runs the
//! generated glue's cycle, and also compares what the glue wrote to the
//! bound output channels (layers 1 and 2).

use super::binding::{Binding, Edge, Origin, Sink, Source, Target};
use super::case::Case;
use super::cgen::{self, period_macro};
use super::plan::Plan;
use super::reference::{self, Table, Vectors};
use crate::compiler::ast::{Model, TestCase};
use crate::compiler::execution::expr::{Ty, Value};
use crate::compiler::simulation::expects::{self, Context, Evaluated};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The flags every generated file must compile with.
pub const STRICT_FLAGS: [&str; 5] = ["-std=c99", "-Wall", "-Wextra", "-Werror", "-pedantic"];

pub const IO_HEADER: &str = include_str!("runtime/arclang_io.h");
pub const HOST_HEADER: &str = include_str!("runtime/arclang_io_host.h");
pub const HOST_SOURCE: &str = include_str!("runtime/arclang_io_host.c");

/// What the host board holds (see `arclang_io_host.h`).
const HOST_PINS: u16 = 256;
const HOST_ANALOG: u16 = 64;
const HOST_CAN_IDS: usize = 64;
const HOST_UARTS: u16 = 8;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BackToBack {
    pub steps: usize,
    pub columns: usize,
    pub max_difference: f64,
    pub tolerance: f64,
    /// The first value out of tolerance: time, column, model, C.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_mismatch: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Judged {
    pub expect: String,
    pub model: Option<Evaluated>,
    pub c: Option<Evaluated>,
    /// Why an expectation could not be judged.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub error: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Outcome {
    pub test_case: String,
    pub name: String,
    pub verifies: Vec<String>,
    pub machine: String,
    pub compiler: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding: Option<String>,
    /// `pass`, `fail` or `error`.
    pub verdict: String,
    /// The verdict of the expectations on the model-level run.
    pub model_verdict: String,
    /// The verdict of the expectations on the C run.
    pub c_verdict: String,
    /// Whether the C run reproduces the model-level run: every value within
    /// tolerance and every expectation judged alike.
    pub agrees: bool,
    pub reason: String,
    pub expectations: Vec<Judged>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub back_to_back: Option<BackToBack>,
    /// Processor in the loop: what the target's run says of time (the
    /// period kept through the I/O, the cost of a step).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<String>,
    /// The behavior's code on the target (`arclang pil`): text, data and bss bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footprint: Option<super::pil::Footprint>,
}

#[derive(Debug, Clone)]
pub struct Options {
    /// The C compiler command (`gcc`, `clang`, `gcc-15`).
    pub compiler: String,
    /// Where the generated files and the binaries go.
    pub work: PathBuf,
    /// Only this case (id or name).
    pub only: Option<String>,
}

/// Bumped when the meaning of a cached SIL verdict changes.
const CACHE_FORMAT: &str = "arclang-sil-cache-1";

/// The hash of what the SIL verdict of a case depends on: every generated
/// file, the case's `execute:` and its expectations. Any change of the model
/// that changes the C, or of the generator, makes the cached verdict stale.
pub fn cache_key(ast: &Model, test: &TestCase) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let (files, _) = super::export(ast)?;
    let mut hash = Sha256::new();
    let mut field = |label: &str, bytes: &[u8]| {
        hash.update(label.as_bytes());
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    };
    field("format", CACHE_FORMAT.as_bytes());
    for (name, text) in &files {
        field("file", name.as_bytes());
        field("text", text.as_bytes());
    }
    if let Some(crate::compiler::ast::AttributeValue::Map(map)) = test.attributes.get("execute") {
        let mut keys: Vec<&String> = map.keys().collect();
        keys.sort();
        for key in keys {
            field("execute", format!("{key}={}", map[key].display()).as_bytes());
        }
    }
    for expect in crate::compiler::simulation::expects_of(&test.attributes) {
        field("expects", expect.as_bytes());
    }
    Ok(hash.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// The verdict of a case over the compilers it ran with, as the
/// simulation cache keeps verdicts: an error with one is an error, a
/// failure with one a failure.
pub fn cached_outcome(runs: &[&Outcome], key: String) -> crate::compiler::simulation::Outcome {
    let first = runs[0];
    let worst = runs.iter().find(|o| o.verdict == "error").or_else(|| runs.iter().find(|o| o.verdict == "fail")).copied().unwrap_or(first);
    let values: Vec<Evaluated> = worst.expectations.iter().filter_map(|j| j.c.clone().or_else(|| j.model.clone())).collect();
    let log = runs
        .iter()
        .map(|o| {
            let b2b = o.back_to_back.as_ref().map(|b| format!("{} steps x {} outputs, largest difference {}", b.steps, b.columns, b.max_difference)).unwrap_or_default();
            format!("{}: {} (model {}, C {}) {b2b} {}", o.compiler, o.verdict, o.model_verdict, o.c_verdict, o.reason).trim().to_string()
        })
        .collect::<Vec<_>>()
        .join("\n");
    let reason = if worst.verdict == "pass" { String::new() } else { format!("SIL with {}: {}", worst.compiler, worst.reason) };
    crate::compiler::simulation::Outcome {
        test_case: first.test_case.clone(),
        name: first.name.clone(),
        verifies: first.verifies.clone(),
        verdict: worst.verdict.clone(),
        reason,
        values,
        key,
        model: format!("state_machine {}", first.machine),
        tool: format!("arclang sil ({})", runs.iter().map(|o| o.compiler.as_str()).collect::<Vec<_>>().join(", ")),
        kind: "sil".to_string(),
        log,
        ..Default::default()
    }
}

/// Keep the verdicts of the cases run in the model's cache
/// (`.arclang/sim-cache/`), where `check`, `gate` and the tables read them.
pub fn cache(ast: &Model, model_path: &Path, outcomes: &[Outcome]) -> Result<Vec<PathBuf>, String> {
    let dir = crate::compiler::simulation::cache::dir(model_path);
    let mut written = Vec::new();
    for test in cases(ast) {
        let runs: Vec<&Outcome> = outcomes.iter().filter(|o| o.test_case == test.id).collect();
        if runs.is_empty() {
            continue;
        }
        let outcome = cached_outcome(&runs, cache_key(ast, test)?);
        written.push(crate::compiler::simulation::cache::write(&dir, &outcome)?);
    }
    Ok(written)
}

/// The cases of a model that execute at fixed step.
pub fn cases(ast: &Model) -> Vec<&TestCase> {
    ast.test_cases.iter().filter(|t| super::case::is_executed(t)).collect()
}

/// Every generated file of the model (behavior, `arclang_io.h`, glue of each
/// binding), as (file name, text).
pub fn files(plan: &Plan, bindings: &[Binding], default_period_us: u64) -> Vec<(String, String)> {
    let generated = cgen::generate(plan, default_period_us);
    let mut out = vec![(generated.header_name, generated.header), (generated.source_name, generated.source), ("arclang_io.h".to_string(), IO_HEADER.to_string())];
    for b in bindings {
        let glue = super::glue::generate(plan, b);
        out.push((glue.header_name, glue.header));
        out.push((glue.source_name, glue.source));
    }
    out
}

/// The first line of `<compiler> --version`.
pub fn compiler_version(compiler: &str) -> Option<String> {
    let out = Command::new(compiler).arg("--version").output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).lines().next().unwrap_or("").trim().to_string())
}

/// Compile C files into a binary with the strict flags; the compiler's
/// messages when it fails.
pub fn compile(compiler: &str, sources: &[PathBuf], include: &Path, defines: &[String], output: &Path) -> Result<(), String> {
    let mut command = Command::new(compiler);
    command.args(STRICT_FLAGS).arg("-O1").arg("-I").arg(include);
    for d in defines {
        command.arg(format!("-D{d}"));
    }
    command.args(sources).arg("-o").arg(output);
    let out = command.output().map_err(|e| format!("cannot run the C compiler '{compiler}': {e}"))?;
    if !out.status.success() {
        return Err(format!("{compiler} rejected the generated C:\n{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)));
    }
    Ok(())
}

fn write(dir: &Path, name: &str, text: &str) -> Result<PathBuf, String> {
    let path = dir.join(name);
    std::fs::write(&path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(path)
}

fn print_outputs(plan: &Plan, out: &mut String) {
    let mut first = true;
    for m in &plan.machines {
        let fields = m.states.iter().map(|s| format!("(int)outputs.{}.{}", m.ident, s.ident)).chain(m.actions.iter().map(|a| format!("(int)outputs.{}.{}", m.ident, a.ident)));
        for field in fields {
            let _ = writeln!(out, "        printf(\"{}%d\", {field});", if first { "" } else { "," });
            first = false;
        }
        for (v, signal) in m.program().variables.iter().zip(&m.variables) {
            let comma = if first { "" } else { "," };
            match v.ty {
                Ty::Bool => {
                    let _ = writeln!(out, "        printf(\"{comma}%d\", (int)outputs.{}.{});", m.ident, signal.ident);
                }
                _ => {
                    let _ = writeln!(out, "        printf(\"{comma}%.17g\", (double)outputs.{}.{});", m.ident, signal.ident);
                }
            }
            first = false;
        }
        for b in &m.budgets {
            let _ = writeln!(out, "        printf(\"{}%d\", (int)outputs.{}.{});", if first { "" } else { "," }, m.ident, b.ident);
            first = false;
        }
    }
}

/// A value as the harness reads it back exactly (`%lf` reads Rust's shortest form).
fn value_text(v: Value) -> String {
    match v {
        Value::Bool(b) => (b as u8).to_string(),
        Value::Int(i) => i.to_string(),
        Value::Real(r) => format!("{r:?}"),
        Value::Float(f) => format!("{f:?}"),
    }
}

/// The harness that drives `step()` directly: per step, one 0/1 per input
/// of every machine on stdin; one CSV row of the outputs on stdout.
pub fn direct_harness(plan: &Plan) -> String {
    let p = &plan.ident;
    let mut out = format!("/* SIL harness generated by ArcLang: drives {p}_step() from stdin. */\n#include <stdio.h>\n#include <string.h>\n#include \"{p}.h\"\n\nint main(void)\n{{\n    {p}_state state;\n    {p}_inputs inputs;\n    {p}_outputs outputs;\n    unsigned long steps = 0UL;\n    unsigned long k;\n    int v = 0;\n    long l = 0L;\n    double d = 0.0;\n    memset(&inputs, 0, sizeof inputs);\n    memset(&outputs, 0, sizeof outputs);\n    {p}_init(&state);\n    if (scanf(\"%lu\", &steps) != 1) {{\n        return 2;\n    }}\n    for (k = 0UL; k < steps; k++) {{\n");
    // Per machine, as `direct_line` writes them: the events, the conditions, the `in` variables.
    for m in &plan.machines {
        if !m.events.is_empty() {
            // The events of the step: how many, then each, in arrival order.
            let t = format!("{}_{}", plan.ident, m.ident);
            let _ = writeln!(out, "        inputs.{i}.queued = 0U;\n        if (scanf(\"%d\", &v) != 1) {{\n            return 3;\n        }}\n        {{\n            int n = v;\n            int e;\n            for (e = 0; e < n; e++) {{\n                if (scanf(\"%d\", &v) != 1) {{\n                    return 3;\n                }}\n                if (!{t}_post(&inputs.{i}, (uint16_t)v)) {{\n                    printf(\"ERROR the event queue of {i} is full at step %lu\\n\", k);\n                    return 4;\n                }}\n            }}\n        }}", i = m.ident);
        }
        for field in m.conditions.iter().map(|c| &c.ident) {
            let _ = writeln!(out, "        if (scanf(\"%d\", &v) != 1) {{\n            return 3;\n        }}\n        inputs.{}.{field} = (v != 0);", m.ident);
        }
        for (i, v) in m.program().inputs() {
            let field = format!("inputs.{}.{}", m.ident, m.variables[i].ident);
            // In fixed point (the reference of the hardware), a Real is its word.
            let ty = if m.fixed.is_some() && v.ty != Ty::Bool { Ty::Int } else { v.ty };
            let read = match ty {
                Ty::Bool => format!("if (scanf(\"%d\", &v) != 1) {{\n            return 3;\n        }}\n        {field} = (v != 0);"),
                Ty::Int => format!("if (scanf(\"%ld\", &l) != 1) {{\n            return 3;\n        }}\n        {field} = (int32_t)l;"),
                Ty::Real => format!("if (scanf(\"%lf\", &d) != 1) {{\n            return 3;\n        }}\n        {field} = d;"),
                Ty::Float => format!("if (scanf(\"%lf\", &d) != 1) {{\n            return 3;\n        }}\n        {field} = (float)d;"),
            };
            let _ = writeln!(out, "        {read}");
        }
    }
    let _ = writeln!(out, "        {p}_step(&state, &inputs, &outputs);");
    print_outputs(plan, &mut out);
    let _ = writeln!(out, "        printf(\"\\n\");\n        fflush(stdout);\n    }}\n    (void)v;\n    (void)l;\n    (void)d;\n    return 0;\n}}");
    out
}

/// What the direct harness reads: the number of steps, then per step one
/// 0/1 per event and condition of every machine.
pub fn direct_stdin(plan: &Plan, vectors: &Vectors) -> String {
    let mut stdin = format!("{}\n", vectors.steps);
    for k in 0..vectors.steps {
        stdin.push_str(&direct_line(plan, vectors, k));
        stdin.push('\n');
    }
    stdin
}

/// One step's line of the direct harness.
pub fn direct_line(plan: &Plan, vectors: &Vectors, k: usize) -> String {
    let mut line = Vec::new();
    for (n, m) in plan.machines.iter().enumerate() {
        if !m.events.is_empty() {
            line.push(vectors.queue[n][k].len().to_string());
            line.extend(vectors.queue[n][k].iter().map(|e| e.to_string()));
        }
        line.extend((0..m.conditions.len()).map(|c| (vectors.conditions[n][c][k] as u8).to_string()));
        line.extend(m.program().inputs().map(|(i, _)| value_text(vectors.values[n][i][k])));
    }
    line.join(" ")
}

/// The columns the I/O run adds: what the glue wrote to each bound output channel.
pub fn io_columns(binding: &Binding) -> Vec<String> {
    let mut out: Vec<String> = binding
        .outputs
        .iter()
        .map(|o| match o.sink {
            Sink::Gpio(p) => format!("io.gpio_{p}"),
            Sink::Pwm(c, _) => format!("io.pwm_{c}"),
            Sink::Dac(c, _) => format!("io.dac_{c}"),
            Sink::Can(id) => format!("io.can_0x{id:X}"),
            Sink::Uart(p, b) => format!("io.uart_{p}_0x{b:02X}"),
        })
        .collect();
    if binding.watchdog {
        out.push("io.watchdog_kicks".to_string());
    }
    out
}

/// The harness that drives the host board: per step, channel operations on
/// stdin (`g pin level`, `a channel raw`, `c id byte0`, `u port byte`, then
/// `.`), one cycle of the glue, a CSV row of the outputs and the bound
/// output channels.
pub fn io_harness(plan: &Plan, binding: &Binding) -> String {
    let p = &plan.ident;
    let b = &binding.ident;
    let mut out = format!(
        "/* SIL harness generated by ArcLang: drives the host I/O board and {b}_cycle(). */\n#include <stdio.h>\n#include <string.h>\n#include \"arclang_io_host.h\"\n#include \"{b}_io.h\"\n\nint main(void)\n{{\n    {p}_state state;\n    {p}_inputs inputs;\n    {p}_outputs outputs;\n    {b}_glue glue;\n    unsigned long steps = 0UL;\n    unsigned long k;\n    memset(&inputs, 0, sizeof inputs);\n    memset(&outputs, 0, sizeof outputs);\n    {p}_init(&state);\n    {b}_glue_init(&glue);\n    arclang_io_host_reset();\n    if (arclang_io_init() != ARCLANG_IO_OK) {{\n        return 5;\n    }}\n    if (scanf(\"%lu\", &steps) != 1) {{\n        return 2;\n    }}\n    for (k = 0UL; k < steps; k++) {{\n        arclang_io_status status;\n        for (;;) {{\n            char op = '.';\n            unsigned long a = 0UL;\n            unsigned long v = 0UL;\n            if (scanf(\" %c\", &op) != 1) {{\n                return 3;\n            }}\n            if (op == '.') {{\n                break;\n            }}\n            if (scanf(\"%lu %lu\", &a, &v) != 2) {{\n                return 3;\n            }}\n            if (op == 'g') {{\n                status = arclang_io_host_set_gpio((uint16_t)a, v != 0UL);\n            }} else if (op == 'a') {{\n                status = arclang_io_host_set_adc((uint16_t)a, (uint16_t)v);\n            }} else if (op == 'c') {{\n                uint8_t data[8] = {{0U}};\n                data[0] = (uint8_t)v;\n                status = arclang_io_host_push_can((uint32_t)a, data, 1U);\n            }} else if (op == 'u') {{\n                status = arclang_io_host_push_uart((uint16_t)a, (uint8_t)v);\n            }} else {{\n                return 3;\n            }}\n            if (status != ARCLANG_IO_OK) {{\n                printf(\"ERROR the host board refused '%c %lu %lu' at step %lu (%d)\\n\", op, a, v, k, (int)status);\n                return 4;\n            }}\n        }}\n        status = {b}_cycle(&glue, &state, &inputs, &outputs);\n        if (status != ARCLANG_IO_OK) {{\n            printf(\"ERROR the cycle failed at step %lu (%d)\\n\", k, (int)status);\n            return 4;\n        }}\n"
    );
    print_outputs(plan, &mut out);
    for o in &binding.outputs {
        let expr = match o.sink {
            Sink::Gpio(pin) => format!("arclang_io_host_gpio_output({pin}U) ? 1 : 0"),
            Sink::Pwm(c, _) => format!("(int)arclang_io_host_pwm({c}U)"),
            Sink::Dac(c, _) => format!("(int)arclang_io_host_dac({c}U)"),
            Sink::Can(id) => format!("arclang_io_host_can_sent(0x{id:X}UL, frame, &length) ? (int)frame[0] : -1"),
            Sink::Uart(port, byte) => format!("(int)arclang_io_host_uart_count({port}U, 0x{byte:02X}U)"),
        };
        if matches!(o.sink, Sink::Can(_)) {
            let _ = writeln!(out, "        {{\n            uint8_t frame[8] = {{0U}};\n            uint8_t length = 0U;\n            printf(\",%d\", {expr});\n        }}");
        } else {
            let _ = writeln!(out, "        printf(\",%d\", {expr});");
        }
    }
    if binding.watchdog {
        let _ = writeln!(out, "        printf(\",%lu\", (unsigned long)arclang_io_host_watchdog_kicks());");
    }
    let mut ports: Vec<u16> = binding.outputs.iter().filter_map(|o| if let Sink::Uart(port, _) = o.sink { Some(port) } else { None }).collect();
    ports.dedup();
    for port in ports {
        let _ = writeln!(out, "        arclang_io_host_uart_clear({port}U);");
    }
    let _ = writeln!(out, "        printf(\"\\n\");\n        (void)arclang_io_clock_wait_period({}UL);\n    }}\n    return 0;\n}}", binding.period_us);
    out
}

/// What the glue must write, from the model-level outputs.
pub fn expected_io(plan: &Plan, binding: &Binding, table: &Table) -> Vec<Vec<f64>> {
    let m = &plan.machines[binding.machine];
    let column = |name: String| table.names.iter().position(|n| *n == name).expect("a column of the model");
    let sources: Vec<usize> = binding
        .outputs
        .iter()
        .map(|o| match o.origin {
            Origin::State(s) => column(format!("{}.{}", m.ident, m.states[s].member)),
            Origin::Action(a) => column(format!("{}.{}", m.ident, m.actions[a].member)),
        })
        .collect();
    table
        .rows
        .iter()
        .enumerate()
        .map(|(k, row)| {
            let mut values: Vec<f64> = binding
                .outputs
                .iter()
                .zip(&sources)
                .map(|(o, &c)| {
                    let v = row[c];
                    let on = v != 0.0;
                    match o.sink {
                        Sink::Gpio(_) | Sink::Uart(..) => on as u8 as f64,
                        Sink::Pwm(_, duty) => if on { duty as f64 } else { 0.0 },
                        Sink::Dac(_, raw) => if on { raw as f64 } else { 0.0 },
                        Sink::Can(_) => v,
                    }
                })
                .collect();
            if binding.watchdog {
                values.push((k + 1) as f64);
            }
            values
        })
        .collect()
}

fn first_step_where(values: &[bool], wanted: &[bool]) -> Option<usize> {
    values.iter().zip(wanted).position(|(a, b)| a != b)
}

/// The channel operations, step by step, that make the glue read the case's vectors.
pub fn channel_operations(case: &Case, plan: &Plan, binding: &Binding, vectors: &Vectors) -> Result<Vec<Vec<String>>, String> {
    let m = &plan.machines[binding.machine];
    let n = vectors.steps;
    let what = format!("io_binding '{}'", binding.name);
    for s in &case.stimuli {
        let target = match s.drive {
            super::case::Drive::Event(e) => Target::Event(e),
            super::case::Drive::Condition(c, _) => Target::Condition(c),
            super::case::Drive::Value(i, _) => {
                return Err(format!("the case sets variable '{}', which {what} binds to no channel: the run through the I/O cannot set it (run it without binding:)", m.program().variables[i].name));
            }
        };
        if binding.input_of(target).is_none() {
            let name = match target {
                Target::Event(e) => &m.events[e].text,
                Target::Condition(c) => &m.conditions[c].text,
            };
            return Err(format!("the case drives '{name}', which {what} binds to no channel: the run through the I/O cannot set it"));
        }
    }
    let vector = |target: Target| -> Vec<bool> {
        match target {
            Target::Event(e) => vectors.events[binding.machine][e].clone(),
            Target::Condition(c) => vectors.conditions[binding.machine][c].clone(),
        }
    };
    let mut ops: Vec<Vec<String>> = vec![Vec::new(); n];
    let mut pins: BTreeMap<u16, Vec<&super::binding::Input>> = BTreeMap::new();
    let mut analog: BTreeMap<u16, Vec<&super::binding::Input>> = BTreeMap::new();
    let mut frames: BTreeMap<u32, Vec<&super::binding::Input>> = BTreeMap::new();
    for input in &binding.inputs {
        match input.source {
            Source::GpioLevel(pin) | Source::GpioEdge(pin, _) => pins.entry(pin).or_default().push(input),
            Source::Adc { channel, .. } => analog.entry(channel).or_default().push(input),
            Source::Can(id) => frames.entry(id).or_default().push(input),
            Source::Uart { port, .. } => {
                if port >= HOST_UARTS {
                    return Err(format!("{what}: uart {port}: the host board has {HOST_UARTS} ports"));
                }
            }
        }
    }
    for (pin, inputs) in &pins {
        if *pin >= HOST_PINS {
            return Err(format!("{what}: gpio {pin}: the host board has {HOST_PINS} pins"));
        }
        // A level bound to the pin decides it; else the edges do: high from a
        // rising edge, low from a falling one, held between them when the pin
        // has both, back to idle (low for rising edges, high for falling) otherwise.
        let edges = |wanted: Edge| -> Option<Vec<bool>> {
            let mut any: Option<Vec<bool>> = None;
            for i in inputs.iter().filter(|i| i.source == Source::GpioEdge(*pin, wanted)) {
                let v = vector(i.target);
                any = Some(match any {
                    None => v,
                    Some(a) => a.iter().zip(&v).map(|(x, y)| *x || *y).collect::<Vec<bool>>(),
                });
            }
            any
        };
        let levels: Vec<bool> = if let Some(level) = inputs.iter().find(|i| matches!(i.source, Source::GpioLevel(_))) {
            vector(level.target)
        } else {
            let (rise, fall) = (edges(Edge::Rising), edges(Edge::Falling));
            let mut previous = false;
            (0..n)
                .map(|k| {
                    let level = if rise.as_ref().is_some_and(|r| r[k]) {
                        true
                    } else if fall.as_ref().is_some_and(|f| f[k]) {
                        false
                    } else {
                        match (&rise, &fall) {
                            (Some(_), Some(_)) => previous,
                            (Some(_), None) => false,
                            _ => true,
                        }
                    };
                    previous = level;
                    level
                })
                .collect()
        };
        for input in inputs {
            let read: Vec<bool> = match input.source {
                Source::GpioLevel(_) => levels.clone(),
                Source::GpioEdge(_, edge) => (0..n)
                    .map(|k| {
                        let before = if k == 0 { false } else { levels[k - 1] };
                        match edge {
                            Edge::Rising => levels[k] && !before,
                            Edge::Falling => !levels[k] && before,
                        }
                    })
                    .collect(),
                _ => unreachable!(),
            };
            if let Some(k) = first_step_where(&read, &vector(input.target)) {
                return Err(format!(
                    "the vectors cannot be produced on gpio {pin} ('{}'): at step {k} the level the case needs gives a different value (an edge needs a step at the other level between two events, a falling edge a high level before it)",
                    input.written
                ));
            }
        }
        for (k, level) in levels.iter().enumerate() {
            ops[k].push(format!("g {pin} {}", *level as u8));
        }
    }
    for (channel, inputs) in &analog {
        if *channel >= HOST_ANALOG {
            return Err(format!("{what}: adc {channel}: the host board has {HOST_ANALOG} channels"));
        }
        let mut candidates: Vec<u32> = vec![0, u16::MAX as u32];
        for input in inputs {
            if let Source::Adc { threshold, .. } = input.source {
                let t = threshold as u32;
                candidates.extend([t.saturating_sub(1), t, t + 1].into_iter().filter(|c| *c <= u16::MAX as u32));
            }
        }
        for k in 0..n {
            let raw = candidates.iter().copied().find(|&raw| {
                inputs.iter().all(|i| match i.source {
                    Source::Adc { above, threshold, .. } => (if above { raw > threshold as u32 } else { raw < threshold as u32 }) == vector(i.target)[k],
                    _ => true,
                })
            });
            match raw {
                Some(raw) => ops[k].push(format!("a {channel} {raw}")),
                None => return Err(format!("the vectors cannot be produced on adc {channel}: at step {k} no raw value gives every condition bound to it its value")),
            }
        }
    }
    if frames.len() > HOST_CAN_IDS {
        return Err(format!("{what}: the host board has {HOST_CAN_IDS} CAN mailboxes"));
    }
    for (id, inputs) in &frames {
        let latch: Option<Vec<bool>> = inputs.iter().find(|i| matches!(i.target, Target::Condition(_))).map(|i| vector(i.target));
        let events: Vec<Vec<bool>> = inputs.iter().filter(|i| matches!(i.target, Target::Event(_))).map(|i| vector(i.target)).collect();
        let mut sent = vec![false; n];
        let mut held = false;
        for k in 0..n {
            let byte = latch.as_ref().is_some_and(|l| l[k]);
            if events.iter().any(|e| e[k]) || byte != held {
                sent[k] = true;
                held = byte;
                ops[k].push(format!("c {id} {}", byte as u8));
            }
        }
        for input in inputs {
            let read: Vec<bool> = match input.target {
                Target::Event(_) => sent.clone(),
                Target::Condition(_) => {
                    let mut value = false;
                    (0..n).map(|k| {
                        if sent[k] {
                            value = latch.as_ref().is_some_and(|l| l[k]);
                        }
                        value
                    }).collect()
                }
            };
            if let Some(k) = first_step_where(&read, &vector(input.target)) {
                return Err(format!("the vectors cannot be produced on can 0x{id:X} ('{}'): at step {k} a frame the other inputs on this identifier need gives it a different value", input.written));
            }
        }
    }
    // The order the glue posts the events of a step in: gpio edges by pin,
    // then CAN frames by identifier, then UART bytes by port as they arrived.
    let rank = |input: &super::binding::Input| -> (u8, u32) {
        match input.source {
            Source::GpioEdge(pin, _) | Source::GpioLevel(pin) => (0, pin as u32),
            Source::Adc { channel, .. } => (1, channel as u32),
            Source::Can(id) => (2, id),
            Source::Uart { port, .. } => (3, port as u32),
        }
    };
    for (k, queue) in vectors.queue[binding.machine].iter().enumerate() {
        let mut glue_order: Vec<(u8, u32, usize, usize)> = Vec::new();
        for (arrival, &e) in queue.iter().enumerate() {
            let input = binding.input_of(Target::Event(e)).expect("checked bound");
            let (kind, channel) = rank(input);
            if kind != 3 && queue.iter().filter(|&&x| x == e).count() > 1 {
                return Err(format!("at step {k} the case has '{}' arrive more than once; through the I/O ('{}') it occurs once per period", m.events[e].text, input.written));
            }
            // Within a pin or a frame the glue follows the binding; bytes follow their arrival.
            let within = if kind == 3 { arrival } else { binding.inputs.iter().position(|i| std::ptr::eq(i, input)).unwrap_or(0) };
            glue_order.push((kind, channel, within, e));
            if let Source::Uart { port, byte } = input.source {
                ops[k].push(format!("u {port} {byte}"));
            }
        }
        glue_order.sort();
        let through: Vec<usize> = glue_order.iter().map(|g| g.3).collect();
        if &through != queue {
            let names = |list: &[usize]| list.iter().map(|&e| format!("'{}'", m.events[e].text)).collect::<Vec<_>>().join(", ");
            return Err(format!(
                "at step {k} the case has {} arrive in this order; through the I/O they arrive {} (the glue reads gpio edges by pin, then CAN frames by identifier, then UART bytes as they came): the run through {what} cannot reproduce it",
                names(queue),
                names(&through)
            ));
        }
    }
    for (k, step) in ops.iter().enumerate() {
        let bytes = step.iter().filter(|o| o.starts_with('u')).count();
        if bytes as u32 > super::glue::UART_BURST {
            return Err(format!("at step {k} the case sends {bytes} UART bytes; the glue reads {} per period", super::glue::UART_BURST));
        }
    }
    Ok(ops)
}

/// Compare two tables of the same columns, step by step.
pub fn compare(model: &Table, c: &Table, tolerance: f64) -> BackToBack {
    let mut max_difference: f64 = 0.0;
    let mut first_mismatch = None;
    if model.names != c.names || model.rows.len() != c.rows.len() {
        first_mismatch = Some(format!("the C run gives {} steps of {} columns, the model {} of {}", c.rows.len(), c.names.len(), model.rows.len(), model.names.len()));
    }
    for (k, (a, b)) in model.rows.iter().zip(&c.rows).enumerate() {
        for (n, (x, y)) in a.iter().zip(b).enumerate() {
            let d = (x - y).abs();
            max_difference = max_difference.max(d);
            if d > tolerance && first_mismatch.is_none() {
                let t = k as f64 * model.period_us as f64 / 1e6;
                first_mismatch = Some(format!("at {t} s (step {k}), {}: model {x}, C {y}", model.names[n]));
            }
        }
    }
    BackToBack { steps: model.rows.len(), columns: model.names.len(), max_difference, tolerance, first_mismatch }
}

pub fn parse_rows(stdout: &str, names: &[String], period_us: u64) -> Result<Table, String> {
    let mut rows = Vec::new();
    for (k, line) in stdout.lines().enumerate() {
        if let Some(error) = line.strip_prefix("ERROR ") {
            return Err(error.to_string());
        }
        let row: Result<Vec<f64>, _> = line.split(',').map(|v| v.trim().parse::<f64>()).collect();
        let row = row.map_err(|_| format!("line {} of the C run is not numbers: '{line}'", k + 1))?;
        if row.len() != names.len() {
            return Err(format!("line {} of the C run has {} values for {} columns", k + 1, row.len(), names.len()));
        }
        rows.push(row);
    }
    Ok(Table { names: names.to_vec(), period_us, rows, units: BTreeMap::new() })
}

pub fn run_binary(binary: &Path, stdin: &str) -> Result<String, String> {
    use std::io::Write;
    let mut child = Command::new(binary).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e| format!("cannot run {}: {e}", binary.display()))?;
    child.stdin.take().expect("piped").write_all(stdin.as_bytes()).map_err(|e| format!("cannot feed the C run: {e}"))?;
    let out = child.wait_with_output().map_err(|e| format!("the C run failed: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    if !out.status.success() {
        let error = stdout.lines().find_map(|l| l.strip_prefix("ERROR ")).map(str::to_string);
        return Err(error.unwrap_or_else(|| format!("the C run exited with {} {}", out.status, String::from_utf8_lossy(&out.stderr))));
    }
    Ok(stdout)
}

/// Judge the expectations of a case on a run.
pub fn judge(texts: &[String], aliases: &BTreeMap<String, String>, table: &Table) -> Result<Vec<Evaluated>, String> {
    let series = table.series()?;
    texts
        .iter()
        .map(|text| {
            let parsed = expects::parse(text, aliases)?;
            let mut notes = Vec::new();
            expects::evaluate(&parsed, &mut Context { series: &series, notes: &mut notes })
        })
        .collect()
}

/// A built harness, by (period, binding).
struct Builds {
    binaries: HashMap<(u64, Option<String>), Result<PathBuf, String>>,
}

impl Builds {
    fn get(&mut self, plan: &Plan, bindings: &[Binding], period_us: u64, binding: Option<&Binding>, options: &Options) -> Result<PathBuf, String> {
        let key = (period_us, binding.map(|b| b.ident.clone()));
        if let Some(built) = self.binaries.get(&key) {
            return built.clone();
        }
        let built = build(plan, bindings, period_us, binding, options);
        self.binaries.insert(key, built.clone());
        built
    }
}

fn build(plan: &Plan, bindings: &[Binding], period_us: u64, binding: Option<&Binding>, options: &Options) -> Result<PathBuf, String> {
    let dir = options.work.join(format!("{}-{}us", binding.map(|b| b.ident.as_str()).unwrap_or("direct"), period_us));
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot make {}: {e}", dir.display()))?;
    let mut sources = Vec::new();
    for (name, text) in files(plan, bindings, period_us) {
        let path = write(&dir, &name, &text)?;
        let wanted = name == format!("{}.c", plan.ident) || binding.is_some_and(|b| name == format!("{}_io.c", b.ident));
        if wanted {
            sources.push(path);
        }
    }
    match binding {
        None => sources.push(write(&dir, "sil_main.c", &direct_harness(plan))?),
        Some(b) => {
            write(&dir, "arclang_io_host.h", HOST_HEADER)?;
            sources.push(write(&dir, "arclang_io_host.c", HOST_SOURCE)?);
            sources.push(write(&dir, "sil_main.c", &io_harness(plan, b))?);
        }
    }
    let binary = dir.join("sil");
    compile(&options.compiler, &sources, &dir, &[format!("{}={period_us}UL", period_macro(plan))], &binary)?;
    Ok(binary)
}

pub fn verdict_of(values: &[Option<Evaluated>]) -> String {
    if values.iter().any(Option::is_none) {
        "error".to_string()
    } else if values.iter().flatten().all(|v| v.holds) {
        "pass".to_string()
    } else {
        "fail".to_string()
    }
}

/// A case ready to run on an implementation: its outcome so far (the
/// model judged), the case, its vectors and binding, the model-level
/// table with the columns of the bound output channels.
pub struct Prepared<'a> {
    pub outcome: Outcome,
    pub case: Case,
    pub binding: Option<&'a Binding>,
    pub vectors: Vectors,
    pub model: Table,
    pub texts: Vec<String>,
    pub aliases: BTreeMap<String, String>,
}

/// Read a case, run it on the model and judge it there; the outcome in
/// error when that fails.
pub fn prepare<'a>(test: &TestCase, plan: &Plan, bindings: &'a [Binding], implementation: &str, ast: &Model, plants: &mut super::plant::Plants) -> Result<Prepared<'a>, Box<Outcome>> {
    let texts = crate::compiler::simulation::expects_of(&test.attributes);
    let mut outcome = Outcome {
        test_case: test.id.clone(),
        name: test.name.clone(),
        verifies: test.verifies.clone(),
        machine: String::new(),
        compiler: implementation.to_string(),
        binding: None,
        verdict: "error".to_string(),
        model_verdict: "error".to_string(),
        c_verdict: "error".to_string(),
        agrees: false,
        reason: String::new(),
        expectations: texts.iter().map(|t| Judged { expect: t.clone(), model: None, c: None, error: String::new() }).collect(),
        back_to_back: None,
        timing: None,
        footprint: None,
    };
    let case = match super::case::read(test, plan, bindings).expect("an executed case") {
        Ok(case) => case,
        Err(e) => {
            outcome.reason = e;
            return Err(Box::new(outcome));
        }
    };
    outcome.machine = plan.machines[case.machine].name.clone();
    outcome.binding = case.binding.clone();
    outcome.compiler.push_str(&super::plant::Plants::describe(&case));
    let binding = case.binding.as_ref().and_then(|r| bindings.iter().find(|b| b.is(r)));
    // With a plant, the model runs in the loop: its vectors are what the plant gave it.
    let (vectors, mut model) = if case.plant.is_some() {
        match super::plant::model_loop(ast, plan, &case, plants) {
            Ok(loop_run) => loop_run,
            Err(e) => {
                outcome.reason = e;
                return Err(Box::new(outcome));
            }
        }
    } else {
        let vectors = case.vectors(plan);
        let model = reference::run(plan, &vectors, case.step_us);
        (vectors, model)
    };
    let aliases = reference::aliases(plan);
    let model_values = match judge(&texts, &aliases, &model) {
        Ok(v) => v,
        Err(e) => {
            outcome.reason = e;
            return Err(Box::new(outcome));
        }
    };
    for (judged, value) in outcome.expectations.iter_mut().zip(&model_values) {
        judged.model = Some(value.clone());
    }
    outcome.model_verdict = verdict_of(&outcome.expectations.iter().map(|j| j.model.clone()).collect::<Vec<_>>());
    if let Some(b) = binding {
        let expected = expected_io(plan, b, &model);
        model.names.extend(io_columns(b));
        for (row, extra) in model.rows.iter_mut().zip(expected) {
            row.extend(extra);
        }
    }
    Ok(Prepared { outcome, case, binding, vectors, model, texts, aliases })
}

/// Judge the run of an implementation (`c`, the columns of the model's
/// table) against the model: the expectations on both, step by step, the
/// verdict and its reason.
pub fn conclude(prepared: Prepared<'_>, c: Table) -> Outcome {
    let Prepared { mut outcome, case, model, texts, aliases, .. } = prepared;
    let c = Table { units: model.units.clone(), ..c };
    let c_values = judge(&texts, &aliases, &c);
    let back_to_back = compare(&model, &c, case.tolerance);
    match c_values {
        Ok(c_values) => {
            for (judged, c) in outcome.expectations.iter_mut().zip(c_values) {
                judged.c = Some(c);
            }
        }
        Err(e) => outcome.reason = e,
    }
    let models: Vec<Option<Evaluated>> = outcome.expectations.iter().map(|j| j.model.clone()).collect();
    let cs: Vec<Option<Evaluated>> = outcome.expectations.iter().map(|j| j.c.clone()).collect();
    outcome.model_verdict = verdict_of(&models);
    outcome.c_verdict = verdict_of(&cs);
    let alike = outcome.expectations.iter().all(|j| j.model.as_ref().map(|v| v.holds) == j.c.as_ref().map(|v| v.holds));
    outcome.agrees = alike && back_to_back.first_mismatch.is_none() && outcome.c_verdict != "error";
    outcome.verdict = if outcome.c_verdict == "error" {
        "error".to_string()
    } else if !outcome.agrees {
        "fail".to_string()
    } else {
        outcome.c_verdict.clone()
    };
    if outcome.reason.is_empty() {
        outcome.reason = if let Some(mismatch) = &back_to_back.first_mismatch {
            format!("the C run departs from the model: {mismatch}")
        } else if !alike {
            "an expectation is judged differently on the C run and on the model".to_string()
        } else if outcome.verdict == "fail" {
            let failing: Vec<String> = outcome.expectations.iter().filter_map(|j| j.c.as_ref()).filter(|v| !v.holds).map(|v| format!("{} (expected {})", v.value, v.expect)).collect();
            format!("{}, on the C run as on the model", failing.join("; "))
        } else {
            String::new()
        };
    }
    outcome.back_to_back = Some(back_to_back);
    outcome
}

/// The stdin of the host harness through the I/O: per step, its channel operations and `.`.
pub fn io_stdin(steps: usize, ops: &[Vec<String>]) -> String {
    let mut stdin = format!("{steps}\n");
    for step in ops {
        for op in step {
            stdin.push_str(op);
            stdin.push('\n');
        }
        stdin.push_str(".\n");
    }
    stdin
}

/// Run one case: the model, the C, both judged, compared.
fn run_case(test: &TestCase, plan: &Plan, bindings: &[Binding], builds: &mut Builds, options: &Options, ast: &Model, plants: &mut super::plant::Plants) -> Outcome {
    let prepared = match prepare(test, plan, bindings, &options.compiler, ast, plants) {
        Ok(p) => p,
        Err(outcome) => return *outcome,
    };
    if prepared.case.plant.is_some() {
        return run_in_loop(prepared, plan, bindings, builds, options, ast, plants);
    }
    let stdin = match prepared.binding {
        None => direct_stdin(plan, &prepared.vectors),
        Some(b) => match channel_operations(&prepared.case, plan, b, &prepared.vectors) {
            Ok(ops) => io_stdin(prepared.case.steps, &ops),
            Err(e) => {
                let mut outcome = prepared.outcome;
                outcome.reason = e;
                return outcome;
            }
        },
    };
    let binary = match builds.get(plan, bindings, prepared.case.step_us, prepared.binding, options) {
        Ok(path) => path,
        Err(e) => {
            let mut outcome = prepared.outcome;
            outcome.reason = e;
            return outcome;
        }
    };
    match run_binary(&binary, &stdin).and_then(|stdout| parse_rows(&stdout, &prepared.model.names, prepared.case.step_us)) {
        Ok(table) => conclude(prepared, table),
        Err(e) => {
            let mut outcome = prepared.outcome;
            outcome.reason = e;
            outcome
        }
    }
}

/// The C in the loop with the plant: the harness fed a step at a time,
/// the plant advanced on what it answers; judged as a batch run is, the
/// model's own loop being the reference.
fn run_in_loop(prepared: Prepared<'_>, plan: &Plan, bindings: &[Binding], builds: &mut Builds, options: &Options, ast: &Model, plants: &mut super::plant::Plants) -> Outcome {
    let fail = |mut prepared: Prepared<'_>, reason: String| {
        prepared.outcome.reason = reason;
        prepared.outcome
    };
    let binary = match builds.get(plan, bindings, prepared.case.step_us, None, options) {
        Ok(path) => path,
        Err(e) => return fail(prepared, e),
    };
    let run = (|| -> Result<Table, String> {
        let plant = plants.get(ast, &prepared.case)?;
        let mut program = super::plant::Program::start(&binary, prepared.case.steps, prepared.model.names.clone(), prepared.case.step_us, Box::new(|v, k| direct_line(plan, v, k)))?;
        Ok(super::plant::run(plan, &prepared.case, plant, &mut program, None)?.table)
    })();
    match run {
        Ok(table) => conclude(prepared, table),
        Err(e) => fail(prepared, e),
    }
}

/// Run every executed case of a compiled model (or `options.only`).
pub fn run(ast: &Model, options: &Options) -> Result<Vec<Outcome>, String> {
    run_with(ast, options, &mut super::plant::Plants::none())
}

/// As `run`, the plants of the cases that have one built by `plants`.
pub fn run_with(ast: &Model, options: &Options, plants: &mut super::plant::Plants) -> Result<Vec<Outcome>, String> {
    let plan = super::plan::plan(ast)?;
    let bindings = super::binding::resolve_all(ast, &plan)?;
    let selected: Vec<&TestCase> = cases(ast).into_iter().filter(|t| options.only.as_deref().is_none_or(|o| t.id == o || t.name == o)).collect();
    if selected.is_empty() {
        return Err(match &options.only {
            Some(only) => format!("no test_case '{only}' that executes (execute: {{ ... }})"),
            None => "the model has no test_case that executes (execute: { machine: ... stop_time: ... inputs: [...] })".to_string(),
        });
    }
    std::fs::create_dir_all(&options.work).map_err(|e| format!("cannot make {}: {e}", options.work.display()))?;
    let mut builds = Builds { binaries: HashMap::new() };
    Ok(selected.into_iter().map(|t| run_case(t, &plan, &bindings, &mut builds, options, ast, plants)).collect())
}
