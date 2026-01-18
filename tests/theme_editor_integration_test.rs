//! Integration tests for theme editor
//!
//! Tests the complete workflow of creating, editing, and exporting themes

use agterm::theme::Theme;
use agterm::theme_editor::{
    iterm2, vscode, ColorField, ColorPicker, ColorRgb, ThemeEditor, ThemePreview,
};
use tempfile::TempDir;

#[test]
fn test_complete_theme_editing_workflow() {
    // Create a theme editor with base theme
    let mut editor = ThemeEditor::new(Theme::warp_dark());

    // Edit multiple colors
    let edits = vec![
        (ColorField::AnsiRed, ColorRgb::new(255, 100, 100)),
        (ColorField::AnsiGreen, ColorRgb::new(100, 255, 100)),
        (ColorField::AnsiBlue, ColorRgb::new(100, 100, 255)),
        (
            ColorField::TerminalBackground,
            ColorRgb::new(20, 20, 30),
        ),
    ];

    for (field, color) in edits {
        editor.select_field(field);
        editor.update_selected_color(color);
    }

    // Verify changes
    assert_eq!(
        ColorRgb::from_color_def(&editor.theme.ansi.red),
        ColorRgb::new(255, 100, 100)
    );
    assert_eq!(
        ColorRgb::from_color_def(&editor.theme.ansi.green),
        ColorRgb::new(100, 255, 100)
    );
    assert_eq!(
        ColorRgb::from_color_def(&editor.theme.ansi.blue),
        ColorRgb::new(100, 100, 255)
    );
}

#[test]
fn test_theme_export_import_toml() {
    let temp_dir = TempDir::new().unwrap();
    let theme_path = temp_dir.path().join("test_theme.toml");

    // Create and export theme
    let original_theme = Theme::dracula();
    original_theme.to_toml_file(&theme_path).unwrap();

    // Import theme
    let imported_theme = Theme::from_toml_file(&theme_path).unwrap();

    // Verify
    assert_eq!(imported_theme.name, original_theme.name);
    assert_eq!(imported_theme.variant, original_theme.variant);
}

#[test]
fn test_color_picker_workflow() {
    let mut picker = ColorPicker::new(ColorRgb::new(0, 0, 0));

    // Test RGB workflow
    picker.rgb_input.r = 128.0;
    picker.rgb_input.g = 64.0;
    picker.rgb_input.b = 192.0;
    picker.update_from_rgb();
    assert_eq!(picker.color, ColorRgb::new(128, 64, 192));

    // Test HSL workflow
    picker.hsl_input.h = 0.0; // Red
    picker.hsl_input.s = 100.0;
    picker.hsl_input.l = 50.0;
    picker.update_from_hsl();
    assert_eq!(picker.color, ColorRgb::new(255, 0, 0));

    // Test hex workflow
    picker.update_from_hex("#00FF00".to_string());
    assert_eq!(picker.color, ColorRgb::new(0, 255, 0));

    // Test recent colors
    let colors = vec![
        ColorRgb::new(255, 0, 0),
        ColorRgb::new(0, 255, 0),
        ColorRgb::new(0, 0, 255),
    ];

    for color in &colors {
        picker.add_recent_color(*color);
    }

    assert_eq!(picker.recent_colors.len(), 3);
    // Most recent should be first
    assert_eq!(picker.recent_colors[0], ColorRgb::new(0, 0, 255));
}

#[test]
fn test_theme_preview_all_samples() {
    let theme = Theme::nord();
    let mut preview = ThemePreview::new(theme.clone());

    // Test all sample types
    let samples = [
        agterm::theme_editor::PreviewSample::AnsiColors,
        agterm::theme_editor::PreviewSample::ShellPrompt,
        agterm::theme_editor::PreviewSample::CodeHighlight,
        agterm::theme_editor::PreviewSample::GitDiff,
    ];

    for sample_type in samples {
        preview.sample_type = sample_type;
        let text = preview.get_sample_text();
        assert!(!text.is_empty(), "Sample {:?} should not be empty", sample_type);
    }
}

