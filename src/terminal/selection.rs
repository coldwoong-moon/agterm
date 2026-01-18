//! Text selection module for terminal
//!
//! Provides text selection functionality including:
//! - Character-based selection (single click + drag)
//! - Word-based selection (double click)
//! - Line-based selection (triple click)
//! - Text extraction from selection ranges

use std::cmp::min;

/// A point in the terminal grid (line, column)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionPoint {
    pub line: usize,
    pub col: usize,
}

impl SelectionPoint {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

impl PartialOrd for SelectionPoint {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SelectionPoint {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.line.cmp(&other.line) {
            std::cmp::Ordering::Equal => self.col.cmp(&other.col),
            ord => ord,
        }
    }
}

/// Selection mode for mouse interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    /// No selection
    None,
    /// Character-level selection (click and drag)
    Character,
    /// Word-level selection (double-click)
    Word,
    /// Line-level selection (triple-click)
    Line,
}

impl Default for SelectionMode {
    fn default() -> Self {
        Self::None
    }
}

/// Text selection state
#[derive(Debug, Clone)]
pub struct Selection {
    pub mode: SelectionMode,
    pub start: SelectionPoint,
    pub end: SelectionPoint,
    pub active: bool,
}

impl Selection {
    /// Create a new empty selection
    pub fn new() -> Self {
        Self {
            mode: SelectionMode::None,
            start: SelectionPoint::new(0, 0),
            end: SelectionPoint::new(0, 0),
            active: false,
        }
    }

    /// Start a new selection at the given position
    pub fn start(&mut self, line: usize, col: usize, mode: SelectionMode) {
        self.mode = mode;
        self.start = SelectionPoint::new(line, col);
        self.end = SelectionPoint::new(line, col);
        self.active = true;
    }

    /// Extend the selection to a new position
    pub fn extend(&mut self, line: usize, col: usize) {
        if !self.active {
            return;
        }
        self.end = SelectionPoint::new(line, col);
    }

    /// Mark the selection as finished (no longer being modified)
    pub fn finish(&mut self) {
        // Selection remains active for rendering/copying, but is no longer being modified
        // If start == end, deactivate the selection
        if self.start == self.end {
            self.active = false;
        }
    }

