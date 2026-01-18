//! Quick Actions system for AgTerm
//!
//! This module provides a command palette / quick action system that allows users to:
//! - Search and execute actions with fuzzy matching
//! - Register custom actions
//! - Track recently used actions
//! - Organize actions by category
//! - Execute both built-in and custom commands

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Maximum number of actions to keep in history
const MAX_HISTORY_SIZE: usize = 50;

/// Represents a quick action that can be executed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuickAction {
    /// Unique identifier for this action
    pub id: String,
    /// Display name shown in the UI
    pub name: String,
    /// Description of what this action does
    pub description: String,
    /// Category this action belongs to
    pub category: ActionCategory,
    /// Optional keyboard shortcut hint (e.g., "Cmd+T")
    pub shortcut: Option<String>,
    /// The command to execute when this action is triggered
    pub command: ActionCommand,
    /// Whether this action is currently enabled
    pub enabled: bool,
    /// Optional icon identifier for UI display
    pub icon: Option<String>,
}

impl QuickAction {
    /// Create a new QuickAction with minimal required fields
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        category: ActionCategory,
        command: ActionCommand,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            category,
            shortcut: None,
            command,
            enabled: true,
            icon: None,
        }
    }

    /// Builder pattern: set keyboard shortcut
    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Builder pattern: set icon
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Builder pattern: set enabled state
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Categories for organizing quick actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionCategory {
    /// Terminal operations (clear, reset, etc.)
    Terminal,
    /// Tab management (new, close, switch)
    Tab,
    /// Navigation actions (scroll, search)
    Navigation,
    /// Clipboard operations (copy, paste)
    Clipboard,
    /// Session management (save, restore)
    Session,
    /// Settings and preferences
    Settings,
    /// Help and documentation
    Help,
    /// User-defined custom actions
    Custom,
}

impl ActionCategory {
    /// Get a human-readable name for the category
    pub fn name(&self) -> &str {
        match self {
            ActionCategory::Terminal => "Terminal",
            ActionCategory::Tab => "Tab",
            ActionCategory::Navigation => "Navigation",
            ActionCategory::Clipboard => "Clipboard",
            ActionCategory::Session => "Session",
            ActionCategory::Settings => "Settings",
            ActionCategory::Help => "Help",
            ActionCategory::Custom => "Custom",
        }
    }
}

/// Commands that can be executed by quick actions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionCommand {
    /// Create a new tab
    NewTab,
    /// Close the current tab
    CloseTab,
    /// Split terminal horizontally
    SplitHorizontal,
    /// Split terminal vertically
    SplitVertical,
    /// Copy selected text to clipboard
    CopySelection,
    /// Paste clipboard content
    PasteClipboard,
    /// Clear the terminal screen
    ClearScreen,
    /// Open terminal search
    SearchTerminal,
    /// Toggle fullscreen mode
    ToggleFullscreen,
    /// Open settings dialog
    OpenSettings,
    /// Execute a shell command
    Shell(String),
    /// Execute a custom action by ID
    Custom(String),
}

/// Fuzzy matcher for searching actions
#[derive(Debug, Default)]
pub struct FuzzyMatcher {
    /// Case-sensitive matching
    case_sensitive: bool,
}

impl FuzzyMatcher {
    /// Create a new fuzzy matcher with default settings
    pub fn new() -> Self {
        Self {
            case_sensitive: false,
        }
    }

    /// Create a case-sensitive fuzzy matcher
    pub fn case_sensitive() -> Self {
        Self {
            case_sensitive: true,
        }
    }

    /// Calculate match score between query and target
    /// Returns a score from 0.0 (no match) to 1.0 (perfect match)
    pub fn score(&self, query: &str, target: &str) -> f64 {
        if query.is_empty() {
            return 1.0; // Empty query matches everything
        }

        let query = if self.case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };

        let target = if self.case_sensitive {
            target.to_string()
        } else {
            target.to_lowercase()
        };

        // Exact match gets highest score
        if query == target {
            return 1.0;
        }

        // Check if target starts with query
        if target.starts_with(&query) {
            return 0.9;
        }

