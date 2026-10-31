//! Verification by simulation: a `test_case` that says `simulate:` is
//! verified by running its Modelica model in OpenModelica and checking its
//! `expects:` on the result.
//!
//! ```text
//! test_case "Winter range, long range pack" {
//!     id: "TC-RANGE-LR"
//!     verifies: ["REQ-RANGE"]
//!     method: "analysis"
//!     simulate: { model: "EVPhysics.TouringRange" configuration: "CFG-TOURING"
//!                 stop_time: 5 h sources: ["physics/EVPhysics.mo"] }
//!     expects: ["final(vehicleBody.distance) >= 300 km", "min(batteryPack.soc) >= 0.1"]
//! }
//! ```
//!
//! `arclang verify` exports the model (`modelica::export`), runs the
//! simulation and caches the verdict in `.arclang/sim-cache/` beside the
//! model, keyed by a hash of the generated package, the sources and the
//! case. Compiling never simulates: it reads the cache, and a verdict whose
//! key no longer matches is stale, so inconclusive, as is a case never run.

pub mod cache;
pub mod expects;
pub mod montecarlo;
pub mod replay;
pub mod results;
pub mod run;
pub mod statistics;
pub mod sweep;
pub mod tool;

use crate::compiler::ast::{AttributeValue, Model, TestCase};
use crate::compiler::quantity::Dimension;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The keys `simulate: { ... }` takes.
pub const SIMULATE_KEYS: [&str; 14] = ["backend", "model", "configuration", "stop_time", "start_time", "sources", "tolerance", "intervals", "sweep", "uncertain", "samples", "seed", "probability", "confidence"];

/// What runs a simulation. Every backend ends in the same result (a table
/// of time series) and the same `expects:` evaluation, so adding one
/// changes no model: `fmu` (any FMU, from Dymola, Simulink, PSCAD..., run
/// by FMPy) and `command` (an external script writing a results CSV) are
/// declared in the language and answer inconclusive until they are built.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Backend {
    #[default]
    OpenModelica,
    Fmu,
    Command,
}

impl Backend {
    pub const NAMES: [&'static str; 3] = ["openmodelica", "fmu", "command"];

    pub fn parse(text: &str) -> Option<Backend> {
        match text {
            "openmodelica" => Some(Backend::OpenModelica),
            "fmu" => Some(Backend::Fmu),
            "command" => Some(Backend::Command),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        Self::NAMES[self as usize]
    }

    /// Whether a run is implemented.
    pub fn available(self) -> bool {
        self == Backend::OpenModelica
    }
}

/// What a case simulates.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Spec {
    pub backend: Backend,
    /// The Modelica class simulated: the hand-written model extending a
    /// system model of the generated package.
    pub model: String,
    pub configuration: Option<String>,
    /// Seconds.
    pub start_time: f64,
    pub stop_time: f64,
    /// Modelica files, relative to the model file.
    pub sources: Vec<String>,
    pub tolerance: Option<f64>,
    pub intervals: Option<u32>,
    /// The axes of a parameter sweep, as written (`body.mass = 1800 kg, 2100 kg`).
    pub sweep: Vec<String>,
    /// The uncertain parameters of a Monte Carlo case, as written
    /// (`body.mass ~ normal(1850 kg, 40 kg)`); empty otherwise.
    pub uncertain: Vec<String>,
    pub samples: usize,
    pub seed: u64,
    /// The probability and confidence a Monte Carlo case must show.
    pub required: Option<montecarlo::Requirement>,
}

/// The status of a simulated case on this compilation, from the cache.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Status {
    /// `pass`, `fail`, `error` or `inconclusive`.
    pub verdict: String,
    pub reason: String,
    /// The values the expectations evaluated to (`final(...) = 320 km; ...`).
    pub evidence: String,
    /// The verdict for each requirement the case verifies, by the reference
    /// written in `verifies:`, when its expectations name requirements: a
    /// requirement is judged on the expectations naming it and those naming none.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub by_requirement: HashMap<String, Status>,
}

