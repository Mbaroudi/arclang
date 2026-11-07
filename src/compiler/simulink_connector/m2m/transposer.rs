//! The parts of Kitalpha's Transposer the connector runs on, ported for its
//! rules: the mappings that choose a rule for an element (`MappingHelper`,
//! `MappingElementSpec`), the analysis that makes a graph of the rules'
//! premises (`GenericAnalyzer`), the schedule (`GenericScheduler`: a depth-first
//! visit that finds the cycles and breaks them, then `GenericTopologicalSorter`)
//! and the application of each rule (`AbstractTransformationRule.apply`).
//!
//! The order the rules run in is the order the connector's are run in: the
//! premises are made in the order the rules give them, and every collection the
//! Transposer walks keeps its order.

use super::{data, function, model, sm, Key, Run, Strategy, Target};
use std::collections::{HashMap, HashSet};

/// The connector's rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    Function,
    FunctionPort,
    FunctionalExchange,
    FunctionalChain,
    InterfacePkg,
    DataPkg,
    Class,
    NumericType,
    PhysicalQuantity,
    Enumeration,
    ExchangeItem,
    StateMachine,
    State,
    FinalState,
    Pseudostate,
    Region,
    StateTransition,
    /// The state machines' rule for a functional exchange: its bus.
    ExchangeBus,
}

/// A mapping element: the class it maps, its own possibilities, its default one.
struct Mapped {
    class: &'static str,
    owned: &'static [Rule],
    default: Rule,
}

const DATA: [Mapped; 7] = [
    Mapped { class: "InterfacePkg", owned: &[], default: Rule::InterfacePkg },
    Mapped { class: "DataPkg", owned: &[], default: Rule::DataPkg },
    Mapped { class: "Class", owned: &[], default: Rule::Class },
    Mapped { class: "NumericType", owned: &[], default: Rule::NumericType },
    Mapped { class: "PhysicalQuantity", owned: &[], default: Rule::PhysicalQuantity },
    Mapped { class: "Enumeration", owned: &[], default: Rule::Enumeration },
    Mapped { class: "ExchangeItem", owned: &[], default: Rule::ExchangeItem },
];

/// The mapping of each strategy (`plugin.xml`), its elements in their order.
fn mapping(strategy: Strategy) -> Vec<&'static Mapped> {
    const SM: [Mapped; 6] = [
        Mapped { class: "StateMachine", owned: &[], default: Rule::StateMachine },
        Mapped { class: "State", owned: &[Rule::FinalState], default: Rule::State },
        Mapped { class: "Pseudostate", owned: &[], default: Rule::Pseudostate },
        Mapped { class: "Region", owned: &[], default: Rule::Region },
        Mapped { class: "StateTransition", owned: &[], default: Rule::StateTransition },
        Mapped { class: "FunctionalExchange", owned: &[], default: Rule::ExchangeBus },
    ];
    const FUNCTIONS: [Mapped; 3] = [
        Mapped { class: "AbstractFunction", owned: &[], default: Rule::Function },
        Mapped { class: "FunctionPort", owned: &[], default: Rule::FunctionPort },
        Mapped { class: "FunctionalExchange", owned: &[], default: Rule::FunctionalExchange },
    ];
    const CHAINS: [Mapped; 4] = [
        Mapped { class: "AbstractFunction", owned: &[], default: Rule::Function },
        Mapped { class: "FunctionPort", owned: &[], default: Rule::FunctionPort },
        Mapped { class: "FunctionalChain", owned: &[], default: Rule::FunctionalChain },
        Mapped { class: "FunctionalExchange", owned: &[], default: Rule::FunctionalExchange },
    ];
    let head: &'static [Mapped] = match strategy {
        Strategy::StateMachines => &SM,
        Strategy::Functions => &FUNCTIONS,
        Strategy::FunctionalChains => &CHAINS,
        Strategy::DataPackages => &[],
    };
    head.iter().chain(DATA.iter()).collect()
}

