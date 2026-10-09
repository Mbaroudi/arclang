//! Writes through the API: `POST /projects/{id}/commits`.
//!
//! A commit request carries changes (see `operations`): attributes to set
//! or remove, elements and traces to create or delete. They are applied to
//! the model file as lossless source edits (`compiler::source_edit`):
//! comments and layout are kept. The request is all or nothing, and it
//! becomes ONE git commit of the model file; nothing is written unless the
//! edited model compiles and holds exactly what was asked.

use super::operations::{check_outcome, operations, Operation};
use super::*;
use crate::compiler::elements;
use crate::compiler::source_edit;

fn refused(description: String) -> Response {
    error(StatusCode::UNPROCESSABLE_ENTITY, description)
}

fn conflict(description: String) -> Response {
    error(StatusCode::CONFLICT, description)
}

/// The commit the request must be based on, when it names one.
fn expected_previous(body: &Value) -> Option<&str> {
    match &body["previousCommit"] {
        Value::Array(items) => items.first().and_then(|item| item["@id"].as_str()),
        other => other["@id"].as_str(),
    }
}

fn commit(
    workspace: &mut Workspace,
    project_id: &str,
    body: &Value,
    vocabulary: Vocabulary,
    principal: Option<&Principal>,
) -> Result<Response, Response> {
    if vocabulary == Vocabulary::SysML {
        return Err(error(
            StatusCode::METHOD_NOT_ALLOWED,
            "the SysML v2 view is read-only; write through the ArcLang vocabulary".to_string(),
        ));
    }
    if !workspace.access.allows_write() {
        return Err(error(
            StatusCode::FORBIDDEN,
            "this server is read-only; writes need `arclang serve --allow-write` and an API token".to_string(),
        ));
    }
    let Some(principal) = principal.filter(|principal| principal.role == Role::Write) else {
        let who = principal.map_or("an unauthenticated client".to_string(), |p| format!("user '{}'", p.name));
        return Err(error(StatusCode::FORBIDDEN, format!("{} may read but not write", who)));
    };
    let position = workspace
        .projects
        .iter()
        .position(|project| project.id == project_id)
        .ok_or_else(|| not_found(format!("project '{}' not found", project_id)))?;
    let project = &workspace.projects[position];
    let Some(path) = project.source.clone() else {
        return Err(refused("this project is not served from a file and cannot be written".to_string()));
    };
    let head = project.head();
    if let Some(expected) = expected_previous(body) {
        if expected != head.id {
            return Err(conflict(format!(
                "the request is based on commit '{}' but the head is '{}'",
                expected, head.id
            )));
        }
    }
    let Some(changes) = body["change"].as_array().filter(|changes| !changes.is_empty()) else {
        return Err(bad_request("a commit request needs a non-empty 'change' array".to_string()));
    };

    // The file must be exactly what git and the head commit say it is:
    // the commit to come must hold this request's edits and nothing else.
    let tracked = history::locate(&path).map_err(refused)?;
    let status = history::git(&tracked.root, &["status", "--porcelain", "--", &tracked.relative]).map_err(refused)?;
    if !status.trim().is_empty() {
        return Err(conflict(format!(
            "{} has uncommitted changes or is not tracked by git; commit it first",
            tracked.relative
        )));
    }
    let on_disk = compile_file(&path).map_err(conflict)?;
    if on_disk.head().digest != head.digest {
        return Err(conflict(format!(
            "{} changed since the server loaded it; restart the server",
            tracked.relative
        )));
    }

    let mut requested: Vec<Operation> = Vec::new();
    for change in changes {
        requested.extend(operations(change, &head.view)?);
    }
    if requested.is_empty() {
        return Err(refused("the request changes nothing: every attribute already has the requested value".to_string()));
    }

    let source = std::fs::read_to_string(&path).map_err(|e| refused(format!("{}: {}", tracked.relative, e)))?;
    let mut edited = source.clone();
    for operation in &requested {
        edited = operation.apply(&edited).map_err(|reason| refused(format!("{}: {}", operation.label(), reason)))?;
    }
    let compiled = source_edit::compile_beside(&path, &edited).map_err(refused)?;
    let graph = elements::build(&compiled.ast, &compiled.semantic_model);
    check_outcome(&requested, head, &Snapshot::from_compilation(&compiled, Utc::now()), &graph)?;

    let labels: Vec<String> = requested.iter().map(Operation::label).collect();
    let message = match body["description"].as_str().map(str::trim) {
        Some(description) if !description.is_empty() => description.to_string(),
        _ => {
            let mut summary = labels.join(", ");
            if let Some(first) = summary.get(..1).map(str::to_uppercase) {
                summary.replace_range(..1, &first);
            }
            format!("{} through the ArcLang API", summary)
        }
    };
    let user = Some(principal.name.as_str()).filter(|name| *name != access::SHARED_TOKEN_USER);

    std::fs::write(&path, &edited).map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let restore = |reason: String| {
        // Leave the file as git has it: a failed write must not half-happen.
        let restored = std::fs::write(&path, &source);
        let note = if restored.is_ok() { "the file was restored" } else { "THE FILE COULD NOT BE RESTORED" };
        error(StatusCode::INTERNAL_SERVER_ERROR, format!("{} ({})", reason, note))
    };
    let result = crate::Compiler::new(crate::CompilerConfig::default())
        .compile_file(&path)
        .map_err(|e| restore(e.to_string()))?;
    history::git(&tracked.root, &["commit", "--quiet", "-m", &message, "--", &tracked.relative]).map_err(restore)?;
    let sha = history::git(&tracked.root, &["rev-parse", "HEAD"])
        .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e))?
        .trim()
        .to_string();
    // Who asked for the change is the API's business: the commit message is
    // left as requested, and the name is kept in a git note so that it is
    // still known after a restart. A note that cannot be written loses
    // only that: the commit stands.
    let mut warnings = Vec::new();
    if let Some(user) = user {
        let noted = history::git(&tracked.root, &["notes", "--ref", history::USER_NOTES, "add", "-f", "-m", user, &sha]);
        if let Err(reason) = noted {
            warnings.push(format!("the user of this commit could not be recorded in git notes: {}", reason));
        }
    }

    let project = &mut workspace.projects[position];
    let mut snapshot = Snapshot::from_compilation(&result, Utc::now());
    if let Some(key) = &project.history_key {
        snapshot.id = history::commit_id(key, &sha);
    }
    snapshot.description = message;
    snapshot.git_commit = Some(sha);
    snapshot.user = user.map(str::to_string);
    snapshot.warnings.extend(warnings);
    project.commits.push(snapshot);

    let created = project.commit_json(project.commits.len() - 1);
    Ok((StatusCode::CREATED, Json(created)).into_response())
}

/// `POST /projects/{id}/commits` — apply the requested changes as one git commit.
pub(super) async fn create_commit(
    State(api): State<handlers::Api>,
    Path(project_id): Path<String>,
    principal: Option<axum::Extension<Principal>>,
    Json(body): Json<Value>,
) -> Response {
    let principal = principal.map(|axum::Extension(principal)| principal);
    let outcome = tokio::task::spawn_blocking(move || {
        let mut workspace = write(&api.workspace);
        commit(&mut workspace, &project_id, &body, api.vocabulary, principal.as_ref())
    })
    .await;
    match outcome {
        Ok(Ok(response)) | Ok(Err(response)) => response,
        Err(failure) => error(StatusCode::INTERNAL_SERVER_ERROR, format!("the write did not complete: {}", failure)),
    }
}
