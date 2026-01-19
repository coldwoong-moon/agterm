//! Terminal View Component
//!
//! Custom Floem view for rendering terminal grid with ANSI colors.
//! PERFORMANCE OPTIMIZATIONS:
//! - Text layout caching to avoid recreating layouts
//! - Dirty region tracking to skip unchanged cells
//! - Batch signal updates

use floem::prelude::*;
use floem::reactive::{RwSignal, SignalGet, SignalUpdate, Scope, Trigger, create_effect};
use floem::peniko::{Color, kurbo::{Rect, Line, Stroke}};
use floem::views::{container, Decorators};
use floem::{View, ViewId};
use floem_renderer::Renderer;
use floem::keyboard::{Key, NamedKey};
use floem::text::{Attrs, AttrsList, FamilyOwned, TextLayout, Weight, Style as FontStyle};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use uuid::Uuid;
use std::time::Instant;

use crate::floem_app::state::AppState;
use crate::floem_app::theme::colors;
use crate::floem_app::settings::CursorStyle;
use crate::terminal::screen::TerminalScreen;

use crate::terminal::search::{SearchState, SearchMatch};
/// Terminal canvas constants
/// Default cell dimensions at default font size (14.0)
const DEFAULT_FONT_SIZE: f64 = 14.0;
const CELL_WIDTH: f64 = 9.0;   // Character width at font size 14
const CELL_HEIGHT: f64 = 18.0;  // Character height at font size 14
const DEFAULT_COLS: usize = 80;
const DEFAULT_ROWS: usize = 24;
/// Double-click time threshold (milliseconds)
const DOUBLE_CLICK_MS: u64 = 500;
/// Triple-click time threshold (milliseconds)
const TRIPLE_CLICK_MS: u64 = 500;

/// Font/text layout cache key
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextCacheKey {
    ch: char,
    bold: bool,
    italic: bool,
}

/// Text selection position (row, col)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SelectionPos {
    row: usize,
    col: usize,
}

impl SelectionPos {
    fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

/// Text selection state
#[derive(Debug, Clone)]
struct Selection {
    /// Selection start position
    start: SelectionPos,
    /// Selection end position (current drag position)
    end: SelectionPos,
}

impl Selection {
    fn new(start: SelectionPos, end: SelectionPos) -> Self {
        Self { start, end }
    }

    /// Get normalized start and end (start <= end)
    fn normalized(&self) -> (SelectionPos, SelectionPos) {
        if self.start.row < self.end.row
            || (self.start.row == self.end.row && self.start.col <= self.end.col)
        {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }

    /// Check if a cell is within the selection
    fn contains(&self, row: usize, col: usize) -> bool {
        let (start, end) = self.normalized();

        if row < start.row || row > end.row {
            return false;
        }

        if row == start.row && row == end.row {
            // Single line selection
            col >= start.col && col <= end.col
        } else if row == start.row {
            // First line of multi-line selection
            col >= start.col
        } else if row == end.row {
            // Last line of multi-line selection
            col <= end.col
        } else {
            // Middle lines of multi-line selection
            true
        }
    }
}

/// Terminal rendering state
///
/// Wraps a TerminalScreen buffer with reactive change tracking.
#[derive(Clone)]
pub struct TerminalState {
    /// The terminal screen buffer
    screen: Arc<Mutex<TerminalScreen>>,
    /// Content version for change detection (public for reactive tracking)
    pub content_version: RwSignal<u64>,
    /// PTY session ID
    pty_session_id: Arc<Mutex<Option<Uuid>>>,
    /// IME composing text
    #[allow(dead_code)]
    pub ime_composing: RwSignal<String>,
    /// Scroll offset (0 = at bottom/live, >0 = scrolled up)
    scroll_offset: RwSignal<usize>,
    /// Cursor blink state (true = visible, false = hidden)
    #[allow(dead_code)]
    pub cursor_blink_on: RwSignal<bool>,
    /// Search state
    search_state: Arc<Mutex<SearchState>>,
    /// Trigger for cross-thread repaint (used with ext_event)
    repaint_trigger: Trigger,
}

impl Default for TerminalState {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalState {
    pub fn new() -> Self {
        // Create a scope for the trigger (required for cross-thread repaint)
        let scope = Scope::new();
        let repaint_trigger = scope.create_trigger();

        Self {
            screen: Arc::new(Mutex::new(TerminalScreen::new(DEFAULT_COLS, DEFAULT_ROWS))),
            content_version: RwSignal::new(0),
            pty_session_id: Arc::new(Mutex::new(None)),
            ime_composing: RwSignal::new(String::new()),
            scroll_offset: RwSignal::new(0),
            cursor_blink_on: RwSignal::new(true),
            search_state: Arc::new(Mutex::new(SearchState::new())),
            repaint_trigger,
        }
    }

    /// Get the repaint trigger for cross-thread signaling
    pub fn repaint_trigger(&self) -> Trigger {
        self.repaint_trigger
    }

    /// Request repaint from any thread (cross-thread safe via ext_event)
    fn request_repaint(&self) {
        // Use ext_event trigger to wake up the event loop from background thread
        // This is the correct way to request repaint from non-UI threads in Floem
        tracing::debug!("Requesting repaint via ext_event trigger");
        floem::ext_event::register_ext_trigger(self.repaint_trigger);
    }

