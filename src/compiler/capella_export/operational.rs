//! The operational analysis: entities and actors (nested, with their parts),
//! operational activities (nested, with their ports) and who performs them,
//! interactions, communication means, operational processes and
//! capabilities.

use super::context::{derived, id_of, port_id, text, Context};
use super::header::apply;
use super::relations;
use super::xml::El;
use crate::compiler::ast::{AttributeValue, OperationalActivity, OperationalAnalysis, PortDirection};
use std::collections::HashMap;

fn level_id(oa: &OperationalAnalysis) -> String {
    if !oa.id.is_empty() {
        return oa.id.clone();
    }
    derived(&format!("operational-analysis:{}", oa.name))
}

/// An entity or an actor, as the levels below see it.
struct Entity<'a> {
    id: String,
    name: &'a str,
    actor: bool,
    attributes: &'a HashMap<String, AttributeValue>,
}

fn entities(oa: &OperationalAnalysis) -> Vec<Entity<'_>> {
    let actors = oa.actors.iter().map(|a| Entity { id: a.id.clone().unwrap_or_else(|| id_of(&a.attributes, &format!("actor:{}", a.name))), name: &a.name, actor: true, attributes: &a.attributes });
    let entities = oa.entities.iter().map(|e| Entity { id: e.id.clone(), name: &e.name, actor: false, attributes: &e.attributes });
    actors.chain(entities).collect()
}

fn interaction_id(exchange: &crate::compiler::ast::OperationalExchange, index: usize, what: &str) -> String {
    id_of(&exchange.attributes, &format!("{what}:{index}:{}->{}", exchange.from, exchange.to))
}

fn register_activity(context: &mut Context, activity: &OperationalActivity) {
    context.functions.insert(activity.id.clone());
    context.register_function_name(&activity.name, &activity.id);
    context.register(&activity.id, &activity.id);
    context.register(&activity.name, &activity.id);
    for port in &activity.ports {
        let id = port_id(&activity.id, &port.name, &port.id);
        context.register(&format!("{}.{}", activity.id, port.name), &id);
    }
    for sub in &activity.sub_activities {
        register_activity(context, sub);
    }
}

pub(super) fn register(context: &mut Context, oa: &OperationalAnalysis) {
    for entity in entities(oa) {
        context.register(&entity.id, &entity.id);
        context.register(entity.name, &entity.id);
    }
    for activity in &oa.activities {
        register_activity(context, activity);
    }
    for (index, exchange) in oa.exchanges.iter().enumerate() {
        let id = interaction_id(exchange, index, "interaction");
        context.register(&id, &id);
    }
    for (index, mean) in oa.communication_means.iter().enumerate() {
        let id = interaction_id(mean, index, "communication-means");
        context.register(&id, &id);
    }
    for process in &oa.processes {
        context.chains.insert(process.id.clone());
        context.register(&process.id, &process.id);
        context.register(&process.name, &process.id);
    }
    for capability in &oa.capabilities {
        context.register(&capability.id, &capability.id);
        context.register(&capability.name, &capability.id);
    }
}

fn activity(activity: &OperationalActivity) -> El {
    let mut el = apply(El::new("ownedFunctions", "oa:OperationalActivity", &activity.id).attr("name", &activity.name), &activity.attributes, &["performed_by", "kind"]);
    for port in &activity.ports {
        let (tag, kind) = if matches!(port.direction, PortDirection::Out) { ("outputs", "fa:FunctionOutputPort") } else { ("inputs", "fa:FunctionInputPort") };
        el.push(El::new(tag, kind, &port_id(&activity.id, &port.name, &port.id)).attr("name", &port.name));
    }
    for sub in &activity.sub_activities {
        el.push(self::activity(sub));
    }
    el
}

/// Who performs each activity: `performed_by` of the activity, by entity id.
fn performers(context: &Context, activities: &[OperationalActivity], into: &mut HashMap<String, Vec<String>>) {
    for a in activities {
        let performer = if a.performed_by.is_empty() { text(&a.attributes, "performed_by").unwrap_or("") } else { a.performed_by.as_str() };
        if let Some(entity) = context.resolve(performer) {
            into.entry(entity.to_string()).or_default().push(a.id.clone());
        }
        performers(context, &a.sub_activities, into);
    }
}

fn entity_element(entity: &Entity<'_>, performed: &HashMap<String, Vec<String>>, children: &HashMap<String, Vec<&Entity<'_>>>) -> El {
    let mut el = El::new("ownedEntities", "oa:Entity", &entity.id).attr("name", entity.name);
    if entity.actor {
        el.set("actor", "true");
    }
    let mut el = apply(el, entity.attributes, &["part_of"]);
    for activity in performed.get(&entity.id).into_iter().flatten() {
        el.push(
            El::new("ownedFunctionalAllocation", "fa:ComponentFunctionalAllocation", &derived(&format!("{}->{activity}:allocation", entity.id)))
                .attr("targetElement", &format!("#{activity}"))
                .attr("sourceElement", &format!("#{}", entity.id)),
        );
    }
    for child in children.get(&entity.id).into_iter().flatten() {
        el.push(El::new("ownedFeatures", "cs:Part", &derived(&format!("{}:part", child.id))).attr("name", child.name).attr("abstractType", &format!("#{}", child.id)));
    }
    for child in children.get(&entity.id).into_iter().flatten() {
        el.push(entity_element(child, performed, children));
    }
    el
}

