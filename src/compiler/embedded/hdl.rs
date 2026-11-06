//! Hardware in simulation, back to back: each `execute:` case of a machine
//! allocated to programmable logic run three ways and compared step by step:
//!
//! 1. the model (`reference`, floating point), the case's expectations
//!    judged on it;
//! 2. the bit-exact fixed-point C reference (`fxc`), compiled and run on
//!    this computer;
//! 3. the VHDL (`vhdl`), simulated by GHDL: a generated VHDL testbench
//!    plays the case on the Wishbone wrapper (the inputs written to their
//!    registers, START, DONE awaited, the outputs read back), so the
//!    register map is exercised with the entity.
//!
//! The VHDL must give the C reference's words exactly (difference 0); the C
//! reference must stay within the quantisation tolerance of the model (for
//! each `Real` variable the bound `fixed::bounds` derives from the formats,
//! or `quantisation:` in the case; the states, actions, integers and
//! Booleans exactly); the expectations are judged on the VHDL run.
//!
//! With `synth`, each allocated machine's wrapper is synthesised for its
//! node's ECP5 (Yosys with the GHDL plugin, nextpnr-ecp5, ecppack), its
//! resources and its Fmax reported against the node's clock.

use super::alloc::Allocation;
use super::fixed::{self, Kind, MachineFixed};
use super::plan::Plan;
use super::reference::{self, Table, Vectors};
use super::regmap::{Map, Reg};
use super::sil::{self, BackToBack, Judged};
use crate::compiler::ast::{AttributeValue, Model, TestCase};
use serde::Serialize;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Bumped when the meaning of a cached HDL verdict changes.
const CACHE_FORMAT: &str = "arclang-hdl-cache-1";

#[derive(Debug, Clone)]
pub struct Options {
    pub work: PathBuf,
    pub only: Option<String>,
    pub synth: bool,
    /// Synthesise every machine (or `machine`) as if allocated to this part at this clock (Hz), allocated or not.
    pub as_if: Option<(String, f64)>,
    /// Only this machine (name or id) for the synthesis.
    pub machine: Option<String>,
    /// The C compiler of the fixed-point reference.
    pub compiler: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Quantisation {
    pub steps: usize,
    pub columns: usize,
    /// The largest |fixed - model| over the Real variables, and where.
    pub max_difference: f64,
    pub column: String,
    /// The tolerance of that column.
    pub tolerance: f64,
    /// `derived from the formats` or `set in the case`.
    pub origin: String,
    /// Each Real variable: its column, format, largest difference, tolerance.
    pub variables: Vec<(String, String, f64, f64)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_mismatch: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Outcome {
    pub test_case: String,
    pub name: String,
    pub verifies: Vec<String>,
    pub machine: String,
    pub block: String,
    pub tools: String,
    /// `pass`, `fail` or `error`.
    pub verdict: String,
    pub model_verdict: String,
    pub hdl_verdict: String,
    pub reason: String,
    pub expectations: Vec<Judged>,
    /// The VHDL against the fixed-point C, word for word.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bit_exact: Option<BackToBack>,
    /// The fixed-point C against the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantisation: Option<Quantisation>,
    /// The clock cycles a step took in the VHDL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cycles: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Synthesis {
    pub block: String,
    pub node: String,
    pub device: String,
    pub clock_mhz: f64,
    pub fmax_mhz: f64,
    pub timing_met: bool,
    /// LUT4 cells (nextpnr's TRELLIS_COMB), flip-flops, 18x18 multipliers, block RAMs.
    pub luts: u64,
    pub ffs: u64,
    pub dsps: u64,
    pub brams: u64,
    pub ios: u64,
    pub bitstream: String,
    pub notes: Vec<String>,
}

/// The first line a tool prints of its version.
pub(super) fn version(program: &str, arg: &str) -> String {
    Command::new(program).arg(arg).output().ok().map(|o| format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))).and_then(|t| t.lines().next().map(|l| l.trim().to_string())).filter(|l| !l.is_empty()).unwrap_or_else(|| format!("{program} (not found)"))
}

/// The tools of a run, as the outcome names them.
pub fn tools(compiler: &str, synth: bool) -> String {
    let mut parts = vec![version("ghdl", "--version"), sil::compiler_version(compiler).unwrap_or_else(|| compiler.to_string())];
    if synth {
        parts.push(version("yosys", "-V"));
        parts.push(version("nextpnr-ecp5", "--version").trim_matches('"').replace("\" -- Next Generation Place and Route", ""));
    }
    parts.join("; ")
}

pub(super) fn run_tool(program: &str, args: &[String], cwd: &Path, what: &str) -> Result<String, String> {
    let out = Command::new(program).args(args).current_dir(cwd).output().map_err(|e| format!("{what}: cannot run '{program}': {e} (run in ArcLang's FPGA image: tools/embedded/fpga/run.sh)"))?;
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    if !out.status.success() {
        let tail: Vec<&str> = text.lines().rev().take(30).collect();
        return Err(format!("{what} failed ({}):\n{}", out.status, tail.into_iter().rev().collect::<Vec<_>>().join("\n")));
    }
    Ok(text)
}