    /// Set PTY session ID
    pub fn set_pty_session(&self, session_id: Uuid) {
        match self.pty_session_id.lock() {
            Ok(mut id) => {
                *id = Some(session_id);
                tracing::debug!("PTY session {} set for terminal state", session_id);
            }
            Err(e) => {
                tracing::error!("Failed to lock PTY session ID mutex: {}", e);
            }
        }
    }

    /// Get PTY session ID
    pub fn pty_session(&self) -> Option<Uuid> {
        match self.pty_session_id.lock() {
            Ok(id) => *id,
            Err(e) => {
                tracing::error!("Failed to lock PTY session ID mutex: {}", e);
                None
            }
        }
    }

    /// Process PTY output (OPTIMIZED: batch processing)
    pub fn process_output(&self, data: &[u8]) {
        if data.is_empty() {
            return;
        }

        match self.screen.lock() {
            Ok(mut screen) => {
                screen.process(data);
                // NOTE: Signal updates don't work from background threads in Floem
                // We rely solely on ViewId.request_paint() for cross-thread repaint
                tracing::debug!("Screen processed {} bytes, requesting repaint", data.len());
                // Request repaint from PTY thread (cross-thread safe)
                self.request_repaint();
            }
            Err(e) => {
                tracing::error!("Failed to lock terminal screen for output processing: {}", e);
            }
        }
    }

    /// Resize the terminal
    pub fn resize(&self, cols: usize, rows: usize) {
        match self.screen.lock() {
            Ok(mut screen) => {
                screen.resize(cols, rows);
                self.content_version.update(|v| *v += 1);
                tracing::debug!("Terminal resized to {}x{}", cols, rows);
            }
            Err(e) => {
                tracing::error!("Failed to lock terminal screen for resize: {}", e);
            }
        }
    }

    /// Get current screen dimensions
    pub fn dimensions(&self) -> (usize, usize) {
        match self.screen.lock() {
            Ok(s) => s.dimensions(),
            Err(e) => {
                tracing::error!("Failed to lock terminal screen to get dimensions: {}", e);
                (DEFAULT_COLS, DEFAULT_ROWS)
            }
        }
    }

    /// Get current cursor position (row, col)
    pub fn cursor_position(&self) -> (usize, usize) {
        match self.screen.lock() {
            Ok(s) => s.cursor_position(),
            Err(e) => {
                tracing::error!("Failed to lock terminal screen to get cursor position: {}", e);
                (0, 0)
            }
        }
    }

    /// Get cell dimensions for calculating pixel positions
    /// Returns (cell_width, cell_height) based on current font size
    pub fn cell_dimensions(font_size: f32) -> (f64, f64) {
        let scale = font_size as f64 / DEFAULT_FONT_SIZE;
        (CELL_WIDTH * scale, CELL_HEIGHT * scale)
    }

    /// Scroll up by lines
    #[allow(dead_code)]
    pub fn scroll_up(&self, lines: usize) {
        let scrollback_size = self.screen
            .lock()
            .map(|s| s.scrollback_size())
            .unwrap_or(0);

        self.scroll_offset.update(|offset| {
            *offset = (*offset + lines).min(scrollback_size);
        });
        self.content_version.update(|v| *v += 1);
    }

    /// Scroll down by lines
    #[allow(dead_code)]
    pub fn scroll_down(&self, lines: usize) {
        self.scroll_offset.update(|offset| {
            *offset = offset.saturating_sub(lines);
        });
        self.content_version.update(|v| *v += 1);
    }

    /// Scroll to top
    #[allow(dead_code)]
    pub fn scroll_to_top(&self) {
        let scrollback_size = self.screen
            .lock()
            .map(|s| s.scrollback_size())
            .unwrap_or(0);
        self.scroll_offset.set(scrollback_size);
        self.content_version.update(|v| *v += 1);
    }

    /// Scroll to bottom (live view)
    #[allow(dead_code)]
    pub fn scroll_to_bottom(&self) {
        self.scroll_offset.set(0);
        self.content_version.update(|v| *v += 1);
    }

    /// Get scroll offset
    pub fn get_scroll_offset(&self) -> usize {
        self.scroll_offset.get()
    }

    /// Get scrollback size
    #[allow(dead_code)]
    pub fn scrollback_size(&self) -> usize {
        self.screen
            .lock()
            .map(|s| s.scrollback_size())
            .unwrap_or(0)
    }

    /// Perform search in terminal buffer (future feature)
    #[allow(dead_code)]
    pub fn search(&self, query: String) -> (Vec<SearchMatch>, usize) {
        if query.is_empty() {
            return (Vec::new(), 0);
        }

        let mut search_state = match self.search_state.lock() {
            Ok(state) => state,
            Err(e) => {
                tracing::error!("Failed to lock search state: {}", e);
                return (Vec::new(), 0);
            }
        };

        search_state.set_query(query);

        // Get all lines from screen (including scrollback)
        let screen = match self.screen.lock() {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to lock screen for search: {}", e);
                return (Vec::new(), 0);
            }
        };

        let all_lines = screen.get_all_lines();
        drop(screen);

