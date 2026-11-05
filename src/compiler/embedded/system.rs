//! The processor and the programmable logic together: one LiteX SoC per
//! model (a VexRiscv, its RAM holding the Zephyr application from reset, a
//! UART, a timer, and every generated block on the Wishbone bus at the
//! address its driver uses), used two ways from one description
//! (`system/soc.py`):
//!
//! - **simulated cycle by cycle** in Verilator (LiteX's simulator): each
//!   `execute:` case of an allocated machine runs the step() API
//!   application (`pil_direct`) on the simulated VexRiscv, which calls the
//!   block through its driver, over the bus, in the simulated logic; the
//!   block's outputs come back to the bench on the console and are judged
//!   as `hdl` judges the VHDL: word for word against the fixed-point C,
//!   within the tolerance of the model, the expectations as the model's;
//! - **built into one bitstream** for a board (Yosys, nextpnr-ecp5,
//!   ecppack): the same SoC, the same Verilog of each block (GHDL's
//!   synthesis of its VHDL), the same Zephyr image in its RAM.
//!
//! On a board, the same bench runs the cases through the board's serial
//! port (`--port`), against the SoC the bitstream holds.

#[path = "system/bench.rs"]
mod bench;
#[path = "system/io.rs"]
pub mod io;

use super::alloc::Allocation;
use super::hdl::{self, Outcome};
use super::plan::Plan;
use super::reference::{self, Table};
use crate::compiler::ast::{Model, TestCase};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// The SoC description, run by LiteX's Python.
pub const SOC_SCRIPT: &str = include_str!("system/soc.py");

/// The target package whose Zephyr board the SoC is (its mains, its build).
const PACKAGE: &str = "zephyr/litex_vexriscv";

/// The RAM of the SoC: the Zephyr application, its data and stacks. 64 KiB
/// of block RAM (32 of the 56 DP16KD of an LFE5U-25F).
pub const RAM_BYTES: u32 = 64 * 1024;

/// A board a bitstream is built for: its LiteX platform (`soc.py` knows
/// it), the parts it carries, its name for openFPGALoader.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Board {
    pub name: &'static str,
    pub parts: &'static [&'static str],
    pub loader: &'static str,
    /// Its oscillator.
    pub clock_hz: u64,
}

/// The boards, the first of a part being its default.
pub const BOARDS: &[Board] = &[
    Board { name: "ulx3s", parts: &["LFE5U-12F-6BG381C", "LFE5U-25F-6BG381C", "LFE5U-45F-6BG381C", "LFE5U-85F-6BG381C"], loader: "ulx3s", clock_hz: 25_000_000 },
    Board { name: "colorlight_i5", parts: &["LFE5U-25F-6BG381C"], loader: "colorlight-i5", clock_hz: 25_000_000 },
    Board { name: "colorlight_i9", parts: &["LFE5U-45F-6BG381C"], loader: "colorlight-i9", clock_hz: 25_000_000 },
    Board { name: "ecp5_evn", parts: &["LFE5UM5G-85F-8BG381C"], loader: "ecp5_evn", clock_hz: 12_000_000 },
];

fn same_part(a: &str, b: &str) -> bool {
    let norm = |p: &str| p.trim().to_uppercase().trim_end_matches(['C', 'I']).to_string();
    norm(a) == norm(b)
}

/// The board of a SoC: the one asked for, else the node's `board:`, else
/// the first board carrying its part; it must carry the part.
pub fn board_for(allocations: &[Allocation], asked: Option<&str>) -> Result<Board, String> {
    let a = allocations.first().ok_or("no state_machine is allocated to programmable logic: the SoC would have no block")?;
    let names = || BOARDS.iter().map(|b| b.name).collect::<Vec<_>>().join(", ");
    let named = asked.or(a.board.as_deref());
    let board = match named {
        Some(n) => *BOARDS.iter().find(|b| b.name == n.trim().to_lowercase().replace('-', "_")).ok_or_else(|| format!("no board '{n}' (boards: {})", names()))?,
        None => *BOARDS.iter().find(|b| b.parts.iter().any(|p| same_part(p, &a.part))).ok_or_else(|| format!("node '{}': no board carries {} (boards: {}); name one with board:, or choose a part of one", a.node, a.part, names()))?,
    };
    if !board.parts.iter().any(|p| same_part(p, &a.part)) {
        return Err(format!("board '{}' carries {}, not node '{}''s {}", board.name, board.parts.join(" or "), a.node, a.part));
    }
    Ok(board)
}

