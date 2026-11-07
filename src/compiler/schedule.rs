//! Tasks and scheduling (`arclang schedule`): several `io_binding`s on one
//! processor, each a task at its period and priority (1 the highest; by
//! the period when none is written, the shortest the highest: rate
//! monotonic). The worst time a task's step takes is what `arclang
//! partition` measured on the target (`arclang pil`: the longest step of
//! the target's virtual time), never a guess; `--wcet` may set it. The
//! analysis is the response-time analysis of fixed-priority preemptive
//! scheduling: a task's response is its own time plus the interference of
//! every higher-priority task over that response, iterated to a fixed
//! point; it meets its period when the response fits in it. The whole set
//! is also given the utilisation and the rate-monotonic bound.

use crate::compiler::embedded::binding::{self, Binding};
use crate::compiler::embedded::plan::Plan;
use crate::compiler::embedded::tasks::TaskRun;
use crate::compiler::partition::Entry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub binding: String,
    pub machine: String,
    pub period_us: u64,
    /// 1 the highest.
    pub priority: u32,
    /// The worst a step takes on the target (microseconds), when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wcet_us: Option<f64>,
    /// Where it comes from: `measured`, `measured (stale)`, `given`, or why it is unknown.
    pub wcet_source: String,
    /// The worst response time (microseconds), when every task that interferes is known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_us: Option<f64>,
    /// Whether the response fits in the period; None when unknown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meets: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Analysis {
    pub target: String,
    pub tasks: Vec<Task>,
    /// Σ C/T over the tasks whose time is known, 0..1.
    pub utilisation: f64,
    /// Liu and Layland's bound for the number of tasks: n(2^(1/n) − 1).
    pub bound: f64,
    /// Every task meets its period (None: a task's time is unknown).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedulable: Option<bool>,
    pub notes: Vec<String>,
}

/// The utilisation of the tasks whose step time is known; 0 when none is
/// (an empty f64 sum is -0.0, which would print as "-0.0 %").
fn utilisation_of(tasks: &[(u64, u32, Option<f64>)]) -> f64 {
    tasks.iter().filter_map(|(period, _, c)| c.map(|c| c / *period as f64)).sum::<f64>() + 0.0
}

/// The rate-monotonic utilisation bound for `n` tasks.
pub fn rm_bound(n: usize) -> f64 {
    if n == 0 {
        return 1.0;
    }
    n as f64 * (2f64.powf(1.0 / n as f64) - 1.0)
}

/// The worst response time of task `i` under fixed priorities: its own
/// time plus the higher-priority tasks' (equal priorities interfere too:
/// the kernel may run them first), iterated until it settles or passes the
/// period (then the period plus one, past it). None when a task that
/// interferes has no known time.
pub fn response(tasks: &[(u64, u32, Option<f64>)], i: usize) -> Option<f64> {
    let (period, priority, own) = tasks[i];
    let own = own?;
    let others: Vec<(u64, f64)> = tasks.iter().enumerate().filter(|(j, (_, p, _))| *j != i && *p <= priority).map(|(_, (t, _, c))| c.map(|c| (*t, c))).collect::<Option<Vec<_>>>()?;
    let mut r = own;
    for _ in 0..10_000 {
        let next = own + others.iter().map(|(t, c)| (r / *t as f64).ceil() * c).sum::<f64>();
        if next > period as f64 {
            return Some(next);
        }
        if (next - r).abs() < 1e-9 {
            return Some(next);
        }
        r = next;
    }
    Some(r)
}

