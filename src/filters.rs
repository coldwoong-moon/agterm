//! Terminal Output Filter System
//!
//! Provides real-time filtering, highlighting, and transformation of terminal output.
//! Supports regex patterns, multiple actions (hide/highlight/replace/notify), and filter groups.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// Filter action to apply when pattern matches
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterAction {
    /// Hide matching lines from output
    Hide,
    /// Highlight matching text with specified color
    Highlight {
        /// RGB color for highlighting (r, g, b in 0-255 range)
        color: (u8, u8, u8),
        /// Optional background color
        bg_color: Option<(u8, u8, u8)>,
    },
    /// Replace matching text with replacement string
    Replace {
        /// Replacement text (supports capture groups: $1, $2, etc.)
        replacement: String,
    },
    /// Send desktop notification when pattern matches
    Notify {
        /// Notification title
        title: String,
        /// Optional notification body (defaults to matched text)
        body: Option<String>,
        /// Sound notification
        sound: bool,
    },
}

/// A single filter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Regex pattern to match
    #[serde(skip)]
    pub pattern: Option<Arc<Regex>>,
    /// Pattern string (for serialization)
    pub pattern_str: String,
    /// Action to apply on match
    pub action: FilterAction,
    /// Whether this filter is active
    pub enabled: bool,
    /// Filter group (for organizing filters)
    pub group: Option<String>,
    /// Priority (higher = processed first)
    pub priority: i32,
    /// Case insensitive matching
    pub case_insensitive: bool,
    /// Match statistics
    #[serde(skip)]
    pub stats: FilterStats,
}

impl Filter {
    /// Create a new filter
    pub fn new(
        id: String,
        name: String,
        pattern: String,
        action: FilterAction,
    ) -> Result<Self, FilterError> {
        let regex = Regex::new(&pattern)?;
        Ok(Self {
            id,
            name,
            description: None,
            pattern: Some(Arc::new(regex)),
            pattern_str: pattern,
            action,
            enabled: true,
            group: None,
            priority: 0,
            case_insensitive: false,
            stats: FilterStats::default(),
        })
    }

    /// Create a new case-insensitive filter
    pub fn new_case_insensitive(
        id: String,
        name: String,
        pattern: String,
        action: FilterAction,
    ) -> Result<Self, FilterError> {
        let pattern_with_flag = format!("(?i){pattern}");
        let regex = Regex::new(&pattern_with_flag)?;
        Ok(Self {
            id,
            name,
            description: None,
            pattern: Some(Arc::new(regex)),
            pattern_str: pattern,
            action,
            enabled: true,
            group: None,
            priority: 0,
            case_insensitive: true,
            stats: FilterStats::default(),
        })
    }

    /// Check if this filter matches the given text
    pub fn matches(&self, text: &str) -> bool {
        if !self.enabled {
            return false;
        }
        self.pattern
            .as_ref()
            .map(|p| p.is_match(text))
            .unwrap_or(false)
    }

    /// Apply this filter to text and return the result
    pub fn apply(&mut self, text: &str) -> FilterResult {
        if !self.enabled {
            return FilterResult::NoMatch;
        }

        let pattern = match self.pattern.as_ref() {
            Some(p) => p,
            None => return FilterResult::NoMatch,
        };

        if !pattern.is_match(text) {
            return FilterResult::NoMatch;
        }

        // Update statistics
        self.stats.increment();

        match &self.action {
            FilterAction::Hide => FilterResult::Hide,
            FilterAction::Highlight { color, bg_color } => {
                FilterResult::Highlight {
                    color: *color,
                    bg_color: *bg_color,
                    ranges: pattern
                        .find_iter(text)
                        .map(|m| (m.start(), m.end()))
                        .collect(),
                }
            }
            FilterAction::Replace { replacement } => {
                let result = pattern.replace_all(text, replacement.as_str());
                FilterResult::Replace {
                    text: result.into_owned(),
                }
            }
            FilterAction::Notify { title, body, sound } => {
                let matched_text = pattern.find(text).map(|m| m.as_str().to_string());
                FilterResult::Notify {
                    title: title.clone(),
                    body: body.clone().or(matched_text),
                    sound: *sound,
                }
            }
        }
    }

