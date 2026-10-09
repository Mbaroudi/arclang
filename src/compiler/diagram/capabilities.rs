//! Missions and capabilities: what the system is for, which capability
//! realizes which, how capabilities of one level nest and relate, and the
//! functions, actors and chains each one involves. Operational, system and
//! logical capabilities share one diagram so the realization chain across
//! levels is visible.

use super::builder::{identity, properties, Builder, Endpoint, Missing, NodeRef};
use super::elements::{all_elements, is_operational, ElementRef};
use super::{Diagram, EdgeKind, NodeKind, ViewKind};
use crate::compiler::ast::{
    AttributeValue, Capability, CapabilityLevel, Model, OperationalCapability,
};
use crate::compiler::capability_relations::{declared_relations, CapabilityRelation};
use std::collections::{BTreeMap, HashMap};

/// Arcadia levels that declare capabilities, top down. A capability
/// realizes one of the level above and relates to those of its own.
const OPERATIONAL: usize = 0;
const SYSTEM: usize = 1;
const LOGICAL: usize = 2;

/// The capabilities drawn for one level, by id and by name.
#[derive(Default)]
struct Level<'a> {
    by_key: HashMap<&'a str, Vec<NodeRef>>,
    /// First capability drawn under each id: what `parent` refers to.
    by_id: HashMap<&'a str, NodeRef>,
}

impl<'a> Level<'a> {
    fn add(&mut self, id: &'a str, name: &'a str, node: NodeRef) {
        self.by_id.entry(id).or_insert(node);
        for key in [id, name] {
            let holders = self.by_key.entry(key).or_default();
            if !holders.contains(&node) {
                holders.push(node);
            }
        }
    }

    fn find(&self, reference: &str) -> Result<NodeRef, Missing> {
        match self.by_key.get(reference).map(Vec::as_slice) {
            Some([single]) => Ok(*single),
            Some(_) => Err(Missing::Ambiguous),
            None => Err(Missing::Unknown),
        }
    }
}

/// A drawn capability and what it refers to.
struct Declared<'a> {
    node: NodeRef,
    level: usize,
    name: &'a str,
    involves: &'a [String],
    realizes: Option<&'a str>,
    mission: Option<&'a str>,
    attributes: &'a HashMap<String, AttributeValue>,
}

