//! In-process tests of the Systems Modeling API (read-only REST binding).

use arclang::web_server::systems_modeling::Workspace;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::{TimeZone, Utc};
use serde_json::{json, Value};
use tower::util::ServiceExt;

const BASE: &str = "/api/systems-modeling";

const MODEL: &str = r#"
model Lane {
    metadata { description: "Lane keeping" }
}
type "ECU" { voltage: 12 V }
type "Safety ECU" extends "ECU" { safety_level: "ASIL-B" }
system_analysis "SA" {
    requirement "REQ-1" { description: "react fast" priority: "Critical" }
    function "Detect" { id: "SF-1" latency: 30 ms port out lane { data_type: "Lane" } }
    function "Decide" { id: "SF-2" latency: 10 ms port in lane { data_type: "Lane" } }
    functional_exchange "SF-1.lane" -> "SF-2.lane" { label: "lane" }
    functional_chain "Chain" { id: "FC-1" involves: ["SF-1", "SF-2"] latency_budget: 50 ms }
}
architecture logical {
    component "Controller" { id: "LC-1" function "Plan" component "Core" { id: "LC-1-1" } }
}
architecture physical {
    node "Chassis ECU" { id: "PN-1" is: "Safety ECU" deploys "LC-1" }
}
constraint "Budget" { id: "CST-1" assert: sum("FC-1", latency) <= "FC-1".latency_budget }
test_case "TC-1" { verifies: ["REQ-1"] method: "test" }
trace "LC-1" satisfies "REQ-1"
"#;

fn workspace(source: &str) -> Workspace {
    let mut workspace = Workspace::default();
    workspace
        .add_source(source, Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap())
        .expect("model compiles");
    workspace
}

async fn call(source: &str, method: &str, path: &str, body: Option<Value>) -> (StatusCode, Option<String>, Value) {
    let app = arclang::web_server::build_router_with(workspace(source));
    let request = Request::builder().method(method).uri(format!("{BASE}{path}"));
    let request = match body {
        Some(body) => request.header("content-type", "application/json").body(Body::from(body.to_string())),
        None => request.body(Body::empty()),
    }
    .unwrap();
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let link = response.headers().get("link").map(|v| v.to_str().unwrap().to_string());
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, link, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

async fn get(path: &str) -> Value {
    let (status, _, json) = call(MODEL, "GET", path, None).await;
    assert_eq!(status, StatusCode::OK, "GET {path}: {json}");
    json
}

/// (project id, commit id) of the single served model.
async fn ids() -> (String, String) {
    let projects = get("/projects").await;
    let project = projects[0]["@id"].as_str().unwrap().to_string();
    let commits = get(&format!("/projects/{project}/commits")).await;
    (project, commits[0]["@id"].as_str().unwrap().to_string())
}

#[tokio::test]
async fn project_branch_and_commit_follow_the_resource_model() {
    let projects = get("/projects").await;
    assert_eq!(projects.as_array().unwrap().len(), 1);
    let project = &projects[0];
    assert_eq!(project["@type"], "Project");
    assert_eq!(project["name"], "Lane");
    assert_eq!(project["description"], "Lane keeping");
    assert_eq!(project["created"], "2026-01-02T03:04:05Z");
    let project_id = project["@id"].as_str().unwrap();

    assert_eq!(get(&format!("/projects/{project_id}")).await, *project);

    let branch_id = project["defaultBranch"]["@id"].as_str().unwrap();
    let branch = get(&format!("/projects/{project_id}/branches/{branch_id}")).await;
    assert_eq!((&branch["@type"], &branch["name"]), (&json!("Branch"), &json!("main")));
    assert_eq!(branch["owningProject"]["@id"], project_id);

    let commit_id = branch["head"]["@id"].as_str().unwrap();
    let commit = get(&format!("/projects/{project_id}/commits/{commit_id}")).await;
    assert_eq!(commit["@type"], "Commit");
    assert_eq!(commit["previousCommit"], json!([]));
    assert_eq!(commit["arclang:unresolvedRelationships"], json!([]));
}

#[tokio::test]
async fn commit_id_is_content_addressed() {
    let commit_of = |source: &'static str| async move {
        let (_, _, projects) = call(source, "GET", "/projects", None).await;
        let project = projects[0]["@id"].as_str().unwrap().to_string();
        let (_, _, commits) = call(source, "GET", &format!("/projects/{project}/commits"), None).await;
        (project, commits[0]["@id"].as_str().unwrap().to_string())
    };
    const A: &str = "model M {}\narchitecture logical { component \"C\" { id: \"LC-1\" ram: 1 MB } }";
    const REFORMATTED: &str = "model M {}\n\n// a comment\narchitecture logical {\n  component \"C\" { ram: 1 MB id: \"LC-1\" }\n}";
    const CHANGED: &str = "model M {}\narchitecture logical { component \"C\" { id: \"LC-1\" ram: 2 MB } }";
    let (project_a, commit_a) = commit_of(A).await;
    let (project_b, commit_b) = commit_of(REFORMATTED).await;
    let (project_c, commit_c) = commit_of(CHANGED).await;
    assert_eq!(project_a, project_b);
    assert_eq!(project_a, project_c, "the project is the model, whatever its content");
    assert_eq!(commit_a, commit_b, "formatting and attribute order do not make a new snapshot");
    assert_ne!(commit_a, commit_c, "a changed value does");
}

