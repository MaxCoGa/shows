use anyhow::Result;
use app::app;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

mod common;
use common::setup_test_db;

#[tokio::test]
async fn test_virtual_network_service() -> Result<()> {
    let db_pool = setup_test_db().await;
    let app = app(db_pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/virtual_network")
                .body(Body::empty())?,
        )
        .await?;

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await?.to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body)?;
    assert_eq!(body, serde_json::json!({ "service": "virtual_network" }));

    Ok(())
}
