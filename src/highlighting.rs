//! Terminal Output Highlighting System
//!
//! Provides pattern-based output highlighting with customizable styles.
//! Supports regex patterns, priority-based matching, and rule categories.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

// ============================================================================
// Color Definition
// ============================================================================

/// RGB color tuple (0-255 range)
pub type Color = (u8, u8, u8);

// ============================================================================
// Highlight Style
// ============================================================================

/// Style to apply to highlighted text
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(Default)]
pub struct HighlightStyle {
    /// Foreground color (text color)
    pub foreground: Option<Color>,
    /// Background color
    pub background: Option<Color>,
    /// Bold text
    pub bold: bool,
    /// Italic text
    pub italic: bool,
    /// Underline text
    pub underline: bool,
}


impl HighlightStyle {
    /// Create a new style with foreground color only
    pub fn with_foreground(color: Color) -> Self {
        Self {
            foreground: Some(color),
            ..Default::default()
        }
    }

    /// Create a new style with both foreground and background colors
    pub fn with_colors(foreground: Color, background: Color) -> Self {
        Self {
            foreground: Some(foreground),
            background: Some(background),
            ..Default::default()
        }
    }

    /// Add bold style
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Add italic style
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Add underline style
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }
}

// ============================================================================
// Highlight Rule
// ============================================================================

/// A single highlighting rule with pattern and style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightRule {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Regex pattern to match
    #[serde(skip)]
    pub pattern: Option<Arc<Regex>>,
    /// Pattern string (for serialization)
    pub pattern_str: String,
    /// Style to apply on match
    pub style: HighlightStyle,
    /// Priority (higher = processed first, can override lower priority matches)
    pub priority: i32,
    /// Whether this rule is active
    pub enabled: bool,
    /// Optional category for grouping rules
    pub category: Option<String>,
}

