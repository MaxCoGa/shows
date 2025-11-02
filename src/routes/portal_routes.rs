use crate::auth::{authenticate, AuthError, models::Credentials, registration::register_user};
use crate::user::{self, NewUser, User};
use crate::AppState;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
    routing::{delete, get},
    Router,
};
use axum_extra::extract::Form;
use tower_sessions::Session;
use askama::Template;
use std::sync::Arc;

// --- Session Keys ---
const USER_ID_KEY: &str = "user_id";

// --- Templates ---
#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate;

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate;

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    user: Option<User>,
}

#[derive(Template)]
#[template(path = "settings.html")]
struct SettingsTemplate {
    user: Option<User>,
}

// --- Routes ---
pub fn create_routes() -> Router<Arc<AppState>> {
    Router::<Arc<AppState>>::new()
        .route("/login", get(login_page).post(login))
        .route("/register", get(register_page).post(register))
        .route("/dashboard", get(dashboard_page))
        .route("/logout", get(logout))
        .route("/settings", get(settings_page))
        .route("/user", delete(delete_current_user))
}

// --- Handlers ---
#[axum::debug_handler]
async fn login(
    State(_state): State<Arc<AppState>>,
    session: Session,
    Form(creds): Form<Credentials>,
) -> impl IntoResponse {
    match authenticate(&creds) {
        Ok(user) => {
            session.insert(USER_ID_KEY, user.id).await.unwrap();
            Redirect::to("/portal/dashboard").into_response()
        }
        Err(AuthError::UserNotFound | AuthError::InvalidPassword) => {
            Redirect::to("/portal/login").into_response()
        }
    }
}

async fn login_page() -> impl IntoResponse {
    LoginTemplate
}

#[axum::debug_handler]
async fn register(
    State(_state): State<Arc<AppState>>,
    Form(creds): Form<Credentials>,
) -> impl IntoResponse {
    let new_user = NewUser {
        username: creds.username,
        password: creds.password,
    };

    match register_user(new_user) {
        Ok(_) => Redirect::to("/portal/login").into_response(),
        Err(_) => Redirect::to("/portal/register").into_response(),
    }
}

async fn register_page() -> impl IntoResponse {
    RegisterTemplate
}

#[axum::debug_handler]
async fn dashboard_page(
    State(_state): State<Arc<AppState>>,
    session: Session,
) -> impl IntoResponse {
    let user: Option<User> = if let Some(user_id) = session.get::<u64>(USER_ID_KEY).await.unwrap() {
        user::find_user_by_id(user_id)
    } else {
        None
    };

    if user.is_some() {
        DashboardTemplate { user }.into_response()
    } else {
        Redirect::to("/portal/login").into_response()
    }
}

#[axum::debug_handler]
async fn logout(session: Session) -> impl IntoResponse {
    let _ = session.clear().await;
    Redirect::to("/portal/login")
}

#[axum::debug_handler]
async fn settings_page(
    State(_state): State<Arc<AppState>>,
    session: Session,
) -> impl IntoResponse {
    let user: Option<User> = if let Some(user_id) = session.get::<u64>(USER_ID_KEY).await.unwrap() {
        user::find_user_by_id(user_id)
    } else {
        None
    };

    if user.is_some() {
        SettingsTemplate { user }.into_response()
    } else {
        Redirect::to("/portal/login").into_response()
    }
}

#[axum::debug_handler]
async fn delete_current_user(
    State(_state): State<Arc<AppState>>,
    session: Session,
) -> impl IntoResponse {
    if let Some(user_id) = session.get::<u64>(USER_ID_KEY).await.unwrap() {
        if user::delete_user(user_id).is_ok() {
            let _ = session.clear().await;
            return Redirect::to("/portal/login").into_response();
        }
    }
    Redirect::to("/portal/settings").into_response()
}
