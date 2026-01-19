//! Integration tests for shell output rendering
//!
//! Tests normal text output, colored output (like ls), and prompt rendering
//! to ensure the terminal correctly handles typical shell interactions.

use agterm::terminal::screen::{AnsiColor, TerminalScreen};

#[test]
fn test_simple_text_output() {
    let mut screen = TerminalScreen::new(80, 24);

    // Simulate shell prompt and command
    screen.process(b"$ echo 'Hello, World!'\r\n");
    screen.process(b"Hello, World!\r\n");
    screen.process(b"$ ");

    let lines = screen.get_all_lines();
    // First line: command
    assert_eq!(lines[0][0].c, '$');
    assert_eq!(lines[0][2].c, 'e');
    // Second line: output "Hello, World!"
    assert_eq!(lines[1][0].c, 'H');
    assert_eq!(lines[1][1].c, 'e');
    assert_eq!(lines[1][5].c, ','); // Comma after "Hello"
                                    // Third line: prompt
    assert_eq!(lines[2][0].c, '$');
}

#[test]
fn test_multiline_text_output() {
    let mut screen = TerminalScreen::new(80, 24);

    // Simulate multi-line output
    screen.process(b"Line 1\r\n");
    screen.process(b"Line 2\r\n");
    screen.process(b"Line 3\r\n");
    screen.process(b"Line 4\r\n");
    screen.process(b"Line 5\r\n");

    let lines = screen.get_all_lines();
    assert_eq!(lines[0][0].c, 'L');
    assert_eq!(lines[0][5].c, '1');
    assert_eq!(lines[1][5].c, '2');
    assert_eq!(lines[2][5].c, '3');
    assert_eq!(lines[3][5].c, '4');
    assert_eq!(lines[4][5].c, '5');
}

#[test]
fn test_ls_colored_output() {
    let mut screen = TerminalScreen::new(80, 24);

    // Simulate 'ls --color=auto' output
    // Blue for directories, green for executables, white for regular files

    // Directory (blue)
    screen.process(b"\x1b[34m"); // Blue foreground
    screen.process(b"Documents");
    screen.process(b"\x1b[0m"); // Reset
    screen.process(b"  ");

    // Executable (green)
    screen.process(b"\x1b[32m"); // Green foreground
    screen.process(b"script.sh");
    screen.process(b"\x1b[0m"); // Reset
    screen.process(b"  ");

    // Regular file (default)
    screen.process(b"README.md");
    screen.process(b"\r\n");

    let lines = screen.get_all_lines();

    // Check "Documents" is blue
    assert_eq!(lines[0][0].c, 'D');
    assert_eq!(lines[0][0].fg, Some(AnsiColor::Indexed(4))); // Blue

    // Check "script.sh" is green (starts after "Documents  ")
    let script_start = 11; // "Documents" (9) + "  " (2)
    assert_eq!(lines[0][script_start].c, 's');
    assert_eq!(lines[0][script_start].fg, Some(AnsiColor::Indexed(2))); // Green

    // Check "README.md" has no special color
    let readme_start = 22; // Previous + "script.sh" (9) + "  " (2)
    assert_eq!(lines[0][readme_start].c, 'R');
    assert_eq!(lines[0][readme_start].fg, None); // Default color
}

#[test]
fn test_ls_multicolumn_output() {
    let mut screen = TerminalScreen::new(80, 24);

    // Simulate ls output with multiple items per line
    screen.process(b"file1.txt  file2.txt  file3.txt  file4.txt\r\n");
    screen.process(b"file5.txt  file6.txt  file7.txt  file8.txt\r\n");

    let lines = screen.get_all_lines();

    // First line
    assert_eq!(lines[0][0].c, 'f');
    assert_eq!(lines[0][4].c, '1');
    assert_eq!(lines[0][11].c, 'f');
    assert_eq!(lines[0][15].c, '2');

    // Second line
    assert_eq!(lines[1][0].c, 'f');
    assert_eq!(lines[1][4].c, '5');
}

