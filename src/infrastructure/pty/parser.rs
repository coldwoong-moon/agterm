//! ANSI Escape Sequence Parser
//!
//! Uses vte to parse terminal output and maintain screen state.

use std::fmt;
use vte::{Params, Parser, Perform};

/// Terminal cell attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CellAttributes {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub blink: bool,
    pub inverse: bool,
    pub hidden: bool,
    pub strikethrough: bool,
    pub fg_color: Color,
    pub bg_color: Color,
}

/// Terminal colors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Color {
    #[default]
    Default,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Indexed(u8),
    Rgb(u8, u8, u8),
}

/// A single cell in the terminal grid
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub char: char,
    pub attrs: CellAttributes,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            char: ' ',
            attrs: CellAttributes::default(),
        }
    }
}

/// Terminal screen buffer
#[derive(Clone)]
pub struct TerminalScreen {
    /// Screen width in columns
    pub cols: usize,
    /// Screen height in rows
    pub rows: usize,
    /// Cell grid (row-major order)
    cells: Vec<Vec<Cell>>,
    /// Current cursor position (0-indexed)
    pub cursor_row: usize,
    pub cursor_col: usize,
    /// Current cell attributes for new characters
    current_attrs: CellAttributes,
    /// Scroll region (top, bottom) - 0-indexed, inclusive
    scroll_top: usize,
    scroll_bottom: usize,
    /// Whether cursor is visible
    pub cursor_visible: bool,
    /// Terminal title
    pub title: String,
    /// Raw output buffer for debugging
    raw_output: Vec<u8>,
    /// Maximum raw output size to keep
    max_raw_output: usize,
}

impl TerminalScreen {
    /// Create a new terminal screen
    pub fn new(cols: usize, rows: usize) -> Self {
        let cells = vec![vec![Cell::default(); cols]; rows];

        Self {
            cols,
            rows,
            cells,
            cursor_row: 0,
            cursor_col: 0,
            current_attrs: CellAttributes::default(),
            scroll_top: 0,
            scroll_bottom: rows.saturating_sub(1),
            cursor_visible: true,
            title: String::new(),
            raw_output: Vec::new(),
            max_raw_output: 65536,
        }
    }

    /// Resize the screen
    pub fn resize(&mut self, cols: usize, rows: usize) {
        // Resize existing rows
        for row in &mut self.cells {
            row.resize(cols, Cell::default());
        }

        // Add or remove rows
        self.cells.resize(rows, vec![Cell::default(); cols]);

        self.cols = cols;
        self.rows = rows;
        self.scroll_bottom = rows.saturating_sub(1);

        // Ensure cursor is within bounds
        self.cursor_row = self.cursor_row.min(rows.saturating_sub(1));
        self.cursor_col = self.cursor_col.min(cols.saturating_sub(1));
    }

    /// Get a cell at the given position
    pub fn get_cell(&self, row: usize, col: usize) -> Option<&Cell> {
        self.cells.get(row).and_then(|r| r.get(col))
    }

    /// Get a mutable cell at the given position
    fn get_cell_mut(&mut self, row: usize, col: usize) -> Option<&mut Cell> {
        self.cells.get_mut(row).and_then(|r| r.get_mut(col))
    }

    /// Set a character at the current cursor position
    fn put_char(&mut self, c: char) {
        if self.cursor_col >= self.cols {
            // Wrap to next line
            self.cursor_col = 0;
            self.cursor_row += 1;
            if self.cursor_row > self.scroll_bottom {
                self.scroll_up(1);
                self.cursor_row = self.scroll_bottom;
            }
        }

        // Copy attrs before mutable borrow
        let attrs = self.current_attrs;
        if let Some(cell) = self.get_cell_mut(self.cursor_row, self.cursor_col) {
            cell.char = c;
            cell.attrs = attrs;
        }

        self.cursor_col += 1;
    }

    /// Scroll the screen up by n lines
    fn scroll_up(&mut self, n: usize) {
        for _ in 0..n {
            if self.scroll_top < self.scroll_bottom {
                self.cells.remove(self.scroll_top);
                self.cells
                    .insert(self.scroll_bottom, vec![Cell::default(); self.cols]);
            }
        }
    }

    /// Scroll the screen down by n lines
    fn scroll_down(&mut self, n: usize) {
        for _ in 0..n {
            if self.scroll_top < self.scroll_bottom {
                self.cells.remove(self.scroll_bottom);
                self.cells
                    .insert(self.scroll_top, vec![Cell::default(); self.cols]);
            }
        }
    }