impl HighlightRule {
    /// Create a new highlight rule
    pub fn new(
        id: String,
        name: String,
        pattern: String,
        style: HighlightStyle,
    ) -> Result<Self, HighlightError> {
        let regex = Regex::new(&pattern)?;
        Ok(Self {
            id,
            name,
            pattern: Some(Arc::new(regex)),
            pattern_str: pattern,
            style,
            priority: 0,
            enabled: true,
            category: None,
        })
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Set category
    pub fn with_category(mut self, category: String) -> Self {
        self.category = Some(category);
        self
    }

    /// Check if this rule matches the given text
    pub fn matches(&self, text: &str) -> bool {
        if !self.enabled {
            return false;
        }
        self.pattern
            .as_ref()
            .map(|p| p.is_match(text))
            .unwrap_or(false)
    }

    /// Find all matches in text and return their ranges
    pub fn find_matches(&self, text: &str) -> Vec<(usize, usize)> {
        if !self.enabled {
            return Vec::new();
        }
        self.pattern
            .as_ref()
            .map(|p| p.find_iter(text).map(|m| (m.start(), m.end())).collect())
            .unwrap_or_default()
    }

    /// Compile/recompile the regex pattern
    pub fn compile(&mut self) -> Result<(), HighlightError> {
        let regex = Regex::new(&self.pattern_str)?;
        self.pattern = Some(Arc::new(regex));
        Ok(())
    }
}

// ============================================================================
// Highlight Match
// ============================================================================

/// Represents a matched highlight with position and style
#[derive(Debug, Clone, PartialEq)]
pub struct HighlightMatch {
    /// ID of the rule that produced this match
    pub rule_id: String,
    /// Start position in the text (byte offset)
    pub start: usize,
    /// End position in the text (byte offset)
    pub end: usize,
    /// Style to apply
    pub style: HighlightStyle,
}

impl HighlightMatch {
    /// Create a new highlight match
    pub fn new(rule_id: String, start: usize, end: usize, style: HighlightStyle) -> Self {
        Self {
            rule_id,
            start,
            end,
            style,
        }
    }
}

// ============================================================================
// Highlight Engine
// ============================================================================

/// Main engine for managing and applying highlight rules
#[derive(Debug, Clone)]
pub struct HighlightEngine {
    /// All registered rules (indexed by ID)
    rules: HashMap<String, HighlightRule>,
    /// Ordered rule IDs (by priority, descending)
    order: Vec<String>,
    /// Category index (category name -> rule IDs)
    categories: HashMap<String, Vec<String>>,
}

impl Default for HighlightEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl HighlightEngine {
    /// Create a new empty highlight engine
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            order: Vec::new(),
            categories: HashMap::new(),
        }
    }

    /// Create a new engine with default rules
    pub fn with_defaults() -> Self {
        let mut engine = Self::new();

        // Error patterns (red, bold)
        let _ = engine.add_rule(
            HighlightRule::new(
                "error".to_string(),
                "Error Messages".to_string(),
                r"(?i)\b(error|fatal|fail(ed|ure)?|exception|panic|crash)\b".to_string(),
                HighlightStyle::with_foreground((255, 85, 85)).bold(),
            )
            .unwrap()
            .with_priority(100)
            .with_category("diagnostics".to_string()),
        );

        // Warning patterns (yellow, bold)
        let _ = engine.add_rule(
            HighlightRule::new(
                "warning".to_string(),
                "Warning Messages".to_string(),
                r"(?i)\b(warn(ing)?|caution|alert|deprecated)\b".to_string(),
                HighlightStyle::with_foreground((255, 215, 0)).bold(),
            )
            .unwrap()
            .with_priority(90)
            .with_category("diagnostics".to_string()),
        );

        // Success patterns (green, bold)
        let _ = engine.add_rule(
            HighlightRule::new(
                "success".to_string(),
                "Success Messages".to_string(),
                r"(?i)\b(success|ok|pass(ed)?|complete(d)?|done)\b".to_string(),
                HighlightStyle::with_foreground((85, 255, 85)).bold(),
            )
            .unwrap()
            .with_priority(80)
            .with_category("diagnostics".to_string()),
        );

        // URL patterns (blue, underline)
        let _ = engine.add_rule(
            HighlightRule::new(
                "url".to_string(),
                "URLs".to_string(),
                r#"https?://[^\s<>"]+|www\.[^\s<>"]+"#.to_string(),
                HighlightStyle::with_foreground((100, 149, 237)).underline(),
            )
            .unwrap()
            .with_priority(70)
            .with_category("links".to_string()),
        );

        // IP address patterns (cyan)
        let _ = engine.add_rule(
            HighlightRule::new(
                "ip".to_string(),
                "IP Addresses".to_string(),
                r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b".to_string(),
                HighlightStyle::with_foreground((0, 255, 255)),
            )
            .unwrap()
            .with_priority(60)
            .with_category("network".to_string()),
        );

        // File path patterns (magenta)
        let _ = engine.add_rule(
            HighlightRule::new(
                "path".to_string(),
                "File Paths".to_string(),
                r"(?:/[\w.-]+)+/?|(?:[A-Za-z]:\\(?:[\w.-]+\\)*[\w.-]+)".to_string(),
                HighlightStyle::with_foreground((255, 105, 180)),
            )
            .unwrap()
            .with_priority(50)
            .with_category("filesystem".to_string()),
        );

        // Number patterns (light blue)
        let _ = engine.add_rule(
            HighlightRule::new(
                "number".to_string(),
                "Numbers".to_string(),
                r"\b\d+(?:\.\d+)?(?:[eE][+-]?\d+)?\b".to_string(),
                HighlightStyle::with_foreground((173, 216, 230)),
            )
            .unwrap()
            .with_priority(40)
            .with_category("syntax".to_string()),
        );

        engine
    }

    /// Add a new highlight rule
    pub fn add_rule(&mut self, rule: HighlightRule) -> Result<(), HighlightError> {
        let id = rule.id.clone();

        // Add to category index if specified
        if let Some(category) = &rule.category {
            self.categories
                .entry(category.clone())
                .or_default()
                .push(id.clone());
        }

        // Insert rule
        self.rules.insert(id.clone(), rule);

        // Update order
        self.reorder();

        Ok(())
    }

    /// Remove a rule by ID
    pub fn remove_rule(&mut self, id: &str) -> Result<(), HighlightError> {
        let rule = self
            .rules
            .remove(id)
            .ok_or_else(|| HighlightError::RuleNotFound(id.to_string()))?;

        // Remove from category index
        if let Some(category) = &rule.category {
            if let Some(ids) = self.categories.get_mut(category) {
                ids.retain(|i| i != id);
                if ids.is_empty() {
                    self.categories.remove(category);
                }
            }
        }

        // Remove from order
        self.order.retain(|i| i != id);

        Ok(())
    }

    /// Toggle a rule's enabled state
    pub fn toggle_rule(&mut self, id: &str) -> Result<bool, HighlightError> {
        let rule = self
            .rules
            .get_mut(id)
            .ok_or_else(|| HighlightError::RuleNotFound(id.to_string()))?;
        rule.enabled = !rule.enabled;
        Ok(rule.enabled)
    }

    /// Update a rule with new values
    pub fn update_rule(
        &mut self,
        id: &str,
        updates: RuleUpdate,
    ) -> Result<(), HighlightError> {
        let rule = self
            .rules
            .get_mut(id)
            .ok_or_else(|| HighlightError::RuleNotFound(id.to_string()))?;

        let mut needs_reorder = false;

        if let Some(name) = updates.name {
            rule.name = name;
        }

        if let Some(pattern) = updates.pattern {
            rule.pattern_str = pattern;
            rule.compile()?;
        }

        if let Some(style) = updates.style {
            rule.style = style;
        }

        if let Some(priority) = updates.priority {
            rule.priority = priority;
            needs_reorder = true;
        }

        if let Some(enabled) = updates.enabled {
            rule.enabled = enabled;
        }

        if let Some(category) = updates.category {
            // Remove from old category
            if let Some(old_category) = &rule.category {
                if let Some(ids) = self.categories.get_mut(old_category) {
                    ids.retain(|i| i != id);
                    if ids.is_empty() {
                        self.categories.remove(old_category);
                    }
                }
            }

            // Add to new category
            if let Some(new_category) = &category {
                self.categories
                    .entry(new_category.clone())
                    .or_default()
                    .push(id.to_string());
            }

            rule.category = category;
        }

        if needs_reorder {
            self.reorder();
        }

        Ok(())
    }

    /// Get a rule by ID
    pub fn get_rule(&self, id: &str) -> Option<&HighlightRule> {
        self.rules.get(id)
    }

    /// List all rules (ordered by priority)
    pub fn list_rules(&self) -> Vec<&HighlightRule> {
        self.order
            .iter()
            .filter_map(|id| self.rules.get(id))
            .collect()
    }

    /// Get rules by category
    pub fn get_by_category(&self, category: &str) -> Vec<&HighlightRule> {
        self.categories
            .get(category)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.rules.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Process a line of text and return all highlight matches
    pub fn process_line(&self, line: &str) -> Vec<HighlightMatch> {
        let mut matches = Vec::new();

        // Process rules in priority order
        for rule_id in &self.order {
            if let Some(rule) = self.rules.get(rule_id) {
                if !rule.enabled {
                    continue;
                }

                // Find all matches for this rule
                for (start, end) in rule.find_matches(line) {
                    matches.push(HighlightMatch::new(
                        rule.id.clone(),
                        start,
                        end,
                        rule.style.clone(),
                    ));
                }
            }
        }

        // Sort matches by start position, then by priority (higher first)
        matches.sort_by(|a, b| {
            let pos_cmp = a.start.cmp(&b.start);
            if pos_cmp == std::cmp::Ordering::Equal {
                // If same position, prioritize based on rule priority
                let rule_a = self.rules.get(&a.rule_id);
                let rule_b = self.rules.get(&b.rule_id);
                match (rule_a, rule_b) {
                    (Some(ra), Some(rb)) => rb.priority.cmp(&ra.priority),
                    _ => std::cmp::Ordering::Equal,
                }
            } else {
                pos_cmp
            }
        });

        // Remove overlapping matches (keep higher priority)
        self.remove_overlaps(matches)
    }

    /// Reorder rules by priority (descending)
    fn reorder(&mut self) {
        let mut rules: Vec<_> = self.rules.values().collect();
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        self.order = rules.iter().map(|r| r.id.clone()).collect();
    }

    /// Remove overlapping matches, keeping higher priority ones
    fn remove_overlaps(&self, matches: Vec<HighlightMatch>) -> Vec<HighlightMatch> {
        let mut result = Vec::new();
        let mut last_end = 0;

        for m in matches {
            // Skip if this match overlaps with previous one
            if m.start >= last_end {
                last_end = m.end;
                result.push(m);
            }
        }

        result
    }

    /// Save rules to a JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), HighlightError> {
        let rules: Vec<_> = self.list_rules().iter().map(|r| (*r).clone()).collect();
        let json = serde_json::to_string_pretty(&rules)?;
        fs::write(path, json).map_err(|e| HighlightError::IoError(e.to_string()))?;
        Ok(())
    }

    /// Load rules from a JSON file
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), HighlightError> {
        let json = fs::read_to_string(path)
            .map_err(|e| HighlightError::IoError(e.to_string()))?;
        let mut rules: Vec<HighlightRule> = serde_json::from_str(&json)?;

        // Compile patterns
        for rule in &mut rules {
            rule.compile()?;
        }

        // Clear existing rules
        self.rules.clear();
        self.order.clear();
        self.categories.clear();

        // Add loaded rules
        for rule in rules {
            self.add_rule(rule)?;
        }

        Ok(())
    }

    /// Get all category names
    pub fn categories(&self) -> Vec<&str> {
        self.categories.keys().map(|s| s.as_str()).collect()
    }

    /// Get rule count
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

