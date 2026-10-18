//! /api/diagrams/generate-professional: a model that compiles is drawn; one
//! that does not is an error naming why, never a diagram of another model.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::util::ServiceExt;

async fn post(code: &str) -> (StatusCode, String) {
    let app = arclang::web_server::build_router();
    let body = serde_json::json!({ "dimension": "logical", "code": code }).to_string();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/diagrams/generate-professional")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

#[tokio::test]
async fn code_that_does_not_compile_is_an_error_not_a_diagram() {
    let (status, body) = post("model M { logical_architecture {").await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|_| panic!("a JSON error, got: {body}"));
    assert_eq!(json["success"], false);
    assert!(!json["error"].as_str().unwrap_or("").is_empty(), "{body}");
}

#[tokio::test]
async fn a_model_that_compiles_is_drawn() {
    let (status, body) = post("model M { name: \"m\" }\nlogical_architecture \"LA\" { component \"Ctrl\" { id: \"LC-1\" } }\n").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.contains("<"), "{body}");
}