/// As `run_tool`, the tool's stdout alone (its notes on stderr go to `log` in `cwd`).
pub(super) fn run_tool_stdout(program: &str, args: &[String], cwd: &Path, what: &str, log: &str) -> Result<String, String> {
    let out = Command::new(program).args(args).current_dir(cwd).output().map_err(|e| format!("{what}: cannot run '{program}': {e} (run in ArcLang's FPGA image: tools/embedded/fpga/run.sh)"))?;
    let stderr = String::from_utf8_lossy(&out.stderr);
    let _ = std::fs::write(cwd.join(log), stderr.as_bytes());
    if !out.status.success() {
        let tail: Vec<&str> = stderr.lines().rev().take(30).collect();
        return Err(format!("{what} failed ({}):\n{}", out.status, tail.into_iter().rev().collect::<Vec<_>>().join("\n")));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

pub(super) fn write(dir: &Path, name: &str, text: &str) -> Result<PathBuf, String> {
    let path = dir.join(name);
    std::fs::write(&path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(path)
}

/// The input of a value of an `in` variable: its word, its integer, 0/1.
/// An input as the word the driver writes: quantised, saturated to its width.
fn input_word(var: &fixed::VarFx, v: crate::compiler::execution::expr::Value) -> i64 {
    match var.kind {
        Kind::Fixed(f) => var.narrow(fixed::quantize(v.number(), f)),
        _ => v.number() as i64,
    }
}

/// What the fixed-point harness reads (one machine, `machine` of the vectors).
fn fixed_stdin(plan: &Plan, machine: usize, fx: &MachineFixed, vectors: &Vectors) -> String {
    let mut stdin = format!("{}\n", vectors.steps);
    for k in 0..vectors.steps {
        stdin.push_str(&fixed_line(plan, machine, fx, vectors, k));
        stdin.push('\n');
    }
    stdin
}

/// One step's line of the fixed-point harness: the inputs as words.
pub(super) fn fixed_line(plan: &Plan, machine: usize, fx: &MachineFixed, vectors: &Vectors, k: usize) -> String {
    let m = &plan.machines[machine];
    let mut line = Vec::new();
    if !m.events.is_empty() {
        line.push(vectors.queue[machine][k].len().to_string());
        line.extend(vectors.queue[machine][k].iter().map(|e| e.to_string()));
    }
    line.extend((0..m.conditions.len()).map(|c| (vectors.conditions[machine][c][k] as u8).to_string()));
    line.extend(m.program().inputs().map(|(i, _)| input_word(&fx.vars[i], vectors.values[machine][i][k]).to_string()));
    line.join(" ")
}

/// The bus operations of a case: per step its inputs written, then `G`.
fn stimulus(plan: &Plan, machine: usize, fx: &MachineFixed, map: &Map, vectors: &Vectors) -> String {
    let m = &plan.machines[machine];
    let mut out = String::new();
    let index = |r: Reg| map.find(&r).expect("a register").index;
    for k in 0..vectors.steps {
        if !m.events.is_empty() {
            let queue = &vectors.queue[machine][k];
            let _ = writeln!(out, "W {} {:08X}", index(Reg::EvCount), queue.len().min(super::regmap::QUEUE));
            for (n, e) in queue.iter().take(super::regmap::QUEUE).enumerate() {
                let _ = writeln!(out, "W {} {:08X}", index(Reg::EvQueue(n)), e);
            }
        }
        for w in 0..super::regmap::words(m.conditions.len()) {
            let mut bits: u32 = 0;
            for c in (32 * w)..m.conditions.len().min(32 * w + 32) {
                if vectors.conditions[machine][c][k] {
                    bits |= 1 << (c - 32 * w);
                }
            }
            let _ = writeln!(out, "W {} {bits:08X}", index(Reg::Cond(w)));
        }
        for (i, _) in m.program().inputs() {
            let word = input_word(&fx.vars[i], vectors.values[machine][i][k]);
            let _ = writeln!(out, "W {} {:08X}", index(Reg::In(i)), word as i32 as u32);
        }
        out.push_str("G\n");
    }
    out
}

/// The registers read after each step, as the raw columns of the machine
/// (states, actions, variables, budgets), and the cycles of each step.
fn decode(plan: &Plan, machine: usize, fx: &MachineFixed, map: &Map, text: &str, steps: usize) -> Result<(Vec<Vec<f64>>, Vec<u64>), String> {
    let m = &plan.machines[machine];
    let outputs = map.outputs();
    let mut rows = Vec::new();
    let mut cycles = Vec::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        if let Some(e) = line.strip_prefix("error ") {
            return Err(format!("in the simulation: {e}"));
        }
        let words: Vec<u32> = line.split_whitespace().map(|w| u32::from_str_radix(w, 16).map_err(|_| format!("the testbench wrote '{w}', no hex word"))).collect::<Result<_, _>>()?;
        if words.len() != outputs.len() {
            return Err(format!("the testbench read {} registers for {}", words.len(), outputs.len()));
        }
        let value = |r: &Reg| words[outputs.iter().position(|o| &o.reg == r).expect("an output register")];
        let mut row = Vec::new();
        for k in 0..m.states.len() {
            row.push(((value(&Reg::State(k / 32)) >> (k % 32)) & 1) as f64);
        }
        for a in 0..m.actions.len() {
            row.push((value(&Reg::Action(a)) & 0xFF) as f64);
        }
        for (i, v) in fx.vars.iter().enumerate() {
            let w = value(&Reg::Var(i));
            row.push(if v.kind == Kind::Bool { (w & 1) as f64 } else { w as i32 as f64 });
        }
        for b in 0..m.budgets.len() {
            row.push(((value(&Reg::Budget(b / 32)) >> (b % 32)) & 1) as f64);
        }
        rows.push(row);
        cycles.push(value(&Reg::Cycles) as u64 & 0xFFFF);
    }
    if rows.len() != steps {
        return Err(format!("the simulation gave {} of {steps} steps", rows.len()));
    }
    Ok((rows, cycles))
}

/// The raw words of a run as values: each fixed-point variable's word scaled.
fn values(plan: &Plan, machine: usize, fx: &MachineFixed, raw: &Table) -> Table {
    let m = &plan.machines[machine];
    let base = m.states.len() + m.actions.len();
    let rows = raw
        .rows
        .iter()
        .map(|row| {
            row.iter()
                .enumerate()
                .map(|(c, x)| match c.checked_sub(base).and_then(|i| fx.vars.get(i)) {
                    Some(v) => match v.kind {
                        Kind::Fixed(f) => fixed::value_of(*x as i64, f),
                        _ => *x,
                    },
                    None => *x,
                })
                .collect()
        })
        .collect();
    Table { rows, ..raw.clone() }
}

/// How far the fixed point may be from the model, column by column.
fn tolerances(plan: &Plan, machine: usize, fx: &MachineFixed, vectors: &Vectors, steps: usize, set: Option<f64>) -> Vec<f64> {
    let m = &plan.machines[machine];
    let inputs: Vec<f64> = (0..fx.vars.len())
        .map(|i| match (fx.vars[i].kind, m.program().variables[i].direction) {
            (Kind::Fixed(f), crate::compiler::execution::expr::Direction::In) => (0..steps).map(|k| {
                let v = vectors.values[machine][i][k].number();
                (v - fixed::value_of(fixed::quantize(v, f), f)).abs()
            }).fold(0.0, f64::max),
            _ => 0.0,
        })
        .collect();
    let bounds = fixed::bounds(fx, steps, &inputs);
    let mut out = vec![0.0; m.states.len() + m.actions.len()];
    for (i, v) in fx.vars.iter().enumerate() {
        out.push(match v.kind {
            Kind::Fixed(_) => {
                let float = if m.program().variables[i].ty == crate::compiler::execution::expr::Ty::Float { 2f64.powi(-20) } else { 2f64.powi(-40) };
                // The model's own rounding: a little of the range, per step.
                let slack = v.range.0.abs().max(v.range.1.abs()).max(1.0) * float * steps as f64;
                set.unwrap_or(bounds[i] + slack)
            }
            _ => 0.0,
        });
    }
    out.extend(vec![0.0; m.budgets.len()]);
    out
}

fn quantisation(model: &Table, fixed: &Table, tolerance: &[f64], plan: &Plan, machine: usize, fx: &MachineFixed, origin: &str) -> Quantisation {
    let m = &plan.machines[machine];
    let base = m.states.len() + m.actions.len();
    let mut first_mismatch = None;
    let mut worst = (0.0, String::new(), 0.0);
    let mut per: Vec<(String, String, f64, f64)> = fx.vars.iter().enumerate().filter(|(_, v)| matches!(v.kind, Kind::Fixed(_))).map(|(i, v)| (model.names[base + i].clone(), v.label(), 0.0, tolerance[base + i])).collect();
    for (k, (a, b)) in model.rows.iter().zip(&fixed.rows).enumerate() {
        for (c, (x, y)) in a.iter().zip(b).enumerate() {
            let d = (x - y).abs();
            let is_real = c >= base && c - base < fx.vars.len() && matches!(fx.vars[c - base].kind, Kind::Fixed(_));
            if is_real {
                if let Some(p) = per.iter_mut().find(|p| p.0 == model.names[c]) {
                    p.2 = p.2.max(d);
                }
                if d > worst.0 || worst.1.is_empty() {
                    worst = (d.max(worst.0), model.names[c].clone(), tolerance[c]);
                }
            }
            if d > tolerance[c] && first_mismatch.is_none() {
                let t = k as f64 * model.period_us as f64 / 1e6;
                let what = if is_real { format!("beyond its tolerance {:e}", tolerance[c]) } else { "a discrete output: the quantisation changed a decision".to_string() };
                first_mismatch = Some(format!("at {t} s (step {k}), {}: model {x}, fixed point {y} ({what})", model.names[c]));
            }
        }
    }
    Quantisation { steps: model.rows.len(), columns: model.names.len(), max_difference: worst.0, column: worst.1, tolerance: worst.2, origin: origin.to_string(), variables: per, first_mismatch }
}

/// The columns of one machine of a table.
pub(super) fn machine_columns(table: &Table, ident: &str) -> Table {
    let keep: Vec<usize> = table.names.iter().enumerate().filter(|(_, n)| n.starts_with(&format!("{ident}."))).map(|(i, _)| i).collect();
    Table { names: keep.iter().map(|&i| table.names[i].clone()).collect(), period_us: table.period_us, rows: table.rows.iter().map(|r| keep.iter().map(|&i| r[i]).collect()).collect(), units: table.units.clone() }
}

fn quantisation_set(test: &TestCase) -> Result<Option<f64>, String> {
    match test.attributes.get("execute") {
        Some(AttributeValue::Map(map)) => match map.get("quantisation") {
            None => Ok(None),
            Some(v) => v.as_number().filter(|t| *t >= 0.0).map(Some).ok_or_else(|| format!("test_case '{}': execute: quantisation is a number of at least 0, got '{}'", test.name, v.display())),
        },
        _ => Ok(None),
    }
}

fn run_case(test: &TestCase, plan: &Plan, bindings: &[super::binding::Binding], options: &Options, tools: &str, ast: &Model, plants: &mut super::plant::Plants) -> Outcome {
    let mut outcome = new_outcome(test, tools);
    let texts = outcome.expectations.iter().map(|j| j.expect.clone()).collect::<Vec<_>>();
    if let Err(e) = run_case_inner(test, plan, bindings, options, &texts, &mut outcome, ast, plants) {
        outcome.verdict = "error".to_string();
        outcome.reason = e;
    }
    outcome
}

/// An outcome not yet run: every expectation unjudged.
pub(super) fn new_outcome(test: &TestCase, tools: &str) -> Outcome {
    let texts = crate::compiler::simulation::expects_of(&test.attributes);
    Outcome {
        test_case: test.id.clone(),
        name: test.name.clone(),
        verifies: test.verifies.clone(),
        machine: String::new(),
        block: String::new(),
        tools: tools.to_string(),
        verdict: "error".to_string(),
        model_verdict: "error".to_string(),
        hdl_verdict: "error".to_string(),
        reason: String::new(),
        expectations: texts.iter().map(|t| Judged { expect: t.clone(), model: None, c: None, error: String::new() }).collect(),
        bit_exact: None,
        quantisation: None,
        cycles: None,
    }
}

#[allow(clippy::too_many_arguments)]
fn run_case_inner(test: &TestCase, plan: &Plan, bindings: &[super::binding::Binding], options: &Options, texts: &[String], outcome: &mut Outcome, ast: &Model, plants: &mut super::plant::Plants) -> Result<(), String> {
    let dir = options.work.join(format!("case-{}", case_dir(test)));
    let r = reference(test, plan, bindings, &options.compiler, &dir, texts, outcome, ast, plants)?;
    let m = &plan.machines[r.machine];
    // The VHDL, its testbench, the case as bus operations.
    let map = super::regmap::map(plan, m, &r.fx);
    let ent = super::vhdl::entity(plan, m);
    let mut sources = Vec::new();
    for (name, text) in super::vhdl::files(plan, m, &r.fx) {
        write(&dir, &name, &text)?;
        if name.ends_with(".vhd") {
            sources.push(name.clone());
        }
    }
    let tb = format!("{ent}_tb.vhd");
    write(&dir, &tb, &super::vhdl::testbench(plan, m, &map))?;
    sources.push(tb);
    write(&dir, "stimulus.txt", &stimulus(plan, r.machine, &r.fx, &map, &r.vectors))?;
    let _ = std::fs::remove_file(dir.join("result.txt"));
    let mut analyse = vec!["-a".to_string(), "--std=08".to_string()];
    analyse.extend(sources.iter().cloned());
    run_tool("ghdl", &analyse, &dir, "ghdl -a (analysis)")?;
    run_tool("ghdl", &["-e".into(), "--std=08".into(), format!("{ent}_tb")], &dir, "ghdl -e (elaboration)")?;
    run_tool("ghdl", &["-r".into(), "--std=08".into(), format!("{ent}_tb"), "-gSTIMULUS=stimulus.txt".into(), "-gRESULT=result.txt".into(), "--ieee-asserts=disable".into()], &dir, "ghdl -r (simulation)")?;
    let result = std::fs::read_to_string(dir.join("result.txt")).map_err(|e| format!("the simulation wrote no result: {e}"))?;
    let (rows, cycles) = decode(plan, r.machine, &r.fx, &map, &result, r.case.steps)?;
    let hdl_raw = Table { names: r.model.names.clone(), period_us: r.case.step_us, rows, units: r.model.units.clone() };
    let (lo, hi) = (cycles.iter().min().copied().unwrap_or(0), cycles.iter().max().copied().unwrap_or(0));
    outcome.cycles = Some(if lo == hi { format!("{lo} clock cycles a step") } else { format!("{lo} to {hi} clock cycles a step (the events of the step: one a clock)") });
    conclude(&r, plan, &hdl_raw, "the VHDL", texts, outcome, false);
    Ok(())
}

/// A case's directory name.
pub(super) fn case_dir(test: &TestCase) -> String {
    test.id.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect()
}

/// What a run of a case on the hardware is judged against: the model and
/// the bit-exact fixed-point C reference, compiled and run on this computer.
pub(super) struct Reference {
    pub case: super::case::Case,
    pub machine: usize,
    pub vectors: Vectors,
    /// The model, the machine's columns.
    pub model: Table,
    pub aliases: std::collections::BTreeMap<String, String>,
    pub fx: MachineFixed,
    /// The fixed-point C's words, the machine's columns, on `vectors`.
    pub fixed_raw: Table,
    /// With a plant: the fixed-point C's words in its own loop with it —
    /// what a device in the loop, bit-exact with the C, reproduces.
    pub fixed_loop: Option<Table>,
    pub tolerance: Vec<f64>,
    pub origin: &'static str,
}

/// The model run (its expectations judged into `outcome`) and the
/// fixed-point C reference of a case, built and run in `dir`.
#[allow(clippy::too_many_arguments)]
pub(super) fn reference(test: &TestCase, plan: &Plan, bindings: &[super::binding::Binding], compiler: &str, dir: &Path, texts: &[String], outcome: &mut Outcome, ast: &Model, plants: &mut super::plant::Plants) -> Result<Reference, String> {
    let case = super::case::read(test, plan, bindings).expect("an executed case")?;
    let machine = case.machine;
    let m = &plan.machines[machine];
    outcome.machine = m.name.clone();
    outcome.block = super::vhdl::entity(plan, m);
    outcome.tools.push_str(&super::plant::Plants::describe(&case));
    let set = quantisation_set(test)?;
    // With a plant, the model runs in the loop: its vectors are what the plant gave it.
    let (vectors, model) = if case.plant.is_some() {
        let (vectors, model) = super::plant::model_loop(ast, plan, &case, plants)?;
        (vectors, machine_columns(&model, &m.ident))
    } else {
        let vectors = case.vectors(plan);
        let model = machine_columns(&reference::run(plan, &vectors, case.step_us), &m.ident);
        (vectors, model)
    };
    // The block alone: its wired inputs replay what the model's run gave them.
    let vectors = reference::wired(plan, &vectors, case.step_us);
    let aliases = reference::aliases(plan);
    let model_values = sil::judge(texts, &aliases, &model)?;
    for (j, v) in outcome.expectations.iter_mut().zip(&model_values) {
        j.model = Some(v.clone());
    }
    outcome.model_verdict = sil::verdict_of(&outcome.expectations.iter().map(|j| j.model.clone()).collect::<Vec<_>>());
    // The fixed-point reference at the case's period.
    let fx_plan = super::fxc::plan(plan, &[machine], case.step_us)?;
    let fx = fx_plan.machines[0].fixed.as_deref().expect("fixed").clone();
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot make {}: {e}", dir.display()))?;
    let generated = super::cgen::generate(&fx_plan, case.step_us);
    write(dir, &generated.header_name, &generated.header)?;
    let source = write(dir, &generated.source_name, &generated.source)?;
    let harness = write(dir, "fx_main.c", &sil::direct_harness(&fx_plan))?;
    let binary = dir.join("fx_reference");
    sil::compile(compiler, &[source, harness], dir, &[format!("{}={}UL", super::cgen::period_macro(&fx_plan), case.step_us)], &binary)?;
    // The fixed-point C on the model's vectors: the quantisation is measured
    // on the same inputs, step by step; the VHDL replays them.
    let stdout = sil::run_binary(&binary, &fixed_stdin(plan, machine, &fx, &vectors))?;
    let fixed_raw = sil::parse_rows(&stdout, &model.names, case.step_us)?;
    // With a plant, the fixed-point C in its own loop with it too: a device
    // in the loop, bit-exact with the C, reproduces that loop, not the model's.
    let fixed_loop = match &case.plant {
        Some(_) => {
            let plant = plants.get(ast, &case)?;
            let fx_case = super::case::Case { machine: 0, ..case.clone() };
            let names = model.names.clone();
            let mut program = super::plant::Program::start(&binary, case.steps, names, case.step_us, Box::new(|v, k| fixed_line(&fx_plan, 0, &fx, v, k)))?;
            let to_value = |i: usize, x: f64| match fx.vars[i].kind {
                Kind::Fixed(f) => fixed::value_of(x as i64, f),
                _ => x,
            };
            Some(super::plant::run(&fx_plan, &fx_case, plant, &mut program, Some(&to_value))?.table)
        }
        None => None,
    };
    let tolerance = tolerances(plan, machine, &fx, &vectors, case.steps, set);
    let origin = if set.is_some() { "set in the case (quantisation:)" } else { "derived from the formats" };
    Ok(Reference { case, machine, vectors, model, aliases, fx, fixed_raw, fixed_loop, tolerance, origin })
}

/// A run on the hardware (`device`: "the VHDL", "the SoC"), its words in
/// `device_raw`, judged: word for word against the fixed-point C (the C
/// in the loop with the plant when the device ran `in_loop`), within the
/// tolerance of the model, its expectations as the model's.
pub(super) fn conclude(r: &Reference, plan: &Plan, device_raw: &Table, device: &str, texts: &[String], outcome: &mut Outcome, in_loop: bool) {
    let exact = if in_loop { r.fixed_loop.as_ref().unwrap_or(&r.fixed_raw) } else { &r.fixed_raw };
    let bit_exact = sil::compare(exact, device_raw, 0.0);
    let fixed_values = values(plan, r.machine, &r.fx, &r.fixed_raw);
    let device_values = values(plan, r.machine, &r.fx, device_raw);
    let q = quantisation(&r.model, &fixed_values, &r.tolerance, plan, r.machine, &r.fx, r.origin);
    match sil::judge(texts, &r.aliases, &Table { units: r.model.units.clone(), ..device_values }) {
        Ok(v) => {
            for (j, x) in outcome.expectations.iter_mut().zip(v) {
                j.c = Some(x);
            }
        }
        Err(e) => outcome.reason = e,
    }
    outcome.hdl_verdict = sil::verdict_of(&outcome.expectations.iter().map(|j| j.c.clone()).collect::<Vec<_>>());
    let alike = outcome.expectations.iter().all(|j| j.model.as_ref().map(|v| v.holds) == j.c.as_ref().map(|v| v.holds));
    outcome.verdict = if outcome.hdl_verdict == "error" {
        "error".to_string()
    } else if bit_exact.first_mismatch.is_some() || q.first_mismatch.is_some() || !alike {
        "fail".to_string()
    } else {
        outcome.hdl_verdict.clone()
    };
    if outcome.reason.is_empty() {
        outcome.reason = if let Some(mm) = &bit_exact.first_mismatch {
            format!("{device} departs from the fixed-point C: {mm}")
        } else if let Some(mm) = &q.first_mismatch {
            format!("the fixed point departs from the model: {mm}")
        } else if !alike {
            format!("an expectation is judged differently on {device} and on the model")
        } else if outcome.verdict == "fail" {
            let failing: Vec<String> = outcome.expectations.iter().filter_map(|j| j.c.as_ref()).filter(|v| !v.holds).map(|v| format!("{} (expected {})", v.value, v.expect)).collect();
            format!("{}, on {device} as on the model", failing.join("; "))
        } else {
            String::new()
        };
    }
    outcome.bit_exact = Some(bit_exact);
    outcome.quantisation = Some(q);
}

/// The executed cases of the allocated machines (or `only`).
pub(super) fn selected<'a>(ast: &'a Model, plan: &Plan, allocations: &[Allocation], bindings: &[super::binding::Binding], only: Option<&str>) -> Vec<&'a TestCase> {
    sil::cases(ast)
        .into_iter()
        .filter(|t| only.is_none_or(|o| t.id == o || t.name == o))
        .filter(|t| match super::case::read(t, plan, bindings) {
            Some(Ok(case)) => allocations.iter().any(|a| a.machine == case.machine),
            _ => true,
        })
        .collect()
}

