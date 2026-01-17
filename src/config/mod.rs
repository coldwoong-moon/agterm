//! Configuration management for AgTerm
//!
//! This module handles loading, parsing, and managing configuration from:
//! 1. Embedded default_config.toml (compile-time defaults)
//! 2. User config at ~/.config/agterm/config.toml (or platform-specific location)
//! 3. Project-local config at ./.agterm/config.toml

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Default configuration embedded in binary
const DEFAULT_CONFIG: &str = include_str!("../../default_config.toml");

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub appearance: AppearanceConfig,
    #[serde(default)]
    pub terminal: TerminalConfig,
    #[serde(default)]
    pub keybindings: KeybindingsConfig,
    #[serde(default)]
    pub shell: ShellConfig,
    #[serde(default)]
    pub mouse: MouseConfig,
    #[serde(default)]
    pub pty: PtyConfig,
    #[serde(default)]
    pub tui: TuiConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub debug: DebugConfig,
}

/// General application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_app_name")]
    pub app_name: String,
    #[serde(default)]
    pub default_shell: Option<String>,
    #[serde(default)]
    pub default_working_dir: Option<PathBuf>,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            app_name: default_app_name(),
            default_shell: None,
            default_working_dir: None,
        }
    }
}

/// Appearance settings (fonts, colors, theme)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    #[serde(default = "default_font_family")]
    pub font_family: String,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_background_opacity")]
    pub background_opacity: f32,
    #[serde(default = "default_true")]
    pub use_ligatures: bool,
    #[serde(default)]
    pub color_scheme: Option<ColorScheme>,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            font_family: default_font_family(),
            font_size: default_font_size(),
            theme: default_theme(),
            background_opacity: default_background_opacity(),
            use_ligatures: true,
            color_scheme: None,
        }
    }
}

/// Custom color scheme (optional override)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub background: String,
    pub foreground: String,
    pub cursor: String,
    #[serde(default)]
    pub selection: Option<String>,
    // ANSI colors (0-15)
    #[serde(default)]
    pub black: Option<String>,
    #[serde(default)]
    pub red: Option<String>,
    #[serde(default)]
    pub green: Option<String>,
    #[serde(default)]
    pub yellow: Option<String>,
    #[serde(default)]
    pub blue: Option<String>,
    #[serde(default)]
    pub magenta: Option<String>,
    #[serde(default)]
    pub cyan: Option<String>,
    #[serde(default)]
    pub white: Option<String>,
    // Bright variants
    #[serde(default)]
    pub bright_black: Option<String>,
    #[serde(default)]
    pub bright_red: Option<String>,
    #[serde(default)]
    pub bright_green: Option<String>,
    #[serde(default)]
    pub bright_yellow: Option<String>,
    #[serde(default)]
    pub bright_blue: Option<String>,
    #[serde(default)]
    pub bright_magenta: Option<String>,
    #[serde(default)]
    pub bright_cyan: Option<String>,
    #[serde(default)]
    pub bright_white: Option<String>,
}

/// Terminal behavior settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    #[serde(default = "default_scrollback_lines")]
    pub scrollback_lines: usize,
    #[serde(default = "default_cursor_style")]
    pub cursor_style: CursorStyle,
    #[serde(default = "default_true")]
    pub cursor_blink: bool,
    #[serde(default = "default_cursor_blink_interval")]
    pub cursor_blink_interval_ms: u64,
    #[serde(default = "default_true")]
    pub bell_enabled: bool,
    #[serde(default = "default_bell_style")]
    pub bell_style: BellStyle,
    #[serde(default = "default_true")]
    pub bracketed_paste: bool,
    #[serde(default = "default_true")]
    pub auto_scroll_on_output: bool,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            scrollback_lines: default_scrollback_lines(),
            cursor_style: default_cursor_style(),
            cursor_blink: true,
            cursor_blink_interval_ms: default_cursor_blink_interval(),
            bell_enabled: true,
            bell_style: default_bell_style(),
            bracketed_paste: true,
            auto_scroll_on_output: true,
        }
    }
}

/// Cursor style options
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CursorStyle {
    Block,
    Underline,
    Beam,
}

impl Default for CursorStyle {
    fn default() -> Self {
        Self::Block
    }
}

/// Bell notification style
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BellStyle {
    Visual,
    Sound,
    Both,
    None,
}

impl Default for BellStyle {
    fn default() -> Self {
        Self::Visual
    }
}

/// Keybinding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    #[serde(default = "default_keybinding_mode")]
    pub mode: String, // "default", "vim", "emacs"
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            mode: default_keybinding_mode(),
            custom: HashMap::new(),
        }
    }
}

