//! A scenario (a sequence diagram) played message by message.
//!
//! Each step sends the next message. Combined fragments decide what comes
//! next, from conditions the reader sets (false until set), as written:
//! - `alt`: the first operand whose label holds, else an operand labelled
//!   `else` (or unlabelled), else none;
//! - `opt`: its operand when its label holds;
//! - `loop`: its body as many times as its label says (`3`, `3 times`,
//!   `x3`), else while its label holds, at most [`LOOP_LIMIT`] times;
//! - `par`, `seq`, `critical`: the operands in the order written (a run is
//!   one interleaving: the one written).
//! The state invariants on the lifelines hold while their messages run.

use crate::compiler::ast::{CombinedFragment, FragmentType, Scenario};
use serde::Serialize;
use std::collections::BTreeMap;

pub const LOOP_LIMIT: usize = 100;

#[derive(Debug, Clone, Serialize)]
pub enum Item {
    Message(usize),
    Fragment { kind: String, label: String, operands: Vec<(String, Vec<Item>)> },
}

/// What one step of the scenario did.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ScenarioStep {
    /// The message sent, by index, when one was.
    pub sent: Option<usize>,
    pub notes: Vec<String>,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize)]
struct Frame {
    items: Vec<Item>,
    position: usize,
    /// A loop's body: its label and how many times it ran, its count when written.
    looping: Option<(String, usize, Option<usize>)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScenarioRun {
    pub name: String,
    /// The messages: from, to, label.
    pub messages: Vec<(String, String, String)>,
    /// The invariants: id, lifeline, name, first and last message.
    pub invariants: Vec<(String, String, String, usize, usize)>,
    pub plan: Vec<Item>,
    stack: Vec<Frame>,
    pub conditions: BTreeMap<String, bool>,
    /// The last message sent.
    pub last: Option<usize>,
}

fn kind_of(fragment: &CombinedFragment) -> String {
    match fragment.fragment_type {
        FragmentType::Alt => "alt",
        FragmentType::Opt => "opt",
        FragmentType::Loop => "loop",
        FragmentType::Par => "par",
        FragmentType::Critical => "critical",
        _ => "seq",
    }
    .to_string()
}

/// The items of the messages `from..=to`, the fragments among them nested.
fn items(from: usize, to: usize, fragments: &[CombinedFragment]) -> Vec<Item> {
    let mut out = Vec::new();
    let mut at = from;
    while at <= to && at != usize::MAX {
        if let Some(fragment) = fragments.iter().find(|f| f.start == at && f.end >= f.start) {
            let operands = fragment
                .operands
                .iter()
                .map(|operand| {
                    let body = if operand.end >= operand.start { items(operand.start, operand.end, &operand.fragments) } else { Vec::new() };
                    (operand.label.clone(), body)
                })
                .collect();
            out.push(Item::Fragment { kind: kind_of(fragment), label: fragment.label.clone(), operands });
            at = fragment.end + 1;
        } else {
            out.push(Item::Message(at));
            at += 1;
        }
    }
    out
}

/// How many times a loop's label says it runs: `3`, `3 times`, `x3`.
fn count_of(label: &str) -> Option<usize> {
    let digits: String = label.chars().filter(|c| c.is_ascii_digit()).collect();
    let words = label.trim().to_ascii_lowercase();
    let plain = words.trim_start_matches('x').trim_end_matches(" times").trim_end_matches(" time").trim();
    (!digits.is_empty() && plain.chars().all(|c| c.is_ascii_digit())).then(|| digits.parse().ok()).flatten()
}

impl ScenarioRun {
    pub fn new(scenario: &Scenario) -> ScenarioRun {
        let messages = scenario.messages.iter().map(|m| (m.from.clone(), m.to.clone(), m.label.clone())).collect::<Vec<_>>();
        let invariants = scenario.invariants.iter().map(|i| (i.id.clone(), i.lifeline.clone(), i.name.clone(), i.start, i.end)).collect();
        let plan = if messages.is_empty() { Vec::new() } else { items(0, messages.len() - 1, &scenario.fragments) };
        ScenarioRun { name: scenario.name.clone(), messages, invariants, plan, stack: Vec::new(), conditions: BTreeMap::new(), last: None }
    }

