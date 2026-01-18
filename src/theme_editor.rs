//! Theme Editor for AgTerm
//!
//! Provides a comprehensive theme editing interface including:
//! - ColorPicker: RGB/HSL/Hex input with presets and recent colors
//! - ThemePreview: Real-time preview with sample terminal text
//! - ThemeEditor: Edit 16-color ANSI palette, 256-color palette, and terminal colors
//! - Theme Import/Export: Support for iTerm2 and VS Code theme formats

use iced::Color;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::Path;

use crate::theme::{AnsiPalette, ColorDef, TerminalColors, Theme, ThemeVariant, UiColors};

// ============================================================================
// Color Picker
// ============================================================================

/// Color picker component with RGB/HSL/Hex input
#[derive(Debug, Clone)]
pub struct ColorPicker {
    /// Current color being edited
    pub color: ColorRgb,
    /// Color input mode (RGB, HSL, or Hex)
    pub input_mode: ColorInputMode,
    /// Recently used colors (max 12)
    pub recent_colors: VecDeque<ColorRgb>,
    /// Text input for hex value
    pub hex_input: String,
    /// RGB component inputs
    pub rgb_input: RgbInput,
    /// HSL component inputs
    pub hsl_input: HslInput,
}

/// RGB color representation (0-255 for each component)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ColorRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// RGB input state
#[derive(Debug, Clone)]
pub struct RgbInput {
    pub r: f32, // 0-255
    pub g: f32, // 0-255
    pub b: f32, // 0-255
}

/// HSL input state
#[derive(Debug, Clone)]
pub struct HslInput {
    pub h: f32, // 0-360
    pub s: f32, // 0-100
    pub l: f32, // 0-100
}

/// Color input mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorInputMode {
    Rgb,
    Hsl,
    Hex,
}

/// Preset color palettes
#[derive(Debug, Clone, Copy)]
pub struct ColorPresets;

impl ColorPresets {
    /// Material Design color palette (subset)
    pub fn material_colors() -> Vec<(&'static str, ColorRgb)> {
        vec![
            ("Red", ColorRgb::from_hex("#F44336")),
            ("Pink", ColorRgb::from_hex("#E91E63")),
            ("Purple", ColorRgb::from_hex("#9C27B0")),
            ("Deep Purple", ColorRgb::from_hex("#673AB7")),
            ("Indigo", ColorRgb::from_hex("#3F51B5")),
            ("Blue", ColorRgb::from_hex("#2196F3")),
            ("Light Blue", ColorRgb::from_hex("#03A9F4")),
            ("Cyan", ColorRgb::from_hex("#00BCD4")),
            ("Teal", ColorRgb::from_hex("#009688")),
            ("Green", ColorRgb::from_hex("#4CAF50")),
            ("Light Green", ColorRgb::from_hex("#8BC34A")),
            ("Lime", ColorRgb::from_hex("#CDDC39")),
            ("Yellow", ColorRgb::from_hex("#FFEB3B")),
            ("Amber", ColorRgb::from_hex("#FFC107")),
            ("Orange", ColorRgb::from_hex("#FF9800")),
            ("Deep Orange", ColorRgb::from_hex("#FF5722")),
        ]
    }

    /// Grayscale palette
    pub fn grayscale() -> Vec<(&'static str, ColorRgb)> {
        vec![
            ("Black", ColorRgb::from_hex("#000000")),
            ("Gray 900", ColorRgb::from_hex("#212121")),
            ("Gray 700", ColorRgb::from_hex("#616161")),
            ("Gray 500", ColorRgb::from_hex("#9E9E9E")),
            ("Gray 300", ColorRgb::from_hex("#E0E0E0")),
            ("White", ColorRgb::from_hex("#FFFFFF")),
        ]
    }
}

impl ColorRgb {
    /// Create from RGB components
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Create from hex string (e.g., "#FF0000" or "FF0000")
    pub fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            Self { r, g, b }
        } else if hex.len() == 3 {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0);
            Self { r, g, b }
        } else {
            Self { r: 0, g: 0, b: 0 }
        }
    }

    /// Convert to hex string (e.g., "#FF0000")
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Convert to Iced Color
    pub fn to_color(&self) -> Color {
        Color::from_rgb(
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        )
    }

    /// Convert from Iced Color
    pub fn from_color(color: Color) -> Self {
        Self {
            r: (color.r * 255.0) as u8,
            g: (color.g * 255.0) as u8,
            b: (color.b * 255.0) as u8,
        }
    }

    /// Convert to ColorDef
    pub fn to_color_def(&self) -> ColorDef {
        ColorDef::from_rgb(self.r, self.g, self.b)
    }

    /// Convert from ColorDef
    pub fn from_color_def(color_def: &ColorDef) -> Self {
        Self::from_color(color_def.to_color())
    }

    /// Convert to HSL
    pub fn to_hsl(&self) -> (f32, f32, f32) {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let l = (max + min) / 2.0;

        if delta == 0.0 {
            return (0.0, 0.0, l * 100.0);
        }

        let s = if l < 0.5 {
            delta / (max + min)
        } else {
            delta / (2.0 - max - min)
        };

        let h = if max == r {
            ((g - b) / delta + if g < b { 6.0 } else { 0.0 }) / 6.0
        } else if max == g {
            ((b - r) / delta + 2.0) / 6.0
        } else {
            ((r - g) / delta + 4.0) / 6.0
        };

        (h * 360.0, s * 100.0, l * 100.0)
    }

    /// Create from HSL (h: 0-360, s: 0-100, l: 0-100)
    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        let h = h / 360.0;
        let s = s / 100.0;
        let l = l / 100.0;

        let hue_to_rgb = |p: f32, q: f32, mut t: f32| {
            if t < 0.0 {
                t += 1.0;
            }
            if t > 1.0 {
                t -= 1.0;
            }
            if t < 1.0 / 6.0 {
                p + (q - p) * 6.0 * t
            } else if t < 1.0 / 2.0 {
                q
            } else if t < 2.0 / 3.0 {
                p + (q - p) * (2.0 / 3.0 - t) * 6.0
            } else {
                p
            }
        };

        if s == 0.0 {
            let gray = (l * 255.0) as u8;
            return Self {
                r: gray,
                g: gray,
                b: gray,
            };
        }

        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;

        let r = (hue_to_rgb(p, q, h + 1.0 / 3.0) * 255.0) as u8;
        let g = (hue_to_rgb(p, q, h) * 255.0) as u8;
        let b = (hue_to_rgb(p, q, h - 1.0 / 3.0) * 255.0) as u8;

        Self { r, g, b }
    }
}

