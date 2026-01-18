# Theme Editor Quick Reference

## Quick Start

```rust
use agterm::theme::Theme;
use agterm::theme_editor::*;

// Create editor
let mut editor = ThemeEditor::new(Theme::dracula());

// Edit color
editor.select_field(ColorField::AnsiRed);
editor.update_selected_color(ColorRgb::new(255, 100, 100));

// Export
editor.export_theme(Path::new("theme.toml"))?;
```

## Color Conversion Cheat Sheet

```rust
// Hex → RGB
let color = ColorRgb::from_hex("#FF0000");

// RGB → Hex
let hex = color.to_hex(); // "#FF0000"

// RGB → HSL
let (h, s, l) = color.to_hsl();

// HSL → RGB
let color = ColorRgb::from_hsl(0.0, 100.0, 50.0);

// Iced Color conversion
let iced_color = color.to_color();
let color = ColorRgb::from_color(iced_color);

// ColorDef conversion
let color_def = color.to_color_def();
let color = ColorRgb::from_color_def(&color_def);
```

## ColorPicker Usage

```rust
let mut picker = ColorPicker::new(ColorRgb::new(255, 0, 0));

// RGB sliders
picker.rgb_input.r = 128.0;
picker.update_from_rgb();

// HSL sliders
picker.hsl_input.h = 240.0;
picker.update_from_hsl();

// Hex input
picker.update_from_hex("#00FF00".to_string());

// Recent colors
picker.add_recent_color(picker.color);
```

## All Color Fields

### Terminal Colors
- `TerminalForeground`
- `TerminalBackground`
- `TerminalCursor`
- `TerminalCursorText`
- `TerminalSelection`
- `TerminalSelectionText`

### ANSI Normal (0-7)
- `AnsiBlack`, `AnsiRed`, `AnsiGreen`, `AnsiYellow`
- `AnsiBlue`, `AnsiMagenta`, `AnsiCyan`, `AnsiWhite`

### ANSI Bright (8-15)
- `AnsiBrightBlack`, `AnsiBrightRed`, `AnsiBrightGreen`, `AnsiBrightYellow`
- `AnsiBrightBlue`, `AnsiBrightMagenta`, `AnsiBrightCyan`, `AnsiBrightWhite`

## Theme Preview Samples

```rust
let mut preview = ThemePreview::new(theme);

// Change sample type
preview.sample_type = PreviewSample::AnsiColors;
preview.sample_type = PreviewSample::ShellPrompt;
preview.sample_type = PreviewSample::CodeHighlight;
preview.sample_type = PreviewSample::GitDiff;

// Get colored text
let samples = preview.get_sample_text();
```

## Import/Export

### AgTerm TOML
```rust
// Save
theme.to_toml_file(Path::new("theme.toml"))?;

// Load
let theme = Theme::from_toml_file(Path::new("theme.toml"))?;
```

### iTerm2
```rust
// Export
let xml = iterm2::export_iterm_theme(&theme);
fs::write("theme.itermcolors", xml)?;

// Import
let xml = fs::read_to_string("theme.itermcolors")?;
let theme = iterm2::parse_iterm_theme(&xml)?;
```

### VS Code
```rust
// Export
let json = vscode::export_vscode_theme(&theme)?;
fs::write("theme.json", json)?;

// Import
let json = fs::read_to_string("theme.json")?;
let theme = vscode::parse_vscode_theme(&json)?;
```

## Color Presets

```rust
use agterm::theme_editor::ColorPresets;

// Material colors
let material = ColorPresets::material_colors();
for (name, color) in material {
    println!("{}: {}", name, color.to_hex());
}

// Grayscale
let grayscale = ColorPresets::grayscale();
```

## Common Patterns

### Edit Multiple Colors
```rust
let edits = vec![
    (ColorField::AnsiRed, ColorRgb::new(255, 0, 0)),
    (ColorField::AnsiGreen, ColorRgb::new(0, 255, 0)),
    (ColorField::AnsiBlue, ColorRgb::new(0, 0, 255)),
];

for (field, color) in edits {
    editor.select_field(field);
    editor.update_selected_color(color);
}
```

