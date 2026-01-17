//! Terminal screen buffer with ANSI escape code parsing

use iced::Color;
use std::cmp::{max, min};
use std::collections::VecDeque;
use unicode_width::UnicodeWidthChar;
use vte::{Params, Parser, Perform};

/// Maximum scrollback buffer lines
const MAX_SCROLLBACK: usize = 10000;

/// ANSI color (16-color palette + 256-color + RGB)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnsiColor {
    /// Standard 16-color palette (0-15)
    Indexed(u8),
    /// 256-color palette (0-255)
    Palette256(u8),
    /// RGB color
    Rgb(u8, u8, u8),
}

impl AnsiColor {
    /// Convert ANSI color to Iced Color
    pub fn to_color(&self) -> Color {
        match self {
            AnsiColor::Indexed(idx) => indexed_to_color(*idx),
            AnsiColor::Palette256(idx) => palette256_to_color(*idx),
            AnsiColor::Rgb(r, g, b) => {
                Color::from_rgb(*r as f32 / 255.0, *g as f32 / 255.0, *b as f32 / 255.0)
            }
        }
    }
}

/// Convert 16-color index to Iced Color
fn indexed_to_color(idx: u8) -> Color {
    match idx {
        0 => Color::from_rgb(0.0, 0.0, 0.0),  // Black
        1 => Color::from_rgb(0.8, 0.2, 0.2),  // Red
        2 => Color::from_rgb(0.2, 0.8, 0.2),  // Green
        3 => Color::from_rgb(0.8, 0.8, 0.2),  // Yellow
        4 => Color::from_rgb(0.2, 0.2, 0.8),  // Blue
        5 => Color::from_rgb(0.8, 0.2, 0.8),  // Magenta
        6 => Color::from_rgb(0.2, 0.8, 0.8),  // Cyan
        7 => Color::from_rgb(0.8, 0.8, 0.8),  // White
        8 => Color::from_rgb(0.5, 0.5, 0.5),  // Bright Black (Gray)
        9 => Color::from_rgb(1.0, 0.3, 0.3),  // Bright Red
        10 => Color::from_rgb(0.3, 1.0, 0.3), // Bright Green
        11 => Color::from_rgb(1.0, 1.0, 0.3), // Bright Yellow
        12 => Color::from_rgb(0.3, 0.3, 1.0), // Bright Blue
        13 => Color::from_rgb(1.0, 0.3, 1.0), // Bright Magenta
        14 => Color::from_rgb(0.3, 1.0, 1.0), // Bright Cyan
        15 => Color::from_rgb(1.0, 1.0, 1.0), // Bright White
        _ => Color::from_rgb(0.8, 0.8, 0.8),  // Default
    }
}

/// Convert 256-color palette index to Iced Color
fn palette256_to_color(idx: u8) -> Color {
    match idx {
        // 0-15: Standard colors
        0..=15 => indexed_to_color(idx),
        // 16-231: 6x6x6 color cube
        16..=231 => {
            let idx = idx - 16;
            let r = (idx / 36) * 51;
            let g = ((idx % 36) / 6) * 51;
            let b = (idx % 6) * 51;
            Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
        }
        // 232-255: Grayscale
        232..=255 => {
            let gray = 8 + (idx - 232) * 10;
            Color::from_rgb(
                gray as f32 / 255.0,
                gray as f32 / 255.0,
                gray as f32 / 255.0,
            )
        }
    }
}

/// Mouse reporting mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MouseMode {
    /// No mouse reporting
    #[default]
    None,
    /// X10 Mouse Reporting (CSI ?9h or CSI ?1000h)
    X10,
    /// Button-Event Mouse Tracking (CSI ?1002h)
    ButtonEvent,
    /// Any-Event Mouse Tracking (CSI ?1003h)
    AnyEvent,
}

/// Mouse encoding mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MouseEncoding {
    /// Default X10/UTF-8 encoding
    #[default]
    Default,
    /// SGR Extended Mouse Mode (CSI ?1006h)
    Sgr,
}

/// Terminal cell with character and styling
#[derive(Clone, Debug)]
pub struct Cell {
    pub c: char,
    pub fg: Option<AnsiColor>,
    /// Background color (reserved for future rendering enhancement)
    #[allow(dead_code)]
    pub bg: Option<AnsiColor>,
    pub bold: bool,
    pub underline: bool,
    /// Reverse video (reserved for future rendering enhancement)
    #[allow(dead_code)]
    pub reverse: bool,
    pub dim: bool,
    pub italic: bool,
    pub strikethrough: bool,
    /// This cell is the first cell of a wide character (CJK, emoji, etc.)
    pub wide: bool,
    /// This cell is a placeholder for the second cell of a wide character
    pub placeholder: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            fg: None,
            bg: None,
            bold: false,
            underline: false,
            reverse: false,
            dim: false,
            italic: false,
            strikethrough: false,
            wide: false,
            placeholder: false,
        }
    }
}

/// Saved state for alternate screen buffer switching.
///
/// Stores complete terminal state when entering alternate screen,
/// allowing full restoration when returning to main screen.
#[derive(Clone, Debug)]
struct AlternateScreenState {
    /// Cursor position (row, col)
    cursor_pos: (usize, usize),
    /// Current text attributes
    current_fg: Option<AnsiColor>,
    current_bg: Option<AnsiColor>,
    bold: bool,
    underline: bool,
    reverse: bool,
    dim: bool,
    italic: bool,
    strikethrough: bool,
    /// Scroll region
    scroll_region: Option<(usize, usize)>,
    /// Saved cursor for DECSC/DECRC
    saved_cursor: Option<(usize, usize)>,
    /// Saved cursor state (DECSC)
    saved_cursor_state: Option<(usize, usize, Option<AnsiColor>, Option<AnsiColor>, bool)>,
}

/// Terminal screen buffer with VTE parser
pub struct TerminalScreen {
    cols: usize,
    rows: usize,
    /// Screen buffer (rows x cols) - only visible lines
    buffer: Vec<Vec<Cell>>,
    /// Scrollback buffer (historical lines)
    scrollback: VecDeque<Vec<Cell>>,
    /// Cursor position
    cursor_row: usize,
    cursor_col: usize,
    /// Current text attributes
    current_fg: Option<AnsiColor>,
    current_bg: Option<AnsiColor>,
    bold: bool,
    underline: bool,
    reverse: bool,
    dim: bool,
    italic: bool,
    strikethrough: bool,
    /// VTE parser
    parser: Parser,
    /// Scroll region (top, bottom) - None means full screen
    scroll_region: Option<(usize, usize)>,
    /// Saved cursor position (for save/restore)
    saved_cursor: Option<(usize, usize)>,
    /// Saved cursor state (DECSC) - position, fg, bg, bold
    saved_cursor_state: Option<(usize, usize, Option<AnsiColor>, Option<AnsiColor>, bool)>,
    /// OSC sequence fields
    /// Window title (OSC 0 or OSC 2)
    window_title: Option<String>,
    /// Icon name (OSC 1)
    icon_name: Option<String>,
    /// Current working directory from shell (OSC 7)
    cwd_from_shell: Option<String>,
    /// Clipboard request data (OSC 52)
    clipboard_request: Option<String>,
    /// Alternate screen buffer (for applications like vim, less, etc.)
    alternate_buffer: Option<Vec<Vec<Cell>>>,
    /// Alternate scrollback buffer
    alternate_scrollback: Option<VecDeque<Vec<Cell>>>,
    /// Whether we're currently using the alternate screen
    use_alternate_screen: bool,
    /// Saved main screen state (cursor pos, attributes, scroll region, etc.)
    alternate_saved_state: Option<AlternateScreenState>,
    /// Mouse reporting mode
    mouse_mode: MouseMode,
    /// Mouse encoding mode
    mouse_encoding: MouseEncoding,
    /// Cursor visibility (DECTCEM)
    cursor_visible: bool,
    /// Auto-wrap mode (DECAWM) - default true
    auto_wrap_mode: bool,
    /// Bracketed paste mode (CSI ?2004h/l) - tracked for terminal compatibility
    #[allow(dead_code)]
    bracketed_paste_mode: bool,
    /// Application cursor keys mode (DECCKM - CSI ?1h/l)
    application_cursor_keys: bool,
    /// Pending responses to be sent to PTY (for DA, DSR, CPR, etc.)
    pending_responses: Vec<String>,
    /// Bell (BEL) triggered flag - set when \x07 is received
    bell_triggered: bool,
}

impl TerminalScreen {
    /// Create a new terminal screen
    pub fn new(cols: usize, rows: usize) -> Self {
        let cols = max(1, cols);
        let rows = max(1, rows);

        Self {
            cols,
            rows,
            buffer: vec![vec![Cell::default(); cols]; rows],
            scrollback: VecDeque::new(),
            cursor_row: 0,
            cursor_col: 0,
            current_fg: None,
            current_bg: None,
            bold: false,
            underline: false,
            reverse: false,
            dim: false,
            italic: false,
            strikethrough: false,
            parser: Parser::new(),
            scroll_region: None,
            saved_cursor: None,
            saved_cursor_state: None,
            window_title: None,
            icon_name: None,
            cwd_from_shell: None,
            clipboard_request: None,
            alternate_buffer: None,
            alternate_scrollback: None,
            use_alternate_screen: false,
            alternate_saved_state: None,
            mouse_mode: MouseMode::None,
            mouse_encoding: MouseEncoding::Default,
            cursor_visible: true,
            auto_wrap_mode: true,
            bracketed_paste_mode: false,
            application_cursor_keys: false,
            pending_responses: Vec::new(),
            bell_triggered: false,
        }
    }

    /// Process incoming bytes through VTE parser
    pub fn process(&mut self, bytes: &[u8]) {
        // We need to temporarily take the parser to avoid borrow checker issues
        let mut parser = std::mem::replace(&mut self.parser, Parser::new());
        for byte in bytes {
            parser.advance(self, *byte);
        }
        self.parser = parser;
    }

    /// Resize the terminal with content preservation
    pub fn resize(&mut self, cols: usize, rows: usize) {
        let cols = max(1, cols);
        let rows = max(1, rows);

        if cols == self.cols && rows == self.rows {
            return;
        }

        // Save old dimensions
        let old_cols = self.cols;
        let old_rows = self.rows;

        // Handle column resize first (reflow lines)
        let mut new_buffer = Vec::new();

        if cols != old_cols {
            // Reflow each line to new width
            for row in &self.buffer {
                let mut new_row = Vec::with_capacity(cols);

                if cols > old_cols {
                    // Columns increased: extend with empty cells
                    new_row.extend_from_slice(&row[..old_cols]);
                    new_row.resize(cols, Cell::default());
                } else {
                    // Columns decreased: truncate long lines
                    new_row.extend_from_slice(&row[..cols]);
                }

                new_buffer.push(new_row);
            }
        } else {
            // No column change, just copy buffer
            new_buffer = self.buffer.clone();
        }

        // Handle row resize
        if rows < old_rows {
            // Rows decreased: move top lines to scrollback
            let lines_to_save = old_rows - rows;
            for i in 0..lines_to_save {
                if i < new_buffer.len() {
                    self.scrollback.push_back(new_buffer[i].clone());
                }
            }

            // Keep only the bottom 'rows' lines
            if new_buffer.len() > rows {
                new_buffer.drain(0..lines_to_save);
            }

            // Limit scrollback
            while self.scrollback.len() > MAX_SCROLLBACK {
                self.scrollback.pop_front();
            }
        } else if rows > old_rows {
            // Rows increased: restore from scrollback if available
            let lines_to_restore = min(rows - old_rows, self.scrollback.len());

            // Restore lines from scrollback (from the end)
            let mut restored_lines = Vec::new();
            for _ in 0..lines_to_restore {
                if let Some(mut line) = self.scrollback.pop_back() {
                    // Reflow restored line to new column width
                    if cols != old_cols {
                        line.resize(cols, Cell::default());
                    }
                    restored_lines.push(line);
                }
            }

            // Insert restored lines at the beginning
            restored_lines.reverse();
            new_buffer.splice(0..0, restored_lines);

            // If still need more rows, add empty ones at the end
            while new_buffer.len() < rows {
                new_buffer.push(vec![Cell::default(); cols]);
            }
        }

        // Ensure buffer is exactly 'rows' lines
        new_buffer.resize(rows, vec![Cell::default(); cols]);

        // Update terminal state
        self.buffer = new_buffer;
        self.cols = cols;
        self.rows = rows;

        // Adjust cursor position to be within new bounds
        self.cursor_row = min(self.cursor_row, rows.saturating_sub(1));
        self.cursor_col = min(self.cursor_col, cols.saturating_sub(1));

        // Reset scroll region as it's no longer valid
        self.scroll_region = None;

        // Also handle alternate screen buffer if active
        if self.use_alternate_screen {
            if let Some(ref mut alt_buffer) = self.alternate_buffer {
                // Resize alternate buffer similarly
                let mut resized_alt = Vec::new();
                for row in alt_buffer.iter() {
                    let mut new_row = Vec::with_capacity(cols);
                    if cols > old_cols {
                        let copy_len = min(row.len(), old_cols);
                        new_row.extend_from_slice(&row[..copy_len]);
                        new_row.resize(cols, Cell::default());
                    } else {
                        let copy_len = min(row.len(), cols);
                        new_row.extend_from_slice(&row[..copy_len]);
                    }
                    resized_alt.push(new_row);
                }
                resized_alt.resize(rows, vec![Cell::default(); cols]);
                *alt_buffer = resized_alt;
            }
        }
    }

