//! Tab management system for AgTerm
//!
//! Provides comprehensive tab management with:
//! - Unique tab identification using UUIDs
//! - Tab state tracking (active, running, completed, error)
//! - Tab grouping and organization
//! - Tab pinning and color coding
//! - Tab history and navigation
//! - Close confirmation logic
//! - Tab search and filtering
//! - Session persistence

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

/// Maximum size for tab history
const MAX_HISTORY_SIZE: usize = 100;

/// Unique identifier for tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TabId(Uuid);

impl TabId {
    /// Create a new unique tab ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for TabId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TabId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// State of a tab
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TabState {
    /// Tab is active and accepting input
    Active,
    /// Tab is inactive
    Inactive,
    /// Tab is running a command
    Running(String),
    /// Tab command completed with exit code
    Completed(i32),
    /// Tab encountered an error
    Error(String),
    /// Tab has a pending bell notification
    Bell,
}

impl TabState {
    /// Check if the tab is in a running state
    pub fn is_running(&self) -> bool {
        matches!(self, TabState::Running(_))
    }

    /// Check if the tab has completed
    pub fn is_completed(&self) -> bool {
        matches!(self, TabState::Completed(_))
    }

    /// Check if the tab has an error
    pub fn is_error(&self) -> bool {
        matches!(self, TabState::Error(_))
    }

    /// Check if the tab has a bell notification
    pub fn has_bell(&self) -> bool {
        matches!(self, TabState::Bell)
    }
}

impl Default for TabState {
    fn default() -> Self {
        TabState::Inactive
    }
}

/// A single terminal tab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    /// Unique identifier for this tab
    pub id: TabId,
    /// Tab title (user-defined or auto-generated)
    pub title: String,
    /// Optional icon (emoji or icon name)
    pub icon: Option<String>,
    /// Current state of the tab
    pub state: TabState,
    /// Current working directory
    pub cwd: PathBuf,
    /// Shell being used
    pub shell: String,
    /// Creation timestamp
    #[serde(with = "datetime_serde")]
    pub created_at: DateTime<Utc>,
    /// Last activity timestamp
    #[serde(with = "datetime_serde")]
    pub last_activity: DateTime<Utc>,
    /// Whether the tab is pinned
    pub pinned: bool,
    /// Optional color accent for the tab
    pub color: Option<String>,
    /// Optional group ID this tab belongs to
    pub group_id: Option<String>,
    /// Custom metadata for extensibility
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Tab {
    /// Create a new tab
    pub fn new(title: String, cwd: PathBuf, shell: String) -> Self {
        let now = Utc::now();
        Self {
            id: TabId::new(),
            title,
            icon: None,
            state: TabState::Active,
            cwd,
            shell,
            created_at: now,
            last_activity: now,
            pinned: false,
            color: None,
            group_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Update the last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = Utc::now();
    }

    /// Set the tab's state
    pub fn set_state(&mut self, state: TabState) {
        self.state = state;
        self.touch();
    }

    /// Set the tab's title
    pub fn set_title(&mut self, title: String) {
        self.title = title;
        self.touch();
    }

    /// Set the tab's color
    pub fn set_color(&mut self, color: Option<String>) {
        self.color = color;
        self.touch();
    }

    /// Toggle the pinned state
    pub fn toggle_pin(&mut self) {
        self.pinned = !self.pinned;
        self.touch();
    }

    /// Set the tab's icon
    pub fn set_icon(&mut self, icon: Option<String>) {
        self.icon = icon;
    }

    /// Set the tab's working directory
    pub fn set_cwd(&mut self, cwd: PathBuf) {
        self.cwd = cwd;
        self.touch();
    }

    /// Add or update metadata
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// A group of tabs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabGroup {
    /// Unique identifier for this group
    pub id: String,
    /// Display name for the group
    pub name: String,
    /// Color for the group
    pub color: String,
    /// Whether the group is collapsed in the UI
    pub collapsed: bool,
    /// IDs of tabs in this group
    pub tab_ids: Vec<TabId>,
}

impl TabGroup {
    /// Create a new tab group
    pub fn new(id: String, name: String, color: String) -> Self {
        Self {
            id,
            name,
            color,
            collapsed: false,
            tab_ids: Vec::new(),
        }
    }

    /// Add a tab to the group
    pub fn add_tab(&mut self, tab_id: TabId) {
        if !self.tab_ids.contains(&tab_id) {
            self.tab_ids.push(tab_id);
        }
    }

    /// Remove a tab from the group
    pub fn remove_tab(&mut self, tab_id: TabId) -> bool {
        if let Some(pos) = self.tab_ids.iter().position(|&id| id == tab_id) {
            self.tab_ids.remove(pos);
            true
        } else {
            false
        }
    }

    /// Check if the group contains a tab
    pub fn contains(&self, tab_id: TabId) -> bool {
        self.tab_ids.contains(&tab_id)
    }

    /// Get the number of tabs in the group
    pub fn tab_count(&self) -> usize {
        self.tab_ids.len()
    }

    /// Toggle the collapsed state
    pub fn toggle_collapsed(&mut self) {
        self.collapsed = !self.collapsed;
    }
}

/// Configuration for tab manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabManagerConfig {
    /// Maximum number of tabs (0 = unlimited)
    pub max_tabs: usize,
    /// Whether to require confirmation before closing tabs
    pub close_confirmation: bool,
    /// Whether new tabs preserve the current tab's CWD
    pub preserve_cwd: bool,
    /// Default shell for new tabs
    pub default_shell: String,
    /// Template for tab titles (e.g., "Tab {index}", "{shell} - {cwd}")
    pub tab_title_template: String,
}

impl Default for TabManagerConfig {
    fn default() -> Self {
        Self {
            max_tabs: 0,
            close_confirmation: false,
            preserve_cwd: true,
            default_shell: std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string()),
            tab_title_template: "Terminal {index}".to_string(),
        }
    }
}

