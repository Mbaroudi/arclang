//! `arclang realize`: climb the realization ladder of a model, requirement
//! by requirement, stage by stage. The plan is the ladder's: every stage
//! that is missing or stale, in order, each case once; each step runs the
//! command that produces the stage (`verify`, `sil`, `pil`, `hdl`, `soc`,
//! `twin`) as this very binary, then the model is compiled again and the
//! ladder read back. A failing stage stops the climb of its requirement:
//! the case is the thing to look at, not a command to run again. The gaps
//! (a requirement nothing climbs) are reported, never guessed at.

use super::CliError;
use crate::compiler::realization::{self, Ladder, Plan, Step};
use crate::{Compiler, CompilerConfig};
use std::path::Path;

pub struct Options {
    pub requirement: Option<String>,
    pub up_to: Option<String>,
    pub dry_run: bool,
    pub keep_going: bool,
    pub board: bool,
    pub json: bool,
}

fn ladders_of(input: &Path) -> Result<Vec<Ladder>, CliError> {
    let result = Compiler::new(CompilerConfig::default()).compile_file(input).map_err(|e| CliError::Compilation(e.to_string()))?;
    Ok(realization::ladders(&result, Some(input)))
}

fn check_stage(stage: &str) -> Result<(), CliError> {
    if realization::STAGES.contains(&stage) {
        Ok(())
    } else {
        Err(CliError::Config(format!("unknown stage '{stage}': one of {}", realization::STAGES.join(", "))))
    }
}

/// One step run as a subprocess of this binary; its output streams to the
/// terminal (captured with --json).
fn run_step(step: &Step, json: bool) -> Result<(bool, String), CliError> {
    let exe = std::env::current_exe().map_err(CliError::Io)?;
    let mut command = std::process::Command::new(exe);
    command.args(&step.args);
    if json {
        let output = command.output().map_err(CliError::Io)?;
        let text = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        Ok((output.status.success(), text))
    } else {
        let status = command.status().map_err(CliError::Io)?;
        Ok((status.success(), String::new()))
    }
}

fn shown(step: &Step) -> String {
    format!("arclang {}", step.args.join(" "))
}

pub fn run(input: &Path, options: Options) -> Result<(), CliError> {
    if let Some(stage) = &options.up_to {
        check_stage(stage)?;
    }
    let model = input.display().to_string();
    let ladders = ladders_of(input)?;
    if let Some(wanted) = &options.requirement {
        if !ladders.iter().any(|l| &l.requirement == wanted || &l.name == wanted) {
            return Err(CliError::Config(format!("no requirement '{wanted}' in {model}")));
        }
    }
    let plan = realization::plan(&ladders, &model, options.requirement.as_deref(), options.up_to.as_deref(), options.board);
    if options.dry_run {
        return report_plan(&ladders, &plan, options.json, &model);
    }
    if options.json {
        return run_json(input, &ladders, plan, &options);
    }
    print_plan(&ladders, &plan, &model);
    if plan.steps.is_empty() {
        return finish(&ladders_of(input)?, &plan, options.requirement.as_deref(), options.up_to.as_deref());
    }
    let mut stopped: Vec<String> = Vec::new();
    let mut ran = 0;
    for step in &plan.steps {
        // A requirement whose climb stopped at a failure climbs no further.
        if step.requirements.iter().all(|r| stopped.contains(r)) {
            continue;
        }
        println!("\n→ {} ({}): {}", realization::label(&step.stage), step.state, shown(step));
        let (ok, _) = run_step(step, false)?;
        ran += 1;
        let after = ladders_of(input)?;
        let case = step.args.last().cloned().unwrap_or_default();
        for requirement in &step.requirements {
            let Some(ladder) = after.iter().find(|l| &l.requirement == requirement) else { continue };
            let Some(stage) = ladder.stages.iter().find(|s| s.stage == step.stage) else { continue };
            // The step produced one case's verdict; the requirement's stage waits for its other cases.
            let state = stage.cases.iter().find(|c| c.id == case).map(|c| c.state.clone()).unwrap_or_else(|| stage.state.clone());
            println!("  {} {} ({case}): {} {}{}", requirement, realization::label(&step.stage), mark(&state), state, if stage.state == state { String::new() } else { format!(" · the stage is {}", stage.state) });
            if matches!(state.as_str(), "fail" | "error") && !options.keep_going {
                stopped.push(requirement.clone());
            } else if !matches!(state.as_str(), "pass") && !ok && !options.keep_going {
                stopped.push(requirement.clone());
            }
        }
        if !ok && !options.keep_going && step.requirements.iter().all(|r| stopped.contains(r)) {
            println!("  the climb of {} stops here", step.requirements.join(", "));
        }
    }
    println!("\n{ran} step(s) run.");
    finish(&ladders_of(input)?, &plan, options.requirement.as_deref(), options.up_to.as_deref())
}

/// Whether a stage is within the climb (`--up-to` included).
fn within(stage: &str, up_to: Option<&str>) -> bool {
    up_to.is_none_or(|top| realization::STAGES.iter().position(|s| *s == stage) <= realization::STAGES.iter().position(|s| *s == top))
}

