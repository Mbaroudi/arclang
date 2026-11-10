//! Behavior: a state machine is a `state def` with its states (nested
//! ones inside their parent) and transitions, `#Mode` on a mode; a scenario
//! is an `occurrence def` with one `ref part` lifeline per participant, an
//! event per message end and a `message` between them, in order. The owner
//! of a machine or a scenario is a dependency named `owner`.

use super::relations::dependency;
use super::{text, Exporter};
use crate::compiler::ast::*;
use std::collections::HashMap;

// Not `Behavior`: that is a metadata definition of the profile.
pub(super) const PACKAGE: &str = "Behaviors";

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
        register_states(e, &machine.name, vec![text::name(&id)], &machine.states, &machine.regions, &names);
    }
    for scenario in &model.scenarios {
        let id = if scenario.id.is_empty() { scenario.name.clone() } else { scenario.id.clone() };
        e.register(PACKAGE, &id, &[&scenario.name]);
    }
    super::relations::register(e);
}

fn register_states(e: &mut Exporter<'_>, machine: &str, path: Vec<String>, states: &[State], regions: &[Region], names: &HashMap<&str, usize>) {
    for state in states {
        let mut segments = path.clone();
        // A state of a region is written in it: the region is on its path.
        if let Some(region) = region_of(regions, state) {
            segments.push(text::name(region_id(region)));
        }
        segments.push(text::name(state_id(state)));
        let qualified = format!("{machine}.{}", state.name);
        let unique = if names.get(state.name.as_str()) == Some(&1) { state.name.as_str() } else { "" };
        e.register_at(PACKAGE, segments.clone(), &state.id, &[&qualified, unique]);
        register_states(e, machine, segments, &state.sub_states, &state.regions, names);
    }
}

/// What a region is declared by: its `id:` when it is written, else its name
/// (the id the parser gives it, `<machine>.<name>`, is not written anywhere).
fn region_id(region: &Region) -> &str {
    region.attributes.get("id").and_then(|id| id.as_string()).unwrap_or(&region.name)
}

/// The region a state is written in, when it is in one: the states of a region
/// are in the list of its machine (or of its state) too.
fn region_of<'a>(regions: &'a [Region], state: &State) -> Option<&'a Region> {
    regions.iter().find(|region| region.states.iter().any(|inner| inner.name == state.name && inner.id == state.id))
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
    e.block(&format!("state {}{}", text::declared(&id, &machine.name), parallel(&machine.regions)), |e| {
        e.header("StateMachine", &machine.attributes, &[]);
        let root = Root { name: text::name(&id), states: &machine.states, regions: &machine.regions };
        let level = Level { states: &machine.states, regions: &machine.regions, initial: &machine.initial_state };
        states(e, &root, &level, &written_here(&machine.transitions, &machine.states, &machine.regions));
    });
    if let Some(owner) = machine.attributes.get("owner").and_then(|o| o.as_string()) {
        dependency(e, "owner", &id, owner);
    }
}

/// The machine a region is in: its name, its top states and its regions.
struct Root<'a> {
    name: String,
    states: &'a [State],
    regions: &'a [Region],
}

/// What a machine, a state or a region holds: its states, the regions some of
/// them are written in, and the state it is entered by.
struct Level<'a> {
    states: &'a [State],
    regions: &'a [Region],
    initial: &'a str,
}

/// A machine or a state that holds regions is in all of them at once: SysML v2
/// says it `parallel`, and each region is a state of it.
fn parallel(regions: &[Region]) -> &'static str {
    if regions.is_empty() {
        ""
    } else {
        " parallel"
    }
}

/// The transitions written at one level, out of the list that level gives. A
/// machine lists those of its regions and of its composite states too, a region
/// those of its states; each is written where the model writes it: written in
/// both, a transition would be two for whoever reads the file.
fn written_here<'a>(transitions: &'a [Transition], states: &'a [State], regions: &'a [Region]) -> Vec<&'a Transition> {
    fn below<'a>(states: &'a [State], regions: &'a [Region], all: &mut Vec<&'a Transition>) {
        for state in states {
            all.extend(&state.transitions);
            below(&state.sub_states, &state.regions, all);
        }
        // The states of a region are in `states` too: what remains is what the
        // region writes itself.
        for region in regions {
            all.extend(written_here(&region.transitions, &region.states, &[]));
        }
    }
    let same = |a: &Transition, b: &Transition| a.id == b.id && a.from == b.from && a.to == b.to && a.trigger == b.trigger && a.guard == b.guard && a.action == b.action && a.timing == b.timing && a.priority == b.priority && a.attributes == b.attributes;
    let mut nested = Vec::new();
    below(states, regions, &mut nested);
    transitions
        .iter()
        .filter(|transition| match nested.iter().position(|inner| same(inner, transition)) {
            Some(written) => {
                nested.swap_remove(written);
                false
            }
            None => true,
        })
        .collect()
}

