//! Settings Management
//!
//! This module handles persistent configuration storage using TOML format.
//!
//! # Configuration File
//!
//! Settings are stored in `~/.config/agterm/config.toml`:
//!
//! ```toml
//! font_size = 14.0
//! theme_name = "Ghostty Dark"
//! shell = "/bin/zsh"
//! default_cols = 80
//! default_rows = 24
//! ```
//!
//! # Persistence
//!
//! Settings are automatically saved when:
//! - Font size changes (via Cmd+/-, Cmd+0)
//! - Theme toggle (via Cmd+T)
//!
//! Settings are loaded on application startup with sensible defaults.
//!
//! # Validation
//!
//! - Font size is clamped to 8.0-24.0 range
//! - Theme names must match exactly (case-sensitive)
//! - Shell paths are validated and expanded

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Cursor style options
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CursorStyle {
    Block,
    Underline,
    Bar,
}

impl Default for CursorStyle {
    fn default() -> Self {
        Self::Block
    }
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Font size in points
    pub font_size: f32,

    /// Theme name ("Ghostty Dark" or "Ghostty Light")
    pub theme_name: String,

    /// Shell command (default: $SHELL or /bin/zsh)
    pub shell: Option<String>,

    /// Default terminal size
    pub default_cols: Option<u16>,
    pub default_rows: Option<u16>,

    /// Cursor style
    #[serde(default)]
    pub cursor_style: CursorStyle,

    /// Enable cursor blinking
    #[serde(default = "default_cursor_blink")]
    pub cursor_blink: bool,

    /// Number of scrollback lines to keep
    #[serde(default = "default_scroll_back_lines")]
    pub scroll_back_lines: usize,

    /// Automatically copy text on selection
    #[serde(default)]
    pub copy_on_select: bool,

    /// Show confirmation dialog when closing with running processes
    #[serde(default = "default_confirm_close")]
    pub confirm_close_with_running_processes: bool,

    /// Default profile ID for new tabs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_profile_id: Option<String>,
}

fn default_cursor_blink() -> bool {
    true
}

fn default_scroll_back_lines() -> usize {
    10000
}

fn default_confirm_close() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            theme_name: "Ghostty Dark".to_string(),
            shell: None,
            default_cols: Some(80),
            default_rows: Some(24),
            cursor_style: CursorStyle::default(),
            cursor_blink: default_cursor_blink(),
            scroll_back_lines: default_scroll_back_lines(),
            copy_on_select: false,
            confirm_close_with_running_processes: default_confirm_close(),
            default_profile_id: None,
        }
    }
}

impl Settings {
    /// Get the config file path
    ///
    /// Platform-specific paths:
    /// - macOS: ~/Library/Application Support/agterm/config.toml
    /// - Linux: ~/.config/agterm/config.toml
    /// - Windows: %APPDATA%\agterm\config.toml
    pub fn config_path() -> Result<PathBuf, std::io::Error> {
        #[cfg(target_os = "macos")]
        {
            let home = dirs::home_dir()
                .ok_or_else(|| std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not find home directory"
                ))?;

            let app_support = home.join("Library").join("Application Support").join("agterm");

            // Create directory if it doesn't exist
            if !app_support.exists() {
                fs::create_dir_all(&app_support)?;
            }

            Ok(app_support.join("config.toml"))
        }

        #[cfg(not(target_os = "macos"))]
        {
            let config_dir = dirs::config_dir()
                .ok_or_else(|| std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not find config directory"
                ))?;

            let agterm_config = config_dir.join("agterm");

            // Create directory if it doesn't exist
            if !agterm_config.exists() {
                fs::create_dir_all(&agterm_config)?;
            }

