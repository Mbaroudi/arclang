//! The realization ladder: for each requirement, what each stage of its
//! realization says — the simulation of the architecture, the generated C
//! (SIL), the C on the target's processor (PIL), the machines in logic
//! (HDL), the SoC in simulation, the board, the twin on measurements — from
//! the verdicts the runs cached beside the model, as fresh or stale as the
//! model left them. Nothing runs here: `check`, the tables and the
//! workbench read the ladder; the commands the rungs name produce the
//! evidence.

use super::ast::{Model, TestCase};
use super::simulation::{is_simulated, Rung, Status};
use super::CompilationResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// The stages, in the order the realization climbs them.
pub const STAGES: [&str; 7] = ["simulation", "sil", "pil", "hdl", "soc", "board", "twin"];

/// How a stage is called in a sentence.
pub fn label(stage: &str) -> &'static str {
    match stage {
        "simulation" => "simulation",
        "sil" => "SIL",
        "pil" => "PIL",
        "hdl" => "HDL",
        "soc" => "SoC",
        "board" => "board",
        _ => "twin",
    }
}

/// The command that produces a stage's evidence.
fn command(stage: &str) -> &'static str {
    match stage {
        "simulation" => "verify",
        "sil" => "sil",
        "pil" => "pil",
        "hdl" => "hdl",
        "soc" => "soc",
        "board" => "soc --port <serial port>",
        _ => "twin",
    }
}

/// One stage of a requirement's ladder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stage {
    pub stage: String,
    /// `pass`, `fail`, `error`, `stale`, `missing` or `n/a` (no case of the
    /// requirement reaches this stage).
    pub state: String,
    /// The cases (or twins) the state comes from, with the state of each.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cases: Vec<StageCase>,
    /// What ran it, when one thing did.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StageCase {
    pub id: String,
    pub state: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub reason: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub evidence: String,
}

/// A requirement's ladder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ladder {
    /// The requirement's id (as `verifies:` names it).
    pub requirement: String,
    pub name: String,
    pub stages: Vec<Stage>,
    /// The first stage that is not evidence yet, and what produces it.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub next: String,
}

impl Ladder {
    /// Whether any stage applies to the requirement.
    pub fn applies(&self) -> bool {
        self.stages.iter().any(|s| s.state != "n/a")
    }
    /// The highest stage with fresh evidence that passes, if any.
    pub fn reached(&self) -> Option<&str> {
        self.stages.iter().rev().find(|s| s.state == "pass").map(|s| s.stage.as_str())
    }
}

/// Whether the model has a machine allocated to programmable logic: the
/// HDL, SoC and board stages exist for its executed cases.
fn has_logic(ast: &Model) -> bool {
    super::embedded::plan::plan(ast).ok().and_then(|plan| super::embedded::alloc::allocations(ast, &plan).ok()).is_some_and(|a| !a.is_empty())
}

/// The requirements of the model: (key as `verifies:` names it, name).
fn requirements(ast: &Model) -> Vec<(String, String)> {
    ast.system_analysis
        .iter()
        .flat_map(|sa| sa.requirements.iter())
        .map(|r| {
            let key = r.attributes.get("id").and_then(|v| v.as_string()).map(str::to_string).unwrap_or_else(|| r.id.clone());
            let name = r.attributes.get("title").and_then(|v| v.as_string()).map(str::to_string).unwrap_or_else(|| r.id.clone());
            (key, name)
        })
        .collect()
}

fn verifies(ast: &Model, written: &[String], key: &str) -> bool {
    written.iter().any(|w| super::verification::requirement_key(ast, w) == key)
}

/// The state of a stage from the rungs of the cases reaching it.
fn merge(entries: &[(String, Option<&Rung>)]) -> (String, String) {
    if entries.is_empty() {
        return ("n/a".to_string(), String::new());
    }
    let state_of = |rung: Option<&Rung>| match rung {
        None => "missing",
        Some(r) if r.state == "stale" => "stale",
        Some(r) if r.state == "missing" => "missing",
        Some(r) => match r.verdict.as_str() {
            "pass" => "pass",
            "fail" => "fail",
            "error" => "error",
            _ => "missing",
        },
    };
    let states: Vec<&str> = entries.iter().map(|(_, r)| state_of(*r)).collect();
    let worst = ["error", "fail", "stale", "missing"].into_iter().find(|w| states.contains(w)).unwrap_or("pass");
    let target = entries.iter().filter_map(|(_, r)| r.map(|r| r.target.clone())).find(|t| !t.is_empty()).unwrap_or_default();
    (worst.to_string(), target)
}

