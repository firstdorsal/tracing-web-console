//! Control API for querying logs and streaming real-time events

use crate::storage::{LogEvent, LogFilter, LogStorage};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Shared state for control API
#[derive(Clone)]
pub struct ControlState {
    pub storage: LogStorage,
}

impl ControlState {
    pub fn new(storage: LogStorage) -> Self {
        Self { storage }
    }
}

/// Request body for POST /api/logs
#[derive(Debug, Deserialize)]
pub struct LogsRequest {
    /// Maximum number of logs to return (None = return all)
    pub limit: Option<usize>,
    /// Offset for pagination
    #[serde(default)]
    pub offset: usize,
    /// Global log level filter (e.g., "INFO", "DEBUG")
    pub global_level: Option<String>,
    /// Target-specific log level filters (e.g., {"my_crate": "DEBUG", "other_crate": "INFO"})
    #[serde(default)]
    pub target_levels: HashMap<String, String>,
    /// Search filter for message content (case-insensitive)
    pub search: Option<String>,
    /// Target filter (case-insensitive contains match)
    pub target: Option<String>,
}

/// Response for GET /api/logs
#[derive(Debug, Serialize)]
pub struct LogsResponse {
    pub logs: Vec<LogEvent>,
    pub total: usize,
}

/// Response for GET /api/targets
#[derive(Debug, Serialize)]
pub struct TargetsResponse {
    pub targets: Vec<String>,
}

/// POST /api/logs - Get historical logs with optional filters
pub async fn get_logs(
    State(state): State<Arc<ControlState>>,
    Json(request): Json<LogsRequest>,
) -> Response {
    // Build filter from request
    let filter = LogFilter {
        global_level: request.global_level.map(|l| l.to_uppercase()),
        target_levels: request
            .target_levels
            .iter()
            .map(|(k, v)| (k.clone(), v.to_uppercase()))
            .collect(),
        search: request.search.filter(|s| !s.is_empty()),
        target: request.target.filter(|t| !t.is_empty()),
    };

    // Get filtered logs (None limit means return all)
    let (logs, total_filtered) = state
        .storage
        .get_filtered(&filter, request.limit, Some(request.offset));

    let response = LogsResponse {
        logs,
        total: total_filtered,
    };

    Json(response).into_response()
}

/// GET /api/ws - WebSocket endpoint for real-time log streaming
pub async fn ws_logs(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ControlState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_ws_connection(socket, state))
}

/// Handle WebSocket connection for real-time log streaming
async fn handle_ws_connection(mut socket: WebSocket, state: Arc<ControlState>) {
    tracing::debug!("WebSocket connection established");

    // Subscribe to the broadcast channel to receive new log events
    let mut rx = state.storage.subscribe();

    // Send log events to the client as they arrive
    loop {
        match rx.recv().await {
            Ok(log_event) => {
                // Serialize the log event to JSON
                let json = match serde_json::to_string(&log_event) {
                    Ok(json) => json,
                    Err(e) => {
                        tracing::error!("Failed to serialize log event: {}", e);
                        continue;
                    }
                };

                // Send the JSON message to the client
                if socket.send(Message::Text(json)).await.is_err() {
                    // Client disconnected
                    tracing::debug!("WebSocket client disconnected");
                    break;
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(count)) => {
                // Receiver fell behind, some messages were dropped - continue receiving
                tracing::debug!("WebSocket receiver lagged, missed {} messages", count);
                continue;
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                // Broadcast channel closed - exit
                tracing::warn!("Broadcast channel closed");
                break;
            }
        }
    }

    tracing::debug!("WebSocket connection closed");
}

/// GET /api/targets - Get list of all unique targets
pub async fn get_targets(State(state): State<Arc<ControlState>>) -> Response {
    let targets = state.storage.get_targets();
    let response = TargetsResponse { targets };
    Json(response).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logs_request_default() {
        let request = LogsRequest {
            limit: default_limit(),
            offset: default_offset(),
            level: vec![],
            target: None,
            search: None,
        };

        assert_eq!(request.limit, 100);
        assert_eq!(request.offset, 0);
    }
}