        // Search through all lines
        for (line_idx, line) in all_lines.iter().enumerate() {
            let line_text: String = line
                .iter()
                .filter(|c| !c.placeholder)
                .map(|c| c.c)
                .collect();
            search_state.search_line(line_idx, &line_text);
        }

        let matches = search_state.matches.clone();
        let count = search_state.match_count();

        (matches, count)
    }

    /// Navigate to next search match (future feature)
    #[allow(dead_code)]
    pub fn search_next(&self) -> Option<SearchMatch> {
        let mut search_state = match self.search_state.lock() {
            Ok(state) => state,
            Err(e) => {
                tracing::error!("Failed to lock search state: {}", e);
                return None;
            }
        };

        search_state.next_match().cloned()
    }

    /// Navigate to previous search match (future feature)
    #[allow(dead_code)]
    pub fn search_prev(&self) -> Option<SearchMatch> {
        let mut search_state = match self.search_state.lock() {
            Ok(state) => state,
            Err(e) => {
                tracing::error!("Failed to lock search state: {}", e);
                return None;
            }
        };

        search_state.prev_match().cloned()
    }

    /// Get current search match (future feature)
    #[allow(dead_code)]
    pub fn get_current_search_match(&self) -> Option<SearchMatch> {
        let search_state = match self.search_state.lock() {
            Ok(state) => state,
            Err(e) => {
                tracing::error!("Failed to lock search state: {}", e);
                return None;
            }
        };

        search_state.current().cloned()
    }

    /// Check if a cell is a search match
    pub fn is_search_match(&self, line: usize, col: usize) -> bool {
        let search_state = match self.search_state.lock() {
            Ok(state) => state,
            Err(e) => {
                tracing::error!("Failed to lock search state: {}", e);
                return false;
            }
        };

        search_state.is_match_at(line, col)
    }

    /// Check if a cell is the current search match
    pub fn is_current_search_match(&self, line: usize, col: usize) -> bool {
        let search_state = match self.search_state.lock() {
            Ok(state) => state,
            Err(e) => {
                tracing::error!("Failed to lock search state: {}", e);
                return false;
            }
        };

        search_state.is_current_match_at(line, col)
    }

    /// Clear search (future feature)
    #[allow(dead_code)]
    pub fn clear_search(&self) {
        let mut search_state = match self.search_state.lock() {
            Ok(state) => state,
            Err(e) => {
                tracing::error!("Failed to lock search state: {}", e);
                return;
            }
        };

        search_state.clear();
        self.content_version.update(|v| *v += 1);
    }

    /// Get window title from terminal (future feature)
    #[allow(dead_code)]
    pub fn window_title(&self) -> Option<String> {
        match self.screen.lock() {
            Ok(s) => s.window_title().map(|t| t.to_string()),
            Err(e) => {
                tracing::error!("Failed to lock terminal screen to get title: {}", e);
                None
            }
        }
    }
}

/// Convert ANSI color to Floem color
fn ansi_to_floem_color(ansi_color: &crate::terminal::screen::AnsiColor) -> Color {
    let agterm_color = ansi_color.to_color();
    Color::rgba(
        agterm_color.r as f64,
        agterm_color.g as f64,
        agterm_color.b as f64,
        agterm_color.a as f64,
    )
}

/// Custom terminal canvas view with optimized rendering
pub struct TerminalCanvas {
    id: ViewId,
    state: TerminalState,
    app_state: AppState,
    /// Monospace font family for terminal text
    font_family: Vec<FamilyOwned>,
    /// Last known canvas size for resize detection (width, height)
    last_size: std::cell::Cell<(f64, f64)>,
    /// Last known font size for cache invalidation
    last_font_size: std::cell::Cell<f32>,
    /// Whether this terminal is focused
    #[allow(dead_code)]
    is_focused: RwSignal<bool>,
    /// OPTIMIZATION: Text layout cache (character + style -> TextLayout base)
    /// Limit to 500 entries to prevent memory bloat
    text_cache: std::cell::RefCell<HashMap<TextCacheKey, TextLayout>>,
    /// Current text selection (if any)
    selection: std::cell::RefCell<Option<Selection>>,
    /// Mouse drag state for text selection
    is_dragging: std::cell::Cell<bool>,
    /// Last click time for double/triple click detection
    last_click_time: std::cell::Cell<Option<Instant>>,
    /// Click count (1, 2, or 3 for single/double/triple)
    click_count: std::cell::Cell<u8>,
}

impl TerminalCanvas {
    pub fn new(state: TerminalState, app_state: AppState, is_focused: RwSignal<bool>) -> Self {
        // Parse monospace font families (fallback chain)
        // Include Noto Sans Mono CJK KR for Korean/CJK text support
        let font_family = FamilyOwned::parse_list("JetBrains Mono, Noto Sans Mono CJK KR, Menlo, Monaco, Courier New, monospace")
            .collect::<Vec<_>>();

        // Create view ID first
        let id = ViewId::new();

        // Set up effect to handle repaint trigger from PTY thread
        // When the trigger fires (via ext_event), this effect runs on the UI thread
        // and calls ViewId.request_paint() which properly queues a repaint
        let repaint_trigger = state.repaint_trigger();
        create_effect(move |_| {
            // Track the trigger - this effect re-runs when trigger fires
            repaint_trigger.track();
            // Request paint on the UI thread (this is safe since we're on UI thread now)
            id.request_paint();
        });

        tracing::debug!("TerminalCanvas created with ext_event trigger for cross-thread repaint");

        // Get initial font size from app state
        let initial_font_size = app_state.font_size.get();

        Self {
            id,
            state,
            app_state,
            font_family,
            last_size: std::cell::Cell::new((0.0, 0.0)),
            last_font_size: std::cell::Cell::new(initial_font_size),
            is_focused,
            text_cache: std::cell::RefCell::new(HashMap::with_capacity(500)),
            selection: std::cell::RefCell::new(None),
            is_dragging: std::cell::Cell::new(false),
            last_click_time: std::cell::Cell::new(None),
            click_count: std::cell::Cell::new(0),
        }
    }

