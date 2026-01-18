//! Session restoration mechanism for AgTerm
//!
//! Provides robust session persistence with:
//! - Automatic crash recovery
//! - Periodic auto-save
//! - Session versioning
//! - Backup management

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;
use thiserror::Error;

/// Version of the session file format
const SESSION_VERSION: u32 = 1;

/// Session data structure containing all state needed to restore a terminal session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    /// Session file format version
    pub version: u32,
    /// Timestamp when session was saved
    #[serde(with = "systemtime_serde")]
    pub timestamp: SystemTime,
    /// List of all tab states
    pub tabs: Vec<TabState>,
    /// Index of the currently active tab
    pub active_tab: usize,
    /// Window state (position, size, maximized)
    pub window_state: WindowState,
}

/// State of a single terminal tab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabState {
    /// Tab title (may be custom or derived from shell)
    pub title: String,
    /// Current working directory of the shell
    pub cwd: PathBuf,
    /// Shell path (e.g., /bin/zsh, /bin/bash)
    pub shell: Option<String>,
    /// Important environment variables to restore (filtered list)
    pub env_vars: Vec<(String, String)>,
    /// Optional hash of scrollback content for verification
    pub scrollback_hash: Option<String>,
    /// Tab ID for tracking
    pub id: usize,
}

/// Window state for restoring position and size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    /// Window X position (None if not tracked)
    pub x: Option<i32>,
    /// Window Y position (None if not tracked)
    pub y: Option<i32>,
    /// Window width in pixels
    pub width: u32,
    /// Window height in pixels
    pub height: u32,
    /// Whether window was maximized
    pub maximized: bool,
    /// Font size at time of save
    pub font_size: f32,
}

/// Errors that can occur during session operations
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u32, actual: u32 },

    #[error("Session file is corrupted")]
    Corrupted,

    #[error("No session file found")]
    NotFound,
}

impl SessionData {
    /// Create a new session data instance
    pub fn new(
        tabs: Vec<TabState>,
        active_tab: usize,
        window_state: WindowState,
    ) -> Self {
        Self {
            version: SESSION_VERSION,
            timestamp: SystemTime::now(),
            tabs,
            active_tab,
            window_state,
        }
    }

    /// Save session to a file
    pub fn save(&self, path: &PathBuf) -> Result<(), SessionError> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Serialize to JSON with pretty formatting
        let json = serde_json::to_string_pretty(self)?;

        // Write to temporary file first, then rename (atomic operation)
        let temp_path = path.with_extension("tmp");
        std::fs::write(&temp_path, json)?;
        std::fs::rename(&temp_path, path)?;

        tracing::info!("Session saved to {:?}", path);
        Ok(())
    }

    /// Load session from a file
    pub fn load(path: &PathBuf) -> Result<Self, SessionError> {
        if !path.exists() {
            return Err(SessionError::NotFound);
        }

        let json = std::fs::read_to_string(path)?;
        let session: SessionData = serde_json::from_str(&json)?;

        // Verify version compatibility
        if session.version != SESSION_VERSION {
            return Err(SessionError::VersionMismatch {
                expected: SESSION_VERSION,
                actual: session.version,
            });
        }

        tracing::info!("Session loaded from {:?}", path);
        Ok(session)
    }

    /// Get the default auto-save path for session files
    pub fn auto_save_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("agterm")
            .join("session.json")
    }

    /// Get the crash recovery file path
    pub fn recovery_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("agterm")
            .join("recovery.json")
    }

    /// Get backup file path with index
    fn backup_path(index: usize) -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("agterm")
            .join(format!("session.backup.{}.json", index))
    }

    /// Save a backup of the current session
    pub fn save_backup(&self, max_backups: usize) -> Result<PathBuf, SessionError> {
        // Rotate existing backups
        for i in (1..max_backups).rev() {
            let from = Self::backup_path(i);
            let to = Self::backup_path(i + 1);
            if from.exists() {
                std::fs::rename(from, to)?;
            }
        }

        // Save current session as backup #1
        let backup_path = Self::backup_path(1);
        self.save(&backup_path)?;

        tracing::info!("Session backup saved to {:?}", backup_path);
        Ok(backup_path)
    }

    /// Check if a recovery file exists (indicating a potential crash)
    pub fn has_recovery_file() -> bool {
        Self::recovery_path().exists()
    }

    /// Attempt to recover from a crash by loading the recovery file
    pub fn attempt_recovery() -> Result<Self, SessionError> {
        let recovery_path = Self::recovery_path();
        let session = Self::load(&recovery_path)?;
        tracing::info!("Session recovered from crash recovery file");
        Ok(session)
    }

    /// Save as recovery file (for crash recovery)
    pub fn save_recovery(&self) -> Result<(), SessionError> {
        let path = Self::recovery_path();
        self.save(&path)?;
        tracing::debug!("Recovery file updated");
        Ok(())
    }

    /// Clean up recovery file (call on successful exit)
    pub fn cleanup_recovery() -> Result<(), SessionError> {
        let path = Self::recovery_path();
        if path.exists() {
            std::fs::remove_file(&path)?;
            tracing::info!("Recovery file cleaned up");
        }
        Ok(())
    }

    /// List all available backup files
    pub fn list_backups() -> Result<Vec<(PathBuf, SystemTime)>, SessionError> {
        let dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("agterm");

        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(name) = path.file_name() {
                if name.to_string_lossy().starts_with("session.backup.") {
                    if let Ok(metadata) = std::fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            backups.push((path, modified));
                        }
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        backups.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(backups)
    }

    /// Validate session data integrity
    pub fn validate(&self) -> Result<(), SessionError> {
        // Check version
        if self.version != SESSION_VERSION {
            return Err(SessionError::VersionMismatch {
                expected: SESSION_VERSION,
                actual: self.version,
            });
        }

        // Check active tab is in bounds
        if self.active_tab >= self.tabs.len() && !self.tabs.is_empty() {
            return Err(SessionError::Corrupted);
        }

        // Check all tab cwds exist (warn but don't fail)
        for tab in &self.tabs {
            if !tab.cwd.exists() {
                tracing::warn!("Tab CWD does not exist: {:?}", tab.cwd);
            }
        }

        Ok(())
    }
}

