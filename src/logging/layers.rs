//! Custom tracing subscriber layers
//!
//! Provides specialized layers for:
//! - In-memory log buffer for debug panel display
//! - Metrics collection

use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::{field::Visit, Event, Level, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

/// A log entry stored in the buffer
#[derive(Clone, Debug)]
pub struct LogEntry {
    /// Timestamp when the log was recorded
    pub timestamp: Instant,
    /// Log level
    pub level: Level,
    /// Target module path
    pub target: String,
    /// Log message
    pub message: String,
    /// Additional fields
    pub fields: Vec<(String, String)>,
}

impl LogEntry {
    /// Get the level as a displayable string with color hint
    pub fn level_str(&self) -> &'static str {
        match self.level {
            Level::TRACE => "TRACE",
            Level::DEBUG => "DEBUG",
            Level::INFO => "INFO",
            Level::WARN => "WARN",
            Level::ERROR => "ERROR",
        }
    }
}

/// Visitor to extract message and fields from tracing events
struct FieldVisitor {
    message: Option<String>,
    fields: Vec<(String, String)>,
}

impl FieldVisitor {
    fn new() -> Self {
        Self {
            message: None,
            fields: Vec::new(),
        }
    }
}

impl Visit for FieldVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn fmt::Debug) {
        let value_str = format!("{:?}", value);
        if field.name() == "message" {
            self.message = Some(value_str);
        } else {
            self.fields.push((field.name().to_string(), value_str));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
        } else {
            self.fields.push((field.name().to_string(), value.to_string()));
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.fields.push((field.name().to_string(), value.to_string()));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.fields.push((field.name().to_string(), value.to_string()));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.fields.push((field.name().to_string(), value.to_string()));
    }
}

/// In-memory log buffer layer for the debug panel
///
/// This layer collects log entries into a ring buffer that can be
/// displayed in the debug UI panel.
pub struct LogBufferLayer {
    buffer: Arc<Mutex<VecDeque<LogEntry>>>,
    max_size: usize,
}

impl LogBufferLayer {
    /// Create a new log buffer layer with the specified maximum size
    pub fn new(max_size: usize) -> (Self, LogBuffer) {
        let buffer = Arc::new(Mutex::new(VecDeque::with_capacity(max_size)));
        let handle = LogBuffer {
            buffer: Arc::clone(&buffer),
        };
        let layer = Self { buffer, max_size };
        (layer, handle)
    }
}

impl<S> Layer<S> for LogBufferLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = FieldVisitor::new();
        event.record(&mut visitor);

        let entry = LogEntry {
            timestamp: Instant::now(),
            level: *event.metadata().level(),
            target: event.metadata().target().to_string(),
            message: visitor.message.unwrap_or_default(),
            fields: visitor.fields,
        };

        if let Ok(mut buffer) = self.buffer.lock() {
            if buffer.len() >= self.max_size {
                buffer.pop_front();
            }
            buffer.push_back(entry);
        }
    }
}

/// Handle to read from the log buffer
#[derive(Clone)]
pub struct LogBuffer {
    buffer: Arc<Mutex<VecDeque<LogEntry>>>,
}

impl std::fmt::Debug for LogBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.buffer.lock().map(|g| g.len()).unwrap_or(0);
        f.debug_struct("LogBuffer")
            .field("entries", &len)
            .finish()
    }
}

impl LogBuffer {
    /// Get a snapshot of the current log entries
    pub fn get_entries(&self) -> Vec<LogEntry> {
        self.buffer
            .lock()
            .map(|guard| guard.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get the most recent N entries
    pub fn get_recent(&self, count: usize) -> Vec<LogEntry> {
        self.buffer
            .lock()
            .map(|guard| {
                let len = guard.len();
                let skip = len.saturating_sub(count);
                guard.iter().skip(skip).cloned().collect()
            })
            .unwrap_or_default()
    }

    /// Filter entries by log level
    pub fn filter_by_level(&self, min_level: Level) -> Vec<LogEntry> {
        self.buffer
            .lock()
            .map(|guard| {
                guard
                    .iter()
                    .filter(|e| e.level <= min_level)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Search entries by message content
    pub fn search(&self, query: &str) -> Vec<LogEntry> {
        let query_lower = query.to_lowercase();
        self.buffer
            .lock()
            .map(|guard| {
                guard
                    .iter()
                    .filter(|e| {
                        e.message.to_lowercase().contains(&query_lower)
                            || e.target.to_lowercase().contains(&query_lower)
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Clear all entries
    pub fn clear(&self) {
        if let Ok(mut guard) = self.buffer.lock() {
            guard.clear();
        }
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.buffer.lock().map(|guard| guard.len()).unwrap_or(0)
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_level_str() {
        let entry = LogEntry {
            timestamp: Instant::now(),
            level: Level::INFO,
            target: "test".to_string(),
            message: "test message".to_string(),
            fields: vec![],
        };
        assert_eq!(entry.level_str(), "INFO");
    }

    #[test]
    fn test_log_buffer_operations() {
        let (layer, buffer) = LogBufferLayer::new(10);

        // Simulate adding entries directly to buffer
        {
            let mut guard = layer.buffer.lock().unwrap();
            for i in 0..5 {
                guard.push_back(LogEntry {
                    timestamp: Instant::now(),
                    level: Level::INFO,
                    target: "test".to_string(),
                    message: format!("message {}", i),
                    fields: vec![],
                });
            }
        }

        assert_eq!(buffer.len(), 5);
        assert!(!buffer.is_empty());

        let recent = buffer.get_recent(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].message, "message 2");

        buffer.clear();
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_log_buffer_max_size() {
        let (layer, buffer) = LogBufferLayer::new(5);

        {
            let mut guard = layer.buffer.lock().unwrap();
            for i in 0..10 {
                if guard.len() >= 5 {
                    guard.pop_front();
                }
                guard.push_back(LogEntry {
                    timestamp: Instant::now(),
                    level: Level::INFO,
                    target: "test".to_string(),
                    message: format!("message {}", i),
                    fields: vec![],
                });
            }
        }

        // Should only have last 5 entries
        assert_eq!(buffer.len(), 5);
        let entries = buffer.get_entries();
        assert_eq!(entries[0].message, "message 5");
        assert_eq!(entries[4].message, "message 9");
    }

    #[test]
    fn test_log_buffer_search() {
        let (layer, buffer) = LogBufferLayer::new(10);

        {
            let mut guard = layer.buffer.lock().unwrap();
            guard.push_back(LogEntry {
                timestamp: Instant::now(),
                level: Level::INFO,
                target: "pty".to_string(),
                message: "PTY session created".to_string(),
                fields: vec![],
            });
            guard.push_back(LogEntry {
                timestamp: Instant::now(),
                level: Level::ERROR,
                target: "main".to_string(),
                message: "Error occurred".to_string(),
                fields: vec![],
            });
        }

        let results = buffer.search("PTY");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].target, "pty");

        let results = buffer.search("error");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].level, Level::ERROR);
    }
}
