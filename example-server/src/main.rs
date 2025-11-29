mod modules;

use axum::{routing::get, Router};
use tracing_web_console::TracingLayer;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route(
            "/",
            get(|| async { "Example Server - Visit /tracing for logs" }),
        )
        .merge(modules::users::router())
        .merge(modules::products::router())
        .merge(modules::orders::router())
        .merge(TracingLayer::new("/tracing").into_router());

    println!("🚀 Server starting on http://localhost:3000");
    println!("📊 Tracing UI available at http://localhost:3000/tracing");
    println!("\nAvailable endpoints:");
    println!("  GET  /");
    println!("  GET  /api/users");
    println!("  POST /api/users");
    println!("  GET  /api/users/:id");
    println!("  GET  /api/products");
    println!("  POST /api/products");
    println!("  PUT  /api/products/:id");
    println!("  GET  /api/orders");
    println!("  POST /api/orders");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