// ============================================================================
// Rule Update Structure
// ============================================================================

/// Structure for updating rule properties
#[derive(Debug, Default)]
pub struct RuleUpdate {
    pub name: Option<String>,
    pub pattern: Option<String>,
    pub style: Option<HighlightStyle>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
    pub category: Option<Option<String>>,
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, Error)]
pub enum HighlightError {
    #[error("Invalid regex pattern: {0}")]
    InvalidPattern(#[from] regex::Error),

    #[error("Rule not found: {0}")]
    RuleNotFound(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(String),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_style_creation() {
        let style = HighlightStyle::with_foreground((255, 0, 0));
        assert_eq!(style.foreground, Some((255, 0, 0)));
        assert_eq!(style.background, None);
        assert!(!style.bold);

        let style = HighlightStyle::with_foreground((255, 0, 0)).bold().underline();
        assert!(style.bold);
        assert!(style.underline);
        assert!(!style.italic);
    }

    #[test]
    fn test_highlight_rule_creation() {
        let rule = HighlightRule::new(
            "test1".to_string(),
            "Test Rule".to_string(),
            r"\berror\b".to_string(),
            HighlightStyle::with_foreground((255, 0, 0)),
        );
        assert!(rule.is_ok());

        let rule = rule.unwrap();
        assert_eq!(rule.id, "test1");
        assert_eq!(rule.name, "Test Rule");
        assert_eq!(rule.priority, 0);
        assert!(rule.enabled);
        assert!(rule.category.is_none());
    }

    #[test]
    fn test_rule_matching() {
        let rule = HighlightRule::new(
            "error".to_string(),
            "Error".to_string(),
            r"(?i)\berror\b".to_string(),
            HighlightStyle::with_foreground((255, 0, 0)),
        )
        .unwrap();

        assert!(rule.matches("This is an error message"));
        assert!(rule.matches("ERROR: something went wrong"));
        assert!(!rule.matches("This is fine"));
    }

    #[test]
    fn test_rule_find_matches() {
        let rule = HighlightRule::new(
            "num".to_string(),
            "Numbers".to_string(),
            r"\d+".to_string(),
            HighlightStyle::with_foreground((0, 0, 255)),
        )
        .unwrap();

        let matches = rule.find_matches("Port 8080 and 443 are open");
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0], (5, 9)); // "8080"
        assert_eq!(matches[1], (14, 17)); // "443"
    }

