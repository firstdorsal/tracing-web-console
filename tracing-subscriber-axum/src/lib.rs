//! # Tracing Subscriber Axum
//!
//! A tracing subscriber that provides a web UI for viewing and controlling logs in real-time.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use axum::Router;
//! use axum::routing::get;
//! use tracing_subscriber_axum::TracingLayer;
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