        // Check for substring match
        if target.contains(&query) {
            return 0.7;
        }

        // Check for word boundary matches
        let word_boundary_score = self.word_boundary_score(&query, &target);
        if word_boundary_score > 0.0 {
            return word_boundary_score;
        }

        // Check for acronym match (e.g., "nwt" matches "New Window Tab")
        let acronym_score = self.acronym_score(&query, &target);
        if acronym_score > 0.0 {
            return acronym_score;
        }

        // Check for consecutive character match
        let consecutive_score = self.consecutive_score(&query, &target);
        if consecutive_score > 0.0 {
            return consecutive_score;
        }

        0.0
    }

    /// Calculate score for word boundary matching
    fn word_boundary_score(&self, query: &str, target: &str) -> f64 {
        let words: Vec<&str> = target.split(|c: char| c.is_whitespace() || c == '_' || c == '-')
            .filter(|w| !w.is_empty())
            .collect();

        for word in &words {
            if word.starts_with(query) {
                return 0.65;
            }
        }

        // Check if query words match target words
        let query_words: Vec<&str> = query.split_whitespace().collect();
        if query_words.len() > 1 {
            let mut matched = 0;
            for qword in &query_words {
                for tword in &words {
                    if tword.contains(qword) {
                        matched += 1;
                        break;
                    }
                }
            }
            if matched == query_words.len() {
                return 0.6;
            }
        }

        0.0
    }

    /// Calculate score for acronym matching
    fn acronym_score(&self, query: &str, target: &str) -> f64 {
        let words: Vec<&str> = target.split(|c: char| c.is_whitespace() || c == '_' || c == '-')
            .filter(|w| !w.is_empty())
            .collect();

        if words.is_empty() {
            return 0.0;
        }

        let acronym: String = words.iter()
            .filter_map(|w| w.chars().next())
            .collect();

        if acronym.starts_with(query) {
            return 0.55;
        }

        0.0
    }

    /// Calculate score for consecutive character matching
    fn consecutive_score(&self, query: &str, target: &str) -> f64 {
        let query_chars: Vec<char> = query.chars().collect();
        let target_chars: Vec<char> = target.chars().collect();

        if query_chars.is_empty() {
            return 0.0;
        }

        let mut query_idx = 0;
        let mut consecutive = 0;
        let mut max_consecutive = 0;

        for &target_char in &target_chars {
            if query_idx < query_chars.len() && target_char == query_chars[query_idx] {
                query_idx += 1;
                consecutive += 1;
                max_consecutive = max_consecutive.max(consecutive);
            } else {
                consecutive = 0;
            }
        }

        if query_idx == query_chars.len() {
            // All characters matched
            let ratio = max_consecutive as f64 / query_chars.len() as f64;
            return 0.3 + (ratio * 0.2);
        }

        0.0
    }
}

/// Manager for quick actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickActionManager {
    /// All registered actions
    actions: Vec<QuickAction>,
    /// History of recently used action IDs
    history: VecDeque<String>,
}

impl QuickActionManager {
    /// Create a new QuickActionManager with default actions
    pub fn new() -> Self {
        let mut manager = Self {
            actions: Vec::new(),
            history: VecDeque::with_capacity(MAX_HISTORY_SIZE),
        };

        // Register default actions
        manager.register_default_actions();
        manager
    }

    /// Create an empty QuickActionManager without default actions
    pub fn empty() -> Self {
        Self {
            actions: Vec::new(),
            history: VecDeque::with_capacity(MAX_HISTORY_SIZE),
        }
    }