    #[test]
    fn test_engine_add_remove_rule() {
        let mut engine = HighlightEngine::new();
        let rule = HighlightRule::new(
            "test".to_string(),
            "Test".to_string(),
            r"test".to_string(),
            HighlightStyle::with_foreground((255, 0, 0)),
        )
        .unwrap();

        assert!(engine.add_rule(rule).is_ok());
        assert_eq!(engine.rule_count(), 1);
        assert!(engine.get_rule("test").is_some());

        assert!(engine.remove_rule("test").is_ok());
        assert_eq!(engine.rule_count(), 0);
        assert!(engine.get_rule("test").is_none());
    }

    #[test]
    fn test_engine_toggle_rule() {
        let mut engine = HighlightEngine::new();
        let rule = HighlightRule::new(
            "test".to_string(),
            "Test".to_string(),
            r"test".to_string(),
            HighlightStyle::with_foreground((255, 0, 0)),
        )
        .unwrap();

        engine.add_rule(rule).unwrap();
        assert!(engine.get_rule("test").unwrap().enabled);

        let enabled = engine.toggle_rule("test").unwrap();
        assert!(!enabled);
        assert!(!engine.get_rule("test").unwrap().enabled);

        let enabled = engine.toggle_rule("test").unwrap();
        assert!(enabled);
        assert!(engine.get_rule("test").unwrap().enabled);
    }

