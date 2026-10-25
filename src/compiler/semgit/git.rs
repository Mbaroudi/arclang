//! The little of git that semantic versioning needs: run a command, find the
//! repository of a file, name a revision, read or export a tree at a revision.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

/// Run git in `dir`; the output on success, git's own words on failure.
pub fn run(dir: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .map_err(|e| format!("git could not be run: {e}"))?;
    if !output.status.success() {
        let mut message = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if message.is_empty() {
            message = String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
        return Err(message);
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// A revision as a user types it must not be taken for an option or a path.
pub fn check_revision(revision: &str) -> Result<(), String> {
    if revision.starts_with('-') || revision.contains(':') || revision.trim().is_empty() || revision.contains('\0') {
        return Err(format!("'{revision}' is not a revision"));
    }
    Ok(())
}

/// The repository a model file belongs to, and the file's path inside it.
#[derive(Debug, Clone)]
pub struct Located {
    pub top: PathBuf,
    /// Path of the file relative to the top, with `/` separators.
    pub relative: String,
}

pub fn locate(path: &Path) -> Result<Located, String> {
    let absolute = std::fs::canonicalize(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let dir = absolute.parent().unwrap_or(Path::new("."));
    let top = run(dir, &["rev-parse", "--show-toplevel"]).map_err(|_| format!("{} is not in a git repository", path.display()))?;
    let top = std::fs::canonicalize(top.trim()).map_err(|e| e.to_string())?;
    let relative = absolute.strip_prefix(&top).map_err(|_| format!("{} is outside its repository", path.display()))?;
    Ok(Located { top, relative: relative.to_string_lossy().replace('\\', "/") })
}

/// The commit a revision names (full hash).
pub fn commit_of(top: &Path, revision: &str) -> Result<String, String> {
    check_revision(revision)?;
    run(top, &["rev-parse", "--verify", "--quiet", &format!("{revision}^{{commit}}")])
        .map(|s| s.trim().to_string())
        .map_err(|_| format!("unknown revision '{revision}'"))
}

/// The text of a file at a revision, or `None` when the revision has no such file.
pub fn show(top: &Path, revision: &str, relative: &str) -> Option<String> {
    run(top, &["show", &format!("{revision}:{relative}")]).ok()
}

/// A scratch directory removed when dropped.
pub struct Scratch(pub PathBuf);

static COUNTER: AtomicU64 = AtomicU64::new(0);

impl Scratch {
    pub fn new(label: &str) -> Result<Scratch, String> {
        let path = std::env::temp_dir().join(format!("arclang-{label}-{}-{}", std::process::id(), COUNTER.fetch_add(1, Ordering::Relaxed)));
        std::fs::create_dir_all(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(Scratch(path))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Export the tree of a commit into `dest` (what `git archive` writes), so
/// that the model compiles there with its imports as they were.
pub fn export_tree(top: &Path, commit: &str, dest: &Path) -> Result<(), String> {
    let mut archive = Command::new("git")
        .arg("-C")
        .arg(top)
        .args(["archive", "--format=tar", commit])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("git could not be run: {e}"))?;
    let stdout = archive.stdout.take().ok_or("git archive has no output")?;
    let extract = Command::new("tar")
        .arg("-x")
        .arg("-C")
        .arg(dest)
        .stdin(stdout)
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("tar could not be run: {e}"))?;
    let finished = archive.wait_with_output().map_err(|e| e.to_string())?;
    if !finished.status.success() {
        return Err(String::from_utf8_lossy(&finished.stderr).trim().to_string());
    }
    if !extract.status.success() {
        return Err(String::from_utf8_lossy(&extract.stderr).trim().to_string());
    }
    Ok(())
}

/// One commit as `git log` tells it.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Commit {
    pub hash: String,
    pub short: String,
    pub author: String,
    pub date: String,
    pub subject: String,
}

/// The commits that touched a file (following renames), newest first.
pub fn file_log(top: &Path, relative: &str, revision: &str, limit: Option<usize>) -> Result<Vec<Commit>, String> {
    check_revision(revision)?;
    let mut args: Vec<String> = vec!["log".into(), "--follow".into(), "--format=%H%x1f%h%x1f%an%x1f%aI%x1f%s".into()];
    if let Some(n) = limit {
        args.push(format!("-n{n}"));
    }
    args.push(revision.into());
    args.push("--".into());
    args.push(relative.into());
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let out = run(top, &refs)?;
    Ok(out
        .lines()
        .filter_map(|line| {
            let mut parts = line.split('\u{1f}');
            Some(Commit {
                hash: parts.next()?.to_string(),
                short: parts.next()?.to_string(),
                author: parts.next()?.to_string(),
                date: parts.next()?.to_string(),
                subject: parts.next().unwrap_or("").to_string(),
            })
        })
        .collect())
}

/// The layout file next to a model file: `a/m.arc` -> `a/m.layout.json`.
pub fn sidecar_relative(relative: &str) -> String {
    match relative.rsplit_once('.') {
        Some((stem, _)) => format!("{stem}.layout.json"),
        None => format!("{relative}.layout.json"),
    }
}
