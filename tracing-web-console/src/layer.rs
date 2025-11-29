//! Main TracingLayer that integrates with Axum

use crate::api::logs::LogsState;
use crate::storage::LogStorage;
use crate::subscriber::LogCaptureLayer;
use axum::routing::get;
use axum::Router;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// Main tracing layer that can be added to an Axum application
#[derive(Clone)]
pub struct TracingLayer {
    router: Router,
}

impl TracingLayer {
    /// Create a new TracingLayer with the specified base path
    ///
    /// # Arguments
    ///
    /// * `base_path` - The base path for all tracing UI routes (e.g., "/tracing")
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use axum::Router;
    /// use axum::routing::get;
    /// use tracing_web_console::TracingLayer;
    ///
    /// let app = Router::new()
    ///     .route("/", get(|| async { "Hello World" }))
    ///     .merge(TracingLayer::new("/tracing").into_router());
    /// ```
    pub fn new(base_path: &str) -> Self {
        Self::with_capacity(base_path, 10_000)
    }

    /// Create a new TracingLayer with custom storage capacity
    ///
    /// # Arguments
    ///
    /// * `base_path` - The base path for all tracing UI routes
    /// * `capacity` - Maximum number of log events to store in memory
    pub fn with_capacity(base_path: &str, capacity: usize) -> Self {
        // Create storage for log events
        let storage = LogStorage::with_capacity(capacity);

        // Set up tracing subscriber with env filter
        // Default to "trace" for all targets except:
        // - this crate (to avoid recursive logging)
        // - "log" target (noisy compatibility layer from log crate)
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("trace,tracing_web_console=off,log=off"));

        // Create our custom log capture layer
        let log_capture_layer = LogCaptureLayer::new(storage.clone());

        // Initialize the tracing subscriber
        // Note: This will set the global default subscriber
        tracing_subscriber::registry()
            .with(env_filter)
            .with(log_capture_layer)
            .try_init()
            .ok(); // Ignore error if already initialized

        // Create shared state
        let logs_state = Arc::new(LogsState::new(storage.clone()));

        // Create frontend state with base path
        let frontend_state = crate::frontend::FrontendState {
            base_path: Arc::new(base_path.to_string()),
        };

        // Create frontend router with its state
        let frontend_router = Router::new()
            .route("/", get(crate::frontend::serve_index))
            .route("/assets/*path", get(crate::frontend::serve_static))
            .with_state(frontend_state);

        // Create the API router
        let api_router = crate::api::create_api_router(logs_state);

        // Merge frontend and API routers
        let inner_router = frontend_router.merge(api_router);

        // Add CORS middleware for development
        // In production this allows all origins, which is fine for a debugging/monitoring tool
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        // Nest everything under the base path and add CORS
        let router = Router::new().nest(base_path, inner_router).layer(cors);

        Self { router }
    }

    /// Merge this tracing layer with an existing Axum router
    ///
    /// This is the recommended way to add the tracing UI to your application
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use axum::Router;
    /// use axum::routing::get;
    /// use tracing_web_console::TracingLayer;
    ///
    /// let app = Router::new()
    ///     .route("/", get(|| async { "Hello World" }))
    ///     .merge(TracingLayer::new("/tracing").into_router());
    /// ```
    pub fn into_router(self) -> Router {
        self.router
    }
}

/// Builder for configuring TracingLayer
#[allow(dead_code)]
pub struct TracingLayerBuilder {
    base_path: String,
    capacity: usize,
    initial_filter: String,
}

impl TracingLayerBuilder {
    /// Create a new builder with the specified base path
    #[allow(dead_code)]
    pub fn new(base_path: &str) -> Self {
        Self {
            base_path: base_path.to_string(),
            capacity: 10_000,
            initial_filter: "trace".to_string(),
        }
    }

    /// Set the storage capacity
    #[allow(dead_code)]
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }

    /// Set the initial log filter
    #[allow(dead_code)]
    pub fn with_filter(mut self, filter: &str) -> Self {
        self.initial_filter = filter.to_string();
        self
    }

    /// Build the TracingLayer
    #[allow(dead_code)]
    pub fn build(self) -> TracingLayer {
        TracingLayer::with_capacity(&self.base_path, self.capacity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracing_layer_creation() {
        // Basic test to ensure it can be created without panic
        let _layer = TracingLayer::new("/tracing");
    }

    #[test]
    fn test_builder_pattern() {
        let builder = TracingLayerBuilder::new("/tracing")
            .with_capacity(5000)
            .with_filter("debug");

        assert_eq!(builder.base_path, "/tracing");
        assert_eq!(builder.capacity, 5000);
        assert_eq!(builder.initial_filter, "debug");
    }

    #[test]
    fn test_builder_defaults_to_trace() {
        let builder = TracingLayerBuilder::new("/tracing");
        assert_eq!(builder.initial_filter, "trace");
    }
}
