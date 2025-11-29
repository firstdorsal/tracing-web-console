use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub sku: String,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub stock: i32,
    pub category: String,
    pub tags: Vec<String>,
    pub metrics: ProductMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductMetrics {
    pub views: u64,
    pub purchases: u64,
    pub avg_rating: f32,
    pub review_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub stock: i32,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub stock: Option<i32>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub search: Option<String>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub category: Option<String>,
    pub in_stock: Option<bool>,
}

type ProductStore = Arc<RwLock<Vec<Product>>>;

pub fn router() -> Router {
    let store: ProductStore = Arc::new(RwLock::new(Vec::new()));

    // Spawn inventory monitoring task
    let inventory_store = store.clone();
    tokio::spawn(inventory_monitor(inventory_store));

    // Spawn price analytics task
    let analytics_store = store.clone();
    tokio::spawn(price_analytics_task(analytics_store));

    // Spawn cache warming simulation
    tokio::spawn(cache_warmer());

    Router::new()
        .route("/api/products", get(list_products))
        .route("/api/products", post(create_product))
        .route("/api/products/:id", put(update_product))
        .with_state(store)
}

#[tracing::instrument(name = "inventory_monitor")]
async fn inventory_monitor(store: ProductStore) {
    let mut interval = tokio::time::interval(Duration::from_secs(15));
    let mut check_count = 0u64;

    loop {
        interval.tick().await;
        check_count += 1;

        let products = store.read();
        let total_products = products.len();
        let low_stock: Vec<_> = products
            .iter()
            .filter(|p| p.stock < 10 && p.stock > 0)
            .collect();
        let out_of_stock: Vec<_> = products.iter().filter(|p| p.stock == 0).collect();
        let total_inventory: i32 = products.iter().map(|p| p.stock).sum();

        tracing::debug!(
            check_number = %check_count,
            total_products = %total_products,
            total_inventory_units = %total_inventory,
            low_stock_count = %low_stock.len(),
            out_of_stock_count = %out_of_stock.len(),
            "Inventory check completed"
        );

        for product in &low_stock {
            tracing::warn!(
                product_id = %product.id,
                sku = %product.sku,
                product_name = %product.name,
                current_stock = %product.stock,
                category = %product.category,
                "Low stock alert"
            );
        }

        for product in &out_of_stock {
            tracing::error!(
                product_id = %product.id,
                sku = %product.sku,
                product_name = %product.name,
                category = %product.category,
                last_purchase_count = %product.metrics.purchases,
                "Product out of stock"
            );
        }
    }
}

#[tracing::instrument(name = "price_analytics")]
async fn price_analytics_task(store: ProductStore) {
    let mut interval = tokio::time::interval(Duration::from_secs(45));

    loop {
        interval.tick().await;

        let products = store.read();
        if products.is_empty() {
            tracing::trace!("No products for price analytics");
            continue;
        }

        let prices: Vec<f64> = products.iter().map(|p| p.price).collect();
        let avg_price: f64 = prices.iter().sum::<f64>() / prices.len() as f64;
        let min_price = prices.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_price = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let categories: std::collections::HashMap<&str, usize> =
            products
                .iter()
                .fold(std::collections::HashMap::new(), |mut acc, p| {
                    *acc.entry(p.category.as_str()).or_insert(0) += 1;
                    acc
                });

        tracing::info!(
            product_count = %products.len(),
            avg_price = %format!("{:.2}", avg_price),
            min_price = %format!("{:.2}", min_price),
            max_price = %format!("{:.2}", max_price),
            price_range = %format!("{:.2}", max_price - min_price),
            categories = ?categories,
            "Price analytics snapshot"
        );
    }
}

#[tracing::instrument(name = "cache_warmer")]
async fn cache_warmer() {
    let mut interval = tokio::time::interval(Duration::from_secs(20));
    let cache_regions = [
        "product_list",
        "category_tree",
        "price_index",
        "search_index",
        "recommendations",
    ];
    let mut warm_count = 0u64;

    loop {
        interval.tick().await;
        warm_count += 1;

        let region = cache_regions[(warm_count as usize) % cache_regions.len()];
        let simulated_entries = (warm_count * 37) % 1000 + 100;
        let simulated_hit_rate = 0.85 + (warm_count % 15) as f64 * 0.01;

        tracing::trace!(
            cache_region = %region,
            entries_warmed = %simulated_entries,
            hit_rate = %format!("{:.2}", simulated_hit_rate),
            warm_cycle = %warm_count,
            ttl_seconds = 300,
            "Cache region warmed"
        );

        // Occasionally report cache pressure
        if warm_count.is_multiple_of(7) {
            tracing::debug!(
                cache_region = %region,
                memory_usage_mb = %(simulated_entries / 10),
                eviction_count = %(warm_count % 50),
                "Cache memory pressure detected"
            );
        }
    }
}