    /// Get normalized range (start always before or equal to end)
    pub fn normalized(&self) -> (SelectionPoint, SelectionPoint) {
        if self.start <= self.end {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }

    /// Check if a specific position is within the selection
    pub fn contains(&self, line: usize, col: usize) -> bool {
        if !self.active {
            return false;
        }

        let (start, end) = self.normalized();
        let point = SelectionPoint::new(line, col);

        point >= start && point <= end
    }

    /// Clear the selection
    pub fn clear(&mut self) {
        self.mode = SelectionMode::None;
        self.active = false;
    }

    /// Extract selected text from the screen buffer
    pub fn extract_text<T: GetLineText>(&self, screen: &T) -> String {
        if !self.active {
            return String::new();
        }

        let (start, end) = self.normalized();

        // Empty selection
        if start == end {
            return String::new();
        }

        let mut result = String::new();

        // Handle single-line selection
        if start.line == end.line {
            if let Some(line_text) = screen.get_line_text(start.line) {
                let chars: Vec<char> = line_text.chars().collect();
                let start_col = min(start.col, chars.len());
                let end_col = min(end.col, chars.len());
                result.push_str(&chars[start_col..end_col].iter().collect::<String>());
            }
            return result;
        }

        // Multi-line selection
        for line_idx in start.line..=end.line {
            if line_idx >= screen.line_count() {
                break;
            }

            if let Some(line_text) = screen.get_line_text(line_idx) {
                let chars: Vec<char> = line_text.chars().collect();

                if line_idx == start.line {
                    // First line: from start.col to end
                    let start_col = min(start.col, chars.len());
                    result.push_str(&chars[start_col..].iter().collect::<String>());
                } else if line_idx == end.line {
                    // Last line: from beginning to end.col
                    let end_col = min(end.col, chars.len());
                    result.push_str(&chars[..end_col].iter().collect::<String>());
                } else {
                    // Middle lines: entire line
                    result.push_str(&line_text);
                }

                // Add newline between lines (but not after the last line)
                if line_idx < end.line {
                    result.push('\n');
                }
            }
        }

        result
    }

    /// Expand selection to word boundaries (for double-click)
    pub fn expand_to_word<T: GetLineText>(&mut self, screen: &T, line: usize, col: usize) {
        if let Some(line_text) = screen.get_line_text(line) {
            let chars: Vec<char> = line_text.chars().collect();
            if col >= chars.len() {
                return;
            }

            // Find word boundaries
            let mut start_col = col;
            let mut end_col = col;

            // Expand backwards
            while start_col > 0 && is_word_char(chars[start_col - 1]) {
                start_col -= 1;
            }

            // Expand forwards
            while end_col < chars.len() && is_word_char(chars[end_col]) {
                end_col += 1;
            }

            self.start = SelectionPoint::new(line, start_col);
            self.end = SelectionPoint::new(line, end_col);
            self.mode = SelectionMode::Word;
            self.active = true;
        }
    }

    /// Expand selection to entire line (for triple-click)
    pub fn expand_to_line<T: GetLineText>(&mut self, screen: &T, line: usize) {
        if let Some(line_text) = screen.get_line_text(line) {
            self.start = SelectionPoint::new(line, 0);
            self.end = SelectionPoint::new(line, line_text.chars().count());
            self.mode = SelectionMode::Line;
            self.active = true;
        }
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for accessing line text from a screen buffer
pub trait GetLineText {
    /// Get the text content of a specific line
    fn get_line_text(&self, line: usize) -> Option<String>;

    /// Get the total number of lines
    fn line_count(&self) -> usize;
}

/// Helper function to determine if a character is part of a word
fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-'
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockScreen {
        lines: Vec<String>,
    }

    impl GetLineText for MockScreen {
        fn get_line_text(&self, line: usize) -> Option<String> {
            self.lines.get(line).cloned()
        }

        fn line_count(&self) -> usize {
            self.lines.len()
        }
    }

    #[test]
    fn test_selection_point_ordering() {
        let p1 = SelectionPoint::new(0, 5);
        let p2 = SelectionPoint::new(0, 10);
        let p3 = SelectionPoint::new(1, 0);

        assert!(p1 < p2);
        assert!(p2 < p3);
        assert!(p1 < p3);
    }

    #[test]
    fn test_selection_normalized() {
        let mut sel = Selection::new();
        sel.start(0, 10, SelectionMode::Character);
        sel.extend(0, 5);

        let (start, end) = sel.normalized();
        assert_eq!(start, SelectionPoint::new(0, 5));
        assert_eq!(end, SelectionPoint::new(0, 10));
    }

    #[test]
    fn test_selection_contains() {
        let mut sel = Selection::new();
        sel.start(0, 5, SelectionMode::Character);
        sel.extend(2, 10);

        assert!(sel.contains(0, 5));
        assert!(sel.contains(0, 10));
        assert!(sel.contains(1, 5));
        assert!(sel.contains(2, 5));
        assert!(sel.contains(2, 10));
        assert!(!sel.contains(0, 4));
        assert!(!sel.contains(2, 11));
        assert!(!sel.contains(3, 0));
    }

    #[test]
    fn test_extract_single_line() {
        let screen = MockScreen {
            lines: vec!["Hello, World!".to_string()],
        };

        let mut sel = Selection::new();
        sel.start(0, 0, SelectionMode::Character);
        sel.extend(0, 5);

        let text = sel.extract_text(&screen);
        assert_eq!(text, "Hello");
    }

    #[test]
    fn test_extract_multi_line() {
        let screen = MockScreen {
            lines: vec![
                "First line".to_string(),
                "Second line".to_string(),
                "Third line".to_string(),
            ],
        };

        let mut sel = Selection::new();
        sel.start(0, 6, SelectionMode::Character);
        sel.extend(2, 5);

        let text = sel.extract_text(&screen);
        assert_eq!(text, "line\nSecond line\nThird");
    }

    #[test]
    fn test_expand_to_word() {
        let screen = MockScreen {
            lines: vec!["Hello World Test".to_string()],
        };

        let mut sel = Selection::new();
        sel.expand_to_word(&screen, 0, 7); // Click on 'W' in "World"

        let text = sel.extract_text(&screen);
        assert_eq!(text, "World");
    }

    #[test]
    fn test_expand_to_line() {
        let screen = MockScreen {
            lines: vec![
                "First line".to_string(),
                "Second line".to_string(),
            ],
        };

        let mut sel = Selection::new();
        sel.expand_to_line(&screen, 1);

        let text = sel.extract_text(&screen);
        assert_eq!(text, "Second line");
    }

    #[test]
    fn test_empty_selection() {
        let screen = MockScreen {
            lines: vec!["Test".to_string()],
        };

        let mut sel = Selection::new();
        sel.start(0, 5, SelectionMode::Character);
        sel.extend(0, 5);

        let text = sel.extract_text(&screen);
        assert_eq!(text, "");
    }

    #[test]
    fn test_clear_selection() {
        let mut sel = Selection::new();
        sel.start(0, 0, SelectionMode::Character);
        sel.extend(1, 10);

        assert!(sel.active);

        sel.clear();

        assert!(!sel.active);
        assert_eq!(sel.mode, SelectionMode::None);
    }
}
