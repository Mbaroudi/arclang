//! "The impact of my changes": the model as committed at a git revision
//! compared with the model now, then propagated as a what-if is.

use super::{ImpactReport, Options};
use crate::compiler::semantic_diff;
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git").arg("-C").arg(dir).args(args).output().map_err(|e| format!("git could not be run: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// The text of a model file at a revision (`HEAD`, `main`, a commit).
pub fn text_at(path: &Path, revision: &str) -> Result<String, String> {
    if revision.starts_with('-') || revision.contains(':') || revision.trim().is_empty() {
        return Err(format!("'{revision}' is not a revision"));
    }
    let absolute = std::fs::canonicalize(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let dir = absolute.parent().unwrap_or(Path::new("."));
    let top = git(dir, &["rev-parse", "--show-toplevel"]).map_err(|_| format!("{} is not in a git repository", path.display()))?;
    let top = std::fs::canonicalize(top.trim()).map_err(|e| e.to_string())?;
    let relative = absolute.strip_prefix(&top).map_err(|_| format!("{} is outside its repository", path.display()))?;
    let relative = relative.to_string_lossy().replace('\\', "/");
    git(&top, &["rev-parse", "--verify", "--quiet", &format!("{revision}^{{commit}}")]).map_err(|_| format!("unknown revision '{revision}'"))?;
    git(&top, &["show", &format!("{revision}:{relative}")]).map_err(|_| format!("{relative} does not exist at {revision}"))
}

/// The impact of what changed in a model file since a revision.
pub fn since(path: &Path, current: &str, revision: &str, options: &Options) -> Result<ImpactReport, String> {
    let old_text = text_at(path, revision)?;
    let compile = |text: &str| crate::compiler::source_edit::compile_beside(path, text);
    let before = compile(&old_text).map_err(|e| format!("the model at {revision} does not compile: {e}"))?;
    let after = compile(current).map_err(|e| format!("the model does not compile: {e}"))?;
    let diff = semantic_diff::diff_compiled(&before, &after);
    let mut report = super::whatif::compare(&before, &after, diff, Vec::new(), options)?;
    report.model = path.display().to_string();
    report.since = Some(revision.to_string());
    Ok(report)
}