    /// Compile/recompile the regex pattern
    pub fn compile(&mut self) -> Result<(), FilterError> {
        let pattern_str = if self.case_insensitive {
            format!("(?i){}", self.pattern_str)
        } else {
            self.pattern_str.clone()
        };
        let regex = Regex::new(&pattern_str)?;
        self.pattern = Some(Arc::new(regex));
        Ok(())
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats.reset();
    }
}

/// Result of applying a filter
#[derive(Debug, Clone)]
pub enum FilterResult {
    /// No match
    NoMatch,
    /// Hide this line
    Hide,
    /// Highlight matched ranges
    Highlight {
        color: (u8, u8, u8),
        bg_color: Option<(u8, u8, u8)>,
        ranges: Vec<(usize, usize)>,
    },
    /// Replace with new text
    Replace { text: String },
    /// Send notification
    Notify {
        title: String,
        body: Option<String>,
        sound: bool,
    },
}

/// Filter matching statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilterStats {
    /// Total number of matches
    pub match_count: u64,
    /// Last match timestamp (Unix epoch seconds)
    pub last_match: Option<u64>,
}

impl FilterStats {
    /// Increment match count and update timestamp
    fn increment(&mut self) {
        self.match_count += 1;
        self.last_match = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
    }

    /// Reset statistics
    fn reset(&mut self) {
        self.match_count = 0;
        self.last_match = None;
    }
}

/// Filter manager for organizing and managing filters
#[derive(Debug, Clone)]
pub struct FilterManager {
    /// All registered filters (indexed by ID)
    filters: HashMap<String, Filter>,
    /// Filter groups (group name -> filter IDs)
    groups: HashMap<String, Vec<String>>,
    /// Ordered filter IDs (by priority)
    order: Vec<String>,
}

impl Default for FilterManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterManager {
    /// Create a new filter manager
    pub fn new() -> Self {
        Self {
            filters: HashMap::new(),
            groups: HashMap::new(),
            order: Vec::new(),
        }
    }

    /// Add a filter
    pub fn add_filter(&mut self, filter: Filter) -> Result<(), FilterError> {
        let id = filter.id.clone();

        // Add to group if specified
        if let Some(group) = &filter.group {
            self.groups
                .entry(group.clone())
                .or_default()
                .push(id.clone());
        }

        // Insert filter
        self.filters.insert(id.clone(), filter);

        // Update order
        self.reorder();

        Ok(())
    }

    /// Remove a filter by ID
    pub fn remove_filter(&mut self, id: &str) -> Option<Filter> {
        let filter = self.filters.remove(id)?;

        // Remove from group
        if let Some(group) = &filter.group {
            if let Some(ids) = self.groups.get_mut(group) {
                ids.retain(|i| i != id);
                if ids.is_empty() {
                    self.groups.remove(group);
                }
            }
        }

        // Remove from order
        self.order.retain(|i| i != id);

        Some(filter)
    }

    /// Get a filter by ID
    pub fn get_filter(&self, id: &str) -> Option<&Filter> {
        self.filters.get(id)
    }

    /// Get a mutable filter by ID
    pub fn get_filter_mut(&mut self, id: &str) -> Option<&mut Filter> {
        self.filters.get_mut(id)
    }

    /// Get all filters
    pub fn filters(&self) -> impl Iterator<Item = &Filter> {
        self.order.iter().filter_map(|id| self.filters.get(id))
    }

    /// Get ordered filter IDs (for manual iteration)
    pub fn filter_ids(&self) -> &[String] {
        &self.order
    }

    /// Get filter count
    pub fn filter_count(&self) -> usize {
        self.filters.len()
    }

    /// Get filters in a specific group
    pub fn filters_in_group(&self, group: &str) -> impl Iterator<Item = &Filter> {
        self.groups
            .get(group)
            .map(|ids| ids.iter().filter_map(|id| self.filters.get(id)))
            .into_iter()
            .flatten()
    }

    /// Get all group names
    pub fn groups(&self) -> impl Iterator<Item = &String> {
        self.groups.keys()
    }

    /// Enable a filter
    pub fn enable_filter(&mut self, id: &str) -> Result<(), FilterError> {
        let filter = self
            .filters
            .get_mut(id)
            .ok_or(FilterError::FilterNotFound(id.to_string()))?;
        filter.enabled = true;
        Ok(())
    }

    /// Disable a filter
    pub fn disable_filter(&mut self, id: &str) -> Result<(), FilterError> {
        let filter = self
            .filters
            .get_mut(id)
            .ok_or(FilterError::FilterNotFound(id.to_string()))?;
        filter.enabled = false;
        Ok(())
    }

