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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Guard {
    Always,
    Never,
    Condition(usize),
    /// A guard that is an expression over the variables: its text, the key
    /// of its code in the machine's program.
    Code(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Trigger {
    Event(usize),
    Timer(usize),
    Completion,
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
    pub action: Option<usize>,
    pub guard: Guard,
}

/// A transition that fires as one, through the pseudo-states it goes
/// through (`execution::machine::Compound`), and what triggers it.
#[derive(Debug, Clone)]
pub struct CompoundPlan {
    pub trigger: Trigger,
    pub compound: crate::compiler::execution::machine::Compound,
}

#[derive(Debug, Clone)]
pub struct MachinePlan {
    pub name: String,
    pub id: String,
    pub ident: String,
    pub machine: Machine,
    pub states: Vec<StatePlan>,
    pub transitions: Vec<TransitionPlan>,
    pub compounds: Vec<CompoundPlan>,
    pub events: Vec<Signal>,
    pub conditions: Vec<Signal>,
    pub actions: Vec<Signal>,
    pub timers: Vec<Timer>,
    /// What fires in a step, in order: each trigger and its compound
    /// transitions in the order they fire (inner source first, then
    /// priority, then declaration).
    pub dispatch: Vec<(Trigger, Vec<usize>)>,
    pub max_depth: usize,
    pub notes: Vec<String>,
    /// The C names of the variables, in the program's order.
    pub variables: Vec<Signal>,
    /// The budgets of its transitions, watched every step.
    pub budgets: Vec<Budget>,
    /// Its code in fixed point, when it is generated so (the bit-exact
    /// reference of a machine the hardware computes: `fxc`).
    pub fixed: Option<Box<super::fixed::MachineFixed>>,
    /// The block that computes it in programmable logic, when the software
    /// calls it through its driver (`<block>_hw.h`) instead of stepping it.
    pub hardware: Option<String>,
}

/// `budget: 1500 ms` on a transition: an output true in every step its
/// source has been active longer than the budget (ceil(budget / period)
/// steps) without the transition firing.
#[derive(Debug, Clone)]
pub struct Budget {
    pub transition: usize,
    pub label: String,
    pub ident: String,
    /// Its result column under the machine (`budget.Starting_Running`).
    pub member: String,
    pub source: usize,
    pub micros: u64,
    pub written: String,
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

    /// The variables and code of the machine.
    pub fn program(&self) -> &crate::compiler::execution::expr::Program {
        &self.machine.program
    }

    pub fn variable(&self, name: &str) -> Option<usize> {
        self.machine.program.variable(name)
    }

    /// A guard as the C reads it.
    pub fn guard(&self, text: &str) -> Guard {
        guard_of(&self.machine, &self.conditions, Some(text))
    }

    /// The history pseudo-states: (index, deep, owner, region range).
    pub fn histories(&self) -> Vec<(usize, bool, usize, (usize, usize))> {
        use crate::compiler::execution::machine::Pseudo;
        self.machine
            .histories()
            .into_iter()
            .filter_map(|h| {
                let node = &self.machine.states[h];
                let owner = node.parent?;
                let region = &self.machine.states[owner].regions[node.region];
                let range = (*region.states.first()?, self.machine.states[*region.states.last()?].last);
                Some((h, node.pseudo == Some(Pseudo::DeepHistory), owner, range))
            })
            .collect()
    }

    /// Whether a `terminate` pseudo-state can be reached.
    pub fn terminates(&self) -> bool {
        self.machine.states.iter().any(|s| s.pseudo == Some(crate::compiler::execution::machine::Pseudo::Terminate))
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

/// A C identifier of at most `max` characters: a longer one keeps its
/// beginning and ends with a hash of the whole, so identifiers stay distinct
/// within the 63 characters C99 guarantees (MISRA C 2012 rules 5.2, 5.4).
pub fn bounded(name: &str, max: usize) -> String {
    if name.len() <= max {
        return name.to_string();
    }
    let mut hash: u32 = 0x811c_9dc5;
    for b in name.bytes() {
        hash = (hash ^ b as u32).wrapping_mul(0x0100_0193);
    }
    let keep: String = name.chars().take(max - 9).collect();
    format!("{}_{hash:08x}", keep.trim_end_matches('_'))
}

fn unique(taken: &mut BTreeSet<String>, wanted: String) -> String {
    let wanted = bounded(&wanted, 60);
    let mut name = wanted.clone();
    let mut n = 2;
    while !taken.insert(name.clone()) {
        name = format!("{wanted}_{n}");
        n += 1;
    }
    name
}

/// Whether a trigger is a timer: `after <time>`, from `after:` or `timing:`
/// (no trigger written), or written so in `trigger:` with a time.
fn is_timer(label: &str, declared: &crate::compiler::ast::Transition) -> bool {
    label.starts_with("after ") && (declared.trigger.trim().is_empty() || micros(label).is_ok())
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

/// A guard as the C reads it: always, never, a condition the caller sets,
/// or code.
pub fn guard_of(machine: &Machine, conditions: &[Signal], guard: Option<&str>) -> Guard {
    match guard.map(str::trim) {
        None | Some("true") | Some("else") => Guard::Always,
        Some("false") => Guard::Never,
        Some(text) if machine.program.guards.contains_key(text) => Guard::Code(text.to_string()),
        Some(text) => Guard::Condition(conditions.iter().position(|c| c.text == text).expect("collected")),
    }
}

pub fn plan_machine(source: &StateMachine, taken: &mut BTreeSet<String>) -> Result<MachinePlan, String> {
    // A mistake in the code of the machine is an error here, not a note.
    crate::compiler::execution::machine::compile(source)?;
    let machine = Machine::new(source);
    let what = format!("state_machine '{}'", source.name);
    let mut events = Vec::new();
    let mut timers_written = Vec::new();
    let mut conditions = Vec::new();
    let mut actions = Vec::new();
    for t in &machine.transitions {
        match t.trigger.as_deref() {
            Some(label) if is_timer(label, &source.transitions[t.index]) => push_unique(&mut timers_written, label),
            Some(label) => push_unique(&mut events, label),
            None => {}
        }
        if let Some(guard) = t.guard.as_deref().filter(|g| *g != "true" && *g != "false" && *g != "else" && !machine.program.guards.contains_key(*g)) {
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
    if let Some(refused) = machine.refused.first() {
        return Err(format!("{what}: {refused}"));
    }
    let mut transitions = Vec::new();
    for t in &machine.transitions {
        transitions.push(TransitionPlan {
            index: t.index,
            label: t.label.clone(),
            source: t.source,
            target: t.target,
            action: t.action.as_ref().map(|a| actions.iter().position(|x| &x.text == a).expect("collected")),
            guard: guard_of(&machine, &conditions, t.guard.as_deref()),
        });
    }
    let trigger_of = |c: &crate::compiler::execution::machine::Compound| -> Trigger {
        match c.trigger.as_deref() {
            None => Trigger::Completion,
            Some(label) => match timers.iter().position(|tm| tm.label == label) {
                Some(i) if is_timer(label, &source.transitions[machine.transitions[c.head].index]) => Trigger::Timer(i),
                _ => Trigger::Event(events.iter().position(|e| e.text == label).expect("collected")),
            },
        }
    };
    let compounds: Vec<CompoundPlan> = machine.compounds.iter().map(|c| CompoundPlan { trigger: trigger_of(c), compound: c.clone() }).collect();
    // The order of a step: events as declared, then timers, then completion.
    let mut triggers: Vec<Trigger> = (0..events.len()).map(Trigger::Event).collect();
    triggers.extend((0..timers.len()).map(Trigger::Timer));
    if compounds.iter().any(|c| c.trigger == Trigger::Completion) {
        triggers.push(Trigger::Completion);
    }
    let dispatch = triggers
        .into_iter()
        .map(|trigger| {
            let mut order: Vec<usize> = (0..compounds.len()).filter(|&i| compounds[i].trigger == trigger).collect();
            // As `Machine::enabled_compounds` sorts: deeper source, lower priority, declaration.
            order.sort_by(|&a, &b| {
                let (x, y) = (&compounds[a].compound, &compounds[b].compound);
                y.depth.cmp(&x.depth).then(x.priority.unwrap_or(i64::MAX).cmp(&y.priority.unwrap_or(i64::MAX))).then(machine.transitions[x.head].index.cmp(&machine.transitions[y.head].index))
            });
            (trigger, order)
        })
        .collect();
    let notes = machine.notes.clone();
    // Budgets: how long the source of a transition may stay active before it fires.
    let mut budget_names = BTreeSet::new();
    let mut budget_members = BTreeSet::new();
    let mut budgets = Vec::new();
    for (k, t) in machine.transitions.iter().enumerate() {
        if let Some(written) = &t.budget {
            let us = micros(&format!("after {written}")).map_err(|e| format!("{what}: transition {}: budget: {}", t.label, e.replace("timing", "budget")))?;
            budgets.push(Budget {
                transition: k,
                label: t.label.clone(),
                ident: unique(&mut budget_names, format!("bud_{}", c_ident(&t.label))),
                member: unique(&mut budget_members, format!("budget.{}", member(&t.label))),
                source: t.source,
                micros: us,
                written: written.clone(),
            });
        }
    }
    let max_depth = (0..machine.states.len()).map(|s| machine.depth(s)).max().unwrap_or(1);
    let mut variable_names = BTreeSet::new();
    let variables: Vec<Signal> = machine
        .program
        .variables
        .iter()
        .map(|v| Signal { text: v.name.clone(), ident: unique(&mut variable_names, format!("v_{}", c_ident(&v.name))), member: v.name.clone() })
        .collect();
    for v in &variables {
        if states.iter().any(|s| s.member == v.member) {
            return Err(format!("{what}: variable '{}' has the name of a state: a result column reads either", v.text));
        }
    }
    let ident = unique(taken, bounded(&c_name(&source.name), 24));
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
        compounds,
        events,
        conditions,
        actions,
        timers,
        dispatch,
        max_depth,
        variables,
        budgets,
        fixed: None,
        hardware: None,
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
    Ok(Plan { ident: bounded(&c_name(&model), 32), model, machines })
}
