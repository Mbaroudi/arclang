//! Change-impact analysis, computed by the compiler.
//!
//! The seeds are elements, or a proposed change: `arclang edit` requests
//! (the workbench's JSON), or what changed in the model since a git
//! revision. From them the change is propagated through the model along
//! typed links (realizations and implementations between layers,
//! allocations and deployments, exchanges and ports, functional chains,
//! satisfactions, verifications, hazards, ownership, types, constraints,
//! the declared diagrams and tables), to a depth, never twice through an
//! element. Every element reached says why, as the shortest path from a
//! seed (`LC-BATT --satisfies--> REQ-RANGE --verified by--> TC-RANGE`),
//! with its distance, a category and a severity.
//!
//! A what-if goes further: the edits are applied to a copy of the text,
//! the copy is compiled, and the two models are compared: the production
//! gate before and after, the constraints that flip, the latency budgets,
//! the requirement coverage, the verdicts left stale, the elements added,
//! removed or changed. An edit that does not compile is refused, with the
//! compiler's reason; nothing is written.
//!
//! The same model and seeds always give the same report.

pub mod graph;
mod propagate;
pub mod report;
pub mod since;
pub mod whatif;

#[cfg(test)]
mod tests;

use crate::compiler::tables::Context;
use crate::compiler::CompilationResult;
use graph::{ImpactGraph, LinkKind};
use serde::Serialize;
use std::collections::BTreeMap;

pub use whatif::{cascade_delete, what_if, WhatIf};

/// Which way the change travels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// Both ways: what depends on the seed, and what the seed depends on.
    Both,
    /// Only to what depends on the seed (a requirement to what satisfies it).
    Downstream,
    /// Only to what the seed depends on (a component to its requirements).
    Upstream,
}

