//! `cesam_grid`: the model seen through the CESAM framework — one row per
//! vision (operational, functional, constructional), one column per aspect
//! (requirements, states, structure, interactions, scenarios, interfaces).
//! A cell lists the elements and exchanges of the model in that view, counted
//! per Arcadia level (CESAM's abstraction levels: how deep the view goes); its
//! status says how mature the view is: `gap` when empty, `warn` when the
//! gate flags one of its elements, else `ok`. A scope keeps one level, or one
//! element and what it holds: the grid of a subsystem taken as the system of
//! interest.

use super::context::Context;
use super::{Cell, Column, Link, Row, TableSpec};
use crate::compiler::cesam::{self, Aspect, Classifier, Placement, Vision, ASPECTS, VISIONS};
use std::collections::{BTreeMap, HashMap};

/// Names shown in a cell's text before `…`.
const NAMES_SHOWN: usize = 6;

fn column(key: &str) -> Column {
    let label = match key {
        "vision" => "Vision".to_string(),
        "question" => "Question".to_string(),
        other => ASPECTS.iter().find(|a| a.key() == other).map(|a| a.label().to_string()).unwrap_or_else(|| super::label_of(other)),
    };
    let ty = if matches!(key, "vision" | "question") { "text" } else { "links" };
    Column { key: key.to_string(), label, ty: ty.to_string() }
}

fn default_columns() -> Vec<String> {
    let mut keys = vec!["vision".to_string()];
    keys.extend(ASPECTS.iter().map(|a| a.key().to_string()));
    keys
}

/// Every element and exchange of the model in scope, by cell, with the
/// element index (for the gate's blockers) when it is an element.
pub fn placements(context: &Context, scope: Option<&str>) -> HashMap<Placement, Vec<(Link, Option<usize>)>> {
    let classifier = Classifier::new(&context.graph, &context.result.ast);
    let mut cells: HashMap<Placement, Vec<(Link, Option<usize>)>> = HashMap::new();
    for (index, element) in context.graph.elements.iter().enumerate() {
        if !context.in_scope(index, scope) {
            continue;
        }
        if let Some(placement) = classifier.element(element) {
            cells.entry(placement).or_default().push((context.link(index), Some(index)));
        }
    }
    for relationship in &context.graph.relationships {
        let Some(placement) = classifier.relationship(relationship) else { continue };
        // An exchange belongs to the scope when either end does: the
        // interfaces of a system of interest are what crosses its boundary.
        let ends = [context.index_of_uuid(&relationship.source), context.index_of_uuid(&relationship.target)];
        if scope.is_some() && !ends.iter().flatten().any(|&i| context.in_scope(i, scope)) {
            continue;
        }
        let end = |uuid: &str| context.index_of_uuid(uuid).map(|i| context.link(i).name).unwrap_or_default();
        let name = relationship.name.clone().filter(|n| !n.is_empty()).unwrap_or_else(|| format!("{} -> {}", end(&relationship.source), end(&relationship.target)));
        let link = Link { id: name.clone(), uuid: relationship.uuid.clone(), name, kind: relationship.kind.to_string(), verdict: None };
        cells.entry(placement).or_default().push((link, None));
    }
    cells
}

