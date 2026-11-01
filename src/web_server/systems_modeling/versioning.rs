//! Branches, snapshots (tags), merges and restores through the API, as git
//! operations on the model's repository: a branch is a git branch, a
//! snapshot a git tag, a merge a merge commit with both parents, a restore
//! a new commit. History is never rewritten.
//!
//! The model files of a merge (`.arc`, `.layout.json`) are merged by element
//! identity (`semgit::merge`): two branches that changed different elements,
//! or different attributes of one, merge without a conflict; a real conflict
//! is answered with the elements in dispute, and the request is sent again
//! with a choice for each.

use super::write::writer;
use super::*;
use crate::compiler::semgit::merge::{layout, merge_arc, resolve_conflicts, restore_element, Side};

fn refused(description: String) -> Response {
    error(StatusCode::UNPROCESSABLE_ENTITY, description)
}

fn conflict(description: String) -> Response {
    error(StatusCode::CONFLICT, description)
}

/// The repository of a project served with history.
fn repository(project: &Project) -> Result<history::Tracked, Response> {
    let path = project.source.as_ref().ok_or_else(|| refused("this project is not served from a file".to_string()))?;
    if project.history_key.is_none() {
        return Err(refused("branches, snapshots and merges need the model served with its history (`arclang serve --history`)".to_string()));
    }
    history::locate(path).map_err(refused)
}

fn branch_named<'p>(project: &'p Project, body: &Value, key: &str) -> Result<&'p BranchState, Response> {
    let id = body[key]["@id"].as_str().or_else(|| body[key].as_str());
    let name = body[format!("{key}Name")].as_str();
    project
        .branches
        .iter()
        .find(|branch| Some(branch.id.as_str()) == id || Some(branch.name.as_str()) == name)
        .ok_or_else(|| not_found(format!("no branch given as '{}' (an {{\"@id\"}} or '{}Name')", key, key)))
}

fn git_sha(project: &Project, commit_id: &str) -> Result<String, Response> {
    let position = project.position(commit_id).ok_or_else(|| not_found(format!("commit '{}' not found", commit_id)))?;
    project.commits[position].git_commit.clone().ok_or_else(|| refused(format!("commit '{}' is not in git (the uncommitted working tree)", commit_id)))
}

/// The JSON of the commit git now names `sha`, after a reload.
fn commit_created(workspace: &mut Workspace, project_id: &str, sha: &str) -> Result<Response, Response> {
    workspace.reload(project_id).map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let project = workspace.project(project_id)?;
    let id = history::commit_id(project.history_key.as_deref().unwrap_or_default(), sha);
    let position = project.position(&id).ok_or_else(|| error(StatusCode::INTERNAL_SERVER_ERROR, "the new commit is not in the history".to_string()))?;
    Ok((StatusCode::CREATED, Json(project.commit_json(position, true))).into_response())
}

/// The user a commit is made for, recorded as the API does for its writes.
fn note_user(root: &FsPath, sha: &str, principal: &Principal) {
    if principal.name != access::SHARED_TOKEN_USER {
        let _ = history::git(root, &["notes", "--ref", history::USER_NOTES, "add", "-f", "-m", &principal.name, sha]);
    }
}

/// Where a branch is written: the working tree when it is checked out
/// (and clean), else a temporary checkout.
struct Tree {
    root: PathBuf,
    _worktree: Option<history::Worktree>,
}

fn tree_of(tracked: &history::Tracked, branch: &str) -> Result<Tree, Response> {
    if history::checked_out(&tracked.root).as_deref() == Some(branch) {
        let status = history::git(&tracked.root, &["status", "--porcelain", "--untracked-files=no"]).map_err(refused)?;
        if !status.trim().is_empty() {
            return Err(conflict(format!("branch '{}' is checked out with uncommitted changes; commit them first", branch)));
        }
        return Ok(Tree { root: tracked.root.clone(), _worktree: None });
    }
    let worktree = history::Worktree::add(&tracked.root, branch).map_err(refused)?;
    Ok(Tree { root: worktree.path.clone(), _worktree: Some(worktree) })
}

// ---- branches ----------------------------------------------------------------

