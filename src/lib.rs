use crate::{
    database::DbPool,
    routes::{
        auth_routes::create_routes as create_auth_routes,
        file_routes::create_routes as create_file_routes,
        portal_routes::create_routes as create_portal_routes,
        services_routes::create_service_routes as create_service_routes,
    },
    services::all_services,
};
use axum::Router;
use std::sync::Arc;
use tower_sessions::{
    cookie::time::Duration, Expiry, SessionManagerLayer
};
use tower_sessions_memory_store::MemoryStore;

pub mod auth;
pub mod database;
pub mod routes;
pub mod services;
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

    let mut service_routes: Router<Arc<AppState>> = Router::new();
    for service in all_services() {
        service_routes = service_routes.nest(&format!("/{}", service.name()), service.routes());
    }

    let api_routes = create_file_routes()
        .merge(service_routes)
        .merge(create_service_routes());

    Router::new()
        .nest("/api", api_routes)
        .nest("/auth", create_auth_routes(app_state.clone()))
        .nest("/portal", create_portal_routes())
        .layer(session_layer)
        .with_state(app_state.clone())
}
