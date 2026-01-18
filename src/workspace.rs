//! Workspace system for AgTerm
//!
//! Provides workspace management with:
//! - Named workspace definitions with descriptions
//! - Tab and pane layout persistence
//! - Terminal state tracking (directory, environment variables)
//! - Automatic workspace restoration
//! - TOML-based configuration format

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use thiserror::Error;

/// Version of the workspace file format
const WORKSPACE_VERSION: u32 = 1;

/// Errors that can occur during workspace operations
#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerError(#[from] toml::ser::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDeError(#[from] toml::de::Error),

    #[error("Version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u32, actual: u32 },

    #[error("Workspace not found: {0}")]
    NotFound(String),

    #[error("Workspace already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid workspace name: {0}")]
    InvalidName(String),

    #[error("Workspace is corrupted")]
    Corrupted,
}

/// A workspace definition containing all state for a named workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// Workspace file format version
    pub version: u32,
    /// Workspace name (unique identifier)
    pub name: String,
    /// User-friendly description
    pub description: String,
    /// Timestamp when workspace was created
    #[serde(with = "systemtime_serde")]
    pub created_at: SystemTime,
    /// Timestamp when workspace was last modified
    #[serde(with = "systemtime_serde")]
    pub modified_at: SystemTime,
    /// Timestamp when workspace was last used
    #[serde(with = "systemtime_serde", default = "SystemTime::now")]
    pub last_used_at: SystemTime,
    /// Layout configuration for tabs
    pub layout: WorkspaceLayout,
    /// Index of the active tab
    pub active_tab: usize,
    /// Whether to automatically restore this workspace on startup
    pub auto_restore: bool,
    /// Custom metadata for extensibility
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Layout configuration for a workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceLayout {
    /// List of tab configurations
    pub tabs: Vec<TabLayout>,
    /// Window dimensions (width, height)
    pub window_size: Option<(u32, u32)>,
    /// Font size
    pub font_size: f32,
}

/// Layout configuration for a single tab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabLayout {
    /// Tab title (custom or shell-derived)
    pub title: Option<String>,
    /// Pane layout structure
    pub pane_layout: PaneLayoutType,
    /// List of pane configurations
    pub panes: Vec<PaneConfig>,
    /// Index of the focused pane within this tab
    pub focused_pane: usize,
}

/// Type of pane layout
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaneLayoutType {
    /// Single pane (no split)
    Single,
    /// Horizontal split (top/bottom)
    HorizontalSplit,
    /// Vertical split (left/right)
    VerticalSplit,
    /// Grid layout (future extension)
    Grid { rows: usize, cols: usize },
}

/// Configuration for a single pane
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneConfig {
    /// Current working directory
    pub cwd: PathBuf,
    /// Shell path (e.g., /bin/zsh, /bin/bash)
    pub shell: Option<String>,
    /// Environment variables to set
    #[serde(default)]
    pub env_vars: Vec<(String, String)>,
    /// Initial command to run (optional)
    pub initial_command: Option<String>,
    /// Whether this pane is focused
    pub focused: bool,
}

/// Manager for workspace operations
#[derive(Debug)]
pub struct WorkspaceManager {
    /// Base directory for workspace files
    workspace_dir: PathBuf,
    /// Currently loaded workspaces (name -> workspace)
    workspaces: HashMap<String, Workspace>,
    /// Name of the currently active workspace
    active_workspace: Option<String>,
}

impl Workspace {
    /// Create a new workspace
    pub fn new(name: String, description: String) -> Result<Self, WorkspaceError> {
        // Validate workspace name
        if name.is_empty() {
            return Err(WorkspaceError::InvalidName(
                "Workspace name cannot be empty".to_string(),
            ));
        }
        if name.contains(['/', '\\', '\0']) {
            return Err(WorkspaceError::InvalidName(format!(
                "Workspace name contains invalid characters: {name}"
            )));
        }

        let now = SystemTime::now();
        Ok(Self {
            version: WORKSPACE_VERSION,
            name,
            description,
            created_at: now,
            modified_at: now,
            last_used_at: now,
            layout: WorkspaceLayout {
                tabs: Vec::new(),
                window_size: None,
                font_size: 14.0,
            },
            active_tab: 0,
            auto_restore: false,
            metadata: HashMap::new(),
        })
    }