#[tokio::test]
async fn elements_carry_identity_ownership_and_typed_attributes() {
    let (project, commit) = ids().await;
    let elements = get(&format!("/projects/{project}/commits/{commit}/elements")).await;
    let elements = elements.as_array().unwrap();
    let find = |short: &str| elements.iter().find(|e| e["shortName"] == short).unwrap_or_else(|| panic!("no {short}"));

    let function = find("SF-1");
    assert_eq!(function["@type"], "SystemFunction");
    assert_eq!(function["qualifiedName"], "Lane::Detect");
    assert_eq!(function["@id"], function["elementId"]);
    let latency = &function["attributes"]["latency"];
    assert_eq!(latency["@type"], "Quantity");
    assert_eq!((&latency["value"], &latency["unit"]), (&json!(30.0), &json!("ms")));
    assert_eq!((&latency["canonicalValue"], &latency["canonicalUnit"]), (&json!(0.03), &json!("s")));

    // Same UUID as /api/compile and every export.
    assert_eq!(find("LC-1")["@id"], json!(arclang::compiler::identity::element_uuid("element", "LC-1")));

    let nested = find("LC-1-1");
    assert_eq!(nested["owner"]["@id"], find("LC-1")["@id"]);
    assert_eq!(nested["qualifiedName"], "Lane::Controller::Core");
    assert!(find("LC-1")["ownedElement"].as_array().unwrap().iter().any(|r| r["@id"] == nested["@id"]));

    // Inherited through the type: effective value.
    assert_eq!(find("PN-1")["attributes"]["voltage"]["canonicalValue"], json!(12.0));
    assert_eq!(find("PN-1")["attributes"]["safety_level"], "ASIL-B");

    let constraint = find("CST-1");
    assert_eq!(constraint["expression"], "sum(\"FC-1\", latency) <= \"FC-1\".latency_budget");
    assert_eq!((&constraint["satisfied"], &constraint["left"], &constraint["right"]), (&json!(true), &json!("40 ms"), &json!("50 ms")));

    // One element by id, and the root.
    let by_id = get(&format!("/projects/{project}/commits/{commit}/elements/{}", function["@id"].as_str().unwrap())).await;
    assert_eq!(&by_id, function);
    let roots = get(&format!("/projects/{project}/commits/{commit}/roots")).await;
    assert_eq!(roots.as_array().unwrap().len(), 1);
    assert_eq!((&roots[0]["@type"], &roots[0]["owner"]), (&json!("Model"), &Value::Null));

    // Every @type is a kind of the published metamodel, or a relationship type.
    let metamodel = arclang::compiler::metamodel::Metamodel::current();
    for element in elements {
        let kind = element["@type"].as_str().unwrap();
        let known = metamodel.kind(kind).is_some() || metamodel.relationship(kind).is_some();
        assert!(known, "@type '{kind}' is not in the metamodel");
    }
}