/// One level of a machine: its regions (each a state of a parallel state, with
/// its own entry), the entry to its initial state, each state (its sub-states
/// inside), the transitions between them.
///
/// SysML v2 allows no transition and no succession directly in a parallel
/// state. A transition the model writes beside the regions (from a state of one
/// to a state of another) is written in the region it leaves, marked `@Beside`
/// and with both ends named from the machine: read back, it is beside them again.
fn states(e: &mut Exporter<'_>, root: &Root<'_>, level: &Level<'_>, transitions: &[&Transition]) {
    let leaves = |region: &Region, transition: &Transition| path_to(&region.states, &[], &transition.from).is_some();
    for region in level.regions {
        e.block(&format!("#Region state {}", text::declared(region_id(region), &region.name)), |e| {
            e.header("Region", &region.attributes, &["initial"]);
            let inner = Level { states: &region.states, regions: &[], initial: &region.initial };
            let own = written_here(&region.transitions, &region.states, &[]);
            states(e, root, &inner, &own);
            let beside = transitions.iter().filter(|transition| leaves(region, transition));
            for (n, transition) in beside.enumerate() {
                match (from_machine(root, &transition.from), from_machine(root, &transition.to)) {
                    (Some(from), Some(to)) => self::transition(e, own.len() + n + 1, transition, &from, &to, true),
                    _ => e.line(&format!("// transition {} -> {}: a state is not in this machine", transition.from, transition.to)),
                }
            }
        });
    }
    let transitions: Vec<&Transition> = transitions.iter().copied().filter(|transition| !level.regions.iter().any(|region| leaves(region, transition))).collect();
    // The states written beside the regions, or all of them when there is none.
    let list: Vec<&State> = level.states.iter().filter(|state| region_of(level.regions, state).is_none()).collect();
    let names: HashMap<&str, &str> = list.iter().flat_map(|s| [(s.name.as_str(), state_id(s)), (state_id(s), state_id(s))]).collect();
    // A parallel state is entered by each of its regions: it has no entry of its own.
    if let Some(initial) = names.get(level.initial).filter(|_| level.regions.is_empty()) {
        e.line(&format!("entry; then {};", text::name(initial)));
    }
    for state in list {
        let metadata = if state.kind == StateKind::Mode { "#Mode " } else { "" };
        e.block(&format!("{metadata}state {}{}", text::declared(state_id(state), &state.name), parallel(&state.regions)), |e| {
            e.header("State", &state.attributes, &[]);
            // The modes a configuration realizes are states named `Machine.State`:
            // text, as `during` and `context` are, where another element's `realizes` is a dependency.
            let realized = super::relations::list(&state.attributes, "realizes");
            for (key, actions) in [("realizes", &realized), ("entry_actions", &state.entry_actions), ("exit_actions", &state.exit_actions)] {
                if !actions.is_empty() {
                    let items: Vec<String> = actions.iter().map(|a| text::string(a)).collect();
                    e.line(&format!("attribute {key} : String [*] = ({}{});", items.join(", "), if items.len() == 1 { "," } else { "" }));
                }
            }
            if !state.sub_states.is_empty() {
                let inner = Level { states: &state.sub_states, regions: &state.regions, initial: &state.initial };
                states(e, root, &inner, &state.transitions.iter().collect::<Vec<_>>());
            }
        });
    }
    for (n, transition) in transitions.iter().enumerate() {
        let end = |reference: &str| path_to(level.states, level.regions, reference).or_else(|| from_machine(root, reference));
        let (Some(from), Some(to)) = (end(&transition.from), end(&transition.to)) else {
            e.line(&format!("// transition {} -> {}: a state is not in this region", transition.from, transition.to));
            continue;
        };
        self::transition(e, n + 1, transition, &from, &to, false);
    }
}

/// A transition between two states named from where it is written: the `n`th
/// there when it has no id of its own. What it says (its trigger, its guard,
/// its action, its timing) is its attributes.
fn transition(e: &mut Exporter<'_>, n: usize, transition: &Transition, from: &str, to: &str, beside: bool) {
    let id = if transition.id.is_empty() { format!("t{n}") } else { transition.id.clone() };
    let mut attributes = transition.attributes.clone();
    for (key, value) in [("trigger", Some(&transition.trigger)), ("guard", transition.guard.as_ref()), ("action", transition.action.as_ref()), ("timing", transition.timing.as_ref())] {
        if let Some(value) = value.filter(|v| !v.is_empty()) {
            attributes.entry(key.to_string()).or_insert(AttributeValue::String(value.clone()));
        }
    }
    e.block(&format!("transition {} first {from} then {to}", text::name(&id)), |e| {
        if beside {
            e.line("@Beside;");
        }
        e.header("Transition", &attributes, &[]);
    });
}

/// A state of this level or nested in one of its states, by id or name: `A`,
/// `A.B` or `Region.A` (a feature chain), the path that names it from here.
fn path_to(list: &[State], regions: &[Region], reference: &str) -> Option<String> {
    let named = |state: &State| match region_of(regions, state) {
        Some(region) => format!("{}.{}", text::name(region_id(region)), text::name(state_id(state))),
        None => text::name(state_id(state)),
    };
    if let Some(state) = list.iter().find(|s| s.name == reference || state_id(s) == reference) {
        return Some(named(state));
    }
    list.iter().find_map(|s| path_to(&s.sub_states, &s.regions, reference).map(|inner| format!("{}.{inner}", named(s))))
}

/// A state of an enclosing region: the chain from the machine, which every
/// region sees (`M.A.B`).
fn from_machine(root: &Root<'_>, reference: &str) -> Option<String> {
    path_to(root.states, root.regions, reference).map(|path| format!("{}.{path}", root.name))
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
/// message `start` to the message `endAt`; without `endAt`, an instant.
/// (`end` is a reserved word of SysML v2: `attribute end` does not parse.)
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
            e.line(&format!("attribute endAt : Integer = {};", invariant.end));
        }
    });
}