impl Default for ColorPicker {
    fn default() -> Self {
        Self::new(ColorRgb::new(255, 255, 255))
    }
}

impl ColorPicker {
    /// Create new color picker with initial color
    pub fn new(color: ColorRgb) -> Self {
        let (h, s, l) = color.to_hsl();
        Self {
            color,
            input_mode: ColorInputMode::Rgb,
            recent_colors: VecDeque::with_capacity(12),
            hex_input: color.to_hex(),
            rgb_input: RgbInput {
                r: color.r as f32,
                g: color.g as f32,
                b: color.b as f32,
            },
            hsl_input: HslInput { h, s, l },
        }
    }

    /// Set color and update all input fields
    pub fn set_color(&mut self, color: ColorRgb) {
        self.color = color;
        self.hex_input = color.to_hex();
        self.rgb_input = RgbInput {
            r: color.r as f32,
            g: color.g as f32,
            b: color.b as f32,
        };
        let (h, s, l) = color.to_hsl();
        self.hsl_input = HslInput { h, s, l };
    }

    /// Add color to recent colors
    pub fn add_recent_color(&mut self, color: ColorRgb) {
        // Remove if already exists
        self.recent_colors.retain(|c| *c != color);
        // Add to front
        self.recent_colors.push_front(color);
        // Keep max 12 colors
        while self.recent_colors.len() > 12 {
            self.recent_colors.pop_back();
        }
    }

    /// Update color from RGB sliders
    pub fn update_from_rgb(&mut self) {
        self.color = ColorRgb::new(
            self.rgb_input.r as u8,
            self.rgb_input.g as u8,
            self.rgb_input.b as u8,
        );
        self.hex_input = self.color.to_hex();
        let (h, s, l) = self.color.to_hsl();
        self.hsl_input = HslInput { h, s, l };
    }

    /// Update color from HSL sliders
    pub fn update_from_hsl(&mut self) {
        self.color = ColorRgb::from_hsl(self.hsl_input.h, self.hsl_input.s, self.hsl_input.l);
        self.hex_input = self.color.to_hex();
        self.rgb_input = RgbInput {
            r: self.color.r as f32,
            g: self.color.g as f32,
            b: self.color.b as f32,
        };
    }

    /// Update color from hex input
    pub fn update_from_hex(&mut self, hex: String) {
        self.hex_input = hex.clone();
        let trimmed = hex.trim_start_matches('#');
        // Accept 3-char (short) or 6-char (full) hex codes
        if trimmed.len() == 3 || trimmed.len() == 6 {
            let new_color = ColorRgb::from_hex(&hex);
            // Only update if it's a valid color (or explicitly black)
            if new_color.to_hex() != "#000000" || trimmed == "000000" || trimmed == "000" {
                self.color = new_color;
                self.rgb_input = RgbInput {
                    r: self.color.r as f32,
                    g: self.color.g as f32,
                    b: self.color.b as f32,
                };
                let (h, s, l) = self.color.to_hsl();
                self.hsl_input = HslInput { h, s, l };
            }
        }
    }
}

// ============================================================================
// Theme Preview
// ============================================================================

/// Theme preview component showing sample terminal text
#[derive(Debug, Clone)]
pub struct ThemePreview {
    /// Theme to preview
    pub theme: Theme,
    /// Preview sample type
    pub sample_type: PreviewSample,
}

/// Preview sample types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewSample {
    /// Standard ANSI colors
    AnsiColors,
    /// Shell prompt with colors
    ShellPrompt,
    /// Code syntax highlighting
    CodeHighlight,
    /// Git diff output
    GitDiff,
}

impl Default for ThemePreview {
    fn default() -> Self {
        Self {
            theme: Theme::warp_dark(),
            sample_type: PreviewSample::AnsiColors,
        }
    }
}