    /// Register default built-in actions
    fn register_default_actions(&mut self) {
        // Tab management
        self.register(
            QuickAction::new(
                "new_tab",
                "New Tab",
                "Create a new terminal tab",
                ActionCategory::Tab,
                ActionCommand::NewTab,
            )
            .with_shortcut("Cmd+T")
            .with_icon("tab_add"),
        );

        self.register(
            QuickAction::new(
                "close_tab",
                "Close Tab",
                "Close the current tab",
                ActionCategory::Tab,
                ActionCommand::CloseTab,
            )
            .with_shortcut("Cmd+W")
            .with_icon("tab_close"),
        );

        // Splitting
        self.register(
            QuickAction::new(
                "split_horizontal",
                "Split Horizontal",
                "Split the terminal horizontally",
                ActionCategory::Terminal,
                ActionCommand::SplitHorizontal,
            )
            .with_shortcut("Cmd+D")
            .with_icon("split_horizontal"),
        );

        self.register(
            QuickAction::new(
                "split_vertical",
                "Split Vertical",
                "Split the terminal vertically",
                ActionCategory::Terminal,
                ActionCommand::SplitVertical,
            )
            .with_shortcut("Cmd+Shift+D")
            .with_icon("split_vertical"),
        );

        // Clipboard
        self.register(
            QuickAction::new(
                "copy",
                "Copy",
                "Copy selected text to clipboard",
                ActionCategory::Clipboard,
                ActionCommand::CopySelection,
            )
            .with_shortcut("Cmd+C")
            .with_icon("copy"),
        );

        self.register(
            QuickAction::new(
                "paste",
                "Paste",
                "Paste from clipboard",
                ActionCategory::Clipboard,
                ActionCommand::PasteClipboard,
            )
            .with_shortcut("Cmd+V")
            .with_icon("paste"),
        );

        // Terminal operations
        self.register(
            QuickAction::new(
                "clear_screen",
                "Clear Screen",
                "Clear the terminal screen",
                ActionCategory::Terminal,
                ActionCommand::ClearScreen,
            )
            .with_shortcut("Cmd+K")
            .with_icon("clear"),
        );

        // Search
        self.register(
            QuickAction::new(
                "search",
                "Search",
                "Search in terminal output",
                ActionCategory::Navigation,
                ActionCommand::SearchTerminal,
            )
            .with_shortcut("Cmd+F")
            .with_icon("search"),
        );

        // Fullscreen
        self.register(
            QuickAction::new(
                "toggle_fullscreen",
                "Toggle Fullscreen",
                "Toggle fullscreen mode",
                ActionCategory::Settings,
                ActionCommand::ToggleFullscreen,
            )
            .with_shortcut("Cmd+Enter")
            .with_icon("fullscreen"),
        );

        // Settings
        self.register(
            QuickAction::new(
                "open_settings",
                "Settings",
                "Open settings dialog",
                ActionCategory::Settings,
                ActionCommand::OpenSettings,
            )
            .with_shortcut("Cmd+,")
            .with_icon("settings"),
        );

        // Reload config
        self.register(
            QuickAction::new(
                "reload_config",
                "Reload Configuration",
                "Reload configuration from disk",
                ActionCategory::Settings,
                ActionCommand::Custom("reload_config".to_string()),
            )
            .with_shortcut("Cmd+Shift+R")
            .with_icon("refresh"),
        );
    }

    /// Register a new action
    pub fn register(&mut self, action: QuickAction) {
        // Remove existing action with same ID if present
        self.actions.retain(|a| a.id != action.id);
        self.actions.push(action);
    }

    /// Unregister an action by ID
    pub fn unregister(&mut self, id: &str) -> Option<QuickAction> {
        if let Some(pos) = self.actions.iter().position(|a| a.id == id) {
            Some(self.actions.remove(pos))
        } else {
            None
        }
    }

    /// Search actions with fuzzy matching
    pub fn search(&self, query: &str) -> Vec<&QuickAction> {
        let matcher = FuzzyMatcher::new();
        let mut results: Vec<(&QuickAction, f64)> = Vec::new();

        for action in &self.actions {
            if !action.enabled {
                continue;
            }

            // Search in name, description, and ID
            let name_score = matcher.score(query, &action.name);
            let desc_score = matcher.score(query, &action.description) * 0.8;
            let id_score = matcher.score(query, &action.id) * 0.6;

            let max_score = name_score.max(desc_score).max(id_score);

            if max_score > 0.0 {
                results.push((action, max_score));
            }
        }

        // Sort by score (descending)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        results.into_iter().map(|(action, _)| action).collect()
    }

