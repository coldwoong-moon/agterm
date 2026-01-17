//! Terminal screen buffer with ANSI escape code parsing

use iced::Color;
use std::cmp::{max, min};
use vte::{Params, Parser, Perform};
use std::collections::VecDeque;

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
            AnsiColor::Rgb(r, g, b) => Color::from_rgb(
                *r as f32 / 255.0,
                *g as f32 / 255.0,
                *b as f32 / 255.0,
            ),
        }
    }
}

/// Convert 16-color index to Iced Color
fn indexed_to_color(idx: u8) -> Color {
    match idx {
        0 => Color::from_rgb(0.0, 0.0, 0.0),         // Black
        1 => Color::from_rgb(0.8, 0.2, 0.2),         // Red
        2 => Color::from_rgb(0.2, 0.8, 0.2),         // Green
        3 => Color::from_rgb(0.8, 0.8, 0.2),         // Yellow
        4 => Color::from_rgb(0.2, 0.2, 0.8),         // Blue
        5 => Color::from_rgb(0.8, 0.2, 0.8),         // Magenta
        6 => Color::from_rgb(0.2, 0.8, 0.8),         // Cyan
        7 => Color::from_rgb(0.8, 0.8, 0.8),         // White
        8 => Color::from_rgb(0.5, 0.5, 0.5),         // Bright Black (Gray)
        9 => Color::from_rgb(1.0, 0.3, 0.3),         // Bright Red
        10 => Color::from_rgb(0.3, 1.0, 0.3),        // Bright Green
        11 => Color::from_rgb(1.0, 1.0, 0.3),        // Bright Yellow
        12 => Color::from_rgb(0.3, 0.3, 1.0),        // Bright Blue
        13 => Color::from_rgb(1.0, 0.3, 1.0),        // Bright Magenta
        14 => Color::from_rgb(0.3, 1.0, 1.0),        // Bright Cyan
        15 => Color::from_rgb(1.0, 1.0, 1.0),        // Bright White
        _ => Color::from_rgb(0.8, 0.8, 0.8),         // Default
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
            Color::from_rgb(gray as f32 / 255.0, gray as f32 / 255.0, gray as f32 / 255.0)
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
    pub bg: Option<AnsiColor>,
    pub bold: bool,
    pub underline: bool,
    pub reverse: bool,
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
        }
    }
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
    /// VTE parser
    parser: Parser,
    /// Scroll region (top, bottom) - None means full screen
    scroll_region: Option<(usize, usize)>,
    /// Saved cursor position (for save/restore)
    saved_cursor: Option<(usize, usize)>,
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
    /// Saved cursor position for alternate screen
    alternate_saved_cursor: Option<(usize, usize)>,
    /// Mouse reporting mode
    mouse_mode: MouseMode,
    /// Mouse encoding mode
    mouse_encoding: MouseEncoding,
    /// Cursor visibility (DECTCEM)
    cursor_visible: bool,
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
            parser: Parser::new(),
            scroll_region: None,
            saved_cursor: None,
            window_title: None,
            icon_name: None,
            cwd_from_shell: None,
            clipboard_request: None,
            alternate_buffer: None,
            alternate_scrollback: None,
            use_alternate_screen: false,
            alternate_saved_cursor: None,
            mouse_mode: MouseMode::None,
            mouse_encoding: MouseEncoding::Default,
            cursor_visible: true,
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

    /// Resize the terminal
    pub fn resize(&mut self, cols: usize, rows: usize) {
        let cols = max(1, cols);
        let rows = max(1, rows);

        if cols == self.cols && rows == self.rows {
            return;
        }

        // Save current buffer to scrollback if shrinking
        if rows < self.rows {
            let lines_to_save = self.rows - rows;
            for i in 0..lines_to_save {
                if i < self.buffer.len() {
                    self.scrollback.push_back(self.buffer[i].clone());
                }
            }
            // Limit scrollback
            if self.scrollback.len() > MAX_SCROLLBACK {
                let excess = self.scrollback.len() - MAX_SCROLLBACK;
                self.scrollback.drain(0..excess);
            }
        }

        // Resize buffer
        self.buffer = vec![vec![Cell::default(); cols]; rows];
        self.cols = cols;
        self.rows = rows;

        // Clamp cursor position
        self.cursor_row = min(self.cursor_row, rows - 1);
        self.cursor_col = min(self.cursor_col, cols - 1);

        // Reset scroll region
        self.scroll_region = None;
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

    /// Get window title (OSC 0 or OSC 2)
    pub fn window_title(&self) -> Option<&str> {
        self.window_title.as_deref()
    }

    /// Get icon name (OSC 1)
    pub fn icon_name(&self) -> Option<&str> {
        self.icon_name.as_deref()
    }

    /// Get current working directory from shell (OSC 7)
    pub fn cwd_from_shell(&self) -> Option<&str> {
        self.cwd_from_shell.as_deref()
    }

    /// Get clipboard request data (OSC 52)
    pub fn clipboard_request(&self) -> Option<&str> {
        self.clipboard_request.as_deref()
    }

    /// Clear clipboard request after it has been read
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

    /// Get current mouse reporting mode
    pub fn mouse_mode(&self) -> MouseMode {
        self.mouse_mode
    }

    /// Get current mouse encoding mode
    pub fn mouse_encoding(&self) -> MouseEncoding {
        self.mouse_encoding
    }

    /// Check if mouse reporting is enabled
    pub fn is_mouse_reporting_enabled(&self) -> bool {
        self.mouse_mode != MouseMode::None
    }

    /// Get cursor visibility state
    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
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

    /// Switch to alternate screen buffer
    fn switch_to_alternate_screen(&mut self) {
        if !self.use_alternate_screen {
            // Save current buffer and scrollback
            self.alternate_buffer = Some(self.buffer.clone());
            self.alternate_scrollback = Some(self.scrollback.clone());
            self.alternate_saved_cursor = self.saved_cursor;

            // Create fresh alternate screen
            self.buffer = vec![vec![Cell::default(); self.cols]; self.rows];
            self.scrollback = VecDeque::new();
            self.cursor_row = 0;
            self.cursor_col = 0;
            self.scroll_region = None;
            self.saved_cursor = None;

            self.use_alternate_screen = true;
        }
    }

    /// Switch back to normal screen buffer
    fn switch_to_normal_screen(&mut self) {
        if self.use_alternate_screen {
            // Restore saved buffer and scrollback
            if let Some(saved_buffer) = self.alternate_buffer.take() {
                self.buffer = saved_buffer;
            }
            if let Some(saved_scrollback) = self.alternate_scrollback.take() {
                self.scrollback = saved_scrollback;
            }
            self.saved_cursor = self.alternate_saved_cursor;

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
                }
                1 => self.bold = true,
                4 => self.underline = true,
                7 => self.reverse = true,
                22 => self.bold = false,
                24 => self.underline = false,
                27 => self.reverse = false,
                // Foreground colors (30-37, 90-97)
                30..=37 => self.current_fg = Some(AnsiColor::Indexed((value - 30) as u8)),
                38 => {
                    // Extended foreground color
                    if let Some(next) = iter.next() {
                        match next[0] {
                            2 => {
                                // RGB
                                if let (Some(r), Some(g), Some(b)) = (iter.next(), iter.next(), iter.next()) {
                                    self.current_fg = Some(AnsiColor::Rgb(r[0] as u8, g[0] as u8, b[0] as u8));
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
                                if let (Some(r), Some(g), Some(b)) = (iter.next(), iter.next(), iter.next()) {
                                    self.current_bg = Some(AnsiColor::Rgb(r[0] as u8, g[0] as u8, b[0] as u8));
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
        // Handle line wrapping
        if self.cursor_col >= self.cols {
            self.cursor_col = 0;
            self.new_line();
        }

        // Write character
        if self.cursor_row < self.rows && self.cursor_col < self.cols {
            self.buffer[self.cursor_row][self.cursor_col] = Cell {
                c,
                fg: self.current_fg,
                bg: self.current_bg,
                bold: self.bold,
                underline: self.underline,
                reverse: self.reverse,
            };
            self.cursor_col += 1;
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
                    1049 => {
                        // Save cursor and switch to alternate screen
                        self.saved_cursor = Some((self.cursor_row, self.cursor_col));
                        self.switch_to_alternate_screen();
                    }
                    47 | 1047 => {
                        // Switch to alternate screen (without saving cursor)
                        self.switch_to_alternate_screen();
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
                    1049 => {
                        // Switch to normal screen and restore cursor
                        self.switch_to_normal_screen();
                        if let Some((row, col)) = self.saved_cursor {
                            self.cursor_row = min(row, self.rows - 1);
                            self.cursor_col = min(col, self.cols - 1);
                        }
                    }
                    47 | 1047 => {
                        // Switch to normal screen (without restoring cursor)
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
                let row = iter.next().map(|p| p[0].saturating_sub(1) as usize).unwrap_or(0);
                let col = iter.next().map(|p| p[0].saturating_sub(1) as usize).unwrap_or(0);
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
                let top = iter.next().map(|p| p[0].saturating_sub(1) as usize).unwrap_or(0);
                let bottom = iter.next().map(|p| p[0].saturating_sub(1) as usize).unwrap_or(self.rows - 1);
                self.scroll_region = Some((
                    min(top, self.rows - 1),
                    min(bottom, self.rows - 1),
                ));
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
            _ => {
                // Unknown CSI sequence - ignore
            }
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {
        // ESC sequences - not implemented yet (for compatibility)
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
}