/// Bumped when the meaning of a cached SoC verdict changes.
const CACHE_FORMAT: &str = "arclang-soc-cache-1";

/// Wall-clock limits of a simulated run (the simulator runs at about a
/// tenth of real time): the boot, then each step.
const BOOT_SECONDS: u64 = 120;
const STEP_SECONDS: u64 = 60;

/// The nodes of the Zephyr board's devicetree the SoC does not have and
/// LiteX's overlay generator does not know: disabled.
const ABSENT_NODES: &[&str] = &["dna0", "prbs0", "i2c1", "clock0"];

#[derive(Debug, Clone)]
pub struct Options {
    pub work: PathBuf,
    pub only: Option<String>,
    /// The C compiler of the fixed-point reference.
    pub compiler: String,
    /// Run the cases in the simulated SoC.
    pub simulate: bool,
    /// Simulate the SoC as synthesised for the ECP5 (a netlist of ECP5
    /// cells), not as written.
    pub gate_level: bool,
    /// Build the bitstream.
    pub bitstream: bool,
    /// For this board (else the node's `board:`, else the board of its part).
    pub board: Option<String>,
    /// Run the cases on the board through this serial port (it holds the bitstream).
    pub port: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Bitstream {
    pub board: String,
    pub device: String,
    pub clock_mhz: f64,
    pub fmax_mhz: f64,
    pub timing_met: bool,
    /// Every clock: its name in nextpnr, its Fmax, its target (MHz).
    pub clocks: Vec<(String, f64, f64)>,
    pub luts: u64,
    pub ffs: u64,
    pub dsps: u64,
    pub brams: u64,
    pub blocks: Vec<String>,
    pub image_bytes: u64,
    pub bitstream: String,
    /// How to write it to the board.
    pub flash: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub outcomes: Vec<Outcome>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bitstream: Option<Result<Bitstream, String>>,
}

fn litex_python() -> String {
    std::env::var("ARCLANG_LITEX_PYTHON").unwrap_or_else(|_| "python3".to_string())
}

/// The versions of what built and ran the SoC, as an outcome names them.
pub fn tools(compiler: &str) -> String {
    let litex = std::process::Command::new(litex_python())
        .args(["-c", "import importlib.metadata as m; print(m.version('litex'))"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "?".to_string());
    format!(
        "LiteX {litex} (VexRiscv standard); {}; {}; {}",
        hdl::version("verilator", "--version"),
        hdl::version("ghdl", "--version"),
        hdl::tools(compiler, false).split("; ").nth(1).unwrap_or(compiler)
    )
}

/// One built SoC: the application at a period, its image, the simulator.
struct Built {
    dir: PathBuf,
    /// The simulator's directory (`sim` or `gatesim`).
    simulator: &'static str,
    image: PathBuf,
    port: u16,
}

/// The SoCs built, by period.
struct Builds<'a> {
    plan: &'a Plan,
    allocations: &'a [Allocation],
    board: Board,
    sys_hz: f64,
    work: PathBuf,
    simulate: bool,
    gate_level: bool,
    built: HashMap<(Option<String>, u64), Result<Built, String>>,
}

impl Builds<'_> {
    /// The SoC whose application runs the step() API (no binding) or the
    /// controller's loop through a binding's I/O, at a period.
    fn get(&mut self, binding: Option<&super::binding::Binding>, period_us: u64) -> Result<&Built, String> {
        let key = (binding.map(|b| b.ident.clone()), period_us);
        if !self.built.contains_key(&key) {
            let dir = match binding {
                Some(b) => self.work.join(format!("soc-{}-{period_us}us", b.ident)),
                None => self.work.join(format!("soc-{period_us}us")),
            };
            let built = prepare(self.plan, self.allocations, &self.board, self.sys_hz, binding, period_us, &dir, self.simulate.then_some(if self.gate_level { "gatesim" } else { "sim" }));
            self.built.insert(key.clone(), built);
        }
        self.built[&key].as_ref().map_err(|e| e.clone())
    }
}

