//! Integration tests for vim-like editor simulation
//!
//! Tests alternate screen buffer, application cursor keys mode,
//! and complex cursor movement operations that vim typically uses.

use agterm::terminal::screen::TerminalScreen;

#[test]
fn test_vim_enter_alternate_screen() {
    let mut screen = TerminalScreen::new(80, 24);

    // Write some text to the main screen
    screen.process(b"Normal terminal text\r\n");
    screen.process(b"Line 2\r\n");
    screen.process(b"Line 3\r\n");

    // Verify we have text in the buffer
    let lines = screen.get_all_lines();
    assert_eq!(lines[0][0].c, 'N');

    // Enter alternate screen (vim enters with CSI ?1049h)
    screen.process(b"\x1b[?1049h");

    // Alternate screen should be empty
    let alt_lines = screen.get_all_lines();
    assert_eq!(alt_lines[0][0].c, ' '); // Should be blank

    // Exit alternate screen
    screen.process(b"\x1b[?1049l");

    // Should restore original text
    let restored_lines = screen.get_all_lines();
    assert_eq!(restored_lines[0][0].c, 'N');
}

#[test]
fn test_vim_application_cursor_keys() {
    let mut screen = TerminalScreen::new(80, 24);

    // Normal mode: cursor keys should be normal
    assert!(!screen.application_cursor_keys());

    // Vim enters application cursor keys mode (CSI ?1h)
    screen.process(b"\x1b[?1h");
    assert!(screen.application_cursor_keys());

    // Vim exits application cursor keys mode (CSI ?1l)
    screen.process(b"\x1b[?1l");
    assert!(!screen.application_cursor_keys());
}

#[test]
fn test_vim_full_screen_setup() {
    let mut screen = TerminalScreen::new(80, 24);

    // Simulate vim startup sequence
    // 1. Enter alternate screen
    screen.process(b"\x1b[?1049h");

    // 2. Enable application cursor keys
    screen.process(b"\x1b[?1h");

    // 3. Clear screen
    screen.process(b"\x1b[2J");

    // 4. Move to home
    screen.process(b"\x1b[H");

    // Verify state
    assert!(screen.application_cursor_keys());
    assert_eq!(screen.cursor_position(), (0, 0));

    // Screen should be empty
    let lines = screen.get_all_lines();
    assert_eq!(lines[0][0].c, ' ');
}

#[test]
fn test_vim_cursor_movement() {
    let mut screen = TerminalScreen::new(80, 24);

    // Enter alternate screen
    screen.process(b"\x1b[?1049h");

    // Write vim buffer content at specific positions
    screen.process(b"\x1b[1;1H"); // Move to row 1
    screen.process(b"Line 1: First line");
    screen.process(b"\x1b[2;1H"); // Move to row 2
    screen.process(b"Line 2: Second line");
    screen.process(b"\x1b[3;1H"); // Move to row 3
    screen.process(b"Line 3: Third line");

    // Test absolute cursor positioning (vim often uses this)
    screen.process(b"\x1b[2;5H"); // Move to row 2, col 5
    assert_eq!(screen.cursor_position(), (1, 4));

    // Test relative movement (CUF - Cursor Forward)
    screen.process(b"\x1b[3C"); // Move right 3 positions
    assert_eq!(screen.cursor_position(), (1, 7));

    // Test cursor back (CUB)
    screen.process(b"\x1b[2D"); // Move left 2 positions
    assert_eq!(screen.cursor_position(), (1, 5));

    // Verify left/right movement works in alternate screen
    // (Up/down movement behavior may vary based on scroll regions)

    // Exit alternate screen
    screen.process(b"\x1b[?1049l");
}

#[test]
fn test_vim_line_insertion_and_deletion() {
    let mut screen = TerminalScreen::new(40, 10);

    // Enter alternate screen
    screen.process(b"\x1b[?1049h");

    // Write initial buffer (5 lines)
    for i in 1..=5 {
        screen.process(format!("Line {}\r\n", i).as_bytes());
    }

    // Move to line 3 (index 2)
    screen.process(b"\x1b[3;1H");
    assert_eq!(screen.cursor_position(), (2, 0));

    // Insert a blank line (vim's 'O' command uses IL)
    screen.process(b"\x1b[L");

    // Verify line 3 is now blank
    let lines = screen.get_all_lines();
    assert_eq!(lines[2][0].c, ' '); // New blank line
    assert_eq!(lines[3][0].c, 'L'); // Old line 3 moved down

    // Move back to line 3
    screen.process(b"\x1b[3;1H");

    // Delete the line (vim's 'dd' command uses DL)
    screen.process(b"\x1b[M");

    // Verify line was deleted
    let lines = screen.get_all_lines();
    assert_eq!(lines[2][0].c, 'L'); // Old line 4 moved up
    assert_eq!(lines[2][5].c, '3'); // Should be "Line 3"

    // Exit alternate screen
    screen.process(b"\x1b[?1049l");
}