/// `MappingHelper.resolveAllAvailableMappingElements`: the mapping element of the class, else
/// those of its supertypes, the most specific kept.
fn resolve(map: &[&'static Mapped], class: &str, found: &mut Vec<&'static Mapped>) {
    if let Some(&element) = map.iter().find(|m| m.class == class) {
        for known in found.clone() {
            if model::is_a(known.class, class) {
                return;
            }
            if model::is_a(class, known.class) {
                found.retain(|m| m.class != known.class);
            }
        }
        if !found.iter().any(|m| m.class == element.class) {
            found.push(element);
        }
        return;
    }
    for &superclass in model::supertypes(class) {
        resolve(map, superclass, found);
    }
}

/// The rule that applies to an element, if one does (`resolveApplicablePossibility`).
fn rule_for(run: &Run, map: &[&'static Mapped], e: usize) -> Result<Option<Rule>, String> {
    let mut found = Vec::new();
    resolve(map, run.m.class(e), &mut found);
    let element = match found.len() {
        0 => return Ok(None),
        1 => found[0],
        _ => return Err(format!("Ambiguous mapping for {}", run.m.class(e))),
    };
    for &rule in element.owned {
        if applicable(run, rule, e) {
            return Ok(Some(rule));
        }
    }
    Ok(applicable(run, element.default, e).then_some(element.default))
}

fn applicable(run: &Run, rule: Rule, e: usize) -> bool {
    match rule {
        Rule::InterfacePkg | Rule::DataPkg => data::applicable(run, rule, e),
        Rule::State | Rule::FinalState | Rule::Pseudostate | Rule::Region => sm::applicable(run, rule, e),
        _ => true,
    }
}

/// A premise: the element that must be transformed first, and whether it is its container.
pub type Premise = (Option<usize>, bool);

fn premises(run: &Run, rule: Rule, e: usize) -> Vec<Premise> {
    match rule {
        Rule::Function | Rule::FunctionPort | Rule::FunctionalExchange | Rule::FunctionalChain => function::premises(run, rule, e),
        Rule::InterfacePkg | Rule::DataPkg | Rule::Class | Rule::NumericType | Rule::PhysicalQuantity | Rule::Enumeration | Rule::ExchangeItem => data::premises(run, rule, e),
        _ => sm::premises(run, rule, e),
    }
}

fn create(run: &mut Run, rule: Rule, e: usize) -> Result<Option<Target>, String> {
    match rule {
        Rule::Function | Rule::FunctionPort | Rule::FunctionalExchange | Rule::FunctionalChain => {
            if !run.selected(Some(e)) {
                return Ok(None);
            }
            function::create(run, rule, e)
        }
        Rule::ExchangeBus => {
            if !run.selected(Some(e)) {
                return Ok(None);
            }
            sm::create(run, rule, e)
        }
        Rule::InterfacePkg | Rule::DataPkg | Rule::Class | Rule::NumericType | Rule::PhysicalQuantity | Rule::Enumeration | Rule::ExchangeItem => data::create(run, rule, e),
        _ => sm::create(run, rule, e),
    }
}

// ---- The graph ---------------------------------------------------------------------

struct Graph {
    /// Each vertex's element, in the order the vertices were made.
    content: Vec<usize>,
    index: HashMap<usize, usize>,
    /// Each vertex's outgoing edges, in order: (target vertex, critical).
    edges: Vec<Vec<(usize, bool)>>,
    rules: HashMap<usize, Option<Rule>>,
}

impl Graph {
    fn vertex(&mut self, e: usize) -> (usize, bool) {
        if let Some(&v) = self.index.get(&e) {
            return (v, false);
        }
        let v = self.content.len();
        self.content.push(e);
        self.edges.push(Vec::new());
        self.index.insert(e, v);
        (v, true)
    }
}

/// `GenericAnalyzer.analyze`: a vertex for each element a rule applies to, and for each of its
/// premises, depth first.
fn analyze(run: &Run, map: &[&'static Mapped]) -> Result<Graph, String> {
    let mut g = Graph { content: Vec::new(), index: HashMap::new(), edges: Vec::new(), rules: HashMap::new() };
    for &source in &run.selection {
        if rule(run, map, &mut g, source)?.is_some() {
            g.vertex(source);
            subgraph(run, map, &mut g, source)?;
        }
    }
    Ok(g)
}

fn rule(run: &Run, map: &[&'static Mapped], g: &mut Graph, e: usize) -> Result<Option<Rule>, String> {
    if let Some(&known) = g.rules.get(&e) {
        return Ok(known);
    }
    let found = rule_for(run, map, e)?;
    g.rules.insert(e, found);
    Ok(found)
}

fn subgraph(run: &Run, map: &[&'static Mapped], g: &mut Graph, e: usize) -> Result<(), String> {
    let v = g.index[&e];
    let Some(r) = rule(run, map, g, e)? else { return Ok(()) };
    for (premise, critical) in premises(run, r, e) {
        let Some(premise) = premise else {
            return Err(format!("NullPointerException: a premise of {} is missing", run.label(e)));
        };
        let (w, created) = g.vertex(premise);
        match g.edges[v].iter().position(|&(t, _)| t == w) {
            Some(k) => {
                if critical {
                    g.edges[v][k].1 = true;
                }
            }
            None => g.edges[v].push((w, critical)),
        }
        if created {
            subgraph(run, map, g, premise)?;
        }
    }
    Ok(())
}

// ---- The schedule ------------------------------------------------------------------

/// An edge: (source vertex, its index among the source's edges).
type Edge = (usize, usize);

struct Scheduler<'g> {
    g: &'g Graph,
    visited: Vec<usize>,
    is_visited: Vec<bool>,
    path: Vec<Edge>,
    cycles: Vec<Vec<Edge>>,
    backtracks: HashSet<Edge>,
}

impl Scheduler<'_> {
    fn target(&self, (v, k): Edge) -> usize {
        self.g.edges[v][k].0
    }

    fn critical(&self, (v, k): Edge) -> bool {
        self.g.edges[v][k].1
    }

    /// `depthFirstVisitLocalWithCycleSearch`, as written: a vertex already visited is entered
    /// again (to find the cycles through it) but its edges are not followed.
    fn visit(&mut self, v: usize) {
        for k in 0..self.g.edges[v].len() {
            let edge = (v, k);
            if self.path.contains(&edge) {
                continue;
            }
            let next = self.target(edge);
            let index = self.path.iter().position(|&(s, _)| s == next);
            if index.is_some() || next == v {
                let mut cycle: Vec<Edge> = index.map(|i| self.path[i..].to_vec()).unwrap_or_default();
                cycle.push(edge);
                let known = self.cycles.iter().any(|c| c.len() == cycle.len() && cycle.iter().all(|e| c.contains(e)));
                if !known {
                    self.cycles.push(cycle);
                }
            } else if !self.is_visited[v] {
                self.path.push(edge);
                self.visit(next);
            }
        }
        if !self.is_visited[v] {
            self.is_visited[v] = true;
            self.visited.push(v);
        }
        self.path.pop();
    }

    /// `identifyBacktracksFromCycles`: a non-critical edge of each cycle is broken.
    fn break_cycles(&mut self) {
        for cycle in self.cycles.clone() {
            if cycle.len() == 2 {
                for &edge in &cycle {
                    if !self.critical(edge) {
                        self.backtracks.insert(edge);
                    }
                }
            } else if !cycle.iter().any(|e| self.backtracks.contains(e)) {
                if let Some(&edge) = cycle.iter().find(|&&e| !self.critical(e)) {
                    self.backtracks.insert(edge);
                }
            }
        }
    }

    fn independent(&self, v: usize, rest: &HashSet<usize>) -> bool {
        (0..self.g.edges[v].len()).all(|k| self.backtracks.contains(&(v, k)) || !rest.contains(&self.target((v, k))))
    }

    /// `GenericTopologicalSorter.sort`: layer after layer, the vertices that depend on none left.
    fn sort(&mut self) -> Result<Vec<usize>, String> {
        let mut sorted = Vec::new();
        let mut rest: Vec<usize> = self.visited.clone();
        let mut broke_more = false;
        while !rest.is_empty() {
            let set: HashSet<usize> = rest.iter().copied().collect();
            let layer: Vec<usize> = rest.iter().copied().filter(|&v| self.independent(v, &set)).collect();
            if layer.is_empty() {
                if broke_more {
                    return Err("CycleException: the rules' premises make a cycle that cannot be broken".to_string());
                }
                self.break_more(&rest);
                broke_more = true;
                continue;
            }
            broke_more = false;
            rest.retain(|v| !layer.contains(v));
            sorted.extend(layer);
        }
        Ok(sorted)
    }

    /// `lookForOtherBacktracks`, as written.
    fn break_more(&mut self, rest: &[usize]) {
        let set: HashSet<usize> = rest.iter().copied().collect();
        let mut single = Vec::new();
        let mut several = Vec::new();
        for &v in rest {
            let breakable: Vec<Edge> = (0..self.g.edges[v].len()).map(|k| (v, k)).filter(|&e| set.contains(&self.target(e)) && !self.critical(e) && !self.backtracks.contains(&e)).collect();
            if breakable.len() > 1 {
                several.extend(breakable);
            } else {
                single.extend(breakable);
            }
        }
        if several.is_empty() {
            self.backtracks.extend(several);
        } else {
            self.backtracks.extend(single);
        }
    }
}

/// `GenericScheduler.schedule` then `getWork`: each vertex once, completely transposable unless
/// one of its edges was broken; those come again at the end, incompletely.
fn schedule(g: &Graph) -> Result<Vec<(usize, bool)>, String> {
    let mut s = Scheduler { g, visited: Vec::new(), is_visited: vec![false; g.content.len()], path: Vec::new(), cycles: Vec::new(), backtracks: HashSet::new() };
    for v in 0..g.content.len() {
        s.visit(v);
    }
    s.break_cycles();
    let sorted = s.sort()?;
    let mut tasks: Vec<(usize, bool)> = Vec::new();
    let mut again = Vec::new();
    for &v in &sorted {
        let broken = (0..g.edges[v].len()).any(|k| s.backtracks.contains(&(v, k)));
        tasks.push((v, !broken));
        if broken {
            again.push((v, true));
        }
    }
    // The first pass of a vertex with a broken edge is the incomplete one.
    tasks.extend(again);
    Ok(tasks)
}

// ---- Running -----------------------------------------------------------------------

/// Builds the graph, orders it, applies each rule.
pub fn run(run: &mut Run, strategy: Strategy) -> Result<(), String> {
    let map = mapping(strategy);
    let g = analyze(run, &map)?;
    for (v, complete) in schedule(&g)? {
        let e = g.content[v];
        let Some(rule) = g.rules.get(&e).copied().flatten() else { continue };
        if !complete && rule != Rule::Class {
            return Err(format!("{rule:?} should have an incomplete rule"));
        }
        apply(run, rule, e)?;
    }
    Ok(())
}

/// `AbstractTransformationRule.apply`: made once, its main target kept, then updated.
fn apply(run: &mut Run, rule: Rule, e: usize) -> Result<(), String> {
    let key = Key::E(e);
    let created = match run.targets.get(&key).cloned() {
        Some(existing) => Some(existing),
        None => {
            let created = create(run, rule, e)?;
            if let Some(target) = &created {
                run.targets.insert(key, target.clone());
            }
            created
        }
    };
    if let (Some(Target::P(i)), Rule::StateMachine) = (created, rule) {
        sm::update_chart(run, i, e);
    }
    Ok(())
}
