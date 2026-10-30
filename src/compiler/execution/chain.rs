//! A functional chain (an activity) as a token game.
//!
//! The steps are what the chain `involves` (functions, exchanges, chains)
//! and its control nodes; the arcs are its `sequence` links, else the order
//! of `involves`. A token starts on every step nothing leads to. A step
//! executes, then its token goes on along every arc that leaves it, except:
//! - an `or` control takes the first arc whose `condition` holds, else the
//!   arc without condition; with neither it waits;
//! - an `iterate` control loops along its arc with a condition while that
//!   condition holds, and leaves by the arc without one;
//! - an `and` control with several arcs arriving waits for a token on each
//!   (a join); with several leaving it splits (a fork).
//! A condition is the author's text: the reader sets it, false until set.

use crate::compiler::ast::{ControlKind, FunctionalChain};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
pub struct ChainNode {
    /// The reference as written in `involves`, or the control's name.
    pub reference: String,
    pub control: Option<ControlKind>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Arc {
    pub from: usize,
    pub to: usize,
    pub condition: Option<String>,
}

/// What one step of the chain did.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ChainStep {
    /// The node executed, when one was.
    pub executed: Option<usize>,
    pub notes: Vec<String>,
    pub idle: Option<String>,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChainRun {
    pub name: String,
    pub nodes: Vec<ChainNode>,
    pub arcs: Vec<Arc>,
    /// Nodes holding a token, in the order they run.
    pub tokens: Vec<usize>,
    /// Tokens arrived at an `and` join, by node.
    pub arrived: BTreeMap<usize, usize>,
    pub conditions: BTreeMap<String, bool>,
    pub notes: Vec<String>,
    /// Steps executed so far, for a chain that loops without end.
    pub executed: usize,
}

pub const STEP_LIMIT: usize = 1000;

impl ChainRun {
    pub fn new(chain: &FunctionalChain) -> ChainRun {
        let mut nodes: Vec<ChainNode> = chain.involves.iter().map(|r| ChainNode { reference: r.clone(), control: None }).collect();
        for control in &chain.controls {
            nodes.push(ChainNode { reference: control.name.clone(), control: Some(control.kind) });
        }
        let mut notes = Vec::new();
        let find = |reference: &str, nodes: &[ChainNode]| {
            nodes.iter().position(|n| n.reference == reference).or_else(|| chain.controls.iter().position(|c| c.id == reference).map(|i| chain.involves.len() + i))
        };
        let mut arcs = Vec::new();
        if chain.sequences.is_empty() {
            for i in 1..chain.involves.len() {
                arcs.push(Arc { from: i - 1, to: i, condition: None });
            }
        } else {
            for link in &chain.sequences {
                match (find(&link.from, &nodes), find(&link.to, &nodes)) {
                    (Some(from), Some(to)) => arcs.push(Arc { from, to, condition: link.condition.clone().filter(|c| !c.trim().is_empty()) }),
                    _ => notes.push(format!("sequence {} -> {} names a step the chain does not involve: left out", link.from, link.to)),
                }
            }
        }
        ChainRun { name: chain.name.clone(), nodes, arcs, tokens: Vec::new(), arrived: BTreeMap::new(), conditions: BTreeMap::new(), notes, executed: 0 }
    }

    fn incoming(&self, node: usize) -> usize {
        self.arcs.iter().filter(|a| a.to == node).count()
    }

    fn holds(&self, condition: &Option<String>) -> bool {
        match condition.as_deref().map(str::trim) {
            None => true,
            Some("true") => true,
            Some("false") => false,
            Some(text) => self.conditions.get(text).copied().unwrap_or(false),
        }
    }

    /// The conditions of the arcs: their text.
    pub fn guards(&self) -> Vec<String> {
        let mut out: Vec<String> = self.arcs.iter().filter_map(|a| a.condition.clone()).filter(|c| c != "true" && c != "false").collect();
        out.sort();
        out.dedup();
        out
    }