    /// Get all lines (scrollback + visible) for rendering
    pub fn get_all_lines(&self) -> Vec<Vec<Cell>> {
        let mut all_lines: Vec<Vec<Cell>> = self.scrollback.iter().cloned().collect();
        all_lines.extend(self.buffer.clone());
        all_lines
    }

    /// Get cursor position (row, col)
    pub fn cursor_position(&self) -> (usize, usize) {
        (self.cursor_row, self.cursor_col)
    }

    /// Get window title (OSC 0 or OSC 2) - reserved for future window title bar integration
    #[allow(dead_code)]
    pub fn window_title(&self) -> Option<&str> {
        self.window_title.as_deref()
    }

    /// Get icon name (OSC 1) - reserved for future icon integration
    #[allow(dead_code)]
    pub fn icon_name(&self) -> Option<&str> {
        self.icon_name.as_deref()
    }

    /// Get current working directory from shell (OSC 7) - reserved for future tab label enhancement
    #[allow(dead_code)]
    pub fn cwd_from_shell(&self) -> Option<&str> {
        self.cwd_from_shell.as_deref()
    }

    /// Get clipboard request data (OSC 52) - reserved for future clipboard integration
    #[allow(dead_code)]
    pub fn clipboard_request(&self) -> Option<&str> {
        self.clipboard_request.as_deref()
    }

    /// Clear clipboard request after it has been read - reserved for future clipboard integration
    #[allow(dead_code)]
    pub fn clear_clipboard_request(&mut self) {
        self.clipboard_request = None;
    }

    /// Parse file:// URI to extract path
    fn parse_file_uri(&self, uri: &str) -> Option<String> {
        if let Some(path_part) = uri.strip_prefix("file://") {
            // Handle both file://hostname/path and file:///path
            if let Some(slash_pos) = path_part.find('/') {
                // file://hostname/path -> extract /path
                Some(path_part[slash_pos..].to_string())
            } else if path_part.is_empty() {
                // file:/// -> next part is the path (malformed, but handle it)
                None
            } else {
                // file://path (no hostname) -> use as is
                Some(format!("/{}", path_part))
            }
        } else if uri.starts_with("file:/") {
            // file:/path (missing one slash)
            Some(uri.strip_prefix("file:").unwrap().to_string())
        } else {
            None
        }
    }

    /// Get current mouse reporting mode - reserved for future mouse interaction support
    #[allow(dead_code)]
    pub fn mouse_mode(&self) -> MouseMode {
        self.mouse_mode
    }

    /// Get current mouse encoding mode - reserved for future mouse interaction support
    #[allow(dead_code)]
    pub fn mouse_encoding(&self) -> MouseEncoding {
        self.mouse_encoding
    }

    /// Check if mouse reporting is enabled - reserved for future mouse interaction support
    #[allow(dead_code)]
    pub fn is_mouse_reporting_enabled(&self) -> bool {
        self.mouse_mode != MouseMode::None
    }

    /// Get cursor visibility state
    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    /// Check if bell was triggered and clear the flag (consume)
    pub fn take_bell_triggered(&mut self) -> bool {
        let triggered = self.bell_triggered;
        self.bell_triggered = false;
        triggered
    }

    /// Get auto-wrap mode state - reserved for future line wrapping logic enhancement
    #[allow(dead_code)]
    pub fn auto_wrap_mode(&self) -> bool {
        self.auto_wrap_mode
    }

    /// Get bracketed paste mode state - reserved for future paste handling
    #[allow(dead_code)]
    pub fn bracketed_paste_mode(&self) -> bool {
        self.bracketed_paste_mode
    }

    /// Get application cursor keys mode state (DECCKM)
    pub fn application_cursor_keys(&self) -> bool {
        self.application_cursor_keys
    }

    /// Take pending responses (for sending to PTY)
    /// This drains the pending_responses vec and returns it
    pub fn take_pending_responses(&mut self) -> Vec<String> {
        std::mem::take(&mut self.pending_responses)
    }

    /// Scroll screen up by n lines
    fn scroll_up(&mut self, n: usize) {
        let (top, bottom) = self.scroll_region.unwrap_or((0, self.rows - 1));

        for _ in 0..n {
            // Save top line to scrollback
            if top == 0 {
                self.scrollback.push_back(self.buffer[top].clone());
                if self.scrollback.len() > MAX_SCROLLBACK {
                    self.scrollback.pop_front();
                }
            }

            // Shift lines up within scroll region
            for row in top..bottom {
                self.buffer[row] = self.buffer[row + 1].clone();
            }

            // Clear bottom line
            self.buffer[bottom] = vec![Cell::default(); self.cols];
        }
    }

    /// Scroll screen down by n lines
    fn scroll_down(&mut self, n: usize) {
        let (top, bottom) = self.scroll_region.unwrap_or((0, self.rows - 1));

        for _ in 0..n {
            // Shift lines down within scroll region
            for row in (top + 1..=bottom).rev() {
                self.buffer[row] = self.buffer[row - 1].clone();
            }

            // Clear top line
            self.buffer[top] = vec![Cell::default(); self.cols];
        }
    }

    /// Insert n blank lines at cursor position (IL)
    /// Lines below cursor are pushed down within scroll region
    fn insert_lines(&mut self, n: usize) {
        let (top, bottom) = self.scroll_region.unwrap_or((0, self.rows - 1));
        let cursor_row = self.cursor_row;

        // IL only works within the scroll region
        if cursor_row < top || cursor_row > bottom {
            return;
        }

        let n = min(n, bottom - cursor_row + 1);

        // Move lines down within scroll region
        for _ in 0..n {
            // Shift lines down from cursor to bottom
            for row in ((cursor_row + 1)..=bottom).rev() {
                self.buffer[row] = self.buffer[row - 1].clone();
            }

            // Insert blank line at cursor position
            self.buffer[cursor_row] = vec![Cell::default(); self.cols];
        }
    }

    /// Delete n lines at cursor position (DL)
    /// Lines below cursor are pulled up within scroll region
    fn delete_lines(&mut self, n: usize) {
        let (top, bottom) = self.scroll_region.unwrap_or((0, self.rows - 1));
        let cursor_row = self.cursor_row;

        // DL only works within the scroll region
        if cursor_row < top || cursor_row > bottom {
            return;
        }

        let n = min(n, bottom - cursor_row + 1);

        // Move lines up within scroll region
        for _ in 0..n {
            // Shift lines up from cursor to bottom
            for row in cursor_row..bottom {
                self.buffer[row] = self.buffer[row + 1].clone();
            }

            // Clear the bottom line
            self.buffer[bottom] = vec![Cell::default(); self.cols];
        }
    }

    /// Move cursor to next line (with scrolling if needed)
    fn new_line(&mut self) {
        let (_, bottom) = self.scroll_region.unwrap_or((0, self.rows - 1));

        if self.cursor_row == bottom {
            self.scroll_up(1);
        } else if self.cursor_row < self.rows - 1 {
            self.cursor_row += 1;
        }
    }

    /// Carriage return
    fn carriage_return(&mut self) {
        self.cursor_col = 0;
    }