impl Status {
    pub fn new(verdict: impl Into<String>, reason: impl Into<String>, evidence: impl Into<String>) -> Status {
        Status { verdict: verdict.into(), reason: reason.into(), evidence: evidence.into(), by_requirement: HashMap::new() }
    }
}

/// One run of a case, as cached and reported.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Outcome {
    pub test_case: String,
    /// The test case's name, the requirements it verifies and the name of
    /// its configuration: what a reader of the log looks for.
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub verifies: Vec<String>,
    #[serde(default)]
    pub configuration_name: Option<String>,
    pub verdict: String,
    pub reason: String,
    pub values: Vec<expects::Evaluated>,
    pub key: String,
    pub model: String,
    #[serde(default)]
    pub configuration: Option<String>,
    pub tool: String,
    #[serde(default)]
    pub omc_version: String,
    pub log: String,
    /// How long the run took, in seconds (0 when nothing ran).
    #[serde(default)]
    pub seconds: f64,
    /// Every point of a parameter sweep, or every sample of a Monte Carlo
    /// case; empty when the case sweeps nothing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub points: Vec<PointOutcome>,
    /// What a Monte Carlo case had to show.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub probability: Option<montecarlo::Requirement>,
    /// How much each uncertain parameter moves each margin.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sensitivity: Vec<montecarlo::Influence>,
}

/// One point of a sweep: its parameters, the values its expectations took.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PointOutcome {
    /// `ambient = 253.15 K, target_speed = 90 km/h`.
    pub label: String,
    pub values: Vec<expects::Evaluated>,
    pub holds: bool,
    /// Why the point has no values (the simulation failed, a parameter did not change).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub error: String,
    /// The values of the uncertain parameters at a sample, in SI units.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<(String, f64)>,
}

/// `verifies REQ-RANGE · configuration City (CFG-CITY) · model EVPhysics.CityRange`.
pub fn context_line(outcome: &Outcome) -> String {
    let mut parts = Vec::new();
    if !outcome.verifies.is_empty() {
        parts.push(format!("verifies {}", outcome.verifies.join(", ")));
    }
    match (&outcome.configuration_name, &outcome.configuration) {
        (Some(name), Some(id)) if name != id => parts.push(format!("configuration {name} ({id})")),
        (_, Some(id)) => parts.push(format!("configuration {id}")),
        _ => {}
    }
    if !outcome.model.is_empty() {
        parts.push(format!("model {}", outcome.model));
    }
    parts.join(" · ")
}

impl Outcome {
    pub fn evidence(&self) -> String {
        evidence_of(&self.values)
    }

    pub fn status(&self) -> Status {
        Status::new(self.verdict.clone(), self.reason.clone(), self.evidence())
    }

    /// Whether an expectation of the case names a requirement.
    pub fn names_requirements(&self) -> bool {
        self.values.iter().chain(self.points.iter().flat_map(|p| p.values.iter())).any(|v| v.requirement.is_some())
    }

    /// The verdict of the case on the expectations `checks` keeps (by the
    /// requirement each names, `None` when it names none). An error or an
    /// inconclusive run is that for every requirement.
    pub fn status_for(&self, checks: impl Fn(Option<&str>) -> bool) -> Status {
        if !matches!(self.verdict.as_str(), "pass" | "fail") || !self.names_requirements() {
            return self.status();
        }
        let kept = |values: &[expects::Evaluated]| values.iter().filter(|v| checks(v.requirement.as_deref())).cloned().collect::<Vec<_>>();
        let point = |label: &str, values: Vec<expects::Evaluated>, error: &str| PointOutcome { label: label.to_string(), holds: error.is_empty() && values.iter().all(|v| v.holds), values, error: error.to_string(), inputs: Vec::new() };
        let points: Vec<PointOutcome> = if self.points.is_empty() {
            vec![point("", kept(&self.values), "")]
        } else {
            self.points.iter().map(|p| point(&p.label, kept(&p.values), &p.error)).collect()
        };
        let (verdict, reason, values) = conclusion_for(&points, self.probability);
        Status::new(verdict, reason, evidence_of(&values))
    }
}

