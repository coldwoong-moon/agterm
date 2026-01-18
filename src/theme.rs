//! AgTerm Theme System
//!
//! Provides comprehensive theming support including:
//! - Dark/Light theme structures
//! - 16-color ANSI palette customization
//! - Popular theme presets (Dracula, Solarized, Nord, One Dark, etc.)
//! - Configuration file loading support
//! - Iced color conversion utilities

use iced::Color;
use serde::{Deserialize, Serialize};

// ============================================================================
// Theme Structure
// ============================================================================

/// Complete theme configuration for AgTerm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Theme metadata
    pub name: String,
    pub variant: ThemeVariant,

    /// UI colors
    pub ui: UiColors,

    /// 16-color ANSI palette
    pub ansi: AnsiPalette,

    /// Terminal-specific colors
    pub terminal: TerminalColors,
}

/// Theme variant (dark or light)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeVariant {
    Dark,
    Light,
}

/// UI element colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiColors {
    // Background colors
    pub bg_primary: ColorDef,
    pub bg_secondary: ColorDef,
    pub bg_block: ColorDef,
    pub bg_block_hover: ColorDef,
    pub bg_input: ColorDef,

    // Text colors
    pub text_primary: ColorDef,
    pub text_secondary: ColorDef,
    pub text_muted: ColorDef,

    // Accent colors
    pub accent_blue: ColorDef,
    pub accent_green: ColorDef,
    pub accent_yellow: ColorDef,
    pub accent_red: ColorDef,
    pub accent_purple: ColorDef,
    pub accent_cyan: ColorDef,

    // UI elements
    pub border: ColorDef,
    pub tab_active: ColorDef,
    pub selection: ColorDef,
}

/// Terminal-specific colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalColors {
    pub foreground: ColorDef,
    pub background: ColorDef,
    pub cursor: ColorDef,
    pub cursor_text: ColorDef,
    pub selection: ColorDef,
    pub selection_text: ColorDef,
}

/// 16-color ANSI palette (standard + bright variants)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnsiPalette {
    // Normal colors (0-7)
    pub black: ColorDef,
    pub red: ColorDef,
    pub green: ColorDef,
    pub yellow: ColorDef,
    pub blue: ColorDef,
    pub magenta: ColorDef,
    pub cyan: ColorDef,
    pub white: ColorDef,

    // Bright colors (8-15)
    pub bright_black: ColorDef,
    pub bright_red: ColorDef,
    pub bright_green: ColorDef,
    pub bright_yellow: ColorDef,
    pub bright_blue: ColorDef,
    pub bright_magenta: ColorDef,
    pub bright_cyan: ColorDef,
    pub bright_white: ColorDef,
}

/// Color definition (supports hex strings and RGB tuples)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColorDef {
    /// Hex string (e.g., "#17171c" or "17171c")
    Hex(String),
    /// RGB array [r, g, b] where each component is 0-255
    Rgb([u8; 3]),
    /// RGB float array [r, g, b] where each component is 0.0-1.0
    RgbFloat([f32; 3]),
}

// ============================================================================
// Color Conversion
// ============================================================================

impl ColorDef {
    /// Convert to Iced Color
    pub fn to_color(&self) -> Color {
        match self {
            ColorDef::Hex(hex) => Self::hex_to_color(hex),
            ColorDef::Rgb([r, g, b]) => {
                Color::from_rgb(*r as f32 / 255.0, *g as f32 / 255.0, *b as f32 / 255.0)
            }
            ColorDef::RgbFloat([r, g, b]) => Color::from_rgb(*r, *g, *b),
        }
    }

    /// Parse hex color string to Iced Color
    fn hex_to_color(hex: &str) -> Color {
        let hex = hex.trim_start_matches('#');

        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
        } else if hex.len() == 3 {
            // Short hex format (#RGB -> #RRGGBB)
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0);
            Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
        } else {
            // Invalid format, return black
            Color::BLACK
        }
    }

    /// Create from hex string
    pub fn from_hex(hex: &str) -> Self {
        ColorDef::Hex(hex.to_string())
    }

    /// Create from RGB bytes (0-255)
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        ColorDef::Rgb([r, g, b])
    }

    /// Create from RGB floats (0.0-1.0)
    pub fn from_rgb_float(r: f32, g: f32, b: f32) -> Self {
        ColorDef::RgbFloat([r, g, b])
    }
}

