//! Application State Management
//!
//! This module manages the global application state using Floem's reactive signal system.
//!
//! # Overview
//!
//! The application uses `RwSignal` for reactive state management. When state changes,
//! signals notify all dependent views, triggering automatic repaints.
//!
//! # Data Structures
//!
//! - **AppState**: Global application state containing tabs, active tab, PTY manager, settings
//! - **Tab**: Individual tab with ID, title, pane tree, and activity status
//! - **PaneTree**: Binary tree structure for managing terminal panes within a tab
//!
//! # Reactivity Model
//!
//! ```ignore
//! Keyboard Input
//!     ↓
//! handle_global_shortcuts (mod.rs)
//!     ↓
//! AppState mutation (e.g., split_pane)
//!     ↓
//! RwSignal update (pane_tree.set())
//!     ↓
//! View reacts (dyn_container watches signal)
//!     ↓
//! View repaints
//! ```
//!
//! # PTY Integration
//!
//! Each pane gets a unique PTY session via `PtyManager`. The background thread
//! polls PTY output and increments version counter on changes, triggering repaints.
//!
//! # Settings Persistence
//!
//! Settings are loaded on startup and saved automatically when modified:
//! - Font size changes
//! - Theme toggle

use floem::reactive::{RwSignal, SignalGet, SignalUpdate, SignalWith};
use std::sync::Arc;
use uuid::Uuid;

use crate::terminal::pty::PtyManager;
use crate::profiles::{ProfileManager, Profile};
use crate::floem_app::pane::PaneTree;
use crate::floem_app::settings::Settings;
use crate::floem_app::theme::{Theme, ColorPalette};

use crate::floem_app::views::SearchBarState;
/// Tab state
#[derive(Clone)]
pub struct Tab {
    pub id: Uuid,
    pub title: RwSignal<String>,
    pub is_active: RwSignal<bool>,
    /// Each tab has a pane tree (supporting splits)
    pub pane_tree: RwSignal<PaneTree>,
    /// Search bar state (future feature)
    #[allow(dead_code)]
    pub search_bar: SearchBarState,
}

impl Tab {
    pub fn new(title: &str, pty_manager: &Arc<PtyManager>) -> Self {
        let tab_id = Uuid::new_v4();
        tracing::debug!("Creating new tab '{}' with ID: {}", title, tab_id);

        // Create initial pane tree with a single leaf
        let pane_tree = PaneTree::new_leaf(pty_manager);

        // Set focus on the initial pane
        if let Some(id) = pane_tree.get_all_leaf_ids().first() {
            pane_tree.set_focus(*id);
            tracing::debug!("Initial pane {} set as focused in tab {}", id, tab_id);
        }

        tracing::info!("Tab '{}' created successfully with ID: {}", title, tab_id);

        Self {
            id: tab_id,
            title: RwSignal::new(title.to_string()),
            is_active: RwSignal::new(false),
            pane_tree: RwSignal::new(pane_tree),
            search_bar: SearchBarState::new(),
        }
    }

    /// Cleanup all PTY sessions in this tab's pane tree
    pub fn cleanup(&self, pty_manager: &Arc<PtyManager>) {
        let tree = self.pane_tree.get();
        Self::cleanup_tree(&tree, pty_manager);
    }

    /// Recursively cleanup all PTY sessions in a pane tree
    fn cleanup_tree(tree: &PaneTree, pty_manager: &Arc<PtyManager>) {
        match tree {
            PaneTree::Leaf { terminal_state, .. } => {
                if let Some(session_id) = terminal_state.pty_session() {
                    if let Err(e) = pty_manager.close_session(&session_id) {
                        tracing::error!("Failed to close PTY session {}: {}", session_id, e);
                    } else {
                        tracing::info!("Closed PTY session {}", session_id);
                    }
                }
            }
            PaneTree::Split { first, second, .. } => {
                Self::cleanup_tree(&first.get(), pty_manager);
                Self::cleanup_tree(&second.get(), pty_manager);
            }
        }
    }

    /// Update tab title from the focused pane's terminal title (future feature)
    #[allow(dead_code)]
    pub fn update_title_from_terminal(&self, default_title: &str) {
        let tree = self.pane_tree.get();
        let new_title = tree.get_focused_title(default_title);

        // Only update if the title actually changed to avoid unnecessary signal updates
        if self.title.get() != new_title {
            self.title.set(new_title);
        }
    }

