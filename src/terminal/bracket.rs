//! Bracket matching for terminal emulator
//!
//! This module provides bracket matching functionality to highlight matching
//! pairs of brackets when the cursor is positioned on one.

/// Matching bracket pairs
pub const BRACKET_PAIRS: [(char, char); 4] = [
    ('(', ')'),
    ('[', ']'),
    ('{', '}'),
    ('<', '>'),
];

/// A matched bracket pair position
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BracketMatch {
    /// Position of the opening bracket (line, col)
    pub open_pos: (usize, usize),
    /// Position of the closing bracket (line, col)
    pub close_pos: (usize, usize),
    /// The opening bracket character
    pub bracket_type: char,
}

/// Trait for accessing characters in the terminal screen
pub trait GetChar {
    /// Get the character at the given position
    fn get_char(&self, line: usize, col: usize) -> Option<char>;
    /// Get the total number of lines
    fn line_count(&self) -> usize;
    /// Get the width of a specific line
    fn line_width(&self, line: usize) -> usize;
}

/// Find matching bracket at the cursor position
///
/// Returns `Some(BracketMatch)` if the cursor is on a bracket and a matching
/// bracket is found, otherwise returns `None`.
pub fn find_matching_bracket(
    screen: &impl GetChar,
    cursor_line: usize,
    cursor_col: usize,
) -> Option<BracketMatch> {
    let current_char = screen.get_char(cursor_line, cursor_col)?;

    // Check if current character is an opening bracket
    for (open, close) in BRACKET_PAIRS {
        if current_char == open {
            return find_closing_bracket(screen, cursor_line, cursor_col, open, close);
        }
        if current_char == close {
            return find_opening_bracket(screen, cursor_line, cursor_col, open, close);
        }
    }

    None
}

/// Find the closing bracket matching the opening bracket at the given position
fn find_closing_bracket(
    screen: &impl GetChar,
    start_line: usize,
    start_col: usize,
    open_char: char,
    close_char: char,
) -> Option<BracketMatch> {
    let mut depth = 1; // We've already seen the opening bracket
    let line_count = screen.line_count();

    // Start searching from the position after the opening bracket
    let mut line = start_line;
    let mut col = start_col + 1;

    // Search forward through all lines
    while line < line_count {
        let line_width = screen.line_width(line);

        while col < line_width {
            if let Some(ch) = screen.get_char(line, col) {
                if ch == open_char {
                    depth += 1;
                } else if ch == close_char {
                    depth -= 1;
                    if depth == 0 {
                        // Found the matching closing bracket
                        return Some(BracketMatch {
                            open_pos: (start_line, start_col),
                            close_pos: (line, col),
                            bracket_type: open_char,
                        });
                    }
                }
            }
            col += 1;
        }

        // Move to the next line
        line += 1;
        col = 0;
    }

    // No matching bracket found
    None
}

