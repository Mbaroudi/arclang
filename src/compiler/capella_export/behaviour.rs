//! State machines: regions, states and modes (nested through their regions),
//! the initial state, transitions with their trigger and guard. A machine
//! belongs to a component in Capella: the one its name starts with
//! (`Owner / Machine`, as the import names machines that share a name),
//! else the first system component.

use super::context::{derived, list, text, Context};
use super::xml::El;
use crate::compiler::ast::{State, StateKind, StateMachine};
use std::collections::HashMap;

/// The machine's owner (a component id) and its own name.
pub(super) fn owner_and_name<'a>(context: &Context, machine: &'a StateMachine, fallback: Option<&str>) -> (Option<String>, &'a str) {
    let declared = text(&machine.attributes, "owner").and_then(|o| context.resolve(o)).map(str::to_string);
    if let Some(declared) = declared {
        // The import prefixes the owner's name when machines share a name.
        let name = machine.name.split_once(" / ").filter(|(owner, _)| context.resolve(owner) == Some(declared.as_str())).map(|(_, n)| n).unwrap_or(&machine.name);
        return (Some(declared), name);
    }
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
    // The parser names an id after the state when none is written; an
    // unnamed Capella state is named after its id, a uuid.
    let looks_like_uuid = state.id.len() == 36 && state.id.matches('-').count() == 4;
    if state.id.is_empty() || (state.id == state.name && !looks_like_uuid) {
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

fn transition(context: &Context, id: &str, index: usize, t: &crate::compiler::ast::Transition, names: &HashMap<String, String>) -> Option<El> {
    let (source, target) = (names.get(&t.from)?, names.get(&t.to)?);
    let tid = if t.id.is_empty() { derived(&format!("{id}:transition:{index}")) } else { t.id.clone() };
    let mut el = El::new("ownedTransitions", "capellacommon:StateTransition", &tid).attr("source", &format!("#{source}")).attr("target", &format!("#{target}"));
    if !t.trigger.is_empty() {
        el.set("triggerDescription", &t.trigger);
    }
    // The effect: the functions the action names.
    let effects: Vec<String> = t.action.as_deref().unwrap_or("").split(", ").filter_map(|f| context.resolve_function(f.trim()).or_else(|| context.resolve_type(f.trim())).or_else(|| context.resolve(f.trim()))).map(|f| format!("#{f}")).collect();
    if !effects.is_empty() {
        el.set("effect", &effects.join(" "));
    }
    let triggers: Vec<String> = list(&t.attributes, "events").iter().map(|e| format!("#{}", event_id(e))).collect();
    if !triggers.is_empty() {
        el.set("triggers", &triggers.join(" "));
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

/// Record the states of a machine, by id and by name.
pub(super) fn register(context: &mut Context, machine: &StateMachine) {
    let id = machine_id(machine);
    let mut names = HashMap::new();
    states_by_name(&id, &machine.states, &mut names);
    for (reference, state) in names {
        context.register(&reference, &state);
    }
    context.register(&id, &id);
    context.register(&machine.name, &id);
    for t in machine.transitions.iter().filter(|t| !t.id.is_empty()) {
        context.register(&t.id, &t.id);
    }
}

/// The Capella kind of a state: its `kind:` (a pseudo-state), else state or mode.
fn state_kind(s: &State) -> &'static str {
    match text(&s.attributes, "kind") {
        Some("final") => "capellacommon:FinalState",
        Some("choice") => "capellacommon:ChoicePseudoState",
        Some("fork") => "capellacommon:ForkPseudoState",
        Some("join") => "capellacommon:JoinPseudoState",
        Some("terminate") => "capellacommon:TerminatePseudoState",
        Some("deep_history") => "capellacommon:DeepHistoryPseudoState",
        Some("shallow_history") => "capellacommon:ShallowHistoryPseudoState",
        Some("entry_point") => "capellacommon:EntryPointPseudoState",
        Some("exit_point") => "capellacommon:ExitPointPseudoState",
        _ if s.kind == StateKind::Mode => "capellacommon:Mode",
        _ => "capellacommon:State",
    }
}

/// The id of a machine's event, by name.
pub(super) fn event_id(name: &str) -> String {
    derived(&format!("event:{name}"))
}

/// The events the transitions of the model name.
pub(super) fn events(machines: &[StateMachine]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for m in machines {
        for t in &m.transitions {
            for event in list(&t.attributes, "events") {
                if !names.contains(&event) {
                    names.push(event);
                }
            }
        }
    }
    names
}

/// An initial pseudo-state and its transition to `target`, for a region.
fn initial(region: &mut El, owner: &str, target: &str) {
    let initial = derived(&format!("{owner}:initial"));
    region.push(El::new("ownedStates", "capellacommon:InitialPseudoState", &initial).attr("name", "Initial"));
    region.push(El::new("ownedTransitions", "capellacommon:StateTransition", &derived(&format!("{owner}:initial-transition"))).attr("source", &format!("#{initial}")).attr("target", &format!("#{target}")));
}

fn state(context: &Context, machine: &str, s: &State, inner: &HashMap<String, Vec<El>>, names: &HashMap<String, String>) -> El {
    let kind = state_kind(s);
    let id = state_id(machine, s);
    let mut el = super::header::apply(El::new("ownedStates", kind, &id).attr("name", &s.name), &s.attributes, &["kind", "initial", "entry", "exit", "do", "owner"]);
    // Entry, exit and do activities: the functions they name.
    for (key, attribute) in [("entry", "entry"), ("exit", "exit"), ("do", "doActivity")] {
        let functions: Vec<String> = list(&s.attributes, key).iter().filter_map(|f| context.resolve_function(f).or_else(|| context.resolve_type(f)).or_else(|| context.resolve(f))).map(|f| format!("#{f}")).collect();
        if !functions.is_empty() {
            el.set(attribute, &functions.join(" "));
        }
    }
    if !s.sub_states.is_empty() {
        let region_id = derived(&format!("{id}:region"));
        let mut region = El::new("ownedRegions", "capellacommon:Region", &region_id).attr("name", "Default Region");
        if let Some(target) = names.get(&s.initial).filter(|_| !s.initial.is_empty()) {
            initial(&mut region, &region_id, target);
        }
        for sub in &s.sub_states {
            region.push(state(context, machine, sub, inner, names));
        }
        for t in inner.get(&id).into_iter().flatten() {
            region.push(t.clone());
        }
        el.push(region);
    }
    el
}

/// The machine element, to be owned by a component (`ownedStateMachines`).
pub(super) fn state_machine(context: &Context, machine: &StateMachine, name: &str) -> El {
    let id = machine_id(machine);
    let mut names = HashMap::new();
    states_by_name(&id, &machine.states, &mut names);
    let mut region = El::new("ownedRegions", "capellacommon:Region", &derived(&format!("{id}:region"))).attr("name", "Default Region");
    if let Some(target) = names.get(&machine.initial_state) {
        initial(&mut region, &id, target);
    }
    // A transition goes to the region it is declared in: a state's, else the machine's.
    let mut declared_in: HashMap<String, String> = HashMap::new();
    fn declared(machine: &str, states: &[State], into: &mut HashMap<String, String>) {
        for s in states {
            for t in &s.transitions {
                into.insert(format!("{}->{}:{}", t.from, t.to, t.id), state_id(machine, s));
            }
            declared(machine, &s.sub_states, into);
        }
    }
    declared(&id, &machine.states, &mut declared_in);
    let mut top: Vec<El> = Vec::new();
    let mut inner: HashMap<String, Vec<El>> = HashMap::new();
    for (index, t) in machine.transitions.iter().enumerate() {
        let Some(el) = transition(context, &id, index, t, &names) else { continue };
        let region = declared_in.get(&format!("{}->{}:{}", t.from, t.to, t.id)).cloned().or_else(|| if declared_in.is_empty() { holder(&machine.states, &t.from, &t.to).map(|s| state_id(&id, s)) } else { None });
        match region {
            Some(state) => inner.entry(state).or_default().push(el),
            None => top.push(el),
        }
    }
    for s in &machine.states {
        region.push(state(context, &id, s, &inner, &names));
    }
    for el in top {
        region.push(el);
    }
    El::new("ownedStateMachines", "capellacommon:StateMachine", &id).attr("name", name).with(region)
}