/// `POST /projects/{id}/branches` — `{"name", "head": {"@id": commit}}`: a
/// new branch at a commit (by default the head of the default branch).
fn create_branch(workspace: &mut Workspace, project_id: &str, body: &Value, principal: Option<&Principal>, vocabulary: Vocabulary) -> Result<Response, Response> {
    writer(workspace, principal, vocabulary)?;
    let project = workspace.project(project_id)?;
    let tracked = repository(project)?;
    let name = body["name"].as_str().map(str::trim).unwrap_or_default().to_string();
    history::valid_ref_name("heads", &name).map_err(bad_request)?;
    if project.branches.iter().any(|branch| branch.name == name) {
        return Err(conflict(format!("branch '{}' already exists", name)));
    }
    let start = match body["head"]["@id"].as_str().or_else(|| body["referencedCommit"]["@id"].as_str()) {
        Some(commit) => git_sha(project, commit)?,
        None => project.head_of(project.default_branch()).git_commit.clone().map_or_else(|| git_sha(project, &project.default_branch().commits[0]), Ok)?,
    };
    history::git(&tracked.root, &["branch", &name, &start]).map_err(refused)?;
    workspace.reload(project_id).map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let project = workspace.project(project_id)?;
    let branch = project.branches.iter().find(|branch| branch.name == name).ok_or_else(|| error(StatusCode::INTERNAL_SERVER_ERROR, "the branch is not in the history".to_string()))?;
    Ok((StatusCode::CREATED, Json(project.branch_json(branch))).into_response())
}

/// `DELETE /projects/{id}/branches/{branchId}` — never the default branch,
/// never a branch whose commits no other branch holds (merge it first).
fn delete_branch(workspace: &mut Workspace, project_id: &str, branch_id: &str, principal: Option<&Principal>, vocabulary: Vocabulary) -> Result<Response, Response> {
    writer(workspace, principal, vocabulary)?;
    let project = workspace.project(project_id)?;
    let tracked = repository(project)?;
    let branch = project.branch(branch_id).ok_or_else(|| not_found(format!("branch '{}' not found", branch_id)))?;
    if branch.id == project.default_branch().id {
        return Err(conflict(format!("'{}' is the default branch and cannot be deleted", branch.name)));
    }
    let deleted = project.branch_json(branch);
    // `-d`, not `-D`: git refuses to drop commits no other branch holds.
    history::git(&tracked.root, &["branch", "-d", &branch.name])
        .map_err(|e| conflict(format!("branch '{}' holds commits no other branch has; merge it first ({})", branch.name, e)))?;
    workspace.reload(project_id).map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(deleted).into_response())
}

// ---- snapshots ----------------------------------------------------------------

/// `POST /projects/{id}/tags` — `{"name", "taggedCommit": {"@id"}, "description"}`:
/// a snapshot, an annotated git tag (who and when), never moved.
fn create_tag(workspace: &mut Workspace, project_id: &str, body: &Value, principal: Option<&Principal>, vocabulary: Vocabulary) -> Result<Response, Response> {
    let principal = writer(workspace, principal, vocabulary)?;
    let project = workspace.project(project_id)?;
    let tracked = repository(project)?;
    let name = body["name"].as_str().map(str::trim).unwrap_or_default().to_string();
    history::valid_ref_name("tags", &name).map_err(bad_request)?;
    if history::git(&tracked.root, &["rev-parse", "--verify", "--quiet", &format!("refs/tags/{name}")]).is_ok() {
        return Err(conflict(format!("snapshot '{}' already exists; a snapshot is never moved", name)));
    }
    let commit = body["taggedCommit"]["@id"].as_str().or_else(|| body["referencedCommit"]["@id"].as_str()).map(str::to_string).unwrap_or_else(|| project.head().id.clone());
    let sha = git_sha(project, &commit)?;
    let mut message = body["description"].as_str().map(str::trim).filter(|d| !d.is_empty()).unwrap_or(&name).to_string();
    if principal.name != access::SHARED_TOKEN_USER {
        message.push_str(&format!("\n\nSnapshot taken by {} through the ArcLang API.", principal.name));
    }
    history::git(&tracked.root, &["tag", "-a", &name, "-m", &message, &sha]).map_err(refused)?;
    workspace.reload(project_id).map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let project = workspace.project(project_id)?;
    let tag = project.tags.iter().find(|tag| tag.name == name).ok_or_else(|| error(StatusCode::INTERNAL_SERVER_ERROR, "the snapshot is not in the history".to_string()))?;
    Ok((StatusCode::CREATED, Json(project.tag_json(tag))).into_response())
}

