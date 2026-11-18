//! Evidence obtained outside the compiler.
//!
//! A requirement is verified when something checked it. The compiler checks
//! what it can run itself: constraints, simulations, the generated code.
//! Other tools check other things: a circuit simulator, a structural solver,
//! a test bench. A case that names where such a verdict is kept takes it:
//!
//! ```text
//! test_case "Arm strength" { id: "TC-ARM" verifies: ["REQ-ARM"] method: "analysis" evidence: "engineering" }
//! ```
//!
//! The tool writes `.arclang/evidence/<case id>.json` beside the model:
//!
//! ```json
//! { "case": "TC-ARM", "tool": "…", "verdict": "pass", "reason": "", "evidence": "stress 39 MPa < 50 MPa",
//!   "subjects": [{ "element": "PN-ARM", "fingerprint": "1a2b3c4d" }],
//!   "inputs": [{ "path": "bracket.py", "sha256": "…" }] }
//! ```
//!
//! The compiler does not trust it further than it can check. The record says
//! what it was obtained on: elements of the model, each with its fingerprint
//! (the one a reviewed link is kept under, [`super::suspect::fingerprint`]),
//! and files beside the model, each with its SHA-256. On every compilation
//! each is compared with what is there now: an element that changed, a file
//! that changed or is gone, and the verdict is stale. A record that names
//! nothing it was obtained on could never go stale, and is refused.
//!
//! Stale, missing, unreadable, or written for another case: inconclusive,
//! with the reason. Never a pass.

use super::ast::TestCase;
use super::elements::ElementGraph;
use super::simulation::Status;
use super::CompilationResult;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

/// The name a case gives the evidence it takes (`evidence: "engineering"`): free text, for the reader.
pub fn named(test: &TestCase) -> Option<&str> {
    test.attributes.get("evidence").and_then(|v| v.as_string()).filter(|name| !name.trim().is_empty())
}

/// Whether a case takes its verdict from evidence obtained outside: it says
/// `evidence:`, and is not one the compiler runs or computes itself.
pub fn is_external(test: &TestCase) -> bool {
    named(test).is_some() && !super::simulation::is_simulated(test) && !super::embedded::case::is_executed(test) && super::verification::asserted(test).is_empty()
}

/// Where the evidence of a model's cases is kept.
pub fn dir(model_path: &Path) -> PathBuf {
    model_path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new(".")).join(".arclang").join("evidence")
}

/// The file of one case: its id, with what a file name cannot hold replaced.
pub fn file(model_path: &Path, case_id: &str) -> PathBuf {
    let safe: String = case_id.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '_' }).collect();
    dir(model_path).join(format!("{safe}.json"))
}

#[derive(Debug, Deserialize)]
struct Record {
    case: String,
    #[serde(default)]
    tool: String,
    verdict: String,
    #[serde(default)]
    reason: String,
    #[serde(default)]
    evidence: String,
    #[serde(default)]
    subjects: Vec<Subject>,
    #[serde(default)]
    inputs: Vec<Input>,
}

#[derive(Debug, Deserialize)]
struct Subject {
    element: String,
    fingerprint: String,
}

#[derive(Debug, Deserialize)]
struct Input {
    path: String,
    sha256: String,
}

fn sha256_of(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    Some(Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect())
}

/// Why a record does not stand for the model as it is now; `None` when it does.
fn stale(record: &Record, graph: &ElementGraph, base: &Path) -> Option<String> {
    for subject in &record.subjects {
        let found: Vec<_> = graph.elements.iter().filter(|e| e.id == subject.element).collect();
        let element = match found.as_slice() {
            [one] => *one,
            [] => return Some(format!("element '{}' it was obtained on is no longer in the model", subject.element)),
            _ => return Some(format!("'{}' names several elements: evidence names each by an id only one bears", subject.element)),
        };
        if super::suspect::fingerprint(element) != subject.fingerprint {
            return Some(format!("element '{}' changed since it was obtained", subject.element));
        }
    }
    for input in &record.inputs {
        let path = Path::new(&input.path);
        // A file beside the model or below it, as a source of a correspondence is.
        if path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
            return Some(format!("it names a file outside the model's directory ('{}')", input.path));
        }
        match sha256_of(&base.join(path)) {
            None => return Some(format!("file '{}' it was obtained on is not there", input.path)),
            Some(now) if !now.eq_ignore_ascii_case(&input.sha256) => return Some(format!("file '{}' changed since it was obtained", input.path)),
            Some(_) => {}
        }
    }
    None
}

/// The status of every case that takes evidence obtained outside, as this
/// compilation sees it. Nothing is run here.
pub fn attach(result: &CompilationResult, model_path: &Path) -> HashMap<String, Status> {
    let cases: Vec<&TestCase> = result.ast.test_cases.iter().filter(|t| is_external(t)).collect();
    if cases.is_empty() {
        return HashMap::new();
    }
    let graph = super::elements::build(&result.ast, &result.semantic_model);
    let base = model_path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let inconclusive = |reason: String| Status::new("inconclusive", reason, "");
    cases
        .into_iter()
        .map(|test| {
            let path = file(model_path, &test.id);
            let status = match std::fs::read_to_string(&path) {
                Err(_) => inconclusive(format!("no evidence recorded: the tool that gives '{}' writes {}", named(test).unwrap_or_default(), path.display())),
                Ok(text) => match serde_json::from_str::<Record>(&text) {
                    Err(error) => inconclusive(format!("the evidence in {} does not read: {error}", path.display())),
                    Ok(record) if record.case != test.id => inconclusive(format!("the evidence in {} is that of case '{}'", path.display(), record.case)),
                    Ok(record) if record.subjects.is_empty() && record.inputs.is_empty() => inconclusive("the evidence names nothing it was obtained on (no subject, no input): it could never go stale, so it is not taken".to_string()),
                    Ok(record) if !super::verification::VERDICTS.contains(&record.verdict.as_str()) => inconclusive(format!("the evidence gives '{}', which is not a verdict (pass, fail, inconclusive, error)", record.verdict)),
                    Ok(record) => match stale(&record, &graph, base) {
                        Some(why) => inconclusive(format!("stale: {why}; obtain it again with {}", if record.tool.is_empty() { "the tool that gave it" } else { record.tool.as_str() })),
                        None => Status::new(record.verdict.clone(), record.reason.clone(), if record.tool.is_empty() { record.evidence.clone() } else { format!("{} ({})", record.evidence, record.tool) }),
                    },
                },
            };
            (test.id.clone(), status)
        })
        .collect()
}
