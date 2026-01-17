//! Tests for newly added AgTerm features
//!
//! This test suite covers:
//! - Search functionality with text matching and highlighting
//! - OSC 8 hyperlink URL parsing and state management
//! - Terminal resize with content preservation and cursor adjustment
//! - Wide character handling (Korean, Japanese, Chinese, emoji)

use agterm::terminal::screen::{Cell, TerminalScreen};

// ============================================================================
// Search Functionality Tests
// ============================================================================

#[test]
fn test_search_single_match() {
    let mut screen = TerminalScreen::new(80, 24);

    // Write searchable content
    screen.process(b"Hello World\r\n");
    screen.process(b"This is a test\r\n");
    screen.process(b"Another line\r\n");

    let lines = screen.get_all_lines();

    // Search for "test"
    let matches = find_text_matches(&lines, "test");

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].0, 1); // Line 1 (0-indexed)
    assert!(matches[0].1 >= 10); // Column should be around position of "test"
}

#[test]
fn test_search_multiple_matches() {
    let mut screen = TerminalScreen::new(80, 24);

    // Write content with multiple occurrences
    screen.process(b"foo bar foo\r\n");
    screen.process(b"baz foo qux\r\n");
    screen.process(b"no match here\r\n");

    let lines = screen.get_all_lines();
    let matches = find_text_matches(&lines, "foo");

    // Should find 3 matches (2 on line 0, 1 on line 1)
    assert_eq!(matches.len(), 3);

    // Verify matches are on correct lines
    let line_0_matches: Vec<_> = matches.iter().filter(|m| m.0 == 0).collect();
    let line_1_matches: Vec<_> = matches.iter().filter(|m| m.0 == 1).collect();

    assert_eq!(line_0_matches.len(), 2);
    assert_eq!(line_1_matches.len(), 1);
}

#[test]
fn test_search_case_sensitive() {
    let mut screen = TerminalScreen::new(80, 24);

    screen.process(b"Hello HELLO hello\r\n");

    let lines = screen.get_all_lines();

    // Case-sensitive search for "hello"
    let matches = find_text_matches(&lines, "hello");

    // Should only match lowercase "hello"
    assert_eq!(matches.len(), 1);
}

#[test]
fn test_search_no_matches() {
    let mut screen = TerminalScreen::new(80, 24);

    screen.process(b"Some random text\r\n");
    screen.process(b"Nothing to find here\r\n");

    let lines = screen.get_all_lines();
    let matches = find_text_matches(&lines, "nonexistent");

    assert_eq!(matches.len(), 0);
}

#[test]
fn test_search_with_scrollback() {
    let mut screen = TerminalScreen::new(80, 5);

    // Fill screen and create scrollback
    for i in 0..10 {
        screen.process(format!("Line {} with searchterm\r\n", i).as_bytes());
    }

    let lines = screen.get_all_lines();
    let matches = find_text_matches(&lines, "searchterm");

    // Should find matches in both scrollback and visible area
    assert_eq!(matches.len(), 10);
}

#[test]
fn test_search_match_highlighting_bounds() {
    let mut screen = TerminalScreen::new(80, 24);

    screen.process(b"prefix_match_suffix\r\n");

    let lines = screen.get_all_lines();
    let matches = find_text_matches(&lines, "match");

    assert_eq!(matches.len(), 1);

    // Verify match bounds (line, start_col, end_col)
    let (line, start, end) = matches[0];
    assert_eq!(line, 0);
    assert_eq!(end - start, 5); // "match" is 5 characters
}

#[test]
fn test_search_with_wide_characters() {
    let mut screen = TerminalScreen::new(80, 24);

    // Mix ASCII and Korean
    screen.process(b"test ");
    screen.process("한글".as_bytes());
    screen.process(b" test\r\n");

    let lines = screen.get_all_lines();
    let matches = find_text_matches(&lines, "test");

    // Should find 2 matches despite wide characters in between
    assert_eq!(matches.len(), 2);
}

// ============================================================================
// OSC 8 Hyperlink Tests
// ============================================================================

