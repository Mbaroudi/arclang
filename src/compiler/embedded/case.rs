//! A test case executed at fixed step (`execute:`): the input vectors of a
//! state machine, the run's length and step, checked by its `expects:` on
//! the model-level run and on the generated C (`arclang sil`).
//!
//! ```text
//! test_case "TC-ACC-SIL-1" {
//!     verifies: ["SYS-ACC-005"]
//!     execute: {
//!         machine: "ACC operating modes"
//!         step: 10 ms
//!         stop_time: 1 s
//!         inputs: ["0 s: main switch on", "10 ms: LF-022", "300 ms: main switch on = false"]
//!     }
//!     expects: ["at(20 ms, \"ACC operating modes\".Standby) == 1", "final(\"ACC operating modes\".Off) == 0"]
//! }
//! ```
//!
//! An input is `<time>: <event>` (the event occurs in the step at that
//! time) or `<time>: <condition>` / `<condition> = true|false` (it holds,
//! or not, from that step on). Every time is on the step grid.

use super::binding::period_of;
use super::plan::Plan;
use super::reference::Vectors;
use crate::compiler::ast::{AttributeValue, TestCase};
use crate::compiler::quantity::{Dimension, Quantity};
use std::collections::HashMap;

pub const EXECUTE_KEYS: [&str; 6] = ["machine", "step", "stop_time", "inputs", "binding", "tolerance"];

/// The default step of a case without `step:` or `binding:`.
pub const DEFAULT_STEP_US: u64 = 10_000;

/// Steps a case runs at most.
pub const MAX_STEPS: u64 = 1_000_000;

