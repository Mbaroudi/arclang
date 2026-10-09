//! Operational Architecture Blank: entities and actors containing their
//! activities, linked by operational interactions.

use super::builder::{identity, properties, text, view_title, Builder, NodeRef};
use super::{Diagram, EdgeKind, NodeKind, ViewKind};
use crate::compiler::ast::{EntityType, Model, OperationalActivity, OperationalExchange};
use std::collections::BTreeMap;

const ACTIVITY_PROPERTIES: [&str; 3] = ["description", "safety_level", "asil"];

pub(super) fn build(model: &Model) -> Option<(Diagram, Vec<String>)> {
    let mut builder = Builder::new(ViewKind::Oab);

    for oa in &model.operational_analysis {
        for actor in &oa.actors {
            // Same identity rule as the semantic analyzer.
            let fallback = format!("ACT-{}", actor.name.replace(' ', "-"));
            let id = actor
                .id
                .as_deref()
                .filter(|id| !id.is_empty())
                .unwrap_or_else(|| identity(&actor.attributes, &fallback));
            builder.add_node(
                None,
                id,
                &actor.name,
                NodeKind::OperationalActor,
                properties(&actor.attributes, &["description", "type"]),
            );
        }
        for entity in &oa.entities {
            let kind = match entity.entity_type {
                EntityType::Actor => NodeKind::OperationalActor,
                EntityType::System | EntityType::Environment => NodeKind::OperationalEntity,
            };
            let node = builder.add_node(
                None,
                identity(&entity.attributes, &entity.id),
                &entity.name,
                kind,
                properties(&entity.attributes, &["description", "type"]),
            );
            for activity in &entity.activities {
                add_activity(&mut builder, Some(node), activity);
            }
        }
    }
    // Free-standing activities come last: `performed_by` may name any
    // entity or actor of the model.
    for oa in &model.operational_analysis {
        for activity in &oa.activities {
            let performer = Some(activity.performed_by.as_str()).filter(|p| !p.is_empty());
            let parent = performer.and_then(|p| builder.resolve(p));
            if let (Some(performer), None) = (performer, parent) {
                builder.report(format!(
                    "activity '{}': performer '{performer}' is not a declared entity or actor",
                    activity.name
                ));
            }
            add_activity(&mut builder, parent, activity);
        }
    }
    if builder.is_empty() {
        return None;
    }

    for oa in &model.operational_analysis {
        add_exchanges(
            &mut builder,
            &oa.exchanges,
            EdgeKind::Interaction,
            "interaction",
        );
        add_exchanges(
            &mut builder,
            &oa.communication_means,
            EdgeKind::CommunicationMean,
            "communication_mean",
        );
    }
    for oa in &model.operational_analysis {
        for process in &oa.processes {
            builder.add_chain(&process.id, &process.name, &process.involves);
        }
    }
    builder.finish(view_title(
        ViewKind::Oab,
        model.operational_analysis.iter().map(|oa| oa.name.clone()),
    ))
}

fn add_activity(builder: &mut Builder, parent: Option<NodeRef>, activity: &OperationalActivity) {
    let node = builder.add_node(
        parent,
        identity(&activity.attributes, &activity.id),
        &activity.name,
        NodeKind::OperationalActivity,
        properties(&activity.attributes, &ACTIVITY_PROPERTIES),
    );
    for sub in &activity.sub_activities {
        add_activity(builder, Some(node), sub);
    }
}

fn add_exchanges(
    builder: &mut Builder,
    exchanges: &[OperationalExchange],
    kind: EdgeKind,
    keyword: &str,
) {
    for exchange in exchanges {
        let name = exchange
            .label
            .clone()
            .unwrap_or_else(|| format!("{} -> {}", exchange.from, exchange.to));
        let what = format!("{keyword} '{name}'");
        let Some((source, target)) = builder.resolve_pair(
            &what,
            &exchange.from,
            &exchange.to,
            "entity, actor or activity",
        ) else {
            continue;
        };
        let mut edge_properties = BTreeMap::new();
        if let Some(description) = text(&exchange.attributes, "description") {
            edge_properties.insert("description".to_string(), description.to_string());
        }
        if let Some(protocol) = &exchange.protocol {
            edge_properties.insert("protocol".to_string(), protocol.clone());
        }
        builder.add_edge(kind, &source, &target, &name, None, edge_properties);
    }
}