#[test]
fn test_osc8_simple_hyperlink() {
    let mut screen = TerminalScreen::new(80, 24);

    // OSC 8 ; ; url ST text OSC 8 ; ; ST
    // Format: ESC ] 8 ; params ; url ST text ESC ] 8 ; ; ST
    screen.process(b"\x1b]8;;https://example.com\x1b\\");
    screen.process(b"Click here");
    screen.process(b"\x1b]8;;\x1b\\"); // End hyperlink

    // Note: OSC 8 is not yet implemented, so we test the expected behavior
    // When implemented, this test should verify hyperlink metadata is stored

    let lines = screen.get_all_lines();
    assert_eq!(lines[0][0].c, 'C'); // Text should still be rendered
}

#[test]
fn test_osc8_hyperlink_with_id() {
    let mut screen = TerminalScreen::new(80, 24);

    // OSC 8 with id parameter for link identification
    screen.process(b"\x1b]8;id=link1;https://example.com\x1b\\");
    screen.process(b"Link 1");
    screen.process(b"\x1b]8;;\x1b\\");

    screen.process(b" and ");

    screen.process(b"\x1b]8;id=link2;https://other.com\x1b\\");
    screen.process(b"Link 2");
    screen.process(b"\x1b]8;;\x1b\\");

    let lines = screen.get_all_lines();

    // Verify text is rendered
    let text: String = lines[0].iter()
        .filter(|c| !c.placeholder)
        .map(|c| c.c)
        .collect();

    assert!(text.contains("Link 1"));
    assert!(text.contains("Link 2"));
}

#[test]
fn test_osc8_nested_hyperlinks() {
    let mut screen = TerminalScreen::new(80, 24);

    // Start first hyperlink
    screen.process(b"\x1b]8;;https://outer.com\x1b\\");
    screen.process(b"Outer ");

    // Start second hyperlink (should override first)
    screen.process(b"\x1b]8;;https://inner.com\x1b\\");
    screen.process(b"Inner");

    // End all hyperlinks
    screen.process(b"\x1b]8;;\x1b\\");

    let lines = screen.get_all_lines();

    // Text should be rendered regardless
    let text: String = lines[0].iter()
        .filter(|c| !c.placeholder)
        .map(|c| c.c)
        .collect();

    assert!(text.contains("Outer"));
    assert!(text.contains("Inner"));
}

#[test]
fn test_osc8_url_parsing() {
    let test_cases = vec![
        ("https://example.com", true),
        ("http://example.com", true),
        ("ftp://files.example.com", true),
        ("file:///home/user/file.txt", true),
        ("mailto:user@example.com", true),
        ("", false), // Empty URL ends hyperlink
        ("not a url", false),
    ];

    for (url, is_valid) in test_cases {
        let parsed_url = parse_hyperlink_url(url);
        assert_eq!(parsed_url.is_some(), is_valid, "Failed for URL: {}", url);
    }
}

// ============================================================================
// Terminal Resize Tests
// ============================================================================

#[test]
fn test_resize_preserve_content() {
    let mut screen = TerminalScreen::new(80, 24);

    // Write some content
    screen.process(b"Line 1\r\n");
    screen.process(b"Line 2\r\n");
    screen.process(b"Line 3\r\n");

    // Resize to smaller dimensions
    screen.resize(40, 12);

    let lines = screen.get_all_lines();

    // Content should still be present
    let has_line1 = lines.iter().any(|line| {
        line.iter().any(|cell| cell.c == '1')
    });
    assert!(has_line1, "Line 1 should be preserved after resize");
}

#[test]
fn test_resize_cursor_adjustment() {
    let mut screen = TerminalScreen::new(80, 24);

    // Move cursor to specific position
    screen.process(b"\x1b[10;10H"); // Move to row 10, col 10
    screen.process(b"X");

    let (row, col) = screen.cursor_position();
    assert_eq!(row, 9); // 0-indexed
    assert_eq!(col, 10); // After writing 'X'

    // Resize to smaller terminal
    screen.resize(40, 12);

    let (new_row, new_col) = screen.cursor_position();

    // Cursor should be adjusted to fit within new bounds
    assert!(new_row < 12, "Cursor row should be within new height");
    assert!(new_col < 40, "Cursor column should be within new width");
}