pub(super) fn build(model: &Model) -> Option<(Diagram, Vec<String>)> {
    let mut builder = Builder::new(ViewKind::Cap);
    let mut levels: [Level<'_>; 3] = Default::default();
    let mut declared: Vec<Declared<'_>> = Vec::new();

    for oa in &model.operational_analysis {
        for capability in &oa.capabilities {
            let parent = parent_node(&mut builder, &levels[OPERATIONAL], capability);
            add_operational(
                &mut builder,
                &mut levels[OPERATIONAL],
                &mut declared,
                parent,
                capability,
            );
        }
    }
    for sa in &model.system_analysis {
        for mission in &sa.missions {
            builder.add_node(
                None,
                identity(&mission.attributes, &mission.id),
                &mission.name,
                NodeKind::Mission,
                properties(&mission.attributes, &["description"]),
            );
        }
    }
    let system = model
        .system_analysis
        .iter()
        .flat_map(|sa| &sa.capabilities)
        .map(|capability| (SYSTEM, NodeKind::Capability, capability));
    let logical = model
        .logical_architecture
        .iter()
        .flat_map(|la| &la.capability_realizations)
        .map(|capability| (LOGICAL, NodeKind::CapabilityRealization, capability));
    for (level, kind, capability) in system.chain(logical) {
        add_capability(
            &mut builder,
            &mut levels[level],
            &mut declared,
            level,
            kind,
            capability,
        );
    }
    if builder.is_empty() {
        return None;
    }

    // Links are added once every mission and capability exists: a
    // capability may realize or extend one declared after it.
    let elements = all_elements(model);
    // Involved elements already drawn, by element id. Kept apart from the
    // builder's own index: an element whose id happens to equal a
    // capability's name must not be mistaken for that capability.
    let mut involved: HashMap<&str, NodeRef> = HashMap::new();
    for capability in &declared {
        let what = format!("capability '{}'", capability.name);
        link_mission(&mut builder, capability, &what);
        link_realization(&mut builder, &levels, capability, &what);
        link_relations(&mut builder, &levels[capability.level], capability, &what);

        let this = builder.whole(capability.node);
        for reference in capability.involves {
            let element = match involved_element(&elements, reference, capability.level) {
                Ok(element) => element,
                Err(missing) => {
                    builder.report_missing(
                        &what,
                        reference,
                        missing,
                        "function, actor, activity or chain",
                    );
                    continue;
                }
            };
            // An involved element is drawn once, however many capabilities
            // involve it.
            let target = *involved.entry(element.id).or_insert_with(|| {
                builder.add_node(
                    None,
                    element.id,
                    element.name,
                    element.kind,
                    BTreeMap::new(),
                )
            });
            let target = builder.whole(target);
            let name = format!("{} involves {reference}", capability.name);
            link(&mut builder, EdgeKind::Involvement, &this, &target, &name);
        }
    }

    for oa in &model.operational_analysis {
        for association in &oa.capability_associations {
            let label = association
                .label
                .clone()
                .unwrap_or_else(|| association.association_type.clone());
            let what = format!("capability association '{label}'");
            let Some((source, target)) =
                builder.resolve_pair(&what, &association.from, &association.to, "capability")
            else {
                continue;
            };
            let name = format!("{} {label} {}", association.from, association.to);
            let edge = builder.add_edge(
                EdgeKind::CapabilityAssociation,
                &source,
                &target,
                &name,
                None,
                BTreeMap::new(),
            );
            builder.set_edge_label(&edge, label);
        }
    }
    builder.finish("Missions and capabilities".to_string())
}

/// The drawn capability this one is declared inside. A parent that is not
/// drawn is reported and the capability is drawn at the top.
fn parent_of(
    builder: &mut Builder,
    level: &Level<'_>,
    name: &str,
    parent: Option<&str>,
) -> Option<NodeRef> {
    let parent = parent.filter(|id| !id.is_empty())?;
    match level.by_id.get(parent) {
        Some(node) => Some(*node),
        None => {
            builder.report(format!(
                "capability '{name}': its parent '{parent}' is not a capability declared before \
                 it — drawn at the top"
            ));
            None
        }
    }
}

fn parent_node(
    builder: &mut Builder,
    level: &Level<'_>,
    capability: &OperationalCapability,
) -> Option<NodeRef> {
    parent_of(
        builder,
        level,
        &capability.name,
        capability.parent.as_deref(),
    )
}

fn add_operational<'a>(
    builder: &mut Builder,
    level: &mut Level<'a>,
    declared: &mut Vec<Declared<'a>>,
    parent: Option<NodeRef>,
    capability: &'a OperationalCapability,
) {
    let kind = match capability.level {
        CapabilityLevel::Mission => NodeKind::Mission,
        CapabilityLevel::Capability | CapabilityLevel::SubCapability => {
            NodeKind::OperationalCapability
        }
    };
    let id = identity(&capability.attributes, &capability.id);
    let node = builder.add_node(
        parent,
        id,
        &capability.name,
        kind,
        properties(&capability.attributes, &["description"]),
    );
    level.add(id, &capability.name, node);
    declared.push(Declared {
        node,
        level: OPERATIONAL,
        name: &capability.name,
        involves: &capability.involves,
        realizes: None,
        mission: None,
        attributes: &capability.attributes,
    });
    // Capabilities nested as a tree rather than by `parent`: models built
    // outside the parser.
    for child in &capability.children {
        add_operational(builder, level, declared, Some(node), child);
    }
}

fn add_capability<'a>(
    builder: &mut Builder,
    level: &mut Level<'a>,
    declared: &mut Vec<Declared<'a>>,
    index: usize,
    kind: NodeKind,
    capability: &'a Capability,
) {
    let parent = parent_of(
        builder,
        level,
        &capability.name,
        capability.parent.as_deref(),
    );
    let id = identity(&capability.attributes, &capability.id);
    let node = builder.add_node(
        parent,
        id,
        &capability.name,
        kind,
        properties(&capability.attributes, &["description"]),
    );
    level.add(id, &capability.name, node);
    declared.push(Declared {
        node,
        level: index,
        name: &capability.name,
        involves: &capability.involves,
        realizes: capability.realizes.as_deref(),
        mission: capability.mission.as_deref(),
        attributes: &capability.attributes,
    });
}

