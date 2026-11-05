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
    let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(input).map_err(|e| CliError::Compilation(e.to_string()))?;
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
    let analysis = schedule::analyse(&p, &bindings, &target, &result.partition, &given);
    if options.json {
        println!("{}", serde_json::to_string_pretty(&analysis).unwrap_or_default());
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