    fn holds(&self, label: &str) -> bool {
        match label.trim() {
            "" | "true" => true,
            "false" => false,
            text => self.conditions.get(text).copied().unwrap_or(false),
        }
    }

    /// The labels a run asks about: of operands of alt and opt, and of loops.
    pub fn guards(&self) -> Vec<String> {
        fn walk(items: &[Item], out: &mut Vec<String>) {
            for item in items {
                if let Item::Fragment { kind, label, operands } = item {
                    match kind.as_str() {
                        "alt" | "opt" => out.extend(operands.iter().map(|(l, _)| l.clone()).filter(|l| !l.trim().is_empty() && l.trim() != "else")),
                        "loop" if count_of(label).is_none() && !label.trim().is_empty() => out.push(label.clone()),
                        _ => {}
                    }
                    if kind == "opt" && !label.trim().is_empty() {
                        out.push(label.clone());
                    }
                    for (_, body) in operands {
                        walk(body, out);
                    }
                }
            }
        }
        let mut out = Vec::new();
        walk(&self.plan, &mut out);
        out.sort();
        out.dedup();
        out
    }

    pub fn start(&mut self) -> ScenarioStep {
        self.stack = vec![Frame { items: self.plan.clone(), position: 0, looping: None }];
        self.last = None;
        ScenarioStep { notes: vec![format!("{} messages to send", self.messages.len())], done: self.plan.is_empty(), ..ScenarioStep::default() }
    }

    /// The invariants that hold at the last message sent.
    pub fn holding(&self) -> Vec<usize> {
        let Some(at) = self.last else { return Vec::new() };
        (0..self.invariants.len()).filter(|&i| self.invariants[i].3 <= at && at <= self.invariants[i].4).collect()
    }

    /// The fragments the next message is in, outermost first.
    pub fn context(&self) -> Vec<String> {
        self.stack.iter().filter_map(|f| f.looping.as_ref().map(|(label, n, _)| format!("loop {label} (pass {})", n + 1))).collect()
    }

