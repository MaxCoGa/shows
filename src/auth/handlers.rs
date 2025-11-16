use crate::auth::models::Credentials;
use crate::user::{self, NewUser, User};
use crate::AppState;
use axum::{extract::State, http::StatusCode, response::{IntoResponse, Json}, Extension};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Serialize;
use std::sync::Arc;
use std::env;
use crate::auth::middleware::Claims;
use serde_json::json;
use bcrypt::verify;


#[derive(Serialize)]
pub struct TokenResponse {
    token: String,
}

#[derive(Serialize)]
pub struct ProtectedResponse {
    message: String,
}

pub async fn register(State(state): State<Arc<AppState>>, Json(payload): Json<NewUser>) -> impl IntoResponse {
    match user::create_user(&state.db_pool, payload).await {
        Ok(_) => (StatusCode::CREATED, Json(json!({ "message": "User created successfully" }))),
        Err(user::UserError::UsernameTaken) => (StatusCode::CONFLICT, Json(json!({ "error": "User already exists" }))),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "An unexpected error occurred" }))),
    }
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Credentials>,
) -> Result<Json<TokenResponse>, StatusCode> {
    let user = user::find_user_by_username(&state.db_pool, &payload.username)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !verify(&payload.password, &user.password_hash).unwrap_or(false) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let claims = Claims {
        sub: user.username.clone(),
        user_id: user.id,
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .unwrap();

    Ok(Json(TokenResponse { token }))
}

pub async fn protected(
    Extension(user): Extension<Arc<User>>,
) -> Result<Json<ProtectedResponse>, StatusCode> {
    Ok(Json(ProtectedResponse {
        message: format!("Hello, {}!", user.username),
    }))
}

pub async fn delete_current_user(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<Arc<User>>,
) -> impl IntoResponse {
    match user::delete_user(&state.db_pool, user.id).await {
        Ok(_) => (StatusCode::OK, Json(json!({ "message": "User deleted successfully" }))),
        Err(user::UserDeleteError::UserNotFound) => (StatusCode::NOT_FOUND, Json(json!({ "error": "User not found" }))),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "An unexpected error occurred" }))),
    }
}