/// A free TCP port for a simulator's console.
fn free_port() -> Result<u16, String> {
    std::net::TcpListener::bind(("127.0.0.1", 0)).and_then(|l| l.local_addr()).map(|a| a.port()).map_err(|e| format!("no free TCP port for the simulator's console: {e}"))
}

/// The JSON the SoC script reads: the model, the board and the part, the
/// clock, the RAM and its image, the console's TCP port, every block (its
/// Verilog, its address, its address bits).
pub fn soc_config(plan: &Plan, allocations: &[Allocation], board: &Board, sys_hz: f64, period_us: u64, dir: &Path, port: u16) -> Result<String, String> {
    let first = allocations.first().ok_or("no state_machine is allocated to programmable logic: the SoC would have no block")?;
    let mut blocks = Vec::new();
    for (k, a) in allocations.iter().enumerate() {
        let m = &plan.machines[a.machine];
        let ent = super::vhdl::entity(plan, m);
        let map = super::regmap::map(plan, m, &super::fixed::machine(m, period_us)?);
        blocks.push(serde_json::json!({
            "name": ent,
            "verilog": dir.join("hw").join(format!("{ent}_wb.v")).display().to_string(),
            "base": super::zephyr::hw_base(k),
            "size": 0x1000,
            "address_bits": map.address_bits(),
            "clock_hz": a.clock_hz as u64,
        }));
    }
    let config = serde_json::json!({
        "name": plan.ident,
        "board": board.name,
        "device": first.part,
        "clock_hz": sys_hz as u64,
        "ram_bytes": RAM_BYTES,
        "image": dir.join("build").join("zephyr").join("zephyr.bin").display().to_string(),
        "tcp_port": port,
        "seed": 1,
        "blocks": blocks,
    });
    serde_json::to_string_pretty(&config).map_err(|e| e.to_string())
}

