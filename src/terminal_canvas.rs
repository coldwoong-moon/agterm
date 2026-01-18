//! Terminal Canvas - Virtual scrolling with hardware-accelerated rendering
//!
//! Renders terminal output using Iced's canvas widget for:
//! - Virtual scrolling (only visible lines rendered)
//! - Hardware acceleration (GPU when using wgpu backend)
//! - Geometry caching between frames
//!
//! ## Performance Optimizations:
//! 1. Span merging: Efficiently merge consecutive cells with identical styles
//! 2. Smart cache invalidation: Selective cache clearing based on change type
//! 3. Memory pre-allocation: Reuse buffers and pre-allocate based on known sizes

use iced::mouse;
use iced::widget::canvas::{self, Cache, Frame, Geometry, Text};
use iced::{Color, Font, Point, Rectangle, Renderer, Size, Theme};

use crate::StyledSpan;
use std::time::{Duration, Instant};

// Re-export Selection types from terminal::selection module
pub use crate::terminal::selection::{Selection, SelectionMode, SelectionPoint};

/// Extract selected text from terminal lines
pub fn get_selected_text(lines: &[Vec<StyledSpan>], selection: &Selection) -> String {
    if !selection.active {
        return String::new();
    }

    let (start, end) = selection.normalized();

    // If selection is empty (start == end), return empty string
    if start == end {
        return String::new();
    }

    let mut result = String::new();

    for row in start.line..=end.line {
        if row >= lines.len() {
            break;
        }

        // Calculate column range for this row
        let (start_col, end_col) = if row == start.line && row == end.line {
            // Single line selection
            (start.col, end.col)
        } else if row == start.line {
            // First line of multi-line selection
            let max_col = lines[row]
                .iter()
                .map(|span| span.text.chars().count())
                .sum::<usize>();
            (start.col, max_col)
        } else if row == end.line {
            // Last line of multi-line selection
            (0, end.col)
        } else {
            // Middle line - select entire line
            let max_col = lines[row]
                .iter()
                .map(|span| span.text.chars().count())
                .sum::<usize>();
            (0, max_col)
        };

        // Extract text from spans
        let mut current_col = 0;
        for span in &lines[row] {
            // Skip placeholder cells (second cell of wide characters)
            let span_text = &span.text;
            let span_len = span_text.chars().count();
            let span_end = current_col + span_len;

            // Check if this span overlaps with selection
            if span_end > start_col && current_col <= end_col {
                // Calculate overlap range within this span
                let copy_start = if current_col < start_col {
                    start_col - current_col
                } else {
                    0
                };
                let copy_end = if span_end > end_col {
                    span_len - (span_end - end_col - 1)
                } else {
                    span_len
                };

                // Extract characters in range
                let chars: Vec<char> = span_text.chars().collect();
                for i in copy_start..copy_end.min(chars.len()) {
                    result.push(chars[i]);
                }
            }

            current_col = span_end;
            if current_col > end_col {
                break;
            }
        }

        // Add newline for multi-line selections (except for the last line)
        if row < end.line {
            result.push('\n');
        }
    }

    result
}