fn link_mission(builder: &mut Builder, capability: &Declared<'_>, what: &str) {
    let Some(mission) = capability.mission.filter(|m| !m.is_empty()) else {
        return;
    };
    match builder.find(mission) {
        Ok(target) if builder.node(target).kind == NodeKind::Mission => {
            let (source, this) = (builder.whole(target), builder.whole(capability.node));
            let name = format!("{} exploits {}", builder.node(target).name, capability.name);
            link(builder, EdgeKind::Exploitation, &source, &this, &name);
        }
        Ok(target) => {
            let other = builder.node(target).name.clone();
            builder.report(format!(
                "{what}: mission '{mission}' names '{other}', which is not a mission"
            ));
        }
        Err(missing) => builder.report_missing(what, mission, missing, "mission"),
    }
}

fn link_realization(
    builder: &mut Builder,
    levels: &[Level<'_>; 3],
    capability: &Declared<'_>,
    what: &str,
) {
    let Some(realized) = capability.realizes.filter(|r| !r.is_empty()) else {
        return;
    };
    // A capability realizes one of the level above: looked up there first,
    // so a name shared across levels still resolves.
    let above = capability
        .level
        .checked_sub(1)
        .and_then(|level| levels[level].find(realized).ok());
    let found = match above {
        Some(target) => Ok(target),
        None => builder.find(realized),
    };
    match found {
        Ok(target) if target == capability.node => {
            builder.report(format!("{what}: a capability cannot realize itself"));
        }
        Ok(target) if is_capability(builder.node(target).kind) => {
            let (this, target) = (builder.whole(capability.node), builder.whole(target));
            let name = format!("{} realizes {realized}", capability.name);
            link(builder, EdgeKind::Realization, &this, &target, &name);
        }
        Ok(target) => {
            let other = builder.node(target).name.clone();
            builder.report(format!(
                "{what}: realizes '{realized}' names '{other}', which is not a capability"
            ));
        }
        Err(missing) => builder.report_missing(what, realized, missing, "capability"),
    }
}

/// Extend, include and generalization towards capabilities of the level.
fn link_relations(builder: &mut Builder, level: &Level<'_>, capability: &Declared<'_>, what: &str) {
    for (relation, reference) in declared_relations(capability.attributes) {
        let key = relation.key();
        match level.find(reference) {
            Ok(target) if target == capability.node => {
                builder.report(format!("{what}: {key} names the capability itself"));
            }
            Ok(target) => {
                let (this, target) = (builder.whole(capability.node), builder.whole(target));
                let name = format!("{} {key} {reference}", capability.name);
                let (kind, label) = match relation {
                    CapabilityRelation::Extends => (EdgeKind::CapabilityAssociation, "«extend»"),
                    CapabilityRelation::Includes => (EdgeKind::CapabilityAssociation, "«include»"),
                    CapabilityRelation::Specializes => (EdgeKind::Generalization, ""),
                };
                let edge = builder.add_edge(kind, &this, &target, &name, None, BTreeMap::new());
                builder.set_edge_label(&edge, label.to_string());
            }
            Err(missing) => builder.report_missing(
                &format!("{what} {key}"),
                reference,
                missing,
                "capability of the same level",
            ),
        }
    }
}

/// The element an involvement names: by id, else by name. A name carried
/// by several elements names the one of the capability's level, when there
/// is exactly one; otherwise it is ambiguous, as it is for the compiler.
fn involved_element<'a>(
    elements: &[ElementRef<'a>],
    reference: &str,
    level: usize,
) -> Result<ElementRef<'a>, Missing> {
    if let Some(element) = elements.iter().find(|element| element.id == reference) {
        return Ok(*element);
    }
    let named: Vec<&ElementRef<'a>> = elements
        .iter()
        .filter(|element| element.name == reference)
        .collect();
    match named.as_slice() {
        [] => Err(Missing::Unknown),
        [single] => Ok(**single),
        several => {
            let operational = level == OPERATIONAL;
            let mut same_level = several
                .iter()
                .filter(|element| is_operational(element.kind) == operational);
            match (same_level.next(), same_level.next()) {
                (Some(single), None) => Ok(**single),
                _ => Err(Missing::Ambiguous),
            }
        }
    }
}

fn is_capability(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::OperationalCapability | NodeKind::Capability | NodeKind::CapabilityRealization
    )
}

/// Add an unlabelled link: its kind says what it means, its id says which.
fn link(builder: &mut Builder, kind: EdgeKind, source: &Endpoint, target: &Endpoint, name: &str) {
    let edge = builder.add_edge(kind, source, target, name, None, BTreeMap::new());
    builder.set_edge_label(&edge, String::new());
}