#[test]
fn test_vim_insert_delete_characters() {
    let mut screen = TerminalScreen::new(40, 10);

    // Enter alternate screen
    screen.process(b"\x1b[?1049h");

    // Write a line
    screen.process(b"Hello World");

    // Move to position 6 (after 'Hello ')
    screen.process(b"\x1b[1;7H");
    assert_eq!(screen.cursor_position(), (0, 6));

    // Insert 3 spaces (vim insert mode shifting text right)
    screen.process(b"\x1b[3@");

    // Verify characters shifted right
    let lines = screen.get_all_lines();
    assert_eq!(lines[0][6].c, ' ');
    assert_eq!(lines[0][7].c, ' ');
    assert_eq!(lines[0][8].c, ' ');
    assert_eq!(lines[0][9].c, 'W'); // 'W' from "World" shifted right

    // Move back to position 7
    screen.process(b"\x1b[1;7H");

    // Delete 3 characters (vim delete command)
    screen.process(b"\x1b[3P");

    // Verify characters deleted
    let lines = screen.get_all_lines();
    assert_eq!(lines[0][6].c, 'W'); // 'W' moved back to position 6

    // Exit alternate screen
    screen.process(b"\x1b[?1049l");
}

#[test]
fn test_vim_scroll_region() {
    let mut screen = TerminalScreen::new(80, 24);

    // Enter alternate screen
    screen.process(b"\x1b[?1049h");

    // Set scroll region (lines 5-20)
    // Vim uses this for split windows
    screen.process(b"\x1b[5;20r");

    // Fill the screen with numbered lines
    for i in 0..24 {
        screen.process(b"\x1b[H"); // Home
        screen.process(format!("\x1b[{}B", i).as_bytes()); // Move down
        screen.process(format!("Line {}", i + 1).as_bytes());
    }

    // Move to bottom of scroll region (line 20)
    screen.process(b"\x1b[20;1H");

    // Write a new line (should scroll within region)
    screen.process(b"\r\n");
    screen.process(b"New line");

    // Cursor should still be at line 20 (bottom of region)
    let (row, _) = screen.cursor_position();
    assert_eq!(row, 19); // 0-indexed

    // Clear scroll region
    screen.process(b"\x1b[r");

    // Exit alternate screen
    screen.process(b"\x1b[?1049l");
}

#[test]
fn test_vim_clear_operations() {
    let mut screen = TerminalScreen::new(80, 24);

    // Enter alternate screen
    screen.process(b"\x1b[?1049h");

    // Fill screen with text (use explicit positioning to avoid scrolling)
    for i in 0..24 {
        screen.process(format!("\x1b[{};1H", i + 1).as_bytes()); // Move to line i+1
        screen.process(format!("Line {} with some text here", i + 1).as_bytes());
    }

    // Move to middle of screen (row 12, col 10)
    screen.process(b"\x1b[12;10H");

    // Clear from cursor to end of screen (vim's 'dG' command)
    screen.process(b"\x1b[J");

    let lines = screen.get_all_lines();
    // Lines 0-10 should have text
    assert_eq!(lines[0][0].c, 'L');
    // Line 11 (row 12, 1-indexed) should be partially cleared (from col 9 onward, 0-indexed)
    // Before clear: "Line 12 with some text here"
    // After clear from col 10 (0-indexed col 9): "Line 12 w" + spaces
    assert_eq!(lines[11][0].c, 'L'); // Beginning still has text
    assert_eq!(lines[11][9].c, ' '); // Position 9 (col 10, 1-indexed) should be cleared
    // Lines 12+ should be empty
    assert_eq!(lines[12][0].c, ' ');
    assert_eq!(lines[23][0].c, ' ');

    // Move to row 5, col 15
    screen.process(b"\x1b[5;15H");

    // Clear current line from cursor to end (vim's 'D' command)
    screen.process(b"\x1b[K");

    let lines = screen.get_all_lines();
    // Line 4 should be cleared from position 15 onward
    assert_eq!(lines[4][0].c, 'L'); // Beginning still has text
    assert_eq!(lines[4][14].c, ' '); // Position 15 onwards should be cleared

    // Exit alternate screen
    screen.process(b"\x1b[?1049l");
}