impl ThemePreview {
    /// Create new preview with theme
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            sample_type: PreviewSample::AnsiColors,
        }
    }

    /// Get sample text for preview
    pub fn get_sample_text(&self) -> Vec<(String, Color)> {
        match self.sample_type {
            PreviewSample::AnsiColors => self.ansi_colors_sample(),
            PreviewSample::ShellPrompt => self.shell_prompt_sample(),
            PreviewSample::CodeHighlight => self.code_highlight_sample(),
            PreviewSample::GitDiff => self.git_diff_sample(),
        }
    }

    fn ansi_colors_sample(&self) -> Vec<(String, Color)> {
        vec![
            ("Black   ".to_string(), self.theme.ansi.black.to_color()),
            ("Red     ".to_string(), self.theme.ansi.red.to_color()),
            ("Green   ".to_string(), self.theme.ansi.green.to_color()),
            ("Yellow  ".to_string(), self.theme.ansi.yellow.to_color()),
            ("Blue    ".to_string(), self.theme.ansi.blue.to_color()),
            (
                "Magenta ".to_string(),
                self.theme.ansi.magenta.to_color(),
            ),
            ("Cyan    ".to_string(), self.theme.ansi.cyan.to_color()),
            ("White   ".to_string(), self.theme.ansi.white.to_color()),
            (
                "Bright Black   ".to_string(),
                self.theme.ansi.bright_black.to_color(),
            ),
            (
                "Bright Red     ".to_string(),
                self.theme.ansi.bright_red.to_color(),
            ),
            (
                "Bright Green   ".to_string(),
                self.theme.ansi.bright_green.to_color(),
            ),
            (
                "Bright Yellow  ".to_string(),
                self.theme.ansi.bright_yellow.to_color(),
            ),
            (
                "Bright Blue    ".to_string(),
                self.theme.ansi.bright_blue.to_color(),
            ),
            (
                "Bright Magenta ".to_string(),
                self.theme.ansi.bright_magenta.to_color(),
            ),
            (
                "Bright Cyan    ".to_string(),
                self.theme.ansi.bright_cyan.to_color(),
            ),
            (
                "Bright White   ".to_string(),
                self.theme.ansi.bright_white.to_color(),
            ),
        ]
    }

    fn shell_prompt_sample(&self) -> Vec<(String, Color)> {
        vec![
            ("user".to_string(), self.theme.ansi.green.to_color()),
            ("@".to_string(), self.theme.terminal.foreground.to_color()),
            ("hostname".to_string(), self.theme.ansi.blue.to_color()),
            (":".to_string(), self.theme.terminal.foreground.to_color()),
            (
                "~/projects/agterm".to_string(),
                self.theme.ansi.cyan.to_color(),
            ),
            (" $ ".to_string(), self.theme.ansi.yellow.to_color()),
            (
                "ls -la".to_string(),
                self.theme.terminal.foreground.to_color(),
            ),
        ]
    }

    fn code_highlight_sample(&self) -> Vec<(String, Color)> {
        vec![
            ("fn ".to_string(), self.theme.ansi.magenta.to_color()),
            ("main".to_string(), self.theme.ansi.blue.to_color()),
            ("() {".to_string(), self.theme.terminal.foreground.to_color()),
            (
                "    println!".to_string(),
                self.theme.ansi.yellow.to_color(),
            ),
            ("(".to_string(), self.theme.terminal.foreground.to_color()),
            (
                "\"Hello, World!\"".to_string(),
                self.theme.ansi.green.to_color(),
            ),
            (");".to_string(), self.theme.terminal.foreground.to_color()),
            ("}".to_string(), self.theme.terminal.foreground.to_color()),
        ]
    }

    fn git_diff_sample(&self) -> Vec<(String, Color)> {
        vec![
            (
                "diff --git a/file.rs b/file.rs".to_string(),
                self.theme.terminal.foreground.to_color(),
            ),
            (
                "- old line".to_string(),
                self.theme.ansi.red.to_color(),
            ),
            (
                "+ new line".to_string(),
                self.theme.ansi.green.to_color(),
            ),
            (
                "  unchanged".to_string(),
                self.theme.terminal.foreground.to_color(),
            ),
        ]
    }
}

// ============================================================================
// Theme Editor
// ============================================================================

/// Theme editor component
#[derive(Debug, Clone)]
pub struct ThemeEditor {
    /// Theme being edited
    pub theme: Theme,
    /// Color picker for current color being edited
    pub color_picker: ColorPicker,
    /// Currently selected color field
    pub selected_field: ColorField,
    /// Theme preview
    pub preview: ThemePreview,
    /// Editor mode
    pub mode: EditorMode,
}

/// Color field being edited
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorField {
    // Terminal colors
    TerminalForeground,
    TerminalBackground,
    TerminalCursor,
    TerminalCursorText,
    TerminalSelection,
    TerminalSelectionText,

    // ANSI colors (0-15)
    AnsiBlack,
    AnsiRed,
    AnsiGreen,
    AnsiYellow,
    AnsiBlue,
    AnsiMagenta,
    AnsiCyan,
    AnsiWhite,
    AnsiBrightBlack,
    AnsiBrightRed,
    AnsiBrightGreen,
    AnsiBrightYellow,
    AnsiBrightBlue,
    AnsiBrightMagenta,
    AnsiBrightCyan,
    AnsiBrightWhite,
}

/// Editor mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    /// Edit ANSI 16-color palette
    Ansi16,
    /// Edit terminal colors
    Terminal,
    /// Edit UI colors
    Ui,
}

impl Default for ThemeEditor {
    fn default() -> Self {
        let theme = Theme::warp_dark();
        Self {
            preview: ThemePreview::new(theme.clone()),
            color_picker: ColorPicker::default(),
            selected_field: ColorField::TerminalForeground,
            theme,
            mode: EditorMode::Ansi16,
        }
    }
}

impl ThemeEditor {
    /// Create new theme editor
    pub fn new(theme: Theme) -> Self {
        Self {
            preview: ThemePreview::new(theme.clone()),
            color_picker: ColorPicker::default(),
            selected_field: ColorField::TerminalForeground,
            theme,
            mode: EditorMode::Ansi16,
        }
    }

    /// Select a color field to edit
    pub fn select_field(&mut self, field: ColorField) {
        self.selected_field = field;
        let color = self.get_field_color(field);
        self.color_picker.set_color(ColorRgb::from_color_def(&color));
    }