    /// Create a workspace with a basic layout (single tab, single pane)
    pub fn with_basic_layout(name: String, description: String, cwd: PathBuf) -> Result<Self, WorkspaceError> {
        let mut workspace = Self::new(name, description)?;

        let pane = PaneConfig {
            cwd,
            shell: None, // Will use default shell
            env_vars: Vec::new(),
            initial_command: None,
            focused: true,
        };

        let tab = TabLayout {
            title: None,
            pane_layout: PaneLayoutType::Single,
            panes: vec![pane],
            focused_pane: 0,
        };

        workspace.layout.tabs.push(tab);
        Ok(workspace)
    }

    /// Add a new tab to the workspace
    pub fn add_tab(&mut self, tab: TabLayout) {
        self.layout.tabs.push(tab);
        self.modified_at = SystemTime::now();
    }

    /// Remove a tab from the workspace
    pub fn remove_tab(&mut self, index: usize) -> Result<TabLayout, WorkspaceError> {
        if index >= self.layout.tabs.len() {
            return Err(WorkspaceError::Corrupted);
        }

        let tab = self.layout.tabs.remove(index);

        // Adjust active_tab if necessary
        if self.active_tab >= self.layout.tabs.len() && !self.layout.tabs.is_empty() {
            self.active_tab = self.layout.tabs.len() - 1;
        }

        self.modified_at = SystemTime::now();
        Ok(tab)
    }

    /// Update workspace timestamp
    pub fn touch(&mut self) {
        self.modified_at = SystemTime::now();
        self.last_used_at = SystemTime::now();
    }

    /// Validate workspace data integrity
    pub fn validate(&self) -> Result<(), WorkspaceError> {
        // Check version
        if self.version != WORKSPACE_VERSION {
            return Err(WorkspaceError::VersionMismatch {
                expected: WORKSPACE_VERSION,
                actual: self.version,
            });
        }

        // Check workspace name
        if self.name.is_empty() {
            return Err(WorkspaceError::InvalidName("Empty workspace name".to_string()));
        }

        // Check active tab is in bounds
        if self.active_tab >= self.layout.tabs.len() && !self.layout.tabs.is_empty() {
            return Err(WorkspaceError::Corrupted);
        }

        // Validate each tab
        for (i, tab) in self.layout.tabs.iter().enumerate() {
            // Check focused pane is in bounds
            if tab.focused_pane >= tab.panes.len() && !tab.panes.is_empty() {
                tracing::warn!("Tab {} has invalid focused_pane index", i);
                return Err(WorkspaceError::Corrupted);
            }

            // Check pane layout matches pane count
            let expected_panes = match tab.pane_layout {
                PaneLayoutType::Single => 1,
                PaneLayoutType::HorizontalSplit | PaneLayoutType::VerticalSplit => 2,
                PaneLayoutType::Grid { rows, cols } => rows * cols,
            };

            if tab.panes.len() != expected_panes {
                tracing::warn!(
                    "Tab {} layout mismatch: expected {} panes, got {}",
                    i,
                    expected_panes,
                    tab.panes.len()
                );
                return Err(WorkspaceError::Corrupted);
            }

            // Warn if CWD doesn't exist
            for (j, pane) in tab.panes.iter().enumerate() {
                if !pane.cwd.exists() {
                    tracing::warn!("Tab {} pane {} CWD does not exist: {:?}", i, j, pane.cwd);
                }
            }
        }

        Ok(())
    }

    /// Serialize workspace to TOML string
    pub fn to_toml(&self) -> Result<String, WorkspaceError> {
        let toml = toml::to_string_pretty(self)?;
        Ok(toml)
    }

