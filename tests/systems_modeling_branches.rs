//! Branches, snapshots, merges, restores and comparisons through the
//! Systems Modeling API, on a real git repository: a branch is a git branch,
//! a snapshot a git tag, a merge a merge commit by element identity, a
//! restore a new commit; the working tree of the checked-out branch is only
//! touched by what is written to that branch.

use arclang::web_server::systems_modeling::Workspace;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use tower::util::ServiceExt;

const BASE: &str = "/api/systems-modeling";
const TOKEN: &str = "test-token-0123456789abcdef-0123456789";

const MODEL: &str = r#"model Brakes {}

logical_architecture "Braking" {
    component "Controller" {
        id: "LC-001"
        latency: 25 ms
    }
    component "Monitor" {
        id: "LC-002"
        latency: 40 ms
    }
}
"#;

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["-c", "user.name=Test", "-c", "user.email=test@example.com", "-c", "commit.gpgsign=false"])
        .args(args)
        .output()
        .expect("git runs");
    assert!(output.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn repository() -> (tempfile::TempDir, PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    git(root, &["init", "-q", "-b", "main"]);
    git(root, &["config", "user.name", "Model Server"]);
    git(root, &["config", "user.email", "server@example.com"]);
    git(root, &["config", "commit.gpgsign", "false"]);
    git(root, &["config", "tag.gpgsign", "false"]);
    let model = root.join("brakes.arc");
    std::fs::write(&model, MODEL).unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-q", "-m", "initial model"]);
    (directory, model)
}

fn server(model: &Path, writable: bool) -> Router {
    let mut workspace = Workspace::default();
    workspace.add_file_with_history(model, 50).expect("model loads");
    workspace.require_token(TOKEN).unwrap();
    if writable {
        workspace.allow_writes().unwrap();
    }
    arclang::web_server::build_router_with(workspace)
}

async fn call(app: &Router, method: &str, path: &str, body: Option<Value>) -> (StatusCode, Value) {
    let request = Request::builder().method(method).uri(format!("{BASE}{path}")).header("authorization", format!("Bearer {TOKEN}"));
    let request = match body {
        Some(body) => request.header("content-type", "application/json").body(Body::from(body.to_string())),
        None => request.body(Body::empty()),
    }
    .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

async fn project(app: &Router) -> String {
    let (_, projects) = call(app, "GET", "/projects", None).await;
    projects[0]["@id"].as_str().unwrap().to_string()
}

async fn branch_id(app: &Router, project: &str, name: &str) -> String {
    let (_, branches) = call(app, "GET", &format!("/projects/{project}/branches"), None).await;
    branches.as_array().unwrap().iter().find(|b| b["name"] == name).unwrap_or_else(|| panic!("no branch {name}: {branches}"))["@id"].as_str().unwrap().to_string()
}

async fn head(app: &Router, project: &str, branch: &str) -> String {
    let (_, b) = call(app, "GET", &format!("/projects/{project}/branches/{branch}"), None).await;
    b["head"]["@id"].as_str().unwrap().to_string()
}

/// The `latency` of an element at a commit, as the API reads it.
async fn latency(app: &Router, project: &str, commit: &str, short_name: &str) -> Value {
    let (_, elements) = call(app, "GET", &format!("/projects/{project}/commits/{commit}/elements"), None).await;
    let element = elements.as_array().unwrap().iter().find(|e| e["shortName"] == short_name).unwrap_or_else(|| panic!("no {short_name} at {commit}"));
    element["attributes"]["latency"].clone()
}

async fn element_uuid(app: &Router, project: &str, commit: &str, short_name: &str) -> String {
    let (_, elements) = call(app, "GET", &format!("/projects/{project}/commits/{commit}/elements"), None).await;
    elements.as_array().unwrap().iter().find(|e| e["shortName"] == short_name).unwrap()["@id"].as_str().unwrap().to_string()
}

/// Set the latency of an element on a branch, through the API.
async fn set_latency(app: &Router, project: &str, branch: &str, short_name: &str, ms: u32) -> Value {
    let h = head(app, project, branch).await;
    let element = element_uuid(app, project, &h, short_name).await;
    let body = json!({ "change": [{ "@type": "DataVersion", "identity": { "@id": element }, "payload": { "attributes": { "latency": { "value": ms, "unit": "ms" } } } }] });
    let (status, commit) = call(app, "POST", &format!("/projects/{project}/commits?branchId={branch}"), Some(body)).await;
    assert_eq!(status, StatusCode::CREATED, "{commit}");
    commit
}

fn ms(value: &Value) -> f64 {
    value["value"].as_f64().unwrap_or_else(|| panic!("not a quantity: {value}"))
}

#[tokio::test]
async fn a_branch_is_created_written_and_listed_without_touching_the_working_tree() {
    let (_guard, model) = repository();
    let app = server(&model, true);
    let project = project(&app).await;
    let main = branch_id(&app, &project, "main").await;
    let main_head = head(&app, &project, &main).await;

    let (status, branch) = call(&app, "POST", &format!("/projects/{project}/branches"), Some(json!({ "name": "feature/faster-controller" }))).await;
    assert_eq!(status, StatusCode::CREATED, "{branch}");
    assert_eq!(branch["head"]["@id"], main_head, "a new branch starts at the head of the default branch");
    let (_, branches) = call(&app, "GET", &format!("/projects/{project}/branches"), None).await;
    assert_eq!(branches.as_array().unwrap().len(), 2);
    assert_eq!(branches[0]["name"], "main", "the default branch comes first");

    let feature = branch["@id"].as_str().unwrap().to_string();
    let commit = set_latency(&app, &project, &feature, "LC-001", 10).await;
    assert_eq!(commit["previousCommit"][0]["@id"], main_head);
    assert_eq!(std::fs::read_to_string(&model).unwrap(), MODEL, "the checked-out main is untouched");
    assert_eq!(head(&app, &project, &main).await, main_head, "main has not moved");
    assert_eq!(ms(&latency(&app, &project, &head(&app, &project, &feature).await, "LC-001").await), 10.0);
    let (_, commits) = call(&app, "GET", &format!("/projects/{project}/commits?branchId={feature}"), None).await;
    assert_eq!(commits.as_array().unwrap().len(), 2, "the branch's line: its commit, then main's");
    assert_eq!(git(model.parent().unwrap(), &["worktree", "list"]).lines().count(), 1, "the temporary checkout is gone");

    let (status, _) = call(&app, "POST", &format!("/projects/{project}/branches"), Some(json!({ "name": "feature/faster-controller" }))).await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _) = call(&app, "POST", &format!("/projects/{project}/branches"), Some(json!({ "name": "bad..name" }))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn a_snapshot_names_a_commit_for_good() {
    let (_guard, model) = repository();
    let app = server(&model, true);
    let project = project(&app).await;
    let main = branch_id(&app, &project, "main").await;
    let before = head(&app, &project, &main).await;

    let (status, tag) = call(&app, "POST", &format!("/projects/{project}/tags"), Some(json!({ "name": "design-review-1", "description": "Model as reviewed on 1 November" }))).await;
    assert_eq!(status, StatusCode::CREATED, "{tag}");
    assert_eq!(tag["taggedCommit"]["@id"], before);
    assert_eq!(tag["description"], "Model as reviewed on 1 November");
    set_latency(&app, &project, &main, "LC-001", 12).await;

    let (_, tags) = call(&app, "GET", &format!("/projects/{project}/tags"), None).await;
    assert_eq!(tags[0]["taggedCommit"]["@id"], before, "the snapshot stays on its commit when the branch moves");
    assert!(git(model.parent().unwrap(), &["tag", "-n1"]).contains("design-review-1"));
    let (status, _) = call(&app, "POST", &format!("/projects/{project}/tags"), Some(json!({ "name": "design-review-1" }))).await;
    assert_eq!(status, StatusCode::CONFLICT, "a snapshot is never moved");
    let (status, _) = call(&app, "GET", &format!("/projects/{project}/tags/{}", tag["@id"].as_str().unwrap()), None).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn a_snapshot_of_a_merge_names_the_merge() {
    // main did not move since the branch: the merge holds the same model as
    // the branch's head, and is still the commit the snapshot names.
    let (_guard, model) = repository();
    let app = server(&model, true);
    let project = project(&app).await;
    let main = branch_id(&app, &project, "main").await;
    let (_, feature) = call(&app, "POST", &format!("/projects/{project}/branches"), Some(json!({ "name": "feature" }))).await;
    let feature = feature["@id"].as_str().unwrap().to_string();
    set_latency(&app, &project, &feature, "LC-001", 10).await;
    let (status, merge) = call(&app, "POST", &format!("/projects/{project}/branches/{main}/merge"), Some(json!({ "source": { "@id": feature } }))).await;
    assert_eq!(status, StatusCode::CREATED, "{merge}");

    let (status, tag) = call(&app, "POST", &format!("/projects/{project}/tags"), Some(json!({ "name": "after-merge" }))).await;
    assert_eq!(status, StatusCode::CREATED, "{tag}");
    assert_eq!(tag["taggedCommit"]["@id"], merge["@id"], "the snapshot is on the merge, not on the branch's head");
    // Made within the same second, the commits still list newest first, each after its parents.
    let (_, commits) = call(&app, "GET", &format!("/projects/{project}/commits?branchId={main}"), None).await;
    let order: Vec<&str> = commits.as_array().unwrap().iter().map(|c| c["@id"].as_str().unwrap()).collect();
    assert_eq!(order.len(), 3, "{commits}");
    assert_eq!(order[0], merge["@id"].as_str().unwrap());
    assert_eq!(order[1], merge["previousCommit"][1]["@id"].as_str().unwrap(), "the branch's commit, then the first one");
}

#[tokio::test]
async fn branches_that_change_different_elements_merge_by_identity() {
    let (_guard, model) = repository();
    let app = server(&model, true);
    let project = project(&app).await;
    let main = branch_id(&app, &project, "main").await;
    let (_, feature) = call(&app, "POST", &format!("/projects/{project}/branches"), Some(json!({ "name": "feature" }))).await;
    let feature = feature["@id"].as_str().unwrap().to_string();
    set_latency(&app, &project, &feature, "LC-001", 10).await;
    set_latency(&app, &project, &main, "LC-002", 30).await;
    let (main_head, feature_head) = (head(&app, &project, &main).await, head(&app, &project, &feature).await);

    let (status, merge) = call(&app, "POST", &format!("/projects/{project}/branches/{main}/merge"), Some(json!({ "source": { "@id": feature } }))).await;
    assert_eq!(status, StatusCode::CREATED, "{merge}");
    let parents: Vec<&str> = merge["previousCommit"].as_array().unwrap().iter().map(|p| p["@id"].as_str().unwrap()).collect();
    assert_eq!(parents, vec![main_head.as_str(), feature_head.as_str()], "a merge commit has both heads as parents");
    let merged = head(&app, &project, &main).await;
    assert_eq!(merged, merge["@id"]);
    assert_eq!(ms(&latency(&app, &project, &merged, "LC-001").await), 10.0, "the feature's change");
    assert_eq!(ms(&latency(&app, &project, &merged, "LC-002").await), 30.0, "main's change");
    let on_disk = std::fs::read_to_string(&model).unwrap();
    assert!(on_disk.contains("latency: 10 ms") && on_disk.contains("latency: 30 ms"), "the checked-out main holds the merge: {on_disk}");
    assert_eq!(git(model.parent().unwrap(), &["log", "-1", "--format=%P"]).split_whitespace().count(), 2);

    let (status, _) = call(&app, "POST", &format!("/projects/{project}/branches/{main}/merge"), Some(json!({ "source": { "@id": feature } }))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "nothing left to merge");
}

#[tokio::test]
async fn an_element_changed_on_both_sides_is_a_conflict_resolved_by_choice() {
    let (_guard, model) = repository();
    let app = server(&model, true);
    let project = project(&app).await;
    let main = branch_id(&app, &project, "main").await;
    let (_, feature) = call(&app, "POST", &format!("/projects/{project}/branches"), Some(json!({ "name": "feature" }))).await;
    let feature = feature["@id"].as_str().unwrap().to_string();
    set_latency(&app, &project, &feature, "LC-001", 10).await;
    set_latency(&app, &project, &main, "LC-001", 15).await;
    let before = head(&app, &project, &main).await;
    let disk_before = std::fs::read_to_string(&model).unwrap();

    let (status, refused) = call(&app, "POST", &format!("/projects/{project}/branches/{main}/merge"), Some(json!({ "sourceName": "feature" }))).await;
    assert_eq!(status, StatusCode::CONFLICT, "{refused}");
    assert_eq!(refused["@type"], "MergeConflicts");
    let conflict = &refused["conflicts"][0];
    assert_eq!(conflict["element"], "LC-001");
    assert!(conflict["target"].as_str().unwrap().contains("15 ms") && conflict["source"].as_str().unwrap().contains("10 ms"), "{conflict}");
    assert_eq!(head(&app, &project, &main).await, before, "nothing is written");
    assert_eq!(std::fs::read_to_string(&model).unwrap(), disk_before, "the working tree is as it was");
    assert!(git(model.parent().unwrap(), &["status", "--porcelain"]).trim().is_empty(), "no merge left half done");

    let body = json!({ "sourceName": "feature", "resolutions": [{ "element": "LC-001", "keep": "source" }], "description": "Take the faster controller" });
    let (status, merge) = call(&app, "POST", &format!("/projects/{project}/branches/{main}/merge"), Some(body)).await;
    assert_eq!(status, StatusCode::CREATED, "{merge}");
    assert_eq!(merge["description"], "Take the faster controller");
    assert_eq!(ms(&latency(&app, &project, &merge["@id"].as_str().unwrap().to_string(), "LC-001").await), 10.0);
    assert!(!std::fs::read_to_string(&model).unwrap().contains("<<<<<<<"));
}

#[tokio::test]
async fn an_element_or_the_whole_model_goes_back_as_a_new_commit() {
    let (_guard, model) = repository();
    let app = server(&model, true);
    let project = project(&app).await;
    let main = branch_id(&app, &project, "main").await;
    let first = head(&app, &project, &main).await;
    set_latency(&app, &project, &main, "LC-001", 5).await;
    set_latency(&app, &project, &main, "LC-002", 7).await;

    let (status, restored) = call(&app, "POST", &format!("/projects/{project}/branches/{main}/restore"), Some(json!({ "commit": { "@id": first }, "elements": ["LC-001"] }))).await;
    assert_eq!(status, StatusCode::CREATED, "{restored}");
    let now = restored["@id"].as_str().unwrap().to_string();
    assert_eq!(ms(&latency(&app, &project, &now, "LC-001").await), 25.0, "the controller is back");
    assert_eq!(ms(&latency(&app, &project, &now, "LC-002").await), 7.0, "the rest is left as it is");
    assert!(restored["description"].as_str().unwrap().starts_with("Restore LC-001 as of"));

    let (status, whole) = call(&app, "POST", &format!("/projects/{project}/branches/{main}/restore"), Some(json!({ "commit": { "@id": first } }))).await;
    assert_eq!(status, StatusCode::CREATED, "{whole}");
    assert_eq!(std::fs::read_to_string(&model).unwrap(), MODEL, "the whole model as it was");
    assert_eq!(git(model.parent().unwrap(), &["rev-list", "--count", "HEAD"]).trim(), "5", "history kept: two edits and two restores on top");
    let (status, _) = call(&app, "POST", &format!("/projects/{project}/branches/{main}/restore"), Some(json!({ "commit": { "@id": first } }))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "nothing left to restore");
}

#[tokio::test]
async fn any_two_moments_compare_by_identity() {
    let (_guard, model) = repository();
    let app = server(&model, true);
    let project = project(&app).await;
    let main = branch_id(&app, &project, "main").await;
    let first = head(&app, &project, &main).await;
    set_latency(&app, &project, &main, "LC-001", 5).await;
    set_latency(&app, &project, &main, "LC-002", 7).await;
    let last = head(&app, &project, &main).await;

    let (_, step) = call(&app, "GET", &format!("/projects/{project}/commits/{last}/changes"), None).await;
    let (_, span) = call(&app, "GET", &format!("/projects/{project}/commits/{last}/changes?baseCommitId={first}&extensions=true"), None).await;
    let names = |changes: &Value| -> Vec<String> { changes.as_array().unwrap().iter().filter_map(|c| c["payload"]["shortName"].as_str().map(str::to_string)).collect() };
    assert_eq!(names(&step), vec!["LC-002"], "the last commit changed the monitor only");
    let mut both = names(&span);
    both.sort();
    assert_eq!(both, vec!["LC-001", "LC-002"], "from the first commit, both changed");
    assert!(span.as_array().unwrap().iter().all(|c| c["arclang:changeKind"] == "modified"));
}

#[tokio::test]
async fn a_branch_is_deleted_only_once_merged_and_never_the_default() {
    let (_guard, model) = repository();
    let app = server(&model, true);
    let project = project(&app).await;
    let main = branch_id(&app, &project, "main").await;
    let (_, feature) = call(&app, "POST", &format!("/projects/{project}/branches"), Some(json!({ "name": "feature" }))).await;
    let feature = feature["@id"].as_str().unwrap().to_string();
    set_latency(&app, &project, &feature, "LC-001", 10).await;

    let (status, _) = call(&app, "DELETE", &format!("/projects/{project}/branches/{feature}"), None).await;
    assert_eq!(status, StatusCode::CONFLICT, "its commit is on no other branch");
    let (status, _) = call(&app, "DELETE", &format!("/projects/{project}/branches/{main}"), None).await;
    assert_eq!(status, StatusCode::CONFLICT, "the default branch stays");
    call(&app, "POST", &format!("/projects/{project}/branches/{main}/merge"), Some(json!({ "source": { "@id": feature } }))).await;
    let (status, _) = call(&app, "DELETE", &format!("/projects/{project}/branches/{feature}"), None).await;
    assert_eq!(status, StatusCode::OK);
    let (_, branches) = call(&app, "GET", &format!("/projects/{project}/branches"), None).await;
    assert_eq!(branches.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn a_read_only_server_creates_no_branch_snapshot_or_merge() {
    let (_guard, model) = repository();
    let app = server(&model, false);
    let project = project(&app).await;
    let main = branch_id(&app, &project, "main").await;
    for (path, body) in [
        (format!("/projects/{project}/branches"), json!({ "name": "x" })),
        (format!("/projects/{project}/tags"), json!({ "name": "x" })),
        (format!("/projects/{project}/branches/{main}/merge"), json!({ "sourceName": "x" })),
        (format!("/projects/{project}/branches/{main}/restore"), json!({ "commit": { "@id": "x" } })),
    ] {
        let (status, _) = call(&app, "POST", &path, Some(body)).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{path}");
    }
    assert!(git(model.parent().unwrap(), &["branch", "--list"]).lines().count() == 1);
}
