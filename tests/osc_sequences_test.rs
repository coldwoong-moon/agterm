// Integration test for OSC sequence handling

use agterm::terminal::screen::TerminalScreen;

#[test]
fn test_osc_window_title() {
    let mut screen = TerminalScreen::new(80, 24);

    // OSC 2 ; title ST (Set window title)
    let osc_sequence = b"\x1b]2;Test Window Title\x1b\\";
    screen.process(osc_sequence);

    assert_eq!(screen.window_title(), Some("Test Window Title"));
}

#[test]
fn test_osc_icon_name() {
    let mut screen = TerminalScreen::new(80, 24);

    // OSC 1 ; name ST (Set icon name)
    let osc_sequence = b"\x1b]1;Test Icon\x1b\\";
    screen.process(osc_sequence);

    assert_eq!(screen.icon_name(), Some("Test Icon"));
}

#[test]
fn test_osc_both_title_and_icon() {
    let mut screen = TerminalScreen::new(80, 24);

    // OSC 0 ; title ST (Set both icon name and window title)
    let osc_sequence = b"\x1b]0;Test Title and Icon\x1b\\";
    screen.process(osc_sequence);

    assert_eq!(screen.window_title(), Some("Test Title and Icon"));
    assert_eq!(screen.icon_name(), Some("Test Title and Icon"));
}

#[test]
fn test_osc_cwd() {
    let mut screen = TerminalScreen::new(80, 24);

    // OSC 7 ; file://hostname/path ST (Set current working directory)
    let osc_sequence = b"\x1b]7;file://localhost/home/user/project\x1b\\";
    screen.process(osc_sequence);

    assert_eq!(screen.cwd_from_shell(), Some("/home/user/project"));
}

#[test]
fn test_osc_clipboard_request() {
    let mut screen = TerminalScreen::new(80, 24);

    // OSC 52 ; c ; base64-data ST (Clipboard operations)
    let osc_sequence = b"\x1b]52;c;SGVsbG8gV29ybGQ=\x1b\\";
    screen.process(osc_sequence);

    assert_eq!(screen.clipboard_request(), Some("SGVsbG8gV29ybGQ="));

    // Test clearing clipboard request
    screen.clear_clipboard_request();
    assert_eq!(screen.clipboard_request(), None);
}

#[test]
fn test_osc_bell_terminated() {
    let mut screen = TerminalScreen::new(80, 24);

    // OSC 2 ; title BEL (Set window title with BEL terminator)
    let osc_sequence = b"\x1b]2;Bell Terminated\x07";
    screen.process(osc_sequence);

    assert_eq!(screen.window_title(), Some("Bell Terminated"));
}

#[test]
fn test_osc_file_uri_parsing() {
    let mut screen = TerminalScreen::new(80, 24);

    // Test file:///absolute/path
    screen.process(b"\x1b]7;file:///absolute/path\x1b\\");
    assert_eq!(screen.cwd_from_shell(), Some("/absolute/path"));

    // Test file://hostname/path/to/dir
    screen.process(b"\x1b]7;file://hostname/path/to/dir\x1b\\");
    assert_eq!(screen.cwd_from_shell(), Some("/path/to/dir"));

    // Test file:/single/slash
    screen.process(b"\x1b]7;file:/single/slash\x1b\\");
    assert_eq!(screen.cwd_from_shell(), Some("/single/slash"));
}