impl TabState {
    /// Create a new tab state
    pub fn new(
        title: String,
        cwd: PathBuf,
        shell: Option<String>,
        id: usize,
    ) -> Self {
        Self {
            title,
            cwd,
            shell,
            env_vars: Vec::new(),
            scrollback_hash: None,
            id,
        }
    }

    /// Add an environment variable to restore
    pub fn add_env_var(&mut self, key: String, value: String) {
        self.env_vars.push((key, value));
    }

    /// Get filtered environment variables (only important ones)
    pub fn get_filtered_env_vars() -> Vec<(String, String)> {
        let important_vars = [
            "PATH",
            "HOME",
            "USER",
            "SHELL",
            "LANG",
            "LC_ALL",
            "TERM",
        ];

        let mut env_vars = Vec::new();
        for var in &important_vars {
            if let Ok(value) = std::env::var(var) {
                env_vars.push((var.to_string(), value));
            }
        }
        env_vars
    }
}

impl WindowState {
    /// Create a new window state
    pub fn new(width: u32, height: u32, font_size: f32) -> Self {
        Self {
            x: None,
            y: None,
            width,
            height,
            maximized: false,
            font_size,
        }
    }

    /// Create with position
    pub fn with_position(mut self, x: i32, y: i32) -> Self {
        self.x = Some(x);
        self.y = Some(y);
        self
    }

    /// Set maximized state
    pub fn with_maximized(mut self, maximized: bool) -> Self {
        self.maximized = maximized;
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
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_session_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let session_path = temp_dir.path().join("session.json");

        // Create a test session
        let tabs = vec![
            TabState::new(
                "Tab 1".to_string(),
                PathBuf::from("/tmp"),
                Some("/bin/bash".to_string()),
                0,
            ),
            TabState::new(
                "Tab 2".to_string(),
                PathBuf::from("/home"),
                Some("/bin/zsh".to_string()),
                1,
            ),
        ];

        let window_state = WindowState::new(1920, 1080, 14.0)
            .with_position(100, 100)
            .with_maximized(false);

        let session = SessionData::new(tabs, 0, window_state);

        // Save session
        session.save(&session_path).unwrap();
        assert!(session_path.exists());

        // Load session
        let loaded = SessionData::load(&session_path).unwrap();
        assert_eq!(loaded.version, SESSION_VERSION);
        assert_eq!(loaded.tabs.len(), 2);
        assert_eq!(loaded.active_tab, 0);
        assert_eq!(loaded.window_state.width, 1920);
        assert_eq!(loaded.window_state.height, 1080);
    }

    #[test]
    fn test_session_validation() {
        let tabs = vec![TabState::new(
            "Test".to_string(),
            PathBuf::from("/tmp"),
            None,
            0,
        )];

        let window_state = WindowState::new(800, 600, 14.0);
        let session = SessionData::new(tabs, 0, window_state);

        // Valid session should pass
        assert!(session.validate().is_ok());

        // Invalid active tab should fail
        let mut invalid_session = session.clone();
        invalid_session.active_tab = 99;
        assert!(invalid_session.validate().is_err());
    }

    #[test]
    fn test_backup_rotation() {
        let temp_dir = TempDir::new().unwrap();

        // Override the data dir for testing
        std::env::set_var("XDG_DATA_HOME", temp_dir.path());

        let tabs = vec![TabState::new(
            "Test".to_string(),
            PathBuf::from("/tmp"),
            None,
            0,
        )];

        let window_state = WindowState::new(800, 600, 14.0);
        let session = SessionData::new(tabs, 0, window_state);

        // Create multiple backups
        session.save_backup(5).unwrap();
        session.save_backup(5).unwrap();
        session.save_backup(5).unwrap();

        // Check backups exist
        let backups = SessionData::list_backups().unwrap();
        assert!(backups.len() >= 1);
        assert!(backups.len() <= 5);
    }

    #[test]
    fn test_recovery_file() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("XDG_DATA_HOME", temp_dir.path());

        let tabs = vec![TabState::new(
            "Recovery Test".to_string(),
            PathBuf::from("/tmp"),
            None,
            0,
        )];

        let window_state = WindowState::new(800, 600, 14.0);
        let session = SessionData::new(tabs, 0, window_state);

        // Save recovery file
        session.save_recovery().unwrap();
        assert!(SessionData::has_recovery_file());

        // Attempt recovery
        let recovered = SessionData::attempt_recovery().unwrap();
        assert_eq!(recovered.tabs.len(), 1);
        assert_eq!(recovered.tabs[0].title, "Recovery Test");

        // Cleanup
        SessionData::cleanup_recovery().unwrap();
        assert!(!SessionData::has_recovery_file());
    }

    #[test]
    fn test_tab_state_env_vars() {
        let mut tab = TabState::new(
            "Test".to_string(),
            PathBuf::from("/tmp"),
            None,
            0,
        );

        tab.add_env_var("TEST_VAR".to_string(), "test_value".to_string());
        assert_eq!(tab.env_vars.len(), 1);
        assert_eq!(tab.env_vars[0].0, "TEST_VAR");
        assert_eq!(tab.env_vars[0].1, "test_value");
    }
}
