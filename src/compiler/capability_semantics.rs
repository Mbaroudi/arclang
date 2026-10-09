//! Resolution of what capabilities refer to: the elements they involve, the
//! capability they realize, their mission, their parent and their relations
//! to other capabilities. A reference that names nothing, or names the
//! wrong kind of element, is an error: it is never dropped.

use super::ast::{AttributeValue, Model};
use super::capability_relations::declared_relations;
use super::semantic::{CapabilityInfo, CapabilityRelationInfo, ElementInfo};
use std::collections::HashMap;

/// Element types that belong to the operational level.
const OPERATIONAL_TYPES: [&str; 6] = [
    "Actor",
    "Entity",
    "Activity",
    "OperationalActivity",
    "OperationalProcess",
    "OperationalCapability",
];

/// Element types a capability can involve: who takes part and what is done.
const INVOLVABLE_TYPES: [&str; 11] = [
    "Actor",
    "Entity",
    "Activity",
    "OperationalActivity",
    "OperationalProcess",
    "SystemActor",
    "SystemComponent",
    "Component",
    "Function",
    "SystemFunction",
    "FunctionalChain",
];

/// Relations that make no sense in a loop: a capability cannot be a special
/// case of itself, nor contain itself.
const ACYCLIC_RELATIONS: [&str; 2] = ["specializes", "includes"];

/// A capability of any level, as its references are resolved.
struct Source<'a> {
    id: &'a str,
    name: &'a str,
    /// Keyword the author wrote, for messages.
    keyword: &'static str,
    /// "Operational", "System" or "Realization".
    kind: &'static str,
    involves: &'a [String],
    realizes: Option<&'a str>,
    mission: Option<&'a str>,
    parent: Option<&'a str>,
    attributes: &'a HashMap<String, AttributeValue>,
}

impl Source<'_> {
    fn element_type(&self) -> String {
        element_type(self.kind)
    }

    fn is_operational(&self) -> bool {
        self.kind == "Operational"
    }

    /// Element type of the capabilities this one can realize.
    fn level_above(&self) -> Option<&'static str> {
        match self.kind {
            "System" => Some("OperationalCapability"),
            "Realization" => Some("SystemCapability"),
            _ => None,
        }
    }
}

/// Element type a capability of this kind is registered under.
pub(super) fn element_type(kind: &str) -> String {
    if kind == "Operational" {
        "OperationalCapability".to_string()
    } else {
        format!("{kind}Capability")
    }
}

fn sources(ast: &Model) -> Vec<Source<'_>> {
    let operational = ast
        .operational_analysis
        .iter()
        .flat_map(|oa| &oa.capabilities)
        .map(|c| Source {
            id: &c.id,
            name: &c.name,
            keyword: "operational_capability",
            kind: "Operational",
            involves: &c.involves,
            realizes: None,
            mission: None,
            parent: c.parent.as_deref(),
            attributes: &c.attributes,
        });
    let system = ast
        .system_analysis
        .iter()
        .flat_map(|sa| &sa.capabilities)
        .map(|c| (c, "System"));
    let logical = ast
        .logical_architecture
        .iter()
        .flat_map(|la| &la.capability_realizations)
        .map(|c| (c, "Realization"));
    let declared = system.chain(logical).map(|(c, kind)| Source {
        id: &c.id,
        name: &c.name,
        keyword: "capability",
        kind,
        involves: &c.involves,
        realizes: c.realizes.as_deref(),
        mission: c.mission.as_deref(),
        parent: c.parent.as_deref(),
        attributes: &c.attributes,
    });
    operational.chain(declared).collect()
}

