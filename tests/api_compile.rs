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
