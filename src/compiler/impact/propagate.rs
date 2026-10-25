//! Propagation: a breadth-first walk of the impact graph from the seeds,
//! along the kinds of link asked for, to a depth. Every element reached
//! keeps the shortest path that explains it, its distance, a category and
//! a severity with its reason.
//!
//! Three rules keep the impact set to what a change carries:
//! - a link of one kind is never walked back the other way right after
//!   (an exchange excepted, which is a connector): a component reaches the
//!   requirement it satisfies, but not, through it, every other component
//!   satisfying it; a function reaches its chain, not the chain's other
//!   functions; an element reaches the diagram showing it, not what else
//!   the diagram shows;
//! - what is reached *upstream* (what the changed element depends on: the
//!   requirement it satisfies, the function it realizes, the node it is
//!   deployed on) is to re-check, not changed: from it only its evidence
//!   follows (verifications, safety analyses, chains and budgets,
//!   constraints, views), not its structure (owned elements, ports,
//!   exchanges, other allocations);
//! - the element at the other end of an exchange, or of a scenario
//!   message, is reached and the walk stops there: an interface change
//!   reaches its consumer, not the consumer's other interfaces.

use super::graph::{Edge, ImpactGraph, LinkKind};
use super::{Category, Direction, ImpactSeverity, Impacted, Options, PathStep};
use crate::compiler::tables::Context;
use std::collections::VecDeque;

struct Visit {
    distance: usize,
    /// The node it was reached from, and the way of the link.
    from: Option<(usize, usize)>,
    /// Whether the path to it went upstream once.
    upstream: bool,
}

/// The links that still follow, downstream, from what was reached upstream.
const EVIDENCE: [LinkKind; 5] = [LinkKind::Verifies, LinkKind::Safety, LinkKind::Chain, LinkKind::Constraint, LinkKind::View];

type Reached = (usize, Vec<PathStep>, usize, Option<LinkKind>, bool);

/// The nodes reached from the seeds, in the order found (distance, then link order).
pub fn walk(graph: &ImpactGraph, seeds: &[usize], options: &Options) -> Vec<Reached> {
    let mut visits: Vec<Option<Visit>> = (0..graph.nodes.len()).map(|_| None).collect();
    let mut queue = VecDeque::new();
    for &seed in seeds {
        if visits[seed].is_none() {
            visits[seed] = Some(Visit { distance: 0, from: None, upstream: false });
            queue.push_back(seed);
        }
    }
    let mut order = Vec::new();
    while let Some(node) = queue.pop_front() {
        let visit = visits[node].as_ref().expect("visited");
        let distance = visit.distance;
        if distance > 0 {
            order.push(node);
        }
        if distance >= options.depth {
            continue;
        }
        let arrived: Option<&Edge> = visit.from.map(|(from, edge)| &graph.edges[from][edge]);
        let went_up = visit.upstream;
        // The far end of an exchange or a message: reached, not crossed.
        if arrived.is_some_and(|e| (e.kind == LinkKind::Exchange && !e.downstream) || e.kind == LinkKind::Behavior) {
            continue;
        }
        for (index, edge) in graph.edges[node].iter().enumerate() {
            if !options.kinds.contains(&edge.kind) {
                continue;
            }
            match options.direction {
                Direction::Downstream if !edge.downstream => continue,
                Direction::Upstream if edge.downstream => continue,
                _ => {}
            }
            if let Some(previous) = arrived {
                if previous.kind == edge.kind && previous.downstream != edge.downstream && edge.kind != LinkKind::Exchange {
                    continue;
                }
            }
            if went_up && edge.downstream && !EVIDENCE.contains(&edge.kind) {
                continue;
            }
            // A view is an end: nothing is reached through a diagram or a table.
            if matches!(graph.node(node).kind.as_str(), "Diagram" | "Table") && distance > 0 {
                continue;
            }
            if visits[edge.to].is_some() {
                continue;
            }
            visits[edge.to] = Some(Visit { distance: distance + 1, from: Some((node, index)), upstream: went_up || !edge.downstream });
            queue.push_back(edge.to);
        }
    }
    order
        .into_iter()
        .map(|node| {
            let mut steps = Vec::new();
            let mut current = node;
            let mut last: Option<&Edge> = None;
            while let Some((from, edge)) = visits[current].as_ref().and_then(|v| v.from) {
                let link = &graph.edges[from][edge];
                if last.is_none() {
                    last = Some(link);
                }
                steps.push(PathStep { from: graph.node(from).id.clone(), relation: link.label.clone(), kind: link.kind.as_str().to_string(), to: graph.node(current).id.clone() });
                current = from;
            }
            steps.reverse();
            let distance = visits[node].as_ref().map(|v| v.distance).unwrap_or(0);
            (node, steps, distance, last.map(|e| e.kind), last.is_some_and(|e| e.downstream))
        })
        .collect()
}

