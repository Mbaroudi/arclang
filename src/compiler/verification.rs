//! Verdicts of verification cases. A `test_case` that `asserts:`
//! constraints is an analysis: its verdict is theirs, pass when they all
//! hold, fail otherwise. Any other case may say the verdict of a test run
//! elsewhere (`verdict: "fail"`), with the SysML v2 verdicts; a case saying
//! none is pending. A case that `simulate:`s takes its verdict from the
//! cached simulation (`compiler::simulation`): inconclusive when it was
//! never run, or was run on another version of the model. A case that names
//! `evidence:` takes a verdict obtained outside the compiler, checked against
//! the model as it is (`compiler::evidence`): stale or missing, it is
//! inconclusive. A requirement
//! with a failing or inconclusive verification case is not verified, for
//! the production gate, even when another case passes; nor is one whose
//! case says no verdict. Where a requirement stands is computed here, once
//! ([`requirement_status`]): the gate, the coverage table, the impact
//! analysis and the requirements view read it.

use super::ast::{AttributeValue, Model, TestCase};
use super::semantic::{ConstraintInfo, SemanticModel};

/// The verdicts of SysML v2 (`VerificationCases::VerdictKind`).
pub const VERDICTS: [&str; 4] = ["pass", "fail", "inconclusive", "error"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Nothing says the outcome yet.
    Pending,
    Pass,
    /// Failed, with why: the constraints that do not hold, or the verdict given.
    Fail(String),
    /// Not known to hold or fail: a simulation not run, stale, or a tool
    /// that is not available. Never a pass.
    Inconclusive(String),
}

/// The constraints a case asserts, by id or name.
pub fn asserted(test: &TestCase) -> Vec<String> {
    match test.attributes.get("asserts") {
        Some(AttributeValue::List(items)) => items.iter().filter_map(|v| v.as_string()).map(str::to_string).collect(),
        Some(AttributeValue::String(one)) => vec![one.clone()],
        _ => Vec::new(),
    }
}

fn constraint<'s>(semantic: &'s SemanticModel, reference: &str) -> Option<&'s ConstraintInfo> {
    semantic.constraints.iter().find(|c| c.id == reference).or_else(|| semantic.constraints.iter().find(|c| c.name == reference))
}

/// The id a reference to a requirement names (see [`resolve_requirement`]),
/// or the reference itself when it names no requirement.
pub fn requirement_key(model: &Model, reference: &str) -> String {
    resolve_requirement(model, reference).unwrap_or_else(|| reference.to_string())
}

/// The id of the requirement a reference names: the requirement of this id,
/// else the one of this name (the compiler refuses two requirements of one
/// name). Every reader of `verifies:` resolves a reference here, the gate
/// included.
pub fn resolve_requirement(model: &Model, reference: &str) -> Option<String> {
    requirements(model)
        .find(|(_, _, id)| id == reference)
        .or_else(|| requirements(model).find(|(_, name, _)| *name == reference))
        .map(|(_, _, id)| id)
}

/// Every requirement: its block, its name, its id.
fn requirements(model: &Model) -> impl Iterator<Item = (&super::ast::Requirement, &str, String)> {
    model.system_analysis.iter().flat_map(|sa| sa.requirements.iter()).map(|r| {
        let id = r.attributes.get("id").and_then(|v| v.as_string()).map(str::to_string).unwrap_or_else(|| r.id.clone());
        (r, r.id.as_str(), id)
    })
}

/// The levels of the V, from the bottom up: a component against its
/// detailed design, components integrated against the architecture, the
/// system against its requirements, the system against the operational need.
pub const LEVELS: [&str; 4] = ["component", "subsystem", "system", "validation"];

/// The level a text names, whatever its case.
fn level_named(text: &str) -> Option<&'static str> {
    LEVELS.into_iter().find(|level| level.eq_ignore_ascii_case(text.trim()))
}

/// The level of the V a case says it belongs to (`level: "system"`); `None`
/// when it says none: a level is never deduced.
pub fn level_of(test: &TestCase) -> Option<&'static str> {
    test.attributes.get("level").and_then(|v| v.as_string()).and_then(level_named)
}

