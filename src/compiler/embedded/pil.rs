//! Processor in the loop, back to back: the generated C built for a target
//! by its toolchain, run on its processor (emulated: Renode for
//! `zephyr/litex_vexriscv`), driven step by step with the vectors of each
//! `execute:` case and compared with the model-level run as SIL compares
//! the host build.
//!
//! - A case with `binding:` runs the controller's loop (`pil_io.c`): the
//!   glue's cycle every period on the target's kernel timer. Between two
//!   cycles the bench sets the bound input channels on the emulated
//!   peripherals (gpio lines, ADC registers, CAN frames, UART bytes); after
//!   each cycle the target reports its outputs on the console and the bench
//!   reads the bound output channels (the gpio output register, the PWM
//!   duty, the bytes sent on each UART, the last CAN frame, the watchdog's
//!   feeds).
//! - A case without runs the step() API (`pil_direct.c`): the bench sends
//!   each step's inputs on the console, the target steps and reports.
//!
//! Steps are synchronised on the target's reports, not on time: the bench
//! runs the emulation in quanta of virtual time until the report of the
//! step arrives, then sets the next step's inputs. The emulator's virtual
//! time does not depend on the host, so a run is the same on any computer.

use super::binding::{Binding, Sink};
use super::plan::Plan;
use super::reference::Table;
use super::sil::{self, Outcome};
use super::target::Package;
use crate::compiler::ast::{Model, TestCase};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Options {
    /// The target package (default: the bindings').
    pub target: Option<String>,
    /// Where the applications, the builds and the plans go.
    pub work: PathBuf,
    /// Only this case (id or name).
    pub only: Option<String>,
}

/// Bumped when the meaning of a cached PIL verdict changes.
const CACHE_FORMAT: &str = "arclang-pil-cache-1";

/// Virtual time the target has to come up, and to report a step.
const BOOT_US: u64 = 5_000_000;
const STEP_US: u64 = 2_000_000;

/// The target of a run: the one given, else the first binding's that is not the host.
pub fn choose_target(bindings: &[Binding], target: Option<&str>) -> Result<Package, String> {
    let name = match target {
        Some(t) => t.to_string(),
        None => bindings
            .iter()
            .map(|b| b.target.clone())
            .find(|t| t != super::target::HOST)
            .ok_or("no target to run on: give one (--target zephyr/litex_vexriscv) or name it in the io_binding (target: \"zephyr/litex_vexriscv\")")?,
    };
    let package = super::target::find(&name)?;
    if package.runtime != "zephyr" || package.build.is_none() || package.emulator.is_none() {
        return Err(format!("target '{name}' has no build and emulator for processor in the loop (its target.json has no \"build\" or \"emulator\")"));
    }
    Ok(package)
}

