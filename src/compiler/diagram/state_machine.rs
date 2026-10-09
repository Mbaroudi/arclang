//! Mode and state machines: one diagram per machine, with its modes and
//! states, the transitions between them and a pseudo-state marking the
//! initial one.

use super::builder::{Builder, NodeRef};
use super::{Diagram, EdgeKind, NodeKind, ViewKind};
use crate::compiler::ast::{Model, State, StateKind, StateMachine, Transition};
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

    for state in &machine.states {
        add_state(&mut builder, None, &machine.name, state);
    }
    if builder.is_empty() {
        builder.report(format!(
            "{what} declares no state or mode: there is nothing to draw"
        ));
        return (None, builder.finish_scoped(machine.name.clone()).1);
    }

    if !machine.initial_state.is_empty() {
        match builder.resolve(&machine.initial_state) {
            Some(first) => {
                let start = builder.add_node(
                    None,
                    &format!("{}.(initial)", machine.name),
                    "",
                    NodeKind::InitialState,
                    BTreeMap::new(),
                );
                let source = builder.whole(start);
                let target = builder.whole(first);
                let edge = builder.add_edge(
                    EdgeKind::Transition,
                    &source,
                    &target,
                    &format!("(initial) -> {}", machine.initial_state),
                    None,
                    BTreeMap::new(),
                );
                builder.set_edge_label(&edge, String::new());
            }
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

/// `owner` is the id of what contains the state: the machine, or the parent
/// state. Two machines may both have `Failed`, and so may two composite
/// states of one machine.
fn add_state(builder: &mut Builder, parent: Option<NodeRef>, owner: &str, state: &State) {
    let kind = match state.kind {
        StateKind::Mode => NodeKind::Mode,
        StateKind::State => NodeKind::State,
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
    let id = format!("{owner}.{}", state.name);
    let node = builder.add_node(parent, &id, &state.name, kind, properties);
    for sub in &state.sub_states {
        add_state(builder, Some(node), &id, sub);
    }
}

/// UML transition label: `trigger [guard] / action`, parts omitted when
/// absent.
fn transition_label(transition: &Transition) -> String {
    let mut label = transition.trigger.clone();
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