fn evidence_of(values: &[expects::Evaluated]) -> String {
    values.iter().map(|v| format!("{}{}", v.value, if v.holds { "" } else { " (fails)" })).collect::<Vec<_>>().join("; ")
}

/// The verdict of a case from its points, its reason and the values to show:
/// an error at one point is an error, a failure at one a failure.
pub fn conclusion(points: &[PointOutcome]) -> (String, String, Vec<expects::Evaluated>) {
    let swept = points.len() > 1 || points.first().is_some_and(|p| !p.label.is_empty());
    let at = |p: &PointOutcome| if p.label.is_empty() { String::new() } else { format!("at {}: ", p.label) };
    if let Some(broken) = points.iter().find(|p| !p.error.is_empty()) {
        return ("error".to_string(), format!("{}{}", at(broken), broken.error), Vec::new());
    }
    if let Some(failing) = points.iter().find(|p| !p.holds) {
        let bad: Vec<&expects::Evaluated> = failing.values.iter().filter(|v| !v.holds).collect();
        let count = points.iter().filter(|p| !p.holds).count();
        let many = if swept { format!("{count} of {} points fail; first ", points.len()) } else { String::new() };
        let reason = format!(
            "{many}{}{} (expected {})",
            at(failing),
            bad.iter().map(|v| v.value.as_str()).collect::<Vec<_>>().join("; "),
            bad.iter().map(|v| v.expect.as_str()).collect::<Vec<_>>().join("; ")
        );
        return ("fail".to_string(), reason, failing.values.clone());
    }
    ("pass".to_string(), String::new(), points.first().map(|p| p.values.clone()).unwrap_or_default())
}

/// The verdict of a case from its points, judged as samples when the case
/// requires a probability, as sweep points otherwise.
pub fn conclusion_for(points: &[PointOutcome], probability: Option<montecarlo::Requirement>) -> (String, String, Vec<expects::Evaluated>) {
    let (verdict, reason, values) = conclusion(points);
    let Some(required) = probability else { return (verdict, reason, values) };
    let (verdict, reason) = montecarlo::judge(points, required);
    (verdict, reason, values)
}

