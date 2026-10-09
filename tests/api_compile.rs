//! In-process integration tests for the /api/compile endpoint (M4).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::util::ServiceExt;

async fn post_compile(source: &str) -> (StatusCode, serde_json::Value) {
    let app = arclang::web_server::build_router();
    let body = serde_json::json!({ "source": source }).to_string();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/compile")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn compile_endpoint_returns_semantic_model_with_uuids() {
    let (status, json) = post_compile(
        "model Test {}\narchitecture logical {\n  component \"Ctrl\" { id: \"LC-001\" }\n}",
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
    assert_eq!(json["model"]["components"][0]["id"], "LC-001");
    // Stable deterministic identity travels through the API.
    assert_eq!(
        json["model"]["all_elements"]["LC-001"]["uuid"],
        "8006ab91-390c-5908-8464-b353219dfc1f"
    );
    assert!(json["warnings"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn compile_endpoint_returns_localized_error() {
    let (status, json) = post_compile("model Test {\n  garbage here\n}").await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(json["success"], false);
    let error = json["error"].as_str().unwrap();
    assert!(
        error.contains("line 2, column 3"),
        "error must be localized, got: {error}"
    );
}

#[tokio::test]
async fn metamodel_endpoint_publishes_the_typed_contract() {
    let app = arclang::web_server::build_router();
    let response = app
        .oneshot(Request::builder().method("GET").uri("/api/metamodel").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["version"], arclang::compiler::metamodel::LANGUAGE_VERSION);
    let kinds = json["kinds"].as_array().unwrap();
    let component = kinds.iter().find(|k| k["name"] == "LogicalComponent").expect("LogicalComponent kind");
    let latency = kinds
        .iter()
        .find(|k| k["name"] == "SystemFunction")
        .and_then(|k| k["attributes"].as_array())
        .and_then(|attrs| attrs.iter().find(|a| a["key"] == "latency"))
        .expect("SystemFunction.latency");
    assert_eq!(latency["type"]["type"], "Quantity");
    assert_eq!(latency["type"]["of"], "Time");
    assert_eq!(component["sysml"], "part def + part");
    assert!(json["units"].as_array().unwrap().iter().any(|u| u["symbol"] == "ms"));
}

#[tokio::test]
async fn compile_endpoint_reports_metamodel_type_violations_as_warnings() {
    let (status, json) = post_compile(
        "model T {}\nsystem_analysis \"SA\" {\n  function \"F\" { id: \"F-1\" latency: 20 MHz }\n}",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "type violations never fail compilation (semver MINOR)");
    let warnings = json["warnings"].as_array().unwrap();
    assert!(
        warnings.iter().any(|w| w.as_str().unwrap().contains("metamodel: SystemFunction 'F-1'.latency")),
        "{warnings:?}"
    );
}
