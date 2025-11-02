use axum::Router;
use std::sync::Arc;
use crate::AppState;

pub mod container;

pub trait Service {
    fn name(&self) -> &'static str;
    fn routes(&self) -> Router<Arc<AppState>>;
}

pub fn all_services() -> Vec<Box<dyn Service + Send + Sync>> {
    vec![
        Box::new(container::ContainerService),
    ]
}