    /// Calculate cell dimensions based on current font size
    /// Returns (cell_width, cell_height) scaled proportionally to font size
    fn cell_dimensions(&self) -> (f64, f64) {
        let font_size = self.app_state.font_size.get() as f64;
        let scale = font_size / DEFAULT_FONT_SIZE;
        (CELL_WIDTH * scale, CELL_HEIGHT * scale)
    }

    /// Check if font size changed and invalidate cache if needed
    fn check_font_size_changed(&self) -> bool {
        let current_font_size = self.app_state.font_size.get();
        let last_font_size = self.last_font_size.get();

        if (current_font_size - last_font_size).abs() > 0.01 {
            self.last_font_size.set(current_font_size);
            // Clear text cache when font size changes
            self.text_cache.borrow_mut().clear();
            tracing::debug!(
                "Font size changed from {} to {}, cache cleared",
                last_font_size,
                current_font_size
            );
            true
        } else {
            false
        }
    }

    /// Calculate terminal dimensions from canvas size
    fn calculate_dimensions(&self, width: f64, height: f64) -> (usize, usize) {
        let (cell_width, cell_height) = self.cell_dimensions();
        let cols = (width / cell_width).floor() as usize;
        let rows = (height / cell_height).floor() as usize;

        // Ensure minimum size
        let cols = cols.max(1);
        let rows = rows.max(1);

        (cols, rows)
    }

    /// Handle terminal resize when canvas size changes
    fn handle_resize(&self, width: f64, height: f64) {
        let last = self.last_size.get();

        // Check if size actually changed (avoid unnecessary resizes)
        if (last.0 - width).abs() < 0.1 && (last.1 - height).abs() < 0.1 {
            return;
        }

        // Update stored size
        self.last_size.set((width, height));

        // Calculate new terminal dimensions
        let (new_cols, new_rows) = self.calculate_dimensions(width, height);
        let (current_cols, current_rows) = self.state.dimensions();

        // Only resize if dimensions actually changed
        if new_cols != current_cols || new_rows != current_rows {
            tracing::debug!(
                old_size = ?(current_cols, current_rows),
                new_size = ?(new_cols, new_rows),
                canvas_size = ?(width, height),
                "Resizing terminal"
            );

            // Resize the terminal screen buffer
            self.state.resize(new_cols, new_rows);

            // Notify PTY of the new size (this sends SIGWINCH to the shell)
            if let Some(session_id) = self.state.pty_session() {
                if let Err(e) = self.app_state.pty_manager.resize(
                    &session_id,
                    new_rows as u16,
                    new_cols as u16
                ) {
                    tracing::error!("Failed to resize PTY: {}", e);
                }
            }

            // Clear cache on resize
            self.text_cache.borrow_mut().clear();
        }
    }

    /// OPTIMIZATION: Get or create cached text layout for a character
    /// This avoids creating a new TextLayout for every cell on every frame
    fn get_cached_text_layout(&self, ch: char, bold: bool, italic: bool, fg_color: Color, font_size: f32) -> TextLayout {
        let cache_key = TextCacheKey { ch, bold, italic };

        let mut cache = self.text_cache.borrow_mut();

        // Check cache
        if let Some(cached_layout) = cache.get(&cache_key) {
            // Clone cached layout and update color and font size
            let mut layout = cached_layout.clone();

            let mut attrs = Attrs::new()
                .color(fg_color)
                .family(&self.font_family)
                .font_size(font_size);

            if bold {
                attrs = attrs.weight(Weight::BOLD);
            }
            if italic {
                attrs = attrs.style(FontStyle::Italic);
            }

            let attrs_list = AttrsList::new(attrs);
            layout.set_text(&ch.to_string(), attrs_list);

            return layout;
        }

        // Create new layout
        let mut attrs = Attrs::new()
            .color(fg_color)
            .family(&self.font_family)
            .font_size(font_size);

        if bold {
            attrs = attrs.weight(Weight::BOLD);
        }
        if italic {
            attrs = attrs.style(FontStyle::Italic);
        }

        let attrs_list = AttrsList::new(attrs);
        let mut text_layout = TextLayout::new();
        text_layout.set_text(&ch.to_string(), attrs_list);

        // Cache it (with size limit to prevent memory bloat)
        if cache.len() < 500 {
            cache.insert(cache_key, text_layout.clone());
        }

        text_layout
    }