/// The hash a cached PIL verdict depends on: what the SIL verdict depends
/// on (every generated file, the case), the target's name and every file
/// of its package.
pub fn cache_key(ast: &Model, test: &TestCase, package: &Package) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    let mut field = |label: &str, bytes: &[u8]| {
        hash.update(label.as_bytes());
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    };
    field("format", CACHE_FORMAT.as_bytes());
    field("sil", sil::cache_key(ast, test)?.as_bytes());
    field("target", package.name.as_bytes());
    for (name, text, _) in package.files()? {
        field("file", name.as_bytes());
        field("text", text.as_bytes());
    }
    Ok(hash.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// Keep the verdicts of the cases run, as `<case>.pil.json` in the model's
/// cache, where `check`, `gate` and the tables read them.
pub fn cache(ast: &Model, model_path: &Path, package: &Package, outcomes: &[Outcome]) -> Result<Vec<PathBuf>, String> {
    let dir = crate::compiler::simulation::cache::dir(model_path);
    let mut written = Vec::new();
    for test in sil::cases(ast) {
        let runs: Vec<&Outcome> = outcomes.iter().filter(|o| o.test_case == test.id).collect();
        if runs.is_empty() {
            continue;
        }
        let mut outcome = sil::cached_outcome(&runs, cache_key(ast, test, package)?);
        outcome.kind = "pil".to_string();
        outcome.target = package.name.clone();
        outcome.tool = format!("arclang pil ({})", runs[0].compiler);
        if outcome.verdict != "pass" {
            outcome.reason = format!("PIL on {}: {}", package.name, runs[0].reason);
        }
        written.push(crate::compiler::simulation::cache::write_kind(&dir, "pil", &outcome)?);
    }
    Ok(written)
}

fn run_command(command: &[String], cwd: Option<&Path>, env: &[(&str, &Path)], what: &str) -> Result<String, String> {
    let (program, args) = command.split_first().ok_or_else(|| format!("{what}: an empty command"))?;
    let mut c = Command::new(program);
    c.args(args);
    if let Some(dir) = cwd {
        c.current_dir(dir);
    }
    for (k, v) in env {
        c.env(k, v);
    }
    let out = c.output().map_err(|e| format!("{what}: cannot run '{program}': {e} (run in ArcLang's SoC image: tools/embedded/soc/run.sh)"))?;
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    if !out.status.success() {
        let tail: Vec<&str> = text.lines().rev().take(25).collect();
        return Err(format!("{what} failed ({}):\n{}", out.status, tail.into_iter().rev().collect::<Vec<_>>().join("\n")));
    }
    Ok(text)
}

fn substitute(words: &[String], values: &[(&str, String)]) -> Vec<String> {
    words
        .iter()
        .map(|w| values.iter().fold(w.clone(), |acc, (k, v)| acc.replace(&format!("{{{k}}}"), v)))
        .filter(|w| !w.is_empty())
        .collect()
}

fn write_app(dir: &Path, app: &super::zephyr::App) -> Result<(), String> {
    for (name, text, executable) in &app.files {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("cannot make {}: {e}", parent.display()))?;
        }
        std::fs::write(&path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        #[cfg(unix)]
        if *executable {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755));
        }
        #[cfg(not(unix))]
        let _ = executable;
    }
    Ok(())
}

/// An application directory and its image, or why it did not build.
type Built = Result<(PathBuf, PathBuf), String>;

/// The images built, by (binding, period).
struct Builds<'a> {
    package: &'a Package,
    work: PathBuf,
    images: HashMap<(Option<String>, u64), Built>,
}

impl Builds<'_> {
    /// The application directory and the image for a binding (or the step() API) and a period.
    fn get(&mut self, plan: &Plan, binding: Option<&Binding>, period_us: u64) -> Result<(PathBuf, PathBuf), String> {
        let key = (binding.map(|b| b.ident.clone()), period_us);
        if let Some(built) = self.images.get(&key) {
            return built.clone();
        }
        let built = self.build(plan, binding, period_us);
        self.images.insert(key, built.clone());
        built
    }

    fn build(&self, plan: &Plan, binding: Option<&Binding>, period_us: u64) -> Result<(PathBuf, PathBuf), String> {
        let name = binding.map(|b| b.ident.clone()).unwrap_or_else(|| "direct".to_string());
        let app_dir = self.work.join(format!("app-{name}"));
        let build_dir = self.work.join(format!("build-{name}-{period_us}us"));
        if !app_dir.join("CMakeLists.txt").is_file() {
            let app = super::zephyr::app(plan, binding, self.package)?;
            write_app(&app_dir, &app)?;
        }
        let build = self.package.build.as_ref().expect("checked");
        let (main, period_define) = match binding {
            Some(_) => ("pil_io".to_string(), String::new()),
            None => ("pil_direct".to_string(), format!("-DARCLANG_PERIOD_US={period_us}")),
        };
        let values = [
            ("app", app_dir.display().to_string()),
            ("build", build_dir.display().to_string()),
            ("main", main),
            ("period_define", period_define),
        ];
        run_command(&substitute(&build.command, &values), Some(&self.work), &[], &format!("building for {}", self.package.name))?;
        let image = PathBuf::from(substitute(std::slice::from_ref(&build.image), &values)[0].clone());
        if !image.is_file() {
            return Err(format!("the build for {} made no {}", self.package.name, image.display()));
        }
        Ok((app_dir, image))
    }
}

