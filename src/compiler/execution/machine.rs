//! A state machine as an executable configuration.
//!
//! Semantics, as UML and Capella read a machine and nothing more:
//! - each region has an initial state; entering a state enters the initial
//!   state of the region it opens, down to a leaf (entry actions in order);
//! - an event enables the transitions whose source is active, whose trigger
//!   is the event (a transition without trigger is enabled by a completion
//!   step, the event `None`) and whose guard holds;
//! - among enabled transitions an inner source wins over an outer one, then
//!   a lower `priority`, then the order of declaration; transitions of
//!   orthogonal regions fire in the same step;
//! - firing exits the active states under the source (deepest first, exit
//!   actions), runs the action, then enters the target and its initial
//!   states. A transition from a state to itself exits and re-enters it.
//!
//! A machine may declare typed variables and equations (`variables:`,
//! `equations:`, see `expr`): a guard that is an expression over them is
//! evaluated, an action that assigns them is executed when it runs (exit,
//! transition, entry actions, in that order). Guards and actions in prose
//! stay conditions the reader sets and actions reported.

use super::expr::{self, Env, Program};
use crate::compiler::ast::{AttributeValue, State, StateMachine, Transition};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

/// One state of the machine, in an arena.
#[derive(Debug, Clone, Serialize)]
pub struct StateNode {
    /// Its path from the top: `Armed`, `Correcting.Steering`.
    pub path: String,
    pub name: String,
    pub id: String,
    pub mode: bool,
    pub parent: Option<usize>,
    /// The initial child of the region it opens, when it has children.
    pub initial: Option<usize>,
    pub children: Vec<usize>,
    pub entry: Vec<String>,
    pub exit: Vec<String>,
    /// Its `do:` activities, as written. The debugger does not run them;
    /// a fixed-step run (`embedded`) counts them once per step the state is active.
    pub activity: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransitionNode {
    pub index: usize,
    /// The written id, else `from -> to`.
    pub label: String,
    pub source: usize,
    pub target: usize,
    pub trigger: Option<String>,
    pub guard: Option<String>,
    pub action: Option<String>,
    pub priority: Option<i64>,
}

/// What one step did.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Step {
    pub event: Option<String>,
    /// Indexes of the transitions fired, in order.
    pub fired: Vec<usize>,
    /// Paths of the states exited, deepest first, then entered, outermost first.
    pub exited: Vec<String>,
    pub entered: Vec<String>,
    /// Actions run, in order: exits, transition actions, entries.
    pub actions: Vec<String>,
    /// Why nothing fired, when nothing did.
    pub idle: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Machine {
    pub name: String,
    pub states: Vec<StateNode>,
    pub transitions: Vec<TransitionNode>,
    /// The initial state of each top-level region.
    pub tops: Vec<usize>,
    /// Active states, ancestors included.
    pub active: BTreeSet<usize>,
    /// The value the reader gave each guard; a guard never set is false.
    pub conditions: BTreeMap<String, bool>,
    /// What the machine says that the compiler did not decide (an initial
    /// state missing, a reference that names no state).
    pub notes: Vec<String>,
    /// Its variables, equations, guards and actions as code.
    pub program: Program,
    /// The values of its variables and the states of its blocks.
    pub env: Env,
}

fn strings(value: Option<&AttributeValue>) -> Vec<String> {
    match value {
        Some(AttributeValue::List(items)) => items.iter().map(|i| i.as_string().map(str::to_string).unwrap_or_else(|| i.display())).collect(),
        Some(AttributeValue::String(one)) => vec![one.clone()],
        Some(other) => vec![other.display()],
        None => Vec::new(),
    }
}

/// The program of a machine: its `variables:` and `equations:`, and every
/// guard and action that is code. An error is a mistake in the code (a
/// type, a unit, a name).
pub fn compile(machine: &StateMachine) -> Result<Program, String> {
    let what = format!("state_machine '{}'", machine.name);
    let mut program = expr::program(&strings(machine.attributes.get("variables")), &strings(machine.attributes.get("equations"))).map_err(|e| format!("{what}: {e}"))?;
    if program.is_empty() {
        return Ok(program);
    }
    fn states_of<'a>(states: &'a [State], out: &mut Vec<&'a State>) {
        for s in states {
            out.push(s);
            states_of(&s.sub_states, out);
        }
    }
    let mut states = Vec::new();
    states_of(&machine.states, &mut states);
    let mut actions: Vec<String> = Vec::new();
    for s in &states {
        actions.extend(s.entry_actions.iter().chain(&s.exit_actions).cloned());
        actions.extend(strings(s.attributes.get("do")));
    }
    let mut guards = Vec::new();
    for t in &machine.transitions {
        actions.extend(t.action.iter().cloned());
        guards.extend(t.guard.iter().cloned());
    }
    for guard in guards.iter().map(|g| g.trim()).filter(|g| !g.is_empty()) {
        if let Some(e) = program.guard(guard).map_err(|e| format!("{what}: {e}"))? {
            program.guards.insert(guard.to_string(), e);
        }
    }
    for action in actions.iter().map(|a| a.trim()).filter(|a| !a.is_empty()) {
        if let Some(code) = program.action(action).map_err(|e| format!("{what}: {e}"))? {
            program.actions.insert(action.to_string(), code);
        }
    }
    Ok(program)
}