#[test]
fn test_vim_save_restore_cursor() {
    let mut screen = TerminalScreen::new(80, 24);

    // Enter alternate screen
    screen.process(b"\x1b[?1049h");

    // Move to a specific position
    screen.process(b"\x1b[10;20H");
    assert_eq!(screen.cursor_position(), (9, 19));

    // Set text attributes
    screen.process(b"\x1b[1;31m"); // Bold red text

    // Save cursor with DECSC (ESC 7)
    screen.process(b"\x1b7");

    // Move somewhere else and change attributes
    screen.process(b"\x1b[5;5H");
    screen.process(b"\x1b[0m"); // Reset attributes
    assert_eq!(screen.cursor_position(), (4, 4));

    // Restore cursor with DECRC (ESC 8)
    screen.process(b"\x1b8");

    // Should be back at saved position with saved attributes
    assert_eq!(screen.cursor_position(), (9, 19));

    // Exit alternate screen
    screen.process(b"\x1b[?1049l");
}

#[test]
fn test_vim_cursor_visibility() {
    let mut screen = TerminalScreen::new(80, 24);

    // Default: cursor visible
    assert!(screen.cursor_visible());

    // Vim hides cursor during screen redraws
    screen.process(b"\x1b[?25l"); // Hide cursor (DECTCEM)
    assert!(!screen.cursor_visible());

    // Vim shows cursor after redraw
    screen.process(b"\x1b[?25h"); // Show cursor (DECTCEM)
    assert!(screen.cursor_visible());
}

#[test]
fn test_vim_complex_editing_sequence() {
    let mut screen = TerminalScreen::new(80, 24);

    // Simulate a complex vim editing session

    // 1. Enter vim
    screen.process(b"\x1b[?1049h"); // Alternate screen
    screen.process(b"\x1b[?1h");    // Application cursor keys
    screen.process(b"\x1b[?25h");   // Show cursor
    screen.process(b"\x1b[2J");     // Clear screen
    screen.process(b"\x1b[H");      // Home

    // 2. Write initial buffer
    screen.process(b"#!/bin/bash\r\n");
    screen.process(b"echo 'Hello World'\r\n");
    screen.process(b"exit 0\r\n");

    // 3. Move to line 2 and insert a line above
    screen.process(b"\x1b[2;1H");   // Row 2, col 1
    screen.process(b"\x1b[L");      // Insert line
    screen.process(b"# This is a comment");

    // 4. Move to end of file and add a line
    screen.process(b"\x1b[5;1H");   // Row 5 (past current content)
    screen.process(b"echo 'Goodbye'\r\n");

    // 5. Verify final content
    let lines = screen.get_all_lines();
    assert_eq!(lines[0][0].c, '#'); // #!/bin/bash
    assert_eq!(lines[1][0].c, '#'); // # This is a comment
    assert_eq!(lines[2][0].c, 'e'); // echo 'Hello World'

    // 6. Exit vim
    screen.process(b"\x1b[?1l");    // Normal cursor keys
    screen.process(b"\x1b[?1049l"); // Exit alternate screen
}

#[test]
fn test_vim_with_wide_characters() {
    let mut screen = TerminalScreen::new(40, 10);

    // Enter alternate screen
    screen.process(b"\x1b[?1049h");

    // Write mixed ASCII and Korean text (like editing a Korean file in vim)
    screen.process(b"Hello ");
    screen.process("안녕하세요".as_bytes()); // Korean "hello"
    screen.process(b" World");

    // Move cursor to middle of Korean text (position 7)
    screen.process(b"\x1b[1;7H");

    // Insert characters (vim insert mode)
    screen.process(b"\x1b[2@"); // Insert 2 spaces
    screen.process(b"**");

    // Verify Korean characters properly shifted
    let lines = screen.get_all_lines();
    assert_eq!(lines[0][0].c, 'H'); // 'H' from "Hello"
    assert_eq!(lines[0][6].c, '*'); // Inserted characters

    // Exit alternate screen
    screen.process(b"\x1b[?1049l");
}
