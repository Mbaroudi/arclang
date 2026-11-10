//! `arclang serve` without users answers anyone who reaches it, on the
//! loopback interface. A page of another site reaches it through the
//! browser of whoever runs it by giving its own name this machine's address
//! (DNS rebinding); the request then names that site as its host. Without
//! users, only a request addressed to this machine is served.

use arclang::web_server::systems_modeling::Workspace;
use arclang::web_server::{build_local_router, build_router_with};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use chrono::{TimeZone, Utc};
use serde_json::Value;
use tower::util::ServiceExt;

const MODEL: &str = r#"
model Lane { metadata { description: "Lane keeping" } }
system_analysis "SA" { requirement "REQ-1" { description: "react fast" } }
"#;
const PROJECTS: &str = "/api/systems-modeling/projects";
const TOKEN: &str = "a-token-long-enough-for-the-server-0123456789";

fn workspace() -> Workspace {
    let mut workspace = Workspace::default();
    workspace.add_source(MODEL, Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap()).expect("model compiles");
    workspace
}

async fn get(app: Router, path: &str, headers: &[(&str, &str)]) -> (StatusCode, Value) {
    let mut request = Request::builder().method("GET").uri(path);
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let response = app.oneshot(request.body(Body::empty()).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

#[tokio::test]
async fn a_request_addressed_to_this_machine_is_served() {
    for host in ["localhost", "localhost:3000", "LocalHost:3000", "127.0.0.1:3000", "127.0.0.1", "[::1]:3000", "[::1]", "localhost.:3000"] {
        let (status, body) = get(build_local_router(workspace()), PROJECTS, &[("host", host)]).await;
        assert_eq!(status, StatusCode::OK, "{host}: {body}");
        assert_eq!(body.as_array().map(Vec::len), Some(1), "{host}: {body}");
    }
    // A client that names no host is not a browser a page drives.
    let (status, _) = get(build_local_router(workspace()), PROJECTS, &[]).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn a_request_addressed_to_another_name_is_refused_when_the_server_has_no_users() {
    for host in ["rebind.example:3000", "rebind.example", "localhost.rebind.example", "127.0.0.1.rebind.example:3000", "192.168.1.20:3000", "", "localhost@rebind.example"] {
        for path in [PROJECTS, "/api/metamodel", "/health"] {
            let (status, body) = get(build_local_router(workspace()), path, &[("host", host)]).await;
            assert_eq!(status, StatusCode::FORBIDDEN, "{host} {path}: {body}");
            let error = body["error"].as_str().unwrap_or_default();
            assert!(error.contains("answers only requests addressed to this machine"), "{host}: {body}");
            assert!(error.contains("--token-file"), "the message says how to serve it under another name: {body}");
        }
    }
    // The target of the request is read too (HTTP/2, absolute form).
    let (status, _) = get(build_local_router(workspace()), &format!("http://rebind.example:3000{PROJECTS}"), &[("host", "localhost:3000")]).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn with_a_token_the_server_may_stand_behind_another_name() {
    let with_token = || {
        let mut workspace = workspace();
        workspace.require_token(TOKEN).unwrap();
        build_local_router(workspace)
    };
    let (status, _) = get(with_token(), PROJECTS, &[("host", "models.plant.example")]).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "the token is what protects");
    let bearer = format!("Bearer {TOKEN}");
    let (status, body) = get(with_token(), PROJECTS, &[("host", "models.plant.example"), ("authorization", bearer.as_str())]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test]
async fn a_server_that_embeds_the_api_is_not_concerned() {
    // The router alone names no host rule: what embeds it checks its own requests.
    let (status, _) = get(build_router_with(workspace()), PROJECTS, &[("host", "models.plant.example")]).await;
    assert_eq!(status, StatusCode::OK);
}