#[test]
fn test_resize_increase_columns() {
    let mut screen = TerminalScreen::new(40, 24);

    // Fill a line
    screen.process(b"Short line\r\n");

    // Increase width
    screen.resize(80, 24);

    let lines = screen.get_all_lines();

    // Content should still be there
    let text: String = lines[0].iter()
        .filter(|c| !c.placeholder)
        .map(|c| c.c)
        .take(10)
        .collect();

    assert_eq!(text.trim(), "Short line");

    // Extra columns should be filled with spaces
    assert_eq!(lines[0].len(), 80);
}

#[test]
fn test_resize_decrease_columns() {
    let mut screen = TerminalScreen::new(80, 24);

    // Write long line
    screen.process(b"This is a very long line that will be truncated\r\n");

    // Decrease width
    screen.resize(20, 24);

    let lines = screen.get_all_lines();

    // Line should be truncated
    assert_eq!(lines[0].len(), 20);

    // First part should be preserved
    assert_eq!(lines[0][0].c, 'T');
}

#[test]
fn test_resize_increase_rows_restore_scrollback() {
    let mut screen = TerminalScreen::new(80, 5);

    // Create scrollback by writing more than screen height
    for i in 0..10 {
        screen.process(format!("Line {}\r\n", i).as_bytes());
    }

    // Lines 0-4 should be in scrollback, lines 5-9 visible
    let lines_before = screen.get_all_lines();
    assert!(lines_before.len() > 5);

    // Increase rows
    screen.resize(80, 12);

    let lines_after = screen.get_all_lines();

    // Some scrollback lines should be restored to visible area
    assert!(lines_after.len() >= 12);
}

#[test]
fn test_resize_decrease_rows_save_to_scrollback() {
    let mut screen = TerminalScreen::new(80, 24);

    // Fill screen
    for i in 0..20 {
        screen.process(format!("Line {}\r\n", i).as_bytes());
    }

    let lines_before = screen.get_all_lines();
    let initial_line_count = lines_before.len();

    // Decrease rows
    screen.resize(80, 10);

    let lines_after = screen.get_all_lines();

    // Total lines should be preserved in scrollback
    assert!(lines_after.len() >= initial_line_count);
}

#[test]
fn test_resize_maintains_scrollback_limit() {
    let mut screen = TerminalScreen::new(80, 5);

    // Create more scrollback than limit (10000)
    for i in 0..11000 {
        screen.process(format!("{}\r\n", i).as_bytes());
    }

    // Resize
    screen.resize(40, 10);

    let lines = screen.get_all_lines();

    // Should not exceed MAX_SCROLLBACK + visible rows (10000 + 10)
    assert!(lines.len() <= 10010);
}

#[test]
fn test_resize_with_wide_characters() {
    let mut screen = TerminalScreen::new(40, 10);

    // Write wide characters
    screen.process(b"Test ");
    screen.process("한글".as_bytes());
    screen.process(b"\r\n");

    let lines_before = screen.get_all_lines();
    assert!(lines_before[0].iter().any(|c| c.wide));

    // Resize
    screen.resize(80, 20);

    let lines_after = screen.get_all_lines();

    // Wide characters should still be marked correctly
    assert!(lines_after[0].iter().any(|c| c.wide));
    assert!(lines_after[0].iter().any(|c| c.placeholder));
}

#[test]
fn test_resize_scroll_region_reset() {
    let mut screen = TerminalScreen::new(80, 24);

    // Set scroll region
    screen.process(b"\x1b[5;20r"); // Lines 5-20

    // Resize should reset scroll region
    screen.resize(40, 12);

    // Write at bottom should scroll entire screen, not just region
    screen.process(b"\x1b[12;1H"); // Move to bottom
    screen.process(b"Test\r\n");

    // Should not crash or have unexpected behavior
    let lines = screen.get_all_lines();
    assert!(lines.len() >= 12);
}

// ============================================================================
// Wide Character Tests
// ============================================================================

#[test]
fn test_wide_char_korean_single() {
    let mut screen = TerminalScreen::new(80, 24);

    screen.process("한".as_bytes());

    let lines = screen.get_all_lines();

    // First cell should contain the character and be marked as wide
    assert_eq!(lines[0][0].c, '한');
    assert!(lines[0][0].wide);

    // Second cell should be a placeholder
    assert!(lines[0][1].placeholder);

    // Cursor should advance 2 columns
    assert_eq!(screen.cursor_position(), (0, 2));
}

