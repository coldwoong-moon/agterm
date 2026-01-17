//! Terminal Canvas - Virtual scrolling with hardware-accelerated rendering
//!
//! Renders terminal output using Iced's canvas widget for:
//! - Virtual scrolling (only visible lines rendered)
//! - Hardware acceleration (GPU when using wgpu backend)
//! - Geometry caching between frames

use iced::widget::canvas::{self, Cache, Frame, Geometry, Text};
use iced::mouse;
use iced::{Color, Font, Point, Rectangle, Renderer, Size, Theme};

use crate::StyledSpan;
use std::time::Instant;

/// Cursor style for terminal rendering
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CursorStyle {
    #[default]
    Block,
    Underline,
    Bar,
}

/// Cursor state for rendering
#[derive(Clone, Debug)]
pub struct CursorState {
    pub row: usize,
    pub col: usize,
    pub style: CursorStyle,
    pub visible: bool,
    pub blink_on: bool,
}

/// Terminal rendering configuration
pub mod config {
    pub const LINE_HEIGHT: f32 = 18.0;
    pub const FONT_SIZE: f32 = 14.0;
    pub const CHAR_WIDTH: f32 = 8.4;  // Approximate monospace char width
    pub const PADDING_LEFT: f32 = 8.0;
    pub const PADDING_TOP: f32 = 4.0;
    pub const SCROLL_SPEED: f32 = 3.0;

    /// Streaming mode detection threshold (ms between updates)
    pub const STREAMING_THRESHOLD_MS: u64 = 50;
    /// Number of rapid updates to enter streaming mode
    pub const STREAMING_COUNT_THRESHOLD: u8 = 3;
    /// Time without updates to exit streaming mode (ms)
    pub const STREAMING_EXIT_MS: u64 = 200;
}

/// Canvas state for virtual scrolling
pub struct TerminalCanvasState {
    /// Current scroll offset in pixels
    pub scroll_offset: f32,
    /// Geometry cache
    cache: Cache,
    /// Last viewport height
    viewport_height: f32,
    /// Last content version
    content_version: u64,
    /// Last update timestamp for streaming detection
    last_update: Option<Instant>,
    /// Counter for rapid updates
    rapid_update_count: u8,
    /// Streaming mode active (bypass cache for performance)
    streaming_mode: bool,
}

impl Default for TerminalCanvasState {
    fn default() -> Self {
        Self {
            scroll_offset: 0.0,
            cache: Cache::new(),
            viewport_height: 0.0,
            content_version: 0,
            last_update: None,
            rapid_update_count: 0,
            streaming_mode: false,
        }
    }
}

impl std::fmt::Debug for TerminalCanvasState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TerminalCanvasState")
            .field("scroll_offset", &self.scroll_offset)
            .field("viewport_height", &self.viewport_height)
            .field("content_version", &self.content_version)
            .field("streaming_mode", &self.streaming_mode)
            .finish()
    }
}

impl TerminalCanvasState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Invalidate cache (call when content changes)
    pub fn invalidate(&mut self) {
        self.cache.clear();
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self, total_lines: usize) {
        let content_height = total_lines as f32 * config::LINE_HEIGHT;
        let max_scroll = (content_height - self.viewport_height).max(0.0);
        self.scroll_offset = max_scroll;
        self.cache.clear();
    }

    /// Check if scrolled to bottom
    pub fn is_at_bottom(&self, total_lines: usize) -> bool {
        let content_height = total_lines as f32 * config::LINE_HEIGHT;
        let max_scroll = (content_height - self.viewport_height).max(0.0);
        self.scroll_offset >= max_scroll - 1.0
    }
}

/// Message emitted by terminal canvas
#[derive(Debug, Clone)]
pub enum TerminalCanvasMessage {
    Scrolled(f32),
}

/// Terminal canvas program
pub struct TerminalCanvas<'a> {
    pub lines: &'a [Vec<StyledSpan>],
    pub content_version: u64,
    pub default_color: Color,
    pub font: Font,
    pub cursor: Option<CursorState>,
}

impl<'a> TerminalCanvas<'a> {
    pub fn new(
        lines: &'a [Vec<StyledSpan>],
        content_version: u64,
        default_color: Color,
        font: Font,
    ) -> Self {
        Self {
            lines,
            content_version,
            default_color,
            font,
            cursor: None,
        }
    }

    /// Set cursor state for rendering
    pub fn with_cursor(mut self, cursor: CursorState) -> Self {
        self.cursor = Some(cursor);
        self
    }

    fn content_height(&self) -> f32 {
        self.lines.len() as f32 * config::LINE_HEIGHT
    }

    fn visible_range(&self, scroll_offset: f32, viewport_height: f32) -> (usize, usize) {
        let first = (scroll_offset / config::LINE_HEIGHT).floor() as usize;
        let visible_count = (viewport_height / config::LINE_HEIGHT).ceil() as usize + 2;
        let last = (first + visible_count).min(self.lines.len());
        (first, last)
    }

    fn draw_lines(&self, frame: &mut Frame, state: &TerminalCanvasState, bounds: Rectangle) {
        let (first, last) = self.visible_range(state.scroll_offset, bounds.height);
        let y_offset = -(state.scroll_offset % config::LINE_HEIGHT);

        for (i, line_idx) in (first..last).enumerate() {
            if line_idx >= self.lines.len() {
                break;
            }

            let y = config::PADDING_TOP + y_offset + (i as f32 * config::LINE_HEIGHT);

            if y + config::LINE_HEIGHT < 0.0 || y > bounds.height {
                continue;
            }

            self.draw_line(frame, &self.lines[line_idx], y);
        }
    }

