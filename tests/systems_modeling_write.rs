//! Authentication and writes of the Systems Modeling API: a write is a
//! lossless edit of the model file and exactly one git commit.

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

const MODEL: &str = r#"// Braking controller
model Brakes {}

logical_architecture "Braking" {
    // The controller decides.
    component "Controller" {
        id: "LC-001"
        latency: 25 ms   // worst case
        safety_level: "ASIL-B"
    }
    component "Monitor" { id: "LC-002" }
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

/// A git repository holding the model, committed once.
fn repository() -> (tempfile::TempDir, PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    git(root, &["init", "-q"]);
    // The server commits with the repository's own identity.
    git(root, &["config", "user.name", "Model Server"]);
    git(root, &["config", "user.email", "server@example.com"]);
    git(root, &["config", "commit.gpgsign", "false"]);
    let model = root.join("brakes.arc");
    std::fs::write(&model, MODEL).unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-q", "-m", "initial model"]);
    (directory, model)
}

fn server(model: &Path, token: bool, writable: bool) -> Router {
    let mut workspace = Workspace::default();
    workspace.add_file_with_history(model, 50).expect("model loads");
    if token {
        workspace.require_token(TOKEN).unwrap();
    }
    if writable {
        workspace.allow_writes().unwrap();
    }
    arclang::web_server::build_router_with(workspace)
}

async fn call(app: &Router, method: &str, path: &str, token: Option<&str>, body: Option<Value>) -> (StatusCode, Value) {
    let mut request = Request::builder().method(method).uri(format!("{BASE}{path}"));
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
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

/// (project id, head commit id, uuid of the element with this short name).
async fn locate(app: &Router, short_name: &str) -> (String, String, String) {
    let (_, projects) = call(app, "GET", "/projects", Some(TOKEN), None).await;
    let project = projects[0]["@id"].as_str().unwrap().to_string();
    let (_, commits) = call(app, "GET", &format!("/projects/{project}/commits"), Some(TOKEN), None).await;
    let head = commits[0]["@id"].as_str().unwrap().to_string();
    let (_, elements) = call(app, "GET", &format!("/projects/{project}/commits/{head}/elements"), Some(TOKEN), None).await;
    let element = elements
        .as_array()
        .unwrap()
        .iter()
        .find(|element| element["shortName"] == short_name)
        .unwrap_or_else(|| panic!("no element {short_name}"));
    (project, head, element["@id"].as_str().unwrap().to_string())
}

fn change(element: &str, attributes: Value) -> Value {
    json!({ "@type": "DataVersion", "identity": { "@id": element }, "payload": { "attributes": attributes } })
}

fn commit_count(model: &Path) -> usize {
    git(model.parent().unwrap(), &["rev-list", "--count", "HEAD"]).trim().parse().unwrap()
}

#[tokio::test]
async fn a_token_protected_server_refuses_requests_without_the_token() {
    let (_guard, model) = repository();
    let app = server(&model, true, false);

    let (missing, body) = call(&app, "GET", "/projects", None, None).await;
    let (wrong, _) = call(&app, "GET", "/projects", Some("not-the-token-not-the-token-xx"), None).await;
    let (right, projects) = call(&app, "GET", "/projects", Some(TOKEN), None).await;

    assert_eq!(missing, StatusCode::UNAUTHORIZED);
    assert_eq!(body["@type"], "Error");
    assert_eq!(wrong, StatusCode::UNAUTHORIZED);
    assert_eq!(right, StatusCode::OK);
    assert_eq!(projects[0]["name"], "Brakes");
}

#[tokio::test]
async fn the_token_also_guards_the_other_api_routes_but_not_health() {
    let (_guard, model) = repository();
    let app = server(&model, true, false);
    let get = |path: &'static str| {
        let app = app.clone();
        async move { app.oneshot(Request::builder().uri(path).body(Body::empty()).unwrap()).await.unwrap().status() }
    };

    assert_eq!(get("/health").await, StatusCode::OK);
    assert_eq!(get("/api/metamodel").await, StatusCode::UNAUTHORIZED);
    assert_eq!(get("/api/sysml-v2/projects").await, StatusCode::UNAUTHORIZED);
}

#[test]
fn writes_cannot_be_enabled_without_a_token_and_short_tokens_are_refused() {
    let mut workspace = Workspace::default();

    assert!(workspace.allow_writes().unwrap_err().contains("token"));
    assert!(workspace.require_token("short").unwrap_err().contains("at least"));
    workspace.require_token(TOKEN).unwrap();
    workspace.allow_writes().unwrap();
    assert!(workspace.access.allows_write());
}

#[tokio::test]
async fn a_server_not_started_for_writing_refuses_commits() {
    let (_guard, model) = repository();
    let app = server(&model, true, false);
    let (project, _, element) = locate(&app, "LC-001").await;

    let body = json!({ "change": [change(&element, json!({ "latency": { "value": 10, "unit": "ms" } }))] });
    let (status, _) = call(&app, "POST", &format!("/projects/{project}/commits"), Some(TOKEN), Some(body)).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(std::fs::read_to_string(&model).unwrap(), MODEL);
    assert_eq!(commit_count(&model), 1);
}

#[tokio::test]
async fn a_write_edits_the_file_losslessly_and_makes_one_git_commit() {
    let (_guard, model) = repository();
    let app = server(&model, true, true);
    let (project, head, element) = locate(&app, "LC-001").await;

    let body = json!({
        "@type": "Commit",
        "description": "Tighten the controller latency",
        "previousCommit": { "@id": head },
        "change": [change(&element, json!({
            "latency": { "@type": "Quantity", "value": 10, "unit": "ms" },
            "owner": "Chassis \"A\" team",
            "safety_level": null
        }))]
    });
    let (status, commit) = call(&app, "POST", &format!("/projects/{project}/commits"), Some(TOKEN), Some(body)).await;

    assert_eq!(status, StatusCode::CREATED, "{commit}");
    assert_eq!(commit["@type"], "Commit");
    assert_eq!(commit["description"], "Tighten the controller latency");
    assert_eq!(commit["previousCommit"], json!([{ "@id": head }]));

    // The file: only the three attributes moved; both comments are intact.
    let expected = MODEL
        .replace("latency: 25 ms", "latency: 10 ms")
        .replace("        safety_level: \"ASIL-B\"\n", "        owner: \"Chassis \\\"A\\\" team\"\n");
    assert_eq!(std::fs::read_to_string(&model).unwrap(), expected);

    // Git: exactly one more commit, with the request's description, clean tree.
    let root = model.parent().unwrap();
    assert_eq!(commit_count(&model), 2);
    assert_eq!(git(root, &["log", "-1", "--format=%s"]).trim(), "Tighten the controller latency");
    assert_eq!(git(root, &["log", "-1", "--format=%an"]).trim(), "Model Server");
    assert!(!git(root, &["log", "-1", "--format=%B"]).contains("Co-Authored-By"));
    assert_eq!(git(root, &["status", "--porcelain"]).trim(), "");
    assert_eq!(commit["arclang:gitCommit"], git(root, &["rev-parse", "HEAD"]).trim());

    // The API: the new commit is the head and shows the values.
    let new_head = commit["@id"].as_str().unwrap();
    let (_, commits) = call(&app, "GET", &format!("/projects/{project}/commits"), Some(TOKEN), None).await;
    assert_eq!(commits[0]["@id"], new_head);
    let (_, record) = call(&app, "GET", &format!("/projects/{project}/commits/{new_head}/elements/{element}"), Some(TOKEN), None).await;
    assert_eq!(record["attributes"]["latency"]["value"], 10.0);
    assert_eq!(record["attributes"]["owner"], "Chassis \"A\" team");
    assert_eq!(record["attributes"].get("safety_level"), None);
    let (_, changes) = call(&app, "GET", &format!("/projects/{project}/commits/{new_head}/changes"), Some(TOKEN), None).await;
    let changed: Vec<&str> = changes.as_array().unwrap().iter().map(|c| c["identity"]["@id"].as_str().unwrap()).collect();
    assert_eq!(changed, [element.as_str()], "one element changed, nothing else");
}

#[tokio::test]
async fn sending_back_a_record_read_from_the_api_with_one_value_changed_works() {
    let (_guard, model) = repository();
    let app = server(&model, true, true);
    let (project, head, element) = locate(&app, "LC-002").await;
    let (_, mut record) = call(&app, "GET", &format!("/projects/{project}/commits/{head}/elements/{element}"), Some(TOKEN), None).await;

    record["attributes"]["latency"] = json!({ "value": 5, "unit": "ms" });
    let body = json!({ "change": [{ "identity": { "@id": element }, "payload": record }] });
    let (status, commit) = call(&app, "POST", &format!("/projects/{project}/commits"), Some(TOKEN), Some(body)).await;

    assert_eq!(status, StatusCode::CREATED, "{commit}");
    assert!(std::fs::read_to_string(&model).unwrap().contains("component \"Monitor\" { id: \"LC-002\" latency: 5 ms }"));
    assert_eq!(commit["description"], "Set LC-002.latency through the ArcLang API");
}

#[tokio::test]
async fn refused_writes_leave_the_file_and_the_history_untouched() {
    let (_guard, model) = repository();
    let app = server(&model, true, true);
    let (project, head, element) = locate(&app, "LC-001").await;
    let (_, _, model_root) = locate(&app, "Brakes").await;
    let commits = format!("/projects/{project}/commits");
    let post = |body: Value| {
        let (app, commits) = (app.clone(), commits.clone());
        async move { call(&app, "POST", &commits, Some(TOKEN), Some(body)).await }
    };

    let cases = [
        ("unknown unit", json!({ "change": [change(&element, json!({ "latency": { "value": 1, "unit": "parsecs" } }))] }), StatusCode::UNPROCESSABLE_ENTITY),
        ("stale base", json!({ "previousCommit": { "@id": "not-the-head" }, "change": [change(&element, json!({ "latency": { "value": 1, "unit": "ms" } }))] }), StatusCode::CONFLICT),
        ("deleting the model", json!({ "change": [{ "identity": { "@id": model_root }, "payload": null }] }), StatusCode::UNPROCESSABLE_ENTITY),
        ("deleting nothing", json!({ "change": [{ "identity": { "@id": "00000000-0000-0000-0000-000000000000" }, "payload": null }] }), StatusCode::BAD_REQUEST),
        ("creation without a kind", json!({ "change": [change("00000000-0000-0000-0000-000000000000", json!({ "x": 1 }))] }), StatusCode::BAD_REQUEST),
        ("creation of an unsupported kind", json!({ "change": [{ "payload": { "@type": "Hazard", "name": "H" } }] }), StatusCode::UNPROCESSABLE_ENTITY),
        ("creation in a layer the file lacks", json!({ "change": [{ "payload": { "@type": "PhysicalNode", "name": "ECU" } }] }), StatusCode::UNPROCESSABLE_ENTITY),
        ("creation with an invalid value", json!({ "change": [{ "payload": { "@type": "LogicalComponent", "name": "X", "attributes": { "latency": { "value": 1, "unit": "parsecs" } } } }] }), StatusCode::UNPROCESSABLE_ENTITY),
        ("creation under a wrong owner", json!({ "change": [{ "payload": { "@type": "PhysicalNode", "name": "ECU", "owner": { "@id": element } } }] }), StatusCode::UNPROCESSABLE_ENTITY),
        ("trace of an unknown kind", json!({ "change": [{ "payload": { "@type": "Trace", "traceKind": "contradicts", "source": [{ "@id": element }], "target": [{ "@id": element }] } }] }), StatusCode::UNPROCESSABLE_ENTITY),
        ("trace without ends", json!({ "change": [{ "payload": { "@type": "Trace", "traceKind": "refines" } }] }), StatusCode::BAD_REQUEST),
        ("change of kind", json!({ "change": [{ "identity": { "@id": element }, "payload": { "@type": "PhysicalNode" } }] }), StatusCode::UNPROCESSABLE_ENTITY),
        ("change of owner", json!({ "change": [{ "identity": { "@id": element }, "payload": { "owner": { "@id": element } } }] }), StatusCode::UNPROCESSABLE_ENTITY),
        ("no change", json!({ "change": [change(&element, json!({ "safety_level": "ASIL-B" }))] }), StatusCode::UNPROCESSABLE_ENTITY),
        ("empty", json!({ "change": [] }), StatusCode::BAD_REQUEST),
        ("injection", json!({ "change": [change(&element, json!({ "latency": { "value": 1, "unit": "ms }\ncomponent \"X\" {" } }))] }), StatusCode::UNPROCESSABLE_ENTITY),
        // All or nothing: a valid change beside an invalid one is not applied.
        ("partial", json!({ "change": [
            change(&element, json!({ "latency": { "value": 1, "unit": "ms" } })),
            change(&model_root, json!({ "nope": { "value": 1, "unit": "parsecs" } }))
        ] }), StatusCode::UNPROCESSABLE_ENTITY),
    ];
    for (name, body, expected) in cases {
        let (status, error) = post(body).await;
        assert_eq!(status, expected, "{name}: {error}");
        assert_eq!(error["@type"], "Error", "{name}");
        assert_eq!(std::fs::read_to_string(&model).unwrap(), MODEL, "{name} wrote the file");
    }

    assert_eq!(commit_count(&model), 1);
    let (_, commits) = call(&app, "GET", &commits, Some(TOKEN), None).await;
    assert_eq!(commits[0]["@id"], head.as_str());
    let leftovers: Vec<_> = std::fs::read_dir(model.parent().unwrap())
        .unwrap()
        .filter_map(|entry| entry.ok().map(|e| e.file_name().to_string_lossy().into_owned()))
        .filter(|name| name.ends_with(".edit.arc"))
        .collect();
    assert!(leftovers.is_empty(), "scratch files left: {leftovers:?}");
}

#[tokio::test]
async fn a_file_with_uncommitted_changes_is_not_written() {
    let (_guard, model) = repository();
    let app = server(&model, true, true);
    let (project, _, element) = locate(&app, "LC-001").await;
    let dirty = MODEL.replace("worst case", "worst case, measured");
    std::fs::write(&model, &dirty).unwrap();

    let body = json!({ "change": [change(&element, json!({ "latency": { "value": 10, "unit": "ms" } }))] });
    let (status, error) = call(&app, "POST", &format!("/projects/{project}/commits"), Some(TOKEN), Some(body)).await;

    assert_eq!(status, StatusCode::CONFLICT, "{error}");
    assert!(error["description"].as_str().unwrap().contains("uncommitted"));
    assert_eq!(std::fs::read_to_string(&model).unwrap(), dirty);
    assert_eq!(commit_count(&model), 1);
}

#[tokio::test]
async fn two_writes_in_a_row_make_two_commits_and_a_stale_client_is_told() {
    let (_guard, model) = repository();
    let app = server(&model, true, true);
    let (project, first_head, element) = locate(&app, "LC-001").await;
    let commits = format!("/projects/{project}/commits");
    let set = |value: u32, base: String| json!({
        "previousCommit": { "@id": base },
        "change": [change(&element, json!({ "latency": { "value": value, "unit": "ms" } }))]
    });

    let (first, commit) = call(&app, "POST", &commits, Some(TOKEN), Some(set(10, first_head.clone()))).await;
    let (stale, _) = call(&app, "POST", &commits, Some(TOKEN), Some(set(11, first_head))).await;
    let new_head = commit["@id"].as_str().unwrap().to_string();
    let (second, _) = call(&app, "POST", &commits, Some(TOKEN), Some(set(12, new_head))).await;

    assert_eq!((first, stale, second), (StatusCode::CREATED, StatusCode::CONFLICT, StatusCode::CREATED));
    assert_eq!(commit_count(&model), 3);
    assert!(std::fs::read_to_string(&model).unwrap().contains("latency: 12 ms   // worst case"));
}

/// (short name or name, uuid) of every element, and every trace record.
async fn head_state(app: &Router, project: &str) -> (String, Vec<Value>) {
    let (_, commits) = call(app, "GET", &format!("/projects/{project}/commits"), Some(TOKEN), None).await;
    let head = commits[0]["@id"].as_str().unwrap().to_string();
    let (_, elements) = call(app, "GET", &format!("/projects/{project}/commits/{head}/elements"), Some(TOKEN), None).await;
    (head, elements.as_array().unwrap().clone())
}

#[tokio::test]
async fn an_element_is_created_in_its_layer_and_inside_another_element() {
    let (_guard, model) = repository();
    let app = server(&model, true, true);
    let (project, _, controller) = locate(&app, "LC-001").await;
    let commits = format!("/projects/{project}/commits");

    let body = json!({
        "description": "Add a logger and the controller's decision function",
        "change": [
            { "payload": { "@type": "LogicalComponent", "name": "Logger", "attributes": { "id": "LC-003", "latency": { "value": 5, "unit": "ms" } } } },
            { "payload": { "@type": "LogicalFunction", "name": "Decide", "owner": { "@id": controller }, "attributes": { "id": "LF-001" } } }
        ]
    });
    let (status, commit) = call(&app, "POST", &commits, Some(TOKEN), Some(body)).await;

    assert_eq!(status, StatusCode::CREATED, "{commit}");
    let text = std::fs::read_to_string(&model).unwrap();
    assert!(text.contains("    component \"Monitor\" { id: \"LC-002\" }\n    component \"Logger\" {\n        id: \"LC-003\"\n        latency: 5 ms\n    }\n}"), "{text}");
    assert!(text.contains("        safety_level: \"ASIL-B\"\n        function \"Decide\" {\n            id: \"LF-001\"\n        }\n    }"), "{text}");
    assert!(text.contains("// The controller decides.") && text.contains("// worst case"));
    assert_eq!(commit_count(&model), 2);

    let (head, elements) = head_state(&app, &project).await;
    let logger = elements.iter().find(|e| e["shortName"] == "LC-003").expect("the logger is served");
    assert_eq!((&logger["@type"], &logger["name"]), (&json!("LogicalComponent"), &json!("Logger")));
    assert_eq!(logger["attributes"]["latency"]["value"], 5.0);
    let decide = elements.iter().find(|e| e["shortName"] == "LF-001").unwrap();
    assert_eq!(decide["owner"], json!({ "@id": controller }));
    let (_, changes) = call(&app, "GET", &format!("/projects/{project}/commits/{head}/changes"), Some(TOKEN), None).await;
    let mut added: Vec<&str> = changes.as_array().unwrap().iter().filter(|c| c["arclang:changeKind"] == "added").map(|c| c["payload"]["name"].as_str().unwrap()).collect();
    added.sort();
    assert_eq!(added, ["Decide", "Logger"], "two elements added, nothing else");
}

#[tokio::test]
async fn an_element_is_deleted_with_its_content_and_nothing_else() {
    let (_guard, model) = repository();
    let app = server(&model, true, true);
    let (project, _, monitor) = locate(&app, "LC-002").await;

    let body = json!({ "change": [{ "identity": { "@id": monitor }, "payload": null }] });
    let (status, commit) = call(&app, "POST", &format!("/projects/{project}/commits"), Some(TOKEN), Some(body)).await;

    assert_eq!(status, StatusCode::CREATED, "{commit}");
    assert_eq!(commit["description"], "Remove LC-002 through the ArcLang API");
    assert_eq!(std::fs::read_to_string(&model).unwrap(), MODEL.replace("    component \"Monitor\" { id: \"LC-002\" }\n", ""));
    let (_, elements) = head_state(&app, &project).await;
    assert!(elements.iter().all(|e| e["shortName"] != "LC-002"));
    assert!(elements.iter().any(|e| e["shortName"] == "LC-001"));
}

const TRACED: &str = r#"model Brakes {}

system_analysis "Braking" {
    requirement "REQ-1" { description: "Stop in time" }
    requirement "REQ-2" { description: "Warn the driver" }
}

logical_architecture "Logical" {
    component "Controller" { id: "LC-1" }
}

// Why the controller exists.
trace "LC-1" satisfies "REQ-1" {}
"#;

fn traced_repository() -> (tempfile::TempDir, PathBuf) {
    let (directory, model) = repository();
    std::fs::write(&model, TRACED).unwrap();
    git(directory.path(), &["commit", "-q", "-am", "traced model"]);
    (directory, model)
}

#[tokio::test]
async fn an_element_something_still_refers_to_cannot_be_deleted() {
    let (_guard, model) = traced_repository();
    let app = server(&model, true, true);
    let (project, _, requirement) = locate(&app, "REQ-1").await;

    let body = json!({ "change": [{ "identity": { "@id": requirement }, "payload": null }] });
    let (status, error) = call(&app, "POST", &format!("/projects/{project}/commits"), Some(TOKEN), Some(body)).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{error}");
    assert_eq!(std::fs::read_to_string(&model).unwrap(), TRACED);
    assert_eq!(commit_count(&model), 2);
}

#[tokio::test]
async fn a_trace_is_deleted_then_its_end_and_another_trace_is_created() {
    let (_guard, model) = traced_repository();
    let app = server(&model, true, true);
    let (project, _, requirement) = locate(&app, "REQ-1").await;
    let (_, _, other) = locate(&app, "REQ-2").await;
    let (_, _, controller) = locate(&app, "LC-1").await;
    let (_, elements) = head_state(&app, &project).await;
    let trace = elements.iter().find(|e| e["@type"] == "Trace").unwrap()["@id"].as_str().unwrap().to_string();
    let commits = format!("/projects/{project}/commits");

    // One request: drop the trace and the requirement it led to, and trace
    // the controller to the other requirement instead.
    let body = json!({ "change": [
        { "identity": { "@id": trace }, "payload": null },
        { "identity": { "@id": requirement }, "payload": null },
        { "payload": { "@type": "Trace", "traceKind": "satisfies", "source": [{ "@id": controller }], "target": [{ "@id": other }] } }
    ] });
    let (status, commit) = call(&app, "POST", &commits, Some(TOKEN), Some(body)).await;

    assert_eq!(status, StatusCode::CREATED, "{commit}");
    let text = std::fs::read_to_string(&model).unwrap();
    assert!(!text.contains("REQ-1"), "{text}");
    assert!(text.contains("// Why the controller exists."), "the comment above the removed trace stays");
    assert!(text.ends_with("trace \"LC-1\" satisfies \"REQ-2\" {}\n"), "{text}");
    assert_eq!(commit_count(&model), 3);

    let (_, elements) = head_state(&app, &project).await;
    let traces: Vec<&Value> = elements.iter().filter(|e| e["@type"] == "Trace").collect();
    assert_eq!(traces.len(), 1);
    assert_eq!(traces[0]["target"], json!([{ "@id": other }]));
    assert!(elements.iter().all(|e| e["shortName"] != "REQ-1"));
}

const READER: &str = "reader-token-0123456789abcdef-0123";
const WRITER: &str = "writer-token-0123456789abcdef-0123";

/// A server with two named users: `ines` reads, `malek` reads and writes.
/// The writer's token is declared by its digest only.
fn server_with_users(model: &Path) -> Router {
    use sha2::{Digest, Sha256};
    let digest: String = Sha256::digest(WRITER.as_bytes()).iter().map(|byte| format!("{byte:02x}")).collect();
    let users = format!("# who may use the API\nines read {READER}\n\nmalek write sha256:{digest}  # digest only\n");
    let mut workspace = Workspace::default();
    workspace.add_file_with_history(model, 50).expect("model loads");
    assert_eq!(workspace.add_users(&users), Ok(2));
    workspace.allow_writes().unwrap();
    arclang::web_server::build_router_with(workspace)
}

#[tokio::test]
async fn a_reader_reads_and_only_a_writer_writes() {
    let (_guard, model) = repository();
    let app = server_with_users(&model);
    let (_, projects) = call(&app, "GET", "/projects", Some(READER), None).await;
    let project = projects[0]["@id"].as_str().unwrap().to_string();
    let commits = format!("/projects/{project}/commits");
    let (_, listed) = call(&app, "GET", &commits, Some(WRITER), None).await;
    let head = listed[0]["@id"].as_str().unwrap().to_string();
    let (_, elements) = call(&app, "GET", &format!("{commits}/{head}/elements"), Some(READER), None).await;
    let element = elements.as_array().unwrap().iter().find(|e| e["shortName"] == "LC-001").unwrap()["@id"].as_str().unwrap().to_string();
    let body = json!({ "description": "Tighten latency", "change": [change(&element, json!({ "latency": { "value": 10, "unit": "ms" } }))] });

    let (anonymous, _) = call(&app, "POST", &commits, None, Some(body.clone())).await;
    let (reader, error) = call(&app, "POST", &commits, Some(READER), Some(body.clone())).await;
    assert_eq!(anonymous, StatusCode::UNAUTHORIZED);
    assert_eq!(reader, StatusCode::FORBIDDEN);
    assert!(error["description"].as_str().unwrap().contains("user 'ines' may read but not write"), "{error}");
    assert_eq!(std::fs::read_to_string(&model).unwrap(), MODEL);

    let (writer, commit) = call(&app, "POST", &commits, Some(WRITER), Some(body)).await;
    assert_eq!(writer, StatusCode::CREATED, "{commit}");
    assert_eq!(commit["arclang:user"], "malek");

    // The commit message is exactly the request's description: the user is
    // not written into it. A git note keeps the name for later servers.
    let root = model.parent().unwrap();
    assert_eq!(git(root, &["log", "-1", "--format=%B"]).trim(), "Tighten latency");
    assert_eq!(git(root, &["notes", "--ref", "arclang-user", "show", "HEAD"]).trim(), "malek");
    let restarted = server_with_users(&model);
    let (_, listed) = call(&restarted, "GET", &commits, Some(READER), None).await;
    assert_eq!(listed[0]["arclang:user"], "malek");
    assert_eq!(listed[1]["arclang:user"], Value::Null, "the initial commit was not made through the API");
}

#[test]
fn a_users_file_is_checked_line_by_line() {
    let load = |text: &str| Workspace::default().add_users(text);
    let token = "0123456789abcdef0123456789abcdef";

    assert!(load(&format!("ines read {token}\nines write {token}x")).unwrap_err().contains("declared twice"));
    assert!(load(&format!("ines read {token}\nmalek write {token}")).unwrap_err().contains("same token"));
    assert!(load("ines read short").unwrap_err().contains("line 1"));
    assert!(load(&format!("ines admin {token}")).unwrap_err().contains("neither read nor write"));
    assert!(load(&format!("ines read {token} extra")).unwrap_err().contains("expected `name role token`"));
    assert!(load(&format!("in/es read {token}")).unwrap_err().contains("not a user name"));
    assert!(load("ines read sha256:abc").unwrap_err().contains("64 hexadecimal"));

    // Readers alone cannot make a server writable.
    let mut readers = Workspace::default();
    readers.add_users(&format!("ines read {token}")).unwrap();
    assert!(readers.allow_writes().unwrap_err().contains("write role"));
}

#[tokio::test]
async fn an_element_with_an_id_is_renamed_and_keeps_its_identity() {
    let (_guard, model) = repository();
    let app = server(&model, true, true);
    let (project, head, element) = locate(&app, "LC-001").await;
    let (_, mut record) = call(&app, "GET", &format!("/projects/{project}/commits/{head}/elements/{element}"), Some(TOKEN), None).await;

    record["name"] = json!("Brake controller");
    record["declaredName"] = json!("Brake controller");
    let body = json!({ "change": [{ "identity": { "@id": element }, "payload": record }] });
    let (status, commit) = call(&app, "POST", &format!("/projects/{project}/commits"), Some(TOKEN), Some(body)).await;

    assert_eq!(status, StatusCode::CREATED, "{commit}");
    assert_eq!(commit["description"], "Rename LC-001 to 'Brake controller' through the ArcLang API");
    assert_eq!(std::fs::read_to_string(&model).unwrap(), MODEL.replace("component \"Controller\" {", "component \"Brake controller\" {"));
    let (new_head, elements) = head_state(&app, &project).await;
    let renamed = elements.iter().find(|e| e["@id"] == element.as_str()).expect("same identity");
    assert_eq!(renamed["name"], "Brake controller");
    let (_, changes) = call(&app, "GET", &format!("/projects/{project}/commits/{new_head}/changes"), Some(TOKEN), None).await;
    assert!(changes.as_array().unwrap().iter().all(|c| c["arclang:changeKind"] == "modified"), "nothing added or deleted: {changes}");
}

const NAMED: &str = r#"model Brakes {}

system_analysis "Braking" {
    requirement "REQ-1" { description: "Stop in time" }
}

logical_architecture "Logical" {
    component "Controller" {}
    component "Monitor" { id: "LC-2" }
}

trace "Monitor" satisfies "REQ-1" {}
"#;

#[tokio::test]
async fn a_rename_that_would_change_an_identity_or_break_a_reference_is_refused() {
    let (directory, model) = repository();
    std::fs::write(&model, NAMED).unwrap();
    git(directory.path(), &["commit", "-q", "-am", "named model"]);
    let app = server(&model, true, true);
    let (_, projects) = call(&app, "GET", "/projects", Some(TOKEN), None).await;
    let project = projects[0]["@id"].as_str().unwrap().to_string();
    let (_, elements) = head_state(&app, &project).await;
    let id_of = |name: &str| elements.iter().find(|e| e["name"] == name).unwrap()["@id"].as_str().unwrap().to_string();

    // "Controller" writes no id: its name is its identity.
    // "Monitor" has an id, but a trace knows it by its name.
    for (name, identity) in [("Controller", id_of("Controller")), ("Monitor", id_of("Monitor"))] {
        let body = json!({ "change": [{ "identity": { "@id": identity }, "payload": { "name": "Renamed" } }] });
        let (status, error) = call(&app, "POST", &format!("/projects/{project}/commits"), Some(TOKEN), Some(body)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{name}: {error}");
    }
    assert_eq!(std::fs::read_to_string(&model).unwrap(), NAMED);
    assert_eq!(commit_count(&model), 2);
}
