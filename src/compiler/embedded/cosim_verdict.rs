//! The verdicts of co-simulations (`arclang cosim`, see `cosim`), kept
//! beside the model as `.arclang/sim-cache/<case>.cosim.json` and read by
//! `check`, the gate, the tables and the workbench as evidence of the SIL
//! stage: the same generated C, run as an FMU under OMSimulator and judged
//! as `arclang sil` judges it. Nothing here runs anything, so the browser's
//! compiler reads the verdicts too.

use super::sil::{self, Outcome};
use crate::compiler::ast::{Model, TestCase};
use std::path::{Path, PathBuf};

/// Bumped when the meaning of a cached co-simulation verdict changes.
const CACHE_FORMAT: &str = "arclang-cosim-cache-1";

/// The hash of what the co-simulation verdict of a case depends on: what
/// its SIL verdict depends on (the generated C, the case, its expectations,
/// the plant it closes the loop on, read in `base`).
pub fn cache_key(ast: &Model, test: &TestCase, base: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let sil = sil::cache_key(ast, test, base)?;
    let mut hash = Sha256::new();
    let mut field = |label: &str, bytes: &[u8]| {
        hash.update(label.as_bytes());
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    };
    field("format", CACHE_FORMAT.as_bytes());
    field("sil", sil.as_bytes());
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
