use crate::auth::models::Credentials;
use crate::user::{self, NewUser, User};
use crate::services::all_services;
use crate::AppState;
use axum::{
    extract::{State, Path},
    response::{IntoResponse, Redirect},
    routing::{delete, get},
    Router,
};
use axum_extra::extract::Form;
use tower_sessions::Session;
use askama::Template;
use std::sync::Arc;
use bcrypt::verify;

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
    services: Vec<String>,
}

#[derive(Template)]
#[template(path = "settings.html")]
struct SettingsTemplate {
    user: Option<User>,
    services: Vec<String>,
}

#[derive(Template)]
#[template(path = "service.html")]
struct ServiceTemplate {
    user: Option<User>,
    services: Vec<String>,
    service_name: String,
    resources: Vec<String>,
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
        .route("/service/:service_name", get(service_page))
}

// --- Handlers ---
#[axum::debug_handler]
async fn login(
    State(state): State<Arc<AppState>>,
    session: Session,
    Form(creds): Form<Credentials>,
) -> impl IntoResponse {
    if let Ok(Some(user)) = user::find_user_by_username(&state.db_pool, &creds.username).await {
        if verify(&creds.password, &user.password_hash).unwrap_or(false) {
            session.insert(USER_ID_KEY, user.id).await.unwrap();
            return Redirect::to("/portal/dashboard").into_response();
        }
    }
    Redirect::to("/portal/login").into_response()
}

async fn login_page() -> impl IntoResponse {
    LoginTemplate
}

#[axum::debug_handler]
async fn register(
    State(state): State<Arc<AppState>>,
    Form(creds): Form<Credentials>,
) -> impl IntoResponse {
    let new_user = NewUser {
        username: creds.username,
        password: creds.password,
    };

    match user::create_user(&state.db_pool, new_user).await {
        Ok(_) => Redirect::to("/portal/login").into_response(),
        Err(_) => Redirect::to("/portal/register").into_response(),
    }
}

async fn register_page() -> impl IntoResponse {
    RegisterTemplate
}

#[axum::debug_handler]
async fn dashboard_page(
    State(state): State<Arc<AppState>>,
    session: Session,
) -> impl IntoResponse {
    let user: Option<User> = if let Some(user_id) = session.get::<i64>(USER_ID_KEY).await.unwrap() {
        user::find_user_by_id(&state.db_pool, user_id).await.unwrap_or(None)
    } else {
        None
    };

    if user.is_some() {
        let services: Vec<String> = all_services().iter().map(|s| s.name().to_string()).collect();
        DashboardTemplate { user, services }.into_response()
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
    State(state): State<Arc<AppState>>,
    session: Session,
) -> impl IntoResponse {
    let user: Option<User> = if let Some(user_id) = session.get::<i64>(USER_ID_KEY).await.unwrap() {
        user::find_user_by_id(&state.db_pool, user_id).await.unwrap_or(None)
    } else {
        None
    };

    if user.is_some() {
        let services: Vec<String> = all_services().iter().map(|s| s.name().to_string()).collect();
        SettingsTemplate { user, services }.into_response()
    } else {
        Redirect::to("/portal/login").into_response()
    }
}

#[axum::debug_handler]
async fn delete_current_user(
    State(state): State<Arc<AppState>>,
    session: Session,
) -> impl IntoResponse {
    if let Some(user_id) = session.get::<i64>(USER_ID_KEY).await.unwrap() {
        if user::delete_user(&state.db_pool, user_id).await.is_ok() {
            let _ = session.clear().await;
            return Redirect::to("/portal/login").into_response();
        }
    }
    Redirect::to("/portal/settings").into_response()
}

#[axum::debug_handler]
async fn service_page(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(service_name): Path<String>,
) -> impl IntoResponse {
    let user: Option<User> = if let Some(user_id) = session.get::<i64>(USER_ID_KEY).await.unwrap() {
        user::find_user_by_id(&state.db_pool, user_id).await.unwrap_or(None)
    } else {
        None
    };

    if user.is_some() {
        let services: Vec<String> = all_services().iter().map(|s| s.name().to_string()).collect();
        let resources = if service_name == "container" {
            vec!["container-1".to_string(), "container-2".to_string()]
        } else if service_name == "virtual_network" {
            vec!["vnet-main".to_string()]
        } else {
            vec![]
        };
        ServiceTemplate { user, services, service_name, resources }.into_response()
    } else {
        Redirect::to("/portal/login").into_response()
    }
}
