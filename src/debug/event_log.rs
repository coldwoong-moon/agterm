//! Event logging system for debugging terminal events
//!
//! Provides a circular buffer of terminal events with filtering capabilities.

use std::collections::VecDeque;
use std::time::Instant;

/// Types of events that can be logged
#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    /// PTY output received (bytes count)
    PtyOutput(usize),
    /// PTY input sent (bytes count)
    PtyInput(usize),
    /// Key press event (key name)
    KeyPress(String),
    /// Mouse click event (x, y)
    MouseClick(i32, i32),
    /// Bell event
    Bell,
    /// Terminal resize (cols, rows)
    Resize(u16, u16),
    /// OSC sequence received (sequence type)
    OscSequence(String),
    /// Escape sequence received (description)
    EscapeSequence(String),
    /// New tab created
    TabCreated,
    /// Tab closed
    TabClosed,
    /// Custom event with description
    Custom(String),
}

impl EventType {
    /// Get a short label for the event type
    pub fn label(&self) -> &str {
        match self {
            EventType::PtyOutput(_) => "PTY OUT",
            EventType::PtyInput(_) => "PTY IN",
            EventType::KeyPress(_) => "KEY",
            EventType::MouseClick(_, _) => "MOUSE",
            EventType::Bell => "BELL",
            EventType::Resize(_, _) => "RESIZE",
            EventType::OscSequence(_) => "OSC",
            EventType::EscapeSequence(_) => "ESC",
            EventType::TabCreated => "TAB+",
            EventType::TabClosed => "TAB-",
            EventType::Custom(_) => "CUSTOM",
        }
    }

    /// Check if this event type matches a filter pattern
    pub fn matches(&self, filter: &EventType) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(filter)
    }
}

/// A single log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// When the event occurred
    pub timestamp: Instant,
    /// Type of event
    pub event_type: EventType,
    /// Human-readable description
    pub description: String,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(event_type: EventType, description: String) -> Self {
        Self {
            timestamp: Instant::now(),
            event_type,
            description,
        }
    }

    /// Get elapsed time since the event occurred
    pub fn elapsed(&self) -> std::time::Duration {
        self.timestamp.elapsed()
    }

    /// Format elapsed time as a string
    pub fn elapsed_str(&self) -> String {
        let elapsed = self.elapsed();
        if elapsed.as_secs() < 1 {
            format!("{}ms", elapsed.as_millis())
        } else if elapsed.as_secs() < 60 {
            format!("{}s", elapsed.as_secs())
        } else {
            format!("{}m", elapsed.as_secs() / 60)
        }
    }
}

/// Event log with circular buffer and filtering
#[derive(Debug)]
pub struct EventLog {
    /// Log entries (circular buffer)
    entries: VecDeque<LogEntry>,
    /// Maximum number of entries to keep
    max_entries: usize,
    /// Optional filter (only log these event types)
    filter: Option<Vec<EventType>>,
}

impl EventLog {
    /// Create a new event log with the specified capacity
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_entries),
            max_entries,
            filter: None,
        }
    }

    /// Log a new event
    pub fn log(&mut self, event_type: EventType, description: String) {
        // Check if event passes filter
        if let Some(ref filter_list) = self.filter {
            if !filter_list.iter().any(|f| event_type.matches(f)) {
                return;
            }
        }

        // Add new entry
        let entry = LogEntry::new(event_type, description);

        // Remove oldest entry if at capacity
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }

        self.entries.push_back(entry);
    }

    /// Get all log entries
    pub fn entries(&self) -> &VecDeque<LogEntry> {
        &self.entries
    }

    /// Get the most recent N entries
    pub fn recent(&self, count: usize) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .rev()
            .take(count)
            .collect()
    }

    /// Clear all log entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Set the event type filter (None = no filter, Some = only these types)
    pub fn set_filter(&mut self, filter: Option<Vec<EventType>>) {
        self.filter = filter;
    }

    /// Get currently active filter
    pub fn filter(&self) -> Option<&Vec<EventType>> {
        self.filter.as_ref()
    }

    /// Get filtered entries matching the current filter
    pub fn filtered_entries(&self) -> Vec<&LogEntry> {
        if let Some(ref filter_list) = self.filter {
            self.entries
                .iter()
                .filter(|e| filter_list.iter().any(|f| e.event_type.matches(f)))
                .collect()
        } else {
            self.entries.iter().collect()
        }
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if log is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get maximum capacity
    pub fn capacity(&self) -> usize {
        self.max_entries
    }
}

impl Default for EventLog {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_log_basic() {
        let mut log = EventLog::new(10);

