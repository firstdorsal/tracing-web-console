//! API module for log endpoints

pub mod logs;

use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;

use logs::LogsState;

/// Create the API router with all endpoints
pub fn create_api_router(state: Arc<LogsState>) -> Router {
    Router::new().nest(
        "/api",
        Router::new()
            .route("/logs", post(logs::get_logs))
            .route("/ws", get(logs::ws_logs))
            .route("/targets", get(logs::get_targets))
            .with_state(state),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::LogStorage;

    #[tokio::test]
    async fn test_api_router_creation() {
        // Router should be created successfully without panic
        let storage = LogStorage::new();
        let state = Arc::new(LogsState::new(storage));
        let _router = create_api_router(state);
    }
}
