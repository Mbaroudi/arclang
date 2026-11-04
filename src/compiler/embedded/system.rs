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
const ABSENT_NODES: &[&str] = &["dna0", "prbs0", "i2c1", "pwm0", "gpio_out", "gpio_in", "clock0"];

#[derive(Debug, Clone)]
pub struct Options {
    pub work: PathBuf,
    pub only: Option<String>,
    /// The C compiler of the fixed-point reference.
    pub compiler: String,
    /// Run the cases in the simulated SoC.
    pub simulate: bool,
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
    image: PathBuf,
    port: u16,
}

/// The SoCs built, by period.
struct Builds<'a> {
    plan: &'a Plan,
    allocations: &'a [Allocation],
    board: Board,
    work: PathBuf,
    simulate: bool,
    built: HashMap<u64, Result<Built, String>>,
}

impl Builds<'_> {
    fn get(&mut self, period_us: u64) -> Result<&Built, String> {
        if !self.built.contains_key(&period_us) {
            let built = prepare(self.plan, self.allocations, &self.board, period_us, &self.work.join(format!("soc-{period_us}us")), self.simulate);
            self.built.insert(period_us, built);
        }
        self.built[&period_us].as_ref().map_err(|e| e.clone())
    }
}

/// A free TCP port for a simulator's console.
fn free_port() -> Result<u16, String> {
    std::net::TcpListener::bind(("127.0.0.1", 0)).and_then(|l| l.local_addr()).map(|a| a.port()).map_err(|e| format!("no free TCP port for the simulator's console: {e}"))
}

/// The JSON the SoC script reads: the model, the board and the part, the
/// clock, the RAM and its image, the console's TCP port, every block (its
/// Verilog, its address, its address bits).
pub fn soc_config(plan: &Plan, allocations: &[Allocation], board: &Board, period_us: u64, dir: &Path, port: u16) -> Result<String, String> {
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
        }));
    }
    let config = serde_json::json!({
        "name": plan.ident,
        "board": board.name,
        "device": first.part,
        "clock_hz": first.clock_hz as u64,
        "ram_bytes": RAM_BYTES,
        "image": dir.join("build").join("zephyr").join("zephyr.bin").display().to_string(),
        "tcp_port": port,
        "seed": 1,
        "blocks": blocks,
    });
    serde_json::to_string_pretty(&config).map_err(|e| e.to_string())
}