/// Find the element a reference names: by id, else by name. A name shared
/// by several elements resolves when exactly one of them is `preferred`.
fn resolve<'a>(
    elements: &'a HashMap<String, ElementInfo>,
    reference: &str,
    preferred: impl Fn(&ElementInfo) -> bool,
) -> Result<&'a ElementInfo, String> {
    if let Some(element) = elements.get(reference) {
        return Ok(element);
    }
    let named: Vec<&ElementInfo> = elements.values().filter(|e| e.name == reference).collect();
    match named.as_slice() {
        [single] => Ok(single),
        [] => Err(format!("unknown element '{reference}'")),
        several => {
            let narrowed: Vec<&&ElementInfo> = several.iter().filter(|e| preferred(e)).collect();
            match narrowed.as_slice() {
                [single] => Ok(single),
                _ => Err(format!("ambiguous name '{reference}' — use an id")),
            }
        }
    }
}

/// Resolve every capability of the model. Capabilities must already be
/// registered in `elements`. Errors are appended to `errors`; a reference
/// in error is left out of the result.
pub(super) fn resolve_capabilities(
    ast: &Model,
    elements: &HashMap<String, ElementInfo>,
    errors: &mut Vec<String>,
) -> Vec<CapabilityInfo> {
    let sources = sources(ast);
    report_shared_ids(&sources, errors);
    let resolved: Vec<CapabilityInfo> = sources
        .iter()
        .map(|source| resolve_one(source, elements, errors))
        .collect();
    report_cycles(&sources, &resolved, errors);
    resolved
}

/// Two capabilities under one id would be merged into one element.
fn report_shared_ids(sources: &[Source<'_>], errors: &mut Vec<String>) {
    let mut holders: HashMap<&str, &str> = HashMap::new();
    for source in sources {
        if let Some(other) = holders.get(source.id) {
            errors.push(format!(
                "capabilities '{other}' and '{}' share the id '{}' — give one an explicit unique id",
                source.name, source.id
            ));
        } else {
            holders.insert(source.id, source.name);
        }
    }
}

/// Report each loop of generalization or inclusion once, from the first
/// capability of the loop in source order.
fn report_cycles(sources: &[Source<'_>], resolved: &[CapabilityInfo], errors: &mut Vec<String>) {
    for relation in ACYCLIC_RELATIONS {
        let targets = |id: &str| -> Vec<&str> {
            resolved
                .iter()
                .filter(|capability| capability.id == id)
                .flat_map(|capability| &capability.relations)
                .filter(|r| r.kind == relation)
                .map(|r| r.target.as_str())
                .collect()
        };
        let mut reported: Vec<&str> = Vec::new();
        for source in sources {
            if reported.contains(&source.id) {
                continue;
            }
            if let Some(path) = path_back_to(source.id, &targets) {
                errors.push(format!(
                    "{} '{}' {relation}: cycle {} -> {}",
                    source.keyword,
                    source.name,
                    path.join(" -> "),
                    source.id
                ));
                reported.extend(path);
            }
        }
    }
}

/// The ids from `start` along relations that lead back to `start`, if any.
fn path_back_to<'a>(
    start: &'a str,
    targets: &impl Fn(&str) -> Vec<&'a str>,
) -> Option<Vec<&'a str>> {
    let mut visited: Vec<&str> = vec![start];
    let mut stack: Vec<Vec<&'a str>> = vec![vec![start]];
    while let Some(path) = stack.pop() {
        let last = path[path.len() - 1];
        for next in targets(last) {
            if next == start {
                return Some(path);
            }
            if !visited.contains(&next) {
                visited.push(next);
                let mut longer = path.clone();
                longer.push(next);
                stack.push(longer);
            }
        }
    }
    None
}

/// Resolve a reference that must name an element of one type. A name
/// shared with elements of other types still resolves to that type.
fn typed(
    elements: &HashMap<String, ElementInfo>,
    role: &str,
    written: &str,
    expected: &str,
    what: &str,
    errors: &mut Vec<String>,
) -> Option<String> {
    match resolve(elements, written, |element| {
        element.element_type == expected
    }) {
        Err(problem) => errors.push(format!("{what} {role}: {problem}")),
        Ok(element) if element.element_type != expected => errors.push(format!(
            "{what} {role}: '{}' is a {}, expected a {expected}",
            element.name, element.element_type
        )),
        Ok(element) => return Some(element.id.clone()),
    }
    None
}

