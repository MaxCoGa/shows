use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;

use crate::AppState;
use crate::services::Service;

pub struct VirtualNetworkService;

impl Service for VirtualNetworkService {
    fn name(&self) -> &'static str {
        "virtual_network"
    }

    fn routes(&self) -> Router<Arc<AppState>> {
        Router::new().route("/", get(self::get::virtual_network))
    }
}

mod get {
    use axum::Json;
    use serde_json::json;

    pub async fn virtual_network() -> Json<serde_json::Value> {
        Json(json!({ "service": "virtual_network" }))
    }
}