// ---- merge ----------------------------------------------------------------------

/// The text of `path` at `revision`, or None when the file is not there.
fn show(root: &FsPath, revision: &str, path: &str) -> Option<String> {
    history::git(root, &["show", &format!("{revision}:{path}")]).ok()
}

/// What a request decides for each element in conflict: `{"element": label
/// or id, "keep": "target" | "source"}`.
fn resolutions(body: &Value) -> Result<Vec<(String, Side)>, Response> {
    let mut out = Vec::new();
    for item in body["resolutions"].as_array().into_iter().flatten() {
        let element = item["element"].as_str().ok_or_else(|| bad_request("a resolution names its 'element'".to_string()))?;
        let side = match item["keep"].as_str() {
            Some("target") => Side::Ours,
            Some("source") => Side::Theirs,
            other => return Err(bad_request(format!("a resolution keeps 'target' or 'source', not {:?}", other))),
        };
        out.push((element.to_string(), side));
    }
    Ok(out)
}

/// `POST /projects/{id}/branches/{branchId}/merge` — `{"source": {"@id"} |
/// "sourceName", "description", "resolutions": [...]}`: merge the source
/// branch into this one as one merge commit. With elements in conflict and
/// no choice for each, nothing is written and the answer (409) lists them.
fn merge(workspace: &mut Workspace, project_id: &str, target_id: &str, body: &Value, principal: Option<&Principal>, vocabulary: Vocabulary) -> Result<Response, Response> {
    let principal = writer(workspace, principal, vocabulary)?.clone();
    let project = workspace.project(project_id)?;
    let tracked = repository(project)?;
    let target = project.branch(target_id).ok_or_else(|| not_found(format!("branch '{}' not found", target_id)))?;
    let source = branch_named(project, body, "source")?;
    if source.id == target.id {
        return Err(bad_request("a branch cannot be merged into itself".to_string()));
    }
    let (target_name, source_name) = (target.name.clone(), source.name.clone());
    let chosen = resolutions(body)?;
    let root = tracked.root.clone();
    let base = history::git(&root, &["merge-base", &target_name, &source_name]).map_err(refused)?.trim().to_string();
    let source_sha = history::git(&root, &["rev-parse", &source_name]).map_err(refused)?.trim().to_string();
    if base == source_sha {
        return Err(refused(format!("'{}' has nothing '{}' does not already hold", source_name, target_name)));
    }

    let tree = tree_of(&tracked, &target_name)?;
    // git merges what is not a model file and records both parents; the
    // model files are then merged by element identity.
    let attempted = history::git(&tree.root, &["merge", "--no-ff", "--no-commit", "--no-edit", &source_name]);
    let abort = |tree: &Tree| {
        let _ = history::git(&tree.root, &["merge", "--abort"]);
    };
    let mut touched: Vec<String> = Vec::new();
    for side in [&target_name, &source_name] {
        let listing = history::git(&root, &["diff", "--name-only", &base, side.as_str()]).map_err(refused)?;
        for file in listing.lines().filter(|f| f.ends_with(".arc") || f.ends_with(".layout.json")) {
            if !touched.iter().any(|t| t == file) {
                touched.push(file.to_string());
            }
        }
    }
    let mut disputes: Vec<Value> = Vec::new();
    for file in &touched {
        let (b, o, t) = (show(&root, &base, file), show(&root, &target_name, file), show(&root, &source_name, file));
        let merged = match (b, o, t) {
            (_, Some(o), None) => Some(o), // kept here, absent there: git decides; keep ours
            (_, None, Some(t)) => Some(t),
            (_, None, None) => None,
            (b, Some(o), Some(t)) => {
                let b = b.unwrap_or_default();
                if file.ends_with(".layout.json") {
                    Some(layout::merge_layout(&b, &o, &t).map_err(|e| { abort(&tree); refused(format!("{file}: {e}")) })?.text)
                } else {
                    let outcome = merge_arc(&b, &o, &t).map_err(|e| { abort(&tree); refused(format!("{file}: {e}")) })?;
                    let (text, open) = resolve_conflicts(&outcome.text, &outcome.conflicts, &chosen);
                    for c in open {
                        disputes.push(json!({ "file": file, "element": c.element, "label": c.label, "reason": c.reason, "target": c.ours, "source": c.theirs }));
                    }
                    Some(text)
                }
            }
        };
        let place = tree.root.join(file);
        match merged {
            Some(text) => {
                if let Some(parent) = place.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| { abort(&tree); error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()) })?;
                }
                std::fs::write(&place, text).map_err(|e| { abort(&tree); error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()) })?;
                history::git(&tree.root, &["add", "--", file]).map_err(|e| { abort(&tree); refused(e) })?;
            }
            None => {
                let _ = history::git(&tree.root, &["rm", "--quiet", "--cached", "--ignore-unmatch", "--", file]);
            }
        }
    }
    if !disputes.is_empty() {
        abort(&tree);
        let body = json!({
            "@type": "MergeConflicts",
            "description": format!("{} element(s) changed differently on '{}' and '{}': send the request again with a 'resolutions' entry for each ({{\"element\": <label>, \"keep\": \"target\" | \"source\"}})", disputes.len(), target_name, source_name),
            "target": target_name,
            "source": source_name,
            "conflicts": disputes,
        });
        return Err((StatusCode::CONFLICT, Json(body)).into_response());
    }
    let unmerged = history::git(&tree.root, &["diff", "--name-only", "--diff-filter=U"]).unwrap_or_default();
    if !unmerged.trim().is_empty() {
        abort(&tree);
        return Err(conflict(format!("files other than the model conflict and must be merged in git: {}", unmerged.lines().collect::<Vec<_>>().join(", "))));
    }
    let _ = attempted; // a failed `git merge` is fine once every conflict is resolved above
    // The merged model must compile before it is a commit.
    let model = tree.root.join(&tracked.relative);
    if let Err(reason) = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(&model) {
        abort(&tree);
        return Err(refused(format!("the merged model does not compile: {}", reason)));
    }
    let message = body["description"].as_str().map(str::trim).filter(|d| !d.is_empty()).map(str::to_string).unwrap_or_else(|| format!("Merge '{}' into '{}' through the ArcLang API", source_name, target_name));
    history::git(&tree.root, &["commit", "--quiet", "--no-verify", "-m", &message]).map_err(|e| { abort(&tree); refused(e) })?;
    let sha = history::git(&tree.root, &["rev-parse", "HEAD"]).map_err(refused)?.trim().to_string();
    note_user(&root, &sha, &principal);
    drop(tree);
    commit_created(workspace, project_id, &sha)
}

