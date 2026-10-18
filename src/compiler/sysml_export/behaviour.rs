//! Behaviour: a state machine is a `state def` with its states (nested
//! ones inside their parent) and transitions, `#Mode` on a mode; a scenario
//! is an `occurrence def` with one `ref part` lifeline per participant, an
//! event per message end and a `message` between them, in order. The owner
//! of a machine or a scenario is a dependency named `owner`.

use super::relations::dependency;
use super::{text, Exporter};
use crate::compiler::ast::*;
use std::collections::HashMap;

// Not `Behaviour`: that is a metadata definition of the profile.
pub(super) const PACKAGE: &str = "Behaviours";

fn state_id(state: &State) -> &str {
    if state.id.is_empty() { &state.name } else { &state.id }
}

pub(super) fn register(e: &mut Exporter<'_>) {
    let model = e.model;
    // A state is known by its id, `Machine.State`, and its name when no
    // other state has it (what a scenario's state fragment names).
    let mut names: HashMap<&str, usize> = HashMap::new();
    fn count<'a>(states: &'a [State], names: &mut HashMap<&'a str, usize>) {
        for state in states {
            *names.entry(state.name.as_str()).or_default() += 1;
            count(&state.sub_states, names);
        }
    }
    for machine in &model.state_machines {
        count(&machine.states, &mut names);
    }
    for machine in &model.state_machines {
        let id = if machine.id.is_empty() { machine.name.clone() } else { machine.id.clone() };
        e.register(PACKAGE, &id, &[&machine.name]);
        register_states(e, &machine.name, vec![text::name(&id)], &machine.states, &names);
    }
    for scenario in &model.scenarios {
        let id = if scenario.id.is_empty() { scenario.name.clone() } else { scenario.id.clone() };
        e.register(PACKAGE, &id, &[&scenario.name]);
    }
    super::relations::register(e);
}

fn register_states(e: &mut Exporter<'_>, machine: &str, path: Vec<String>, states: &[State], names: &HashMap<&str, usize>) {
    for state in states {
        let mut segments = path.clone();
        segments.push(text::name(state_id(state)));
        let qualified = format!("{machine}.{}", state.name);
        let unique = if names.get(state.name.as_str()) == Some(&1) { state.name.as_str() } else { "" };
        e.register_at(PACKAGE, segments.clone(), &state.id, &[&qualified, unique]);
        register_states(e, machine, segments, &state.sub_states, names);
    }
}

pub(super) fn write(e: &mut Exporter<'_>) {
    let model = e.model;
    if model.state_machines.is_empty() && model.scenarios.is_empty() {
        return;
    }
    e.package(&format!("package {PACKAGE}"), PACKAGE, |e| {
        for machine in &model.state_machines {
            state_machine(e, machine);
        }
        for scenario in &model.scenarios {
            occurrence(e, scenario);
        }
    });
}

fn state_machine(e: &mut Exporter<'_>, machine: &StateMachine) {
    let id = if machine.id.is_empty() { machine.name.clone() } else { machine.id.clone() };
    e.block(&format!("state {}", text::declared(&id, &machine.name)), |e| {
        e.header("StateMachine", &machine.attributes, &[]);
        states(e, &machine.states, &machine.initial_state, &machine.transitions);
    });
    if let Some(owner) = machine.attributes.get("owner").and_then(|o| o.as_string()) {
        dependency(e, "owner", &id, owner);
    }
}

/// The states of one region: the entry to its initial state, each state
/// (its sub-states inside), the transitions between them.
fn states(e: &mut Exporter<'_>, list: &[State], initial: &str, transitions: &[Transition]) {
    let names: HashMap<&str, &str> = list.iter().flat_map(|s| [(s.name.as_str(), state_id(s)), (state_id(s), state_id(s))]).collect();
    if let Some(initial) = names.get(initial) {
        e.line(&format!("entry; then {};", text::name(initial)));
    }
    for state in list {
        let metadata = if state.kind == StateKind::Mode { "#Mode " } else { "" };
        e.block(&format!("{metadata}state {}", text::declared(state_id(state), &state.name)), |e| {
            e.header("State", &state.attributes, &[]);
            for (key, actions) in [("entry_actions", &state.entry_actions), ("exit_actions", &state.exit_actions)] {
                if !actions.is_empty() {
                    let items: Vec<String> = actions.iter().map(|a| text::string(a)).collect();
                    e.line(&format!("attribute {key} : String [*] = ({}{});", items.join(", "), if items.len() == 1 { "," } else { "" }));
                }
            }
            if !state.sub_states.is_empty() {
                states(e, &state.sub_states, &state.initial, &state.transitions);
            }
        });
    }
    for (n, transition) in transitions.iter().enumerate() {
        let (Some(from), Some(to)) = (names.get(transition.from.as_str()), names.get(transition.to.as_str())) else {
            e.line(&format!("// transition {} -> {}: a state is not in this region", transition.from, transition.to));
            continue;
        };
        let id = if transition.id.is_empty() { format!("t{}", n + 1) } else { transition.id.clone() };
        let mut attributes = transition.attributes.clone();
        for (key, value) in [("trigger", Some(&transition.trigger)), ("guard", transition.guard.as_ref()), ("action", transition.action.as_ref()), ("timing", transition.timing.as_ref())] {
            if let Some(value) = value.filter(|v| !v.is_empty()) {
                attributes.entry(key.to_string()).or_insert(AttributeValue::String(value.clone()));
            }
        }
        e.block(&format!("transition {} first {} then {}", text::name(&id), text::name(from), text::name(to)), |e| e.header("Transition", &attributes, &[]));
    }
}

