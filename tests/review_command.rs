//! `arclang review`: the TypeSafe client against a local stand-in for the
//! service, and the command's dry run. No test reaches the real service.

use arclang::review::typesafe::TypeSafe;
use arclang::review::{Judge, Pair};
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{json, Value};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const KEY: &str = "ts_test_key_that_must_never_leak";

/// What the stand-in service saw, and the statuses it answers in turn.
#[derive(Default)]
struct Service {
    statuses: Vec<u16>,
    requests: Vec<(Option<String>, Value)>,
}

async fn answer(State(service): State<Arc<Mutex<Service>>>, headers: HeaderMap, Json(body): Json<Value>) -> (StatusCode, Json<Value>) {
    let mut service = service.lock().unwrap();
    let authorization = headers.get("authorization").and_then(|v| v.to_str().ok()).map(str::to_string);
    service.requests.push((authorization, body));
    let status = if service.statuses.is_empty() { 200 } else { service.statuses.remove(0) };
    let body = match status {
        200 => json!({ "model": "jev-test", "answers": { "plausible": { "type": "noul", "noul": 0.25 } }, "usage": {} }),
        _ => json!({ "detail": "no" }),
    };
    (StatusCode::from_u16(status).unwrap(), Json(body))
}

/// Start the stand-in service; returns its endpoint and what it records.
async fn service(statuses: &[u16]) -> (String, Arc<Mutex<Service>>) {
    let state = Arc::new(Mutex::new(Service { statuses: statuses.to_vec(), requests: Vec::new() }));
    let app = Router::new().route("/v1/systemone", post(answer)).with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}/v1/systemone", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (endpoint, state)
}

fn pair() -> Pair {
    Pair {
        source_id: "LC-1".to_string(),
        target_id: "REQ-1".to_string(),
        relation: "satisfies".to_string(),
        source: json!({ "kind": "LogicalComponent", "name": "Radar", "id": "LC-1" }),
        target: json!({ "kind": "Requirement", "name": "REQ-1", "id": "REQ-1", "description": "Detect obstacles" }),
        rationale: None,
    }
}

/// Ask the judge once, off the async runtime (the client is blocking).
async fn ask(endpoint: String) -> (Result<f64, String>, Option<String>) {
    tokio::task::spawn_blocking(move || {
        let mut judge = TypeSafe::new(KEY.to_string()).unwrap().at(&endpoint, Duration::from_millis(1));
        let answer = judge.plausibility(&pair());
        (answer, judge.model())
    })
    .await
    .unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn a_question_carries_the_key_as_bearer_and_the_two_ends_as_state() {
    let (endpoint, seen) = service(&[]).await;

    let (answer, model) = ask(endpoint).await;

    assert_eq!(answer, Ok(0.25));
    assert_eq!(model.as_deref(), Some("jev-test"));
    let seen = seen.lock().unwrap();
    let (authorization, body) = &seen.requests[0];
    assert_eq!(authorization.as_deref(), Some(format!("Bearer {KEY}").as_str()));
    assert_eq!(body["state"]["source"]["name"], "Radar");
    assert_eq!(body["state"]["target"]["description"], "Detect obstacles");
    assert_eq!(body["state"]["relation"], "satisfies");
    assert_eq!(body["questions"]["plausible"]["type"], "noul");
    assert!(!body.to_string().contains(KEY), "the key travels in the header only");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_request_to_slow_down_is_retried_then_answered() {
    let (endpoint, seen) = service(&[429, 529]).await;

    let (answer, _) = ask(endpoint).await;

    assert_eq!(answer, Ok(0.25));
    assert_eq!(seen.lock().unwrap().requests.len(), 3);
}

#[tokio::test(flavor = "multi_thread")]
async fn failures_are_reported_without_the_key() {
    for (statuses, expected) in [
        (vec![401], "refused the API key"),
        (vec![500], "status 500"),
        (vec![429, 429, 429, 429], "slow down"),
    ] {
        let (endpoint, _) = service(&statuses).await;
        let (answer, _) = ask(endpoint).await;
        let error = answer.unwrap_err();
        assert!(error.contains(expected), "{error}");
        assert!(!error.contains(KEY), "{error}");
    }
    // A service that is not there.
    let (answer, _) = ask("http://127.0.0.1:9/v1/systemone".to_string()).await;
    let error = answer.unwrap_err();
    assert!(error.contains("cannot be reached") && !error.contains(KEY), "{error}");
}

#[test]
fn a_dry_run_says_what_would_be_sent_and_needs_no_key() {
    let directory = tempfile::tempdir().unwrap();
    let model = directory.path().join("model.arc");
    std::fs::write(
        &model,
        "model M {}\nsystem_analysis \"S\" {\n  requirement \"REQ-1\" { description: \"Detect obstacles\" }\n}\nlogical_architecture \"L\" {\n  component \"Radar\" { id: \"LC-1\" description: \"Front radar\" }\n  component \"Display\" { id: \"LC-2\" }\n}\ntrace \"LC-1\" satisfies \"REQ-1\" {}\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_arclang"))
        .args(["review", &model.to_string_lossy(), "--suggest", "--dry-run"])
        .current_dir(directory.path())
        .env_remove("TYPESAFE_API_KEY")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(stdout.contains("1 declared trace(s) and 1 undeclared pair(s) to judge"), "{stdout}");
    assert!(stdout.contains("Nothing was sent"), "{stdout}");
    assert!(stdout.contains("\"description\": \"Front radar\""), "{stdout}");
}

#[test]
fn without_a_key_the_review_stops_before_sending_anything() {
    let directory = tempfile::tempdir().unwrap();
    let model = directory.path().join("model.arc");
    std::fs::write(&model, "model M {}\nsystem_analysis \"S\" {\n  requirement \"REQ-1\" { description: \"D\" }\n}\nlogical_architecture \"L\" {\n  component \"Radar\" { id: \"LC-1\" }\n}\ntrace \"LC-1\" satisfies \"REQ-1\" {}\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_arclang"))
        .args(["review", &model.to_string_lossy()])
        .current_dir(directory.path())
        .env_remove("TYPESAFE_API_KEY")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("no API key"));
}
