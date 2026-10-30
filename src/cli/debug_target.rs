//! What `arclang debug` runs: a state machine, a scenario or a functional
//! chain of a model, with the lines of the source each step reaches (where
//! the breakpoints are) and what the workbench lights on the diagram.

use crate::compiler::ast::Model;
use crate::compiler::execution::chain::ChainRun;
use crate::compiler::execution::scenario::ScenarioRun;
use crate::compiler::execution::Machine;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

pub enum Run {
    Machine(Machine),
    Scenario(ScenarioRun),
    Chain(ChainRun),
}

/// What one step did, for the console, the breakpoints and the diagram.
#[derive(Debug, Clone, Default)]
pub struct Outcome {
    pub text: String,
    /// The lines the step reached: a state entered, a transition fired, a message sent, a step executed.
    pub lines: Vec<i64>,
    pub actions: Vec<String>,
    pub idle: Option<String>,
    pub done: bool,
}

pub struct Target {
    pub run: Run,
    pub source: String,
    /// Element ids by name, to light a chain step written by name.
    ids: BTreeMap<String, String>,
    pub last: Outcome,
}

/// The runnable parts of a model: state machines, scenarios, functional chains.
pub fn runnables(model: &Model) -> Value {
    let chains = model
        .system_analysis
        .iter()
        .flat_map(|s| &s.functional_chains)
        .chain(model.logical_architecture.iter().flat_map(|l| &l.functional_chains))
        .chain(model.physical_architecture.iter().flat_map(|p| &p.functional_chains))
        .chain(model.operational_analysis.iter().flat_map(|o| &o.processes))
        .map(|c| c.name.clone())
        .collect::<Vec<_>>();
    json!({
        "machines": model.state_machines.iter().map(|m| m.name.clone()).collect::<Vec<_>>(),
        "scenarios": model.scenarios.iter().map(|s| s.name.clone()).collect::<Vec<_>>(),
        "chains": chains,
    })
}

/// What to run: by kind and name, else the first state machine, else the
/// first scenario, else the first chain.
pub fn load(path: &Path, machine: Option<&str>, scenario: Option<&str>, chain: Option<&str>) -> Result<(Target, Value), String> {
    let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(path).map_err(|e| e.to_string())?;
    let model = &result.ast;
    let listed = runnables(model);
    let chains: Vec<&crate::compiler::ast::FunctionalChain> = model
        .system_analysis
        .iter()
        .flat_map(|s| &s.functional_chains)
        .chain(model.logical_architecture.iter().flat_map(|l| &l.functional_chains))
        .chain(model.physical_architecture.iter().flat_map(|p| &p.functional_chains))
        .chain(model.operational_analysis.iter().flat_map(|o| &o.processes))
        .collect();
    let missing = |what: &str, name: &str, key: &str| format!("no {what} '{name}': the model has {}", listed[key].as_array().map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", ")).unwrap_or_default());
    let run = match (machine, scenario, chain) {
        (Some(name), _, _) => Run::Machine(Machine::new(model.state_machines.iter().find(|m| m.name == name || m.id == name).ok_or_else(|| missing("state machine", name, "machines"))?)),
        (_, Some(name), _) => Run::Scenario(ScenarioRun::new(model.scenarios.iter().find(|s| s.name == name || s.id == name).ok_or_else(|| missing("scenario", name, "scenarios"))?)),
        (_, _, Some(name)) => Run::Chain(ChainRun::new(chains.iter().find(|c| c.name == name || c.id == name).ok_or_else(|| missing("functional chain", name, "chains"))?)),
        _ => match (model.state_machines.first(), model.scenarios.first(), chains.first()) {
            (Some(m), _, _) => Run::Machine(Machine::new(m)),
            (None, Some(s), _) => Run::Scenario(ScenarioRun::new(s)),
            (None, None, Some(c)) => Run::Chain(ChainRun::new(c)),
            _ => return Err(format!("{} declares no state machine, scenario or functional chain to run", path.display())),
        },
    };
    let ids = result.semantic_model.all_elements.values().map(|e| (e.name.clone(), e.id.clone())).collect();
    let source = std::fs::read_to_string(path).unwrap_or_default();
    Ok((Target { run, source, ids, last: Outcome::default() }, listed))
}

fn quoted(name: &str) -> String {
    format!("\"{name}\"")
}