/// Cursor style for terminal rendering
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CursorStyle {
    #[default]
    Block,
    /// Underline cursor (reserved for future cursor style support)
    #[allow(dead_code)]
    Underline,
    /// Bar/I-beam cursor (reserved for future cursor style support)
    #[allow(dead_code)]
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
    // Base configuration (font_size = 14.0)
    pub const BASE_FONT_SIZE: f32 = 14.0;
    pub const BASE_LINE_HEIGHT: f32 = 18.0;
    pub const BASE_CHAR_WIDTH: f32 = 8.4;

    pub const PADDING_LEFT: f32 = 8.0;
    pub const PADDING_TOP: f32 = 4.0;
    pub const SCROLL_SPEED: f32 = 3.0;

    /// Streaming mode detection threshold (ms between updates)
    pub const STREAMING_THRESHOLD_MS: u64 = 50;
    /// Number of rapid updates to enter streaming mode
    pub const STREAMING_COUNT_THRESHOLD: u8 = 3;
    /// Time without updates to exit streaming mode (ms)
    pub const STREAMING_EXIT_MS: u64 = 200;

    /// Calculate line height based on font size
    pub fn line_height(font_size: f32) -> f32 {
        BASE_LINE_HEIGHT * (font_size / BASE_FONT_SIZE)
    }

    /// Calculate character width based on font size
    pub fn char_width(font_size: f32) -> f32 {
        BASE_CHAR_WIDTH * (font_size / BASE_FONT_SIZE)
    }
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
    /// Text selection state
    pub selection: Option<Selection>,
    /// Mouse drag state
    pub is_dragging: bool,
    /// Last clicked URL (for Cmd+Click handling)
    pub clicked_url: Option<String>,
    /// Performance tracking (debug mode)
    #[cfg(debug_assertions)]
    last_frame_time: Option<Duration>,
    #[cfg(debug_assertions)]
    last_lines_rendered: usize,
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
            selection: None,
            is_dragging: false,
            clicked_url: None,
            #[cfg(debug_assertions)]
            last_frame_time: None,
            #[cfg(debug_assertions)]
            last_lines_rendered: 0,
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

    /// Scroll to top
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0.0;
        // Don't clear cache on scroll - virtual scrolling handles visibility changes
        if !self.streaming_mode {
            self.cache.clear();
        }
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self, total_lines: usize, font_size: f32) {
        let content_height = total_lines as f32 * config::line_height(font_size);
        let max_scroll = (content_height - self.viewport_height).max(0.0);
        self.scroll_offset = max_scroll;
        // Don't clear cache on scroll - virtual scrolling handles visibility changes
        // Only clear if not in streaming mode
        if !self.streaming_mode {
            self.cache.clear();
        }
    }

    /// Check if scrolled to bottom (reserved for auto-scroll logic)
    #[allow(dead_code)]
    pub fn is_at_bottom(&self, total_lines: usize, font_size: f32) -> bool {
        let content_height = total_lines as f32 * config::line_height(font_size);
        let max_scroll = (content_height - self.viewport_height).max(0.0);
        self.scroll_offset >= max_scroll - 1.0
    }
}

