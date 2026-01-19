//! Theme and Color Definitions
//!
//! This module provides a professional theme system with Dark and Light variants inspired by Ghostty.
//!
//! # Themes
//!
//! - **GhosttyDark** (default): Dark background (#17171c), light text (#edeff2)
//! - **GhosttyLight**: Light background (#fcfcfd), dark text (#1c1c21)
//!
//! # Usage
//!
//! ```ignore
//! let theme = Theme::from_name("Ghostty Dark");
//! let colors = theme.colors();
//! let bg_color = colors.bg_primary;
//! ```
//!
//! # Color Palette
//!
//! Each theme provides a complete color palette:
//! - Background colors (primary, secondary, hover)
//! - Text colors (primary, secondary, muted)
//! - Terminal colors (16 ANSI colors)
//! - Accent colors (focus, success, warning, error)

use floem::peniko::Color;

/// Theme variant
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    GhosttyDark,
    GhosttyLight,
}

impl Theme {
    /// Parse theme from string name
    pub fn from_name(name: &str) -> Self {
        match name {
            "Ghostty Light" => Self::GhosttyLight,
            _ => Self::GhosttyDark, // Default to dark
        }
    }

    /// Parse theme from string name, returning None if not found (future feature)
    #[allow(dead_code)]
    pub fn from_name_opt(name: &str) -> Option<Self> {
        match name {
            "Ghostty Dark" => Some(Self::GhosttyDark),
            "Ghostty Light" => Some(Self::GhosttyLight),
            _ => None,
        }
    }

    /// Get theme name as string
    pub fn name(&self) -> &'static str {
        match self {
            Self::GhosttyDark => "Ghostty Dark",
            Self::GhosttyLight => "Ghostty Light",
        }
    }

    /// Toggle between dark and light
    pub fn toggle(&self) -> Self {
        match self {
            Self::GhosttyDark => Self::GhosttyLight,
            Self::GhosttyLight => Self::GhosttyDark,
        }
    }

    /// Get color palette for this theme
    pub fn colors(&self) -> ColorPalette {
        match self {
            Self::GhosttyDark => DARK_COLORS,
            Self::GhosttyLight => LIGHT_COLORS,
        }
    }
}

/// Color palette structure
#[derive(Debug, Clone, Copy)]
pub struct ColorPalette {
    pub bg_primary: Color,
    pub bg_secondary: Color,
    pub bg_tab_bar: Color,
    pub bg_tab_active: Color,
    pub bg_tab_hover: Color,
    pub bg_status: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub accent_blue: Color,
    pub accent_green: Color,
    #[allow(dead_code)]
    pub accent_yellow: Color,
    pub accent_red: Color,
    pub border: Color,
    pub border_subtle: Color,
}

/// Dark theme colors (Ghostty Dark)
const DARK_COLORS: ColorPalette = ColorPalette {
    bg_primary: Color::rgb8(23, 23, 28),      // #17171c
    bg_secondary: Color::rgb8(30, 30, 38),    // #1e1e26
    bg_tab_bar: Color::rgb8(20, 20, 25),      // #14141a
    bg_tab_active: Color::rgb8(40, 40, 50),   // #28283a
    bg_tab_hover: Color::rgb8(35, 35, 45),    // #23232d
    bg_status: Color::rgb8(25, 25, 32),       // #191920
    text_primary: Color::rgb8(237, 239, 242),   // #edeff2
    text_secondary: Color::rgb8(153, 158, 173), // #999ead
    text_muted: Color::rgb8(115, 120, 133),     // #737885
    accent_blue: Color::rgb8(92, 138, 250),     // #5c8afa
    accent_green: Color::rgb8(89, 199, 140),    // #59c78c
    accent_yellow: Color::rgb8(242, 197, 92),   // #f2c55c
    accent_red: Color::rgb8(235, 100, 115),     // #eb6473
    border: Color::rgb8(56, 56, 71),            // #383847
    border_subtle: Color::rgb8(45, 45, 58),     // #2d2d3a
};