/// The application, the blocks' Verilog, the SoC's map, the Zephyr image
/// and the simulator (`simulator`: `sim` as written, `gatesim` as
/// synthesised), for one period.
#[allow(clippy::too_many_arguments)]
fn prepare(plan: &Plan, allocations: &[Allocation], board: &Board, sys_hz: f64, binding: Option<&super::binding::Binding>, period_us: u64, dir: &Path, simulator: Option<&'static str>) -> Result<Built, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot make {}: {e}", dir.display()))?;
    let package = super::target::find(PACKAGE)?;
    // One SoC is one FPGA: every block on the same part (each on its own clock).
    if let Some(a) = allocations.iter().find(|a| a.part != allocations[0].part) {
        return Err(format!("node '{}' ({}) and node '{}' ({}): one SoC is one FPGA, one part; two parts are two chips", allocations[0].node, allocations[0].part, a.node, a.part));
    }
    if let Some(b) = binding {
        super::target::check(b, &package)?;
    }
    let app = super::zephyr::hw_app(plan, allocations, binding, &package, period_us)?;
    let app_dir = dir.join("app");
    let _ = std::fs::remove_dir_all(&app_dir);
    super::pil::write_app(&app_dir, &app)?;
    // Each block's VHDL as Verilog, by GHDL's synthesis: the netlist the
    // simulator and the bitstream share.
    let hw = dir.join("hw");
    std::fs::create_dir_all(&hw).map_err(|e| format!("cannot make {}: {e}", hw.display()))?;
    for a in allocations {
        let ent = super::vhdl::entity(plan, &plan.machines[a.machine]);
        let sources = ["arclang_fx_pkg.vhd".to_string(), format!("{ent}.vhd"), format!("{ent}_wb.vhd")];
        for s in &sources {
            std::fs::copy(app_dir.join("hw").join(s), hw.join(s)).map_err(|e| format!("cannot copy {s}: {e}"))?;
        }
        let mut analyse = vec!["-a".to_string(), "--std=08".to_string()];
        analyse.extend(sources.iter().cloned());
        hdl::run_tool("ghdl", &analyse, &hw, "ghdl -a")?;
        hdl::write(&hw, &format!("{ent}_wb.v"), &verilog_of(&hw, &format!("{ent}_wb"))?)?;
    }
    let port = free_port()?;
    hdl::write(dir, "soc.json", &soc_config(plan, allocations, board, sys_hz, period_us, dir, port)?)?;
    hdl::write(dir, "soc.py", SOC_SCRIPT)?;
    let python = litex_python();
    run_step(&[&python, "soc.py", "soc.json", "csr"], dir, "LiteX (the SoC's map)")?;
    let csr = overlay(dir)?;
    // The application tells the bench where the cycles end.
    hdl::write(&app_dir.join("src").join("model"), "arclang_bench_hook.c", &io::hook(&csr)?)?;
    let build = dir.join("build");
    let overlay_arg = format!("-DDTC_OVERLAY_FILE={}", dir.join("soc.overlay").display());
    let conf_arg = format!("-DEXTRA_CONF_FILE={}", dir.join("soc.conf").display());
    let period_arg = format!("-DARCLANG_PERIOD_US={period_us}");
    let build_s = build.display().to_string();
    let app_s = app_dir.display().to_string();
    let main = if binding.is_some() { "-DARCLANG_MAIN=pil_io" } else { "-DARCLANG_MAIN=pil_direct" };
    let mut west = vec!["west", "build", "-p", "always", "-b", "litex_vexriscv", "-d", &build_s, &app_s, "--", main, &overlay_arg, &conf_arg];
    if binding.is_none() {
        west.push(&period_arg);
    }
    run_step(&west, dir, "west build (Zephyr for the SoC)")?;
    let image = build.join("zephyr").join("zephyr.bin");
    if !image.is_file() {
        return Err(format!("the Zephyr build made no {}", image.display()));
    }
    if let Some(what) = simulator {
        let label = if what == "gatesim" { "LiteX, Yosys and Verilator (the SoC synthesised for the ECP5, simulated as a netlist)" } else { "LiteX and Verilator (the simulator)" };
        run_step(&[&python, "soc.py", "soc.json", what], dir, label)?;
    }
    Ok(Built { dir: dir.to_path_buf(), simulator: simulator.unwrap_or("sim"), image, port })
}

/// GHDL's synthesis of a block as Verilog: its standard output only (its
/// notes, on the standard error, are no Verilog).
fn verilog_of(dir: &Path, top: &str) -> Result<String, String> {
    let out = std::process::Command::new("ghdl")
        .args(["--synth", "--std=08", "--out=verilog", top])
        .current_dir(dir)
        .output()
        .map_err(|e| format!("cannot run ghdl: {e} (run in ArcLang's system image: tools/embedded/system/run.sh)"))?;
    if !out.status.success() {
        return Err(format!("ghdl --synth --out=verilog {top} failed:\n{}", String::from_utf8_lossy(&out.stderr)));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn run_step(command: &[&str], dir: &Path, what: &str) -> Result<String, String> {
    let words: Vec<String> = command.iter().map(|s| s.to_string()).collect();
    super::pil::run_command(&words, Some(dir), &[], what).map_err(|e| format!("{e}\n(run in ArcLang's system image: tools/embedded/system/run.sh)"))
}

/// Zephyr's devicetree and configuration overlays from the SoC's map:
/// LiteX's generator, the channels' nodes and aliases, and the board's
/// other nodes disabled. The map, read.
fn overlay(dir: &Path) -> Result<serde_json::Value, String> {
    let python = litex_python();
    let script = std::process::Command::new(&python)
        .args(["-c", "import litex.tools.litex_json2dts_zephyr as m; print(m.__file__)"])
        .output()
        .map_err(|e| format!("cannot run {python}: {e}"))?;
    let script = String::from_utf8_lossy(&script.stdout).trim().to_string();
    run_step(&[&python, &script, "--dts", "soc.overlay", "--config", "soc.conf", "csr.json"], dir, "LiteX (Zephyr's overlay)")?;
    let csr: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("csr.json")).map_err(|e| format!("no csr.json: {e}"))?).map_err(|e| format!("csr.json: {e}"))?;
    let mut text = std::fs::read_to_string(dir.join("soc.overlay")).map_err(|e| format!("no soc.overlay: {e}"))?;
    text.push_str(&io::overlay(&csr)?);
    text.push_str("\n/* ArcLang: the board's nodes this SoC does not have. */\n");
    for node in ABSENT_NODES {
        let _ = writeln!(text, "&{node} {{\n    status = \"disabled\";\n}};");
    }
    hdl::write(dir, "soc.overlay", &text)?;
    Ok(csr)
}

