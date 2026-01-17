//! Status Bar Widget
//!
//! Bottom status bar showing key bindings and status information.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

/// Status bar widget
pub struct StatusBar<'a> {
    /// Key binding hints
    hints: Vec<(&'a str, &'a str)>,
    /// Status message
    message: Option<&'a str>,
    /// Background style
    style: Style,
}

impl<'a> StatusBar<'a> {
    /// Create a new status bar with default hints
    #[must_use]
    pub fn new() -> Self {
        Self {
            hints: vec![
                ("F1", "Help"),
                ("F2", "Tree"),
                ("F3", "Split"),
                ("F4", "Graph"),
                ("F5", "MCP"),
                ("F6", "Archive"),
                ("^C", "Quit"),
            ],
            message: None,
            style: Style::default().bg(Color::DarkGray),
        }
    }

    /// Set custom key hints
    #[must_use]
    pub fn hints(mut self, hints: Vec<(&'a str, &'a str)>) -> Self {
        self.hints = hints;
        self
    }

    /// Set a status message
    #[must_use]
    pub fn message(mut self, message: &'a str) -> Self {
        self.message = Some(message);
        self
    }

    /// Set the background style
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Default for StatusBar<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for StatusBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Fill background
        for x in area.x..area.x + area.width {
            for y in area.y..area.y + area.height {
                buf.get_mut(x, y).set_style(self.style);
            }
        }

        // Build the line
        let mut spans = Vec::new();

        let key_style = Style::default()
            .fg(Color::Black)
            .bg(Color::Gray)
            .add_modifier(Modifier::BOLD);

        let desc_style = Style::default().fg(Color::White).bg(Color::DarkGray);

        for (key, desc) in &self.hints {
            spans.push(Span::styled(format!("[{key}]"), key_style));
            spans.push(Span::styled(format!("{desc} "), desc_style));
        }

        // Add message if present
        if let Some(msg) = self.message {
            // Calculate remaining space
            let hints_len: usize = spans.iter().map(|s| s.content.len()).sum();
            let remaining = area.width as usize - hints_len.min(area.width as usize);

            if remaining > msg.len() + 3 {
                spans.push(Span::styled(" | ", desc_style));
                spans.push(Span::styled(
                    msg,
                    Style::default().fg(Color::Yellow).bg(Color::DarkGray),
                ));
            }
        }

        let line = Line::from(spans);
        buf.set_line(area.x, area.y, &line, area.width);
    }
}
