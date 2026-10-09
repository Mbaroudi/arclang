//! `arclang review`: an advisory reading of what a model MEANS, where the
//! compiler only checks what it declares.
//!
//! The compiler verifies that `trace "LC-1" satisfies "REQ-1"` links two
//! elements that exist; it cannot tell whether that component has anything
//! to do with that requirement. This module asks an external judgment model
//! (TypeSafe's Jev, see `typesafe`) one yes/no question per trace, "is this
//! trace plausible, from what the two elements say about themselves?", and
//! reports the probabilities:
//! - declared traces that look implausible, to look at;
//! - optionally, undeclared pairs that look like a trace that is missing.
//!
//! Rules this module keeps:
//! - It is advice. A probability is never a verdict: nothing here feeds the
//!   compiler, `arclang check` or the production gate, whose results stay
//!   deterministic.
//! - It sends model content to a third party, and only on request: the
//!   kind, name, identifier and text attributes of the elements concerned,
//!   and the rationale of a trace that states one.
//!   `Plan` says exactly what would be sent before anything is.

pub mod typesafe;

use crate::compiler::ast::AttributeValue;
use crate::compiler::elements::{ElementGraph, ElementRecord};
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, HashSet};

/// Kinds whose elements can satisfy a requirement, for suggested traces.
const SATISFYING_KINDS: &[&str] = &["LogicalComponent", "SystemFunction"];

/// One relation between two elements, to be judged.
#[derive(Debug, Clone, PartialEq)]
pub struct Pair {
    pub source_id: String,
    pub target_id: String,
    pub relation: String,
    /// What each end says about itself. With the rationale, this is the
    /// only model content sent.
    pub source: Value,
    pub target: Value,
    /// Why the author declared the trace, when the trace says so.
    pub rationale: Option<String>,
}

/// Something that estimates how plausible a relation is.
pub trait Judge {
    /// Probability, between 0 and 1, that the relation of `pair` is
    /// plausible given what its two ends say about themselves.
    fn plausibility(&mut self, pair: &Pair) -> Result<f64, String>;