/// The cases' outcomes and, with `--synth`, each block's synthesis.
pub type Report = (Vec<Outcome>, Vec<Result<Synthesis, String>>);

/// Run the cases of the machines allocated to programmable logic.
pub fn run(ast: &Model, options: &Options) -> Result<Report, String> {
    run_with(ast, options, &mut super::plant::Plants::none())
}

/// As `run`; a case with a plant replays, in the VHDL, the vectors of the
/// fixed-point C's loop around it (`plants` builds the plant).
pub fn run_with(ast: &Model, options: &Options, plants: &mut super::plant::Plants) -> Result<Report, String> {
    let plan = super::plan::plan(ast)?;
    let bindings = super::binding::resolve_all(ast, &plan)?;
    let allocations = super::alloc::allocations(ast, &plan)?;
    if allocations.is_empty() && options.as_if.is_none() {
        return Err("no state_machine is allocated to programmable logic: give it to a node of kind \"fpga\" (node \"Fabric\" { kind: \"fpga\" deploys \"<machine or its owner>\" }; docs/embedded-fpga.md)".to_string());
    }
    if let Some(wanted) = &options.machine {
        if !plan.machines.iter().any(|m| m.is(wanted)) {
            return Err(format!("no state_machine '{wanted}' in the model"));
        }
    }
    std::fs::create_dir_all(&options.work).map_err(|e| format!("cannot make {}: {e}", options.work.display()))?;
    let work = options.work.canonicalize().unwrap_or_else(|_| options.work.clone());
    let options = Options { work, ..options.clone() };
    // As if: the synthesis only; the cases stay those of the machines allocated.
    let cases = if options.as_if.is_some() { Vec::new() } else { selected(ast, &plan, &allocations, &bindings, options.only.as_deref()) };
    if cases.is_empty() && !options.synth {
        return Err(match &options.only {
            Some(o) => format!("no test_case '{o}' that executes a machine allocated to programmable logic"),
            None => "no test_case executes a machine allocated to programmable logic (execute: { machine: ... })".to_string(),
        });
    }
    let tools = tools(&options.compiler, options.synth);
    let outcomes = cases.into_iter().map(|t| run_case(t, &plan, &bindings, &options, &tools, ast, plants)).collect();
    let mut synthesis = Vec::new();
    if options.synth {
        let targets: Vec<Allocation> = match &options.as_if {
            Some((part, clock_hz)) => plan.machines.iter().enumerate().map(|(i, _)| Allocation { machine: i, node: format!("as if on {part}"), part: part.clone(), clock_hz: *clock_hz, board: None }).collect(),
            None => allocations.clone(),
        };
        for a in targets.iter().filter(|a| options.machine.as_deref().is_none_or(|w| plan.machines[a.machine].is(w))) {
            let period = crate::compiler::partition::period_of(&bindings, a.machine);
            synthesis.push(synthesise(&plan, a, period, &options.work));
        }
    }
    Ok((outcomes, synthesis))
}