            Ok(agterm_config.join("config.toml"))
        }
    }

    /// Load settings from config file
    ///
    /// If no config file exists, creates one with default settings.
    /// If the config file is malformed, logs a warning and falls back to defaults.
    pub fn load() -> Self {
        match Self::config_path() {
            Ok(path) => {
                if path.exists() {
                    // Try to load existing config
                    match fs::read_to_string(&path) {
                        Ok(contents) => {
                            match toml::from_str::<Settings>(&contents) {
                                Ok(mut settings) => {
                                    tracing::info!("Loaded settings from {:?}", path);
                                    settings.validate();
                                    return settings;
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "Failed to parse settings file {:?}: {}. Using defaults and backing up old file.",
                                        path, e
                                    );

                                    // Backup the corrupted file
                                    let backup_path = path.with_extension("toml.backup");
                                    if let Err(backup_err) = fs::rename(&path, &backup_path) {
                                        tracing::error!("Failed to backup corrupted config: {}", backup_err);
                                    } else {
                                        tracing::info!("Backed up corrupted config to {:?}", backup_path);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to read settings file {:?}: {}, using defaults", path, e);
                        }
                    }
                } else {
                    tracing::info!("No settings file found at {:?}, creating default config", path);
                }

                // Create default config file
                let default_settings = Settings::default();
                if let Err(e) = default_settings.save() {
                    tracing::error!("Failed to save default settings: {}", e);
                }
                default_settings
            }
            Err(e) => {
                tracing::error!("Failed to get config path: {}, using defaults", e);
                Settings::default()
            }
        }
    }

    /// Save settings to config file
    ///
    /// Uses atomic write (write to temp file then rename) to prevent corruption.
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path()?;
        let contents = toml::to_string_pretty(self)?;

        // Write to temporary file first
        let temp_path = path.with_extension("toml.tmp");
        fs::write(&temp_path, &contents)?;

        // Atomic rename
        fs::rename(&temp_path, &path)?;

        tracing::info!("Saved settings to {:?}", path);
        Ok(())
    }

    /// Auto-save settings to disk
    ///
    /// Convenience method that logs but doesn't propagate errors.
    #[allow(dead_code)]
    pub fn auto_save(&self) {
        if let Err(e) = self.save() {
            tracing::error!("Failed to auto-save settings: {}", e);
        }
    }

    /// Validate and fix any out-of-range settings
    pub fn validate(&mut self) {
        // Clamp font size
        self.font_size = self.font_size.clamp(
            crate::floem_app::theme::fonts::FONT_SIZE_MIN,
            crate::floem_app::theme::fonts::FONT_SIZE_MAX,
        );

        // Validate scrollback lines (reasonable limits)
        if self.scroll_back_lines < 100 {
            tracing::warn!("scroll_back_lines too small ({}), setting to 100", self.scroll_back_lines);
            self.scroll_back_lines = 100;
        } else if self.scroll_back_lines > 100_000 {
            tracing::warn!("scroll_back_lines too large ({}), setting to 100000", self.scroll_back_lines);
            self.scroll_back_lines = 100_000;
        }

        // Validate terminal size
        if let Some(cols) = self.default_cols {
            if !(20..=500).contains(&cols) {
                tracing::warn!("default_cols out of range ({}), using 80", cols);
                self.default_cols = Some(80);
            }
        }

        if let Some(rows) = self.default_rows {
            if !(10..=200).contains(&rows) {
                tracing::warn!("default_rows out of range ({}), using 24", rows);
                self.default_rows = Some(24);
            }
        }
    }

    /// Validate and clamp font size
    #[allow(dead_code)]
    #[deprecated(since = "0.1.0", note = "Use validate() instead")]
    pub fn clamp_font_size(&mut self) {
        self.font_size = self.font_size.clamp(
            crate::floem_app::theme::fonts::FONT_SIZE_MIN,
            crate::floem_app::theme::fonts::FONT_SIZE_MAX,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.font_size, 14.0);
        assert_eq!(settings.theme_name, "Ghostty Dark");
        assert_eq!(settings.cursor_style, CursorStyle::Block);
        assert_eq!(settings.cursor_blink, true);
        assert_eq!(settings.scroll_back_lines, 10000);
        assert_eq!(settings.copy_on_select, false);
        assert_eq!(settings.confirm_close_with_running_processes, true);
    }

    #[test]
    #[allow(deprecated)]
    fn test_clamp_font_size() {
        let mut settings = Settings::default();
        settings.font_size = 100.0;
        settings.clamp_font_size();
        assert_eq!(settings.font_size, 24.0);

        settings.font_size = 1.0;
        settings.clamp_font_size();
        assert_eq!(settings.font_size, 8.0);
    }

    #[test]
    fn test_validate() {
        let mut settings = Settings::default();

        // Test font size validation
        settings.font_size = 100.0;
        settings.validate();
        assert_eq!(settings.font_size, 24.0);

        // Test scrollback validation
        settings.scroll_back_lines = 50;
        settings.validate();
        assert_eq!(settings.scroll_back_lines, 100);

        settings.scroll_back_lines = 200_000;
        settings.validate();
        assert_eq!(settings.scroll_back_lines, 100_000);

        // Test cols validation
        settings.default_cols = Some(5);
        settings.validate();
        assert_eq!(settings.default_cols, Some(80));

        settings.default_cols = Some(1000);
        settings.validate();
        assert_eq!(settings.default_cols, Some(80));

        // Test rows validation
        settings.default_rows = Some(5);
        settings.validate();
        assert_eq!(settings.default_rows, Some(24));
    }

    #[test]
    fn test_cursor_style() {
        assert_eq!(CursorStyle::default(), CursorStyle::Block);

        let settings = Settings {
            cursor_style: CursorStyle::Bar,
            ..Default::default()
        };
        assert_eq!(settings.cursor_style, CursorStyle::Bar);
    }

    #[test]
    fn test_toml_serialization() {
        let settings = Settings::default();
        let toml_str = toml::to_string_pretty(&settings).unwrap();

        // Should be able to parse it back
        let parsed: Settings = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.font_size, settings.font_size);
        assert_eq!(parsed.cursor_style, settings.cursor_style);
        assert_eq!(parsed.cursor_blink, settings.cursor_blink);
    }

    #[test]
    fn test_cursor_style_serialization() {
        // Test that cursor styles serialize to lowercase within a struct context
        // (TOML cannot serialize bare enum values, only as part of a struct)
        let mut settings = Settings::default();

        settings.cursor_style = CursorStyle::Block;
        let toml_str = toml::to_string(&settings).unwrap();
        assert!(toml_str.contains("cursor_style = \"block\""));

        settings.cursor_style = CursorStyle::Bar;
        let toml_str = toml::to_string(&settings).unwrap();
        assert!(toml_str.contains("cursor_style = \"bar\""));

        settings.cursor_style = CursorStyle::Underline;
        let toml_str = toml::to_string(&settings).unwrap();
        assert!(toml_str.contains("cursor_style = \"underline\""));
    }

    #[test]
    fn test_partial_config_loading() {
        // Test that settings with only some fields can be loaded with defaults for the rest
        let minimal_toml = r#"
            font_size = 16.0
            theme_name = "Ghostty Light"
        "#;

        let settings: Settings = toml::from_str(minimal_toml).unwrap();
        assert_eq!(settings.font_size, 16.0);
        assert_eq!(settings.theme_name, "Ghostty Light");
        // These should have default values
        assert_eq!(settings.cursor_style, CursorStyle::Block);
        assert_eq!(settings.cursor_blink, true);
        assert_eq!(settings.scroll_back_lines, 10000);
    }
}
