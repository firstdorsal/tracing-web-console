//! Log storage with circular buffer implementation

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Maximum number of log events to store in memory
const DEFAULT_MAX_EVENTS: usize = 10_000;
/// Capacity of the broadcast channel for real-time log streaming
const BROADCAST_CAPACITY: usize = 100;

/// A single log event captured by the subscriber
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub target: String,
    pub message: String,
    pub fields: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<SpanInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

/// Information about the span context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanInfo {
    pub name: String,
    pub fields: HashMap<String, String>,
}

/// Sort order for log queries
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SortOrder {
    /// Newest logs first (default)
    #[default]
    NewestFirst,
    /// Oldest logs first
    OldestFirst,
}

/// Filters for querying log events
#[derive(Debug, Clone, Default)]
pub struct LogFilter {
    pub global_level: Option<String>,
    pub target_levels: HashMap<String, String>,
    pub search: Option<String>,
    pub target: Option<String>,
    pub sort_order: SortOrder,
}

/// Convert log level string to numeric value for comparison
/// Higher number = higher severity (ERROR > WARN > INFO > DEBUG > TRACE)
fn level_to_number(level: &str) -> u8 {
    match level.to_uppercase().as_str() {
        "ERROR" => 5,
        "WARN" => 4,
        "INFO" => 3,
        "DEBUG" => 2,
        "TRACE" => 1,
        _ => 0, // Unknown levels are lowest priority
    }
}

/// Thread-safe circular buffer for storing log events
#[derive(Clone)]
pub struct LogStorage {
    events: Arc<RwLock<VecDeque<LogEvent>>>,
    max_events: usize,
    tx: broadcast::Sender<LogEvent>,
}

impl LogStorage {
    /// Create a new log storage with default capacity
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_MAX_EVENTS)
    }

    /// Create a new log storage with specified capacity
    pub fn with_capacity(max_events: usize) -> Self {
        let (tx, _) = broadcast::channel(BROADCAST_CAPACITY);
        Self {
            events: Arc::new(RwLock::new(VecDeque::with_capacity(max_events))),
            max_events,
            tx,
        }
    }

    /// Add a new log event, removing oldest if at capacity
    pub fn push(&self, event: LogEvent) {
        let mut events = self.events.write();

        if events.len() >= self.max_events {
            events.pop_front();
        }

        // Send to broadcast channel, ignore if no receivers
        let _ = self.tx.send(event.clone());

        events.push_back(event);
    }

    /// Subscribe to real-time log events
    pub fn subscribe(&self) -> broadcast::Receiver<LogEvent> {
        self.tx.subscribe()
    }

    /// Get all log events matching the filter
    pub fn get_filtered(
        &self,
        filter: &LogFilter,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> (Vec<LogEvent>, usize) {
        let events = self.events.read();
        let offset = offset.unwrap_or(0);

        let filtered: Vec<LogEvent> = events
            .iter()
            .filter(|event| self.matches_filter(event, filter))
            .cloned()
            .collect();

        let total_filtered = filtered.len();

        // Apply sort order and pagination
        let paginated: Vec<LogEvent> = match filter.sort_order {
            SortOrder::NewestFirst => {
                // Reverse to get newest first, then paginate
                filtered
                    .into_iter()
                    .rev()
                    .skip(offset)
                    .take(limit.unwrap_or(usize::MAX))
                    .collect()
            }
            SortOrder::OldestFirst => {
                // Keep natural order (oldest first), then paginate
                filtered
                    .into_iter()
                    .skip(offset)
                    .take(limit.unwrap_or(usize::MAX))
                    .collect()
            }
        };

        (paginated, total_filtered)
    }

    /// Get all unique targets from stored events
    pub fn get_targets(&self) -> Vec<String> {
        let events = self.events.read();
        let mut targets: Vec<String> = events
            .iter()
            .map(|e| e.target.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        targets.sort();
        targets
    }

    /// Check if storage is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.events.read().is_empty()
    }

    /// Clear all stored events
    #[allow(dead_code)]
    pub fn clear(&self) {
        self.events.write().clear();
    }

    /// Check if an event matches the filter criteria
    fn matches_filter(&self, event: &LogEvent, filter: &LogFilter) -> bool {
        // Determine the required log level for this event's target
        // Target filters take precedence over global level
        // Use prefix matching: "my_crate" matches "my_crate::module::thing"
        let target_level = filter
            .target_levels
            .iter()
            .filter(|(target, _)| {
                event.target == **target || event.target.starts_with(&format!("{}::", target))
            })
            // If multiple matches, use the most specific (longest) target
            .max_by_key(|(target, _)| target.len())
            .map(|(_, level)| level);

        // Target-specific level takes precedence, then fall back to global level
        let required_level = target_level.or(filter.global_level.as_ref());

        // If a level filter is specified, check if event level meets it
        if let Some(level_str) = required_level {
            let event_level_num = level_to_number(&event.level);
            let required_level_num = level_to_number(level_str);

            // Event level must be >= required level (higher severity)
            if event_level_num < required_level_num {
                return false;
            }
        }

        // Filter by target (case-insensitive contains)
        if let Some(ref target_filter) = filter.target {
            if !event
                .target
                .to_lowercase()
                .contains(&target_filter.to_lowercase())
            {
                return false;
            }
        }

        // Filter by search term in message (case-insensitive contains)
        if let Some(ref search) = filter.search {
            if !event
                .message
                .to_lowercase()
                .contains(&search.to_lowercase())
            {
                return false;
            }
        }

        true
    }
}