/// Message emitted by terminal canvas (for future use)
#[derive(Debug, Clone)]
#[allow(dead_code)]
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
    pub font_size: f32,
    pub search_matches: &'a [(usize, usize, usize)], // (line, start_col, end_col)
    pub current_match_index: Option<usize>,
    pub bracket_match: Option<crate::terminal::bracket::BracketMatch>,
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
            font_size: config::BASE_FONT_SIZE,
            search_matches: &[],
            current_match_index: None,
            bracket_match: None,
        }
    }

    /// Set cursor state for rendering
    pub fn with_cursor(mut self, cursor: CursorState) -> Self {
        self.cursor = Some(cursor);
        self
    }

    /// Set font size for rendering
    pub fn with_font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    /// Set search matches for highlighting
    pub fn with_search_matches(
        mut self,
        matches: &'a [(usize, usize, usize)],
        current_index: Option<usize>,
    ) -> Self {
        self.search_matches = matches;
        self.current_match_index = current_index;
        self
    }

    /// Set bracket match for highlighting
    pub fn with_bracket_match(
        mut self,
        bracket_match: Option<crate::terminal::bracket::BracketMatch>,
    ) -> Self {
        self.bracket_match = bracket_match;
        self
    }

    fn content_height(&self) -> f32 {
        self.lines.len() as f32 * config::line_height(self.font_size)
    }

    fn visible_range(&self, scroll_offset: f32, viewport_height: f32) -> (usize, usize) {
        let first = (scroll_offset / config::line_height(self.font_size)).floor() as usize;
        let visible_count =
            (viewport_height / config::line_height(self.font_size)).ceil() as usize + 2;
        let last = (first + visible_count).min(self.lines.len());
        (first, last)
    }

    /// Convert pixel coordinates to cell coordinates
    fn pixel_to_cell(&self, x: f32, y: f32, scroll_offset: f32) -> (usize, usize) {
        let adjusted_x = (x - config::PADDING_LEFT).max(0.0);
        let adjusted_y = (y - config::PADDING_TOP + scroll_offset).max(0.0);

        let col = (adjusted_x / config::char_width(self.font_size)) as usize;
        let row = (adjusted_y / config::line_height(self.font_size)) as usize;

        // Clamp to valid ranges
        let row = row.min(self.lines.len().saturating_sub(1));
        let max_col = if row < self.lines.len() {
            self.lines[row]
                .iter()
                .map(|span| span.text.chars().count())
                .sum::<usize>()
                .saturating_sub(1)
        } else {
            0
        };
        let col = col.min(max_col.max(0));

        (row, col)
    }

    /// Get hyperlink at the given cell position
    pub fn get_hyperlink_at(&self, row: usize, col: usize) -> Option<String> {
        if row >= self.lines.len() {
            return None;
        }

        let line = &self.lines[row];
        let mut current_col = 0;

        for span in line {
            let span_len = span.text.chars().count();
            if col >= current_col && col < current_col + span_len {
                return span.hyperlink.as_ref().map(|arc| arc.as_ref().clone());
            }
            current_col += span_len;
        }

        None
    }

    fn draw_selection(&self, frame: &mut Frame, state: &TerminalCanvasState, bounds: Rectangle) {
        let selection = match &state.selection {
            Some(s) if s.active => s,
            _ => return,
        };

        let (start, end) = selection.normalized();
        let (first_visible, last_visible) = self.visible_range(state.scroll_offset, bounds.height);
        let y_offset = -(state.scroll_offset % config::line_height(self.font_size));

        // Selection color - semi-transparent blue
        let selection_color = Color::from_rgba(0.3, 0.5, 0.8, 0.3);

        for row in start.line..=end.line {
            if row < first_visible || row >= last_visible {
                continue;
            }

            let visible_row = row - first_visible;
            let y = config::PADDING_TOP
                + y_offset
                + (visible_row as f32 * config::line_height(self.font_size));

            if y + config::line_height(self.font_size) < 0.0 || y > bounds.height {
                continue;
            }

            // Calculate selection start and end columns for this row
            let (start_col, end_col) = if row == start.line && row == end.line {
                // Selection within single line
                (start.col, end.col)
            } else if row == start.line {
                // First line of multi-line selection
                let max_col = if row < self.lines.len() {
                    self.lines[row]
                        .iter()
                        .map(|span| span.text.chars().count())
                        .sum::<usize>()
                } else {
                    0
                };
                (start.col, max_col)
            } else if row == end.line {
                // Last line of multi-line selection
                (0, end.col)
            } else {
                // Middle line - select entire line
                let max_col = if row < self.lines.len() {
                    self.lines[row]
                        .iter()
                        .map(|span| span.text.chars().count())
                        .sum::<usize>()
                } else {
                    0
                };
                (0, max_col)
            };

            // Draw selection rectangle
            let x_start =
                config::PADDING_LEFT + (start_col as f32 * config::char_width(self.font_size));
            let width = ((end_col - start_col + 1) as f32 * config::char_width(self.font_size))
                .max(config::char_width(self.font_size));

            frame.fill_rectangle(
                Point::new(x_start, y),
                Size::new(width, config::line_height(self.font_size)),
                selection_color,
            );
        }
    }

    fn draw_bracket_highlights(&self, frame: &mut Frame, state: &TerminalCanvasState, bounds: Rectangle) {
        let bracket_match = match &self.bracket_match {
            Some(m) => m,
            None => return,
        };

        let config = crate::config::AppConfig::default();
        let bracket_color = crate::config::parse_hex_color(&config.terminal.bracket.highlight_color)
            .map(|(r, g, b, a)| Color::from_rgba(r, g, b, a))
            .unwrap_or(Color::from_rgb(0.36, 0.54, 0.98)); // Default to accent blue

        let (first_visible, last_visible) = self.visible_range(state.scroll_offset, bounds.height);
        let y_offset = -(state.scroll_offset % config::line_height(self.font_size));

        // Draw highlight for opening bracket
        self.draw_bracket_highlight_at(
            frame,
            bracket_match.open_pos.0,
            bracket_match.open_pos.1,
            bracket_color,
            first_visible,
            last_visible,
            y_offset,
            bounds.height,
        );

        // Draw highlight for closing bracket
        self.draw_bracket_highlight_at(
            frame,
            bracket_match.close_pos.0,
            bracket_match.close_pos.1,
            bracket_color,
            first_visible,
            last_visible,
            y_offset,
            bounds.height,
        );
    }

    fn draw_bracket_highlight_at(
        &self,
        frame: &mut Frame,
        row: usize,
        col: usize,
        color: Color,
        first_visible: usize,
        last_visible: usize,
        y_offset: f32,
        viewport_height: f32,
    ) {
        // Only draw if bracket is in visible range
        if row < first_visible || row >= last_visible {
            return;
        }

        let visible_row = row - first_visible;
        let y = config::PADDING_TOP
            + y_offset
            + (visible_row as f32 * config::line_height(self.font_size));

        if y + config::line_height(self.font_size) < 0.0 || y > viewport_height {
            return;
        }

        let x = config::PADDING_LEFT + (col as f32 * config::char_width(self.font_size));

        // Draw a border rectangle around the bracket
        let border_width = 2.0;
        let char_width = config::char_width(self.font_size);
        let line_height = config::line_height(self.font_size);

        // Top border
        frame.fill_rectangle(
            Point::new(x, y),
            Size::new(char_width, border_width),
            color,
        );

        // Bottom border
        frame.fill_rectangle(
            Point::new(x, y + line_height - border_width),
            Size::new(char_width, border_width),
            color,
        );

        // Left border
        frame.fill_rectangle(
            Point::new(x, y),
            Size::new(border_width, line_height),
            color,
        );

        // Right border
        frame.fill_rectangle(
            Point::new(x + char_width - border_width, y),
            Size::new(border_width, line_height),
            color,
        );
    }

    fn draw_lines(&self, frame: &mut Frame, state: &TerminalCanvasState, bounds: Rectangle) {
        let (first, last) = self.visible_range(state.scroll_offset, bounds.height);
        let y_offset = -(state.scroll_offset % config::line_height(self.font_size));

        for (i, line_idx) in (first..last).enumerate() {
            if line_idx >= self.lines.len() {
                break;
            }

            let y =
                config::PADDING_TOP + y_offset + (i as f32 * config::line_height(self.font_size));

            if y + config::line_height(self.font_size) < 0.0 || y > bounds.height {
                continue;
            }

            self.draw_line(frame, &self.lines[line_idx], y);
        }
    }

    fn draw_line(&self, frame: &mut Frame, spans: &[StyledSpan], y: f32) {
        if spans.is_empty() {
            return;
        }

        let mut x = config::PADDING_LEFT;

        // Optimization: Merge consecutive spans with identical styles before rendering
        // This reduces the number of draw calls significantly
        let mut merged_text = String::with_capacity(128); // Pre-allocate
        let mut current_color: Option<Color> = None;
        let mut current_bold = false;
        let mut current_underline = false;
        let mut current_dim = false;
        let mut current_italic = false;
        let mut current_strikethrough = false;
        let mut segment_start_x = x;

        for span in spans {
            if span.text.is_empty() {
                continue;
            }

            let mut color = span.color.unwrap_or(self.default_color);

            // Apply dim effect by reducing alpha
            if span.dim {
                color = Color::from_rgba(color.r, color.g, color.b, color.a * 0.5);
            }

            // Apply italic effect (color shift as Iced doesn't support italic directly)
            if span.italic {
                color = Color::from_rgba(
                    color.r * 0.9,
                    color.g * 0.9 + 0.1,
                    color.b * 0.9 + 0.1,
                    color.a,
                );
            }

            // Check if we can merge with current segment
            let can_merge = current_color == Some(color)
                && current_bold == span.bold
                && current_underline == span.underline
                && current_dim == span.dim
                && current_italic == span.italic
                && current_strikethrough == span.strikethrough
                && !merged_text.is_empty();

            if can_merge {
                // Merge with current segment
                merged_text.push_str(&span.text);
            } else {
                // Flush current segment if any
                if !merged_text.is_empty() {
                    let effective_color = current_color.unwrap_or(self.default_color);
                    let span_data = StyledSpan {
                        text: merged_text.clone(),
                        color: Some(effective_color),
                        bold: current_bold,
                        underline: current_underline,
                        dim: current_dim,
                        italic: current_italic,
                        strikethrough: current_strikethrough,
                        hyperlink: None,
                    };
                    self.draw_text_segment(
                        frame,
                        &merged_text,
                        segment_start_x,
                        y.round(),
                        effective_color,
                        &span_data,
                    );

                    let char_count = merged_text.chars().count();
                    segment_start_x += char_count as f32 * config::char_width(self.font_size);
                    x = segment_start_x;
                    merged_text.clear();
                }

                // Start new segment
                merged_text.push_str(&span.text);
                current_color = Some(color);
                current_bold = span.bold;
                current_underline = span.underline;
                current_dim = span.dim;
                current_italic = span.italic;
                current_strikethrough = span.strikethrough;
                segment_start_x = x;
            }
        }

        // Flush final segment
        if !merged_text.is_empty() {
            let effective_color = current_color.unwrap_or(self.default_color);
            let span_data = StyledSpan {
                text: merged_text.clone(),
                color: Some(effective_color),
                bold: current_bold,
                underline: current_underline,
                dim: current_dim,
                italic: current_italic,
                strikethrough: current_strikethrough,
                hyperlink: None,
            };
            self.draw_text_segment(
                frame,
                &merged_text,
                segment_start_x,
                y.round(),
                effective_color,
                &span_data,
            );
        }
    }

    fn draw_text_segment(
        &self,
        frame: &mut Frame,
        text: &str,
        x: f32,
        y: f32,
        color: Color,
        span: &StyledSpan,
    ) {
        // Use cyan color for hyperlinks
        let display_color = if span.hyperlink.is_some() {
            Color::from_rgb(0.3, 0.8, 0.8) // Cyan for URLs
        } else {
            color
        };

        let text_obj = Text {
            content: text.to_string(),
            position: Point::new(x, y),
            color: display_color,
            size: self.font_size.into(),
            font: self.font,
            horizontal_alignment: iced::alignment::Horizontal::Left,
            vertical_alignment: iced::alignment::Vertical::Top,
            ..Default::default()
        };
        frame.fill_text(text_obj);

        let char_count = text.chars().count();
        let text_width = char_count as f32 * config::char_width(self.font_size);

        // Draw underline for URLs or underlined text
        if span.underline || span.hyperlink.is_some() {
            let underline_y = y + config::line_height(self.font_size) - 2.0;
            frame.fill_rectangle(
                Point::new(x, underline_y),
                Size::new(text_width, 1.0),
                display_color,
            );
        }

        // Draw strikethrough
        if span.strikethrough {
            let strikethrough_y = y + config::line_height(self.font_size) / 2.0;
            frame.fill_rectangle(
                Point::new(x, strikethrough_y),
                Size::new(text_width, 1.0),
                display_color,
            );
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
        let y_offset = -(state.scroll_offset % config::line_height(self.font_size));
        let x = config::PADDING_LEFT + (cursor.col as f32 * config::char_width(self.font_size));
        let y = config::PADDING_TOP
            + y_offset
            + (visible_row as f32 * config::line_height(self.font_size));

        let cursor_color = Color::from_rgba(0.9, 0.9, 0.9, 0.9);

        match cursor.style {
            CursorStyle::Block => {
                // Semi-transparent block cursor
                frame.fill_rectangle(
                    Point::new(x, y),
                    Size::new(
                        config::char_width(self.font_size),
                        config::line_height(self.font_size),
                    ),
                    Color::from_rgba(0.9, 0.9, 0.9, 0.7),
                );
            }
            CursorStyle::Underline => {
                // Underline cursor (2px height)
                frame.fill_rectangle(
                    Point::new(x, y + config::line_height(self.font_size) - 2.0),
                    Size::new(config::char_width(self.font_size), 2.0),
                    cursor_color,
                );
            }
            CursorStyle::Bar => {
                // Bar/I-beam cursor (2px width)
                frame.fill_rectangle(
                    Point::new(x, y),
                    Size::new(2.0, config::line_height(self.font_size)),
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
        cursor: mouse::Cursor,
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
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(position) = cursor.position() {
                    let (row, col) =
                        self.pixel_to_cell(position.x, position.y, state.scroll_offset);

                    // Check if clicking on a URL
                    if let Some(url) = self.get_hyperlink_at(row, col) {
                        state.clicked_url = Some(url);
                        // Don't start selection when clicking URL
                        return (canvas::event::Status::Captured, None);
                    }

                    let mut selection = Selection::new();
                    selection.start(row, col, SelectionMode::Character);
                    state.selection = Some(selection);
                    state.is_dragging = true;
                    // Don't clear cache for selection start - selection overlay is separate
                    if !state.streaming_mode {
                        state.cache.clear();
                    }
                    return (canvas::event::Status::Captured, None);
                }
                (canvas::event::Status::Ignored, None)
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.is_dragging {
                    if let Some(position) = cursor.position() {
                        let (row, col) =
                            self.pixel_to_cell(position.x, position.y, state.scroll_offset);
                        if let Some(selection) = &mut state.selection {
                            selection.extend(row, col);
                            // Only clear cache for selection updates in non-streaming mode
                            if !state.streaming_mode {
                                state.cache.clear();
                            }
                            return (canvas::event::Status::Captured, None);
                        }
                    }
                }
                (canvas::event::Status::Ignored, None)
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.is_dragging {
                    state.is_dragging = false;
                    if let Some(selection) = &mut state.selection {
                        selection.finish();
                        // If no actual selection (start == end), clear it
                        if !selection.active {
                            state.selection = None;
                            if !state.streaming_mode {
                                state.cache.clear();
                            }
                        }
                    }
                    return (canvas::event::Status::Captured, None);
                }
                (canvas::event::Status::Ignored, None)
            }
            canvas::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let scroll_amount = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => {
                        -y * config::line_height(self.font_size) * config::SCROLL_SPEED
                    }
                    mouse::ScrollDelta::Pixels { y, .. } => -y,
                };

                let content_height = self.content_height();
                let max_scroll = (content_height - bounds.height).max(0.0);
                let new_offset = (state.scroll_offset + scroll_amount).clamp(0.0, max_scroll);

                if (state.scroll_offset - new_offset).abs() > 0.1 {
                    state.scroll_offset = new_offset;
                    // Scrolling doesn't require cache clear - just visible range change
                    // Only clear in non-streaming mode if really needed (large scrolls)
                    if !state.streaming_mode && scroll_amount.abs() > bounds.height {
                        // Large scroll - clear cache
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
        #[cfg(debug_assertions)]
        let start_time = Instant::now();

        // In streaming mode, bypass cache for better performance
        if state.streaming_mode {
            let mut frame = Frame::new(renderer, bounds.size());

            // Draw background
            frame.fill_rectangle(Point::ORIGIN, bounds.size(), Color::TRANSPARENT);

            // Draw selection highlight
            self.draw_selection(&mut frame, state, bounds);

            // Draw bracket highlights
            self.draw_bracket_highlights(&mut frame, state, bounds);

            // Draw visible lines directly
            self.draw_lines(&mut frame, state, bounds);

            // Draw cursor
            self.draw_cursor(&mut frame, state, bounds);

            return vec![frame.into_geometry()];
        }

        // Normal mode: use geometry cache for text
        let text_geometry = state.cache.draw(renderer, bounds.size(), |frame| {
            // Draw background
            frame.fill_rectangle(Point::ORIGIN, bounds.size(), Color::TRANSPARENT);

            // Draw selection highlight
            self.draw_selection(frame, state, bounds);

            // Draw bracket highlights
            self.draw_bracket_highlights(frame, state, bounds);

            // Draw visible lines
            self.draw_lines(frame, state, bounds);
        });

        // Cursor is drawn separately (no cache) for blinking support
        let mut cursor_frame = Frame::new(renderer, bounds.size());
        self.draw_cursor(&mut cursor_frame, state, bounds);
        let cursor_geometry = cursor_frame.into_geometry();

        #[cfg(debug_assertions)]
        {
            let elapsed = start_time.elapsed();
            let (first, last) = self.visible_range(state.scroll_offset, bounds.height);
            let lines_rendered = last - first;

            // Log performance every 60 frames (roughly once per second at 60fps)
            if state.content_version % 60 == 0 {
                tracing::debug!(
                    "Render perf: {:.2}ms, {} lines, streaming={}",
                    elapsed.as_secs_f64() * 1000.0,
                    lines_rendered,
                    state.streaming_mode
                );
            }
        }

        vec![text_geometry, cursor_geometry]
    }
}
