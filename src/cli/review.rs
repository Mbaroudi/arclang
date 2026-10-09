//! `arclang review`: ask a judgment model whether the model's traces are
//! plausible (see `crate::review`). Advice only; it never changes what
//! `arclang check` or the gate say.

use super::CliError;
use crate::compiler::elements;
use crate::review::typesafe::{self, TypeSafe};
use crate::review::{plan, review, Finding, Options, Plan};
use crate::{Compiler, CompilerConfig};
use std::path::Path;

fn lines(findings: &[Finding]) -> String {
    findings
        .iter()
        .map(|f| format!("  {:.2}  {} {} {}\n", f.plausibility, f.source, f.relation, f.target))
        .collect()
}

fn describe(plan: &Plan) -> String {
    let mut text = format!(
        "{} declared trace(s) and {} undeclared pair(s) to judge: {} question(s), one request each.\n",
        plan.traces.len(),
        plan.candidates.len(),
        plan.traces.len() + plan.candidates.len()
    );
    if plan.left_out > 0 {
        text.push_str(&format!("{} undeclared pair(s) left out by --max-questions.\n", plan.left_out));
    }
    text.push_str("Sent to api.typesafe.ai for each: the kind, name, id and text attributes of the two elements, and the trace's rationale.\n");
    text
}

pub fn run(input: &Path, options: Options, json: bool, dry_run: bool) -> Result<(), CliError> {
    let result = Compiler::new(CompilerConfig::default())
        .compile_file(input)
        .map_err(|e| CliError::Compilation(format!("{}: {e}", input.display())))?;
    let graph = elements::build(&result.ast, &result.semantic_model);
    let plan = plan(&graph, &options);

    if dry_run {
        print!("{}", describe(&plan));
        println!("Nothing was sent (--dry-run).");
        if let Some(first) = plan.traces.first().or(plan.candidates.first()) {
            let body = serde_json::to_string_pretty(&typesafe::request_body(first)).unwrap_or_default();
            println!("First request:\n{body}");
        }
        return Ok(());
    }
    if plan.traces.is_empty() && plan.candidates.is_empty() {
        println!("Nothing to review: the model declares no trace.");
        return Ok(());
    }

    let key = typesafe::find_key().map_err(CliError::Config)?;
    let mut judge = TypeSafe::new(key).map_err(CliError::Config)?;
    if !json {
        eprint!("{}", describe(&plan));
    }
    let outcome = review(&plan, &mut judge, &options).map_err(CliError::Compilation)?;

    if json {
        let text = serde_json::to_string_pretty(&outcome).map_err(|e| CliError::Compilation(e.to_string()))?;
        println!("{text}");
        return Ok(());
    }
    println!(
        "Judged by {}. These are probabilities, not verdicts: `arclang check` and the gate are unchanged.",
        outcome.model.as_deref().unwrap_or("the judgment model")
    );
    if outcome.doubtful_traces.is_empty() {
        println!("\nNo declared trace under a plausibility of {:.2}.", options.threshold);
    } else {
        println!("\nDeclared traces to look at (plausibility under {:.2}):", options.threshold);
        print!("{}", lines(&outcome.doubtful_traces));
    }
    if let Some(over) = options.suggest_over {
        if outcome.suggested_traces.is_empty() {
            println!("\nNo undeclared pair over a plausibility of {:.2}.", over);
        } else {
            println!("\nPairs that may be a missing trace (plausibility over {:.2}):", over);
            print!("{}", lines(&outcome.suggested_traces));
        }
    }
    Ok(())
}