    #[test]
    fn test_engine_update_rule() {
        let mut engine = HighlightEngine::new();
        let rule = HighlightRule::new(
            "test".to_string(),
            "Test".to_string(),
            r"test".to_string(),
            HighlightStyle::with_foreground((255, 0, 0)),
        )
        .unwrap();

        engine.add_rule(rule).unwrap();

        let update = RuleUpdate {
            name: Some("Updated Test".to_string()),
            priority: Some(100),
            ..Default::default()
        };

        engine.update_rule("test", update).unwrap();
        let rule = engine.get_rule("test").unwrap();
        assert_eq!(rule.name, "Updated Test");
        assert_eq!(rule.priority, 100);
    }

    #[test]
    fn test_engine_process_line() {
        let mut engine = HighlightEngine::new();

        engine
            .add_rule(
                HighlightRule::new(
                    "error".to_string(),
                    "Error".to_string(),
                    r"(?i)\berror\b".to_string(),
                    HighlightStyle::with_foreground((255, 0, 0)),
                )
                .unwrap()
                .with_priority(100),
            )
            .unwrap();

        engine
            .add_rule(
                HighlightRule::new(
                    "num".to_string(),
                    "Number".to_string(),
                    r"\d+".to_string(),
                    HighlightStyle::with_foreground((0, 0, 255)),
                )
                .unwrap()
                .with_priority(50),
            )
            .unwrap();

        let matches = engine.process_line("Error at line 42");
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].rule_id, "error");
        assert_eq!(matches[1].rule_id, "num");
    }

    #[test]
    fn test_engine_priority_ordering() {
        let mut engine = HighlightEngine::new();

        engine
            .add_rule(
                HighlightRule::new(
                    "low".to_string(),
                    "Low".to_string(),
                    r"test".to_string(),
                    HighlightStyle::with_foreground((0, 0, 255)),
                )
                .unwrap()
                .with_priority(10),
            )
            .unwrap();

        engine
            .add_rule(
                HighlightRule::new(
                    "high".to_string(),
                    "High".to_string(),
                    r"test".to_string(),
                    HighlightStyle::with_foreground((255, 0, 0)),
                )
                .unwrap()
                .with_priority(100),
            )
            .unwrap();

        let rules = engine.list_rules();
        assert_eq!(rules[0].id, "high");
        assert_eq!(rules[1].id, "low");
    }

    #[test]
    fn test_engine_categories() {
        let mut engine = HighlightEngine::new();

        engine
            .add_rule(
                HighlightRule::new(
                    "error".to_string(),
                    "Error".to_string(),
                    r"error".to_string(),
                    HighlightStyle::with_foreground((255, 0, 0)),
                )
                .unwrap()
                .with_category("diagnostics".to_string()),
            )
            .unwrap();

        engine
            .add_rule(
                HighlightRule::new(
                    "warning".to_string(),
                    "Warning".to_string(),
                    r"warning".to_string(),
                    HighlightStyle::with_foreground((255, 255, 0)),
                )
                .unwrap()
                .with_category("diagnostics".to_string()),
            )
            .unwrap();

        let rules = engine.get_by_category("diagnostics");
        assert_eq!(rules.len(), 2);

        let categories = engine.categories();
        assert_eq!(categories.len(), 1);
        assert!(categories.contains(&"diagnostics"));
    }

    #[test]
    fn test_with_defaults() {
        let engine = HighlightEngine::with_defaults();
        assert!(engine.rule_count() > 0);

        let matches = engine.process_line("Error: connection failed");
        assert!(!matches.is_empty());

        let matches = engine.process_line("Visit https://example.com");
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_overlap_removal() {
        let mut engine = HighlightEngine::new();

        // Create two rules that might overlap
        engine
            .add_rule(
                HighlightRule::new(
                    "word".to_string(),
                    "Word".to_string(),
                    r"\w+".to_string(),
                    HighlightStyle::with_foreground((255, 0, 0)),
                )
                .unwrap()
                .with_priority(50),
            )
            .unwrap();

        engine
            .add_rule(
                HighlightRule::new(
                    "error".to_string(),
                    "Error".to_string(),
                    r"error".to_string(),
                    HighlightStyle::with_foreground((255, 255, 0)),
                )
                .unwrap()
                .with_priority(100),
            )
            .unwrap();

        let matches = engine.process_line("error");
        // Should only have one match (the higher priority one)
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].rule_id, "error");
    }

    #[test]
    fn test_serialization() {
        use tempfile::NamedTempFile;

        let mut engine = HighlightEngine::new();
        engine
            .add_rule(
                HighlightRule::new(
                    "test".to_string(),
                    "Test".to_string(),
                    r"test".to_string(),
                    HighlightStyle::with_foreground((255, 0, 0)).bold(),
                )
                .unwrap()
                .with_priority(100)
                .with_category("testing".to_string()),
            )
            .unwrap();

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Save
        assert!(engine.save_to_file(path).is_ok());

        // Load
        let mut new_engine = HighlightEngine::new();
        assert!(new_engine.load_from_file(path).is_ok());

        assert_eq!(new_engine.rule_count(), 1);
        let rule = new_engine.get_rule("test").unwrap();
        assert_eq!(rule.name, "Test");
        assert_eq!(rule.priority, 100);
        assert_eq!(rule.category, Some("testing".to_string()));
        assert!(rule.style.bold);
    }

    #[test]
    fn test_disabled_rule_no_match() {
        let mut engine = HighlightEngine::new();
        let mut rule = HighlightRule::new(
            "test".to_string(),
            "Test".to_string(),
            r"test".to_string(),
            HighlightStyle::with_foreground((255, 0, 0)),
        )
        .unwrap();

        rule.enabled = false;
        engine.add_rule(rule).unwrap();

        let matches = engine.process_line("test message");
        assert!(matches.is_empty());
    }
}