    /// Get PTY session of the focused pane (or the first pane if none focused)
    pub fn get_focused_pty_session(&self) -> Option<Uuid> {
        let tree = self.pane_tree.get();

        // Try to get the focused leaf's PTY session
        if let Some((_, terminal_state)) = tree.get_focused_leaf() {
            return terminal_state.pty_session();
        }

        // Fallback: get the first leaf's PTY session
        Self::get_first_pty_session(&tree)
    }

    /// Helper to get the first PTY session in the tree
    fn get_first_pty_session(tree: &PaneTree) -> Option<Uuid> {
        match tree {
            PaneTree::Leaf { terminal_state, .. } => terminal_state.pty_session(),
            PaneTree::Split { first, .. } => {
                first.with(Self::get_first_pty_session)
            }
        }
    }
}

/// Pane state
#[derive(Clone)]
#[allow(dead_code)]
pub struct Pane {
    pub id: Uuid,
    pub session_id: Option<Uuid>,
}

#[allow(dead_code)]
impl Pane {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id: None,
        }
    }
}

/// Main application state
#[derive(Clone)]
pub struct AppState {
    /// All tabs
    pub tabs: RwSignal<Vec<Tab>>,

    /// Active tab index
    pub active_tab: RwSignal<usize>,

    /// PTY manager
    pub pty_manager: Arc<PtyManager>,

    /// Font size
    pub font_size: RwSignal<f32>,

    /// Current theme
    pub theme: RwSignal<Theme>,

    /// Profile manager for managing terminal profiles (future feature)
    #[allow(dead_code)]
    pub profile_manager: Arc<std::sync::RwLock<ProfileManager>>,

    /// Default profile ID for new tabs (future feature)
    #[allow(dead_code)]
    pub default_profile_id: RwSignal<Option<String>>,
}

impl AppState {
    pub fn new() -> Self {
        tracing::info!("Initializing AgTerm application state");

        let pty_manager = Arc::new(PtyManager::new());
        tracing::debug!("PTY manager initialized");

        // Load settings from config file
        let settings = Settings::load();
        tracing::info!(
            "Loaded settings: font_size={}, theme={}, shell={:?}",
            settings.font_size,
            settings.theme_name,
            settings.shell
        );

        // Parse theme from settings
        let theme = Theme::from_name(&settings.theme_name);
        tracing::debug!("Using theme: {:?}", theme);

        // Initialize profile manager
        let mut profile_manager = ProfileManager::new();
        if let Err(e) = profile_manager.init() {
            tracing::error!("Failed to initialize profile manager: {}", e);
        } else {
            tracing::info!("Profile manager initialized with {} profiles", profile_manager.list_profiles().len());
        }

        let default_profile_id = profile_manager
            .get_default_profile()
            .map(|p| p.id.clone());

        let profile_manager = Arc::new(std::sync::RwLock::new(profile_manager));

        // Create initial tab
        tracing::info!("Creating initial terminal tab");
        let initial_tab = Tab::new("Terminal 1", &pty_manager);
        initial_tab.is_active.set(true);

        let state = Self {
            tabs: RwSignal::new(vec![initial_tab]),
            active_tab: RwSignal::new(0),
            pty_manager,
            font_size: RwSignal::new(settings.font_size),
            theme: RwSignal::new(theme),
            profile_manager,
            default_profile_id: RwSignal::new(default_profile_id),
        };

        tracing::info!("Application state initialized successfully");
        state
    }

    /// Get current color palette from theme
    pub fn colors(&self) -> ColorPalette {
        self.theme.get().colors()
    }

    /// Save current settings to config file
    pub fn save_settings(&self) {
        let mut settings = Settings::load();
        settings.font_size = self.font_size.get();
        settings.theme_name = self.theme.get().name().to_string();
        settings.validate();

        if let Err(e) = settings.save() {
            tracing::error!("Failed to save settings: {}", e);
        } else {
            tracing::info!("Settings saved successfully");
        }
    }

    /// Increase font size
    pub fn increase_font_size(&self) {
        let current = self.font_size.get();
        let new_size = (current + crate::floem_app::theme::fonts::FONT_SIZE_STEP)
            .min(crate::floem_app::theme::fonts::FONT_SIZE_MAX);

        if new_size != current {
            self.font_size.set(new_size);
            self.save_settings();
            tracing::info!("Font size increased to {}", new_size);
        }
    }

