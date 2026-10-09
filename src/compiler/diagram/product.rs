//! Product breakdown structure (EPBS): the configuration items the product
//! is delivered as, from systems down to items.

use super::builder::{all_properties, identity, Builder, NodeRef};
use super::{Diagram, EdgeKind, NodeKind, ViewKind};
use crate::compiler::ast::{AttributeValue, Model};
use std::collections::{BTreeMap, HashMap};

pub(super) fn build(model: &Model) -> Option<(Diagram, Vec<String>)> {
    let mut builder = Builder::new(ViewKind::Pbs);

    for epbs in &model.epbs {
        for system in &epbs.systems {
            let system_node = add_item(
                &mut builder,
                None,
                "",
                &system.name,
                &system.attributes,
                "system",
            );
            let system_id = builder.node(system_node).id.clone();
            for subsystem in &system.subsystems {
                let subsystem_node = add_item(
                    &mut builder,
                    Some(system_node),
                    &system_id,
                    &subsystem.name,
                    &subsystem.attributes,
                    "subsystem",
                );
                let subsystem_id = builder.node(subsystem_node).id.clone();
                for item in &subsystem.items {
                    add_item(
                        &mut builder,
                        Some(subsystem_node),
                        &subsystem_id,
                        &item.name,
                        &item.attributes,
                        "item",
                    );
                }
            }
        }
    }
    let title = model
        .epbs
        .iter()
        .map(|epbs| epbs.name.clone())
        .find(|name| !name.is_empty())
        .unwrap_or_else(|| ViewKind::Pbs.label().to_string());
    builder.finish(title)
}

/// Add a configuration item and the link from the item it is part of.
///
/// The breakdown is drawn as a tree of links, not as nested boxes, so the
/// nodes stay flat. An item without an explicit id is identified by its
/// path (`<parent id>/<name>`): the same part name may appear under two
/// subsystems.
fn add_item(
    builder: &mut Builder,
    parent: Option<NodeRef>,
    parent_id: &str,
    name: &str,
    attributes: &HashMap<String, AttributeValue>,
    level: &str,
) -> NodeRef {
    let path = if parent_id.is_empty() {
        name.to_string()
    } else {
        format!("{parent_id}/{name}")
    };
    let mut properties = all_properties(attributes);
    // Not `level`: an item may carry an attribute of that name.
    properties.insert("breakdown_level".to_string(), level.to_string());
    let node = builder.add_node(
        None,
        identity(attributes, &path),
        name,
        NodeKind::ConfigurationItem,
        properties,
    );
    if let Some(parent) = parent {
        let (source, target) = (builder.whole(parent), builder.whole(node));
        let link = format!("{} / {name}", builder.node(parent).name);
        let edge = builder.add_edge(
            EdgeKind::Breakdown,
            &source,
            &target,
            &link,
            None,
            BTreeMap::new(),
        );
        builder.set_edge_label(&edge, String::new());
    }
    node
}
