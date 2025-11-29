//! Logs API for querying logs and streaming real-time events

use crate::storage::{LogEvent, LogFilter, LogStorage, SortOrder};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Shared state for logs API
#[derive(Clone)]
pub struct LogsState {
    pub storage: LogStorage,
}

impl LogsState {
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
    /// Sort order: "newest_first" (default) or "oldest_first"
    #[serde(default)]
    pub sort_order: Option<String>,
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
    State(state): State<Arc<LogsState>>,
    Json(request): Json<LogsRequest>,
) -> Response {
    // Parse sort order from request
    let sort_order = match request.sort_order.as_deref() {
        Some("oldest_first") => SortOrder::OldestFirst,
        _ => SortOrder::NewestFirst, // Default
    };

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
        sort_order,
    };

    // Get filtered logs (None limit means return all)
    let (logs, total_filtered) =
        state
            .storage
            .get_filtered(&filter, request.limit, Some(request.offset));

    let response = LogsResponse {
        logs,
        total: total_filtered,
    };

    Json(response).into_response()
}

/// GET /api/ws - WebSocket endpoint for real-time log streaming
pub async fn ws_logs(ws: WebSocketUpgrade, State(state): State<Arc<LogsState>>) -> Response {
    ws.on_upgrade(|socket| handle_ws_connection(socket, state))
}

/// Handle WebSocket connection for real-time log streaming
async fn handle_ws_connection(mut socket: WebSocket, state: Arc<LogsState>) {
    tracing::debug!("WebSocket connection established");

    // Subscribe to the broadcast channel to receive new log events
    let mut rx = state.storage.subscribe();

    // Ping interval to keep connection alive (every 30 seconds)
    let mut ping_interval = tokio::time::interval(std::time::Duration::from_secs(30));
    ping_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    // Send log events to the client as they arrive
    loop {
        tokio::select! {
            // Handle incoming log events from broadcast channel
            result = rx.recv() => {
                match result {
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

            // Handle incoming messages from client (ping/pong, close)
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Ping(data))) => {
                        // Respond to ping with pong
                        if socket.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        // Client responded to our ping - connection is alive
                    }
                    Some(Ok(Message::Close(_))) => {
                        // Client requested close
                        tracing::debug!("WebSocket client sent close frame");
                        break;
                    }
                    Some(Ok(_)) => {
                        // Ignore other message types
                    }
                    Some(Err(e)) => {
                        tracing::debug!("WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        // Connection closed
                        tracing::debug!("WebSocket connection closed by client");
                        break;
                    }
                }
            }

            // Send periodic ping to keep connection alive
            _ = ping_interval.tick() => {
                if socket.send(Message::Ping(vec![])).await.is_err() {
                    tracing::debug!("Failed to send ping, client disconnected");
                    break;
                }
            }
        }
    }

    tracing::debug!("WebSocket connection closed");
}

/// GET /api/targets - Get list of all unique targets
pub async fn get_targets(State(state): State<Arc<LogsState>>) -> Response {
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
            limit: Some(100),
            offset: 0,
            global_level: None,
            target_levels: HashMap::new(),
            search: None,
            target: None,
            sort_order: None,
        };

        assert_eq!(request.limit, Some(100));
        assert_eq!(request.offset, 0);
    }
}
