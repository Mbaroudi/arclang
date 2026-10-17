//! The EPBS: the tree of configuration items, each with its parts and the
//! physical components it realizes. An item that names another as
//! `part_of:` is a configuration item of that one.

use super::context::{derived, id_of, list, text, Context};
use super::header::apply;
use super::xml::El;
use crate::compiler::ast::{AttributeValue, Epbs};
use std::collections::HashMap;

/// A configuration item before nesting: id, name, attributes, parent.
struct Item<'a> {
    id: String,
    name: &'a str,
    attributes: &'a HashMap<String, AttributeValue>,
    parent: Option<String>,
}

fn items(epbs: &Epbs) -> Vec<Item<'_>> {
    let mut all = Vec::new();
    for system in &epbs.systems {
        let sid = id_of(&system.attributes, &format!("ci:{}", system.name));
        all.push(Item { id: sid.clone(), name: &system.name, attributes: &system.attributes, parent: None });
        for sub in &system.subsystems {
            let bid = id_of(&sub.attributes, &format!("ci:{}:{}", sid, sub.name));
            all.push(Item { id: bid.clone(), name: &sub.name, attributes: &sub.attributes, parent: Some(sid.clone()) });
            for item in &sub.items {
                let iid = id_of(&item.attributes, &format!("ci:{}:{}", bid, item.name));
                all.push(Item { id: iid, name: &item.name, attributes: &item.attributes, parent: Some(bid.clone()) });
            }
        }
    }
    all
}

pub(super) fn register(context: &mut Context, epbs: &Epbs) {
    for item in items(epbs) {
        context.register(&item.id, &item.id);
        context.register(item.name, &item.id);
    }
}

fn element(context: &Context, item: &Item<'_>, children: &HashMap<String, Vec<&Item<'_>>>) -> El {
    let el = El::new("ownedConfigurationItems", "epbs:ConfigurationItem", &item.id).attr("name", item.name).opt("kind", text(item.attributes, "kind"));
    let mut el = apply(el, item.attributes, &["kind", "part_of", "realizes"]);
    for target in list(item.attributes, "realizes") {
        if let Some(target) = context.resolve(&target) {
            el.push(super::realization("ownedPhysicalArtifactRealizations", "epbs:PhysicalArtifactRealization", &item.id, target));
        }
    }
    let mine = children.get(&item.id).map(Vec::as_slice).unwrap_or(&[]);
    for child in mine {
        el.push(El::new("ownedFeatures", "cs:Part", &derived(&format!("{}:part", child.id))).attr("name", child.name).attr("abstractType", &format!("#{}", child.id)));
    }
    for child in mine {
        el.push(element(context, child, children));
    }
    el
}

/// The EPBS architecture, with its realization of the physical one.
pub(super) fn write(context: &Context, epbs: &Epbs, project_id: &str) -> El {
    let id = derived(&format!("{project_id}:epbs"));
    let mut level = El::new("ownedArchitectures", "epbs:EPBSArchitecture", &id).attr("name", &epbs.name);
    let all = items(epbs);
    let mut children: HashMap<String, Vec<&Item<'_>>> = HashMap::new();
    let mut roots = Vec::new();
    for item in &all {
        // `part_of:` moves an item under the item it names.
        let parent = text(item.attributes, "part_of").and_then(|p| context.resolve(p)).map(str::to_string).or_else(|| item.parent.clone());
        match parent {
            Some(parent) => children.entry(parent).or_default().push(item),
            None => roots.push(item),
        }
    }
    let mut pkg = El::new("ownedConfigurationItemPkg", "epbs:ConfigurationItemPkg", &derived(&format!("{id}:items"))).attr("name", "Configuration Items");
    for root in &roots {
        pkg.push(element(context, root, &children));
    }
    for root in &roots {
        pkg.push(El::new("ownedParts", "cs:Part", &derived(&format!("{}:part", root.id))).attr("name", root.name).attr("abstractType", &format!("#{}", root.id)));
    }
    level.push(pkg);
    level
}