/// The bench's plan of a case.
fn bench_plan(package: &Package, app: &Path, image: &Path, binding: Option<&Binding>, steps: &[Vec<String>], result: &Path, period_us: u64) -> String {
    let emulator = package.emulator.as_ref().expect("checked");
    let mut out = String::new();
    let _ = writeln!(out, "mode {}", if binding.is_some() { "io" } else { "direct" });
    let _ = writeln!(out, "elf {}", image.display());
    let _ = writeln!(out, "platform {}", app.join(&emulator.platform).display());
    for p in &emulator.peripherals {
        let _ = writeln!(out, "peripheral {}", app.join(p).display());
    }
    for (kind, value) in &emulator.channels {
        match value {
            serde_json::Value::String(name) => {
                let _ = writeln!(out, "{kind} {name}");
            }
            serde_json::Value::Array(names) => {
                for (k, name) in names.iter().enumerate() {
                    if let Some(name) = name.as_str() {
                        let _ = writeln!(out, "{kind} {k} {name}");
                    }
                }
            }
            _ => {}
        }
    }
    if let Some(b) = binding {
        for o in &b.outputs {
            let spec = match o.sink {
                Sink::Gpio(p) => format!("gpio {p}"),
                Sink::Pwm(c, _) => format!("pwm {c}"),
                Sink::Dac(c, _) => format!("dac {c}"),
                Sink::Can(id) => format!("can 0x{id:X}"),
                Sink::Uart(p, byte) => format!("uart {p} 0x{byte:02X}"),
            };
            let _ = writeln!(out, "output {spec}");
        }
        if b.watchdog {
            let _ = writeln!(out, "output watchdog");
        }
    }
    // Through the I/O, the inputs of a step go in within a twentieth of the
    // period after the report of the step before; the step() API has no period.
    let quantum = if binding.is_some() { (period_us / 20).clamp(1, 100) } else { 100 };
    let _ = writeln!(out, "quantum_us {quantum}\nboot_us {BOOT_US}\nstep_us {}\nresult {}", STEP_US.max(period_us * 4), result.display());
    for step in steps {
        out.push_str("step\n");
        for op in step {
            out.push_str(op);
            out.push('\n');
        }
        out.push_str(".\n");
    }
    out
}

/// A row of the target's report: `R step,time,status,values...|io values`.
struct Row {
    time: u64,
    values: Vec<f64>,
}

fn parse_result(text: &str, width: usize, io: usize, steps: usize) -> Result<Vec<Row>, String> {
    let mut rows = Vec::new();
    for line in text.lines() {
        if let Some(error) = line.strip_prefix("error ") {
            return Err(format!("on the target: {error}"));
        }
        let Some(report) = line.strip_prefix("R ") else { continue };
        let (target, channels) = report.split_once('|').unwrap_or((report, ""));
        let fields: Vec<&str> = target.split(',').collect();
        if fields.len() != 3 + width {
            return Err(format!("the target reported {} values for {width} outputs: '{line}'", fields.len().saturating_sub(3)));
        }
        let step: usize = fields[0].parse().map_err(|_| format!("a report without its step: '{line}'"))?;
        if step != rows.len() {
            return Err(format!("the target reported step {step} where step {} was expected", rows.len()));
        }
        let time: u64 = fields[1].parse().map_err(|_| format!("a report without its time: '{line}'"))?;
        let status: i64 = fields[2].parse().map_err(|_| format!("a report without its status: '{line}'"))?;
        if status != 0 {
            return Err(format!("the cycle failed at step {step} on the target (status {status})"));
        }
        let mut values = fields[3..].iter().map(|t| super::zephyr::decode(t)).collect::<Result<Vec<f64>, String>>()?;
        let channel_values: Vec<f64> = channels.split(',').filter(|v| !v.is_empty()).map(|v| v.trim().parse::<f64>().map_err(|_| format!("a channel value '{v}' is no number"))).collect::<Result<_, _>>()?;
        if channel_values.len() != io {
            return Err(format!("the bench read {} output channels for {io}", channel_values.len()));
        }
        values.extend(channel_values);
        rows.push(Row { time, values });
    }
    if rows.len() != steps {
        return Err(format!("the target reported {} of {steps} steps", rows.len()));
    }
    Ok(rows)
}

