//! Terminal Pane Widget
//!
//! Displays PTY output in a ratatui widget.

use crate::infrastructure::pty::{CellAttributes, Color as PtyColor, TerminalScreen};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

/// Terminal pane widget for displaying PTY output
pub struct TerminalPane<'a> {
    /// The terminal screen to display
    screen: &'a TerminalScreen,
    /// Block decoration
    block: Option<Block<'a>>,
    /// Whether this pane is focused
    focused: bool,
    /// Show cursor
    show_cursor: bool,
}

impl<'a> TerminalPane<'a> {
    /// Create a new terminal pane widget
    pub fn new(screen: &'a TerminalScreen) -> Self {
        Self {
            screen,
            block: None,
            focused: false,
            show_cursor: true,
        }
    }

    /// Set the block decoration
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set whether this pane is focused
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Set whether to show the cursor
    pub fn show_cursor(mut self, show: bool) -> Self {
        self.show_cursor = show;
        self
    }

    /// Convert PTY color to ratatui color
    fn convert_color(pty_color: PtyColor) -> Color {
        match pty_color {
            PtyColor::Default => Color::Reset,
            PtyColor::Black => Color::Black,
            PtyColor::Red => Color::Red,
            PtyColor::Green => Color::Green,
            PtyColor::Yellow => Color::Yellow,
            PtyColor::Blue => Color::Blue,
            PtyColor::Magenta => Color::Magenta,
            PtyColor::Cyan => Color::Cyan,
            PtyColor::White => Color::White,
            PtyColor::BrightBlack => Color::DarkGray,
            PtyColor::BrightRed => Color::LightRed,
            PtyColor::BrightGreen => Color::LightGreen,
            PtyColor::BrightYellow => Color::LightYellow,
            PtyColor::BrightBlue => Color::LightBlue,
            PtyColor::BrightMagenta => Color::LightMagenta,
            PtyColor::BrightCyan => Color::LightCyan,
            PtyColor::BrightWhite => Color::White,
            PtyColor::Indexed(n) => Color::Indexed(n),
            PtyColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
        }
    }

    /// Convert cell attributes to ratatui style
    fn convert_attrs(attrs: &CellAttributes) -> Style {
        let mut style = Style::default()
            .fg(Self::convert_color(attrs.fg_color))
            .bg(Self::convert_color(attrs.bg_color));

        let mut modifiers = Modifier::empty();
        if attrs.bold {
            modifiers |= Modifier::BOLD;
        }
        if attrs.italic {
            modifiers |= Modifier::ITALIC;
        }
        if attrs.underline {
            modifiers |= Modifier::UNDERLINED;
        }
        if attrs.blink {
            modifiers |= Modifier::SLOW_BLINK;
        }
        if attrs.inverse {
            modifiers |= Modifier::REVERSED;
        }
        if attrs.hidden {
            modifiers |= Modifier::HIDDEN;
        }
        if attrs.strikethrough {
            modifiers |= Modifier::CROSSED_OUT;
        }

        if !modifiers.is_empty() {
            style = style.add_modifier(modifiers);
        }

        style
    }
}

impl<'a> Widget for TerminalPane<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render block if present
        let inner_area = if let Some(block) = &self.block {
            let inner = block.inner(area);
            block.clone().render(area, buf);
            inner
        } else {
            area
        };

        // Calculate visible rows
        let visible_rows = inner_area.height as usize;
        let visible_cols = inner_area.width as usize;

        // Render each visible row
        for (y, screen_row) in (0..visible_rows).enumerate() {
            if screen_row >= self.screen.rows {
                break;
            }

            for (x, screen_col) in (0..visible_cols).enumerate() {
                if screen_col >= self.screen.cols {
                    break;
                }

                let buf_x = inner_area.x + x as u16;
                let buf_y = inner_area.y + y as u16;

                if let Some(cell) = self.screen.get_cell(screen_row, screen_col) {
                    let style = Self::convert_attrs(&cell.attrs);

                    // Check if this is the cursor position
                    let is_cursor = self.show_cursor
                        && self.screen.cursor_visible
                        && screen_row == self.screen.cursor_row
                        && screen_col == self.screen.cursor_col;

                    let final_style = if is_cursor {
                        style.add_modifier(Modifier::REVERSED)
                    } else {
                        style
                    };

                    buf.get_mut(buf_x, buf_y).set_char(cell.char).set_style(final_style);
                }
            }
        }
    }
}

/// Simple terminal output widget that displays raw text
pub struct SimpleTerminalOutput<'a> {
    /// Lines of output
    lines: Vec<Line<'a>>,
    /// Block decoration
    block: Option<Block<'a>>,
}

impl<'a> SimpleTerminalOutput<'a> {
    /// Create from raw bytes
    pub fn from_bytes(data: &'a [u8]) -> Self {
        let text = String::from_utf8_lossy(data);
        let lines: Vec<Line<'a>> = text
            .lines()
            .map(|line| Line::from(line.to_string()))
            .collect();

        Self { lines, block: None }
    }

    /// Create from string
    pub fn from_str(text: &'a str) -> Self {
        let lines: Vec<Line<'a>> = text
            .lines()
            .map(|line| Line::from(line.to_string()))
            .collect();

        Self { lines, block: None }
    }

    /// Set the block decoration
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> Widget for SimpleTerminalOutput<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let paragraph = Paragraph::new(self.lines)
            .wrap(Wrap { trim: false });

        let paragraph = if let Some(block) = self.block {
            paragraph.block(block)
        } else {
            paragraph
        };

        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_conversion() {
        assert_eq!(TerminalPane::convert_color(PtyColor::Red), Color::Red);
        assert_eq!(TerminalPane::convert_color(PtyColor::Default), Color::Reset);
        assert_eq!(
            TerminalPane::convert_color(PtyColor::Rgb(255, 0, 0)),
            Color::Rgb(255, 0, 0)
        );
    }
}