// ============================================================================
// ANSI Color Index Mapping
// ============================================================================

impl AnsiPalette {
    /// Get color by ANSI index (0-15)
    pub fn get_color(&self, index: u8) -> Color {
        match index {
            0 => self.black.to_color(),
            1 => self.red.to_color(),
            2 => self.green.to_color(),
            3 => self.yellow.to_color(),
            4 => self.blue.to_color(),
            5 => self.magenta.to_color(),
            6 => self.cyan.to_color(),
            7 => self.white.to_color(),
            8 => self.bright_black.to_color(),
            9 => self.bright_red.to_color(),
            10 => self.bright_green.to_color(),
            11 => self.bright_yellow.to_color(),
            12 => self.bright_blue.to_color(),
            13 => self.bright_magenta.to_color(),
            14 => self.bright_cyan.to_color(),
            15 => self.bright_white.to_color(),
            _ => Color::WHITE, // Fallback
        }
    }
}

// ============================================================================
// Theme Presets
// ============================================================================

impl Theme {
    /// Warp-inspired dark theme (default)
    pub fn warp_dark() -> Self {
        Self {
            name: "Warp Dark".to_string(),
            variant: ThemeVariant::Dark,
            ui: UiColors {
                bg_primary: ColorDef::from_hex("#17171c"),
                bg_secondary: ColorDef::from_hex("#1e1e26"),
                bg_block: ColorDef::from_hex("#242430"),
                bg_block_hover: ColorDef::from_hex("#2d2d38"),
                bg_input: ColorDef::from_hex("#1c1c24"),
                text_primary: ColorDef::from_hex("#edeff2"),
                text_secondary: ColorDef::from_hex("#999ead"),
                text_muted: ColorDef::from_hex("#737885"),
                accent_blue: ColorDef::from_hex("#5c8afa"),
                accent_green: ColorDef::from_hex("#59c78c"),
                accent_yellow: ColorDef::from_hex("#f2c55c"),
                accent_red: ColorDef::from_hex("#eb6473"),
                accent_purple: ColorDef::from_hex("#8c5cfa"),
                accent_cyan: ColorDef::from_hex("#5ce6fa"),
                border: ColorDef::from_hex("#383847"),
                tab_active: ColorDef::from_hex("#5c8afa"),
                selection: ColorDef::from_rgba(92, 138, 250, 0.3),
            },
            ansi: AnsiPalette {
                black: ColorDef::from_hex("#17171c"),
                red: ColorDef::from_hex("#eb6473"),
                green: ColorDef::from_hex("#59c78c"),
                yellow: ColorDef::from_hex("#f2c55c"),
                blue: ColorDef::from_hex("#5c8afa"),
                magenta: ColorDef::from_hex("#8c5cfa"),
                cyan: ColorDef::from_hex("#5ce6fa"),
                white: ColorDef::from_hex("#edeff2"),
                bright_black: ColorDef::from_hex("#737885"),
                bright_red: ColorDef::from_hex("#ff7f8a"),
                bright_green: ColorDef::from_hex("#6fe0a3"),
                bright_yellow: ColorDef::from_hex("#ffd973"),
                bright_blue: ColorDef::from_hex("#73a1ff"),
                bright_magenta: ColorDef::from_hex("#a373ff"),
                bright_cyan: ColorDef::from_hex("#73f3ff"),
                bright_white: ColorDef::from_hex("#ffffff"),
            },
            terminal: TerminalColors {
                foreground: ColorDef::from_hex("#edeff2"),
                background: ColorDef::from_hex("#1e1e26"),
                cursor: ColorDef::from_hex("#5c8afa"),
                cursor_text: ColorDef::from_hex("#17171c"),
                selection: ColorDef::from_rgba(92, 138, 250, 0.3),
                selection_text: ColorDef::from_hex("#edeff2"),
            },
        }
    }