/// The category of an element reached through `via` (going downstream or not).
pub fn category(graph: &ImpactGraph, node: usize, via: Option<LinkKind>, downstream: bool) -> Category {
    let n = graph.node(node);
    match n.kind.as_str() {
        "Requirement" => Category::Requirement,
        "TestCase" => Category::Test,
        "Hazard" | "FmeaEntry" | "Threat" | "PrimaryAsset" => Category::Safety,
        "Constraint" | "Budget" => Category::Budget,
        "FunctionalChain" if n.budget_ms.is_some() => Category::Budget,
        "Diagram" | "Table" => Category::View,
        "FunctionalExchange" | "ComponentExchange" | "OperationalExchange" | "PhysicalExchange" | "PhysicalLink" | "LogicalInterface" | "ExchangeItem" => Category::Interface,
        kind if kind.ends_with("Port") => Category::Interface,
        _ if via == Some(LinkKind::Typing) && downstream => Category::TypeUser,
        _ => Category::Architecture,
    }
}

/// The severity of an impacted element before any what-if: the gate's
/// blockers, the integrity levels, the kind of what it is.
pub fn severity(context: &Context, graph: &ImpactGraph, node: usize, category: Category) -> (ImpactSeverity, String) {
    let n = graph.node(node);
    let named = blockers_naming(context, &n.id, &n.name);
    if let Some(first) = named.first() {
        return (ImpactSeverity::Blocker, format!("gate blocker: {first}"));
    }
    let integrity = n.integrity.clone();
    match category {
        Category::Requirement | Category::Safety | Category::Budget if integrity.is_some() => (ImpactSeverity::Blocker, format!("{} {}: safety re-assessment", integrity.unwrap_or_default(), category.noun())),
        Category::Requirement => (ImpactSeverity::Major, "requirement to re-check".to_string()),
        Category::Safety => (ImpactSeverity::Major, "safety analysis to revisit".to_string()),
        Category::Budget => (ImpactSeverity::Major, "budget or constraint to re-evaluate".to_string()),
        Category::Interface => (ImpactSeverity::Major, "interface change: ICD to update".to_string()),
        Category::Test => (ImpactSeverity::Major, "verification to redo".to_string()),
        Category::TypeUser => (ImpactSeverity::Major, "inherits from a changed type".to_string()),
        Category::Architecture if integrity.is_some() => (ImpactSeverity::Major, format!("{} element: safety re-assessment", integrity.unwrap_or_default())),
        Category::Architecture => (ImpactSeverity::Minor, "to review".to_string()),
        Category::View => (ImpactSeverity::Minor, "view to refresh".to_string()),
    }
}

/// The gate blockers whose message names an element (`'REQ-1'`).
pub fn blockers_naming(context: &Context, id: &str, name: &str) -> Vec<String> {
    let quoted = [format!("'{id}'"), format!("'{name}'")];
    context.gate_blockers.iter().filter(|(_, m)| quoted.iter().any(|q| m.contains(q.as_str()))).map(|(_, m)| m.clone()).collect()
}

/// Every impacted element of a walk, classified.
pub fn impacted(context: &Context, graph: &ImpactGraph, seeds: &[usize], options: &Options) -> Vec<Impacted> {
    walk(graph, seeds, options)
        .into_iter()
        .map(|(node, path, distance, via, downstream)| {
            let category = category(graph, node, via, downstream);
            let (severity, reason) = severity(context, graph, node, category);
            let n = graph.node(node);
            Impacted {
                id: n.id.clone(),
                uuid: n.uuid.clone(),
                name: n.name.clone(),
                kind: n.kind.clone(),
                layer: n.layer.clone(),
                category,
                severity,
                reason,
                distance,
                why: super::path_text(&path),
                path,
                integrity: n.integrity.clone(),
                removed: false,
            }
        })
        .collect()
}
