//! Git history of a model file, compiled revision by revision.

use super::*;
use std::path::PathBuf;
use std::process::Command;

pub(super) fn git(directory: &FsPath, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(directory)
        .args(args)
        .output()
        .map_err(|e| format!("cannot run git: {}", e))?;
    if !output.status.success() {
        return Err(format!("git {}: {}", args.join(" "), String::from_utf8_lossy(&output.stderr).trim()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Write `relative` as it was at `revision`, and every file it imports
/// (transitively), under `destination`, keeping relative paths.
fn materialize(root: &FsPath, revision: &str, relative: &FsPath, destination: &FsPath, done: &mut Vec<PathBuf>) -> Result<(), String> {
    if done.iter().any(|p| p == relative) {
        return Ok(());
    }
    done.push(relative.to_path_buf());
    let spec = format!("{}:{}", revision, relative.to_string_lossy().replace('\\', "/"));
    let content = git(root, &["show", &spec])?;
    let target = destination.join(relative);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&target, &content).map_err(|e| e.to_string())?;
    for line in content.lines() {
        let line = line.trim_start();
        let Some(rest) = line.strip_prefix("import") else { continue };
        let rest = rest.trim_start();
        let Some(rest) = rest.strip_prefix('"') else { continue };
        let Some(end) = rest.find('"') else { continue };
        let imported = relative.parent().unwrap_or_else(|| FsPath::new("")).join(&rest[..end]);
        // Normalise `a/../b` without touching the filesystem.
        let mut normalised = PathBuf::new();
        for component in imported.components() {
            match component {
                std::path::Component::ParentDir => {
                    normalised.pop();
                }
                std::path::Component::CurDir => {}
                other => normalised.push(other.as_os_str()),
            }
        }
        materialize(root, revision, &normalised, destination, done)?;
    }
    Ok(())
}

/// Git notes ref holding, for a commit made through the API by a named
/// user, the name of that user (`git log --notes=arclang-user`).
pub(super) const USER_NOTES: &str = "arclang-user";

/// The file's place in its git repository: the working directory to run git
/// in, and the path of the file relative to it.
pub(super) struct Tracked {
    pub root: PathBuf,
    pub relative: String,
}

pub(super) fn locate(path: &FsPath) -> Result<Tracked, String> {
    let absolute = path.canonicalize().map_err(|e| format!("{}: {}", path.display(), e))?;
    let directory = absolute.parent().ok_or("model file has no parent directory")?;
    let root = PathBuf::from(
        git(directory, &["rev-parse", "--show-toplevel"])
            .map_err(|e| format!("{}: not in a git repository ({})", path.display(), e))?
            .trim(),
    );
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let relative = absolute
        .strip_prefix(&root)
        .map_err(|_| format!("{} is outside its git repository", path.display()))?
        .to_string_lossy()
        .replace('\\', "/");
    Ok(Tracked { root, relative })
}

/// Identity of the API commit standing for git commit `sha` of a file.
pub(super) fn commit_id(key: &str, sha: &str) -> String {
    element_uuid("commit", &format!("git|{}|{}", key, sha))
}

/// The file's history key and its snapshots, oldest first.
pub(super) fn snapshots(path: &FsPath, depth: usize) -> Result<(String, Vec<Snapshot>), String> {
    let Tracked { root, relative: relative_text } = locate(path)?;
    let relative = PathBuf::from(&relative_text);

    let depth_text = depth.max(1).to_string();
    // One record per commit, ended by a record separator: a note ends
    // with a line break of its own.
    let notes = format!("--notes={}", USER_NOTES);
    let log = git(&root, &["log", "-n", &depth_text, &notes, "--format=%H%x1f%cI%x1f%s%x1f%N%x1e", "--", &relative_text])?;
    let mut revisions: Vec<(String, DateTime<Utc>, String, Option<String>)> = Vec::new();
    for record in log.split('\u{1e}') {
        let mut fields = record.trim_start_matches('\n').split('\u{1f}');
        let (Some(sha), Some(date), Some(subject)) = (fields.next(), fields.next(), fields.next()) else { continue };
        let user = fields.next().map(str::trim).filter(|user| !user.is_empty()).map(str::to_string);
        let created = DateTime::parse_from_rfc3339(date)
            .map_err(|e| format!("git date '{}': {}", date, e))?
            .with_timezone(&Utc);
        revisions.push((sha.to_string(), created, subject.to_string(), user));
    }
    revisions.reverse(); // oldest first

    let project_key = relative_text.clone();
    let mut snapshots = Vec::with_capacity(revisions.len());
    for (sha, created, subject, user) in revisions {
        // Unique per call: identical revisions may be materialised
        // concurrently (several models, several servers, tests).
        static SCRATCH_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let scratch = std::env::temp_dir().join(format!(
            "arclang-history-{}-{}-{}",
            std::process::id(),
            SCRATCH_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            &sha[..12.min(sha.len())]
        ));
        let compiled = materialize(&root, &sha, &relative, &scratch, &mut Vec::new()).and_then(|()| {
            crate::Compiler::new(crate::CompilerConfig::default())
                .compile_file(scratch.join(&relative))
                .map_err(|e| e.to_string())
        });
        let _ = std::fs::remove_dir_all(&scratch);
        let mut snapshot = match compiled {
            Ok(result) => Snapshot::from_compilation(&result, created),
            // Scratch paths would leak into the message; keep the cause.
            Err(reason) => Snapshot::failed(reason.replace(&scratch.to_string_lossy().into_owned(), "."), created),
        };
        // In a history the commit IS the git commit: two revisions with
        // the same content are still two commits.
        snapshot.id = commit_id(&project_key, &sha);
        snapshot.description = subject;
        snapshot.git_commit = Some(sha);
        snapshot.user = user;
        snapshots.push(snapshot);
    }
    Ok((project_key, snapshots))
}