    /// Decrease font size
    pub fn decrease_font_size(&self) {
        let current = self.font_size.get();
        let new_size = (current - crate::floem_app::theme::fonts::FONT_SIZE_STEP)
            .max(crate::floem_app::theme::fonts::FONT_SIZE_MIN);

        if new_size != current {
            self.font_size.set(new_size);
            self.save_settings();
            tracing::info!("Font size decreased to {}", new_size);
        }
    }

    /// Reset font size to default
    pub fn reset_font_size(&self) {
        let default_size = crate::floem_app::theme::fonts::FONT_SIZE_DEFAULT;
        if self.font_size.get() != default_size {
            self.font_size.set(default_size);
            self.save_settings();
            tracing::info!("Font size reset to {}", default_size);
        }
    }

    /// Toggle theme between Dark and Light
    pub fn toggle_theme(&self) {
        let current = self.theme.get();
        let new_theme = current.toggle();

        self.theme.set(new_theme);
        self.save_settings();
        tracing::info!("Theme switched to {}", new_theme.name());
    }

    /// Add a new tab
    pub fn add_tab(&self) {
        let tabs = self.tabs.get();
        let new_index = tabs.len();
        tracing::info!("Creating new tab {} (total: {})", new_index + 1, new_index + 1);

        let new_tab = Tab::new(&format!("Terminal {}", new_index + 1), &self.pty_manager);

        // Deactivate all other tabs
        for (idx, tab) in tabs.iter().enumerate() {
            tab.is_active.set(false);
            tracing::debug!("Deactivated tab {}", idx);
        }
        new_tab.is_active.set(true);

        self.tabs.update(|t| t.push(new_tab));
        self.active_tab.set(new_index);
        tracing::info!("New tab created and activated: index {}", new_index);
    }

    /// Create a new tab (alias for add_tab)
    pub fn new_tab(&self) {
        self.add_tab();
    }

    /// Close the currently active tab
    pub fn close_active_tab(&self) {
        let active_index = self.active_tab.get();
        self.close_tab(active_index);
    }

    /// Close a tab by index
    pub fn close_tab(&self, index: usize) {
        let tabs = self.tabs.get();

        if index >= tabs.len() {
            tracing::warn!("Attempted to close invalid tab index: {} (total tabs: {})", index, tabs.len());
            return;
        }

        if tabs.len() <= 1 {
            tracing::warn!("Cannot close the last tab");
            return;
        }

        tracing::info!("Closing tab at index {} (total: {})", index, tabs.len());

        // Cleanup the PTY session for the closed tab
        if let Some(tab) = tabs.get(index) {
            tracing::debug!("Cleaning up PTY sessions for tab {}", index);
            tab.cleanup(&self.pty_manager);
        }

        self.tabs.update(|t| {
            t.remove(index);
        });

        // Update active tab
        let new_active = if index >= self.tabs.get().len() {
            self.tabs.get().len() - 1
        } else {
            index
        };
        self.active_tab.set(new_active);
        tracing::debug!("Active tab updated to index {}", new_active);

        // Activate the new active tab
        if let Some(tab) = self.tabs.get().get(new_active) {
            tab.is_active.set(true);
            tracing::info!("Tab {} closed successfully, activated tab {}", index, new_active);
        }
    }

    /// Select a tab by index
    pub fn select_tab(&self, index: usize) {
        let tabs = self.tabs.get();
        if index >= tabs.len() {
            tracing::warn!("Attempted to select invalid tab index: {} (total tabs: {})", index, tabs.len());
            return;
        }

        tracing::info!("Switching to tab {}", index);

        // Deactivate all tabs
        for tab in tabs.iter() {
            tab.is_active.set(false);
        }

        // Activate the selected tab
        if let Some(tab) = tabs.get(index) {
            tab.is_active.set(true);
            tracing::debug!("Tab {} activated", index);
        }

        self.active_tab.set(index);
    }

    /// Get the active tab
    pub fn active_tab_ref(&self) -> Option<Tab> {
        let tabs = self.tabs.get();
        let index = self.active_tab.get();
        tabs.get(index).cloned()
    }

    /// Stub method for tab bell notifications (not yet implemented)
    pub fn tab_has_bell(&self, _tab_index: usize) -> bool {
        false
    }

    /// Stub method to clear bell notifications (not yet implemented)
    #[allow(dead_code)]
    pub fn clear_tab_bell_notifications(&self, _tab_index: usize) {
        // Bell notifications not yet implemented in Floem GUI
    }

    // ========================================================================
    // Profile Management Methods (future features)
    // ========================================================================