    /// Get color for a specific field
    pub fn get_field_color(&self, field: ColorField) -> ColorDef {
        match field {
            ColorField::TerminalForeground => self.theme.terminal.foreground.clone(),
            ColorField::TerminalBackground => self.theme.terminal.background.clone(),
            ColorField::TerminalCursor => self.theme.terminal.cursor.clone(),
            ColorField::TerminalCursorText => self.theme.terminal.cursor_text.clone(),
            ColorField::TerminalSelection => self.theme.terminal.selection.clone(),
            ColorField::TerminalSelectionText => self.theme.terminal.selection_text.clone(),
            ColorField::AnsiBlack => self.theme.ansi.black.clone(),
            ColorField::AnsiRed => self.theme.ansi.red.clone(),
            ColorField::AnsiGreen => self.theme.ansi.green.clone(),
            ColorField::AnsiYellow => self.theme.ansi.yellow.clone(),
            ColorField::AnsiBlue => self.theme.ansi.blue.clone(),
            ColorField::AnsiMagenta => self.theme.ansi.magenta.clone(),
            ColorField::AnsiCyan => self.theme.ansi.cyan.clone(),
            ColorField::AnsiWhite => self.theme.ansi.white.clone(),
            ColorField::AnsiBrightBlack => self.theme.ansi.bright_black.clone(),
            ColorField::AnsiBrightRed => self.theme.ansi.bright_red.clone(),
            ColorField::AnsiBrightGreen => self.theme.ansi.bright_green.clone(),
            ColorField::AnsiBrightYellow => self.theme.ansi.bright_yellow.clone(),
            ColorField::AnsiBrightBlue => self.theme.ansi.bright_blue.clone(),
            ColorField::AnsiBrightMagenta => self.theme.ansi.bright_magenta.clone(),
            ColorField::AnsiBrightCyan => self.theme.ansi.bright_cyan.clone(),
            ColorField::AnsiBrightWhite => self.theme.ansi.bright_white.clone(),
        }
    }

    /// Update color for selected field
    pub fn update_selected_color(&mut self, color: ColorRgb) {
        let color_def = color.to_color_def();
        match self.selected_field {
            ColorField::TerminalForeground => self.theme.terminal.foreground = color_def,
            ColorField::TerminalBackground => self.theme.terminal.background = color_def,
            ColorField::TerminalCursor => self.theme.terminal.cursor = color_def,
            ColorField::TerminalCursorText => self.theme.terminal.cursor_text = color_def,
            ColorField::TerminalSelection => self.theme.terminal.selection = color_def,
            ColorField::TerminalSelectionText => self.theme.terminal.selection_text = color_def,
            ColorField::AnsiBlack => self.theme.ansi.black = color_def,
            ColorField::AnsiRed => self.theme.ansi.red = color_def,
            ColorField::AnsiGreen => self.theme.ansi.green = color_def,
            ColorField::AnsiYellow => self.theme.ansi.yellow = color_def,
            ColorField::AnsiBlue => self.theme.ansi.blue = color_def,
            ColorField::AnsiMagenta => self.theme.ansi.magenta = color_def,
            ColorField::AnsiCyan => self.theme.ansi.cyan = color_def,
            ColorField::AnsiWhite => self.theme.ansi.white = color_def,
            ColorField::AnsiBrightBlack => self.theme.ansi.bright_black = color_def,
            ColorField::AnsiBrightRed => self.theme.ansi.bright_red = color_def,
            ColorField::AnsiBrightGreen => self.theme.ansi.bright_green = color_def,
            ColorField::AnsiBrightYellow => self.theme.ansi.bright_yellow = color_def,
            ColorField::AnsiBrightBlue => self.theme.ansi.bright_blue = color_def,
            ColorField::AnsiBrightMagenta => self.theme.ansi.bright_magenta = color_def,
            ColorField::AnsiBrightCyan => self.theme.ansi.bright_cyan = color_def,
            ColorField::AnsiBrightWhite => self.theme.ansi.bright_white = color_def,
        }
        self.preview.theme = self.theme.clone();
    }

    /// Export theme to file
    pub fn export_theme(&self, path: &Path) -> Result<(), std::io::Error> {
        self.theme.to_toml_file(path)
    }

    /// Import theme from file
    pub fn import_theme(&mut self, path: &Path) -> Result<(), std::io::Error> {
        self.theme = Theme::from_toml_file(path)?;
        self.preview.theme = self.theme.clone();
        Ok(())
    }
}

// ============================================================================
// Theme Format Converters
// ============================================================================

/// iTerm2 theme format support
pub mod iterm2 {
    use super::*;
    use std::collections::HashMap;

    /// iTerm2 color entry in XML plist format
    #[derive(Debug, Clone)]
    pub struct ITermColor {
        pub red: f32,
        pub green: f32,
        pub blue: f32,
    }

    impl ITermColor {
        pub fn to_color_rgb(&self) -> ColorRgb {
            ColorRgb::new(
                (self.red * 255.0) as u8,
                (self.green * 255.0) as u8,
                (self.blue * 255.0) as u8,
            )
        }

        pub fn from_color_rgb(color: &ColorRgb) -> Self {
            Self {
                red: color.r as f32 / 255.0,
                green: color.g as f32 / 255.0,
                blue: color.b as f32 / 255.0,
            }
        }
    }