#[test]
fn test_iterm2_export_structure() {
    let theme = Theme::tokyo_night();
    let xml = iterm2::export_iterm_theme(&theme);

    // Verify XML structure
    assert!(xml.contains("<?xml version"));
    assert!(xml.contains("<!DOCTYPE plist"));
    assert!(xml.contains(&theme.name));

    // Verify all ANSI colors are present
    for i in 0..16 {
        assert!(xml.contains(&format!("Ansi {} Color", i)));
    }

    // Verify special colors
    assert!(xml.contains("Foreground Color"));
    assert!(xml.contains("Background Color"));
    assert!(xml.contains("Cursor Color"));

    // Verify color components
    assert!(xml.contains("Red Component"));
    assert!(xml.contains("Green Component"));
    assert!(xml.contains("Blue Component"));
}

#[test]
fn test_vscode_export_structure() {
    let theme = Theme::one_dark();
    let json = vscode::export_vscode_theme(&theme).unwrap();

    // Verify JSON structure
    assert!(json.contains("\"name\""));
    assert!(json.contains(&theme.name));
    assert!(json.contains("\"type\""));
    assert!(json.contains("\"colors\""));

    // Verify terminal colors
    assert!(json.contains("terminal.foreground"));
    assert!(json.contains("terminal.background"));

    // Verify ANSI colors
    for color in &[
        "Black", "Red", "Green", "Yellow", "Blue", "Magenta", "Cyan", "White",
    ] {
        assert!(json.contains(&format!("terminal.ansi{}", color)));
        assert!(json.contains(&format!("terminal.ansiBright{}", color)));
    }
}

#[test]
fn test_vscode_round_trip() {
    let original_theme = Theme::monokai_pro();

    // Export to VS Code format
    let json = vscode::export_vscode_theme(&original_theme).unwrap();

    // Import back
    let imported_theme = vscode::parse_vscode_theme(&json).unwrap();

    // Verify key properties
    assert_eq!(imported_theme.name, original_theme.name);
    assert_eq!(imported_theme.variant, original_theme.variant);

    // Verify ANSI colors (compare hex values)
    let orig_red = ColorRgb::from_color_def(&original_theme.ansi.red);
    let imp_red = ColorRgb::from_color_def(&imported_theme.ansi.red);
    assert_eq!(orig_red, imp_red);

    let orig_green = ColorRgb::from_color_def(&original_theme.ansi.green);
    let imp_green = ColorRgb::from_color_def(&imported_theme.ansi.green);
    assert_eq!(orig_green, imp_green);
}

#[test]
fn test_color_conversions() {
    // Test RGB to Hex
    let color = ColorRgb::new(255, 128, 64);
    assert_eq!(color.to_hex(), "#FF8040");

    // Test Hex to RGB
    let color = ColorRgb::from_hex("#FF8040");
    assert_eq!(color, ColorRgb::new(255, 128, 64));

    // Test short hex
    let color = ColorRgb::from_hex("#F80");
    assert_eq!(color, ColorRgb::new(255, 136, 0));

    // Test RGB to HSL and back
    let color = ColorRgb::new(255, 0, 0);
    let (h, s, l) = color.to_hsl();
    let converted = ColorRgb::from_hsl(h, s, l);
    assert_eq!(converted.r, 255);
    assert_eq!(converted.g, 0);
    assert_eq!(converted.b, 0);

    // Test grayscale
    let gray = ColorRgb::from_hsl(0.0, 0.0, 50.0);
    assert_eq!(gray.r, gray.g);
    assert_eq!(gray.g, gray.b);
}

#[test]
fn test_color_picker_hex_validation() {
    let mut picker = ColorPicker::default();

    // Valid hex
    picker.update_from_hex("#FF0000".to_string());
    assert_eq!(picker.color.r, 255);

    // Valid hex without #
    picker.update_from_hex("00FF00".to_string());
    assert_eq!(picker.color.g, 255);

    // Short hex
    picker.update_from_hex("#00F".to_string());
    assert_eq!(picker.color.b, 255);

    // Invalid hex (too short) - hex input is updated but color remains unchanged
    let prev_color = picker.color;
    picker.update_from_hex("#FF".to_string());
    // Hex input should be stored
    assert_eq!(picker.hex_input, "#FF");
    // Color should remain unchanged because hex is invalid
    assert_eq!(picker.color, prev_color);
}

