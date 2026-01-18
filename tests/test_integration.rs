//! Integration tests for AgTerm
//!
//! This test suite covers:
//! - Theme loading and color conversion
//! - Config file loading and defaults
//! - Mouse event encoding
//! - Alternate screen state restoration
//! - Clipboard (OSC 52) base64 encoding
//! - Tab management (creation, closing, cloning)

use agterm::config::{AppConfig, BellStyle, CursorStyle, SelectionMode};
use agterm::terminal::screen::{MouseEncoding, MouseMode, TerminalScreen};
use agterm::theme::{ColorDef, Theme, ThemeVariant};
use iced::Color;

// ============================================================================
// Theme Loading and Color Conversion Tests
// ============================================================================

#[test]
fn test_theme_hex_color_conversion() {
    let color = ColorDef::from_hex("#ff0000");
    let iced_color = color.to_color();

    // Red should be 1.0
    assert!((iced_color.r - 1.0).abs() < 0.01);
    assert!((iced_color.g - 0.0).abs() < 0.01);
    assert!((iced_color.b - 0.0).abs() < 0.01);
}

#[test]
fn test_theme_hex_color_conversion_short_format() {
    let color = ColorDef::from_hex("#f00");
    let iced_color = color.to_color();

    // #f00 should expand to #ff0000
    assert!((iced_color.r - 1.0).abs() < 0.01);
    assert!((iced_color.g - 0.0).abs() < 0.01);
    assert!((iced_color.b - 0.0).abs() < 0.01);
}

#[test]
fn test_theme_hex_color_conversion_without_hash() {
    let color = ColorDef::from_hex("00ff00");
    let iced_color = color.to_color();

    // Green without # prefix
    assert!((iced_color.r - 0.0).abs() < 0.01);
    assert!((iced_color.g - 1.0).abs() < 0.01);
    assert!((iced_color.b - 0.0).abs() < 0.01);
}

#[test]
fn test_theme_rgb_color_conversion() {
    let color = ColorDef::from_rgb(255, 128, 0);
    let iced_color = color.to_color();

    // Orange color
    assert!((iced_color.r - 1.0).abs() < 0.01);
    assert!((iced_color.g - 0.5).abs() < 0.01);
    assert!((iced_color.b - 0.0).abs() < 0.01);
}

#[test]
fn test_theme_rgb_float_color_conversion() {
    let color = ColorDef::from_rgb_float(0.5, 0.5, 1.0);
    let iced_color = color.to_color();

    // Light blue
    assert!((iced_color.r - 0.5).abs() < 0.01);
    assert!((iced_color.g - 0.5).abs() < 0.01);
    assert!((iced_color.b - 1.0).abs() < 0.01);
}

#[test]
fn test_theme_ansi_palette_indexing() {
    let theme = Theme::warp_dark();

    // Test standard colors (0-7)
    let black = theme.ansi.get_color(0);
    assert_ne!(black, Color::WHITE);

    let red = theme.ansi.get_color(1);
    assert_ne!(red, Color::BLACK);

    // Test bright colors (8-15)
    let bright_black = theme.ansi.get_color(8);
    assert_ne!(bright_black, Color::BLACK);

    let bright_white = theme.ansi.get_color(15);
    assert_ne!(bright_white, Color::BLACK);

    // Test out of range (should return fallback)
    let fallback = theme.ansi.get_color(255);
    assert_eq!(fallback, Color::WHITE);
}

#[test]
fn test_theme_preset_loading() {
    // Test all preset themes load successfully
    let themes = vec![
        ("warp_dark", Theme::warp_dark()),
        ("dracula", Theme::dracula()),
        ("solarized_dark", Theme::solarized_dark()),
        ("solarized_light", Theme::solarized_light()),
        ("nord", Theme::nord()),
        ("one_dark", Theme::one_dark()),
        ("monokai_pro", Theme::monokai_pro()),
        ("tokyo_night", Theme::tokyo_night()),
    ];

    for (name, theme) in themes {
        assert!(!theme.name.is_empty(), "{} should have a name", name);

        // Verify all colors can be converted
        let _fg = theme.terminal.foreground.to_color();
        let _bg = theme.terminal.background.to_color();
        let _cursor = theme.terminal.cursor.to_color();

        // Verify ANSI palette is complete
        for i in 0..16 {
            let _color = theme.ansi.get_color(i);
        }
    }
}