fn add_states(states: &[State], parent: Option<usize>, prefix: &str, out: &mut Vec<StateNode>) -> Vec<usize> {
    let mut ids = Vec::new();
    for state in states {
        let path = if prefix.is_empty() { state.name.clone() } else { format!("{prefix}.{}", state.name) };
        let index = out.len();
        out.push(StateNode {
            path: path.clone(),
            name: state.name.clone(),
            id: state.id.clone(),
            mode: state.kind == crate::compiler::ast::StateKind::Mode,
            parent,
            initial: None,
            children: Vec::new(),
            entry: state.entry_actions.clone(),
            exit: state.exit_actions.clone(),
            activity: match state.attributes.get("do") {
                Some(crate::compiler::ast::AttributeValue::List(items)) => items.iter().filter_map(|i| i.as_string()).map(str::to_string).collect(),
                Some(crate::compiler::ast::AttributeValue::String(one)) => vec![one.clone()],
                _ => Vec::new(),
            },
        });
        let children = add_states(&state.sub_states, Some(index), &path, out);
        out[index].children = children;
        ids.push(index);
    }
    ids
}

/// The period the blocks integrate over until a run sets one (10 ms).
pub const DEFAULT_PERIOD_US: u64 = 10_000;

impl Machine {
    /// Build the machine; it is not started.
    pub fn new(machine: &StateMachine) -> Machine {
        let mut states = Vec::new();
        let mut notes = Vec::new();
        let top: Vec<usize> = if machine.regions.is_empty() { add_states(&machine.states, None, "", &mut states) } else { Vec::new() };
        let mut regions: Vec<(String, Vec<usize>)> = Vec::new();
        if machine.regions.is_empty() {
            regions.push((machine.initial_state.clone(), top));
        } else {
            for region in &machine.regions {
                let ids = add_states(&region.states, None, "", &mut states);
                regions.push((region.initial.clone(), ids));
            }
        }
        let program = compile(machine).unwrap_or_else(|e| {
            notes.push(format!("{e}: the machine runs without its code"));
            Program::default()
        });
        let env = Env::new(&program, DEFAULT_PERIOD_US);
        let mut built = Machine { name: machine.name.clone(), states, transitions: Vec::new(), tops: Vec::new(), active: BTreeSet::new(), conditions: BTreeMap::new(), notes: Vec::new(), program, env };
        for (initial, ids) in &regions {
            match built.pick_initial(initial, ids) {
                Some(first) => built.tops.push(first),
                None if !ids.is_empty() => {
                    notes.push(format!("no initial state among {}: the first, {}, is entered", built.states[ids[0]].path, built.states[ids[0]].path));
                    built.tops.push(ids[0]);
                }
                None => {}
            }
        }
        let initials: Vec<(usize, Option<usize>)> = (0..built.states.len()).map(|i| (i, None)).collect();
        let mut chosen = initials;
        Self::initials_of(&machine.states, &built, "", &mut chosen, &mut notes);
        for region in &machine.regions {
            Self::initials_of(&region.states, &built, "", &mut chosen, &mut notes);
        }
        for (i, first) in chosen {
            built.states[i].initial = first;
        }
        for (index, transition) in machine.transitions.iter().enumerate() {
            match (built.resolve(&transition.from), built.resolve(&transition.to)) {
                (Some(source), Some(target)) => built.transitions.push(Self::transition(index, transition, source, target)),
                _ => notes.push(format!("transition {} -> {} names a state the machine does not declare: left out", transition.from, transition.to)),
            }
        }
        built.notes = notes;
        built
    }