    /// Execute an action by ID
    pub fn execute(&mut self, id: &str) -> Result<ActionResult, ActionError> {
        let action = self
            .actions
            .iter()
            .find(|a| a.id == id)
            .ok_or(ActionError::NotFound)?;

        if !action.enabled {
            return Err(ActionError::Disabled);
        }

        // Mark as used
        self.mark_used(id);

        // Return success - actual execution is handled by the caller
        Ok(ActionResult::Success)
    }

    /// Get actions by category
    pub fn get_by_category(&self, category: ActionCategory) -> Vec<&QuickAction> {
        self.actions
            .iter()
            .filter(|a| a.category == category && a.enabled)
            .collect()
    }

    /// Get recently used actions
    pub fn get_recent(&self, limit: usize) -> Vec<&QuickAction> {
        self.history
            .iter()
            .take(limit)
            .filter_map(|id| self.actions.iter().find(|a| &a.id == id && a.enabled))
            .collect()
    }

    /// Mark an action as recently used
    pub fn mark_used(&mut self, id: &str) {
        // Remove if already in history
        self.history.retain(|h| h != id);

        // Add to front
        self.history.push_front(id.to_string());

        // Trim to max size
        while self.history.len() > MAX_HISTORY_SIZE {
            self.history.pop_back();
        }
    }

    /// Set enabled state for an action
    pub fn set_enabled(&mut self, id: &str, enabled: bool) {
        if let Some(action) = self.actions.iter_mut().find(|a| a.id == id) {
            action.enabled = enabled;
        }
    }

    /// Get all actions
    pub fn get_all(&self) -> &[QuickAction] {
        &self.actions
    }

    /// Get mutable reference to all actions
    pub fn get_all_mut(&mut self) -> &mut Vec<QuickAction> {
        &mut self.actions
    }

    /// Get action by ID
    pub fn get(&self, id: &str) -> Option<&QuickAction> {
        self.actions.iter().find(|a| a.id == id)
    }

    /// Get mutable action by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut QuickAction> {
        self.actions.iter_mut().find(|a| a.id == id)
    }

    /// Clear history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

impl Default for QuickActionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of executing an action
#[derive(Debug, Clone, PartialEq)]
pub enum ActionResult {
    /// Action executed successfully
    Success,
    /// Action requires user confirmation before execution
    RequiresConfirmation(String),
    /// Action execution is deferred (handled asynchronously)
    DeferredExecution,
}

/// Errors that can occur when executing actions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionError {
    /// Action with the given ID was not found
    NotFound,
    /// Action is currently disabled
    Disabled,
    /// Action execution failed with an error message
    ExecutionFailed(String),
    /// Invalid command format
    InvalidCommand,
}

impl std::fmt::Display for ActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionError::NotFound => write!(f, "Action not found"),
            ActionError::Disabled => write!(f, "Action is disabled"),
            ActionError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            ActionError::InvalidCommand => write!(f, "Invalid command"),
        }
    }
}

impl std::error::Error for ActionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_creation() {
        let action = QuickAction::new(
            "test_action",
            "Test Action",
            "A test action",
            ActionCategory::Custom,
            ActionCommand::Custom("test".to_string()),
        )
        .with_shortcut("Cmd+T")
        .with_icon("test_icon");