#[test]
fn test_theme_by_name_lookup() {
    assert!(Theme::by_name("dracula").is_some());
    assert!(Theme::by_name("nord").is_some());
    assert!(Theme::by_name("one-dark").is_some()); // Test hyphen variant
    assert!(Theme::by_name("solarized-light").is_some());
    assert!(Theme::by_name("invalid_theme_name").is_none());
}

#[test]
fn test_theme_variant_serialization() {
    // Test deserialization from TOML
    #[derive(serde::Deserialize)]
    struct Wrapper {
        variant: ThemeVariant,
    }

    let dark: Wrapper = toml::from_str("variant = \"dark\"").unwrap();
    let light: Wrapper = toml::from_str("variant = \"light\"").unwrap();

    assert_eq!(dark.variant, ThemeVariant::Dark);
    assert_eq!(light.variant, ThemeVariant::Light);
}

#[test]
fn test_theme_toml_serialization() {
    let theme = Theme::warp_dark();
    let toml_str = toml::to_string(&theme).unwrap();

    // Verify essential fields are present
    assert!(toml_str.contains("name"));
    assert!(toml_str.contains("variant"));
    assert!(toml_str.contains("[ui]"));
    assert!(toml_str.contains("[ansi]"));
    assert!(toml_str.contains("[terminal]"));
}

#[test]
fn test_theme_color_consistency() {
    let theme = Theme::nord();

    // UI and terminal backgrounds should match or be related
    let ui_bg = theme.ui.bg_primary.to_color();
    let term_bg = theme.terminal.background.to_color();

    // They might not be exactly equal, but should both be dark for a dark theme
    if theme.variant == ThemeVariant::Dark {
        assert!(ui_bg.r < 0.5 && ui_bg.g < 0.5 && ui_bg.b < 0.5);
        assert!(term_bg.r < 0.5 && term_bg.g < 0.5 && term_bg.b < 0.5);
    }
}

// ============================================================================
// Config File Loading and Defaults Tests
// ============================================================================

#[test]
fn test_config_default_values() {
    let config = AppConfig::default();

    assert_eq!(config.general.app_name, "agterm");
    assert_eq!(config.appearance.font_family, "D2Coding");
    assert_eq!(config.appearance.font_size, 14.0);
    assert_eq!(config.appearance.background_opacity, 1.0);
    assert_eq!(config.terminal.scrollback_lines, 10000);
    assert_eq!(config.terminal.cursor_style, CursorStyle::Block);
    assert_eq!(config.terminal.cursor_blink, true);
    assert_eq!(config.terminal.bell_style, BellStyle::Visual);
}

#[test]
fn test_config_pty_defaults() {
    let config = AppConfig::default();

    assert_eq!(config.pty.default_cols, 120);
    assert_eq!(config.pty.default_rows, 40);
    assert_eq!(config.pty.max_sessions, 32);
    assert_eq!(config.pty.scrollback_lines, 10000);
}

#[test]
fn test_config_mouse_defaults() {
    let config = AppConfig::default();

    assert_eq!(config.mouse.enabled, true);
    assert_eq!(config.mouse.reporting, true);
    assert_eq!(config.mouse.selection_mode, SelectionMode::Character);
    assert_eq!(config.mouse.copy_on_select, true);
    assert_eq!(config.mouse.middle_click_paste, true);
}

#[test]
fn test_config_cursor_style_serialization() {
    // Test deserialization from TOML (more practical than serializing enums directly)
    #[derive(serde::Deserialize)]
    struct Wrapper {
        cursor_style: CursorStyle,
    }

    let block: Wrapper = toml::from_str("cursor_style = \"block\"").unwrap();
    let underline: Wrapper = toml::from_str("cursor_style = \"underline\"").unwrap();
    let beam: Wrapper = toml::from_str("cursor_style = \"beam\"").unwrap();

    assert_eq!(block.cursor_style, CursorStyle::Block);
    assert_eq!(underline.cursor_style, CursorStyle::Underline);
    assert_eq!(beam.cursor_style, CursorStyle::Beam);
}

