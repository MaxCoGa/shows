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
async fn test_container_service_returns_a_list_of_containers() {
    // Arrange
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/api/container", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.expect("Failed to parse json body");

    assert!(body.is_array(), "Response body is not a JSON array");
}