    /// Put a token on every step nothing leads to.
    pub fn start(&mut self) -> ChainStep {
        self.tokens = (0..self.nodes.len()).filter(|&n| self.incoming(n) == 0 && (self.nodes[n].control.is_none() || self.arcs.iter().any(|a| a.from == n))).collect();
        if self.tokens.is_empty() && !self.nodes.is_empty() {
            self.tokens.push(0);
        }
        self.arrived.clear();
        self.executed = 0;
        ChainStep { notes: vec![format!("tokens on {}", self.holding().join(", "))], ..ChainStep::default() }
    }

    /// The references holding a token.
    pub fn holding(&self) -> Vec<String> {
        self.tokens.iter().map(|&n| self.nodes[n].reference.clone()).collect()
    }

    fn arrive(&mut self, node: usize, notes: &mut Vec<String>) {
        let incoming = self.incoming(node);
        if self.nodes[node].control == Some(ControlKind::And) && incoming > 1 {
            let count = self.arrived.entry(node).or_insert(0);
            *count += 1;
            if *count < incoming {
                notes.push(format!("and {} waits ({}/{incoming})", self.nodes[node].reference, *count));
                return;
            }
            self.arrived.remove(&node);
        }
        self.tokens.push(node);
    }

    /// Execute the first step holding a token.
    pub fn step(&mut self) -> ChainStep {
        let mut step = ChainStep::default();
        if self.tokens.is_empty() {
            step.done = true;
            step.idle = Some("the chain is done: no step holds a token".to_string());
            return step;
        }
        if self.executed >= STEP_LIMIT {
            step.idle = Some(format!("{STEP_LIMIT} steps executed: an iteration that never ends? Set its condition false"));
            return step;
        }
        let node = self.tokens[0];
        let leaving: Vec<Arc> = self.arcs.iter().filter(|a| a.from == node).cloned().collect();
        let taken: Vec<Arc> = match self.nodes[node].control {
            Some(ControlKind::Or) => {
                let pick = leaving.iter().find(|a| a.condition.is_some() && self.holds(&a.condition)).or_else(|| leaving.iter().find(|a| a.condition.is_none()));
                match pick {
                    Some(arc) => vec![arc.clone()],
                    None => {
                        let waiting: Vec<String> = leaving.iter().filter_map(|a| a.condition.clone()).map(|c| format!("'{c}'")).collect();
                        step.idle = Some(format!("or {}: no branch whose condition holds ({})", self.nodes[node].reference, waiting.join(", ")));
                        return step;
                    }
                }
            }
            Some(ControlKind::Iterate) => {
                let again = leaving.iter().find(|a| a.condition.is_some() && self.holds(&a.condition));
                let out = leaving.iter().find(|a| a.condition.is_none());
                match again.or(out) {
                    Some(arc) => vec![arc.clone()],
                    None => {
                        step.idle = Some(format!("iterate {}: its condition does not hold and no arc leaves it without one", self.nodes[node].reference));
                        return step;
                    }
                }
            }
            _ => leaving,
        };
        self.tokens.remove(0);
        self.executed += 1;
        step.executed = Some(node);
        let what = match self.nodes[node].control {
            Some(kind) => format!("{} {}", format!("{kind:?}").to_lowercase(), self.nodes[node].reference),
            None => self.nodes[node].reference.clone(),
        };
        step.notes.push(format!("executed {what}"));
        if taken.len() > 1 {
            step.notes.push(format!("splits into {}", taken.iter().map(|a| self.nodes[a.to].reference.clone()).collect::<Vec<_>>().join(", ")));
        }
        for arc in taken {
            if let Some(condition) = &arc.condition {
                step.notes.push(format!("'{condition}' holds: on to {}", self.nodes[arc.to].reference));
            }
            self.arrive(arc.to, &mut step.notes);
        }
        if self.tokens.is_empty() && self.arrived.is_empty() {
            step.done = true;
            step.notes.push("the chain is done".to_string());
        }
        step
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{Compiler, CompilerConfig};

    fn chain(source: &str) -> ChainRun {
        let result = Compiler::new(CompilerConfig::default()).compile_string(source).expect("compiles");
        ChainRun::new(&result.ast.system_analysis[0].functional_chains[0])
    }

    #[test]
    fn a_plain_chain_runs_in_the_order_it_involves() {
        let mut run = chain("model T { name: \"t\" }\nsystem_analysis \"S\" {\n  function \"A\" { id: \"SF-A\" }\n  function \"B\" { id: \"SF-B\" }\n  function \"C\" { id: \"SF-C\" }\n  functional_chain \"Run\" { id: \"FC-1\" involves: [\"SF-A\", \"SF-B\", \"SF-C\"] }\n}\n");
        run.start();
        let order: Vec<String> = (0..3).map(|_| { let s = run.step(); run.nodes[s.executed.unwrap()].reference.clone() }).collect();
        assert_eq!(order, ["SF-A", "SF-B", "SF-C"]);
        assert!(run.step().done);
    }

    const BRANCHED: &str = "model T { name: \"t\" }\nsystem_analysis \"S\" {\n  function \"Acquire\" { id: \"SF-A\" }\n  function \"Left\" { id: \"SF-L\" }\n  function \"Right\" { id: \"SF-R\" }\n  function \"Merge\" { id: \"SF-M\" }\n  functional_chain \"Branches\" {\n    id: \"FC-2\"\n    involves: [\"SF-A\", \"SF-L\", \"SF-R\", \"SF-M\"]\n    control and \"Fork\"\n    control and \"Join\"\n    control or \"Choose\"\n    sequence \"SF-A\" -> \"Choose\"\n    sequence \"Choose\" -> \"Fork\" { condition: \"both channels\" }\n    sequence \"Choose\" -> \"SF-M\"\n    sequence \"Fork\" -> \"SF-L\"\n    sequence \"Fork\" -> \"SF-R\"\n    sequence \"SF-L\" -> \"Join\"\n    sequence \"SF-R\" -> \"Join\"\n    sequence \"Join\" -> \"SF-M\"\n  }\n}\n";

    #[test]
    fn or_takes_the_branch_whose_condition_holds_and_and_forks_then_joins() {
        let mut run = chain(BRANCHED);
        assert!(run.notes.is_empty(), "{:?}", run.notes);
        run.start();
        assert_eq!(run.holding(), ["SF-A"]);
        run.step();
        // The condition not set: the else branch, straight to Merge.
        let chose = run.step();
        assert!(chose.notes.iter().any(|n| n == "executed or Choose"), "{chose:?}");
        assert_eq!(run.holding(), ["SF-M"]);

        let mut run = chain(BRANCHED);
        run.conditions.insert("both channels".into(), true);
        run.start();
        run.step();
        run.step();
        let fork = run.step();
        assert!(fork.notes.iter().any(|n| n.starts_with("splits into SF-L, SF-R")), "{fork:?}");
        let left = run.step();
        assert!(left.notes.iter().any(|n| n == "and Join waits (1/2)"), "{left:?}");
        let executed: Vec<String> = std::iter::from_fn(|| { let s = run.step(); s.executed.map(|n| run.nodes[n].reference.clone()) }).collect();
        assert_eq!(executed.last().map(String::as_str), Some("SF-M"), "{executed:?}");
        assert!(executed.contains(&"Join".to_string()), "{executed:?}");
    }

    #[test]
    fn iterate_loops_while_its_condition_holds() {
        let mut run = chain("model T { name: \"t\" }\nsystem_analysis \"S\" {\n  function \"Sample\" { id: \"SF-S\" }\n  function \"Report\" { id: \"SF-R\" }\n  functional_chain \"Loop\" {\n    id: \"FC-3\"\n    involves: [\"SF-S\", \"SF-R\"]\n    control iterate \"Again\"\n    sequence \"SF-S\" -> \"Again\"\n    sequence \"Again\" -> \"SF-S\" { condition: \"more samples\" }\n    sequence \"Again\" -> \"SF-R\"\n  }\n}\n");
        run.conditions.insert("more samples".into(), true);
        run.start();
        for _ in 0..4 {
            run.step();
        }
        assert_eq!(run.holding(), ["SF-S"], "looped back twice");
        run.conditions.insert("more samples".into(), false);
        run.step();
        run.step();
        assert_eq!(run.holding(), ["SF-R"]);
        run.step();
        assert!(run.step().done);
    }
}
