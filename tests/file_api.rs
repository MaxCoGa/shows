use app::app;
use serial_test::serial;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

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
async fn test_list_files() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    // Create a dummy file for listing
    fs::create_dir_all("pcloud_files").unwrap();
    fs::write(
        Path::new("pcloud_files").join("test_file.txt"),
        "test content",
    )
    .unwrap();

    let response = client
        .get(&format!("{}/api/files", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let files: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0]["name"], "test_file.txt");

    // Clean up
    fs::remove_dir_all("pcloud_files").unwrap();
}

#[tokio::test]
#[serial]
async fn test_upload_file() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();
    let test_file_name = "test_upload.txt";
    let test_file_content = "This is a test upload.";

    // Create a multipart form
    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(test_file_content.as_bytes().to_vec())
            .file_name(test_file_name.to_string()),
    );

    // Send the request
    let response = client
        .post(&format!("{}/api/files", &address))
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status(), reqwest::StatusCode::CREATED);
    let response_text = response.text().await.unwrap();
    assert_eq!(
        response_text,
        format!("File \'{}\' uploaded successfully.", test_file_name)
    );

    // Verify the file was uploaded
    let file_path = Path::new("pcloud_files").join(test_file_name);
    assert!(file_path.exists());

    // Clean up the created directory and file
    fs::remove_dir_all("pcloud_files").unwrap();
}

#[tokio::test]
#[serial]
async fn test_download_file() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();
    let test_file_name = "test_download.txt";
    let test_file_content = "This is the content of the test download file.";

    // Create a dummy file for downloading
    fs::create_dir_all("pcloud_files").unwrap();
    fs::write(
        Path::new("pcloud_files").join(test_file_name),
        test_file_content,
    )
    .unwrap();

    // Send the request to download the file
    let response = client
        .get(&format!("{}/api/files/{}", &address, test_file_name))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let downloaded_content = response.text().await.unwrap();
    assert_eq!(downloaded_content, test_file_content);

    // Clean up
    fs::remove_dir_all("pcloud_files").unwrap();
}

#[tokio::test]
#[serial]
async fn test_delete_file() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();
    let test_file_name = "test_delete.txt";

    // Create a dummy file for deletion
    fs::create_dir_all("pcloud_files").unwrap();
    fs::write(
        Path::new("pcloud_files").join(test_file_name),
        "test content",
    )
    .unwrap();

    // Send the request to delete the file
    let response = client
        .delete(&format!("{}/api/files/{}", &address, test_file_name))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let response_text = response.text().await.unwrap();
    assert_eq!(
        response_text,
        format!("File \'{}\' deleted successfully.", test_file_name)
    );

    // Verify the file was deleted
    let file_path = Path::new("pcloud_files").join(test_file_name);
    assert!(!file_path.exists());

    // Clean up
    fs::remove_dir_all("pcloud_files").unwrap();
}
