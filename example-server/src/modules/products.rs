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
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub price: f64,
    pub stock: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub price: f64,
    pub stock: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub price: Option<f64>,
    pub stock: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub search: Option<String>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
}

type ProductStore = Arc<RwLock<Vec<Product>>>;

pub fn router() -> Router {
    let store: ProductStore = Arc::new(RwLock::new(Vec::new()));

    Router::new()
        .route("/api/products", get(list_products))
        .route("/api/products", post(create_product))
        .route("/api/products/:id", put(update_product))
        .with_state(store)
}

#[tracing::instrument(name = "list_products", skip(store))]
async fn list_products(
    Query(query): Query<SearchQuery>,
    State(store): State<ProductStore>,
) -> impl IntoResponse {
    // DEBUG level for search parameters
    tracing::debug!(
        search = ?query.search,
        min_price = ?query.min_price,
        max_price = ?query.max_price,
        "Processing product search request"
    );

    let products = store.read();
    let mut results: Vec<Product> = products.clone();

    // Filter by search term
    if let Some(search_term) = &query.search {
        tracing::debug!(search_term = %search_term, "Filtering products by search term");
        results.retain(|p| p.name.to_lowercase().contains(&search_term.to_lowercase()));
    }

    // Filter by price range
    if let Some(min) = query.min_price {
        tracing::debug!(min_price = %min, "Applying minimum price filter");
        results.retain(|p| p.price >= min);
    }

    if let Some(max) = query.max_price {
        tracing::debug!(max_price = %max, "Applying maximum price filter");
        results.retain(|p| p.price <= max);
    }

    tracing::info!(
        total_products = %products.len(),
        filtered_count = %results.len(),
        "Products list retrieved"
    );

    (StatusCode::OK, Json(results))
}

#[tracing::instrument(name = "create_product", skip(store))]
async fn create_product(
    State(store): State<ProductStore>,
    Json(req): Json<CreateProductRequest>,
) -> Response {
    // Validation
    if req.name.is_empty() {
        tracing::warn!(name = %req.name, "Product creation failed: empty name");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Product name cannot be empty"
            })),
        ).into_response();
    }

    if req.price <= 0.0 {
        tracing::warn!(price = %req.price, "Product creation failed: invalid price");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Price must be greater than 0"
            })),
        ).into_response();
    }

    if req.stock < 0 {
        tracing::warn!(stock = %req.stock, "Product creation failed: negative stock");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Stock cannot be negative"
            })),
        ).into_response();
    }

    let product_id = Uuid::new_v4().to_string();

    let product = Product {
        id: product_id.clone(),
        name: req.name.clone(),
        price: req.price,
        stock: req.stock,
    };

    // INFO level with structured fields
    tracing::info!(
        product_id = %product_id,
        product_name = %req.name,
        price = %req.price,
        stock = %req.stock,
        "Product created successfully"
    );

    store.write().push(product.clone());

    (StatusCode::CREATED, Json(product)).into_response()
}

#[tracing::instrument(name = "update_product", skip(store), fields(product_id = %id))]
async fn update_product(
    Path(id): Path<String>,
    State(store): State<ProductStore>,
    Json(req): Json<UpdateProductRequest>,
) -> Response {
    tracing::debug!(product_id = %id, "Attempting to update product");

    let mut products = store.write();

    match products.iter_mut().find(|p| p.id == id) {
        Some(product) => {
            // DEBUG level: log old and new values
            if let Some(new_name) = &req.name {
                tracing::debug!(
                    product_id = %id,
                    old_name = %product.name,
                    new_name = %new_name,
                    "Updating product name"
                );
                product.name = new_name.clone();
            }

            if let Some(new_price) = req.price {
                tracing::debug!(
                    product_id = %id,
                    old_price = %product.price,
                    new_price = %new_price,
                    "Updating product price"
                );
                product.price = new_price;
            }

            if let Some(new_stock) = req.stock {
                tracing::debug!(
                    product_id = %id,
                    old_stock = %product.stock,
                    new_stock = %new_stock,
                    "Updating product stock"
                );
                product.stock = new_stock;
            }

            tracing::info!(
                product_id = %id,
                product_name = %product.name,
                "Product updated successfully"
            );

            (StatusCode::OK, Json(product.clone())).into_response()
        }
        None => {
            tracing::warn!(product_id = %id, "Product not found for update");
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Product not found"
                })),
            ).into_response()
        }
    }
}