    /// The model that answered, once it has.
    fn model(&self) -> Option<String> {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Options {
    /// A declared trace under this plausibility is reported.
    pub threshold: f64,
    /// Also judge undeclared pairs, and report those over this plausibility.
    pub suggest_over: Option<f64>,
    /// Most questions asked in one review; declared traces come first.
    pub max_questions: usize,
}

impl Default for Options {
    fn default() -> Self {
        Options { threshold: 0.3, suggest_over: None, max_questions: 200 }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Finding {
    pub source: String,
    pub relation: String,
    pub target: String,
    pub plausibility: f64,
}

/// The questions a review would ask, before any is sent.
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub traces: Vec<Pair>,
    pub candidates: Vec<Pair>,
    /// Candidate pairs left out because of `max_questions`.
    pub left_out: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Review {
    /// The judgment model that answered.
    pub model: Option<String>,
    pub traces_judged: usize,
    pub candidates_judged: usize,
    pub candidates_left_out: usize,
    /// Declared traces under the threshold, least plausible first.
    pub doubtful_traces: Vec<Finding>,
    /// Undeclared pairs over the suggestion threshold, most plausible first.
    pub suggested_traces: Vec<Finding>,
}

/// What an element says about itself: its kind, name, identifier and text
/// attributes. Presentation attributes say nothing of its meaning.
fn card(element: &ElementRecord) -> Value {
    let mut card = Map::new();
    card.insert("kind".into(), json!(element.kind));
    card.insert("name".into(), json!(element.name));
    card.insert("id".into(), json!(element.id));
    let text: BTreeMap<&String, &str> = element
        .attributes
        .iter()
        .filter(|(key, _)| !matches!(key.as_str(), "id" | "name" | "is" | "color" | "layer"))
        .filter_map(|(key, value)| match value {
            AttributeValue::String(text) if !text.trim().is_empty() => Some((key, text.as_str())),
            _ => None,
        })
        .collect();
    for (key, value) in text {
        card.insert(key.clone(), json!(value));
    }
    Value::Object(card)
}

fn pair(source: &ElementRecord, relation: &str, target: &ElementRecord, rationale: Option<&str>) -> Pair {
    Pair {
        source_id: source.id.clone(),
        target_id: target.id.clone(),
        relation: relation.to_string(),
        source: card(source),
        target: card(target),
        rationale: rationale.map(str::trim).filter(|text| !text.is_empty()).map(str::to_string),
    }
}

/// The questions a review of `graph` asks, in a fixed order: every declared
/// trace, then (when suggestions are wanted) every undeclared pair of an
/// element that can satisfy a requirement and a requirement.
pub fn plan(graph: &ElementGraph, options: &Options) -> Plan {
    let element = |uuid: &str| graph.elements.iter().find(|element| element.uuid == uuid);
    let mut traces = Vec::new();
    let mut declared: HashSet<(&str, &str)> = HashSet::new();
    for relationship in graph.relationships.iter().filter(|r| r.kind == "Trace") {
        let (Some(source), Some(target)) = (element(&relationship.source), element(&relationship.target)) else { continue };
        let relation = relationship.extra.get("traceKind").and_then(Value::as_str).unwrap_or("traces to");
        declared.insert((source.uuid.as_str(), target.uuid.as_str()));
        let rationale = relationship.extra.get("rationale").and_then(Value::as_str);
        traces.push(pair(source, relation, target, rationale));
    }
    traces.truncate(options.max_questions);

    let mut candidates = Vec::new();
    let mut left_out = 0;
    if options.suggest_over.is_some() {
        let requirements: Vec<&ElementRecord> = graph.elements.iter().filter(|e| e.kind == "Requirement").collect();
        let sources = graph.elements.iter().filter(|e| SATISFYING_KINDS.contains(&e.kind));
        for source in sources {
            for requirement in &requirements {
                if declared.contains(&(source.uuid.as_str(), requirement.uuid.as_str())) {
                    continue;
                }
                if traces.len() + candidates.len() < options.max_questions {
                    candidates.push(pair(source, "satisfies", requirement, None));
                } else {
                    left_out += 1;
                }
            }
        }
    }
    Plan { traces, candidates, left_out }
}

fn finding(pair: &Pair, plausibility: f64) -> Finding {
    Finding {
        source: pair.source_id.clone(),
        relation: pair.relation.clone(),
        target: pair.target_id.clone(),
        plausibility,
    }
}

/// Ask `judge` every question of `plan` and keep what crosses a threshold.
pub fn review(plan: &Plan, judge: &mut dyn Judge, options: &Options) -> Result<Review, String> {
    let mut doubtful = Vec::new();
    for pair in &plan.traces {
        let plausibility = judge.plausibility(pair)?;
        if plausibility < options.threshold {
            doubtful.push(finding(pair, plausibility));
        }
    }
    let mut suggested = Vec::new();
    if let Some(over) = options.suggest_over {
        for pair in &plan.candidates {
            let plausibility = judge.plausibility(pair)?;
            if plausibility > over {
                suggested.push(finding(pair, plausibility));
            }
        }
    }
    doubtful.sort_by(|a, b| a.plausibility.total_cmp(&b.plausibility));
    suggested.sort_by(|a, b| b.plausibility.total_cmp(&a.plausibility));
    Ok(Review {
        model: judge.model(),
        traces_judged: plan.traces.len(),
        candidates_judged: plan.candidates.len(),
        candidates_left_out: plan.left_out,
        doubtful_traces: doubtful,
        suggested_traces: suggested,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODEL: &str = r##"
model Brakes {}
system_analysis "Braking" {
    requirement "REQ-1" { description: "Detect obstacles at 150 m" }
    requirement "REQ-2" { description: "Warn the driver before braking" }
    function "Detect" { id: "SF-1" description: "Finds obstacles ahead" }
}
logical_architecture "Logical" {
    component "Radar" { id: "LC-1" description: "Front radar sensor" color: "#fff" latency: 5 ms }
    component "Display" { id: "LC-2" description: "Driver display" }
}
trace "LC-1" satisfies "REQ-1" { rationale: "Long-range radar detection" }
trace "LC-2" satisfies "REQ-1" {}
"##;

    fn graph() -> ElementGraph {
        let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_string(MODEL).unwrap();
        crate::compiler::elements::build(&result.ast, &result.semantic_model)
    }

    /// Answers from a table, and remembers what it was asked.
    struct Scripted {
        answers: Vec<((&'static str, &'static str), f64)>,
        asked: Vec<(String, String)>,
    }

    impl Judge for Scripted {
        fn plausibility(&mut self, pair: &Pair) -> Result<f64, String> {
            self.asked.push((pair.source_id.clone(), pair.target_id.clone()));
            let key = (pair.source_id.as_str(), pair.target_id.as_str());
            Ok(self.answers.iter().find(|(k, _)| *k == key).map_or(0.5, |(_, p)| *p))
        }

        fn model(&self) -> Option<String> {
            Some("scripted".to_string())
        }
    }

    #[test]
    fn a_card_holds_what_an_element_says_about_itself_and_nothing_else() {
        let plan = plan(&graph(), &Options::default());
        let radar = &plan.traces.iter().find(|p| p.source_id == "LC-1").unwrap().source;
        assert_eq!(
            radar,
            &json!({ "kind": "LogicalComponent", "name": "Radar", "id": "LC-1", "description": "Front radar sensor" })
        );
    }

    #[test]
    fn a_trace_carries_its_rationale_and_a_candidate_has_none() {
        let plan = plan(&graph(), &Options { suggest_over: Some(0.8), ..Options::default() });
        let rationale = |source: &str| plan.traces.iter().find(|p| p.source_id == source).unwrap().rationale.clone();
        assert_eq!(rationale("LC-1").as_deref(), Some("Long-range radar detection"));
        assert_eq!(rationale("LC-2"), None);
        assert!(plan.candidates.iter().all(|p| p.rationale.is_none()));
    }

    #[test]
    fn declared_traces_under_the_threshold_are_reported_least_plausible_first() {
        let options = Options::default();
        let plan = plan(&graph(), &options);
        let mut judge = Scripted { answers: vec![(("LC-1", "REQ-1"), 0.9), (("LC-2", "REQ-1"), 0.05)], asked: Vec::new() };

        let review = review(&plan, &mut judge, &options).unwrap();

        assert_eq!(review.traces_judged, 2);
        assert_eq!(review.candidates_judged, 0, "no suggestion unless asked for");
        assert_eq!(review.model.as_deref(), Some("scripted"));
        let doubtful: Vec<(&str, &str)> = review.doubtful_traces.iter().map(|f| (f.source.as_str(), f.target.as_str())).collect();
        assert_eq!(doubtful, [("LC-2", "REQ-1")]);
        assert_eq!(review.doubtful_traces[0].plausibility, 0.05);
    }

    #[test]
    fn suggestions_cover_undeclared_pairs_only_and_keep_the_plausible_ones() {
        let options = Options { suggest_over: Some(0.8), ..Options::default() };
        let plan = plan(&graph(), &options);
        // 3 sources (2 components, 1 function) x 2 requirements, minus the 2 declared.
        assert_eq!(plan.candidates.len(), 4);
        assert!(plan.candidates.iter().all(|p| (p.source_id.as_str(), p.target_id.as_str()) != ("LC-1", "REQ-1")));
        let mut judge = Scripted { answers: vec![(("LC-2", "REQ-2"), 0.93), (("SF-1", "REQ-1"), 0.85), (("LC-1", "REQ-2"), 0.1)], asked: Vec::new() };

        let review = review(&plan, &mut judge, &options).unwrap();

        let suggested: Vec<(&str, &str)> = review.suggested_traces.iter().map(|f| (f.source.as_str(), f.target.as_str())).collect();
        assert_eq!(suggested, [("LC-2", "REQ-2"), ("SF-1", "REQ-1")], "most plausible first");
        assert_eq!(judge.asked.len(), 6);
    }

    #[test]
    fn the_number_of_questions_is_capped_and_what_is_left_out_is_counted() {
        let options = Options { suggest_over: Some(0.8), max_questions: 3, ..Options::default() };
        let plan = plan(&graph(), &options);
        assert_eq!((plan.traces.len(), plan.candidates.len(), plan.left_out), (2, 1, 3));
    }

    #[test]
    fn a_failing_judge_fails_the_review() {
        struct Broken;
        impl Judge for Broken {
            fn plausibility(&mut self, _: &Pair) -> Result<f64, String> {
                Err("service unavailable".to_string())
            }
        }
        let options = Options::default();
        assert_eq!(review(&plan(&graph(), &options), &mut Broken, &options), Err("service unavailable".to_string()));
    }
}