    /// Deserialize workspace from TOML string
    pub fn from_toml(toml_str: &str) -> Result<Self, WorkspaceError> {
        let workspace: Workspace = toml::from_str(toml_str)?;
        workspace.validate()?;
        Ok(workspace)
    }
}

impl WorkspaceManager {
    /// Create a new workspace manager
    pub fn new() -> Result<Self, WorkspaceError> {
        let workspace_dir = Self::default_workspace_dir();
        std::fs::create_dir_all(&workspace_dir)?;

        Ok(Self {
            workspace_dir,
            workspaces: HashMap::new(),
            active_workspace: None,
        })
    }

    /// Create a workspace manager with custom directory
    pub fn with_directory(workspace_dir: PathBuf) -> Result<Self, WorkspaceError> {
        std::fs::create_dir_all(&workspace_dir)?;

        Ok(Self {
            workspace_dir,
            workspaces: HashMap::new(),
            active_workspace: None,
        })
    }

    /// Get the default workspace directory
    pub fn default_workspace_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("agterm")
            .join("workspaces")
    }

    /// Get the file path for a workspace
    fn workspace_path(&self, name: &str) -> PathBuf {
        self.workspace_dir.join(format!("{name}.toml"))
    }

    /// Save a workspace to disk
    pub fn save_workspace(&mut self, workspace: &Workspace) -> Result<(), WorkspaceError> {
        workspace.validate()?;

        let path = self.workspace_path(&workspace.name);
        let toml = workspace.to_toml()?;

        // Write to temporary file first, then rename (atomic operation)
        let temp_path = path.with_extension("tmp");
        std::fs::write(&temp_path, toml)?;
        std::fs::rename(&temp_path, &path)?;

        // Update in-memory cache
        self.workspaces.insert(workspace.name.clone(), workspace.clone());

        tracing::info!("Workspace '{}' saved to {:?}", workspace.name, path);
        Ok(())
    }

    /// Load a workspace from disk
    pub fn load_workspace(&mut self, name: &str) -> Result<Workspace, WorkspaceError> {
        // Check cache first
        if let Some(workspace) = self.workspaces.get(name) {
            return Ok(workspace.clone());
        }

        let path = self.workspace_path(name);
        if !path.exists() {
            return Err(WorkspaceError::NotFound(name.to_string()));
        }

        let toml_str = std::fs::read_to_string(&path)?;
        let workspace = Workspace::from_toml(&toml_str)?;

        // Update cache
        self.workspaces.insert(name.to_string(), workspace.clone());

        tracing::info!("Workspace '{}' loaded from {:?}", name, path);
        Ok(workspace)
    }

    /// Create a new workspace
    pub fn create_workspace(
        &mut self,
        name: String,
        description: String,
    ) -> Result<Workspace, WorkspaceError> {
        // Check if workspace already exists
        if self.workspace_path(&name).exists() {
            return Err(WorkspaceError::AlreadyExists(name));
        }

        let workspace = Workspace::new(name, description)?;
        self.save_workspace(&workspace)?;
        Ok(workspace)
    }

    /// Delete a workspace
    pub fn delete_workspace(&mut self, name: &str) -> Result<(), WorkspaceError> {
        let path = self.workspace_path(name);
        if !path.exists() {
            return Err(WorkspaceError::NotFound(name.to_string()));
        }

        std::fs::remove_file(&path)?;
        self.workspaces.remove(name);

        // Clear active workspace if it was deleted
        if self.active_workspace.as_deref() == Some(name) {
            self.active_workspace = None;
        }

        tracing::info!("Workspace '{}' deleted", name);
        Ok(())
    }

    /// List all available workspaces
    pub fn list_workspaces(&self) -> Result<Vec<String>, WorkspaceError> {
        let mut names = Vec::new();

        for entry in std::fs::read_dir(&self.workspace_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    names.push(name.to_string());
                }
            }
        }

        names.sort();
        Ok(names)
    }

    /// Get workspace metadata without fully loading it
    pub fn get_workspace_info(&self, name: &str) -> Result<WorkspaceInfo, WorkspaceError> {
        let path = self.workspace_path(name);
        if !path.exists() {
            return Err(WorkspaceError::NotFound(name.to_string()));
        }

        let toml_str = std::fs::read_to_string(&path)?;
        let workspace: Workspace = toml::from_str(&toml_str)?;

        Ok(WorkspaceInfo {
            name: workspace.name,
            description: workspace.description,
            created_at: workspace.created_at,
            modified_at: workspace.modified_at,
            last_used_at: workspace.last_used_at,
            tab_count: workspace.layout.tabs.len(),
            auto_restore: workspace.auto_restore,
        })
    }

    /// Switch to a different workspace
    pub fn switch_workspace(&mut self, name: &str) -> Result<Workspace, WorkspaceError> {
        let mut workspace = self.load_workspace(name)?;
        workspace.touch();
        self.save_workspace(&workspace)?;
        self.active_workspace = Some(name.to_string());

        tracing::info!("Switched to workspace '{}'", name);
        Ok(workspace)
    }

    /// Get the currently active workspace
    pub fn get_active_workspace(&self) -> Option<&String> {
        self.active_workspace.as_ref()
    }

    /// Save current state as a workspace
    pub fn save_current_state(
        &mut self,
        name: String,
        description: String,
        tabs: Vec<TabLayout>,
        active_tab: usize,
        window_size: Option<(u32, u32)>,
        font_size: f32,
    ) -> Result<Workspace, WorkspaceError> {
        let mut workspace = Workspace::new(name, description)?;
        workspace.layout = WorkspaceLayout {
            tabs,
            window_size,
            font_size,
        };
        workspace.active_tab = active_tab;
        workspace.touch();

        self.save_workspace(&workspace)?;
        Ok(workspace)
    }

    /// Find the workspace marked for auto-restore
    pub fn get_auto_restore_workspace(&mut self) -> Result<Option<Workspace>, WorkspaceError> {
        for name in self.list_workspaces()? {
            let workspace = self.load_workspace(&name)?;
            if workspace.auto_restore {
                return Ok(Some(workspace));
            }
        }
        Ok(None)
    }

    /// Set auto-restore for a workspace (clears it for all others)
    pub fn set_auto_restore(&mut self, name: &str, auto_restore: bool) -> Result<(), WorkspaceError> {
        // If enabling auto-restore, disable it for all other workspaces
        if auto_restore {
            for workspace_name in self.list_workspaces()? {
                if workspace_name != name {
                    if let Ok(mut ws) = self.load_workspace(&workspace_name) {
                        if ws.auto_restore {
                            ws.auto_restore = false;
                            self.save_workspace(&ws)?;
                        }
                    }
                }
            }
        }

        // Update the target workspace
        let mut workspace = self.load_workspace(name)?;
        workspace.auto_restore = auto_restore;
        self.save_workspace(&workspace)?;

        Ok(())
    }
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new().expect("Failed to create workspace manager")
    }
}

