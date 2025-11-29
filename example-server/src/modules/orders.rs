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
use std::time::Instant;
use tokio::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub items: Vec<OrderItem>,
    pub subtotal: f64,
    pub tax: f64,
    pub total: f64,
    pub status: OrderStatus,
    pub payment: PaymentInfo,
    pub shipping: ShippingInfo,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub product_id: String,
    pub sku: String,
    pub name: String,
    pub quantity: u32,
    pub unit_price: f64,
    pub total_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Pending,
    PaymentProcessing,
    PaymentFailed,
    Confirmed,
    Preparing,
    Shipped,
    Delivered,
    Cancelled,
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentInfo {
    pub method: String,
    pub status: String,
    pub transaction_id: Option<String>,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingInfo {
    pub method: String,
    pub address: String,
    pub city: String,
    pub postal_code: String,
    pub country: String,
    pub estimated_delivery: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub user_id: String,
    pub items: Vec<CreateOrderItem>,
    pub shipping_method: Option<String>,
    pub shipping_address: String,
    pub shipping_city: String,
    pub shipping_postal_code: String,
    pub shipping_country: Option<String>,
    pub payment_method: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrderItem {
    pub product_id: String,
    pub quantity: u32,
}

type OrderStore = Arc<RwLock<Vec<Order>>>;

pub fn router() -> Router {
    let store: OrderStore = Arc::new(RwLock::new(Vec::new()));

    // Spawn background heartbeat task
    tokio::spawn(heartbeat_task());

    // Spawn order processing simulation
    let processing_store = store.clone();
    tokio::spawn(order_processor(processing_store));

    // Spawn metrics collector
    let metrics_store = store.clone();
    tokio::spawn(order_metrics_collector(metrics_store));

    // Spawn fraud detection simulation
    tokio::spawn(fraud_detection_monitor());

    Router::new()
        .route("/api/orders", get(list_orders))
        .route("/api/orders", post(create_order))
        .with_state(store)
}

#[tracing::instrument(name = "system_heartbeat")]
async fn heartbeat_task() {
    let mut ticker = tokio::time::interval(Duration::from_secs(10));
    let mut count = 0u64;
    let start_time = Instant::now();

    loop {
        ticker.tick().await;
        count += 1;

        let uptime = start_time.elapsed();
        let memory_mb = (count * 3 % 512) + 128; // Simulated

        tracing::trace!(
            heartbeat_count = %count,
            uptime_seconds = %uptime.as_secs(),
            uptime_formatted = %format!("{}h {}m {}s", uptime.as_secs() / 3600, (uptime.as_secs() % 3600) / 60, uptime.as_secs() % 60),
            memory_usage_mb = %memory_mb,
            goroutines = %(50 + count % 20),
            "System heartbeat"
        );
    }
}

#[tracing::instrument(name = "order_processor")]
async fn order_processor(store: OrderStore) {
    let mut interval = tokio::time::interval(Duration::from_secs(5));
    let mut processed_count = 0u64;

    loop {
        interval.tick().await;

        let orders = store.write();
        let pending: Vec<_> = orders
            .iter()
            .enumerate()
            .filter(|(_, o)| {
                matches!(
                    o.status,
                    OrderStatus::Pending | OrderStatus::Confirmed | OrderStatus::Preparing
                )
            })
            .map(|(i, o)| (i, o.id.clone(), o.status.clone()))
            .collect();

        drop(orders);

        for (idx, order_id, current_status) in pending {
            processed_count += 1;

            // Simulate state transitions
            let new_status = match current_status {
                OrderStatus::Pending => {
                    if processed_count.is_multiple_of(10) {
                        tracing::warn!(
                            order_id = %order_id,
                            current_status = "pending",
                            reason = "payment_timeout",
                            "Order payment timeout - marking as failed"
                        );
                        OrderStatus::PaymentFailed
                    } else {
                        tracing::debug!(
                            order_id = %order_id,
                            transition = "pending -> confirmed",
                            "Order confirmed"
                        );
                        OrderStatus::Confirmed
                    }
                }
                OrderStatus::Confirmed => {
                    tracing::debug!(
                        order_id = %order_id,
                        transition = "confirmed -> preparing",
                        warehouse = "WH-001",
                        "Order being prepared"
                    );
                    OrderStatus::Preparing
                }
                OrderStatus::Preparing => {
                    tracing::info!(
                        order_id = %order_id,
                        transition = "preparing -> shipped",
                        carrier = "FastShip",
                        tracking_number = %format!("FS{}", Uuid::new_v4().to_string()[..12].to_uppercase()),
                        "Order shipped"
                    );
                    OrderStatus::Shipped
                }
                _ => continue,
            };

            let mut orders = store.write();
            if let Some(order) = orders.get_mut(idx) {
                order.status = new_status;
            }
        }
    }
}

#[tracing::instrument(name = "order_metrics")]
async fn order_metrics_collector(store: OrderStore) {
    let mut interval = tokio::time::interval(Duration::from_secs(25));

    loop {
        interval.tick().await;

        let orders = store.read();
        let total_orders = orders.len();

        if total_orders == 0 {
            tracing::trace!("No orders for metrics collection");
            continue;
        }

        let total_revenue: f64 = orders.iter().map(|o| o.total).sum();
        let avg_order_value = total_revenue / total_orders as f64;

        let status_counts: std::collections::HashMap<String, usize> =
            orders
                .iter()
                .fold(std::collections::HashMap::new(), |mut acc, o| {
                    *acc.entry(format!("{:?}", o.status)).or_insert(0) += 1;
                    acc
                });

        let payment_methods: std::collections::HashMap<&str, usize> =
            orders
                .iter()
                .fold(std::collections::HashMap::new(), |mut acc, o| {
                    *acc.entry(o.payment.method.as_str()).or_insert(0) += 1;
                    acc
                });

        tracing::info!(
            total_orders = %total_orders,
            total_revenue = %format!("{:.2}", total_revenue),
            avg_order_value = %format!("{:.2}", avg_order_value),
            status_breakdown = ?status_counts,
            payment_methods = ?payment_methods,
            "Order metrics snapshot"
        );
    }
}

#[tracing::instrument(name = "fraud_detection")]
async fn fraud_detection_monitor() {
    let mut interval = tokio::time::interval(Duration::from_secs(12));
    let mut scan_count = 0u64;

    let suspicious_patterns = [
        "multiple_cards_same_address",
        "velocity_check_failed",
        "address_mismatch",
        "high_risk_country",
        "unusual_purchase_time",
    ];

    loop {
        interval.tick().await;
        scan_count += 1;

        let transactions_scanned = (scan_count * 47) % 200 + 50;
        let flagged_count = scan_count % 7;

        tracing::debug!(
            scan_id = %scan_count,
            transactions_scanned = %transactions_scanned,
            flagged_transactions = %flagged_count,
            scan_duration_ms = %(scan_count % 150 + 50),
            "Fraud detection scan completed"
        );

        if flagged_count > 3 {
            let pattern = suspicious_patterns[(scan_count as usize) % suspicious_patterns.len()];
            tracing::warn!(
                scan_id = %scan_count,
                pattern_detected = %pattern,
                flagged_count = %flagged_count,
                risk_score = %(60 + flagged_count * 5),
                "Suspicious activity pattern detected"
            );
        }

        // Simulate occasional false positive review
        if scan_count.is_multiple_of(23) {
            tracing::info!(
                scan_id = %scan_count,
                review_type = "manual_review_required",
                priority = "medium",
                estimated_review_time_minutes = 15,
                "Transaction flagged for manual review"
            );
        }
    }
}

#[tracing::instrument(name = "list_orders", skip(store))]
async fn list_orders(State(store): State<OrderStore>) -> impl IntoResponse {
    let start = Instant::now();
    let request_id = Uuid::new_v4();

    tracing::info!(
        request_id = %request_id,
        "Processing list orders request"
    );

    let orders = store.read();
    let count = orders.len();

    let status_summary: std::collections::HashMap<String, usize> =
        orders
            .iter()
            .fold(std::collections::HashMap::new(), |mut acc, o| {
                *acc.entry(format!("{:?}", o.status)).or_insert(0) += 1;
                acc
            });

    tracing::debug!(
        request_id = %request_id,
        order_count = %count,
        status_summary = ?status_summary,
        query_duration_us = %start.elapsed().as_micros(),
        "Orders query completed"
    );

    (StatusCode::OK, Json(orders.clone()))
}

#[tracing::instrument(name = "create_order", skip(store), fields(request_id = %Uuid::new_v4()))]
async fn create_order(
    State(store): State<OrderStore>,
    Json(req): Json<CreateOrderRequest>,
) -> Response {
    let start = Instant::now();

    tracing::debug!(
        user_id = %req.user_id,
        item_count = %req.items.len(),
        shipping_address = %req.shipping_address,
        shipping_city = %req.shipping_city,
        payment_method = ?req.payment_method,
        "Validating order creation request"
    );

    // Validation checks
    if req.user_id.is_empty() {
        tracing::warn!(
            field = "user_id",
            rule = "required",
            "Order validation failed: empty user_id"
        );
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "User ID cannot be empty",
                "field": "user_id",
                "code": "VALIDATION_ERROR"
            })),
        )
            .into_response();
    }

    if req.items.is_empty() {
        tracing::warn!(
            field = "items",
            rule = "min_length",
            "Order validation failed: no items in order"
        );
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Order must contain at least one item",
                "field": "items",
                "code": "VALIDATION_ERROR"
            })),
        )
            .into_response();
    }

    // Validate each item
    for (idx, item) in req.items.iter().enumerate() {
        if item.quantity == 0 {
            tracing::warn!(
                field = "items",
                item_index = %idx,
                product_id = %item.product_id,
                rule = "min_quantity",
                "Order validation failed: zero quantity"
            );
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": format!("Item {} has zero quantity", idx),
                    "field": "items",
                    "code": "VALIDATION_ERROR"
                })),
            )
                .into_response();
        }
    }

    // Build order items with simulated prices
    let order_items: Vec<OrderItem> = req
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let unit_price = 19.99 + (i as f64 * 10.0);
            OrderItem {
                product_id: item.product_id.clone(),
                sku: format!(
                    "SKU-{}",
                    &item.product_id[..8.min(item.product_id.len())].to_uppercase()
                ),
                name: format!("Product {}", i + 1),
                quantity: item.quantity,
                unit_price,
                total_price: unit_price * item.quantity as f64,
            }
        })
        .collect();

    let subtotal: f64 = order_items.iter().map(|i| i.total_price).sum();
    let tax = subtotal * 0.08; // 8% tax
    let total = subtotal + tax;

    let order_id = Uuid::new_v4();
    let order_id_str = order_id.to_string();

    // Simulate payment processing
    tracing::debug!(
        order_id = %order_id_str,
        payment_method = %req.payment_method.as_deref().unwrap_or("credit_card"),
        amount = %format!("{:.2}", total),
        "Initiating payment processing"
    );

    // Simulate occasional payment failure
    if order_id.as_u128().is_multiple_of(10) {
        tracing::error!(
            order_id = %order_id_str,
            user_id = %req.user_id,
            error_code = "PAYMENT_DECLINED",
            payment_method = %req.payment_method.as_deref().unwrap_or("credit_card"),
            amount = %format!("{:.2}", total),
            decline_reason = "insufficient_funds",
            "Payment processing failed"
        );
        return (
            StatusCode::PAYMENT_REQUIRED,
            Json(serde_json::json!({
                "error": "Payment declined",
                "code": "PAYMENT_DECLINED",
                "order_id": order_id_str
            })),
        )
            .into_response();
    }

    let order = Order {
        id: order_id_str.clone(),
        user_id: req.user_id.clone(),
        items: order_items.clone(),
        subtotal,
        tax,
        total,
        status: OrderStatus::Pending,
        payment: PaymentInfo {
            method: req
                .payment_method
                .clone()
                .unwrap_or_else(|| "credit_card".to_string()),
            status: "authorized".to_string(),
            transaction_id: Some(Uuid::new_v4().to_string()),
            amount: total,
        },
        shipping: ShippingInfo {
            method: req
                .shipping_method
                .clone()
                .unwrap_or_else(|| "standard".to_string()),
            address: req.shipping_address.clone(),
            city: req.shipping_city.clone(),
            postal_code: req.shipping_postal_code.clone(),
            country: req
                .shipping_country
                .clone()
                .unwrap_or_else(|| "US".to_string()),
            estimated_delivery: None,
        },
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    tracing::info!(
        order_id = %order_id_str,
        user_id = %req.user_id,
        item_count = %order_items.len(),
        subtotal = %format!("{:.2}", subtotal),
        tax = %format!("{:.2}", tax),
        total = %format!("{:.2}", total),
        payment_method = %order.payment.method,
        shipping_method = %order.shipping.method,
        shipping_city = %order.shipping.city,
        shipping_country = %order.shipping.country,
        processing_time_ms = %start.elapsed().as_millis(),
        "Order created successfully"
    );

    store.write().push(order.clone());

    // Simulate async notifications
    let oid = order_id_str.clone();
    let uid = req.user_id.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        tracing::debug!(
            order_id = %oid,
            user_id = %uid,
            notification_type = "order_confirmation",
            channel = "email",
            "Order confirmation notification sent"
        );
    });

    // Simulate inventory reservation
    let oid2 = order_id_str.clone();
    let items_for_reservation = order_items.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        for item in &items_for_reservation {
            tracing::trace!(
                order_id = %oid2,
                product_id = %item.product_id,
                sku = %item.sku,
                quantity = %item.quantity,
                "Inventory reserved for order item"
            );
        }
    });

    (StatusCode::CREATED, Json(order)).into_response()
}
