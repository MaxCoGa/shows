use crate::routes::{
    auth_routes::create_routes as create_auth_routes,
    file_routes::create_routes as create_file_routes,
    portal_routes::create_routes as create_portal_routes,
};
use crate::database::DbPool;
use axum::Router;
use std::sync::Arc;
use tower_sessions::{
    cookie::time::Duration,
    Expiry,
    SessionManagerLayer,
};
use tower_sessions_memory_store::MemoryStore;

pub mod auth;
pub mod database;
pub mod routes;
pub mod user;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
}

pub fn app(db_pool: DbPool) -> Router {
    let app_state = Arc::new(AppState { db_pool });

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)));

    Router::new()
        .nest("/api", create_file_routes())
        .nest("/auth", create_auth_routes(app_state.clone()))
        .nest("/portal", create_portal_routes())
        .layer(session_layer)
        .with_state(app_state)
}