/// Lightweight workspace information
#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    pub name: String,
    pub description: String,
    pub created_at: SystemTime,
    pub modified_at: SystemTime,
    pub last_used_at: SystemTime,
    pub tab_count: usize,
    pub auto_restore: bool,
}

impl TabLayout {
    /// Create a new tab layout with a single pane
    pub fn single_pane(cwd: PathBuf, title: Option<String>) -> Self {
        let pane = PaneConfig {
            cwd,
            shell: None,
            env_vars: Vec::new(),
            initial_command: None,
            focused: true,
        };

        Self {
            title,
            pane_layout: PaneLayoutType::Single,
            panes: vec![pane],
            focused_pane: 0,
        }
    }

    /// Create a new tab layout with horizontal split
    pub fn horizontal_split(top_cwd: PathBuf, bottom_cwd: PathBuf, title: Option<String>) -> Self {
        let top_pane = PaneConfig {
            cwd: top_cwd,
            shell: None,
            env_vars: Vec::new(),
            initial_command: None,
            focused: true,
        };

        let bottom_pane = PaneConfig {
            cwd: bottom_cwd,
            shell: None,
            env_vars: Vec::new(),
            initial_command: None,
            focused: false,
        };

        Self {
            title,
            pane_layout: PaneLayoutType::HorizontalSplit,
            panes: vec![top_pane, bottom_pane],
            focused_pane: 0,
        }
    }

