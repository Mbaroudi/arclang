//! A state machine as an executable configuration.
//!
//! Semantics, as UML and Capella read a machine and nothing more:
//! - each region has an initial state; entering a state enters the initial
//!   state of every region it opens, down to the leaves (entry actions in
//!   order: the state, then its regions in the order written);
//! - an event enables the transitions whose source is active, whose trigger
//!   is the event (a transition without trigger is enabled by a completion
//!   step, the event `None`) and whose guard holds;
//! - among enabled transitions an inner source wins over an outer one, then
//!   a lower `priority`, then the order of declaration; transitions of
//!   orthogonal regions fire in the same step;
//! - firing exits the active states of the transition's scope (deepest and
//!   last first, exit actions), runs the actions, then enters the targets,
//!   their ancestors under the scope and the initial states of every other
//!   region entered. A transition from a state to itself exits and re-enters
//!   it; a transition to a state inside its source (local) leaves the source
//!   active and changes only the region of the target; a transition between
//!   orthogonal regions exits and re-enters the state holding both regions
//!   (the whole machine for top-level regions).
//!
//! Pseudo-states are states written with a `kind:` (as the Capella import
//! writes them): `junction` (its branches chosen before the transition
//! fires), `choice` (chosen when reached, after the exits and actions
//! before it), `fork` (several targets in orthogonal regions), `join`
//! (several sources, fired when all are active), `shallow_history` and
//! `deep_history` (the configuration a region had when its state was last
//! exited), `entry_point`, `exit_point` (pass through) and `terminate`
//! (the machine stops, no exit actions). A branch guarded `else` is taken
//! when no other holds. See docs/embedded-c.md.
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

/// The kinds of pseudo-state the executor runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Pseudo {
    Junction,
    Choice,
    Fork,
    Join,
    ShallowHistory,
    DeepHistory,
    EntryPoint,
    ExitPoint,
    Terminate,
}

impl Pseudo {
    pub fn parse(kind: &str) -> Option<Pseudo> {
        Some(match kind {
            "junction" => Pseudo::Junction,
            "choice" => Pseudo::Choice,
            "fork" => Pseudo::Fork,
            "join" => Pseudo::Join,
            "shallow_history" => Pseudo::ShallowHistory,
            "deep_history" => Pseudo::DeepHistory,
            "entry_point" => Pseudo::EntryPoint,
            "exit_point" => Pseudo::ExitPoint,
            "terminate" => Pseudo::Terminate,
            _ => return None,
        })
    }

    pub fn is_history(self) -> bool {
        matches!(self, Pseudo::ShallowHistory | Pseudo::DeepHistory)
    }

    /// A pseudo-state a transition goes through to another state.
    fn passes(self) -> bool {
        matches!(self, Pseudo::Junction | Pseudo::EntryPoint | Pseudo::ExitPoint)
    }
}

/// A region a state (or the machine) opens: its initial state, its states.
#[derive(Debug, Clone, Serialize)]
pub struct RegionNode {
    pub name: String,
    pub initial: Option<usize>,
    pub states: Vec<usize>,
}

/// One state of the machine, in an arena (pre-order: a state, then the
/// states of its regions in the order written, each with its own).
#[derive(Debug, Clone, Serialize)]
pub struct StateNode {
    /// Its path from the top: `Armed`, `Correcting.Steering`.
    pub path: String,
    pub name: String,
    pub id: String,
    pub mode: bool,
    pub parent: Option<usize>,
    /// The initial state of its first region, when it has children.
    pub initial: Option<usize>,
    /// The states of every region it opens.
    pub children: Vec<usize>,
    /// The regions it opens (one for a composite state written without `region`).
    pub regions: Vec<RegionNode>,
    /// The region of its parent (or of the machine) it is in.
    pub region: usize,
    /// The last state of its subtree in the arena: its subtree is `index..=last`.
    pub last: usize,
    /// A pseudo-state is never active.
    pub pseudo: Option<Pseudo>,
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
    /// How long its source may stay active before it fires (`budget: 1500 ms`,
    /// or `timing:` beside a trigger): a deadline watched, not a timer.
    pub budget: Option<String>,
}

/// What is entered, in order.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum EntryOp {
    /// Enter this state alone (its entry actions run).
    Enter(usize),
    /// Run the action of this transition (a default history transition).
    Action(usize),
    /// Restore the region of a history pseudo-state; `default` when it has
    /// no history yet.
    History { pseudo: usize, default: Vec<EntryOp> },
}

/// A part of a compound transition: what it exits (ranges of the arena,
/// each exited last state first), the transitions whose actions it runs,
/// what comes next.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Segment {
    pub exit: Vec<(usize, usize)>,
    pub transitions: Vec<usize>,
    pub next: Next,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Next {
    Enter(Vec<EntryOp>),
    /// A choice: the first branch whose guards hold, when it is reached.
    Choice { pseudo: usize, branches: Vec<Branch> },
    Terminate,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Branch {
    /// Every guard of the branch (empty: unguarded or `else`).
    pub guards: Vec<String>,
    pub segment: Segment,
}

/// One way a compound transition can go, with the guards that choose it
/// before it fires (its own, a junction's branches).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Alternative {
    pub guards: Vec<String>,
    pub segment: Segment,
}

/// A transition from active states to states, through pseudo-states: what
/// fires as one.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Compound {
    /// The transition that starts it (for a join, its first incoming one).
    pub head: usize,
    pub sources: Vec<usize>,
    pub trigger: Option<String>,
    pub priority: Option<i64>,
    /// The depth of its deepest source: an inner source fires first.
    pub depth: usize,
    pub alternatives: Vec<Alternative>,
}