        log.log(EventType::KeyPress("A".to_string()), "Key A pressed".to_string());
        log.log(EventType::PtyOutput(100), "100 bytes received".to_string());

        assert_eq!(log.len(), 2);
        assert!(!log.is_empty());
    }

    #[test]
    fn test_event_log_capacity() {
        let mut log = EventLog::new(5);

        // Add more than capacity
        for i in 0..10 {
            log.log(
                EventType::Custom(format!("event{}", i)),
                format!("Event {}", i),
            );
        }

        // Should only keep the last 5
        assert_eq!(log.len(), 5);

        // First entry should be event5 (event0-4 were dropped)
        let entries = log.entries();
        if let EventType::Custom(ref s) = entries[0].event_type {
            assert_eq!(s, "event5");
        } else {
            panic!("Expected Custom event");
        }
    }

    #[test]
    fn test_event_log_clear() {
        let mut log = EventLog::new(10);

        log.log(EventType::Bell, "Bell rang".to_string());
        log.log(EventType::TabCreated, "Tab created".to_string());

        assert_eq!(log.len(), 2);

        log.clear();

        assert_eq!(log.len(), 0);
        assert!(log.is_empty());
    }

    #[test]
    fn test_event_log_filter() {
        let mut log = EventLog::new(100);

        // Set filter to only log key presses and PTY input
        log.set_filter(Some(vec![
            EventType::KeyPress(String::new()),
            EventType::PtyInput(0),
        ]));

        // Try to log various events
        log.log(EventType::KeyPress("A".to_string()), "Key A".to_string());
        log.log(EventType::PtyInput(10), "Input 10 bytes".to_string());
        log.log(EventType::PtyOutput(100), "Output 100 bytes".to_string()); // Should be filtered out
        log.log(EventType::Bell, "Bell".to_string()); // Should be filtered out

        // Only 2 events should have been logged
        assert_eq!(log.len(), 2);
    }

    #[test]
    fn test_event_log_filtered_entries() {
        let mut log = EventLog::new(100);

        // Add various events without filter
        log.log(EventType::KeyPress("A".to_string()), "Key A".to_string());
        log.log(EventType::PtyOutput(100), "Output".to_string());
        log.log(EventType::KeyPress("B".to_string()), "Key B".to_string());
        log.log(EventType::Bell, "Bell".to_string());

        assert_eq!(log.len(), 4);

        // Set filter
        log.set_filter(Some(vec![EventType::KeyPress(String::new())]));

        // filtered_entries should only return key press events
        let filtered = log.filtered_entries();
        assert_eq!(filtered.len(), 2);

        // But all entries should still be in the log
        assert_eq!(log.len(), 4);
    }

    #[test]
    fn test_event_type_label() {
        assert_eq!(EventType::KeyPress("A".to_string()).label(), "KEY");
        assert_eq!(EventType::PtyOutput(100).label(), "PTY OUT");
        assert_eq!(EventType::PtyInput(50).label(), "PTY IN");
        assert_eq!(EventType::Bell.label(), "BELL");
        assert_eq!(EventType::Resize(80, 24).label(), "RESIZE");
        assert_eq!(EventType::MouseClick(10, 20).label(), "MOUSE");
    }

    #[test]
    fn test_event_type_matches() {
        let key1 = EventType::KeyPress("A".to_string());
        let key2 = EventType::KeyPress("B".to_string());
        let bell = EventType::Bell;

        assert!(key1.matches(&EventType::KeyPress(String::new())));
        assert!(key2.matches(&EventType::KeyPress(String::new())));
        assert!(!bell.matches(&EventType::KeyPress(String::new())));
    }

    #[test]
    fn test_log_entry_elapsed() {
        let entry = LogEntry::new(EventType::Bell, "Test".to_string());

        // Should be very recent
        assert!(entry.elapsed().as_secs() < 1);

        // elapsed_str should return milliseconds for recent events
        let elapsed_str = entry.elapsed_str();
        assert!(elapsed_str.ends_with("ms"));
    }

    #[test]
    fn test_recent_entries() {
        let mut log = EventLog::new(100);

        for i in 0..10 {
            log.log(
                EventType::Custom(format!("{}", i)),
                format!("Event {}", i),
            );
        }

        // Get 5 most recent
        let recent = log.recent(5);
        assert_eq!(recent.len(), 5);

        // Should be in reverse order (most recent first)
        if let EventType::Custom(ref s) = recent[0].event_type {
            assert_eq!(s, "9"); // Most recent
        } else {
            panic!("Expected Custom event");
        }
    }
}