impl TabManagerConfig {
    /// Create a new configuration with custom defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of tabs
    pub fn with_max_tabs(mut self, max_tabs: usize) -> Self {
        self.max_tabs = max_tabs;
        self
    }

    /// Set whether to require close confirmation
    pub fn with_close_confirmation(mut self, enabled: bool) -> Self {
        self.close_confirmation = enabled;
        self
    }

    /// Set whether to preserve CWD
    pub fn with_preserve_cwd(mut self, enabled: bool) -> Self {
        self.preserve_cwd = enabled;
        self
    }

    /// Set the default shell
    pub fn with_default_shell(mut self, shell: String) -> Self {
        self.default_shell = shell;
        self
    }

    /// Set the tab title template
    pub fn with_title_template(mut self, template: String) -> Self {
        self.tab_title_template = template;
        self
    }
}

/// Permission result for closing a tab
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClosePermission {
    /// Tab can be closed without confirmation
    Allowed,
    /// Tab requires confirmation with a reason
    RequiresConfirmation(String),
    /// Tab cannot be closed with a reason
    Denied(String),
}

/// Errors that can occur during tab operations
#[derive(Debug, Error)]
pub enum TabError {
    #[error("Maximum number of tabs ({0}) reached")]
    MaxTabsReached(usize),

    #[error("Tab not found: {0}")]
    TabNotFound(TabId),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Group not found: {0}")]
    GroupNotFound(String),

    #[error("Cannot close the last tab")]
    CannotCloseLastTab,

    #[error("Tab is pinned and cannot be closed")]
    TabIsPinned,
}

/// Event notifications for tab changes
#[derive(Debug, Clone)]
pub enum TabEvent {
    /// A new tab was created
    Created(TabId),
    /// A tab was closed
    Closed(TabId),
    /// A tab was activated
    Activated(TabId),
    /// A tab's title changed
    TitleChanged(TabId, String),
    /// A tab's state changed
    StateChanged(TabId, TabState),
    /// A tab was moved to a new position
    Moved(TabId, usize),
    /// A tab's group membership changed
    GroupChanged(TabId),
}

/// Tab manager for handling all tab operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabManager {
    /// List of all tabs
    tabs: Vec<Tab>,
    /// Currently active tab ID
    active_tab: Option<TabId>,
    /// Tab groups
    groups: Vec<TabGroup>,
    /// Configuration
    config: TabManagerConfig,
    /// Tab visit history (most recent first)
    #[serde(skip)]
    history: VecDeque<TabId>,
}