fn stage_case(id: &str, rung: Option<&Rung>) -> StageCase {
    match rung {
        None => StageCase { id: id.to_string(), state: "missing".to_string(), reason: String::new(), evidence: String::new() },
        Some(r) => StageCase {
            id: id.to_string(),
            state: if r.state == "fresh" { r.verdict.clone() } else { r.state.clone() },
            reason: r.reason.clone(),
            evidence: r.evidence.clone(),
        },
    }
}

/// The ladder of every requirement, from the statuses `attach` computed
/// (`semantic_model.simulations`, by case or twin id).
pub fn ladders(result: &CompilationResult, model_path: Option<&Path>) -> Vec<Ladder> {
    let ast = &result.ast;
    let statuses = &result.semantic_model.simulations;
    let logic = has_logic(ast);
    fn rung_of<'a>(status: Option<&'a Status>, stage: &str) -> Option<&'a Rung> {
        status.and_then(|s| s.rungs.iter().find(|r| r.stage == stage))
    }
    let model = model_path.map(|p| p.display().to_string()).unwrap_or_else(|| "<model>".to_string());
    requirements(ast)
        .into_iter()
        .map(|(key, name)| {
            let cases: Vec<&TestCase> = ast.test_cases.iter().filter(|t| verifies(ast, &t.verifies, &key)).collect();
            let twins: Vec<&super::ast::Twin> = ast.twins.iter().filter(|t| verifies(ast, &t.verifies, &key)).collect();
            let simulated: Vec<&TestCase> = cases.iter().copied().filter(|t| is_simulated(t)).collect();
            let executed: Vec<&TestCase> = cases.iter().copied().filter(|t| super::embedded::case::is_executed(t)).collect();
            let mut stages = Vec::new();
            let mut next = String::new();
            for stage in STAGES {
                let ids: Vec<String> = match stage {
                    "simulation" => simulated.iter().map(|t| t.id.clone()).collect(),
                    "sil" | "pil" => executed.iter().map(|t| t.id.clone()).collect(),
                    "hdl" | "soc" | "board" if logic => executed.iter().map(|t| t.id.clone()).collect(),
                    "twin" => twins.iter().map(|t| t.id.clone()).collect(),
                    _ => Vec::new(),
                };
                let entries: Vec<(String, Option<&Rung>)> = ids.iter().map(|id| (id.clone(), rung_of(statuses.get(id), stage))).collect();
                let (state, target) = merge(&entries);
                if next.is_empty() && !matches!(state.as_str(), "pass" | "n/a") {
                    let first = entries.iter().find(|(_, r)| r.is_none_or(|r| r.state != "fresh" || r.verdict != "pass")).map(|(id, _)| id.clone()).unwrap_or_default();
                    let flag = if stage == "twin" { "--twin" } else { "--test" };
                    // A rung's reason already names the command with the model's real path.
                    let named = entries.iter().filter_map(|(_, r)| r.map(|r| r.reason.as_str())).find_map(|reason| {
                        let start = reason.find("`arclang ")? + 1;
                        let end = start + reason[start..].find('`')?;
                        Some(reason[start..end].to_string())
                    });
                    next = match state.as_str() {
                        "fail" | "error" => format!("{} {}: the {} verdict {}; see the case", label(stage), first, label(stage), if state == "fail" { "fails" } else { "is in error" }),
                        _ => named.filter(|_| model_path.is_none()).unwrap_or_else(|| format!("arclang {} {model} {flag} {first}", command(stage))),
                    };
                }
                stages.push(Stage { stage: stage.to_string(), state, cases: entries.iter().map(|(id, r)| stage_case(id, *r)).collect(), target });
            }
            Ladder { requirement: key, name, stages, next }
        })
        .collect()
}

/// Whether any requirement of the model has a stage to climb.
pub fn applies(ladders: &[Ladder]) -> bool {
    ladders.iter().any(Ladder::applies)
}

/// A ladder on one line: `REQ-X "Name"  simulation ✓ · SIL ✓ · PIL ~ · HDL – · SoC ○ · board – · twin –`.
pub fn line(ladder: &Ladder) -> String {
    let mark = |state: &str| match state {
        "pass" => "✓",
        "fail" | "error" => "✗",
        "stale" => "~",
        "missing" => "○",
        _ => "–",
    };
    let stages: Vec<String> = ladder.stages.iter().map(|s| format!("{} {}", label(&s.stage), mark(&s.state))).collect();
    let name = if ladder.name == ladder.requirement { String::new() } else { format!(" \"{}\"", ladder.name) };
    format!("{}{name}  {}", ladder.requirement, stages.join(" · "))
}

