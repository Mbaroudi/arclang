//! Links between elements across views, from the traces of the model.
//!
//! A view shows one Arcadia level. What ties the levels together — this
//! function realizes that activity, this component satisfies that
//! requirement — is written as traces. They are exported next to the
//! diagrams so a viewer can take the reader from an element to the ones it
//! is traced to, in whichever view draws them.

use super::{Diagram, Node, NodeKind};
use crate::compiler::ast::Model;
use serde::Serialize;
use std::collections::{BTreeSet, HashMap, HashSet};

/// One trace, its ends resolved to the ids the diagrams use.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Link {
    /// The trace type as written: `realizes`, `satisfies`, `refines`...
    pub kind: String,
    /// Id of a drawn element, or the reference as written when no view
    /// draws it (a requirement) or several drawn elements answer to it.
    pub source: String,
    pub target: String,
}

/// Every trace of the model, in declared order. Two traces of one type
/// between the same elements are one link.
pub(super) fn cross_links(model: &Model, diagrams: &[Diagram]) -> Vec<Link> {
    let drawn = Drawn::of(diagrams, undrawn_names(model));
    let traces = model.traces.iter().chain(
        model
            .operational_analysis
            .iter()
            .flat_map(|oa| oa.traces.iter()),
    );
    let mut seen: HashSet<Link> = HashSet::new();
    traces
        .map(|trace| Link {
            kind: trace.trace_type.clone(),
            source: drawn.resolve(&trace.from),
            target: drawn.resolve(&trace.to),
        })
        .filter(|link| seen.insert(link.clone()))
        .collect()
}

/// The elements the diagrams draw, by id and by name.
struct Drawn<'a> {
    /// Ids and names of elements no view draws.
    undrawn: HashSet<&'a str>,
    ids: HashSet<&'a str>,
    ids_by_name: HashMap<&'a str, BTreeSet<&'a str>>,
}

impl<'a> Drawn<'a> {
    fn of(diagrams: &'a [Diagram], undrawn: HashSet<&'a str>) -> Self {
        let mut drawn = Drawn {
            undrawn,
            ids: HashSet::new(),
            ids_by_name: HashMap::new(),
        };
        let mut stack: Vec<&Node> = diagrams.iter().flat_map(|d| d.nodes.iter()).collect();
        while let Some(node) = stack.pop() {
            stack.extend(node.children.iter());
            if stands_for_an_element(node.kind) {
                drawn.ids.insert(&node.id);
                drawn
                    .ids_by_name
                    .entry(&node.name)
                    .or_default()
                    .insert(&node.id);
            }
        }
        drawn
    }

    /// The id a reference names: itself when it is an id, the element
    /// carrying that name when there is exactly one, else the reference
    /// unchanged. A name an undrawn element also answers to is left as
    /// written: the trace may be about that element.
    fn resolve(&self, reference: &str) -> String {
        if self.ids.contains(reference) || self.undrawn.contains(reference) {
            return reference.to_string();
        }
        let named = self.ids_by_name.get(reference);
        match named.map(|ids| ids.iter().collect::<Vec<_>>()).as_deref() {
            Some([single]) => (**single).to_string(),
            _ => reference.to_string(),
        }
    }
}

/// Ids and names of the elements that have no diagram: requirements, test
/// cases and hazards.
fn undrawn_names(model: &Model) -> HashSet<&str> {
    let mut names: HashSet<&str> = HashSet::new();
    for requirement in model.system_analysis.iter().flat_map(|sa| &sa.requirements) {
        names.insert(&requirement.id);
        for key in ["id", "name", "title"] {
            names.extend(requirement.attributes.get(key).and_then(|v| v.as_string()));
        }
    }
    for case in &model.test_cases {
        names.insert(&case.id);
        names.insert(&case.name);
    }
    for hazard in model
        .safety_analysis
        .iter()
        .flat_map(|safety| &safety.hazards)
    {
        names.insert(&hazard.name);
        names.extend(hazard.attributes.get("id").and_then(|v| v.as_string()));
    }
    names
}

/// Nodes that are an element of the model, as opposed to a stand-in for one
/// (a lifeline, a component shown on the node it is deployed on) or a
/// notation mark.
fn stands_for_an_element(kind: NodeKind) -> bool {
    !matches!(
        kind,
        NodeKind::Lifeline | NodeKind::DeployedComponent | NodeKind::InitialState
    )
}
