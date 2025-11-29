//! # tracing-web-console
//!
//! A real-time web-based console for viewing and filtering tracing logs.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use axum::Router;
//! use axum::routing::get;
//! use tracing_web_console::TracingLayer;
//!
//! #[tokio::main]
//! async fn main() {
//!     let app = Router::new()
//!         .route("/", get(|| async { "Hello World" }))
//!         .merge(TracingLayer::new("/tracing").into_router());
//!
//!     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
//!         .await
//!         .unwrap();
//!
//!     axum::serve(listener, app).await.unwrap();
//! }
//! ```

mod api;
mod frontend;
mod layer;
mod storage;
mod subscriber;

pub use layer::TracingLayer;
pub use storage::LogEvent;
