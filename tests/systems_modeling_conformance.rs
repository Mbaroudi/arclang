//! What an independent audit of the Systems Modeling API found, held:
//! spec shapes of projects, branches, commits and data versions, cursor
//! pagination that keeps the request's parameters and survives any size,
//! queries paginated and with array values, errors as the spec's Error,
//! and writes that refuse a dirty imported file.

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

const MAIN: &str = r#"model Brakes {}

import "requirements.arc"

logical_architecture "Braking" {
    component "Controller" { id: "LC-001" }
    component "Monitor" { id: "LC-002" }
}

trace "LC-001" satisfies "REQ-001"
trace "LC-002" satisfies "REQ-001"
"#;

const REQUIREMENTS: &str = r#"requirements safety {
    req "REQ-001" "Braking reaction" { description: "Brake within 100 ms" }
}
"#;

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["-c", "user.name=Test", "-c", "user.email=test@example.com", "-c", "commit.gpgsign=false"])
        .args(args)
        .env("GIT_COMMITTER_DATE", "2020-01-02T03:04:05Z")
        .env("GIT_AUTHOR_DATE", "2020-01-02T03:04:05Z")
        .output()
        .expect("git runs");
    assert!(output.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// A git repository holding a model of two files, committed once.
fn repository() -> (tempfile::TempDir, PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.name", "Model Server"]);
    git(root, &["config", "user.email", "server@example.com"]);
    git(root, &["config", "commit.gpgsign", "false"]);
    std::fs::write(root.join("brakes.arc"), MAIN).unwrap();
    std::fs::write(root.join("requirements.arc"), REQUIREMENTS).unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-q", "-m", "initial model"]);
    let model = root.join("brakes.arc");
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

async fn raw(app: &Router, method: &str, uri: &str, authorization: Option<&str>, body: Option<&str>) -> (StatusCode, Option<String>, Value) {
    let mut request = Request::builder().method(method).uri(uri);
    if let Some(authorization) = authorization {
        request = request.header("authorization", authorization);
    }
    let request = match body {
        Some(body) => request.header("content-type", "application/json").body(Body::from(body.to_string())),
        None => request.body(Body::empty()),
    }
    .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let link = response.headers().get("link").map(|v| v.to_str().unwrap().to_string());
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, link, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

async fn get(app: &Router, path: &str) -> (StatusCode, Option<String>, Value) {
    raw(app, "GET", &format!("{BASE}{path}"), Some(&format!("Bearer {TOKEN}")), None).await
}

/// (project id, head commit id).
async fn head(app: &Router) -> (String, String) {
    let (_, _, projects) = get(app, "/projects").await;
    let project = projects[0]["@id"].as_str().unwrap().to_string();
    let (_, _, commits) = get(app, &format!("/projects/{project}/commits")).await;
    (project, commits[0]["@id"].as_str().unwrap().to_string())
}

fn keys(value: &Value) -> Vec<String> {
    let mut keys: Vec<String> = value.as_object().unwrap().keys().cloned().collect();
    keys.sort();
    keys
}

#[tokio::test]
async fn projects_branches_and_commits_have_the_shapes_of_the_specification() {
    let (_guard, model) = repository();
    let app = server(&model, false);
    let (project, commit) = head(&app).await;

    let (_, _, project_json) = get(&app, &format!("/projects/{project}")).await;
    assert_eq!(keys(&project_json), ["@id", "@type", "alias", "created", "defaultBranch", "description", "name"]);
    let (_, _, branches) = get(&app, &format!("/projects/{project}/branches")).await;
    assert_eq!(keys(&branches[0]), ["@id", "@type", "alias", "created", "deleted", "description", "head", "name", "owningProject", "referencedCommit"]);
    assert_eq!(branches[0]["deleted"], Value::Null);
    let (_, _, commit_json) = get(&app, &format!("/projects/{project}/commits/{commit}")).await;
    assert_eq!(keys(&commit_json), ["@id", "@type", "alias", "created", "description", "name", "owningProject", "previousCommit"]);

    // The ArcLang extensions, when asked for.
    let (_, _, extended) = get(&app, &format!("/projects/{project}/commits/{commit}?extensions=true")).await;
    assert!(extended["arclang:gitCommit"].is_string(), "{extended}");
}

#[tokio::test]
async fn a_data_version_and_its_identity_have_the_shapes_of_the_specification() {
    let (_guard, model) = repository();
    let app = server(&model, false);
    let (project, commit) = head(&app).await;
    let (_, _, changes) = get(&app, &format!("/projects/{project}/commits/{commit}/changes")).await;
    let version = &changes[0];
    assert_eq!(keys(version), ["@id", "@type", "alias", "description", "identity", "name", "payload"]);
    assert_eq!(keys(&version["identity"]), ["@id", "@type", "alias", "description", "name"]);
    let (_, _, extended) = get(&app, &format!("/projects/{project}/commits/{commit}/changes?extensions=true")).await;
    assert_eq!(extended[0]["arclang:changeKind"], "added");
}

#[tokio::test]
async fn a_commit_is_dated_by_git() {
    let (_guard, model) = repository();
    let app = server(&model, false);
    let (project, commit) = head(&app).await;
    let (_, _, commit_json) = get(&app, &format!("/projects/{project}/commits/{commit}")).await;
    assert_eq!(commit_json["created"], "2020-01-02T03:04:05Z");
}

#[tokio::test]
async fn a_huge_page_size_is_clamped_not_a_panic() {
    let (_guard, model) = repository();
    let app = server(&model, false);
    let (project, commit) = head(&app).await;
    let (_, _, all) = get(&app, &format!("/projects/{project}/commits/{commit}/elements")).await;
    let first = all[0]["@id"].as_str().unwrap();
    let (status, _, page) = get(&app, &format!("/projects/{project}/commits/{commit}/elements?page%5Bafter%5D={first}&page%5Bsize%5D=18446744073709551615")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(page.as_array().unwrap().len(), all.as_array().unwrap().len() - 1);
}

#[tokio::test]
async fn the_link_header_keeps_the_other_parameters_of_the_request() {
    let (_guard, model) = repository();
    let app = server(&model, false);
    let (project, commit) = head(&app).await;
    let (_, _, all) = get(&app, &format!("/projects/{project}/commits/{commit}/elements")).await;
    let requirement = all.as_array().unwrap().iter().find(|e| e["shortName"] == "REQ-001").unwrap()["@id"].as_str().unwrap().to_string();
    let path = format!("/projects/{project}/commits/{commit}/elements/{requirement}/relationships");
    let (_, _, every) = get(&app, &format!("{path}?direction=in")).await;
    assert_eq!(every.as_array().unwrap().len(), 2, "{every}");
    let (_, link, first) = get(&app, &format!("{path}?direction=in&page%5Bsize%5D=1")).await;
    let link = link.expect("a next link");
    assert!(link.contains("direction=in"), "{link}");
    // Following the link gives the second incoming relationship, not every one.
    let next = link.split('<').nth(1).unwrap().split('>').next().unwrap().replace('[', "%5B").replace(']', "%5D");
    let (_, _, second) = raw(&app, "GET", &next, Some(&format!("Bearer {TOKEN}")), None).await;
    assert_eq!(second.as_array().unwrap().len(), 1, "{second}");
    assert_ne!(second[0]["@id"], first[0]["@id"]);
}

#[tokio::test]
async fn query_results_are_paginated_and_a_value_may_be_an_array() {
    let (_guard, model) = repository();
    let app = server(&model, false);
    let (project, _) = head(&app).await;
    let query = |value: Value| json!({ "@type": "Query", "where": { "@type": "PrimitiveConstraint", "property": "@type", "operator": "=", "value": value } }).to_string();
    let uri = format!("{BASE}/projects/{project}/query-results");
    let bearer = format!("Bearer {TOKEN}");
    let (_, _, scalar) = raw(&app, "POST", &uri, Some(&bearer), Some(&query(json!("LogicalComponent")))).await;
    let (_, _, array) = raw(&app, "POST", &uri, Some(&bearer), Some(&query(json!(["LogicalComponent"])))).await;
    assert_eq!(scalar.as_array().unwrap().len(), 2, "{scalar}");
    assert_eq!(array, scalar);
    let (_, link, page) = raw(&app, "POST", &format!("{uri}?page%5Bsize%5D=1"), Some(&bearer), Some(&query(json!("LogicalComponent")))).await;
    assert_eq!(page.as_array().unwrap().len(), 1);
    assert!(link.is_some_and(|link| link.contains("rel=\"next\"")));
}

#[tokio::test]
async fn the_bearer_scheme_is_case_insensitive() {
    let (_guard, model) = repository();
    let app = server(&model, false);
    let (status, _, _) = raw(&app, "GET", &format!("{BASE}/projects"), Some(&format!("bearer {TOKEN}")), None).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn errors_are_the_error_of_the_specification() {
    let (_guard, model) = repository();
    let app = server(&model, true);
    let (project, _) = head(&app).await;
    let bearer = format!("Bearer {TOKEN}");
    let (status, _, body) = raw(&app, "POST", &format!("{BASE}/projects/{project}/commits"), Some(&bearer), Some("{not json")).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["@type"], "Error", "{body}");
    let (status, _, body) = raw(&app, "GET", &format!("{BASE}/projects/{project}/commits/x/elements?page%5Bsize%5D=abc"), Some(&bearer), None).await;
    assert!(status.is_client_error());
    assert_eq!(body["@type"], "Error", "{body}");
}

fn commit_request(project_head: &str, changes: Value) -> String {
    json!({ "@type": "Commit", "description": "audit", "previousCommit": { "@id": project_head }, "change": changes }).to_string()
}

#[tokio::test]
async fn a_write_while_an_imported_file_is_dirty_is_a_conflict() {
    let (guard, model) = repository();
    let app = server(&model, true);
    let (project, commit) = head(&app).await;
    std::fs::write(guard.path().join("requirements.arc"), REQUIREMENTS.replace("Braking reaction", "DIRTY")).unwrap();
    let (_, _, all) = get(&app, &format!("/projects/{project}/commits/{commit}/elements")).await;
    let monitor = all.as_array().unwrap().iter().find(|e| e["shortName"] == "LC-002").unwrap()["@id"].as_str().unwrap().to_string();
    let body = commit_request(&commit, json!([{ "@type": "DataVersion", "identity": { "@id": monitor }, "payload": { "attributes": { "latency": { "value": 3, "unit": "ms" } } } }]));
    let (status, _, response) = raw(&app, "POST", &format!("{BASE}/projects/{project}/commits"), Some(&format!("Bearer {TOKEN}")), Some(&body)).await;
    assert_eq!(status, StatusCode::CONFLICT, "{response}");
    assert!(response["description"].as_str().unwrap().contains("requirements.arc"), "{response}");
    assert_eq!(git(guard.path(), &["rev-list", "--count", "HEAD"]).trim(), "1");
}

#[tokio::test]
async fn a_requirement_and_a_component_are_created_in_the_blocks_the_docs_write() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.name", "Model Server"]);
    git(root, &["config", "user.email", "server@example.com"]);
    git(root, &["config", "commit.gpgsign", "false"]);
    let model = root.join("brake.arc");
    std::fs::write(&model, "model BrakeAssist {\n  requirements safety {\n    req \"REQ-001\" \"Braking reaction\" { description: \"Command braking within 100 ms\" }\n  }\n  architecture logical {\n    component \"Brake controller\" { id: \"LC-BRAKE\" }\n  }\n}\n").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-q", "-m", "initial"]);
    let app = server(&model, true);
    let (project, commit) = head(&app).await;
    let body = commit_request(&commit, json!([
        { "payload": { "@type": "Requirement", "name": "Range", "attributes": { "id": "REQ-002" } } },
        { "payload": { "@type": "LogicalComponent", "name": "Logger", "attributes": { "id": "LC-LOG" } } }
    ]));
    let (status, _, response) = raw(&app, "POST", &format!("{BASE}/projects/{project}/commits"), Some(&format!("Bearer {TOKEN}")), Some(&body)).await;
    assert_eq!(status, StatusCode::CREATED, "{response}");
    let text = std::fs::read_to_string(&model).unwrap();
    assert!(text.contains("REQ-002") && text.contains("LC-LOG"), "{text}");
}

#[tokio::test]
async fn a_commit_that_touched_only_an_imported_file_is_listed() {
    let (guard, model) = repository();
    std::fs::write(guard.path().join("requirements.arc"), REQUIREMENTS.replace("100 ms", "80 ms")).unwrap();
    git(guard.path(), &["commit", "-q", "-am", "tighter requirement"]);
    let app = server(&model, false);
    let (project, _) = head(&app).await;
    let (_, _, commits) = get(&app, &format!("/projects/{project}/commits")).await;
    let descriptions: Vec<&str> = commits.as_array().unwrap().iter().map(|c| c["description"].as_str().unwrap()).collect();
    assert_eq!(descriptions, ["tighter requirement", "initial model"]);
}
