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
use std::time::Instant;
use tokio::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub metadata: UserMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMetadata {
    pub created_at: String,
    pub login_count: u32,
    pub last_ip: Option<String>,
    pub preferences: UserPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub theme: String,
    pub notifications_enabled: bool,
    pub language: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub role: Option<String>,
}

type UserStore = Arc<RwLock<Vec<User>>>;

pub fn router() -> Router {
    let store: UserStore = Arc::new(RwLock::new(Vec::new()));

    // Spawn background session cleanup task
    let cleanup_store = store.clone();
    tokio::spawn(session_cleanup_task(cleanup_store));

    // Spawn user activity simulator
    let activity_store = store.clone();
    tokio::spawn(user_activity_simulator(activity_store));

    Router::new()
        .route("/api/users", get(list_users))
        .route("/api/users", post(create_user))
        .route("/api/users/:id", get(get_user))
        .with_state(store)
}

#[tracing::instrument(name = "session_cleanup")]
async fn session_cleanup_task(store: UserStore) {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    let mut cleanup_count = 0u64;

    loop {
        interval.tick().await;
        cleanup_count += 1;

        let users = store.read();
        let active_sessions = users.len();
        let expired_sessions = (cleanup_count % 5) as usize; // Simulated

        tracing::debug!(
            cleanup_cycle = %cleanup_count,
            active_sessions = %active_sessions,
            expired_sessions = %expired_sessions,
            memory_freed_kb = %(expired_sessions * 128),
            "Session cleanup cycle completed"
        );

        if expired_sessions > 3 {
            tracing::info!(
                expired_count = %expired_sessions,
                "Cleaned up stale user sessions"
            );
        }
    }
}

#[tracing::instrument(name = "user_activity_monitor")]
async fn user_activity_simulator(store: UserStore) {
    let mut interval = tokio::time::interval(Duration::from_secs(8));
    let actions = [
        "page_view",
        "click",
        "scroll",
        "form_submit",
        "navigation",
        "api_call",
    ];
    let pages = [
        "/dashboard",
        "/settings",
        "/profile",
        "/orders",
        "/products",
        "/checkout",
    ];
    let mut event_id = 0u64;

    loop {
        interval.tick().await;
        event_id += 1;

        let users = store.read();
        if users.is_empty() {
            tracing::trace!("No active users to simulate activity for");
            continue;
        }

        let user_idx = (event_id as usize) % users.len();
        let user = &users[user_idx];
        let action = actions[(event_id as usize) % actions.len()];
        let page = pages[(event_id as usize / 2) % pages.len()];

        tracing::trace!(
            event_id = %event_id,
            user_id = %user.id,
            username = %user.username,
            action = %action,
            page = %page,
            session_duration_ms = %(event_id * 1500 % 300000),
            "User activity event"
        );

        // Occasionally log suspicious activity
        if event_id.is_multiple_of(17) {
            tracing::warn!(
                user_id = %user.id,
                username = %user.username,
                action = %action,
                page = %page,
                rate_limit_remaining = %(5 - (event_id % 6)),
                "Unusual activity pattern detected"
            );
        }
    }
}

#[tracing::instrument(name = "list_users", skip(store))]
async fn list_users(State(store): State<UserStore>) -> impl IntoResponse {
    let start = Instant::now();
    let request_id = Uuid::new_v4();

    tracing::info!(
        request_id = %request_id,
        "Processing list users request"
    );

    let users = store.read();
    let count = users.len();

    let roles_breakdown: std::collections::HashMap<&str, usize> =
        users
            .iter()
            .fold(std::collections::HashMap::new(), |mut acc, u| {
                *acc.entry(u.role.as_str()).or_insert(0) += 1;
                acc
            });

    tracing::debug!(
        request_id = %request_id,
        total_users = %count,
        roles = ?roles_breakdown,
        query_duration_us = %start.elapsed().as_micros(),
        "Users query completed"
    );

    (StatusCode::OK, Json(users.clone()))
}

#[tracing::instrument(name = "create_user", skip(store), fields(request_id = %Uuid::new_v4()))]
async fn create_user(
    State(store): State<UserStore>,
    Json(req): Json<CreateUserRequest>,
) -> Response {
    let start = Instant::now();

    tracing::debug!(
        username = %req.username,
        email = %req.email,
        role = ?req.role,
        "Validating user creation request"
    );

    // Simulate async validation
    tokio::time::sleep(Duration::from_millis(10)).await;

    if req.username.is_empty() {
        tracing::warn!(
            username = %req.username,
            validation_field = "username",
            validation_rule = "required",
            "Validation failed: empty username"
        );
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Username cannot be empty",
                "field": "username",
                "code": "VALIDATION_ERROR"
            })),
        )
            .into_response();
    }

    if !req.email.contains('@') {
        tracing::warn!(
            email = %req.email,
            validation_field = "email",
            validation_rule = "format",
            "Validation failed: invalid email format"
        );
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid email format",
                "field": "email",
                "code": "VALIDATION_ERROR"
            })),
        )
            .into_response();
    }

    // Check for duplicate email
    {
        let users = store.read();
        if users.iter().any(|u| u.email == req.email) {
            tracing::warn!(
                email = %req.email,
                validation_field = "email",
                validation_rule = "unique",
                "Validation failed: email already exists"
            );
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({
                    "error": "Email already registered",
                    "field": "email",
                    "code": "DUPLICATE_ERROR"
                })),
            )
                .into_response();
        }
    }

    let user_id = Uuid::new_v4().to_string();
    let role = req.role.clone().unwrap_or_else(|| "user".to_string());

    let user = User {
        id: user_id.clone(),
        username: req.username.clone(),
        email: req.email.clone(),
        role: role.clone(),
        metadata: UserMetadata {
            created_at: chrono::Utc::now().to_rfc3339(),
            login_count: 0,
            last_ip: None,
            preferences: UserPreferences {
                theme: "system".to_string(),
                notifications_enabled: true,
                language: "en".to_string(),
            },
        },
    };

    tracing::info!(
        user_id = %user_id,
        username = %req.username,
        email = %req.email,
        role = %role,
        processing_time_ms = %start.elapsed().as_millis(),
        "User created successfully"
    );

    store.write().push(user.clone());

    // Simulate sending welcome email async
    let email = req.email.clone();
    let uid = user_id.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(500)).await;
        tracing::debug!(
            user_id = %uid,
            email = %email,
            email_type = "welcome",
            template_id = "welcome_v2",
            "Welcome email queued for delivery"
        );
    });

    (StatusCode::CREATED, Json(user)).into_response()
}

#[tracing::instrument(name = "get_user", skip(store), fields(user_id = %id))]
async fn get_user(Path(id): Path<String>, State(store): State<UserStore>) -> Response {
    let start = Instant::now();

    tracing::trace!(
        user_id = %id,
        cache_checked = true,
        cache_hit = false,
        "Querying user by ID"
    );

    let users = store.read();

    match users.iter().find(|u| u.id == id) {
        Some(user) => {
            tracing::trace!(
                user_id = %id,
                username = %user.username,
                role = %user.role,
                login_count = %user.metadata.login_count,
                query_duration_us = %start.elapsed().as_micros(),
                "User found"
            );
            (StatusCode::OK, Json(serde_json::json!(user))).into_response()
        }
        None => {
            tracing::warn!(
                user_id = %id,
                query_duration_us = %start.elapsed().as_micros(),
                "User not found"
            );
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "User not found",
                    "code": "NOT_FOUND",
                    "requested_id": id
                })),
            )
                .into_response()
        }
    }
}
