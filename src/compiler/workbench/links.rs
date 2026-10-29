//! Edits of what connects elements, and copies: an exchange, a link, a
//! transition, a message or a trace reconnected or turned around; a
//! relationship renamed; a port moved to another owner with what it
//! carries; an element copied with new identities. Each one is checked on
//! the compiled model: every element keeps its identity, and the
//! relationships are those of the original with exactly the asked change.

use super::{compile_text, identities, written_designator};
use crate::compiler::elements::{self, ElementGraph, ElementRecord, RelationshipRecord};
use crate::compiler::source_edit::{self, Container, CopyTarget, RelationshipEnds, RelationshipSpots, Spot};
use std::collections::BTreeMap;

/// The keywords a relationship of `kind` may be declared with.
fn keywords_of(kind: &str) -> &'static [&'static str] {
    match kind {
        "FunctionalExchange" => &["functional_exchange", "exchange"],
        "ComponentExchange" => &["component_exchange", "connection"],
        "OperationalExchange" => &["interaction", "operational_exchange", "exchange"],
        "PhysicalLink" => &["physical_link", "link"],
        "PhysicalExchange" => &["physical_exchange"],
        "Transition" => &["transition"],
        "Message" => &["message"],
        "Trace" => &["trace"],
        "Association" => &["association"],
        _ => &[],
    }
}

fn graph_of(source: &str, what: &str) -> Result<ElementGraph, String> {
    Ok(compiled_graph(source, what)?.0)
}

/// The element graph of a text, with the compiler's warnings about it.
fn compiled_graph(source: &str, what: &str) -> Result<(ElementGraph, Vec<String>), String> {
    let compiled = compile_text(source).map_err(|e| format!("{what} does not compile: {e}"))?;
    Ok((elements::build(&compiled.ast, &compiled.semantic_model), compiled.warnings))
}

/// The kinds a reconnection names, as the element graph spells them.
const RECONNECTABLE: &[&str] = &["ComponentExchange", "FunctionalExchange", "OperationalExchange", "PhysicalLink", "PhysicalExchange", "Transition", "Message", "Trace", "Association"];

/// A port's direction as the compiler read it: `in`, `out`, `inout`; None when it has none.
fn direction_of(element: &ElementRecord) -> Option<&str> {
    element.extra.get("direction").and_then(|d| d.as_str()).filter(|d| matches!(*d, "in" | "out" | "inout"))
}

/// The rules a new end of a relationship follows, as when it is created:
/// a source leaves by an out (or inout) port, a target arrives by an in
/// (or inout) port, and an exchange joins two different ends.
fn check_ends(kind: &str, source: &ElementRecord, target: &ElementRecord, moved_source: bool, moved_target: bool) -> Result<(), String> {
    if source.uuid == target.uuid && !matches!(kind, "Transition" | "Message") {
        return Err(format!("edit refused: a {kind} does not join '{}' to itself", source.id));
    }
    if moved_source && direction_of(source) == Some("in") {
        return Err(format!("edit refused: '{}' is an in port; the source of a {kind} leaves by an out or inout port", source.id));
    }
    if moved_target && direction_of(target) == Some("out") {
        return Err(format!("edit refused: '{}' is an out port; the target of a {kind} arrives by an in or inout port", target.id));
    }
    Ok(())
}

/// The element a designator names: its id, else its name when only one carries it.
fn resolve<'g>(graph: &'g ElementGraph, designator: &str) -> Result<&'g ElementRecord, String> {
    if let Some(found) = graph.elements.iter().find(|e| e.id == designator) {
        return Ok(found);
    }
    let named: Vec<&ElementRecord> = graph.elements.iter().filter(|e| e.name == designator).collect();
    match named.as_slice() {
        [one] => Ok(one),
        [] => Err(format!("no element with id or name '{designator}'")),
        _ => Err(format!("'{designator}' names {} elements: use its id", named.len())),
    }
}

/// Every way the text may write an element: its id, its name, what the
/// text knows it by, and for an element inside another (a port, a state)
/// its owner's name or id before its own.
fn spellings(graph: &ElementGraph, source: &str, element: &ElementRecord) -> Vec<String> {
    let mut all = vec![element.id.clone(), element.name.clone(), written_designator(graph, source, &element.id)];
    if let Some(owner) = element.owner.as_ref().and_then(|uuid| graph.elements.iter().find(|e| &e.uuid == uuid)) {
        if let Some(local) = element.id.strip_prefix(&format!("{}.", owner.id)) {
            all.push(local.to_string());
            all.push(format!("{}.{local}", owner.name));
            all.push(format!("{}.{local}", written_designator(graph, source, &owner.id)));
        }
    }
    all.retain(|s| !s.is_empty());
    all.dedup();
    all
}

