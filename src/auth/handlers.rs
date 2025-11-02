use crate::auth::models::Credentials;
use crate::user::{delete_user, NewUser, User};
use axum::{http::StatusCode, response::{IntoResponse, Json}, Extension};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Serialize;
use std::sync::Arc;
use std::env;
use crate::auth::middleware::Claims;
use crate::auth::registration::register_user;
use serde_json::json;

#[derive(Serialize)]
pub struct TokenResponse {
    token: String,
}

#[derive(Serialize)]
pub struct ProtectedResponse {
    message: String,
}

pub async fn register(Json(payload): Json<NewUser>) -> impl IntoResponse {
    match register_user(payload) {
        Ok(_) => (StatusCode::CREATED, Json(json!({ "message": "User created successfully" }))),
        Err(_) => (StatusCode::CONFLICT, Json(json!({ "error": "User already exists" }))),
    }
}

pub async fn login(
    Json(payload): Json<Credentials>,
) -> Result<Json<TokenResponse>, StatusCode> {
    let user = crate::auth::authenticate(&payload).map_err(|_| StatusCode::UNAUTHORIZED)?;

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
    Extension(user): Extension<Arc<User>>,
) -> impl IntoResponse {
    match delete_user(user.id) {
        Ok(_) => (StatusCode::OK, Json(json!({ "message": "User deleted successfully" }))),
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "error": "User not found" }))),
    }
}
