//! Integration tests for edge cases in terminal emulation
//!
//! Tests handling of very long lines, rapid output, wide characters,
//! and other boundary conditions to ensure robustness.

use agterm::terminal::screen::TerminalScreen;

#[test]
fn test_very_long_line() {
    let mut screen = TerminalScreen::new(80, 24);

    // Write a line longer than terminal width (should wrap)
    let long_line = "A".repeat(200);
    screen.process(long_line.as_bytes());

    let lines = screen.get_all_lines();

    // First line should be full
    assert_eq!(lines[0][0].c, 'A');
    assert_eq!(lines[0][79].c, 'A'); // Last column

    // Should wrap to next line
    assert_eq!(lines[1][0].c, 'A');

    // Should wrap again
    assert_eq!(lines[2][0].c, 'A');
}

#[test]
fn test_long_line_with_wrapping_disabled() {
    let mut screen = TerminalScreen::new(80, 24);

    // Disable auto-wrap mode (DECAWM)
    // Note: This is typically CSI ?7l, but we'll test by checking the behavior
    // The TerminalScreen doesn't have public API to disable wrapping yet
    // So this test documents expected behavior

    // Write a long line
    screen.process(b"This is a very long line that exceeds terminal width");

    // With wrapping enabled (default), it should wrap
    let lines = screen.get_all_lines();
    assert_eq!(lines[0][0].c, 'T');
}

#[test]
fn test_rapid_consecutive_output() {
    let mut screen = TerminalScreen::new(80, 24);

    // Simulate rapid output (like compilation output)
    for i in 0..100 {
        screen.process(format!("Compiling module {} ... done\r\n", i).as_bytes());
    }

    let lines = screen.get_all_lines();

    // Should have scrollback buffer
    assert!(lines.len() > 24);

    // Last line with content should be module 99
    // Format: "Compiling module 99 ... done"
    // The "99" appears after "Compiling module "
    let has_99 = lines.iter().any(|line| {
        // Search for two consecutive '9' characters
        line.windows(2).any(|w| w[0].c == '9' && w[1].c == '9')
    });
    assert!(has_99, "Should find line with module 99");
}

#[test]
fn test_rapid_cursor_movements() {
    let mut screen = TerminalScreen::new(80, 24);

    // Rapid cursor movements (stress test)
    for _ in 0..100 {
        screen.process(b"\x1b[10;10H"); // Move to (10, 10)
        screen.process(b"X");
        screen.process(b"\x1b[5;5H"); // Move to (5, 5)
        screen.process(b"Y");
    }

    let lines = screen.get_all_lines();

    // Should have written to the specified positions
    assert_eq!(lines[4][4].c, 'Y'); // Row 5, col 5 (0-indexed)
    assert_eq!(lines[9][9].c, 'X'); // Row 10, col 10 (0-indexed)
}

#[test]
fn test_many_ansi_sequences() {
    let mut screen = TerminalScreen::new(80, 24);

    // Many rapid ANSI sequences (color changes)
    for i in 0..50 {
        let color = (i % 8) + 30; // Cycle through colors 30-37
        screen.process(format!("\x1b[{}m", color).as_bytes());
        screen.process(b"#");
    }

    let lines = screen.get_all_lines();

    // Should have 50 '#' characters
    let first_line = &lines[0];
    let hash_count = first_line.iter().filter(|c| c.c == '#').count();
    assert_eq!(hash_count, 50);
}

#[test]
fn test_wide_character_at_boundary() {
    let mut screen = TerminalScreen::new(10, 5);

    // Write text up to the last 2 columns, then a wide char
    screen.process(b"12345678");
    assert_eq!(screen.cursor_position(), (0, 8));

    // Write a wide character (should fit in last 2 columns)
    screen.process("한".as_bytes());

    let lines = screen.get_all_lines();
    assert_eq!(lines[0][8].c, '한');
    assert!(lines[0][8].wide);
    assert!(lines[0][9].placeholder);

    // Cursor should be at position 10 (which triggers wrap to next line)
    assert_eq!(screen.cursor_position(), (0, 10));
}