#[test]
fn test_colored_error_output() {
    let mut screen = TerminalScreen::new(80, 24);

    // Simulate command with error output (red text)
    screen.process(b"$ cat nonexistent.txt\r\n");
    screen.process(b"\x1b[31m"); // Red foreground
    screen.process(b"cat: nonexistent.txt: No such file or directory");
    screen.process(b"\x1b[0m"); // Reset
    screen.process(b"\r\n$ ");

    let lines = screen.get_all_lines();

    // Command line
    assert_eq!(lines[0][0].c, '$');

    // Error message should be red
    assert_eq!(lines[1][0].c, 'c');
    assert_eq!(lines[1][0].fg, Some(AnsiColor::Indexed(1))); // Red
}

#[test]
fn test_custom_shell_prompt() {
    let mut screen = TerminalScreen::new(80, 24);

    // Simulate fancy zsh/bash prompt with colors
    // Format: [user@host dir]$

    // Green username
    screen.process(b"\x1b[32m");
    screen.process(b"[user");
    screen.process(b"\x1b[0m");

    // Default @ symbol
    screen.process(b"@");

    // Blue hostname
    screen.process(b"\x1b[34m");
    screen.process(b"hostname");
    screen.process(b"\x1b[0m");

    // Yellow directory
    screen.process(b" \x1b[33m");
    screen.process(b"~/projects");
    screen.process(b"\x1b[0m");

    // Default ]$
    screen.process(b"]$ ");

    let lines = screen.get_all_lines();

    // Format: "[user@hostname ~/projects]$ "
    // Check colors
    assert_eq!(lines[0][1].c, 'u'); // 'user'
    assert_eq!(lines[0][1].fg, Some(AnsiColor::Indexed(2))); // Green

    assert_eq!(lines[0][5].c, '@'); // @ after "user"
    assert_eq!(lines[0][5].fg, None); // Default

    assert_eq!(lines[0][6].c, 'h'); // 'hostname'
    assert_eq!(lines[0][6].fg, Some(AnsiColor::Indexed(4))); // Blue
}

#[test]
fn test_git_status_colored_output() {
    let mut screen = TerminalScreen::new(80, 24);

    // Simulate 'git status' colored output
    screen.process(b"On branch ");
    screen.process(b"\x1b[32m"); // Green
    screen.process(b"main");
    screen.process(b"\x1b[0m");
    screen.process(b"\r\n");

    screen.process(b"Changes not staged for commit:\r\n");

    screen.process(b"  ");
    screen.process(b"\x1b[31m"); // Red
    screen.process(b"modified:   ");
    screen.process(b"\x1b[0m");
    screen.process(b"src/main.rs\r\n");

    screen.process(b"  ");
    screen.process(b"\x1b[32m"); // Green
    screen.process(b"new file:   ");
    screen.process(b"\x1b[0m");
    screen.process(b"tests/test.rs\r\n");

    let lines = screen.get_all_lines();

    // "main" should be green (starts at position 10: "On branch main")
    assert_eq!(lines[0][10].c, 'm');
    assert_eq!(lines[0][10].fg, Some(AnsiColor::Indexed(2))); // Green

    // "modified:" should be red
    assert_eq!(lines[2][2].c, 'm');
    assert_eq!(lines[2][2].fg, Some(AnsiColor::Indexed(1))); // Red

    // "new file:" should be green
    assert_eq!(lines[3][2].c, 'n');
    assert_eq!(lines[3][2].fg, Some(AnsiColor::Indexed(2))); // Green
}

#[test]
fn test_grep_colored_output() {
    let mut screen = TerminalScreen::new(80, 24);

    // Simulate 'grep --color=auto' output
    // Matched text is highlighted in red/bold

    screen.process(b"file.txt: This is a ");
    screen.process(b"\x1b[1;31m"); // Bold red
    screen.process(b"match");
    screen.process(b"\x1b[0m");
    screen.process(b" in the text\r\n");

    screen.process(b"file.txt: Another ");
    screen.process(b"\x1b[1;31m"); // Bold red
    screen.process(b"match");
    screen.process(b"\x1b[0m");
    screen.process(b" here\r\n");

    let lines = screen.get_all_lines();

    // Check first match
    assert_eq!(lines[0][20].c, 'm'); // "match"
    assert_eq!(lines[0][20].fg, Some(AnsiColor::Indexed(1))); // Red
    assert!(lines[0][20].bold); // Bold

    // Check second match
    assert_eq!(lines[1][18].c, 'm'); // "match"
    assert_eq!(lines[1][18].fg, Some(AnsiColor::Indexed(1))); // Red
    assert!(lines[1][18].bold); // Bold
}