fn number(v: &serde_json::Value, path: &[&str]) -> Option<f64> {
    let mut at = v;
    for p in path {
        at = at.get(p)?;
    }
    at.as_f64()
}

/// Synthesise a machine's wrapper for its node's ECP5 and place and route it.
pub fn synthesise(plan: &Plan, a: &Allocation, period_us: u64, work: &Path) -> Result<Synthesis, String> {
    let m = &plan.machines[a.machine];
    let fx = super::fixed::machine(m, period_us)?;
    let ent = super::vhdl::entity(plan, m);
    let top = format!("{ent}_wb");
    let dir = work.join(format!("synth-{ent}"));
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot make {}: {e}", dir.display()))?;
    let mut sources = Vec::new();
    for (name, text) in super::vhdl::files(plan, m, &fx) {
        write(&dir, &name, &text)?;
        if name.ends_with(".vhd") {
            sources.push(name);
        }
    }
    // GHDL's own synthesis first: the design elaborates as hardware, and
    // its netlist goes to Yosys as Verilog (no plugin to keep in step with
    // the Yosys binary).
    let mut analyse = vec!["-a".to_string(), "--std=08".to_string()];
    analyse.extend(sources.iter().cloned());
    run_tool("ghdl", &analyse, &dir, "ghdl -a")?;
    let netlist = run_tool_stdout("ghdl", &["--synth".into(), "--std=08".into(), "--out=verilog".into(), top.clone()], &dir, "ghdl --synth --out=verilog", "ghdl_synth.log")?;
    let verilog = format!("{ent}_synth.v");
    write(&dir, &verilog, &netlist)?;
    let script = format!("read_verilog {verilog}; synth_ecp5 -top {top} -json {ent}.json");
    run_tool("yosys", &["-q".into(), "-l".into(), "yosys.log".into(), "-p".into(), script], &dir, "yosys (synth_ecp5)")?;
    let device = a.device()?;
    let mhz = a.clock_hz / 1e6;
    let args: Vec<String> = vec![
        format!("--{}", device.size),
        "--package".into(),
        device.package.clone(),
        "--speed".into(),
        device.speed.to_string(),
        "--freq".into(),
        format!("{mhz}"),
        "--json".into(),
        format!("{ent}.json"),
        "--textcfg".into(),
        format!("{ent}.config"),
        "--report".into(),
        "report.json".into(),
        "--timing-allow-fail".into(),
        "--seed".into(),
        "1".into(),
        "--log".into(),
        "nextpnr.log".into(),
    ];
    run_tool("nextpnr-ecp5", &args, &dir, "nextpnr-ecp5 (place and route)")?;
    run_tool("ecppack", &[format!("{ent}.config"), format!("{ent}.bit")], &dir, "ecppack (bitstream)")?;
    let report: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("report.json")).map_err(|e| format!("nextpnr wrote no report: {e}"))?).map_err(|e| format!("the nextpnr report: {e}"))?;
    let used = |cell: &str| number(&report, &["utilization", cell, "used"]).unwrap_or(0.0) as u64;
    let fmax = report.get("fmax").and_then(|f| f.as_object()).and_then(|clocks| clocks.values().filter_map(|c| c.get("achieved").and_then(|x| x.as_f64())).reduce(f64::min)).ok_or("the nextpnr report has no Fmax")?;
    let bitstream = dir.join(format!("{ent}.bit"));
    Ok(Synthesis {
        block: top,
        node: a.node.clone(),
        device: format!("{} (nextpnr-ecp5 --{} --package {} --speed {})", device.part, device.size, device.package, device.speed),
        clock_mhz: mhz,
        fmax_mhz: fmax,
        timing_met: fmax >= mhz,
        luts: used("TRELLIS_COMB"),
        ffs: used("TRELLIS_FF"),
        dsps: used("MULT18X18D"),
        brams: used("DP16KD"),
        ios: used("TRELLIS_IO"),
        bitstream: bitstream.display().to_string(),
        notes: vec![format!("the block alone, its Wishbone bus on pins nextpnr chose: the bitstream proves the flow, a board takes the block inside the SoC; fixed point at a period of {period_us} us")],
    })
}