/// The legend of the marks `line` uses.
pub const LEGEND: &str = "✓ fresh pass · ✗ fail or error · ~ stale (changed since) · ○ never run · – not a stage of this requirement";

/// One step of a climb: a command, and the requirements it climbs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Step {
    pub stage: String,
    /// The arguments after `arclang` (`["sil", "model.arc", "--test", "TC-1"]`).
    pub args: Vec<String>,
    pub requirements: Vec<String>,
    /// Why: the state the stage is in for these requirements (`missing`, `stale`).
    pub state: String,
}

/// What no command can climb: a requirement with nothing to run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gap {
    pub requirement: String,
    pub name: String,
    pub what: String,
    pub hint: String,
}

/// The climb of a model: the steps in the order of the stages, each once,
/// the failures that stop it, and the gaps no step fills.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    pub steps: Vec<Step>,
    /// Requirements whose ladder has a fresh failure: a climb stops there.
    pub failing: Vec<(String, String)>,
    pub gaps: Vec<Gap>,
}

/// The arguments of the command that produces a stage for a case.
fn step_args(stage: &str, model: &str, case: &str) -> Vec<String> {
    let flag = if stage == "twin" { "--twin" } else { "--test" };
    let mut args: Vec<String> = match stage {
        "simulation" => vec!["verify".into()],
        "board" => vec!["soc".into(), "--port".into(), "<serial port>".into()],
        other => vec![other.into()],
    };
    args.push(model.to_string());
    args.push(flag.to_string());
    args.push(case.to_string());
    args
}

/// The plan that climbs `ladders`: for `only` (a requirement) or all, up to
/// `up_to` (a stage) or the top. A step is a stage missing or stale for a
/// case, each case once per stage; the stages come in order, so a stale
/// SIL is run again before PIL is tried. A stage with a fresh failure is
/// not climbed again: it is reported, the engineer looks at the case.
pub fn plan(ladders: &[Ladder], model: &str, only: Option<&str>, up_to: Option<&str>, with_board: bool) -> Plan {
    // `up_to` is climbed too: the limit is the stage after it.
    let limit = up_to.map(|s| stage_index(s) + 1).unwrap_or(STAGES.len());
    let chosen: Vec<&Ladder> = ladders.iter().filter(|l| only.is_none_or(|o| l.requirement == o || l.name == o)).collect();
    let mut steps: Vec<Step> = Vec::new();
    let mut failing = Vec::new();
    let mut gaps = Vec::new();
    for ladder in &chosen {
        if !ladder.applies() {
            gaps.push(Gap {
                requirement: ladder.requirement.clone(),
                name: ladder.name.clone(),
                what: "no case simulates or executes for it, and no twin verifies it".to_string(),
                hint: "add a test_case that verifies it with simulate: { ... } (the architecture in OpenModelica) or execute: (a state machine, run as C), or a twin".to_string(),
            });
            continue;
        }
        for (index, stage) in ladder.stages.iter().enumerate() {
            if index >= limit || stage.state == "n/a" || stage.state == "pass" {
                continue;
            }
            if stage.stage == "board" && !with_board {
                continue;
            }
            if matches!(stage.state.as_str(), "fail" | "error") {
                let cases: Vec<&str> = stage.cases.iter().filter(|c| matches!(c.state.as_str(), "fail" | "error")).map(|c| c.id.as_str()).collect();
                failing.push((ladder.requirement.clone(), format!("{}: {} ({})", label(&stage.stage), stage.state, cases.join(", "))));
                break;
            }
            for case in stage.cases.iter().filter(|c| c.state != "pass") {
                let args = step_args(&stage.stage, model, &case.id);
                match steps.iter_mut().find(|s| s.args == args) {
                    Some(step) => {
                        if !step.requirements.contains(&ladder.requirement) {
                            step.requirements.push(ladder.requirement.clone());
                        }
                    }
                    None => steps.push(Step { stage: stage.stage.clone(), args, requirements: vec![ladder.requirement.clone()], state: case.state.clone() }),
                }
            }
        }
    }
    steps.sort_by_key(|s| stage_index(&s.stage));
    Plan { steps, failing, gaps }
}

fn stage_index(stage: &str) -> usize {
    STAGES.iter().position(|s| *s == stage).unwrap_or(STAGES.len())
}
