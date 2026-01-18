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
    #[serde(default)]
    pub session: SessionConfig,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            app_name: default_app_name(),
            default_shell: None,
            default_working_dir: None,
            session: SessionConfig::default(),
        }
    }
}

/// Session management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    #[serde(default = "default_true")]
    pub restore_on_startup: bool,
    #[serde(default = "default_true")]
    pub save_on_exit: bool,
    #[serde(default)]
    pub session_file: Option<PathBuf>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            restore_on_startup: true,
            save_on_exit: true,
            session_file: None,
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    #[serde(default = "default_bell_volume")]
    pub bell_volume: f32,
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
            bell_volume: default_bell_volume(),
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
    #[serde(default)]
    pub keyboard: KeyboardConfig,
}

/// Keyboard repeat configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardConfig {
    #[serde(default = "default_repeat_delay")]
    pub repeat_delay_ms: u64,
    #[serde(default = "default_repeat_rate")]
    pub repeat_rate_ms: u64,
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            mode: default_keybinding_mode(),
            custom: HashMap::new(),
            keyboard: KeyboardConfig::default(),
        }
    }
}

impl Default for KeyboardConfig {
    fn default() -> Self {
        Self {
            repeat_delay_ms: default_repeat_delay(),
            repeat_rate_ms: default_repeat_rate(),
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

fn default_bell_volume() -> f32 {
    0.5 // 50% volume
}

fn default_keybinding_mode() -> String {
    "default".to_string()
}

fn default_repeat_delay() -> u64 {
    500 // 500ms initial delay before repeat
}

fn default_repeat_rate() -> u64 {
    30 // 30ms between repeats (approximately 33 keys per second)
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
        let mut config: AppConfig = toml::from_str(DEFAULT_CONFIG).map_err(|e| {
            ConfigError::ParseError(format!("Failed to parse default config: {}", e))
        })?;

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
        let contents =
            std::fs::read_to_string(path).map_err(|e| ConfigError::IoError(e.to_string()))?;

        toml::from_str(&contents).map_err(|e| {
            ConfigError::ParseError(format!("Failed to parse {}: {}", path.display(), e))
        })
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

    /// Get the session file path (defaults to ~/.config/agterm/session.json)
    pub fn session_file_path(&self) -> PathBuf {
        self.general
            .session
            .session_file
            .clone()
            .or_else(|| {
                dirs::config_dir().map(|config_dir| config_dir.join("agterm").join("session.json"))
            })
            .unwrap_or_else(|| PathBuf::from("session.json"))
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
        let config_path = Self::user_config_path().ok_or_else(|| {
            ConfigError::IoError("Could not determine user config directory".to_string())
        })?;

        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ConfigError::IoError(e.to_string()))?;
        }

        let toml_string =
            toml::to_string_pretty(self).map_err(|e| ConfigError::SerializeError(e.to_string()))?;

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
    SerializeError(String),
}

// ============================================================================
// Profile System
// ============================================================================

/// Terminal profile with custom settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Profile name
    pub name: String,
    /// Shell program (e.g., "zsh", "bash", "/bin/fish")
    pub shell: Option<String>,
    /// Shell arguments
    #[serde(default)]
    pub shell_args: Vec<String>,
    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Color theme name
    #[serde(default)]
    pub theme: Option<String>,
    /// Font size
    #[serde(default)]
    pub font_size: Option<f32>,
    /// Working directory
    #[serde(default)]
    pub working_dir: Option<PathBuf>,
    /// Custom color scheme (overrides theme)
    #[serde(default)]
    pub color_scheme: Option<ColorScheme>,
}

impl Profile {
    /// Create a new profile with default settings
    pub fn new(name: String) -> Self {
        Self {
            name,
            shell: None,
            shell_args: Vec::new(),
            env: HashMap::new(),
            theme: None,
            font_size: None,
            working_dir: None,
            color_scheme: None,
        }
    }