    /// Parse iTerm2 theme from XML string (simplified parser)
    /// Note: This is a basic implementation. For production use, consider using an XML parser crate.
    pub fn parse_iterm_theme(xml: &str) -> Result<Theme, String> {
        // This is a simplified parser that looks for color patterns
        // In a real implementation, you'd use a proper XML parser like quick-xml

        let mut colors: HashMap<String, ColorRgb> = HashMap::new();

        // Extract theme name (basic implementation)
        let name = xml
            .lines()
            .find(|line| line.contains("<key>name</key>"))
            .and_then(|_| {
                xml.lines()
                    .skip_while(|line| !line.contains("<key>name</key>"))
                    .nth(1)
            })
            .and_then(|line| {
                line.trim()
                    .strip_prefix("<string>")
                    .and_then(|s| s.strip_suffix("</string>"))
            })
            .unwrap_or("Imported Theme")
            .to_string();

        // Extract colors (basic pattern matching)
        // This would need to be more robust in production
        let color_keys = vec![
            ("Ansi 0 Color", "black"),
            ("Ansi 1 Color", "red"),
            ("Ansi 2 Color", "green"),
            ("Ansi 3 Color", "yellow"),
            ("Ansi 4 Color", "blue"),
            ("Ansi 5 Color", "magenta"),
            ("Ansi 6 Color", "cyan"),
            ("Ansi 7 Color", "white"),
            ("Ansi 8 Color", "bright_black"),
            ("Ansi 9 Color", "bright_red"),
            ("Ansi 10 Color", "bright_green"),
            ("Ansi 11 Color", "bright_yellow"),
            ("Ansi 12 Color", "bright_blue"),
            ("Ansi 13 Color", "bright_magenta"),
            ("Ansi 14 Color", "bright_cyan"),
            ("Ansi 15 Color", "bright_white"),
            ("Foreground Color", "foreground"),
            ("Background Color", "background"),
            ("Cursor Color", "cursor"),
        ];

        for (iterm_key, color_name) in color_keys {
            if let Some(color) = extract_iterm_color(xml, iterm_key) {
                colors.insert(color_name.to_string(), color);
            }
        }

        // Build theme from extracted colors
        Ok(build_theme_from_colors(name, colors))
    }

    fn extract_iterm_color(xml: &str, key: &str) -> Option<ColorRgb> {
        // Find the key line
        let key_line = format!("<key>{}</key>", key);
        let lines: Vec<&str> = xml.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if line.contains(&key_line) {
                // Look for color component in next few lines
                let mut red = 0.0;
                let mut green = 0.0;
                let mut blue = 0.0;

                for j in 1..10 {
                    if i + j >= lines.len() {
                        break;
                    }
                    let next_line = lines[i + j];

                    if next_line.contains("<key>Red Component</key>") {
                        if let Some(val) = extract_real_value(lines.get(i + j + 1)?) {
                            red = val;
                        }
                    }
                    if next_line.contains("<key>Green Component</key>") {
                        if let Some(val) = extract_real_value(lines.get(i + j + 1)?) {
                            green = val;
                        }
                    }
                    if next_line.contains("<key>Blue Component</key>") {
                        if let Some(val) = extract_real_value(lines.get(i + j + 1)?) {
                            blue = val;
                        }
                    }
                }

                return Some(ColorRgb::new(
                    (red * 255.0) as u8,
                    (green * 255.0) as u8,
                    (blue * 255.0) as u8,
                ));
            }
        }
        None
    }

    fn extract_real_value(line: &str) -> Option<f32> {
        line.trim()
            .strip_prefix("<real>")?
            .strip_suffix("</real>")?
            .parse()
            .ok()
    }

    fn build_theme_from_colors(name: String, colors: HashMap<String, ColorRgb>) -> Theme {
        let get_color = |key: &str, default: ColorRgb| -> ColorDef {
            colors
                .get(key)
                .unwrap_or(&default)
                .to_color_def()
        };

        Theme {
            name,
            variant: ThemeVariant::Dark,
            ui: UiColors {
                bg_primary: get_color("background", ColorRgb::from_hex("#17171c")),
                bg_secondary: get_color("background", ColorRgb::from_hex("#1e1e26")),
                bg_block: ColorDef::from_hex("#242430"),
                bg_block_hover: ColorDef::from_hex("#2d2d38"),
                bg_input: ColorDef::from_hex("#1c1c24"),
                text_primary: get_color("foreground", ColorRgb::from_hex("#edeff2")),
                text_secondary: ColorDef::from_hex("#999ead"),
                text_muted: ColorDef::from_hex("#737885"),
                accent_blue: get_color("blue", ColorRgb::from_hex("#5c8afa")),
                accent_green: get_color("green", ColorRgb::from_hex("#59c78c")),
                accent_yellow: get_color("yellow", ColorRgb::from_hex("#f2c55c")),
                accent_red: get_color("red", ColorRgb::from_hex("#eb6473")),
                accent_purple: get_color("magenta", ColorRgb::from_hex("#8c5cfa")),
                accent_cyan: get_color("cyan", ColorRgb::from_hex("#5ce6fa")),
                border: ColorDef::from_hex("#383847"),
                tab_active: get_color("blue", ColorRgb::from_hex("#5c8afa")),
                selection: ColorDef::from_rgba(92, 138, 250, 0.3),
            },
            ansi: AnsiPalette {
                black: get_color("black", ColorRgb::from_hex("#000000")),
                red: get_color("red", ColorRgb::from_hex("#ff0000")),
                green: get_color("green", ColorRgb::from_hex("#00ff00")),
                yellow: get_color("yellow", ColorRgb::from_hex("#ffff00")),
                blue: get_color("blue", ColorRgb::from_hex("#0000ff")),
                magenta: get_color("magenta", ColorRgb::from_hex("#ff00ff")),
                cyan: get_color("cyan", ColorRgb::from_hex("#00ffff")),
                white: get_color("white", ColorRgb::from_hex("#ffffff")),
                bright_black: get_color("bright_black", ColorRgb::from_hex("#808080")),
                bright_red: get_color("bright_red", ColorRgb::from_hex("#ff8080")),
                bright_green: get_color("bright_green", ColorRgb::from_hex("#80ff80")),
                bright_yellow: get_color("bright_yellow", ColorRgb::from_hex("#ffff80")),
                bright_blue: get_color("bright_blue", ColorRgb::from_hex("#8080ff")),
                bright_magenta: get_color("bright_magenta", ColorRgb::from_hex("#ff80ff")),
                bright_cyan: get_color("bright_cyan", ColorRgb::from_hex("#80ffff")),
                bright_white: get_color("bright_white", ColorRgb::from_hex("#ffffff")),
            },
            terminal: TerminalColors {
                foreground: get_color("foreground", ColorRgb::from_hex("#edeff2")),
                background: get_color("background", ColorRgb::from_hex("#1e1e26")),
                cursor: get_color("cursor", ColorRgb::from_hex("#5c8afa")),
                cursor_text: get_color("background", ColorRgb::from_hex("#17171c")),
                selection: ColorDef::from_rgba(92, 138, 250, 0.3),
                selection_text: get_color("foreground", ColorRgb::from_hex("#edeff2")),
            },
        }
    }

    /// Export theme to iTerm2 format
    pub fn export_iterm_theme(theme: &Theme) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>name</key>
    <string>{}</string>
    {}