/// What a region of a state remembers for its history pseudo-state.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Remembered {
    Shallow(usize),
    Deep(Vec<usize>),
}

/// What one step did.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Step {
    pub event: Option<String>,
    /// Indexes of the transitions fired (each segment of a compound one), in order.
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
    /// The top-level regions.
    pub top_regions: Vec<RegionNode>,
    /// The initial state of each top-level region.
    pub tops: Vec<usize>,
    /// The transitions that fire, each with the pseudo-states it goes through.
    pub compounds: Vec<Compound>,
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
    /// History pseudo-state -> what its region was when last exited.
    pub history: BTreeMap<usize, Remembered>,
    /// Whether a `terminate` pseudo-state was reached.
    pub terminated: bool,
    /// The transitions left out because what they go through is ill-formed
    /// (a join with two outgoing transitions, a fork into one region...).
    pub refused: Vec<String>,
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
    for guard in guards.iter().map(|g| g.trim()).filter(|g| !g.is_empty() && *g != "else") {
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

/// The regions of a state as written: its `region` blocks, else one region
/// holding its sub-states.
fn regions_of(state: &State) -> Vec<(String, String, Vec<&State>)> {
    if state.regions.is_empty() {
        if state.sub_states.is_empty() {
            return Vec::new();
        }
        return vec![(String::new(), state.initial.clone(), state.sub_states.iter().collect())];
    }
    state.regions.iter().map(|r| (r.name.clone(), r.initial.clone(), r.states.iter().collect())).collect()
}

fn kind_of(state: &State) -> Option<String> {
    state.attributes.get("kind").and_then(|k| k.as_string()).map(str::to_string)
}

struct Builder<'a> {
    out: Vec<StateNode>,
    /// Arena index -> its regions as written, initial by name.
    written: Vec<Vec<(String, String, Vec<&'a State>)>>,
}

impl<'a> Builder<'a> {
    fn add(&mut self, state: &'a State, parent: Option<usize>, region: usize, prefix: &str) -> usize {
        let path = if prefix.is_empty() { state.name.clone() } else { format!("{prefix}.{}", state.name) };
        let index = self.out.len();
        self.out.push(StateNode {
            path: path.clone(),
            name: state.name.clone(),
            id: state.id.clone(),
            mode: state.kind == crate::compiler::ast::StateKind::Mode,
            parent,
            initial: None,
            children: Vec::new(),
            regions: Vec::new(),
            region,
            last: index,
            pseudo: kind_of(state).as_deref().and_then(Pseudo::parse),
            entry: state.entry_actions.clone(),
            exit: state.exit_actions.clone(),
            activity: strings(state.attributes.get("do")),
        });
        self.written.push(Vec::new());
        let regions = regions_of(state);
        let mut children = Vec::new();
        let mut nodes = Vec::new();
        for (r, (name, initial, states)) in regions.iter().enumerate() {
            let ids: Vec<usize> = states.iter().map(|s| self.add(s, Some(index), r, &path)).collect();
            children.extend(ids.iter().copied());
            nodes.push(RegionNode { name: name.clone(), initial: None, states: ids });
            let _ = initial;
        }
        self.out[index].children = children;
        self.out[index].regions = nodes;
        self.out[index].last = self.out.len() - 1;
        self.written[index] = regions;
        index
    }
}

impl Machine {
    /// Build the machine; it is not started.
    pub fn new(machine: &StateMachine) -> Machine {
        let mut notes = Vec::new();
        let mut builder = Builder { out: Vec::new(), written: Vec::new() };
        let written_tops: Vec<(String, String, Vec<&State>)> = if machine.regions.is_empty() {
            vec![(String::new(), machine.initial_state.clone(), machine.states.iter().collect())]
        } else {
            machine.regions.iter().map(|r| (r.name.clone(), r.initial.clone(), r.states.iter().collect())).collect()
        };
        let mut top_regions = Vec::new();
        for (r, (name, _, states)) in written_tops.iter().enumerate() {
            let ids: Vec<usize> = states.iter().map(|s| builder.add(s, None, r, "")).collect();
            top_regions.push(RegionNode { name: name.clone(), initial: None, states: ids });
        }
        let written = builder.written;
        let program = compile(machine).unwrap_or_else(|e| {
            notes.push(format!("{e}: the machine runs without its code"));
            Program::default()
        });
        let env = Env::new(&program, DEFAULT_PERIOD_US);
        let mut built = Machine {
            name: machine.name.clone(),
            states: builder.out,
            transitions: Vec::new(),
            top_regions,
            tops: Vec::new(),
            compounds: Vec::new(),
            active: BTreeSet::new(),
            conditions: BTreeMap::new(),
            notes: Vec::new(),
            program,
            env,
            history: BTreeMap::new(),
            terminated: false,
            refused: Vec::new(),
        };
        // The initial state of every region.
        for (r, (_, initial, _)) in written_tops.iter().enumerate() {
            let ids = built.top_regions[r].states.clone();
            let first = built.initial_among(initial, &ids, if written_tops.len() > 1 { format!("region {}", built.top_regions[r].name) } else { "the machine".to_string() }, &mut notes);
            built.top_regions[r].initial = first;
            built.tops.extend(first);
        }
        for (s, regions) in written.iter().enumerate() {
            for (r, (_, initial, _)) in regions.iter().enumerate() {
                let ids = built.states[s].regions[r].states.clone();
                let what = built.states[s].path.clone();
                let first = built.initial_among(initial, &ids, what, &mut notes);
                built.states[s].regions[r].initial = first;
                if r == 0 {
                    built.states[s].initial = first;
                }
            }
        }
        let mut conflicts = Vec::new();
        for (index, transition) in machine.transitions.iter().enumerate() {
            if !transition.trigger.trim().is_empty() && transition.attributes.contains_key("after") {
                conflicts.push(format!("transition {} -> {} has a trigger and `after:`: a transition has one trigger (an event, or after a time)", transition.from, transition.to));
            }
            match (built.resolve(&transition.from), built.resolve(&transition.to)) {
                (Some(source), Some(target)) => built.transitions.push(Self::transition(index, transition, source, target)),
                _ => notes.push(format!("transition {} -> {} names a state the machine does not declare: left out", transition.from, transition.to)),
            }
        }
        let (compounds, mut refused) = built.compound_transitions();
        refused.extend(conflicts);
        built.compounds = compounds;
        notes.extend(refused.iter().cloned());
        built.refused = refused;
        built.notes = notes;
        built
    }

    /// The initial state of a region: the one written, else its first state
    /// that is no pseudo-state (with a note).
    fn initial_among(&self, written: &str, among: &[usize], what: String, notes: &mut Vec<String>) -> Option<usize> {
        if among.is_empty() {
            return None;
        }
        if let Some(found) = self.pick_initial(written, among) {
            return Some(found);
        }
        let first = among.iter().copied().find(|&s| self.states[s].pseudo.is_none()).unwrap_or(among[0]);
        notes.push(if what == "the machine" {
            format!("no initial state among {}: the first, {}, is entered", self.states[among[0]].path, self.states[first].path)
        } else {
            format!("{what} has no initial state: its first, {}, is entered", self.states[first].path)
        });
        Some(first)
    }

    fn transition(index: usize, t: &Transition, source: usize, target: usize) -> TransitionNode {
        let nonempty = |s: &Option<String>| s.as_ref().map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
        // `timing: 5 s` is a quantity, not a string: read it as written.
        let quantity = |key: &str| t.attributes.get(key).map(|v| v.display().trim().to_string()).filter(|v| !v.is_empty());
        let timing = nonempty(&t.timing).or_else(|| quantity("timing"));
        let written = Some(t.trigger.trim().to_string()).filter(|v| !v.is_empty());
        // A timer: `after: 5 s` (SysML v2 `accept after`), or `timing:`
        // without a trigger. A budget: `budget: 5 s`, or `timing:` beside a trigger.
        let after = quantity("after").or_else(|| if written.is_none() { timing.clone() } else { None });
        let budget = quantity("budget").or_else(|| if written.is_some() { timing.clone() } else { None });
        let trigger = written.or_else(|| after.map(|a| format!("after {a}")));
        TransitionNode {
            index,
            label: if t.id.is_empty() { format!("{} -> {}", t.from, t.to) } else { t.id.clone() },
            source,
            target,
            trigger,
            guard: nonempty(&t.guard),
            action: nonempty(&t.action),
            priority: t.priority.as_deref().and_then(|p| p.trim().parse().ok()),
            budget,
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

    // --- compound transitions ----------------------------------------------------------

    fn outgoing(&self, state: usize) -> Vec<usize> {
        let mut out: Vec<usize> = (0..self.transitions.len()).filter(|&t| self.transitions[t].source == state).collect();
        // Guarded branches first by priority and order, `else` last.
        let is_else = |t: usize| self.transitions[t].guard.as_deref() == Some("else");
        out.sort_by(|&a, &b| {
            is_else(a)
                .cmp(&is_else(b))
                .then(self.transitions[a].priority.unwrap_or(i64::MAX).cmp(&self.transitions[b].priority.unwrap_or(i64::MAX)))
                .then(self.transitions[a].index.cmp(&self.transitions[b].index))
        });
        out
    }

    fn guards_of(&self, t: usize) -> Vec<String> {
        self.transitions[t].guard.iter().filter(|g| *g != "else").cloned().collect()
    }

    /// Every compound transition: one per transition leaving a state that is
    /// no pseudo-state (and not entering a join), one per join.
    fn compound_transitions(&self) -> (Vec<Compound>, Vec<String>) {
        let mut out = Vec::new();
        let mut refused = Vec::new();
        for (t, tr) in self.transitions.iter().enumerate() {
            if self.states[tr.source].pseudo.is_some() || self.states[tr.target].pseudo == Some(Pseudo::Join) {
                continue;
            }
            match self.alternatives(&[tr.source], vec![t], tr.target, self.guards_of(t), 0) {
                Ok(alternatives) => out.push(Compound { head: t, sources: vec![tr.source], trigger: tr.trigger.clone(), priority: tr.priority, depth: self.depth(tr.source), alternatives }),
                Err(e) => refused.push(format!("transition {}: {e}", tr.label)),
            }
        }
        for join in (0..self.states.len()).filter(|&s| self.states[s].pseudo == Some(Pseudo::Join)) {
            match self.join(join) {
                Ok(Some(compound)) => out.push(compound),
                Ok(None) => {}
                Err(e) => refused.push(e),
            }
        }
        (out, refused)
    }

    fn join(&self, join: usize) -> Result<Option<Compound>, String> {
        let incoming: Vec<usize> = (0..self.transitions.len()).filter(|&t| self.transitions[t].target == join).collect();
        let Some(&head) = incoming.first() else { return Ok(None) };
        let sources: Vec<usize> = incoming.iter().map(|&t| self.transitions[t].source).collect();
        if let Some(s) = sources.iter().find(|&&s| self.states[s].pseudo.is_some()) {
            return Err(format!("join {}: its incoming transition from {} comes from a pseudo-state: left out", self.states[join].path, self.states[*s].path));
        }
        let trigger = self.transitions[head].trigger.clone();
        if incoming.iter().any(|&t| self.transitions[t].trigger != trigger) {
            return Err(format!("join {}: its incoming transitions have different triggers; a join fires on one: left out", self.states[join].path));
        }
        let mut guards: Vec<String> = incoming.iter().flat_map(|&t| self.guards_of(t)).collect();
        let outgoing = self.outgoing(join);
        let [out_t] = outgoing.as_slice() else {
            return Err(format!("join {} has {} outgoing transitions; a join has one: left out", self.states[join].path, outgoing.len()));
        };
        guards.extend(self.guards_of(*out_t));
        let mut segment_transitions = incoming.clone();
        segment_transitions.push(*out_t);
        let alternatives = self.alternatives(&sources, segment_transitions, self.transitions[*out_t].target, guards, 0).map_err(|e| format!("join {}: {e}", self.states[join].path))?;
        let depth = sources.iter().map(|&s| self.depth(s)).max().unwrap_or(1);
        Ok(Some(Compound { head, sources, trigger, priority: self.transitions[head].priority, depth, alternatives }))
    }

    /// The ways a compound transition from `sources`, having run
    /// `transitions` so far under `guards`, goes on from `target`.
    fn alternatives(&self, sources: &[usize], transitions: Vec<usize>, target: usize, guards: Vec<String>, depth: usize) -> Result<Vec<Alternative>, String> {
        if depth > self.states.len() + 1 {
            return Err(format!("a transition goes round through pseudo-states without reaching a state ({}): left out", self.states[target].path));
        }
        match self.states[target].pseudo {
            Some(p) if p.passes() => {
                let mut out = Vec::new();
                let outgoing = self.outgoing(target);
                if outgoing.is_empty() {
                    return Err(format!("{} {} has no outgoing transition", kind_name(p), self.states[target].path));
                }
                for o in outgoing {
                    let mut g = guards.clone();
                    g.extend(self.guards_of(o));
                    let mut t = transitions.clone();
                    t.push(o);
                    out.extend(self.alternatives(sources, t, self.transitions[o].target, g, depth + 1)?);
                }
                Ok(out)
            }
            Some(Pseudo::Choice) => {
                let (exit, _) = self.scope(sources, &[target])?;
                let branches = self.branches(target, depth)?;
                Ok(vec![Alternative { guards, segment: Segment { exit, transitions, next: Next::Choice { pseudo: target, branches } } }])
            }
            Some(Pseudo::Fork) => {
                let outgoing = self.outgoing(target);
                let targets: Vec<usize> = outgoing.iter().map(|&o| self.transitions[o].target).collect();
                if let Some(&bad) = targets.iter().find(|&&t| self.states[t].pseudo.is_some_and(|p| !p.is_history())) {
                    return Err(format!("fork {} goes to the pseudo-state {}: a fork goes to states", self.states[target].path, self.states[bad].path));
                }
                if targets.is_empty() {
                    return Err(format!("fork {} has no outgoing transition", self.states[target].path));
                }
                let mut t = transitions;
                t.extend(outgoing);
                let (exit, entry) = self.scope(sources, &targets)?;
                Ok(vec![Alternative { guards, segment: Segment { exit, transitions: t, next: Next::Enter(entry) } }])
            }
            Some(Pseudo::Terminate) => Ok(vec![Alternative { guards, segment: Segment { exit: Vec::new(), transitions, next: Next::Terminate } }]),
            Some(Pseudo::Join) => Err(format!("join {} reached from a pseudo-state: left out", self.states[target].path)),
            _ => {
                let (exit, entry) = self.scope(sources, &[target])?;
                Ok(vec![Alternative { guards, segment: Segment { exit, transitions, next: Next::Enter(entry) } }])
            }
        }
    }

    /// The branches of a choice, each from the choice to a state.
    fn branches(&self, choice: usize, depth: usize) -> Result<Vec<Branch>, String> {
        let outgoing = self.outgoing(choice);
        if outgoing.is_empty() {
            return Err(format!("choice {} has no outgoing transition", self.states[choice].path));
        }
        let mut out = Vec::new();
        for o in outgoing {
            for alternative in self.alternatives(&[choice], vec![o], self.transitions[o].target, self.guards_of(o), depth + 1)? {
                out.push(Branch { guards: alternative.guards, segment: alternative.segment });
            }
        }
        Ok(out)
    }

    /// The children of `holder` (`None`: the top) on the way to `state`.
    fn branch_under(&self, holder: Option<usize>, state: usize) -> usize {
        let chain = self.ancestors(state);
        match holder {
            None => *chain.last().expect("a chain"),
            Some(h) => {
                let at = chain.iter().position(|&s| s == h).expect("under the holder");
                chain[at - 1]
            }
        }
    }

    /// The arena range of a region of `holder` (`None`: a top region).
    fn region_range(&self, holder: Option<usize>, region: usize) -> Option<(usize, usize)> {
        let states = match holder {
            None => &self.top_regions.get(region)?.states,
            Some(h) => &self.states[h].regions.get(region)?.states,
        };
        let first = *states.first()?;
        let last = self.states[*states.last()?].last;
        Some((first, last))
    }

    fn regions_count(&self, holder: Option<usize>) -> usize {
        match holder {
            None => self.top_regions.len(),
            Some(h) => self.states[h].regions.len(),
        }
    }

    fn region_initial(&self, holder: Option<usize>, region: usize) -> Option<usize> {
        match holder {
            None => self.top_regions.get(region)?.initial,
            Some(h) => self.states[h].regions.get(region)?.initial,
        }
    }

    /// The deepest state holding every one of `nodes` (`None`: the top).
    fn holder_of(&self, nodes: &[usize]) -> Option<usize> {
        let chains: Vec<Vec<usize>> = nodes.iter().map(|&n| self.ancestors(n)).collect();
        chains[0].iter().copied().find(|s| chains.iter().all(|c| c.contains(s)))
    }

    /// What a transition from `sources` to `targets` exits and enters.
    pub(crate) fn scope(&self, sources: &[usize], targets: &[usize]) -> Result<Scope, String> {
        // From a state to itself: out and in again.
        if let ([s], [t]) = (sources, targets) {
            if s == t {
                return Ok((vec![(*s, self.states[*s].last)], self.default_entry(*s)));
            }
        }
        let mut all = sources.to_vec();
        all.extend_from_slice(targets);
        let holder = self.holder_of(&all);
        // A target inside its source: the source stays, the regions of the targets change.
        if let (Some(h), [s]) = (holder, sources) {
            if h == *s {
                let mut exit = Vec::new();
                let mut entry = Vec::new();
                let regions: BTreeSet<usize> = targets.iter().map(|&t| self.states[self.branch_under(Some(h), t)].region).collect();
                for r in regions {
                    exit.extend(self.region_range(Some(h), r));
                    let branch = self.one_branch(Some(h), r, targets)?;
                    entry.extend(self.entry_towards(branch, targets));
                }
                return Ok((exit, entry));
            }
            // A source inside its target: out of the target and in again.
            if targets.contains(&h) {
                return Ok((vec![(h, self.states[h].last)], self.default_entry(h)));
            }
        }
        let regions: BTreeSet<usize> = all.iter().map(|&n| self.states[self.branch_under(holder, n)].region).collect();
        if regions.len() == 1 {
            let r = *regions.iter().next().expect("one");
            let exit: Vec<(usize, usize)> = self.region_range(holder, r).into_iter().collect();
            let branch = self.one_branch(holder, r, targets)?;
            return Ok((exit, self.entry_towards(branch, targets)));
        }
        // Between orthogonal regions: out of the state holding them, and in again.
        match holder {
            Some(h) => Ok((vec![(h, self.states[h].last)], self.entry_towards(h, targets))),
            None => {
                let exit = vec![(0, self.states.len() - 1)];
                let mut entry = Vec::new();
                for r in 0..self.top_regions.len() {
                    if targets.iter().any(|&t| self.states[self.branch_under(None, t)].region == r) {
                        entry.extend(self.entry_towards(self.one_branch(None, r, targets)?, targets));
                    } else if let Some(first) = self.top_regions[r].initial {
                        entry.extend(self.default_entry(first));
                    }
                }
                Ok((exit, entry))
            }
        }
    }

    /// The one state of region `r` of `holder` the targets in it are under.
    fn one_branch(&self, holder: Option<usize>, r: usize, targets: &[usize]) -> Result<usize, String> {
        let branches: BTreeSet<usize> = targets.iter().map(|&t| self.branch_under(holder, t)).filter(|&b| self.states[b].region == r).collect();
        match branches.len() {
            1 => Ok(*branches.iter().next().expect("one")),
            _ => Err(format!(
                "a transition enters {} states of one region ({}): a fork goes to states of orthogonal regions",
                branches.len(),
                branches.iter().map(|&b| self.states[b].path.clone()).collect::<Vec<_>>().join(", ")
            )),
        }
    }

    /// Enter `state` and what is under it: its initial states, down to the leaves.
    pub fn default_entry(&self, state: usize) -> Vec<EntryOp> {
        let mut out = vec![EntryOp::Enter(state)];
        for r in 0..self.states[state].regions.len() {
            if let Some(first) = self.states[state].regions[r].initial {
                out.extend(self.default_entry(first));
            }
        }
        out
    }

    /// Enter `state` (not active) on the way to `targets` under it: the
    /// regions holding a target toward it, the others by their initial state.
    fn entry_towards(&self, state: usize, targets: &[usize]) -> Vec<EntryOp> {
        if targets.contains(&state) {
            return self.target_entry(state);
        }
        let mut out = vec![EntryOp::Enter(state)];
        for r in 0..self.regions_count(Some(state)) {
            let inside: Vec<usize> = targets.iter().copied().filter(|&t| t != state && self.ancestors(t).contains(&state) && self.states[self.branch_under(Some(state), t)].region == r).collect();
            if let Some(&t) = inside.first() {
                out.extend(self.entry_towards(self.branch_under(Some(state), t), targets));
            } else if let Some(first) = self.region_initial(Some(state), r) {
                out.extend(self.default_entry(first));
            }
        }
        out
    }

    /// Enter a target: a state by default, a history pseudo-state by what it remembers.
    fn target_entry(&self, target: usize) -> Vec<EntryOp> {
        match self.states[target].pseudo {
            Some(p) if p.is_history() => {
                let holder = self.states[target].parent;
                let region = self.states[target].region;
                // The default history transition, else the region's initial state.
                let default = match self.outgoing(target).first() {
                    Some(&o) => {
                        let to = self.transitions[o].target;
                        let mut ops = vec![EntryOp::Action(o)];
                        let under = holder.is_none_or(|h| to != h && self.ancestors(to).contains(&h));
                        if under {
                            ops.extend(self.entry_towards(self.branch_under(holder, to), &[to]));
                        }
                        ops
                    }
                    None => self.region_initial(holder, region).map(|f| self.default_entry(f)).unwrap_or_default(),
                };
                vec![EntryOp::History { pseudo: target, default }]
            }
            _ => self.default_entry(target),
        }
    }

    // --- running -----------------------------------------------------------------------

    /// Enter `state` alone: active, its entry actions run.
    fn enter_one(&mut self, state: usize, step: &mut Step) {
        self.active.insert(state);
        step.entered.push(self.states[state].path.clone());
        for a in self.states[state].entry.clone() {
            step.actions.push(format!("entry {}: {a}", self.states[state].path));
            self.run(&a);
        }
    }

    fn run_entry(&mut self, ops: &[EntryOp], step: &mut Step) {
        for op in ops {
            match op {
                EntryOp::Enter(s) => self.enter_one(*s, step),
                EntryOp::Action(t) => self.run_transition(*t, step),
                EntryOp::History { pseudo, default } => match self.history.get(pseudo).cloned() {
                    Some(Remembered::Shallow(s)) => {
                        let ops = self.default_entry(s);
                        self.run_entry(&ops, step);
                    }
                    Some(Remembered::Deep(states)) => {
                        for s in states {
                            self.enter_one(s, step);
                        }
                    }
                    None => self.run_entry(default, step),
                },
            }
        }
    }

    fn run_transition(&mut self, t: usize, step: &mut Step) {
        step.fired.push(self.transitions[t].index);
        if let Some(action) = self.transitions[t].action.clone() {
            step.actions.push(format!("do {}: {action}", self.transitions[t].label));
            self.run(&action);
        }
    }

    /// The history pseudo-states of the machine, by index.
    pub fn histories(&self) -> Vec<usize> {
        (0..self.states.len()).filter(|&s| self.states[s].pseudo.is_some_and(Pseudo::is_history)).collect()
    }

    /// Exit the active states of `lo..=hi`, the last first; the history
    /// pseudo-states of the states exited remember their region first.
    fn exit_range(&mut self, lo: usize, hi: usize, step: &mut Step) {
        for h in self.histories() {
            let Some(owner) = self.states[h].parent else { continue };
            if owner < lo || owner > hi || !self.active.contains(&owner) {
                continue;
            }
            let region = self.states[h].region;
            let (first, last) = self.region_range(Some(owner), region).expect("the region of a history");
            let remembered = if self.states[h].pseudo == Some(Pseudo::ShallowHistory) {
                self.states[owner].regions[region].states.iter().copied().find(|s| self.active.contains(s)).map(Remembered::Shallow)
            } else {
                let states: Vec<usize> = (first..=last).filter(|s| self.active.contains(s)).collect();
                (!states.is_empty()).then_some(Remembered::Deep(states))
            };
            if let Some(r) = remembered {
                self.history.insert(h, r);
            }
        }
        for s in (lo..=hi).rev() {
            if self.active.remove(&s) {
                step.exited.push(self.states[s].path.clone());
                for a in self.states[s].exit.clone() {
                    step.actions.push(format!("exit {}: {a}", self.states[s].path));
                    self.run(&a);
                }
            }
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
        if !self.terminated {
            expr::flow(&self.program, &mut self.env);
        }
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

    /// Enter the initial state of every top-level region.
    pub fn start(&mut self) -> Step {
        self.active.clear();
        self.history.clear();
        self.terminated = false;
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
            let ops = self.default_entry(top);
            self.run_entry(&ops, &mut step);
        }
        step
    }

    /// Whether the guard of a transition holds.
    pub fn guard_holds(&self, guard: &Option<String>) -> bool {
        match guard.as_deref().map(str::trim) {
            None => true,
            Some("true") | Some("else") => true,
            Some("false") => false,
            Some(text) => self.holds(text),
        }
    }

    fn holds(&self, text: &str) -> bool {
        match self.program.guards.get(text) {
            Some(code) => {
                let mut env = self.env.clone();
                expr::eval(&self.program, &mut env, code) == expr::Value::Bool(true)
            }
            None => match text {
                "true" => true,
                "false" => false,
                _ => self.conditions.get(text).copied().unwrap_or(false),
            },
        }
    }

    fn all_hold(&self, guards: &[String]) -> bool {
        guards.iter().all(|g| self.holds(g))
    }

    /// Whether a segment can go on to a state now: a choice in it has a branch that holds.
    fn feasible(&self, segment: &Segment) -> bool {
        match &segment.next {
            Next::Choice { branches, .. } => branches.iter().any(|b| self.all_hold(&b.guards) && self.feasible(&b.segment)),
            _ => true,
        }
    }

    /// The conditions of the guards of the machine (those in prose): their text, set or not.
    pub fn guards(&self) -> Vec<String> {
        let mut out: Vec<String> = self.transitions.iter().filter_map(|t| t.guard.clone()).filter(|g| g != "true" && g != "false" && g != "else" && !self.program.guards.contains_key(g)).collect();
        out.sort();
        out.dedup();
        out
    }

    /// The events that would fire a transition now, guards aside.
    pub fn events(&self) -> Vec<String> {
        let mut out: Vec<String> = self.compounds.iter().filter(|c| c.sources.iter().all(|s| self.active.contains(s))).filter_map(|c| c.trigger.clone()).collect();
        out.sort();
        out.dedup();
        out
    }

    /// The transitions `event` enables now (the head of each), in the order they fire.
    pub fn enabled(&self, event: Option<&str>) -> Vec<usize> {
        self.enabled_where(event, &|_| true)
    }

    /// The transitions `event` enables now among those `keep` keeps (a timed
    /// transition whose source has been active long enough), in firing order.
    pub fn enabled_where(&self, event: Option<&str>, keep: &dyn Fn(&TransitionNode) -> bool) -> Vec<usize> {
        self.enabled_compounds(event, keep).iter().map(|(c, _)| self.transitions[self.compounds[*c].head].index).collect()
    }

    /// The compounds enabled, each with the alternative it takes, in firing order.
    pub fn enabled_compounds(&self, event: Option<&str>, keep: &dyn Fn(&TransitionNode) -> bool) -> Vec<(usize, usize)> {
        if self.terminated {
            return Vec::new();
        }
        let mut out: Vec<(usize, usize)> = Vec::new();
        for (c, compound) in self.compounds.iter().enumerate() {
            if compound.trigger.as_deref() != event || !compound.sources.iter().all(|s| self.active.contains(s)) || !keep(&self.transitions[compound.head]) {
                continue;
            }
            if let Some(a) = compound.alternatives.iter().position(|a| self.all_hold(&a.guards) && self.feasible(&a.segment)) {
                out.push((c, a));
            }
        }
        out.sort_by(|&(a, _), &(b, _)| {
            let (x, y) = (&self.compounds[a], &self.compounds[b]);
            y.depth.cmp(&x.depth).then(x.priority.unwrap_or(i64::MAX).cmp(&y.priority.unwrap_or(i64::MAX))).then(self.transitions[x.head].index.cmp(&self.transitions[y.head].index))
        });
        out
    }

    fn run_segment(&mut self, segment: &Segment, step: &mut Step) {
        for &(lo, hi) in &segment.exit {
            self.exit_range(lo, hi, step);
        }
        for &t in &segment.transitions {
            self.run_transition(t, step);
        }
        match &segment.next {
            Next::Enter(ops) => self.run_entry(ops, step),
            Next::Terminate => {
                self.active.clear();
                self.terminated = true;
            }
            Next::Choice { branches, .. } => {
                // The first branch that holds now; if the actions before made
                // none hold, the first branch.
                let chosen = branches.iter().position(|b| self.all_hold(&b.guards) && self.feasible(&b.segment)).unwrap_or(0);
                let next = branches[chosen].segment.clone();
                self.run_segment(&next, step);
            }
        }
    }

    /// Fire the transitions `event` enables (`None`: a completion step).
    pub fn fire(&mut self, event: Option<&str>) -> Step {
        self.fire_where(event, &|_| true)
    }

    /// Fire the transitions `event` enables among those `keep` keeps.
    pub fn fire_where(&mut self, event: Option<&str>, keep: &dyn Fn(&TransitionNode) -> bool) -> Step {
        let mut step = Step { event: event.map(str::to_string), ..Step::default() };
        for (c, a) in self.enabled_compounds(event, keep) {
            // A transition whose source an earlier one left is no longer enabled.
            if !self.compounds[c].sources.iter().all(|s| self.active.contains(s)) {
                continue;
            }
            let segment = self.compounds[c].alternatives[a].segment.clone();
            self.run_segment(&segment, &mut step);
        }
        if step.fired.is_empty() {
            let pending: Vec<&Compound> = self.compounds.iter().filter(|c| c.sources.iter().all(|s| self.active.contains(s)) && c.trigger.as_deref() == event).collect();
            step.idle = Some(match (event, pending.is_empty()) {
                _ if self.terminated => "the machine has terminated".to_string(),
                (Some(e), true) => format!("no transition from the active states is triggered by '{e}'"),
                (None, true) => "no transition without trigger leaves the active states".to_string(),
                (_, false) => format!(
                    "every transition it would fire has a guard that does not hold: {}",
                    pending.iter().flat_map(|c| c.alternatives.iter().flat_map(|a| a.guards.clone())).map(|g| format!("'{g}'")).collect::<Vec<_>>().join(", ")
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

fn kind_name(p: Pseudo) -> &'static str {
    match p {
        Pseudo::Junction => "junction",
        Pseudo::Choice => "choice",
        Pseudo::Fork => "fork",
        Pseudo::Join => "join",
        Pseudo::ShallowHistory => "shallow history",
        Pseudo::DeepHistory => "deep history",
        Pseudo::EntryPoint => "entry point",
        Pseudo::ExitPoint => "exit point",
        Pseudo::Terminate => "terminate",
    }
}

/// What a transition exits (ranges of the arena) and what it enters.
pub type Scope = (Vec<(usize, usize)>, Vec<EntryOp>);

/// The period the blocks integrate over until a run sets one (10 ms).
pub const DEFAULT_PERIOD_US: u64 = 10_000;

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

    const PSEUDO: &str = r#"model T { name: "t" }
state_machine "P" {
  variables: ["n: Integer = 0"]
  initial: "Idle"
  state Idle {}
  state Work {
    entry: "work in"
    region "Main" {
      initial: "Prep"
      state Prep {}
      state Run {
        initial: "Slow"
        state Slow {}
        state Fast {}
        transition Slow -> Fast { trigger: "faster" }
      }
      state Deep { kind: "deep_history" }
      state Flat { kind: "shallow_history" }
      transition Prep -> Run { trigger: "go" }
    }
    region "Aux" {
      initial: "Quiet"
      state Quiet { exit: "aux out" }
      state Busy {}
      transition Quiet -> Busy { trigger: "busy" }
    }
    transition Work -> Fast { trigger: "boost" }
    transition Slow -> Work { trigger: "up" }
  }
  state Choose { kind: "choice" }
  state Static { kind: "junction" }
  state High {}
  state Low {}
  state Stop { kind: "terminate" }
  transition Idle -> Work { trigger: "start" }
  transition Work -> Idle { trigger: "pause" }
  transition Idle -> Deep { trigger: "resume deep" }
  transition Idle -> Flat { trigger: "resume flat" }
  transition Idle -> Choose { trigger: "pick" action: "n := n + 1" }
  transition Choose -> High { guard: "n > 0" }
  transition Choose -> Low { guard: "else" }
  transition Low -> Static { trigger: "pick" action: "n := n + 1" }
  transition High -> Static { trigger: "pick" action: "n := 0" }
  transition Static -> High { guard: "n > 0" }
  transition Static -> Low { guard: "else" }
  transition High -> Idle { trigger: "back" }
  transition Low -> Idle { trigger: "back" }
  transition Idle -> Stop { trigger: "kill" }
}
"#;

    fn leaves_sorted(m: &Machine) -> Vec<String> {
        let mut l = m.leaves();
        l.sort();
        l
    }

    #[test]
    fn a_state_with_regions_enters_and_exits_all_of_them() {
        let mut m = machine(PSEUDO);
        assert!(m.notes.is_empty(), "{:?}", m.notes);
        m.start();
        let entered = m.fire(Some("start"));
        assert_eq!(entered.entered, ["Work", "Work.Prep", "Work.Quiet"]);
        assert_eq!(leaves_sorted(&m), ["Work.Prep", "Work.Quiet"]);
        // A local transition (to a state inside its source) changes only its region.
        m.fire(Some("go"));
        m.fire(Some("busy"));
        let boost = m.fire(Some("boost"));
        assert_eq!(boost.exited, ["Work.Run.Slow", "Work.Run"]);
        assert_eq!(boost.entered, ["Work.Run", "Work.Run.Fast"]);
        assert_eq!(leaves_sorted(&m), ["Work.Busy", "Work.Run.Fast"]);
        // To an ancestor: out of it and in again, by its initial states.
        m.fire(Some("pause"));
        m.fire(Some("start"));
        m.fire(Some("go"));
        let up = m.fire(Some("up"));
        assert_eq!(up.exited, ["Work.Quiet", "Work.Run.Slow", "Work.Run", "Work"], "the last state first");
        assert_eq!(up.entered, ["Work", "Work.Prep", "Work.Quiet"]);
    }

    #[test]
    fn deep_history_restores_the_leaves_shallow_history_the_state() {
        let mut m = machine(PSEUDO);
        m.start();
        // No history yet: the region's initial state.
        assert_eq!(m.fire(Some("resume deep")).entered, ["Work", "Work.Prep", "Work.Quiet"]);
        m.fire(Some("go"));
        m.fire(Some("faster"));
        m.fire(Some("busy"));
        m.fire(Some("pause"));
        let deep = m.fire(Some("resume deep"));
        assert_eq!(deep.entered, ["Work", "Work.Run", "Work.Run.Fast", "Work.Quiet"], "the deep history of Main, Aux by its initial state");
        m.fire(Some("pause"));
        let flat = m.fire(Some("resume flat"));
        assert_eq!(flat.entered, ["Work", "Work.Run", "Work.Run.Slow", "Work.Quiet"], "shallow: Run, entered by its initial state");
    }

    #[test]
    fn a_choice_sees_the_action_before_it_a_junction_does_not() {
        let mut m = machine(PSEUDO);
        m.start();
        // Choice: n is 1 when it is reached.
        let pick = m.fire(Some("pick"));
        assert_eq!(pick.entered, ["High"]);
        assert_eq!(m.env.values[0], expr::Value::Int(1));
        // Junction: chosen before High -> Static runs `n := 0`: n > 0 held.
        let again = m.fire(Some("pick"));
        assert_eq!(again.entered, ["High"]);
        assert_eq!(m.env.values[0], expr::Value::Int(0));
        // Now n is 0 before: the else branch.
        assert_eq!(m.fire(Some("pick")).entered, ["Low"]);
    }

    #[test]
    fn terminate_stops_the_machine_without_exit_actions() {
        let mut m = machine(PSEUDO);
        m.start();
        let kill = m.fire(Some("kill"));
        assert!(kill.exited.is_empty() && m.active.is_empty() && m.terminated, "{kill:?}");
        let after = m.fire(Some("start"));
        assert!(after.fired.is_empty() && after.idle.as_deref() == Some("the machine has terminated"));
    }

    #[test]
    fn entry_and_exit_points_pass_through() {
        let mut m = machine(r#"model T { name: "t" }
state_machine "E" {
  initial: "Out"
  state Out {}
  state In {
    initial: "A"
    state A {}
    state B {}
    state Enter { kind: "entry_point" }
    state Leave { kind: "exit_point" }
    transition Enter -> B { action: "through entry" }
    transition B -> Leave { trigger: "done" }
  }
  transition Out -> Enter { trigger: "go" }
  transition Leave -> Out { action: "through exit" }
}
"#);
        assert!(m.notes.is_empty(), "{:?}", m.notes);
        m.start();
        let go = m.fire(Some("go"));
        assert_eq!(go.entered, ["In", "In.B"]);
        assert_eq!(go.actions, ["do Enter -> B: through entry"]);
        let done = m.fire(Some("done"));
        assert_eq!(done.exited, ["In.B", "In"]);
        assert_eq!(done.entered, ["Out"]);
        assert_eq!(done.fired.len(), 2);
    }
}

