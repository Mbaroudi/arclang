//! The model-level run of a fixed-step case: the machines executed by
//! `execution::Machine` (the debugger's executor), step by step, with the
//! same step semantics as the generated `step()`:
//!
//! 1. the `in` variables take the step's values; on the first step, the
//!    initial states are entered; the equations are evaluated, in the order
//!    written;
//! 2. the events of the step fire, in the order they arrive (the queue);
//! 3. the timed transitions whose source has been active long enough fire
//!    (`after 5 s`: at least ceil(5 s / period) steps);
//! 4. the completion transitions (no trigger) fire, once;
//! 5. the `do:` activities of the active states run;
//! 6. each active state has been active one step more.
//!
//! The result is a table: per step, each state (1 while active), each
//! action (the times it ran in the step, at most 255) and each variable
//! (its value after the step, in its unit).

use super::plan::{MachinePlan, Plan, Trigger};
use crate::compiler::execution::expr::Value;
use crate::compiler::simulation::results::Series;
use std::collections::BTreeMap;

/// The inputs of a run: per machine, each event and each condition at each step.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Vectors {
    pub steps: usize,
    /// [machine][event][step]: whether the event occurs in the step.
    pub events: Vec<Vec<Vec<bool>>>,
    /// [machine][step]: the events of the step in the order they arrive
    /// (an event may arrive more than once).
    pub queue: Vec<Vec<Vec<usize>>>,
    /// [machine][condition][step]
    pub conditions: Vec<Vec<Vec<bool>>>,
    /// [machine][variable][step]: the value of each `in` variable (the
    /// others' entries are their initial values, unused).
    pub values: Vec<Vec<Vec<Value>>>,
}

impl Vectors {
    /// Every input false at every step.
    pub fn quiet(plan: &Plan, steps: usize) -> Vectors {
        Vectors {
            steps,
            events: plan.machines.iter().map(|m| vec![vec![false; steps]; m.events.len()]).collect(),
            queue: plan.machines.iter().map(|_| vec![Vec::new(); steps]).collect(),
            conditions: plan.machines.iter().map(|m| vec![vec![false; steps]; m.conditions.len()]).collect(),
            values: plan.machines.iter().map(|m| m.program().variables.iter().map(|v| vec![v.initial; steps]).collect()).collect(),
        }
    }
}

/// A table of values by step: what the model run and the C run both give.
#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub names: Vec<String>,
    pub period_us: u64,
    pub rows: Vec<Vec<f64>>,
    /// The unit of a column that has one (a variable's), else `1`.
    pub units: BTreeMap<String, String>,
}

impl Table {
    pub fn series(&self) -> Result<Series, String> {
        let mut csv = String::from("\"time\"");
        for name in &self.names {
            csv.push_str(&format!(",\"{name}\""));
        }
        csv.push('\n');
        for (k, row) in self.rows.iter().enumerate() {
            csv.push_str(&format!("{}", k as f64 * self.period_us as f64 / 1e6));
            for value in row {
                csv.push_str(&format!(",{value}"));
            }
            csv.push('\n');
        }
        let mut series = Series::from_csv(&csv)?;
        series.set_units(self.names.iter().map(|n| (n.clone(), self.units.get(n).cloned().unwrap_or_else(|| "1".to_string()))).collect());
        Ok(series)
    }
}

/// The result columns of the machines: `<machine>.<State>` then `<machine>.action.<name>`.
pub fn columns(plan: &Plan) -> Vec<String> {
    let mut out = Vec::new();
    for m in &plan.machines {
        out.extend(m.states.iter().map(|s| format!("{}.{}", m.ident, s.member)));
        out.extend(m.actions.iter().map(|a| format!("{}.{}", m.ident, a.member)));
        out.extend(m.variables.iter().map(|v| format!("{}.{}", m.ident, v.member)));
        out.extend(m.budgets.iter().map(|b| format!("{}.{}", m.ident, b.member)));
    }
    out
}

/// The unit of each variable column that has one.
pub fn units(plan: &Plan) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for m in &plan.machines {
        for (v, signal) in m.program().variables.iter().zip(&m.variables) {
            if let Some(unit) = &v.unit {
                out.insert(format!("{}.{}", m.ident, signal.member), unit.clone());
            }
        }
    }
    out
}

/// Machine name and id -> the first segment of its columns, for `"ACC modes".Following`.
pub fn aliases(plan: &Plan) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for m in &plan.machines {
        out.insert(m.name.clone(), m.ident.clone());
        if !m.id.is_empty() {
            out.insert(m.id.clone(), m.ident.clone());
        }
        out.insert(m.ident.clone(), m.ident.clone());
    }
    out
}

/// Steps a timed trigger waits at this period.
pub fn ticks(micros: u64, period_us: u64) -> u64 {
    micros.div_ceil(period_us)
}

