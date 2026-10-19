//! Tags: metadata on any element (`tags: ["Safety"]`, SysML v2 `#Safety`).
//! A declared diagram `tagged: [...]` draws the elements carrying one of
//! them.

use crate::compiler::ast::*;
use std::collections::{HashMap, HashSet};

/// The tags an element carries.
pub fn tags_of(attributes: &HashMap<String, AttributeValue>) -> Vec<String> {
    match attributes.get("tags") {
        Some(AttributeValue::List(items)) => items.iter().filter_map(|v| v.as_string()).map(str::to_string).collect(),
        Some(AttributeValue::String(one)) => vec![one.clone()],
        _ => Vec::new(),
    }
}

/// The ids and names of the elements carrying one of `wanted`.
pub fn tagged(model: &Model, wanted: &[String]) -> HashSet<String> {
    let mut out = HashSet::new();
    let mut visit = |id: &str, name: &str, attributes: &HashMap<String, AttributeValue>| {
        if tags_of(attributes).iter().any(|t| wanted.contains(t)) {
            let written = attributes.get("id").and_then(|v| v.as_string()).unwrap_or(id);
            out.extend([written.to_string(), id.to_string(), name.to_string()]);
        }
    };
    fn functions(list: &[LogicalFunction], visit: &mut dyn FnMut(&str, &str, &HashMap<String, AttributeValue>)) {
        for f in list {
            visit(&f.id, &f.name, &f.attributes);
            functions(&f.sub_functions, visit);
        }
    }
    fn components(list: &[LogicalComponent], visit: &mut dyn FnMut(&str, &str, &HashMap<String, AttributeValue>)) {
        for c in list {
            visit(&c.id, &c.name, &c.attributes);
            functions(&c.functions, visit);
            components(&c.sub_components, visit);
        }
    }
    fn system_functions(list: &[SystemFunction], visit: &mut dyn FnMut(&str, &str, &HashMap<String, AttributeValue>)) {
        for f in list {
            visit(&f.id, &f.name, &f.attributes);
            system_functions(&f.sub_functions, visit);
        }
    }
    for oa in &model.operational_analysis {
        for entity in &oa.entities {
            visit(&entity.id, &entity.name, &entity.attributes);
        }
    }
    for sa in &model.system_analysis {
        system_functions(&sa.functions, &mut visit);
        for component in &sa.components {
            visit(&component.name, &component.name, &component.attributes);
        }
    }
    for la in &model.logical_architecture {
        components(&la.components, &mut visit);
        functions(&la.functions, &mut visit);
    }
    for pa in &model.physical_architecture {
        for node in &pa.nodes {
            visit(&node.id, &node.name, &node.attributes);
            functions(&node.functions, &mut visit);
            for behavior in &node.behavior_components {
                visit(&behavior.id, &behavior.name, &behavior.attributes);
                functions(&behavior.functions, &mut visit);
            }
        }
        functions(&pa.functions, &mut visit);
    }
    out
}