/// The machine's columns of a run of the whole plan, as the words of the
/// block (a `Real` in fixed point: its value times 2^frac, exact).
fn words(plan: &Plan, machine: usize, fx: &super::fixed::MachineFixed, table: &Table) -> Table {
    let m = &plan.machines[machine];
    let mine = hdl::machine_columns(table, &m.ident);
    let base = m.states.len() + m.actions.len();
    let rows = mine
        .rows
        .iter()
        .map(|row| {
            row.iter()
                .enumerate()
                .map(|(c, x)| match c.checked_sub(base).and_then(|i| fx.vars.get(i)) {
                    Some(v) => match v.kind {
                        super::fixed::Kind::Fixed(f) => (x * 2f64.powi(f)).round(),
                        _ => *x,
                    },
                    None => *x,
                })
                .collect()
        })
        .collect();
    Table { rows, ..mine }
}

fn run_case(test: &TestCase, plan: &Plan, bindings: &[super::binding::Binding], builds: &mut Builds, options: &Options, tools: &str) -> Outcome {
    let mut outcome = hdl::new_outcome(test, tools);
    let texts: Vec<String> = outcome.expectations.iter().map(|j| j.expect.clone()).collect();
    if let Err(e) = run_case_inner(test, plan, bindings, builds, options, &texts, &mut outcome) {
        outcome.verdict = "error".to_string();
        outcome.reason = e;
    }
    outcome
}

