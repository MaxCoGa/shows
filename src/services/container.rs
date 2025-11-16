use axum::{
    body::Body,
    http::StatusCode,
    response::Response,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::services::Service;
use crate::AppState;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Container {
    #[serde(rename = "Names")]
    pub names: String,
    #[serde(rename = "Image")]
    pub image: String,
    #[serde(rename = "State")]
    pub state: String,
}

pub struct ContainerService;

impl Service for ContainerService {
    fn name(&self) -> &'static str {
        "container"
    }

    fn routes(&self) -> Router<Arc<AppState>> {
        Router::new().route("/", get(self::get::container))
    }
}

mod get {
    use super::{Body, Container, Response, StatusCode};
    use crate::ansible;

    pub async fn container() -> Response<Body> {
        match ansible::list_containers().await {
            Ok(json_values) => {
                let containers: Vec<Container> = json_values
                    .into_iter()
                    .filter_map(|value| serde_json::from_value(value).ok())
                    .collect();

                let json_body = serde_json::to_string(&containers).unwrap_or_else(|_| "[]".to_string());

                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(Body::from(json_body))
                    .unwrap()
            }
            Err(e) => {
                eprintln!("Failed to list containers: {:?}", e);
                let error_body = "{\"error\":\"Failed to retrieve container information.\"}".to_string();
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("Content-Type", "application/json")
                    .body(Body::from(error_body))
                    .unwrap()
            }
        }
    }
}