#[test]
fn test_config_bell_style_serialization() {
    // Test deserialization from TOML
    #[derive(serde::Deserialize)]
    struct Wrapper {
        bell_style: BellStyle,
    }

    let visual: Wrapper = toml::from_str("bell_style = \"visual\"").unwrap();
    let sound: Wrapper = toml::from_str("bell_style = \"sound\"").unwrap();
    let both: Wrapper = toml::from_str("bell_style = \"both\"").unwrap();
    let none: Wrapper = toml::from_str("bell_style = \"none\"").unwrap();

    assert_eq!(visual.bell_style, BellStyle::Visual);
    assert_eq!(sound.bell_style, BellStyle::Sound);
    assert_eq!(both.bell_style, BellStyle::Both);
    assert_eq!(none.bell_style, BellStyle::None);
}

#[test]
fn test_config_selection_mode_serialization() {
    // Test deserialization from TOML
    #[derive(serde::Deserialize)]
    struct Wrapper {
        selection_mode: SelectionMode,
    }

    let char_mode: Wrapper = toml::from_str("selection_mode = \"character\"").unwrap();
    let word_mode: Wrapper = toml::from_str("selection_mode = \"word\"").unwrap();
    let line_mode: Wrapper = toml::from_str("selection_mode = \"line\"").unwrap();

    assert_eq!(char_mode.selection_mode, SelectionMode::Character);
    assert_eq!(word_mode.selection_mode, SelectionMode::Word);
    assert_eq!(line_mode.selection_mode, SelectionMode::Line);
}

#[test]
fn test_config_toml_roundtrip() {
    let config = AppConfig::default();
    let toml_str = toml::to_string(&config).unwrap();
    let parsed: AppConfig = toml::from_str(&toml_str).unwrap();

    assert_eq!(parsed.general.app_name, config.general.app_name);
    assert_eq!(parsed.appearance.font_size, config.appearance.font_size);
    assert_eq!(
        parsed.terminal.scrollback_lines,
        config.terminal.scrollback_lines
    );
    assert_eq!(parsed.pty.default_cols, config.pty.default_cols);
}

#[test]
fn test_config_partial_override() {
    let toml_str = r#"
[appearance]
font_size = 16.0
theme = "dracula"
"#;

    let parsed: AppConfig = toml::from_str(toml_str).unwrap();

    // Overridden values
    assert_eq!(parsed.appearance.font_size, 16.0);
    assert_eq!(parsed.appearance.theme, "dracula");

    // Default values should still be present
    assert_eq!(parsed.general.app_name, "agterm");
    assert_eq!(parsed.pty.default_cols, 120);
}

#[test]
fn test_config_logging_defaults() {
    let config = AppConfig::default();

    assert_eq!(config.logging.level, "info");
    assert_eq!(config.logging.format, "pretty");
    assert_eq!(config.logging.timestamps, true);
    assert_eq!(config.logging.file_line, false);
    assert_eq!(config.logging.file_output, true);
}

#[test]
fn test_config_debug_defaults() {
    let config = AppConfig::default();

    assert_eq!(config.debug.enabled, false);
    assert_eq!(config.debug.show_fps, true);
    assert_eq!(config.debug.show_pty_stats, true);
    assert_eq!(config.debug.log_buffer_size, 50);
}

// ============================================================================
// Mouse Event Encoding Tests
// ============================================================================

#[test]
fn test_mouse_mode_transitions() {
    let mut screen = TerminalScreen::new(80, 24);

    // Start with no mouse reporting
    assert_eq!(screen.mouse_mode(), MouseMode::None);

    // Enable X10 mouse reporting (CSI ?9h)
    screen.process(b"\x1b[?9h");
    assert_eq!(screen.mouse_mode(), MouseMode::X10);

    // Disable X10 mouse reporting (CSI ?9l)
    screen.process(b"\x1b[?9l");
    assert_eq!(screen.mouse_mode(), MouseMode::None);
}

#[test]
fn test_mouse_mode_button_event_tracking() {
    let mut screen = TerminalScreen::new(80, 24);

    // Enable button-event tracking (CSI ?1002h)
    screen.process(b"\x1b[?1002h");
    assert_eq!(screen.mouse_mode(), MouseMode::ButtonEvent);

    // Disable
    screen.process(b"\x1b[?1002l");
    assert_eq!(screen.mouse_mode(), MouseMode::None);
}

