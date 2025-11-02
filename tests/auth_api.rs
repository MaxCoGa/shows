use anyhow::Result;
use app::app;
use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use dotenvy::dotenv;
use http_body_util::BodyExt;
use serde::Deserialize;
use serde_json::json;
use tower::ServiceExt;

mod common;
use common::setup_test_db;

#[derive(Debug, Deserialize)]
struct TokenResponse {
    token: String,
}

// Helper function to register a user and return the response
async fn register_user(app: &axum::Router, user_creds: &serde_json::Value) -> http::Response<Body> {
    app.clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/auth/register")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(user_creds).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap()
}

// Helper function to log in a user and return the token
async fn login_user(app: &axum::Router, user_creds: &serde_json::Value) -> Result<String> {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/auth/login")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(user_creds).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let token_response: TokenResponse = serde_json::from_slice(&body)?;
    Ok(token_response.token)
}

#[tokio::test]
async fn test_register_and_login() -> Result<()> {
    dotenv().ok();
    let db_pool = setup_test_db().await;
    let app = app(db_pool);

    let user_creds = json!({
        "username": "testuser",
        "password": "password123",
    });

    // Register a new user
    let response = register_user(&app, &user_creds).await;
    assert_eq!(response.status(), StatusCode::CREATED);

    // Log in with the new user
    let token = login_user(&app, &user_creds).await?;
    assert!(!token.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_protected_route_access() -> Result<()> {
    dotenv().ok();
    let db_pool = setup_test_db().await;
    let app = app(db_pool);

    let user_creds = json!({
        "username": "testuser2",
        "password": "password123",
    });

    // Register and login to get a token
    register_user(&app, &user_creds).await;
    let token = login_user(&app, &user_creds).await?;

    // Access the protected route with the token
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/auth/protected")
                .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Access the protected route without a token
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/auth/protected")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    Ok(())
}

#[tokio::test]
async fn test_delete_user() -> Result<()> {
    dotenv().ok();
    let db_pool = setup_test_db().await;
    let app = app(db_pool);

    let user_creds = json!({
        "username": "todelete",
        "password": "password123",
    });

    // 1. Register and login to get a token
    register_user(&app, &user_creds).await;
    let token = login_user(&app, &user_creds).await?;

    // 2. Use the token to delete the user
    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::DELETE)
                .uri("/auth/user")
                .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_response.status(), StatusCode::OK);

    // 3. Verify that the user can no longer log in
    let login_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/auth/login")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&user_creds).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(login_response.status(), StatusCode::UNAUTHORIZED);

    Ok(())
}

#[tokio::test]
async fn test_delete_user_leaves_other_users_unaffected() -> Result<()> {
    dotenv().ok();
    let db_pool = setup_test_db().await;
    let app = app(db_pool);

    let user_one_creds = json!({
        "username": "user_one",
        "password": "password123",
    });

    let user_two_creds = json!({
        "username": "user_two",
        "password": "password456",
    });

    // 1. Register both users
    register_user(&app, &user_one_creds).await;
    register_user(&app, &user_two_creds).await;

    // 2. Log in as user_one to get their token
    let user_one_token = login_user(&app, &user_one_creds).await?;

    // 3. Delete user_one
    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::DELETE)
                .uri("/auth/user")
                .header(
                    http::header::AUTHORIZATION,
                    format!("Bearer {}", user_one_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_response.status(), StatusCode::OK);

    // 4. Verify user_one is deleted
    let login_response_one = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/auth/login")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&user_one_creds).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(login_response_one.status(), StatusCode::UNAUTHORIZED);

    // 5. Verify user_two can still log in
    let login_response_two = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/auth/login")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&user_two_creds).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(login_response_two.status(), StatusCode::OK);

    Ok(())
}