    /// Get a profile by ID
    #[allow(dead_code)]
    pub fn get_profile(&self, profile_id: &str) -> Option<Profile> {
        self.profile_manager
            .read()
            .ok()
            .and_then(|pm| pm.get_profile(profile_id).cloned())
    }

    /// Get profile by name
    #[allow(dead_code)]
    pub fn get_profile_by_name(&self, name: &str) -> Option<Profile> {
        self.profile_manager
            .read()
            .ok()
            .and_then(|pm| pm.get_profile_by_name(name).cloned())
    }

    /// Get the default profile
    #[allow(dead_code)]
    pub fn get_default_profile(&self) -> Option<Profile> {
        self.profile_manager
            .read()
            .ok()
            .and_then(|pm| pm.get_default_profile().cloned())
    }

    /// Set the default profile
    #[allow(dead_code)]
    pub fn set_default_profile(&self, profile_id: &str) {
        if let Ok(mut pm) = self.profile_manager.write() {
            if let Err(e) = pm.set_default_profile(profile_id) {
                tracing::error!("Failed to set default profile: {}", e);
            } else {
                self.default_profile_id.set(Some(profile_id.to_string()));
                tracing::info!("Default profile set to: {}", profile_id);
            }
        }
    }

    /// List all available profiles
    #[allow(dead_code)]
    pub fn list_profiles(&self) -> Vec<String> {
        self.profile_manager
            .read()
            .map(|pm| pm.list_profiles())
            .unwrap_or_default()
    }

    /// Get all profiles
    #[allow(dead_code)]
    pub fn get_all_profiles(&self) -> Vec<Profile> {
        self.profile_manager
            .read()
            .map(|pm| pm.get_all_profiles().iter().map(|p| (*p).clone()).collect())
            .unwrap_or_default()
    }

    /// Add a new profile
    #[allow(dead_code)]
    pub fn add_profile(&self, profile: Profile) -> Result<String, String> {
        self.profile_manager
            .write()
            .map_err(|e| format!("Lock error: {e}"))
            .and_then(|mut pm| pm.add_profile(profile).map_err(|e| e.to_string()))
    }

    /// Update a profile
    #[allow(dead_code)]
    pub fn update_profile(&self, profile_id: &str, profile: Profile) -> Result<(), String> {
        self.profile_manager
            .write()
            .map_err(|e| format!("Lock error: {e}"))
            .and_then(|mut pm| pm.update_profile(profile_id, profile).map_err(|e| e.to_string()))
    }

    /// Delete a profile
    #[allow(dead_code)]
    pub fn delete_profile(&self, profile_id: &str) -> Result<(), String> {
        self.profile_manager
            .write()
            .map_err(|e| format!("Lock error: {e}"))
            .and_then(|mut pm| pm.delete_profile(profile_id).map_err(|e| e.to_string()))
    }

    /// Clone a profile with a new name
    #[allow(dead_code)]
    pub fn clone_profile(&self, profile_id: &str, new_name: String) -> Result<String, String> {
        self.profile_manager
            .write()
            .map_err(|e| format!("Lock error: {e}"))
            .and_then(|mut pm| pm.clone_profile(profile_id, new_name).map_err(|e| e.to_string()))
    }

    /// Add a new tab with a specific profile
    #[allow(dead_code)]
    pub fn add_tab_with_profile(&self, profile_id: Option<&str>) {
        let tabs = self.tabs.get();
        let new_index = tabs.len();

        // Get profile (use default if not specified)
        let profile = profile_id
            .and_then(|id| self.get_profile(id))
            .or_else(|| self.get_default_profile());

        let tab_name = if let Some(ref p) = profile {
            format!("{} {}", p.name, new_index + 1)
        } else {
            format!("Terminal {}", new_index + 1)
        };

        tracing::info!("Creating new tab with profile: {:?}", profile.as_ref().map(|p| &p.name));

        let new_tab = Tab::new(&tab_name, &self.pty_manager);

        // Apply profile settings if available
        if let Some(profile) = profile {
            self.apply_profile_to_tab(&new_tab, &profile);
        }

        // Deactivate all other tabs
        for tab in tabs.iter() {
            tab.is_active.set(false);
        }
        new_tab.is_active.set(true);

        self.tabs.update(|t| t.push(new_tab));
        self.active_tab.set(new_index);
        tracing::info!("New tab created and activated: index {}", new_index);
    }

