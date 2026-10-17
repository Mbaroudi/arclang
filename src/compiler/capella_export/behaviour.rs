//! State machines: regions, states and modes (nested through their regions),
//! the initial state, transitions with their trigger and guard. A machine
//! belongs to a component in Capella: the one its name starts with
//! (`Owner / Machine`, as the import names machines that share a name),
//! else the first system component.

use super::context::{derived, Context};
use super::xml::El;
use crate::compiler::ast::{State, StateKind, StateMachine};
use std::collections::HashMap;

/// The machine's owner (a component id) and its own name.
pub(super) fn owner_and_name<'a>(context: &Context, machine: &'a StateMachine, fallback: Option<&str>) -> (Option<String>, &'a str) {
    if let Some((owner, name)) = machine.name.split_once(" / ") {
        if let Some(id) = context.resolve(owner) {
            return (Some(id.to_string()), name);
        }
    }
    (fallback.map(str::to_string), machine.name.as_str())
}

fn machine_id(machine: &StateMachine) -> String {
    if !machine.id.is_empty() {
        return machine.id.clone();
    }
    derived(&format!("state-machine:{}", machine.name))
}

/// A state's Capella id: its `id:`, else one made up in its machine.
fn state_id(machine: &str, state: &State) -> String {
    if state.id.is_empty() || state.id == state.name {
        derived(&format!("{machine}:state:{}", state.name))
    } else {
        state.id.clone()
    }
}

fn states_by_name(machine: &str, states: &[State], into: &mut HashMap<String, String>) {
    for state in states {
        into.insert(state.name.clone(), state_id(machine, state));
        into.insert(state.id.clone(), state_id(machine, state));
        states_by_name(machine, &state.sub_states, into);
    }
}

/// The names (and ids) of a state and every state nested in it.
fn descendants(s: &State, into: &mut Vec<String>) {
    for sub in &s.sub_states {
        into.push(sub.name.clone());
        into.push(sub.id.clone());
        descendants(sub, into);
    }
}

/// The innermost state holding both ends of a transition in its regions.
fn holder<'a>(states: &'a [State], from: &str, to: &str) -> Option<&'a State> {
    for s in states {
        let mut inside = Vec::new();
        descendants(s, &mut inside);
        if inside.iter().any(|n| n == from) && inside.iter().any(|n| n == to) {
            return holder(&s.sub_states, from, to).or(Some(s));
        }
    }
    None
}

fn transition(id: &str, index: usize, t: &crate::compiler::ast::Transition, names: &HashMap<String, String>) -> Option<El> {
    let (source, target) = (names.get(&t.from)?, names.get(&t.to)?);
    let tid = if t.id.is_empty() { derived(&format!("{id}:transition:{index}")) } else { t.id.clone() };
    let mut el = El::new("ownedTransitions", "capellacommon:StateTransition", &tid).attr("source", &format!("#{source}")).attr("target", &format!("#{target}"));
    if !t.trigger.is_empty() {
        el.set("triggerDescription", &t.trigger);
    }
    if let Some(guard) = t.guard.as_deref().filter(|g| !g.is_empty()) {
        let constraint = derived(&format!("{tid}:guard"));
        el.set("guard", &format!("#{constraint}"));
        el.push(
            El::new("ownedConstraints", "capellacore:Constraint", &constraint)
                .with(El::new("ownedSpecification", "datavalue:OpaqueExpression", &derived(&format!("{tid}:guard:expression"))).with(El::text("bodies", guard)).with(El::text("languages", "capella:linkedText"))),
        );
    }
    Some(el)
}

fn state(machine: &str, s: &State, inner: &HashMap<String, Vec<El>>) -> El {
    let kind = match s.kind {
        StateKind::Mode => "capellacommon:Mode",
        _ => "capellacommon:State",
    };
    let id = state_id(machine, s);
    let mut el = El::new("ownedStates", kind, &id).attr("name", &s.name);
    if !s.sub_states.is_empty() {
        let mut region = El::new("ownedRegions", "capellacommon:Region", &derived(&format!("{id}:region"))).attr("name", "Default Region");
        for sub in &s.sub_states {
            region.push(state(machine, sub, inner));
        }
        for t in inner.get(&id).into_iter().flatten() {
            region.push(t.clone());
        }
        el.push(region);
    }
    el
}

/// The machine element, to be owned by a component (`ownedStateMachines`).
pub(super) fn state_machine(machine: &StateMachine, name: &str) -> El {
    let id = machine_id(machine);
    let mut names = HashMap::new();
    states_by_name(&id, &machine.states, &mut names);
    let mut region = El::new("ownedRegions", "capellacommon:Region", &derived(&format!("{id}:region"))).attr("name", "Default Region");
    let initial_target = names.get(&machine.initial_state).cloned();
    if let Some(target) = &initial_target {
        let initial = derived(&format!("{id}:initial"));
        region.push(El::new("ownedStates", "capellacommon:InitialPseudoState", &initial).attr("name", "Initial"));
        region.push(El::new("ownedTransitions", "capellacommon:StateTransition", &derived(&format!("{id}:initial-transition"))).attr("source", &format!("#{initial}")).attr("target", &format!("#{target}")));
    }
    // A transition goes to the region of the innermost state holding both its ends.
    let mut top: Vec<El> = Vec::new();
    let mut inner: HashMap<String, Vec<El>> = HashMap::new();
    for (index, t) in machine.transitions.iter().enumerate() {
        let Some(el) = transition(&id, index, t, &names) else { continue };
        match holder(&machine.states, &t.from, &t.to) {
            Some(s) => inner.entry(state_id(&id, s)).or_default().push(el),
            None => top.push(el),
        }
    }
    for s in &machine.states {
        region.push(state(&id, s, &inner));
    }
    for el in top {
        region.push(el);
    }
    El::new("ownedStateMachines", "capellacommon:StateMachine", &id).attr("name", name).with(region)
}