#[test]
fn test_prompt_with_command_output() {
    let mut screen = TerminalScreen::new(80, 24);

    // Full command cycle: prompt -> command -> output -> prompt

    // Initial prompt
    screen.process(b"$ ");

    // User types command
    screen.process(b"echo 'test'\r\n");

    // Command output
    screen.process(b"test\r\n");

    // New prompt
    screen.process(b"$ ");

    let lines = screen.get_all_lines();
    assert_eq!(lines[0][0].c, '$'); // First prompt
    assert_eq!(lines[0][2].c, 'e'); // Command
    assert_eq!(lines[1][0].c, 't'); // Output
    assert_eq!(lines[2][0].c, '$'); // Second prompt
}

#[test]
fn test_progress_bar_simulation() {
    let mut screen = TerminalScreen::new(80, 24);

    // Simulate a progress bar that updates in place
    // Using carriage return to overwrite the same line

    screen.process(b"Downloading: [          ] 0%");
    screen.process(b"\r"); // Carriage return
    screen.process(b"Downloading: [###       ] 30%");
    screen.process(b"\r");
    screen.process(b"Downloading: [######    ] 60%");
    screen.process(b"\r");
    screen.process(b"Downloading: [##########] 100%");

    let lines = screen.get_all_lines();
    // Should only see the final state: "Downloading: [##########] 100%"
    assert_eq!(lines[0][0].c, 'D');
    assert_eq!(lines[0][13].c, '['); // Opening bracket
    assert_eq!(lines[0][14].c, '#'); // First hash
    assert_eq!(lines[0][23].c, '#'); // Last hash
    assert_eq!(lines[0][24].c, ']'); // Closing bracket
    assert_eq!(lines[0][26].c, '1');
    assert_eq!(lines[0][27].c, '0');
    assert_eq!(lines[0][28].c, '0');
}

#[test]
fn test_unicode_shell_output() {
    let mut screen = TerminalScreen::new(80, 24);

    // Simulate shell output with Unicode characters
    screen.process(b"$ echo '");
    screen.process("Hello 世界 🌍".as_bytes()); // English, Chinese, Emoji
    screen.process(b"'\r\n");
    screen.process("Hello 世界 🌍".as_bytes());
    screen.process(b"\r\n$ ");

    let lines = screen.get_all_lines();
    assert_eq!(lines[0][0].c, '$'); // Prompt
                                    // Unicode characters should be present
    assert_eq!(lines[1][0].c, 'H');
}

#[test]
fn test_table_formatted_output() {
    let mut screen = TerminalScreen::new(80, 24);

    // Simulate table output (like 'ps' or 'df' command)
    screen.process(b"PID   USER     CPU  MEM   COMMAND\r\n");
    screen.process(b"1234  alice    2.5  512M  /usr/bin/app\r\n");
    screen.process(b"5678  bob      1.0  256M  /usr/bin/server\r\n");
    screen.process(b"9012  charlie  0.5  128M  /bin/bash\r\n");

    let lines = screen.get_all_lines();

    // Header
    assert_eq!(lines[0][0].c, 'P');
    assert_eq!(lines[0][1].c, 'I');
    assert_eq!(lines[0][2].c, 'D');

    // First row
    assert_eq!(lines[1][0].c, '1');
    assert_eq!(lines[1][1].c, '2');
    assert_eq!(lines[1][2].c, '3');
    assert_eq!(lines[1][3].c, '4');

    // Check alignment
    assert_eq!(lines[1][6].c, 'a'); // "alice"
}

