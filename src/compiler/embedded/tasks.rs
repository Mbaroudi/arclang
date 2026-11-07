//! Every task together on the target (`arclang schedule --prove`): the
//! application of all the io_bindings of a target (a thread each, as the
//! firmware runs them) under the processor-in-the-loop bench in Renode,
//! each binding's channels driven by a case of its machine on the
//! binding's own period grid, all at once; every cycle of every task
//! reported with its start and its end. Read back: each task's cycles
//! came one a period (its period kept under the load of the others), and
//! the longest a cycle took — the step time under load, what the
//! response-time analysis then runs on. Nothing is real time: Renode's
//! virtual time, the same image and the same plan give the same cycles on
//! any host.

use super::binding::Binding;
use super::plan::Plan;
use super::target::Package;
use super::{pil, sil};
use crate::compiler::ast::Model;
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// One task's cycles under the bench.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskRun {
    pub binding: String,
    pub machine: String,
    pub target: String,
    pub period_us: u64,
    pub priority: u32,
    /// The case whose channels the bench played, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case: Option<String>,
    pub cycles: usize,
    pub expected: usize,
    /// Cycles of the longest period the bench was asked to run (`--cycles`).
    pub cycles_asked: usize,
    /// Between two cycles' starts, microseconds of the target's virtual time.
    pub gap_min_us: u64,
    pub gap_max_us: u64,
    /// The longest a cycle took (start to end).
    pub worst_us: f64,
    /// Cycles whose status was not OK.
    pub failed: usize,
    /// Every expected cycle came, one a period (within a twentieth of it), none failed.
    pub kept: bool,
    pub reason: String,
    /// What the run depends on: the application's files, the plan.
    pub key: String,
}

pub struct Options {
    pub work: PathBuf,
    pub target: Option<String>,
    /// Cycles to run of the longest period (the others run accordingly).
    pub cycles: usize,
}

const CACHE_FORMAT: &str = "arclang-schedule-cache-1";

/// `.arclang/sim-cache/schedule.json` beside the model.
pub fn cache_path(model_path: &Path) -> PathBuf {
    crate::compiler::simulation::cache::dir(model_path).join("schedule.json")
}

