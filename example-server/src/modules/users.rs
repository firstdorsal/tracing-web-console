use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
}

type UserStore = Arc<RwLock<Vec<User>>>;

pub fn router() -> Router {
    let store: UserStore = Arc::new(RwLock::new(Vec::new()));

    Router::new()
        .route("/api/users", get(list_users))
        .route("/api/users", post(create_user))
        .route("/api/users/:id", get(get_user))
        .with_state(store)
}

#[tracing::instrument(name = "list_users", skip(store))]
async fn list_users(State(store): State<UserStore>) -> impl IntoResponse {
    tracing::info!("Fetching users list");

    let users = store.read();
    let count = users.len();

    tracing::debug!(count = %count, "Retrieved users from store");

    (StatusCode::OK, Json(users.clone()))
}

#[tracing::instrument(name = "create_user", skip(store), fields(request_id = %Uuid::new_v4()))]
async fn create_user(
    State(store): State<UserStore>,
    Json(req): Json<CreateUserRequest>,
) -> Response {
    // Validation with DEBUG level
    tracing::debug!(
        username = %req.username,
        email = %req.email,
        "Validating user creation request"
    );

    if req.username.is_empty() {
        tracing::warn!(username = %req.username, "Username validation failed: empty username");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Username cannot be empty"
            })),
        ).into_response();
    }

    if !req.email.contains('@') {
        tracing::warn!(email = %req.email, "Email validation failed: invalid format");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid email format"
            })),
        ).into_response();
    }

    let user_id = Uuid::new_v4().to_string();

    let user = User {
        id: user_id.clone(),
        username: req.username.clone(),
        email: req.email.clone(),
    };

    // INFO level for successful creation
    tracing::info!(
        user_id = %user_id,
        username = %req.username,
        email = %req.email,
        "User created successfully"
    );

    store.write().push(user.clone());

    (StatusCode::CREATED, Json(user)).into_response()
}

#[tracing::instrument(name = "get_user", skip(store), fields(user_id = %id))]
async fn get_user(
    Path(id): Path<String>,
    State(store): State<UserStore>,
) -> Response {
    // TRACE level for detailed query logging
    tracing::trace!(user_id = %id, "Querying user by ID");

    let users = store.read();

    match users.iter().find(|u| u.id == id) {
        Some(user) => {
            tracing::trace!(user_id = %id, username = %user.username, "User found");
            (StatusCode::OK, Json(serde_json::json!(user))).into_response()
        }
        None => {
            // WARN level when user not found
            tracing::warn!(user_id = %id, "User not found");
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "User not found"
                })),
            ).into_response()
        }
    }
}