fn is_class_name(text: &str) -> bool {
    !text.is_empty() && text.split('.').all(|part| part.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_') && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
}

fn seconds(map: &HashMap<String, AttributeValue>, key: &str) -> Result<Option<f64>, String> {
    let Some(value) = map.get(key) else { return Ok(None) };
    match value.as_quantity() {
        Some(Ok(q)) if q.dimension() == Dimension::Time => Ok(Some(q.canonical())),
        Some(Ok(q)) => Err(format!("{key} is a {}, not a time", q.dimension().label())),
        _ if value.as_number().is_some() => Err(format!("{key} has no unit: write `{key}: {} s`", value.display())),
        _ => Err(format!("{key} '{}' is not a time (e.g. `{key}: 3600 s`)", value.display())),
    }
}

/// The `simulate:` of a case, `None` when it says none.
pub fn spec_of(attributes: &HashMap<String, AttributeValue>) -> Option<Result<Spec, String>> {
    let value = attributes.get("simulate")?;
    let AttributeValue::Map(map) = value else {
        return Some(Err("simulate is a block: simulate: { model: \"Package.Model\" stop_time: 3600 s }".to_string()));
    };
    Some(read_spec(map))
}

fn read_spec(map: &HashMap<String, AttributeValue>) -> Result<Spec, String> {
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();
    if let Some(unknown) = keys.iter().find(|k| !SIMULATE_KEYS.contains(&k.as_str())) {
        return Err(format!("simulate: '{unknown}' is not one of {}", SIMULATE_KEYS.join(", ")));
    }
    let backend = match map.get("backend").map(|v| v.display()) {
        None => Backend::default(),
        Some(name) => Backend::parse(&name).ok_or_else(|| format!("simulate: backend '{name}' is not one of {}", Backend::NAMES.join(", ")))?,
    };
    let model = map.get("model").and_then(|v| v.as_string()).ok_or("simulate: model is required (the Modelica class to simulate)")?.to_string();
    if !is_class_name(&model) {
        return Err(format!("simulate: model '{model}' is not a Modelica class name (Package.Model)"));
    }
    let stop_time = seconds(map, "stop_time").map_err(|e| format!("simulate: {e}"))?.ok_or("simulate: stop_time is required (e.g. stop_time: 3600 s)")?;
    let start_time = seconds(map, "start_time").map_err(|e| format!("simulate: {e}"))?.unwrap_or(0.0);
    if stop_time <= start_time {
        return Err("simulate: stop_time must come after start_time".to_string());
    }
    let sources = match map.get("sources") {
        None => Vec::new(),
        Some(AttributeValue::List(items)) => items.iter().map(|i| i.as_string().map(str::to_string).ok_or("simulate: sources are file paths")).collect::<Result<_, _>>()?,
        Some(AttributeValue::String(one)) => vec![one.clone()],
        Some(_) => return Err("simulate: sources are file paths".to_string()),
    };
    let tolerance = match map.get("tolerance") {
        None => None,
        Some(v) => Some(v.as_number().filter(|t| *t > 0.0).ok_or("simulate: tolerance is a positive number")?),
    };
    let intervals = match map.get("intervals") {
        None => None,
        Some(AttributeValue::Integer(n)) if *n > 0 && *n <= 1_000_000 => Some(*n as u32),
        Some(_) => return Err("simulate: intervals is a whole number of output intervals".to_string()),
    };
    let configuration = map.get("configuration").and_then(|v| v.as_string()).map(str::to_string);
    let sweep = match map.get("sweep") {
        None => Vec::new(),
        Some(AttributeValue::List(items)) => items.iter().map(|i| i.as_string().map(str::to_string).ok_or("simulate: sweep is a list of \"parameter = values\"")).collect::<Result<_, _>>()?,
        Some(AttributeValue::String(one)) => vec![one.clone()],
        Some(_) => return Err("simulate: sweep is a list of \"parameter = values\"".to_string()),
    };
    // ArcLang ids are resolved when the package is exported: here, the syntax and the size.
    let unresolved: Vec<String> = sweep
        .iter()
        .map(|t| match t.trim_start().strip_prefix('"').and_then(|rest| rest.split_once('"')) {
            Some((_, path)) => format!("part{path}"),
            None => t.clone(),
        })
        .collect();
    sweep::parse(&unresolved, &Default::default())?;
    let (uncertain, samples, seed, required) = read_monte_carlo(map, &sweep)?;
    Ok(Spec { backend, model, configuration, start_time, stop_time, sources, tolerance, intervals, sweep, uncertain, samples, seed, required })
}

fn fraction(map: &HashMap<String, AttributeValue>, key: &str, low: f64) -> Result<Option<f64>, String> {
    match map.get(key) {
        None => Ok(None),
        Some(v) => match v.as_number() {
            Some(p) if p > low && p < 1.0 => Ok(Some(p)),
            _ => Err(format!("simulate: {key} is a number between {low} and 1 (e.g. {key}: 0.99), got '{}'", v.display())),
        },
    }
}

/// `uncertain:`, `samples:`, `seed:`, `probability:` and `confidence:`.
fn read_monte_carlo(map: &HashMap<String, AttributeValue>, sweep: &[String]) -> Result<(Vec<String>, usize, u64, Option<montecarlo::Requirement>), String> {
    let uncertain: Vec<String> = match map.get("uncertain") {
        None => Vec::new(),
        Some(AttributeValue::List(items)) => items.iter().map(|i| i.as_string().map(str::to_string).ok_or("simulate: uncertain is a list of \"parameter ~ law\"")).collect::<Result<_, _>>()?,
        Some(AttributeValue::String(one)) => vec![one.clone()],
        Some(_) => return Err("simulate: uncertain is a list of \"parameter ~ law\"".to_string()),
    };
    let probability = fraction(map, "probability", 0.0)?;
    let confidence = fraction(map, "confidence", 0.5)?;
    if uncertain.is_empty() {
        if let Some(key) = ["samples", "seed", "probability", "confidence"].into_iter().find(|k| map.contains_key(*k)) {
            return Err(format!("simulate: {key} needs uncertain: [\"parameter ~ law\", ...], the parameters to sample"));
        }
        return Ok((uncertain, 0, 0, None));
    }
    if !sweep.is_empty() {
        return Err("simulate: a case either sweeps or samples: give sweep: or uncertain:, not both".to_string());
    }
    let probability = probability.ok_or("simulate: uncertain: needs probability: (e.g. probability: 0.99), how likely the expectations must hold")?;
    let samples = match map.get("samples") {
        None => montecarlo::DEFAULT_SAMPLES,
        Some(AttributeValue::Integer(n)) if *n >= 10 && (*n as usize) <= montecarlo::MAX_SAMPLES => *n as usize,
        Some(v) => return Err(format!("simulate: samples is a whole number from 10 to {}, got '{}'", montecarlo::MAX_SAMPLES, v.display())),
    };
    let seed = match map.get("seed") {
        None => montecarlo::DEFAULT_SEED,
        Some(AttributeValue::Integer(n)) if *n >= 0 => *n as u64,
        Some(v) => return Err(format!("simulate: seed is a whole number, got '{}'", v.display())),
    };
    let unresolved: Vec<String> = uncertain
        .iter()
        .map(|t| match t.trim_start().strip_prefix('"').and_then(|rest| rest.split_once('"')) {
            Some((_, path)) => format!("part{path}"),
            None => t.clone(),
        })
        .collect();
    montecarlo::parse(&unresolved, &Default::default())?;
    let required = montecarlo::Requirement { probability, confidence: confidence.unwrap_or(montecarlo::DEFAULT_CONFIDENCE) };
    Ok((uncertain, samples, seed, Some(required)))
}

/// The `expects:` of a case.
pub fn expects_of(attributes: &HashMap<String, AttributeValue>) -> Vec<String> {
    match attributes.get("expects") {
        Some(AttributeValue::List(items)) => items.iter().filter_map(|v| v.as_string()).map(str::to_string).collect(),
        Some(AttributeValue::String(one)) => vec![one.clone()],
        _ => Vec::new(),
    }
}

/// What the parser checks of a case's simulation: a well-formed block,
/// expectations that parse, and no other source of verdict.
pub fn validate(attributes: &HashMap<String, AttributeValue>) -> Result<(), String> {
    let has_expects = attributes.contains_key("expects");
    let Some(spec) = spec_of(attributes) else {
        return if has_expects { Err("expects: needs simulate: { ... }, the simulation they are read on".to_string()) } else { Ok(()) };
    };
    spec?;
    if attributes.contains_key("asserts") || attributes.contains_key("verdict") {
        return Err("its verdict comes from the simulation: it neither asserts constraints nor gives a verdict".to_string());
    }
    let expects = expects_of(attributes);
    if expects.is_empty() {
        return Err("simulate: needs expects: [\"final(x) >= 1\", ...], what the result must show".to_string());
    }
    expects.iter().try_for_each(|e| expects::check_syntax(e))
}

/// The compile-time check of simulated cases: their configuration exists.
pub fn check(model: &Model) -> Result<(), String> {
    for test in &model.test_cases {
        let Some(Ok(spec)) = spec_of(&test.attributes) else { continue };
        if let Some(configuration) = &spec.configuration {
            if !model.configurations.iter().any(|c| &c.id == configuration || &c.name == configuration) {
                return Err(format!("test_case '{}' simulates configuration '{configuration}', which is none", test.name));
            }
        }
    }
    Ok(())
}

pub fn is_simulated(test: &TestCase) -> bool {
    test.attributes.contains_key("simulate")
}
