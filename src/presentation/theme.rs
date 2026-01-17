//! Theme System
//!
//! Customizable themes for the TUI interface.

use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Theme name
    pub name: String,
    /// Theme description
    pub description: Option<String>,
    /// Color palette
    #[serde(default)]
    pub colors: ThemeColors,
    /// Component styles
    #[serde(default)]
    pub components: ComponentStyles,
}

/// Color palette for the theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    /// Primary color (main accents)
    #[serde(default = "default_primary")]
    pub primary: ThemeColor,
    /// Secondary color
    #[serde(default = "default_secondary")]
    pub secondary: ThemeColor,
    /// Success color (completed tasks, etc.)
    #[serde(default = "default_success")]
    pub success: ThemeColor,
    /// Warning color
    #[serde(default = "default_warning")]
    pub warning: ThemeColor,
    /// Error color (failed tasks, etc.)
    #[serde(default = "default_error")]
    pub error: ThemeColor,
    /// Info color
    #[serde(default = "default_info")]
    pub info: ThemeColor,
    /// Background color
    #[serde(default = "default_background")]
    pub background: ThemeColor,
    /// Foreground (text) color
    #[serde(default = "default_foreground")]
    pub foreground: ThemeColor,
    /// Muted/dim color
    #[serde(default = "default_muted")]
    pub muted: ThemeColor,
    /// Border color
    #[serde(default = "default_border")]
    pub border: ThemeColor,
    /// Highlight color (selection, focus)
    #[serde(default = "default_highlight")]
    pub highlight: ThemeColor,
}

/// A color value that can be an RGB tuple or named color
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ThemeColor {
    /// Named color (e.g., "red", "blue")
    Named(String),
    /// RGB tuple
    Rgb { r: u8, g: u8, b: u8 },
    /// Hex color (e.g., "#ff0000")
    Hex(String),
}

impl ThemeColor {
    /// Convert to ratatui Color
    #[must_use]
    pub fn to_color(&self) -> Color {
        match self {
            ThemeColor::Named(name) => match name.to_lowercase().as_str() {
                "black" => Color::Black,
                "red" => Color::Red,
                "green" => Color::Green,
                "yellow" => Color::Yellow,
                "blue" => Color::Blue,
                "magenta" => Color::Magenta,
                "cyan" => Color::Cyan,
                "gray" | "grey" => Color::Gray,
                "darkgray" | "darkgrey" => Color::DarkGray,
                "lightred" => Color::LightRed,
                "lightgreen" => Color::LightGreen,
                "lightyellow" => Color::LightYellow,
                "lightblue" => Color::LightBlue,
                "lightmagenta" => Color::LightMagenta,
                "lightcyan" => Color::LightCyan,
                "white" => Color::White,
                "reset" | "default" => Color::Reset,
                _ => Color::Reset,
            },
            ThemeColor::Rgb { r, g, b } => Color::Rgb(*r, *g, *b),
            ThemeColor::Hex(hex) => {
                let hex = hex.trim_start_matches('#');
                if hex.len() == 6 {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        u8::from_str_radix(&hex[0..2], 16),
                        u8::from_str_radix(&hex[2..4], 16),
                        u8::from_str_radix(&hex[4..6], 16),
                    ) {
                        return Color::Rgb(r, g, b);
                    }
                }
                Color::Reset
            }
        }
    }
}

/// Component-specific styles
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentStyles {
    /// Status bar style
    #[serde(default)]
    pub status_bar: StatusBarStyle,
    /// Task tree style
    #[serde(default)]
    pub task_tree: TaskTreeStyle,
    /// Terminal pane style
    #[serde(default)]
    pub terminal: TerminalStyle,
    /// Graph view style
    #[serde(default)]
    pub graph: GraphStyle,
    /// Archive browser style
    #[serde(default)]
    pub archive: ArchiveStyle,
}

/// Status bar component style
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusBarStyle {
    /// Show clock
    #[serde(default = "default_true")]
    pub show_clock: bool,
    /// Show task count
    #[serde(default = "default_true")]
    pub show_task_count: bool,
    /// Show MCP status
    #[serde(default = "default_true")]
    pub show_mcp_status: bool,
}