    /// Handle keyboard input and send to PTY
    #[allow(dead_code)]
    fn handle_key_input(&self, key: &Key, modifiers: &floem::keyboard::Modifiers) -> Option<Vec<u8>> {
        // Convert keyboard input to bytes for PTY
        match key {
            // Named keys
            Key::Named(named) => match named {
                NamedKey::Enter => Some(b"\r".to_vec()),
                NamedKey::Backspace => Some(b"\x7f".to_vec()),
                NamedKey::Tab => Some(b"\t".to_vec()),
                NamedKey::Escape => Some(b"\x1b".to_vec()),
                NamedKey::ArrowUp => Some(b"\x1b[A".to_vec()),
                NamedKey::ArrowDown => Some(b"\x1b[B".to_vec()),
                NamedKey::ArrowRight => Some(b"\x1b[C".to_vec()),
                NamedKey::ArrowLeft => Some(b"\x1b[D".to_vec()),
                NamedKey::Home => Some(b"\x1b[H".to_vec()),
                NamedKey::End => Some(b"\x1b[F".to_vec()),
                NamedKey::PageUp => Some(b"\x1b[5~".to_vec()),
                NamedKey::PageDown => Some(b"\x1b[6~".to_vec()),
                NamedKey::Delete => Some(b"\x1b[3~".to_vec()),
                _ => None,
            },
            // Character keys
            Key::Character(ch) => {
                let ch_str = ch.as_str();

                // Handle Ctrl combinations
                if modifiers.control() {
                    if let Some(c) = ch_str.chars().next() {
                        match c.to_ascii_lowercase() {
                            'a'..='z' => {
                                // Ctrl+A = 0x01, Ctrl+B = 0x02, etc.
                                let ctrl_byte = (c.to_ascii_lowercase() as u8) - b'a' + 1;
                                Some(vec![ctrl_byte])
                            }
                            '[' => Some(b"\x1b".to_vec()),  // Ctrl+[
                            ']' => Some(b"\x1d".to_vec()),  // Ctrl+]
                            '\\' => Some(b"\x1c".to_vec()), // Ctrl+\
                            _ => None,
                        }
                    } else {
                        None
                    }
                } else {
                    // Regular character input
                    Some(ch_str.as_bytes().to_vec())
                }
            }
            _ => None,
        }
    }

    /// Send input to PTY
    #[allow(dead_code)]
    fn send_to_pty(&self, data: &[u8]) {
        match self.state.pty_session() {
            Some(session_id) => {
                if let Err(e) = self.app_state.pty_manager.write(&session_id, data) {
                    tracing::error!(
                        "Failed to write {} bytes to PTY session {}: {}",
                        data.len(),
                        session_id,
                        e
                    );
                } else {
                    tracing::trace!("Wrote {} bytes to PTY session {}", data.len(), session_id);
                }
            }
            None => {
                tracing::warn!("Attempted to send input to terminal without active PTY session");
            }
        }
    }

    /// Convert screen coordinates to terminal cell coordinates
    fn screen_to_cell(&self, x: f64, y: f64) -> SelectionPos {
        let (cell_width, cell_height) = self.cell_dimensions();
        let col = (x / cell_width).floor() as usize;
        let row = (y / cell_height).floor() as usize;
        let (cols, rows) = self.state.dimensions();
        SelectionPos::new(row.min(rows.saturating_sub(1)), col.min(cols.saturating_sub(1)))
    }

    /// Handle mouse down event for text selection
    fn handle_mouse_down(&self, x: f64, y: f64, modifiers: &floem::keyboard::Modifiers) {
        // Set focus when clicking on terminal
        self.is_focused.set(true);

        let pos = self.screen_to_cell(x, y);
        let now = Instant::now();

        // Detect double/triple click
        let mut click_count = 1u8;
        if let Some(last_time) = self.last_click_time.get() {
            let elapsed = now.duration_since(last_time).as_millis() as u64;
            let last_count = self.click_count.get();

            if elapsed < DOUBLE_CLICK_MS && last_count == 1 {
                click_count = 2; // Double click
            } else if elapsed < TRIPLE_CLICK_MS && last_count == 2 {
                click_count = 3; // Triple click
            }
        }

        self.last_click_time.set(Some(now));
        self.click_count.set(click_count);

        match click_count {
            1 => {
                // Single click: start drag selection
                if modifiers.shift() {
                    // Extend existing selection
                    if let Some(selection) = self.selection.borrow().as_ref() {
                        let mut new_selection = selection.clone();
                        new_selection.end = pos;
                        *self.selection.borrow_mut() = Some(new_selection);
                    } else {
                        *self.selection.borrow_mut() = Some(Selection::new(pos, pos));
                    }
                } else {
                    // Start new selection
                    *self.selection.borrow_mut() = Some(Selection::new(pos, pos));
                }
                self.is_dragging.set(true);
            }
            2 => {
                // Double click: select word
                self.select_word_at(pos);
                self.is_dragging.set(false);
            }
            3 => {
                // Triple click: select line
                self.select_line_at(pos);
                self.is_dragging.set(false);
            }
            _ => {}
        }

        self.id.request_paint();
    }

    /// Handle mouse move event for drag selection
    fn handle_mouse_move(&self, x: f64, y: f64) {
        if !self.is_dragging.get() {
            return;
        }

        let pos = self.screen_to_cell(x, y);

        if let Some(selection) = self.selection.borrow_mut().as_mut() {
            selection.end = pos;
            self.id.request_paint();
        }
    }