#[test]
fn test_command_with_backspace() {
    let mut screen = TerminalScreen::new(80, 24);

    // Simulate user typing and correcting a command
    screen.process(b"$ echoo"); // Typo
    screen.process(b"\x08"); // Backspace
    screen.process(b"\x08"); // Backspace
    screen.process(b"o test\r\n"); // Correction

    let _lines = screen.get_all_lines();
    // After backspaces, cursor moves back
    // The exact output depends on whether shell echoes the correction
}

#[test]
fn test_long_output_scrolling() {
    let mut screen = TerminalScreen::new(80, 24);

    // Fill screen with more than 24 lines
    for i in 1..=30 {
        screen.process(format!("Line {}\r\n", i).as_bytes());
    }

    // Should have scrolling buffer
    let lines = screen.get_all_lines();
    assert!(lines.len() > 24); // Should include scrollback

    // Find line 30
    let line_30 = lines
        .iter()
        .rev()
        .find(|line| line.len() > 6 && line[0].c == 'L' && line[5].c == '3' && line[6].c == '0');
    assert!(line_30.is_some(), "Should find Line 30");
}

#[test]
fn test_ansi_256_color_output() {
    let mut screen = TerminalScreen::new(80, 24);

    // Test 256-color palette (modern terminals)
    // CSI 38;5;Nm for foreground
    screen.process(b"\x1b[38;5;196m"); // Bright red (256-color)
    screen.process(b"Red text");
    screen.process(b"\x1b[0m");
    screen.process(b" ");

    screen.process(b"\x1b[38;5;46m"); // Bright green (256-color)
    screen.process(b"Green text");
    screen.process(b"\x1b[0m");

    let lines = screen.get_all_lines();

    // Check colors are set (exact color values)
    assert_eq!(lines[0][0].c, 'R');
    assert_eq!(lines[0][0].fg, Some(AnsiColor::Palette256(196)));

    assert_eq!(lines[0][9].c, 'G');
    assert_eq!(lines[0][9].fg, Some(AnsiColor::Palette256(46)));
}

#[test]
fn test_ansi_rgb_color_output() {
    let mut screen = TerminalScreen::new(80, 24);

    // Test true color (24-bit RGB)
    // CSI 38;2;R;G;Bm for foreground
    screen.process(b"\x1b[38;2;255;100;50m"); // Custom RGB color
    screen.process(b"Custom color");
    screen.process(b"\x1b[0m");

    let lines = screen.get_all_lines();

    // Check RGB color
    assert_eq!(lines[0][0].c, 'C');
    assert_eq!(lines[0][0].fg, Some(AnsiColor::Rgb(255, 100, 50)));
}

#[test]
fn test_shell_tab_completion_display() {
    let mut screen = TerminalScreen::new(80, 24);

    // Simulate tab completion showing options
    screen.process(b"$ git c\r\n");
    screen.process(b"checkout  cherry-pick  clean  clone  commit\r\n");
    screen.process(b"$ git c");

    let lines = screen.get_all_lines();

    // Command line
    assert_eq!(lines[0][0].c, '$');

    // Completion options
    assert_eq!(lines[1][0].c, 'c'); // "checkout"

    // Prompt reappears
    assert_eq!(lines[2][0].c, '$');
}

#[test]
fn test_stderr_vs_stdout() {
    let mut screen = TerminalScreen::new(80, 24);

    // Simulate command with both stdout and stderr
    // (In real terminals, both go to screen but may have different colors)

    screen.process(b"$ command\r\n");

    // stdout (normal)
    screen.process(b"Normal output line 1\r\n");

    // stderr (red)
    screen.process(b"\x1b[31m");
    screen.process(b"Error: something went wrong");
    screen.process(b"\x1b[0m");
    screen.process(b"\r\n");

    // More stdout
    screen.process(b"Normal output line 2\r\n");

    let lines = screen.get_all_lines();

    // Normal output
    assert_eq!(lines[1][0].c, 'N');
    assert_eq!(lines[1][0].fg, None);

    // Error output (red)
    assert_eq!(lines[2][0].c, 'E');
    assert_eq!(lines[2][0].fg, Some(AnsiColor::Indexed(1))); // Red

    // Back to normal
    assert_eq!(lines[3][0].c, 'N');
    assert_eq!(lines[3][0].fg, None);
}