/// Task tree component style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTreeStyle {
    /// Icons for task statuses
    #[serde(default = "default_status_icons")]
    pub status_icons: HashMap<String, String>,
    /// Show task duration
    #[serde(default = "default_true")]
    pub show_duration: bool,
    /// Tree indent size
    #[serde(default = "default_indent")]
    pub indent: usize,
}

/// Terminal pane style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalStyle {
    /// Show scroll indicator
    #[serde(default = "default_true")]
    pub show_scroll_indicator: bool,
    /// Cursor blink rate (ms, 0 = no blink)
    #[serde(default = "default_cursor_blink")]
    pub cursor_blink_ms: u64,
    /// Show line numbers
    #[serde(default)]
    pub show_line_numbers: bool,
}

/// Graph view style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStyle {
    /// Node border style
    #[serde(default = "default_node_border")]
    pub node_border: String,
    /// Show progress bars in nodes
    #[serde(default = "default_true")]
    pub show_progress: bool,
    /// Show ETA in nodes
    #[serde(default = "default_true")]
    pub show_eta: bool,
}

/// Archive browser style
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArchiveStyle {
    /// Date format
    #[serde(default = "default_date_format")]
    pub date_format: String,
    /// Show preview in list
    #[serde(default = "default_true")]
    pub show_preview: bool,
}

// Default functions
fn default_true() -> bool {
    true
}

fn default_primary() -> ThemeColor {
    ThemeColor::Named("blue".to_string())
}

fn default_secondary() -> ThemeColor {
    ThemeColor::Named("cyan".to_string())
}

fn default_success() -> ThemeColor {
    ThemeColor::Named("green".to_string())
}

fn default_warning() -> ThemeColor {
    ThemeColor::Named("yellow".to_string())
}

fn default_error() -> ThemeColor {
    ThemeColor::Named("red".to_string())
}

fn default_info() -> ThemeColor {
    ThemeColor::Named("cyan".to_string())
}

fn default_background() -> ThemeColor {
    ThemeColor::Named("reset".to_string())
}

fn default_foreground() -> ThemeColor {
    ThemeColor::Named("white".to_string())
}

fn default_muted() -> ThemeColor {
    ThemeColor::Named("darkgray".to_string())
}

fn default_border() -> ThemeColor {
    ThemeColor::Named("gray".to_string())
}

fn default_highlight() -> ThemeColor {
    ThemeColor::Named("blue".to_string())
}

fn default_status_icons() -> HashMap<String, String> {
    let mut icons = HashMap::new();
    icons.insert("pending".to_string(), "○".to_string());
    icons.insert("queued".to_string(), "◐".to_string());
    icons.insert("running".to_string(), "●".to_string());
    icons.insert("completed".to_string(), "✓".to_string());
    icons.insert("failed".to_string(), "✗".to_string());
    icons.insert("cancelled".to_string(), "⊘".to_string());
    icons.insert("timeout".to_string(), "⏱".to_string());
    icons
}

fn default_indent() -> usize {
    2
}

fn default_cursor_blink() -> u64 {
    500
}

fn default_node_border() -> String {
    "rounded".to_string()
}

fn default_date_format() -> String {
    "%Y-%m-%d %H:%M".to_string()
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            primary: default_primary(),
            secondary: default_secondary(),
            success: default_success(),
            warning: default_warning(),
            error: default_error(),
            info: default_info(),
            background: default_background(),
            foreground: default_foreground(),
            muted: default_muted(),
            border: default_border(),
            highlight: default_highlight(),
        }
    }
}

impl Default for TaskTreeStyle {
    fn default() -> Self {
        Self {
            status_icons: default_status_icons(),
            show_duration: true,
            indent: default_indent(),
        }
    }
}

impl Default for TerminalStyle {
    fn default() -> Self {
        Self {
            show_scroll_indicator: true,
            cursor_blink_ms: default_cursor_blink(),
            show_line_numbers: false,
        }
    }
}

impl Default for GraphStyle {
    fn default() -> Self {
        Self {
            node_border: default_node_border(),
            show_progress: true,
            show_eta: true,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            description: Some("Default AgTerm theme".to_string()),
            colors: ThemeColors::default(),
            components: ComponentStyles::default(),
        }
    }
}