    /// Dracula theme
    pub fn dracula() -> Self {
        Self {
            name: "Dracula".to_string(),
            variant: ThemeVariant::Dark,
            ui: UiColors {
                bg_primary: ColorDef::from_hex("#21222c"),
                bg_secondary: ColorDef::from_hex("#282a36"),
                bg_block: ColorDef::from_hex("#343746"),
                bg_block_hover: ColorDef::from_hex("#44475a"),
                bg_input: ColorDef::from_hex("#282a36"),
                text_primary: ColorDef::from_hex("#f8f8f2"),
                text_secondary: ColorDef::from_hex("#6272a4"),
                text_muted: ColorDef::from_hex("#4d5066"),
                accent_blue: ColorDef::from_hex("#8be9fd"),
                accent_green: ColorDef::from_hex("#50fa7b"),
                accent_yellow: ColorDef::from_hex("#f1fa8c"),
                accent_red: ColorDef::from_hex("#ff5555"),
                accent_purple: ColorDef::from_hex("#bd93f9"),
                accent_cyan: ColorDef::from_hex("#8be9fd"),
                border: ColorDef::from_hex("#44475a"),
                tab_active: ColorDef::from_hex("#bd93f9"),
                selection: ColorDef::from_rgba(189, 147, 249, 0.3),
            },
            ansi: AnsiPalette {
                black: ColorDef::from_hex("#21222c"),
                red: ColorDef::from_hex("#ff5555"),
                green: ColorDef::from_hex("#50fa7b"),
                yellow: ColorDef::from_hex("#f1fa8c"),
                blue: ColorDef::from_hex("#bd93f9"),
                magenta: ColorDef::from_hex("#ff79c6"),
                cyan: ColorDef::from_hex("#8be9fd"),
                white: ColorDef::from_hex("#f8f8f2"),
                bright_black: ColorDef::from_hex("#6272a4"),
                bright_red: ColorDef::from_hex("#ff6e6e"),
                bright_green: ColorDef::from_hex("#69ff94"),
                bright_yellow: ColorDef::from_hex("#ffffa5"),
                bright_blue: ColorDef::from_hex("#d6acff"),
                bright_magenta: ColorDef::from_hex("#ff92df"),
                bright_cyan: ColorDef::from_hex("#a4ffff"),
                bright_white: ColorDef::from_hex("#ffffff"),
            },
            terminal: TerminalColors {
                foreground: ColorDef::from_hex("#f8f8f2"),
                background: ColorDef::from_hex("#282a36"),
                cursor: ColorDef::from_hex("#bd93f9"),
                cursor_text: ColorDef::from_hex("#282a36"),
                selection: ColorDef::from_rgba(189, 147, 249, 0.3),
                selection_text: ColorDef::from_hex("#f8f8f2"),
            },
        }
    }

    /// Solarized Dark theme
    pub fn solarized_dark() -> Self {
        Self {
            name: "Solarized Dark".to_string(),
            variant: ThemeVariant::Dark,
            ui: UiColors {
                bg_primary: ColorDef::from_hex("#002b36"),
                bg_secondary: ColorDef::from_hex("#073642"),
                bg_block: ColorDef::from_hex("#094352"),
                bg_block_hover: ColorDef::from_hex("#0e5565"),
                bg_input: ColorDef::from_hex("#073642"),
                text_primary: ColorDef::from_hex("#839496"),
                text_secondary: ColorDef::from_hex("#586e75"),
                text_muted: ColorDef::from_hex("#475b62"),
                accent_blue: ColorDef::from_hex("#268bd2"),
                accent_green: ColorDef::from_hex("#859900"),
                accent_yellow: ColorDef::from_hex("#b58900"),
                accent_red: ColorDef::from_hex("#dc322f"),
                accent_purple: ColorDef::from_hex("#6c71c4"),
                accent_cyan: ColorDef::from_hex("#2aa198"),
                border: ColorDef::from_hex("#094352"),
                tab_active: ColorDef::from_hex("#268bd2"),
                selection: ColorDef::from_rgba(38, 139, 210, 0.3),
            },
            ansi: AnsiPalette {
                black: ColorDef::from_hex("#073642"),
                red: ColorDef::from_hex("#dc322f"),
                green: ColorDef::from_hex("#859900"),
                yellow: ColorDef::from_hex("#b58900"),
                blue: ColorDef::from_hex("#268bd2"),
                magenta: ColorDef::from_hex("#d33682"),
                cyan: ColorDef::from_hex("#2aa198"),
                white: ColorDef::from_hex("#eee8d5"),
                bright_black: ColorDef::from_hex("#002b36"),
                bright_red: ColorDef::from_hex("#cb4b16"),
                bright_green: ColorDef::from_hex("#586e75"),
                bright_yellow: ColorDef::from_hex("#657b83"),
                bright_blue: ColorDef::from_hex("#839496"),
                bright_magenta: ColorDef::from_hex("#6c71c4"),
                bright_cyan: ColorDef::from_hex("#93a1a1"),
                bright_white: ColorDef::from_hex("#fdf6e3"),
            },
            terminal: TerminalColors {
                foreground: ColorDef::from_hex("#839496"),
                background: ColorDef::from_hex("#002b36"),
                cursor: ColorDef::from_hex("#268bd2"),
                cursor_text: ColorDef::from_hex("#002b36"),
                selection: ColorDef::from_rgba(38, 139, 210, 0.3),
                selection_text: ColorDef::from_hex("#839496"),
            },
        }
    }