/// How `new` is written where `spot` wrote `old`: like it, by name when
/// the old end was written by its name or its local part, by what the text
/// knows it by otherwise.
fn written_like(graph: &ElementGraph, source: &str, spot: &Spot, old: &ElementRecord, new: &ElementRecord) -> Vec<String> {
    let full = written_designator(graph, source, &new.id);
    let mut options = Vec::new();
    let by_name = spot.text == old.name && old.name != old.id;
    let local = old.owner.as_ref().and_then(|uuid| graph.elements.iter().find(|e| &e.uuid == uuid)).and_then(|owner| old.id.strip_prefix(&format!("{}.", owner.id)).map(str::to_string));
    if by_name || local.as_deref() == Some(spot.text.as_str()) {
        let new_local = new.owner.as_ref().and_then(|uuid| graph.elements.iter().find(|e| &e.uuid == uuid)).and_then(|owner| new.id.strip_prefix(&format!("{}.", owner.id)).map(str::to_string));
        options.extend(new_local);
        options.push(new.name.clone());
    }
    options.push(full);
    options.push(new.id.clone());
    options.dedup();
    options.into_iter().map(|text| source_edit::spelled_like(spot, &text)).collect()
}

type Edge = (String, Option<String>, String, String);

fn edges(graph: &ElementGraph) -> BTreeMap<Edge, usize> {
    let mut all = BTreeMap::new();
    for r in &graph.relationships {
        *all.entry((r.kind.to_string(), r.name.clone(), r.source.clone(), r.target.clone())).or_insert(0) += 1;
    }
    all
}

fn element_uuids(graph: &ElementGraph) -> Vec<String> {
    let mut all: Vec<String> = graph.elements.iter().map(|e| e.uuid.clone()).collect();
    all.sort();
    all
}

/// The relationship of `kind` from `from` to `to` (named `name`, when given).
fn find_relationship<'g>(graph: &'g ElementGraph, kind: &str, name: Option<&str>, from: &ElementRecord, to: &ElementRecord) -> Result<&'g RelationshipRecord, String> {
    let found: Vec<&RelationshipRecord> = graph
        .relationships
        .iter()
        .filter(|r| r.kind == kind && r.source == from.uuid && r.target == to.uuid)
        // A trace is named by its type (`satisfies`): it has no name of its own.
        .filter(|r| name.is_none_or(|name| r.name.as_deref() == Some(name) || (r.kind == "Trace" && r.extra.get("traceKind").and_then(|k| k.as_str()) == Some(name))))
        .collect();
    match found.as_slice() {
        [one] => Ok(one),
        [] => Err(format!("no {kind}{} from '{}' to '{}'", name.map(|n| format!(" '{n}'")).unwrap_or_default(), from.id, to.id)),
        _ => Err(format!("{} relationships {kind} go from '{}' to '{}': give the name", found.len(), from.id, to.id)),
    }
}

fn spots_of(graph: &ElementGraph, source: &str, relationship: &RelationshipRecord, from: &ElementRecord, to: &ElementRecord) -> Result<RelationshipSpots, String> {
    let mut designators = Vec::new();
    if let Some(crate::compiler::ast::AttributeValue::String(id)) = relationship.attributes.get("id") {
        designators.push(id.clone());
    }
    designators.extend(relationship.name.clone());
    let from_spellings = spellings(graph, source, from);
    let to_spellings = spellings(graph, source, to);
    let ends = RelationshipEnds { keywords: keywords_of(relationship.kind), designators: &designators, name: relationship.name.as_deref(), verb: relationship.extra.get("traceKind").and_then(|k| k.as_str()), from: &from_spellings, to: &to_spellings };
    source_edit::relationship_spots(source, &ends).map_err(|e| e.to_string())
}

/// Reconnect one relationship: its source to `new_from`, its target to
/// `new_to` (either or both; both swapped is a reversal).
pub fn reconnect(source: &str, kind: &str, name: Option<&str>, from: &str, to: &str, new_from: Option<&str>, new_to: Option<&str>) -> Result<String, String> {
    if !RECONNECTABLE.contains(&kind) {
        return Err(format!("'{kind}' is not a kind of relationship a reconnection changes: use one of {}", RECONNECTABLE.join(", ")));
    }
    let (graph, warnings) = compiled_graph(source, "the model")?;
    let (old_from, old_to) = (resolve(&graph, from)?, resolve(&graph, to)?);
    let relationship = find_relationship(&graph, kind, name, old_from, old_to)?;
    let next_from = new_from.map(|d| resolve(&graph, d)).transpose()?;
    let next_to = new_to.map(|d| resolve(&graph, d)).transpose()?;
    if next_from.is_none() && next_to.is_none() {
        return Err("a reconnection names a new source, a new target or both".to_string());
    }
    check_ends(kind, next_from.unwrap_or(old_from), next_to.unwrap_or(old_to), next_from.is_some(), next_to.is_some())?;
    let spots = spots_of(&graph, source, relationship, old_from, old_to)?;
    let source_options = next_from.map(|e| written_like(&graph, source, &spots.from, old_from, e)).unwrap_or_else(|| vec![String::new()]);
    let target_options = next_to.map(|e| written_like(&graph, source, &spots.to, old_to, e)).unwrap_or_else(|| vec![String::new()]);

    let mut expected = edges(&graph);
    let old: Edge = (relationship.kind.to_string(), relationship.name.clone(), relationship.source.clone(), relationship.target.clone());
    if let Some(count) = expected.get_mut(&old) {
        *count -= 1;
    }
    expected.retain(|_, count| *count > 0);
    let new_edge: Edge = (old.0.clone(), old.1.clone(), next_from.map_or(old.2.clone(), |e| e.uuid.clone()), next_to.map_or(old.3.clone(), |e| e.uuid.clone()));
    *expected.entry(new_edge).or_insert(0) += 1;

    let mut last = String::from("edit refused: the reconnection could not be written");
    for source_text in &source_options {
        for target_text in &target_options {
            let mut replacements = Vec::new();
            if next_from.is_some() {
                replacements.push((spots.from.index, source_text.clone()));
            }
            if next_to.is_some() {
                replacements.push((spots.to.index, target_text.clone()));
            }
            let edited = match source_edit::replace_lexemes(source, &replacements) {
                Ok(edited) => edited,
                Err(e) => {
                    last = e.to_string();
                    continue;
                }
            };
            match check_relationships(&graph, &edited, &expected, None).and_then(|()| no_new_warning(&warnings, &edited)) {
                Ok(()) => return Ok(edited),
                Err(e) => last = e,
            }
        }
    }
    Err(last)
}