    /// Toggle a filter's enabled state
    pub fn toggle_filter(&mut self, id: &str) -> Result<bool, FilterError> {
        let filter = self
            .filters
            .get_mut(id)
            .ok_or(FilterError::FilterNotFound(id.to_string()))?;
        filter.enabled = !filter.enabled;
        Ok(filter.enabled)
    }

    /// Enable all filters in a group
    pub fn enable_group(&mut self, group: &str) -> Result<(), FilterError> {
        let ids = self
            .groups
            .get(group)
            .ok_or(FilterError::GroupNotFound(group.to_string()))?
            .clone();

        for id in ids {
            if let Some(filter) = self.filters.get_mut(&id) {
                filter.enabled = true;
            }
        }
        Ok(())
    }

    /// Disable all filters in a group
    pub fn disable_group(&mut self, group: &str) -> Result<(), FilterError> {
        let ids = self
            .groups
            .get(group)
            .ok_or(FilterError::GroupNotFound(group.to_string()))?
            .clone();

        for id in ids {
            if let Some(filter) = self.filters.get_mut(&id) {
                filter.enabled = false;
            }
        }
        Ok(())
    }

    /// Clear all filters
    pub fn clear(&mut self) {
        self.filters.clear();
        self.groups.clear();
        self.order.clear();
    }

    /// Get total match count across all filters
    pub fn total_matches(&self) -> u64 {
        self.filters.values().map(|f| f.stats.match_count).sum()
    }

    /// Reset all filter statistics
    pub fn reset_stats(&mut self) {
        for filter in self.filters.values_mut() {
            filter.reset_stats();
        }
    }

    /// Reorder filters by priority (highest first)
    fn reorder(&mut self) {
        let mut filters: Vec<_> = self.filters.values().collect();
        filters.sort_by(|a, b| b.priority.cmp(&a.priority));
        self.order = filters.iter().map(|f| f.id.clone()).collect();
    }

    /// Update filter priority and reorder
    pub fn set_priority(&mut self, id: &str, priority: i32) -> Result<(), FilterError> {
        let filter = self
            .filters
            .get_mut(id)
            .ok_or(FilterError::FilterNotFound(id.to_string()))?;
        filter.priority = priority;
        self.reorder();
        Ok(())
    }

    /// Export filters to JSON
    pub fn export_json(&self) -> Result<String, FilterError> {
        let filters: Vec<_> = self.filters.values().collect();
        serde_json::to_string_pretty(&filters).map_err(FilterError::SerializationError)
    }

    /// Import filters from JSON
    pub fn import_json(&mut self, json: &str) -> Result<usize, FilterError> {
        let filters: Vec<Filter> =
            serde_json::from_str(json).map_err(FilterError::SerializationError)?;

        let count = filters.len();
        for mut filter in filters {
            // Recompile patterns after deserialization
            filter.compile()?;
            self.add_filter(filter)?;
        }

        Ok(count)
    }
}

/// Filter processor for applying filters to terminal output
#[derive(Debug)]
pub struct FilterProcessor {
    /// Filter manager
    manager: FilterManager,
    /// Whether processor is enabled
    enabled: bool,
}

impl Default for FilterProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterProcessor {
    /// Create a new filter processor
    pub fn new() -> Self {
        Self {
            manager: FilterManager::new(),
            enabled: true,
        }
    }

    /// Create with existing manager
    pub fn with_manager(manager: FilterManager) -> Self {
        Self {
            manager,
            enabled: true,
        }
    }

    /// Get the filter manager
    pub fn manager(&self) -> &FilterManager {
        &self.manager
    }

    /// Get mutable filter manager
    pub fn manager_mut(&mut self) -> &mut FilterManager {
        &mut self.manager
    }

    /// Enable the processor
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable the processor
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Toggle processor enabled state
    pub fn toggle(&mut self) -> bool {
        self.enabled = !self.enabled;
        self.enabled
    }