</dict>
</plist>"#,
            theme.name,
            generate_iterm_colors(theme)
        )
    }

    fn generate_iterm_colors(theme: &Theme) -> String {
        let mut output = String::new();

        let colors = vec![
            ("Ansi 0 Color", &theme.ansi.black),
            ("Ansi 1 Color", &theme.ansi.red),
            ("Ansi 2 Color", &theme.ansi.green),
            ("Ansi 3 Color", &theme.ansi.yellow),
            ("Ansi 4 Color", &theme.ansi.blue),
            ("Ansi 5 Color", &theme.ansi.magenta),
            ("Ansi 6 Color", &theme.ansi.cyan),
            ("Ansi 7 Color", &theme.ansi.white),
            ("Ansi 8 Color", &theme.ansi.bright_black),
            ("Ansi 9 Color", &theme.ansi.bright_red),
            ("Ansi 10 Color", &theme.ansi.bright_green),
            ("Ansi 11 Color", &theme.ansi.bright_yellow),
            ("Ansi 12 Color", &theme.ansi.bright_blue),
            ("Ansi 13 Color", &theme.ansi.bright_magenta),
            ("Ansi 14 Color", &theme.ansi.bright_cyan),
            ("Ansi 15 Color", &theme.ansi.bright_white),
            ("Foreground Color", &theme.terminal.foreground),
            ("Background Color", &theme.terminal.background),
            ("Cursor Color", &theme.terminal.cursor),
        ];

        for (key, color_def) in colors {
            let color = ColorRgb::from_color_def(color_def);
            let iterm_color = ITermColor::from_color_rgb(&color);
            output.push_str(&format!(
                r#"    <key>{}</key>
    <dict>
        <key>Red Component</key>
        <real>{}</real>
        <key>Green Component</key>
        <real>{}</real>
        <key>Blue Component</key>
        <real>{}</real>
    </dict>
"#,
                key, iterm_color.red, iterm_color.green, iterm_color.blue
            ));
        }

        output
    }
}

/// VS Code theme format support
pub mod vscode {
    use super::*;