    /// Apply profile settings to a tab
    #[allow(dead_code)]
    fn apply_profile_to_tab(&self, tab: &Tab, profile: &Profile) {
        // Apply font settings
        if profile.font.size != self.font_size.get() {
            tracing::debug!("Applying font size from profile: {}", profile.font.size);
            // Note: In a full implementation, each tab might have its own font size
            // For now, we apply to global state
            self.font_size.set(profile.font.size);
        }

        // Apply theme
        tracing::debug!("Applying theme from profile: {}", profile.colors.theme);
        // Convert from profile theme name to floem_app theme
        let floem_theme = crate::floem_app::theme::Theme::from_name(&profile.colors.theme);
        self.theme.set(floem_theme);

        // TODO: Apply other profile settings:
        // - Environment variables (requires PTY integration)
        // - Working directory (requires PTY integration)
        // - Startup commands (requires PTY integration)
        // - Key bindings (requires UI integration)

        tracing::info!("Profile '{}' applied to tab '{}'", profile.name, tab.title.get());
    }

    // ========================================================================
    // Pane Management Methods
    // ========================================================================

    /// Split the focused pane vertically (top/bottom)
    pub fn split_pane_vertical(&self) {
        if let Some(active_tab) = self.active_tab_ref() {
            let mut tree = active_tab.pane_tree.get();
            if let Some((focused_id, _)) = tree.get_focused_leaf() {
                tracing::info!("Splitting pane {} vertically", focused_id);
                Self::split_pane_recursive(&mut tree, focused_id, true, &self.pty_manager);
                active_tab.pane_tree.set(tree);
            }
        }
    }

    /// Split the focused pane horizontally (left/right)
    pub fn split_pane_horizontal(&self) {
        if let Some(active_tab) = self.active_tab_ref() {
            let mut tree = active_tab.pane_tree.get();
            if let Some((focused_id, _)) = tree.get_focused_leaf() {
                tracing::info!("Splitting pane {} horizontally", focused_id);
                Self::split_pane_recursive(&mut tree, focused_id, false, &self.pty_manager);
                active_tab.pane_tree.set(tree);
            }
        }
    }

    /// Navigate to the next pane
    pub fn next_pane(&self) {
        if let Some(active_tab) = self.active_tab_ref() {
            let tree = active_tab.pane_tree.get();
            if let Some(next_id) = tree.navigate(crate::floem_app::pane::NavigationDirection::Next) {
                tracing::info!("Navigating to next pane: {}", next_id);
                tree.clear_focus();
                tree.set_focus(next_id);
                active_tab.pane_tree.set(tree);
            }
        }
    }

    /// Navigate to the previous pane
    pub fn previous_pane(&self) {
        if let Some(active_tab) = self.active_tab_ref() {
            let tree = active_tab.pane_tree.get();
            if let Some(prev_id) = tree.navigate(crate::floem_app::pane::NavigationDirection::Previous) {
                tracing::info!("Navigating to previous pane: {}", prev_id);
                tree.clear_focus();
                tree.set_focus(prev_id);
                active_tab.pane_tree.set(tree);
            }
        }
    }

    /// Close the focused pane
    pub fn close_focused_pane(&self) {
        if let Some(active_tab) = self.active_tab_ref() {
            let mut tree = active_tab.pane_tree.get();

            // Don't close if it's the last pane
            if tree.count_leaves() <= 1 {
                tracing::warn!("Cannot close the last pane");
                return;
            }

            if tree.close_focused_pane(&self.pty_manager) {
                active_tab.pane_tree.set(tree);
                tracing::info!("Focused pane closed");
            }
        }
    }

    /// Helper function to split a specific pane in the tree
    fn split_pane_recursive(
        tree: &mut crate::floem_app::pane::PaneTree,
        target_id: uuid::Uuid,
        vertical: bool,
        pty_manager: &std::sync::Arc<crate::terminal::pty::PtyManager>,
    ) -> bool {
        use crate::floem_app::pane::PaneTree;
        use floem::reactive::{SignalGet, SignalUpdate};

        match tree {
            PaneTree::Leaf { id, .. } => {
                if *id == target_id {
                    if vertical {
                        tree.split_vertical(pty_manager);
                    } else {
                        tree.split_horizontal(pty_manager);
                    }
                    true
                } else {
                    false
                }
            }
            PaneTree::Split { first, second, .. } => {
                let mut first_val = first.get();
                if Self::split_pane_recursive(&mut first_val, target_id, vertical, pty_manager) {
                    first.set(first_val);
                    return true;
                }

                let mut second_val = second.get();
                if Self::split_pane_recursive(&mut second_val, target_id, vertical, pty_manager) {
                    second.set(second_val);
                    return true;
                }

                false
            }
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