impl Theme {
    /// Create a new theme with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Get the dark theme preset
    #[must_use]
    pub fn dark() -> Self {
        Self {
            name: "dark".to_string(),
            description: Some("Dark theme with high contrast".to_string()),
            colors: ThemeColors {
                primary: ThemeColor::Hex("#61afef".to_string()),
                secondary: ThemeColor::Hex("#56b6c2".to_string()),
                success: ThemeColor::Hex("#98c379".to_string()),
                warning: ThemeColor::Hex("#e5c07b".to_string()),
                error: ThemeColor::Hex("#e06c75".to_string()),
                info: ThemeColor::Hex("#56b6c2".to_string()),
                background: ThemeColor::Hex("#282c34".to_string()),
                foreground: ThemeColor::Hex("#abb2bf".to_string()),
                muted: ThemeColor::Hex("#5c6370".to_string()),
                border: ThemeColor::Hex("#4b5263".to_string()),
                highlight: ThemeColor::Hex("#3e4451".to_string()),
            },
            components: ComponentStyles::default(),
        }
    }

    /// Get the light theme preset
    #[must_use]
    pub fn light() -> Self {
        Self {
            name: "light".to_string(),
            description: Some("Light theme for bright environments".to_string()),
            colors: ThemeColors {
                primary: ThemeColor::Hex("#4078f2".to_string()),
                secondary: ThemeColor::Hex("#0184bc".to_string()),
                success: ThemeColor::Hex("#50a14f".to_string()),
                warning: ThemeColor::Hex("#c18401".to_string()),
                error: ThemeColor::Hex("#e45649".to_string()),
                info: ThemeColor::Hex("#0184bc".to_string()),
                background: ThemeColor::Hex("#fafafa".to_string()),
                foreground: ThemeColor::Hex("#383a42".to_string()),
                muted: ThemeColor::Hex("#a0a1a7".to_string()),
                border: ThemeColor::Hex("#e5e5e6".to_string()),
                highlight: ThemeColor::Hex("#d0d0d0".to_string()),
            },
            components: ComponentStyles::default(),
        }
    }

    /// Get the monokai theme preset
    #[must_use]
    pub fn monokai() -> Self {
        Self {
            name: "monokai".to_string(),
            description: Some("Classic Monokai color scheme".to_string()),
            colors: ThemeColors {
                primary: ThemeColor::Hex("#66d9ef".to_string()),
                secondary: ThemeColor::Hex("#ae81ff".to_string()),
                success: ThemeColor::Hex("#a6e22e".to_string()),
                warning: ThemeColor::Hex("#e6db74".to_string()),
                error: ThemeColor::Hex("#f92672".to_string()),
                info: ThemeColor::Hex("#66d9ef".to_string()),
                background: ThemeColor::Hex("#272822".to_string()),
                foreground: ThemeColor::Hex("#f8f8f2".to_string()),
                muted: ThemeColor::Hex("#75715e".to_string()),
                border: ThemeColor::Hex("#49483e".to_string()),
                highlight: ThemeColor::Hex("#3e3d32".to_string()),
            },
            components: ComponentStyles::default(),
        }
    }

    /// Get a theme by name
    #[must_use]
    pub fn by_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "dark" => Self::dark(),
            "light" => Self::light(),
            "monokai" => Self::monokai(),
            _ => Self::default(),
        }
    }

    /// Get style for primary elements
    #[must_use]
    pub fn primary_style(&self) -> Style {
        Style::default().fg(self.colors.primary.to_color())
    }

    /// Get style for secondary elements
    #[must_use]
    pub fn secondary_style(&self) -> Style {
        Style::default().fg(self.colors.secondary.to_color())
    }

    /// Get style for success elements
    #[must_use]
    pub fn success_style(&self) -> Style {
        Style::default().fg(self.colors.success.to_color())
    }

    /// Get style for warning elements
    #[must_use]
    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.colors.warning.to_color())
    }

    /// Get style for error elements
    #[must_use]
    pub fn error_style(&self) -> Style {
        Style::default().fg(self.colors.error.to_color())
    }

    /// Get style for info elements
    #[must_use]
    pub fn info_style(&self) -> Style {
        Style::default().fg(self.colors.info.to_color())
    }

    /// Get style for muted/dim elements
    #[must_use]
    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.colors.muted.to_color())
    }

    /// Get style for borders
    #[must_use]
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.colors.border.to_color())
    }

    /// Get style for highlighted elements
    #[must_use]
    pub fn highlight_style(&self) -> Style {
        Style::default()
            .bg(self.colors.highlight.to_color())
            .add_modifier(Modifier::BOLD)
    }

    /// Get style for focused borders
    #[must_use]
    pub fn focused_border_style(&self) -> Style {
        Style::default().fg(self.colors.primary.to_color())
    }

    /// Get status icon for a task status
    #[must_use]
    pub fn status_icon(&self, status: &str) -> &str {
        self.components
            .task_tree
            .status_icons
            .get(status)
            .map_or("?", std::string::String::as_str)
    }
}