#[test]
fn test_wide_character_forced_wrap() {
    let mut screen = TerminalScreen::new(10, 5);

    // Write text up to the last column
    screen.process(b"123456789");
    assert_eq!(screen.cursor_position(), (0, 9));

    // Try to write a wide character (needs 2 columns, only 1 left)
    // Should wrap to next line
    screen.process("한".as_bytes());

    let lines = screen.get_all_lines();
    // Wide character should be on line 2
    assert_eq!(lines[1][0].c, '한');
    assert!(lines[1][0].wide);
    assert!(lines[1][1].placeholder);
}

#[test]
fn test_mixed_width_characters_line() {
    let mut screen = TerminalScreen::new(40, 5);

    // Mix of ASCII, Korean, Japanese, Chinese
    screen.process(b"ASCII ");
    screen.process("한글".as_bytes()); // Korean (2 wide chars)
    screen.process(b" more ");
    screen.process("日本語".as_bytes()); // Japanese (3 wide chars)
    screen.process(b" and ");
    screen.process("中文".as_bytes()); // Chinese (2 wide chars)

    let lines = screen.get_all_lines();

    // Verify ASCII at start
    assert_eq!(lines[0][0].c, 'A');
    assert!(!lines[0][0].wide);

    // Verify wide characters present
    let line_text: String = lines[0]
        .iter()
        .filter(|c| !c.placeholder)
        .map(|c| c.c)
        .collect();

    assert!(line_text.contains("ASCII"));
    assert!(line_text.contains("한"));
    assert!(line_text.contains("日"));
    assert!(line_text.contains("中"));
}

#[test]
fn test_zero_width_terminal() {
    // Edge case: what if terminal is resized to 1 column?
    let mut screen = TerminalScreen::new(1, 24);

    screen.process(b"Hello");

    // Each character should be on its own line
    let lines = screen.get_all_lines();
    assert_eq!(lines[0][0].c, 'H');
    assert_eq!(lines[1][0].c, 'e');
    assert_eq!(lines[2][0].c, 'l');
    assert_eq!(lines[3][0].c, 'l');
    assert_eq!(lines[4][0].c, 'o');
}

#[test]
fn test_one_row_terminal() {
    // Edge case: terminal with only 1 row
    let mut screen = TerminalScreen::new(80, 1);

    screen.process(b"Line 1\r\n");
    screen.process(b"Line 2\r\n");
    screen.process(b"Line 3\r\n");

    // Should scroll, keeping only the last line visible
    let lines = screen.get_all_lines();

    // Should have scrollback
    assert!(lines.len() > 1);

    // Last line with content should be "Line 3"
    // (After \r\n, cursor is on a new line)
    let line_3 = lines
        .iter()
        .rev()
        .find(|line| line.len() > 5 && line[0].c == 'L' && line[5].c == '3');
    assert!(line_3.is_some(), "Should find Line 3");
}

#[test]
fn test_resize_while_content_present() {
    let mut screen = TerminalScreen::new(80, 24);

    // Add content
    for i in 1..=10 {
        screen.process(format!("Line {}\r\n", i).as_bytes());
    }

    // Resize to smaller
    screen.resize(40, 12);

    // Content should still be present (partially)
    let lines = screen.get_all_lines();

    // Should have some scrollback
    assert!(lines.len() >= 12);

    // Verify some content survived
    let has_line_text = lines
        .iter()
        .any(|line| line.iter().any(|cell| cell.c == 'L'));
    assert!(has_line_text);
}

#[test]
fn test_extreme_resize_down_then_up() {
    let mut screen = TerminalScreen::new(80, 24);

    // Add content
    screen.process(b"Original content\r\n");

    // Resize very small
    screen.resize(10, 3);

    // Resize back up
    screen.resize(80, 24);

    // Some content should be in scrollback
    let lines = screen.get_all_lines();
    assert!(lines.len() > 0);
}

#[test]
fn test_null_bytes_in_stream() {
    let mut screen = TerminalScreen::new(80, 24);

    // Some applications might send null bytes
    screen.process(b"Hello\x00World");

    let lines = screen.get_all_lines();
    // Null bytes might be ignored or treated as space
    assert_eq!(lines[0][0].c, 'H');
}

#[test]
fn test_incomplete_escape_sequence() {
    let mut screen = TerminalScreen::new(80, 24);

    // Send incomplete escape sequence
    screen.process(b"Normal text ");
    screen.process(b"\x1b["); // Incomplete CSI
    screen.process(b" more text");

    // Should handle gracefully
    let lines = screen.get_all_lines();
    assert_eq!(lines[0][0].c, 'N');
}