fn timing(rows: &[Row], binding: Option<&Binding>, period_us: u64) -> String {
    match binding {
        Some(_) => {
            let gaps: Vec<u64> = rows.windows(2).map(|w| w[1].time.saturating_sub(w[0].time)).collect();
            let (lo, hi) = (gaps.iter().min().copied().unwrap_or(period_us), gaps.iter().max().copied().unwrap_or(period_us));
            if lo == period_us && hi == period_us {
                format!("{} cycles, one every {period_us} us of the target's virtual time (the first at {} us)", rows.len(), rows.first().map(|r| r.time).unwrap_or(0))
            } else {
                format!("{} cycles, {lo} to {hi} us apart for a period of {period_us} us", rows.len())
            }
        }
        None => {
            let worst = rows.iter().map(|r| r.time).max().unwrap_or(0);
            format!("step() took at most {:.1} us of the target's virtual time", worst as f64 / 1000.0)
        }
    }
}

/// What ran: the target, its compiler and its emulator, as the outcome names it.
fn implementation(package: &Package, work: &Path) -> String {
    let mut parts = vec![package.name.clone()];
    if let Some(toolchain) = &package.toolchain {
        let gcc = std::fs::read_dir(work)
            .ok()
            .and_then(|entries| entries.flatten().map(|e| e.path().join("CMakeCache.txt")).find(|p| p.is_file()))
            .and_then(|cache| std::fs::read_to_string(cache).ok())
            .and_then(|text| text.lines().find_map(|l| l.strip_prefix("CMAKE_C_COMPILER:").and_then(|rest| rest.split_once('=')).map(|(_, path)| path.to_string())));
        let version = gcc.and_then(|g| sil::compiler_version(&g));
        parts.push(version.unwrap_or_else(|| toolchain.clone()));
    }
    if package.runtime == "zephyr" {
        let version = std::env::var_os("ZEPHYR_BASE").and_then(|base| std::fs::read_to_string(Path::new(&base).join("VERSION")).ok()).map(|text| {
            let field = |key: &str| text.lines().find_map(|l| l.split_once('=').filter(|(k, _)| k.trim() == key).map(|(_, v)| v.trim().to_string())).unwrap_or_default();
            format!("Zephyr {}.{}.{}", field("VERSION_MAJOR"), field("VERSION_MINOR"), field("PATCHLEVEL"))
        });
        parts.extend(version);
    }
    if let Some(emulator) = &package.emulator {
        let version = Command::new(&emulator.command[0]).arg("--version").output().ok().map(|o| String::from_utf8_lossy(&o.stdout).lines().next().unwrap_or("").trim().to_string());
        parts.push(version.filter(|v| !v.is_empty()).unwrap_or_else(|| emulator.kind.clone()));
    }
    parts.join("; ")
}