/// Find the opening bracket matching the closing bracket at the given position
fn find_opening_bracket(
    screen: &impl GetChar,
    start_line: usize,
    start_col: usize,
    open_char: char,
    close_char: char,
) -> Option<BracketMatch> {
    let mut depth = 1; // We've already seen the closing bracket

    // Start searching from the position before the closing bracket
    let mut line = start_line as isize;
    let mut col = start_col as isize - 1;

    // Search backward through all lines
    while line >= 0 {
        // If col is negative, move to the previous line
        if col < 0 {
            line -= 1;
            if line < 0 {
                break;
            }
            col = screen.line_width(line as usize) as isize - 1;
            continue;
        }

        if let Some(ch) = screen.get_char(line as usize, col as usize) {
            if ch == close_char {
                depth += 1;
            } else if ch == open_char {
                depth -= 1;
                if depth == 0 {
                    // Found the matching opening bracket
                    return Some(BracketMatch {
                        open_pos: (line as usize, col as usize),
                        close_pos: (start_line, start_col),
                        bracket_type: open_char,
                    });
                }
            }
        }

        col -= 1;
    }

    // No matching bracket found
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Simple test screen implementation
    struct TestScreen {
        lines: Vec<String>,
    }

    impl TestScreen {
        fn new(lines: Vec<&str>) -> Self {
            Self {
                lines: lines.iter().map(|s| s.to_string()).collect(),
            }
        }
    }

    impl GetChar for TestScreen {
        fn get_char(&self, line: usize, col: usize) -> Option<char> {
            self.lines.get(line)?.chars().nth(col)
        }

        fn line_count(&self) -> usize {
            self.lines.len()
        }

        fn line_width(&self, line: usize) -> usize {
            self.lines.get(line).map(|s| s.len()).unwrap_or(0)
        }
    }

    #[test]
    fn test_simple_parentheses() {
        let screen = TestScreen::new(vec!["(hello)"]);

        // Test opening parenthesis
        let result = find_matching_bracket(&screen, 0, 0);
        assert_eq!(
            result,
            Some(BracketMatch {
                open_pos: (0, 0),
                close_pos: (0, 6),
                bracket_type: '(',
            })
        );

        // Test closing parenthesis
        let result = find_matching_bracket(&screen, 0, 6);
        assert_eq!(
            result,
            Some(BracketMatch {
                open_pos: (0, 0),
                close_pos: (0, 6),
                bracket_type: '(',
            })
        );
    }

    #[test]
    fn test_nested_brackets() {
        let screen = TestScreen::new(vec!["((a))"]);

        // Test outer opening bracket
        let result = find_matching_bracket(&screen, 0, 0);
        assert_eq!(
            result,
            Some(BracketMatch {
                open_pos: (0, 0),
                close_pos: (0, 4),
                bracket_type: '(',
            })
        );

        // Test inner opening bracket
        let result = find_matching_bracket(&screen, 0, 1);
        assert_eq!(
            result,
            Some(BracketMatch {
                open_pos: (0, 1),
                close_pos: (0, 3),
                bracket_type: '(',
            })
        );

        // Test inner closing bracket
        let result = find_matching_bracket(&screen, 0, 3);
        assert_eq!(
            result,
            Some(BracketMatch {
                open_pos: (0, 1),
                close_pos: (0, 3),
                bracket_type: '(',
            })
        );

        // Test outer closing bracket
        let result = find_matching_bracket(&screen, 0, 4);
        assert_eq!(
            result,
            Some(BracketMatch {
                open_pos: (0, 0),
                close_pos: (0, 4),
                bracket_type: '(',
            })
        );
    }

    #[test]
    fn test_multiline_brackets() {
        let screen = TestScreen::new(vec!["function() {", "  return 42;", "}"]);

        // Test opening brace on line 0
        let result = find_matching_bracket(&screen, 0, 11);
        assert_eq!(
            result,
            Some(BracketMatch {
                open_pos: (0, 11),
                close_pos: (2, 0),
                bracket_type: '{',
            })
        );

        // Test closing brace on line 2
        let result = find_matching_bracket(&screen, 2, 0);
        assert_eq!(
            result,
            Some(BracketMatch {
                open_pos: (0, 11),
                close_pos: (2, 0),
                bracket_type: '{',
            })
        );
    }

    #[test]
    fn test_different_bracket_types() {
        let screen = TestScreen::new(vec!["[{<>}]"]);

        // Test square brackets
        let result = find_matching_bracket(&screen, 0, 0);
        assert_eq!(
            result,
            Some(BracketMatch {
                open_pos: (0, 0),
                close_pos: (0, 5),
                bracket_type: '[',
            })
        );

        // Test curly braces
        let result = find_matching_bracket(&screen, 0, 1);
        assert_eq!(
            result,
            Some(BracketMatch {
                open_pos: (0, 1),
                close_pos: (0, 4),
                bracket_type: '{',
            })
        );

        // Test angle brackets
        let result = find_matching_bracket(&screen, 0, 2);
        assert_eq!(
            result,
            Some(BracketMatch {
                open_pos: (0, 2),
                close_pos: (0, 3),
                bracket_type: '<',
            })
        );
    }

    #[test]
    fn test_unmatched_bracket() {
        let screen = TestScreen::new(vec!["(hello"]);

        // Should return None for unmatched bracket
        let result = find_matching_bracket(&screen, 0, 0);
        assert_eq!(result, None);
    }

    #[test]
    fn test_no_bracket_at_cursor() {
        let screen = TestScreen::new(vec!["hello"]);

        // Should return None when cursor is not on a bracket
        let result = find_matching_bracket(&screen, 0, 0);
        assert_eq!(result, None);
    }

    #[test]
    fn test_complex_nested_brackets() {
        let screen = TestScreen::new(vec!["if (x > 0 && (y < 10)) {"]);

        // Test outer parenthesis
        let result = find_matching_bracket(&screen, 0, 3);
        assert_eq!(
            result,
            Some(BracketMatch {
                open_pos: (0, 3),
                close_pos: (0, 21),
                bracket_type: '(',
            })
        );

        // Test inner parenthesis
        let result = find_matching_bracket(&screen, 0, 13);
        assert_eq!(
            result,
            Some(BracketMatch {
                open_pos: (0, 13),
                close_pos: (0, 20),
                bracket_type: '(',
            })
        );
    }
}