#[derive(Debug, Clone, PartialEq)]
pub enum Drive {
    Event(usize),
    Condition(usize, bool),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stimulus {
    pub written: String,
    pub step: usize,
    pub drive: Drive,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Case {
    pub machine: usize,
    pub step_us: u64,
    /// Steps run: the one at 0 and one per step up to `stop_time`.
    pub steps: usize,
    pub stimuli: Vec<Stimulus>,
    pub binding: Option<String>,
    /// How far a value of the C run may be from the model's (absolute).
    pub tolerance: f64,
}

pub fn is_executed(test: &TestCase) -> bool {
    test.attributes.contains_key("execute")
}

fn block(attributes: &HashMap<String, AttributeValue>) -> Option<Result<&HashMap<String, AttributeValue>, String>> {
    match attributes.get("execute")? {
        AttributeValue::Map(map) => Some(Ok(map)),
        _ => Some(Err("execute is a block: execute: { machine: \"...\" stop_time: 1 s inputs: [...] }".to_string())),
    }
}

/// The syntax of `execute:`, without the model: what the parser checks.
pub fn validate(attributes: &HashMap<String, AttributeValue>) -> Result<(), String> {
    let Some(map) = block(attributes) else { return Ok(()) };
    let map = map?;
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();
    if let Some(unknown) = keys.iter().find(|k| !EXECUTE_KEYS.contains(&k.as_str())) {
        return Err(format!("execute: '{unknown}' is not one of {}", EXECUTE_KEYS.join(", ")));
    }
    if !map.contains_key("stop_time") {
        return Err("execute: stop_time is required (e.g. stop_time: 1 s)".to_string());
    }
    for line in inputs(map)? {
        split(&line)?;
    }
    Ok(())
}

fn inputs(map: &HashMap<String, AttributeValue>) -> Result<Vec<String>, String> {
    match map.get("inputs") {
        None => Ok(Vec::new()),
        Some(AttributeValue::List(items)) => items.iter().map(|i| i.as_string().map(str::to_string).ok_or_else(|| "execute: inputs is a list of \"<time>: <event or condition>\"".to_string())).collect(),
        Some(AttributeValue::String(one)) => Ok(vec![one.clone()]),
        Some(_) => Err("execute: inputs is a list of \"<time>: <event or condition>\"".to_string()),
    }
}

/// `100 ms: LF-022` -> (seconds, "LF-022").
fn split(line: &str) -> Result<(f64, String), String> {
    let (time, name) = line.split_once(':').ok_or_else(|| format!("execute: input '{line}' reads \"<time>: <event or condition>\" (\"100 ms: LF-022\")"))?;
    let q = Quantity::parse(time).map_err(|_| format!("execute: input '{line}': '{}' is not a time (\"100 ms: ...\")", time.trim()))?;
    if q.dimension() != Dimension::Time {
        return Err(format!("execute: input '{line}': '{}' is not a time", time.trim()));
    }
    let name = name.trim();
    if name.is_empty() {
        return Err(format!("execute: input '{line}' names no event or condition"));
    }
    Ok((q.canonical(), name.to_string()))
}

fn on_grid(seconds: f64, step_us: u64, what: &str) -> Result<usize, String> {
    let steps = seconds * 1e6 / step_us as f64;
    if steps < -1e-9 || (steps - steps.round()).abs() > 1e-6 {
        return Err(format!("{what} is not on the step grid ({} us): use a multiple of the step", step_us));
    }
    Ok(steps.round() as usize)
}

/// `execute:` resolved against the machines; `None` when the case has none.
pub fn read(test: &TestCase, plan: &Plan, bindings: &[super::binding::Binding]) -> Option<Result<Case, String>> {
    let map = match block(&test.attributes)? {
        Ok(map) => map,
        Err(e) => return Some(Err(e)),
    };
    Some(resolve(map, plan, bindings).map_err(|e| format!("test_case '{}': {e}", test.name)))
}

fn resolve(map: &HashMap<String, AttributeValue>, plan: &Plan, bindings: &[super::binding::Binding]) -> Result<Case, String> {
    let machine = match map.get("machine").and_then(|v| v.as_string()) {
        Some(reference) => plan.machines.iter().position(|m| m.is(reference)).ok_or_else(|| format!("execute: machine '{reference}' is no state_machine of the model"))?,
        None if plan.machines.len() == 1 => 0,
        None => return Err("execute: name the state_machine it drives (machine: \"...\")".to_string()),
    };
    let binding = map.get("binding").map(|v| v.display());
    let bound = match &binding {
        None => None,
        Some(reference) => {
            let b = bindings.iter().find(|b| b.is(reference)).ok_or_else(|| format!("execute: binding '{reference}' is no io_binding of the model"))?;
            if b.machine != machine {
                return Err(format!("execute: io_binding '{reference}' binds state_machine '{}', not '{}'", plan.machines[b.machine].name, plan.machines[machine].name));
            }
            Some(b)
        }
    };
    let step_us = match (map.get("step"), bound) {
        (Some(v), b) => {
            let us = period_of(v, "execute: step")?;
            if let Some(b) = b.filter(|b| b.period_us != us) {
                return Err(format!("execute: step is {us} us but io_binding '{}' runs every {} us", b.name, b.period_us));
            }
            us
        }
        (None, Some(b)) => b.period_us,
        (None, None) => DEFAULT_STEP_US,
    };
    let stop = map.get("stop_time").ok_or("execute: stop_time is required (e.g. stop_time: 1 s)")?;
    let stop_s = match stop.as_quantity() {
        Some(Ok(q)) if q.dimension() == Dimension::Time => q.canonical(),
        _ => return Err(format!("execute: stop_time is a time, got '{}'", stop.display())),
    };
    let last = on_grid(stop_s, step_us, "execute: stop_time")?;
    if last as u64 >= MAX_STEPS {
        return Err(format!("execute: {} steps is more than {MAX_STEPS}", last + 1));
    }
    let m = &plan.machines[machine];
    let mut stimuli = Vec::new();
    for line in inputs(map)? {
        let (seconds, name) = split(&line)?;
        let step = on_grid(seconds, step_us, &format!("execute: input '{line}'"))?;
        if step > last {
            return Err(format!("execute: input '{line}' comes after stop_time"));
        }
        let drive = if let Some(e) = m.event(&name) {
            Drive::Event(e)
        } else {
            let (text, value) = match name.rsplit_once('=') {
                Some((text, value)) if matches!(value.trim(), "true" | "false") => (text.trim().to_string(), value.trim() == "true"),
                _ => (name.clone(), true),
            };
            match m.condition(&text) {
                Some(c) => Drive::Condition(c, value),
                None => {
                    return Err(format!(
                        "execute: input '{line}': '{text}' is no event or condition of state_machine '{}' (events: {}; conditions: {})",
                        m.name,
                        m.events.iter().map(|e| format!("'{}'", e.text)).collect::<Vec<_>>().join(", "),
                        m.conditions.iter().map(|c| format!("'{}'", c.text)).collect::<Vec<_>>().join(", ")
                    ))
                }
            }
        };
        stimuli.push(Stimulus { written: line.clone(), step, drive });
    }
    let tolerance = match map.get("tolerance") {
        None => 0.0,
        Some(v) => v.as_number().filter(|t| *t >= 0.0).ok_or_else(|| format!("execute: tolerance is a number of at least 0, got '{}'", v.display()))?,
    };
    Ok(Case { machine, step_us, steps: last + 1, stimuli, binding, tolerance })
}

impl Case {
    /// The vectors of every machine: this case's inputs on its machine, nothing on the others.
    pub fn vectors(&self, plan: &Plan) -> Vectors {
        let mut v = Vectors::quiet(plan, self.steps);
        let mut changes: Vec<&Stimulus> = self.stimuli.iter().filter(|s| matches!(s.drive, Drive::Condition(..))).collect();
        changes.sort_by_key(|s| s.step);
        for s in &self.stimuli {
            if let Drive::Event(e) = s.drive {
                v.events[self.machine][e][s.step] = true;
            }
        }
        for s in changes {
            if let Drive::Condition(c, value) = s.drive {
                for k in s.step..self.steps {
                    v.conditions[self.machine][c][k] = value;
                }
            }
        }
        v
    }
}