#[tracing::instrument(name = "list_products", skip(store))]
async fn list_products(
    Query(query): Query<SearchQuery>,
    State(store): State<ProductStore>,
) -> impl IntoResponse {
    let start = Instant::now();
    let request_id = Uuid::new_v4();

    tracing::debug!(
        request_id = %request_id,
        search = ?query.search,
        min_price = ?query.min_price,
        max_price = ?query.max_price,
        category = ?query.category,
        in_stock_only = ?query.in_stock,
        "Processing product search request"
    );

    let products = store.read();
    let initial_count = products.len();
    let mut results: Vec<Product> = products.clone();

    // Filter by search term
    if let Some(search_term) = &query.search {
        let before = results.len();
        results.retain(|p| {
            p.name.to_lowercase().contains(&search_term.to_lowercase())
                || p.description
                    .to_lowercase()
                    .contains(&search_term.to_lowercase())
                || p.tags
                    .iter()
                    .any(|t| t.to_lowercase().contains(&search_term.to_lowercase()))
        });
        tracing::trace!(
            request_id = %request_id,
            filter = "search",
            search_term = %search_term,
            before_count = %before,
            after_count = %results.len(),
            "Applied search filter"
        );
    }

    // Filter by category
    if let Some(category) = &query.category {
        let before = results.len();
        results.retain(|p| p.category.to_lowercase() == category.to_lowercase());
        tracing::trace!(
            request_id = %request_id,
            filter = "category",
            category = %category,
            before_count = %before,
            after_count = %results.len(),
            "Applied category filter"
        );
    }

    // Filter by price range
    if let Some(min) = query.min_price {
        let before = results.len();
        results.retain(|p| p.price >= min);
        tracing::trace!(
            request_id = %request_id,
            filter = "min_price",
            min_price = %min,
            before_count = %before,
            after_count = %results.len(),
            "Applied minimum price filter"
        );
    }

    if let Some(max) = query.max_price {
        let before = results.len();
        results.retain(|p| p.price <= max);
        tracing::trace!(
            request_id = %request_id,
            filter = "max_price",
            max_price = %max,
            before_count = %before,
            after_count = %results.len(),
            "Applied maximum price filter"
        );
    }

    // Filter by stock availability
    if let Some(true) = query.in_stock {
        let before = results.len();
        results.retain(|p| p.stock > 0);
        tracing::trace!(
            request_id = %request_id,
            filter = "in_stock",
            before_count = %before,
            after_count = %results.len(),
            "Applied in-stock filter"
        );
    }

    let query_duration = start.elapsed();

    tracing::info!(
        request_id = %request_id,
        initial_count = %initial_count,
        result_count = %results.len(),
        filters_applied = %(query.search.is_some() as u8 + query.category.is_some() as u8 + query.min_price.is_some() as u8 + query.max_price.is_some() as u8 + query.in_stock.is_some() as u8),
        query_duration_us = %query_duration.as_micros(),
        "Product search completed"
    );

    (StatusCode::OK, Json(results))
}