    /// Create a new tab layout with vertical split
    pub fn vertical_split(left_cwd: PathBuf, right_cwd: PathBuf, title: Option<String>) -> Self {
        let left_pane = PaneConfig {
            cwd: left_cwd,
            shell: None,
            env_vars: Vec::new(),
            initial_command: None,
            focused: true,
        };

        let right_pane = PaneConfig {
            cwd: right_cwd,
            shell: None,
            env_vars: Vec::new(),
            initial_command: None,
            focused: false,
        };

        Self {
            title,
            pane_layout: PaneLayoutType::VerticalSplit,
            panes: vec![left_pane, right_pane],
            focused_pane: 0,
        }
    }
}

impl PaneConfig {
    /// Create a new pane configuration
    pub fn new(cwd: PathBuf) -> Self {
        Self {
            cwd,
            shell: None,
            env_vars: Vec::new(),
            initial_command: None,
            focused: false,
        }
    }

    /// Set the shell for this pane
    pub fn with_shell(mut self, shell: String) -> Self {
        self.shell = Some(shell);
        self
    }

    /// Add an environment variable
    pub fn with_env_var(mut self, key: String, value: String) -> Self {
        self.env_vars.push((key, value));
        self
    }

    /// Set an initial command to run
    pub fn with_command(mut self, command: String) -> Self {
        self.initial_command = Some(command);
        self
    }

    /// Set focused state
    pub fn with_focus(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

// Custom serialization for SystemTime
mod systemtime_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time
            .duration_since(UNIX_EPOCH)
            .map_err(serde::ser::Error::custom)?;
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + std::time::Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_workspace_creation() {
        let workspace = Workspace::new(
            "test-workspace".to_string(),
            "A test workspace".to_string(),
        )
        .unwrap();

        assert_eq!(workspace.name, "test-workspace");
        assert_eq!(workspace.description, "A test workspace");
        assert_eq!(workspace.version, WORKSPACE_VERSION);
        assert_eq!(workspace.layout.tabs.len(), 0);
    }

    #[test]
    fn test_workspace_with_basic_layout() {
        let cwd = PathBuf::from("/tmp");
        let workspace = Workspace::with_basic_layout(
            "dev".to_string(),
            "Development workspace".to_string(),
            cwd.clone(),
        )
        .unwrap();

        assert_eq!(workspace.layout.tabs.len(), 1);
        assert_eq!(workspace.layout.tabs[0].panes.len(), 1);
        assert_eq!(workspace.layout.tabs[0].panes[0].cwd, cwd);
    }

    #[test]
    fn test_workspace_validation() {
        let mut workspace = Workspace::new("test".to_string(), "Test".to_string()).unwrap();

        // Valid workspace should pass
        assert!(workspace.validate().is_ok());

        // Add a tab
        let tab = TabLayout::single_pane(PathBuf::from("/tmp"), None);
        workspace.add_tab(tab);
        assert!(workspace.validate().is_ok());

        // Invalid active tab should fail
        workspace.active_tab = 99;
        assert!(workspace.validate().is_err());
    }

    #[test]
    fn test_workspace_serialization() {
        let cwd = PathBuf::from("/tmp");
        let workspace = Workspace::with_basic_layout(
            "test".to_string(),
            "Test workspace".to_string(),
            cwd,
        )
        .unwrap();

        // Serialize to TOML
        let toml = workspace.to_toml().unwrap();
        assert!(toml.contains("name = \"test\""));
        assert!(toml.contains("description = \"Test workspace\""));

        // Deserialize from TOML
        let deserialized = Workspace::from_toml(&toml).unwrap();
        assert_eq!(deserialized.name, workspace.name);
        assert_eq!(deserialized.description, workspace.description);
        assert_eq!(deserialized.layout.tabs.len(), workspace.layout.tabs.len());
    }

    #[test]
    fn test_workspace_manager_create_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = WorkspaceManager::with_directory(temp_dir.path().to_path_buf()).unwrap();

        // Create a workspace
        let workspace = manager
            .create_workspace("test".to_string(), "Test workspace".to_string())
            .unwrap();

        assert_eq!(workspace.name, "test");

        // Load the workspace
        let loaded = manager.load_workspace("test").unwrap();
        assert_eq!(loaded.name, "test");
        assert_eq!(loaded.description, "Test workspace");
    }