/// The levels as an attribute writes them: a list, or one alone.
fn levels_written(value: &AttributeValue) -> Vec<&str> {
    match value {
        AttributeValue::List(items) => items.iter().filter_map(|v| v.as_string()).collect(),
        AttributeValue::String(one) => vec![one.as_str()],
        _ => Vec::new(),
    }
}

/// The verification plan of a requirement: the levels at which it is to be
/// verified (`verification_levels: ["system", "validation"]`), from the
/// bottom of the V up; empty when it plans none.
pub fn planned_levels(model: &Model, requirement: &str) -> Vec<&'static str> {
    let key = requirement_key(model, requirement);
    let Some((block, _, _)) = requirements(model).find(|(_, _, id)| *id == key) else { return Vec::new() };
    let written: Vec<&'static str> = block.attributes.get("verification_levels").map(levels_written).unwrap_or_default().into_iter().filter_map(level_named).collect();
    LEVELS.into_iter().filter(|level| written.contains(level)).collect()
}

/// The id of a configuration named by its id or its name.
fn configuration_key(model: &Model, reference: &str) -> Option<String> {
    model.configurations.iter().find(|c| c.id == reference || c.name == reference).map(|c| c.id.clone())
}

/// The configurations a requirement applies to (`applies_to: ["CFG-TOURING"]`),
/// by id; `None` when it applies to every one.
pub fn applies_to(model: &Model, requirement: &str) -> Option<Vec<String>> {
    let key = requirement_key(model, requirement);
    let (block, _, _) = requirements(model).find(|(_, _, id)| *id == key)?;
    let written: Vec<String> = match block.attributes.get("applies_to")? {
        AttributeValue::List(items) => items.iter().filter_map(|v| v.as_string()).map(str::to_string).collect(),
        AttributeValue::String(one) => vec![one.clone()],
        _ => return None,
    };
    Some(written.iter().map(|w| configuration_key(model, w).unwrap_or_else(|| w.clone())).collect())
}

/// The configuration a case runs in, by id: the one it simulates, else the
/// one it names (`configuration:`, where a verdict recorded by hand was
/// obtained); `None` when it names none, and so stands for every configuration.
pub fn configuration_of(model: &Model, test: &TestCase) -> Option<String> {
    let written = simulated_configuration(test).or_else(|| named_configuration(test))?;
    Some(configuration_key(model, &written).unwrap_or(written))
}

/// The configuration of `simulate: { configuration: ... }`, as written.
fn simulated_configuration(test: &TestCase) -> Option<String> {
    super::simulation::spec_of(&test.attributes)?.ok()?.configuration
}

/// The `configuration:` of the case itself, as written.
fn named_configuration(test: &TestCase) -> Option<String> {
    test.attributes.get("configuration").and_then(|v| v.as_string()).map(str::to_string)
}

/// The verdict of a case for one requirement it verifies, by the reference
/// written in its `verifies:`. A simulated case whose expectations name
/// requirements is judged, for each, on the expectations that check it.
pub fn verdict_for(test: &TestCase, reference: &str, semantic: &SemanticModel) -> Verdict {
    if let Some(status) = semantic.simulations.get(&test.id).and_then(|s| s.by_requirement.get(reference)) {
        return match status.verdict.as_str() {
            "pass" => Verdict::Pass,
            "fail" => Verdict::Fail(status.reason.clone()),
            "error" => Verdict::Fail(format!("the simulation is in error: {}", status.reason)),
            _ => Verdict::Inconclusive(status.reason.clone()),
        };
    }
    verdict(test, semantic)
}

