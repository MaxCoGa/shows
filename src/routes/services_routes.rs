use crate::AppState;
use crate::services::all_services;
use axum::{
    extract::State,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use std::sync::Arc;

// --- Routes ---
pub fn create_routes() -> Router<Arc<AppState>> {
    Router::<Arc<AppState>>::new().route("/services", get(list_services))
}

// --- Handlers ---
#[axum::debug_handler]
async fn list_services(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let services = all_services();
    let service_names: Vec<&str> = services.iter().map(|s| s.name()).collect();
    Json(service_names)
}