/// Theme manager for loading and switching themes
pub struct ThemeManager {
    current: Theme,
    themes: HashMap<String, Theme>,
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeManager {
    /// Create a new theme manager with built-in themes
    #[must_use]
    pub fn new() -> Self {
        let mut themes = HashMap::new();
        themes.insert("default".to_string(), Theme::default());
        themes.insert("dark".to_string(), Theme::dark());
        themes.insert("light".to_string(), Theme::light());
        themes.insert("monokai".to_string(), Theme::monokai());

        Self {
            current: Theme::default(),
            themes,
        }
    }

    /// Get the current theme
    #[must_use]
    pub fn current(&self) -> &Theme {
        &self.current
    }

    /// Set the current theme by name
    pub fn set_theme(&mut self, name: &str) -> bool {
        if let Some(theme) = self.themes.get(name) {
            self.current = theme.clone();
            true
        } else {
            false
        }
    }

    /// Register a custom theme
    pub fn register(&mut self, theme: Theme) {
        let name = theme.name.clone();
        self.themes.insert(name, theme);
    }

    /// List available theme names
    #[must_use]
    pub fn available_themes(&self) -> Vec<&str> {
        self.themes
            .keys()
            .map(std::string::String::as_str)
            .collect()
    }

    /// Load a theme from a TOML file
    pub fn load_from_file(&mut self, path: &std::path::Path) -> Result<(), String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read theme file: {e}"))?;
        let theme: Theme =
            toml::from_str(&content).map_err(|e| format!("Failed to parse theme: {e}"))?;
        self.register(theme);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme() {
        let theme = Theme::default();
        assert_eq!(theme.name, "default");
        assert!(theme.description.is_some());
    }

    #[test]
    fn test_theme_by_name() {
        let dark = Theme::by_name("dark");
        assert_eq!(dark.name, "dark");

        let light = Theme::by_name("light");
        assert_eq!(light.name, "light");

        let unknown = Theme::by_name("unknown");
        assert_eq!(unknown.name, "default");
    }

    #[test]
    fn test_color_conversion() {
        let rgb = ThemeColor::Rgb {
            r: 255,
            g: 128,
            b: 0,
        };
        assert_eq!(rgb.to_color(), Color::Rgb(255, 128, 0));

        let hex = ThemeColor::Hex("#ff8000".to_string());
        assert_eq!(hex.to_color(), Color::Rgb(255, 128, 0));

        let named = ThemeColor::Named("red".to_string());
        assert_eq!(named.to_color(), Color::Red);
    }

    #[test]
    fn test_theme_manager() {
        let mut manager = ThemeManager::new();
        assert_eq!(manager.current().name, "default");

        assert!(manager.set_theme("dark"));
        assert_eq!(manager.current().name, "dark");

        assert!(!manager.set_theme("nonexistent"));
        assert_eq!(manager.current().name, "dark"); // unchanged

        let themes = manager.available_themes();
        assert!(themes.contains(&"default"));
        assert!(themes.contains(&"dark"));
        assert!(themes.contains(&"light"));
    }

    #[test]
    fn test_status_icons() {
        let theme = Theme::default();
        assert_eq!(theme.status_icon("running"), "●");
        assert_eq!(theme.status_icon("completed"), "✓");
        assert_eq!(theme.status_icon("unknown"), "?");
    }
}
