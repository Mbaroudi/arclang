//! Mode and state machines: one diagram per machine, with its modes and
//! states, the transitions between them and a pseudo-state marking the
//! initial one.

use super::builder::{self, Builder, NodeRef};
use super::{Diagram, EdgeKind, NodeKind, ViewKind};
use crate::compiler::ast::{Model, Region, State, StateKind, StateMachine, Transition};
use crate::compiler::execution::machine::Pseudo;
use std::collections::BTreeMap;

/// One entry per machine: its diagram (none when it declares no state) and
/// the diagnostics found while building it.
pub(super) fn build(model: &Model) -> Vec<(Option<Diagram>, Vec<String>)> {
    model.state_machines.iter().map(build_machine).collect()
}

fn build_machine(machine: &StateMachine) -> (Option<Diagram>, Vec<String>) {
    let mut builder = Builder::scoped(
        ViewKind::Msm,
        format!("{}:{}", ViewKind::Msm.id(), machine.name),
    );
    let what = format!("state_machine '{}'", machine.name);

    add_level(&mut builder, None, &machine.name, &machine.states, &machine.regions);
    if builder.is_empty() {
        builder.report(format!(
            "{what} declares no state or mode: there is nothing to draw"
        ));
        return (None, builder.finish_scoped(machine.name.clone()).1);
    }

    // The start of the machine; a machine with regions starts in each of them.
    if machine.regions.is_empty() && !machine.initial_state.is_empty() {
        match builder.resolve(&machine.initial_state) {
            Some(first) => mark_start(&mut builder, None, &machine.name, first, &machine.initial_state),
            None => builder.report(format!(
                "{what}: initial '{}' is not a declared state or mode",
                machine.initial_state
            )),
        }
    }

    for transition in &machine.transitions {
        let Some((source, target)) =
            builder.resolve_pair(&what, &transition.from, &transition.to, "state or mode")
        else {
            continue;
        };
        let mut properties = BTreeMap::new();
        for (key, value) in [
            (
                "trigger",
                Some(&transition.trigger).filter(|t| !t.is_empty()),
            ),
            ("guard", transition.guard.as_ref()),
            ("action", transition.action.as_ref()),
            ("timing", transition.timing.as_ref()),
            ("priority", transition.priority.as_ref()),
        ] {
            if let Some(value) = value {
                properties.insert(key.to_string(), value.clone());
            }
        }
        let edge = builder.add_edge(
            EdgeKind::Transition,
            &source,
            &target,
            &format!("{} -> {}", transition.from, transition.to),
            None,
            properties,
        );
        builder.set_edge_label(&edge, transition_label(transition));
    }

    let (diagram, diagnostics) = builder.finish_scoped(machine.name.clone());
    (Some(diagram), diagnostics)
}

/// The states of one level (a machine, a state): those of each region in
/// its region, with where the region starts, the others beside. `owner` is
/// the id of what contains them. The states of a region are in `states` too.
fn add_level(builder: &mut Builder, parent: Option<NodeRef>, owner: &str, states: &[State], regions: &[Region]) {
    let in_a_region = |state: &State| regions.iter().any(|region| region.states.iter().any(|inner| inner.name == state.name && inner.id == state.id));
    for region in regions {
        // The region is no box of its own in the model's elements: an edit
        // names it by the id the compiler gives it.
        let properties = BTreeMap::from([("region".to_string(), region.id.clone())]);
        let node = builder.add_node(parent, &format!("{owner}.(region {})", region.name), &region.name, NodeKind::Region, properties);
        let mut first = None;
        for state in &region.states {
            let added = add_state(builder, Some(node), owner, state);
            if state.name == region.initial {
                first = Some(added);
            }
        }
        if let Some(first) = first {
            mark_start(builder, Some(node), &format!("{owner}.(region {})", region.name), first, &region.initial);
        }
    }
    for state in states.iter().filter(|state| !in_a_region(state)) {
        add_state(builder, parent, owner, state);
    }
}

/// The mark of where `owner` starts, and its transition to that state.
fn mark_start(builder: &mut Builder, parent: Option<NodeRef>, owner: &str, first: NodeRef, name: &str) {
    let start = builder.add_node(parent, &format!("{owner}.(initial)"), "", NodeKind::InitialState, BTreeMap::new());
    let source = builder.whole(start);
    let target = builder.whole(first);
    let edge = builder.add_edge(EdgeKind::Transition, &source, &target, &format!("(initial) -> {name}"), None, BTreeMap::new());
    builder.set_edge_label(&edge, String::new());
}

/// `owner` is the id of what contains the state: the machine, or the parent
/// state. Two machines may both have `Failed`, and so may two composite
/// states of one machine.
fn add_state(builder: &mut Builder, parent: Option<NodeRef>, owner: &str, state: &State) -> NodeRef {
    let text = |key: &str| state.attributes.get(key).and_then(|v| v.as_string()).map(str::to_string);
    let pseudo = text("kind").filter(|kind| Pseudo::parse(kind).is_some());
    let kind = match (&pseudo, &state.kind) {
        (Some(_), _) => NodeKind::PseudoState,
        (None, StateKind::Mode) => NodeKind::Mode,
        (None, StateKind::State) => NodeKind::State,
    };
    let mut properties = BTreeMap::new();
    for (key, actions) in [
        ("entry", &state.entry_actions),
        ("exit", &state.exit_actions),
    ] {
        if !actions.is_empty() {
            properties.insert(key.to_string(), actions.join(", "));
        }
    }
    let activity = builder::string_list(&state.attributes, "do");
    if !activity.is_empty() {
        properties.insert("do".to_string(), activity.join(", "));
    }
    if let Some(pseudo) = pseudo {
        properties.insert("pseudo".to_string(), pseudo);
    }
    let id = format!("{owner}.{}", state.name);
    let node = builder.add_node(parent, &id, &state.name, kind, properties);
    let mut first = None;
    if state.regions.is_empty() {
        for sub in &state.sub_states {
            let added = add_state(builder, Some(node), &id, sub);
            if sub.name == state.initial {
                first = Some(added);
            }
        }
    } else {
        add_level(builder, Some(node), &id, &state.sub_states, &state.regions);
    }
    if let Some(first) = first {
        mark_start(builder, Some(node), &id, first, &state.initial);
    }
    node
}

/// UML transition label: `trigger after time [guard] / action`, parts omitted when
/// absent.
fn transition_label(transition: &Transition) -> String {
    let mut label = transition.trigger.clone();
    // A timed transition fires once its source has been active that long.
    // Written as a text (`"200 ms"`) or as a quantity (`200 ms`).
    let timing = transition.timing.clone().or_else(|| transition.attributes.get("timing").map(|value| value.source()));
    if let Some(timing) = timing.as_ref().filter(|t| !t.is_empty()) {
        if !label.is_empty() {
            label.push(' ');
        }
        label.push_str(&format!("after {timing}"));
    }
    if let Some(guard) = transition.guard.as_ref().filter(|g| !g.is_empty()) {
        if !label.is_empty() {
            label.push(' ');
        }
        label.push_str(&format!("[{guard}]"));
    }
    if let Some(action) = transition.action.as_ref().filter(|a| !a.is_empty()) {
        if !label.is_empty() {
            label.push(' ');
        }
        label.push_str(&format!("/ {action}"));
    }
    label
}