    /// Check if processor is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Process a line of text through all filters
    pub fn process_line(&mut self, line: &str) -> ProcessedLine {
        if !self.enabled {
            return ProcessedLine {
                text: line.to_string(),
                hidden: false,
                highlights: Vec::new(),
                notifications: Vec::new(),
            };
        }

        let mut text = line.to_string();
        let mut hidden = false;
        let mut highlights = Vec::new();
        let mut notifications = Vec::new();

        // Apply filters in priority order
        // Clone filter IDs to avoid borrow issues
        let filter_ids: Vec<_> = self.manager.filter_ids().to_vec();

        for id in filter_ids {
            let filter = match self.manager.get_filter_mut(&id) {
                Some(f) => f,
                None => continue,
            };

            if !filter.enabled {
                continue;
            }

            match filter.apply(&text) {
                FilterResult::NoMatch => continue,
                FilterResult::Hide => {
                    hidden = true;
                    break; // No need to process further if hiding
                }
                FilterResult::Highlight {
                    color,
                    bg_color,
                    ranges,
                } => {
                    highlights.push(HighlightInfo {
                        color,
                        bg_color,
                        ranges,
                    });
                }
                FilterResult::Replace { text: new_text } => {
                    text = new_text;
                }
                FilterResult::Notify { title, body, sound } => {
                    notifications.push(NotificationInfo { title, body, sound });
                }
            }
        }

        ProcessedLine {
            text,
            hidden,
            highlights,
            notifications,
        }
    }

    /// Process multiple lines
    pub fn process_lines(&mut self, lines: &[String]) -> Vec<ProcessedLine> {
        lines.iter().map(|line| self.process_line(line)).collect()
    }

    /// Get statistics summary
    pub fn get_stats(&self) -> HashMap<String, FilterStats> {
        self.manager
            .filters()
            .map(|f| (f.name.clone(), f.stats.clone()))
            .collect()
    }
}

/// A processed line with filter results
#[derive(Debug, Clone)]
pub struct ProcessedLine {
    /// The final text (after replacements)
    pub text: String,
    /// Whether this line should be hidden
    pub hidden: bool,
    /// Highlight information
    pub highlights: Vec<HighlightInfo>,
    /// Notifications to trigger
    pub notifications: Vec<NotificationInfo>,
}

/// Highlight information for a processed line
#[derive(Debug, Clone)]
pub struct HighlightInfo {
    /// RGB color (0-255)
    pub color: (u8, u8, u8),
    /// Optional background color
    pub bg_color: Option<(u8, u8, u8)>,
    /// Character ranges to highlight
    pub ranges: Vec<(usize, usize)>,
}

/// Notification information
#[derive(Debug, Clone)]
pub struct NotificationInfo {
    /// Notification title
    pub title: String,
    /// Notification body
    pub body: Option<String>,
    /// Whether to play sound
    pub sound: bool,
}

