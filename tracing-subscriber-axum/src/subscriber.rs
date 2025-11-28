//! Custom tracing subscriber that captures log events

use crate::storage::{LogEvent, LogStorage, SpanInfo};
use chrono::Utc;
use std::collections::HashMap;
use std::fmt;
use tracing::field::{Field, Visit};
use tracing::{Level, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

/// Visitor that collects fields from tracing events
struct FieldVisitor {
    fields: HashMap<String, String>,
}

impl FieldVisitor {
    fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }
}

impl Visit for FieldVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.fields
            .insert(field.name().to_string(), format!("{:?}", value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields.insert(field.name().to_string(), value.to_string());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }
}

/// Custom layer that captures tracing events and stores them
pub struct LogCaptureLayer {
    storage: LogStorage,
}

impl LogCaptureLayer {
    /// Create a new log capture layer
    pub fn new(storage: LogStorage) -> Self {
        Self { storage }
    }

    /// Extract the message from event fields
    fn extract_message(event: &tracing::Event) -> String {
        let mut visitor = FieldVisitor::new();
        event.record(&mut visitor);

        // Try to get the message field first
        if let Some(message) = visitor.fields.get("message") {
            return message.clone();
        }

        // If no message field, join all fields
        visitor
            .fields
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Convert tracing Level to string
    fn level_to_string(level: &Level) -> String {
        match *level {
            Level::TRACE => "TRACE",
            Level::DEBUG => "DEBUG",
            Level::INFO => "INFO",
            Level::WARN => "WARN",
            Level::ERROR => "ERROR",
        }
        .to_string()
    }

    /// Extract span information from the current context
    fn extract_span_info<S>(event: &tracing::Event<'_>, ctx: &Context<'_, S>) -> Option<SpanInfo>
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        let span = ctx.event_span(event)?;
        let ext = span.extensions();

        // Get span name and fields
        let name = span.name().to_string();
        let mut fields = HashMap::new();

        // Try to collect span fields
        if let Some(field_visitor) = ext.get::<FieldVisitor>() {
            fields = field_visitor.fields.clone();
        }

        Some(SpanInfo { name, fields })
    }
}

/// Targets to filter out to avoid noise and recursive logging
const FILTERED_TARGETS: &[&str] = &[
    "log",                      // log crate compatibility layer
    "tracing_subscriber_axum",  // our own crate (avoid recursion)
    "tungstenite",              // WebSocket library internals
    "tokio_tungstenite",        // async WebSocket library internals
];

impl<S> Layer<S> for LogCaptureLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &tracing::Event<'_>, ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let target = metadata.target();

        // Extract all fields including the message
        let mut visitor = FieldVisitor::new();
        event.record(&mut visitor);

        // Determine the actual target - for events from the log crate bridge,
        // the real target is in the "log.target" field
        let actual_target = visitor
            .fields
            .get("log.target")
            .cloned()
            .unwrap_or_else(|| target.to_string());

        // Filter out noisy targets (check actual target, not metadata target)
        for filtered in FILTERED_TARGETS {
            if actual_target == *filtered || actual_target.starts_with(&format!("{}::", filtered)) {
                return;
            }
        }

        // Extract message separately
        let message = Self::extract_message(event);

        // Remove "message" and log crate fields from fields to avoid duplication/noise
        visitor.fields.remove("message");
        visitor.fields.remove("log.target");
        visitor.fields.remove("log.module_path");
        visitor.fields.remove("log.file");
        visitor.fields.remove("log.line");

        // Create log event
        let log_event = LogEvent {
            timestamp: Utc::now(),
            level: Self::level_to_string(metadata.level()),
            target: actual_target,
            message,
            fields: visitor.fields,
            span: Self::extract_span_info(event, &ctx),
        };

        // Store the event
        self.storage.push(log_event);
    }

    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::span::Id,
        ctx: Context<'_, S>,
    ) {
        // Store span fields for later use
        let span = ctx.span(id).expect("Span not found");
        let mut visitor = FieldVisitor::new();
        attrs.record(&mut visitor);

        let mut extensions = span.extensions_mut();
        extensions.insert(visitor);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_conversion() {
        assert_eq!(LogCaptureLayer::level_to_string(&Level::TRACE), "TRACE");
        assert_eq!(LogCaptureLayer::level_to_string(&Level::DEBUG), "DEBUG");
        assert_eq!(LogCaptureLayer::level_to_string(&Level::INFO), "INFO");
        assert_eq!(LogCaptureLayer::level_to_string(&Level::WARN), "WARN");
        assert_eq!(LogCaptureLayer::level_to_string(&Level::ERROR), "ERROR");
    }

    #[test]
    fn test_field_visitor() {
        let mut visitor = FieldVisitor::new();
        assert_eq!(visitor.fields.len(), 0);

        // FieldVisitor is tested implicitly through the subscriber integration tests
        // Direct testing requires complex tracing infrastructure setup
    }

    #[test]
    fn test_log_capture_layer_creation() {
        let storage = LogStorage::new();
        let layer = LogCaptureLayer::new(storage);

        // Layer should be created successfully
        assert_eq!(layer.storage.len(), 0);
    }
}