/// The 1-based line of the first line satisfying `test`, from line `after` on.
fn find_line(source: &str, after: usize, test: impl Fn(&str) -> bool) -> Option<i64> {
    source.lines().enumerate().skip(after).find(|(_, l)| test(l.trim_start())).map(|(i, _)| i as i64 + 1)
}

fn first_word_is(line: &str, words: &[&str]) -> bool {
    line.split_whitespace().next().is_some_and(|w| words.contains(&w))
}

impl Target {
    pub fn kind(&self) -> &'static str {
        match self.run {
            Run::Machine(_) => "state machine",
            Run::Scenario(_) => "scenario",
            Run::Chain(_) => "functional chain",
        }
    }

    pub fn name(&self) -> String {
        match &self.run {
            Run::Machine(m) => m.name.clone(),
            Run::Scenario(s) => s.name.clone(),
            Run::Chain(c) => c.name.clone(),
        }
    }

    pub fn notes(&self) -> Vec<String> {
        match &self.run {
            Run::Machine(m) => m.notes.clone(),
            Run::Chain(c) => c.notes.clone(),
            Run::Scenario(_) => Vec::new(),
        }
    }

    /// The line that opens the declaration of what runs.
    fn header_line(&self) -> usize {
        let (keywords, name): (&[&str], String) = match &self.run {
            Run::Machine(m) => (&["state_machine"], m.name.clone()),
            Run::Scenario(s) => (&["scenario"], s.name.clone()),
            Run::Chain(c) => (&["functional_chain", "operational_process"], c.name.clone()),
        };
        find_line(&self.source, 0, |l| first_word_is(l, keywords) && (l.contains(&quoted(&name)) || l.split_whitespace().nth(1) == Some(name.as_str()))).map(|l| l as usize - 1).unwrap_or(0)
    }

    fn state_line(&self, name: &str) -> Option<i64> {
        let start = self.header_line();
        find_line(&self.source, start, |l| first_word_is(l, &["state", "mode"]) && l.split_whitespace().nth(1).is_some_and(|w| w == name || w == quoted(name) || w.trim_end_matches('{') == name))
    }

    fn transition_lines(&self, from: &str, to: &str) -> Vec<i64> {
        let bare = |s: &str| s.trim_matches('"').to_string();
        let start = self.header_line();
        self.source
            .lines()
            .enumerate()
            .skip(start)
            .filter(|(_, l)| {
                let w: Vec<&str> = l.split_whitespace().collect();
                w.first() == Some(&"transition") && w.len() >= 4 && w[2] == "->" && bare(w[1]) == from && bare(w[3]) == to
            })
            .map(|(i, _)| i as i64 + 1)
            .collect()
    }

    /// The line of the n-th message of the scenario (messages are in the order written).
    fn message_line(&self, index: usize) -> Option<i64> {
        let start = self.header_line();
        self.source.lines().enumerate().skip(start).filter(|(_, l)| first_word_is(l.trim_start(), &["message"])).nth(index).map(|(i, _)| i as i64 + 1)
    }

    /// The line declaring a step of a chain: its element, or its control node.
    fn chain_step_line(&self, reference: &str, control: bool) -> Option<i64> {
        if control {
            let start = self.header_line();
            return find_line(&self.source, start, |l| first_word_is(l, &["control"]) && l.contains(&quoted(reference)));
        }
        find_line(&self.source, 0, |l| l.contains(&format!("id: {}", quoted(reference))))
            .or_else(|| find_line(&self.source, 0, |l| !first_word_is(l, &["trace", "sequence"]) && l.split_whitespace().nth(1) == Some(quoted(reference).as_str())))
    }

    /// Whether a breakpoint may stand on `line`.
    pub fn breakable(&self, line: i64) -> bool {
        match &self.run {
            Run::Machine(m) => m.states.iter().any(|s| self.state_line(&s.name) == Some(line)) || m.transitions.iter().any(|t| self.transition_lines(&m.states[t.source].name, &m.states[t.target].name).contains(&line)),
            Run::Scenario(s) => (0..s.messages.len()).any(|i| self.message_line(i) == Some(line)),
            Run::Chain(c) => c.nodes.iter().any(|n| self.chain_step_line(&n.reference, n.control.is_some()) == Some(line)),
        }
    }

    pub fn start(&mut self) -> Outcome {
        let outcome = match &mut self.run {
            Run::Machine(m) => {
                let step = m.start();
                let lines = step.entered.iter().filter_map(|p| m.resolve(p)).map(|s| m.states[s].name.clone()).collect::<Vec<_>>();
                Outcome { text: format!("start\n  active: {}\n", m.leaves().join(", ")), lines: lines.iter().filter_map(|n| self.state_line_of(n)).collect(), actions: step.actions, ..Outcome::default() }
            }
            Run::Scenario(s) => {
                let step = s.start();
                Outcome { text: format!("start: {}\n", step.notes.join("; ")), done: step.done, ..Outcome::default() }
            }
            Run::Chain(c) => {
                let step = c.start();
                Outcome { text: format!("start: {}\n", step.notes.join("; ")), ..Outcome::default() }
            }
        };
        self.last = outcome.clone();
        outcome
    }

    fn state_line_of(&self, name: &str) -> Option<i64> {
        self.state_line(name)
    }

    /// One step: `event` fires a state machine (`None`: a completion step);
    /// a scenario sends its next message, a chain executes its next step.
    pub fn step(&mut self, event: Option<&str>) -> Outcome {
        let outcome = match &mut self.run {
            Run::Machine(m) => {
                let step = m.fire(event);
                let label = step.event.clone().unwrap_or_else(|| "(completion)".to_string());
                let mut text = String::new();
                let mut lines = Vec::new();
                match &step.idle {
                    Some(idle) => text.push_str(&format!("{label}: nothing fired, {idle}\n")),
                    None => {
                        for index in &step.fired {
                            if let Some(t) = m.transitions.iter().find(|t| t.index == *index) {
                                text.push_str(&format!("{label}: fired {}\n", t.label));
                                let (from, to) = (m.states[t.source].name.clone(), m.states[t.target].name.clone());
                                lines.push((from, to));
                            }
                        }
                        for action in &step.actions {
                            text.push_str(&format!("  {action}\n"));
                        }
                    }
                }
                text.push_str(&format!("  active: {}\n", m.leaves().join(", ")));
                let entered: Vec<String> = step.entered.iter().filter_map(|p| m.resolve(p)).map(|s| m.states[s].name.clone()).collect();
                let mut reached: Vec<i64> = Vec::new();
                for (from, to) in lines {
                    reached.extend(self.transition_lines(&from, &to));
                }
                reached.extend(entered.iter().filter_map(|n| self.state_line(n)));
                Outcome { text, lines: reached, actions: step.actions.clone(), idle: step.idle.clone(), done: false }
            }
            Run::Scenario(s) => {
                let step = s.step();
                let mut text = String::new();
                for note in &step.notes {
                    text.push_str(&format!("  {note}\n"));
                }
                let mut lines = Vec::new();
                match step.sent {
                    Some(i) => {
                        let (from, to, label) = s.messages[i].clone();
                        text.insert_str(0, &format!("message {}: {from} -> {to} \"{label}\"\n", i + 1));
                        lines.extend(self.message_line(i));
                    }
                    None => text.push_str("  nothing to send\n"),
                }
                Outcome { text, lines, actions: step.notes.clone(), idle: (step.sent.is_none()).then(|| "the scenario is done".to_string()), done: step.done }
            }
            Run::Chain(c) => {
                let step = c.step();
                let mut text = String::new();
                let mut lines = Vec::new();
                if let Some(n) = step.executed {
                    let (reference, control) = (c.nodes[n].reference.clone(), c.nodes[n].control.is_some());
                    lines.extend(self.chain_step_line(&reference, control));
                }
                for note in &step.notes {
                    text.push_str(&format!("{note}\n"));
                }
                if let Some(idle) = &step.idle {
                    text.push_str(&format!("nothing executed: {idle}\n"));
                }
                if let Run::Chain(c) = &self.run {
                    text.push_str(&format!("  tokens: {}\n", c.holding().join(", ")));
                }
                Outcome { text, lines, actions: step.notes.clone(), idle: step.idle.clone(), done: step.done }
            }
        };
        self.last = outcome.clone();
        outcome
    }

    pub fn conditions(&self) -> Vec<(String, bool)> {
        let (guards, set) = match &self.run {
            Run::Machine(m) => (m.guards(), &m.conditions),
            Run::Scenario(s) => (s.guards(), &s.conditions),
            Run::Chain(c) => (c.guards(), &c.conditions),
        };
        guards.into_iter().map(|g| { let v = set.get(&g).copied().unwrap_or(false); (g, v) }).collect()
    }

    pub fn set_condition(&mut self, name: &str, holds: bool) {
        let set = match &mut self.run {
            Run::Machine(m) => &mut m.conditions,
            Run::Scenario(s) => &mut s.conditions,
            Run::Chain(c) => &mut c.conditions,
        };
        set.insert(name.trim().to_string(), holds);
    }

    /// The events a state machine waits for (none for a scenario or a chain).
    pub fn events(&self) -> Vec<String> {
        match &self.run {
            Run::Machine(m) => m.events(),
            _ => Vec::new(),
        }
    }

    /// The call stack: where the run is, deepest first, at its lines.
    pub fn frames(&self) -> Vec<(String, i64)> {
        match &self.run {
            Run::Machine(m) => {
                let mut active: Vec<usize> = m.active.iter().copied().collect();
                active.sort_by_key(|&s| std::cmp::Reverse(m.states[s].path.matches('.').count()));
                active.into_iter().map(|s| (format!("{} {}", if m.states[s].mode { "mode" } else { "state" }, m.states[s].path), self.state_line(&m.states[s].name).unwrap_or(1))).collect()
            }
            Run::Scenario(s) => {
                let mut frames = Vec::new();
                if let Some(i) = s.last {
                    let (from, to, label) = &s.messages[i];
                    frames.push((format!("message {}: {from} -> {to} \"{label}\"", i + 1), self.message_line(i).unwrap_or(1)));
                }
                for context in s.context().into_iter().rev() {
                    frames.push((context, self.header_line() as i64 + 1));
                }
                frames.push((format!("scenario {}", s.name), self.header_line() as i64 + 1));
                frames
            }
            Run::Chain(c) => {
                let mut frames: Vec<(String, i64)> = c.tokens.iter().map(|&n| (format!("token on {}", c.nodes[n].reference), self.chain_step_line(&c.nodes[n].reference, c.nodes[n].control.is_some()).unwrap_or(1))).collect();
                frames.push((format!("chain {}", c.name), self.header_line() as i64 + 1));
                frames
            }
        }
    }

    /// The first scope of the Variables view: where the run is.
    pub fn position(&self) -> (&'static str, Vec<(String, String)>) {
        match &self.run {
            Run::Machine(m) => ("Active states", m.leaves().into_iter().enumerate().map(|(i, l)| (format!("active[{i}]"), l)).collect()),
            Run::Scenario(s) => {
                let mut out: Vec<(String, String)> = s.last.map(|i| ("last message".to_string(), format!("{} -> {} \"{}\"", s.messages[i].0, s.messages[i].1, s.messages[i].2))).into_iter().collect();
                out.extend(s.holding().into_iter().map(|i| (s.invariants[i].1.clone(), format!("{} (invariant)", s.invariants[i].2))));
                ("Scenario", out)
            }
            Run::Chain(c) => ("Tokens", c.holding().into_iter().enumerate().map(|(i, h)| (format!("token[{i}]"), h)).collect()),
        }
    }

    /// What the workbench lights: nodes and edges by drawing id, or by the
    /// reference the page resolves.
    pub fn lit(&self) -> Value {
        match &self.run {
            Run::Machine(m) => {
                let fired: Vec<String> = m.transitions.iter().filter(|t| self.last.lines.iter().any(|l| self.transition_lines(&m.states[t.source].name, &m.states[t.target].name).contains(l))).map(|t| format!("tr:{} -> {}", m.states[t.source].name, m.states[t.target].name)).collect();
                json!({ "kind": "machine", "name": m.name, "nodes": m.active.iter().map(|&s| format!("{}.{}", m.name, m.states[s].path)).collect::<Vec<_>>(), "edges": fired, "leaves": m.leaves(), "events": m.events() })
            }
            Run::Scenario(s) => json!({ "kind": "scenario", "name": s.name, "nodes": s.holding().into_iter().map(|i| s.invariants[i].0.clone()).collect::<Vec<_>>(), "edges": s.last.map(|i| vec![format!("msg:{}", i + 1)]).unwrap_or_default(), "events": [] }),
            Run::Chain(c) => {
                let refs: Vec<String> = c.holding().into_iter().map(|r| self.ids.get(&r).cloned().unwrap_or(r)).collect();
                json!({ "kind": "chain", "name": c.name, "nodes": refs, "edges": [], "events": [] })
            }
        }
    }
}