    /// Clear from cursor to end of line
    fn clear_to_eol(&mut self) {
        for col in self.cursor_col..self.cols {
            if let Some(cell) = self.get_cell_mut(self.cursor_row, col) {
                *cell = Cell::default();
            }
        }
    }

    /// Clear from cursor to start of line
    fn clear_to_sol(&mut self) {
        for col in 0..=self.cursor_col {
            if let Some(cell) = self.get_cell_mut(self.cursor_row, col) {
                *cell = Cell::default();
            }
        }
    }

    /// Clear entire line
    fn clear_line(&mut self) {
        for col in 0..self.cols {
            if let Some(cell) = self.get_cell_mut(self.cursor_row, col) {
                *cell = Cell::default();
            }
        }
    }

    /// Clear from cursor to end of screen
    fn clear_to_eos(&mut self) {
        self.clear_to_eol();
        for row in (self.cursor_row + 1)..self.rows {
            for col in 0..self.cols {
                if let Some(cell) = self.get_cell_mut(row, col) {
                    *cell = Cell::default();
                }
            }
        }
    }

    /// Clear from cursor to start of screen
    fn clear_to_sos(&mut self) {
        self.clear_to_sol();
        for row in 0..self.cursor_row {
            for col in 0..self.cols {
                if let Some(cell) = self.get_cell_mut(row, col) {
                    *cell = Cell::default();
                }
            }
        }
    }

    /// Clear entire screen
    fn clear_screen(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                *cell = Cell::default();
            }
        }
    }

    /// Get the content as a string (for display)
    pub fn to_string_content(&self) -> String {
        let mut result = String::new();
        for (i, row) in self.cells.iter().enumerate() {
            if i > 0 {
                result.push('\n');
            }
            let line: String = row.iter().map(|c| c.char).collect();
            result.push_str(line.trim_end());
        }
        result
    }

    /// Get a specific row as a string
    pub fn get_row_string(&self, row: usize) -> Option<String> {
        self.cells.get(row).map(|r| {
            r.iter().map(|c| c.char).collect::<String>()
        })
    }

    /// Process raw output bytes
    pub fn process(&mut self, data: &[u8]) {
        // Store raw output for debugging
        self.raw_output.extend_from_slice(data);
        if self.raw_output.len() > self.max_raw_output {
            let excess = self.raw_output.len() - self.max_raw_output;
            self.raw_output.drain(0..excess);
        }
    }

    /// Get raw output buffer
    pub fn raw_output(&self) -> &[u8] {
        &self.raw_output
    }
}

impl fmt::Debug for TerminalScreen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TerminalScreen")
            .field("cols", &self.cols)
            .field("rows", &self.rows)
            .field("cursor", &(self.cursor_row, self.cursor_col))
            .finish()
    }
}

/// ANSI parser that updates a TerminalScreen
pub struct AnsiParser {
    parser: Parser,
    screen: TerminalScreen,
}

impl AnsiParser {
    /// Create a new ANSI parser
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            parser: Parser::new(),
            screen: TerminalScreen::new(cols, rows),
        }
    }

    /// Process input bytes
    pub fn process(&mut self, data: &[u8]) {
        self.screen.process(data);

        for byte in data {
            self.parser.advance(&mut self.screen, *byte);
        }
    }

    /// Get the screen
    pub fn screen(&self) -> &TerminalScreen {
        &self.screen
    }

    /// Get mutable screen
    pub fn screen_mut(&mut self) -> &mut TerminalScreen {
        &mut self.screen
    }

    /// Resize the screen
    pub fn resize(&mut self, cols: usize, rows: usize) {
        self.screen.resize(cols, rows);
    }
}