pub(super) fn write(context: &mut Context, oa: &OperationalAnalysis) -> El {
    let id = level_id(oa);
    let mut level = El::new("ownedArchitectures", "oa:OperationalAnalysis", &id).attr("name", &oa.name);

    let all = entities(oa);
    let mut children: HashMap<String, Vec<&Entity<'_>>> = HashMap::new();
    let mut tops = Vec::new();
    for entity in &all {
        match text(entity.attributes, "part_of").and_then(|p| context.resolve(p)) {
            Some(parent) => children.entry(parent.to_string()).or_default().push(entity),
            None => tops.push(entity),
        }
    }
    let mut performed = HashMap::new();
    performers(context, &oa.activities, &mut performed);

    // Interactions and their ends, for the processes.
    let mut ends: HashMap<String, (String, String)> = HashMap::new();
    let mut root = El::new("ownedOperationalActivities", "oa:OperationalActivity", &derived(&format!("{id}:root-activity"))).attr("name", "Root Operational Activity");
    for a in &oa.activities {
        root.push(activity(a));
    }
    for (index, exchange) in oa.exchanges.iter().enumerate() {
        let xid = interaction_id(exchange, index, "interaction");
        let (Some(from), Some(to)) = (context.resolve(&exchange.from).map(str::to_string), context.resolve(&exchange.to).map(str::to_string)) else {
            context.note(format!("interaction '{}' left out: an end is not exported", exchange.label.as_deref().unwrap_or(&xid)));
            continue;
        };
        ends.insert(xid.clone(), (from.clone(), to.clone()));
        let el = El::new("ownedFunctionalExchanges", "fa:FunctionalExchange", &xid).attr("name", exchange.label.as_deref().unwrap_or("")).attr("target", &format!("#{to}")).attr("source", &format!("#{from}"));
        root.push(relations::exchange(context, el, &exchange.attributes));
    }
    for process in &oa.processes {
        let el = apply(El::new("ownedFunctionalChains", "oa:OperationalProcess", &process.id).attr("name", &process.name), &process.attributes, &["involves"]);
        root.push(relations::chain_steps(context, el, process, &ends));
    }
    super::functions::place_exchanges(&mut root);
    level.push(El::new("ownedFunctionPkg", "oa:OperationalActivityPkg", &derived(&format!("{id}:activities"))).attr("name", "Operational Activities").with(root));

    let mut pkg = El::new("ownedEntityPkg", "oa:EntityPkg", &derived(&format!("{id}:entities"))).attr("name", "Operational Entities");
    for entity in &tops {
        pkg.push(entity_element(entity, &performed, &children));
    }
    for (index, mean) in oa.communication_means.iter().enumerate() {
        let mid = interaction_id(mean, index, "communication-means");
        let (Some(from), Some(to)) = (context.resolve(&mean.from).map(str::to_string), context.resolve(&mean.to).map(str::to_string)) else { continue };
        let el = El::new("ownedComponentExchanges", "oa:CommunicationMean", &mid).attr("name", mean.label.as_deref().unwrap_or("")).attr("source", &format!("#{from}")).attr("target", &format!("#{to}"));
        pkg.push(relations::exchange(context, el, &mean.attributes));
    }
    for entity in &tops {
        pkg.push(El::new("ownedParts", "cs:Part", &derived(&format!("{}:part", entity.id))).attr("name", entity.name).attr("abstractType", &format!("#{}", entity.id)));
    }
    level.push(pkg);

    let mut capabilities = El::new("ownedAbstractCapabilityPkg", "oa:OperationalCapabilityPkg", &derived(&format!("{id}:capabilities"))).attr("name", "Operational Capabilities");
    let entity_ids: Vec<&str> = all.iter().map(|e| e.id.as_str()).collect();
    let process_ids: Vec<&str> = oa.processes.iter().map(|p| p.id.as_str()).collect();
    for capability in &oa.capabilities {
        let mut el = apply(
            El::new("ownedOperationalCapabilities", "oa:OperationalCapability", &capability.id).attr("name", &capability.name),
            &capability.attributes,
            &["involves", "extends", "includes", "specializes", "parent", "level"],
        );
        for involved in &capability.involves {
            let Some(target) = context.resolve(involved).map(str::to_string) else { continue };
            let (tag, kind) = if entity_ids.contains(&target.as_str()) {
                ("ownedEntityOperationalCapabilityInvolvements", "oa:EntityOperationalCapabilityInvolvement")
            } else if process_ids.contains(&target.as_str()) {
                ("ownedFunctionalChainAbstractCapabilityInvolvements", "interaction:FunctionalChainAbstractCapabilityInvolvement")
            } else {
                ("ownedAbstractFunctionAbstractCapabilityInvolvements", "interaction:AbstractFunctionAbstractCapabilityInvolvement")
            };
            el.push(El::new(tag, kind, &derived(&format!("{}->{target}:involvement", capability.id))).attr("involved", &format!("#{target}")));
        }
        capabilities.push(relations::capability(context, el, &capability.id, &capability.attributes, None));
    }
    level.push(capabilities);
    level
}

/// An empty operational analysis.
pub(super) fn empty(project_id: &str) -> El {
    // As Capella's own empty project: no package, no root function.
    El::new("ownedArchitectures", "oa:OperationalAnalysis", &derived(&format!("{project_id}:oa"))).attr("name", "Operational Analysis")
}
