//! Framework-independent color types for AgTerm
//!
//! This module provides color types that are independent of any specific GUI framework,
//! allowing easy migration between Iced, Floem, or other frameworks.

use serde::{Deserialize, Serialize};

/// Framework-independent RGBA color
///
/// All values are normalized to 0.0-1.0 range.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Create a new color from RGB values (0.0-1.0)
    #[inline]
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Create a color from RGB values (0.0-1.0) - Iced compatibility alias
    #[inline]
    pub const fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b)
    }

    /// Create a color from RGBA values (0.0-1.0) - Iced compatibility alias
    #[inline]
    pub const fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::with_alpha(r, g, b, a)
    }

    /// Create a new color from RGBA values (0.0-1.0)
    #[inline]
    pub const fn with_alpha(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create a color from RGB values (0-255)
    #[inline]
    pub fn from_rgb8(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        }
    }

    /// Create a color from RGBA values (0-255)
    #[inline]
    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    /// Create a color from a hex string (e.g., "#FF5500" or "FF5500")
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 && hex.len() != 8 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).ok()?
        } else {
            255
        };

        Some(Self::from_rgba8(r, g, b, a))
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        format!(
            "#{:02X}{:02X}{:02X}",
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8
        )
    }

    /// Create a new color with modified alpha
    #[inline]
    pub const fn alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }

    /// Lighten the color by a factor (0.0-1.0)
    pub fn lighten(&self, factor: f32) -> Self {
        Self {
            r: (self.r + (1.0 - self.r) * factor).min(1.0),
            g: (self.g + (1.0 - self.g) * factor).min(1.0),
            b: (self.b + (1.0 - self.b) * factor).min(1.0),
            a: self.a,
        }
    }

    /// Darken the color by a factor (0.0-1.0)
    pub fn darken(&self, factor: f32) -> Self {
        Self {
            r: (self.r * (1.0 - factor)).max(0.0),
            g: (self.g * (1.0 - factor)).max(0.0),
            b: (self.b * (1.0 - factor)).max(0.0),
            a: self.a,
        }
    }

    // Common colors
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0);
    pub const RED: Self = Self::new(1.0, 0.0, 0.0);
    pub const GREEN: Self = Self::new(0.0, 1.0, 0.0);
    pub const BLUE: Self = Self::new(0.0, 0.0, 1.0);
    pub const YELLOW: Self = Self::new(1.0, 1.0, 0.0);
    pub const CYAN: Self = Self::new(0.0, 1.0, 1.0);
    pub const MAGENTA: Self = Self::new(1.0, 0.0, 1.0);
    pub const TRANSPARENT: Self = Self::with_alpha(0.0, 0.0, 0.0, 0.0);
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

// Iced integration (compile-time feature flag)
#[cfg(feature = "iced-gui")]
impl From<Color> for iced::Color {
    fn from(c: Color) -> Self {
        iced::Color::from_rgba(c.r, c.g, c.b, c.a)
    }
}

#[cfg(feature = "iced-gui")]
impl From<iced::Color> for Color {
    fn from(c: iced::Color) -> Self {
        Self {
            r: c.r,
            g: c.g,
            b: c.b,
            a: c.a,
        }
    }
}

// Floem integration (compile-time feature flag)
#[cfg(feature = "floem-gui")]
impl From<Color> for floem::peniko::Color {
    fn from(c: Color) -> Self {
        floem::peniko::Color::rgba(c.r as f64, c.g as f64, c.b as f64, c.a as f64)
    }
}

#[cfg(feature = "floem-gui")]
impl From<floem::peniko::Color> for Color {
    fn from(c: floem::peniko::Color) -> Self {
        // Extract RGBA components from peniko Color (stored as u32 RGBA)
        let rgba = c.to_premul_u32();
        Self {
            r: ((rgba >> 24) & 0xFF) as f32 / 255.0,
            g: ((rgba >> 16) & 0xFF) as f32 / 255.0,
            b: ((rgba >> 8) & 0xFF) as f32 / 255.0,
            a: (rgba & 0xFF) as f32 / 255.0,
        }
    }
}

/// ANSI 16-color palette
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum AnsiColor {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,
    BrightBlack = 8,
    BrightRed = 9,
    BrightGreen = 10,
    BrightYellow = 11,
    BrightBlue = 12,
    BrightMagenta = 13,
    BrightCyan = 14,
    BrightWhite = 15,
}