pub fn verdict(test: &TestCase, semantic: &SemanticModel) -> Verdict {
    let executed = super::embedded::case::is_executed(test);
    if super::simulation::is_simulated(test) || executed {
        return match semantic.simulations.get(&test.id) {
            None if executed => Verdict::Inconclusive("no SIL verdict: run `arclang sil` on the model file".to_string()),
            None => Verdict::Inconclusive("no simulation verdict: run `arclang verify` on the model file".to_string()),
            Some(status) if executed && status.verdict == "error" => Verdict::Fail(format!("the SIL run is in error: {}", status.reason)),
            Some(status) => match status.verdict.as_str() {
                "pass" => Verdict::Pass,
                "fail" => Verdict::Fail(status.reason.clone()),
                "error" => Verdict::Fail(format!("the simulation is in error: {}", status.reason)),
                _ => Verdict::Inconclusive(status.reason.clone()),
            },
        };
    }
    if super::evidence::is_external(test) {
        // A verdict obtained outside the compiler, read and checked against the model as it is (`compiler::evidence`).
        return match semantic.simulations.get(&test.id) {
            None => Verdict::Inconclusive(format!("no evidence read: compile the model file, so that what '{}' recorded beside it is found", super::evidence::named(test).unwrap_or_default())),
            Some(status) => match status.verdict.as_str() {
                "pass" => Verdict::Pass,
                "fail" => Verdict::Fail(status.reason.clone()),
                "error" => Verdict::Fail(format!("the evidence is in error: {}", status.reason)),
                _ => Verdict::Inconclusive(status.reason.clone()),
            },
        };
    }
    let asserts = asserted(test);
    if !asserts.is_empty() {
        let failing: Vec<&str> = asserts.iter().filter_map(|a| constraint(semantic, a)).filter(|c| !c.satisfied).map(|c| c.name.as_str()).collect();
        return match failing.as_slice() {
            [] => Verdict::Pass,
            [one] => Verdict::Fail(format!("constraint '{one}' does not hold")),
            many => Verdict::Fail(format!("constraints '{}' do not hold", many.join("', '"))),
        };
    }
    match test.attributes.get("verdict").and_then(|v| v.as_string()) {
        Some("pass") => Verdict::Pass,
        Some(other @ ("fail" | "inconclusive" | "error")) => Verdict::Fail(format!("its verdict is {other}")),
        _ => Verdict::Pending,
    }
}

/// Where the verification of a requirement stands, the worst first: one
/// case that fails, that is inconclusive or that says nothing is not hidden
/// by another that passes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// No case verifies it, or none runs in a configuration it applies to,
    /// or none at a level it is planned to be verified at.
    Unverified,
    Fail,
    Inconclusive,
    /// A case verifies it and says no verdict yet.
    Pending,
    /// Every case passes, in every configuration it applies to and at
    /// every level it is planned at.
    Pass,
}

impl Standing {
    pub fn word(self) -> &'static str {
        match self {
            Standing::Unverified => "unverified",
            Standing::Fail => "fail",
            Standing::Inconclusive => "inconclusive",
            Standing::Pending => "pending",
            Standing::Pass => "pass",
        }
    }
}

impl Verdict {
    pub fn word(&self) -> &'static str {
        match self {
            Verdict::Pending => "pending",
            Verdict::Pass => "pass",
            Verdict::Fail(_) => "fail",
            Verdict::Inconclusive(_) => "inconclusive",
        }
    }

    /// Why a case fails or is inconclusive.
    pub fn why(&self) -> Option<&str> {
        match self {
            Verdict::Fail(why) | Verdict::Inconclusive(why) => Some(why),
            _ => None,
        }
    }
}

/// What satisfies a requirement and what verifies it, with the verdicts.
#[derive(Debug, Clone)]
pub struct RequirementStatus<'a> {
    /// The elements a `satisfies` trace leads from, by id, in the order of the model.
    pub satisfied_by: Vec<&'a str>,
    /// The cases that verify it (their `verifies:`), each with its verdict
    /// for this requirement.
    pub cases: Vec<(&'a TestCase, Verdict)>,
    /// The configurations it applies to that no case runs in, by id.
    pub uncovered: Vec<String>,
    /// The levels of its plan (`verification_levels:`) no case verifies it
    /// at, from the bottom of the V up.
    pub unverified_levels: Vec<&'static str>,
}

