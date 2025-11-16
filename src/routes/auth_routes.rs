use crate::auth::handlers::{delete_current_user, login, protected, register};
use crate::auth::middleware::auth_middleware;
use crate::AppState;
use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;

pub fn create_routes(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    let protected_routes = Router::<Arc<AppState>>::new()
        .route("/protected", get(protected))
        .route("/user", delete(delete_current_user))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    Router::<Arc<AppState>>::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .merge(protected_routes)
}