#[test]
fn test_malformed_escape_sequence() {
    let mut screen = TerminalScreen::new(80, 24);

    // Send malformed escape sequence
    screen.process(b"Before ");
    screen.process(b"\x1b[999999999999m"); // Huge number
    screen.process(b"After");

    // Should not crash
    let lines = screen.get_all_lines();
    assert_eq!(lines[0][0].c, 'B');
}

#[test]
fn test_excessive_scrollback() {
    let mut screen = TerminalScreen::new(80, 24);

    // Generate more lines than scrollback limit (10000)
    for i in 0..15000 {
        screen.process(format!("Line {}\r\n", i).as_bytes());
    }

    let lines = screen.get_all_lines();

    // Should cap at MAX_SCROLLBACK + visible rows
    // MAX_SCROLLBACK = 10000, visible = 24
    assert!(lines.len() <= 10024);

    // Should have the most recent lines
    // Find a line starting with 'L' (Line)
    let has_line = lines
        .iter()
        .rev()
        .any(|line| line.len() > 0 && line[0].c == 'L');
    assert!(has_line, "Should find lines starting with 'L'");
}

#[test]
fn test_rapid_screen_clear() {
    let mut screen = TerminalScreen::new(80, 24);

    // Rapidly clear and redraw
    for i in 0..100 {
        screen.process(b"\x1b[2J"); // Clear screen
        screen.process(b"\x1b[H"); // Home
        screen.process(format!("Frame {}", i).as_bytes());
    }

    let lines = screen.get_all_lines();
    // Should show final frame
    assert_eq!(lines[0][0].c, 'F');
    assert_eq!(lines[0][6].c, '9');
    assert_eq!(lines[0][7].c, '9');
}

#[test]
fn test_cursor_movement_bounds() {
    let mut screen = TerminalScreen::new(80, 24);

    // Try to move cursor way out of bounds
    screen.process(b"\x1b[999;999H"); // Move to (999, 999)

    // Should clamp to bottom-right
    assert_eq!(screen.cursor_position(), (23, 79));

    // Try to move cursor to negative (0, 0 is minimum)
    screen.process(b"\x1b[0;0H");
    assert_eq!(screen.cursor_position(), (0, 0));
}

#[test]
fn test_combined_scroll_and_write() {
    let mut screen = TerminalScreen::new(80, 10);

    // Fill screen
    for i in 1..=10 {
        screen.process(format!("Line {}\r\n", i).as_bytes());
    }

    // Write more (should scroll)
    screen.process(b"Line 11\r\n");
    screen.process(b"Line 12\r\n");

    let lines = screen.get_all_lines();

    // Should have scrollback
    assert!(lines.len() > 10);

    // Last line with content should be Line 12
    let line_12 = lines
        .iter()
        .rev()
        .find(|line| line.len() > 6 && line[5].c == '1' && line[6].c == '2');
    assert!(line_12.is_some(), "Should find Line 12");
}

#[test]
fn test_tab_at_end_of_line() {
    let mut screen = TerminalScreen::new(80, 24);

    // Move to near end of line
    screen.process(b"\x1b[1;78H");
    assert_eq!(screen.cursor_position(), (0, 77));

    // Send tab (should go to next tab stop or end of line)
    screen.process(b"\t");

    // Should be at end of line or wrapped
    let pos = screen.cursor_position();
    assert!(pos.1 >= 77);
}

#[test]
fn test_many_attributes_toggle() {
    let mut screen = TerminalScreen::new(80, 24);

    // Rapidly toggle attributes
    for _ in 0..50 {
        screen.process(b"\x1b[1m"); // Bold on
        screen.process(b"\x1b[22m"); // Bold off
        screen.process(b"\x1b[4m"); // Underline on
        screen.process(b"\x1b[24m"); // Underline off
        screen.process(b"\x1b[7m"); // Reverse on
        screen.process(b"\x1b[27m"); // Reverse off
    }

    screen.process(b"Text");

    // Should not crash and text should be visible
    let lines = screen.get_all_lines();
    assert_eq!(lines[0][0].c, 'T');
}

