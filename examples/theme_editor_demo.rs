//! Theme Editor Demo
//!
//! Demonstrates the theme editor functionality including:
//! - Creating and editing themes
//! - Color picker usage
//! - Theme preview
//! - Import/Export to iTerm2 and VS Code formats
//!
//! Run with: cargo run --example theme_editor_demo --features iced-gui
//!
//! Requires the `iced-gui` feature to be enabled.

#[cfg(not(feature = "iced-gui"))]
fn main() {
    eprintln!("This example requires the `iced-gui` feature. Run with:");
    eprintln!("  cargo run --example theme_editor_demo --features iced-gui");
}

#[cfg(feature = "iced-gui")]
mod demo {
    use agterm::theme::Theme;
    use agterm::theme_editor::{
        iterm2, vscode, ColorField, ColorPicker, ColorRgb, ThemeEditor, ThemePreview,
    };

    pub fn run() {
        println!("=== AgTerm Theme Editor Demo ===\n");

        // Demo 1: Color Picker
        demo_color_picker();

        // Demo 2: Theme Preview
        demo_theme_preview();

        // Demo 3: Theme Editor
        demo_theme_editor();

        // Demo 4: Export to iTerm2
        demo_iterm2_export();

        // Demo 5: Export to VS Code
        demo_vscode_export();

        println!("\n=== Demo Complete ===");
    }

    fn demo_color_picker() {
        println!("--- Demo 1: Color Picker ---");

        let mut picker = ColorPicker::new(ColorRgb::new(255, 0, 0));
        println!("Initial color: {} (Red)", picker.color.to_hex());

        // Test RGB input
        picker.rgb_input.r = 0.0;
        picker.rgb_input.g = 255.0;
        picker.rgb_input.b = 0.0;
        picker.update_from_rgb();
        println!("Updated to RGB(0, 255, 0): {}", picker.color.to_hex());

        // Test HSL input
        picker.hsl_input.h = 240.0; // Blue
        picker.hsl_input.s = 100.0;
        picker.hsl_input.l = 50.0;
        picker.update_from_hsl();
        println!("Updated to HSL(240, 100%, 50%): {}", picker.color.to_hex());

        // Test Hex input
        picker.update_from_hex("#FF00FF".to_string());
        println!("Updated to Hex #FF00FF: {}", picker.color.to_hex());

        // Test recent colors
        picker.add_recent_color(ColorRgb::new(255, 0, 0));
        picker.add_recent_color(ColorRgb::new(0, 255, 0));
        picker.add_recent_color(ColorRgb::new(0, 0, 255));
        println!("Recent colors count: {}", picker.recent_colors.len());

        println!();
    }

    fn demo_theme_preview() {
        println!("--- Demo 2: Theme Preview ---");

        let theme = Theme::dracula();
        let preview = ThemePreview::new(theme.clone());

        println!("Theme: {}", theme.name);
        println!("Variant: {:?}", theme.variant);

        let sample_text = preview.get_sample_text();
        println!("Sample text entries: {}", sample_text.len());

        for (i, (text, color)) in sample_text.iter().take(5).enumerate() {
            println!(
                "  {}: '{}' (R:{:.2}, G:{:.2}, B:{:.2})",
                i, text, color.r, color.g, color.b
            );
        }

        println!();
    }

    fn demo_theme_editor() {
        println!("--- Demo 3: Theme Editor ---");

        let mut editor = ThemeEditor::new(Theme::nord());
        println!("Editing theme: {}", editor.theme.name);

        // Select a color field
        editor.select_field(ColorField::AnsiRed);
        println!("Selected field: AnsiRed");

        // Get current color
        let current_color = editor.get_field_color(ColorField::AnsiRed);
        println!(
            "Current AnsiRed color: {}",
            ColorRgb::from_color_def(&current_color).to_hex()
        );

        // Update color
        let new_color = ColorRgb::new(255, 100, 100);
        editor.update_selected_color(new_color);
        println!("Updated AnsiRed to: {}", new_color.to_hex());

        // Verify update
        let updated_color = ColorRgb::from_color_def(&editor.theme.ansi.red);
        println!("Verified color: {}", updated_color.to_hex());

        println!();
    }

    fn demo_iterm2_export() {
        println!("--- Demo 4: iTerm2 Export ---");

        let theme = Theme::tokyo_night();
        let iterm_xml = iterm2::export_iterm_theme(&theme);

        println!("Exported {} theme to iTerm2 format", theme.name);
        println!("XML length: {} bytes", iterm_xml.len());
        println!(
            "First 200 chars:\n{}\n",
            &iterm_xml[..200.min(iterm_xml.len())]
        );

        println!();
    }

    fn demo_vscode_export() {
        println!("--- Demo 5: VS Code Export ---");

        let theme = Theme::one_dark();
        let vscode_json = vscode::export_vscode_theme(&theme).unwrap();

        println!("Exported {} theme to VS Code format", theme.name);
        println!("JSON length: {} bytes", vscode_json.len());
        println!(
            "First 300 chars:\n{}\n",
            &vscode_json[..300.min(vscode_json.len())]
        );

        // Test round-trip: export and import
        let imported_theme = vscode::parse_vscode_theme(&vscode_json).unwrap();
        println!("Round-trip test: {}", imported_theme.name);
        println!("  Original variant: {:?}", theme.variant);
        println!("  Imported variant: {:?}", imported_theme.variant);

        println!();
    }
}

#[cfg(feature = "iced-gui")]
fn main() {
    demo::run();
}