    /// Solarized Light theme
    pub fn solarized_light() -> Self {
        Self {
            name: "Solarized Light".to_string(),
            variant: ThemeVariant::Light,
            ui: UiColors {
                bg_primary: ColorDef::from_hex("#fdf6e3"),
                bg_secondary: ColorDef::from_hex("#eee8d5"),
                bg_block: ColorDef::from_hex("#e4ddc8"),
                bg_block_hover: ColorDef::from_hex("#d9d2bb"),
                bg_input: ColorDef::from_hex("#eee8d5"),
                text_primary: ColorDef::from_hex("#657b83"),
                text_secondary: ColorDef::from_hex("#93a1a1"),
                text_muted: ColorDef::from_hex("#a6b4b9"),
                accent_blue: ColorDef::from_hex("#268bd2"),
                accent_green: ColorDef::from_hex("#859900"),
                accent_yellow: ColorDef::from_hex("#b58900"),
                accent_red: ColorDef::from_hex("#dc322f"),
                accent_purple: ColorDef::from_hex("#6c71c4"),
                accent_cyan: ColorDef::from_hex("#2aa198"),
                border: ColorDef::from_hex("#d9d2bb"),
                tab_active: ColorDef::from_hex("#268bd2"),
                selection: ColorDef::from_rgba(38, 139, 210, 0.3),
            },
            ansi: AnsiPalette {
                black: ColorDef::from_hex("#073642"),
                red: ColorDef::from_hex("#dc322f"),
                green: ColorDef::from_hex("#859900"),
                yellow: ColorDef::from_hex("#b58900"),
                blue: ColorDef::from_hex("#268bd2"),
                magenta: ColorDef::from_hex("#d33682"),
                cyan: ColorDef::from_hex("#2aa198"),
                white: ColorDef::from_hex("#eee8d5"),
                bright_black: ColorDef::from_hex("#002b36"),
                bright_red: ColorDef::from_hex("#cb4b16"),
                bright_green: ColorDef::from_hex("#586e75"),
                bright_yellow: ColorDef::from_hex("#657b83"),
                bright_blue: ColorDef::from_hex("#839496"),
                bright_magenta: ColorDef::from_hex("#6c71c4"),
                bright_cyan: ColorDef::from_hex("#93a1a1"),
                bright_white: ColorDef::from_hex("#fdf6e3"),
            },
            terminal: TerminalColors {
                foreground: ColorDef::from_hex("#657b83"),
                background: ColorDef::from_hex("#fdf6e3"),
                cursor: ColorDef::from_hex("#268bd2"),
                cursor_text: ColorDef::from_hex("#fdf6e3"),
                selection: ColorDef::from_rgba(38, 139, 210, 0.3),
                selection_text: ColorDef::from_hex("#657b83"),
            },
        }
    }