    /// VS Code theme structure (simplified)
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct VSCodeTheme {
        pub name: String,
        #[serde(rename = "type")]
        pub theme_type: String,
        pub colors: VSCodeColors,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct VSCodeColors {
        #[serde(rename = "terminal.foreground")]
        pub terminal_foreground: Option<String>,
        #[serde(rename = "terminal.background")]
        pub terminal_background: Option<String>,
        #[serde(rename = "terminal.ansiBlack")]
        pub terminal_ansi_black: Option<String>,
        #[serde(rename = "terminal.ansiRed")]
        pub terminal_ansi_red: Option<String>,
        #[serde(rename = "terminal.ansiGreen")]
        pub terminal_ansi_green: Option<String>,
        #[serde(rename = "terminal.ansiYellow")]
        pub terminal_ansi_yellow: Option<String>,
        #[serde(rename = "terminal.ansiBlue")]
        pub terminal_ansi_blue: Option<String>,
        #[serde(rename = "terminal.ansiMagenta")]
        pub terminal_ansi_magenta: Option<String>,
        #[serde(rename = "terminal.ansiCyan")]
        pub terminal_ansi_cyan: Option<String>,
        #[serde(rename = "terminal.ansiWhite")]
        pub terminal_ansi_white: Option<String>,
        #[serde(rename = "terminal.ansiBrightBlack")]
        pub terminal_ansi_bright_black: Option<String>,
        #[serde(rename = "terminal.ansiBrightRed")]
        pub terminal_ansi_bright_red: Option<String>,
        #[serde(rename = "terminal.ansiBrightGreen")]
        pub terminal_ansi_bright_green: Option<String>,
        #[serde(rename = "terminal.ansiBrightYellow")]
        pub terminal_ansi_bright_yellow: Option<String>,
        #[serde(rename = "terminal.ansiBrightBlue")]
        pub terminal_ansi_bright_blue: Option<String>,
        #[serde(rename = "terminal.ansiBrightMagenta")]
        pub terminal_ansi_bright_magenta: Option<String>,
        #[serde(rename = "terminal.ansiBrightCyan")]
        pub terminal_ansi_bright_cyan: Option<String>,
        #[serde(rename = "terminal.ansiBrightWhite")]
        pub terminal_ansi_bright_white: Option<String>,
    }

    /// Parse VS Code theme from JSON string
    pub fn parse_vscode_theme(json_str: &str) -> Result<Theme, String> {
        let vscode_theme: VSCodeTheme =
            serde_json::from_str(json_str).map_err(|e| e.to_string())?;

        let variant = match vscode_theme.theme_type.as_str() {
            "light" => ThemeVariant::Light,
            _ => ThemeVariant::Dark,
        };

        let get_color = |opt: &Option<String>, default: &str| -> ColorDef {
            opt.as_ref()
                .map(|s| ColorRgb::from_hex(s).to_color_def())
                .unwrap_or_else(|| ColorDef::from_hex(default))
        };

        Ok(Theme {
            name: vscode_theme.name,
            variant,
            ui: UiColors {
                bg_primary: get_color(&vscode_theme.colors.terminal_background, "#17171c"),
                bg_secondary: ColorDef::from_hex("#1e1e26"),
                bg_block: ColorDef::from_hex("#242430"),
                bg_block_hover: ColorDef::from_hex("#2d2d38"),
                bg_input: ColorDef::from_hex("#1c1c24"),
                text_primary: get_color(&vscode_theme.colors.terminal_foreground, "#edeff2"),
                text_secondary: ColorDef::from_hex("#999ead"),
                text_muted: ColorDef::from_hex("#737885"),
                accent_blue: get_color(&vscode_theme.colors.terminal_ansi_blue, "#5c8afa"),
                accent_green: get_color(&vscode_theme.colors.terminal_ansi_green, "#59c78c"),
                accent_yellow: get_color(&vscode_theme.colors.terminal_ansi_yellow, "#f2c55c"),
                accent_red: get_color(&vscode_theme.colors.terminal_ansi_red, "#eb6473"),
                accent_purple: get_color(&vscode_theme.colors.terminal_ansi_magenta, "#8c5cfa"),
                accent_cyan: get_color(&vscode_theme.colors.terminal_ansi_cyan, "#5ce6fa"),
                border: ColorDef::from_hex("#383847"),
                tab_active: get_color(&vscode_theme.colors.terminal_ansi_blue, "#5c8afa"),
                selection: ColorDef::from_rgba(92, 138, 250, 0.3),
            },
            ansi: AnsiPalette {
                black: get_color(&vscode_theme.colors.terminal_ansi_black, "#000000"),
                red: get_color(&vscode_theme.colors.terminal_ansi_red, "#ff0000"),
                green: get_color(&vscode_theme.colors.terminal_ansi_green, "#00ff00"),
                yellow: get_color(&vscode_theme.colors.terminal_ansi_yellow, "#ffff00"),
                blue: get_color(&vscode_theme.colors.terminal_ansi_blue, "#0000ff"),
                magenta: get_color(&vscode_theme.colors.terminal_ansi_magenta, "#ff00ff"),
                cyan: get_color(&vscode_theme.colors.terminal_ansi_cyan, "#00ffff"),
                white: get_color(&vscode_theme.colors.terminal_ansi_white, "#ffffff"),
                bright_black: get_color(&vscode_theme.colors.terminal_ansi_bright_black, "#808080"),
                bright_red: get_color(&vscode_theme.colors.terminal_ansi_bright_red, "#ff8080"),
                bright_green: get_color(&vscode_theme.colors.terminal_ansi_bright_green, "#80ff80"),
                bright_yellow: get_color(&vscode_theme.colors.terminal_ansi_bright_yellow, "#ffff80"),
                bright_blue: get_color(&vscode_theme.colors.terminal_ansi_bright_blue, "#8080ff"),
                bright_magenta: get_color(&vscode_theme.colors.terminal_ansi_bright_magenta, "#ff80ff"),
                bright_cyan: get_color(&vscode_theme.colors.terminal_ansi_bright_cyan, "#80ffff"),
                bright_white: get_color(&vscode_theme.colors.terminal_ansi_bright_white, "#ffffff"),
            },
            terminal: TerminalColors {
                foreground: get_color(&vscode_theme.colors.terminal_foreground, "#edeff2"),
                background: get_color(&vscode_theme.colors.terminal_background, "#1e1e26"),
                cursor: get_color(&vscode_theme.colors.terminal_ansi_blue, "#5c8afa"),
                cursor_text: get_color(&vscode_theme.colors.terminal_background, "#17171c"),
                selection: ColorDef::from_rgba(92, 138, 250, 0.3),
                selection_text: get_color(&vscode_theme.colors.terminal_foreground, "#edeff2"),
            },
        })
    }

    /// Export theme to VS Code format
    pub fn export_vscode_theme(theme: &Theme) -> Result<String, String> {
        let vscode_theme = VSCodeTheme {
            name: theme.name.clone(),
            theme_type: match theme.variant {
                ThemeVariant::Dark => "dark".to_string(),
                ThemeVariant::Light => "light".to_string(),
            },
            colors: VSCodeColors {
                terminal_foreground: Some(ColorRgb::from_color_def(&theme.terminal.foreground).to_hex()),
                terminal_background: Some(ColorRgb::from_color_def(&theme.terminal.background).to_hex()),
                terminal_ansi_black: Some(ColorRgb::from_color_def(&theme.ansi.black).to_hex()),
                terminal_ansi_red: Some(ColorRgb::from_color_def(&theme.ansi.red).to_hex()),
                terminal_ansi_green: Some(ColorRgb::from_color_def(&theme.ansi.green).to_hex()),
                terminal_ansi_yellow: Some(ColorRgb::from_color_def(&theme.ansi.yellow).to_hex()),
                terminal_ansi_blue: Some(ColorRgb::from_color_def(&theme.ansi.blue).to_hex()),
                terminal_ansi_magenta: Some(ColorRgb::from_color_def(&theme.ansi.magenta).to_hex()),
                terminal_ansi_cyan: Some(ColorRgb::from_color_def(&theme.ansi.cyan).to_hex()),
                terminal_ansi_white: Some(ColorRgb::from_color_def(&theme.ansi.white).to_hex()),
                terminal_ansi_bright_black: Some(ColorRgb::from_color_def(&theme.ansi.bright_black).to_hex()),
                terminal_ansi_bright_red: Some(ColorRgb::from_color_def(&theme.ansi.bright_red).to_hex()),
                terminal_ansi_bright_green: Some(ColorRgb::from_color_def(&theme.ansi.bright_green).to_hex()),
                terminal_ansi_bright_yellow: Some(ColorRgb::from_color_def(&theme.ansi.bright_yellow).to_hex()),
                terminal_ansi_bright_blue: Some(ColorRgb::from_color_def(&theme.ansi.bright_blue).to_hex()),
                terminal_ansi_bright_magenta: Some(ColorRgb::from_color_def(&theme.ansi.bright_magenta).to_hex()),
                terminal_ansi_bright_cyan: Some(ColorRgb::from_color_def(&theme.ansi.bright_cyan).to_hex()),
                terminal_ansi_bright_white: Some(ColorRgb::from_color_def(&theme.ansi.bright_white).to_hex()),
            },
        };

        serde_json::to_string_pretty(&vscode_theme).map_err(|e| e.to_string())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_rgb_from_hex() {
        let color = ColorRgb::from_hex("#FF0000");
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);

        let color = ColorRgb::from_hex("00FF00");
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 255);
        assert_eq!(color.b, 0);

        let color = ColorRgb::from_hex("#F00");
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_color_rgb_to_hex() {
        let color = ColorRgb::new(255, 0, 0);
        assert_eq!(color.to_hex(), "#FF0000");

        let color = ColorRgb::new(0, 255, 0);
        assert_eq!(color.to_hex(), "#00FF00");
    }

    #[test]
    fn test_color_rgb_to_hsl() {
        let red = ColorRgb::new(255, 0, 0);
        let (h, s, l) = red.to_hsl();
        assert!((h - 0.0).abs() < 1.0);
        assert!((s - 100.0).abs() < 1.0);
        assert!((l - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_color_rgb_from_hsl() {
        let color = ColorRgb::from_hsl(0.0, 100.0, 50.0); // Red
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);

        let color = ColorRgb::from_hsl(120.0, 100.0, 50.0); // Green
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 255);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_color_rgb_to_color() {
        let color = ColorRgb::new(255, 0, 0);
        let iced_color = color.to_color();
        assert_eq!(iced_color, Color::from_rgb(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_color_picker_new() {
        let picker = ColorPicker::new(ColorRgb::new(255, 0, 0));
        assert_eq!(picker.color.r, 255);
        assert_eq!(picker.color.g, 0);
        assert_eq!(picker.color.b, 0);
        assert_eq!(picker.hex_input, "#FF0000");
    }

    #[test]
    fn test_color_picker_set_color() {
        let mut picker = ColorPicker::default();
        picker.set_color(ColorRgb::new(0, 255, 0));
        assert_eq!(picker.color.r, 0);
        assert_eq!(picker.color.g, 255);
        assert_eq!(picker.color.b, 0);
        assert_eq!(picker.hex_input, "#00FF00");
    }

    #[test]
    fn test_color_picker_recent_colors() {
        let mut picker = ColorPicker::default();

        // Add colors
        picker.add_recent_color(ColorRgb::new(255, 0, 0));
        picker.add_recent_color(ColorRgb::new(0, 255, 0));
        picker.add_recent_color(ColorRgb::new(0, 0, 255));

        assert_eq!(picker.recent_colors.len(), 3);
        assert_eq!(picker.recent_colors[0], ColorRgb::new(0, 0, 255)); // Most recent first

        // Add duplicate
        picker.add_recent_color(ColorRgb::new(255, 0, 0));
        assert_eq!(picker.recent_colors.len(), 3); // Should still be 3
        assert_eq!(picker.recent_colors[0], ColorRgb::new(255, 0, 0)); // Moved to front
    }

    #[test]
    fn test_color_picker_update_from_rgb() {
        let mut picker = ColorPicker::default();
        picker.rgb_input = RgbInput {
            r: 128.0,
            g: 64.0,
            b: 32.0,
        };
        picker.update_from_rgb();

        assert_eq!(picker.color.r, 128);
        assert_eq!(picker.color.g, 64);
        assert_eq!(picker.color.b, 32);
        assert_eq!(picker.hex_input, "#804020");
    }

    #[test]
    fn test_theme_editor_new() {
        let theme = Theme::warp_dark();
        let editor = ThemeEditor::new(theme.clone());
        assert_eq!(editor.theme.name, theme.name);
        assert_eq!(editor.mode, EditorMode::Ansi16);
    }

    #[test]
    fn test_theme_editor_select_field() {
        let mut editor = ThemeEditor::default();
        editor.select_field(ColorField::AnsiRed);
        assert_eq!(editor.selected_field, ColorField::AnsiRed);
    }

    #[test]
    fn test_theme_editor_update_color() {
        let mut editor = ThemeEditor::default();
        editor.select_field(ColorField::AnsiRed);

        let new_color = ColorRgb::new(128, 0, 0);
        editor.update_selected_color(new_color);

        let updated_color = ColorRgb::from_color_def(&editor.theme.ansi.red);
        assert_eq!(updated_color.r, 128);
    }

    #[test]
    fn test_theme_preview_sample_types() {
        let preview = ThemePreview::default();

        let ansi_sample = preview.get_sample_text();
        assert!(!ansi_sample.is_empty());
    }

    #[test]
    fn test_color_presets() {
        let material = ColorPresets::material_colors();
        assert!(!material.is_empty());

        let grayscale = ColorPresets::grayscale();
        assert!(!grayscale.is_empty());
    }

    #[test]
    fn test_vscode_export_import() {
        let theme = Theme::warp_dark();

        // Export to VS Code format
        let vscode_json = vscode::export_vscode_theme(&theme).unwrap();
        assert!(vscode_json.contains("terminal.foreground"));

        // Import back
        let imported_theme = vscode::parse_vscode_theme(&vscode_json).unwrap();
        assert_eq!(imported_theme.name, theme.name);
    }

    #[test]
    fn test_iterm2_export() {
        let theme = Theme::warp_dark();
        let iterm_xml = iterm2::export_iterm_theme(&theme);

        assert!(iterm_xml.contains("<?xml version"));
        assert!(iterm_xml.contains("Ansi 0 Color"));
        assert!(iterm_xml.contains(&theme.name));
    }
}