pub fn read(model_path: &Path) -> Vec<TaskRun> {
    std::fs::read_to_string(cache_path(model_path)).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

pub fn write(model_path: &Path, runs: &[TaskRun]) -> Result<PathBuf, String> {
    let path = cache_path(model_path);
    std::fs::create_dir_all(path.parent().expect("a parent")).map_err(|e| format!("cannot make {}: {e}", path.parent().expect("a parent").display()))?;
    std::fs::write(&path, serde_json::to_string_pretty(runs).map_err(|e| e.to_string())?).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(path)
}

/// What a run depends on: the application's generated files and the plan.
pub fn key(app: &super::zephyr::App, plan_text: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    hash.update(CACHE_FORMAT.as_bytes());
    for (name, text, _) in &app.files {
        hash.update(name.as_bytes());
        hash.update((text.len() as u64).to_le_bytes());
        hash.update(text.as_bytes());
    }
    hash.update(plan_text.as_bytes());
    hash.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// The cached runs that still hold for the model, by binding.
pub fn fresh(ast: &Model, model_path: &Path) -> Vec<(TaskRun, bool)> {
    let runs = read(model_path);
    if runs.is_empty() {
        return Vec::new();
    }
    let current = (|| -> Option<String> {
        let plan = super::plan::plan(ast).ok()?;
        let bindings = super::binding::resolve_all(ast, &plan).ok()?;
        let target = runs[0].target.clone();
        let chosen: Vec<&Binding> = bindings.iter().filter(|b| b.target == target).collect();
        let package = super::target::find(&target).ok()?;
        let app = super::zephyr::app_of(&plan, &chosen, &package).ok()?;
        let plan_text = plan_text(&plan, &chosen, &bindings, &package, Path::new("/app"), Path::new("/image"), Path::new("/result"), ast, runs[0].cycles_asked.max(1)).ok()?;
        Some(key(&app, &plan_text))
    })();
    runs.into_iter().map(|r| {
        let is_fresh = current.as_deref() == Some(r.key.as_str());
        (r, is_fresh)
    }).collect()
}

/// The case the bench plays on a binding: the first executed case of its
/// machine whose stimuli all go through the binding's channels, if any.
fn case_of<'a>(ast: &'a Model, plan: &Plan, binding: &Binding, bindings: &[Binding]) -> Option<(&'a crate::compiler::ast::TestCase, Vec<Vec<String>>)> {
    for test in sil::cases(ast) {
        let Some(Ok(case)) = super::case::read(test, plan, bindings) else { continue };
        if case.machine != binding.machine || case.plant.is_some() {
            continue;
        }
        if let Some(reference) = &case.binding {
            if !binding.is(reference) {
                continue;
            }
        }
        let vectors = case.vectors(plan);
        if let Ok(ops) = sil::channel_operations(&case, plan, binding, &vectors) {
            return Some((test, ops));
        }
    }
    None
}

/// The plan of the bench: the tasks, their operations dated on their own
/// period grids from the go, how long to run.
#[allow(clippy::too_many_arguments)]
fn plan_text(plan: &Plan, chosen: &[&Binding], bindings: &[Binding], package: &Package, app: &Path, image: &Path, result: &Path, ast: &Model, cycles: usize) -> Result<String, String> {
    let mut out = pil::plan_head(package, app, image, "tasks");
    let longest = chosen.iter().map(|b| b.period_us).max().unwrap_or(10_000);
    let shortest = chosen.iter().map(|b| b.period_us).min().unwrap_or(10_000);
    let horizon = longest * cycles as u64;
    let quantum = (shortest / 20).clamp(1, 100);
    // The bench runs the horizon and one short period past it, so the last cycles on the horizon are reported.
    let _ = writeln!(out, "quantum_us {quantum}\nboot_us {}\nstep_us {}\nuntil {}\nresult {}", pil::BOOT_MICROS, horizon.max(100_000) * 2, horizon + shortest, result.display());
    for (k, b) in chosen.iter().enumerate() {
        let steps = (horizon / b.period_us) as usize;
        let played = case_of(ast, plan, b, bindings);
        let _ = writeln!(out, "task {k} {} {steps} {}", b.period_us, played.as_ref().map(|(t, _)| t.id.as_str()).unwrap_or("-"));
        if let Some((_, ops)) = played {
            // Step j's inputs go in half a period before its cycle (the first cycle a period after the go).
            for (j, step) in ops.iter().enumerate().take(steps) {
                let at = j as u64 * b.period_us + b.period_us / 2;
                for op in step {
                    let _ = writeln!(out, "at {at} {op}");
                }
            }
        }
    }
    Ok(out)
}

/// Parse the bench's result: per task, its cycles (step, start, end, status).
fn parse(text: &str) -> Result<(u64, Vec<Vec<(u32, u64, u64, i64)>>), String> {
    let mut go = 0;
    let mut tasks: Vec<Vec<(u32, u64, u64, i64)>> = Vec::new();
    for line in text.lines() {
        if let Some(error) = line.strip_prefix("error ") {
            return Err(format!("on the target: {error}"));
        }
        if let Some(t) = line.strip_prefix("go ") {
            go = t.trim().parse().unwrap_or(0);
        }
        let Some(rest) = line.strip_prefix("T ") else { continue };
        let fields: Vec<&str> = rest.split(',').map(str::trim).collect();
        if fields.len() != 5 {
            return Err(format!("a cycle report without its five fields: '{line}'"));
        }
        let task: usize = fields[0].parse().map_err(|_| format!("a cycle report without its task: '{line}'"))?;
        let step: u32 = fields[1].parse().map_err(|_| format!("a cycle report without its step: '{line}'"))?;
        let start: u64 = fields[2].parse().map_err(|_| format!("a cycle report without its start: '{line}'"))?;
        let end: u64 = fields[3].parse().map_err(|_| format!("a cycle report without its end: '{line}'"))?;
        let status: i64 = fields[4].parse().map_err(|_| format!("a cycle report without its status: '{line}'"))?;
        while tasks.len() <= task {
            tasks.push(Vec::new());
        }
        tasks[task].push((step, start, end, status));
    }
    Ok((go, tasks))
}

/// A task's cycles judged: came one a period, none failed.
fn judge(b: &Binding, machine: &str, priority: u32, case: Option<String>, expected: usize, cycles_asked: usize, cycles: &[(u32, u64, u64, i64)], key: &str) -> TaskRun {
    let mut sorted: Vec<&(u32, u64, u64, i64)> = cycles.iter().collect();
    sorted.sort_by_key(|c| c.1);
    let gaps: Vec<u64> = sorted.windows(2).map(|w| w[1].1.saturating_sub(w[0].1)).collect();
    let gap_min_us = gaps.iter().min().copied().unwrap_or(0);
    let gap_max_us = gaps.iter().max().copied().unwrap_or(0);
    let worst_us = sorted.iter().map(|c| c.2.saturating_sub(c.1) as f64).fold(0.0, f64::max);
    let failed = sorted.iter().filter(|c| c.3 != 0).count();
    let tolerance = b.period_us / 20;
    let late = gaps.iter().any(|g| *g > b.period_us + tolerance);
    let early = gaps.iter().any(|g| *g + tolerance < b.period_us);
    let (kept, reason) = if sorted.len() < expected {
        (false, format!("{} of {expected} cycles came", sorted.len()))
    } else if failed > 0 {
        (false, format!("{failed} cycle(s) failed on the target"))
    } else if late {
        (false, format!("a period slipped: {gap_max_us} us between two cycles for a period of {} us", b.period_us))
    } else if early {
        (false, format!("two cycles {gap_min_us} us apart for a period of {} us", b.period_us))
    } else {
        (true, format!("{} cycles, one every {} us ({gap_min_us} to {gap_max_us})", sorted.len(), b.period_us))
    };
    TaskRun { binding: b.name.clone(), machine: machine.to_string(), target: b.target.clone(), period_us: b.period_us, priority, case, cycles: sorted.len(), expected, cycles_asked, gap_min_us, gap_max_us, worst_us, failed, kept, reason, key: key.to_string() }
}

/// Run every io_binding of the target together under the bench: a run per task.
pub fn prove(ast: &Model, options: &Options) -> Result<Vec<TaskRun>, String> {
    let plan = super::plan::plan(ast)?;
    let bindings = super::binding::resolve_all(ast, &plan)?;
    let targets: Vec<String> = {
        let mut t: Vec<String> = bindings.iter().map(|b| b.target.clone()).collect();
        t.sort();
        t.dedup();
        t
    };
    let target = match &options.target {
        Some(t) => t.clone(),
        None if targets.len() == 1 => targets[0].clone(),
        None if targets.is_empty() => return Err("the model has no io_binding: nothing runs as a task".to_string()),
        None => return Err(format!("the io_bindings are deployed on several targets ({}): name one (--target)", targets.join(", "))),
    };
    let chosen: Vec<&Binding> = bindings.iter().filter(|b| b.target == target).collect();
    if chosen.len() < 2 {
        return Err(format!("{} io_binding is deployed on {target}: the tasks are proved together from two (one binding runs as the controller's loop, arclang pil proves it)", chosen.len()));
    }
    let package = super::target::find(&target)?;
    if package.emulator.is_none() {
        return Err(format!("target {target} has no emulator: the tasks cannot be run"));
    }
    for b in &chosen {
        super::target::check(b, &package)?;
    }
    let owned: Vec<Binding> = chosen.iter().map(|b| (*b).clone()).collect();
    let priorities = super::binding::priorities(&owned);
    std::fs::create_dir_all(&options.work).map_err(|e| format!("cannot make {}: {e}", options.work.display()))?;
    let work = options.work.canonicalize().unwrap_or_else(|_| options.work.clone());
    let app = super::zephyr::app_of(&plan, &chosen, &package)?;
    let app_dir = work.join("app-tasks");
    if !app_dir.join("CMakeLists.txt").is_file() {
        pil::write_app(&app_dir, &app)?;
    }
    let image = pil::build_app(&package, &work, &app_dir, &work.join("build-tasks"), "pil_tasks", "")?;
    let result = work.join("result.txt");
    let _ = std::fs::remove_file(&result);
    let text = plan_text(&plan, &chosen, &bindings, &package, &app_dir, &image, &result, ast, options.cycles.max(1))?;
    let key = key(&app, &plan_text(&plan, &chosen, &bindings, &package, Path::new("/app"), Path::new("/image"), Path::new("/result"), ast, options.cycles.max(1))?);
    let plan_path = work.join("plan.txt");
    std::fs::write(&plan_path, &text).map_err(|e| format!("cannot write {}: {e}", plan_path.display()))?;
    let log = pil::run_command(&pil::emulator_command(&package, &app_dir), Some(&work), &[("ARCLANG_PIL_PLAN", &plan_path)], &format!("running the tasks on {}", package.name))?;
    let _ = std::fs::write(work.join("emulator.log"), &log);
    let out = std::fs::read_to_string(&result).unwrap_or_default();
    if out.trim().is_empty() {
        return Err(format!("the bench wrote no result; the emulator said:\n{}", log.lines().rev().take(15).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n")));
    }
    let (_, cycles) = parse(&out)?;
    let longest = chosen.iter().map(|b| b.period_us).max().unwrap_or(10_000);
    let horizon = longest * options.cycles.max(1) as u64;
    Ok(chosen
        .iter()
        .zip(&priorities)
        .enumerate()
        .map(|(k, (b, p))| {
            let expected = (horizon / b.period_us) as usize;
            let case = case_of(ast, &plan, b, &bindings).map(|(t, _)| t.id.clone());
            judge(b, &plan.machines[b.machine].name, *p, case, expected, options.cycles.max(1), cycles.get(k).map(Vec::as_slice).unwrap_or(&[]), &key)
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding(period_us: u64) -> Binding {
        Binding { name: "B".into(), id: "B".into(), ident: "m_b".into(), machine: 0, period_us, watchdog: false, priority: None, target: "zephyr/litex_vexriscv".into(), inputs: vec![], outputs: vec![], warnings: vec![] }
    }

    #[test]
    fn a_task_keeps_its_period_when_every_cycle_comes_a_period_apart() {
        let b = binding(20_000);
        let cycles: Vec<(u32, u64, u64, i64)> = (0..5).map(|k| (k, 20_000 * (k as u64 + 1), 20_000 * (k as u64 + 1) + 30, 0)).collect();
        let run = judge(&b, "M", 1, None, 5, 5, &cycles, "k");
        assert!(run.kept, "{}", run.reason);
        assert_eq!((run.cycles, run.gap_min_us, run.gap_max_us, run.worst_us), (5, 20_000, 20_000, 30.0));
        let slipped: Vec<(u32, u64, u64, i64)> = vec![(0, 20_000, 20_030, 0), (1, 40_000, 40_030, 0), (2, 65_000, 65_030, 0)];
        let run = judge(&b, "M", 1, None, 3, 3, &slipped, "k");
        assert!(!run.kept && run.reason.contains("slipped: 25000 us"), "{}", run.reason);
        let short: Vec<(u32, u64, u64, i64)> = vec![(0, 20_000, 20_030, 0), (1, 40_000, 40_030, 0)];
        let run = judge(&b, "M", 1, None, 3, 3, &short, "k");
        assert!(!run.kept && run.reason == "2 of 3 cycles came", "{}", run.reason);
        let failing: Vec<(u32, u64, u64, i64)> = vec![(0, 20_000, 20_030, 0), (1, 40_000, 40_030, 3)];
        assert!(judge(&b, "M", 1, None, 2, 2, &failing, "k").reason.contains("1 cycle(s) failed"));
    }

    #[test]
    fn the_result_is_read_per_task() {
        let (go, tasks) = parse("ready 5000\ngo 5100\nT 0,0,25100,25130,0\nT 1,0,105100,105140,0\nT 0,1,45100,45131,0\n").unwrap();
        assert_eq!(go, 5100);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0], vec![(0, 25100, 25130, 0), (1, 45100, 45131, 0)]);
        assert!(parse("error on the target: the I/O did not come up").unwrap_err().contains("I/O did not come up"));
        assert!(parse("T 0,1,2\n").unwrap_err().contains("five fields"));
    }
}
