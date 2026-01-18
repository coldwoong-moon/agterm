//! Configuration management for AgTerm
//!
//! This module handles loading, parsing, and managing configuration from:
//! 1. Embedded default_config.toml (compile-time defaults)
//! 2. User config at ~/.config/agterm/config.toml (or platform-specific location)
//! 3. Project-local config at ./.agterm/config.toml

use regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Default configuration embedded in binary
const DEFAULT_CONFIG: &str = include_str!("../../default_config.toml");

/// Parse hex color string to RGBA components
/// Supports formats: #RRGGBB, #RRGGBBAA
pub fn parse_hex_color(hex: &str) -> Option<(f32, f32, f32, f32)> {
    let hex = hex.trim_start_matches('#');

    if hex.len() == 6 {
        // #RRGGBB format
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some((r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0))
    } else if hex.len() == 8 {
        // #RRGGBBAA format
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
        Some((
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        ))
    } else {
        None
    }
}

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
    pub environment: EnvironmentConfig,
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
    #[serde(default)]
    pub notification: NotificationConfig,
    #[serde(default)]
    pub status_bar: StatusBarConfig,
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
    /// Restore previous session on startup
    #[serde(default = "default_true")]
    pub restore_on_startup: bool,
    /// Save session on normal exit
    #[serde(default = "default_true")]
    pub save_on_exit: bool,
    /// Enable automatic saving at intervals
    #[serde(default = "default_true")]
    pub auto_save: bool,
    /// Auto-save interval in seconds
    #[serde(default = "default_auto_save_interval")]
    pub auto_save_interval_seconds: u64,
    /// Maximum number of backup files to keep
    #[serde(default = "default_max_backups")]
    pub max_backups: usize,
    /// Custom session file path (None = use default)
    #[serde(default)]
    pub session_file: Option<PathBuf>,
    /// Prompt user before restoring crashed session
    #[serde(default = "default_true")]
    pub prompt_on_recovery: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            restore_on_startup: true,
            save_on_exit: true,
            auto_save: true,
            auto_save_interval_seconds: default_auto_save_interval(),
            max_backups: default_max_backups(),
            session_file: None,
            prompt_on_recovery: true,
        }
    }
}

/// Font configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    #[serde(default = "default_font_family")]
    pub family: String,
    #[serde(default = "default_font_size")]
    pub size: f32,
    #[serde(default = "default_line_height")]
    pub line_height: f32,
    #[serde(default = "default_true")]
    pub bold_as_bright: bool,
    #[serde(default = "default_false")]
    pub use_thin_strokes: bool,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            family: default_font_family(),
            size: default_font_size(),
            line_height: default_line_height(),
            bold_as_bright: true,
            use_thin_strokes: false,
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
    #[serde(default)]
    pub font: FontConfig,
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
            font: FontConfig::default(),
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

/// Scrollback buffer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollbackConfig {
    /// Maximum number of lines to keep in scrollback (0 = unlimited)
    #[serde(default = "default_scrollback_lines")]
    pub max_lines: usize,
    /// Enable RLE compression for scrollback lines
    #[serde(default = "default_true")]
    pub compression: bool,
    /// Save scrollback to file on exit (future feature)
    #[serde(default = "default_false")]
    pub save_to_file: bool,
}

impl Default for ScrollbackConfig {
    fn default() -> Self {
        Self {
            max_lines: default_scrollback_lines(),
            compression: true,
            save_to_file: false,
        }
    }
}

/// Terminal behavior settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    #[serde(default = "default_scrollback_lines")]
    pub scrollback_lines: usize,
    #[serde(default)]
    pub scrollback: ScrollbackConfig,
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
    /// Enable visual flash effect for bell
    #[serde(default = "default_true")]
    pub visual_flash: bool,
    /// Flash overlay color (hex format: #RRGGBB or #RRGGBBAA)
    #[serde(default = "default_flash_color")]
    pub flash_color: String,
    /// Flash duration in milliseconds
    #[serde(default = "default_flash_duration")]
    pub flash_duration_ms: u64,
    #[serde(default = "default_true")]
    pub bracketed_paste: bool,
    #[serde(default = "default_true")]
    pub auto_scroll_on_output: bool,
    #[serde(default)]
    pub images: ImageConfig,
    #[serde(default)]
    pub bracket: BracketConfig,
    #[serde(default)]
    pub link: LinkConfig,
    #[serde(default)]
    pub title: TitleConfig,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            scrollback_lines: default_scrollback_lines(),
            scrollback: ScrollbackConfig::default(),
            cursor_style: default_cursor_style(),
            cursor_blink: true,
            cursor_blink_interval_ms: default_cursor_blink_interval(),
            bell_enabled: true,
            bell_style: default_bell_style(),
            bell_volume: default_bell_volume(),
            visual_flash: true,
            flash_color: default_flash_color(),
            flash_duration_ms: default_flash_duration(),
            bracketed_paste: true,
            auto_scroll_on_output: true,
            images: ImageConfig::default(),
            bracket: BracketConfig::default(),
            link: LinkConfig::default(),
            title: TitleConfig::default(),
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

/// Image display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    /// Enable image display support
    #[serde(default = "default_false")]
    pub enabled: bool,
    /// Maximum image size in bytes (default: 10MB)
    #[serde(default = "default_image_max_size")]
    pub max_size_bytes: usize,
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_size_bytes: default_image_max_size(),
        }
    }
}

/// Bracket matching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BracketConfig {
    /// Enable bracket matching highlighting
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Highlight color for matching brackets (hex format: #RRGGBB or #RRGGBBAA)
    #[serde(default = "default_bracket_color")]
    pub highlight_color: String,
}

impl Default for BracketConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            highlight_color: default_bracket_color(),
        }
    }
}

/// Link detection and opening configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkConfig {
    /// Enable link detection and Ctrl+Click to open
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Modifier key for opening links ("ctrl", "cmd", "alt")
    #[serde(default = "default_link_modifier")]
    pub modifier: String,
    /// Show underline for detected links
    #[serde(default = "default_true")]
    pub underline: bool,
}

impl Default for LinkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            modifier: default_link_modifier(),
            underline: true,
        }
    }
}