fn run_case(test: &TestCase, plan: &Plan, bindings: &[Binding], builds: &mut Builds, label: &str) -> Outcome {
    let mut prepared = match sil::prepare(test, plan, bindings, label) {
        Ok(p) => p,
        Err(outcome) => return *outcome,
    };
    let fail = |mut prepared: sil::Prepared, reason: String| {
        prepared.outcome.reason = reason;
        prepared.outcome
    };
    let binding = prepared.binding;
    if let Some(b) = binding {
        if let Err(e) = super::target::check(b, builds.package) {
            return fail(prepared, e);
        }
    }
    let steps: Vec<Vec<String>> = match binding {
        Some(b) => match sil::channel_operations(&prepared.case, plan, b, &prepared.vectors) {
            Ok(ops) => ops,
            Err(e) => return fail(prepared, e),
        },
        None => (0..prepared.case.steps).map(|k| vec![format!("line {}", super::zephyr::direct_line(plan, &prepared.vectors, k))]).collect(),
    };
    let period = prepared.case.step_us;
    let (app, image) = match builds.get(plan, binding, period) {
        Ok(built) => built,
        Err(e) => return fail(prepared, e),
    };
    let dir = builds.work.join(format!("case-{}", prepared.outcome.test_case.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect::<String>()));
    if let Err(e) = std::fs::create_dir_all(&dir) {
        return fail(prepared, format!("cannot make {}: {e}", dir.display()));
    }
    let result = dir.join("result.txt");
    let plan_path = dir.join("plan.txt");
    let _ = std::fs::remove_file(&result);
    if let Err(e) = std::fs::write(&plan_path, bench_plan(builds.package, &app, &image, binding, &steps, &result, period)) {
        return fail(prepared, format!("cannot write {}: {e}", plan_path.display()));
    }
    let emulator = builds.package.emulator.as_ref().expect("checked");
    let mut command = substitute(&emulator.command, &[("app", app.display().to_string())]);
    // The bench writes its result; the emulator ends with it.
    if let Some(last) = command.last_mut() {
        last.push_str("; quit");
    }
    let log = match run_command(&command, Some(&dir), &[("ARCLANG_PIL_PLAN", &plan_path)], &format!("running on {}", builds.package.name)) {
        Ok(log) => log,
        Err(e) => return fail(prepared, e),
    };
    let _ = std::fs::write(dir.join("emulator.log"), &log);
    let text = std::fs::read_to_string(&result).unwrap_or_default();
    let width = prepared.model.names.len() - binding.map(|b| sil::io_columns(b).len()).unwrap_or(0);
    let io = prepared.model.names.len() - width;
    let rows = match parse_result(&text, width, io, prepared.case.steps) {
        Ok(rows) => rows,
        Err(e) => {
            let reason = if text.is_empty() { format!("the bench wrote no result; the emulator said:\n{}", log.lines().rev().take(15).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n")) } else { e };
            return fail(prepared, reason);
        }
    };
    prepared.outcome.timing = Some(timing(&rows, binding, period));
    let table = Table { names: prepared.model.names.clone(), period_us: period, rows: rows.into_iter().map(|r| r.values).collect(), units: BTreeMap::new() };
    sil::conclude(prepared, table)
}

/// Run every executed case of a compiled model (or `options.only`) on the target.
pub fn run(ast: &Model, options: &Options) -> Result<(Package, Vec<Outcome>), String> {
    let plan = super::plan::plan(ast)?;
    let bindings = super::binding::resolve_all(ast, &plan)?;
    let package = choose_target(&bindings, options.target.as_deref())?;
    let selected: Vec<&TestCase> = sil::cases(ast).into_iter().filter(|t| options.only.as_deref().is_none_or(|o| t.id == o || t.name == o)).collect();
    if selected.is_empty() {
        return Err(match &options.only {
            Some(only) => format!("no test_case '{only}' that executes (execute: {{ ... }})"),
            None => "the model has no test_case that executes (execute: { machine: ... stop_time: ... inputs: [...] })".to_string(),
        });
    }
    std::fs::create_dir_all(&options.work).map_err(|e| format!("cannot make {}: {e}", options.work.display()))?;
    let work = options.work.canonicalize().unwrap_or_else(|_| options.work.clone());
    let mut builds = Builds { package: &package, work: work.clone(), images: HashMap::new() };
    // Build first, so the outcomes name the compiler that built them.
    for test in &selected {
        if let Some(Ok(case)) = super::case::read(test, &plan, &bindings) {
            let binding = case.binding.as_ref().and_then(|r| bindings.iter().find(|b| b.is(r)));
            if binding.is_none_or(|b| super::target::check(b, &package).is_ok()) {
                let _ = builds.get(&plan, binding, case.step_us);
            }
        }
    }
    let label = implementation(&package, &work);
    let outcomes = selected.into_iter().map(|t| run_case(t, &plan, &bindings, &mut builds, &label)).collect();
    Ok((package, outcomes))
}
