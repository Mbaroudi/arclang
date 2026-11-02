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
        // Normalize `a/../b` without touching the filesystem.
        let mut normalized = PathBuf::new();
        for component in imported.components() {
            match component {
                std::path::Component::ParentDir => {
                    normalized.pop();
                }
                std::path::Component::CurDir => {}
                other => normalized.push(other.as_os_str()),
            }
        }
        materialize(root, revision, &normalized, destination, done)?;
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

/// The files of the model, relative to the repository: the file itself and
/// every file it imports, as the working tree has them.
pub(super) fn model_files(path: &FsPath, tracked: &Tracked) -> Result<Vec<String>, String> {
    let files = crate::Compiler::files_of(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    let mut relative = Vec::new();
    for file in files {
        let inside = file.strip_prefix(&tracked.root).map_err(|_| format!("{} is outside the git repository of the model", file.display()))?;
        let text = inside.to_string_lossy().replace('\\', "/");
        if !relative.contains(&text) {
            relative.push(text);
        }
    }
    if !relative.contains(&tracked.relative) {
        relative.insert(0, tracked.relative.clone());
    }
    Ok(relative)
}

/// Identity of the API commit standing for git commit `sha` of a file.
pub(super) fn commit_id(key: &str, sha: &str) -> String {
    element_uuid("commit", &format!("git|{}|{}", key, sha))
}

/// One revision of the model in git: a commit that touched one of its files.
pub(super) struct Revision {
    pub sha: String,
    /// Its parents among the commits that touched the model (git's history
    /// simplification), so that a merge keeps both lines.
    pub parents: Vec<String>,
    pub created: DateTime<Utc>,
    pub subject: String,
    pub user: Option<String>,
}

/// The revisions of the model reachable from `revision`, newest first.
pub(super) fn log(root: &FsPath, revision: &str, depth: usize, files: &[String]) -> Result<Vec<Revision>, String> {
    let depth_text = depth.max(1).to_string();
    // One record per commit, ended by a record separator: a note ends
    // with a line break of its own.
    let notes = format!("--notes={}", USER_NOTES);
    let mut arguments: Vec<&str> = vec!["log", revision, "-n", &depth_text, "--full-history", "--parents", &notes, "--format=%H%x1f%P%x1f%cI%x1f%s%x1f%N%x1e", "--"];
    arguments.extend(files.iter().map(String::as_str));
    let log = git(root, &arguments)?;
    let mut revisions = Vec::new();
    for record in log.split('\u{1e}') {
        let mut fields = record.trim_start_matches('\n').split('\u{1f}');
        let (Some(sha), Some(parents), Some(date), Some(subject)) = (fields.next(), fields.next(), fields.next(), fields.next()) else { continue };
        let user = fields.next().map(str::trim).filter(|user| !user.is_empty()).map(str::to_string);
        let created = DateTime::parse_from_rfc3339(date)
            .map_err(|e| format!("git date '{}': {}", date, e))?
            .with_timezone(&Utc);
        revisions.push(Revision {
            sha: sha.to_string(),
            parents: parents.split_whitespace().map(str::to_string).collect(),
            created,
            subject: subject.to_string(),
            user,
        });
    }
    Ok(revisions)
}

/// The model as it is at git commit `sha`, compiled; a revision that does
/// not compile is a commit without elements that says why.
pub(super) fn compile_revision(root: &FsPath, relative: &FsPath, key: &str, revision: &Revision) -> Snapshot {
    // Unique per call: identical revisions may be materialized
    // concurrently (several models, several servers, tests).
    static SCRATCH_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let sha = &revision.sha;
    let scratch = std::env::temp_dir().join(format!(
        "arclang-history-{}-{}-{}",
        std::process::id(),
        SCRATCH_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        &sha[..12.min(sha.len())]
    ));
    let compiled = materialize(root, sha, relative, &scratch, &mut Vec::new()).and_then(|()| {
        crate::Compiler::new(crate::CompilerConfig::default())
            .compile_file(scratch.join(relative))
            .map_err(|e| e.to_string())
    });
    let _ = std::fs::remove_dir_all(&scratch);
    let mut snapshot = match compiled {
        Ok(result) => Snapshot::from_compilation(&result, revision.created),
        // Scratch paths would leak into the message; keep the cause.
        Err(reason) => Snapshot::failed(reason.replace(&scratch.to_string_lossy().into_owned(), "."), revision.created),
    };
    // In a history the commit IS the git commit: two revisions with the
    // same content are still two commits.
    snapshot.id = commit_id(key, sha);
    snapshot.description = revision.subject.clone();
    snapshot.git_commit = Some(sha.clone());
    snapshot.user = revision.user.clone();
    snapshot.parents = revision.parents.iter().map(|parent| commit_id(key, parent)).collect();
    snapshot
}

/// The local branches of the repository and the commit each points at.
pub(super) fn branches(root: &FsPath) -> Result<Vec<(String, String)>, String> {
    let listing = git(root, &["for-each-ref", "--format=%(refname:short)%1f%(objectname)", "refs/heads"])?;
    Ok(listing
        .lines()
        .filter_map(|line| line.split_once('\u{1f}'))
        .map(|(name, sha)| (name.to_string(), sha.to_string()))
        .collect())
}

/// The branch checked out in the working tree, if any.
pub(super) fn checked_out(root: &FsPath) -> Option<String> {
    git(root, &["symbolic-ref", "--quiet", "--short", "HEAD"]).ok().map(|name| name.trim().to_string()).filter(|name| !name.is_empty())
}

/// A tag of the repository: name, the commit it points at, when it was made, its message.
pub(super) struct GitTag {
    pub name: String,
    pub sha: String,
    pub created: DateTime<Utc>,
    pub message: String,
}

pub(super) fn tags(root: &FsPath) -> Result<Vec<GitTag>, String> {
    // `%(*objectname)` is the commit an annotated tag points at; empty for a lightweight tag.
    let listing = git(
        root,
        &["for-each-ref", "--format=%(refname:short)%1f%(*objectname)%1f%(objectname)%1f%(creatordate:iso-strict)%1f%(contents:subject)%1e", "refs/tags"],
    )?;
    let mut tags = Vec::new();
    for record in listing.split('\u{1e}') {
        let fields: Vec<&str> = record.trim_start_matches('\n').split('\u{1f}').collect();
        let [name, peeled, object, date, message] = fields.as_slice() else { continue };
        let created = DateTime::parse_from_rfc3339(date).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now());
        let sha = if peeled.is_empty() { object } else { peeled };
        tags.push(GitTag { name: name.to_string(), sha: sha.to_string(), created, message: message.to_string() });
    }
    Ok(tags)
}