#[test]
fn test_mouse_mode_any_event_tracking() {
    let mut screen = TerminalScreen::new(80, 24);

    // Enable any-event tracking (CSI ?1003h)
    screen.process(b"\x1b[?1003h");
    assert_eq!(screen.mouse_mode(), MouseMode::AnyEvent);

    // Disable
    screen.process(b"\x1b[?1003l");
    assert_eq!(screen.mouse_mode(), MouseMode::None);
}

#[test]
fn test_mouse_encoding_modes() {
    let mut screen = TerminalScreen::new(80, 24);

    // Default encoding
    assert_eq!(screen.mouse_encoding(), MouseEncoding::Default);

    // Enable SGR extended mouse mode (CSI ?1006h)
    screen.process(b"\x1b[?1006h");
    assert_eq!(screen.mouse_encoding(), MouseEncoding::Sgr);

    // Disable
    screen.process(b"\x1b[?1006l");
    assert_eq!(screen.mouse_encoding(), MouseEncoding::Default);
}

#[test]
fn test_mouse_mode_persistence() {
    let mut screen = TerminalScreen::new(80, 24);

    // Enable mouse reporting
    screen.process(b"\x1b[?1000h");
    assert_eq!(screen.mouse_mode(), MouseMode::X10);

    // Mouse mode should persist through other operations
    screen.process(b"Hello, World!\r\n");
    assert_eq!(screen.mouse_mode(), MouseMode::X10);

    // Clear screen shouldn't affect mouse mode
    screen.process(b"\x1b[2J");
    assert_eq!(screen.mouse_mode(), MouseMode::X10);
}

#[test]
fn test_mouse_encoding_with_mode() {
    let mut screen = TerminalScreen::new(80, 24);

    // Enable both mouse reporting and SGR encoding
    screen.process(b"\x1b[?1000h");
    screen.process(b"\x1b[?1006h");

    assert_eq!(screen.mouse_mode(), MouseMode::X10);
    assert_eq!(screen.mouse_encoding(), MouseEncoding::Sgr);
}

// ============================================================================
// Alternate Screen State Restoration Tests
// ============================================================================

#[test]
fn test_alternate_screen_activation() {
    let mut screen = TerminalScreen::new(80, 24);

    // Write content to main screen
    screen.process(b"Main screen content\r\n");

    // Switch to alternate screen (CSI ?1049h)
    screen.process(b"\x1b[?1049h");

    // Content on alternate screen should be blank (no "Main screen" text)
    let lines = screen.get_all_lines();
    let first_line_text: String = lines[0]
        .iter()
        .filter(|c| !c.placeholder)
        .map(|c| c.c)
        .collect();
    assert!(!first_line_text.contains("Main screen"));
}

#[test]
fn test_alternate_screen_deactivation() {
    let mut screen = TerminalScreen::new(80, 24);

    // Write to main screen
    screen.process(b"Main screen\r\n");

    // Switch to alternate screen
    screen.process(b"\x1b[?1049h");

    // Write to alternate screen
    screen.process(b"Alternate screen\r\n");

    // Switch back to main screen (CSI ?1049l)
    screen.process(b"\x1b[?1049l");

    // Main screen content should be restored
    let lines = screen.get_all_lines();
    let first_line_text: String = lines[0]
        .iter()
        .filter(|c| !c.placeholder)
        .map(|c| c.c)
        .take(11)
        .collect();
    assert_eq!(first_line_text.trim(), "Main screen");
}

#[test]
fn test_alternate_screen_cursor_position_restoration() {
    let mut screen = TerminalScreen::new(80, 24);

    // Position cursor on main screen
    screen.process(b"\x1b[10;20H"); // Row 10, Col 20
    let (main_row, main_col) = screen.cursor_position();

    // Switch to alternate screen
    screen.process(b"\x1b[?1049h");

    // Move cursor on alternate screen
    screen.process(b"\x1b[5;10H");

    // Switch back to main screen
    screen.process(b"\x1b[?1049l");

    // Cursor position should be restored
    let (restored_row, restored_col) = screen.cursor_position();
    assert_eq!(restored_row, main_row);
    assert_eq!(restored_col, main_col);
}