/// Shell configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    #[serde(default)]
    pub program: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default = "default_true")]
    pub login_shell: bool,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            program: None,
            args: Vec::new(),
            env: HashMap::new(),
            login_shell: true,
        }
    }
}

/// Mouse behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub reporting: bool,
    #[serde(default = "default_selection_mode")]
    pub selection_mode: SelectionMode,
    #[serde(default = "default_true")]
    pub copy_on_select: bool,
    #[serde(default = "default_true")]
    pub middle_click_paste: bool,
}

impl Default for MouseConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            reporting: true,
            selection_mode: default_selection_mode(),
            copy_on_select: true,
            middle_click_paste: true,
        }
    }
}

/// Selection mode for mouse
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SelectionMode {
    Character,
    Word,
    Line,
}

impl Default for SelectionMode {
    fn default() -> Self {
        Self::Character
    }
}

/// PTY configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtyConfig {
    #[serde(default = "default_max_sessions")]
    pub max_sessions: usize,
    #[serde(default = "default_cols")]
    pub default_cols: u16,
    #[serde(default = "default_rows")]
    pub default_rows: u16,
    #[serde(default = "default_scrollback_lines")]
    pub scrollback_lines: usize,
}

impl Default for PtyConfig {
    fn default() -> Self {
        Self {
            max_sessions: default_max_sessions(),
            default_cols: default_cols(),
            default_rows: default_rows(),
            scrollback_lines: default_scrollback_lines(),
        }
    }
}

/// TUI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiConfig {
    #[serde(default = "default_target_fps")]
    pub target_fps: u32,
    #[serde(default = "default_false")]
    pub show_line_numbers: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_true")]
    pub mouse_support: bool,
    #[serde(default = "default_keybinding_mode")]
    pub keybindings: String,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            target_fps: default_target_fps(),
            show_line_numbers: false,
            theme: default_theme(),
            mouse_support: true,
            keybindings: default_keybinding_mode(),
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
    #[serde(default = "default_true")]
    pub timestamps: bool,
    #[serde(default = "default_false")]
    pub file_line: bool,
    #[serde(default = "default_true")]
    pub file_output: bool,
    #[serde(default)]
    pub file_path: Option<PathBuf>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
            timestamps: true,
            file_line: false,
            file_output: true,
            file_path: None,
        }
    }
}

/// Debug panel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    #[serde(default = "default_false")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub show_fps: bool,
    #[serde(default = "default_true")]
    pub show_pty_stats: bool,
    #[serde(default = "default_log_buffer_size")]
    pub log_buffer_size: usize,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            show_fps: true,
            show_pty_stats: true,
            log_buffer_size: default_log_buffer_size(),
        }
    }
}

// ============================================================================
// Default value functions
// ============================================================================

fn default_app_name() -> String {
    "agterm".to_string()
}

fn default_font_family() -> String {
    "D2Coding".to_string()
}

fn default_font_size() -> f32 {
    14.0
}

fn default_theme() -> String {
    "default".to_string()
}

fn default_background_opacity() -> f32 {
    1.0
}

fn default_scrollback_lines() -> usize {
    10000
}

fn default_cursor_style() -> CursorStyle {
    CursorStyle::Block
}

fn default_cursor_blink_interval() -> u64 {
    530
}

fn default_bell_style() -> BellStyle {
    BellStyle::Visual
}

fn default_keybinding_mode() -> String {
    "default".to_string()
}

fn default_selection_mode() -> SelectionMode {
    SelectionMode::Character
}

fn default_max_sessions() -> usize {
    32
}

fn default_cols() -> u16 {
    120
}

fn default_rows() -> u16 {
    40
}

fn default_target_fps() -> u32 {
    60
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "pretty".to_string()
}

