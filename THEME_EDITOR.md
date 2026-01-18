# AgTerm Theme Editor

Comprehensive theme editing system for AgTerm terminal emulator with support for multiple theme formats.

## Features

### 1. Color Picker
- **RGB Input**: Sliders for Red, Green, Blue (0-255)
- **HSL Input**: Sliders for Hue (0-360°), Saturation (0-100%), Lightness (0-100%)
- **Hex Input**: Direct hex color code input (#RRGGBB)
- **Preset Colors**: Material Design and grayscale palettes
- **Recent Colors**: Automatic history of recently used colors (max 12)

### 2. Theme Preview
Real-time preview with multiple sample types:
- **ANSI Colors**: Shows all 16 ANSI colors (normal + bright)
- **Shell Prompt**: Simulated shell prompt with colors
- **Code Highlight**: Syntax highlighted code sample
- **Git Diff**: Git diff output with colors

### 3. Theme Editor
- **ANSI 16-Color Palette**: Edit all 16 standard ANSI colors
- **Terminal Colors**: Foreground, background, cursor, selection
- **UI Colors**: Application interface colors
- **Live Preview**: See changes in real-time

### 4. Import/Export
- **AgTerm Format**: Native TOML format
- **iTerm2 Format**: Import/export .itermcolors files
- **VS Code Format**: Import/export VS Code theme JSON

## Usage

### Basic Example

```rust
use agterm::theme::Theme;
use agterm::theme_editor::{ColorPicker, ColorRgb, ThemeEditor, ColorField};

// Create theme editor
let mut editor = ThemeEditor::new(Theme::dracula());

// Select a color to edit
editor.select_field(ColorField::AnsiRed);

// Update the color
let new_color = ColorRgb::new(255, 100, 100);
editor.update_selected_color(new_color);

// Export theme
editor.export_theme(Path::new("my_theme.toml"))?;
```

### Color Picker Example

```rust
use agterm::theme_editor::{ColorPicker, ColorRgb};

// Create color picker
let mut picker = ColorPicker::new(ColorRgb::new(255, 0, 0));

// Update via RGB sliders
picker.rgb_input.r = 128.0;
picker.rgb_input.g = 64.0;
picker.rgb_input.b = 32.0;
picker.update_from_rgb();

// Update via HSL sliders
picker.hsl_input.h = 240.0; // Blue hue
picker.hsl_input.s = 100.0; // Full saturation
picker.hsl_input.l = 50.0;  // Medium lightness
picker.update_from_hsl();

// Update via hex input
picker.update_from_hex("#FF00FF".to_string());

// Add to recent colors
picker.add_recent_color(picker.color);
```

### Theme Preview Example

```rust
use agterm::theme::Theme;
use agterm::theme_editor::{ThemePreview, PreviewSample};

// Create preview
let mut preview = ThemePreview::new(Theme::nord());

// Change preview sample
preview.sample_type = PreviewSample::ShellPrompt;

// Get sample text with colors
let sample = preview.get_sample_text();
for (text, color) in sample {
    println!("{} - {:?}", text, color);
}
```

### iTerm2 Import/Export

```rust
use agterm::theme::Theme;
use agterm::theme_editor::iterm2;

// Export to iTerm2
let theme = Theme::tokyo_night();
let iterm_xml = iterm2::export_iterm_theme(&theme);
std::fs::write("tokyo_night.itermcolors", iterm_xml)?;

// Import from iTerm2
let xml = std::fs::read_to_string("my_theme.itermcolors")?;
let theme = iterm2::parse_iterm_theme(&xml)?;
```

### VS Code Import/Export

```rust
use agterm::theme::Theme;
use agterm::theme_editor::vscode;

// Export to VS Code
let theme = Theme::one_dark();
let vscode_json = vscode::export_vscode_theme(&theme)?;
std::fs::write("one_dark.json", vscode_json)?;

// Import from VS Code
let json = std::fs::read_to_string("my_theme.json")?;
let theme = vscode::parse_vscode_theme(&json)?;
```

## Color Conversion

The `ColorRgb` type provides comprehensive color conversion:

```rust
use agterm::theme_editor::ColorRgb;

let color = ColorRgb::new(255, 0, 0);

// Convert to hex
let hex = color.to_hex(); // "#FF0000"

// Convert to HSL
let (h, s, l) = color.to_hsl(); // (0.0, 100.0, 50.0)

// Convert from hex
let color = ColorRgb::from_hex("#00FF00");

// Convert from HSL
let color = ColorRgb::from_hsl(240.0, 100.0, 50.0); // Blue

// Convert to Iced Color
let iced_color = color.to_color();

// Convert to/from ColorDef
let color_def = color.to_color_def();
let color = ColorRgb::from_color_def(&color_def);
```

## Theme Structure

### Color Fields

The editor supports editing the following color fields:

#### Terminal Colors
- `TerminalForeground`: Default text color
- `TerminalBackground`: Background color
- `TerminalCursor`: Cursor color
- `TerminalCursorText`: Text under cursor
- `TerminalSelection`: Selection background
- `TerminalSelectionText`: Selected text color

#### ANSI Colors (0-7)
- `AnsiBlack`
- `AnsiRed`
- `AnsiGreen`
- `AnsiYellow`
- `AnsiBlue`
- `AnsiMagenta`
- `AnsiCyan`
- `AnsiWhite`

#### ANSI Bright Colors (8-15)
- `AnsiBrightBlack`
- `AnsiBrightRed`
- `AnsiBrightGreen`
- `AnsiBrightYellow`
- `AnsiBrightBlue`
- `AnsiBrightMagenta`
- `AnsiBrightCyan`
- `AnsiBrightWhite`

## File Formats

### AgTerm TOML Format

Native format with complete theme definition:

```toml
name = "My Theme"
variant = "dark"

[terminal]
foreground = "#edeff2"
background = "#1e1e26"
cursor = "#5c8afa"
cursor_text = "#17171c"
selection = "#5c8afa"
selection_text = "#edeff2"

[ansi]
black = "#17171c"
red = "#eb6473"
green = "#59c78c"
# ... etc
```

### iTerm2 Format

XML plist format compatible with iTerm2:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "...">
<plist version="1.0">
<dict>
    <key>name</key>
    <string>My Theme</string>
    <key>Ansi 0 Color</key>
    <dict>
        <key>Red Component</key>
        <real>0.09</real>
        <key>Green Component</key>
        <real>0.09</real>
        <key>Blue Component</key>
        <real>0.11</real>
    </dict>
    <!-- ... -->
</dict>
</plist>
```

### VS Code Format

JSON format compatible with VS Code:

```json
{
  "name": "My Theme",
  "type": "dark",
  "colors": {
    "terminal.foreground": "#edeff2",
    "terminal.background": "#1e1e26",
    "terminal.ansiBlack": "#17171c",
    "terminal.ansiRed": "#eb6473",
    "terminal.ansiGreen": "#59c78c"
    // ... etc
  }
}
```

## Testing

Run the demo to see all features in action:

```bash
cargo run --example theme_editor_demo
```

Run the test suite:

```bash
cargo test theme_editor
```

## Architecture

### Module Structure

```
theme_editor/
├── ColorPicker       - Color selection and input
│   ├── ColorRgb      - RGB color representation
│   ├── RgbInput      - RGB slider state
│   ├── HslInput      - HSL slider state
│   └── ColorPresets  - Preset color palettes
├── ThemePreview      - Live theme preview
│   └── PreviewSample - Sample text types
├── ThemeEditor       - Main editor component
│   ├── ColorField    - Editable color fields
│   └── EditorMode    - Editing modes
├── iterm2            - iTerm2 format support
│   ├── parse_iterm_theme()
│   └── export_iterm_theme()
└── vscode            - VS Code format support
    ├── parse_vscode_theme()
    └── export_vscode_theme()
```

### Color Conversion Pipeline

```
Hex String ──────────┐
                     ├──> ColorRgb <──> HSL (h, s, l)
RGB (0-255) ─────────┤        │
                     │        ├──> Iced Color
ColorDef ────────────┘        └──> ColorDef
```

## Best Practices

### Creating Custom Themes

1. **Start with a preset**: Use a similar theme as a base
2. **Test readability**: Ensure sufficient contrast for terminal colors
3. **Check all samples**: Verify colors work with all preview samples
4. **Save frequently**: Export to TOML during editing
5. **Test in terminal**: Apply theme to actual terminal to verify

### Color Selection Tips

- **Foreground/Background**: Aim for 7:1 contrast ratio (WCAG AAA)
- **ANSI Colors**: Keep distinct hues for different colors
- **Bright Colors**: Should be noticeably brighter than normal colors
- **Cursor**: Should contrast well with both foreground and background
- **Selection**: Use semi-transparent overlay or contrasting color

### Import/Export Guidelines

- **iTerm2**: Best for sharing with macOS users
- **VS Code**: Good for cross-platform terminal themes
- **AgTerm TOML**: Best for complete theme including UI colors

## Limitations

### Current Limitations

1. **256-Color Palette**: Not yet implemented (only 16 ANSI colors)
2. **True Color Editing**: Coming soon
3. **GUI Editor**: Command-line only (GUI coming in future update)
4. **iTerm2 Parser**: Simplified parser (may not support all features)

### Future Enhancements

- [ ] 256-color palette editor
- [ ] True color (24-bit) palette support
- [ ] Visual color picker widget (not just sliders)
- [ ] Theme gallery with popular themes
- [ ] Color blindness simulation
- [ ] Automatic contrast checking
- [ ] Theme generator from images
- [ ] More theme format support (Alacritty, Kitty, etc.)

## Examples

See the [examples/theme_editor_demo.rs](examples/theme_editor_demo.rs) file for comprehensive examples of all features.

## API Reference

Full API documentation is available via rustdoc:

```bash
cargo doc --open --no-deps --document-private-items
```

Navigate to `agterm::theme_editor` module.

## Contributing

When contributing theme editor features:

1. Add tests for new functionality
2. Update this documentation
3. Add examples to the demo
4. Ensure color conversions are accurate
5. Maintain compatibility with existing theme format

## License

Same as AgTerm project (MIT).