#[test]
fn test_theme_editor_all_color_fields() {
    let mut editor = ThemeEditor::default();

    // Test editing each color field
    let fields = vec![
        ColorField::TerminalForeground,
        ColorField::TerminalBackground,
        ColorField::TerminalCursor,
        ColorField::TerminalCursorText,
        ColorField::TerminalSelection,
        ColorField::TerminalSelectionText,
        ColorField::AnsiBlack,
        ColorField::AnsiRed,
        ColorField::AnsiGreen,
        ColorField::AnsiYellow,
        ColorField::AnsiBlue,
        ColorField::AnsiMagenta,
        ColorField::AnsiCyan,
        ColorField::AnsiWhite,
        ColorField::AnsiBrightBlack,
        ColorField::AnsiBrightRed,
        ColorField::AnsiBrightGreen,
        ColorField::AnsiBrightYellow,
        ColorField::AnsiBrightBlue,
        ColorField::AnsiBrightMagenta,
        ColorField::AnsiBrightCyan,
        ColorField::AnsiBrightWhite,
    ];

    for field in fields {
        // Select field
        editor.select_field(field);
        assert_eq!(editor.selected_field, field);

        // Get current color
        let _color = editor.get_field_color(field);

        // Update color
        let new_color = ColorRgb::new(123, 45, 67);
        editor.update_selected_color(new_color);

        // Verify update
        let updated = editor.get_field_color(field);
        let rgb = ColorRgb::from_color_def(&updated);
        assert_eq!(rgb, new_color);
    }
}

#[test]
fn test_color_presets() {
    use agterm::theme_editor::ColorPresets;

    let material = ColorPresets::material_colors();
    assert!(!material.is_empty());
    assert!(material.len() >= 10);

    let grayscale = ColorPresets::grayscale();
    assert!(!grayscale.is_empty());
    assert!(grayscale.len() >= 5);

    // Verify preset colors are valid
    for (name, color) in material {
        assert!(!name.is_empty());
        let hex = color.to_hex();
        assert!(hex.starts_with('#'));
        assert_eq!(hex.len(), 7);
    }
}

#[test]
fn test_recent_colors_deduplication() {
    let mut picker = ColorPicker::default();

    let red = ColorRgb::new(255, 0, 0);
    let green = ColorRgb::new(0, 255, 0);
    let blue = ColorRgb::new(0, 0, 255);

    // Add colors
    picker.add_recent_color(red);
    picker.add_recent_color(green);
    picker.add_recent_color(blue);
    assert_eq!(picker.recent_colors.len(), 3);

    // Add duplicate - should move to front
    picker.add_recent_color(red);
    assert_eq!(picker.recent_colors.len(), 3);
    assert_eq!(picker.recent_colors[0], red);

    // Add another duplicate
    picker.add_recent_color(green);
    assert_eq!(picker.recent_colors.len(), 3);
    assert_eq!(picker.recent_colors[0], green);
}

#[test]
fn test_recent_colors_max_limit() {
    let mut picker = ColorPicker::default();

    // Add more than 12 colors
    for i in 0..20 {
        picker.add_recent_color(ColorRgb::new(i * 10, i * 10, i * 10));
    }

    // Should be limited to 12
    assert_eq!(picker.recent_colors.len(), 12);

    // Most recent should be first
    assert_eq!(picker.recent_colors[0], ColorRgb::new(190, 190, 190));
}

#[test]
fn test_all_preset_themes_with_editor() {
    let themes = vec![
        Theme::warp_dark(),
        Theme::dracula(),
        Theme::solarized_dark(),
        Theme::solarized_light(),
        Theme::nord(),
        Theme::one_dark(),
        Theme::monokai_pro(),
        Theme::tokyo_night(),
    ];

    for theme in themes {
        let mut editor = ThemeEditor::new(theme.clone());

        // Verify theme loaded correctly
        assert_eq!(editor.theme.name, theme.name);

        // Test editing a color
        editor.select_field(ColorField::AnsiRed);
        let new_color = ColorRgb::new(200, 100, 50);
        editor.update_selected_color(new_color);

        // Verify update
        let updated = ColorRgb::from_color_def(&editor.theme.ansi.red);
        assert_eq!(updated, new_color);
    }
}