/// Title configuration for window and tab titles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleConfig {
    /// Format string for tab titles (supports ${command}, ${cwd}, ${title})
    #[serde(default = "default_title_format")]
    pub format: String,
    /// Show current working directory in title
    #[serde(default = "default_true")]
    pub show_cwd: bool,
    /// Maximum length of title before truncation
    #[serde(default = "default_title_max_length")]
    pub max_length: usize,
}

impl Default for TitleConfig {
    fn default() -> Self {
        Self {
            format: default_title_format(),
            show_cwd: true,
            max_length: default_title_max_length(),
        }
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
    #[serde(default)]
    pub bindings: Vec<KeyBinding>,
}

/// Keyboard repeat configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardConfig {
    #[serde(default = "default_repeat_delay")]
    pub repeat_delay_ms: u64,
    #[serde(default = "default_repeat_rate")]
    pub repeat_rate_ms: u64,
}

/// A single key binding mapping key combination to action
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    /// Key name (e.g., "t", "c", "Escape", "ArrowUp", "F12")
    pub key: String,
    /// Modifier keys (Ctrl, Shift, Alt, Cmd)
    #[serde(default)]
    pub modifiers: KeyModifiers,
    /// Action to execute (e.g., "new_tab", "copy", "paste")
    pub action: String,
    /// Optional description for documentation
    #[serde(default)]
    pub description: Option<String>,
}

/// Modifier keys for key bindings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub struct KeyModifiers {
    #[serde(default)]
    pub ctrl: bool,
    #[serde(default)]
    pub shift: bool,
    #[serde(default)]
    pub alt: bool,
    #[serde(default)]
    pub cmd: bool,
}

impl KeyBinding {
    /// Create a new key binding
    pub fn new(key: String, modifiers: KeyModifiers, action: String) -> Self {
        Self {
            key,
            modifiers,
            action,
            description: None,
        }
    }

    /// Create a new key binding with description
    pub fn with_description(
        key: String,
        modifiers: KeyModifiers,
        action: String,
        description: String,
    ) -> Self {
        Self {
            key,
            modifiers,
            action,
            description: Some(description),
        }
    }

    /// Get a human-readable representation of the key combination
    pub fn key_combination(&self) -> String {
        let mut parts = Vec::new();
        if self.modifiers.ctrl {
            parts.push("Ctrl");
        }
        if self.modifiers.shift {
            parts.push("Shift");
        }
        if self.modifiers.alt {
            parts.push("Alt");
        }
        if self.modifiers.cmd {
            parts.push("Cmd");
        }
        parts.push(&self.key);
        parts.join("+")
    }
}

impl KeyModifiers {
    /// Create modifiers with no keys pressed
    pub fn none() -> Self {
        Self::default()
    }

    /// Create modifiers with Ctrl
    pub fn ctrl() -> Self {
        Self {
            ctrl: true,
            ..Default::default()
        }
    }

    /// Create modifiers with Cmd (Command/Super)
    pub fn cmd() -> Self {
        Self {
            cmd: true,
            ..Default::default()
        }
    }

    /// Create modifiers with Shift
    pub fn shift() -> Self {
        Self {
            shift: true,
            ..Default::default()
        }
    }

    /// Create modifiers with Ctrl+Shift
    pub fn ctrl_shift() -> Self {
        Self {
            ctrl: true,
            shift: true,
            ..Default::default()
        }
    }

    /// Create modifiers with Cmd+Shift
    pub fn cmd_shift() -> Self {
        Self {
            cmd: true,
            shift: true,
            ..Default::default()
        }
    }
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            mode: default_keybinding_mode(),
            custom: HashMap::new(),
            keyboard: KeyboardConfig::default(),
            bindings: Self::default_bindings(),
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

impl KeybindingsConfig {
    /// Get default key bindings
    pub fn default_bindings() -> Vec<KeyBinding> {
        vec![
            // Tab management
            KeyBinding::with_description(
                "t".to_string(),
                KeyModifiers::cmd(),
                "new_tab".to_string(),
                "Open a new tab".to_string(),
            ),
            KeyBinding::with_description(
                "w".to_string(),
                KeyModifiers::cmd(),
                "close_tab".to_string(),
                "Close current tab".to_string(),
            ),
            KeyBinding::with_description(
                "]".to_string(),
                KeyModifiers::cmd(),
                "next_tab".to_string(),
                "Switch to next tab".to_string(),
            ),
            KeyBinding::with_description(
                "[".to_string(),
                KeyModifiers::cmd(),
                "prev_tab".to_string(),
                "Switch to previous tab".to_string(),
            ),
            KeyBinding::with_description(
                "d".to_string(),
                KeyModifiers::cmd_shift(),
                "duplicate_tab".to_string(),
                "Duplicate current tab".to_string(),
            ),
            // Tab selection (Cmd+1 through Cmd+9)
            KeyBinding::new(
                "1".to_string(),
                KeyModifiers::cmd(),
                "select_tab_1".to_string(),
            ),
            KeyBinding::new(
                "2".to_string(),
                KeyModifiers::cmd(),
                "select_tab_2".to_string(),
            ),
            KeyBinding::new(
                "3".to_string(),
                KeyModifiers::cmd(),
                "select_tab_3".to_string(),
            ),
            KeyBinding::new(
                "4".to_string(),
                KeyModifiers::cmd(),
                "select_tab_4".to_string(),
            ),
            KeyBinding::new(
                "5".to_string(),
                KeyModifiers::cmd(),
                "select_tab_5".to_string(),
            ),
            KeyBinding::new(
                "6".to_string(),
                KeyModifiers::cmd(),
                "select_tab_6".to_string(),
            ),
            KeyBinding::new(
                "7".to_string(),
                KeyModifiers::cmd(),
                "select_tab_7".to_string(),
            ),
            KeyBinding::new(
                "8".to_string(),
                KeyModifiers::cmd(),
                "select_tab_8".to_string(),
            ),
            KeyBinding::new(
                "9".to_string(),
                KeyModifiers::cmd(),
                "select_tab_9".to_string(),
            ),
            // Clipboard
            KeyBinding::with_description(
                "c".to_string(),
                KeyModifiers::cmd_shift(),
                "force_copy".to_string(),
                "Force copy selection".to_string(),
            ),
            KeyBinding::with_description(
                "v".to_string(),
                KeyModifiers::cmd(),
                "paste".to_string(),
                "Paste from clipboard".to_string(),
            ),
            KeyBinding::with_description(
                "v".to_string(),
                KeyModifiers::cmd_shift(),
                "force_paste".to_string(),
                "Force paste without bracketed paste".to_string(),
            ),
            // Terminal control
            KeyBinding::with_description(
                "k".to_string(),
                KeyModifiers::cmd(),
                "clear_screen".to_string(),
                "Clear terminal screen".to_string(),
            ),
            KeyBinding::with_description(
                "Home".to_string(),
                KeyModifiers::cmd(),
                "scroll_to_top".to_string(),
                "Scroll to top".to_string(),
            ),
            KeyBinding::with_description(
                "End".to_string(),
                KeyModifiers::cmd(),
                "scroll_to_bottom".to_string(),
                "Scroll to bottom".to_string(),
            ),
            // Font size
            KeyBinding::with_description(
                "+".to_string(),
                KeyModifiers::cmd(),
                "increase_font_size".to_string(),
                "Increase font size".to_string(),
            ),
            KeyBinding::with_description(
                "=".to_string(),
                KeyModifiers::cmd(),
                "increase_font_size".to_string(),
                "Increase font size".to_string(),
            ),
            KeyBinding::with_description(
                "-".to_string(),
                KeyModifiers::cmd(),
                "decrease_font_size".to_string(),
                "Decrease font size".to_string(),
            ),
            KeyBinding::with_description(
                "0".to_string(),
                KeyModifiers::cmd(),
                "reset_font_size".to_string(),
                "Reset font size".to_string(),
            ),
            // Debug panel
            KeyBinding::with_description(
                "d".to_string(),
                KeyModifiers::cmd(),
                "toggle_debug_panel".to_string(),
                "Toggle debug panel".to_string(),
            ),
            KeyBinding::with_description(
                "F12".to_string(),
                KeyModifiers::none(),
                "toggle_debug_panel".to_string(),
                "Toggle debug panel".to_string(),
            ),
            // Pane management
            KeyBinding::with_description(
                "h".to_string(),
                KeyModifiers::cmd_shift(),
                "split_horizontal".to_string(),
                "Split pane horizontally".to_string(),
            ),
            KeyBinding::with_description(
                "|".to_string(),
                KeyModifiers::cmd_shift(),
                "split_vertical".to_string(),
                "Split pane vertically".to_string(),
            ),
        ]
    }