impl RequirementStatus<'_> {
    /// Where the requirement stands at one level of the V: on the cases of
    /// that level alone, the worst first; `None` when none is of that level.
    pub fn standing_at(&self, level: &str) -> Option<Standing> {
        let mut words = self.cases.iter().filter(|(test, _)| level_of(test) == Some(level)).map(|(_, verdict)| verdict.word()).peekable();
        words.peek()?;
        let words: Vec<&str> = words.collect();
        Some(if words.contains(&"fail") {
            Standing::Fail
        } else if words.contains(&"inconclusive") {
            Standing::Inconclusive
        } else if words.contains(&"pending") {
            Standing::Pending
        } else {
            Standing::Pass
        })
    }

    pub fn satisfied(&self) -> bool {
        !self.satisfied_by.is_empty()
    }

    pub fn standing(&self) -> Standing {
        let any = |wanted: &str| self.cases.iter().any(|(_, verdict)| verdict.word() == wanted);
        if self.cases.is_empty() {
            Standing::Unverified
        } else if any("fail") {
            Standing::Fail
        } else if any("inconclusive") {
            Standing::Inconclusive
        } else if any("pending") {
            Standing::Pending
        } else if !self.uncovered.is_empty() || !self.unverified_levels.is_empty() {
            Standing::Unverified
        } else {
            Standing::Pass
        }
    }

    /// Verified: every case passes, and none is missing. Nothing else counts.
    pub fn verified(&self) -> bool {
        self.standing() == Standing::Pass
    }
}

/// The status of the requirement of this id (see [`requirement_key`]).
/// Only a test case verifies, by its `verifies:`.
pub fn requirement_status<'a>(model: &'a Model, semantic: &'a SemanticModel, requirement: &str) -> RequirementStatus<'a> {
    let mut satisfied_by: Vec<&str> = Vec::new();
    for trace in semantic.traces.iter().filter(|t| t.trace_type == "satisfies" && t.to == requirement) {
        if !satisfied_by.contains(&trace.from.as_str()) {
            satisfied_by.push(&trace.from);
        }
    }
    let cases: Vec<(&TestCase, Verdict)> = model
        .test_cases
        .iter()
        .filter_map(|test| {
            // The reference as the case writes it: an expectation naming its requirement is judged under it.
            let written = test.verifies.iter().find(|r| requirement_key(model, r) == requirement)?;
            Some((test, verdict_for(test, written, semantic)))
        })
        .collect();
    let uncovered = applies_to(model, requirement)
        .unwrap_or_default()
        .into_iter()
        .filter(|configuration| !cases.iter().any(|(test, _)| configuration_of(model, test).is_none_or(|runs_in| &runs_in == configuration)))
        .collect();
    let unverified_levels = planned_levels(model, requirement).into_iter().filter(|level| !cases.iter().any(|(test, _)| level_of(test) == Some(level))).collect();
    RequirementStatus { satisfied_by, cases, uncovered, unverified_levels }
}

/// An asserted constraint that is none is an error; a failing case, a
/// `verification:` warning.
pub fn check(model: &Model, semantic: &SemanticModel) -> Result<Vec<String>, String> {
    let mut warnings = Vec::new();
    super::simulation::check(model)?;
    check_applicability(model)?;
    check_levels(model)?;
    for test in &model.test_cases {
        if let Some(unknown) = asserted(test).into_iter().find(|a| constraint(semantic, a).is_none()) {
            return Err(format!("test_case '{}' asserts '{unknown}', which is no constraint", test.name));
        }
        // A simulated or executed case is reported with its cached verdict, on the file.
        if super::simulation::is_simulated(test) || super::embedded::case::is_executed(test) {
            continue;
        }
        if let Verdict::Fail(reason) = verdict(test, semantic) {
            warnings.push(format!("verification: test_case '{}' fails: {reason}", test.name));
        }
    }
    Ok(warnings)
}