    /// Nord theme
    pub fn nord() -> Self {
        Self {
            name: "Nord".to_string(),
            variant: ThemeVariant::Dark,
            ui: UiColors {
                bg_primary: ColorDef::from_hex("#2e3440"),
                bg_secondary: ColorDef::from_hex("#3b4252"),
                bg_block: ColorDef::from_hex("#434c5e"),
                bg_block_hover: ColorDef::from_hex("#4c566a"),
                bg_input: ColorDef::from_hex("#3b4252"),
                text_primary: ColorDef::from_hex("#eceff4"),
                text_secondary: ColorDef::from_hex("#d8dee9"),
                text_muted: ColorDef::from_hex("#a3aebb"),
                accent_blue: ColorDef::from_hex("#88c0d0"),
                accent_green: ColorDef::from_hex("#a3be8c"),
                accent_yellow: ColorDef::from_hex("#ebcb8b"),
                accent_red: ColorDef::from_hex("#bf616a"),
                accent_purple: ColorDef::from_hex("#b48ead"),
                accent_cyan: ColorDef::from_hex("#8fbcbb"),
                border: ColorDef::from_hex("#434c5e"),
                tab_active: ColorDef::from_hex("#88c0d0"),
                selection: ColorDef::from_rgba(136, 192, 208, 0.3),
            },
            ansi: AnsiPalette {
                black: ColorDef::from_hex("#3b4252"),
                red: ColorDef::from_hex("#bf616a"),
                green: ColorDef::from_hex("#a3be8c"),
                yellow: ColorDef::from_hex("#ebcb8b"),
                blue: ColorDef::from_hex("#81a1c1"),
                magenta: ColorDef::from_hex("#b48ead"),
                cyan: ColorDef::from_hex("#88c0d0"),
                white: ColorDef::from_hex("#e5e9f0"),
                bright_black: ColorDef::from_hex("#4c566a"),
                bright_red: ColorDef::from_hex("#d08770"),
                bright_green: ColorDef::from_hex("#a3be8c"),
                bright_yellow: ColorDef::from_hex("#ebcb8b"),
                bright_blue: ColorDef::from_hex("#81a1c1"),
                bright_magenta: ColorDef::from_hex("#b48ead"),
                bright_cyan: ColorDef::from_hex("#8fbcbb"),
                bright_white: ColorDef::from_hex("#eceff4"),
            },
            terminal: TerminalColors {
                foreground: ColorDef::from_hex("#d8dee9"),
                background: ColorDef::from_hex("#2e3440"),
                cursor: ColorDef::from_hex("#88c0d0"),
                cursor_text: ColorDef::from_hex("#2e3440"),
                selection: ColorDef::from_rgba(136, 192, 208, 0.3),
                selection_text: ColorDef::from_hex("#d8dee9"),
            },
        }
    }

    /// One Dark theme (Atom-inspired)
    pub fn one_dark() -> Self {
        Self {
            name: "One Dark".to_string(),
            variant: ThemeVariant::Dark,
            ui: UiColors {
                bg_primary: ColorDef::from_hex("#21252b"),
                bg_secondary: ColorDef::from_hex("#282c34"),
                bg_block: ColorDef::from_hex("#2c313c"),
                bg_block_hover: ColorDef::from_hex("#383e4a"),
                bg_input: ColorDef::from_hex("#282c34"),
                text_primary: ColorDef::from_hex("#abb2bf"),
                text_secondary: ColorDef::from_hex("#5c6370"),
                text_muted: ColorDef::from_hex("#4b5263"),
                accent_blue: ColorDef::from_hex("#61afef"),
                accent_green: ColorDef::from_hex("#98c379"),
                accent_yellow: ColorDef::from_hex("#e5c07b"),
                accent_red: ColorDef::from_hex("#e06c75"),
                accent_purple: ColorDef::from_hex("#c678dd"),
                accent_cyan: ColorDef::from_hex("#56b6c2"),
                border: ColorDef::from_hex("#3e4451"),
                tab_active: ColorDef::from_hex("#61afef"),
                selection: ColorDef::from_rgba(97, 175, 239, 0.3),
            },
            ansi: AnsiPalette {
                black: ColorDef::from_hex("#282c34"),
                red: ColorDef::from_hex("#e06c75"),
                green: ColorDef::from_hex("#98c379"),
                yellow: ColorDef::from_hex("#e5c07b"),
                blue: ColorDef::from_hex("#61afef"),
                magenta: ColorDef::from_hex("#c678dd"),
                cyan: ColorDef::from_hex("#56b6c2"),
                white: ColorDef::from_hex("#abb2bf"),
                bright_black: ColorDef::from_hex("#5c6370"),
                bright_red: ColorDef::from_hex("#e06c75"),
                bright_green: ColorDef::from_hex("#98c379"),
                bright_yellow: ColorDef::from_hex("#e5c07b"),
                bright_blue: ColorDef::from_hex("#61afef"),
                bright_magenta: ColorDef::from_hex("#c678dd"),
                bright_cyan: ColorDef::from_hex("#56b6c2"),
                bright_white: ColorDef::from_hex("#ffffff"),
            },
            terminal: TerminalColors {
                foreground: ColorDef::from_hex("#abb2bf"),
                background: ColorDef::from_hex("#282c34"),
                cursor: ColorDef::from_hex("#61afef"),
                cursor_text: ColorDef::from_hex("#282c34"),
                selection: ColorDef::from_rgba(97, 175, 239, 0.3),
                selection_text: ColorDef::from_hex("#abb2bf"),
            },
        }
    }