/// Filter system errors
#[derive(Debug, Error)]
pub enum FilterError {
    #[error("Invalid regex pattern: {0}")]
    InvalidPattern(#[from] regex::Error),

    #[error("Filter not found: {0}")]
    FilterNotFound(String),

    #[error("Group not found: {0}")]
    GroupNotFound(String),

    #[error("Serialization error: {0}")]
    SerializationError(serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_creation() {
        let filter = Filter::new(
            "test1".to_string(),
            "Test Filter".to_string(),
            r"error".to_string(),
            FilterAction::Hide,
        );
        assert!(filter.is_ok());
        let filter = filter.unwrap();
        assert_eq!(filter.id, "test1");
        assert_eq!(filter.name, "Test Filter");
        assert!(filter.enabled);
    }

    #[test]
    fn test_filter_matching() {
        let mut filter = Filter::new(
            "test1".to_string(),
            "Error Filter".to_string(),
            r"ERROR".to_string(),
            FilterAction::Hide,
        )
        .unwrap();

        assert!(filter.matches("This is an ERROR message"));
        assert!(!filter.matches("This is a warning message"));

        filter.enabled = false;
        assert!(!filter.matches("This is an ERROR message"));
    }

    #[test]
    fn test_filter_case_insensitive() {
        let filter = Filter::new_case_insensitive(
            "test1".to_string(),
            "Error Filter".to_string(),
            r"error".to_string(),
            FilterAction::Hide,
        )
        .unwrap();

        assert!(filter.matches("This is an ERROR message"));
        assert!(filter.matches("This is an error message"));
        assert!(filter.matches("This is an Error message"));
    }

    #[test]
    fn test_filter_hide_action() {
        let mut filter = Filter::new(
            "test1".to_string(),
            "Hide Errors".to_string(),
            r"ERROR".to_string(),
            FilterAction::Hide,
        )
        .unwrap();

        let result = filter.apply("This is an ERROR message");
        assert!(matches!(result, FilterResult::Hide));
    }

    #[test]
    fn test_filter_highlight_action() {
        let mut filter = Filter::new(
            "test1".to_string(),
            "Highlight Warnings".to_string(),
            r"WARN".to_string(),
            FilterAction::Highlight {
                color: (255, 255, 0),
                bg_color: None,
            },
        )
        .unwrap();

        let result = filter.apply("This is a WARN message");
        match result {
            FilterResult::Highlight { color, ranges, .. } => {
                assert_eq!(color, (255, 255, 0));
                assert_eq!(ranges.len(), 1);
            }
            _ => panic!("Expected Highlight result"),
        }
    }

    #[test]
    fn test_filter_replace_action() {
        let mut filter = Filter::new(
            "test1".to_string(),
            "Replace Passwords".to_string(),
            r"password=\S+".to_string(),
            FilterAction::Replace {
                replacement: "password=***".to_string(),
            },
        )
        .unwrap();

        let result = filter.apply("User logged in with password=secret123");
        match result {
            FilterResult::Replace { text } => {
                assert_eq!(text, "User logged in with password=***");
            }
            _ => panic!("Expected Replace result"),
        }
    }

    #[test]
    fn test_filter_stats() {
        let mut filter = Filter::new(
            "test1".to_string(),
            "Error Counter".to_string(),
            r"ERROR".to_string(),
            FilterAction::Hide,
        )
        .unwrap();

        assert_eq!(filter.stats.match_count, 0);

        filter.apply("ERROR 1");
        assert_eq!(filter.stats.match_count, 1);

        filter.apply("ERROR 2");
        assert_eq!(filter.stats.match_count, 2);

        filter.apply("No match");
        assert_eq!(filter.stats.match_count, 2);
    }

    #[test]
    fn test_filter_manager_add_remove() {
        let mut manager = FilterManager::new();

        let filter = Filter::new(
            "test1".to_string(),
            "Test".to_string(),
            r"test".to_string(),
            FilterAction::Hide,
        )
        .unwrap();

        manager.add_filter(filter).unwrap();
        assert!(manager.get_filter("test1").is_some());

        let removed = manager.remove_filter("test1");
        assert!(removed.is_some());
        assert!(manager.get_filter("test1").is_none());
    }

    #[test]
    fn test_filter_manager_groups() {
        let mut manager = FilterManager::new();

        let mut filter1 = Filter::new(
            "test1".to_string(),
            "Test 1".to_string(),
            r"error".to_string(),
            FilterAction::Hide,
        )
        .unwrap();
        filter1.group = Some("errors".to_string());

        let mut filter2 = Filter::new(
            "test2".to_string(),
            "Test 2".to_string(),
            r"warn".to_string(),
            FilterAction::Hide,
        )
        .unwrap();
        filter2.group = Some("errors".to_string());

        manager.add_filter(filter1).unwrap();
        manager.add_filter(filter2).unwrap();

        let group_filters: Vec<_> = manager.filters_in_group("errors").collect();
        assert_eq!(group_filters.len(), 2);
    }

    #[test]
    fn test_filter_manager_enable_disable() {
        let mut manager = FilterManager::new();

        let filter = Filter::new(
            "test1".to_string(),
            "Test".to_string(),
            r"test".to_string(),
            FilterAction::Hide,
        )
        .unwrap();

        manager.add_filter(filter).unwrap();

        manager.disable_filter("test1").unwrap();
        assert!(!manager.get_filter("test1").unwrap().enabled);

        manager.enable_filter("test1").unwrap();
        assert!(manager.get_filter("test1").unwrap().enabled);

        let toggled = manager.toggle_filter("test1").unwrap();
        assert!(!toggled);
    }

    #[test]
    fn test_filter_manager_priority() {
        let mut manager = FilterManager::new();

        let mut filter1 = Filter::new(
            "low".to_string(),
            "Low Priority".to_string(),
            r"test".to_string(),
            FilterAction::Hide,
        )
        .unwrap();
        filter1.priority = 1;

        let mut filter2 = Filter::new(
            "high".to_string(),
            "High Priority".to_string(),
            r"test".to_string(),
            FilterAction::Hide,
        )
        .unwrap();
        filter2.priority = 10;

        manager.add_filter(filter1).unwrap();
        manager.add_filter(filter2).unwrap();

        let filters: Vec<_> = manager.filters().collect();
        assert_eq!(filters[0].id, "high");
        assert_eq!(filters[1].id, "low");
    }

    #[test]
    fn test_filter_processor_basic() {
        let mut processor = FilterProcessor::new();

        let filter = Filter::new(
            "test1".to_string(),
            "Hide Errors".to_string(),
            r"ERROR".to_string(),
            FilterAction::Hide,
        )
        .unwrap();

        processor.manager_mut().add_filter(filter).unwrap();

        let result = processor.process_line("This is an ERROR message");
        assert!(result.hidden);

        let result = processor.process_line("This is a normal message");
        assert!(!result.hidden);
    }

    #[test]
    fn test_filter_processor_multiple_filters() {
        let mut processor = FilterProcessor::new();

        let filter1 = Filter::new(
            "hide".to_string(),
            "Hide Debug".to_string(),
            r"DEBUG".to_string(),
            FilterAction::Hide,
        )
        .unwrap();

        let filter2 = Filter::new(
            "replace".to_string(),
            "Mask Passwords".to_string(),
            r"pwd=\S+".to_string(),
            FilterAction::Replace {
                replacement: "pwd=***".to_string(),
            },
        )
        .unwrap();

        processor.manager_mut().add_filter(filter1).unwrap();
        processor.manager_mut().add_filter(filter2).unwrap();

        let result = processor.process_line("Login with pwd=secret123");
        assert!(!result.hidden);
        assert!(result.text.contains("pwd=***"));

        let result = processor.process_line("DEBUG: Something");
        assert!(result.hidden);
    }

    #[test]
    fn test_filter_processor_disabled() {
        let mut processor = FilterProcessor::new();

        let filter = Filter::new(
            "test1".to_string(),
            "Hide Errors".to_string(),
            r"ERROR".to_string(),
            FilterAction::Hide,
        )
        .unwrap();

        processor.manager_mut().add_filter(filter).unwrap();
        processor.disable();

        let result = processor.process_line("This is an ERROR message");
        assert!(!result.hidden);
    }

    #[test]
    fn test_filter_processor_stats() {
        let mut processor = FilterProcessor::new();

        let filter = Filter::new(
            "test1".to_string(),
            "Count Errors".to_string(),
            r"ERROR".to_string(),
            FilterAction::Highlight {
                color: (255, 0, 0),
                bg_color: None,
            },
        )
        .unwrap();

        processor.manager_mut().add_filter(filter).unwrap();

        processor.process_line("ERROR 1");
        processor.process_line("ERROR 2");
        processor.process_line("Normal message");
        processor.process_line("ERROR 3");

        assert_eq!(processor.manager().total_matches(), 3);
    }

    #[test]
    fn test_filter_serialization() {
        let mut manager = FilterManager::new();

        let filter = Filter::new(
            "test1".to_string(),
            "Test Filter".to_string(),
            r"error".to_string(),
            FilterAction::Hide,
        )
        .unwrap();

        manager.add_filter(filter).unwrap();

        let json = manager.export_json().unwrap();
        assert!(json.contains("test1"));
        assert!(json.contains("Test Filter"));

        let mut new_manager = FilterManager::new();
        let count = new_manager.import_json(&json).unwrap();
        assert_eq!(count, 1);
        assert!(new_manager.get_filter("test1").is_some());
    }

    #[test]
    fn test_filter_multiple_highlights() {
        let mut filter = Filter::new(
            "test1".to_string(),
            "Highlight Numbers".to_string(),
            r"\d+".to_string(),
            FilterAction::Highlight {
                color: (0, 255, 0),
                bg_color: Some((0, 0, 0)),
            },
        )
        .unwrap();

        let result = filter.apply("Found 3 errors and 5 warnings");
        match result {
            FilterResult::Highlight { ranges, .. } => {
                assert_eq!(ranges.len(), 2);
            }
            _ => panic!("Expected Highlight result"),
        }
    }

    #[test]
    fn test_filter_regex_groups() {
        let mut filter = Filter::new(
            "test1".to_string(),
            "Extract Info".to_string(),
            r"User: (\w+)".to_string(),
            FilterAction::Replace {
                replacement: "Username=$1".to_string(),
            },
        )
        .unwrap();

        let result = filter.apply("User: john_doe logged in");
        match result {
            FilterResult::Replace { text } => {
                assert_eq!(text, "Username=john_doe logged in");
            }
            _ => panic!("Expected Replace result"),
        }
    }
}