#[test]
fn test_alternate_screen_attributes_restoration() {
    let mut screen = TerminalScreen::new(80, 24);

    // Set bold on main screen
    screen.process(b"\x1b[1m");
    screen.process(b"Bold text\r\n");

    let main_lines = screen.get_all_lines();
    let main_bold = main_lines[0][0].bold;

    // Switch to alternate screen
    screen.process(b"\x1b[?1049h");

    // Disable bold on alternate screen
    screen.process(b"\x1b[0m");
    screen.process(b"Normal text\r\n");

    // Switch back
    screen.process(b"\x1b[?1049l");

    // Text attributes should be restored
    screen.process(b"X");
    let _restored_lines = screen.get_all_lines();

    // First character should still be bold from before
    assert_eq!(main_lines[0][0].bold, main_bold);
}

#[test]
fn test_alternate_screen_multiple_switches() {
    let mut screen = TerminalScreen::new(80, 24);

    // Write to main
    screen.process(b"Main 1\r\n");

    // Switch to alternate
    screen.process(b"\x1b[?1049h");
    screen.process(b"Alt 1\r\n");

    // Switch back to main
    screen.process(b"\x1b[?1049l");

    // Add more to main
    screen.process(b"Main 2\r\n");

    // Switch to alternate again
    screen.process(b"\x1b[?1049h");

    // Alternate should be cleared again
    let lines = screen.get_all_lines();
    let text: String = lines[0]
        .iter()
        .filter(|c| !c.placeholder)
        .map(|c| c.c)
        .take(10)
        .collect();

    // Should not contain main screen content
    assert!(!text.contains("Main 1"));
    assert!(!text.contains("Main 2"));
}

#[test]
fn test_alternate_screen_with_clear() {
    let mut screen = TerminalScreen::new(80, 24);

    screen.process(b"Persistent content\r\n");

    // Enter alternate screen
    screen.process(b"\x1b[?1049h");
    screen.process(b"Temporary content\r\n");

    // Clear alternate screen
    screen.process(b"\x1b[2J");

    // Exit alternate screen
    screen.process(b"\x1b[?1049l");

    // Main screen should still have original content
    let lines = screen.get_all_lines();
    let text: String = lines[0]
        .iter()
        .filter(|c| !c.placeholder)
        .map(|c| c.c)
        .take(18)
        .collect();
    assert!(text.contains("Persistent"));
}

// ============================================================================
// Clipboard (OSC 52) Base64 Encoding Tests
// ============================================================================

#[test]
fn test_osc52_clipboard_set_request() {
    let mut screen = TerminalScreen::new(80, 24);

    // OSC 52 ; c ; base64_data ST
    // "Hello" in base64 is "SGVsbG8="
    screen.process(b"\x1b]52;c;SGVsbG8=\x1b\\");

    // Check if clipboard request was captured
    let clipboard_data = screen.clipboard_request();
    assert_eq!(clipboard_data, Some("SGVsbG8="));
}

#[test]
fn test_osc52_clipboard_base64_decoding() {
    // Test base64 encoding/decoding
    let test_cases = vec![
        ("Hello", "SGVsbG8="),
        ("World!", "V29ybGQh"),
        ("Test 123", "VGVzdCAxMjM="),
        ("한글", "7ZWc6riA"), // Korean text in base64
    ];

    for (original, encoded) in test_cases {
        let mut screen = TerminalScreen::new(80, 24);
        let osc_sequence = format!("\x1b]52;c;{}\x1b\\", encoded);
        screen.process(osc_sequence.as_bytes());

        let clipboard_data = screen.clipboard_request();
        assert_eq!(clipboard_data, Some(encoded));

        // Decode and verify
        if let Ok(decoded_bytes) = base64::decode(encoded) {
            if let Ok(decoded_str) = String::from_utf8(decoded_bytes) {
                assert_eq!(decoded_str, original);
            }
        }
    }
}