    /// Get the profiles directory path (~/.config/agterm/profiles/)
    pub fn profiles_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|config_dir| config_dir.join("agterm").join("profiles"))
    }

    /// Get the path for a profile file
    pub fn profile_path(name: &str) -> Option<PathBuf> {
        Self::profiles_dir().map(|dir| dir.join(format!("{}.toml", name)))
    }

    /// Load a profile by name
    pub fn load(name: &str) -> Result<Self, ConfigError> {
        let path = Self::profile_path(name).ok_or_else(|| {
            ConfigError::IoError("Could not determine profile directory".to_string())
        })?;

        if !path.exists() {
            return Err(ConfigError::IoError(format!(
                "Profile '{}' not found",
                name
            )));
        }

        let contents =
            std::fs::read_to_string(&path).map_err(|e| ConfigError::IoError(e.to_string()))?;

        toml::from_str(&contents).map_err(|e| {
            ConfigError::ParseError(format!("Failed to parse profile '{}': {}", name, e))
        })
    }

    /// Save this profile to disk
    pub fn save(&self) -> Result<(), ConfigError> {
        let profiles_dir = Self::profiles_dir().ok_or_else(|| {
            ConfigError::IoError("Could not determine profile directory".to_string())
        })?;

        // Create profiles directory if it doesn't exist
        std::fs::create_dir_all(&profiles_dir).map_err(|e| ConfigError::IoError(e.to_string()))?;

        let path = profiles_dir.join(format!("{}.toml", self.name));
        let toml_string =
            toml::to_string_pretty(self).map_err(|e| ConfigError::SerializeError(e.to_string()))?;

        std::fs::write(&path, toml_string).map_err(|e| ConfigError::IoError(e.to_string()))?;

        tracing::info!("Saved profile '{}' to {:?}", self.name, path);
        Ok(())
    }

    /// Delete a profile by name
    pub fn delete(name: &str) -> Result<(), ConfigError> {
        let path = Self::profile_path(name).ok_or_else(|| {
            ConfigError::IoError("Could not determine profile directory".to_string())
        })?;

        if !path.exists() {
            return Err(ConfigError::IoError(format!(
                "Profile '{}' not found",
                name
            )));
        }

        std::fs::remove_file(&path).map_err(|e| ConfigError::IoError(e.to_string()))?;

        tracing::info!("Deleted profile '{}'", name);
        Ok(())
    }

    /// List all available profiles
    pub fn list() -> Result<Vec<String>, ConfigError> {
        let profiles_dir = Self::profiles_dir().ok_or_else(|| {
            ConfigError::IoError("Could not determine profile directory".to_string())
        })?;

        if !profiles_dir.exists() {
            return Ok(Vec::new());
        }

        let entries =
            std::fs::read_dir(&profiles_dir).map_err(|e| ConfigError::IoError(e.to_string()))?;

        let mut profiles = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| ConfigError::IoError(e.to_string()))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    profiles.push(stem.to_string());
                }
            }
        }

        profiles.sort();
        Ok(profiles)
    }

    /// Create default profiles (default, zsh, bash)
    pub fn create_default_profiles() -> Result<(), ConfigError> {
        // Default profile
        let default = Profile {
            name: "default".to_string(),
            shell: None, // Use system default
            shell_args: Vec::new(),
            env: HashMap::new(),
            theme: Some("default".to_string()),
            font_size: Some(14.0),
            working_dir: None,
            color_scheme: None,
        };
        default.save()?;

        // Zsh profile
        let mut zsh_env = HashMap::new();
        zsh_env.insert("SHELL".to_string(), "/bin/zsh".to_string());
        let zsh = Profile {
            name: "zsh".to_string(),
            shell: Some("/bin/zsh".to_string()),
            shell_args: vec!["-l".to_string()], // Login shell
            env: zsh_env,
            theme: Some("default".to_string()),
            font_size: Some(14.0),
            working_dir: None,
            color_scheme: None,
        };
        zsh.save()?;

        // Bash profile
        let mut bash_env = HashMap::new();
        bash_env.insert("SHELL".to_string(), "/bin/bash".to_string());
        let bash = Profile {
            name: "bash".to_string(),
            shell: Some("/bin/bash".to_string()),
            shell_args: vec!["-l".to_string()], // Login shell
            env: bash_env,
            theme: Some("default".to_string()),
            font_size: Some(14.0),
            working_dir: None,
            color_scheme: None,
        };
        bash.save()?;

        tracing::info!("Created default profiles: default, zsh, bash");
        Ok(())
    }

    /// Apply this profile's settings to the given AppConfig
    pub fn apply_to_config(&self, config: &mut AppConfig) {
        // Apply shell settings
        if let Some(shell) = &self.shell {
            config.shell.program = Some(shell.clone());
        }
        if !self.shell_args.is_empty() {
            config.shell.args = self.shell_args.clone();
        }
        if !self.env.is_empty() {
            config.shell.env.extend(self.env.clone());
        }

        // Apply appearance settings
        if let Some(theme) = &self.theme {
            config.appearance.theme = theme.clone();
        }
        if let Some(font_size) = self.font_size {
            config.appearance.font_size = font_size;
        }
        if let Some(color_scheme) = &self.color_scheme {
            config.appearance.color_scheme = Some(color_scheme.clone());
        }

        // Apply working directory
        if let Some(working_dir) = &self.working_dir {
            config.general.default_working_dir = Some(working_dir.clone());
        }
    }
}