    fn draw_line(&self, frame: &mut Frame, spans: &[StyledSpan], y: f32) {
        let mut x = config::PADDING_LEFT;

        for span in spans {
            if span.text.is_empty() {
                continue;
            }

            let color = span.color.unwrap_or(self.default_color);

            let text = Text {
                content: span.text.clone(),
                position: Point::new(x, y),
                color,
                size: config::FONT_SIZE.into(),
                font: self.font,
                horizontal_alignment: iced::alignment::Horizontal::Left,
                vertical_alignment: iced::alignment::Vertical::Top,
                ..Default::default()
            };

            frame.fill_text(text);

            // Advance x position
            x += span.text.chars().count() as f32 * config::CHAR_WIDTH;
        }
    }

    fn draw_cursor(&self, frame: &mut Frame, state: &TerminalCanvasState, bounds: Rectangle) {
        let cursor = match &self.cursor {
            Some(c) if c.visible && c.blink_on => c,
            _ => return,
        };

        // Check if cursor is in visible range
        let (first_visible, last_visible) = self.visible_range(state.scroll_offset, bounds.height);
        if cursor.row < first_visible || cursor.row >= last_visible {
            return;
        }

        // Calculate cursor screen position
        let visible_row = cursor.row - first_visible;
        let y_offset = -(state.scroll_offset % config::LINE_HEIGHT);
        let x = config::PADDING_LEFT + (cursor.col as f32 * config::CHAR_WIDTH);
        let y = config::PADDING_TOP + y_offset + (visible_row as f32 * config::LINE_HEIGHT);

        let cursor_color = Color::from_rgba(0.9, 0.9, 0.9, 0.9);

        match cursor.style {
            CursorStyle::Block => {
                // Semi-transparent block cursor
                frame.fill_rectangle(
                    Point::new(x, y),
                    Size::new(config::CHAR_WIDTH, config::LINE_HEIGHT),
                    Color::from_rgba(0.9, 0.9, 0.9, 0.7),
                );
            }
            CursorStyle::Underline => {
                // Underline cursor (2px height)
                frame.fill_rectangle(
                    Point::new(x, y + config::LINE_HEIGHT - 2.0),
                    Size::new(config::CHAR_WIDTH, 2.0),
                    cursor_color,
                );
            }
            CursorStyle::Bar => {
                // Bar/I-beam cursor (2px width)
                frame.fill_rectangle(
                    Point::new(x, y),
                    Size::new(2.0, config::LINE_HEIGHT),
                    cursor_color,
                );
            }
        }
    }
}

impl<'a, Message> canvas::Program<Message> for TerminalCanvas<'a>
where
    Message: Clone,
{
    type State = TerminalCanvasState;

    fn update(
        &self,
        state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        // Track viewport height
        if (state.viewport_height - bounds.height).abs() > 1.0 {
            state.viewport_height = bounds.height;
            if !state.streaming_mode {
                state.cache.clear();
            }
        }

        // Content change detection with streaming mode
        if state.content_version != self.content_version {
            state.content_version = self.content_version;
            let now = Instant::now();

            // Check for rapid updates (streaming mode detection)
            if let Some(last) = state.last_update {
                let elapsed_ms = now.duration_since(last).as_millis() as u64;

                if elapsed_ms < config::STREAMING_THRESHOLD_MS {
                    state.rapid_update_count = state.rapid_update_count.saturating_add(1);
                    if state.rapid_update_count >= config::STREAMING_COUNT_THRESHOLD {
                        state.streaming_mode = true;
                    }
                } else if elapsed_ms > config::STREAMING_EXIT_MS {
                    // Exit streaming mode after idle period
                    state.rapid_update_count = 0;
                    state.streaming_mode = false;
                }
            }

            state.last_update = Some(now);

            // Only invalidate cache in non-streaming mode
            if !state.streaming_mode {
                state.cache.clear();
            }
        }

        match event {
            canvas::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let scroll_amount = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => {
                        -y * config::LINE_HEIGHT * config::SCROLL_SPEED
                    }
                    mouse::ScrollDelta::Pixels { y, .. } => -y,
                };

                let content_height = self.content_height();
                let max_scroll = (content_height - bounds.height).max(0.0);
                let new_offset = (state.scroll_offset + scroll_amount).clamp(0.0, max_scroll);

                if (state.scroll_offset - new_offset).abs() > 0.1 {
                    state.scroll_offset = new_offset;
                    if !state.streaming_mode {
                        state.cache.clear();
                    }
                }

                (canvas::event::Status::Captured, None)
            }
            _ => (canvas::event::Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        // In streaming mode, bypass cache for better performance
        if state.streaming_mode {
            let mut frame = Frame::new(renderer, bounds.size());

            // Draw background
            frame.fill_rectangle(
                Point::ORIGIN,
                bounds.size(),
                Color::TRANSPARENT,
            );

            // Draw visible lines directly
            self.draw_lines(&mut frame, state, bounds);

            // Draw cursor
            self.draw_cursor(&mut frame, state, bounds);

            return vec![frame.into_geometry()];
        }

        // Normal mode: use geometry cache for text
        let text_geometry = state.cache.draw(renderer, bounds.size(), |frame| {
            // Draw background
            frame.fill_rectangle(
                Point::ORIGIN,
                bounds.size(),
                Color::TRANSPARENT,
            );

            // Draw visible lines
            self.draw_lines(frame, state, bounds);
        });

        // Cursor is drawn separately (no cache) for blinking support
        let mut cursor_frame = Frame::new(renderer, bounds.size());
        self.draw_cursor(&mut cursor_frame, state, bounds);
        let cursor_geometry = cursor_frame.into_geometry();

        vec![text_geometry, cursor_geometry]
    }
}