/// The application, the blocks' Verilog, the SoC's map, the Zephyr image
/// and (`simulate`) the simulator, for one period.
fn prepare(plan: &Plan, allocations: &[Allocation], board: &Board, period_us: u64, dir: &Path, simulate: bool) -> Result<Built, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot make {}: {e}", dir.display()))?;
    let package = super::target::find(PACKAGE)?;
    // Every allocated machine on one clock: the SoC has one.
    if let Some(a) = allocations.iter().find(|a| a.clock_hz != allocations[0].clock_hz || a.part != allocations[0].part) {
        return Err(format!("node '{}' ({}, {} MHz) and node '{}' ({}, {} MHz): one SoC has one part and one clock", allocations[0].node, allocations[0].part, allocations[0].clock_hz / 1e6, a.node, a.part, a.clock_hz / 1e6));
    }
    let app = super::zephyr::hw_app(plan, allocations, None, &package, period_us)?;
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
    hdl::write(dir, "soc.json", &soc_config(plan, allocations, board, period_us, dir, port)?)?;
    hdl::write(dir, "soc.py", SOC_SCRIPT)?;
    let python = litex_python();
    run_step(&[&python, "soc.py", "soc.json", "csr"], dir, "LiteX (the SoC's map)")?;
    overlay(dir)?;
    let build = dir.join("build");
    let overlay_arg = format!("-DDTC_OVERLAY_FILE={}", dir.join("soc.overlay").display());
    let conf_arg = format!("-DEXTRA_CONF_FILE={}", dir.join("soc.conf").display());
    let period_arg = format!("-DARCLANG_PERIOD_US={period_us}");
    let build_s = build.display().to_string();
    let app_s = app_dir.display().to_string();
    run_step(&["west", "build", "-p", "always", "-b", "litex_vexriscv", "-d", &build_s, &app_s, "--", "-DARCLANG_MAIN=pil_direct", &period_arg, &overlay_arg, &conf_arg], dir, "west build (Zephyr for the SoC)")?;
    let image = build.join("zephyr").join("zephyr.bin");
    if !image.is_file() {
        return Err(format!("the Zephyr build made no {}", image.display()));
    }
    if simulate {
        run_step(&[&python, "soc.py", "soc.json", "sim"], dir, "LiteX and Verilator (the simulator)")?;
    }
    Ok(Built { dir: dir.to_path_buf(), image, port })
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
/// LiteX's generator, and the board's other nodes disabled.
fn overlay(dir: &Path) -> Result<(), String> {
    let python = litex_python();
    let script = std::process::Command::new(&python)
        .args(["-c", "import litex.tools.litex_json2dts_zephyr as m; print(m.__file__)"])
        .output()
        .map_err(|e| format!("cannot run {python}: {e}"))?;
    let script = String::from_utf8_lossy(&script.stdout).trim().to_string();
    run_step(&[&python, &script, "--dts", "soc.overlay", "--config", "soc.conf", "csr.json"], dir, "LiteX (Zephyr's overlay)")?;
    let mut text = std::fs::read_to_string(dir.join("soc.overlay")).map_err(|e| format!("no soc.overlay: {e}"))?;
    text.push_str("\n/* ArcLang: the board's nodes this SoC does not have. */\n");
    for node in ABSENT_NODES {
        let _ = writeln!(text, "&{node} {{\n    status = \"disabled\";\n}};");
    }
    hdl::write(dir, "soc.overlay", &text)?;
    Ok(())
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

/// Where a case's steps go: the simulator, started for the case, or the board.
enum Target<'a> {
    Simulator(&'a Built),
    Board(&'a str),
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
    let built = builds.get(r.case.step_us)?;
    let lines: Vec<String> = (0..r.case.steps).map(|k| super::zephyr::direct_line(plan, &r.vectors, k)).collect();
    let target = match &options.port {
        Some(port) => Target::Board(port),
        None => Target::Simulator(built),
    };
    let names = reference::run(plan, &r.vectors, r.case.step_us).names;
    let (text, log) = match target {
        Target::Simulator(b) => bench::simulate(&b.dir.join("sim").join("gateware"), b.port, &lines, BOOT_SECONDS, STEP_SECONDS)?,
        Target::Board(port) => bench::board(port, &lines, STEP_SECONDS)?,
    };
    let _ = std::fs::write(dir.join("console.txt"), &log);
    let rows = super::pil::parse_result(&text, names.len(), 0, r.case.steps)?;
    let worst = rows.iter().map(|row| row.time).max().unwrap_or(0);
    let table = Table { names, period_us: r.case.step_us, rows: rows.into_iter().map(|row| row.values).collect(), units: Default::default() };
    let device_raw = words(plan, r.machine, &r.fx, &table);
    let clock = builds.allocations.first().map(|a| a.clock_hz / 1e6).unwrap_or(0.0);
    let on = if options.port.is_some() { "on the board" } else { "in the simulated SoC" };
    outcome.cycles = Some(format!("step() took at most {:.1} us {on} at {clock} MHz (the driver, the bus and the block)", worst as f64 / 1000.0));
    hdl::conclude(&r, plan, &device_raw, if options.port.is_some() { "the board" } else { "the SoC" }, texts, outcome);
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
    let mut builds = Builds { plan: &plan, allocations: &allocations, board, work, simulate: options.simulate && options.port.is_none(), built: HashMap::new() };
    let mut outcomes = Vec::new();
    if options.simulate || options.port.is_some() {
        let selected = hdl::selected(ast, &plan, &allocations, &bindings, options.only.as_deref());
        if selected.is_empty() {
            return Err(match &options.only {
                Some(o) => format!("no test_case '{o}' that executes a machine of the programmable logic"),
                None => "no test_case executes a machine of the programmable logic".to_string(),
            });
        }
        let tools = tools(&options.compiler);
        outcomes = selected.into_iter().map(|t| run_case(t, &plan, &bindings, &mut builds, &options, &tools)).collect();
    }
    let bitstream = options.bitstream.then(|| {
        let period = super::default_period_us(&bindings);
        builds.get(period).map(|b| (b.dir.clone(), b.image.clone())).and_then(|(dir, image)| bitstream(&plan, &allocations, &dir, &image, &board))
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
    // "Max frequency for clock 'sys_clk': 41.23 MHz (PASS at 25.00 MHz)"
    let fmax = log
        .lines()
        .filter(|l| l.contains("Max frequency for clock"))
        .filter_map(|l| l.split_once("': ").and_then(|(_, r)| r.split_whitespace().next()).and_then(|n| n.parse::<f64>().ok()))
        .next_back()
        .ok_or("nextpnr reported no Fmax")?;
    let a = &allocations[0];
    let mhz = a.clock_hz / 1e6;
    Ok(Bitstream {
        board: board.name.to_string(),
        device: a.part.clone(),
        clock_mhz: mhz,
        fmax_mhz: fmax,
        timing_met: fmax >= mhz,
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