    /// Get keybindings file path (~/.config/agterm/keybindings.toml)
    pub fn keybindings_file_path() -> Option<PathBuf> {
        dirs::config_dir().map(|config_dir| config_dir.join("agterm").join("keybindings.toml"))
    }

    /// Load keybindings from file and merge with defaults
    pub fn load_keybindings() -> Vec<KeyBinding> {
        let mut bindings = Self::default_bindings();

        if let Some(path) = Self::keybindings_file_path() {
            if path.exists() {
                match std::fs::read_to_string(&path) {
                    Ok(contents) => {
                        #[derive(Deserialize)]
                        struct KeybindingsFile {
                            #[serde(default)]
                            bindings: Vec<KeyBinding>,
                        }

                        match toml::from_str::<KeybindingsFile>(&contents) {
                            Ok(file) => {
                                // Merge custom bindings (override defaults)
                                for custom_binding in file.bindings {
                                    // Remove any existing binding with same key+modifiers
                                    bindings.retain(|b| {
                                        b.key != custom_binding.key
                                            || b.modifiers != custom_binding.modifiers
                                    });
                                    bindings.push(custom_binding);
                                }
                                tracing::info!("Loaded custom keybindings from {:?}", path);
                            }
                            Err(e) => {
                                tracing::error!("Failed to parse keybindings.toml: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to read keybindings.toml: {}", e);
                    }
                }
            }
        }

        bindings
    }

    /// Save keybindings to file
    pub fn save_keybindings(bindings: &[KeyBinding]) -> Result<(), ConfigError> {
        let path = Self::keybindings_file_path().ok_or_else(|| {
            ConfigError::IoError("Could not determine keybindings directory".to_string())
        })?;

        // Create config directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ConfigError::IoError(e.to_string()))?;
        }

        #[derive(Serialize)]
        struct KeybindingsFile<'a> {
            bindings: &'a [KeyBinding],
        }

        let file = KeybindingsFile { bindings };
        let toml_string = toml::to_string_pretty(&file)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;

        std::fs::write(&path, toml_string).map_err(|e| ConfigError::IoError(e.to_string()))?;

        tracing::info!("Saved {} keybindings to {:?}", bindings.len(), path);
        Ok(())
    }

    /// Check for conflicting keybindings
    pub fn detect_conflicts(bindings: &[KeyBinding]) -> Vec<(KeyBinding, KeyBinding)> {
        let mut conflicts = Vec::new();
        let mut seen: HashMap<(String, KeyModifiers), KeyBinding> = HashMap::new();

        for binding in bindings {
            let key = (binding.key.clone(), binding.modifiers.clone());
            if let Some(existing) = seen.get(&key) {
                conflicts.push((binding.clone(), existing.clone()));
            } else {
                seen.insert(key, binding.clone());
            }
        }

        conflicts
    }

    /// Reset keybindings to defaults
    pub fn reset_to_defaults() -> Result<(), ConfigError> {
        let path = Self::keybindings_file_path().ok_or_else(|| {
            ConfigError::IoError("Could not determine keybindings directory".to_string())
        })?;

        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| ConfigError::IoError(e.to_string()))?;
            tracing::info!("Reset keybindings to defaults (removed {:?})", path);
        }

        Ok(())
    }

    /// Find binding by key combination
    pub fn find_binding(&self, key: &str, modifiers: &KeyModifiers) -> Option<&KeyBinding> {
        self.bindings
            .iter()
            .find(|b| b.key == key && &b.modifiers == modifiers)
    }

    /// Get all bindings for a specific action
    pub fn bindings_for_action(&self, action: &str) -> Vec<&KeyBinding> {
        self.bindings
            .iter()
            .filter(|b| b.action == action)
            .collect()
    }
}