impl Direction {
    pub fn parse(text: &str) -> Option<Direction> {
        match text.trim().to_ascii_lowercase().as_str() {
            "both" | "all" => Some(Direction::Both),
            "downstream" | "down" | "dependents" => Some(Direction::Downstream),
            "upstream" | "up" | "dependencies" => Some(Direction::Upstream),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Options {
    pub depth: usize,
    pub kinds: Vec<LinkKind>,
    pub direction: Direction,
    pub gate_standard: String,
}

pub const DEFAULT_DEPTH: usize = 4;
pub const MAX_DEPTH: usize = 12;

impl Default for Options {
    fn default() -> Self {
        Options { depth: DEFAULT_DEPTH, kinds: LinkKind::ALL.to_vec(), direction: Direction::Both, gate_standard: "ISO26262".to_string() }
    }
}

impl Options {
    /// The kinds written as `satisfies,verifies` (any of [`LinkKind::ALL`]).
    pub fn parse_kinds(list: &[String]) -> Result<Vec<LinkKind>, String> {
        let mut out = Vec::new();
        for item in list.iter().flat_map(|s| s.split(',')).map(str::trim).filter(|s| !s.is_empty()) {
            let kind = LinkKind::parse(item).ok_or_else(|| format!("unknown link kind '{item}': one of {}", LinkKind::ALL.iter().map(|k| k.as_str()).collect::<Vec<_>>().join(", ")))?;
            if !out.contains(&kind) {
                out.push(kind);
            }
        }
        Ok(if out.is_empty() { LinkKind::ALL.to_vec() } else { out })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Requirement,
    Test,
    Safety,
    /// An interface: a port, an exchange, a link: the ICD.
    Interface,
    /// A constraint or a budget (a functional chain's latency budget).
    Budget,
    /// A declared diagram or table.
    View,
    /// An element typed by a changed type (`is:`).
    TypeUser,
    /// Any other element of the architecture, of any layer.
    Architecture,
}

impl Category {
    pub fn as_str(self) -> &'static str {
        match self {
            Category::Requirement => "requirement",
            Category::Test => "test",
            Category::Safety => "safety",
            Category::Interface => "interface",
            Category::Budget => "budget",
            Category::View => "view",
            Category::TypeUser => "type_user",
            Category::Architecture => "architecture",
        }
    }

    pub fn noun(self) -> &'static str {
        match self {
            Category::Requirement => "requirement",
            Category::Test => "verification case",
            Category::Safety => "safety item",
            Category::Interface => "interface",
            Category::Budget => "budget",
            Category::View => "view",
            Category::TypeUser => "type user",
            Category::Architecture => "element",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactSeverity {
    Blocker,
    Major,
    Minor,
}

impl ImpactSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            ImpactSeverity::Blocker => "blocker",
            ImpactSeverity::Major => "major",
            ImpactSeverity::Minor => "minor",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathStep {
    pub from: String,
    pub relation: String,
    /// The kind of link (`satisfies`, `exchange`...).
    pub kind: String,
    pub to: String,
}

/// `A --satisfies--> B --verified by--> C`.
pub fn path_text(path: &[PathStep]) -> String {
    let Some(first) = path.first() else { return String::new() };
    let mut out = first.from.clone();
    for step in path {
        out.push_str(&format!(" --{}--> {}", step.relation, step.to));
    }
    out
}

#[derive(Debug, Clone, Serialize)]
pub struct Impacted {
    pub id: String,
    pub uuid: String,
    pub name: String,
    pub kind: String,
    pub layer: String,
    pub category: Category,
    pub severity: ImpactSeverity,
    /// Why this severity, in words.
    pub reason: String,
    pub distance: usize,
    /// The shortest explanation, as text.
    pub why: String,
    pub path: Vec<PathStep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrity: Option<String>,
    /// Reached in the model before the change only: the change removes it.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub removed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Seed {
    pub id: String,
    pub uuid: String,
    pub name: String,
    pub kind: String,
    pub layer: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Summary {
    pub total: usize,
    pub blockers: usize,
    pub majors: usize,
    pub minors: usize,
    pub by_layer: BTreeMap<String, usize>,
    pub by_category: BTreeMap<String, usize>,
    pub max_distance: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct GateState {
    pub standard: String,
    pub passed: bool,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImpactReport {
    pub model: String,
    pub seeds: Vec<Seed>,
    pub depth: usize,
    pub kinds: Vec<&'static str>,
    pub direction: Direction,
    pub impacted: Vec<Impacted>,
    pub summary: Summary,
    pub gate: GateState,
    /// The proposed change and its consequences, for a what-if.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub what_if: Option<WhatIf>,
    /// For `--since`: the revision compared with.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    /// Links of each kind in the model, for the filters.
    pub links: BTreeMap<&'static str, usize>,
}

pub fn summarize(impacted: &[Impacted]) -> Summary {
    let mut summary = Summary { total: impacted.len(), ..Summary::default() };
    for item in impacted {
        match item.severity {
            ImpactSeverity::Blocker => summary.blockers += 1,
            ImpactSeverity::Major => summary.majors += 1,
            ImpactSeverity::Minor => summary.minors += 1,
        }
        *summary.by_layer.entry(item.layer.clone()).or_insert(0) += 1;
        *summary.by_category.entry(item.category.as_str().to_string()).or_insert(0) += 1;
        summary.max_distance = summary.max_distance.max(item.distance);
    }
    summary
}

pub fn gate_state(context: &Context) -> GateState {
    GateState { standard: context.gate_standard.clone(), passed: context.gate_passed, blockers: context.gate_blockers.iter().map(|(_, m)| m.clone()).collect() }
}

/// A compiled model indexed for the analysis.
pub struct Analysis<'a> {
    pub context: Context<'a>,
    pub graph: ImpactGraph,
}

impl<'a> Analysis<'a> {
    pub fn new(result: &'a CompilationResult, gate_standard: &str) -> Analysis<'a> {
        let context = Context::new(result, gate_standard);
        let graph = ImpactGraph::build(&context);
        Analysis { context, graph }
    }

    /// The node each reference names; an unknown one is an error naming it.
    pub fn resolve_all(&self, references: &[String]) -> Result<Vec<usize>, String> {
        references
            .iter()
            .map(|r| self.graph.resolve(&self.context, r).ok_or_else(|| format!("element '{r}' not found (give its id, its unique name or Owner.port)")))
            .collect()
    }

    pub fn seed(&self, node: usize) -> Seed {
        let n = self.graph.node(node);
        Seed { id: n.id.clone(), uuid: n.uuid.clone(), name: n.name.clone(), kind: n.kind.clone(), layer: n.layer.clone() }
    }

    pub fn impacted(&self, seeds: &[usize], options: &Options) -> Vec<Impacted> {
        propagate::impacted(&self.context, &self.graph, seeds, options)
    }
}

/// The impact of a change to the given elements.
pub fn analyze(result: &CompilationResult, model: &str, elements: &[String], options: &Options) -> Result<ImpactReport, String> {
    let analysis = Analysis::new(result, &options.gate_standard);
    let seeds = analysis.resolve_all(elements)?;
    let impacted = analysis.impacted(&seeds, options);
    Ok(ImpactReport {
        model: model.to_string(),
        seeds: seeds.iter().map(|&s| analysis.seed(s)).collect(),
        depth: options.depth,
        kinds: options.kinds.iter().map(|k| k.as_str()).collect(),
        direction: options.direction,
        summary: summarize(&impacted),
        impacted,
        gate: gate_state(&analysis.context),
        what_if: None,
        since: None,
        links: analysis.graph.census(),
    })
}
