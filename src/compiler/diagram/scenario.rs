//! Exchange scenarios: one sequence diagram per scenario. Participants are
//! lifelines, in declared order; messages keep their order, which is time.

use super::builder::{Builder, Role};
use super::elements::{element_index, kind_name, ElementRef};
use super::{Diagram, EdgeKind, Fragment, Invariant, NodeKind, Operand, ViewKind};
use crate::compiler::ast::{CombinedFragment, MessageType, Model, Scenario};
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

    let (mut diagram, diagnostics) = builder.finish_scoped(scenario.name.clone());
    // The lifelines a row of messages touches, by row.
    let rows: Vec<(String, String)> = scenario
        .messages
        .iter()
        .map(|message| (format!("{}.{}", scenario.name, message.from), format!("{}.{}", scenario.name, message.to)))
        .collect();
    let all: Vec<String> = diagram.nodes.iter().map(|node| node.id.clone()).collect();
    diagram.fragments = scenario.fragments.iter().map(|fragment| frame(fragment, &rows, &all)).collect();
    // The invariants, on the lifeline each one names, labelled with the
    // name of the state or function (the model may reference it by id).
    let mut diagnostics = diagnostics;
    for invariant in &scenario.invariants {
        let lifeline = format!("{}.{}", scenario.name, invariant.lifeline);
        if !all.contains(&lifeline) {
            diagnostics.push(format!("[{}] {what}: {} '{}' sits on '{}', which is not a participant", diagram.id, invariant.kind.keyword(), invariant.name, invariant.lifeline));
            continue;
        }
        let label = elements.get(invariant.name.as_str()).map(|element| element.name.to_string()).unwrap_or_else(|| invariant.name.clone());
        diagram.invariants.push(Invariant {
            id: invariant.id.clone(),
            lifeline,
            label,
            kind: invariant.kind.keyword().to_string(),
            start: invariant.start,
            end: invariant.end,
        });
    }
    (Some(diagram), diagnostics)
}

/// A fragment as the sheet draws it: its rows and the lifelines it spans.
fn frame(fragment: &CombinedFragment, rows: &[(String, String)], all: &[String]) -> Fragment {
    let mut lifelines: Vec<String> = Vec::new();
    if fragment.end >= fragment.start {
        for (from, to) in rows.iter().skip(fragment.start).take(fragment.end + 1 - fragment.start) {
            for end in [from, to] {
                if all.contains(end) && !lifelines.contains(end) {
                    lifelines.push(end.clone());
                }
            }
        }
    }
    if lifelines.is_empty() {
        lifelines = all.to_vec();
    }
    Fragment {
        kind: fragment.fragment_type.keyword().to_string(),
        label: fragment.label.clone(),
        start: fragment.start,
        end: fragment.end,
        lifelines,
        operands: fragment
            .operands
            .iter()
            .map(|operand| Operand {
                label: operand.label.clone(),
                start: operand.start,
                end: operand.end,
                fragments: operand.fragments.iter().map(|nested| frame(nested, rows, all)).collect(),
            })
            .collect(),
    }
}