        assert_eq!(action.id, "test_action");
        assert_eq!(action.name, "Test Action");
        assert_eq!(action.shortcut, Some("Cmd+T".to_string()));
        assert_eq!(action.icon, Some("test_icon".to_string()));
        assert!(action.enabled);
    }

    #[test]
    fn test_fuzzy_matcher_exact_match() {
        let matcher = FuzzyMatcher::new();
        let score = matcher.score("new tab", "new tab");
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_fuzzy_matcher_starts_with() {
        let matcher = FuzzyMatcher::new();
        let score = matcher.score("new", "new tab");
        assert_eq!(score, 0.9);
    }

    #[test]
    fn test_fuzzy_matcher_substring() {
        let matcher = FuzzyMatcher::new();
        let score = matcher.score("tab", "new tab");
        assert_eq!(score, 0.7);
    }

    #[test]
    fn test_fuzzy_matcher_case_insensitive() {
        let matcher = FuzzyMatcher::new();
        let score = matcher.score("NEW", "new tab");
        assert_eq!(score, 0.9);
    }

    #[test]
    fn test_fuzzy_matcher_acronym() {
        let matcher = FuzzyMatcher::new();
        let score = matcher.score("nt", "New Tab");
        assert_eq!(score, 0.55);
    }

    #[test]
    fn test_fuzzy_matcher_word_boundary() {
        let matcher = FuzzyMatcher::new();
        let score = matcher.score("tab", "New Tab Action");
        assert_eq!(score, 0.7);
    }

    #[test]
    fn test_fuzzy_matcher_no_match() {
        let matcher = FuzzyMatcher::new();
        let score = matcher.score("xyz", "new tab");
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_fuzzy_matcher_empty_query() {
        let matcher = FuzzyMatcher::new();
        let score = matcher.score("", "anything");
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_manager_register_and_get() {
        let mut manager = QuickActionManager::empty();
        let action = QuickAction::new(
            "test",
            "Test",
            "Test action",
            ActionCategory::Custom,
            ActionCommand::Custom("test".to_string()),
        );

        manager.register(action.clone());
        assert_eq!(manager.get_all().len(), 1);
        assert_eq!(manager.get("test").unwrap().id, "test");
    }

    #[test]
    fn test_manager_unregister() {
        let mut manager = QuickActionManager::empty();
        let action = QuickAction::new(
            "test",
            "Test",
            "Test action",
            ActionCategory::Custom,
            ActionCommand::Custom("test".to_string()),
        );

        manager.register(action);
        assert_eq!(manager.get_all().len(), 1);

        let removed = manager.unregister("test");
        assert!(removed.is_some());
        assert_eq!(manager.get_all().len(), 0);
    }

    #[test]
    fn test_manager_search() {
        let mut manager = QuickActionManager::empty();
        manager.register(QuickAction::new(
            "new_tab",
            "New Tab",
            "Create a new tab",
            ActionCategory::Tab,
            ActionCommand::NewTab,
        ));
        manager.register(QuickAction::new(
            "close_tab",
            "Close Tab",
            "Close current tab",
            ActionCategory::Tab,
            ActionCommand::CloseTab,
        ));

        let results = manager.search("new");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "new_tab");
    }

    #[test]
    fn test_manager_search_multiple_results() {
        let mut manager = QuickActionManager::empty();
        manager.register(QuickAction::new(
            "new_tab",
            "New Tab",
            "Create a new tab",
            ActionCategory::Tab,
            ActionCommand::NewTab,
        ));
        manager.register(QuickAction::new(
            "close_tab",
            "Close Tab",
            "Close current tab",
            ActionCategory::Tab,
            ActionCommand::CloseTab,
        ));

        let results = manager.search("tab");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_manager_get_by_category() {
        let mut manager = QuickActionManager::empty();
        manager.register(QuickAction::new(
            "new_tab",
            "New Tab",
            "Create a new tab",
            ActionCategory::Tab,
            ActionCommand::NewTab,
        ));
        manager.register(QuickAction::new(
            "clear",
            "Clear",
            "Clear screen",
            ActionCategory::Terminal,
            ActionCommand::ClearScreen,
        ));

        let tab_actions = manager.get_by_category(ActionCategory::Tab);
        assert_eq!(tab_actions.len(), 1);
        assert_eq!(tab_actions[0].id, "new_tab");
    }

    #[test]
    fn test_manager_mark_used() {
        let mut manager = QuickActionManager::empty();
        manager.register(QuickAction::new(
            "action1",
            "Action 1",
            "First action",
            ActionCategory::Custom,
            ActionCommand::Custom("1".to_string()),
        ));
        manager.register(QuickAction::new(
            "action2",
            "Action 2",
            "Second action",
            ActionCategory::Custom,
            ActionCommand::Custom("2".to_string()),
        ));

        manager.mark_used("action1");
        manager.mark_used("action2");

        let recent = manager.get_recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].id, "action2"); // Most recent first
        assert_eq!(recent[1].id, "action1");
    }

    #[test]
    fn test_manager_history_limit() {
        let mut manager = QuickActionManager::empty();

        // Register more than MAX_HISTORY_SIZE actions
        for i in 0..MAX_HISTORY_SIZE + 10 {
            let id = format!("action{}", i);
            manager.register(QuickAction::new(
                id.clone(),
                format!("Action {}", i),
                format!("Action number {}", i),
                ActionCategory::Custom,
                ActionCommand::Custom(i.to_string()),
            ));
            manager.mark_used(&id);
        }

        assert!(manager.history.len() <= MAX_HISTORY_SIZE);
    }

    #[test]
    fn test_manager_set_enabled() {
        let mut manager = QuickActionManager::empty();
        manager.register(QuickAction::new(
            "test",
            "Test",
            "Test action",
            ActionCategory::Custom,
            ActionCommand::Custom("test".to_string()),
        ));

        manager.set_enabled("test", false);
        assert!(!manager.get("test").unwrap().enabled);

        // Disabled actions shouldn't appear in search
        let results = manager.search("test");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_manager_execute_not_found() {
        let mut manager = QuickActionManager::empty();
        let result = manager.execute("nonexistent");
        assert_eq!(result, Err(ActionError::NotFound));
    }

    #[test]
    fn test_manager_execute_disabled() {
        let mut manager = QuickActionManager::empty();
        manager.register(QuickAction::new(
            "test",
            "Test",
            "Test action",
            ActionCategory::Custom,
            ActionCommand::Custom("test".to_string()),
        ).with_enabled(false));

        let result = manager.execute("test");
        assert_eq!(result, Err(ActionError::Disabled));
    }

    #[test]
    fn test_manager_execute_success() {
        let mut manager = QuickActionManager::empty();
        manager.register(QuickAction::new(
            "test",
            "Test",
            "Test action",
            ActionCategory::Custom,
            ActionCommand::Custom("test".to_string()),
        ));

        let result = manager.execute("test");
        assert!(result.is_ok());

        // Should be in history after execution
        let recent = manager.get_recent(1);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].id, "test");
    }

    #[test]
    fn test_manager_default_actions() {
        let manager = QuickActionManager::new();

        // Should have default actions registered
        assert!(manager.get_all().len() > 0);

        // Check for some expected default actions
        assert!(manager.get("new_tab").is_some());
        assert!(manager.get("close_tab").is_some());
        assert!(manager.get("copy").is_some());
        assert!(manager.get("paste").is_some());
    }

    #[test]
    fn test_action_category_name() {
        assert_eq!(ActionCategory::Terminal.name(), "Terminal");
        assert_eq!(ActionCategory::Tab.name(), "Tab");
        assert_eq!(ActionCategory::Navigation.name(), "Navigation");
        assert_eq!(ActionCategory::Clipboard.name(), "Clipboard");
        assert_eq!(ActionCategory::Session.name(), "Session");
        assert_eq!(ActionCategory::Settings.name(), "Settings");
        assert_eq!(ActionCategory::Help.name(), "Help");
        assert_eq!(ActionCategory::Custom.name(), "Custom");
    }

    #[test]
    fn test_serialization() {
        let action = QuickAction::new(
            "test",
            "Test",
            "Test action",
            ActionCategory::Custom,
            ActionCommand::Custom("test".to_string()),
        );

        // Should be serializable
        let json = serde_json::to_string(&action).unwrap();
        let deserialized: QuickAction = serde_json::from_str(&json).unwrap();

        assert_eq!(action, deserialized);
    }

    #[test]
    fn test_manager_clear_history() {
        let mut manager = QuickActionManager::empty();
        manager.register(QuickAction::new(
            "test",
            "Test",
            "Test action",
            ActionCategory::Custom,
            ActionCommand::Custom("test".to_string()),
        ));

        manager.mark_used("test");
        assert_eq!(manager.get_recent(10).len(), 1);

        manager.clear_history();
        assert_eq!(manager.get_recent(10).len(), 0);
    }

    #[test]
    fn test_fuzzy_matcher_consecutive_chars() {
        let matcher = FuzzyMatcher::new();
        let score = matcher.score("nwt", "new_window_tab");
        assert!(score > 0.0);
    }
}