impl AnsiColor {
    /// Convert ANSI color index to enum
    pub fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::Black),
            1 => Some(Self::Red),
            2 => Some(Self::Green),
            3 => Some(Self::Yellow),
            4 => Some(Self::Blue),
            5 => Some(Self::Magenta),
            6 => Some(Self::Cyan),
            7 => Some(Self::White),
            8 => Some(Self::BrightBlack),
            9 => Some(Self::BrightRed),
            10 => Some(Self::BrightGreen),
            11 => Some(Self::BrightYellow),
            12 => Some(Self::BrightBlue),
            13 => Some(Self::BrightMagenta),
            14 => Some(Self::BrightCyan),
            15 => Some(Self::BrightWhite),
            _ => None,
        }
    }

    /// Get default color for this ANSI color
    pub fn default_color(&self) -> Color {
        match self {
            Self::Black => Color::from_rgb8(0, 0, 0),
            Self::Red => Color::from_rgb8(205, 49, 49),
            Self::Green => Color::from_rgb8(13, 188, 121),
            Self::Yellow => Color::from_rgb8(229, 229, 16),
            Self::Blue => Color::from_rgb8(36, 114, 200),
            Self::Magenta => Color::from_rgb8(188, 63, 188),
            Self::Cyan => Color::from_rgb8(17, 168, 205),
            Self::White => Color::from_rgb8(229, 229, 229),
            Self::BrightBlack => Color::from_rgb8(102, 102, 102),
            Self::BrightRed => Color::from_rgb8(241, 76, 76),
            Self::BrightGreen => Color::from_rgb8(35, 209, 139),
            Self::BrightYellow => Color::from_rgb8(245, 245, 67),
            Self::BrightBlue => Color::from_rgb8(59, 142, 234),
            Self::BrightMagenta => Color::from_rgb8(214, 112, 214),
            Self::BrightCyan => Color::from_rgb8(41, 184, 219),
            Self::BrightWhite => Color::from_rgb8(255, 255, 255),
        }
    }
}

/// Terminal color that can be ANSI indexed or true color (24-bit RGB)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TerminalColor {
    /// ANSI 16-color palette (0-15)
    Ansi(AnsiColor),
    /// ANSI 256-color palette (16-255)
    Indexed(u8),
    /// True color (24-bit RGB)
    Rgb(Color),
    /// Default foreground/background
    Default,
}

impl TerminalColor {
    /// Convert to actual color using the given palette
    pub fn to_color(&self, palette: &AnsiPalette) -> Color {
        match self {
            Self::Ansi(ansi) => palette.get(*ansi),
            Self::Indexed(idx) => {
                if *idx < 16 {
                    AnsiColor::from_index(*idx)
                        .map(|c| palette.get(c))
                        .unwrap_or(Color::WHITE)
                } else if *idx < 232 {
                    // 216 color cube (6x6x6)
                    let idx = *idx - 16;
                    let r = (idx / 36) % 6;
                    let g = (idx / 6) % 6;
                    let b = idx % 6;
                    Color::from_rgb8(
                        if r == 0 { 0 } else { 55 + r * 40 },
                        if g == 0 { 0 } else { 55 + g * 40 },
                        if b == 0 { 0 } else { 55 + b * 40 },
                    )
                } else {
                    // 24 grayscale colors
                    let gray = (*idx - 232) * 10 + 8;
                    Color::from_rgb8(gray, gray, gray)
                }
            }
            Self::Rgb(color) => *color,
            Self::Default => Color::WHITE,
        }
    }
}

/// ANSI 16-color palette
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnsiPalette {
    colors: [Color; 16],
}

impl AnsiPalette {
    /// Create a new palette with default colors
    pub fn new() -> Self {
        Self::default()
    }

    /// Get color for an ANSI color
    pub fn get(&self, ansi: AnsiColor) -> Color {
        self.colors[ansi as usize]
    }

    /// Set color for an ANSI color
    pub fn set(&mut self, ansi: AnsiColor, color: Color) {
        self.colors[ansi as usize] = color;
    }
}

impl Default for AnsiPalette {
    fn default() -> Self {
        Self {
            colors: [
                AnsiColor::Black.default_color(),
                AnsiColor::Red.default_color(),
                AnsiColor::Green.default_color(),
                AnsiColor::Yellow.default_color(),
                AnsiColor::Blue.default_color(),
                AnsiColor::Magenta.default_color(),
                AnsiColor::Cyan.default_color(),
                AnsiColor::White.default_color(),
                AnsiColor::BrightBlack.default_color(),
                AnsiColor::BrightRed.default_color(),
                AnsiColor::BrightGreen.default_color(),
                AnsiColor::BrightYellow.default_color(),
                AnsiColor::BrightBlue.default_color(),
                AnsiColor::BrightMagenta.default_color(),
                AnsiColor::BrightCyan.default_color(),
                AnsiColor::BrightWhite.default_color(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_rgb8() {
        let color = Color::from_rgb8(255, 128, 0);
        assert!((color.r - 1.0).abs() < 0.01);
        assert!((color.g - 0.5).abs() < 0.01);
        assert!((color.b - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex("#FF8000").unwrap();
        assert!((color.r - 1.0).abs() < 0.01);
        assert!((color.g - 0.5).abs() < 0.01);
        assert!((color.b - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_color_to_hex() {
        let color = Color::from_rgb8(255, 128, 0);
        assert_eq!(color.to_hex(), "#FF8000");
    }

    #[test]
    fn test_ansi_256_color_cube() {
        let palette = AnsiPalette::default();

        // Test color cube index 16 (first color after ANSI 16)
        let color = TerminalColor::Indexed(16).to_color(&palette);
        assert_eq!(color, Color::from_rgb8(0, 0, 0));

        // Test grayscale
        let color = TerminalColor::Indexed(232).to_color(&palette);
        assert_eq!(color, Color::from_rgb8(8, 8, 8));
    }
}