/// How many entries of a cell sit at each Arcadia level, in level order
/// (` (OA 2, SA 5)`): an exchange counts at its source's level.
fn levels_of(context: &Context, entries: &[(Link, Option<usize>)]) -> String {
    use crate::compiler::metamodel::Layer;
    let mut counts: BTreeMap<usize, (&'static str, usize)> = BTreeMap::new();
    for (link, index) in entries {
        let index = index.or_else(|| {
            let relationship = context.graph.relationships.iter().find(|r| r.uuid == link.uuid)?;
            context.index_of_uuid(&relationship.source)
        });
        let Some(index) = index else { continue };
        let layer = context.layer_of(index);
        let short = match layer {
            Layer::Operational => "OA",
            Layer::System => "SA",
            Layer::Logical => "LA",
            Layer::Physical => "PA",
            Layer::Epbs => "EPBS",
            Layer::Data => "Data",
            Layer::Transverse => continue,
        };
        counts.entry(super::context::layer_rank(layer)).or_insert((short, 0)).1 += 1;
    }
    if counts.is_empty() {
        return String::new();
    }
    format!(" ({})", counts.values().map(|(short, n)| format!("{short} {n}")).collect::<Vec<_>>().join(", "))
}

fn cell_of(context: &Context, vision: Vision, aspect: Aspect, entries: &[(Link, Option<usize>)], issues: &mut Vec<String>) -> Cell {
    let view = cesam::view(vision, aspect);
    if entries.is_empty() {
        issues.push(format!("{}: nothing modelled yet ({})", view.name, view.capella));
        return Cell::text("—").with_status("gap");
    }
    let flagged: Vec<String> = entries.iter().filter_map(|(link, index)| index.filter(|&i| !context.blockers_of(i).is_empty()).map(|_| link.name.clone())).collect();
    let mut names: Vec<String> = entries.iter().take(NAMES_SHOWN).map(|(l, _)| l.name.clone()).collect();
    if entries.len() > NAMES_SHOWN {
        names.push("…".to_string());
    }
    let links: Vec<Link> = entries.iter().map(|(l, _)| l.clone()).collect();
    let mut cell = Cell::links(links);
    cell.text = format!("{}{} · {}", entries.len(), levels_of(context, entries), names.join(", "));
    if flagged.is_empty() {
        cell.with_status("ok")
    } else {
        issues.push(format!("{}: gate blockers on {}", view.name, flagged.join(", ")));
        cell.with_status("warn")
    }
}

pub fn compute(context: &Context, spec: &TableSpec) -> Result<(Vec<Column>, Vec<Column>, Vec<Row>), String> {
    context.check_scope(spec.scope.as_deref())?;
    let mut available_keys = default_columns();
    available_keys.push("question".to_string());
    let available: Vec<Column> = available_keys.iter().map(|k| column(k)).collect();
    let keys = if spec.columns.is_empty() { default_columns() } else { spec.columns.clone() };
    for key in &keys {
        if !available_keys.contains(key) {
            return Err(format!("cesam_grid: unknown column '{key}' (one of {})", available_keys.join(", ")));
        }
    }
    let cells = placements(context, spec.scope.as_deref());
    let empty = Vec::new();
    let mut rows = Vec::new();
    for vision in VISIONS {
        let mut row_cells: BTreeMap<String, Cell> = BTreeMap::new();
        let mut issues = Vec::new();
        for key in &keys {
            let cell = match key.as_str() {
                "vision" => Cell::text(vision.label()),
                "question" => Cell::text(vision.question()),
                aspect_key => {
                    let Some(aspect) = ASPECTS.iter().copied().find(|a| a.key() == aspect_key) else { continue };
                    let entries = cells.get(&Placement { vision, aspect }).unwrap_or(&empty);
                    cell_of(context, vision, aspect, entries, &mut issues)
                }
            };
            row_cells.insert(key.clone(), cell);
        }
        let worst = row_cells.values().filter_map(|c| c.status.as_deref()).fold("ok", |worst, s| match (worst, s) {
            ("gap", _) | (_, "gap") => "gap",
            ("warn", _) | (_, "warn") => "warn",
            _ => "ok",
        });
        rows.push(Row {
            id: vision.key().to_string(),
            uuid: String::new(),
            kind: "CesamVision".to_string(),
            name: vision.label().to_string(),
            status: worst.to_string(),
            issues,
            cells: row_cells,
            parent: None,
            depth: 0,
        });
    }
    let columns = keys.iter().map(|k| column(k)).collect();
    Ok((columns, available, rows))
}