#[tokio::test]
async fn relationships_are_navigable_in_both_directions() {
    let (project, commit) = ids().await;
    let elements = get(&format!("/projects/{project}/commits/{commit}/elements")).await;
    let id_of = |short: &str| {
        elements.as_array().unwrap().iter().find(|e| e["shortName"] == short).unwrap()["@id"].as_str().unwrap().to_string()
    };
    let relationships = |element: String, direction: &'static str| {
        let (project, commit) = (project.clone(), commit.clone());
        async move { get(&format!("/projects/{project}/commits/{commit}/elements/{element}/relationships?direction={direction}")).await }
    };

    let out = relationships(id_of("LC-1"), "out").await;
    let kinds: Vec<&str> = out.as_array().unwrap().iter().map(|r| r["@type"].as_str().unwrap()).collect();
    assert_eq!(kinds, ["Deployment", "Trace"], "{out}");
    let trace = &out[1];
    assert_eq!(trace["traceKind"], "satisfies");
    assert_eq!(trace["source"][0]["@id"], id_of("LC-1"));
    assert_eq!(trace["target"][0]["@id"], id_of("REQ-1"));
    assert_eq!(trace["relatedElement"].as_array().unwrap().len(), 2);

    let incoming = relationships(id_of("REQ-1"), "in").await;
    let kinds: Vec<&str> = incoming.as_array().unwrap().iter().map(|r| r["@type"].as_str().unwrap()).collect();
    assert_eq!(kinds, ["Verification", "Trace"]);
    assert_eq!(relationships(id_of("REQ-1"), "out").await, json!([]));
    assert_eq!(relationships(id_of("REQ-1"), "both").await.as_array().unwrap().len(), 2);

    // Port-level exchange, and ordered chain membership.
    let exchange = &relationships(id_of("SF-1.lane"), "out").await[0];
    assert_eq!((&exchange["@type"], &exchange["name"]), (&json!("FunctionalExchange"), &json!("lane")));
    assert_eq!(exchange["target"][0]["@id"], id_of("SF-2.lane"));
    let chain = relationships(id_of("FC-1"), "out").await;
    let order: Vec<_> = chain.as_array().unwrap().iter().map(|r| (r["order"].clone(), r["target"][0]["@id"].clone())).collect();
    assert_eq!(order, vec![(json!(1), json!(id_of("SF-1"))), (json!(2), json!(id_of("SF-2")))]);
}

