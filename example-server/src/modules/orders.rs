use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub product_ids: Vec<String>,
    pub total: f64,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub user_id: String,
    pub product_ids: Vec<String>,
    pub total: f64,
}

type OrderStore = Arc<RwLock<Vec<Order>>>;

pub fn router() -> Router {
    let store: OrderStore = Arc::new(RwLock::new(Vec::new()));

    // Spawn background heartbeat task
    tokio::spawn(heartbeat_task());

    Router::new()
        .route("/api/orders", get(list_orders))
        .route("/api/orders", post(create_order))
        .with_state(store)
}

// Background task that logs heartbeat every 10 seconds at TRACE level
#[tracing::instrument(name = "heartbeat_task")]
async fn heartbeat_task() {
    let mut ticker = interval(Duration::from_secs(10));
    let mut count = 0u64;

    loop {
        ticker.tick().await;
        count += 1;

        tracing::trace!(
            heartbeat_count = %count,
            uptime_seconds = %(count * 10),
            "Order service heartbeat"
        );
    }
}

#[tracing::instrument(name = "list_orders", skip(store))]
async fn list_orders(State(store): State<OrderStore>) -> impl IntoResponse {
    tracing::info!("Fetching orders list");

    let orders = store.read();
    let count = orders.len();

    tracing::debug!(order_count = %count, "Retrieved orders from store");

    (StatusCode::OK, Json(orders.clone()))
}

#[tracing::instrument(name = "create_order", skip(store), fields(request_id = %Uuid::new_v4()))]
async fn create_order(
    State(store): State<OrderStore>,
    Json(req): Json<CreateOrderRequest>,
) -> Response {
    // DEBUG level: Validation
    tracing::debug!(
        user_id = %req.user_id,
        product_count = %req.product_ids.len(),
        total = %req.total,
        "Validating order creation request"
    );

    // Validation checks
    if req.user_id.is_empty() {
        tracing::warn!(user_id = %req.user_id, "Order validation failed: empty user_id");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "User ID cannot be empty"
            })),
        ).into_response();
    }

    if req.product_ids.is_empty() {
        tracing::warn!("Order validation failed: no products in order");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Order must contain at least one product"
            })),
        ).into_response();
    }

    if req.total <= 0.0 {
        tracing::warn!(total = %req.total, "Order validation failed: invalid total");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Order total must be greater than 0"
            })),
        ).into_response();
    }

    // Simulate stock check - WARN if low stock detected
    let simulated_stock = (req.product_ids.len() * 7) % 15; // Random-ish stock level
    if simulated_stock < 5 {
        tracing::warn!(
            product_ids = ?req.product_ids,
            estimated_stock = %simulated_stock,
            "Low stock warning for products in order"
        );
    }

    // Simulate occasional order processing failure - ERROR level
    let order_id = Uuid::new_v4();
    let order_id_str = order_id.to_string();

    // Use order ID to deterministically fail some orders (for demo purposes)
    if order_id.as_u128() % 10 == 0 {
        tracing::error!(
            order_id = %order_id_str,
            user_id = %req.user_id,
            error_code = "PAYMENT_FAILED",
            "Order processing failed: payment gateway error"
        );
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Payment processing failed",
                "order_id": order_id_str
            })),
        ).into_response();
    }

    let order = Order {
        id: order_id_str.clone(),
        user_id: req.user_id.clone(),
        product_ids: req.product_ids.clone(),
        total: req.total,
        status: "pending".to_string(),
    };

    tracing::info!(
        order_id = %order_id_str,
        user_id = %req.user_id,
        product_count = %req.product_ids.len(),
        total = %req.total,
        "Order created successfully"
    );

    store.write().push(order.clone());

    // Simulate async processing
    let order_id_for_task = order_id_str.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;
        tracing::debug!(
            order_id = %order_id_for_task,
            "Order processing completed asynchronously"
        );
    });

    (StatusCode::CREATED, Json(order)).into_response()
}