fn mark(state: &str) -> &'static str {
    match state {
        "pass" => "✓",
        "fail" | "error" => "✗",
        "stale" => "~",
        "missing" => "○",
        _ => "–",
    }
}

fn print_plan(ladders: &[Ladder], plan: &Plan, model: &str) {
    println!("Realization of {model}");
    for ladder in ladders.iter().filter(|l| l.applies()) {
        println!("  {}", realization::line(ladder));
    }
    if plan.steps.is_empty() && plan.failing.is_empty() && plan.gaps.is_empty() {
        println!("Every stage that applies has fresh evidence: nothing to climb.");
    }
    if !plan.steps.is_empty() {
        println!("\nPlan ({} step(s)):", plan.steps.len());
        for step in &plan.steps {
            println!("  {} ({}) for {}: {}", realization::label(&step.stage), step.state, step.requirements.join(", "), shown(step));
        }
    }
    for (requirement, why) in &plan.failing {
        println!("\n✗ {requirement} fails at {why}: look at the case; the climb does not go past a failure");
    }
    for gap in &plan.gaps {
        println!("\n○ {} \"{}\": {}\n  {}", gap.requirement, gap.name, gap.what, gap.hint);
    }
}

fn report_plan(ladders: &[Ladder], plan: &Plan, json: bool, model: &str) -> Result<(), CliError> {
    if json {
        println!("{}", serde_json::json!({ "model": model, "ladders": ladders, "plan": plan }));
    } else {
        print_plan(ladders, plan, model);
    }
    Ok(())
}

fn run_json(input: &Path, ladders: &[Ladder], plan: Plan, options: &Options) -> Result<(), CliError> {
    let mut runs = Vec::new();
    let mut stopped: Vec<String> = Vec::new();
    for step in &plan.steps {
        if step.requirements.iter().all(|r| stopped.contains(r)) {
            continue;
        }
        let (ok, output) = run_step(step, true)?;
        let after = ladders_of(input)?;
        let case = step.args.last().cloned().unwrap_or_default();
        let states: Vec<serde_json::Value> = step
            .requirements
            .iter()
            .filter_map(|r| {
                let stage = after.iter().find(|l| &l.requirement == r).and_then(|l| l.stages.iter().find(|s| s.stage == step.stage))?;
                let state = stage.cases.iter().find(|c| c.id == case).map(|c| c.state.clone()).unwrap_or_else(|| stage.state.clone());
                Some(serde_json::json!({ "requirement": r, "case": case, "state": state, "stage_state": stage.state }))
            })
            .collect();
        for state in &states {
            let failed = matches!(state["state"].as_str(), Some("fail" | "error")) || (!ok && state["state"] != "pass");
            if failed && !options.keep_going {
                stopped.push(state["requirement"].as_str().unwrap_or_default().to_string());
            }
        }
        runs.push(serde_json::json!({ "step": step, "ok": ok, "output": output, "after": states }));
    }
    let after = ladders_of(input)?;
    let complete = after
        .iter()
        .filter(|l| options.requirement.as_deref().is_none_or(|o| l.requirement == o || l.name == o))
        .filter(|l| l.applies())
        .all(|l| l.stages.iter().filter(|s| within(&s.stage, options.up_to.as_deref())).all(|s| s.state == "pass" || s.state == "n/a"));
    println!("{}", serde_json::json!({ "model": input.display().to_string(), "before": ladders, "plan": plan, "runs": runs, "after": after, "complete": complete }));
    if complete {
        Ok(())
    } else {
        Err(CliError::Compilation("the realization is not complete".to_string()))
    }
}

/// The ladder after the climb; the command fails when a chosen requirement
/// has a stage that is not fresh evidence, or a gap.
fn finish(ladders: &[Ladder], plan: &Plan, only: Option<&str>, up_to: Option<&str>) -> Result<(), CliError> {
    println!("\nAfter the climb:");
    let chosen: Vec<&Ladder> = ladders.iter().filter(|l| only.is_none_or(|o| l.requirement == o || l.name == o)).collect();
    for ladder in chosen.iter().filter(|l| l.applies()) {
        println!("  {}", realization::line(ladder));
    }
    let pending = |s: &realization::Stage| within(&s.stage, up_to) && !matches!(s.state.as_str(), "pass" | "n/a");
    let open: Vec<String> = chosen
        .iter()
        .filter(|l| l.applies())
        .filter(|l| l.stages.iter().any(pending))
        .map(|l| format!("{} ({})", l.requirement, l.stages.iter().filter(|s| pending(s)).map(|s| format!("{} {}", realization::label(&s.stage), s.state)).collect::<Vec<_>>().join(", ")))
        .collect();
    if open.is_empty() && plan.gaps.is_empty() {
        match up_to {
            Some(top) => println!("Complete up to {}: every stage that applies, up to it, is fresh evidence.", realization::label(top)),
            None => println!("Complete: every stage that applies is fresh evidence."),
        }
        return Ok(());
    }
    if !open.is_empty() {
        println!("Not complete: {}", open.join("; "));
    }
    if !plan.gaps.is_empty() {
        println!("Gaps: {}", plan.gaps.iter().map(|g| g.requirement.as_str()).collect::<Vec<_>>().join(", "));
    }
    Err(CliError::Compilation("the realization is not complete".to_string()))
}
