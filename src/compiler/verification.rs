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
