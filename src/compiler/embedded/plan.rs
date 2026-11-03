//! A state machine lowered to what fixed-step code needs: every state, every
//! transition with what it exits and enters, computed once from the
//! executor of the debugger (`execution::Machine`), so the generated C, the
//! model-level run and the debugger read a machine the same way.
//!
//! The inputs of a machine are its events (the triggers of its transitions)
//! and its conditions (the guards as written, `true` and `false` aside): the
//! compiler cannot decide a guard written in prose, so the caller sets it.
//! The outputs are its states (active or not) and its actions (how many
//! times each ran in the step): an action written in prose is not code, so
//! the generated code reports it and does nothing else.

use crate::compiler::ast::{Model, StateMachine};
use crate::compiler::c_header_generator::c_ident;
use crate::compiler::execution::Machine;
use crate::compiler::quantity::{Dimension, Quantity};
use std::collections::BTreeSet;

/// An input or output of a machine: the text as written and its C name.
#[derive(Debug, Clone, PartialEq)]
pub struct Signal {
    pub text: String,
    pub ident: String,
    /// Its name in a result column (`action.close_contactors`).
    pub member: String,
}

/// A timed trigger (`timing: 5 s` without a trigger): `after 5 s`.
#[derive(Debug, Clone, PartialEq)]
pub struct Timer {
    pub label: String,
    pub ident: String,
    pub micros: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Guard {
    Always,
    Never,
    Condition(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Trigger {
    Event(usize),
    Timer(usize),
    Completion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// From a state to itself: it is exited and entered again.
    SelfLoop,
    /// To a state inside the source: what is under the source is exited.
    Inner,
    /// Anywhere else: the source's branch under the common ancestor is exited.
    Outer,
}

#[derive(Debug, Clone)]
pub struct StatePlan {
    pub path: String,
    /// Its result column under the machine (`Charging.Bulk`).
    pub member: String,
    pub ident: String,
    pub parent: Option<usize>,
    pub initial: Option<usize>,
    pub children: Vec<usize>,
    pub entry: Vec<usize>,
    pub exit: Vec<usize>,
    pub activity: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct TransitionPlan {
    /// Its index among the machine's transitions, as declared.
    pub index: usize,
    pub label: String,
    pub source: usize,
    pub target: usize,
    pub kind: Kind,
    pub exit_from: usize,
    /// Entered in order; the last is entered with its initial states.
    pub entry_path: Vec<usize>,
    pub action: Option<usize>,
    pub guard: Guard,
    pub trigger: Trigger,
}

#[derive(Debug, Clone)]
pub struct MachinePlan {
    pub name: String,
    pub id: String,
    pub ident: String,
    pub machine: Machine,
    pub states: Vec<StatePlan>,
    pub transitions: Vec<TransitionPlan>,
    pub events: Vec<Signal>,
    pub conditions: Vec<Signal>,
    pub actions: Vec<Signal>,
    pub timers: Vec<Timer>,
    /// What fires in a step, in order: each trigger and its transitions in
    /// the order they fire (inner source first, then priority, then declaration).
    pub dispatch: Vec<(Trigger, Vec<usize>)>,
    pub max_depth: usize,
    pub notes: Vec<String>,
}

impl MachinePlan {
    /// Whether `reference` names this machine (its name or its id).
    pub fn is(&self, reference: &str) -> bool {
        self.name == reference || (!self.id.is_empty() && self.id == reference) || self.ident == reference
    }

    pub fn event(&self, text: &str) -> Option<usize> {
        self.events.iter().position(|e| e.text == text)
    }

    pub fn condition(&self, text: &str) -> Option<usize> {
        self.conditions.iter().position(|c| c.text == text)
    }

    pub fn action(&self, text: &str) -> Option<usize> {
        self.actions.iter().position(|a| a.text == text)
    }

    /// The state a reference names: its path, its id, or a name only one state has.
    pub fn state(&self, reference: &str) -> Option<usize> {
        self.machine.resolve(reference)
    }
}

/// Every machine of a model, lowered.
#[derive(Debug, Clone)]
pub struct Plan {
    pub model: String,
    pub ident: String,
    pub machines: Vec<MachinePlan>,
}

const C_KEYWORDS: [&str; 37] = [
    "auto", "break", "case", "char", "const", "continue", "default", "do", "double", "else", "enum", "extern", "float", "for", "goto", "if", "inline", "int", "long", "register", "restrict", "return", "short", "signed", "sizeof", "static", "struct", "switch", "typedef", "union", "unsigned", "void", "volatile", "while", "bool", "true", "false",
];

/// A C identifier that is no keyword.
pub fn c_name(raw: &str) -> String {
    let ident = c_ident(raw);
    if C_KEYWORDS.contains(&ident.as_str()) {
        format!("{ident}_")
    } else {
        ident
    }
}

/// A name for a result column: the text with its case, other characters `_`.
pub fn member(raw: &str) -> String {
    let mut out = String::new();
    for c in raw.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('_') {
            out.push('_');
        }
    }
    let out = out.trim_matches('_').to_string();
    match out.chars().next() {
        None => "x".to_string(),
        Some(c) if c.is_ascii_digit() => format!("_{out}"),
        _ => out,
    }
}

fn unique(taken: &mut BTreeSet<String>, wanted: String) -> String {
    let mut name = wanted.clone();
    let mut n = 2;
    while !taken.insert(name.clone()) {
        name = format!("{wanted}_{n}");
        n += 1;
    }
    name
}

/// The duration of a timed trigger (`after 5 s`), in microseconds.
fn micros(label: &str) -> Result<u64, String> {
    let written = label.trim_start_matches("after").trim();
    let quantity = Quantity::parse(written).map_err(|_| format!("timing '{written}' is not a time (write `timing: 5 s`)"))?;
    if quantity.dimension() != Dimension::Time {
        return Err(format!("timing '{written}' is a {}, not a time", quantity.dimension().label()));
    }
    let us = quantity.canonical() * 1e6;
    if !(0.0..=u32::MAX as f64).contains(&us) || (us - us.round()).abs() > 1e-6 {
        return Err(format!("timing '{written}' is not a whole number of microseconds up to {} s", u32::MAX / 1_000_000));
    }
    Ok(us.round() as u64)
}

fn signals(texts: Vec<String>, prefix: &str, columns: &str) -> Vec<Signal> {
    let mut taken = BTreeSet::new();
    let mut members = BTreeSet::new();
    texts
        .into_iter()
        .map(|text| {
            let ident = unique(&mut taken, format!("{prefix}_{}", c_ident(&text)));
            let member = unique(&mut members, format!("{columns}{}", member(&text)));
            Signal { text, ident, member }
        })
        .collect()
}

fn push_unique(list: &mut Vec<String>, text: &str) {
    if !list.iter().any(|t| t == text) {
        list.push(text.to_string());
    }
}

/// The top-level region each top state is in, in arena order.
fn regions_of(source: &StateMachine, machine: &Machine) -> Vec<usize> {
    let sizes: Vec<usize> = if source.regions.is_empty() { vec![source.states.len()] } else { source.regions.iter().map(|r| r.states.len()).collect() };
    let mut region = vec![0; machine.states.len()];
    let mut top = 0;
    for (i, state) in machine.states.iter().enumerate() {
        match state.parent {
            None => {
                let mut seen = 0;
                let mut r = 0;
                for (n, size) in sizes.iter().enumerate() {
                    if top < seen + size {
                        r = n;
                        break;
                    }
                    seen += size;
                }
                region[i] = r;
                top += 1;
            }
            Some(parent) => region[i] = region[parent],
        }
    }
    region
}

pub fn plan_machine(source: &StateMachine, taken: &mut BTreeSet<String>) -> Result<MachinePlan, String> {
    let machine = Machine::new(source);
    let what = format!("state_machine '{}'", source.name);
    let mut events = Vec::new();
    let mut timers_written = Vec::new();
    let mut conditions = Vec::new();
    let mut actions = Vec::new();
    for t in &machine.transitions {
        match t.trigger.as_deref() {
            Some(label) if label.starts_with("after ") && source.transitions.get(t.index).is_some_and(|d| d.trigger.trim().is_empty()) => push_unique(&mut timers_written, label),
            Some(label) => push_unique(&mut events, label),
            None => {}
        }
        if let Some(guard) = t.guard.as_deref().filter(|g| *g != "true" && *g != "false") {
            push_unique(&mut conditions, guard);
        }
    }
    for state in &machine.states {
        for a in state.entry.iter().chain(&state.exit).chain(&state.activity) {
            push_unique(&mut actions, a);
        }
    }
    for t in &machine.transitions {
        if let Some(a) = &t.action {
            push_unique(&mut actions, a);
        }
    }
    let events = signals(events, "ev", "event.");
    let conditions = signals(conditions, "cond", "condition.");
    let actions = signals(actions, "act", "action.");
    let mut timer_names = BTreeSet::new();
    let timers = timers_written
        .into_iter()
        .map(|label| {
            let us = micros(&label).map_err(|e| format!("{what}: {e}"))?;
            Ok(Timer { ident: unique(&mut timer_names, format!("tm_{}", c_ident(&label))), label, micros: us })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let mut state_names = BTreeSet::new();
    let states: Vec<StatePlan> = machine
        .states
        .iter()
        .map(|s| {
            let find = |list: &[String]| list.iter().map(|a| actions.iter().position(|x| &x.text == a).expect("collected")).collect::<Vec<_>>();
            StatePlan {
                path: s.path.clone(),
                member: s.path.split('.').map(member).collect::<Vec<_>>().join("."),
                ident: unique(&mut state_names, format!("st_{}", c_ident(&s.path))),
                parent: s.parent,
                initial: s.initial,
                children: s.children.clone(),
                entry: find(&s.entry),
                exit: find(&s.exit),
                activity: find(&s.activity),
            }
        })
        .collect();
    let region = regions_of(source, &machine);
    let mut transitions = Vec::new();
    for t in &machine.transitions {
        if region[t.source] != region[t.target] {
            return Err(format!("{what}: transition {} goes from one region to another; such a transition is not generated (each region is a machine of its own)", t.label));
        }
        let (exit_from, entry_path) = machine.scope(t.source, t.target);
        let kind = if t.source == t.target {
            Kind::SelfLoop
        } else if machine.ancestors(t.target).contains(&t.source) {
            Kind::Inner
        } else {
            Kind::Outer
        };
        let guard = match t.guard.as_deref() {
            None | Some("true") => Guard::Always,
            Some("false") => Guard::Never,
            Some(text) => Guard::Condition(conditions.iter().position(|c| c.text == text).expect("collected")),
        };
        let trigger = match t.trigger.as_deref() {
            None => Trigger::Completion,
            Some(label) => match timers.iter().position(|tm| tm.label == label) {
                Some(i) if source.transitions.get(t.index).is_some_and(|d| d.trigger.trim().is_empty()) => Trigger::Timer(i),
                _ => Trigger::Event(events.iter().position(|e| e.text == label).expect("collected")),
            },
        };
        transitions.push(TransitionPlan {
            index: t.index,
            label: t.label.clone(),
            source: t.source,
            target: t.target,
            kind,
            exit_from,
            entry_path,
            action: t.action.as_ref().map(|a| actions.iter().position(|x| &x.text == a).expect("collected")),
            guard,
            trigger,
        });
    }
    // The order of a step: events as declared, then timers, then completion.
    let mut triggers: Vec<Trigger> = (0..events.len()).map(Trigger::Event).collect();
    triggers.extend((0..timers.len()).map(Trigger::Timer));
    if transitions.iter().any(|t| t.trigger == Trigger::Completion) {
        triggers.push(Trigger::Completion);
    }
    let dispatch = triggers
        .into_iter()
        .map(|trigger| {
            let mut order: Vec<usize> = (0..transitions.len()).filter(|&i| transitions[i].trigger == trigger).collect();
            // As `Machine::enabled` sorts: deeper source, lower priority, declaration.
            let priority = |i: usize| machine.transitions[i].priority.unwrap_or(i64::MAX);
            order.sort_by(|&a, &b| {
                machine
                    .depth(transitions[b].source)
                    .cmp(&machine.depth(transitions[a].source))
                    .then(priority(a).cmp(&priority(b)))
                    .then(transitions[a].index.cmp(&transitions[b].index))
            });
            (trigger, order)
        })
        .collect();
    let mut notes = machine.notes.clone();
    for t in &machine.transitions {
        let declared = &source.transitions[t.index];
        if !declared.trigger.trim().is_empty() && (declared.timing.is_some() || declared.attributes.contains_key("timing")) {
            notes.push(format!("transition {} has a trigger and a timing: the trigger fires it, the timing is not generated (it is a budget, not a timer)", t.label));
        }
    }
    let max_depth = (0..machine.states.len()).map(|s| machine.depth(s)).max().unwrap_or(1);
    let ident = unique(taken, c_name(&source.name));
    if machine.states.is_empty() {
        return Err(format!("{what} has no state: nothing to generate"));
    }
    if machine.states.len() > u16::MAX as usize - 1 {
        return Err(format!("{what} has more than {} states", u16::MAX - 1));
    }
    Ok(MachinePlan {
        name: source.name.clone(),
        id: source.id.clone(),
        ident,
        notes,
        machine,
        states,
        transitions,
        events,
        conditions,
        actions,
        timers,
        dispatch,
        max_depth,
    })
}

/// The name of the model, for the files and the functions.
pub fn model_name(ast: &Model) -> String {
    crate::compiler::modelica::model_name(ast)
}

pub fn plan(ast: &Model) -> Result<Plan, String> {
    if ast.state_machines.is_empty() {
        return Err("the model has no state_machine: there is no behavior to generate".to_string());
    }
    let model = model_name(ast);
    let mut taken = BTreeSet::new();
    let machines = ast.state_machines.iter().map(|m| plan_machine(m, &mut taken)).collect::<Result<Vec<_>, _>>()?;
    Ok(Plan { ident: c_name(&model), model, machines })
}