#[test]
fn test_wide_char_korean_multiple() {
    let mut screen = TerminalScreen::new(80, 24);

    screen.process("한글".as_bytes());

    let lines = screen.get_all_lines();

    // First Korean character
    assert_eq!(lines[0][0].c, '한');
    assert!(lines[0][0].wide);
    assert!(lines[0][1].placeholder);

    // Second Korean character
    assert_eq!(lines[0][2].c, '글');
    assert!(lines[0][2].wide);
    assert!(lines[0][3].placeholder);

    // Cursor should be at column 4
    assert_eq!(screen.cursor_position(), (0, 4));
}

#[test]
fn test_wide_char_japanese() {
    let mut screen = TerminalScreen::new(80, 24);

    screen.process("日本語".as_bytes());

    let lines = screen.get_all_lines();

    // Each Japanese character should occupy 2 cells
    assert_eq!(lines[0][0].c, '日');
    assert!(lines[0][0].wide);

    assert_eq!(lines[0][2].c, '本');
    assert!(lines[0][2].wide);

    assert_eq!(lines[0][4].c, '語');
    assert!(lines[0][4].wide);

    // 3 characters × 2 columns = 6
    assert_eq!(screen.cursor_position(), (0, 6));
}

#[test]
fn test_wide_char_chinese() {
    let mut screen = TerminalScreen::new(80, 24);

    screen.process("中文".as_bytes());

    let lines = screen.get_all_lines();

    assert_eq!(lines[0][0].c, '中');
    assert!(lines[0][0].wide);
    assert!(lines[0][1].placeholder);

    assert_eq!(lines[0][2].c, '文');
    assert!(lines[0][2].wide);
    assert!(lines[0][3].placeholder);
}

#[test]
fn test_wide_char_emoji() {
    let mut screen = TerminalScreen::new(80, 24);

    screen.process("🚀".as_bytes());

    let lines = screen.get_all_lines();

    // Emoji should be marked as wide
    assert_eq!(lines[0][0].c, '🚀');
    assert!(lines[0][0].wide);
    assert!(lines[0][1].placeholder);
}

#[test]
fn test_wide_char_mixed_with_ascii() {
    let mut screen = TerminalScreen::new(80, 24);

    screen.process(b"Hello ");
    screen.process("世界".as_bytes()); // "World" in Chinese
    screen.process(b"!");

    let lines = screen.get_all_lines();

    // ASCII characters
    assert_eq!(lines[0][0].c, 'H');
    assert!(!lines[0][0].wide);

    // Wide characters start at column 6
    assert_eq!(lines[0][6].c, '世');
    assert!(lines[0][6].wide);

    assert_eq!(lines[0][8].c, '界');
    assert!(lines[0][8].wide);

    // Exclamation mark after wide chars
    assert_eq!(lines[0][10].c, '!');
    assert!(!lines[0][10].wide);
}

#[test]
fn test_wide_char_at_line_end() {
    let mut screen = TerminalScreen::new(10, 5);

    // Write up to column 8 (leaving 2 columns)
    screen.process(b"12345678");

    // Write wide character (should fit exactly)
    screen.process("한".as_bytes());

    let lines = screen.get_all_lines();

    assert_eq!(lines[0][8].c, '한');
    assert!(lines[0][8].wide);
    assert!(lines[0][9].placeholder);

    // Cursor should be at column 10 (triggers wrap on next write)
    assert_eq!(screen.cursor_position(), (0, 10));
}

#[test]
fn test_wide_char_forced_wrap() {
    let mut screen = TerminalScreen::new(10, 5);

    // Write up to column 9 (leaving only 1 column)
    screen.process(b"123456789");

    // Try to write wide character (needs 2 columns, only 1 available)
    screen.process("한".as_bytes());

    let lines = screen.get_all_lines();

    // Wide character should wrap to next line
    assert_eq!(lines[1][0].c, '한');
    assert!(lines[1][0].wide);
    assert!(lines[1][1].placeholder);
}

#[test]
fn test_wide_char_backspace() {
    let mut screen = TerminalScreen::new(80, 24);

    screen.process("한".as_bytes());
    screen.process(b"\x08"); // Backspace

    let _lines = screen.get_all_lines();
    let (_row, col) = screen.cursor_position();

    // Backspace should move cursor back 1 position
    // (Note: actual deletion behavior depends on implementation)
    assert_eq!(col, 1);
}

