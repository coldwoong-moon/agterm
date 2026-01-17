# AgTerm Theme System

AgTerm features a comprehensive theme system with popular presets and full customization support.

## Built-in Themes

AgTerm includes several popular terminal themes out of the box:

### Dark Themes
- **warp_dark** (default) - Warp terminal-inspired modern dark theme
- **dracula** - Popular purple-tinted dark theme
- **solarized_dark** - Ethan Schoonover's precision colors
- **nord** - Arctic, north-bluish color palette
- **one_dark** - Atom editor's iconic dark theme
- **monokai_pro** - Modern take on Monokai
- **tokyo_night** - Clean, elegant dark theme

### Light Themes
- **solarized_light** - Solarized's light variant

## Using Built-in Themes

Set the theme in your configuration file (`~/.config/agterm/config.toml`):

```toml
[appearance]
theme = "dracula"
```

Or in the TUI section:

```toml
[tui]
theme = "nord"
```

## Creating Custom Themes

### Theme File Structure

Create a TOML file with the following structure:

```toml
name = "My Theme"
variant = "dark"  # or "light"

[ui]
# Background colors
bg_primary = "#17171c"
bg_secondary = "#1e1e26"
bg_block = "#242430"
bg_block_hover = "#2d2d38"
bg_input = "#1c1c24"

# Text colors
text_primary = "#edeff2"
text_secondary = "#999ead"
text_muted = "#737885"

# Accent colors
accent_blue = "#5c8afa"
accent_green = "#59c78c"
accent_yellow = "#f2c55c"
accent_red = "#eb6473"
accent_purple = "#8c5cfa"
accent_cyan = "#5ce6fa"

# UI elements
border = "#383847"
tab_active = "#5c8afa"
selection = "#5c8afa"

[ansi]
# Normal colors (0-7)
black = "#000000"
red = "#ff0000"
green = "#00ff00"
yellow = "#ffff00"
blue = "#0000ff"
magenta = "#ff00ff"
cyan = "#00ffff"
white = "#ffffff"

# Bright colors (8-15)
bright_black = "#808080"
bright_red = "#ff8080"
bright_green = "#80ff80"
bright_yellow = "#ffff80"
bright_blue = "#8080ff"
bright_magenta = "#ff80ff"
bright_cyan = "#80ffff"
bright_white = "#ffffff"

[terminal]
foreground = "#edeff2"
background = "#1e1e26"
cursor = "#5c8afa"
cursor_text = "#17171c"
selection = "#5c8afa"
selection_text = "#edeff2"
```

### Color Formats

The theme system supports multiple color formats:

1. **Hex string** (recommended):
   ```toml
   red = "#ff0000"
   ```

2. **Short hex**:
   ```toml
   red = "#f00"
   ```

3. **RGB array** (0-255):
   ```toml
   red = [255, 0, 0]
   ```

4. **RGB float array** (0.0-1.0):
   ```toml
   red = [1.0, 0.0, 0.0]
   ```

### Theme Locations

Place custom theme files in:

- User themes: `~/.config/agterm/themes/mytheme.toml`
- Project themes: `./.agterm/themes/mytheme.toml`

Then reference by filename (without extension):

```toml
[appearance]
theme = "mytheme"
```

## Theme Structure Explained

### UI Colors

- **bg_primary**: Main window background
- **bg_secondary**: Secondary panels/areas
- **bg_block**: Block elements (like code blocks)
- **bg_block_hover**: Hover state for blocks
- **bg_input**: Input field backgrounds
- **text_primary**: Main text color
- **text_secondary**: Secondary text (labels, etc.)
- **text_muted**: Dimmed/inactive text
- **accent_blue**: Primary accent (links, active elements)
- **accent_green**: Success states
- **accent_yellow**: Warning states
- **accent_red**: Error/danger states
- **accent_purple**: Special highlights
- **accent_cyan**: Information states
- **border**: Border colors
- **tab_active**: Active tab indicator
- **selection**: Selected text background

### ANSI Palette

The 16 ANSI colors used for terminal output:

- **0-7**: Normal intensity colors (black, red, green, yellow, blue, magenta, cyan, white)
- **8-15**: Bright/bold variants

### Terminal Colors

- **foreground**: Default text color
- **background**: Terminal background
- **cursor**: Cursor color
- **cursor_text**: Text under cursor
- **selection**: Selection highlight
- **selection_text**: Selected text color

## Examples

See `examples/custom_theme.toml` for a complete example.

### Converting from Other Terminal Themes

Most terminal emulator themes can be converted to AgTerm format:

1. Extract the color palette (usually 16 ANSI colors + foreground/background)
2. Map to the ANSI section
3. Set terminal foreground/background
4. Choose appropriate UI colors (or use defaults)

### iTerm2 Color Schemes

```toml
# From iTerm2 color scheme
[terminal]
foreground = "<Foreground Color>"
background = "<Background Color>"
cursor = "<Cursor Color>"

[ansi]
black = "<Ansi 0 Color>"
red = "<Ansi 1 Color>"
green = "<Ansi 2 Color>"
yellow = "<Ansi 3 Color>"
blue = "<Ansi 4 Color>"
magenta = "<Ansi 5 Color>"
cyan = "<Ansi 6 Color>"
white = "<Ansi 7 Color>"
bright_black = "<Ansi 8 Color>"
# ... etc
```

## Programmatic Usage

The theme system is available as a Rust library:

```rust
use agterm::theme::{Theme, ColorDef};

// Load a preset theme
let theme = Theme::dracula();

// Load from file
let theme = Theme::from_toml_file(path)?;

// Create programmatically
let mut theme = Theme::warp_dark();
theme.ui.accent_blue = ColorDef::from_hex("#ff00ff");

// Convert to Iced colors
let color = theme.ui.text_primary.to_color();

// Get ANSI color by index (0-15)
let red = theme.ansi.get_color(1);
```

## Tips

1. **Start with a preset**: Copy a built-in theme and modify it
2. **Test contrast**: Ensure good readability between text and background
3. **ANSI compatibility**: Test with tools like `ls --color`, `vim`, etc.
4. **Accessibility**: Maintain WCAG contrast ratios for accessibility

## Troubleshooting

### Theme not loading

1. Check file location: `~/.config/agterm/themes/`
2. Verify TOML syntax: `cargo test theme::tests::test_theme_deserialization`
3. Check logs for parsing errors

### Colors look wrong

1. Verify hex format: `#RRGGBB` (not `0xRRGGBB`)
2. Check terminal true color support
3. Ensure proper sRGB values

## Future Enhancements

Planned features:

- [ ] Hot reload themes without restart
- [ ] Theme picker UI
- [ ] Import from other terminal formats (iTerm2, Alacritty, etc.)
- [ ] Theme validation and preview tools
- [ ] Dark/Light mode switching
- [ ] Per-tab themes