    fn transition(index: usize, t: &Transition, source: usize, target: usize) -> TransitionNode {
        let nonempty = |s: &Option<String>| s.as_ref().map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
        // `timing: 5 s` is a quantity, not a string: read it as written.
        let timing = nonempty(&t.timing).or_else(|| t.attributes.get("timing").map(|v| v.display().trim().to_string()).filter(|v| !v.is_empty()));
        let trigger = Some(t.trigger.trim().to_string()).filter(|v| !v.is_empty()).or_else(|| timing.map(|timing| format!("after {timing}")));
        TransitionNode {
            index,
            label: if t.id.is_empty() { format!("{} -> {}", t.from, t.to) } else { t.id.clone() },
            source,
            target,
            trigger,
            guard: nonempty(&t.guard),
            action: nonempty(&t.action),
            priority: t.priority.as_deref().and_then(|p| p.trim().parse().ok()),
        }
    }

    /// The initial child of every composite state, by its `initial:`.
    fn initials_of(states: &[State], built: &Machine, prefix: &str, chosen: &mut [(usize, Option<usize>)], notes: &mut Vec<String>) {
        for state in states {
            let path = if prefix.is_empty() { state.name.clone() } else { format!("{prefix}.{}", state.name) };
            if let Some(index) = built.states.iter().position(|s| s.path == path) {
                let children = built.states[index].children.clone();
                if !children.is_empty() {
                    let first = built.pick_initial(&state.initial, &children).unwrap_or_else(|| {
                        notes.push(format!("{path} has no initial state: its first, {}, is entered", built.states[children[0]].path));
                        children[0]
                    });
                    chosen[index].1 = Some(first);
                }
            }
            Self::initials_of(&state.sub_states, built, &path, chosen, notes);
        }
    }

    fn pick_initial(&self, written: &str, among: &[usize]) -> Option<usize> {
        let written = written.trim();
        if written.is_empty() {
            return None;
        }
        among.iter().copied().find(|&i| self.states[i].name == written || self.states[i].id == written || self.states[i].path == written)
    }

    /// The state a reference names: its path, its id, or a name only one state has.
    pub fn resolve(&self, reference: &str) -> Option<usize> {
        let reference = reference.trim();
        if let Some(i) = self.states.iter().position(|s| s.path == reference || (!s.id.is_empty() && s.id == reference)) {
            return Some(i);
        }
        let named: Vec<usize> = (0..self.states.len()).filter(|&i| self.states[i].name == reference || self.states[i].path.ends_with(&format!(".{reference}"))).collect();
        match named.as_slice() {
            [one] => Some(*one),
            _ => None,
        }
    }

    pub(crate) fn ancestors(&self, state: usize) -> Vec<usize> {
        let mut out = vec![state];
        let mut at = state;
        while let Some(parent) = self.states[at].parent {
            out.push(parent);
            at = parent;
        }
        out
    }

    pub(crate) fn depth(&self, state: usize) -> usize {
        self.ancestors(state).len()
    }

    /// Enter `state` (its ancestors are active) and the initial states under it.
    fn enter(&mut self, state: usize, step: &mut Step) {
        self.enter_one(state, step);
        if let Some(first) = self.states[state].initial {
            self.enter(first, step);
        }
    }

    /// Enter `state` alone: active, its entry actions run.
    fn enter_one(&mut self, state: usize, step: &mut Step) {
        self.active.insert(state);
        step.entered.push(self.states[state].path.clone());
        for a in self.states[state].entry.clone() {
            step.actions.push(format!("entry {}: {a}", self.states[state].path));
            self.run(&a);
        }
    }

    /// Run an action that is code (nothing for prose).
    pub fn run(&mut self, action: &str) {
        if let Some(code) = self.program.actions.get(action.trim()) {
            expr::execute(&self.program, &mut self.env, code);
        }
    }

    /// Evaluate the equations (the data flow of one step).
    pub fn flow(&mut self) {
        expr::flow(&self.program, &mut self.env);
    }

