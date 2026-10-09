//! System Architecture Blank: the system with its functions, the external
//! actors, and the functional exchanges between function ports.

use super::builder::{identity, properties, view_title, Builder};
use super::functions::{add_function, add_functional_exchanges};
use super::{Diagram, NodeKind, ViewKind};
use crate::compiler::ast::Model;
use std::collections::BTreeMap;

/// Identity of the box standing for the system under study.
const SYSTEM_ID: &str = "SA-System";

pub(super) fn build(model: &Model) -> Option<(Diagram, Vec<String>)> {
    let mut builder = Builder::new(ViewKind::Sab);

    let has_functions = model
        .system_analysis
        .iter()
        .any(|sa| !sa.functions.is_empty());
    if has_functions {
        let name = super::builder::text(&model.attributes, "name").unwrap_or("System");
        let system = builder.add_node(None, SYSTEM_ID, name, NodeKind::System, BTreeMap::new());
        for sa in &model.system_analysis {
            for function in &sa.functions {
                add_function(&mut builder, Some(system), function);
            }
        }
    }
    for sa in &model.system_analysis {
        for actor in &sa.external_actors {
            builder.add_node(
                None,
                identity(&actor.attributes, &actor.id),
                &actor.name,
                NodeKind::SystemActor,
                properties(&actor.attributes, &["description", "type"]),
            );
        }
        for component in &sa.components {
            builder.add_node(
                None,
                identity(&component.attributes, &component.name),
                &component.name,
                NodeKind::SystemComponent,
                properties(
                    &component.attributes,
                    &["description", "type", "safety_level", "asil"],
                ),
            );
        }
    }
    if builder.is_empty() {
        return None;
    }

    add_functional_exchanges(
        &mut builder,
        model
            .system_analysis
            .iter()
            .flat_map(|sa| sa.functional_exchanges.iter()),
        None,
    );
    for sa in &model.system_analysis {
        for chain in &sa.functional_chains {
            builder.add_chain(&chain.id, &chain.name, &chain.involves);
        }
    }
    builder.finish(view_title(
        ViewKind::Sab,
        model.system_analysis.iter().map(|sa| sa.name.clone()),
    ))
}