    /// Monokai Pro theme
    pub fn monokai_pro() -> Self {
        Self {
            name: "Monokai Pro".to_string(),
            variant: ThemeVariant::Dark,
            ui: UiColors {
                bg_primary: ColorDef::from_hex("#2d2a2e"),
                bg_secondary: ColorDef::from_hex("#221f22"),
                bg_block: ColorDef::from_hex("#363337"),
                bg_block_hover: ColorDef::from_hex("#423f43"),
                bg_input: ColorDef::from_hex("#221f22"),
                text_primary: ColorDef::from_hex("#fcfcfa"),
                text_secondary: ColorDef::from_hex("#939293"),
                text_muted: ColorDef::from_hex("#727072"),
                accent_blue: ColorDef::from_hex("#78dce8"),
                accent_green: ColorDef::from_hex("#a9dc76"),
                accent_yellow: ColorDef::from_hex("#ffd866"),
                accent_red: ColorDef::from_hex("#ff6188"),
                accent_purple: ColorDef::from_hex("#ab9df2"),
                accent_cyan: ColorDef::from_hex("#78dce8"),
                border: ColorDef::from_hex("#423f43"),
                tab_active: ColorDef::from_hex("#78dce8"),
                selection: ColorDef::from_rgba(120, 220, 232, 0.3),
            },
            ansi: AnsiPalette {
                black: ColorDef::from_hex("#2d2a2e"),
                red: ColorDef::from_hex("#ff6188"),
                green: ColorDef::from_hex("#a9dc76"),
                yellow: ColorDef::from_hex("#ffd866"),
                blue: ColorDef::from_hex("#78dce8"),
                magenta: ColorDef::from_hex("#ab9df2"),
                cyan: ColorDef::from_hex("#78dce8"),
                white: ColorDef::from_hex("#fcfcfa"),
                bright_black: ColorDef::from_hex("#727072"),
                bright_red: ColorDef::from_hex("#ff6188"),
                bright_green: ColorDef::from_hex("#a9dc76"),
                bright_yellow: ColorDef::from_hex("#ffd866"),
                bright_blue: ColorDef::from_hex("#78dce8"),
                bright_magenta: ColorDef::from_hex("#ab9df2"),
                bright_cyan: ColorDef::from_hex("#78dce8"),
                bright_white: ColorDef::from_hex("#ffffff"),
            },
            terminal: TerminalColors {
                foreground: ColorDef::from_hex("#fcfcfa"),
                background: ColorDef::from_hex("#2d2a2e"),
                cursor: ColorDef::from_hex("#ffd866"),
                cursor_text: ColorDef::from_hex("#2d2a2e"),
                selection: ColorDef::from_rgba(120, 220, 232, 0.3),
                selection_text: ColorDef::from_hex("#fcfcfa"),
            },
        }
    }