    /// Run the `do:` activities of the active states, in the order of the
    /// states; what ran, as (state, activity).
    pub fn activities(&mut self) -> Vec<(usize, String)> {
        let mut ran = Vec::new();
        for s in 0..self.states.len() {
            if self.active.contains(&s) {
                for a in self.states[s].activity.clone() {
                    self.run(&a);
                    ran.push((s, a));
                }
            }
        }
        ran
    }

    /// The step period the blocks integrate over.
    pub fn set_period(&mut self, period_us: u64) {
        self.env.period = expr::period_seconds(period_us);
    }

    /// Exit `state` and every active state under it, deepest first.
    fn exit(&mut self, state: usize, step: &mut Step) {
        for child in self.states[state].children.clone() {
            if self.active.contains(&child) {
                self.exit(child, step);
            }
        }
        if self.active.remove(&state) {
            step.exited.push(self.states[state].path.clone());
            for a in self.states[state].exit.clone() {
                step.actions.push(format!("exit {}: {a}", self.states[state].path));
                self.run(&a);
            }
        }
    }

    /// Enter the initial state of every top-level region.
    pub fn start(&mut self) -> Step {
        self.active.clear();
        // The variables start again from their initial values; the inputs
        // keep what the caller set.
        let mut env = Env::new(&self.program, DEFAULT_PERIOD_US);
        env.period = self.env.period;
        for (i, _) in self.program.inputs() {
            env.values[i] = self.env.values[i];
        }
        self.env = env;
        let mut step = Step::default();
        for top in self.tops.clone() {
            self.enter(top, &mut step);
        }
        step
    }

    /// Whether the guard of a transition holds.
    pub fn guard_holds(&self, guard: &Option<String>) -> bool {
        match guard.as_deref().map(str::trim) {
            None => true,
            Some("true") => true,
            Some("false") => false,
            Some(text) => match self.program.guards.get(text) {
                Some(code) => {
                    let mut env = self.env.clone();
                    expr::eval(&self.program, &mut env, code) == expr::Value::Bool(true)
                }
                None => self.conditions.get(text).copied().unwrap_or(false),
            },
        }
    }

    /// The conditions of the guards of the machine (those in prose): their text, set or not.
    pub fn guards(&self) -> Vec<String> {
        let mut out: Vec<String> = self.transitions.iter().filter_map(|t| t.guard.clone()).filter(|g| g != "true" && g != "false" && !self.program.guards.contains_key(g)).collect();
        out.sort();
        out.dedup();
        out
    }

    /// The events that would fire a transition now, guards aside.
    pub fn events(&self) -> Vec<String> {
        let mut out: Vec<String> = self.transitions.iter().filter(|t| self.active.contains(&t.source)).filter_map(|t| t.trigger.clone()).collect();
        out.sort();
        out.dedup();
        out
    }

    /// The transitions `event` enables now, in the order they fire.
    pub fn enabled(&self, event: Option<&str>) -> Vec<usize> {
        self.enabled_where(event, &|_| true)
    }

    /// The transitions `event` enables now among those `keep` keeps (a timed
    /// transition whose source has been active long enough), in firing order.
    pub fn enabled_where(&self, event: Option<&str>, keep: &dyn Fn(&TransitionNode) -> bool) -> Vec<usize> {
        let mut candidates: Vec<&TransitionNode> = self
            .transitions
            .iter()
            .filter(|t| self.active.contains(&t.source) && t.trigger.as_deref() == event && self.guard_holds(&t.guard) && keep(t))
            .collect();
        candidates.sort_by(|a, b| {
            self.depth(b.source)
                .cmp(&self.depth(a.source))
                .then(a.priority.unwrap_or(i64::MAX).cmp(&b.priority.unwrap_or(i64::MAX)))
                .then(a.index.cmp(&b.index))
        });
        candidates.iter().map(|t| t.index).collect()
    }