fn run_case_inner(test: &TestCase, plan: &Plan, bindings: &[super::binding::Binding], builds: &mut Builds, options: &Options, texts: &[String], outcome: &mut Outcome) -> Result<(), String> {
    let dir = options.work.join(format!("case-{}", hdl::case_dir(test)));
    let r = hdl::reference(test, plan, bindings, &options.compiler, &dir, texts, outcome)?;
    let bound = r.case.binding.as_ref().map(|name| bindings.iter().find(|b| b.is(name)).ok_or_else(|| format!("no io_binding '{name}'"))).transpose()?;
    // A binding with CAN runs through the step() API: the SoC has no CAN
    // core yet (its channels stay proved by PIL in Renode).
    let uses_can = |b: &super::binding::Binding| b.inputs.iter().any(|i| matches!(i.source, super::binding::Source::Can(_))) || b.outputs.iter().any(|o| matches!(o.sink, super::binding::Sink::Can(_)));
    let binding = bound.filter(|b| !uses_can(b) && options.port.is_none());
    let note = match bound {
        Some(b) if binding.is_none() && uses_can(b) => Some(format!(" (through the step() API: io_binding '{}' uses CAN, which the SoC has no core for yet; its channels are proved by arclang pil in Renode)", b.name)),
        Some(b) if binding.is_none() => Some(format!(" (through the step() API: on a board the channels of io_binding '{}' are not wired to pins yet)", b.name)),
        _ => None,
    };
    let names = reference::run(plan, &r.vectors, r.case.step_us).names;
    let lines: Vec<String> = (0..r.case.steps).map(|k| super::zephyr::direct_line(plan, &r.vectors, k)).collect();
    let built = builds.get(binding, r.case.step_us)?;
    let gateware = built.dir.join(built.simulator).join("gateware");
    let drive = match binding {
        Some(b) => {
            // The case as channel operations, played by the simulated bench.
            let ops = super::sil::channel_operations(&r.case, plan, b, &r.vectors)?;
            hdl::write(&gateware, "player.hex", &io::player(&ops)?)?;
            let _ = std::fs::remove_file(gateware.join("recorder.txt"));
            bench::Drive::Cycles(r.case.steps)
        }
        None => bench::Drive::Lines(&lines),
    };
    let (text, log) = match &options.port {
        Some(port) => bench::board(port, drive, STEP_SECONDS)?,
        None => bench::simulate(&gateware, built.port, drive, BOOT_SECONDS, STEP_SECONDS)?,
    };
    let _ = std::fs::write(dir.join("console.txt"), &log);
    // Through the I/O: the channels the bench recorded after each cycle.
    let (text, io_count) = match binding {
        Some(b) => {
            let recorder = std::fs::read_to_string(gateware.join("recorder.txt")).map_err(|e| format!("the bench recorded nothing: {e}"))?;
            let _ = std::fs::write(dir.join("recorder.txt"), &recorder);
            let channels = io::outputs(b, &recorder, r.case.steps)?;
            let joined: String = text.lines().zip(&channels).map(|(l, c)| format!("{l}|{c}\n")).collect();
            (joined, super::sil::io_columns(b).len())
        }
        None => (text, 0),
    };
    let rows = super::pil::parse_result(&text, names.len(), io_count, r.case.steps)?;
    // The channels must be what the model makes them, exactly.
    if let Some(b) = binding {
        let model_full = reference::run(plan, &r.vectors, r.case.step_us);
        let expected = super::sil::expected_io(plan, b, &model_full);
        let columns = super::sil::io_columns(b);
        if let Some((k, c)) = rows.iter().enumerate().find_map(|(k, row)| (0..io_count).find(|&c| row.values[names.len() + c] != expected[k][c]).map(|c| (k, c))) {
            outcome.reason = format!("the channel {} departs from the model at step {k} ({} ms): {} on the SoC, {} expected", columns[c], k as u64 * r.case.step_us / 1000, rows[k].values[names.len() + c], expected[k][c]);
        }
    }
    let rows: Vec<super::pil::Row> = rows.into_iter().map(|row| super::pil::Row { time: row.time, values: row.values[..names.len()].to_vec() }).collect();
    let worst = rows.iter().map(|row| row.time).max().unwrap_or(0);
    let times: Vec<u64> = rows.iter().map(|row| row.time).collect();
    let table = Table { names, period_us: r.case.step_us, rows: rows.into_iter().map(|row| row.values).collect(), units: Default::default() };
    let device_raw = words(plan, r.machine, &r.fx, &table);
    let clock = builds.sys_hz / 1e6;
    let on = if options.port.is_some() {
        "on the board"
    } else if options.gate_level {
        "in the SoC's ECP5 netlist (gate level)"
    } else {
        "in the simulated SoC"
    };
    outcome.cycles = Some(format!("step() took at most {:.1} us {on} at {clock} MHz (the driver, the bus and the block)", worst as f64 / 1000.0));
    if let (Some(n), Some(c)) = (&note, outcome.cycles.as_mut()) {
        c.push_str(n);
    }
    let departed = std::mem::take(&mut outcome.reason);
    hdl::conclude(&r, plan, &device_raw, if options.port.is_some() { "the board" } else { "the SoC" }, texts, outcome);
    if !departed.is_empty() {
        outcome.verdict = "fail".to_string();
        outcome.reason = departed;
    }
    if binding.is_some() {
        // Through the I/O the report gives each cycle's time on the target (us).
        let gaps: Vec<u64> = times.windows(2).map(|w| w[1].saturating_sub(w[0])).collect();
        let (lo, hi) = (gaps.iter().min().copied().unwrap_or(0), gaps.iter().max().copied().unwrap_or(0));
        outcome.cycles = Some(format!("{} cycles through the I/O {on}, {lo} to {hi} us apart for a period of {} us (the first at {} us)", times.len(), r.case.step_us, times.first().copied().unwrap_or(0)));
    }
    Ok(())
}