impl Perform for TerminalScreen {
    fn print(&mut self, c: char) {
        self.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            // Bell
            0x07 => {}
            // Backspace
            0x08 => {
                self.cursor_col = self.cursor_col.saturating_sub(1);
            }
            // Horizontal Tab
            0x09 => {
                let next_tab = ((self.cursor_col / 8) + 1) * 8;
                self.cursor_col = next_tab.min(self.cols - 1);
            }
            // Line Feed / Vertical Tab / Form Feed
            0x0A | 0x0B | 0x0C => {
                self.cursor_row += 1;
                if self.cursor_row > self.scroll_bottom {
                    self.scroll_up(1);
                    self.cursor_row = self.scroll_bottom;
                }
            }
            // Carriage Return
            0x0D => {
                self.cursor_col = 0;
            }
            _ => {}
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {}

    fn put(&mut self, _byte: u8) {}

    fn unhook(&mut self) {}

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        // Handle OSC sequences (e.g., setting title)
        if params.len() >= 2 {
            match params[0] {
                // Set title
                b"0" | b"2" => {
                    if let Ok(title) = std::str::from_utf8(params[1]) {
                        self.title = title.to_string();
                    }
                }
                _ => {}
            }
        }
    }

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, action: char) {
        let params: Vec<u16> = params.iter().flatten().copied().collect();

        match action {
            // Cursor Up
            'A' => {
                let n = params.first().copied().unwrap_or(1).max(1) as usize;
                self.cursor_row = self.cursor_row.saturating_sub(n);
            }
            // Cursor Down
            'B' => {
                let n = params.first().copied().unwrap_or(1).max(1) as usize;
                self.cursor_row = (self.cursor_row + n).min(self.rows - 1);
            }
            // Cursor Forward
            'C' => {
                let n = params.first().copied().unwrap_or(1).max(1) as usize;
                self.cursor_col = (self.cursor_col + n).min(self.cols - 1);
            }
            // Cursor Back
            'D' => {
                let n = params.first().copied().unwrap_or(1).max(1) as usize;
                self.cursor_col = self.cursor_col.saturating_sub(n);
            }
            // Cursor Next Line
            'E' => {
                let n = params.first().copied().unwrap_or(1).max(1) as usize;
                self.cursor_row = (self.cursor_row + n).min(self.rows - 1);
                self.cursor_col = 0;
            }
            // Cursor Previous Line
            'F' => {
                let n = params.first().copied().unwrap_or(1).max(1) as usize;
                self.cursor_row = self.cursor_row.saturating_sub(n);
                self.cursor_col = 0;
            }
            // Cursor Horizontal Absolute
            'G' => {
                let col = params.first().copied().unwrap_or(1).max(1) as usize;
                self.cursor_col = (col - 1).min(self.cols - 1);
            }
            // Cursor Position
            'H' | 'f' => {
                let row = params.first().copied().unwrap_or(1).max(1) as usize;
                let col = params.get(1).copied().unwrap_or(1).max(1) as usize;
                self.cursor_row = (row - 1).min(self.rows - 1);
                self.cursor_col = (col - 1).min(self.cols - 1);
            }
            // Erase in Display
            'J' => {
                let mode = params.first().copied().unwrap_or(0);
                match mode {
                    0 => self.clear_to_eos(),
                    1 => self.clear_to_sos(),
                    2 | 3 => self.clear_screen(),
                    _ => {}
                }
            }
            // Erase in Line
            'K' => {
                let mode = params.first().copied().unwrap_or(0);
                match mode {
                    0 => self.clear_to_eol(),
                    1 => self.clear_to_sol(),
                    2 => self.clear_line(),
                    _ => {}
                }
            }
            // Scroll Up
            'S' => {
                let n = params.first().copied().unwrap_or(1).max(1) as usize;
                self.scroll_up(n);
            }
            // Scroll Down
            'T' => {
                let n = params.first().copied().unwrap_or(1).max(1) as usize;
                self.scroll_down(n);
            }
            // SGR - Select Graphic Rendition
            'm' => {
                self.handle_sgr(&params);
            }
            // Show/Hide cursor
            'l' | 'h' => {
                // DECSET/DECRST
                if params.first() == Some(&25) {
                    self.cursor_visible = action == 'h';
                }
            }
            // Set scroll region
            'r' => {
                let top = params.first().copied().unwrap_or(1).max(1) as usize;
                let bottom = params.get(1).copied().unwrap_or(self.rows as u16) as usize;
                self.scroll_top = (top - 1).min(self.rows - 1);
                self.scroll_bottom = (bottom - 1).min(self.rows - 1).max(self.scroll_top);
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}

impl TerminalScreen {
    fn handle_sgr(&mut self, params: &[u16]) {
        if params.is_empty() {
            self.current_attrs = CellAttributes::default();
            return;
        }

        let mut i = 0;
        while i < params.len() {
            match params[i] {
                0 => self.current_attrs = CellAttributes::default(),
                1 => self.current_attrs.bold = true,
                3 => self.current_attrs.italic = true,
                4 => self.current_attrs.underline = true,
                5 => self.current_attrs.blink = true,
                7 => self.current_attrs.inverse = true,
                8 => self.current_attrs.hidden = true,
                9 => self.current_attrs.strikethrough = true,
                22 => self.current_attrs.bold = false,
                23 => self.current_attrs.italic = false,
                24 => self.current_attrs.underline = false,
                25 => self.current_attrs.blink = false,
                27 => self.current_attrs.inverse = false,
                28 => self.current_attrs.hidden = false,
                29 => self.current_attrs.strikethrough = false,
                // Foreground colors
                30 => self.current_attrs.fg_color = Color::Black,
                31 => self.current_attrs.fg_color = Color::Red,
                32 => self.current_attrs.fg_color = Color::Green,
                33 => self.current_attrs.fg_color = Color::Yellow,
                34 => self.current_attrs.fg_color = Color::Blue,
                35 => self.current_attrs.fg_color = Color::Magenta,
                36 => self.current_attrs.fg_color = Color::Cyan,
                37 => self.current_attrs.fg_color = Color::White,
                38 => {
                    // Extended foreground color
                    if i + 2 < params.len() && params[i + 1] == 5 {
                        self.current_attrs.fg_color = Color::Indexed(params[i + 2] as u8);
                        i += 2;
                    } else if i + 4 < params.len() && params[i + 1] == 2 {
                        self.current_attrs.fg_color = Color::Rgb(
                            params[i + 2] as u8,
                            params[i + 3] as u8,
                            params[i + 4] as u8,
                        );
                        i += 4;
                    }
                }
                39 => self.current_attrs.fg_color = Color::Default,
                // Background colors
                40 => self.current_attrs.bg_color = Color::Black,
                41 => self.current_attrs.bg_color = Color::Red,
                42 => self.current_attrs.bg_color = Color::Green,
                43 => self.current_attrs.bg_color = Color::Yellow,
                44 => self.current_attrs.bg_color = Color::Blue,
                45 => self.current_attrs.bg_color = Color::Magenta,
                46 => self.current_attrs.bg_color = Color::Cyan,
                47 => self.current_attrs.bg_color = Color::White,
                48 => {
                    // Extended background color
                    if i + 2 < params.len() && params[i + 1] == 5 {
                        self.current_attrs.bg_color = Color::Indexed(params[i + 2] as u8);
                        i += 2;
                    } else if i + 4 < params.len() && params[i + 1] == 2 {
                        self.current_attrs.bg_color = Color::Rgb(
                            params[i + 2] as u8,
                            params[i + 3] as u8,
                            params[i + 4] as u8,
                        );
                        i += 4;
                    }
                }
                49 => self.current_attrs.bg_color = Color::Default,
                // Bright foreground colors
                90 => self.current_attrs.fg_color = Color::BrightBlack,
                91 => self.current_attrs.fg_color = Color::BrightRed,
                92 => self.current_attrs.fg_color = Color::BrightGreen,
                93 => self.current_attrs.fg_color = Color::BrightYellow,
                94 => self.current_attrs.fg_color = Color::BrightBlue,
                95 => self.current_attrs.fg_color = Color::BrightMagenta,
                96 => self.current_attrs.fg_color = Color::BrightCyan,
                97 => self.current_attrs.fg_color = Color::BrightWhite,
                // Bright background colors
                100 => self.current_attrs.bg_color = Color::BrightBlack,
                101 => self.current_attrs.bg_color = Color::BrightRed,
                102 => self.current_attrs.bg_color = Color::BrightGreen,
                103 => self.current_attrs.bg_color = Color::BrightYellow,
                104 => self.current_attrs.bg_color = Color::BrightBlue,
                105 => self.current_attrs.bg_color = Color::BrightMagenta,
                106 => self.current_attrs.bg_color = Color::BrightCyan,
                107 => self.current_attrs.bg_color = Color::BrightWhite,
                _ => {}
            }
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_screen_creation() {
        let screen = TerminalScreen::new(80, 24);
        assert_eq!(screen.cols, 80);
        assert_eq!(screen.rows, 24);
        assert_eq!(screen.cursor_row, 0);
        assert_eq!(screen.cursor_col, 0);
    }

    #[test]
    fn test_ansi_parser_simple_text() {
        let mut parser = AnsiParser::new(80, 24);
        parser.process(b"Hello, World!");

        let content = parser.screen().to_string_content();
        assert!(content.starts_with("Hello, World!"));
    }

    #[test]
    fn test_ansi_parser_newline() {
        let mut parser = AnsiParser::new(80, 24);
        parser.process(b"Line 1\r\nLine 2");

        let line0 = parser.screen().get_row_string(0).unwrap();
        let line1 = parser.screen().get_row_string(1).unwrap();

        assert!(line0.starts_with("Line 1"));
        assert!(line1.starts_with("Line 2"));
    }

    #[test]
    fn test_ansi_parser_cursor_movement() {
        let mut parser = AnsiParser::new(80, 24);
        // Move cursor to row 5, col 10 (1-indexed in ANSI)
        parser.process(b"\x1b[5;10H");

        assert_eq!(parser.screen().cursor_row, 4);
        assert_eq!(parser.screen().cursor_col, 9);
    }
}