/// The analysis of the bindings deployed on `target`, the step times from
/// the tasks run together on the target (`proved`, `arclang schedule
/// --prove`: the longest a cycle took under the others' load), else the
/// partition's measurements (`(entry, fresh)`), else the `given` overrides
/// (`machine`, microseconds) — a given time wins over both.
pub fn analyse(plan: &Plan, bindings: &[Binding], target: &str, measured: &[(Entry, bool)], proved: &[(TaskRun, bool)], given: &[(String, f64)]) -> Analysis {
    let mine: Vec<&Binding> = bindings.iter().filter(|b| b.target == target).collect();
    let owned: Vec<Binding> = mine.iter().map(|b| (*b).clone()).collect();
    let priorities = binding::priorities(&owned);
    let mut notes = Vec::new();
    let mut tasks: Vec<Task> = mine
        .iter()
        .zip(&priorities)
        .map(|(b, p)| {
            let m = &plan.machines[b.machine];
            let (wcet_us, wcet_source) = if let Some((_, us)) = given.iter().find(|(name, _)| m.is(name)) {
                (Some(*us), "given".to_string())
            } else if let Some((r, fresh)) = proved.iter().find(|(r, _)| r.binding == b.name && r.target == target && r.cycles > 0) {
                (Some(r.worst_us), if *fresh { "measured under load (arclang schedule --prove)".to_string() } else { "measured under load (stale: run arclang schedule --prove)".to_string() })
            } else if let Some((e, fresh)) = measured.iter().find(|(e, _)| e.machine == m.name) {
                match &e.software {
                    Some(s) if s.target == target => (Some(s.step_us), if *fresh { "measured".to_string() } else { "measured (stale: run arclang partition)".to_string() }),
                    Some(s) => (None, format!("measured on {}, not on {target}: run arclang partition --target {target}", s.target)),
                    None => (None, e.software_error.clone().unwrap_or_else(|| "not measured".to_string())),
                }
            } else {
                (None, "not measured: run arclang partition (or give it with --wcet)".to_string())
            };
            Task { binding: b.name.clone(), machine: m.name.clone(), period_us: b.period_us, priority: *p, wcet_us, wcet_source, response_us: None, meets: None }
        })
        .collect();
    let shape: Vec<(u64, u32, Option<f64>)> = tasks.iter().map(|t| (t.period_us, t.priority, t.wcet_us)).collect();
    for i in 0..tasks.len() {
        tasks[i].response_us = response(&shape, i);
        tasks[i].meets = tasks[i].response_us.map(|r| r <= tasks[i].period_us as f64);
    }
    let utilisation = utilisation_of(&shape);
    let bound = rm_bound(tasks.len());
    let schedulable = if tasks.iter().any(|t| t.meets.is_none()) { None } else { Some(tasks.iter().all(|t| t.meets == Some(true))) };
    if tasks.is_empty() {
        notes.push(format!("no io_binding is deployed on {target}"));
    }
    if owned.iter().any(|b| b.priority.is_none()) && !tasks.is_empty() {
        notes.push("priorities by the period (rate monotonic): the shortest period the highest; write priority: in a binding to choose".to_string());
    }
    if let Some(true) = schedulable {
        if utilisation > bound {
            notes.push(format!("utilisation {:.1} % is above the rate-monotonic bound {:.1} %: the periods are met by the response times, not by the bound", utilisation * 100.0, bound * 100.0));
        }
    }
    Analysis { target: target.to_string(), tasks, utilisation, bound, schedulable, notes }
}

fn us(x: f64) -> String {
    if x >= 1000.0 {
        format!("{:.2} ms", x / 1000.0)
    } else {
        format!("{x:.1} us")
    }
}

/// The analysis as `arclang schedule` prints it.
pub fn lines(a: &Analysis) -> Vec<String> {
    let mut out = Vec::new();
    for t in &a.tasks {
        let timing = match (t.wcet_us, t.response_us) {
            (Some(c), Some(r)) => format!("step ≤ {} · response ≤ {} · {}", us(c), us(r), if t.meets == Some(true) { format!("meets its period ({:.1} % of it)", r / t.period_us as f64 * 100.0) } else { "MISSES ITS PERIOD".to_string() }),
            (Some(c), None) => format!("step ≤ {} · response unknown (a task above it has no time)", us(c)),
            (None, _) => format!("step time unknown ({})", t.wcet_source),
        };
        let source = if t.wcet_us.is_some() { format!(" [{}]", t.wcet_source) } else { String::new() };
        out.push(format!("task \"{}\" ({})  period {}  priority {}  {timing}{source}", t.binding, t.machine, us(t.period_us as f64), t.priority));
    }
    out.push(match a.schedulable {
        Some(true) => format!("utilisation {:.1} % (rate-monotonic bound {:.1} %) · every task meets its period", a.utilisation * 100.0, a.bound * 100.0),
        Some(false) => format!("utilisation {:.1} % (rate-monotonic bound {:.1} %) · a task MISSES its period", a.utilisation * 100.0, a.bound * 100.0),
        None => format!("utilisation ≥ {:.1} % over the tasks measured · a task's time is unknown: the schedule is not proved", a.utilisation * 100.0),
    });
    out.extend(a.notes.iter().map(|n| format!("note: {n}")));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_response_time_adds_the_interference_of_the_tasks_above() {
        // The textbook set: (T, C) = (50, 12.5), (40, 10), (30, 10), priorities by period.
        let tasks = [(50, 3, Some(12.5)), (40, 2, Some(10.0)), (30, 1, Some(10.0))];
        assert_eq!(response(&tasks, 2), Some(10.0));
        assert_eq!(response(&tasks, 1), Some(20.0));
        assert_eq!(response(&tasks, 0), Some(52.5), "the lowest task misses its period of 50");
        assert_eq!(response(&[(10, 1, None)], 0), None);
        assert_eq!(response(&[(10, 1, Some(1.0)), (20, 2, None)], 0), Some(1.0), "a task below does not interfere");
        assert_eq!(response(&[(10, 2, Some(1.0)), (20, 1, None)], 0), None, "a task above with no time leaves it unknown");
    }

    #[test]
    fn the_utilisation_counts_the_known_tasks_and_is_never_negative_zero() {
        assert_eq!(utilisation_of(&[(10, 1, Some(1.0)), (20, 2, Some(5.0))]), 0.35);
        let none = utilisation_of(&[(10, 1, None), (20, 2, None)]);
        assert!(none == 0.0 && none.is_sign_positive(), "no task measured: 0, not -0");
        assert_eq!(format!("{:.1}", none * 100.0), "0.0");
    }

    #[test]
    fn the_rate_monotonic_bound_is_liu_and_laylands() {
        assert!((rm_bound(1) - 1.0).abs() < 1e-12);
        assert!((rm_bound(2) - 0.8284).abs() < 1e-3);
        assert!((rm_bound(3) - 0.7798).abs() < 1e-3);
    }
}
