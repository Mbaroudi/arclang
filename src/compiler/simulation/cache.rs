//! The verdicts of simulations, kept in `.arclang/sim-cache/` beside the
//! model, one JSON file per case, keyed by what the simulation depends on.

use super::{Outcome, Spec};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Bumped when the meaning of a cached verdict changes.
const FORMAT: &str = "arclang-sim-cache-1";

/// The directory a model lives in: that of its path, or the one named by
/// `ARCLANG_MODEL_DIR` when the model is read from a temporary copy (the VS
/// Code extension computes on the text of an editor, not on the saved file).
pub fn model_dir(model_path: &Path) -> PathBuf {
    match std::env::var_os("ARCLANG_MODEL_DIR") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => model_path.parent().map(Path::to_path_buf).unwrap_or_default(),
    }
}

pub fn dir(model_path: &Path) -> PathBuf {
    model_dir(model_path).join(".arclang").join("sim-cache")
}

fn file(dir: &Path, test_id: &str) -> PathBuf {
    let safe: String = test_id.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
    dir.join(format!("{safe}.json"))
}

/// The hash of what a verdict depends on: the generated package, every
/// source (path and content), the simulation parameters and the
/// expectations. A source that cannot be read hashes as missing.
pub fn key(package: &str, base: &Path, spec: &Spec, expects: &[String]) -> String {
    let mut hash = Sha256::new();
    let mut field = |label: &str, bytes: &[u8]| {
        hash.update(label.as_bytes());
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    };
    field("format", FORMAT.as_bytes());
    field("package", package.as_bytes());
    for source in &spec.sources {
        field("source", source.as_bytes());
        match std::fs::read(base.join(source)) {
            Ok(content) => field("content", &content),
            Err(_) => field("missing", b""),
        }
    }
    field("backend", spec.backend.name().as_bytes());
    field("model", spec.model.as_bytes());
    field("configuration", spec.configuration.as_deref().unwrap_or("").as_bytes());
    field("time", format!("{}..{}", spec.start_time, spec.stop_time).as_bytes());
    field("tolerance", format!("{:?}", spec.tolerance).as_bytes());
    field("intervals", format!("{:?}", spec.intervals).as_bytes());
    for expect in expects {
        field("expects", expect.as_bytes());
    }
    hash.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

pub fn read(dir: &Path, test_id: &str) -> Option<Outcome> {
    let text = std::fs::read_to_string(file(dir, test_id)).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn write(dir: &Path, outcome: &Outcome) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let path = file(dir, &outcome.test_case);
    let text = serde_json::to_string_pretty(outcome).map_err(|e| e.to_string())?;
    std::fs::write(&path, text + "\n").map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(sources: Vec<String>) -> Spec {
        Spec { backend: Default::default(), model: "P.M".into(), configuration: None, start_time: 0.0, stop_time: 10.0, sources, tolerance: None, intervals: None }
    }

    #[test]
    fn the_key_follows_the_package_the_sources_and_the_case() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.mo"), "model A end A;").unwrap();
        let s = spec(vec!["a.mo".into()]);
        let base = key("package P end P;", dir.path(), &s, &["final(x) > 1".into()]);
        assert_eq!(base, key("package P end P;", dir.path(), &s, &["final(x) > 1".into()]), "deterministic");
        assert_ne!(base, key("package P2 end P2;", dir.path(), &s, &["final(x) > 1".into()]));
        assert_ne!(base, key("package P end P;", dir.path(), &s, &["final(x) > 2".into()]));
        assert_ne!(base, key("package P end P;", dir.path(), &Spec { stop_time: 20.0, ..s.clone() }, &["final(x) > 1".into()]));
        std::fs::write(dir.path().join("a.mo"), "model A Real x; end A;").unwrap();
        assert_ne!(base, key("package P end P;", dir.path(), &s, &["final(x) > 1".into()]), "a source changed");
    }

    #[test]
    fn outcomes_are_written_and_read_back() {
        let dir = tempfile::tempdir().unwrap();
        let outcome = Outcome {
            test_case: "TC/1".into(),
            verdict: "pass".into(),
            reason: String::new(),
            values: Vec::new(),
            key: "k".into(),
            model: "P.M".into(),
            configuration: None,
            tool: "omc".into(),
            omc_version: "v1".into(),
            log: String::new(),
        };
        let path = write(dir.path(), &outcome).unwrap();
        assert!(path.ends_with("TC_1.json"));
        assert_eq!(read(dir.path(), "TC/1"), Some(outcome));
        assert_eq!(read(dir.path(), "TC-2"), None);
    }
}
