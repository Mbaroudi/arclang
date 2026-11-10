//! The verdicts of simulations, kept in `.arclang/sim-cache/` beside the
//! model, one JSON file per case, keyed by what the simulation depends on.

use super::{Outcome, Spec};
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Bumped when the meaning of a cached verdict changes.
const FORMAT: &str = "arclang-sim-cache-2";

thread_local! {
    /// Files given in place of the file system (the browser: the workbench
    /// snapshot reads the verdicts and the Modelica sources the page holds,
    /// by path relative to the model), for the span of `with_files`.
    static FILES: RefCell<Option<HashMap<PathBuf, Vec<u8>>>> = const { RefCell::new(None) };
}

/// Runs `f` with `files` standing for the file system in every read of
/// this module (the verdicts, the sources a key hashes): what the browser
/// knows of the model's folder, where there is no file system.
pub fn with_files<R>(files: HashMap<PathBuf, Vec<u8>>, f: impl FnOnce() -> R) -> R {
    FILES.with(|cell| *cell.borrow_mut() = Some(files));
    let out = f();
    FILES.with(|cell| *cell.borrow_mut() = None);
    out
}

/// The bytes of a file: of those given to `with_files` when inside it, else of the file system.
pub fn read_bytes(path: &Path) -> Option<Vec<u8>> {
    let given = FILES.with(|cell| cell.borrow().as_ref().map(|files| files.get(path).cloned()));
    match given {
        Some(found) => found,
        None => std::fs::read(path).ok(),
    }
}

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
        match read_bytes(&base.join(source)) {
            Some(content) => field("content", &content),
            None => field("missing", b""),
        }
    }
    field("backend", spec.backend.name().as_bytes());
    field("model", spec.model.as_bytes());
    field("configuration", spec.configuration.as_deref().unwrap_or("").as_bytes());
    field("time", format!("{}..{}", spec.start_time, spec.stop_time).as_bytes());
    field("tolerance", format!("{:?}", spec.tolerance).as_bytes());
    field("intervals", format!("{:?}", spec.intervals).as_bytes());
    for axis in &spec.sweep {
        field("sweep", axis.as_bytes());
    }
    for parameter in &spec.uncertain {
        field("uncertain", parameter.as_bytes());
    }
    if let Some(required) = &spec.required {
        field("samples", format!("{} seed {} p {} c {}", spec.samples, spec.seed, required.probability, required.confidence).as_bytes());
    }
    for expect in expects {
        field("expects", expect.as_bytes());
    }
    hash.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// The verdict of another kind of run of the case (`pil`), kept beside its
/// simulation or SIL verdict as `<case>.<kind>.json`.
fn kind_file(dir: &Path, test_id: &str, kind: &str) -> PathBuf {
    let base = file(dir, test_id);
    base.with_extension(format!("{kind}.json"))
}

pub fn read_kind(dir: &Path, test_id: &str, kind: &str) -> Option<Outcome> {
    serde_json::from_slice(&read_bytes(&kind_file(dir, test_id, kind))?).ok()
}

pub fn write_kind(dir: &Path, kind: &str, outcome: &Outcome) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let path = kind_file(dir, &outcome.test_case, kind);
    let text = serde_json::to_string_pretty(outcome).map_err(|e| e.to_string())?;
    std::fs::write(&path, text + "\n").map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(path)
}

pub fn read(dir: &Path, test_id: &str) -> Option<Outcome> {
    serde_json::from_slice(&read_bytes(&file(dir, test_id))?).ok()
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
        Spec { model: "P.M".into(), stop_time: 10.0, sources, ..Default::default() }
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
            ..Default::default()
        };
        let path = write(dir.path(), &outcome).unwrap();
        assert!(path.ends_with("TC_1.json"));
        assert_eq!(read(dir.path(), "TC/1"), Some(outcome));
        assert_eq!(read(dir.path(), "TC-2"), None);
    }
}