/// Light theme colors (Ghostty Light)
const LIGHT_COLORS: ColorPalette = ColorPalette {
    bg_primary: Color::rgb8(252, 252, 253),     // #fcfcfd
    bg_secondary: Color::rgb8(246, 246, 248),   // #f6f6f8
    bg_tab_bar: Color::rgb8(240, 240, 243),     // #f0f0f3
    bg_tab_active: Color::rgb8(255, 255, 255),  // #ffffff
    bg_tab_hover: Color::rgb8(248, 248, 250),   // #f8f8fa
    bg_status: Color::rgb8(244, 244, 246),      // #f4f4f6
    text_primary: Color::rgb8(28, 28, 33),      // #1c1c21
    text_secondary: Color::rgb8(90, 95, 110),   // #5a5f6e
    text_muted: Color::rgb8(140, 145, 160),     // #8c91a0
    accent_blue: Color::rgb8(42, 102, 220),     // #2a66dc
    accent_green: Color::rgb8(39, 159, 100),    // #279f64
    accent_yellow: Color::rgb8(200, 145, 30),   // #c8911e
    accent_red: Color::rgb8(205, 50, 65),       // #cd3241
    border: Color::rgb8(220, 220, 228),         // #dcdce4
    border_subtle: Color::rgb8(230, 230, 236),  // #e6e6ec
};

/// Static color access for current theme (default to dark)
/// Note: In a real app, this should be dynamic based on app state
#[allow(dead_code)]
pub mod colors {
    use super::Color;

    // Backgrounds
    pub const BG_PRIMARY: Color = Color::rgb8(23, 23, 28);      // #17171c
    pub const BG_SECONDARY: Color = Color::rgb8(30, 30, 38);    // #1e1e26
    pub const BG_TAB_BAR: Color = Color::rgb8(20, 20, 25);      // #14141a
    pub const BG_TAB_ACTIVE: Color = Color::rgb8(40, 40, 50);   // #28283a
    pub const BG_TAB_HOVER: Color = Color::rgb8(35, 35, 45);    // #23232d
    pub const BG_STATUS: Color = Color::rgb8(25, 25, 32);       // #191920
    pub const BG_HOVER: Color = Color::rgb8(40, 40, 50);        // #28283a (same as BG_TAB_ACTIVE)

    // Text
    pub const TEXT_PRIMARY: Color = Color::rgb8(237, 239, 242);   // #edeff2
    pub const TEXT_SECONDARY: Color = Color::rgb8(153, 158, 173); // #999ead
    pub const TEXT_MUTED: Color = Color::rgb8(115, 120, 133);     // #737885

    // Accents
    pub const ACCENT_BLUE: Color = Color::rgb8(92, 138, 250);     // #5c8afa
    pub const ACCENT_GREEN: Color = Color::rgb8(89, 199, 140);    // #59c78c
    pub const ACCENT_YELLOW: Color = Color::rgb8(242, 197, 92);   // #f2c55c
    pub const ACCENT_RED: Color = Color::rgb8(235, 100, 115);     // #eb6473

    // Borders
    pub const BORDER: Color = Color::rgb8(56, 56, 71);            // #383847
    pub const BORDER_SUBTLE: Color = Color::rgb8(45, 45, 58);     // #2d2d3a
    pub const BORDER_HOVER: Color = Color::rgb8(80, 80, 95);      // #50505f

    // Interactive elements
    pub const SURFACE_HOVER: Color = Color::rgb8(40, 40, 50);     // #28283a
    pub const TEXT_DISABLED: Color = Color::rgb8(80, 83, 95);     // #50535f
}

/// ANSI 16-color palette
#[allow(dead_code)]
pub mod ansi {
    use super::Color;

    pub const BLACK: Color = Color::rgb8(0, 0, 0);
    pub const RED: Color = Color::rgb8(205, 49, 49);
    pub const GREEN: Color = Color::rgb8(13, 188, 121);
    pub const YELLOW: Color = Color::rgb8(229, 229, 16);
    pub const BLUE: Color = Color::rgb8(36, 114, 200);
    pub const MAGENTA: Color = Color::rgb8(188, 63, 188);
    pub const CYAN: Color = Color::rgb8(17, 168, 205);
    pub const WHITE: Color = Color::rgb8(229, 229, 229);