### Theme Validation
```rust
// Check foreground/background contrast
let fg = ColorRgb::from_color_def(&theme.terminal.foreground);
let bg = ColorRgb::from_color_def(&theme.terminal.background);
// Calculate contrast ratio (implement your own)
```

### Create Theme from Scratch
```rust
let mut theme = Theme::warp_dark();
theme.name = "My Theme".to_string();

let mut editor = ThemeEditor::new(theme);
// Edit colors...
let final_theme = editor.theme;
```

## Testing

```bash
# Run all tests
cargo test theme_editor

# Run unit tests only
cargo test --lib theme_editor::tests

# Run integration tests
cargo test --test theme_editor_integration_test

# Run demo
cargo run --example theme_editor_demo
```

## Hex Code Reference

| Color | Hex | RGB |
|-------|-----|-----|
| Black | #000000 | 0,0,0 |
| White | #FFFFFF | 255,255,255 |
| Red | #FF0000 | 255,0,0 |
| Green | #00FF00 | 0,255,0 |
| Blue | #0000FF | 0,0,255 |
| Yellow | #FFFF00 | 255,255,0 |
| Cyan | #00FFFF | 0,255,255 |
| Magenta | #FF00FF | 255,0,255 |

## HSL Reference

| Color | H | S | L |
|-------|---|---|---|
| Red | 0° | 100% | 50% |
| Orange | 30° | 100% | 50% |
| Yellow | 60° | 100% | 50% |
| Green | 120° | 100% | 50% |
| Cyan | 180° | 100% | 50% |
| Blue | 240° | 100% | 50% |
| Magenta | 300° | 100% | 50% |

## Error Handling

```rust
// File operations
match editor.export_theme(path) {
    Ok(_) => println!("Theme saved"),
    Err(e) => eprintln!("Export failed: {}", e),
}

// Theme parsing
match Theme::from_toml_file(path) {
    Ok(theme) => editor.theme = theme,
    Err(e) => eprintln!("Import failed: {}", e),
}

// Format conversion
match vscode::parse_vscode_theme(&json) {
    Ok(theme) => /* use theme */,
    Err(e) => eprintln!("Parse error: {}", e),
}
```

## Performance Tips

1. **Batch updates**: Collect all edits before updating preview
2. **Reuse pickers**: Create once, update color instead of recreating
3. **Cache conversions**: Store ColorRgb instead of converting repeatedly
4. **Recent colors**: Limit to 12 for optimal memory usage

## Debug Helpers

```rust
// Print color info
println!("RGB: ({}, {}, {})", color.r, color.g, color.b);
println!("Hex: {}", color.to_hex());
let (h, s, l) = color.to_hsl();
println!("HSL: ({:.1}°, {:.1}%, {:.1}%)", h, s, l);

// Verify field update
let before = editor.get_field_color(field);
editor.update_selected_color(new_color);
let after = editor.get_field_color(field);
assert_ne!(before.to_color(), after.to_color());
```

## Module Structure

```
theme_editor
├── ColorRgb              - RGB color type
├── ColorPicker           - Color selection UI
│   ├── RgbInput
│   ├── HslInput
│   └── ColorInputMode
├── ThemePreview          - Live preview
│   └── PreviewSample
├── ThemeEditor           - Main editor
│   ├── ColorField
│   └── EditorMode
├── ColorPresets          - Preset palettes
├── iterm2                - iTerm2 format
└── vscode                - VS Code format
```

## See Also

- [THEME_EDITOR.md](THEME_EDITOR.md) - Full documentation
- [examples/theme_editor_demo.rs](examples/theme_editor_demo.rs) - Working examples
- [tests/theme_editor_integration_test.rs](tests/theme_editor_integration_test.rs) - Test examples
- [src/theme.rs](src/theme.rs) - Base theme system