fn default_log_buffer_size() -> usize {
    50
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

// ============================================================================
// Configuration loading
// ============================================================================

impl AppConfig {
    /// Load configuration with fallback chain:
    /// 1. Project-local .agterm/config.toml
    /// 2. User config ~/.config/agterm/config.toml
    /// 3. Embedded default_config.toml
    pub fn load() -> Result<Self, ConfigError> {
        // Start with default config
        let mut config: AppConfig = toml::from_str(DEFAULT_CONFIG)
            .map_err(|e| ConfigError::ParseError(format!("Failed to parse default config: {}", e)))?;

        // Try to load user config
        if let Some(user_config_path) = Self::user_config_path() {
            if user_config_path.exists() {
                match Self::load_from_file(&user_config_path) {
                    Ok(user_config) => {
                        config = Self::merge(config, user_config);
                        tracing::info!("Loaded user config from {:?}", user_config_path);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load user config: {}", e);
                    }
                }
            }
        }

        // Try to load project-local config
        if let Some(project_config_path) = Self::project_config_path() {
            if project_config_path.exists() {
                match Self::load_from_file(&project_config_path) {
                    Ok(project_config) => {
                        config = Self::merge(config, project_config);
                        tracing::info!("Loaded project config from {:?}", project_config_path);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load project config: {}", e);
                    }
                }
            }
        }

        Ok(config)
    }

    /// Load configuration from a specific file
    pub fn load_from_file(path: &PathBuf) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        toml::from_str(&contents)
            .map_err(|e| ConfigError::ParseError(format!("Failed to parse {}: {}", path.display(), e)))
    }

    /// Get the user config path (~/.config/agterm/config.toml)
    pub fn user_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|config_dir| config_dir.join("agterm").join("config.toml"))
    }

    /// Get the project-local config path (./.agterm/config.toml)
    pub fn project_config_path() -> Option<PathBuf> {
        std::env::current_dir()
            .ok()
            .map(|cwd| cwd.join(".agterm").join("config.toml"))
    }

    /// Merge two configs (overlay takes precedence)
    fn merge(_base: Self, overlay: Self) -> Self {
        // For now, just return overlay (in future, implement deep merge)
        // TODO: Implement proper deep merge for nested structures
        Self {
            general: overlay.general,
            appearance: overlay.appearance,
            terminal: overlay.terminal,
            keybindings: overlay.keybindings,
            shell: overlay.shell,
            mouse: overlay.mouse,
            pty: overlay.pty,
            tui: overlay.tui,
            logging: overlay.logging,
            debug: overlay.debug,
        }
    }

    /// Save configuration to user config path
    #[allow(dead_code)]
    pub fn save(&self) -> Result<(), ConfigError> {
        let config_path = Self::user_config_path()
            .ok_or_else(|| ConfigError::IoError("Could not determine user config directory".to_string()))?;

        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ConfigError::IoError(e.to_string()))?;
        }

        let toml_string = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;

        std::fs::write(&config_path, toml_string)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        toml::from_str(DEFAULT_CONFIG).unwrap_or_else(|_| Self {
            general: GeneralConfig::default(),
            appearance: AppearanceConfig::default(),
            terminal: TerminalConfig::default(),
            keybindings: KeybindingsConfig::default(),
            shell: ShellConfig::default(),
            mouse: MouseConfig::default(),
            pty: PtyConfig::default(),
            tui: TuiConfig::default(),
            logging: LoggingConfig::default(),
            debug: DebugConfig::default(),
        })
    }
}

// ============================================================================
// Error handling
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Serialize error: {0}")]
    #[allow(dead_code)]
    SerializeError(String),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_loads() {
        let config = AppConfig::default();
        assert_eq!(config.general.app_name, "agterm");
        assert_eq!(config.appearance.font_family, "D2Coding");
        assert_eq!(config.appearance.font_size, 14.0);
        assert_eq!(config.terminal.scrollback_lines, 10000);
        assert_eq!(config.pty.default_cols, 120);
        assert_eq!(config.pty.default_rows, 40);
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let toml_string = toml::to_string(&config).unwrap();
        let parsed: AppConfig = toml::from_str(&toml_string).unwrap();

        assert_eq!(parsed.general.app_name, config.general.app_name);
        assert_eq!(parsed.appearance.font_size, config.appearance.font_size);
    }

    #[test]
    fn test_embedded_default_config_is_valid() {
        let result: Result<AppConfig, _> = toml::from_str(DEFAULT_CONFIG);
        assert!(result.is_ok(), "Default config should be valid TOML");
    }

    #[test]
    fn test_cursor_style_serde() {
        #[derive(Deserialize)]
        struct Wrapper { v: CursorStyle }

        assert_eq!(
            toml::from_str::<Wrapper>("v = \"block\"").unwrap().v,
            CursorStyle::Block
        );
        assert_eq!(
            toml::from_str::<Wrapper>("v = \"underline\"").unwrap().v,
            CursorStyle::Underline
        );
        assert_eq!(
            toml::from_str::<Wrapper>("v = \"beam\"").unwrap().v,
            CursorStyle::Beam
        );
    }

    #[test]
    fn test_bell_style_serde() {
        #[derive(Deserialize)]
        struct Wrapper { v: BellStyle }

        assert_eq!(
            toml::from_str::<Wrapper>("v = \"visual\"").unwrap().v,
            BellStyle::Visual
        );
        assert_eq!(
            toml::from_str::<Wrapper>("v = \"sound\"").unwrap().v,
            BellStyle::Sound
        );
        assert_eq!(
            toml::from_str::<Wrapper>("v = \"both\"").unwrap().v,
            BellStyle::Both
        );
        assert_eq!(
            toml::from_str::<Wrapper>("v = \"none\"").unwrap().v,
            BellStyle::None
        );
    }
}