/// The edited model keeps every element (or the same ones, `renamed` aside)
/// and holds exactly the `expected` relationships.
fn check_relationships(before: &ElementGraph, edited: &str, expected: &BTreeMap<Edge, usize>, port: Option<(&str, &str)>) -> Result<(), String> {
    let after = graph_of(edited, "edit refused: the edited model")?;
    let mut was = element_uuids(before);
    if let Some((old, new)) = port {
        was = was.into_iter().map(|uuid| if uuid == old { new.to_string() } else { uuid }).collect();
        was.sort();
    }
    if was != element_uuids(&after) {
        return Err("edit refused: it would change the identity of an element".to_string());
    }
    if &edges(&after) != expected {
        return Err("edit refused: the edited model would not connect exactly what was asked".to_string());
    }
    if after.unresolved.len() > before.unresolved.len() {
        return Err("edit refused: it would leave a reference without an end".to_string());
    }
    Ok(())
}

/// The edited model draws no warning the original did not (a trace whose
/// ends break the trace rules, for one).
fn no_new_warning(before: &[String], edited: &str) -> Result<(), String> {
    let (_, after) = compiled_graph(edited, "edit refused: the edited model")?;
    match after.iter().find(|warning| !before.contains(warning)) {
        Some(warning) => Err(format!("edit refused: {warning}")),
        None => Ok(()),
    }
}

/// Rename a relationship (an exchange, a link, a message): its name where
/// the text writes it, its ends untouched.
pub fn rename_relationship(source: &str, designator: &str, new_name: &str) -> Result<String, String> {
    let graph = graph_of(source, "the model")?;
    let written_id = |r: &RelationshipRecord| matches!(r.attributes.get("id"), Some(crate::compiler::ast::AttributeValue::String(id)) if id == designator);
    let named: Vec<&RelationshipRecord> = graph.relationships.iter().filter(|r| r.name.as_deref() == Some(designator) || written_id(r)).collect();
    let relationship = match named.as_slice() {
        [one] => *one,
        [] => return Err(format!("no element or relationship with id or name '{designator}'")),
        _ => return Err(format!("'{designator}' names {} relationships: rename it in the text", named.len())),
    };
    let by_uuid = |uuid: &str| graph.elements.iter().find(|e| e.uuid == uuid).ok_or_else(|| format!("an end of '{designator}' is not an element"));
    let (from, to) = (by_uuid(&relationship.source)?, by_uuid(&relationship.target)?);
    let spots = spots_of(&graph, source, relationship, from, to)?;
    let name = spots.name.ok_or_else(|| format!("'{designator}' writes no name this edit can change"))?;
    let edited = source_edit::replace_lexemes(source, &[(name.index, source_edit::spelled_like(&Spot { quoted: true, ..name.clone() }, new_name))]).map_err(|e| e.to_string())?;
    let mut expected = edges(&graph);
    let old: Edge = (relationship.kind.to_string(), relationship.name.clone(), relationship.source.clone(), relationship.target.clone());
    if let Some(count) = expected.get_mut(&old) {
        *count -= 1;
    }
    expected.retain(|_, count| *count > 0);
    *expected.entry((old.0, Some(new_name.to_string()), old.2, old.3)).or_insert(0) += 1;
    check_relationships(&graph, &edited, &expected, None)?;
    Ok(edited)
}

