use app::app;
use serial_test::serial;
use std::net::SocketAddr;

mod common;
use common::setup_test_db;

async fn spawn_app() -> String {
    let db_pool = setup_test_db().await;
    let app = app(db_pool);
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service()).await.unwrap();
    });

    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
#[serial]
async fn test_container_service() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/api/container", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body, serde_json::json!({ "service": "container" }));
}