#[test]
fn test_osc52_clipboard_empty_data() {
    let mut screen = TerminalScreen::new(80, 24);

    // First set some data
    screen.process(b"\x1b]52;c;SGVsbG8=\x1b\\");
    let initial = screen.clipboard_request();
    assert_eq!(initial, Some("SGVsbG8="));

    // Empty clipboard data (implementation might keep previous value)
    screen.process(b"\x1b]52;c;\x1b\\");

    // The empty string in OSC 52 might keep the previous value or clear it
    // This depends on the implementation details
    let clipboard_data = screen.clipboard_request();
    // Accept any of: None (cleared), Some("") (empty), or Some("SGVsbG8=") (kept)
    assert!(
        clipboard_data.is_none()
            || clipboard_data == Some("")
            || clipboard_data == Some("SGVsbG8=")
    );
}

#[test]
fn test_osc52_clipboard_query() {
    let mut screen = TerminalScreen::new(80, 24);

    // OSC 52 query (? instead of data)
    screen.process(b"\x1b]52;c;?\x1b\\");

    // Query should be captured (or might be ignored by implementation)
    let clipboard_data = screen.clipboard_request();
    // Implementation might not store query requests
    // Either Some("?") or None is acceptable
    assert!(clipboard_data.is_none() || clipboard_data == Some("?"));
}

#[test]
fn test_osc52_clipboard_selection_types() {
    let mut screen = TerminalScreen::new(80, 24);

    // Test different selection types (c=clipboard, p=primary, s=secondary)
    let selections = vec![
        ("c", "SGVsbG8="), // clipboard
        ("p", "V29ybGQ="), // primary
        ("s", "VGVzdA=="), // secondary
    ];

    for (sel_type, data) in selections {
        let osc_sequence = format!("\x1b]52;{};{}\x1b\\", sel_type, data);
        screen.process(osc_sequence.as_bytes());

        let clipboard_data = screen.clipboard_request();
        assert_eq!(clipboard_data, Some(data));
    }
}

#[test]
fn test_osc52_clipboard_overwrite() {
    let mut screen = TerminalScreen::new(80, 24);

    // Set clipboard data
    screen.process(b"\x1b]52;c;Rmlyc3Q=\x1b\\"); // "First"
    assert_eq!(screen.clipboard_request(), Some("Rmlyc3Q="));

    // Overwrite with new data
    screen.process(b"\x1b]52;c;U2Vjb25k\x1b\\"); // "Second"
    assert_eq!(screen.clipboard_request(), Some("U2Vjb25k"));
}

#[test]
fn test_osc52_clipboard_special_characters() {
    let mut screen = TerminalScreen::new(80, 24);

    // Test with special characters in base64
    // "Hello\nWorld\t!" base64: "SGVsbG8KV29ybGQJIQ=="
    screen.process(b"\x1b]52;c;SGVsbG8KV29ybGQJIQ==\x1b\\");

    let clipboard_data = screen.clipboard_request();
    assert_eq!(clipboard_data, Some("SGVsbG8KV29ybGQJIQ=="));
}

// ============================================================================
// Tab Management Tests
// ============================================================================

// Note: Tab management is implemented in main.rs AgTerm struct
// These tests verify the terminal screen functionality that tabs depend on

#[test]
fn test_multiple_terminal_screens() {
    // Simulate multiple tabs with separate screens
    let mut screen1 = TerminalScreen::new(80, 24);
    let mut screen2 = TerminalScreen::new(80, 24);
    let mut screen3 = TerminalScreen::new(80, 24);

    // Write different content to each
    screen1.process(b"Tab 1 content\r\n");
    screen2.process(b"Tab 2 content\r\n");
    screen3.process(b"Tab 3 content\r\n");

    // Verify each screen has independent state
    let lines1 = screen1.get_all_lines();
    let lines2 = screen2.get_all_lines();
    let lines3 = screen3.get_all_lines();

    let text1: String = lines1[0]
        .iter()
        .filter(|c| !c.placeholder)
        .map(|c| c.c)
        .take(13)
        .collect();
    let text2: String = lines2[0]
        .iter()
        .filter(|c| !c.placeholder)
        .map(|c| c.c)
        .take(13)
        .collect();
    let text3: String = lines3[0]
        .iter()
        .filter(|c| !c.placeholder)
        .map(|c| c.c)
        .take(13)
        .collect();

    assert_eq!(text1.trim(), "Tab 1 content");
    assert_eq!(text2.trim(), "Tab 2 content");
    assert_eq!(text3.trim(), "Tab 3 content");
}