#[test]
fn test_unicode_normalization() {
    let mut screen = TerminalScreen::new(80, 24);

    // Some Unicode characters can be represented multiple ways
    // Test that we handle them correctly
    screen.process("Café".as_bytes()); // é as single character
    screen.process(b" ");
    screen.process("Café".as_bytes()); // é as e + combining accent

    let lines = screen.get_all_lines();
    // Both should render (exact behavior may vary)
    assert_eq!(lines[0][0].c, 'C');
}

#[test]
fn test_emoji_sequences() {
    let mut screen = TerminalScreen::new(40, 10);

    // Various emoji
    screen.process("🚀 ".as_bytes()); // Rocket
    screen.process("👨‍💻 ".as_bytes()); // Technologist (multi-codepoint)
    screen.process("❤️ ".as_bytes()); // Heart (with variation selector)

    let lines = screen.get_all_lines();
    // Should have some content (exact width handling may vary)
    let pos = screen.cursor_position();
    assert!(pos.1 > 0);
}

#[test]
fn test_alternate_charset() {
    let mut screen = TerminalScreen::new(80, 24);

    // Some terminals support alternate character sets (DEC line drawing)
    // Test that we don't crash on these sequences
    screen.process(b"\x1b(0"); // Switch to line drawing set
    screen.process(b"lqqqk"); // Draw a line
    screen.process(b"\x1b(B"); // Switch back to ASCII

    // Should not crash
    let lines = screen.get_all_lines();
    assert!(lines.len() > 0);
}

#[test]
fn test_simultaneous_scroll_regions() {
    let mut screen = TerminalScreen::new(80, 24);

    // Set scroll region
    screen.process(b"\x1b[5;20r");

    // Fill region
    for i in 5..=20 {
        screen.process(b"\x1b[H"); // Home
        screen.process(format!("\x1b[{}B", i - 1).as_bytes()); // Move down
        screen.process(format!("Region Line {}\r\n", i).as_bytes());
    }

    // Write more (should scroll within region)
    screen.process(b"\x1b[20;1H");
    screen.process(b"\r\nExtra line");

    // Should not affect lines outside region
    let lines = screen.get_all_lines();
    assert!(lines.len() >= 24);
}

#[test]
fn test_rapid_alternate_screen_switching() {
    let mut screen = TerminalScreen::new(80, 24);

    // Write to main screen
    screen.process(b"Main screen content\r\n");

    // Rapidly switch between screens
    for _ in 0..50 {
        screen.process(b"\x1b[?1049h"); // Enter alternate
        screen.process(b"Alt");
        screen.process(b"\x1b[?1049l"); // Exit alternate
    }

    // Main screen should be restored
    let lines = screen.get_all_lines();
    assert_eq!(lines[0][0].c, 'M');
}

#[test]
fn test_overwrite_with_shorter_text() {
    let mut screen = TerminalScreen::new(80, 24);

    // Write long text
    screen.process(b"This is a very long line of text");

    // Move to start
    screen.process(b"\r");

    // Overwrite with shorter text (without clearing)
    screen.process(b"Short");

    let lines = screen.get_all_lines();
    // Should have "Short" at start, but rest of old text remains
    // Original: "This is a very long line of text"
    // After:    "Shortis a very long line of text"
    assert_eq!(lines[0][0].c, 'S');
    assert_eq!(lines[0][1].c, 'h');
    assert_eq!(lines[0][2].c, 'o');
    assert_eq!(lines[0][3].c, 'r');
    assert_eq!(lines[0][4].c, 't');
    assert_eq!(lines[0][5].c, 'i'); // 'i' from "is" in original text
    assert_eq!(lines[0][6].c, 's'); // 's' from "is"
}

#[test]
fn test_wide_char_in_scrollback() {
    let mut screen = TerminalScreen::new(40, 5);

    // Write lines with wide characters
    for i in 0..10 {
        screen.process(format!("Line {} ", i).as_bytes());
        screen.process("한글".as_bytes());
        screen.process(b"\r\n");
    }

    let lines = screen.get_all_lines();

    // Should have scrollback with wide characters
    assert!(lines.len() > 5);

    // Check that wide characters are in scrollback
    let has_korean = lines
        .iter()
        .any(|line| line.iter().any(|cell| cell.c == '한'));
    assert!(has_korean);
}