/// Move a port to another owner: the exchanges it carries follow it.
pub fn move_port(source: &str, port: &str, owner: &str) -> Result<String, String> {
    let graph = graph_of(source, "the model")?;
    let port = resolve(&graph, port)?;
    let owner = resolve(&graph, owner)?;
    if port.owner.as_deref() == Some(owner.uuid.as_str()) {
        return Err(format!("'{}' is already a port of '{}'", port.id, owner.id));
    }
    let local = port.id.rsplit_once('.').map_or(port.name.clone(), |(_, local)| local.to_string());
    let new_id = format!("{}.{local}", owner.id);
    if graph.elements.iter().any(|e| e.id == new_id) {
        return Err(format!("edit refused: {} already has a port '{local}'", owner.id));
    }
    let written_new = format!("{}.{local}", written_designator(&graph, source, &owner.id));
    // Every end the port is written as, rewritten to the port of the new owner.
    let mut replacements = Vec::new();
    let mut touching = Vec::new();
    for relationship in graph.relationships.iter().filter(|r| r.source == port.uuid || r.target == port.uuid) {
        let by_uuid = |uuid: &str| graph.elements.iter().find(|e| e.uuid == uuid).ok_or_else(|| "an end is not an element".to_string());
        let (from, to) = (by_uuid(&relationship.source)?, by_uuid(&relationship.target)?);
        let spots = spots_of(&graph, source, relationship, from, to)?;
        if relationship.source == port.uuid {
            replacements.push((spots.from.index, source_edit::spelled_like(&spots.from, &written_new)));
        }
        if relationship.target == port.uuid {
            replacements.push((spots.to.index, source_edit::spelled_like(&spots.to, &written_new)));
        }
        touching.push(relationship);
    }
    let rewritten = if replacements.is_empty() { source.to_string() } else { source_edit::replace_lexemes(source, &replacements).map_err(|e| e.to_string())? };
    let edited = source_edit::move_element(&rewritten, &written_designator(&graph, source, &port.id), Container::Element(&written_designator(&graph, source, &owner.id))).map_err(|e| e.to_string())?;
    let after = graph_of(&edited, "edit refused: the edited model")?;
    let moved = after.elements.iter().find(|e| e.id == new_id).ok_or_else(|| format!("edit refused: '{new_id}' would not be a port once compiled"))?;
    let follow = |uuid: &String| if uuid == &port.uuid { moved.uuid.clone() } else { uuid.clone() };
    let mut expected = BTreeMap::new();
    for r in &graph.relationships {
        *expected.entry((r.kind.to_string(), r.name.clone(), follow(&r.source), follow(&r.target))).or_insert(0) += 1;
    }
    check_relationships(&graph, &edited, &expected, Some((&port.uuid, &moved.uuid)))?;
    Ok(edited)
}

/// A new id like `id`: `LC-X` -> `LC-X-2`, `LC-X-2` -> `LC-X-3`, never one in `taken`.
pub fn fresh_id(id: &str, taken: &std::collections::HashSet<String>) -> String {
    let (base, mut n, width) = match id.rsplit_once('-') {
        Some((base, digits)) if !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) && !base.is_empty() => (base.to_string(), digits.parse::<u64>().unwrap_or(1) + 1, digits.len()),
        _ => (id.to_string(), 2, 1),
    };
    loop {
        let candidate = format!("{base}-{n:0width$}");
        if !taken.contains(&candidate) {
            return candidate;
        }
        n += 1;
    }
}

/// Copy an element with everything inside it: beside the original, or into
/// a container. Ids are made unique; an element without id gets a new name.
pub fn duplicate(source: &str, element: &str, container: Option<Container>) -> Result<String, String> {
    let graph = graph_of(source, "the model")?;
    let record = resolve(&graph, element)?;
    let mut taken: std::collections::HashSet<String> = graph.elements.iter().flat_map(|e| [e.id.clone(), e.name.clone()]).collect();
    let mut fresh = |id: &str| {
        let next = fresh_id(id, &taken);
        taken.insert(next.clone());
        next
    };
    let names: std::collections::HashSet<&str> = graph.elements.iter().map(|e| e.name.as_str()).collect();
    let new_name = (2..).map(|n| format!("{} {n}", record.name)).find(|name| !names.contains(name.as_str())).expect("an unused name");
    let target = container.map_or(CopyTarget::Beside, CopyTarget::Into);
    let edited = source_edit::duplicate_element(source, &written_designator(&graph, source, &record.id), target, &mut fresh, &new_name).map_err(|e| e.to_string())?;
    let after = graph_of(&edited, "edit refused: the model with the copy")?;
    let before_ids = identities(&graph);
    let after_ids: std::collections::BTreeSet<String> = identities(&after).into_iter().collect();
    if !before_ids.iter().all(|uuid| after_ids.contains(uuid)) {
        return Err("edit refused: the copy would change the identity of another element".to_string());
    }
    if after.elements.len() <= graph.elements.len() {
        return Err(format!("edit refused: the copy of '{element}' would not be a new element"));
    }
    if after.unresolved.len() > graph.unresolved.len() {
        return Err("edit refused: the copy would leave a reference without an end".to_string());
    }
    // The relationships are those of the original, plus the copy's own: one
    // that joins two elements already there (an exchange written inside the
    // copied block between the originals) is a second exchange, refused.
    let old: std::collections::HashSet<&str> = graph.elements.iter().map(|e| e.uuid.as_str()).collect();
    let mut kept = edges(&graph);
    for r in &after.relationships {
        if !(old.contains(r.source.as_str()) && old.contains(r.target.as_str())) {
            continue;
        }
        let key: Edge = (r.kind.to_string(), r.name.clone(), r.source.clone(), r.target.clone());
        match kept.get_mut(&key) {
            Some(count) if *count > 0 => *count -= 1,
            _ => {
                let name = |uuid: &str| graph.elements.iter().find(|e| e.uuid == uuid).map_or(uuid.to_string(), |e| e.id.clone());
                return Err(format!(
                    "edit refused: the copy of '{element}' would add a {} between '{}' and '{}', which are not copied; copy what it joins with it, or write it again for the copy",
                    r.kind,
                    name(&r.source),
                    name(&r.target)
                ));
            }
        }
    }
    if kept.values().any(|count| *count > 0) {
        return Err(format!("edit refused: the copy of '{element}' would lose a relationship of the model"));
    }
    Ok(edited)
}