    /// Tokyo Night theme
    pub fn tokyo_night() -> Self {
        Self {
            name: "Tokyo Night".to_string(),
            variant: ThemeVariant::Dark,
            ui: UiColors {
                bg_primary: ColorDef::from_hex("#1a1b26"),
                bg_secondary: ColorDef::from_hex("#16161e"),
                bg_block: ColorDef::from_hex("#24283b"),
                bg_block_hover: ColorDef::from_hex("#292e42"),
                bg_input: ColorDef::from_hex("#16161e"),
                text_primary: ColorDef::from_hex("#a9b1d6"),
                text_secondary: ColorDef::from_hex("#787c99"),
                text_muted: ColorDef::from_hex("#565f89"),
                accent_blue: ColorDef::from_hex("#7aa2f7"),
                accent_green: ColorDef::from_hex("#9ece6a"),
                accent_yellow: ColorDef::from_hex("#e0af68"),
                accent_red: ColorDef::from_hex("#f7768e"),
                accent_purple: ColorDef::from_hex("#bb9af7"),
                accent_cyan: ColorDef::from_hex("#7dcfff"),
                border: ColorDef::from_hex("#292e42"),
                tab_active: ColorDef::from_hex("#7aa2f7"),
                selection: ColorDef::from_rgba(122, 162, 247, 0.3),
            },
            ansi: AnsiPalette {
                black: ColorDef::from_hex("#1a1b26"),
                red: ColorDef::from_hex("#f7768e"),
                green: ColorDef::from_hex("#9ece6a"),
                yellow: ColorDef::from_hex("#e0af68"),
                blue: ColorDef::from_hex("#7aa2f7"),
                magenta: ColorDef::from_hex("#bb9af7"),
                cyan: ColorDef::from_hex("#7dcfff"),
                white: ColorDef::from_hex("#a9b1d6"),
                bright_black: ColorDef::from_hex("#414868"),
                bright_red: ColorDef::from_hex("#f7768e"),
                bright_green: ColorDef::from_hex("#9ece6a"),
                bright_yellow: ColorDef::from_hex("#e0af68"),
                bright_blue: ColorDef::from_hex("#7aa2f7"),
                bright_magenta: ColorDef::from_hex("#bb9af7"),
                bright_cyan: ColorDef::from_hex("#7dcfff"),
                bright_white: ColorDef::from_hex("#c0caf5"),
            },
            terminal: TerminalColors {
                foreground: ColorDef::from_hex("#a9b1d6"),
                background: ColorDef::from_hex("#1a1b26"),
                cursor: ColorDef::from_hex("#7aa2f7"),
                cursor_text: ColorDef::from_hex("#1a1b26"),
                selection: ColorDef::from_rgba(122, 162, 247, 0.3),
                selection_text: ColorDef::from_hex("#a9b1d6"),
            },
        }
    }

    /// Get a theme by name
    pub fn by_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "warp" | "warp_dark" | "default" => Some(Self::warp_dark()),
            "dracula" => Some(Self::dracula()),
            "solarized_dark" | "solarized-dark" => Some(Self::solarized_dark()),
            "solarized_light" | "solarized-light" => Some(Self::solarized_light()),
            "nord" => Some(Self::nord()),
            "one_dark" | "one-dark" | "onedark" => Some(Self::one_dark()),
            "monokai_pro" | "monokai-pro" => Some(Self::monokai_pro()),
            "tokyo_night" | "tokyo-night" => Some(Self::tokyo_night()),
            _ => None,
        }
    }

    /// List all available theme names
    pub fn available_themes() -> Vec<&'static str> {
        vec![
            "warp_dark",
            "dracula",
            "solarized_dark",
            "solarized_light",
            "nord",
            "one_dark",
            "monokai_pro",
            "tokyo_night",
        ]
    }
}

// ============================================================================
// Theme Loading
// ============================================================================

impl Theme {
    /// Load theme from TOML string
    pub fn from_toml_str(toml_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml_str)
    }

    /// Load theme from TOML file
    pub fn from_toml_file(path: &std::path::Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_toml_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Save theme to TOML file
    pub fn to_toml_file(&self, path: &std::path::Path) -> std::io::Result<()> {
        let toml_str = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, toml_str)
    }
}

// ============================================================================
// Helper for RGBA colors
// ============================================================================

impl ColorDef {
    /// Create from RGBA bytes (0-255) with alpha (0.0-1.0)
    pub fn from_rgba(r: u8, g: u8, b: u8, a: f32) -> Self {
        ColorDef::RgbFloat([r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0]).with_alpha(a)
    }

    /// Apply alpha to color (returns new ColorDef)
    pub fn with_alpha(&self, _alpha: f32) -> Self {
        // Note: Iced doesn't directly support alpha in Color::from_rgb
        // This is a placeholder for future RGBA support
        self.clone()
    }
}

// ============================================================================
// Iced Style Functions
// ============================================================================

use iced::widget::container;
use iced::Border;