/// Shell configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    /// Shell program path (None = auto-detect)
    #[serde(default)]
    pub program: Option<String>,
    /// Shell arguments
    #[serde(default)]
    pub args: Vec<String>,
    /// Environment variables for the shell
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Launch as login shell
    #[serde(default = "default_true")]
    pub login_shell: bool,
    /// Working directory (None = current directory)
    #[serde(default)]
    pub working_directory: Option<PathBuf>,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            program: None,
            args: Vec::new(),
            env: HashMap::new(),
            login_shell: true,
            working_directory: None,
        }
    }
}

/// Environment variable configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Inherit environment variables from parent process
    #[serde(default = "default_true")]
    pub inherit: bool,
    /// Additional/override environment variables
    #[serde(default)]
    pub variables: HashMap<String, String>,
    /// TERM environment variable (default: xterm-256color)
    #[serde(default = "default_term")]
    pub term: String,
    /// LANG environment variable
    #[serde(default)]
    pub lang: Option<String>,
    /// Directories to prepend to PATH
    #[serde(default)]
    pub path_prepend: Vec<String>,
    /// Directories to append to PATH
    #[serde(default)]
    pub path_append: Vec<String>,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            inherit: true,
            variables: HashMap::new(),
            term: default_term(),
            lang: None,
            path_prepend: Vec::new(),
            path_append: Vec::new(),
        }
    }
}

impl EnvironmentConfig {
    /// Convert to PtyEnvironment for PTY session creation
    pub fn to_pty_environment(&self) -> crate::terminal::pty::PtyEnvironment {
        use crate::terminal::pty::PtyEnvironment;

        let mut variables = HashMap::new();

        // Set TERM
        variables.insert("TERM".to_string(), self.term.clone());

        // Set LANG if specified
        if let Some(lang) = &self.lang {
            variables.insert("LANG".to_string(), lang.clone());
        }

        // Handle PATH modifications
        if !self.path_prepend.is_empty() || !self.path_append.is_empty() {
            let current_path = std::env::var("PATH").unwrap_or_default();
            let mut path_parts = Vec::new();

            // Add prepend paths
            path_parts.extend(self.path_prepend.iter().cloned());

            // Add current PATH
            if !current_path.is_empty() {
                path_parts.push(current_path);
            }

            // Add append paths
            path_parts.extend(self.path_append.iter().cloned());

            let new_path = path_parts.join(":");
            variables.insert("PATH".to_string(), new_path);
        }

        // Add default AgTerm variables
        variables.insert("COLORTERM".to_string(), "truecolor".to_string());
        variables.insert("TERM_PROGRAM".to_string(), "agterm".to_string());
        variables.insert("AGTERM_VERSION".to_string(), env!("CARGO_PKG_VERSION").to_string());

        // Apply user-specified variables (these override defaults)
        for (key, value) in &self.variables {
            variables.insert(key.clone(), value.clone());
        }

        PtyEnvironment {
            inherit_env: self.inherit,
            variables,
            unset: Vec::new(),
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

/// Notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub on_bell: bool,
    #[serde(default = "default_false")]
    pub on_command_complete: bool,
    #[serde(default = "default_notification_timeout")]
    pub timeout_seconds: u64,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            on_bell: true,
            on_command_complete: false,
            timeout_seconds: default_notification_timeout(),
        }
    }
}

/// Status bar configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusBarConfig {
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default = "default_true")]
    pub show_cwd: bool,
    #[serde(default = "default_true")]
    pub show_size: bool,
    #[serde(default = "default_true")]
    pub show_encoding: bool,
    #[serde(default = "default_true")]
    pub show_scroll_position: bool,
    #[serde(default = "default_true")]
    pub show_mode: bool,
}

