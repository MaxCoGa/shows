use crate::AppState;
use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use futures_util::stream::StreamExt;
use serde::Serialize;
use std::fs;
use std::path::Path as StdPath;
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

const UPLOAD_DIR: &str = "pcloud_files";

// --- Error ---
type AppError = (StatusCode, String);

// --- Response Structs ---
#[derive(Serialize)]
pub struct FileInfo {
    name: String,
    size: u64,
}

// --- Routes ---
pub fn create_routes() -> Router<Arc<AppState>> {
    Router::<Arc<AppState>>::new()
        .route("/files", get(list_files).post(upload_file))
        .route("/files/:filename", get(download_file).delete(delete_file))
}

// --- Handlers ---
#[axum::debug_handler]
async fn list_files(State(_state): State<Arc<AppState>>) -> Result<Json<Vec<FileInfo>>, AppError> {
    fs::create_dir_all(UPLOAD_DIR).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create directory: {}", e),
        )
    })?;

    let mut files = Vec::new();
    let entries = fs::read_dir(UPLOAD_DIR).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read directory: {}", e),
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read directory entry: {}", e),
            )
        })?;
        let path = entry.path();
        if path.is_file() {
            let metadata = entry.metadata().map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to get metadata: {}", e),
                )
            })?;
            files.push(FileInfo {
                name: path.file_name().unwrap().to_str().unwrap().to_string(),
                size: metadata.len(),
            });
        }
    }

    Ok(Json(files))
}

#[axum::debug_handler]
async fn upload_file(
    State(_state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<(StatusCode, String), AppError> {
    fs::create_dir_all(UPLOAD_DIR).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create directory: {}", e),
        )
    })?;

    let mut uploaded_filename = String::new();
    while let Some(field) = multipart.next_field().await.unwrap() {
        if let Some(file_name) = field.file_name() {
            uploaded_filename = file_name.to_string();
            let dest_path = StdPath::new(UPLOAD_DIR).join(&uploaded_filename);
            let file_content = field.bytes().await.unwrap();
            fs::write(&dest_path, file_content).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to write to file: {}", e),
                )
            })?;
        }
    }

    if uploaded_filename.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No file uploaded".to_string()));
    }

    Ok((
        StatusCode::CREATED,
        format!("File '{}' uploaded successfully.", uploaded_filename),
    ))
}

#[axum::debug_handler]
async fn download_file(
    State(_state): State<Arc<AppState>>,
    Path(filename): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let path = StdPath::new(UPLOAD_DIR).join(&filename);

    let file = File::open(&path).await.map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            format!("File not found: {}", filename),
        )
    })?;

    let stream = FramedRead::new(file, BytesCodec::new()).map(|r| r.map(|bytes| bytes.freeze()));
    let body = Body::from_stream(stream);

    let response = axum::response::Response::builder()
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(body)
        .unwrap();

    Ok(response)
}

#[axum::debug_handler]
async fn delete_file(
    State(_state): State<Arc<AppState>>,
    Path(filename): Path<String>,
) -> Result<(StatusCode, String), AppError> {
    let path = StdPath::new(UPLOAD_DIR).join(&filename);

    if !path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("File not found: {}", filename),
        ));
    }

    fs::remove_file(&path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to delete file: {}", e),
        )
    })?;

    Ok((
        StatusCode::OK,
        format!("File '{}' deleted successfully.", filename),
    ))
}