fn occurrence(e: &mut Exporter<'_>, scenario: &Scenario) {
    let id = if scenario.id.is_empty() { scenario.name.clone() } else { scenario.id.clone() };
    // A lifeline per participant, named after it: `l_<id>`.
    let mut lifeline: HashMap<&str, String> = HashMap::new();
    for p in &scenario.participants {
        let name = text::name(&format!("l_{}", if p.id.is_empty() { &p.name } else { &p.id }));
        lifeline.insert(p.id.as_str(), name.clone());
        lifeline.insert(p.name.as_str(), name);
    }
    let mut events: HashMap<String, Vec<String>> = HashMap::new();
    let mut messages: Vec<String> = Vec::new();
    for (n, message) in scenario.messages.iter().enumerate() {
        let (Some(from), Some(to)) = (lifeline.get(message.from.as_str()), lifeline.get(message.to.as_str())) else {
            messages.push(format!("// message {} -> {}: a participant is not declared", message.from, message.to));
            continue;
        };
        let (sent, received) = (format!("m{}_sent", n + 1), format!("m{}_received", n + 1));
        events.entry(from.clone()).or_default().push(sent.clone());
        events.entry(to.clone()).or_default().push(received.clone());
        let mid = if message.id.is_empty() { format!("m{}", n + 1) } else { message.id.clone() };
        let declared = text::declared(&mid, if message.label.is_empty() { &mid } else { &message.label });
        messages.push(format!("message {declared} from {from}.{sent} to {to}.{received};"));
    }
    e.block(&format!("occurrence {}", text::declared(&id, &scenario.name)), |e| {
        e.header("Scenario", &scenario.attributes, &[]);
        let mut written: Vec<&String> = Vec::new();
        for p in &scenario.participants {
            let name = &lifeline[p.id.as_str()];
            if written.contains(&name) {
                continue;
            }
            written.push(name);
            // A lifeline stands for an element: a subset of its usage.
            let typing = e.feature(&p.id).or_else(|| e.feature(&p.name)).map(|t| format!(" :> {t}")).unwrap_or_default();
            let list = events.get(name).cloned().unwrap_or_default();
            let invariants: Vec<&StateInvariant> = scenario.invariants.iter().filter(|i| lifeline.get(i.lifeline.as_str()) == Some(name)).collect();
            if list.is_empty() && invariants.is_empty() {
                e.line(&format!("ref part {name}{typing};"));
                continue;
            }
            e.block(&format!("ref part {name}{typing}"), |e| {
                for (n, event) in list.iter().enumerate() {
                    e.line(&format!("{}event occurrence {event};", if n == 0 { "" } else { "then " }));
                }
                for invariant in invariants {
                    state_invariant(e, invariant);
                }
            });
        }
        for message in &messages {
            e.line(message);
        }
    });
    if let Some(owner) = scenario.attributes.get("owner").and_then(|o| o.as_string()) {
        dependency(e, "owner", &id, owner);
    }
}

/// `#Invariant state 'id' :> S { attribute start : Integer = 0; }`: a
/// state (or `ref action`, a function) asserted on its lifeline from the
/// message `start` to the message `end`; without `end`, an instant.
fn state_invariant(e: &mut Exporter<'_>, invariant: &StateInvariant) {
    let keyword = match invariant.kind {
        InvariantKind::State => "state",
        InvariantKind::Function => "action",
    };
    let target = e.feature(&invariant.name);
    let typing = target.as_ref().map(|t| format!(" :> {t}")).unwrap_or_default();
    e.block(&format!("#Invariant {keyword} {}{typing}", text::name(&invariant.id)), |e| {
        e.header("StateFragment", &invariant.attributes, &[]);
        if target.is_none() {
            e.line(&format!("attribute target : String = {};", text::string(&invariant.name)));
        }
        e.line(&format!("attribute start : Integer = {};", invariant.start));
        // An instant has no end: `end < start`, or no message at all to end on.
        if invariant.end >= invariant.start && invariant.end != usize::MAX {
            e.line(&format!("attribute end : Integer = {};", invariant.end));
        }
    });
}