    pub const BRIGHT_BLACK: Color = Color::rgb8(102, 102, 102);
    pub const BRIGHT_RED: Color = Color::rgb8(241, 76, 76);
    pub const BRIGHT_GREEN: Color = Color::rgb8(35, 209, 139);
    pub const BRIGHT_YELLOW: Color = Color::rgb8(245, 245, 67);
    pub const BRIGHT_BLUE: Color = Color::rgb8(59, 142, 234);
    pub const BRIGHT_MAGENTA: Color = Color::rgb8(214, 112, 214);
    pub const BRIGHT_CYAN: Color = Color::rgb8(41, 184, 219);
    pub const BRIGHT_WHITE: Color = Color::rgb8(255, 255, 255);

    /// Get ANSI color by index (0-15)
    pub fn by_index(index: u8) -> Color {
        match index {
            0 => BLACK,
            1 => RED,
            2 => GREEN,
            3 => YELLOW,
            4 => BLUE,
            5 => MAGENTA,
            6 => CYAN,
            7 => WHITE,
            8 => BRIGHT_BLACK,
            9 => BRIGHT_RED,
            10 => BRIGHT_GREEN,
            11 => BRIGHT_YELLOW,
            12 => BRIGHT_BLUE,
            13 => BRIGHT_MAGENTA,
            14 => BRIGHT_CYAN,
            15 => BRIGHT_WHITE,
            _ => WHITE,
        }
    }
}

/// Font configuration
#[allow(dead_code)]
pub mod fonts {
    pub const FONT_SIZE_DEFAULT: f32 = 14.0;
    pub const FONT_SIZE_MIN: f32 = 8.0;
    pub const FONT_SIZE_MAX: f32 = 24.0;
    pub const FONT_SIZE_STEP: f32 = 1.0;

    pub const LINE_HEIGHT: f32 = 1.2;
}

/// Layout constants
#[allow(dead_code)]
pub mod layout {
    pub const TAB_BAR_HEIGHT: f64 = 36.0;
    pub const STATUS_BAR_HEIGHT: f64 = 24.0;
    pub const TAB_PADDING: f64 = 12.0;
    pub const TAB_GAP: f64 = 2.0;
    pub const PANE_DIVIDER_WIDTH: f64 = 4.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_from_name() {
        assert_eq!(Theme::from_name("Ghostty Dark"), Theme::GhosttyDark);
        assert_eq!(Theme::from_name("Ghostty Light"), Theme::GhosttyLight);

        // Default to dark for unknown themes
        assert_eq!(Theme::from_name("Unknown"), Theme::GhosttyDark);
        assert_eq!(Theme::from_name(""), Theme::GhosttyDark);
        assert_eq!(Theme::from_name("ghostty dark"), Theme::GhosttyDark); // case sensitive
    }

    #[test]
    fn test_theme_from_name_opt() {
        assert_eq!(Theme::from_name_opt("Ghostty Dark"), Some(Theme::GhosttyDark));
        assert_eq!(Theme::from_name_opt("Ghostty Light"), Some(Theme::GhosttyLight));

        // Return None for unknown themes
        assert_eq!(Theme::from_name_opt("Unknown"), None);
        assert_eq!(Theme::from_name_opt(""), None);
        assert_eq!(Theme::from_name_opt("GHOSTTY DARK"), None); // case sensitive
    }

    #[test]
    fn test_theme_name() {
        assert_eq!(Theme::GhosttyDark.name(), "Ghostty Dark");
        assert_eq!(Theme::GhosttyLight.name(), "Ghostty Light");
    }

    #[test]
    fn test_theme_toggle() {
        let dark = Theme::GhosttyDark;
        let light = Theme::GhosttyLight;

        // Toggle dark to light
        assert_eq!(dark.toggle(), light);

        // Toggle light to dark
        assert_eq!(light.toggle(), dark);

        // Toggle twice returns to original
        assert_eq!(dark.toggle().toggle(), dark);
        assert_eq!(light.toggle().toggle(), light);
    }

    #[test]
    fn test_theme_colors_exist() {
        // Dark theme
        let dark_colors = Theme::GhosttyDark.colors();
        assert!(format!("{:?}", dark_colors.bg_primary).contains("Color"));

        // Light theme
        let light_colors = Theme::GhosttyLight.colors();
        assert!(format!("{:?}", light_colors.bg_primary).contains("Color"));

        // Colors should be different between themes
        assert_ne!(
            format!("{:?}", dark_colors.bg_primary),
            format!("{:?}", light_colors.bg_primary)
        );
    }