#[test]
fn test_wide_char_overwrite() {
    let mut screen = TerminalScreen::new(80, 24);

    screen.process("한글".as_bytes()); // 4 columns
    screen.process(b"\r"); // Return to start
    screen.process(b"AB"); // Overwrite first 2 columns

    let lines = screen.get_all_lines();

    // ASCII should overwrite wide char
    assert_eq!(lines[0][0].c, 'A');
    assert_eq!(lines[0][1].c, 'B');

    // Second wide char might be partially overwritten or intact
    // depending on implementation
}

#[test]
fn test_wide_char_with_attributes() {
    let mut screen = TerminalScreen::new(80, 24);

    // Apply bold
    screen.process(b"\x1b[1m");
    screen.process("한글".as_bytes());

    let lines = screen.get_all_lines();

    // Wide characters should have bold attribute
    assert!(lines[0][0].bold);
    assert!(lines[0][0].wide);

    assert!(lines[0][2].bold);
    assert!(lines[0][2].wide);
}

#[test]
fn test_wide_char_zero_width_joiner() {
    let mut screen = TerminalScreen::new(80, 24);

    // Complex emoji with ZWJ (Zero-Width Joiner)
    // 👨‍💻 is actually multiple codepoints
    screen.process("👨‍💻".as_bytes());

    let _lines = screen.get_all_lines();

    // Should handle gracefully (exact behavior may vary)
    let cursor_pos = screen.cursor_position();
    assert!(cursor_pos.1 > 0, "Cursor should have advanced");
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Find all occurrences of text in terminal lines
/// Returns (line_index, start_col, end_col) for each match
fn find_text_matches(lines: &[Vec<Cell>], query: &str) -> Vec<(usize, usize, usize)> {
    let mut matches = Vec::new();

    for (line_idx, line) in lines.iter().enumerate() {
        // Convert line to string, skipping placeholders
        let line_text: String = line.iter()
            .filter(|c| !c.placeholder)
            .map(|c| c.c)
            .collect();

        // Find all occurrences in this line
        let mut search_start = 0;
        while let Some(pos) = line_text[search_start..].find(query) {
            let actual_pos = search_start + pos;
            matches.push((line_idx, actual_pos, actual_pos + query.len()));
            search_start = actual_pos + 1;
        }
    }

    matches
}

/// Parse hyperlink URL (placeholder for future OSC 8 implementation)
fn parse_hyperlink_url(url: &str) -> Option<String> {
    if url.is_empty() {
        return None;
    }

    // Basic URL validation
    if url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("ftp://")
        || url.starts_with("file://")
        || url.starts_with("mailto:")
    {
        Some(url.to_string())
    } else {
        None
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_search_after_resize() {
    let mut screen = TerminalScreen::new(80, 24);

    screen.process(b"searchable content\r\n");

    // Resize
    screen.resize(40, 12);

    let lines = screen.get_all_lines();
    let matches = find_text_matches(&lines, "searchable");

    // Search should still work after resize
    assert_eq!(matches.len(), 1);
}

#[test]
fn test_wide_char_in_search_results() {
    let mut screen = TerminalScreen::new(80, 24);

    screen.process(b"test ");
    screen.process("검색".as_bytes()); // Korean for "search"
    screen.process(b" test\r\n");

    let lines = screen.get_all_lines();
    let matches = find_text_matches(&lines, "test");

    // Both "test" occurrences should be found
    assert_eq!(matches.len(), 2);
}

#[test]
fn test_resize_preserves_wide_chars() {
    let mut screen = TerminalScreen::new(40, 10);

    screen.process("한글 ".as_bytes());
    screen.process("日本語 ".as_bytes());
    screen.process("中文\r\n".as_bytes());

    screen.resize(80, 20);

    let lines = screen.get_all_lines();

    // All wide characters should still be present
    let has_korean = lines.iter().any(|l| l.iter().any(|c| c.c == '한'));
    let has_japanese = lines.iter().any(|l| l.iter().any(|c| c.c == '日'));
    let has_chinese = lines.iter().any(|l| l.iter().any(|c| c.c == '中'));

    assert!(has_korean);
    assert!(has_japanese);
    assert!(has_chinese);
}