/// What a cached HDL verdict depends on: the generated VHDL and fixed-point
/// C at the case's period, the case, the generator and the tools.
pub fn cache_key(ast: &Model, test: &TestCase, tools: &str) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let plan = super::plan::plan(ast)?;
    let bindings = super::binding::resolve_all(ast, &plan)?;
    let case = super::case::read(test, &plan, &bindings).ok_or("the case does not execute")??;
    let fx_plan = super::fxc::plan(&plan, &[case.machine], case.step_us)?;
    let fx = fx_plan.machines[0].fixed.as_deref().expect("fixed");
    let mut hash = Sha256::new();
    let mut field = |label: &str, bytes: &[u8]| {
        hash.update(label.as_bytes());
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    };
    field("format", CACHE_FORMAT.as_bytes());
    field("generator", env!("CARGO_PKG_VERSION").as_bytes());
    field("tools", tools.as_bytes());
    for (name, text) in super::vhdl::files(&plan, &plan.machines[case.machine], fx) {
        field("file", name.as_bytes());
        field("text", text.as_bytes());
    }
    let generated = super::cgen::generate(&fx_plan, case.step_us);
    field("c", generated.source.as_bytes());
    field("h", generated.header.as_bytes());
    if let Some(AttributeValue::Map(map)) = test.attributes.get("execute") {
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

/// Keep each case's verdict as `<case>.hdl.json` in the model's cache.
pub fn cache(ast: &Model, model_path: &Path, outcomes: &[Outcome]) -> Result<Vec<PathBuf>, String> {
    cache_kind(ast, model_path, outcomes, "hdl", |test, tools| cache_key(ast, test, tools))
}

/// Keep each case's verdict as `<case>.<kind>.json`: `hdl` (the VHDL in
/// GHDL), `soc` (the SoC, simulated or on its board).
pub(super) fn cache_kind(ast: &Model, model_path: &Path, outcomes: &[Outcome], kind: &str, key: impl Fn(&TestCase, &str) -> Result<String, String>) -> Result<Vec<PathBuf>, String> {
    let dir = crate::compiler::simulation::cache::dir(model_path);
    let (label, device) = if kind == "hdl" { ("HDL", "VHDL") } else { ("SoC", "SoC") };
    let mut written = Vec::new();
    for o in outcomes {
        let Some(test) = ast.test_cases.iter().find(|t| t.id == o.test_case) else { continue };
        let values = o.expectations.iter().filter_map(|j| j.c.clone().or_else(|| j.model.clone())).collect();
        let b2b = o.bit_exact.as_ref().map(|b| format!("{device} vs fixed-point C: {} steps x {} outputs, largest difference {}", b.steps, b.columns, b.max_difference)).unwrap_or_default();
        let q = o.quantisation.as_ref().map(|q| format!("; fixed point vs model: largest difference {:e} ({}, tolerance {:e} {})", q.max_difference, q.column, q.tolerance, q.origin)).unwrap_or_default();
        let outcome = crate::compiler::simulation::Outcome {
            test_case: o.test_case.clone(),
            name: o.name.clone(),
            verifies: o.verifies.clone(),
            verdict: o.verdict.clone(),
            reason: if o.verdict == "pass" { String::new() } else { format!("{label}: {}", o.reason) },
            values,
            key: key(test, &o.tools)?,
            model: format!("state_machine {}", o.machine),
            tool: o.tools.clone(),
            kind: kind.to_string(),
            target: o.block.clone(),
            log: format!("{b2b}{q}"),
            ..Default::default()
        };
        written.push(crate::compiler::simulation::cache::write_kind(&dir, kind, &outcome)?);
    }
    Ok(written)
}

/// `arclang export -f vhdl`: for each allocated machine, the VHDL, the
/// register map and the driver, and the fixed-point C reference, at the
/// period of the C.
pub fn export(ast: &Model) -> Result<(super::Files, Vec<String>), String> {
    let plan = super::plan::plan(ast)?;
    let bindings = super::binding::resolve_all(ast, &plan)?;
    let allocations = super::alloc::allocations(ast, &plan)?;
    if allocations.is_empty() {
        return Err("no state_machine is allocated to programmable logic: give it to a node of kind \"fpga\" (docs/embedded-fpga.md)".to_string());
    }
    let period = super::default_period_us(&bindings);
    let mut files = Vec::new();
    let mut notes = Vec::new();
    let machines: Vec<usize> = allocations.iter().map(|a| a.machine).collect();
    let fx_plan = super::fxc::plan(&plan, &machines, period)?;
    for (a, fm) in allocations.iter().zip(&fx_plan.machines) {
        let fx = fm.fixed.as_deref().expect("fixed");
        let m = &plan.machines[a.machine];
        files.extend(super::vhdl::files(&plan, m, fx));
        notes.push(format!("state_machine '{}' on '{}' ({}, {} MHz): block {}_wb", m.name, a.node, a.part, a.clock_hz / 1e6, super::vhdl::entity(&plan, m)));
        for (i, v) in fx.vars.iter().enumerate() {
            if let Kind::Fixed(_) = v.kind {
                notes.push(format!("  {} {} (range {} .. {}, {})", m.program().variables[i].name, v.label(), v.range.0, v.range.1, v.origin));
            }
        }
    }
    let generated = super::cgen::generate(&fx_plan, period);
    files.push((format!("c/{}", generated.header_name), generated.header));
    files.push((format!("c/{}", generated.source_name), generated.source));
    notes.push(format!("fixed point at a period of {period} us; the C reference in c/ (bit-exact with the VHDL)"));
    Ok((files, notes))
}
