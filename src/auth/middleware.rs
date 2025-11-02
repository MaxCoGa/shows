use crate::user::find_user_by_id;
use axum::{
    extract::State,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use crate::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub user_id: u64,
    pub exp: usize,
}

pub async fn auth_middleware(
    State(_app_state): State<Arc<AppState>>,
    mut req: axum::extract::Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_value| auth_value.strip_prefix("Bearer "));

    if let Some(token) = token {
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());
        let validation = Validation::default();

        if let Ok(token_data) = decode::<Claims>(token, &decoding_key, &validation) {
            if let Some(user) = find_user_by_id(token_data.claims.user_id) {
                req.extensions_mut().insert(Arc::new(user));
                return Ok(next.run(req).await);
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}