/// Run the cases of the allocated machines in the simulated SoC (or on the
/// board) and (`options.board`) build the bitstream.
pub fn run(ast: &Model, options: &Options) -> Result<Report, String> {
    let plan = super::plan::plan(ast)?;
    let bindings = super::binding::resolve_all(ast, &plan)?;
    let allocations = super::alloc::allocations(ast, &plan)?;
    if allocations.is_empty() {
        return Err("no state_machine is allocated to programmable logic: give it to a node of kind \"fpga\" (docs/embedded-fpga.md)".to_string());
    }
    let board = board_for(&allocations, options.board.as_deref())?;
    std::fs::create_dir_all(&options.work).map_err(|e| format!("cannot make {}: {e}", options.work.display()))?;
    let work = options.work.canonicalize().unwrap_or_else(|_| options.work.clone());
    let options = Options { work: work.clone(), ..options.clone() };
        let sys_hz = super::alloc::processor_clock(ast)?.unwrap_or(allocations[0].clock_hz);
    let mut builds = Builds { plan: &plan, allocations: &allocations, board, sys_hz, work, simulate: options.simulate && options.port.is_none(), gate_level: options.gate_level, built: HashMap::new() };
    let mut outcomes = Vec::new();
    if options.simulate || options.port.is_some() {
        let selected = hdl::selected(ast, &plan, &allocations, &bindings, options.only.as_deref());
        if selected.is_empty() {
            return Err(match &options.only {
                Some(o) => format!("no test_case '{o}' that executes a machine of the programmable logic"),
                None => "no test_case executes a machine of the programmable logic".to_string(),
            });
        }
        let mut tools = tools(&options.compiler);
        if options.gate_level {
            tools.push_str(&format!("; blocks at gate level ({}, synth_ecp5)", hdl::version("yosys", "-V")));
        }
        outcomes = selected.into_iter().map(|t| run_case(t, &plan, &bindings, &mut builds, &options, &tools)).collect();
    }
    let bitstream = options.bitstream.then(|| {
        let period = super::default_period_us(&bindings);
        builds.get(None, period).map(|b| (b.dir.clone(), b.image.clone())).and_then(|(dir, image)| bitstream(&plan, &allocations, &dir, &image, &board))
    });
    Ok(Report { outcomes, bitstream })
}

/// The SoC built for the board: synthesis, place and route, the bitstream;
/// its resources and its Fmax from nextpnr's log.
fn bitstream(plan: &Plan, allocations: &[Allocation], dir: &Path, image: &Path, board: &Board) -> Result<Bitstream, String> {
    let log = run_step(&[&litex_python(), "soc.py", "soc.json", "board"], dir, "LiteX, Yosys, nextpnr-ecp5 and ecppack (the bitstream)")?;
    let _ = std::fs::write(dir.join("board.log"), &log);
    let gateware = dir.join("board").join("gateware");
    let bit = std::fs::read_dir(&gateware)
        .map_err(|e| format!("no {}: {e}", gateware.display()))?
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|x| x == "bit"))
        .ok_or_else(|| format!("the build made no bitstream in {}", gateware.display()))?;
    let used = |cell: &str| -> u64 {
        log.lines()
            .filter_map(|l| l.split_once(&format!("{cell}:")).map(|(_, rest)| rest.trim()))
            .filter_map(|rest| rest.split('/').next().and_then(|n| n.trim().parse().ok()))
            .next_back()
            .unwrap_or(0)
    };
    let clocks = clocks_of(&log);
    if clocks.is_empty() {
        return Err("nextpnr reported no Fmax".to_string());
    }
    let a = &allocations[0];
    // The clock the summary names: the slowest margin.
    let worst = clocks.iter().min_by(|x, y| (x.1 / x.2).total_cmp(&(y.1 / y.2))).expect("one");
    Ok(Bitstream {
        board: board.name.to_string(),
        device: a.part.clone(),
        clock_mhz: worst.2,
        fmax_mhz: worst.1,
        timing_met: clocks.iter().all(|c| c.1 >= c.2),
        clocks: clocks.clone(),
        luts: used("TRELLIS_COMB"),
        ffs: used("TRELLIS_FF"),
        dsps: used("MULT18X18D"),
        brams: used("DP16KD"),
        blocks: allocations.iter().map(|a| format!("{}_wb", super::vhdl::entity(plan, &plan.machines[a.machine]))).collect(),
        image_bytes: std::fs::metadata(image).map(|m| m.len()).unwrap_or(0),
        flash: format!("openFPGALoader -b {} {}", board.loader, bit.display()),
        bitstream: bit.display().to_string(),
    })
}