/// A level is one of the V, and a plan names each of its levels once.
fn check_levels(model: &Model) -> Result<(), String> {
    let known = LEVELS.join(", ");
    for test in &model.test_cases {
        let Some(value) = test.attributes.get("level") else { continue };
        if value.as_string().and_then(level_named).is_none() {
            return Err(format!("test_case '{}': level '{}' is not a level of the V (expected one of: {known})", test.name, value.display()));
        }
    }
    for (block, _, id) in requirements(model) {
        let Some(value) = block.attributes.get("verification_levels") else { continue };
        let written = levels_written(value);
        if written.is_empty() {
            return Err(format!("requirement '{id}': verification_levels names the levels it is to be verified at: verification_levels: [\"system\"] (one of: {known})"));
        }
        let mut seen: Vec<&str> = Vec::new();
        for text in written {
            let Some(level) = level_named(text) else {
                return Err(format!("requirement '{id}': verification level '{text}' is not a level of the V (expected one of: {known})"));
            };
            if seen.contains(&level) {
                return Err(format!("requirement '{id}' plans the {level} level twice"));
            }
            seen.push(level);
        }
    }
    Ok(())
}

/// `applies_to:` names configurations; a case verifying a requirement runs in
/// one it applies to; an expectation naming a requirement names one its case
/// verifies, and every requirement a case verifies is checked by one.
fn check_applicability(model: &Model) -> Result<(), String> {
    for (block, name, id) in requirements(model) {
        let Some(value) = block.attributes.get("applies_to") else { continue };
        let written: Vec<&str> = match value {
            AttributeValue::List(items) => items.iter().filter_map(|v| v.as_string()).collect(),
            AttributeValue::String(one) => vec![one.as_str()],
            _ => return Err(format!("requirement '{name}': applies_to is a list of configurations: applies_to: [\"CFG-A\"]")),
        };
        if written.is_empty() {
            return Err(format!("requirement '{name}': applies_to names no configuration"));
        }
        if let Some(unknown) = written.iter().find(|w| configuration_key(model, w).is_none()) {
            return Err(format!("requirement '{id}' applies to '{unknown}', which is no configuration"));
        }
    }
    for test in &model.test_cases {
        if let Some(named) = named_configuration(test) {
            let Some(key) = configuration_key(model, &named) else {
                return Err(format!("test_case '{}' runs in '{named}', which is no configuration", test.name));
            };
            // Said twice and differently, one of the two would be dropped without a word.
            if let Some(simulated) = simulated_configuration(test).filter(|s| configuration_key(model, s).as_ref() != Some(&key)) {
                return Err(format!("test_case '{}' names configuration '{named}' and simulates in '{simulated}': say it once", test.name));
            }
        }
        let configuration = configuration_of(model, test);
        for reference in &test.verifies {
            let (Some(only), Some(runs_in)) = (applies_to(model, reference), &configuration) else { continue };
            if !only.contains(runs_in) {
                let shown = model.configurations.iter().find(|c| &c.id == runs_in).map(|c| c.name.clone()).unwrap_or_else(|| runs_in.clone());
                return Err(format!(
                    "test_case '{}' verifies '{reference}' in configuration {shown}, but '{reference}' applies only to {}",
                    test.name,
                    only.join(", ")
                ));
            }
        }
        // A case that runs (the simulation, the machine in C): its expectations
        // are what judges each requirement it verifies.
        if !super::simulation::is_simulated(test) && !super::embedded::case::is_executed(test) {
            continue;
        }
        let expects = super::simulation::case_expects(model, test);
        let named: Vec<Option<&str>> = expects.iter().map(|e| super::simulation::expects::split_requirement(e).0).collect();
        if named.iter().all(Option::is_none) {
            continue;
        }
        let verified: Vec<String> = test.verifies.iter().map(|r| requirement_key(model, r)).collect();
        if let Some(stray) = named.iter().flatten().find(|n| !verified.contains(&requirement_key(model, n))) {
            return Err(format!("test_case '{}': an expectation checks '{stray}', which the case does not verify (verifies: [{}])", test.name, test.verifies.join(", ")));
        }
        let unchecked = |key: &String| !named.iter().any(|n| n.is_none_or(|n| &requirement_key(model, n) == key));
        if let Some(reference) = test.verifies.iter().find(|r| unchecked(&requirement_key(model, r))) {
            return Err(format!("test_case '{}' verifies '{reference}', but none of its expectations checks it: name it (\"{reference}: ...\") or leave one naming no requirement", test.name));
        }
    }
    Ok(())
}