impl Default for LogStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_event(level: &str, target: &str, message: &str) -> LogEvent {
        LogEvent {
            timestamp: Utc::now(),
            level: level.to_string(),
            target: target.to_string(),
            message: message.to_string(),
            fields: HashMap::new(),
            span: None,
            file: None,
            line: None,
        }
    }

    #[test]
    fn test_circular_buffer() {
        let storage = LogStorage::with_capacity(3);

        storage.push(create_test_event("INFO", "test", "msg1"));
        storage.push(create_test_event("INFO", "test", "msg2"));
        storage.push(create_test_event("INFO", "test", "msg3"));

        let filter = LogFilter::default();
        let (_events, count) = storage.get_filtered(&filter, None, None);
        assert_eq!(count, 3);

        // Adding 4th should remove oldest
        storage.push(create_test_event("INFO", "test", "msg4"));

        let (events, count) = storage.get_filtered(&filter, None, None);
        assert_eq!(count, 3);
        // NewestFirst by default, so msg4 should be first
        assert_eq!(events[0].message, "msg4");
        assert_eq!(events[2].message, "msg2");
    }

    #[test]
    fn test_level_filter() {
        let storage = LogStorage::new();

        storage.push(create_test_event("INFO", "test", "info msg"));
        storage.push(create_test_event("ERROR", "test", "error msg"));
        storage.push(create_test_event("DEBUG", "test", "debug msg"));

        let filter = LogFilter {
            global_level: Some("ERROR".to_string()),
            ..Default::default()
        };

        let (filtered, count) = storage.get_filtered(&filter, None, None);
        assert_eq!(count, 1);
        assert_eq!(filtered[0].level, "ERROR");
    }

    #[test]
    fn test_search_filter() {
        let storage = LogStorage::new();

        storage.push(create_test_event("INFO", "test", "hello world"));
        storage.push(create_test_event("INFO", "test", "goodbye world"));
        storage.push(create_test_event("INFO", "test", "testing"));

        let filter = LogFilter {
            search: Some("hello".to_string()),
            ..Default::default()
        };

        let (filtered, count) = storage.get_filtered(&filter, None, None);
        assert_eq!(count, 1);
        assert!(filtered[0].message.contains("hello"));
    }
}
