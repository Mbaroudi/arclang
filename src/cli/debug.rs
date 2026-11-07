//! `arclang debug`: run a state machine, a scenario or a functional chain
//! of a model step by step.
//!
//! In the terminal, `arclang debug model.arc --events plug,ready` runs the
//! first state machine (or `--machine`, `--scenario`, `--chain`) and prints
//! each step; a scenario sends its messages and a chain executes its steps
//! to the end. With `--dap` it speaks the Debug Adapter Protocol on
//! stdin/stdout, as VS Code runs a debugger: a breakpoint on a line stops
//! when a step reaches it (a state entered, a transition fired, a message
//! sent, a chain step executed); the call stack shows where the run is; the
//! guards and conditions are set by the reader; *Step Over* is one step;
//! events are fired from the debug console (`fire <event>`, or the event).

use crate::compiler::execution::dap::Adapter;
use crate::compiler::execution::debug_target::{self, load, Outcome};
use super::CliError;
use serde_json::{json, Value};
use std::path::PathBuf;

#[derive(clap::Args, Debug)]
pub struct DebugArgs {
    /// The model (not needed with --dap: the launch request names it)
    pub input: Option<PathBuf>,
    /// The state machine to run, by name or id
    #[clap(long)]
    pub machine: Option<String>,
    /// The scenario to play, by name or id
    #[clap(long)]
    pub scenario: Option<String>,
    /// The functional chain (or operational process) to run, by name or id
    #[clap(long)]
    pub chain: Option<String>,
    /// Events to fire in order, comma-separated (`-` for a completion step)
    #[clap(long, value_delimiter = ',')]
    pub events: Vec<String>,
    /// Guards and conditions that hold, comma-separated, as written in the model
    #[clap(long = "set", value_delimiter = ',')]
    pub conditions: Vec<String>,
    /// List what the model can run, as JSON
    #[clap(long)]
    pub list: bool,
    /// Print the steps as JSON
    #[clap(long)]
    pub json: bool,
    /// Speak the Debug Adapter Protocol on stdin/stdout (VS Code)
    #[clap(long)]
    pub dap: bool,
}

/// Steps a scenario or a chain runs at most from the terminal.
const TERMINAL_STEPS: usize = 1000;

pub fn run(args: &DebugArgs) -> Result<(), CliError> {
    if args.dap {
        return Adapter::default().serve(std::io::stdin().lock(), std::io::stdout()).map_err(CliError::Io);
    }
    let input = args.input.as_ref().ok_or_else(|| CliError::Config("name the model: arclang debug model.arc --events ...".to_string()))?;
    if args.list {
        // What there is to run, possibly nothing: a model without a machine still compiles.
        let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(input).map_err(|e| CliError::Compilation(e.to_string()))?;
        println!("{}", serde_json::to_string_pretty(&debug_target::runnables(&result.ast)).unwrap_or_default());
        return Ok(());
    }
    let (mut target, _) = load(input, args.machine.as_deref(), args.scenario.as_deref(), args.chain.as_deref()).map_err(CliError::Compilation)?;
    for condition in &args.conditions {
        target.set_condition(condition, true);
    }
    let mut text = format!("{} {}\n", target.kind(), target.name());
    for note in target.notes() {
        text.push_str(&format!("note: {note}\n"));
    }
    let mut steps: Vec<Value> = Vec::new();
    let mut record = |outcome: &Outcome, text: &mut String| {
        text.push_str(&outcome.text);
        steps.push(json!({ "text": outcome.text, "lines": outcome.lines, "actions": outcome.actions, "idle": outcome.idle, "done": outcome.done }));
    };
    let started = target.start();
    record(&started, &mut text);
    if matches!(target.run, debug_target::Run::Machine(_)) {
        for event in &args.events {
            let event = event.trim();
            let outcome = target.step(if event == "-" { None } else { Some(event) });
            record(&outcome, &mut text);
        }
    } else {
        for _ in 0..TERMINAL_STEPS {
            let outcome = target.step(None);
            let stop = outcome.done || outcome.idle.is_some();
            record(&outcome, &mut text);
            if stop {
                break;
            }
        }
    }
    if args.json {
        println!("{}", serde_json::to_string_pretty(&json!({ "kind": target.kind(), "name": target.name(), "steps": steps })).unwrap_or_default());
    } else {
        print!("{text}");
    }
    Ok(())
}