    /// Send the next message, deciding the fragments on the way.
    pub fn step(&mut self) -> ScenarioStep {
        let mut step = ScenarioStep::default();
        loop {
            let Some(frame) = self.stack.last_mut() else {
                step.done = true;
                step.notes.push("the scenario is done".to_string());
                return step;
            };
            if frame.position >= frame.items.len() {
                // A loop's body ends: once more, or out.
                if let Some((label, runs, count)) = frame.looping.clone() {
                    let runs = runs + 1;
                    let again = match count { Some(n) => runs < n, None => !label.trim().is_empty() && self.conditions.get(label.trim()).copied().unwrap_or(false) };
                    let frame = self.stack.last_mut().expect("a frame");
                    if again && runs < LOOP_LIMIT {
                        frame.position = 0;
                        frame.looping = Some((label.clone(), runs, count));
                        step.notes.push(format!("loop {label}: pass {}", runs + 1));
                        continue;
                    }
                    if runs >= LOOP_LIMIT {
                        step.notes.push(format!("loop {label}: stopped after {LOOP_LIMIT} passes"));
                    }
                }
                self.stack.pop();
                continue;
            }
            let item = frame.items[frame.position].clone();
            frame.position += 1;
            match item {
                Item::Message(index) => {
                    self.last = Some(index);
                    step.sent = Some(index);
                    let before: Vec<usize> = self.holding();
                    for &i in &before {
                        if self.invariants[i].3 == index {
                            step.notes.push(format!("{} is {} from here", self.invariants[i].1, self.invariants[i].2));
                        }
                    }
                    let finished = self.stack.iter().all(|f| f.position >= f.items.len() && f.looping.is_none());
                    step.done = finished;
                    return step;
                }
                Item::Fragment { kind, label, operands } => {
                    let chosen: Vec<(String, Vec<Item>)> = match kind.as_str() {
                        "alt" => {
                            let pick = operands.iter().find(|(l, _)| !l.trim().is_empty() && l.trim() != "else" && self.holds(l)).or_else(|| operands.iter().find(|(l, _)| l.trim().is_empty() || l.trim() == "else"));
                            match pick {
                                Some((l, body)) => {
                                    step.notes.push(format!("alt {label}: takes {}", if l.trim().is_empty() { "the unlabelled operand".to_string() } else { format!("'{l}'") }));
                                    vec![(l.clone(), body.clone())]
                                }
                                None => {
                                    step.notes.push(format!("alt {label}: no operand holds, skipped"));
                                    Vec::new()
                                }
                            }
                        }
                        "opt" => {
                            let guard = operands.first().map(|(l, _)| l.clone()).filter(|l| !l.trim().is_empty()).unwrap_or_else(|| label.clone());
                            if self.holds(&guard) {
                                step.notes.push(format!("opt '{guard}' holds: in"));
                                operands.into_iter().take(1).collect()
                            } else {
                                step.notes.push(format!("opt '{guard}' does not hold: skipped"));
                                Vec::new()
                            }
                        }
                        "loop" => {
                            let guard = if label.trim().is_empty() { operands.first().map(|(l, _)| l.clone()).unwrap_or_default() } else { label.clone() };
                            let count = count_of(&guard);
                            let enter = match count { Some(n) => n > 0, None => self.holds(&guard) && !guard.trim().is_empty() };
                            if enter {
                                step.notes.push(format!("loop {guard}: pass 1"));
                                let body = operands.into_iter().flat_map(|(_, b)| b).collect();
                                self.stack.push(Frame { items: body, position: 0, looping: Some((guard, 0, count)) });
                            } else {
                                step.notes.push(format!("loop {guard}: not entered"));
                            }
                            continue;
                        }
                        other => {
                            if other == "par" {
                                step.notes.push(format!("par {label}: its operands in the order written"));
                            }
                            operands
                        }
                    };
                    let body: Vec<Item> = chosen.into_iter().flat_map(|(_, b)| b).collect();
                    self.stack.push(Frame { items: body, position: 0, looping: None });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{Compiler, CompilerConfig};

    fn run(source: &str) -> ScenarioRun {
        let result = Compiler::new(CompilerConfig::default()).compile_string(source).expect("compiles");
        ScenarioRun::new(&result.ast.scenarios[0])
    }

    const RUN: &str = "model T { name: \"t\" }\nlogical_architecture LA {\n    component A { id: \"LC-A\" }\n    component B { id: \"LC-B\" }\n}\nstate_machine M {\n    initial: \"Idle\"\n    state Idle {}\n    state Busy {}\n}\nscenario Run {\n    participants: [\"A\", \"B\"]\n    message A -> B \"start\"\n    fragment alt \"answer\" {\n        operand \"accepted\" {\n            message B -> A \"ok\"\n        }\n        operand \"else\" {\n            message B -> A \"refused\"\n        }\n    }\n    fragment loop \"2 times\" {\n        state \"B\" \"Busy\" {\n            message A -> B \"tick\"\n        }\n    }\n    message A -> B \"stop\"\n}\n";

    fn sent(run: &mut ScenarioRun) -> Vec<String> {
        let mut out = Vec::new();
        loop {
            let step = run.step();
            if let Some(i) = step.sent {
                out.push(run.messages[i].2.clone());
            }
            if step.done || step.sent.is_none() {
                return out;
            }
        }
    }

    #[test]
    fn alt_takes_the_operand_that_holds_and_loop_runs_as_written() {
        let mut r = run(RUN);
        r.start();
        assert_eq!(sent(&mut r), ["start", "refused", "tick", "tick", "stop"]);
        assert_eq!(r.guards(), ["accepted"]);
        let mut r = run(RUN);
        r.conditions.insert("accepted".into(), true);
        r.start();
        assert_eq!(sent(&mut r), ["start", "ok", "tick", "tick", "stop"]);
    }

    #[test]
    fn an_invariant_holds_while_its_messages_run() {
        let mut r = run(RUN);
        r.start();
        r.step();
        r.step();
        let tick = r.step();
        assert_eq!(r.messages[tick.sent.unwrap()].2, "tick");
        assert!(tick.notes.iter().any(|n| n == "B is Busy from here"), "{tick:?}");
        assert_eq!(r.holding().len(), 1);
        r.step();
        r.step();
        assert!(r.holding().is_empty());
    }
}