impl Theme {
    /// Container style for status bar
    pub fn status_bar_style(&self) -> impl Fn(&iced::Theme) -> container::Style {
        let bg = self.ui.bg_primary.to_color();
        let border = self.ui.border.to_color();

        move |_theme: &iced::Theme| container::Style {
            background: Some(bg.into()),
            border: Border {
                color: border,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }

    /// Container style for primary background
    pub fn primary_background_style(&self) -> impl Fn(&iced::Theme) -> container::Style {
        let bg = self.ui.bg_primary.to_color();

        move |_theme: &iced::Theme| container::Style {
            background: Some(bg.into()),
            ..Default::default()
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_def_hex_parsing() {
        let color = ColorDef::from_hex("#ff0000");
        let iced_color = color.to_color();
        assert_eq!(iced_color, Color::from_rgb(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_color_def_hex_parsing_short() {
        let color = ColorDef::from_hex("#f00");
        let iced_color = color.to_color();
        assert_eq!(iced_color, Color::from_rgb(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_color_def_rgb() {
        let color = ColorDef::from_rgb(255, 0, 0);
        let iced_color = color.to_color();
        assert_eq!(iced_color, Color::from_rgb(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_ansi_palette_get_color() {
        let theme = Theme::warp_dark();
        let red = theme.ansi.get_color(1);
        // Should return the red color from ANSI palette
        assert_ne!(red, Color::WHITE);
    }

    #[test]
    fn test_theme_by_name() {
        assert!(Theme::by_name("dracula").is_some());
        assert!(Theme::by_name("nord").is_some());
        assert!(Theme::by_name("invalid_name").is_none());
    }

    #[test]
    fn test_available_themes() {
        let themes = Theme::available_themes();
        assert!(themes.contains(&"dracula"));
        assert!(themes.contains(&"nord"));
        assert!(themes.contains(&"one_dark"));
    }

    #[test]
    fn test_theme_serialization() {
        let theme = Theme::warp_dark();
        let toml_str = toml::to_string(&theme).unwrap();
        assert!(toml_str.contains("name"));
        assert!(toml_str.contains("variant"));
    }

    #[test]
    fn test_theme_deserialization() {
        let toml_str = concat!(
            "name = \"Test Theme\"\n",
            "variant = \"dark\"\n",
            "\n",
            "[ui]\n",
            "bg_primary = \"#17171c\"\n",
            "bg_secondary = \"#1e1e26\"\n",
            "bg_block = \"#242430\"\n",
            "bg_block_hover = \"#2d2d38\"\n",
            "bg_input = \"#1c1c24\"\n",
            "text_primary = \"#edeff2\"\n",
            "text_secondary = \"#999ead\"\n",
            "text_muted = \"#737885\"\n",
            "accent_blue = \"#5c8afa\"\n",
            "accent_green = \"#59c78c\"\n",
            "accent_yellow = \"#f2c55c\"\n",
            "accent_red = \"#eb6473\"\n",
            "accent_purple = \"#8c5cfa\"\n",
            "accent_cyan = \"#5ce6fa\"\n",
            "border = \"#383847\"\n",
            "tab_active = \"#5c8afa\"\n",
            "selection = \"#5c8afa\"\n",
            "\n",
            "[ansi]\n",
            "black = \"#000000\"\n",
            "red = \"#ff0000\"\n",
            "green = \"#00ff00\"\n",
            "yellow = \"#ffff00\"\n",
            "blue = \"#0000ff\"\n",
            "magenta = \"#ff00ff\"\n",
            "cyan = \"#00ffff\"\n",
            "white = \"#ffffff\"\n",
            "bright_black = \"#808080\"\n",
            "bright_red = \"#ff8080\"\n",
            "bright_green = \"#80ff80\"\n",
            "bright_yellow = \"#ffff80\"\n",
            "bright_blue = \"#8080ff\"\n",
            "bright_magenta = \"#ff80ff\"\n",
            "bright_cyan = \"#80ffff\"\n",
            "bright_white = \"#ffffff\"\n",
            "\n",
            "[terminal]\n",
            "foreground = \"#edeff2\"\n",
            "background = \"#1e1e26\"\n",
            "cursor = \"#5c8afa\"\n",
            "cursor_text = \"#17171c\"\n",
            "selection = \"#5c8afa\"\n",
            "selection_text = \"#edeff2\"\n",
        );

        let theme: Result<Theme, _> = toml::from_str(toml_str);
        assert!(theme.is_ok());
        let theme = theme.unwrap();
        assert_eq!(theme.name, "Test Theme");
        assert_eq!(theme.variant, ThemeVariant::Dark);
    }

    #[test]
    fn test_all_preset_themes() {
        // Ensure all preset themes can be created without panicking
        let _warp = Theme::warp_dark();
        let _dracula = Theme::dracula();
        let _solarized_dark = Theme::solarized_dark();
        let _solarized_light = Theme::solarized_light();
        let _nord = Theme::nord();
        let _one_dark = Theme::one_dark();
        let _monokai = Theme::monokai_pro();
        let _tokyo = Theme::tokyo_night();
    }
}