impl Default for StatusBarConfig {
    fn default() -> Self {
        Self {
            visible: true,
            show_cwd: true,
            show_size: true,
            show_encoding: true,
            show_scroll_position: true,
            show_mode: true,
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

fn default_line_height() -> f32 {
    1.2
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

fn default_flash_color() -> String {
    "#FFFFFF80".to_string() // White with 50% opacity
}

fn default_flash_duration() -> u64 {
    100 // 100ms
}

fn default_image_max_size() -> usize {
    10 * 1024 * 1024 // 10MB
}

fn default_bracket_color() -> String {
    "#5c8afa".to_string() // Accent blue for bracket highlights
}

fn default_link_modifier() -> String {
    #[cfg(target_os = "macos")]
    return "cmd".to_string();
    #[cfg(not(target_os = "macos"))]
    return "ctrl".to_string();
}

fn default_title_format() -> String {
    "${title}".to_string()
}

fn default_title_max_length() -> usize {
    50
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

fn default_timeout() -> u64 {
    5
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

fn default_term() -> String {
    "xterm-256color".to_string()
}

fn default_notification_timeout() -> u64 {
    5
}

fn default_auto_save_interval() -> u64 {
    30 // Auto-save every 30 seconds
}

fn default_max_backups() -> usize {
    5 // Keep up to 5 backup files
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
            environment: overlay.environment,
            mouse: overlay.mouse,
            pty: overlay.pty,
            tui: overlay.tui,
            logging: overlay.logging,
            debug: overlay.debug,
            notification: overlay.notification,
            status_bar: overlay.status_bar,
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
            environment: EnvironmentConfig::default(),
            mouse: MouseConfig::default(),
            pty: PtyConfig::default(),
            tui: TuiConfig::default(),
            logging: LoggingConfig::default(),
            debug: DebugConfig::default(),
            notification: NotificationConfig::default(),
            status_bar: StatusBarConfig::default(),
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
// Hook System
// ============================================================================

/// Hook for custom terminal event handling
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Hook {
    /// Hook name (for identification)
    pub name: String,
    /// Event type that triggers this hook
    pub event_type: HookEvent,
    /// Action to perform when triggered
    pub action: HookAction,
    /// Whether this hook is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Terminal event types that can trigger hooks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum HookEvent {
    /// Command execution completed
    CommandComplete {
        /// Optional command pattern to match (regex)
        #[serde(default)]
        command_pattern: Option<String>,
        /// Optional exit code to match (None matches any)
        #[serde(default)]
        exit_code: Option<i32>,
    },
    /// Directory changed
    DirectoryChange {
        /// Optional directory pattern to match (glob)
        #[serde(default)]
        directory_pattern: Option<String>,
    },
    /// Terminal output matches a pattern
    OutputMatch {
        /// Pattern to match in output (regex)
        pattern: String,
    },
    /// Terminal bell received
    Bell,
}

/// Actions to perform when a hook is triggered
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum HookAction {
    /// Send a desktop notification
    Notify {
        /// Notification title
        title: String,
        /// Notification message
        message: String,
    },
    /// Run a shell command
    RunCommand {
        /// Command to execute
        command: String,
        /// Arguments for the command
        #[serde(default)]
        args: Vec<String>,
    },
    /// Play a sound file
    PlaySound {
        /// Path to sound file
        path: String,
        /// Volume (0.0 to 1.0)
        #[serde(default = "default_hook_volume")]
        volume: f32,
    },
    /// Custom function (for future extension)
    Custom {
        /// Custom action identifier
        id: String,
        /// Custom action parameters
        #[serde(default)]
        params: HashMap<String, String>,
    },
}

fn default_hook_volume() -> f32 {
    0.5
}

impl Hook {
    /// Create a new hook
    pub fn new(name: String, event_type: HookEvent, action: HookAction) -> Self {
        Self {
            name,
            event_type,
            action,
            enabled: true,
        }
    }

    /// Get the hooks directory path (~/.config/agterm/)
    pub fn hooks_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|config_dir| config_dir.join("agterm"))
    }

    /// Get the hooks file path (~/.config/agterm/hooks.toml)
    pub fn hooks_file_path() -> Option<PathBuf> {
        Self::hooks_dir().map(|dir| dir.join("hooks.toml"))
    }

    /// Load hooks from file
    pub fn load_from_file() -> Result<Vec<Hook>, ConfigError> {
        let path = Self::hooks_file_path().ok_or_else(|| {
            ConfigError::IoError("Could not determine hooks directory".to_string())
        })?;

        if !path.exists() {
            // Return default hooks if file doesn't exist
            return Ok(Self::default_hooks());
        }

        let contents =
            std::fs::read_to_string(&path).map_err(|e| ConfigError::IoError(e.to_string()))?;

        #[derive(Deserialize)]
        struct HooksFile {
            hooks: Vec<Hook>,
        }

        let file: HooksFile = toml::from_str(&contents)
            .map_err(|e| ConfigError::ParseError(format!("Failed to parse hooks.toml: {}", e)))?;

        Ok(file.hooks)
    }

    /// Save hooks to file
    pub fn save_to_file(hooks: &[Hook]) -> Result<(), ConfigError> {
        let hooks_dir = Self::hooks_dir().ok_or_else(|| {
            ConfigError::IoError("Could not determine hooks directory".to_string())
        })?;

        // Create config directory if it doesn't exist
        std::fs::create_dir_all(&hooks_dir).map_err(|e| ConfigError::IoError(e.to_string()))?;

        let path = hooks_dir.join("hooks.toml");

        #[derive(Serialize)]
        struct HooksFile<'a> {
            hooks: &'a [Hook],
        }

        let file = HooksFile { hooks };
        let toml_string = toml::to_string_pretty(&file)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;

        std::fs::write(&path, toml_string).map_err(|e| ConfigError::IoError(e.to_string()))?;

        tracing::info!("Saved {} hooks to {:?}", hooks.len(), path);
        Ok(())
    }

    /// Get default hooks
    pub fn default_hooks() -> Vec<Hook> {
        vec![
            // Notify on long-running command completion
            Hook::new(
                "Long Command Complete".to_string(),
                HookEvent::CommandComplete {
                    command_pattern: None,
                    exit_code: None,
                },
                HookAction::Notify {
                    title: "Command Complete".to_string(),
                    message: "Your long-running command has finished".to_string(),
                },
            ),
            // Play sound on error
            Hook {
                name: "Error Bell".to_string(),
                event_type: HookEvent::Bell,
                action: HookAction::PlaySound {
                    path: "/System/Library/Sounds/Basso.aiff".to_string(),
                    volume: 0.3,
                },
                enabled: false, // Disabled by default
            },
            // Notify on directory change to home
            Hook {
                name: "Home Directory".to_string(),
                event_type: HookEvent::DirectoryChange {
                    directory_pattern: Some("~".to_string()),
                },
                action: HookAction::Notify {
                    title: "Directory Changed".to_string(),
                    message: "Entered home directory".to_string(),
                },
                enabled: false, // Disabled by default
            },
            // Notify on error output
            Hook {
                name: "Error Pattern".to_string(),
                event_type: HookEvent::OutputMatch {
                    pattern: "(?i)(error|fail|fatal)".to_string(),
                },
                action: HookAction::Notify {
                    title: "Error Detected".to_string(),
                    message: "Error pattern detected in output".to_string(),
                },
                enabled: false, // Disabled by default
            },
        ]
    }

    /// Initialize hooks file with defaults if it doesn't exist
    pub fn initialize_default_file() -> Result<(), ConfigError> {
        let path = Self::hooks_file_path().ok_or_else(|| {
            ConfigError::IoError("Could not determine hooks directory".to_string())
        })?;

        if !path.exists() {
            let default_hooks = Self::default_hooks();
            Self::save_to_file(&default_hooks)?;
            tracing::info!("Created default hooks file at {:?}", path);
        }

        Ok(())
    }

    /// Check if this hook should trigger for the given event
    pub fn matches_event(&self, event: &HookEvent) -> bool {
        if !self.enabled {
            return false;
        }

        match (&self.event_type, event) {
            (
                HookEvent::CommandComplete {
                    command_pattern: pattern1,
                    exit_code: code1,
                },
                HookEvent::CommandComplete {
                    command_pattern: pattern2,
                    exit_code: code2,
                },
            ) => {
                // Check exit code match
                let code_matches = match (code1, code2) {
                    (Some(c1), Some(c2)) => c1 == c2,
                    (None, _) => true, // None matches any
                    (Some(_), None) => false,
                };

                // Check command pattern match
                let pattern_matches = match (pattern1, pattern2) {
                    (Some(p1), Some(p2)) => {
                        // Try regex match
                        if let Ok(re) = regex::Regex::new(p1) {
                            re.is_match(p2)
                        } else {
                            p1 == p2 // Fallback to exact match
                        }
                    }
                    (None, _) => true, // None matches any
                    (Some(_), None) => false,
                };

                code_matches && pattern_matches
            }
            (
                HookEvent::DirectoryChange {
                    directory_pattern: pattern1,
                },
                HookEvent::DirectoryChange {
                    directory_pattern: pattern2,
                },
            ) => {
                match (pattern1, pattern2) {
                    (Some(p1), Some(p2)) => {
                        // Simple glob-style matching
                        p1 == p2 || p1 == "*" || p2.contains(p1)
                    }
                    (None, _) => true, // None matches any
                    (Some(_), None) => false,
                }
            }
            (
                HookEvent::OutputMatch { pattern: pattern1 },
                HookEvent::OutputMatch { pattern: pattern2 },
            ) => {
                // Try regex match
                if let Ok(re) = regex::Regex::new(pattern1) {
                    re.is_match(pattern2)
                } else {
                    pattern1 == pattern2 // Fallback to exact match
                }
            }
            (HookEvent::Bell, HookEvent::Bell) => true,
            _ => false,
        }
    }

    /// Execute the action associated with this hook
    pub fn execute(&self) -> Result<(), String> {
        match &self.action {
            HookAction::Notify { title, message } => {
                tracing::info!(
                    "Hook '{}' triggered notification: {} - {}",
                    self.name,
                    title,
                    message
                );
                // TODO: Implement actual notification system
                Ok(())
            }
            HookAction::RunCommand { command, args } => {
                tracing::info!(
                    "Hook '{}' running command: {} {:?}",
                    self.name,
                    command,
                    args
                );
                // TODO: Implement command execution
                Ok(())
            }
            HookAction::PlaySound { path, volume } => {
                tracing::info!(
                    "Hook '{}' playing sound: {} (volume: {})",
                    self.name,
                    path,
                    volume
                );
                // TODO: Integrate with sound system
                Ok(())
            }
            HookAction::Custom { id, params } => {
                tracing::info!(
                    "Hook '{}' executing custom action: {} with params: {:?}",
                    self.name,
                    id,
                    params
                );
                // TODO: Implement custom action registry
                Ok(())
            }
        }
    }
}

/// Hook manager for handling terminal events
pub struct HookManager {
    hooks: Vec<Hook>,
}

impl HookManager {
    /// Create a new hook manager
    pub fn new() -> Self {
        let hooks = Hook::load_from_file().unwrap_or_else(|e| {
            tracing::warn!("Failed to load hooks: {}, using defaults", e);
            Hook::default_hooks()
        });

        Self { hooks }
    }

    /// Get all hooks
    pub fn hooks(&self) -> &[Hook] {
        &self.hooks
    }

    /// Add a new hook
    pub fn add_hook(&mut self, hook: Hook) {
        self.hooks.push(hook);
    }

    /// Remove a hook by name
    pub fn remove_hook(&mut self, name: &str) -> bool {
        if let Some(pos) = self.hooks.iter().position(|h| h.name == name) {
            self.hooks.remove(pos);
            true
        } else {
            false
        }
    }

    /// Enable or disable a hook by name
    pub fn set_hook_enabled(&mut self, name: &str, enabled: bool) -> bool {
        if let Some(hook) = self.hooks.iter_mut().find(|h| h.name == name) {
            hook.enabled = enabled;
            true
        } else {
            false
        }
    }

    /// Process an event and execute matching hooks
    pub fn process_event(&self, event: &HookEvent) {
        for hook in &self.hooks {
            if hook.matches_event(event) {
                if let Err(e) = hook.execute() {
                    tracing::error!("Failed to execute hook '{}': {}", hook.name, e);
                }
            }
        }
    }

    /// Save current hooks to file
    pub fn save(&self) -> Result<(), ConfigError> {
        Hook::save_to_file(&self.hooks)
    }

    /// Reload hooks from file
    pub fn reload(&mut self) -> Result<(), ConfigError> {
        self.hooks = Hook::load_from_file()?;
        Ok(())
    }
}

impl Default for HookManager {
    fn default() -> Self {
        Self::new()
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

    // ========== Hook System Tests ==========

    #[test]
    fn test_hook_creation() {
        let hook = Hook::new(
            "Test Hook".to_string(),
            HookEvent::Bell,
            HookAction::Notify {
                title: "Test".to_string(),
                message: "Test message".to_string(),
            },
        );

        assert_eq!(hook.name, "Test Hook");
        assert_eq!(hook.event_type, HookEvent::Bell);
        assert!(hook.enabled);
    }

    #[test]
    fn test_hook_serialization() {
        let hook = Hook::new(
            "Test".to_string(),
            HookEvent::CommandComplete {
                command_pattern: Some("git.*".to_string()),
                exit_code: Some(0),
            },
            HookAction::Notify {
                title: "Success".to_string(),
                message: "Command completed".to_string(),
            },
        );

        let toml_str = toml::to_string(&hook).unwrap();
        let deserialized: Hook = toml::from_str(&toml_str).unwrap();

        assert_eq!(hook, deserialized);
    }

    #[test]
    fn test_default_hooks() {
        let hooks = Hook::default_hooks();
        assert!(!hooks.is_empty());

        let has_notify = hooks
            .iter()
            .any(|h| matches!(h.action, HookAction::Notify { .. }));
        assert!(has_notify);

        let has_bell = hooks
            .iter()
            .any(|h| matches!(h.event_type, HookEvent::Bell));
        assert!(has_bell);
    }

    #[test]
    fn test_hook_event_matching_bell() {
        let hook = Hook::new(
            "Bell Test".to_string(),
            HookEvent::Bell,
            HookAction::Notify {
                title: "Bell".to_string(),
                message: "Bell received".to_string(),
            },
        );

        assert!(hook.matches_event(&HookEvent::Bell));
        assert!(!hook.matches_event(&HookEvent::CommandComplete {
            command_pattern: None,
            exit_code: None,
        }));
    }

    #[test]
    fn test_hook_event_matching_command_complete() {
        let hook = Hook::new(
            "Command Test".to_string(),
            HookEvent::CommandComplete {
                command_pattern: Some("git.*".to_string()),
                exit_code: Some(0),
            },
            HookAction::Notify {
                title: "Git Success".to_string(),
                message: "Git command succeeded".to_string(),
            },
        );

        assert!(hook.matches_event(&HookEvent::CommandComplete {
            command_pattern: Some("git status".to_string()),
            exit_code: Some(0),
        }));

        assert!(!hook.matches_event(&HookEvent::CommandComplete {
            command_pattern: Some("ls".to_string()),
            exit_code: Some(0),
        }));

        assert!(!hook.matches_event(&HookEvent::CommandComplete {
            command_pattern: Some("git status".to_string()),
            exit_code: Some(1),
        }));
    }

    #[test]
    fn test_hook_event_matching_wildcard() {
        let hook = Hook::new(
            "Any Command".to_string(),
            HookEvent::CommandComplete {
                command_pattern: None,
                exit_code: None,
            },
            HookAction::Notify {
                title: "Command".to_string(),
                message: "Any command completed".to_string(),
            },
        );

        assert!(hook.matches_event(&HookEvent::CommandComplete {
            command_pattern: Some("anything".to_string()),
            exit_code: Some(0),
        }));

        assert!(hook.matches_event(&HookEvent::CommandComplete {
            command_pattern: Some("something else".to_string()),
            exit_code: Some(1),
        }));
    }

    #[test]
    fn test_hook_event_matching_directory_change() {
        let hook = Hook::new(
            "Home Dir".to_string(),
            HookEvent::DirectoryChange {
                directory_pattern: Some("home".to_string()),
            },
            HookAction::Notify {
                title: "Directory".to_string(),
                message: "Changed to home".to_string(),
            },
        );

        assert!(hook.matches_event(&HookEvent::DirectoryChange {
            directory_pattern: Some("/home/user".to_string()),
        }));

        assert!(!hook.matches_event(&HookEvent::DirectoryChange {
            directory_pattern: Some("/tmp".to_string()),
        }));
    }

    #[test]
    fn test_hook_event_matching_output_pattern() {
        let hook = Hook::new(
            "Error Pattern".to_string(),
            HookEvent::OutputMatch {
                pattern: "(?i)error".to_string(),
            },
            HookAction::Notify {
                title: "Error".to_string(),
                message: "Error detected".to_string(),
            },
        );

        assert!(hook.matches_event(&HookEvent::OutputMatch {
            pattern: "Error occurred".to_string(),
        }));

        assert!(hook.matches_event(&HookEvent::OutputMatch {
            pattern: "ERROR: something failed".to_string(),
        }));

        assert!(!hook.matches_event(&HookEvent::OutputMatch {
            pattern: "Success".to_string(),
        }));
    }

    #[test]
    fn test_hook_disabled() {
        let mut hook = Hook::new(
            "Disabled Hook".to_string(),
            HookEvent::Bell,
            HookAction::Notify {
                title: "Test".to_string(),
                message: "Test".to_string(),
            },
        );

        hook.enabled = false;
        assert!(!hook.matches_event(&HookEvent::Bell));
    }

    #[test]
    fn test_hook_action_types() {
        let notify_hook = Hook::new(
            "Notify".to_string(),
            HookEvent::Bell,
            HookAction::Notify {
                title: "Title".to_string(),
                message: "Message".to_string(),
            },
        );
        assert!(matches!(notify_hook.action, HookAction::Notify { .. }));

        let command_hook = Hook::new(
            "Command".to_string(),
            HookEvent::Bell,
            HookAction::RunCommand {
                command: "echo".to_string(),
                args: vec!["hello".to_string()],
            },
        );
        assert!(matches!(command_hook.action, HookAction::RunCommand { .. }));

        let sound_hook = Hook::new(
            "Sound".to_string(),
            HookEvent::Bell,
            HookAction::PlaySound {
                path: "/path/to/sound.wav".to_string(),
                volume: 0.5,
            },
        );
        assert!(matches!(sound_hook.action, HookAction::PlaySound { .. }));

        let custom_hook = Hook::new(
            "Custom".to_string(),
            HookEvent::Bell,
            HookAction::Custom {
                id: "custom_action".to_string(),
                params: HashMap::new(),
            },
        );
        assert!(matches!(custom_hook.action, HookAction::Custom { .. }));
    }

    #[test]
    fn test_hook_manager_creation() {
        let manager = HookManager::new();
        assert!(!manager.hooks().is_empty());
    }

    #[test]
    fn test_hook_manager_add_remove() {
        let mut manager = HookManager::new();
        let initial_count = manager.hooks().len();

        let hook = Hook::new(
            "New Hook".to_string(),
            HookEvent::Bell,
            HookAction::Notify {
                title: "Test".to_string(),
                message: "Test".to_string(),
            },
        );

        manager.add_hook(hook);
        assert_eq!(manager.hooks().len(), initial_count + 1);

        assert!(manager.remove_hook("New Hook"));
        assert_eq!(manager.hooks().len(), initial_count);

        assert!(!manager.remove_hook("Nonexistent Hook"));
    }

    #[test]
    fn test_hook_manager_enable_disable() {
        let mut manager = HookManager::new();

        let hook = Hook::new(
            "Toggle Hook".to_string(),
            HookEvent::Bell,
            HookAction::Notify {
                title: "Test".to_string(),
                message: "Test".to_string(),
            },
        );
        manager.add_hook(hook);

        assert!(manager.set_hook_enabled("Toggle Hook", false));
        let hook = manager
            .hooks()
            .iter()
            .find(|h| h.name == "Toggle Hook")
            .unwrap();
        assert!(!hook.enabled);

        assert!(manager.set_hook_enabled("Toggle Hook", true));
        let hook = manager
            .hooks()
            .iter()
            .find(|h| h.name == "Toggle Hook")
            .unwrap();
        assert!(hook.enabled);

        assert!(!manager.set_hook_enabled("Nonexistent", false));
    }

    #[test]
    fn test_hook_manager_process_event() {
        let manager = HookManager::new();
        manager.process_event(&HookEvent::Bell);
        manager.process_event(&HookEvent::CommandComplete {
            command_pattern: None,
            exit_code: None,
        });
    }

    #[test]
    fn test_hook_save_and_load() {
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let hooks_file = temp_dir.path().join("hooks.toml");

        let hooks = vec![
            Hook::new(
                "Test1".to_string(),
                HookEvent::Bell,
                HookAction::Notify {
                    title: "Test1".to_string(),
                    message: "Message1".to_string(),
                },
            ),
            Hook::new(
                "Test2".to_string(),
                HookEvent::CommandComplete {
                    command_pattern: None,
                    exit_code: Some(0),
                },
                HookAction::RunCommand {
                    command: "echo".to_string(),
                    args: vec!["done".to_string()],
                },
            ),
        ];

        #[derive(Serialize)]
        struct HooksFile<'a> {
            hooks: &'a [Hook],
        }
        let file = HooksFile { hooks: &hooks };
        let toml_string = toml::to_string_pretty(&file).unwrap();
        std::fs::write(&hooks_file, toml_string).unwrap();

        let contents = std::fs::read_to_string(&hooks_file).unwrap();

        #[derive(Deserialize)]
        struct HooksFileLoad {
            hooks: Vec<Hook>,
        }
        let loaded: HooksFileLoad = toml::from_str(&contents).unwrap();

        assert_eq!(loaded.hooks.len(), 2);
        assert_eq!(loaded.hooks[0].name, "Test1");
        assert_eq!(loaded.hooks[1].name, "Test2");
    }

    #[test]
    fn test_hook_paths() {
        if let Some(hooks_dir) = Hook::hooks_dir() {
            assert!(hooks_dir.to_string_lossy().contains("agterm"));
        }

        if let Some(hooks_file) = Hook::hooks_file_path() {
            assert!(hooks_file.to_string_lossy().ends_with("hooks.toml"));
        }
    }

    #[test]
    fn test_hook_execute_methods() {
        let hooks = Hook::default_hooks();

        for hook in hooks {
            let result = hook.execute();
            assert!(result.is_ok());
        }
    }

    // ========== Keybinding Tests ==========

    #[test]
    fn test_keybinding_creation() {
        let binding = KeyBinding::new("t".to_string(), KeyModifiers::cmd(), "new_tab".to_string());

        assert_eq!(binding.key, "t");
        assert_eq!(binding.action, "new_tab");
        assert_eq!(binding.modifiers.cmd, true);
        assert_eq!(binding.modifiers.ctrl, false);
    }

    #[test]
    fn test_keybinding_key_combination() {
        let binding = KeyBinding::new(
            "t".to_string(),
            KeyModifiers::cmd_shift(),
            "new_tab".to_string(),
        );

        assert_eq!(binding.key_combination(), "Shift+Cmd+t");
    }

    #[test]
    fn test_default_bindings() {
        let bindings = KeybindingsConfig::default_bindings();

        assert!(!bindings.is_empty());

        // Check that specific bindings exist
        let has_new_tab = bindings.iter().any(|b| b.action == "new_tab");
        let has_close_tab = bindings.iter().any(|b| b.action == "close_tab");
        let has_copy = bindings.iter().any(|b| b.action == "force_copy");

        assert!(has_new_tab);
        assert!(has_close_tab);
        assert!(has_copy);
    }

    #[test]
    fn test_conflict_detection_no_conflicts() {
        let bindings = vec![
            KeyBinding::new("t".to_string(), KeyModifiers::cmd(), "new_tab".to_string()),
            KeyBinding::new(
                "w".to_string(),
                KeyModifiers::cmd(),
                "close_tab".to_string(),
            ),
            KeyBinding::new(
                "t".to_string(),
                KeyModifiers::ctrl(),
                "different_action".to_string(),
            ),
        ];

        let conflicts = KeybindingsConfig::detect_conflicts(&bindings);
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_conflict_detection_with_conflicts() {
        let binding1 = KeyBinding::new("t".to_string(), KeyModifiers::cmd(), "new_tab".to_string());
        let binding2 = KeyBinding::new(
            "t".to_string(),
            KeyModifiers::cmd(),
            "other_action".to_string(),
        );

        let bindings = vec![binding1, binding2];

        let conflicts = KeybindingsConfig::detect_conflicts(&bindings);
        assert_eq!(conflicts.len(), 1);
    }

    #[test]
    fn test_find_binding() {
        let config = KeybindingsConfig::default();

        // Find Cmd+T binding
        let binding = config.find_binding("t", &KeyModifiers::cmd());
        assert!(binding.is_some());
        assert_eq!(binding.unwrap().action, "new_tab");

        // Try to find non-existent binding
        let no_binding = config.find_binding("x", &KeyModifiers::cmd());
        assert!(no_binding.is_none());
    }

    #[test]
    fn test_bindings_for_action() {
        let config = KeybindingsConfig::default();

        let bindings = config.bindings_for_action("toggle_debug_panel");
        assert!(bindings.len() >= 1); // Should have at least one binding for debug panel
    }

    #[test]
    fn test_key_modifiers() {
        let ctrl = KeyModifiers::ctrl();
        assert!(ctrl.ctrl);
        assert!(!ctrl.shift);
        assert!(!ctrl.alt);
        assert!(!ctrl.cmd);

        let cmd_shift = KeyModifiers::cmd_shift();
        assert!(cmd_shift.cmd);
        assert!(cmd_shift.shift);
        assert!(!cmd_shift.ctrl);
        assert!(!cmd_shift.alt);
    }

    #[test]
    fn test_keybinding_serialization() {
        let binding = KeyBinding::with_description(
            "t".to_string(),
            KeyModifiers::cmd(),
            "new_tab".to_string(),
            "Open a new tab".to_string(),
        );

        // Serialize to TOML
        let toml_str = toml::to_string(&binding).unwrap();

        // Deserialize back
        let deserialized: KeyBinding = toml::from_str(&toml_str).unwrap();

        assert_eq!(binding.key, deserialized.key);
        assert_eq!(binding.action, deserialized.action);
        assert_eq!(binding.modifiers, deserialized.modifiers);
        assert_eq!(binding.description, deserialized.description);
    }

    #[test]
    fn test_keybindings_file_path() {
        if let Some(path) = KeybindingsConfig::keybindings_file_path() {
            assert!(path.to_string_lossy().contains("agterm"));
            assert!(path.to_string_lossy().ends_with("keybindings.toml"));
        }
    }
}
