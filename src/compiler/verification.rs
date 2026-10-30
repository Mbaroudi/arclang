//! Verdicts of verification cases. A `test_case` that `asserts:`
//! constraints is an analysis: its verdict is theirs, pass when they all
//! hold, fail otherwise. Any other case may say the verdict of a test run
//! elsewhere (`verdict: "fail"`), with the SysML v2 verdicts; a case saying
//! none is pending. A case that `simulate:`s takes its verdict from the
//! cached simulation (`compiler::simulation`): inconclusive when it was
//! never run, or was run on another version of the model. A requirement
//! with a failing or inconclusive verification case is not verified, for
//! the production gate, even when another case passes.

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

/// The id a reference to a requirement names (its `id:`, or its name when
/// it has none), or the reference itself when it names no requirement.
pub fn requirement_key(model: &Model, reference: &str) -> String {
    requirements(model)
        .find(|(_, name, written_id)| written_id == reference || *name == reference)
        .map(|(_, _, written_id)| written_id)
        .unwrap_or_else(|| reference.to_string())
}

/// Every requirement: its block, its name, its id.
fn requirements(model: &Model) -> impl Iterator<Item = (&super::ast::Requirement, &str, String)> {
    model.system_analysis.iter().flat_map(|sa| sa.requirements.iter()).map(|r| {
        let id = r.attributes.get("id").and_then(|v| v.as_string()).map(str::to_string).unwrap_or_else(|| r.id.clone());
        (r, r.id.as_str(), id)
    })
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

/// The configuration a case runs in (the one it simulates), by id; `None`
/// when it names none, and so stands for every configuration.
pub fn configuration_of(model: &Model, test: &TestCase) -> Option<String> {
    let spec = super::simulation::spec_of(&test.attributes)?.ok()?;
    let written = spec.configuration?;
    Some(configuration_key(model, &written).unwrap_or(written))
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
    if super::simulation::is_simulated(test) {
        return match semantic.simulations.get(&test.id) {
            None => Verdict::Inconclusive("no simulation verdict: run `arclang verify` on the model file".to_string()),
            Some(status) => match status.verdict.as_str() {
                "pass" => Verdict::Pass,
                "fail" => Verdict::Fail(status.reason.clone()),
                "error" => Verdict::Fail(format!("the simulation is in error: {}", status.reason)),
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

/// An asserted constraint that is none is an error; a failing case, a
/// `verification:` warning.
pub fn check(model: &Model, semantic: &SemanticModel) -> Result<Vec<String>, String> {
    let mut warnings = Vec::new();
    super::simulation::check(model)?;
    check_applicability(model)?;
    for test in &model.test_cases {
        if let Some(unknown) = asserted(test).into_iter().find(|a| constraint(semantic, a).is_none()) {
            return Err(format!("test_case '{}' asserts '{unknown}', which is no constraint", test.name));
        }
        // A simulated case is reported with its cached verdict, on the file.
        if super::simulation::is_simulated(test) {
            continue;
        }
        if let Verdict::Fail(reason) = verdict(test, semantic) {
            warnings.push(format!("verification: test_case '{}' fails: {reason}", test.name));
        }
    }
    Ok(warnings)
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
        if !super::simulation::is_simulated(test) {
            continue;
        }
        let expects = super::simulation::expects_of(&test.attributes);
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