// ---- restore ----------------------------------------------------------------------

/// `POST /projects/{id}/branches/{branchId}/restore` — `{"commit": {"@id"},
/// "elements": ["LC-001", ...], "description"}`: the model (or only the
/// elements named, by their ArcLang id) as it was at a commit, written as a
/// new commit on the branch. History is not rewritten.
fn restore(workspace: &mut Workspace, project_id: &str, branch_id: &str, body: &Value, principal: Option<&Principal>, vocabulary: Vocabulary) -> Result<Response, Response> {
    let principal = writer(workspace, principal, vocabulary)?.clone();
    let project = workspace.project(project_id)?;
    let tracked = repository(project)?;
    let branch = project.branch(branch_id).ok_or_else(|| not_found(format!("branch '{}' not found", branch_id)))?;
    let branch_name = branch.name.clone();
    let commit = body["commit"]["@id"].as_str().ok_or_else(|| bad_request("a restore names the 'commit' to go back to".to_string()))?;
    let from = git_sha(project, commit)?;
    let from_description = project.commits[project.position(commit).expect("checked")].description.clone();
    let elements: Vec<String> = body["elements"].as_array().into_iter().flatten().filter_map(|e| e.as_str().map(str::to_string)).collect();
    let tree = tree_of(&tracked, &branch_name)?;
    let model = tree.root.join(&tracked.relative);
    let files = history::model_files(&model, &history::Tracked { root: tree.root.clone(), relative: tracked.relative.clone() }).map_err(refused)?;
    if elements.is_empty() {
        // The whole model: every file it is made of, as the commit has them.
        let mut arguments: Vec<&str> = vec!["checkout", &from, "--"];
        let at_commit: Vec<String> = files.iter().filter(|file| show(&tree.root, &from, file).is_some()).cloned().collect();
        arguments.extend(at_commit.iter().map(String::as_str));
        history::git(&tree.root, &arguments).map_err(refused)?;
    } else {
        for element in &elements {
            let mut done = false;
            for file in &files {
                let (Some(old), Ok(now)) = (show(&tree.root, &from, file), std::fs::read_to_string(tree.root.join(file))) else { continue };
                match restore_element(&now, &old, element) {
                    Ok(Some(text)) => {
                        std::fs::write(tree.root.join(file), text).map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                        history::git(&tree.root, &["add", "--", file]).map_err(refused)?;
                        done = true;
                        break;
                    }
                    Ok(None) => continue,
                    Err(reason) => return Err(refused(format!("{file}: {reason}"))),
                }
            }
            if !done {
                let _ = history::git(&tree.root, &["checkout", "--", "."]);
                return Err(not_found(format!("element '{}' is not in the model at that commit", element)));
            }
        }
    }
    let changed = history::git(&tree.root, &["status", "--porcelain", "--untracked-files=no"]).unwrap_or_default();
    if changed.trim().is_empty() {
        return Err(refused("nothing to restore: the branch already holds that state".to_string()));
    }
    if let Err(reason) = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(&model) {
        let _ = history::git(&tree.root, &["checkout", "HEAD", "--", "."]);
        return Err(refused(format!("the restored model does not compile: {}", reason)));
    }
    let what = if elements.is_empty() { "the model".to_string() } else { elements.join(", ") };
    let message = body["description"].as_str().map(str::trim).filter(|d| !d.is_empty()).map(str::to_string).unwrap_or_else(|| format!("Restore {} as of \"{}\" through the ArcLang API", what, from_description));
    history::git(&tree.root, &["commit", "--quiet", "--no-verify", "-a", "-m", &message]).map_err(refused)?;
    let sha = history::git(&tree.root, &["rev-parse", "HEAD"]).map_err(refused)?.trim().to_string();
    note_user(&tracked.root, &sha, &principal);
    drop(tree);
    commit_created(workspace, project_id, &sha)
}

