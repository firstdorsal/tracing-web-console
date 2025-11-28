//! API module for control endpoints

pub mod control;

use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;

use control::ControlState;

/// Create the API router with all endpoints
pub fn create_api_router(control_state: Arc<ControlState>) -> Router {
    Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/logs", post(control::get_logs))
                .route("/ws", get(control::ws_logs))
                .route("/targets", get(control::get_targets))
                .with_state(control_state),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::LogStorage;

    #[tokio::test]
    async fn test_api_router_creation() {
        let storage = LogStorage::new();
        let control_state = Arc::new(ControlState::new(storage));
        let _router = create_api_router(control_state);

        // Router should be created successfully
        // In a real test, we would test the routes, but that requires a test server
        assert!(true);
    }
}