#[tracing::instrument(name = "create_product", skip(store))]
async fn create_product(
    State(store): State<ProductStore>,
    Json(req): Json<CreateProductRequest>,
) -> Response {
    let start = Instant::now();
    let request_id = Uuid::new_v4();

    tracing::debug!(
        request_id = %request_id,
        name = %req.name,
        price = %req.price,
        stock = %req.stock,
        category = ?req.category,
        tags = ?req.tags,
        "Validating product creation"
    );

    // Validation
    if req.name.is_empty() {
        tracing::warn!(
            request_id = %request_id,
            field = "name",
            rule = "required",
            "Product validation failed: empty name"
        );
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Product name cannot be empty",
                "field": "name",
                "code": "VALIDATION_ERROR"
            })),
        )
            .into_response();
    }

    if req.price <= 0.0 {
        tracing::warn!(
            request_id = %request_id,
            field = "price",
            value = %req.price,
            rule = "positive",
            "Product validation failed: invalid price"
        );
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Price must be greater than 0",
                "field": "price",
                "code": "VALIDATION_ERROR"
            })),
        )
            .into_response();
    }

    if req.stock < 0 {
        tracing::warn!(
            request_id = %request_id,
            field = "stock",
            value = %req.stock,
            rule = "non_negative",
            "Product validation failed: negative stock"
        );
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Stock cannot be negative",
                "field": "stock",
                "code": "VALIDATION_ERROR"
            })),
        )
            .into_response();
    }

    let product_id = Uuid::new_v4().to_string();
    let sku = format!("SKU-{}", &product_id[..8].to_uppercase());
    let category = req
        .category
        .clone()
        .unwrap_or_else(|| "uncategorized".to_string());
    let tags = req.tags.clone().unwrap_or_default();

    let product = Product {
        id: product_id.clone(),
        sku: sku.clone(),
        name: req.name.clone(),
        description: req.description.clone().unwrap_or_default(),
        price: req.price,
        stock: req.stock,
        category: category.clone(),
        tags: tags.clone(),
        metrics: ProductMetrics {
            views: 0,
            purchases: 0,
            avg_rating: 0.0,
            review_count: 0,
        },
    };

    tracing::info!(
        request_id = %request_id,
        product_id = %product_id,
        sku = %sku,
        name = %req.name,
        price = %req.price,
        stock = %req.stock,
        category = %category,
        tag_count = %tags.len(),
        processing_time_ms = %start.elapsed().as_millis(),
        "Product created successfully"
    );

    store.write().push(product.clone());

    // Simulate search index update
    let pid = product_id.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        tracing::debug!(
            product_id = %pid,
            index = "products",
            operation = "insert",
            "Search index updated"
        );
    });

    (StatusCode::CREATED, Json(product)).into_response()
}

#[tracing::instrument(name = "update_product", skip(store), fields(product_id = %id))]
async fn update_product(
    Path(id): Path<String>,
    State(store): State<ProductStore>,
    Json(req): Json<UpdateProductRequest>,
) -> Response {
    let start = Instant::now();
    let request_id = Uuid::new_v4();

    tracing::debug!(
        request_id = %request_id,
        product_id = %id,
        updates = ?req,
        "Processing product update"
    );

    let mut products = store.write();

    match products.iter_mut().find(|p| p.id == id) {
        Some(product) => {
            let mut changes = Vec::new();

            if let Some(new_name) = &req.name {
                tracing::trace!(
                    request_id = %request_id,
                    field = "name",
                    old_value = %product.name,
                    new_value = %new_name,
                    "Updating field"
                );
                changes.push("name");
                product.name = new_name.clone();
            }

            if let Some(new_desc) = &req.description {
                changes.push("description");
                product.description = new_desc.clone();
            }

            if let Some(new_price) = req.price {
                let price_change_pct = ((new_price - product.price) / product.price * 100.0).abs();
                tracing::trace!(
                    request_id = %request_id,
                    field = "price",
                    old_value = %product.price,
                    new_value = %new_price,
                    change_percent = %format!("{:.1}", price_change_pct),
                    "Updating field"
                );
                if price_change_pct > 20.0 {
                    tracing::warn!(
                        request_id = %request_id,
                        product_id = %id,
                        old_price = %product.price,
                        new_price = %new_price,
                        change_percent = %format!("{:.1}", price_change_pct),
                        "Large price change detected"
                    );
                }
                changes.push("price");
                product.price = new_price;
            }

            if let Some(new_stock) = req.stock {
                let stock_delta = new_stock - product.stock;
                tracing::trace!(
                    request_id = %request_id,
                    field = "stock",
                    old_value = %product.stock,
                    new_value = %new_stock,
                    delta = %stock_delta,
                    "Updating field"
                );
                changes.push("stock");
                product.stock = new_stock;
            }

            if let Some(new_category) = &req.category {
                changes.push("category");
                product.category = new_category.clone();
            }

            if let Some(new_tags) = &req.tags {
                changes.push("tags");
                product.tags = new_tags.clone();
            }

            tracing::info!(
                request_id = %request_id,
                product_id = %id,
                sku = %product.sku,
                fields_updated = ?changes,
                update_count = %changes.len(),
                processing_time_ms = %start.elapsed().as_millis(),
                "Product updated successfully"
            );

            (StatusCode::OK, Json(product.clone())).into_response()
        }
        None => {
            tracing::warn!(
                request_id = %request_id,
                product_id = %id,
                "Product not found for update"
            );
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Product not found",
                    "code": "NOT_FOUND",
                    "requested_id": id
                })),
            )
                .into_response()
        }
    }
}