// ---- HTTP -----------------------------------------------------------------------------

type Shared = State<handlers::Api>;
type Who = Option<axum::Extension<Principal>>;

async fn blocking(api: handlers::Api, work: impl FnOnce(&mut Workspace) -> Result<Response, Response> + Send + 'static) -> Response {
    let outcome = tokio::task::spawn_blocking(move || {
        let mut workspace = write(&api.workspace);
        work(&mut workspace)
    })
    .await;
    match outcome {
        Ok(Ok(response)) | Ok(Err(response)) => response,
        Err(failure) => error(StatusCode::INTERNAL_SERVER_ERROR, format!("the operation did not complete: {}", failure)),
    }
}

fn principal(who: Who) -> Option<Principal> {
    who.map(|axum::Extension(principal)| principal)
}

pub(super) async fn post_branch(State(api): Shared, Path(project_id): Path<String>, who: Who, Json(body): Json<Value>) -> Response {
    let (who, vocabulary) = (principal(who), api.vocabulary);
    blocking(api, move |w| create_branch(w, &project_id, &body, who.as_ref(), vocabulary)).await
}

pub(super) async fn delete_branch_handler(State(api): Shared, Path((project_id, branch_id)): Path<(String, String)>, who: Who) -> Response {
    let (who, vocabulary) = (principal(who), api.vocabulary);
    blocking(api, move |w| delete_branch(w, &project_id, &branch_id, who.as_ref(), vocabulary)).await
}

pub(super) async fn post_tag(State(api): Shared, Path(project_id): Path<String>, who: Who, Json(body): Json<Value>) -> Response {
    let (who, vocabulary) = (principal(who), api.vocabulary);
    blocking(api, move |w| create_tag(w, &project_id, &body, who.as_ref(), vocabulary)).await
}

pub(super) async fn post_merge(State(api): Shared, Path((project_id, branch_id)): Path<(String, String)>, who: Who, Json(body): Json<Value>) -> Response {
    let (who, vocabulary) = (principal(who), api.vocabulary);
    blocking(api, move |w| merge(w, &project_id, &branch_id, &body, who.as_ref(), vocabulary)).await
}

pub(super) async fn post_restore(State(api): Shared, Path((project_id, branch_id)): Path<(String, String)>, who: Who, Json(body): Json<Value>) -> Response {
    let (who, vocabulary) = (principal(who), api.vocabulary);
    blocking(api, move |w| restore(w, &project_id, &branch_id, &body, who.as_ref(), vocabulary)).await
}