    /// Handle mouse up event
    fn handle_mouse_up(&self) {
        self.is_dragging.set(false);

        // Copy on select (disabled by default - TODO: make this configurable)
        // if let Some(selection) = self.selection.borrow().as_ref() {
        //     self.copy_selection_to_clipboard();
        // }
    }

    /// Select word at given position
    fn select_word_at(&self, pos: SelectionPos) {
        let screen = match self.state.screen.lock() {
            Ok(s) => s,
            Err(_) => return,
        };

        let scroll_offset = self.state.get_scroll_offset();
        let lines = if scroll_offset > 0 {
            screen.get_rendered_lines(scroll_offset)
        } else {
            screen.current_buffer().clone()
        };

        if pos.row >= lines.len() {
            return;
        }

        let line = &lines[pos.row];
        if pos.col >= line.len() {
            return;
        }

        // Find word boundaries
        let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

        // Find start of word
        let mut start_col = pos.col;
        while start_col > 0 && is_word_char(line[start_col - 1].c) {
            start_col -= 1;
        }

        // Find end of word
        let mut end_col = pos.col;
        while end_col < line.len() - 1 && is_word_char(line[end_col + 1].c) {
            end_col += 1;
        }

        *self.selection.borrow_mut() = Some(Selection::new(
            SelectionPos::new(pos.row, start_col),
            SelectionPos::new(pos.row, end_col),
        ));

        self.id.request_paint();
    }

    /// Select entire line at given position
    fn select_line_at(&self, pos: SelectionPos) {
        let (cols, _) = self.state.dimensions();

        *self.selection.borrow_mut() = Some(Selection::new(
            SelectionPos::new(pos.row, 0),
            SelectionPos::new(pos.row, cols.saturating_sub(1)),
        ));

        self.id.request_paint();
    }

    /// Get selected text as string
    fn get_selected_text(&self) -> Option<String> {
        let selection = self.selection.borrow();
        let selection = selection.as_ref()?;

        let screen = self.state.screen.lock().ok()?;
        let scroll_offset = self.state.get_scroll_offset();
        let lines = if scroll_offset > 0 {
            screen.get_rendered_lines(scroll_offset)
        } else {
            screen.current_buffer().clone()
        };

        let (start, end) = selection.normalized();
        let mut text = String::new();

        for row in start.row..=end.row {
            if row >= lines.len() {
                break;
            }

            let line = &lines[row];
            let start_col = if row == start.row { start.col } else { 0 };
            let end_col = if row == end.row {
                end.col.min(line.len().saturating_sub(1))
            } else {
                line.len().saturating_sub(1)
            };

            for col in start_col..=end_col {
                if col < line.len() {
                    let cell = &line[col];
                    // Skip placeholder cells for wide characters
                    if !cell.placeholder {
                        text.push(cell.c);
                    }
                }
            }

            // Add newline for multi-line selections (except last line)
            if row < end.row {
                text.push('\n');
            }
        }

        Some(text)
    }

    /// Copy selection to clipboard
    fn copy_selection_to_clipboard(&self) {
        if let Some(text) = self.get_selected_text() {
            if text.is_empty() {
                return;
            }

            match arboard::Clipboard::new() {
                Ok(mut clipboard) => {
                    if let Err(e) = clipboard.set_text(&text) {
                        tracing::error!("Failed to copy to clipboard: {}", e);
                    } else {
                        tracing::debug!("Copied {} bytes to clipboard", text.len());
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to create clipboard instance: {}", e);
                }
            }
        }
    }

    /// Clear selection
    #[allow(dead_code)]
    fn clear_selection(&self) {
        *self.selection.borrow_mut() = None;
        self.id.request_paint();
    }

    /// Draw cursor with different styles and focus awareness
    #[allow(dead_code)]
    fn draw_cursor(
        &self,
        cx: &mut floem::context::PaintCx,
        x: f64,
        y: f64,
        cursor_style: CursorStyle,
        is_focused: bool,
        blink_on: bool,
    ) {
        // Don't draw cursor if it's supposed to be hidden
        if !blink_on && is_focused {
            return;
        }

        let (cell_width, cell_height) = self.cell_dimensions();

        let cursor_color = if is_focused {
            colors::ACCENT_BLUE
        } else {
            // Dimmer color when unfocused
            Color::rgba(0.4, 0.4, 0.4, 0.8)
        };

        match cursor_style {
            CursorStyle::Block => {
                let cursor_rect = Rect::new(
                    x,
                    y,
                    x + cell_width,
                    y + cell_height,
                );

                if is_focused && blink_on {
                    // Filled block when focused and visible
                    cx.fill(&cursor_rect, Color::rgba(0.9, 0.9, 0.9, 0.3), 0.0);
                }

                // Always draw outline
                cx.stroke(
                    &cursor_rect,
                    cursor_color,
                    &floem::peniko::kurbo::Stroke::new(if is_focused { 2.0 } else { 1.0 }),
                );
            }
            CursorStyle::Underline => {
                // Underline cursor (2px height at bottom of cell)
                let underline_rect = Rect::new(
                    x,
                    y + cell_height - 2.0,
                    x + cell_width,
                    y + cell_height,
                );

                cx.fill(&underline_rect, cursor_color, 0.0);
            }
            CursorStyle::Bar => {
                // Vertical bar cursor (2px width at left of cell)
                let bar_rect = Rect::new(
                    x,
                    y,
                    x + 2.0,
                    y + cell_height,
                );

                cx.fill(&bar_rect, cursor_color, 0.0);
            }
        }
    }
}

impl View for TerminalCanvas {
    fn id(&self) -> ViewId {
        self.id
    }