    #[test]
    fn test_workspace_manager_list() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = WorkspaceManager::with_directory(temp_dir.path().to_path_buf()).unwrap();

        // Create multiple workspaces
        manager.create_workspace("ws1".to_string(), "First".to_string()).unwrap();
        manager.create_workspace("ws2".to_string(), "Second".to_string()).unwrap();
        manager.create_workspace("ws3".to_string(), "Third".to_string()).unwrap();

        // List workspaces
        let list = manager.list_workspaces().unwrap();
        assert_eq!(list.len(), 3);
        assert!(list.contains(&"ws1".to_string()));
        assert!(list.contains(&"ws2".to_string()));
        assert!(list.contains(&"ws3".to_string()));
    }

    #[test]
    fn test_workspace_manager_delete() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = WorkspaceManager::with_directory(temp_dir.path().to_path_buf()).unwrap();

        // Create and delete a workspace
        manager.create_workspace("test".to_string(), "Test".to_string()).unwrap();
        assert!(manager.load_workspace("test").is_ok());

        manager.delete_workspace("test").unwrap();
        assert!(manager.load_workspace("test").is_err());
    }

    #[test]
    fn test_workspace_manager_switch() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = WorkspaceManager::with_directory(temp_dir.path().to_path_buf()).unwrap();

        // Create workspaces
        manager.create_workspace("ws1".to_string(), "First".to_string()).unwrap();
        manager.create_workspace("ws2".to_string(), "Second".to_string()).unwrap();

        // Switch to ws1
        let ws1 = manager.switch_workspace("ws1").unwrap();
        assert_eq!(ws1.name, "ws1");
        assert_eq!(manager.get_active_workspace(), Some(&"ws1".to_string()));

        // Switch to ws2
        let ws2 = manager.switch_workspace("ws2").unwrap();
        assert_eq!(ws2.name, "ws2");
        assert_eq!(manager.get_active_workspace(), Some(&"ws2".to_string()));
    }

    #[test]
    fn test_workspace_auto_restore() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = WorkspaceManager::with_directory(temp_dir.path().to_path_buf()).unwrap();

        // Create workspaces
        manager.create_workspace("ws1".to_string(), "First".to_string()).unwrap();
        manager.create_workspace("ws2".to_string(), "Second".to_string()).unwrap();

        // Set auto-restore on ws1
        manager.set_auto_restore("ws1", true).unwrap();

        let ws1 = manager.load_workspace("ws1").unwrap();
        assert!(ws1.auto_restore);

        // Get auto-restore workspace
        let auto_ws = manager.get_auto_restore_workspace().unwrap();
        assert!(auto_ws.is_some());
        assert_eq!(auto_ws.unwrap().name, "ws1");

        // Set auto-restore on ws2 (should clear ws1)
        manager.set_auto_restore("ws2", true).unwrap();

        let ws1 = manager.load_workspace("ws1").unwrap();
        let ws2 = manager.load_workspace("ws2").unwrap();
        assert!(!ws1.auto_restore);
        assert!(ws2.auto_restore);
    }

    #[test]
    fn test_tab_layout_builders() {
        // Single pane
        let single = TabLayout::single_pane(PathBuf::from("/tmp"), Some("Terminal".to_string()));
        assert_eq!(single.panes.len(), 1);
        assert_eq!(single.pane_layout, PaneLayoutType::Single);

        // Horizontal split
        let horizontal = TabLayout::horizontal_split(
            PathBuf::from("/tmp"),
            PathBuf::from("/home"),
            Some("Split".to_string()),
        );
        assert_eq!(horizontal.panes.len(), 2);
        assert_eq!(horizontal.pane_layout, PaneLayoutType::HorizontalSplit);

        // Vertical split
        let vertical = TabLayout::vertical_split(
            PathBuf::from("/tmp"),
            PathBuf::from("/home"),
            Some("Split".to_string()),
        );
        assert_eq!(vertical.panes.len(), 2);
        assert_eq!(vertical.pane_layout, PaneLayoutType::VerticalSplit);
    }

    #[test]
    fn test_pane_config_builder() {
        let pane = PaneConfig::new(PathBuf::from("/tmp"))
            .with_shell("/bin/zsh".to_string())
            .with_env_var("TEST".to_string(), "value".to_string())
            .with_command("ls -la".to_string())
            .with_focus(true);

        assert_eq!(pane.cwd, PathBuf::from("/tmp"));
        assert_eq!(pane.shell, Some("/bin/zsh".to_string()));
        assert_eq!(pane.env_vars.len(), 1);
        assert_eq!(pane.initial_command, Some("ls -la".to_string()));
        assert!(pane.focused);
    }

    #[test]
    fn test_workspace_add_remove_tabs() {
        let mut workspace = Workspace::new("test".to_string(), "Test".to_string()).unwrap();

        // Add tabs
        let tab1 = TabLayout::single_pane(PathBuf::from("/tmp"), Some("Tab 1".to_string()));
        let tab2 = TabLayout::single_pane(PathBuf::from("/home"), Some("Tab 2".to_string()));

        workspace.add_tab(tab1);
        workspace.add_tab(tab2);

        assert_eq!(workspace.layout.tabs.len(), 2);

        // Remove a tab
        let removed = workspace.remove_tab(0).unwrap();
        assert_eq!(removed.title, Some("Tab 1".to_string()));
        assert_eq!(workspace.layout.tabs.len(), 1);
    }

    #[test]
    fn test_invalid_workspace_name() {
        // Empty name
        assert!(Workspace::new("".to_string(), "Test".to_string()).is_err());

        // Invalid characters
        assert!(Workspace::new("test/workspace".to_string(), "Test".to_string()).is_err());
        assert!(Workspace::new("test\\workspace".to_string(), "Test".to_string()).is_err());
    }

    #[test]
    fn test_workspace_info() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = WorkspaceManager::with_directory(temp_dir.path().to_path_buf()).unwrap();

        // Create a workspace with tabs
        let mut workspace = manager
            .create_workspace("test".to_string(), "Test workspace".to_string())
            .unwrap();

        workspace.add_tab(TabLayout::single_pane(PathBuf::from("/tmp"), None));
        workspace.add_tab(TabLayout::single_pane(PathBuf::from("/home"), None));
        manager.save_workspace(&workspace).unwrap();

        // Get workspace info
        let info = manager.get_workspace_info("test").unwrap();
        assert_eq!(info.name, "test");
        assert_eq!(info.description, "Test workspace");
        assert_eq!(info.tab_count, 2);
        assert!(!info.auto_restore);
    }

    #[test]
    fn test_workspace_save_current_state() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = WorkspaceManager::with_directory(temp_dir.path().to_path_buf()).unwrap();

        let tabs = vec![
            TabLayout::single_pane(PathBuf::from("/tmp"), Some("Tab 1".to_string())),
            TabLayout::single_pane(PathBuf::from("/home"), Some("Tab 2".to_string())),
        ];

        let workspace = manager
            .save_current_state(
                "current".to_string(),
                "Current state".to_string(),
                tabs,
                1,
                Some((1920, 1080)),
                16.0,
            )
            .unwrap();

        assert_eq!(workspace.layout.tabs.len(), 2);
        assert_eq!(workspace.active_tab, 1);
        assert_eq!(workspace.layout.window_size, Some((1920, 1080)));
        assert_eq!(workspace.layout.font_size, 16.0);
    }
}
