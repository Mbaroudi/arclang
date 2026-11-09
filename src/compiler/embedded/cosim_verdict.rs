//! The verdicts of co-simulations (`arclang cosim`, see `cosim`), kept
//! beside the model as `.arclang/sim-cache/<case>.cosim.json` and read by
//! `check`, the gate, the tables and the workbench as evidence of the SIL
//! stage: the same generated C, run as an FMU under OMSimulator and judged
//! as `arclang sil` judges it. Nothing here runs anything, so the browser's
//! compiler reads the verdicts too.

use super::sil::{self, Outcome};
use crate::compiler::ast::{AttributeValue, Model, TestCase};
use std::path::{Path, PathBuf};

/// Bumped when the meaning of a cached co-simulation verdict changes.
const CACHE_FORMAT: &str = "arclang-cosim-cache-1";

/// The files of the plant a case closes the loop on, as written in its
/// `execute: { plant: { sources: [...] } }` or `fmu:`, relative to the model.
fn plant_files(test: &TestCase) -> Vec<String> {
    let Some(AttributeValue::Map(execute)) = test.attributes.get("execute") else { return Vec::new() };
    let Some(AttributeValue::Map(plant)) = execute.get("plant") else { return Vec::new() };
    let mut files = Vec::new();
    match plant.get("sources") {
        Some(AttributeValue::List(items)) => files.extend(items.iter().filter_map(|i| i.as_string().map(str::to_string))),
        Some(AttributeValue::String(one)) => files.push(one.clone()),
        _ => {}
    }
    if let Some(fmu) = plant.get("fmu").and_then(|v| v.as_string()) {
        files.push(fmu.to_string());
    }
    files
}

/// The hash of what the co-simulation verdict of a case depends on: what
/// its SIL verdict depends on (the generated C, the case, its expectations)
/// and the plant it closes the loop on (its Modelica sources or its FMU,
/// read beside the model; one that cannot be read hashes as missing).
pub fn cache_key(ast: &Model, test: &TestCase, base: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let sil = sil::cache_key(ast, test)?;
    let mut hash = Sha256::new();
    let mut field = |label: &str, bytes: &[u8]| {
        hash.update(label.as_bytes());
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    };
    field("format", CACHE_FORMAT.as_bytes());
    field("sil", sil.as_bytes());
    for file in plant_files(test) {
        field("plant", file.as_bytes());
        match crate::compiler::simulation::cache::read_bytes(&base.join(&file)) {
            Some(content) => field("content", &content),
            None => field("missing", b""),
        }
    }
    Ok(hash.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// Keep the verdicts of the cases co-simulated in the model's cache
/// (`.arclang/sim-cache/<case>.cosim.json`), where `check`, `gate` and the
/// tables read them as evidence of the SIL stage. A case skipped (through
/// an io_binding) leaves no verdict.
pub fn cache(ast: &Model, model_path: &Path, outcomes: &[Outcome]) -> Result<Vec<PathBuf>, String> {
    let dir = crate::compiler::simulation::cache::dir(model_path);
    let base = crate::compiler::simulation::cache::model_dir(model_path);
    let mut written = Vec::new();
    for test in sil::cases(ast) {
        let runs: Vec<&Outcome> = outcomes.iter().filter(|o| o.test_case == test.id && o.verdict != "skipped").collect();
        if runs.is_empty() {
            continue;
        }
        let mut outcome = sil::cached_outcome(&runs, cache_key(ast, test, &base)?);
        outcome.kind = "cosim".to_string();
        outcome.tool = format!("arclang cosim ({})", runs.iter().map(|o| o.compiler.as_str()).collect::<Vec<_>>().join(", "));
        if let Some(rest) = outcome.reason.strip_prefix("SIL with ") {
            outcome.reason = format!("co-simulation with {rest}");
        }
        written.push(crate::compiler::simulation::cache::write_kind(&dir, "cosim", &outcome)?);
    }
    Ok(written)
}

/// What ran a cached co-simulation verdict (`OMSimulator v2.1.2`), from its tool.
pub fn target_of(outcome: &crate::compiler::simulation::Outcome) -> String {
    outcome.tool.trim_start_matches("arclang cosim").trim().trim_start_matches('(').trim_end_matches(')').to_string()
}