struct Running<'p> {
    plan: &'p MachinePlan,
    machine: crate::compiler::execution::Machine,
    in_state: Vec<u64>,
    counts: Vec<u32>,
}

impl Running<'_> {
    fn account(&mut self, step: &crate::compiler::execution::Step) {
        let by_path = |path: &str| self.plan.states.iter().position(|s| s.path == path);
        for path in &step.exited {
            if let Some(s) = by_path(path) {
                for &a in &self.plan.states[s].exit {
                    self.counts[a] += 1;
                }
            }
        }
        for &fired in &step.fired {
            if let Some(t) = self.plan.transitions.iter().find(|t| t.index == fired) {
                if let Some(a) = t.action {
                    self.counts[a] += 1;
                }
            }
        }
        for path in &step.entered {
            if let Some(s) = by_path(path) {
                self.in_state[s] = 0;
                for &a in &self.plan.states[s].entry {
                    self.counts[a] += 1;
                }
            }
        }
    }
}

/// The model run step by step: the machines of the plan, each at the
/// period, advanced one step at a time on the vectors (a closed loop fills
/// the vectors as it goes).
pub struct Runner<'p> {
    machines: Vec<Running<'p>>,
    period_us: u64,
}

impl<'p> Runner<'p> {
    pub fn new(plan: &'p Plan, period_us: u64) -> Runner<'p> {
        let machines = plan
            .machines
            .iter()
            .map(|m| {
                let mut machine = m.machine.clone();
                machine.set_period(period_us);
                Running { plan: m, machine, in_state: vec![0; m.states.len()], counts: vec![0; m.actions.len()] }
            })
            .collect();
        Runner { machines, period_us }
    }

    /// Step `k` of every machine on the vectors' inputs of that step: the row of `columns`.
    pub fn step(&mut self, vectors: &Vectors, k: usize) -> Vec<f64> {
        let period_us = self.period_us;
        let mut row = Vec::new();
        for (n, r) in self.machines.iter_mut().enumerate() {
            r.counts.iter_mut().for_each(|c| *c = 0);
            let inputs: Vec<usize> = r.plan.program().inputs().map(|(i, _)| i).collect();
            for i in inputs {
                r.machine.env.values[i] = vectors.values[n][i][k];
            }
            r.machine.conditions = r.plan.conditions.iter().enumerate().map(|(c, signal)| (signal.text.clone(), vectors.conditions[n][c][k])).collect();
            if k == 0 {
                let started = r.machine.start();
                r.account(&started);
            }
            r.machine.flow();
            for &e in &vectors.queue[n][k] {
                let step = r.machine.fire(Some(&r.plan.events[e].text));
                r.account(&step);
            }
            for (trigger, _) in &r.plan.dispatch.clone() {
                let step = match *trigger {
                    Trigger::Event(_) => continue,
                    Trigger::Timer(tm) => {
                        let timer = &r.plan.timers[tm];
                        let wait = ticks(timer.micros, period_us);
                        let timed: Vec<usize> = r.plan.compounds.iter().filter(|c| c.trigger == Trigger::Timer(tm)).map(|c| r.plan.transitions[c.compound.head].index).collect();
                        let in_state = r.in_state.clone();
                        let keep = move |t: &crate::compiler::execution::machine::TransitionNode| timed.contains(&t.index) && in_state[t.source] >= wait;
                        r.machine.fire_where(Some(&timer.label), &keep)
                    }
                    Trigger::Completion => r.machine.fire(None),
                };
                r.account(&step);
            }
            // The do activities that are code run here, in the order of the states.
            r.machine.activities();
            for s in 0..r.plan.states.len() {
                if r.machine.active.contains(&s) {
                    for &a in &r.plan.states[s].activity {
                        r.counts[a] += 1;
                    }
                }
            }
            for s in 0..r.plan.states.len() {
                if r.machine.active.contains(&s) {
                    r.in_state[s] = r.in_state[s].saturating_add(1).min(u32::MAX as u64);
                }
            }
            row.extend((0..r.plan.states.len()).map(|s| if r.machine.active.contains(&s) { 1.0 } else { 0.0 }));
            row.extend(r.counts.iter().map(|&c| c.min(255) as f64));
            row.extend(r.machine.env.values.iter().map(|v| v.number()));
            // A budget is overrun when its source has been active longer than it.
            row.extend(r.plan.budgets.iter().map(|b| (r.machine.active.contains(&b.source) && r.in_state[b.source] > ticks(b.micros, period_us)) as u8 as f64));
        }
        row
    }
}

pub fn run(plan: &Plan, vectors: &Vectors, period_us: u64) -> Table {
    let mut runner = Runner::new(plan, period_us);
    let rows = (0..vectors.steps).map(|k| runner.step(vectors, k)).collect();
    Table { names: columns(plan), period_us, rows, units: units(plan) }
}