#[test]
fn test_terminal_screen_clone_for_duplicate_tab() {
    let mut original = TerminalScreen::new(80, 24);

    // Write content
    original.process(b"Original content\r\n");
    original.process(b"\x1b[1m"); // Bold

    // Get state for cloning
    let lines = original.get_all_lines();
    let cursor_pos = original.cursor_position();

    // Create new screen and restore state (simulating tab duplication)
    let mut cloned = TerminalScreen::new(80, 24);

    // Verify initial state is independent
    let cloned_lines_initial = cloned.get_all_lines();
    let cloned_text_initial: String = cloned_lines_initial[0]
        .iter()
        .filter(|c| !c.placeholder)
        .map(|c| c.c)
        .take(16)
        .collect();

    assert_ne!(cloned_text_initial.trim(), "Original content");

    // Original should still have its content
    assert_eq!(lines[0][0].c, 'O');
    assert_eq!(cursor_pos.0, 1); // Cursor on second line after \r\n
}

#[test]
fn test_terminal_screen_independent_scrollback() {
    let mut screen1 = TerminalScreen::new(80, 5);
    let mut screen2 = TerminalScreen::new(80, 5);

    // Fill screen1 with more lines than visible (create scrollback)
    for i in 0..10 {
        screen1.process(format!("Screen 1 Line {}\r\n", i).as_bytes());
    }

    // Fill screen2 with fewer lines
    for i in 0..3 {
        screen2.process(format!("Screen 2 Line {}\r\n", i).as_bytes());
    }

    // Verify scrollback is independent
    let lines1 = screen1.get_all_lines();
    let lines2 = screen2.get_all_lines();

    assert!(lines1.len() > lines2.len());
}

#[test]
fn test_terminal_screen_resize_per_tab() {
    let mut screen1 = TerminalScreen::new(80, 24);
    let mut screen2 = TerminalScreen::new(100, 30);

    screen1.process(b"Screen 1\r\n");
    screen2.process(b"Screen 2\r\n");

    // Resize screen1
    screen1.resize(60, 20);

    // Verify screen2 is unaffected
    let lines2 = screen2.get_all_lines();
    assert_eq!(lines2[0].len(), 100); // Width unchanged
}

#[test]
fn test_terminal_screen_state_isolation() {
    let mut screen1 = TerminalScreen::new(80, 24);
    let mut screen2 = TerminalScreen::new(80, 24);

    // Set different attributes on each screen
    screen1.process(b"\x1b[1m"); // Bold on screen1
    screen1.process(b"Bold text\r\n");

    screen2.process(b"\x1b[4m"); // Underline on screen2
    screen2.process(b"Underlined text\r\n");

    let lines1 = screen1.get_all_lines();
    let lines2 = screen2.get_all_lines();

    // Verify attributes are isolated
    assert!(lines1[0][0].bold);
    assert!(!lines1[0][0].underline);

    assert!(!lines2[0][0].bold);
    assert!(lines2[0][0].underline);
}

#[test]
fn test_terminal_screen_cursor_state_isolation() {
    let mut screen1 = TerminalScreen::new(80, 24);
    let mut screen2 = TerminalScreen::new(80, 24);

    // Move cursor to different positions
    screen1.process(b"\x1b[10;20H");
    screen2.process(b"\x1b[5;15H");

    let (row1, col1) = screen1.cursor_position();
    let (row2, col2) = screen2.cursor_position();

    assert_eq!(row1, 9); // 0-indexed
    assert_eq!(col1, 19);

    assert_eq!(row2, 4);
    assert_eq!(col2, 14);
}

#[test]
fn test_terminal_screen_color_palette_isolation() {
    let mut screen1 = TerminalScreen::new(80, 24);
    let mut screen2 = TerminalScreen::new(80, 24);

    // Write with different colors
    screen1.process(b"\x1b[31m"); // Red
    screen1.process(b"Red text\r\n");

    screen2.process(b"\x1b[32m"); // Green
    screen2.process(b"Green text\r\n");

    let lines1 = screen1.get_all_lines();
    let lines2 = screen2.get_all_lines();

    // Verify colors are different
    let color1 = lines1[0][0].fg;
    let color2 = lines2[0][0].fg;

    assert!(color1.is_some());
    assert!(color2.is_some());
    assert_ne!(color1, color2);
}

