//! `arclang schedule`: the io_bindings of a target as tasks, their response
//! times from the step times `arclang partition` measured, whether every
//! task meets its period.

use super::CliError;
use crate::compiler::embedded::{binding, plan};
use crate::compiler::schedule;
use std::path::Path;

pub struct Options {
    pub target: Option<String>,
    /// `MACHINE=MICROSECONDS`.
    pub wcet: Vec<String>,
    /// Run every task together on the target first (Renode), the step times under load.
    pub prove: bool,
    /// Cycles of the longest period the proof runs.
    pub cycles: usize,
    pub keep: Option<std::path::PathBuf>,
    pub json: bool,
}

fn given(wcet: &[String]) -> Result<Vec<(String, f64)>, CliError> {
    wcet.iter()
        .map(|text| {
            let (machine, us) = text.split_once('=').ok_or_else(|| CliError::Config(format!("--wcet '{text}' reads MACHINE=MICROSECONDS (--wcet \"Charger=25.5\")")))?;
            let us: f64 = us.trim().parse().ok().filter(|u: &f64| *u > 0.0).ok_or_else(|| CliError::Config(format!("--wcet '{text}': '{}' is not a time in microseconds", us.trim())))?;
            Ok((machine.trim().to_string(), us))
        })
        .collect()
}

pub fn run(input: &Path, options: Options) -> Result<(), CliError> {
    // The proof needs the target's toolchain and emulator: in the SoC image when the host has none.
    if options.prove {
        if let Some(code) = crate::compiler::embedded::docker::relay("pil", input).map_err(CliError::Config)? {
            std::process::exit(code);
        }
    }
    let mut result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(input).map_err(|e| CliError::Compilation(e.to_string()))?;
    let p = plan::plan(&result.ast).map_err(CliError::Compilation)?;
    let bindings = binding::resolve_all(&result.ast, &p).map_err(CliError::Compilation)?;
    if bindings.is_empty() {
        return Err(CliError::Config("the model has no io_binding: nothing runs as a task (docs/embedded-c.md, \"Tasks and scheduling\")".to_string()));
    }
    let given = given(&options.wcet)?;
    for (machine, _) in &given {
        if !p.machines.iter().any(|m| m.is(machine)) {
            return Err(CliError::Config(format!("--wcet: no state_machine '{machine}' in the model")));
        }
    }
    let targets: Vec<String> = {
        let mut t: Vec<String> = bindings.iter().map(|b| b.target.clone()).collect();
        t.sort();
        t.dedup();
        t
    };
    let target = match &options.target {
        Some(t) => t.clone(),
        None if targets.len() == 1 => targets[0].clone(),
        None => return Err(CliError::Config(format!("the io_bindings are deployed on several targets ({}): name one (--target)", targets.join(", ")))),
    };
    if !targets.contains(&target) {
        return Err(CliError::Config(format!("no io_binding is deployed on {target} (targets: {})", targets.join(", "))));
    }
    let mut proved: Vec<serde_json::Value> = Vec::new();
    if options.prove {
        use crate::compiler::embedded::tasks;
        let scratch = options.keep.is_none();
        let work = options.keep.clone().unwrap_or_else(|| std::env::temp_dir().join(format!("arclang-schedule-{}", std::process::id())));
        let runs = tasks::prove(&result.ast, &tasks::Options { work: work.clone(), target: Some(target.clone()), cycles: options.cycles.max(1) }).map_err(CliError::Compilation)?;
        if scratch {
            let _ = std::fs::remove_dir_all(&work);
        } else if !options.json {
            println!("  Application, build, plan and logs kept in {}", work.display());
        }
        let path = tasks::write(input, &runs).map_err(CliError::Config)?;
        if !options.json {
            println!("Tasks on {target} together, in Renode ({} cycle(s) of the longest period):", options.cycles.max(1));
            for r in &runs {
                println!("  {} task \"{}\" ({}) every {} us at priority {}: {}; a cycle took at most {:.1} us{}", if r.kept { "✓" } else { "✗" }, r.binding, r.machine, r.period_us, r.priority, r.reason, r.worst_us, r.case.as_ref().map(|c| format!(", the channels of {c} played")).unwrap_or_else(|| ", the channels quiet".to_string()));
            }
            println!("  kept in {path}\n", path = path.display());
        }
        proved = runs.iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect();
        result.schedule = runs.into_iter().map(|r| (r, true)).collect();
        if result.schedule.iter().any(|(r, _)| !r.kept) {
            let failed: Vec<String> = result.schedule.iter().filter(|(r, _)| !r.kept).map(|(r, _)| format!("'{}': {}", r.binding, r.reason)).collect();
            return Err(CliError::CiFailed(format!("a task did not keep its period on the target: {}", failed.join("; "))));
        }
    }
    let analysis = schedule::analyse(&p, &bindings, &target, &result.partition, &result.schedule, &given);
    if options.json {
        let mut value = serde_json::to_value(&analysis).unwrap_or_default();
        if options.prove {
            value["proved"] = serde_json::Value::Array(proved);
        }
        println!("{}", serde_json::to_string_pretty(&value).unwrap_or_default());
    } else {
        println!("Schedule {} — {} task(s) on {target}, fixed priorities, preemptive (response-time analysis)", input.display(), analysis.tasks.len());
        for line in schedule::lines(&analysis) {
            println!("  {line}");
        }
    }
    match analysis.schedulable {
        Some(true) => Ok(()),
        Some(false) => Err(CliError::CiFailed("a task misses its period".to_string())),
        None => Err(CliError::CiFailed("a task's step time is unknown: run arclang partition on the target, or give it with --wcet".to_string())),
    }
}