    /// Backspace
    fn backspace(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    /// Tab (move to next tab stop, assume every 8 columns)
    fn tab(&mut self) {
        let next_tab = ((self.cursor_col / 8) + 1) * 8;
        self.cursor_col = min(next_tab, self.cols - 1);
    }

    /// Erase in display (ED)
    fn erase_in_display(&mut self, mode: u16) {
        match mode {
            0 => {
                // Clear from cursor to end of screen
                // Clear current line from cursor to end
                for col in self.cursor_col..self.cols {
                    self.buffer[self.cursor_row][col] = Cell::default();
                }
                // Clear all lines below
                for row in (self.cursor_row + 1)..self.rows {
                    self.buffer[row] = vec![Cell::default(); self.cols];
                }
            }
            1 => {
                // Clear from beginning to cursor
                // Clear all lines above
                for row in 0..self.cursor_row {
                    self.buffer[row] = vec![Cell::default(); self.cols];
                }
                // Clear current line from beginning to cursor
                for col in 0..=self.cursor_col {
                    self.buffer[self.cursor_row][col] = Cell::default();
                }
            }
            2 | 3 => {
                // Clear entire screen (3 also clears scrollback)
                for row in 0..self.rows {
                    self.buffer[row] = vec![Cell::default(); self.cols];
                }
                if mode == 3 {
                    self.scrollback.clear();
                }
            }
            _ => {}
        }
    }

    /// Erase in line (EL)
    fn erase_in_line(&mut self, mode: u16) {
        match mode {
            0 => {
                // Clear from cursor to end of line
                for col in self.cursor_col..self.cols {
                    self.buffer[self.cursor_row][col] = Cell::default();
                }
            }
            1 => {
                // Clear from beginning to cursor
                for col in 0..=self.cursor_col {
                    self.buffer[self.cursor_row][col] = Cell::default();
                }
            }
            2 => {
                // Clear entire line
                self.buffer[self.cursor_row] = vec![Cell::default(); self.cols];
            }
            _ => {}
        }
    }

    /// Insert characters (ICH) - CSI n @
    /// Insert n blank characters at cursor position, shifting existing characters to the right
    fn insert_characters(&mut self, n: usize) {
        if self.cursor_row >= self.rows || self.cursor_col >= self.cols {
            return;
        }

        let n = max(1, min(n, self.cols - self.cursor_col));
        let row = self.cursor_row;
        let col = self.cursor_col;

        // Shift existing characters to the right
        for i in (col..(self.cols - n)).rev() {
            self.buffer[row][i + n] = self.buffer[row][i].clone();
        }

        // Insert blank characters at cursor position
        for i in col..(col + n) {
            self.buffer[row][i] = Cell::default();
        }
    }

    /// Delete characters (DCH) - CSI n P
    /// Delete n characters at cursor position, shifting remaining characters to the left
    fn delete_characters(&mut self, n: usize) {
        if self.cursor_row >= self.rows || self.cursor_col >= self.cols {
            return;
        }

        let n = max(1, min(n, self.cols - self.cursor_col));
        let row = self.cursor_row;
        let col = self.cursor_col;

        // Shift remaining characters to the left
        for i in col..(self.cols - n) {
            self.buffer[row][i] = self.buffer[row][i + n].clone();
        }

        // Fill the end of the line with blank characters
        for i in (self.cols - n)..self.cols {
            self.buffer[row][i] = Cell::default();
        }
    }

    /// Switch to alternate screen buffer (CSI ?47h, ?1047h, ?1049h).
    ///
    /// Saves complete main screen state including:
    /// - Buffer contents and scrollback
    /// - Cursor position and attributes
    /// - Scroll region
    /// - DECSC saved cursor state
    ///
    /// The `clear_screen` parameter determines if the alternate screen should be cleared.
    fn switch_to_alternate_screen(&mut self, clear_screen: bool) {
        if !self.use_alternate_screen {
            // Save complete main screen state
            self.alternate_buffer = Some(self.buffer.clone());
            self.alternate_scrollback = Some(self.scrollback.clone());
            self.alternate_saved_state = Some(AlternateScreenState {
                cursor_pos: (self.cursor_row, self.cursor_col),
                current_fg: self.current_fg,
                current_bg: self.current_bg,
                bold: self.bold,
                underline: self.underline,
                reverse: self.reverse,
                dim: self.dim,
                italic: self.italic,
                strikethrough: self.strikethrough,
                scroll_region: self.scroll_region,
                saved_cursor: self.saved_cursor,
                saved_cursor_state: self.saved_cursor_state.clone(),
            });

            // Create alternate screen buffer
            if clear_screen {
                // Clear alternate screen (CSI ?1047h, ?1049h)
                self.buffer = vec![vec![Cell::default(); self.cols]; self.rows];
            } else {
                // Keep current content (CSI ?47h - simple buffer switch)
                // This is rarely used, but some apps might rely on it
                self.buffer = vec![vec![Cell::default(); self.cols]; self.rows];
            }

            self.scrollback = VecDeque::new();
            self.cursor_row = 0;
            self.cursor_col = 0;
            self.scroll_region = None;
            self.saved_cursor = None;
            self.saved_cursor_state = None;

            // Reset text attributes to defaults in alternate screen
            self.current_fg = None;
            self.current_bg = None;
            self.bold = false;
            self.underline = false;
            self.reverse = false;
            self.dim = false;
            self.italic = false;
            self.strikethrough = false;

            self.use_alternate_screen = true;
        }
    }

    /// Switch back to normal screen buffer (CSI ?47l, ?1047l, ?1049l).
    ///
    /// Fully restores main screen state including:
    /// - Buffer contents and scrollback
    /// - Cursor position and attributes
    /// - Scroll region
    /// - DECSC saved cursor state
    fn switch_to_normal_screen(&mut self) {
        if self.use_alternate_screen {
            // Restore buffer and scrollback
            if let Some(saved_buffer) = self.alternate_buffer.take() {
                self.buffer = saved_buffer;
            }
            if let Some(saved_scrollback) = self.alternate_scrollback.take() {
                self.scrollback = saved_scrollback;
            }

            // Restore complete terminal state
            if let Some(state) = self.alternate_saved_state.take() {
                self.cursor_row = min(state.cursor_pos.0, self.rows.saturating_sub(1));
                self.cursor_col = min(state.cursor_pos.1, self.cols.saturating_sub(1));
                self.current_fg = state.current_fg;
                self.current_bg = state.current_bg;
                self.bold = state.bold;
                self.underline = state.underline;
                self.reverse = state.reverse;
                self.dim = state.dim;
                self.italic = state.italic;
                self.strikethrough = state.strikethrough;
                self.scroll_region = state.scroll_region;
                self.saved_cursor = state.saved_cursor;
                self.saved_cursor_state = state.saved_cursor_state;
            }

            self.use_alternate_screen = false;
        }
    }

    /// Set graphics rendition (SGR)
    fn set_sgr(&mut self, params: &Params) {
        let mut iter = params.iter();

        while let Some(param) = iter.next() {
            let value = param[0];
            match value {
                0 => {
                    // Reset all attributes
                    self.current_fg = None;
                    self.current_bg = None;
                    self.bold = false;
                    self.underline = false;
                    self.reverse = false;
                    self.dim = false;
                    self.italic = false;
                    self.strikethrough = false;
                }
                1 => self.bold = true,
                2 => self.dim = true,
                3 => self.italic = true,
                4 => self.underline = true,
                7 => self.reverse = true,
                9 => self.strikethrough = true,
                22 => {
                    // Normal intensity (not bold and not dim)
                    self.bold = false;
                    self.dim = false;
                }
                23 => self.italic = false,
                24 => self.underline = false,
                27 => self.reverse = false,
                29 => self.strikethrough = false,
                // Foreground colors (30-37, 90-97)
                30..=37 => self.current_fg = Some(AnsiColor::Indexed((value - 30) as u8)),
                38 => {
                    // Extended foreground color
                    if let Some(next) = iter.next() {
                        match next[0] {
                            2 => {
                                // RGB
                                if let (Some(r), Some(g), Some(b)) =
                                    (iter.next(), iter.next(), iter.next())
                                {
                                    self.current_fg =
                                        Some(AnsiColor::Rgb(r[0] as u8, g[0] as u8, b[0] as u8));
                                }
                            }
                            5 => {
                                // 256-color palette
                                if let Some(idx) = iter.next() {
                                    self.current_fg = Some(AnsiColor::Palette256(idx[0] as u8));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                39 => self.current_fg = None, // Default foreground
                // Background colors (40-47, 100-107)
                40..=47 => self.current_bg = Some(AnsiColor::Indexed((value - 40) as u8)),
                48 => {
                    // Extended background color
                    if let Some(next) = iter.next() {
                        match next[0] {
                            2 => {
                                // RGB
                                if let (Some(r), Some(g), Some(b)) =
                                    (iter.next(), iter.next(), iter.next())
                                {
                                    self.current_bg =
                                        Some(AnsiColor::Rgb(r[0] as u8, g[0] as u8, b[0] as u8));
                                }
                            }
                            5 => {
                                // 256-color palette
                                if let Some(idx) = iter.next() {
                                    self.current_bg = Some(AnsiColor::Palette256(idx[0] as u8));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                49 => self.current_bg = None, // Default background
                // Bright foreground colors (90-97)
                90..=97 => self.current_fg = Some(AnsiColor::Indexed((value - 90 + 8) as u8)),
                // Bright background colors (100-107)
                100..=107 => self.current_bg = Some(AnsiColor::Indexed((value - 100 + 8) as u8)),
                _ => {}
            }
        }
    }
}

impl Perform for TerminalScreen {
    fn print(&mut self, c: char) {
        // Get character width (1 for regular, 2 for wide characters like CJK)
        let width = UnicodeWidthChar::width(c).unwrap_or(1);

        // Handle line wrapping based on auto_wrap_mode
        // For wide characters, check if we have enough space (need 2 columns)
        if self.cursor_col >= self.cols || (width == 2 && self.cursor_col >= self.cols - 1) {
            if self.auto_wrap_mode {
                // Auto-wrap enabled: move to next line
                self.cursor_col = 0;
                self.new_line();
            } else {
                // Auto-wrap disabled: stay at last column (overwrite)
                self.cursor_col = self.cols - 1;
                // For wide characters at the edge, skip them
                if width == 2 {
                    return;
                }
            }
        }

        // Write character
        if self.cursor_row < self.rows && self.cursor_col < self.cols {
            // If this is overwriting a wide character placeholder, clear the wide char too
            if self.buffer[self.cursor_row][self.cursor_col].placeholder && self.cursor_col > 0 {
                self.buffer[self.cursor_row][self.cursor_col - 1] = Cell::default();
            }

            // If this is overwriting a wide character, clear its placeholder too
            if self.buffer[self.cursor_row][self.cursor_col].wide && self.cursor_col + 1 < self.cols
            {
                self.buffer[self.cursor_row][self.cursor_col + 1] = Cell::default();
            }

            if width == 2 {
                // Wide character: write to first cell and placeholder to second cell
                self.buffer[self.cursor_row][self.cursor_col] = Cell {
                    c,
                    fg: self.current_fg,
                    bg: self.current_bg,
                    bold: self.bold,
                    underline: self.underline,
                    reverse: self.reverse,
                    dim: self.dim,
                    italic: self.italic,
                    strikethrough: self.strikethrough,
                    wide: true,
                    placeholder: false,
                };

                // Write placeholder to next cell if there's space
                if self.cursor_col + 1 < self.cols {
                    self.buffer[self.cursor_row][self.cursor_col + 1] = Cell {
                        c: ' ', // Placeholder character
                        fg: self.current_fg,
                        bg: self.current_bg,
                        bold: self.bold,
                        underline: self.underline,
                        reverse: self.reverse,
                        dim: self.dim,
                        italic: self.italic,
                        strikethrough: self.strikethrough,
                        wide: false,
                        placeholder: true,
                    };
                }

                self.cursor_col += 2;
            } else {
                // Regular character: write normally
                self.buffer[self.cursor_row][self.cursor_col] = Cell {
                    c,
                    fg: self.current_fg,
                    bg: self.current_bg,
                    bold: self.bold,
                    underline: self.underline,
                    reverse: self.reverse,
                    dim: self.dim,
                    italic: self.italic,
                    strikethrough: self.strikethrough,
                    wide: false,
                    placeholder: false,
                };
                self.cursor_col += 1;
            }
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                // Line Feed (LF)
                self.new_line();
            }
            b'\r' => {
                // Carriage Return (CR)
                self.carriage_return();
            }
            b'\x08' => {
                // Backspace (BS)
                self.backspace();
            }
            b'\t' => {
                // Tab
                self.tab();
            }
            0x07 => {
                // Bell (BEL) - ignore for now
            }
            _ => {}
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {
        // DCS sequences - not implemented yet
    }

    fn put(&mut self, _byte: u8) {
        // DCS data - not implemented yet
    }

    fn unhook(&mut self) {
        // End of DCS - not implemented yet
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        // OSC sequences: ESC ] <command> ; <data> ST (or BEL)
        // Parse OSC command number and data
        if params.is_empty() {
            return;
        }

        // First param is the command number
        let command_str = String::from_utf8_lossy(params[0]);
        let command = command_str.parse::<u16>().unwrap_or(0);

        match command {
            0 => {
                // OSC 0 ; title - Set icon name and window title
                if params.len() > 1 {
                    let title = String::from_utf8_lossy(params[1]).to_string();
                    self.window_title = Some(title.clone());
                    self.icon_name = Some(title);
                }
            }
            1 => {
                // OSC 1 ; name - Set icon name
                if params.len() > 1 {
                    self.icon_name = Some(String::from_utf8_lossy(params[1]).to_string());
                }
            }
            2 => {
                // OSC 2 ; title - Set window title
                if params.len() > 1 {
                    self.window_title = Some(String::from_utf8_lossy(params[1]).to_string());
                }
            }
            7 => {
                // OSC 7 ; file://hostname/path - Set current working directory
                if params.len() > 1 {
                    let cwd_uri = String::from_utf8_lossy(params[1]).to_string();
                    // Extract path from file:// URI
                    // Format: file://hostname/path or file:///path
                    if let Some(path) = self.parse_file_uri(&cwd_uri) {
                        self.cwd_from_shell = Some(path);
                    }
                }
            }
            52 => {
                // OSC 52 ; c ; <base64-data> - Clipboard operations
                // c = clipboard selection (usually 'c' for clipboard, 'p' for primary)
                // base64-data = clipboard content in base64
                if params.len() > 2 {
                    let _selection = String::from_utf8_lossy(params[1]);
                    let data = String::from_utf8_lossy(params[2]).to_string();

                    // Store the base64 data for external handling
                    // Applications can read this and decode it
                    if !data.is_empty() && data != "?" {
                        self.clipboard_request = Some(data);
                    }
                }
            }
            _ => {
                // Unknown OSC command - ignore
            }
        }
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], _ignore: bool, action: char) {
        // Check for private mode sequences (CSI ? ...)
        let is_private = intermediates.contains(&b'?');

        if is_private && action == 'h' {
            // Set private mode (DEC Private Mode Set - DECSET)
            for param in params.iter() {
                match param[0] {
                    1 => {
                        // DECCKM - Application Cursor Keys (CSI ?1h)
                        self.application_cursor_keys = true;
                    }
                    9 | 1000 => {
                        // X10 Mouse Reporting (CSI ?9h or CSI ?1000h)
                        self.mouse_mode = MouseMode::X10;
                    }
                    1002 => {
                        // Button-Event Mouse Tracking (CSI ?1002h)
                        self.mouse_mode = MouseMode::ButtonEvent;
                    }
                    1003 => {
                        // Any-Event Mouse Tracking (CSI ?1003h)
                        self.mouse_mode = MouseMode::AnyEvent;
                    }
                    25 => {
                        // DECTCEM - Show Cursor (CSI ?25h)
                        self.cursor_visible = true;
                    }
                    1006 => {
                        // SGR Extended Mouse Mode (CSI ?1006h)
                        self.mouse_encoding = MouseEncoding::Sgr;
                    }
                    47 => {
                        // CSI ?47h - Switch to alternate screen (basic mode)
                        // Used by some older applications
                        self.switch_to_alternate_screen(false);
                    }
                    1047 => {
                        // CSI ?1047h - Switch to alternate screen and clear it
                        // More common in modern terminals
                        self.switch_to_alternate_screen(true);
                    }
                    1049 => {
                        // CSI ?1049h - Save cursor, switch to alternate screen and clear
                        // Most complete mode - used by vim, less, htop, etc.
                        // Note: cursor is saved in the state structure
                        self.switch_to_alternate_screen(true);
                    }
                    _ => {
                        // Other private modes not implemented yet
                    }
                }
            }
            return;
        }

        if is_private && action == 'l' {
            // Reset private mode (DEC Private Mode Reset - DECRST)
            for param in params.iter() {
                match param[0] {
                    1 => {
                        // DECCKM - Normal Cursor Keys (CSI ?1l)
                        self.application_cursor_keys = false;
                    }
                    9 | 1000 | 1002 | 1003 => {
                        // Disable mouse reporting (CSI ?9l, ?1000l, ?1002l, ?1003l)
                        self.mouse_mode = MouseMode::None;
                    }
                    25 => {
                        // DECTCEM - Hide Cursor (CSI ?25l)
                        self.cursor_visible = false;
                    }
                    1006 => {
                        // Disable SGR Extended Mouse Mode (CSI ?1006l)
                        self.mouse_encoding = MouseEncoding::Default;
                    }
                    47 | 1047 | 1049 => {
                        // CSI ?47l, ?1047l, ?1049l - Switch back to normal screen
                        // All three modes restore the main screen completely
                        // The saved state includes cursor position and all attributes
                        self.switch_to_normal_screen();
                    }
                    _ => {
                        // Other private modes not implemented yet
                    }
                }
            }
            return;
        }

        // Standard CSI sequences
        match action {
            'A' => {
                // Cursor Up (CUU)
                let n = params.iter().next().map(|p| p[0] as usize).unwrap_or(1);
                let (top, _) = self.scroll_region.unwrap_or((0, self.rows - 1));
                self.cursor_row = max(top, self.cursor_row.saturating_sub(n));
            }
            'B' => {
                // Cursor Down (CUD)
                let n = params.iter().next().map(|p| p[0] as usize).unwrap_or(1);
                let (_, bottom) = self.scroll_region.unwrap_or((0, self.rows - 1));
                self.cursor_row = min(bottom, self.cursor_row + n);
            }
            'C' => {
                // Cursor Forward (CUF)
                let n = params.iter().next().map(|p| p[0] as usize).unwrap_or(1);
                self.cursor_col = min(self.cols - 1, self.cursor_col + n);
            }
            'D' => {
                // Cursor Back (CUB)
                let n = params.iter().next().map(|p| p[0] as usize).unwrap_or(1);
                self.cursor_col = self.cursor_col.saturating_sub(n);
            }
            'H' | 'f' => {
                // Cursor Position (CUP) or Horizontal Vertical Position (HVP)
                let mut iter = params.iter();
                let row = iter
                    .next()
                    .map(|p| p[0].saturating_sub(1) as usize)
                    .unwrap_or(0);
                let col = iter
                    .next()
                    .map(|p| p[0].saturating_sub(1) as usize)
                    .unwrap_or(0);
                self.cursor_row = min(row, self.rows - 1);
                self.cursor_col = min(col, self.cols - 1);
            }
            'J' => {
                // Erase in Display (ED)
                let mode = params.iter().next().map(|p| p[0]).unwrap_or(0);
                self.erase_in_display(mode);
            }
            'K' => {
                // Erase in Line (EL)
                let mode = params.iter().next().map(|p| p[0]).unwrap_or(0);
                self.erase_in_line(mode);
            }
            'm' => {
                // Select Graphic Rendition (SGR)
                self.set_sgr(params);
            }
            'r' => {
                // Set Scrolling Region (DECSTBM)
                let mut iter = params.iter();
                let top = iter
                    .next()
                    .map(|p| p[0].saturating_sub(1) as usize)
                    .unwrap_or(0);
                let bottom = iter
                    .next()
                    .map(|p| p[0].saturating_sub(1) as usize)
                    .unwrap_or(self.rows - 1);
                self.scroll_region = Some((min(top, self.rows - 1), min(bottom, self.rows - 1)));
            }
            's' => {
                // Save Cursor Position (SCOSC)
                self.saved_cursor = Some((self.cursor_row, self.cursor_col));
            }
            'u' => {
                // Restore Cursor Position (SCORC)
                if let Some((row, col)) = self.saved_cursor {
                    self.cursor_row = min(row, self.rows - 1);
                    self.cursor_col = min(col, self.cols - 1);
                }
            }
            'S' => {
                // Scroll Up (SU)
                let n = params.iter().next().map(|p| p[0] as usize).unwrap_or(1);
                self.scroll_up(n);
            }
            'T' => {
                // Scroll Down (SD)
                let n = params.iter().next().map(|p| p[0] as usize).unwrap_or(1);
                self.scroll_down(n);
            }
            'L' => {
                // Insert Lines (IL)
                let n = params
                    .iter()
                    .next()
                    .map(|p| max(1, p[0] as usize))
                    .unwrap_or(1);
                self.insert_lines(n);
            }
            'M' => {
                // Delete Lines (DL)
                let n = params
                    .iter()
                    .next()
                    .map(|p| max(1, p[0] as usize))
                    .unwrap_or(1);
                self.delete_lines(n);
            }
            '@' => {
                // Insert Characters (ICH)
                let n = params.iter().next().map(|p| p[0] as usize).unwrap_or(1);
                self.insert_characters(n);
            }
            'P' => {
                // Delete Characters (DCH)
                let n = params.iter().next().map(|p| p[0] as usize).unwrap_or(1);
                self.delete_characters(n);
            }
            'c' => {
                // Device Attributes (DA)
                if intermediates.contains(&b'>') {
                    // Secondary DA (DA2) - CSI > c
                    // Response: CSI > 0 ; 0 ; 0 c (VT100 compatible)
                    self.pending_responses.push("\x1b[>0;0;0c".to_string());
                } else {
                    // Primary DA (DA1) - CSI c
                    // Response: CSI ? 1 ; 2 c (VT100 with Advanced Video Option)
                    self.pending_responses.push("\x1b[?1;2c".to_string());
                }
            }
            'n' => {
                // Device Status Report (DSR)
                let n = params.iter().next().map(|p| p[0]).unwrap_or(0);
                match n {
                    5 => {
                        // DSR - Device Status Report
                        // Response: CSI 0 n (Terminal OK)
                        self.pending_responses.push("\x1b[0n".to_string());
                    }
                    6 => {
                        // CPR - Cursor Position Report
                        // Response: CSI <row> ; <col> R
                        // Note: VT100 uses 1-based indexing
                        let row = self.cursor_row + 1;
                        let col = self.cursor_col + 1;
                        self.pending_responses
                            .push(format!("\x1b[{};{}R", row, col));
                    }
                    _ => {
                        // Unknown DSR request - ignore
                    }
                }
            }
            _ => {
                // Unknown CSI sequence - ignore
            }
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, byte: u8) {
        match byte {
            b'7' => {
                // DECSC - Save Cursor (ESC 7)
                // Save cursor position and attributes
                self.saved_cursor_state = Some((
                    self.cursor_row,
                    self.cursor_col,
                    self.current_fg,
                    self.current_bg,
                    self.bold,
                ));
            }
            b'8' => {
                // DECRC - Restore Cursor (ESC 8)
                // Restore cursor position and attributes
                if let Some((row, col, fg, bg, bold)) = self.saved_cursor_state {
                    self.cursor_row = min(row, self.rows - 1);
                    self.cursor_col = min(col, self.cols - 1);
                    self.current_fg = fg;
                    self.current_bg = bg;
                    self.bold = bold;
                }
            }
            b'M' => {
                // RI - Reverse Index (ESC M)
                // Move cursor up one line, scroll down if at top of scroll region
                let (top, _) = self.scroll_region.unwrap_or((0, self.rows - 1));

                if self.cursor_row == top {
                    // At top of scroll region, scroll down
                    self.scroll_down(1);
                } else if self.cursor_row > 0 {
                    // Not at top, just move up
                    self.cursor_row -= 1;
                }
            }
            b'D' => {
                // IND - Index (ESC D)
                // Move cursor down one line, scroll up if at bottom of scroll region
                let (_, bottom) = self.scroll_region.unwrap_or((0, self.rows - 1));

                if self.cursor_row == bottom {
                    // At bottom of scroll region, scroll up
                    self.scroll_up(1);
                } else if self.cursor_row < self.rows - 1 {
                    // Not at bottom, just move down
                    self.cursor_row += 1;
                }
            }
            b'E' => {
                // NEL - Next Line (ESC E)
                // Move to first column of next line
                self.cursor_col = 0;
                let (_, bottom) = self.scroll_region.unwrap_or((0, self.rows - 1));

                if self.cursor_row == bottom {
                    // At bottom of scroll region, scroll up
                    self.scroll_up(1);
                } else if self.cursor_row < self.rows - 1 {
                    // Not at bottom, just move down
                    self.cursor_row += 1;
                }
            }
            b'c' => {
                // RIS - Reset to Initial State (ESC c)
                // Full terminal reset
                self.buffer = vec![vec![Cell::default(); self.cols]; self.rows];
                self.scrollback.clear();
                self.cursor_row = 0;
                self.cursor_col = 0;
                self.current_fg = None;
                self.current_bg = None;
                self.bold = false;
                self.underline = false;
                self.reverse = false;
                self.dim = false;
                self.italic = false;
                self.strikethrough = false;
                self.scroll_region = None;
                self.saved_cursor = None;
                self.saved_cursor_state = None;
                self.cursor_visible = true;
                self.auto_wrap_mode = true;
            }
            _ => {
                // Unknown ESC sequence - ignore
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mouse_mode_default() {
        let screen = TerminalScreen::new(80, 24);
        assert_eq!(screen.mouse_mode(), MouseMode::None);
        assert_eq!(screen.mouse_encoding(), MouseEncoding::Default);
        assert!(!screen.is_mouse_reporting_enabled());
    }

    #[test]
    fn test_mouse_mode_x10_enable() {
        let mut screen = TerminalScreen::new(80, 24);
        // CSI ?1000h - Enable X10 mouse reporting
        screen.process(b"\x1b[?1000h");
        assert_eq!(screen.mouse_mode(), MouseMode::X10);
        assert!(screen.is_mouse_reporting_enabled());
    }

    #[test]
    fn test_mouse_mode_x10_disable() {
        let mut screen = TerminalScreen::new(80, 24);
        // Enable then disable
        screen.process(b"\x1b[?1000h");
        assert_eq!(screen.mouse_mode(), MouseMode::X10);
        screen.process(b"\x1b[?1000l");
        assert_eq!(screen.mouse_mode(), MouseMode::None);
        assert!(!screen.is_mouse_reporting_enabled());
    }

    #[test]
    fn test_mouse_mode_button_event() {
        let mut screen = TerminalScreen::new(80, 24);
        // CSI ?1002h - Enable button-event tracking
        screen.process(b"\x1b[?1002h");
        assert_eq!(screen.mouse_mode(), MouseMode::ButtonEvent);
        assert!(screen.is_mouse_reporting_enabled());
    }

    #[test]
    fn test_mouse_mode_any_event() {
        let mut screen = TerminalScreen::new(80, 24);
        // CSI ?1003h - Enable any-event tracking
        screen.process(b"\x1b[?1003h");
        assert_eq!(screen.mouse_mode(), MouseMode::AnyEvent);
        assert!(screen.is_mouse_reporting_enabled());
    }

    #[test]
    fn test_mouse_encoding_sgr() {
        let mut screen = TerminalScreen::new(80, 24);
        // CSI ?1006h - Enable SGR extended mouse mode
        screen.process(b"\x1b[?1006h");
        assert_eq!(screen.mouse_encoding(), MouseEncoding::Sgr);
        // Disable
        screen.process(b"\x1b[?1006l");
        assert_eq!(screen.mouse_encoding(), MouseEncoding::Default);
    }

    #[test]
    fn test_mouse_mode_multiple_enables() {
        let mut screen = TerminalScreen::new(80, 24);
        // Enable X10
        screen.process(b"\x1b[?1000h");
        assert_eq!(screen.mouse_mode(), MouseMode::X10);
        // Upgrade to ButtonEvent
        screen.process(b"\x1b[?1002h");
        assert_eq!(screen.mouse_mode(), MouseMode::ButtonEvent);
        // Upgrade to AnyEvent
        screen.process(b"\x1b[?1003h");
        assert_eq!(screen.mouse_mode(), MouseMode::AnyEvent);
    }

    #[test]
    fn test_mouse_mode_with_sgr_encoding() {
        let mut screen = TerminalScreen::new(80, 24);
        // Enable X10 mouse + SGR encoding
        screen.process(b"\x1b[?1000h\x1b[?1006h");
        assert_eq!(screen.mouse_mode(), MouseMode::X10);
        assert_eq!(screen.mouse_encoding(), MouseEncoding::Sgr);
        assert!(screen.is_mouse_reporting_enabled());
    }

    #[test]
    fn test_mouse_mode_csi_9() {
        let mut screen = TerminalScreen::new(80, 24);
        // CSI ?9h - Alternative X10 mouse reporting
        screen.process(b"\x1b[?9h");
        assert_eq!(screen.mouse_mode(), MouseMode::X10);
        screen.process(b"\x1b[?9l");
        assert_eq!(screen.mouse_mode(), MouseMode::None);
    }

    #[test]
    fn test_cursor_visible_default() {
        let screen = TerminalScreen::new(80, 24);
        assert!(screen.cursor_visible());
    }

    #[test]
    fn test_cursor_hide() {
        let mut screen = TerminalScreen::new(80, 24);
        // CSI ?25l - Hide cursor (DECTCEM)
        screen.process(b"\x1b[?25l");
        assert!(!screen.cursor_visible());
    }

    #[test]
    fn test_cursor_show() {
        let mut screen = TerminalScreen::new(80, 24);
        // Hide first
        screen.process(b"\x1b[?25l");
        assert!(!screen.cursor_visible());
        // CSI ?25h - Show cursor (DECTCEM)
        screen.process(b"\x1b[?25h");
        assert!(screen.cursor_visible());
    }

    #[test]
    fn test_cursor_toggle() {
        let mut screen = TerminalScreen::new(80, 24);
        // Default is visible
        assert!(screen.cursor_visible());
        // Hide
        screen.process(b"\x1b[?25l");
        assert!(!screen.cursor_visible());
        // Show
        screen.process(b"\x1b[?25h");
        assert!(screen.cursor_visible());
        // Hide again
        screen.process(b"\x1b[?25l");
        assert!(!screen.cursor_visible());
    }

    #[test]
    fn test_esc_decsc_decrc() {
        let mut screen = TerminalScreen::new(80, 24);

        // Move cursor and set attributes
        screen.process(b"\x1b[10;20H"); // Move to (9, 19)
        screen.process(b"\x1b[1m"); // Set bold
        screen.process(b"\x1b[31m"); // Set red foreground
        screen.process(b"\x1b[42m"); // Set green background

        assert_eq!(screen.cursor_position(), (9, 19));
        assert!(screen.bold);
        assert_eq!(screen.current_fg, Some(AnsiColor::Indexed(1)));
        assert_eq!(screen.current_bg, Some(AnsiColor::Indexed(2)));

        // Save cursor (ESC 7)
        screen.process(b"\x1b7");

        // Move cursor and change attributes
        screen.process(b"\x1b[5;10H"); // Move to (4, 9)
        screen.process(b"\x1b[0m"); // Reset attributes

        assert_eq!(screen.cursor_position(), (4, 9));
        assert!(!screen.bold);
        assert_eq!(screen.current_fg, None);
        assert_eq!(screen.current_bg, None);

        // Restore cursor (ESC 8)
        screen.process(b"\x1b8");

        // Should be back to saved position and attributes
        assert_eq!(screen.cursor_position(), (9, 19));
        assert!(screen.bold);
        assert_eq!(screen.current_fg, Some(AnsiColor::Indexed(1)));
        assert_eq!(screen.current_bg, Some(AnsiColor::Indexed(2)));
    }

    #[test]
    fn test_esc_reverse_index() {
        let mut screen = TerminalScreen::new(10, 5);

        // Fill screen with text
        for i in 0..5 {
            screen.process(format!("Line {}\r\n", i).as_bytes());
        }

        // Move to middle line
        screen.process(b"\x1b[3;1H");
        assert_eq!(screen.cursor_position(), (2, 0));

        // Reverse index (ESC M) - should move up one line
        screen.process(b"\x1bM");
        assert_eq!(screen.cursor_position(), (1, 0));

        // Reverse index again
        screen.process(b"\x1bM");
        assert_eq!(screen.cursor_position(), (0, 0));

        // Reverse index at top - should scroll down
        screen.process(b"\x1bM");
        assert_eq!(screen.cursor_position(), (0, 0)); // Stay at top
                                                      // (scrolling behavior is tested by checking the buffer contents)
    }

    #[test]
    fn test_esc_index() {
        let mut screen = TerminalScreen::new(10, 5);

        // Fill screen with text
        for i in 0..5 {
            screen.process(format!("Line {}\r\n", i).as_bytes());
        }

        // Move to second line
        screen.process(b"\x1b[2;1H");
        assert_eq!(screen.cursor_position(), (1, 0));

        // Index (ESC D) - should move down one line
        screen.process(b"\x1bD");
        assert_eq!(screen.cursor_position(), (2, 0));

        // Move to bottom line
        screen.process(b"\x1b[5;1H");
        assert_eq!(screen.cursor_position(), (4, 0));

        // Index at bottom - should scroll up
        screen.process(b"\x1bD");
        assert_eq!(screen.cursor_position(), (4, 0)); // Stay at bottom
    }

    #[test]
    fn test_esc_next_line() {
        let mut screen = TerminalScreen::new(10, 5);

        // Move to position
        screen.process(b"\x1b[2;5H");
        assert_eq!(screen.cursor_position(), (1, 4));

        // Next line (ESC E) - should move to next line, column 0
        screen.process(b"\x1bE");
        assert_eq!(screen.cursor_position(), (2, 0));

        // Next line again
        screen.process(b"\x1bE");
        assert_eq!(screen.cursor_position(), (3, 0));

        // Move to bottom
        screen.process(b"\x1b[5;10H");
        assert_eq!(screen.cursor_position(), (4, 9));

        // Next line at bottom - should scroll and stay at column 0
        screen.process(b"\x1bE");
        assert_eq!(screen.cursor_position(), (4, 0));
    }

    #[test]
    fn test_esc_reset() {
        let mut screen = TerminalScreen::new(80, 24);

        // Set up screen with various state
        screen.process(b"Some text\r\n");
        screen.process(b"\x1b[10;20H"); // Move cursor
        screen.process(b"\x1b[1;4;31;42m"); // Bold, underline, red fg, green bg
        screen.process(b"\x1b[5;10r"); // Set scroll region
        screen.process(b"\x1b[s"); // Save cursor
        screen.process(b"\x1b[?25l"); // Hide cursor

        assert_eq!(screen.cursor_position(), (9, 19));
        assert!(screen.bold);
        assert!(screen.underline);
        assert!(!screen.cursor_visible);

        // Reset (ESC c)
        screen.process(b"\x1bc");

        // Everything should be reset
        assert_eq!(screen.cursor_position(), (0, 0));
        assert!(!screen.bold);
        assert!(!screen.underline);
        assert!(!screen.reverse);
        assert_eq!(screen.current_fg, None);
        assert_eq!(screen.current_bg, None);
        assert!(screen.cursor_visible);
        assert_eq!(screen.scroll_region, None);
        assert_eq!(screen.saved_cursor, None);
        assert_eq!(screen.saved_cursor_state, None);

        // Buffer should be cleared
        let buffer = &screen.buffer;
        for row in buffer.iter() {
            for cell in row.iter() {
                assert_eq!(cell.c, ' ');
            }
        }
    }

    #[test]
    fn test_esc_sequences_with_scroll_region() {
        let mut screen = TerminalScreen::new(10, 10);

        // Fill with text
        for i in 0..10 {
            screen.process(format!("Line {}\r\n", i).as_bytes());
        }

        // Set scroll region to lines 3-7 (indices 2-6)
        screen.process(b"\x1b[3;7r");

        // Move to top of scroll region
        screen.process(b"\x1b[3;1H");
        assert_eq!(screen.cursor_position(), (2, 0));

        // Reverse index at top of scroll region - should scroll down within region
        screen.process(b"\x1bM");
        assert_eq!(screen.cursor_position(), (2, 0));

        // Move to bottom of scroll region
        screen.process(b"\x1b[7;1H");
        assert_eq!(screen.cursor_position(), (6, 0));

        // Index at bottom of scroll region - should scroll up within region
        screen.process(b"\x1bD");
        assert_eq!(screen.cursor_position(), (6, 0));
    }

    #[test]
    fn test_insert_lines_basic() {
        let mut screen = TerminalScreen::new(10, 5);
        // Write lines of text
        screen.process(b"Line 0\r\nLine 1\r\nLine 2\r\nLine 3\r\nLine 4");

        // Move cursor to line 1 (second line)
        screen.process(b"\x1b[2;1H");
        assert_eq!(screen.cursor_position(), (1, 0));

        // Insert 1 line at cursor position (CSI L)
        screen.process(b"\x1b[L");

        // Line 1 should now be blank, and old Line 1 should be at Line 2
        let buffer = &screen.buffer;
        assert_eq!(buffer[0][0].c, 'L'); // Line 0 unchanged
        assert_eq!(buffer[1][0].c, ' '); // New blank line
        assert_eq!(buffer[2][0].c, 'L'); // Old Line 1
        assert_eq!(buffer[3][0].c, 'L'); // Old Line 2
        assert_eq!(buffer[4][0].c, 'L'); // Old Line 3
    }

    #[test]
    fn test_insert_lines_multiple() {
        let mut screen = TerminalScreen::new(10, 5);
        // Write lines of text
        screen.process(b"Line 0\r\nLine 1\r\nLine 2\r\nLine 3\r\nLine 4");

        // Move cursor to line 1
        screen.process(b"\x1b[2;1H");

        // Insert 2 lines (CSI 2L)
        screen.process(b"\x1b[2L");

        let buffer = &screen.buffer;
        assert_eq!(buffer[0][0].c, 'L'); // Line 0 unchanged
        assert_eq!(buffer[1][0].c, ' '); // New blank line
        assert_eq!(buffer[2][0].c, ' '); // New blank line
        assert_eq!(buffer[3][0].c, 'L'); // Old Line 1
        assert_eq!(buffer[4][0].c, 'L'); // Old Line 2
    }

    #[test]
    fn test_delete_lines_basic() {
        let mut screen = TerminalScreen::new(10, 5);
        // Write lines of text
        screen.process(b"Line 0\r\nLine 1\r\nLine 2\r\nLine 3\r\nLine 4");

        // Move cursor to line 1 (second line)
        screen.process(b"\x1b[2;1H");
        assert_eq!(screen.cursor_position(), (1, 0));

        // Delete 1 line at cursor position (CSI M)
        screen.process(b"\x1b[M");

        // Line 1 should be deleted, and lines below should move up
        let buffer = &screen.buffer;
        assert_eq!(buffer[0][0].c, 'L'); // Line 0 unchanged
        assert_eq!(buffer[1][0].c, 'L'); // Old Line 2 moved up
        assert_eq!(buffer[1][5].c, '2');
        assert_eq!(buffer[2][0].c, 'L'); // Old Line 3
        assert_eq!(buffer[2][5].c, '3');
        assert_eq!(buffer[3][0].c, 'L'); // Old Line 4
        assert_eq!(buffer[3][5].c, '4');
        assert_eq!(buffer[4][0].c, ' '); // Last line is now blank
    }

    #[test]
    fn test_delete_lines_multiple() {
        let mut screen = TerminalScreen::new(10, 5);
        // Write lines of text
        screen.process(b"Line 0\r\nLine 1\r\nLine 2\r\nLine 3\r\nLine 4");

        // Move cursor to line 1
        screen.process(b"\x1b[2;1H");

        // Delete 2 lines (CSI 2M)
        screen.process(b"\x1b[2M");

        let buffer = &screen.buffer;
        assert_eq!(buffer[0][0].c, 'L'); // Line 0 unchanged
        assert_eq!(buffer[1][0].c, 'L'); // Old Line 3 moved up
        assert_eq!(buffer[1][5].c, '3');
        assert_eq!(buffer[2][0].c, 'L'); // Old Line 4
        assert_eq!(buffer[2][5].c, '4');
        assert_eq!(buffer[3][0].c, ' '); // Blank line
        assert_eq!(buffer[4][0].c, ' '); // Blank line
    }
}

#[test]
fn test_application_cursor_keys_default() {
    let screen = TerminalScreen::new(80, 24);
    assert!(!screen.application_cursor_keys());
}

#[test]
fn test_application_cursor_keys_enable() {
    let mut screen = TerminalScreen::new(80, 24);
    // CSI ?1h - Enable application cursor keys (DECCKM)
    screen.process(b"\x1b[?1h");
    assert!(screen.application_cursor_keys());
}

#[test]
fn test_application_cursor_keys_disable() {
    let mut screen = TerminalScreen::new(80, 24);
    // Enable first
    screen.process(b"\x1b[?1h");
    assert!(screen.application_cursor_keys());
    // CSI ?1l - Disable application cursor keys
    screen.process(b"\x1b[?1l");
    assert!(!screen.application_cursor_keys());
}

#[test]
fn test_application_cursor_keys_toggle() {
    let mut screen = TerminalScreen::new(80, 24);
    // Default is false (normal mode)
    assert!(!screen.application_cursor_keys());
    // Enable
    screen.process(b"\x1b[?1h");
    assert!(screen.application_cursor_keys());
    // Disable
    screen.process(b"\x1b[?1l");
    assert!(!screen.application_cursor_keys());
    // Enable again
    screen.process(b"\x1b[?1h");
    assert!(screen.application_cursor_keys());
}

#[test]
fn test_wide_character_korean() {
    let mut screen = TerminalScreen::new(10, 3);

    // Write Korean character (한) which is a wide character
    screen.process("한".as_bytes());

    let buffer = &screen.buffer;
    // First cell should contain the character and be marked as wide
    assert_eq!(buffer[0][0].c, '한');
    assert!(buffer[0][0].wide);
    assert!(!buffer[0][0].placeholder);

    // Second cell should be a placeholder
    assert_eq!(buffer[0][1].c, ' ');
    assert!(!buffer[0][1].wide);
    assert!(buffer[0][1].placeholder);

    // Cursor should have advanced by 2
    assert_eq!(screen.cursor_position(), (0, 2));
}

#[test]
fn test_wide_character_chinese() {
    let mut screen = TerminalScreen::new(10, 3);

    // Write Chinese character (中) which is a wide character
    screen.process("中".as_bytes());

    let buffer = &screen.buffer;
    assert_eq!(buffer[0][0].c, '中');
    assert!(buffer[0][0].wide);
    assert!(buffer[0][1].placeholder);
    assert_eq!(screen.cursor_position(), (0, 2));
}

#[test]
fn test_wide_character_japanese() {
    let mut screen = TerminalScreen::new(10, 3);

    // Write Japanese character (日) which is a wide character
    screen.process("日".as_bytes());

    let buffer = &screen.buffer;
    assert_eq!(buffer[0][0].c, '日');
    assert!(buffer[0][0].wide);
    assert!(buffer[0][1].placeholder);
    assert_eq!(screen.cursor_position(), (0, 2));
}

#[test]
fn test_mixed_ascii_and_wide() {
    let mut screen = TerminalScreen::new(10, 3);

    // Write mixed ASCII and Korean: "A한B"
    screen.process("A한B".as_bytes());

    let buffer = &screen.buffer;
    // A at position 0
    assert_eq!(buffer[0][0].c, 'A');
    assert!(!buffer[0][0].wide);
    assert!(!buffer[0][0].placeholder);

    // 한 at positions 1-2
    assert_eq!(buffer[0][1].c, '한');
    assert!(buffer[0][1].wide);
    assert!(!buffer[0][1].placeholder);
    assert!(buffer[0][2].placeholder);

    // B at position 3
    assert_eq!(buffer[0][3].c, 'B');
    assert!(!buffer[0][3].wide);
    assert!(!buffer[0][3].placeholder);

    assert_eq!(screen.cursor_position(), (0, 4));
}

#[test]
fn test_wide_character_with_colors() {
    let mut screen = TerminalScreen::new(10, 3);

    // Set red foreground and write Korean character
    screen.process(b"\x1b[31m");
    screen.process("한".as_bytes());

    let buffer = &screen.buffer;
    // Check both cells have the same foreground color
    assert_eq!(buffer[0][0].fg, Some(AnsiColor::Indexed(1)));
    assert_eq!(buffer[0][1].fg, Some(AnsiColor::Indexed(1)));
    assert!(buffer[0][0].wide);
    assert!(buffer[0][1].placeholder);
}

#[test]
fn test_wide_character_wrapping() {
    let mut screen = TerminalScreen::new(5, 3);

    // Write "ABC한" - the wide character should fit on the first line
    // at positions 3-4 (exactly 2 columns available)
    screen.process("ABC한".as_bytes());

    let buffer = &screen.buffer;
    // First line: "ABC한" (한 occupies positions 3-4)
    assert_eq!(buffer[0][0].c, 'A');
    assert_eq!(buffer[0][1].c, 'B');
    assert_eq!(buffer[0][2].c, 'C');
    assert_eq!(buffer[0][3].c, '한');
    assert!(buffer[0][3].wide);
    assert!(buffer[0][4].placeholder);

    // Cursor should be at the end of the first line
    assert_eq!(screen.cursor_position(), (0, 5));

    // Now test actual wrapping: write "ABCD한" where 한 should wrap
    let mut screen2 = TerminalScreen::new(5, 3);
    screen2.process("ABCD한".as_bytes());

    let buffer2 = &screen2.buffer;
    // First line: "ABCD"
    assert_eq!(buffer2[0][0].c, 'A');
    assert_eq!(buffer2[0][1].c, 'B');
    assert_eq!(buffer2[0][2].c, 'C');
    assert_eq!(buffer2[0][3].c, 'D');

    // Wide character should wrap to next line (only 1 column left)
    assert_eq!(buffer2[1][0].c, '한');
    assert!(buffer2[1][0].wide);
    assert!(buffer2[1][1].placeholder);

    assert_eq!(screen2.cursor_position(), (1, 2));
}

#[test]
fn test_wide_character_at_edge() {
    let mut screen = TerminalScreen::new(5, 3);

    // Move to position 4 (last column)
    screen.process(b"\x1b[1;5H");
    assert_eq!(screen.cursor_position(), (0, 4));

    // Try to write wide character - should wrap to next line
    screen.process("한".as_bytes());

    let buffer = &screen.buffer;
    // Wide character should be on the next line
    assert_eq!(buffer[1][0].c, '한');
    assert!(buffer[1][0].wide);
    assert!(buffer[1][1].placeholder);

    assert_eq!(screen.cursor_position(), (1, 2));
}

#[test]
fn test_overwrite_wide_character() {
    let mut screen = TerminalScreen::new(10, 3);

    // Write wide character
    screen.process("한".as_bytes());
    assert!(screen.buffer[0][0].wide);
    assert!(screen.buffer[0][1].placeholder);

    // Move cursor back to position 0
    screen.process(b"\x1b[1;1H");

    // Overwrite with regular character
    screen.process(b"X");

    let buffer = &screen.buffer;
    // X should be at position 0
    assert_eq!(buffer[0][0].c, 'X');
    assert!(!buffer[0][0].wide);

    // Placeholder should be cleared
    assert_eq!(buffer[0][1].c, ' ');
    assert!(!buffer[0][1].wide);
    assert!(!buffer[0][1].placeholder);
}

#[test]
fn test_overwrite_wide_character_placeholder() {
    let mut screen = TerminalScreen::new(10, 3);

    // Write wide character
    screen.process("한".as_bytes());
    assert!(screen.buffer[0][0].wide);
    assert!(screen.buffer[0][1].placeholder);

    // Move cursor to placeholder position
    screen.process(b"\x1b[1;2H");

    // Overwrite placeholder with regular character
    screen.process(b"X");

    let buffer = &screen.buffer;
    // Wide character should be cleared
    assert_eq!(buffer[0][0].c, ' ');
    assert!(!buffer[0][0].wide);

    // X should be at position 1
    assert_eq!(buffer[0][1].c, 'X');
    assert!(!buffer[0][1].wide);
    assert!(!buffer[0][1].placeholder);
}

#[test]
fn test_multiple_wide_characters() {
    let mut screen = TerminalScreen::new(10, 3);

    // Write multiple Korean characters: "한글"
    screen.process("한글".as_bytes());

    let buffer = &screen.buffer;
    // First character at positions 0-1
    assert_eq!(buffer[0][0].c, '한');
    assert!(buffer[0][0].wide);
    assert!(buffer[0][1].placeholder);

    // Second character at positions 2-3
    assert_eq!(buffer[0][2].c, '글');
    assert!(buffer[0][2].wide);
    assert!(buffer[0][3].placeholder);

    assert_eq!(screen.cursor_position(), (0, 4));
}

#[test]
fn test_wide_character_with_formatting() {
    let mut screen = TerminalScreen::new(10, 3);

    // Write bold, underlined Korean character
    screen.process(b"\x1b[1;4m");
    screen.process("한".as_bytes());

    let buffer = &screen.buffer;
    // Both cells should have the same formatting
    assert!(buffer[0][0].bold);
    assert!(buffer[0][0].underline);
    assert!(buffer[0][1].bold);
    assert!(buffer[0][1].underline);
    assert!(buffer[0][0].wide);
    assert!(buffer[0][1].placeholder);
}

#[test]
fn test_emoji_wide_character() {
    let mut screen = TerminalScreen::new(10, 3);

    // Some emojis are wide characters
    // Note: emoji width depends on unicode-width crate's classification
    screen.process("👋".as_bytes());

    // Check that cursor moved (exact behavior depends on emoji width)
    let pos = screen.cursor_position();
    assert!(pos.1 > 0);
}

#[test]
fn test_application_cursor_keys_mode_sequences() {
    let mut screen = TerminalScreen::new(80, 24);

    // Test that mode changes are correctly applied via escape sequences
    // Normal mode (default)
    assert!(!screen.application_cursor_keys());

    // Enable application mode (like vim does)
    screen.process(b"\x1b[?1h");
    assert!(screen.application_cursor_keys());

    // Disable application mode (back to normal)
    screen.process(b"\x1b[?1l");
    assert!(!screen.application_cursor_keys());

    // Test multiple mode changes
    for _ in 0..3 {
        screen.process(b"\x1b[?1h");
        assert!(screen.application_cursor_keys());
        screen.process(b"\x1b[?1l");
        assert!(!screen.application_cursor_keys());
    }
}

#[test]
fn test_device_attributes_primary_da1() {
    let mut screen = TerminalScreen::new(80, 24);

    // Send Primary DA request (CSI c)
    screen.process(b"\x1b[c");

    // Should have one pending response
    let responses = screen.take_pending_responses();
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0], "\x1b[?1;2c");
}

#[test]
fn test_device_attributes_secondary_da2() {
    let mut screen = TerminalScreen::new(80, 24);

    // Send Secondary DA request (CSI > c)
    screen.process(b"\x1b[>c");

    // Should have one pending response
    let responses = screen.take_pending_responses();
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0], "\x1b[>0;0;0c");
}

#[test]
fn test_device_status_report_dsr() {
    let mut screen = TerminalScreen::new(80, 24);

    // Send DSR request (CSI 5 n)
    screen.process(b"\x1b[5n");

    // Should have one pending response
    let responses = screen.take_pending_responses();
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0], "\x1b[0n");
}

#[test]
fn test_cursor_position_report_cpr() {
    let mut screen = TerminalScreen::new(80, 24);

    // Move cursor to position (5, 10)
    screen.process(b"\x1b[6;11H");
    assert_eq!(screen.cursor_position(), (5, 10));

    // Send CPR request (CSI 6 n)
    screen.process(b"\x1b[6n");

    // Should have one pending response with 1-based indexing
    let responses = screen.take_pending_responses();
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0], "\x1b[6;11R");
}

#[test]
fn test_cpr_at_origin() {
    let mut screen = TerminalScreen::new(80, 24);

    // Cursor should be at origin (0, 0)
    assert_eq!(screen.cursor_position(), (0, 0));

    // Send CPR request
    screen.process(b"\x1b[6n");

    // Response should be (1, 1) in 1-based indexing
    let responses = screen.take_pending_responses();
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0], "\x1b[1;1R");
}

#[test]
fn test_multiple_device_attribute_requests() {
    let mut screen = TerminalScreen::new(80, 24);

    // Send multiple requests
    screen.process(b"\x1b[c"); // Primary DA
    screen.process(b"\x1b[>c"); // Secondary DA
    screen.process(b"\x1b[5n"); // DSR
    screen.process(b"\x1b[6n"); // CPR

    // Should have four pending responses
    let responses = screen.take_pending_responses();
    assert_eq!(responses.len(), 4);
    assert_eq!(responses[0], "\x1b[?1;2c");
    assert_eq!(responses[1], "\x1b[>0;0;0c");
    assert_eq!(responses[2], "\x1b[0n");
    assert_eq!(responses[3], "\x1b[1;1R");
}

#[test]
fn test_take_pending_responses_clears_queue() {
    let mut screen = TerminalScreen::new(80, 24);

    // Send DA request
    screen.process(b"\x1b[c");

    // Take responses
    let responses = screen.take_pending_responses();
    assert_eq!(responses.len(), 1);

    // Queue should be empty now
    let responses2 = screen.take_pending_responses();
    assert_eq!(responses2.len(), 0);
}

#[test]
fn test_device_attributes_with_regular_content() {
    let mut screen = TerminalScreen::new(80, 24);

    // Write some text
    screen.process(b"Hello, World!");

    // Send DA request
    screen.process(b"\x1b[c");

    // Should have response and text should be preserved
    let responses = screen.take_pending_responses();
    assert_eq!(responses.len(), 1);

    // Check that text is still there
    let buffer = &screen.buffer;
    assert_eq!(buffer[0][0].c, 'H');
    assert_eq!(buffer[0][1].c, 'e');
}

#[test]
fn test_sgr_dim() {
    let mut screen = TerminalScreen::new(80, 24);

    // Enable dim
    screen.process(b"\x1b[2m");
    assert!(screen.dim);

    // Write text with dim
    screen.process(b"dim text");
    assert!(screen.buffer[0][0].dim);

    // Reset
    screen.process(b"\x1b[0m");
    assert!(!screen.dim);
}

#[test]
fn test_sgr_italic() {
    let mut screen = TerminalScreen::new(80, 24);

    // Enable italic
    screen.process(b"\x1b[3m");
    assert!(screen.italic);

    // Write text with italic
    screen.process(b"italic text");
    assert!(screen.buffer[0][0].italic);

    // Disable italic (SGR 23)
    screen.process(b"\x1b[23m");
    assert!(!screen.italic);
}

#[test]
fn test_sgr_strikethrough() {
    let mut screen = TerminalScreen::new(80, 24);

    // Enable strikethrough
    screen.process(b"\x1b[9m");
    assert!(screen.strikethrough);

    // Write text with strikethrough
    screen.process(b"strike");
    assert!(screen.buffer[0][0].strikethrough);

    // Disable strikethrough (SGR 29)
    screen.process(b"\x1b[29m");
    assert!(!screen.strikethrough);
}

#[test]
fn test_sgr_normal_intensity() {
    let mut screen = TerminalScreen::new(80, 24);

    // Enable bold
    screen.process(b"\x1b[1m");
    assert!(screen.bold);
    assert!(!screen.dim);

    // Enable dim (should not affect bold in this test)
    screen.process(b"\x1b[2m");
    assert!(screen.dim);

    // SGR 22 - Normal intensity (disables both bold and dim)
    screen.process(b"\x1b[22m");
    assert!(!screen.bold);
    assert!(!screen.dim);
}

#[test]
fn test_sgr_combined_attributes() {
    let mut screen = TerminalScreen::new(80, 24);

    // Enable multiple attributes
    screen.process(b"\x1b[1;3;4;9m");
    assert!(screen.bold);
    assert!(screen.italic);
    assert!(screen.underline);
    assert!(screen.strikethrough);

    // Write text with combined attributes
    screen.process(b"styled");
    let cell = &screen.buffer[0][0];
    assert!(cell.bold);
    assert!(cell.italic);
    assert!(cell.underline);
    assert!(cell.strikethrough);

    // Reset all
    screen.process(b"\x1b[0m");
    assert!(!screen.bold);
    assert!(!screen.italic);
    assert!(!screen.underline);
    assert!(!screen.strikethrough);
}

#[test]
fn test_sgr_dim_and_bold() {
    let mut screen = TerminalScreen::new(80, 24);

    // Enable bold
    screen.process(b"\x1b[1mBold");
    assert!(screen.bold);
    assert!(!screen.dim);

    // Enable dim (both can be active, though visually may conflict)
    screen.process(b"\x1b[2mDim");
    assert!(screen.bold);
    assert!(screen.dim);

    // SGR 22 should reset both
    screen.process(b"\x1b[22mNormal");
    assert!(!screen.bold);
    assert!(!screen.dim);
}

#[test]
fn test_sgr_reset_specific_attributes() {
    let mut screen = TerminalScreen::new(80, 24);

    // Enable all new attributes
    screen.process(b"\x1b[2;3;9m");
    assert!(screen.dim);
    assert!(screen.italic);
    assert!(screen.strikethrough);

    // Reset italic (SGR 23)
    screen.process(b"\x1b[23m");
    assert!(screen.dim);
    assert!(!screen.italic);
    assert!(screen.strikethrough);

    // Reset dim (SGR 22)
    screen.process(b"\x1b[22m");
    assert!(!screen.dim);
    assert!(!screen.italic);
    assert!(screen.strikethrough);

    // Reset strikethrough (SGR 29)
    screen.process(b"\x1b[29m");
    assert!(!screen.dim);
    assert!(!screen.italic);
    assert!(!screen.strikethrough);
}

#[test]
fn test_sgr_256_color_foreground() {
    let mut screen = TerminalScreen::new(80, 24);

    // Test 256-color foreground (SGR 38;5;N)
    // Standard color (0-15)
    screen.process(b"\x1b[38;5;9mRed");
    assert_eq!(screen.current_fg, Some(AnsiColor::Palette256(9)));
    let cell = &screen.buffer[0][0];
    assert_eq!(cell.c, 'R');
    assert_eq!(cell.fg, Some(AnsiColor::Palette256(9)));

    // 6x6x6 color cube (16-231)
    screen.process(b"\x1b[38;5;196mBright");
    assert_eq!(screen.current_fg, Some(AnsiColor::Palette256(196)));

    // Grayscale (232-255)
    screen.process(b"\x1b[38;5;244mGray");
    assert_eq!(screen.current_fg, Some(AnsiColor::Palette256(244)));

    // Reset to default
    screen.process(b"\x1b[39m");
    assert_eq!(screen.current_fg, None);
}

#[test]
fn test_sgr_256_color_background() {
    let mut screen = TerminalScreen::new(80, 24);

    // Test 256-color background (SGR 48;5;N)
    // Standard color
    screen.process(b"\x1b[48;5;12mBlue");
    assert_eq!(screen.current_bg, Some(AnsiColor::Palette256(12)));
    let cell = &screen.buffer[0][0];
    assert_eq!(cell.c, 'B');
    assert_eq!(cell.bg, Some(AnsiColor::Palette256(12)));

    // 6x6x6 color cube
    screen.process(b"\x1b[48;5;46mGreen");
    assert_eq!(screen.current_bg, Some(AnsiColor::Palette256(46)));

    // Grayscale
    screen.process(b"\x1b[48;5;240mDark");
    assert_eq!(screen.current_bg, Some(AnsiColor::Palette256(240)));

    // Reset to default
    screen.process(b"\x1b[49m");
    assert_eq!(screen.current_bg, None);
}

#[test]
fn test_sgr_truecolor_foreground() {
    let mut screen = TerminalScreen::new(80, 24);

    // Test TrueColor foreground (SGR 38;2;R;G;B)
    screen.process(b"\x1b[38;2;255;128;64mOrange");
    assert_eq!(screen.current_fg, Some(AnsiColor::Rgb(255, 128, 64)));
    let cell = &screen.buffer[0][0];
    assert_eq!(cell.c, 'O');
    assert_eq!(cell.fg, Some(AnsiColor::Rgb(255, 128, 64)));

    // Test pure colors
    screen.process(b"\x1b[38;2;255;0;0mRed");
    assert_eq!(screen.current_fg, Some(AnsiColor::Rgb(255, 0, 0)));

    screen.process(b"\x1b[38;2;0;255;0mGreen");
    assert_eq!(screen.current_fg, Some(AnsiColor::Rgb(0, 255, 0)));

    screen.process(b"\x1b[38;2;0;0;255mBlue");
    assert_eq!(screen.current_fg, Some(AnsiColor::Rgb(0, 0, 255)));

    // Test black and white
    screen.process(b"\x1b[38;2;0;0;0mBlack");
    assert_eq!(screen.current_fg, Some(AnsiColor::Rgb(0, 0, 0)));

    screen.process(b"\x1b[38;2;255;255;255mWhite");
    assert_eq!(screen.current_fg, Some(AnsiColor::Rgb(255, 255, 255)));

    // Reset to default
    screen.process(b"\x1b[39m");
    assert_eq!(screen.current_fg, None);
}

#[test]
fn test_sgr_truecolor_background() {
    let mut screen = TerminalScreen::new(80, 24);

    // Test TrueColor background (SGR 48;2;R;G;B)
    screen.process(b"\x1b[48;2;100;200;150mCyan");
    assert_eq!(screen.current_bg, Some(AnsiColor::Rgb(100, 200, 150)));
    let cell = &screen.buffer[0][0];
    assert_eq!(cell.c, 'C');
    assert_eq!(cell.bg, Some(AnsiColor::Rgb(100, 200, 150)));

    // Test various colors
    screen.process(b"\x1b[48;2;64;128;192mBlue");
    assert_eq!(screen.current_bg, Some(AnsiColor::Rgb(64, 128, 192)));

    screen.process(b"\x1b[48;2;255;255;0mYellow");
    assert_eq!(screen.current_bg, Some(AnsiColor::Rgb(255, 255, 0)));

    // Reset to default
    screen.process(b"\x1b[49m");
    assert_eq!(screen.current_bg, None);
}

#[test]
fn test_sgr_mixed_color_modes() {
    let mut screen = TerminalScreen::new(80, 24);

    // Mix 16-color, 256-color, and TrueColor
    // 16-color foreground + 256-color background
    screen.process(b"\x1b[31;48;5;220mTest1");
    assert_eq!(screen.current_fg, Some(AnsiColor::Indexed(1)));
    assert_eq!(screen.current_bg, Some(AnsiColor::Palette256(220)));

    // TrueColor foreground + 16-color background
    screen.process(b"\x1b[38;2;128;64;200;44mTest2");
    assert_eq!(screen.current_fg, Some(AnsiColor::Rgb(128, 64, 200)));
    assert_eq!(screen.current_bg, Some(AnsiColor::Indexed(4)));

    // 256-color foreground + TrueColor background
    screen.process(b"\x1b[38;5;100;48;2;50;100;150mTest3");
    assert_eq!(screen.current_fg, Some(AnsiColor::Palette256(100)));
    assert_eq!(screen.current_bg, Some(AnsiColor::Rgb(50, 100, 150)));

    // Reset all
    screen.process(b"\x1b[0m");
    assert_eq!(screen.current_fg, None);
    assert_eq!(screen.current_bg, None);
}

#[test]
fn test_sgr_color_with_attributes() {
    let mut screen = TerminalScreen::new(80, 24);

    // Combine TrueColor with text attributes
    screen.process(b"\x1b[1;3;4;38;2;255;100;50mBold Italic Underline Orange");
    assert!(screen.bold);
    assert!(screen.italic);
    assert!(screen.underline);
    assert_eq!(screen.current_fg, Some(AnsiColor::Rgb(255, 100, 50)));

    let cell = &screen.buffer[0][0];
    assert_eq!(cell.c, 'B');
    assert!(cell.bold);
    assert!(cell.italic);
    assert!(cell.underline);
    assert_eq!(cell.fg, Some(AnsiColor::Rgb(255, 100, 50)));

    // Combine 256-color with attributes
    screen.process(b"\x1b[2;9;48;5;200mDim Strikethrough");
    assert!(screen.dim);
    assert!(screen.strikethrough);
    assert_eq!(screen.current_bg, Some(AnsiColor::Palette256(200)));
}

#[test]
fn test_ansi_color_to_iced_color() {
    // Test Indexed color conversion
    let indexed = AnsiColor::Indexed(1); // Red
    let color = indexed.to_color();
    assert_eq!(color, Color::from_rgb(0.8, 0.2, 0.2));

    // Test 256-color palette conversion (standard color)
    let palette_std = AnsiColor::Palette256(9); // Bright Red
    let color_std = palette_std.to_color();
    assert_eq!(color_std, Color::from_rgb(1.0, 0.3, 0.3));

    // Test 256-color palette conversion (color cube)
    let palette_cube = AnsiColor::Palette256(16); // First color in 6x6x6 cube
    let color_cube = palette_cube.to_color();
    assert_eq!(color_cube, Color::from_rgb(0.0, 0.0, 0.0));

    let palette_cube2 = AnsiColor::Palette256(231); // Last color in 6x6x6 cube
    let color_cube2 = palette_cube2.to_color();
    assert_eq!(color_cube2, Color::from_rgb(1.0, 1.0, 1.0));

    // Test 256-color palette conversion (grayscale)
    let palette_gray = AnsiColor::Palette256(232); // First grayscale
    let color_gray = palette_gray.to_color();
    let expected_gray = 8.0 / 255.0;
    assert_eq!(
        color_gray,
        Color::from_rgb(expected_gray, expected_gray, expected_gray)
    );

    let palette_gray_mid = AnsiColor::Palette256(244); // Mid grayscale
    let color_gray_mid = palette_gray_mid.to_color();
    let expected_gray_mid = (8.0 + 12.0 * 10.0) / 255.0;
    assert_eq!(
        color_gray_mid,
        Color::from_rgb(expected_gray_mid, expected_gray_mid, expected_gray_mid)
    );

    // Test RGB color conversion
    let rgb = AnsiColor::Rgb(255, 128, 64);
    let color_rgb = rgb.to_color();
    assert_eq!(color_rgb, Color::from_rgb(1.0, 128.0 / 255.0, 64.0 / 255.0));

    let rgb_black = AnsiColor::Rgb(0, 0, 0);
    let color_black = rgb_black.to_color();
    assert_eq!(color_black, Color::from_rgb(0.0, 0.0, 0.0));

    let rgb_white = AnsiColor::Rgb(255, 255, 255);
    let color_white = rgb_white.to_color();
    assert_eq!(color_white, Color::from_rgb(1.0, 1.0, 1.0));
}

#[test]
fn test_256_color_palette_ranges() {
    let mut screen = TerminalScreen::new(80, 24);

    // Test all standard colors (0-15)
    for i in 0..=15 {
        screen.process(format!("\x1b[38;5;{}m.", i).as_bytes());
        assert_eq!(screen.current_fg, Some(AnsiColor::Palette256(i)));
    }

    // Test color cube boundaries (16-231)
    screen.process(b"\x1b[38;5;16m."); // First cube color
    assert_eq!(screen.current_fg, Some(AnsiColor::Palette256(16)));

    screen.process(b"\x1b[38;5;231m."); // Last cube color
    assert_eq!(screen.current_fg, Some(AnsiColor::Palette256(231)));

    // Test grayscale boundaries (232-255)
    screen.process(b"\x1b[38;5;232m."); // First grayscale
    assert_eq!(screen.current_fg, Some(AnsiColor::Palette256(232)));

    screen.process(b"\x1b[38;5;255m."); // Last grayscale
    assert_eq!(screen.current_fg, Some(AnsiColor::Palette256(255)));
}

#[test]
fn test_color_cube_calculation() {
    // Test the 6x6x6 color cube calculation
    // Color 16 should be (0,0,0) in the cube = RGB(0,0,0)
    let color_16 = palette256_to_color(16);
    assert_eq!(color_16, Color::from_rgb(0.0, 0.0, 0.0));

    // Color 21 should be (0,0,5) in the cube = RGB(0,0,255)
    let color_21 = palette256_to_color(21);
    assert_eq!(color_21, Color::from_rgb(0.0, 0.0, 255.0 / 255.0));

    // Color 226 should be (5,5,0) in the cube = RGB(255,255,0) - yellow
    let color_226 = palette256_to_color(226);
    assert_eq!(
        color_226,
        Color::from_rgb(255.0 / 255.0, 255.0 / 255.0, 0.0)
    );

    // Color 231 should be (5,5,5) in the cube = RGB(255,255,255)
    let color_231 = palette256_to_color(231);
    assert_eq!(color_231, Color::from_rgb(1.0, 1.0, 1.0));
}
#[cfg(test)]
mod resize_tests {
    use super::*;

    #[test]
    fn test_resize_no_change() {
        let mut screen = TerminalScreen::new(80, 24);
        screen.process(b"Hello, World!");

        // Resize to same dimensions should be a no-op
        screen.resize(80, 24);

        assert_eq!(screen.buffer.len(), 24);
        assert_eq!(screen.buffer[0].len(), 80);
        assert_eq!(screen.buffer[0][0].c, 'H');
    }

    #[test]
    fn test_resize_columns_increase() {
        let mut screen = TerminalScreen::new(10, 5);
        screen.process(b"Hello");

        // Increase columns from 10 to 20
        screen.resize(20, 5);

        assert_eq!(screen.buffer.len(), 5);
        assert_eq!(screen.buffer[0].len(), 20);
        // Original content should be preserved
        assert_eq!(screen.buffer[0][0].c, 'H');
        assert_eq!(screen.buffer[0][1].c, 'e');
        assert_eq!(screen.buffer[0][2].c, 'l');
        assert_eq!(screen.buffer[0][3].c, 'l');
        assert_eq!(screen.buffer[0][4].c, 'o');
        // New cells should be empty
        assert_eq!(screen.buffer[0][10].c, ' ');
        assert_eq!(screen.buffer[0][19].c, ' ');
    }

    #[test]
    fn test_resize_columns_decrease() {
        let mut screen = TerminalScreen::new(20, 5);
        screen.process(b"Hello, World!");

        // Decrease columns from 20 to 10
        screen.resize(10, 5);

        assert_eq!(screen.buffer.len(), 5);
        assert_eq!(screen.buffer[0].len(), 10);
        // Content should be truncated
        assert_eq!(screen.buffer[0][0].c, 'H');
        assert_eq!(screen.buffer[0][1].c, 'e');
        assert_eq!(screen.buffer[0][9].c, 'r');
    }

    #[test]
    fn test_resize_rows_decrease() {
        let mut screen = TerminalScreen::new(10, 5);
        // Fill with text
        for i in 0..5 {
            screen.process(format!("Line {}\r\n", i).as_bytes());
        }

        let scrollback_before = screen.scrollback.len();

        // Decrease rows from 5 to 3
        screen.resize(10, 3);

        assert_eq!(screen.buffer.len(), 3);
        // Top 2 lines should be moved to scrollback
        assert_eq!(screen.scrollback.len(), scrollback_before + 2);
    }

    #[test]
    fn test_resize_cursor_adjustment() {
        let mut screen = TerminalScreen::new(80, 24);
        // Move cursor to position
        screen.process(b"\x1b[10;40H"); // Row 10, Col 40

        // Resize to smaller dimensions
        screen.resize(30, 15);

        let (row, col) = screen.cursor_position();
        // Cursor should be adjusted to fit within new bounds
        assert!(row < 15);
        assert!(col < 30);
    }

    #[test]
    fn test_resize_minimum_dimensions() {
        let mut screen = TerminalScreen::new(80, 24);

        // Try to resize to 0x0 (should be clamped to 1x1)
        screen.resize(0, 0);

        assert_eq!(screen.buffer.len(), 1);
        assert_eq!(screen.buffer[0].len(), 1);
    }
}

#[cfg(test)]
mod alternate_screen_tests {
    use super::*;

    #[test]
    fn test_alternate_screen_basic_switch() {
        let mut screen = TerminalScreen::new(80, 24);

        // Write some text on main screen
        screen.process(b"Main screen text");
        let main_buffer = screen.buffer.clone();

        // Switch to alternate screen (CSI ?47h)
        screen.process(b"\x1b[?47h");
        assert!(screen.use_alternate_screen);

        // Alternate screen should be clear
        assert_eq!(screen.buffer[0][0].c, ' ');

        // Write on alternate screen
        screen.process(b"Alternate screen text");

        // Switch back (CSI ?47l)
        screen.process(b"\x1b[?47l");
        assert!(!screen.use_alternate_screen);

        // Main screen text should be restored
        assert_eq!(screen.buffer[0][0].c, main_buffer[0][0].c);
    }

    #[test]
    fn test_alternate_screen_1047_clear() {
        let mut screen = TerminalScreen::new(80, 24);

        // Write on main screen
        screen.process(b"Main screen");

        // Switch with CSI ?1047h (should clear alternate screen)
        screen.process(b"\x1b[?1047h");
        assert!(screen.use_alternate_screen);
        assert_eq!(screen.buffer[0][0].c, ' ');

        // Write on alternate screen
        screen.process(b"Alt");
        assert_eq!(screen.buffer[0][0].c, 'A');

        // Switch back
        screen.process(b"\x1b[?1047l");
        assert!(!screen.use_alternate_screen);
        assert_eq!(screen.buffer[0][0].c, 'M'); // 'Main screen'
    }

    #[test]
    fn test_alternate_screen_1049_full_save() {
        let mut screen = TerminalScreen::new(80, 24);

        // Set up main screen with cursor position and attributes
        screen.process(b"\x1b[5;10HMain \x1b[31mRed Text\x1b[0m");
        let (row, col) = screen.cursor_position();

        // Switch with CSI ?1049h (save cursor + clear)
        screen.process(b"\x1b[?1049h");
        assert!(screen.use_alternate_screen);

        // Cursor should be reset to 0,0 in alternate screen
        let (alt_row, alt_col) = screen.cursor_position();
        assert_eq!(alt_row, 0);
        assert_eq!(alt_col, 0);

        // Write on alternate screen
        screen.process(b"Alternate content");

        // Switch back - should restore cursor position
        screen.process(b"\x1b[?1049l");
        assert!(!screen.use_alternate_screen);

        // Original content and cursor position should be restored
        let (restored_row, restored_col) = screen.cursor_position();
        assert_eq!(restored_row, row);
        assert_eq!(restored_col, col);
    }

    #[test]
    fn test_alternate_screen_preserves_attributes() {
        let mut screen = TerminalScreen::new(80, 24);

        // Set text attributes on main screen
        screen.process(b"\x1b[1;4;31mBold Red Underline\x1b[0m");
        screen.process(b"\x1b[1m"); // Set bold again

        // Switch to alternate screen
        screen.process(b"\x1b[?1049h");

        // Attributes should be reset in alternate screen
        assert!(!screen.bold);

        // Set different attributes in alternate screen
        screen.process(b"\x1b[34mBlue");
        assert!(screen.current_fg.is_some());

        // Switch back to main screen
        screen.process(b"\x1b[?1049l");

        // Original attributes should be restored (bold was set)
        assert!(screen.bold);
    }

    #[test]
    fn test_alternate_screen_preserves_scroll_region() {
        let mut screen = TerminalScreen::new(80, 24);

        // Set scroll region on main screen
        screen.process(b"\x1b[5;20r"); // Lines 5-20
        assert_eq!(screen.scroll_region, Some((4, 19))); // 0-indexed

        // Switch to alternate screen
        screen.process(b"\x1b[?1049h");

        // Scroll region should be cleared in alternate screen
        assert_eq!(screen.scroll_region, None);

        // Set different scroll region in alternate screen
        screen.process(b"\x1b[1;10r");
        assert_eq!(screen.scroll_region, Some((0, 9)));

        // Switch back
        screen.process(b"\x1b[?1049l");

        // Original scroll region should be restored
        assert_eq!(screen.scroll_region, Some((4, 19)));
    }

    #[test]
    fn test_alternate_screen_preserves_scrollback() {
        let mut screen = TerminalScreen::new(80, 3);

        // Generate scrollback on main screen
        for i in 0..10 {
            screen.process(format!("Line {}\n", i).as_bytes());
        }

        let scrollback_len = screen.scrollback.len();
        assert!(scrollback_len > 0, "Should have scrollback");

        // Switch to alternate screen
        screen.process(b"\x1b[?1049h");

        // Alternate screen should have empty scrollback
        assert_eq!(screen.scrollback.len(), 0);

        // Generate scrollback in alternate screen
        for i in 0..5 {
            screen.process(format!("Alt Line {}\n", i).as_bytes());
        }

        // Switch back
        screen.process(b"\x1b[?1049l");

        // Original scrollback should be restored
        assert_eq!(screen.scrollback.len(), scrollback_len);
    }

    #[test]
    fn test_alternate_screen_vim_like_behavior() {
        let mut screen = TerminalScreen::new(80, 24);

        // Simulate vim-like usage
        // 1. User has text on main screen
        screen.process(b"$ ls\nfile1.txt\nfile2.txt\n$ ");
        let main_content = screen.buffer.clone();

        // 2. Launch vim (CSI ?1049h)
        screen.process(b"\x1b[?1049h");
        assert!(screen.use_alternate_screen);

        // 3. Vim content
        screen.process(b"~\n~\n~\nfile.txt [New File]");

        // 4. Exit vim (CSI ?1049l)
        screen.process(b"\x1b[?1049l");
        assert!(!screen.use_alternate_screen);

        // 5. User should see original shell prompt
        assert_eq!(screen.buffer[3][2].c, main_content[3][2].c);
    }

    #[test]
    fn test_alternate_screen_less_like_behavior() {
        let mut screen = TerminalScreen::new(80, 24);

        // Main screen with command
        screen.process(b"$ cat bigfile.txt | less\n");

        // less enters alternate screen with CSI ?1047h
        screen.process(b"\x1b[?1047h");

        // Display file content
        screen.process(b"Line 1 of file\nLine 2 of file\nLine 3 of file");

        // User quits less (CSI ?1047l)
        screen.process(b"\x1b[?1047l");

        // Should return to shell prompt
        assert_eq!(screen.buffer[0][0].c, '$');
    }

    #[test]
    fn test_alternate_screen_multiple_switches() {
        let mut screen = TerminalScreen::new(80, 24);

        screen.process(b"Main1");

        // First switch
        screen.process(b"\x1b[?1049h");
        screen.process(b"Alt1");
        screen.process(b"\x1b[?1049l");
        assert_eq!(screen.buffer[0][0].c, 'M');

        // Second switch
        screen.process(b"\x1b[?1049h");
        screen.process(b"Alt2");
        screen.process(b"\x1b[?1049l");
        assert_eq!(screen.buffer[0][0].c, 'M');

        // State should be consistent after multiple switches
        assert!(!screen.use_alternate_screen);
    }

    #[test]
    fn test_alternate_screen_htop_like_behavior() {
        let mut screen = TerminalScreen::new(80, 24);

        // Shell prompt
        screen.process(b"$ htop\n");

        // htop uses CSI ?1049h
        screen.process(b"\x1b[?1049h");
        assert!(screen.use_alternate_screen);

        // htop draws interface with colors and positioning
        screen.process(b"\x1b[1;1H\x1b[7mCPU [|||||||||||||||| 100%]\x1b[0m");
        screen.process(b"\x1b[2;1HMem [|||||            50%]");

        // Exit htop
        screen.process(b"\x1b[?1049l");
        assert!(!screen.use_alternate_screen);

        // Back to shell
        assert_eq!(screen.buffer[0][0].c, '$');
    }

    #[test]
    fn test_alternate_screen_preserves_decsc_state() {
        let mut screen = TerminalScreen::new(80, 24);

        // Save cursor with DECSC on main screen
        screen.process(b"\x1b[10;20H"); // Move cursor
        screen.process(b"\x1b7"); // DECSC - save cursor
        screen.process(b"\x1b[1;1H"); // Move somewhere else

        // Switch to alternate screen
        screen.process(b"\x1b[?1049h");

        // DECSC state should be cleared in alternate screen
        screen.process(b"\x1b8"); // DECRC - restore cursor
        let (_row, _col) = screen.cursor_position();
        // Should be at origin or unchanged, not at saved position

        // Switch back
        screen.process(b"\x1b[?1049l");

        // Original DECSC state should be restored
        screen.process(b"\x1b8"); // DECRC
        let (restored_row, restored_col) = screen.cursor_position();
        assert_eq!(restored_row, 9); // 10th row (0-indexed)
        assert_eq!(restored_col, 19); // 20th column (0-indexed)
    }
}