impl TabManager {
    /// Create a new tab manager with configuration
    pub fn new(config: TabManagerConfig) -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: None,
            groups: Vec::new(),
            config,
            history: VecDeque::new(),
        }
    }

    /// Create a tab manager with default configuration
    pub fn with_defaults() -> Self {
        Self::new(TabManagerConfig::default())
    }

    /// Create a new tab
    pub fn create_tab(
        &mut self,
        title: Option<String>,
        cwd: Option<PathBuf>,
    ) -> Result<TabId, TabError> {
        // Check max tabs limit
        if self.config.max_tabs > 0 && self.tabs.len() >= self.config.max_tabs {
            return Err(TabError::MaxTabsReached(self.config.max_tabs));
        }

        // Determine CWD
        let cwd = if let Some(cwd) = cwd {
            cwd
        } else if self.config.preserve_cwd {
            self.get_active_tab()
                .map(|tab| tab.cwd.clone())
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")))
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"))
        };

        // Determine title
        let title = title.unwrap_or_else(|| {
            self.config
                .tab_title_template
                .replace("{index}", &(self.tabs.len() + 1).to_string())
        });

        // Create the tab
        let tab = Tab::new(title, cwd, self.config.default_shell.clone());
        let tab_id = tab.id;

        self.tabs.push(tab);

        // If this is the first tab, activate it
        if self.tabs.len() == 1 {
            self.active_tab = Some(tab_id);
        }

        tracing::debug!("Created tab: {} (total: {})", tab_id, self.tabs.len());
        Ok(tab_id)
    }

    /// Close a tab by ID
    pub fn close_tab(&mut self, id: TabId) -> Result<Option<Tab>, TabError> {
        // Check if we can close this tab
        match self.can_close(id) {
            ClosePermission::Allowed => {}
            ClosePermission::RequiresConfirmation(reason) => {
                if self.config.close_confirmation {
                    return Err(TabError::InvalidOperation(format!(
                        "Confirmation required: {reason}"
                    )));
                }
            }
            ClosePermission::Denied(reason) => {
                return Err(TabError::InvalidOperation(reason));
            }
        }

        // Find and remove the tab
        let position = self
            .tabs
            .iter()
            .position(|tab| tab.id == id)
            .ok_or(TabError::TabNotFound(id))?;

        let tab = self.tabs.remove(position);

        // Remove from all groups
        for group in &mut self.groups {
            group.remove_tab(id);
        }

        // Remove from history
        self.history.retain(|&hist_id| hist_id != id);

        // If we closed the active tab, activate another
        if self.active_tab == Some(id) {
            self.active_tab = if self.tabs.is_empty() {
                None
            } else if position < self.tabs.len() {
                Some(self.tabs[position].id)
            } else if position > 0 {
                Some(self.tabs[position - 1].id)
            } else {
                Some(self.tabs[0].id)
            };
        }

        tracing::debug!("Closed tab: {} (remaining: {})", id, self.tabs.len());
        Ok(Some(tab))
    }

    /// Duplicate a tab
    pub fn duplicate_tab(&mut self, id: TabId) -> Result<TabId, TabError> {
        // Check max tabs limit
        if self.config.max_tabs > 0 && self.tabs.len() >= self.config.max_tabs {
            return Err(TabError::MaxTabsReached(self.config.max_tabs));
        }

        let tab = self.get_tab(id).ok_or(TabError::TabNotFound(id))?;

        // Create a duplicate with a new ID
        let mut new_tab = tab.clone();
        new_tab.id = TabId::new();
        new_tab.title = format!("{} (Copy)", tab.title);
        new_tab.created_at = Utc::now();
        new_tab.last_activity = Utc::now();
        new_tab.pinned = false; // Don't copy pinned state

        let new_id = new_tab.id;

        // Insert after the original tab
        let position = self.tabs.iter().position(|t| t.id == id).unwrap();
        self.tabs.insert(position + 1, new_tab);

        tracing::debug!("Duplicated tab: {} -> {}", id, new_id);
        Ok(new_id)
    }

    /// Move a tab to a new position
    pub fn move_tab(&mut self, id: TabId, new_index: usize) {
        if let Some(current_pos) = self.tabs.iter().position(|tab| tab.id == id) {
            if current_pos != new_index && new_index < self.tabs.len() {
                let tab = self.tabs.remove(current_pos);
                self.tabs.insert(new_index, tab);
                tracing::debug!("Moved tab: {} from {} to {}", id, current_pos, new_index);
            }
        }
    }

    /// Activate a tab by ID
    pub fn activate_tab(&mut self, id: TabId) -> Result<(), TabError> {
        if !self.tabs.iter().any(|tab| tab.id == id) {
            return Err(TabError::TabNotFound(id));
        }

        // Add current active tab to history
        if let Some(current_id) = self.active_tab {
            if current_id != id {
                self.history.push_front(current_id);
                if self.history.len() > MAX_HISTORY_SIZE {
                    self.history.pop_back();
                }
            }
        }

        self.active_tab = Some(id);

        // Update state of tabs
        for tab in &mut self.tabs {
            if tab.id == id {
                if matches!(tab.state, TabState::Inactive) {
                    tab.state = TabState::Active;
                }
            } else if matches!(tab.state, TabState::Active) {
                tab.state = TabState::Inactive;
            }
        }

        tracing::debug!("Activated tab: {}", id);
        Ok(())
    }

    /// Get a tab by ID
    pub fn get_tab(&self, id: TabId) -> Option<&Tab> {
        self.tabs.iter().find(|tab| tab.id == id)
    }

    /// Get a mutable reference to a tab by ID
    pub fn get_tab_mut(&mut self, id: TabId) -> Option<&mut Tab> {
        self.tabs.iter_mut().find(|tab| tab.id == id)
    }

    /// Get the currently active tab
    pub fn get_active_tab(&self) -> Option<&Tab> {
        self.active_tab.and_then(|id| self.get_tab(id))
    }

    /// Get a mutable reference to the currently active tab
    pub fn get_active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.active_tab.and_then(|id| self.get_tab_mut(id))
    }

    /// Cycle to the next tab
    pub fn next_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }

        let current_index = self
            .active_tab
            .and_then(|id| self.tabs.iter().position(|tab| tab.id == id))
            .unwrap_or(0);

        let next_index = (current_index + 1) % self.tabs.len();
        let next_id = self.tabs[next_index].id;

        let _ = self.activate_tab(next_id);
    }

    /// Cycle to the previous tab
    pub fn prev_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }

        let current_index = self
            .active_tab
            .and_then(|id| self.tabs.iter().position(|tab| tab.id == id))
            .unwrap_or(0);

        let prev_index = if current_index == 0 {
            self.tabs.len() - 1
        } else {
            current_index - 1
        };

        let prev_id = self.tabs[prev_index].id;
        let _ = self.activate_tab(prev_id);
    }

    /// Switch to a tab by index
    pub fn switch_to_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            let tab_id = self.tabs[index].id;
            let _ = self.activate_tab(tab_id);
        }
    }

    /// Go back to the previously active tab
    pub fn go_back(&mut self) {
        if let Some(prev_id) = self.history.pop_front() {
            if self.get_tab(prev_id).is_some() {
                let _ = self.activate_tab(prev_id);
            } else {
                // Tab no longer exists, try next in history
                self.go_back();
            }
        }
    }

    /// Set a tab's title
    pub fn set_tab_title(&mut self, id: TabId, title: String) {
        if let Some(tab) = self.get_tab_mut(id) {
            tab.set_title(title);
        }
    }

    /// Set a tab's color
    pub fn set_tab_color(&mut self, id: TabId, color: Option<String>) {
        if let Some(tab) = self.get_tab_mut(id) {
            tab.set_color(color);
        }
    }

    /// Toggle pin state of a tab
    pub fn toggle_pin(&mut self, id: TabId) {
        if let Some(tab) = self.get_tab_mut(id) {
            tab.toggle_pin();
        }
    }

    /// Get the total number of tabs
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Get the number of pinned tabs
    pub fn pinned_count(&self) -> usize {
        self.tabs.iter().filter(|tab| tab.pinned).count()
    }

    /// Check if a tab can be closed
    pub fn can_close(&self, id: TabId) -> ClosePermission {
        // Cannot close the last tab
        if self.tabs.len() == 1 {
            return ClosePermission::Denied("Cannot close the last tab".to_string());
        }

        if let Some(tab) = self.get_tab(id) {
            // Cannot close pinned tabs without confirmation
            if tab.pinned {
                return ClosePermission::RequiresConfirmation(
                    "Tab is pinned".to_string(),
                );
            }

            // Require confirmation for running commands
            if tab.state.is_running() {
                return ClosePermission::RequiresConfirmation(
                    "Tab is running a command".to_string(),
                );
            }
        }

        ClosePermission::Allowed
    }

    /// Find tabs by title (case-insensitive substring match)
    pub fn find_by_title(&self, query: &str) -> Vec<&Tab> {
        let query_lower = query.to_lowercase();
        self.tabs
            .iter()
            .filter(|tab| tab.title.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Find tabs by working directory
    pub fn find_by_cwd(&self, path: &Path) -> Vec<&Tab> {
        self.tabs
            .iter()
            .filter(|tab| tab.cwd.starts_with(path))
            .collect()
    }

    /// Get all tabs
    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    /// Get all tabs mutably
    pub fn tabs_mut(&mut self) -> &mut [Tab] {
        &mut self.tabs
    }

    /// Get the active tab ID
    pub fn active_tab_id(&self) -> Option<TabId> {
        self.active_tab
    }

    /// Get the configuration
    pub fn config(&self) -> &TabManagerConfig {
        &self.config
    }

    /// Get mutable configuration
    pub fn config_mut(&mut self) -> &mut TabManagerConfig {
        &mut self.config
    }

    // === Tab Group Operations ===

    /// Create a new tab group
    pub fn create_group(&mut self, name: String, color: String) -> String {
        let id = Uuid::new_v4().to_string();
        let group = TabGroup::new(id.clone(), name, color);
        self.groups.push(group);
        tracing::debug!("Created group: {}", id);
        id
    }

    /// Delete a tab group
    pub fn delete_group(&mut self, id: &str) {
        if let Some(pos) = self.groups.iter().position(|g| g.id == id) {
            let group = self.groups.remove(pos);

            // Remove group association from all tabs
            for tab_id in group.tab_ids {
                if let Some(tab) = self.get_tab_mut(tab_id) {
                    tab.group_id = None;
                }
            }

            tracing::debug!("Deleted group: {}", id);
        }
    }

    /// Add a tab to a group
    pub fn add_to_group(&mut self, tab_id: TabId, group_id: &str) {
        // First, remove from any existing group
        self.remove_from_group(tab_id);

        // Find the group and add the tab
        if let Some(group) = self.groups.iter_mut().find(|g| g.id == group_id) {
            group.add_tab(tab_id);

            // Update the tab's group_id
            if let Some(tab) = self.get_tab_mut(tab_id) {
                tab.group_id = Some(group_id.to_string());
            }

            tracing::debug!("Added tab {} to group {}", tab_id, group_id);
        }
    }

    /// Remove a tab from its group
    pub fn remove_from_group(&mut self, tab_id: TabId) {
        // Find the group containing this tab
        if let Some(group) = self.groups.iter_mut().find(|g| g.contains(tab_id)) {
            group.remove_tab(tab_id);
            tracing::debug!("Removed tab {} from group {}", tab_id, group.id);
        }

        // Update the tab's group_id
        if let Some(tab) = self.get_tab_mut(tab_id) {
            tab.group_id = None;
        }
    }

    /// Toggle the collapsed state of a group
    pub fn toggle_group_collapsed(&mut self, group_id: &str) {
        if let Some(group) = self.groups.iter_mut().find(|g| g.id == group_id) {
            group.toggle_collapsed();
        }
    }

    /// Get all tabs in a group
    pub fn get_tabs_in_group(&self, group_id: &str) -> Vec<&Tab> {
        if let Some(group) = self.groups.iter().find(|g| g.id == group_id) {
            self.tabs
                .iter()
                .filter(|tab| group.contains(tab.id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Reorder groups
    pub fn reorder_groups(&mut self, order: &[String]) {
        let mut new_groups = Vec::new();

        // Add groups in the specified order
        for id in order {
            if let Some(pos) = self.groups.iter().position(|g| g.id == *id) {
                new_groups.push(self.groups.remove(pos));
            }
        }

        // Add any remaining groups that weren't in the order
        new_groups.append(&mut self.groups);
        self.groups = new_groups;
    }

    /// Get all groups
    pub fn groups(&self) -> &[TabGroup] {
        &self.groups
    }

    /// Get a group by ID
    pub fn get_group(&self, id: &str) -> Option<&TabGroup> {
        self.groups.iter().find(|g| g.id == id)
    }

    /// Get a mutable reference to a group by ID
    pub fn get_group_mut(&mut self, id: &str) -> Option<&mut TabGroup> {
        self.groups.iter_mut().find(|g| g.id == id)
    }
}

impl Default for TabManager {
    fn default() -> Self {
        Self::with_defaults()
    }
}

// Custom serialization for DateTime<Utc>
mod datetime_serde {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        dt.to_rfc3339().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manager() -> TabManager {
        TabManager::new(TabManagerConfig::default())
    }

    #[test]
    fn test_tab_id_creation() {
        let id1 = TabId::new();
        let id2 = TabId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_tab_creation() {
        let mut manager = create_test_manager();
        let tab_id = manager
            .create_tab(Some("Test Tab".to_string()), None)
            .unwrap();

        assert_eq!(manager.tab_count(), 1);
        assert_eq!(manager.active_tab_id(), Some(tab_id));

        let tab = manager.get_tab(tab_id).unwrap();
        assert_eq!(tab.title, "Test Tab");
        assert!(!tab.pinned);
    }

    #[test]
    fn test_max_tabs_limit() {
        let mut config = TabManagerConfig::default();
        config.max_tabs = 3;
        let mut manager = TabManager::new(config);

        // Create 3 tabs (should succeed)
        for i in 0..3 {
            manager.create_tab(Some(format!("Tab {}", i)), None).unwrap();
        }

        // Try to create a 4th tab (should fail)
        let result = manager.create_tab(Some("Tab 4".to_string()), None);
        assert!(matches!(result, Err(TabError::MaxTabsReached(3))));
    }

    #[test]
    fn test_tab_close() {
        let mut manager = create_test_manager();
        let tab1 = manager.create_tab(Some("Tab 1".to_string()), None).unwrap();
        let tab2 = manager.create_tab(Some("Tab 2".to_string()), None).unwrap();

        assert_eq!(manager.tab_count(), 2);

        manager.close_tab(tab1).unwrap();
        assert_eq!(manager.tab_count(), 1);
        assert_eq!(manager.active_tab_id(), Some(tab2));
    }

    #[test]
    fn test_cannot_close_last_tab() {
        let mut manager = create_test_manager();
        let tab_id = manager.create_tab(Some("Last Tab".to_string()), None).unwrap();

        let result = manager.close_tab(tab_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_tab_activation() {
        let mut manager = create_test_manager();
        let tab1 = manager.create_tab(Some("Tab 1".to_string()), None).unwrap();
        let tab2 = manager.create_tab(Some("Tab 2".to_string()), None).unwrap();

        manager.activate_tab(tab2).unwrap();
        assert_eq!(manager.active_tab_id(), Some(tab2));

        manager.activate_tab(tab1).unwrap();
        assert_eq!(manager.active_tab_id(), Some(tab1));
    }

    #[test]
    fn test_tab_cycling() {
        let mut manager = create_test_manager();
        let tab1 = manager.create_tab(Some("Tab 1".to_string()), None).unwrap();
        let tab2 = manager.create_tab(Some("Tab 2".to_string()), None).unwrap();
        let tab3 = manager.create_tab(Some("Tab 3".to_string()), None).unwrap();

        manager.activate_tab(tab1).unwrap();

        manager.next_tab();
        assert_eq!(manager.active_tab_id(), Some(tab2));

        manager.next_tab();
        assert_eq!(manager.active_tab_id(), Some(tab3));

        manager.next_tab(); // Should wrap to tab1
        assert_eq!(manager.active_tab_id(), Some(tab1));

        manager.prev_tab(); // Should go back to tab3
        assert_eq!(manager.active_tab_id(), Some(tab3));
    }

    #[test]
    fn test_go_back() {
        let mut manager = create_test_manager();
        let tab1 = manager.create_tab(Some("Tab 1".to_string()), None).unwrap();
        let tab2 = manager.create_tab(Some("Tab 2".to_string()), None).unwrap();

        manager.activate_tab(tab1).unwrap();
        manager.activate_tab(tab2).unwrap();

        manager.go_back();
        assert_eq!(manager.active_tab_id(), Some(tab1));
    }

    #[test]
    fn test_tab_pinning() {
        let mut manager = create_test_manager();
        let tab_id = manager.create_tab(Some("Tab 1".to_string()), None).unwrap();

        assert!(!manager.get_tab(tab_id).unwrap().pinned);

        manager.toggle_pin(tab_id);
        assert!(manager.get_tab(tab_id).unwrap().pinned);
        assert_eq!(manager.pinned_count(), 1);

        manager.toggle_pin(tab_id);
        assert!(!manager.get_tab(tab_id).unwrap().pinned);
        assert_eq!(manager.pinned_count(), 0);
    }

    #[test]
    fn test_tab_duplicate() {
        let mut manager = create_test_manager();
        let tab1 = manager.create_tab(Some("Original".to_string()), None).unwrap();

        let tab2 = manager.duplicate_tab(tab1).unwrap();
        assert_ne!(tab1, tab2);
        assert_eq!(manager.tab_count(), 2);

        let original = manager.get_tab(tab1).unwrap();
        let duplicate = manager.get_tab(tab2).unwrap();

        assert_eq!(duplicate.title, "Original (Copy)");
        assert_eq!(duplicate.cwd, original.cwd);
        assert_eq!(duplicate.shell, original.shell);
    }

    #[test]
    fn test_tab_move() {
        let mut manager = create_test_manager();
        let tab1 = manager.create_tab(Some("Tab 1".to_string()), None).unwrap();
        let tab2 = manager.create_tab(Some("Tab 2".to_string()), None).unwrap();
        let tab3 = manager.create_tab(Some("Tab 3".to_string()), None).unwrap();

        // Move tab3 to position 0
        manager.move_tab(tab3, 0);

        let tabs = manager.tabs();
        assert_eq!(tabs[0].id, tab3);
        assert_eq!(tabs[1].id, tab1);
        assert_eq!(tabs[2].id, tab2);
    }

    #[test]
    fn test_find_by_title() {
        let mut manager = create_test_manager();
        manager.create_tab(Some("Frontend Work".to_string()), None).unwrap();
        manager.create_tab(Some("Backend API".to_string()), None).unwrap();
        manager.create_tab(Some("Frontend Tests".to_string()), None).unwrap();

        let results = manager.find_by_title("frontend");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_find_by_cwd() {
        let mut manager = create_test_manager();
        manager
            .create_tab(Some("Tab 1".to_string()), Some(PathBuf::from("/home/user/project")))
            .unwrap();
        manager
            .create_tab(Some("Tab 2".to_string()), Some(PathBuf::from("/home/user/project/src")))
            .unwrap();
        manager
            .create_tab(Some("Tab 3".to_string()), Some(PathBuf::from("/tmp")))
            .unwrap();

        let results = manager.find_by_cwd(&PathBuf::from("/home/user/project"));
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_tab_state() {
        let mut manager = create_test_manager();
        let tab_id = manager.create_tab(Some("Test".to_string()), None).unwrap();

        let tab = manager.get_tab_mut(tab_id).unwrap();
        tab.set_state(TabState::Running("npm test".to_string()));
        assert!(tab.state.is_running());

        tab.set_state(TabState::Completed(0));
        assert!(tab.state.is_completed());

        tab.set_state(TabState::Error("Command failed".to_string()));
        assert!(tab.state.is_error());
    }

    #[test]
    fn test_close_permission() {
        let mut manager = create_test_manager();
        let tab1 = manager.create_tab(Some("Tab 1".to_string()), None).unwrap();
        let tab2 = manager.create_tab(Some("Tab 2".to_string()), None).unwrap();

        // Should be allowed to close
        assert!(matches!(manager.can_close(tab2), ClosePermission::Allowed));

        // Pin the tab
        manager.toggle_pin(tab2);
        assert!(matches!(
            manager.can_close(tab2),
            ClosePermission::RequiresConfirmation(_)
        ));

        // Set running state
        manager.get_tab_mut(tab2).unwrap().set_state(TabState::Running("test".to_string()));
        assert!(matches!(
            manager.can_close(tab2),
            ClosePermission::RequiresConfirmation(_)
        ));

        // Cannot close last tab
        manager.close_tab(tab2).ok();
        assert!(matches!(
            manager.can_close(tab1),
            ClosePermission::Denied(_)
        ));
    }

    #[test]
    fn test_tab_groups() {
        let mut manager = create_test_manager();
        let tab1 = manager.create_tab(Some("Tab 1".to_string()), None).unwrap();
        let tab2 = manager.create_tab(Some("Tab 2".to_string()), None).unwrap();

        // Create a group
        let group_id = manager.create_group("Development".to_string(), "#FF0000".to_string());
        assert_eq!(manager.groups().len(), 1);

        // Add tabs to the group
        manager.add_to_group(tab1, &group_id);
        manager.add_to_group(tab2, &group_id);

        assert_eq!(manager.get_tabs_in_group(&group_id).len(), 2);
        assert_eq!(
            manager.get_tab(tab1).unwrap().group_id,
            Some(group_id.clone())
        );

        // Remove a tab from the group
        manager.remove_from_group(tab1);
        assert_eq!(manager.get_tabs_in_group(&group_id).len(), 1);
        assert_eq!(manager.get_tab(tab1).unwrap().group_id, None);
    }

    #[test]
    fn test_group_collapse() {
        let mut manager = create_test_manager();
        let group_id = manager.create_group("Test".to_string(), "#00FF00".to_string());

        let group = manager.get_group(&group_id).unwrap();
        assert!(!group.collapsed);

        manager.toggle_group_collapsed(&group_id);
        let group = manager.get_group(&group_id).unwrap();
        assert!(group.collapsed);
    }

    #[test]
    fn test_group_deletion() {
        let mut manager = create_test_manager();
        let tab1 = manager.create_tab(Some("Tab 1".to_string()), None).unwrap();

        let group_id = manager.create_group("Test".to_string(), "#0000FF".to_string());
        manager.add_to_group(tab1, &group_id);

        assert!(manager.get_tab(tab1).unwrap().group_id.is_some());

        manager.delete_group(&group_id);
        assert_eq!(manager.groups().len(), 0);
        assert!(manager.get_tab(tab1).unwrap().group_id.is_none());
    }

    #[test]
    fn test_group_reorder() {
        let mut manager = create_test_manager();
        let group1 = manager.create_group("Group 1".to_string(), "#FF0000".to_string());
        let group2 = manager.create_group("Group 2".to_string(), "#00FF00".to_string());
        let group3 = manager.create_group("Group 3".to_string(), "#0000FF".to_string());

        // Reorder: 3, 1, 2
        manager.reorder_groups(&[group3.clone(), group1.clone(), group2.clone()]);

        let groups = manager.groups();
        assert_eq!(groups[0].id, group3);
        assert_eq!(groups[1].id, group1);
        assert_eq!(groups[2].id, group2);
    }

    #[test]
    fn test_tab_metadata() {
        let mut manager = create_test_manager();
        let tab_id = manager.create_tab(Some("Test".to_string()), None).unwrap();

        let tab = manager.get_tab_mut(tab_id).unwrap();
        tab.set_metadata("session_id".to_string(), "12345".to_string());
        tab.set_metadata("user".to_string(), "alice".to_string());

        assert_eq!(tab.get_metadata("session_id"), Some(&"12345".to_string()));
        assert_eq!(tab.get_metadata("user"), Some(&"alice".to_string()));
        assert_eq!(tab.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_tab_title_template() {
        let mut config = TabManagerConfig::default();
        config.tab_title_template = "Terminal {index}".to_string();
        let mut manager = TabManager::new(config);

        let tab_id = manager.create_tab(None, None).unwrap();
        let tab = manager.get_tab(tab_id).unwrap();
        assert_eq!(tab.title, "Terminal 1");
    }

    #[test]
    fn test_preserve_cwd() {
        let mut config = TabManagerConfig::default();
        config.preserve_cwd = true;
        let mut manager = TabManager::new(config);

        let tab1_id = manager
            .create_tab(Some("Tab 1".to_string()), Some(PathBuf::from("/home/user")))
            .unwrap();

        manager.activate_tab(tab1_id).unwrap();

        let tab2_id = manager.create_tab(Some("Tab 2".to_string()), None).unwrap();
        let tab2 = manager.get_tab(tab2_id).unwrap();

        assert_eq!(tab2.cwd, PathBuf::from("/home/user"));
    }

    #[test]
    fn test_switch_to_tab() {
        let mut manager = create_test_manager();
        let _tab1 = manager.create_tab(Some("Tab 1".to_string()), None).unwrap();
        let tab2 = manager.create_tab(Some("Tab 2".to_string()), None).unwrap();
        let _tab3 = manager.create_tab(Some("Tab 3".to_string()), None).unwrap();

        manager.switch_to_tab(1);
        assert_eq!(manager.active_tab_id(), Some(tab2));
    }

    #[test]
    fn test_history_size_limit() {
        let mut manager = create_test_manager();

        // Create tabs
        let mut tab_ids = Vec::new();
        for i in 0..10 {
            let id = manager.create_tab(Some(format!("Tab {}", i)), None).unwrap();
            tab_ids.push(id);
        }

        // Activate tabs to build up history
        for &id in &tab_ids {
            manager.activate_tab(id).unwrap();
        }

        // History should be limited
        assert!(manager.history.len() <= MAX_HISTORY_SIZE);
    }
}