    #[test]
    fn test_color_palette_completeness() {
        let colors = Theme::GhosttyDark.colors();

        // All fields should be accessible
        let _ = colors.bg_primary;
        let _ = colors.bg_secondary;
        let _ = colors.bg_tab_bar;
        let _ = colors.bg_tab_active;
        let _ = colors.bg_tab_hover;
        let _ = colors.bg_status;
        let _ = colors.text_primary;
        let _ = colors.text_secondary;
        let _ = colors.text_muted;
        let _ = colors.accent_blue;
        let _ = colors.accent_green;
        let _ = colors.accent_yellow;
        let _ = colors.accent_red;
        let _ = colors.border;
        let _ = colors.border_subtle;
    }

    #[test]
    fn test_ansi_colors_by_index() {
        // Test all 16 ANSI colors
        for i in 0..16 {
            let color = ansi::by_index(i);
            assert!(format!("{:?}", color).contains("Color"));
        }

        // Test out of range (should return white)
        let out_of_range = ansi::by_index(255);
        let white = ansi::by_index(7);
        assert_eq!(format!("{:?}", out_of_range), format!("{:?}", white));
    }

    #[test]
    fn test_ansi_colors_constants() {
        // Ensure all ANSI constants are accessible
        let _ = ansi::BLACK;
        let _ = ansi::RED;
        let _ = ansi::GREEN;
        let _ = ansi::YELLOW;
        let _ = ansi::BLUE;
        let _ = ansi::MAGENTA;
        let _ = ansi::CYAN;
        let _ = ansi::WHITE;
        let _ = ansi::BRIGHT_BLACK;
        let _ = ansi::BRIGHT_RED;
        let _ = ansi::BRIGHT_GREEN;
        let _ = ansi::BRIGHT_YELLOW;
        let _ = ansi::BRIGHT_BLUE;
        let _ = ansi::BRIGHT_MAGENTA;
        let _ = ansi::BRIGHT_CYAN;
        let _ = ansi::BRIGHT_WHITE;
    }

    #[test]
    fn test_font_constants() {
        assert_eq!(fonts::FONT_SIZE_DEFAULT, 14.0);
        assert_eq!(fonts::FONT_SIZE_MIN, 8.0);
        assert_eq!(fonts::FONT_SIZE_MAX, 24.0);
        assert_eq!(fonts::FONT_SIZE_STEP, 1.0);
        assert_eq!(fonts::LINE_HEIGHT, 1.2);

        // Sanity checks
        assert!(fonts::FONT_SIZE_MIN < fonts::FONT_SIZE_DEFAULT);
        assert!(fonts::FONT_SIZE_DEFAULT < fonts::FONT_SIZE_MAX);
    }

    #[test]
    fn test_layout_constants() {
        assert_eq!(layout::TAB_BAR_HEIGHT, 36.0);
        assert_eq!(layout::STATUS_BAR_HEIGHT, 24.0);
        assert_eq!(layout::TAB_PADDING, 12.0);
        assert_eq!(layout::TAB_GAP, 2.0);
        assert_eq!(layout::PANE_DIVIDER_WIDTH, 4.0);

        // Sanity checks
        assert!(layout::TAB_BAR_HEIGHT > 0.0);
        assert!(layout::STATUS_BAR_HEIGHT > 0.0);
    }

    #[test]
    fn test_theme_eq_and_clone() {
        let dark1 = Theme::GhosttyDark;
        let dark2 = Theme::GhosttyDark;
        let light = Theme::GhosttyLight;

        assert_eq!(dark1, dark2);
        assert_ne!(dark1, light);

        // Test Clone
        let cloned = dark1.clone();
        assert_eq!(dark1, cloned);
    }

    #[test]
    fn test_theme_roundtrip() {
        // Test that theme names can be round-tripped
        for theme in [Theme::GhosttyDark, Theme::GhosttyLight] {
            let name = theme.name();
            let parsed = Theme::from_name(name);
            assert_eq!(theme, parsed);
        }
    }
}