// ============================================================================
// Snippet System
// ============================================================================

/// A snippet/macro that can be triggered to insert text
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Snippet {
    /// Snippet name (for display/search)
    pub name: String,
    /// Trigger string (e.g., "/git" triggers git status)
    pub trigger: String,
    /// Content to insert when triggered
    pub content: String,
    /// Category for organization (e.g., "git", "docker", "custom")
    pub category: String,
}

impl Snippet {
    /// Create a new snippet
    pub fn new(name: String, trigger: String, content: String, category: String) -> Self {
        Self {
            name,
            trigger,
            content,
            category,
        }
    }

    /// Get the snippets directory path (~/.config/agterm/)
    pub fn snippets_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|config_dir| config_dir.join("agterm"))
    }

    /// Get the snippets file path (~/.config/agterm/snippets.toml)
    pub fn snippets_file_path() -> Option<PathBuf> {
        Self::snippets_dir().map(|dir| dir.join("snippets.toml"))
    }

    /// Load snippets from file
    pub fn load_from_file() -> Result<Vec<Snippet>, ConfigError> {
        let path = Self::snippets_file_path().ok_or_else(|| {
            ConfigError::IoError("Could not determine snippets directory".to_string())
        })?;

        if !path.exists() {
            // Return default snippets if file doesn't exist
            return Ok(Self::default_snippets());
        }

        let contents =
            std::fs::read_to_string(&path).map_err(|e| ConfigError::IoError(e.to_string()))?;

        #[derive(Deserialize)]
        struct SnippetsFile {
            snippets: Vec<Snippet>,
        }

        let file: SnippetsFile = toml::from_str(&contents).map_err(|e| {
            ConfigError::ParseError(format!("Failed to parse snippets.toml: {}", e))
        })?;

        Ok(file.snippets)
    }

    /// Save snippets to file
    pub fn save_to_file(snippets: &[Snippet]) -> Result<(), ConfigError> {
        let snippets_dir = Self::snippets_dir().ok_or_else(|| {
            ConfigError::IoError("Could not determine snippets directory".to_string())
        })?;

        // Create config directory if it doesn't exist
        std::fs::create_dir_all(&snippets_dir).map_err(|e| ConfigError::IoError(e.to_string()))?;

        let path = snippets_dir.join("snippets.toml");

        #[derive(Serialize)]
        struct SnippetsFile<'a> {
            snippets: &'a [Snippet],
        }

        let file = SnippetsFile { snippets };
        let toml_string = toml::to_string_pretty(&file)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;

        std::fs::write(&path, toml_string).map_err(|e| ConfigError::IoError(e.to_string()))?;

        tracing::info!("Saved {} snippets to {:?}", snippets.len(), path);
        Ok(())
    }

    /// Get default snippets (git, docker, etc.)
    pub fn default_snippets() -> Vec<Snippet> {
        vec![
            // Git snippets
            Snippet::new(
                "Git Status".to_string(),
                "/gs".to_string(),
                "git status".to_string(),
                "git".to_string(),
            ),
            Snippet::new(
                "Git Add All".to_string(),
                "/ga".to_string(),
                "git add .".to_string(),
                "git".to_string(),
            ),
            Snippet::new(
                "Git Commit".to_string(),
                "/gc".to_string(),
                "git commit -m \"".to_string(),
                "git".to_string(),
            ),
            Snippet::new(
                "Git Push".to_string(),
                "/gp".to_string(),
                "git push".to_string(),
                "git".to_string(),
            ),
            Snippet::new(
                "Git Pull".to_string(),
                "/gpl".to_string(),
                "git pull".to_string(),
                "git".to_string(),
            ),
            Snippet::new(
                "Git Log".to_string(),
                "/gl".to_string(),
                "git log --oneline -10".to_string(),
                "git".to_string(),
            ),
            Snippet::new(
                "Git Diff".to_string(),
                "/gd".to_string(),
                "git diff".to_string(),
                "git".to_string(),
            ),
            Snippet::new(
                "Git Branch".to_string(),
                "/gb".to_string(),
                "git branch".to_string(),
                "git".to_string(),
            ),
            Snippet::new(
                "Git Checkout".to_string(),
                "/gco".to_string(),
                "git checkout ".to_string(),
                "git".to_string(),
            ),
            // Docker snippets
            Snippet::new(
                "Docker PS".to_string(),
                "/dps".to_string(),
                "docker ps".to_string(),
                "docker".to_string(),
            ),
            Snippet::new(
                "Docker Images".to_string(),
                "/di".to_string(),
                "docker images".to_string(),
                "docker".to_string(),
            ),
            Snippet::new(
                "Docker Compose Up".to_string(),
                "/dcu".to_string(),
                "docker-compose up -d".to_string(),
                "docker".to_string(),
            ),
            Snippet::new(
                "Docker Compose Down".to_string(),
                "/dcd".to_string(),
                "docker-compose down".to_string(),
                "docker".to_string(),
            ),
            Snippet::new(
                "Docker Logs".to_string(),
                "/dlogs".to_string(),
                "docker logs -f ".to_string(),
                "docker".to_string(),
            ),
            // Common commands
            Snippet::new(
                "List Files Long".to_string(),
                "/ll".to_string(),
                "ls -lah".to_string(),
                "common".to_string(),
            ),
            Snippet::new(
                "Find File".to_string(),
                "/ff".to_string(),
                "find . -name ".to_string(),
                "common".to_string(),
            ),
            Snippet::new(
                "Grep Recursive".to_string(),
                "/gr".to_string(),
                "grep -r \"\" .".to_string(),
                "common".to_string(),
            ),
            // Kubernetes snippets
            Snippet::new(
                "Kubectl Get Pods".to_string(),
                "/kgp".to_string(),
                "kubectl get pods".to_string(),
                "kubernetes".to_string(),
            ),
            Snippet::new(
                "Kubectl Describe".to_string(),
                "/kdesc".to_string(),
                "kubectl describe pod ".to_string(),
                "kubernetes".to_string(),
            ),
            Snippet::new(
                "Kubectl Logs".to_string(),
                "/klogs".to_string(),
                "kubectl logs -f ".to_string(),
                "kubernetes".to_string(),
            ),
            // Cargo (Rust) snippets
            Snippet::new(
                "Cargo Build".to_string(),
                "/cb".to_string(),
                "cargo build".to_string(),
                "cargo".to_string(),
            ),
            Snippet::new(
                "Cargo Run".to_string(),
                "/cr".to_string(),
                "cargo run".to_string(),
                "cargo".to_string(),
            ),
            Snippet::new(
                "Cargo Test".to_string(),
                "/ct".to_string(),
                "cargo test".to_string(),
                "cargo".to_string(),
            ),
            Snippet::new(
                "Cargo Check".to_string(),
                "/cc".to_string(),
                "cargo check".to_string(),
                "cargo".to_string(),
            ),
        ]
    }

    /// Initialize snippets file with defaults if it doesn't exist
    pub fn initialize_default_file() -> Result<(), ConfigError> {
        let path = Self::snippets_file_path().ok_or_else(|| {
            ConfigError::IoError("Could not determine snippets directory".to_string())
        })?;

        if !path.exists() {
            let default_snippets = Self::default_snippets();
            Self::save_to_file(&default_snippets)?;
            tracing::info!("Created default snippets file at {:?}", path);
        }

        Ok(())
    }

    /// Find snippet by trigger
    pub fn find_by_trigger<'a>(snippets: &'a [Snippet], trigger: &str) -> Option<&'a Snippet> {
        snippets.iter().find(|s| s.trigger == trigger)
    }

    /// Find snippets by category
    pub fn find_by_category<'a>(snippets: &'a [Snippet], category: &str) -> Vec<&'a Snippet> {
        snippets.iter().filter(|s| s.category == category).collect()
    }

    /// Get all unique categories
    pub fn get_categories(snippets: &[Snippet]) -> Vec<String> {
        let mut categories: Vec<String> = snippets.iter().map(|s| s.category.clone()).collect();
        categories.sort();
        categories.dedup();
        categories
    }
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
        struct Wrapper {
            v: CursorStyle,
        }

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
        struct Wrapper {
            v: BellStyle,
        }

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

    // ========== Profile System Tests ==========

    #[test]
    fn test_profile_creation() {
        let profile = Profile::new("test".to_string());
        assert_eq!(profile.name, "test");
        assert_eq!(profile.shell, None);
        assert!(profile.shell_args.is_empty());
        assert!(profile.env.is_empty());
        assert_eq!(profile.theme, None);
        assert_eq!(profile.font_size, None);
        assert_eq!(profile.working_dir, None);
        assert_eq!(profile.color_scheme, None);
    }

    #[test]
    fn test_profile_serialization() {
        let mut env = HashMap::new();
        env.insert("TEST_VAR".to_string(), "test_value".to_string());

        let profile = Profile {
            name: "test_profile".to_string(),
            shell: Some("/bin/zsh".to_string()),
            shell_args: vec!["-l".to_string()],
            env: env.clone(),
            theme: Some("dark".to_string()),
            font_size: Some(16.0),
            working_dir: Some(PathBuf::from("/home/user")),
            color_scheme: None,
        };

        let toml_string = toml::to_string(&profile).unwrap();
        let parsed: Profile = toml::from_str(&toml_string).unwrap();

        assert_eq!(parsed.name, profile.name);
        assert_eq!(parsed.shell, profile.shell);
        assert_eq!(parsed.shell_args, profile.shell_args);
        assert_eq!(parsed.env.get("TEST_VAR"), Some(&"test_value".to_string()));
        assert_eq!(parsed.theme, profile.theme);
        assert_eq!(parsed.font_size, profile.font_size);
        assert_eq!(parsed.working_dir, profile.working_dir);
    }

    #[test]
    fn test_profile_save_and_load() {
        use tempfile::tempdir;

        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let profiles_dir = temp_dir.path().join("agterm").join("profiles");
        std::fs::create_dir_all(&profiles_dir).unwrap();

        // Create a test profile
        let mut env = HashMap::new();
        env.insert("CUSTOM_VAR".to_string(), "custom_value".to_string());

        let profile = Profile {
            name: "test_save_load".to_string(),
            shell: Some("/bin/bash".to_string()),
            shell_args: vec!["-i".to_string()],
            env: env.clone(),
            theme: Some("light".to_string()),
            font_size: Some(12.0),
            working_dir: Some(PathBuf::from("/tmp")),
            color_scheme: None,
        };

        // Save to file
        let profile_path = profiles_dir.join("test_save_load.toml");
        let toml_string = toml::to_string_pretty(&profile).unwrap();
        std::fs::write(&profile_path, toml_string).unwrap();

        // Load from file
        let contents = std::fs::read_to_string(&profile_path).unwrap();
        let loaded_profile: Profile = toml::from_str(&contents).unwrap();

        // Verify loaded profile matches original
        assert_eq!(loaded_profile.name, profile.name);
        assert_eq!(loaded_profile.shell, profile.shell);
        assert_eq!(loaded_profile.shell_args, profile.shell_args);
        assert_eq!(loaded_profile.env, profile.env);
        assert_eq!(loaded_profile.theme, profile.theme);
        assert_eq!(loaded_profile.font_size, profile.font_size);
        assert_eq!(loaded_profile.working_dir, profile.working_dir);
    }

    #[test]
    fn test_profile_apply_to_config() {
        let mut config = AppConfig::default();
        let original_font_size = config.appearance.font_size;

        // Create a profile with custom settings
        let mut env = HashMap::new();
        env.insert("PROFILE_VAR".to_string(), "value".to_string());

        let profile = Profile {
            name: "test_apply".to_string(),
            shell: Some("/bin/fish".to_string()),
            shell_args: vec!["--login".to_string()],
            env: env.clone(),
            theme: Some("custom_theme".to_string()),
            font_size: Some(18.0),
            working_dir: Some(PathBuf::from("/workspace")),
            color_scheme: None,
        };

        // Apply profile to config
        profile.apply_to_config(&mut config);

        // Verify settings were applied
        assert_eq!(config.shell.program, Some("/bin/fish".to_string()));
        assert_eq!(config.shell.args, vec!["--login".to_string()]);
        assert_eq!(
            config.shell.env.get("PROFILE_VAR"),
            Some(&"value".to_string())
        );
        assert_eq!(config.appearance.theme, "custom_theme");
        assert_eq!(config.appearance.font_size, 18.0);
        assert_ne!(config.appearance.font_size, original_font_size);
        assert_eq!(
            config.general.default_working_dir,
            Some(PathBuf::from("/workspace"))
        );
    }

    #[test]
    fn test_profile_partial_application() {
        let mut config = AppConfig::default();
        let original_shell = config.shell.program.clone();
        let original_theme = config.appearance.theme.clone();

        // Create a profile with only some settings
        let profile = Profile {
            name: "partial".to_string(),
            shell: None, // Don't override shell
            shell_args: Vec::new(),
            env: HashMap::new(),
            theme: Some("new_theme".to_string()),
            font_size: None, // Don't override font size
            working_dir: None,
            color_scheme: None,
        };

        profile.apply_to_config(&mut config);

        // Only theme should be updated
        assert_eq!(config.shell.program, original_shell);
        assert_ne!(config.appearance.theme, original_theme);
        assert_eq!(config.appearance.theme, "new_theme");
    }

    #[test]
    fn test_profile_with_color_scheme() {
        let color_scheme = ColorScheme {
            background: "#1e1e2e".to_string(),
            foreground: "#cdd6f4".to_string(),
            cursor: "#f5e0dc".to_string(),
            selection: Some("#585b70".to_string()),
            black: Some("#45475a".to_string()),
            red: Some("#f38ba8".to_string()),
            green: Some("#a6e3a1".to_string()),
            yellow: Some("#f9e2af".to_string()),
            blue: Some("#89b4fa".to_string()),
            magenta: Some("#f5c2e7".to_string()),
            cyan: Some("#94e2d5".to_string()),
            white: Some("#bac2de".to_string()),
            bright_black: Some("#585b70".to_string()),
            bright_red: Some("#f38ba8".to_string()),
            bright_green: Some("#a6e3a1".to_string()),
            bright_yellow: Some("#f9e2af".to_string()),
            bright_blue: Some("#89b4fa".to_string()),
            bright_magenta: Some("#f5c2e7".to_string()),
            bright_cyan: Some("#94e2d5".to_string()),
            bright_white: Some("#a6adc8".to_string()),
        };

        let profile = Profile {
            name: "catppuccin".to_string(),
            shell: None,
            shell_args: Vec::new(),
            env: HashMap::new(),
            theme: Some("catppuccin".to_string()),
            font_size: Some(14.0),
            working_dir: None,
            color_scheme: Some(color_scheme),
        };

        // Serialize and deserialize
        let toml_string = toml::to_string_pretty(&profile).unwrap();
        let parsed: Profile = toml::from_str(&toml_string).unwrap();

        assert_eq!(parsed.name, profile.name);
        assert!(parsed.color_scheme.is_some());
        let parsed_scheme = parsed.color_scheme.unwrap();
        assert_eq!(parsed_scheme.background, "#1e1e2e");
        assert_eq!(parsed_scheme.foreground, "#cdd6f4");
    }

    #[test]
    fn test_default_profiles_structure() {
        // Test that default profiles have expected structure
        let default_profile = Profile {
            name: "default".to_string(),
            shell: None,
            shell_args: Vec::new(),
            env: HashMap::new(),
            theme: Some("default".to_string()),
            font_size: Some(14.0),
            working_dir: None,
            color_scheme: None,
        };

        assert_eq!(default_profile.name, "default");
        assert_eq!(default_profile.shell, None);
        assert_eq!(default_profile.font_size, Some(14.0));

        let mut zsh_env = HashMap::new();
        zsh_env.insert("SHELL".to_string(), "/bin/zsh".to_string());
        let zsh_profile = Profile {
            name: "zsh".to_string(),
            shell: Some("/bin/zsh".to_string()),
            shell_args: vec!["-l".to_string()],
            env: zsh_env,
            theme: Some("default".to_string()),
            font_size: Some(14.0),
            working_dir: None,
            color_scheme: None,
        };

        assert_eq!(zsh_profile.name, "zsh");
        assert_eq!(zsh_profile.shell, Some("/bin/zsh".to_string()));
        assert_eq!(zsh_profile.shell_args, vec!["-l".to_string()]);
    }

    #[test]
    fn test_profile_paths() {
        // Test profile path generation
        if let Some(profiles_dir) = Profile::profiles_dir() {
            assert!(profiles_dir.to_string_lossy().contains("agterm/profiles"));
        }

        if let Some(profile_path) = Profile::profile_path("test") {
            assert!(profile_path.to_string_lossy().ends_with("test.toml"));
        }
    }

    // ========== Snippet System Tests ==========

    #[test]
    fn test_snippet_creation() {
        let snippet = Snippet::new(
            "Test Snippet".to_string(),
            "/test".to_string(),
            "echo test".to_string(),
            "testing".to_string(),
        );

        assert_eq!(snippet.name, "Test Snippet");
        assert_eq!(snippet.trigger, "/test");
        assert_eq!(snippet.content, "echo test");
        assert_eq!(snippet.category, "testing");
    }

    #[test]
    fn test_default_snippets() {
        let snippets = Snippet::default_snippets();

        // Should have multiple categories
        assert!(snippets.len() > 20);

        // Check git snippets exist
        let git_snippets: Vec<_> = snippets.iter().filter(|s| s.category == "git").collect();
        assert!(!git_snippets.is_empty());

        // Check docker snippets exist
        let docker_snippets: Vec<_> = snippets.iter().filter(|s| s.category == "docker").collect();
        assert!(!docker_snippets.is_empty());

        // Check cargo snippets exist
        let cargo_snippets: Vec<_> = snippets.iter().filter(|s| s.category == "cargo").collect();
        assert!(!cargo_snippets.is_empty());
    }

    #[test]
    fn test_snippet_find_by_trigger() {
        let snippets = Snippet::default_snippets();

        // Test finding git status
        let result = Snippet::find_by_trigger(&snippets, "/gs");
        assert!(result.is_some());
        assert_eq!(result.unwrap().content, "git status");

        // Test finding non-existent trigger
        let result = Snippet::find_by_trigger(&snippets, "/nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_snippet_find_by_category() {
        let snippets = Snippet::default_snippets();

        // Find all git snippets
        let git_snippets = Snippet::find_by_category(&snippets, "git");
        assert!(!git_snippets.is_empty());

        // All returned snippets should be in git category
        for snippet in git_snippets {
            assert_eq!(snippet.category, "git");
        }

        // Find all docker snippets
        let docker_snippets = Snippet::find_by_category(&snippets, "docker");
        assert!(!docker_snippets.is_empty());

        // Find non-existent category
        let empty = Snippet::find_by_category(&snippets, "nonexistent");
        assert!(empty.is_empty());
    }

    #[test]
    fn test_snippet_get_categories() {
        let snippets = Snippet::default_snippets();
        let categories = Snippet::get_categories(&snippets);

        // Should have multiple categories
        assert!(categories.len() > 3);

        // Should contain expected categories
        assert!(categories.contains(&"git".to_string()));
        assert!(categories.contains(&"docker".to_string()));
        assert!(categories.contains(&"cargo".to_string()));
        assert!(categories.contains(&"common".to_string()));

        // Categories should be sorted and unique
        let mut sorted_categories = categories.clone();
        sorted_categories.sort();
        assert_eq!(categories, sorted_categories);
    }

    #[test]
    fn test_snippet_serialization() {
        let snippet = Snippet::new(
            "Test".to_string(),
            "/test".to_string(),
            "echo hello".to_string(),
            "testing".to_string(),
        );

        // Serialize to TOML
        let toml_str = toml::to_string(&snippet).unwrap();

        // Deserialize back
        let deserialized: Snippet = toml::from_str(&toml_str).unwrap();

        // Should match original
        assert_eq!(snippet, deserialized);
    }

    #[test]
    fn test_snippet_save_and_load() {
        use tempfile::tempdir;

        // Create temporary directory
        let temp_dir = tempdir().unwrap();
        let snippets_file = temp_dir.path().join("snippets.toml");

        // Create test snippets
        let snippets = vec![
            Snippet::new(
                "Test 1".to_string(),
                "/t1".to_string(),
                "echo test1".to_string(),
                "test".to_string(),
            ),
            Snippet::new(
                "Test 2".to_string(),
                "/t2".to_string(),
                "echo test2".to_string(),
                "test".to_string(),
            ),
        ];

        // Save to file
        #[derive(Serialize)]
        struct SnippetsFile<'a> {
            snippets: &'a [Snippet],
        }
        let file = SnippetsFile {
            snippets: &snippets,
        };
        let toml_string = toml::to_string_pretty(&file).unwrap();
        std::fs::write(&snippets_file, toml_string).unwrap();

        // Load from file
        let contents = std::fs::read_to_string(&snippets_file).unwrap();

        #[derive(Deserialize)]
        struct SnippetsFileLoad {
            snippets: Vec<Snippet>,
        }
        let loaded: SnippetsFileLoad = toml::from_str(&contents).unwrap();

        // Verify loaded snippets match original
        assert_eq!(loaded.snippets.len(), 2);
        assert_eq!(loaded.snippets[0].name, "Test 1");
        assert_eq!(loaded.snippets[1].name, "Test 2");
    }

    #[test]
    fn test_snippet_paths() {
        // Test snippets directory path
        if let Some(snippets_dir) = Snippet::snippets_dir() {
            assert!(snippets_dir.to_string_lossy().contains("agterm"));
        }

        // Test snippets file path
        if let Some(snippets_file) = Snippet::snippets_file_path() {
            assert!(snippets_file.to_string_lossy().ends_with("snippets.toml"));
            assert!(snippets_file.to_string_lossy().contains("agterm"));
        }
    }

    #[test]
    fn test_snippet_triggers_unique() {
        let snippets = Snippet::default_snippets();
        let mut triggers = std::collections::HashSet::new();

        // All triggers should be unique
        for snippet in &snippets {
            assert!(
                triggers.insert(&snippet.trigger),
                "Duplicate trigger found: {}",
                snippet.trigger
            );
        }
    }

    #[test]
    fn test_snippet_git_commands() {
        let snippets = Snippet::default_snippets();

        // Test specific git commands
        let gs = Snippet::find_by_trigger(&snippets, "/gs").unwrap();
        assert_eq!(gs.content, "git status");

        let ga = Snippet::find_by_trigger(&snippets, "/ga").unwrap();
        assert_eq!(ga.content, "git add .");

        let gp = Snippet::find_by_trigger(&snippets, "/gp").unwrap();
        assert_eq!(gp.content, "git push");

        let gl = Snippet::find_by_trigger(&snippets, "/gl").unwrap();
        assert_eq!(gl.content, "git log --oneline -10");
    }

    #[test]
    fn test_snippet_docker_commands() {
        let snippets = Snippet::default_snippets();

        // Test specific docker commands
        let dps = Snippet::find_by_trigger(&snippets, "/dps").unwrap();
        assert_eq!(dps.content, "docker ps");

        let di = Snippet::find_by_trigger(&snippets, "/di").unwrap();
        assert_eq!(di.content, "docker images");

        let dcu = Snippet::find_by_trigger(&snippets, "/dcu").unwrap();
        assert_eq!(dcu.content, "docker-compose up -d");
    }

    #[test]
    fn test_snippet_cargo_commands() {
        let snippets = Snippet::default_snippets();

        // Test specific cargo commands
        let cb = Snippet::find_by_trigger(&snippets, "/cb").unwrap();
        assert_eq!(cb.content, "cargo build");

        let cr = Snippet::find_by_trigger(&snippets, "/cr").unwrap();
        assert_eq!(cr.content, "cargo run");

        let ct = Snippet::find_by_trigger(&snippets, "/ct").unwrap();
        assert_eq!(ct.content, "cargo test");
    }

    #[test]
    fn test_snippet_common_commands() {
        let snippets = Snippet::default_snippets();

        // Test common commands
        let ll = Snippet::find_by_trigger(&snippets, "/ll").unwrap();
        assert_eq!(ll.content, "ls -lah");

        let ff = Snippet::find_by_trigger(&snippets, "/ff").unwrap();
        assert_eq!(ff.content, "find . -name ");
    }
}
