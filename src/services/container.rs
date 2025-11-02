use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;

use crate::AppState;
use crate::services::Service;

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
    use axum::Json;
    use serde_json::json;

    pub async fn container() -> Json<serde_json::Value> {
        Json(json!({ "service": "container" }))
    }
}