    fn event_before_children(
        &mut self,
        _cx: &mut floem::context::EventCx,
        event: &floem::event::Event,
    ) -> floem::event::EventPropagation {
        match event {
            floem::event::Event::PointerDown(e) => {
                self.handle_mouse_down(e.pos.x, e.pos.y, &e.modifiers);
                floem::event::EventPropagation::Stop
            }
            floem::event::Event::PointerMove(e) => {
                self.handle_mouse_move(e.pos.x, e.pos.y);
                if self.is_dragging.get() {
                    floem::event::EventPropagation::Stop
                } else {
                    floem::event::EventPropagation::Continue
                }
            }
            floem::event::Event::PointerUp(_e) => {
                self.handle_mouse_up();
                floem::event::EventPropagation::Stop
            }
            floem::event::Event::KeyDown(key_event) => {
                // Handle Cmd+C for copy
                if key_event.modifiers.meta()
                    && matches!(key_event.key.logical_key, Key::Character(ref s) if s.as_str() == "c")
                {
                    self.copy_selection_to_clipboard();
                    return floem::event::EventPropagation::Stop;
                }
                floem::event::EventPropagation::Continue
            }
            _ => floem::event::EventPropagation::Continue,
        }
    }

    fn paint(&mut self, cx: &mut floem::context::PaintCx) {
        // Paint is called whenever request_paint() is invoked (including from PTY thread)

        // PERFORMANCE: Use get_untracked() in paint to avoid signal subscription overhead
        // paint() is called frequently and shouldn't create reactive dependencies

        // Check if font size changed and invalidate cache if needed
        let font_changed = self.check_font_size_changed();

        // Get dynamic cell dimensions based on current font size
        let (cell_width, cell_height) = self.cell_dimensions();
        let font_size = self.app_state.font_size.get_untracked();

        // Get layout size from Floem
        let layout = self.id.get_layout().unwrap_or_default();
        let canvas_width = layout.size.width as f64;
        let canvas_height = layout.size.height as f64;

        // Handle resize if canvas size changed or font changed
        if font_changed {
            self.last_size.set((0.0, 0.0)); // Force resize recalculation
        }
        self.handle_resize(canvas_width, canvas_height);

        let screen = match self.state.screen.lock() {
            Ok(s) => s,
            Err(_) => return,
        };

        let (cols, rows) = screen.dimensions();
        let scroll_offset = self.state.get_scroll_offset();

        // Get lines to render (with scrollback if scrolled up)
        let lines = if scroll_offset > 0 {
            screen.get_rendered_lines(scroll_offset)
        } else {
            screen.current_buffer().clone()
        };

        let (cursor_row, cursor_col) = screen.cursor_position();
        let cursor_visible = screen.cursor_visible();

        // Hide cursor if we're scrolled up
        let cursor_visible = cursor_visible && scroll_offset == 0;

        let canvas_rect = Rect::new(
            0.0,
            0.0,
            canvas_width,
            canvas_height,
        );

        // PERFORMANCE: Get colors without signal tracking in paint
        let colors = self.app_state.theme.get_untracked().colors();

        // Draw background
        cx.fill(&canvas_rect, colors.bg_primary, 0.0);

        // Get current selection for rendering
        let selection = self.selection.borrow();

        // Render all cells (dirty tracking disabled for cross-thread repaint compatibility)
        for (row_idx, row) in lines.iter().enumerate().take(rows) {
            for (col_idx, cell) in row.iter().enumerate().take(cols) {
                let x = col_idx as f64 * cell_width;
                let y = row_idx as f64 * cell_height;

                // Check if this cell is selected
                let is_selected = selection
                    .as_ref()
                    .map(|sel| sel.contains(row_idx, col_idx))
                    .unwrap_or(false);

                // Handle reverse video: swap foreground and background colors
                let (mut fg_color, mut bg_color) = if cell.reverse {
                    // Reverse video: swap fg and bg
                    let fg = cell.bg.as_ref()
                        .map(ansi_to_floem_color)
                        .unwrap_or(colors.bg_primary);
                    let bg = cell.fg.as_ref()
                        .map(ansi_to_floem_color)
                        .unwrap_or(colors.text_primary);
                    (fg, Some(bg))
                } else {
                    // Normal: use standard colors
                    let fg = cell.fg.as_ref()
                        .map(ansi_to_floem_color)
                        .unwrap_or(colors.text_primary);
                    let bg = cell.bg.as_ref().map(ansi_to_floem_color);
                    (fg, bg)
                };

                // Apply selection highlight
                if is_selected {
                    // Use a blue highlight for selected text
                    bg_color = Some(Color::rgba(0.2, 0.4, 0.8, 0.4));
                    // Ensure text is visible on selection background
                    fg_color = colors.text_primary;
                }

                // Apply dim attribute by reducing color intensity
                if cell.dim {
                    // Floem's Color doesn't expose r,g,b fields, so we use multiply_alpha to dim
                    // This is a workaround - ideally we'd multiply RGB by 0.5

                // Apply search match highlight (higher priority than selection)
                if self.state.is_current_search_match(row_idx, col_idx) {
                    // Current search match: bright yellow/orange highlight
                    bg_color = Some(Color::rgba(1.0, 0.6, 0.0, 0.6));
                    fg_color = Color::rgb(0.0, 0.0, 0.0); // Black text for contrast
                } else if self.state.is_search_match(row_idx, col_idx) {
                    // Other search matches: lighter yellow highlight
                    bg_color = Some(Color::rgba(1.0, 1.0, 0.0, 0.3));
                }
                    fg_color = fg_color.multiply_alpha(0.6);
                }

                // Draw cell background if present
                if let Some(bg) = bg_color {
                    let cell_rect = Rect::new(
                        x,
                        y,
                        x + cell_width,
                        y + cell_height,
                    );
                    cx.fill(&cell_rect, bg, 0.0);
                }

                // OPTIMIZATION: Draw character using cached text layouts
                if cell.c != ' ' && cell.c != '\0' {
                    let text_layout = self.get_cached_text_layout(
                        cell.c,
                        cell.bold,
                        cell.italic,
                        fg_color,
                        font_size,
                    );

                    // Draw the text at cell position (with vertical centering adjustment)
                    let text_offset = (cell_height - font_size as f64) / 2.0;
                    cx.draw_text(&text_layout, (x, y + text_offset));
                }

                // Draw underline if attribute is set
                if cell.underline {
                    let underline_y = y + cell_height - 2.0;
                    let underline = Line::new(
                        (x, underline_y),
                        (x + cell_width, underline_y)
                    );
                    cx.stroke(&underline, fg_color, &Stroke::new(1.0));
                }

                // Draw strikethrough if attribute is set
                if cell.strikethrough {
                    let strike_y = y + cell_height / 2.0;
                    let strike = Line::new(
                        (x, strike_y),
                        (x + cell_width, strike_y)
                    );
                    cx.stroke(&strike, fg_color, &Stroke::new(1.0));
                }
            }
        }

        // Draw cursor (only when focused)
        let is_focused = self.is_focused.get_untracked();
        if is_focused && cursor_visible && cursor_row < rows && cursor_col < cols {
            let cursor_x = cursor_col as f64 * cell_width;
            let cursor_y = cursor_row as f64 * cell_height;

            let cursor_rect = Rect::new(
                cursor_x,
                cursor_y,
                cursor_x + cell_width,
                cursor_y + cell_height,
            );

            // Draw cursor as outlined rectangle (use untracked in paint)
            let colors = self.app_state.theme.get_untracked().colors();
            cx.stroke(
                &cursor_rect,
                colors.accent_blue,
                &floem::peniko::kurbo::Stroke::new(2.0),
            );
        }

        // Draw dim overlay for inactive panes
        if !is_focused {
            let overlay_color = Color::rgba8(0, 0, 0, 100); // Semi-transparent black
            cx.fill(&canvas_rect, overlay_color, 0.0);
        }
    }