    /// The state under the least common ancestor of `a` and `b` that holds
    /// `a`, and the ancestors of `b` under it, outermost first.
    pub(crate) fn scope(&self, source: usize, target: usize) -> (usize, Vec<usize>) {
        if source == target {
            return (source, Vec::new());
        }
        let up_source = self.ancestors(source);
        let up_target = self.ancestors(target);
        let common = up_source.iter().copied().find(|s| up_target.contains(s));
        let below = |chain: &[usize]| -> Vec<usize> {
            match common {
                Some(c) => chain.iter().copied().take_while(|&s| s != c).collect(),
                None => chain.to_vec(),
            }
        };
        let exit_from = *below(&up_source).last().unwrap_or(&source);
        let mut path = below(&up_target);
        path.reverse();
        // A target inside the source: the source stays, only what is under it changes.
        if common == Some(source) {
            return (source, path);
        }
        (exit_from, path)
    }

    /// Fire the transitions `event` enables (`None`: a completion step).
    pub fn fire(&mut self, event: Option<&str>) -> Step {
        self.fire_where(event, &|_| true)
    }

    /// Fire the transitions `event` enables among those `keep` keeps.
    pub fn fire_where(&mut self, event: Option<&str>, keep: &dyn Fn(&TransitionNode) -> bool) -> Step {
        let mut step = Step { event: event.map(str::to_string), ..Step::default() };
        for index in self.enabled_where(event, keep) {
            let Some(t) = self.transitions.iter().find(|t| t.index == index).cloned() else { continue };
            // A transition of a region an earlier one left is no longer enabled.
            if !self.active.contains(&t.source) {
                continue;
            }
            let (exit_from, entry_path) = self.scope(t.source, t.target);
            let inner = t.source != t.target && self.ancestors(t.target).contains(&t.source);
            if inner {
                for child in self.states[t.source].children.clone() {
                    if self.active.contains(&child) {
                        self.exit(child, &mut step);
                    }
                }
            } else {
                self.exit(exit_from, &mut step);
            }
            if let Some(action) = &t.action {
                step.actions.push(format!("do {}: {action}", t.label));
                self.run(action);
            }
            if t.source == t.target {
                self.enter(t.target, &mut step);
            } else {
                // Enter down the target's ancestors, then the target and its initial states.
                for (n, &state) in entry_path.iter().enumerate() {
                    if n + 1 == entry_path.len() {
                        self.enter(state, &mut step);
                    } else {
                        self.enter_one(state, &mut step);
                    }
                }
            }
            step.fired.push(index);
        }
        if step.fired.is_empty() {
            let pending: Vec<&TransitionNode> = self.transitions.iter().filter(|t| self.active.contains(&t.source) && t.trigger.as_deref() == event).collect();
            step.idle = Some(match (event, pending.is_empty()) {
                (Some(e), true) => format!("no transition from the active states is triggered by '{e}'"),
                (None, true) => "no transition without trigger leaves the active states".to_string(),
                (_, false) => format!(
                    "every transition it would fire has a guard that does not hold: {}",
                    pending.iter().filter_map(|t| t.guard.clone()).map(|g| format!("'{g}'")).collect::<Vec<_>>().join(", ")
                ),
            });
        }
        step
    }