/// The last commit at or before `revision` that touched the model: the
/// state of the model a tag or a branch on an unrelated commit stands for.
pub(super) fn model_commit_at(root: &FsPath, revision: &str, files: &[String]) -> Result<Option<String>, String> {
    let mut arguments: Vec<&str> = vec!["log", "-1", "--format=%H", revision, "--"];
    arguments.extend(files.iter().map(String::as_str));
    Ok(Some(git(root, &arguments)?.trim().to_string()).filter(|sha| !sha.is_empty()))
}

/// A checkout of `branch` in a temporary directory (`git worktree`), so
/// that a branch can be written without touching the user's working tree.
/// Removed when dropped.
pub(super) struct Worktree {
    repository: PathBuf,
    pub path: PathBuf,
}

impl Worktree {
    pub(super) fn add(repository: &FsPath, branch: &str) -> Result<Worktree, String> {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "arclang-branch-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let path_text = path.to_string_lossy().into_owned();
        git(repository, &["worktree", "add", "--quiet", &path_text, branch])?;
        Ok(Worktree { repository: repository.to_path_buf(), path })
    }
}

impl Drop for Worktree {
    fn drop(&mut self) {
        let path_text = self.path.to_string_lossy().into_owned();
        if git(&self.repository, &["worktree", "remove", "--force", &path_text]).is_err() {
            let _ = std::fs::remove_dir_all(&self.path);
            let _ = git(&self.repository, &["worktree", "prune"]);
        }
    }
}

/// A branch or tag name git accepts (`git check-ref-format`), and no more.
pub(super) fn valid_ref_name(kind: &str, name: &str) -> Result<(), String> {
    let reference = format!("refs/{}/{}", kind, name);
    let accepted = Command::new("git").args(["check-ref-format", &reference]).output().map(|output| output.status.success()).unwrap_or(false);
    if name.is_empty() || name.starts_with('-') || !accepted {
        return Err(format!("'{}' is not a valid {} name", name, if kind == "heads" { "branch" } else { "tag" }));
    }
    Ok(())
}