    fn update(
        &mut self,
        _cx: &mut floem::context::UpdateCx,
        _state: Box<dyn std::any::Any>,
    ) {
        // Track content version to trigger repaints when terminal updates
        let _version = self.state.content_version.get();

        // Request paint when state changes
        self.id.request_paint();
    }
}

/// Terminal area view - displays the active tab's pane tree
pub fn terminal_area(state: &AppState) -> impl IntoView {
    let app_state = state.clone();
    let state_fallback = state.clone();
    let state_fallback2 = state.clone();
    let state_bg = state.clone();

    // Get the active tab
    let active_tab = match state.active_tab_ref() {
        Some(tab) => tab,
        None => {
            // Fallback: no active tab
            tracing::warn!("No active terminal tab available");
            return container(
                floem::views::label(|| "No active terminal")
                    .style(move |s| {
                        let colors = state_fallback.colors();
                        s.color(colors.text_secondary).font_size(14.0)
                    })
            )
            .style(move |s| {
                let colors = state_fallback2.colors();
                s.flex_grow(1.0)
                    .width_full()
                    .height_full()
                    .justify_center()
                    .items_center()
                    .background(colors.bg_primary)
            })
            .into_any();
        }
    };

    let pane_tree_signal = active_tab.pane_tree;

    // Render the pane tree
    container(
        crate::floem_app::views::pane_tree_view(pane_tree_signal, app_state)
    )
    .style(move |s| {
        let colors = state_bg.colors();
        s.flex_grow(1.0)
            .width_full()
            .height_full()
            .background(colors.bg_primary)
    })
    .into_any()
}

/// Create a terminal canvas view (when needed)
#[allow(dead_code)]
pub fn terminal_canvas_view(
    terminal_state: TerminalState,
    app_state: AppState,
    is_focused: RwSignal<bool>,
) -> TerminalCanvas {
    TerminalCanvas::new(terminal_state, app_state, is_focused)
}