/// A reference is a name or a list of names. Anything else the author
/// wrote there would otherwise be read as no reference at all.
fn report_malformed(source: &Source<'_>, what: &str, errors: &mut Vec<String>) {
    for key in ["involves", "extends", "includes", "specializes"] {
        let strangers: Vec<&AttributeValue> = match source.attributes.get(key) {
            None | Some(AttributeValue::String(_)) => Vec::new(),
            Some(AttributeValue::List(items)) => items
                .iter()
                .filter(|item| item.as_string().is_none())
                .collect(),
            Some(other) => vec![other],
        };
        for stranger in strangers {
            errors.push(format!(
                "{what} {key}: expected a name or a list of names, got '{}'",
                stranger.display()
            ));
        }
    }
    for key in ["realizes", "mission"] {
        match source.attributes.get(key) {
            None | Some(AttributeValue::String(_)) => {}
            Some(AttributeValue::List(_)) => {
                errors.push(format!("{what} {key}: expected one name, got a list"));
            }
            Some(other) => errors.push(format!(
                "{what} {key}: expected one name, got '{}'",
                other.display()
            )),
        }
    }
    if source.is_operational() && source.attributes.contains_key("realizes") {
        errors.push(format!(
            "{what} realizes: an operational capability realizes nothing"
        ));
    }
}

fn resolve_one(
    source: &Source<'_>,
    elements: &HashMap<String, ElementInfo>,
    errors: &mut Vec<String>,
) -> CapabilityInfo {
    let what = format!("{} '{}'", source.keyword, source.name);
    let own_type = source.element_type();
    let same_level = |element: &ElementInfo| {
        OPERATIONAL_TYPES.contains(&element.element_type.as_str()) == source.is_operational()
    };
    let reference = |role: &str, reference: &str, errors: &mut Vec<String>| {
        resolve(elements, reference, same_level)
            .map_err(|problem| errors.push(format!("{what} {role}: {problem}")))
            .ok()
    };

    let mut involves = Vec::new();
    for written in source.involves {
        let Some(element) = reference("involves", written, errors) else {
            continue;
        };
        if INVOLVABLE_TYPES.contains(&element.element_type.as_str()) {
            involves.push(element.id.clone());
        } else {
            errors.push(format!(
                "{what} involves: '{}' is a {}, which a capability cannot involve",
                element.name, element.element_type
            ));
        }
    }
    report_malformed(source, &what, errors);
    let realizes = match (source.realizes, source.level_above()) {
        (Some(written), Some(expected)) => {
            typed(elements, "realizes", written, expected, &what, errors)
        }
        _ => None,
    };
    let mission = source
        .mission
        .and_then(|written| typed(elements, "mission", written, "Mission", &what, errors));

    let mut relations: Vec<CapabilityRelationInfo> = Vec::new();
    for (relation, target) in declared_relations(source.attributes) {
        let role = relation.key();
        let same_kind = |element: &ElementInfo| element.element_type == own_type;
        match resolve(elements, target, same_kind) {
            Err(problem) => errors.push(format!("{what} {role}: {problem}")),
            Ok(element) if element.id == source.id => errors.push(format!(
                "{what} {role}: a capability cannot relate to itself"
            )),
            Ok(element) if element.element_type != own_type => errors.push(format!(
                "{what} {role}: '{}' is a {}, not a capability of the same level",
                element.name, element.element_type
            )),
            Ok(element)
                if relations
                    .iter()
                    .any(|r| r.kind == role && r.target == element.id) =>
            {
                errors.push(format!("{what} {role}: '{}' is named twice", element.name));
            }
            Ok(element) => relations.push(CapabilityRelationInfo {
                kind: role.to_string(),
                target: element.id.clone(),
            }),
        }
    }

    CapabilityInfo {
        id: source.id.to_string(),
        name: source.name.to_string(),
        involves,
        realizes,
        mission,
        kind: source.kind.to_string(),
        parent: source.parent.map(str::to_string),
        relations,
    }
}