#[tokio::test]
async fn pagination_uses_cursors_and_link_headers() {
    let (project, commit) = ids().await;
    let path = format!("/projects/{project}/commits/{commit}/elements");
    let all = get(&path).await;
    let all = all.as_array().unwrap();
    assert!(all.len() > 6);

    let (status, link, first) = call(MODEL, "GET", &format!("{path}?page[size]=3"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(first.as_array().unwrap()[..], all[..3]);
    let link = link.expect("first page links to the next");
    let cursor = all[2]["@id"].as_str().unwrap();
    assert_eq!(link, format!("<{BASE}{path}?page[after]={cursor}&page[size]=3>; rel=\"next\""));

    let (_, link, second) = call(MODEL, "GET", &format!("{path}?page[after]={cursor}&page[size]=3"), None).await;
    assert_eq!(second.as_array().unwrap()[..], all[3..6]);
    let link = link.unwrap();
    assert!(link.contains("rel=\"next\"") && link.contains("rel=\"prev\""), "{link}");

    let before = all[3]["@id"].as_str().unwrap();
    let (_, _, back) = call(MODEL, "GET", &format!("{path}?page[before]={before}&page[size]=3"), None).await;
    assert_eq!(back.as_array().unwrap()[..], all[..3]);

    // Walking every page yields every record exactly once.
    let mut seen = Vec::new();
    let mut next = format!("{path}?page[size]=4");
    loop {
        let (_, link, page) = call(MODEL, "GET", &next, None).await;
        seen.extend(page.as_array().unwrap().iter().cloned());
        match link.and_then(|l| l.split(", ").find(|part| part.ends_with("rel=\"next\"")).map(str::to_string)) {
            Some(part) => next = part[1..part.find('>').unwrap()].trim_start_matches(BASE).to_string(),
            None => break,
        }
    }
    assert_eq!(&seen, all);

    let (status, _, error) = call(MODEL, "GET", &format!("{path}?page[size]=0"), None).await;
    assert_eq!((status, &error["@type"]), (StatusCode::BAD_REQUEST, &json!("Error")));
    let (status, _, _) = call(MODEL, "GET", &format!("{path}?page[after]=nope"), None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn queries_filter_and_project_records() {
    let (project, _) = ids().await;
    let query = |body: Value| {
        let project = project.clone();
        async move { call(MODEL, "POST", &format!("/projects/{project}/query-results"), Some(body)).await }
    };

    let (status, _, functions) = query(json!({
        "@type": "Query",
        "select": ["name", "attributes.latency.canonicalValue"],
        "where": { "@type": "PrimitiveConstraint", "property": "@type", "operator": "=", "value": "SystemFunction" }
    }))
    .await;
    assert_eq!(status, StatusCode::OK);
    let rows = functions.as_array().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["name"], "Detect");
    assert_eq!(rows[0]["attributes.latency.canonicalValue"], json!(0.03));
    assert!(rows[0]["@id"].is_string() && rows[0].get("@type").is_none(), "only selected properties plus @id");

    // Functions slower than 20 ms: a numeric comparison on the SI value.
    let (_, _, slow) = query(json!({
        "@type": "Query",
        "select": ["shortName"],
        "where": { "@type": "CompositeConstraint", "operator": "and", "constraint": [
            { "@type": "PrimitiveConstraint", "property": "@type", "operator": "instanceOf", "value": "SystemFunction" },
            { "@type": "PrimitiveConstraint", "property": "attributes.latency.canonicalValue", "operator": ">", "value": 0.02 }
        ]}
    }))
    .await;
    assert_eq!(slow, json!([{ "@id": slow[0]["@id"], "shortName": "SF-1" }]));

    // Everything that is NOT satisfied among constraints: none here.
    let (_, _, violated) = query(json!({
        "@type": "Query",
        "where": { "@type": "CompositeConstraint", "operator": "and", "constraint": [
            { "@type": "PrimitiveConstraint", "property": "@type", "operator": "=", "value": "Constraint" },
            { "@type": "PrimitiveConstraint", "property": "satisfied", "operator": "=", "value": true, "inverse": true }
        ]}
    }))
    .await;
    assert_eq!(violated, json!([]));

    let (_, _, traces) = query(json!({
        "@type": "Query",
        "select": ["traceKind", "source.@id"],
        "where": { "@type": "PrimitiveConstraint", "property": "@type", "operator": "in", "value": ["Trace", "Verification"] }
    }))
    .await;
    assert_eq!(traces.as_array().unwrap().len(), 2);

    let (status, _, error) = query(json!({ "@type": "Query", "where": { "@type": "PrimitiveConstraint", "property": "name", "operator": "~", "value": "x" } })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(error["description"].as_str().unwrap().contains("unsupported operator '~'"), "{error}");
}

#[tokio::test]
async fn unknown_resources_are_404_with_an_error_body_and_writes_are_refused() {
    let (project, commit) = ids().await;
    for path in [
        "/projects/nope".to_string(),
        format!("/projects/{project}/commits/nope"),
        format!("/projects/{project}/commits/{commit}/elements/nope"),
        format!("/projects/{project}/commits/{commit}/elements/nope/relationships"),
        format!("/projects/{project}/branches/nope"),
    ] {
        let (status, _, error) = call(MODEL, "GET", &path, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path}");
        assert_eq!(error["@type"], "Error", "{path}");
        assert!(error["description"].as_str().unwrap().contains("not found"), "{path}: {error}");
    }
    // Projects cannot be created; commits only on a server started for it.
    let (status, _, _) = call(MODEL, "POST", "/projects", Some(json!({ "@type": "Project", "name": "X" }))).await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    let (status, _, error) = call(MODEL, "POST", &format!("/projects/{project}/commits"), Some(json!({}))).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(error["description"].as_str().unwrap().contains("read-only"), "{error}");
}

#[tokio::test]
async fn the_flagship_model_is_served_with_nothing_unresolved() {
    let mut workspace = Workspace::default();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/automotive/timing_constraints.arc");
    let project = workspace.add_file(&path).expect("example compiles");
    assert_eq!(project.head().unresolved, Vec::<String>::new());
    let (project_id, commit_id) = (project.id.clone(), project.head().id.clone());
    let app = arclang::web_server::build_router_with(workspace);
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("{BASE}/projects/{project_id}/commits/{commit_id}/elements"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let elements: Value = serde_json::from_slice(&bytes).unwrap();
    let kinds: std::collections::BTreeSet<&str> =
        elements.as_array().unwrap().iter().map(|e| e["@type"].as_str().unwrap()).collect();
    for kind in ["Type", "Specialization", "Typing", "Constraint", "PhysicalLink", "Deployment", "Mitigation"] {
        assert!(kinds.contains(kind), "missing {kind} in {kinds:?}");
    }
}

// ---- Git history ---------------------------------------------------------

mod history {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    fn git(directory: &Path, args: &[&str]) {
        let output = Command::new("git")
            .arg("-C")
            .arg(directory)
            .args(["-c", "user.name=Test", "-c", "user.email=test@example.org", "-c", "commit.gpgsign=false"])
            .args(args)
            .env("GIT_AUTHOR_DATE", "2026-03-01T10:00:00Z")
            .env("GIT_COMMITTER_DATE", "2026-03-01T10:00:00Z")
            .output()
            .expect("git is available");
        assert!(output.status.success(), "git {:?}: {}", args, String::from_utf8_lossy(&output.stderr));
    }

    fn commit(directory: &Path, files: &[(&str, &str)], message: &str) {
        for (name, content) in files {
            let path = directory.join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, content).unwrap();
        }
        git(directory, &["add", "-A"]);
        git(directory, &["commit", "-q", "-m", message]);
    }

    const V1: &str = "model Braking {}\nimport \"parts/hw.arc\"\narchitecture logical {\n  component \"Controller\" { id: \"LC-1\" ram: 64 MB }\n  component \"Legacy\" { id: \"LC-9\" }\n}\n";
    const HW: &str = "architecture physical {\n  node \"ECU\" { id: \"PN-1\" }\n}\n";
    // v2: more RAM, Legacy removed, a sensor added.
    const V2: &str = "model Braking {}\nimport \"parts/hw.arc\"\narchitecture logical {\n  component \"Controller\" { id: \"LC-1\" ram: 128 MB }\n  component \"Sensor\" { id: \"LC-2\" }\n}\n";
    // v3 does not compile.
    const V3: &str = "model Braking {}\nimport \"parts/hw.arc\"\narchitecture logical {\n  component \"Controller\" { id: \"LC-1\" ram: 128 furlongs }\n}\n";
    // v4: reformatted v2 (same content), then an uncommitted rename.
    const V4: &str = "model Braking {}\nimport \"parts/hw.arc\"\n\n// tidy\narchitecture logical {\n  component \"Controller\" { ram: 128 MB id: \"LC-1\" }\n  component \"Sensor\" { id: \"LC-2\" }\n}\n";
    const WORKING: &str = "model Braking {}\nimport \"parts/hw.arc\"\narchitecture logical {\n  component \"Brake controller\" { id: \"LC-1\" ram: 128 MB }\n  component \"Sensor\" { id: \"LC-2\" }\n}\n";

    /// A repository with four commits of `sys/model.arc` and a dirty working tree.
    fn repository() -> (tempfile::TempDir, PathBuf) {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        git(root, &["init", "-q"]);
        commit(root, &[("sys/model.arc", V1), ("sys/parts/hw.arc", HW)], "initial architecture");
        commit(root, &[("sys/model.arc", V2)], "more RAM, replace Legacy by Sensor");
        commit(root, &[("sys/model.arc", V3)], "broken unit");
        commit(root, &[("sys/model.arc", V4)], "fix and tidy");
        std::fs::write(root.join("sys/model.arc"), WORKING).unwrap();
        let model = root.join("sys/model.arc");
        (directory, model)
    }

    async fn fetch(workspace: Workspace, path: &str) -> Value {
        let app = arclang::web_server::build_router_with(workspace);
        let response = app.oneshot(Request::builder().uri(format!("{BASE}{path}")).body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK, "GET {path}");
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    fn served(model: &Path, depth: usize) -> Workspace {
        let mut workspace = Workspace::default();
        workspace.add_file_with_history(model, depth).expect("history loads");
        workspace
    }

    #[tokio::test]
    async fn git_commits_become_api_commits_with_the_working_tree_as_head() {
        let (_guard, model) = repository();
        let project = served(&model, 50).projects.remove(0);
        let project_id = project.id.clone();
        let descriptions: Vec<&str> = project.commits.iter().map(|c| c.description.as_str()).collect();
        assert_eq!(descriptions[..4], ["initial architecture", "more RAM, replace Legacy by Sensor", "broken unit", "fix and tidy"]);
        assert!(descriptions[4].starts_with("Working tree (uncommitted)"), "{descriptions:?}");
        assert!(project.commits[..4].iter().all(|c| c.git_commit.as_deref().map_or(false, |sha| sha.len() == 40)));
        assert_eq!(project.commits[4].git_commit, None);
        // Reformatting made a new git commit but not new content.
        assert_eq!(project.commits[1].digest, project.commits[3].digest);
        assert_ne!(project.commits[1].id, project.commits[3].id, "in a history a commit is a git commit");
        // Imports are resolved at each revision.
        assert!(project.commits[0].compile_error.is_none());

        let commits = fetch(served(&model, 50), &format!("/projects/{project_id}/commits")).await;
        let commits = commits.as_array().unwrap();
        assert_eq!(commits.len(), 5);
        assert!(commits[0]["description"].as_str().unwrap().starts_with("Working tree"), "newest first");
        assert_eq!(commits[4]["description"], "initial architecture");
        assert_eq!(commits[4]["previousCommit"], json!([]));
        assert_eq!(commits[4]["created"], "2026-03-01T10:00:00Z");
        for pair in commits.windows(2) {
            assert_eq!(pair[0]["previousCommit"], json!([{ "@id": pair[1]["@id"] }]));
        }
        // The broken revision is an honest, empty commit.
        let broken = &commits[2];
        assert_eq!(broken["description"], "broken unit");
        assert!(broken["arclang:compileError"].as_str().unwrap().contains("unknown unit 'furlongs'"), "{broken}");
        let elements = fetch(served(&model, 50), &format!("/projects/{project_id}/commits/{}/elements", broken["@id"].as_str().unwrap())).await;
        assert_eq!(elements, json!([]));

        let branches = fetch(served(&model, 50), &format!("/projects/{project_id}/branches")).await;
        assert_eq!(branches[0]["head"]["@id"], commits[0]["@id"]);
    }

    #[tokio::test]
    async fn changes_are_a_diff_by_identity() {
        let (_guard, model) = repository();
        let project = served(&model, 50).projects.remove(0);
        let project_id = project.id.clone();
        let commit_ids: Vec<String> = project.commits.iter().map(|c| c.id.clone()).collect();
        let changes = |position: usize| {
            let (model, project_id, commit) = (model.clone(), project_id.clone(), commit_ids[position].clone());
            async move {
                let changes = fetch(served(&model, 50), &format!("/projects/{project_id}/commits/{commit}/changes")).await;
                let mut summary: Vec<(String, String)> = changes
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|c| {
                        assert_eq!(c["@type"], "DataVersion");
                        assert_eq!(c["identity"]["@type"], "DataIdentity");
                        let kind = c["arclang:changeKind"].as_str().unwrap().to_string();
                        assert_eq!(c["payload"].is_null(), kind == "deleted");
                        let name = match &c["payload"]["shortName"] {
                            Value::String(short) => short.clone(),
                            _ => c["identity"]["@id"].as_str().unwrap().to_string(),
                        };
                        (kind, name)
                    })
                    .collect();
                summary.sort();
                summary
            }
        };
        let uuid = |id: &str| arclang::compiler::identity::element_uuid("element", id);
        let pair = |kind: &str, name: &str| (kind.to_string(), name.to_string());

        // The first commit adds everything it contains.
        let first = changes(0).await;
        assert!(first.iter().all(|(kind, _)| kind == "added"));
        assert!(first.contains(&pair("added", "LC-1")) && first.contains(&pair("added", "PN-1")), "{first:?}");

        // v2: RAM changed on LC-1, LC-9 deleted, LC-2 added; the model root's
        // owned elements changed with them. PN-1 (imported file) is untouched.
        let second = changes(1).await;
        assert_eq!(
            second,
            vec![pair("added", "LC-2"), pair("deleted", &uuid("LC-9")), pair("modified", "Braking"), pair("modified", "LC-1")]
        );

        // v3 does not compile: everything disappears, and comes back in v4.
        assert!(changes(2).await.iter().all(|(kind, _)| kind == "deleted"));
        assert!(changes(3).await.iter().all(|(kind, _)| kind == "added"));

        // Working tree: only the rename of LC-1 — same identity, new name.
        assert_eq!(changes(4).await, vec![pair("modified", "LC-1")]);
    }

    #[tokio::test]
    async fn queries_run_on_any_commit_and_depth_limits_the_history() {
        let (_guard, model) = repository();
        let project = served(&model, 50).projects.remove(0);
        let query = json!({
            "@type": "Query",
            "select": ["name", "attributes.ram.value"],
            "where": { "@type": "PrimitiveConstraint", "property": "shortName", "operator": "=", "value": "LC-1" }
        });
        let run = |commit: Option<String>| {
            let (model, project_id, query) = (model.clone(), project.id.clone(), query.clone());
            async move {
                let app = arclang::web_server::build_router_with(served(&model, 50));
                let suffix = commit.map(|c| format!("?commitId={c}")).unwrap_or_default();
                let response = app
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri(format!("{BASE}/projects/{project_id}/query-results{suffix}"))
                            .header("content-type", "application/json")
                            .body(Body::from(query.to_string()))
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
                serde_json::from_slice::<Value>(&bytes).unwrap()
            }
        };
        let first = run(Some(project.commits[0].id.clone())).await;
        assert_eq!((&first[0]["name"], &first[0]["attributes.ram.value"]), (&json!("Controller"), &json!(64.0)));
        let head = run(None).await;
        assert_eq!((&head[0]["name"], &head[0]["attributes.ram.value"]), (&json!("Brake controller"), &json!(128.0)));

        // Depth 2: the two most recent git commits, plus the working tree.
        let shallow = served(&model, 2).projects.remove(0);
        let descriptions: Vec<&str> = shallow.commits.iter().map(|c| c.description.as_str()).collect();
        assert_eq!(descriptions[..2], ["broken unit", "fix and tidy"]);
        assert_eq!(shallow.commits.len(), 3);
    }

    #[tokio::test]
    async fn a_clean_working_tree_adds_no_commit_and_non_repositories_are_refused() {
        let (_guard, model) = repository();
        std::fs::write(&model, V4).unwrap();
        let project = served(&model, 50).projects.remove(0);
        assert_eq!(project.commits.len(), 4, "working tree equals the last commit");
        assert_eq!(project.head().description, "fix and tidy");

        let outside = tempfile::tempdir().unwrap();
        let lonely = outside.path().join("m.arc");
        std::fs::write(&lonely, "model Lonely {}\n").unwrap();
        let error = Workspace::default().add_file_with_history(&lonely, 10).map(|_| ()).unwrap_err();
        assert!(error.contains("not in a git repository"), "{error}");
    }
}
