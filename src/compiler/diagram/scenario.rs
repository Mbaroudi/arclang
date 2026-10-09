//! Exchange scenarios: one sequence diagram per scenario. Participants are
//! lifelines, in declared order; messages keep their order, which is time.

use super::builder::{Builder, Role};
use super::elements::{element_index, kind_name, ElementRef};
use super::{Diagram, EdgeKind, NodeKind, ViewKind};
use crate::compiler::ast::{MessageType, Model, Scenario};
use std::collections::{BTreeMap, HashMap};

/// One entry per scenario: its diagram (none when it has no participant)
/// and the diagnostics found while building it.
pub(super) fn build(model: &Model) -> Vec<(Option<Diagram>, Vec<String>)> {
    let elements = element_index(model);
    model
        .scenarios
        .iter()
        .map(|scenario| build_scenario(scenario, &elements))
        .collect()
}

fn build_scenario(
    scenario: &Scenario,
    elements: &HashMap<&str, ElementRef<'_>>,
) -> (Option<Diagram>, Vec<String>) {
    let mut builder = Builder::scoped(
        ViewKind::Es,
        format!("{}:{}", ViewKind::Es.id(), scenario.name),
    );
    let what = format!("scenario '{}'", scenario.name);

    for participant in &scenario.participants {
        let mut properties = BTreeMap::new();
        match elements.get(participant.name.as_str()) {
            Some(element) => {
                properties.insert("represents".to_string(), kind_name(element.kind));
            }
            None => builder.report(format!(
                "{what}: participant '{}' is not a declared actor, component, function or node",
                participant.name
            )),
        }
        // A lifeline belongs to its scenario: the same component takes
        // part in many.
        builder.add_node(
            None,
            &format!("{}.{}", scenario.name, participant.name),
            &participant.name,
            NodeKind::Lifeline,
            properties,
        );
    }
    if builder.is_empty() {
        builder.report(format!(
            "{what} has no participant: its {} message(s) cannot be drawn",
            scenario.messages.len()
        ));
        return (None, builder.finish_scoped(scenario.name.clone()).1);
    }

    for (index, message) in scenario.messages.iter().enumerate() {
        let number = index + 1;
        let label = &message.label;
        let message_what = format!("{what}: message {number} '{label}'");
        let ends = [(&message.from, Role::Source), (&message.to, Role::Target)].map(
            |(reference, role)| {
                let end = builder.resolve_endpoint(&message_what, reference, role);
                if let Err(missing) = &end {
                    builder.report_missing(
                        &message_what,
                        reference,
                        *missing,
                        "participant of the scenario",
                    );
                }
                end.ok()
            },
        );
        let [Some(source), Some(target)] = ends else {
            continue;
        };
        let mut properties = BTreeMap::new();
        let kind = match message.message_type {
            MessageType::Synchronous => "sync",
            MessageType::Asynchronous => "async",
            MessageType::Return => "return",
        };
        properties.insert("type".to_string(), kind.to_string());
        if let Some(timing) = &message.timing {
            properties.insert("timing".to_string(), timing.clone());
        }
        let edge = builder.add_edge(
            EdgeKind::Message,
            &source,
            &target,
            &number.to_string(),
            None,
            properties,
        );
        builder.set_edge_label(&edge, label.clone());
    }

    let (diagram, diagnostics) = builder.finish_scoped(scenario.name.clone());
    (Some(diagram), diagnostics)
}