/// Each clock nextpnr timed after routing: its name, its Fmax, its target
/// (MHz). "Max frequency for clock 'sys_clk': 41.23 MHz (PASS at 25.00
/// MHz)", printed after placement and again after routing: the last wins.
pub fn clocks_of(log: &str) -> Vec<(String, f64, f64)> {
    let mut out: Vec<(String, f64, f64)> = Vec::new();
    for line in log.lines().filter(|l| l.contains("Max frequency for clock")) {
        // The name is quoted after spaces that align the lines.
        let Some((_, rest)) = line.split_once("for clock").and_then(|(a, r)| r.split_once('\'').map(|(_, n)| (a, n))) else { continue };
        let Some((name, rest)) = rest.split_once("': ") else { continue };
        let fmax = rest.split_whitespace().next().and_then(|n| n.parse::<f64>().ok());
        let target = rest.split_once(" at ").and_then(|(_, t)| t.split_whitespace().next()).and_then(|n| n.parse::<f64>().ok());
        if let (Some(fmax), Some(target)) = (fmax, target) {
            out.retain(|c| c.0 != name);
            out.push((name.to_string(), fmax, target));
        }
    }
    out
}

/// What a cached SoC verdict depends on: what the HDL verdict does, the
/// SoC's description, the application and its target package.
pub fn cache_key(ast: &Model, test: &TestCase, tools: &str) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let plan = super::plan::plan(ast)?;
    let bindings = super::binding::resolve_all(ast, &plan)?;
    let allocations = super::alloc::allocations(ast, &plan)?;
    let case = super::case::read(test, &plan, &bindings).ok_or("the case does not execute")??;
    let package = super::target::find(PACKAGE)?;
    let app = super::zephyr::hw_app(&plan, &allocations, None, &package, case.step_us)?;
    let mut hash = Sha256::new();
    let mut field = |label: &str, bytes: &[u8]| {
        hash.update(label.as_bytes());
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    };
    field("format", CACHE_FORMAT.as_bytes());
    field("hdl", hdl::cache_key(ast, test, tools)?.as_bytes());
    field("soc", SOC_SCRIPT.as_bytes());
    field("ram", &RAM_BYTES.to_le_bytes());
    let sys_hz = super::alloc::processor_clock(ast)?.unwrap_or(allocations.first().map(|a| a.clock_hz).unwrap_or(0.0));
    field("clocks", format!("{sys_hz} {:?}", allocations.iter().map(|a| (a.clock_hz, a.part.clone(), a.board.clone())).collect::<Vec<_>>()).as_bytes());
    for (name, text, _) in &app.files {
        field("file", name.as_bytes());
        field("text", text.as_bytes());
    }
    Ok(hash.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// Keep each case's verdict as `<case>.soc.json` in the model's cache.
pub fn cache(ast: &Model, model_path: &Path, outcomes: &[Outcome]) -> Result<Vec<PathBuf>, String> {
    hdl::cache_kind(ast, model_path, outcomes, "soc", |test, tools| cache_key(ast, test, tools))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_clock_nextpnr_timed_is_read_after_routing() {
        let log = "Info: Max frequency for clock      '$glbnet$block.clk': 37.97 MHz (PASS at 20.00 MHz)\nInfo: Max frequency for clock '$glbnet$cpu.clk': 55.06 MHz (PASS at 25.00 MHz)\nInfo: Max frequency for clock      '$glbnet$block.clk': 38.49 MHz (PASS at 20.00 MHz)\nInfo: Max frequency for clock '$glbnet$cpu.clk': 18.00 MHz (FAIL at 25.00 MHz)\n";
        assert_eq!(clocks_of(log), vec![("$glbnet$block.clk".to_string(), 38.49, 20.0), ("$glbnet$cpu.clk".to_string(), 18.0, 25.0)]);
    }
}