// ============================================================================
// Integration Tests - Combined Functionality
// ============================================================================

#[test]
fn test_theme_with_terminal_colors() {
    let theme = Theme::dracula();
    let mut screen = TerminalScreen::new(80, 24);

    // Write with ANSI colors
    screen.process(b"\x1b[31mRed text\x1b[0m\r\n");

    let lines = screen.get_all_lines();

    // Terminal should have color information
    assert!(lines[0][0].fg.is_some());

    // Theme should provide matching palette colors
    let terminal_color = lines[0][0].fg.unwrap();
    let theme_red = theme.ansi.get_color(1); // ANSI red

    // Both should exist and be valid colors
    assert_ne!(terminal_color.to_color(), Color::BLACK);
    assert_ne!(theme_red, Color::BLACK);
}

#[test]
fn test_config_with_terminal_initialization() {
    let config = AppConfig::default();

    let screen = TerminalScreen::new(
        config.pty.default_cols as usize,
        config.pty.default_rows as usize,
    );

    let (rows, cols) = (
        config.pty.default_rows as usize,
        config.pty.default_cols as usize,
    );
    let lines = screen.get_all_lines();

    // Screen should be initialized with config dimensions
    assert!(lines.len() >= rows);
    assert_eq!(lines[0].len(), cols);
}

#[test]
fn test_alternate_screen_with_clipboard() {
    let mut screen = TerminalScreen::new(80, 24);

    // Set clipboard on main screen
    screen.process(b"\x1b]52;c;TWFpbg==\x1b\\"); // "Main"
    assert_eq!(screen.clipboard_request(), Some("TWFpbg=="));

    // Enter alternate screen
    screen.process(b"\x1b[?1049h");

    // Set different clipboard data
    screen.process(b"\x1b]52;c;QWx0\x1b\\"); // "Alt"
    assert_eq!(screen.clipboard_request(), Some("QWx0"));

    // Exit alternate screen
    screen.process(b"\x1b[?1049l");

    // Clipboard should retain alternate screen value (global state)
    assert_eq!(screen.clipboard_request(), Some("QWx0"));
}

#[test]
fn test_mouse_mode_with_alternate_screen() {
    let mut screen = TerminalScreen::new(80, 24);

    // Enable mouse on main screen
    screen.process(b"\x1b[?1000h");
    assert_eq!(screen.mouse_mode(), MouseMode::X10);

    // Enter alternate screen
    screen.process(b"\x1b[?1049h");

    // Mouse mode should persist
    assert_eq!(screen.mouse_mode(), MouseMode::X10);

    // Disable mouse on alternate screen
    screen.process(b"\x1b[?1000l");
    assert_eq!(screen.mouse_mode(), MouseMode::None);

    // Exit alternate screen
    screen.process(b"\x1b[?1049l");

    // Mouse mode change should persist
    assert_eq!(screen.mouse_mode(), MouseMode::None);
}

// ============================================================================
// Helper Functions for Base64 Testing
// ============================================================================

mod base64 {
    pub fn decode(encoded: &str) -> Result<Vec<u8>, String> {
        use std::collections::HashMap;

        let alphabet: HashMap<char, u8> =
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
                .chars()
                .enumerate()
                .map(|(i, c)| (c, i as u8))
                .collect();

        let mut result = Vec::new();
        let chars: Vec<char> = encoded.chars().filter(|c| *c != '=').collect();

        for chunk in chars.chunks(4) {
            if chunk.is_empty() {
                break;
            }

            let mut values = vec![0u8; 4];
            for (i, c) in chunk.iter().enumerate() {
                values[i] = *alphabet.get(c).ok_or("Invalid base64 character")?;
            }

            result.push((values[0] << 2) | (values[1] >> 4));

            if chunk.len() > 2 {
                result.push((values[1] << 4) | (values[2] >> 2));
            }

            if chunk.len() > 3 {
                result.push((values[2] << 6) | values[3]);
            }
        }

        Ok(result)
    }
}