/// The relationships a copy of several elements takes with it when both
/// their ends are copied: exchanges, links and traces.
const COPIED_WITH_ENDS: &[&str] = &["ComponentExchange", "FunctionalExchange", "OperationalExchange", "PhysicalLink", "PhysicalExchange", "Trace", "Association"];

/// Whether `element` is `ancestor` or inside it.
fn within(graph: &ElementGraph, element: &ElementRecord, ancestor: &str) -> bool {
    let mut at = Some(element);
    while let Some(current) = at {
        if current.uuid == ancestor {
            return true;
        }
        at = current.owner.as_ref().and_then(|uuid| graph.elements.iter().find(|e| &e.uuid == uuid));
    }
    false
}

/// Copy several elements at once, each with what it holds, beside its
/// original or into one container, as [`duplicate`] copies one. An
/// exchange, a link or a trace declared outside them whose two ends are
/// both copied is copied too, joined to the copies; one between an original
/// and anything else is not. The same identity checks as [`duplicate`].
pub fn duplicate_many(source: &str, elements: &[String], container: Option<Container>) -> Result<String, String> {
    if elements.is_empty() {
        return Err("nothing to copy".to_string());
    }
    let graph = graph_of(source, "the model")?;
    let mut picked: Vec<&ElementRecord> = Vec::new();
    for designator in elements {
        let record = resolve(&graph, designator)?;
        if !picked.iter().any(|p| p.uuid == record.uuid) {
            picked.push(record);
        }
    }
    // An element inside another one copied comes with it.
    let tops: Vec<&ElementRecord> = picked.iter().copied().filter(|r| !picked.iter().any(|o| o.uuid != r.uuid && within(&graph, r, &o.uuid))).collect();
    let old: std::collections::HashSet<&str> = graph.elements.iter().map(|e| e.uuid.as_str()).collect();
    let mut taken: std::collections::HashSet<String> = graph.elements.iter().flat_map(|e| [e.id.clone(), e.name.clone()]).collect();
    let mut renamed: BTreeMap<String, String> = BTreeMap::new();
    let mut names: std::collections::HashSet<String> = graph.elements.iter().map(|e| e.name.clone()).collect();
    let mut new_names: Vec<String> = Vec::new();
    let mut text = source.to_string();
    for record in &tops {
        let new_name = (2..).map(|n| format!("{} {n}", record.name)).find(|name| !names.contains(name)).expect("an unused name");
        names.insert(new_name.clone());
        let mut fresh = |id: &str| {
            let next = fresh_id(id, &taken);
            taken.insert(next.clone());
            renamed.insert(id.to_string(), next.clone());
            next
        };
        let target = container.map_or(CopyTarget::Beside, CopyTarget::Into);
        text = source_edit::duplicate_element(&text, &written_designator(&graph, source, &record.id), target, &mut fresh, &new_name).map_err(|e| e.to_string())?;
        new_names.push(new_name);
    }
    let copied = graph_of(&text, "edit refused: the model with the copies")?;
    // Each copied element and its copy.
    let mut copies: BTreeMap<String, String> = BTreeMap::new();
    let fresh_in = |kind: &str, owner: Option<&str>, matches: &dyn Fn(&ElementRecord) -> bool| -> Option<String> {
        let found: Vec<&ElementRecord> = copied.elements.iter().filter(|e| !old.contains(e.uuid.as_str()) && e.kind == kind && owner.is_none_or(|o| e.owner.as_deref() == Some(o)) && matches(e)).collect();
        (found.len() == 1).then(|| found[0].uuid.clone())
    };
    for (record, new_name) in tops.iter().zip(&new_names) {
        if let Some(copy) = fresh_in(record.kind, None, &|e: &ElementRecord| &e.name == new_name) {
            copies.insert(record.uuid.clone(), copy);
        }
    }
    // Inside a copy, owner first: the order of the graph lists owners before what they hold.
    loop {
        let before = copies.len();
        for element in &graph.elements {
            if copies.contains_key(&element.uuid) {
                continue;
            }
            let Some(owner_copy) = element.owner.as_ref().and_then(|owner| copies.get(owner)).cloned() else { continue };
            let wanted_id = renamed.get(&element.id).cloned();
            let copy = fresh_in(element.kind, Some(&owner_copy), &|e: &ElementRecord| match &wanted_id {
                Some(id) => &e.id == id,
                None => e.name == element.name,
            });
            if let Some(copy) = copy {
                copies.insert(element.uuid.clone(), copy);
            }
        }
        if copies.len() == before {
            break;
        }
    }
    // The copied blocks join nothing that was already there.
    let between_old = |g: &ElementGraph| -> BTreeMap<Edge, usize> {
        let mut all = BTreeMap::new();
        for r in g.relationships.iter().filter(|r| old.contains(r.source.as_str()) && old.contains(r.target.as_str())) {
            *all.entry((r.kind.to_string(), r.name.clone(), r.source.clone(), r.target.clone())).or_insert(0) += 1;
        }
        all
    };
    let kept = edges(&graph);
    if between_old(&copied) != kept {
        return Err("edit refused: a copied block declares a relationship between elements that are not copied; copy what it joins with it, or write it again for the copy".to_string());
    }
    // The relationships declared outside, both ends copied: copied, joined to the copies.
    let mut relationship_names: std::collections::HashSet<String> = graph.relationships.iter().filter_map(|r| r.name.clone()).chain(names.iter().cloned()).collect();
    let mut current = copied;
    for relationship in graph.relationships.iter().filter(|r| COPIED_WITH_ENDS.contains(&r.kind)) {
        let (Some(from_copy), Some(to_copy)) = (copies.get(&relationship.source), copies.get(&relationship.target)) else { continue };
        let count = |g: &ElementGraph| g.relationships.iter().filter(|r| r.kind == relationship.kind && &r.source == from_copy && &r.target == to_copy).count();
        if count(&current) > 0 {
            continue; // declared inside a copied block, it came with it
        }
        let element = |g: &ElementGraph, uuid: &str| g.elements.iter().find(|e| e.uuid == uuid).cloned();
        let (Some(from), Some(to), Some(new_from), Some(new_to)) = (element(&current, &relationship.source), element(&current, &relationship.target), element(&current, from_copy), element(&current, to_copy)) else { continue };
        let Some(written) = current.relationships.iter().find(|r| r.uuid == relationship.uuid).cloned() else { continue };
        let spots = spots_of(&current, &text, &written, &from, &to)?;
        let mut renaming = Vec::new();
        if let (Some(spot), Some(name)) = (&spots.name, &relationship.name) {
            let new_name = (2..).map(|n| format!("{name} {n}")).find(|candidate| !relationship_names.contains(candidate)).expect("an unused name");
            relationship_names.insert(new_name.clone());
            renaming.push((spot.index, source_edit::spelled_like(&Spot { quoted: true, ..spot.clone() }, &new_name)));
        }
        let wanted = count(&current) + 1;
        let mut done = None;
        'spellings: for from_text in written_like(&current, &text, &spots.from, &from, &new_from) {
            for to_text in written_like(&current, &text, &spots.to, &to, &new_to) {
                let mut changes = vec![(spots.from.index, from_text.clone()), (spots.to.index, to_text)];
                changes.extend(renaming.iter().cloned());
                let mut fresh = |id: &str| {
                    let next = fresh_id(id, &taken);
                    taken.insert(next.clone());
                    next
                };
                let Ok(edited) = source_edit::duplicate_relationship(&text, spots.from.index, &changes, &mut fresh) else { continue };
                let Ok(after) = graph_of(&edited, "the copy") else { continue };
                if count(&after) == wanted && between_old(&after) == kept {
                    done = Some((edited, after));
                    break 'spellings;
                }
            }
        }
        let (edited, after) = done.ok_or_else(|| format!("edit refused: the {} between '{}' and '{}' could not be copied with them", relationship.kind, from.id, to.id))?;
        text = edited;
        current = after;
    }
    let before_ids = identities(&graph);
    let after_ids: std::collections::BTreeSet<String> = identities(&current).into_iter().collect();
    if !before_ids.iter().all(|uuid| after_ids.contains(uuid)) {
        return Err("edit refused: the copy would change the identity of another element".to_string());
    }
    if current.elements.len() < graph.elements.len() + tops.len() {
        return Err("edit refused: the copies would not all be new elements".to_string());
    }
    if current.unresolved.len() > graph.unresolved.len() {
        return Err("edit refused: the copy would leave a reference without an end".to_string());
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODEL: &str = "model T { name: \"t\" }\nlogical_architecture \"L\" {\n    component \"A\" {\n        id: \"LC-A\"\n        port out \"p\"\n    }\n    component \"B\" {\n        id: \"LC-B\"\n        port in \"q\"\n    }\n    component \"C\" {\n        id: \"LC-C\"\n        port in \"r\"\n    }\n    component_exchange \"Flow\" {\n        from_port: \"LC-A.p\"\n        to_port: \"LC-B.q\"\n    }\n}\nstate_machine \"M\" {\n    initial: \"Off\"\n    state Off {}\n    state On {}\n    state Idle {}\n    transition Off -> On { guard: \"go\" }\n}\nscenario \"S\" {\n    participants: [\"LC-A\", \"LC-B\", \"LC-C\"]\n    message \"LC-A\" -> \"LC-B\" \"ping\"\n}\nrequirements {\n    requirement \"REQ-1\" { description: \"one\" }\n    requirement \"REQ-2\" { description: \"two\" }\n}\ntrace \"LC-A\" satisfies \"REQ-1\" { rationale: \"kept\" }\n";

    #[test]
    fn an_exchange_a_transition_a_message_and_a_trace_are_reconnected() {
        let flow = reconnect(MODEL, "ComponentExchange", Some("Flow"), "LC-A.p", "LC-B.q", None, Some("LC-C.r")).unwrap();
        assert!(flow.contains("to_port: \"LC-C.r\""), "{flow}");
        let reversed = reconnect(MODEL, "Transition", None, "M.Off", "M.On", Some("M.On"), Some("M.Off")).unwrap();
        assert!(reversed.contains("transition On -> Off { guard: \"go\" }"), "{reversed}");
        let transition = reconnect(MODEL, "Transition", None, "M.Off", "M.On", None, Some("M.Idle")).unwrap();
        assert!(transition.contains("transition Off -> Idle { guard: \"go\" }"), "{transition}");
        let message = reconnect(MODEL, "Message", Some("ping"), "LC-A", "LC-B", None, Some("LC-C")).unwrap();
        assert!(message.contains("message \"LC-A\" -> \"LC-C\" \"ping\""), "{message}");
        let trace = reconnect(MODEL, "Trace", None, "LC-A", "REQ-1", None, Some("REQ-2")).unwrap();
        assert!(trace.contains("trace \"LC-A\" satisfies \"REQ-2\" { rationale: \"kept\" }"), "{trace}");
        let refused = reconnect(MODEL, "ComponentExchange", Some("Flow"), "LC-A.p", "LC-B.q", None, Some("Nobody"));
        assert!(refused.is_err());
    }

    #[test]
    fn a_trace_is_told_apart_by_its_type() {
        let model = "model T { name: \"t\" }\nlogical_architecture \"L\" {\n    component \"A\" { id: \"LC-A\" }\n    component \"B\" { id: \"LC-B\" }\n    component \"C\" { id: \"LC-C\" }\n}\ntrace \"LC-A\" allocates \"LC-B\"\ntrace \"LC-A\" implements \"LC-B\" { rationale: \"kept\" }\n";
        let both = reconnect(model, "Trace", None, "LC-A", "LC-B", None, Some("LC-C")).unwrap_err();
        assert!(both.contains("give the name"), "{both}");
        let moved = reconnect(model, "Trace", Some("implements"), "LC-A", "LC-B", None, Some("LC-C")).unwrap();
        assert!(moved.contains("trace \"LC-A\" allocates \"LC-B\"\n"), "{moved}");
        assert!(moved.contains("trace \"LC-A\" implements \"LC-C\" { rationale: \"kept\" }"), "{moved}");
        let one_line = reconnect(model, "Trace", Some("allocates"), "LC-A", "LC-B", Some("LC-C"), None).unwrap();
        assert!(one_line.contains("trace \"LC-C\" allocates \"LC-B\"\n"), "{one_line}");
    }

    #[test]
    fn a_reconnection_follows_the_rules_of_creating_the_relationship() {
        let to_itself = reconnect(MODEL, "ComponentExchange", Some("Flow"), "LC-A.p", "LC-B.q", None, Some("LC-A.p")).unwrap_err();
        assert!(to_itself.contains("to itself"), "{to_itself}");
        let wrong_way = reconnect(MODEL, "ComponentExchange", Some("Flow"), "LC-A.p", "LC-B.q", Some("LC-B.q"), Some("LC-A.p")).unwrap_err();
        assert!(wrong_way.contains("in port"), "{wrong_way}");
        let model = "model T { name: \"t\" }\nlogical_architecture \"L\" {\n    component \"A\" { id: \"LC-A\" }\n}\nrequirements {\n    requirement \"REQ-1\" { description: \"d\" }\n}\ntrace \"LC-A\" satisfies \"REQ-1\"\n";
        let reversed = reconnect(model, "Trace", None, "LC-A", "REQ-1", Some("REQ-1"), Some("LC-A")).unwrap_err();
        assert!(reversed.contains("edit refused") && reversed.contains("requirement"), "{reversed}");
        let unknown = reconnect(MODEL, "component_exchange", Some("Flow"), "LC-A.p", "LC-B.q", None, Some("LC-C.r")).unwrap_err();
        assert!(unknown.contains("ComponentExchange, FunctionalExchange"), "{unknown}");
    }

    #[test]
    fn a_relationship_is_renamed_and_a_port_moves_with_its_exchanges() {
        let renamed = rename_relationship(MODEL, "Flow", "Main flow").unwrap();
        assert!(renamed.contains("component_exchange \"Main flow\" {"), "{renamed}");
        let relabelled = rename_relationship(MODEL, "ping", "hello").unwrap();
        assert!(relabelled.contains("message \"LC-A\" -> \"LC-B\" \"hello\""), "{relabelled}");
        let inline = "model T { name: \"t\" }\nlogical_architecture \"L\" {\n    component \"A\" { id: \"LC-A\" port out \"p\" }\n    component \"B\" { id: \"LC-B\" port in \"q\" }\n    component \"C\" { id: \"LC-C\" }\n    component_exchange \"Flow\" { from_port: \"LC-A.p\" to_port: \"LC-B.q\" }\n}\n";
        let moved_inline = move_port(inline, "LC-B.q", "LC-C").unwrap();
        assert!(moved_inline.contains("to_port: \"LC-C.q\"") && moved_inline.contains("component \"B\" { id: \"LC-B\" }"), "{moved_inline}");
        assert!(moved_inline.contains("component \"C\" { id: \"LC-C\" port in \"q\" }"), "a one-line block stays on its line: {moved_inline}");
        let twice = move_port(&moved_inline.replace("component \"A\" { id: \"LC-A\" port out \"p\" }", "component \"A\" { id: \"LC-A\" port out \"q\" }").replace("LC-A.p", "LC-A.q"), "LC-A.q", "LC-C").unwrap_err();
        assert!(twice.contains("LC-C already has a port 'q'"), "{twice}");
        let expanded = duplicate(inline, "LC-A", Some(Container::Element("LC-C"))).unwrap();
        assert!(expanded.contains("component \"C\" { id: \"LC-C\" component \"A 2\" { id: \"LC-A-2\" port out \"p\" } }"), "{expanded}");
        let multi = "model T { name: \"t\" }\nlogical_architecture \"L\" {\n    component \"A\" {\n        id: \"LC-A\"\n        function \"F\" {\n            id: \"LF-F\"\n        }\n    }\n    component \"C\" { id: \"LC-C\" description: \"d\" port in \"r\" }\n}\n";
        let opened = duplicate(multi, "LF-F", Some(Container::Element("LC-C"))).unwrap();
        assert!(opened.contains("component \"C\" {\n        id: \"LC-C\"\n        description: \"d\"\n        port in \"r\"\n        function \"F 2\" {\n            id: \"LF-F-2\"\n        }\n    }"), "{opened}");
        let moved = move_port(MODEL, "LC-B.q", "LC-C").unwrap();
        assert!(moved.contains("to_port: \"LC-C.q\""), "{moved}");
        let c = moved.find("component \"C\"").unwrap();
        assert!(moved.find("port in \"q\"").unwrap() > c, "{moved}");
    }

    #[test]
    fn a_copy_takes_unique_ids() {
        let copied = duplicate(MODEL, "LC-A", None).unwrap();
        assert!(copied.contains("id: \"LC-A-2\""), "{copied}");
        let again = duplicate(&copied, "LC-A", None).unwrap();
        assert!(again.contains("id: \"LC-A-3\""), "{again}");
        let commented = "model T { name: \"t\" }\nlogical_architecture \"L\" {\n    component \"A\" { id: \"LC-A\" port out \"p\" }   // keep this comment\n}\n";
        let copy = duplicate(commented, "LC-A", None).unwrap();
        assert!(copy.contains("component \"A\" { id: \"LC-A\" port out \"p\" }   // keep this comment\n    component \"A 2\" { id: \"LC-A-2\" port out \"p\" }\n"), "{copy}");
        // A scenario's messages join its participants, which are not copied: a second message each.
        let second = duplicate(MODEL, "S", None).unwrap_err();
        assert!(second.contains("would add a Message between 'LC-A' and 'LC-B'"), "{second}");
        // A machine's transitions join its own states: copied with them, rewired to the copy.
        let machine = duplicate(MODEL, "M", None).unwrap();
        assert!(machine.contains("state_machine \"M 2\"") && machine.matches("transition Off -> On").count() == 2, "{machine}");
        let named = duplicate(MODEL, "LC-A", None).unwrap();
        assert!(named.contains("component \"A 2\" {\n        id: \"LC-A-2\""), "the copy has a name of its own: {named}");
        let into = duplicate(MODEL, "Off", Some(Container::Element("M"))).unwrap();
        assert!(into.contains("state \"Off 2\" {}"), "{into}");
        let taken: std::collections::HashSet<String> = ["LC-X-2".to_string()].into_iter().collect();
        assert_eq!(fresh_id("LC-X", &taken), "LC-X-3");
        assert_eq!(fresh_id("REQ-007", &Default::default()), "REQ-008");
    }
    #[test]
    fn several_copies_take_the_exchanges_between_them() {
        let copied = duplicate_many(MODEL, &["LC-A".to_string(), "LC-B".to_string()], None).unwrap();
        assert!(copied.contains("id: \"LC-A-2\"") && copied.contains("id: \"LC-B-2\""), "{copied}");
        assert!(copied.contains("component_exchange \"Flow 2\" {\n        from_port: \"LC-A-2.p\"\n        to_port: \"LC-B-2.q\"\n    }"), "{copied}");
        assert_eq!(copied.matches("component_exchange").count(), 2, "{copied}");
        // One end only copied: the exchange stays with the originals.
        let alone = duplicate_many(MODEL, &["LC-A".to_string(), "LC-C".to_string()], None).unwrap();
        assert_eq!(alone.matches("component_exchange").count(), 1, "{alone}");
        // A trace between two copied elements is copied too.
        let traced = duplicate_many(MODEL, &["LC-A".to_string(), "REQ-1".to_string()], None).unwrap();
        assert!(traced.contains("trace \"LC-A-2\" satisfies \"REQ-1 2\"") || traced.contains("trace \"LC-A-2\" satisfies \"REQ-2\""), "{traced}");
        // Into a container, the exchange follows the copies there.
        let into = duplicate_many(MODEL, &["LC-A".to_string(), "LC-B".to_string()], Some(Container::Element("LC-C"))).unwrap();
        assert!(into.contains("from_port: \"LC-A-2.p\"") && into.contains("to_port: \"LC-B-2.q\""), "{into}");
        // A scenario still refuses: its messages join participants that are not copied.
        assert!(duplicate_many(MODEL, &["S".to_string()], None).is_err());
    }

}