    /// The active leaves, by path.
    pub fn leaves(&self) -> Vec<String> {
        self.active.iter().filter(|&&s| !self.states[s].children.iter().any(|c| self.active.contains(c))).map(|&s| self.states[s].path.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{Compiler, CompilerConfig};

    fn machine(source: &str) -> Machine {
        let result = Compiler::new(CompilerConfig::default()).compile_string(source).expect("compiles");
        Machine::new(&result.ast.state_machines[0])
    }

    const LKA: &str = r#"model T { name: "t" }
state_machine "LKA modes" {
  initial: "Off"
  mode Off { description: "Not armed" }
  mode Armed { description: "Monitoring the lane" }
  mode Correcting { description: "Steering correction active" }
  transition Off -> Armed { trigger: "SF-DET" }
  transition Armed -> Correcting { trigger: "SF-EST" guard: "departure imminent" }
  transition Correcting -> Armed { trigger: "SF-CMD" }
  transition Correcting -> Off { trigger: "SF-CMD" guard: "driver override" }
}
"#;

    #[test]
    fn events_fire_transitions_and_guards_are_conditions() {
        let mut m = machine(LKA);
        assert!(m.notes.is_empty(), "{:?}", m.notes);
        let start = m.start();
        assert_eq!(start.entered, ["Off"]);
        assert_eq!(m.events(), ["SF-DET"]);
        let armed = m.fire(Some("SF-DET"));
        assert_eq!((armed.exited.as_slice(), armed.entered.as_slice()), (&["Off".to_string()][..], &["Armed".to_string()][..]));
        // The guard is the author's text: false until the reader sets it.
        let idle = m.fire(Some("SF-EST"));
        assert!(idle.fired.is_empty() && idle.idle.as_deref().unwrap().contains("'departure imminent'"), "{idle:?}");
        assert_eq!(m.guards(), ["departure imminent", "driver override"]);
        m.conditions.insert("departure imminent".into(), true);
        assert_eq!(m.fire(Some("SF-EST")).entered, ["Correcting"]);
        // Two transitions on SF-CMD: the unguarded one, the override not set.
        assert_eq!(m.fire(Some("SF-CMD")).entered, ["Armed"]);
        let none = m.fire(Some("nothing"));
        assert!(none.idle.unwrap().contains("no transition from the active states is triggered by 'nothing'"));
    }

    #[test]
    fn composite_states_enter_their_initial_state_and_exit_deepest_first() {
        let mut m = machine(r#"model T { name: "t" }
state_machine "Charger" {
  initial: "Idle"
  state Idle { exit: "lock connector" }
  state Charging {
    initial: "Precharge"
    entry: "close contactors"
    exit: "open contactors"
    state Precharge {}
    state Bulk { entry: "ramp current" }
    transition Precharge -> Bulk { trigger: "ready" }
  }
  transition Idle -> Charging { trigger: "plug" action: "authorize" }
  transition Charging -> Idle { trigger: "unplug" }
  transition Charging -> Charging { trigger: "reset" }
}
"#);
        assert!(m.notes.is_empty(), "{:?}", m.notes);
        m.start();
        let plugged = m.fire(Some("plug"));
        assert_eq!(plugged.entered, ["Charging", "Charging.Precharge"]);
        assert_eq!(plugged.actions, ["exit Idle: lock connector", "do Idle -> Charging: authorize", "entry Charging: close contactors"]);
        assert_eq!(m.fire(Some("ready")).entered, ["Charging.Bulk"]);
        assert_eq!(m.leaves(), ["Charging.Bulk"]);
        let reset = m.fire(Some("reset"));
        assert_eq!(reset.exited, ["Charging.Bulk", "Charging"]);
        assert_eq!(reset.entered, ["Charging", "Charging.Precharge"]);
        let unplugged = m.fire(Some("unplug"));
        assert_eq!(unplugged.exited, ["Charging.Precharge", "Charging"]);
        assert_eq!(unplugged.actions, ["exit Charging: open contactors"]);
        assert_eq!(m.leaves(), ["Idle"]);
    }

    #[test]
    fn orthogonal_regions_fire_in_the_same_step() {
        let mut m = machine(r#"model T { name: "t" }
state_machine "Vehicle" {
  region "Power" {
    initial: "Off"
    state Off {}
    state On {}
    transition Off -> On { trigger: "start" }
  }
  region "Lights" {
    initial: "Dark"
    state Dark {}
    state Lit {}
    transition Dark -> Lit { trigger: "start" }
  }
}
"#);
        assert!(m.notes.is_empty(), "{:?}", m.notes);
        assert_eq!(m.start().entered, ["Off", "Dark"]);
        let step = m.fire(Some("start"));
        assert_eq!(step.fired.len(), 2);
        let mut leaves = m.leaves();
        leaves.sort();
        assert_eq!(leaves, ["Lit", "On"]);
    }

    #[test]
    fn a_timing_without_trigger_is_a_timed_event_and_do_activities_are_read() {
        let mut m = machine(r#"model T { name: "t" }
state_machine "Charger" {
  initial: "Precharge"
  state Precharge { do: "limit current" }
  state Bulk {}
  transition Precharge -> Bulk { timing: 200 ms }
}
"#);
        assert_eq!(m.transitions[0].trigger.as_deref(), Some("after 200 ms"));
        assert_eq!(m.states[0].activity, ["limit current"]);
        m.start();
        // Not a completion transition: a step without event leaves it alone.
        assert!(m.fire(None).fired.is_empty());
        assert!(m.fire_where(Some("after 200 ms"), &|_| false).fired.is_empty());
        assert_eq!(m.fire(Some("after 200 ms")).entered, ["Bulk"]);
    }
}
