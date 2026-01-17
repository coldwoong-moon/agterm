# AgTerm Configuration Guide

AgTerm uses a layered configuration system with TOML files. Configuration is loaded in the following order (later sources override earlier ones):

1. **Default config** (embedded in binary) - `/default_config.toml`
2. **User config** - `~/.config/agterm/config.toml` (Linux/macOS) or `%APPDATA%\agterm\config.toml` (Windows)
3. **Project config** - `./.agterm/config.toml` (current directory)

## Configuration Sections

### [general] - General Settings

```toml
[general]
app_name = "agterm"
# default_shell = "/bin/zsh"  # Auto-detected from $SHELL if not set
# default_working_dir = "~"    # Default working directory for new terminals
```

### [appearance] - Visual Settings

```toml
[appearance]
font_family = "D2Coding"      # Font family name
font_size = 14.0              # Font size (8.0 - 24.0)
theme = "default"             # Theme name
background_opacity = 1.0      # Background opacity (0.0 - 1.0)
use_ligatures = true          # Enable font ligatures
```

#### Custom Color Scheme

You can override the theme with a custom color scheme:

```toml
[appearance.color_scheme]
background = "#17171c"
foreground = "#edeff2"
cursor = "#5c8afa"
selection = "#383847"

# ANSI colors (0-7)
black = "#000000"
red = "#eb6473"
green = "#59c78c"
yellow = "#f2c55c"
blue = "#5c8afa"
magenta = "#8c5cfa"
cyan = "#5cc8fa"
white = "#cccccc"

# Bright ANSI colors (8-15)
bright_black = "#808080"
bright_red = "#ff6f7f"
bright_green = "#6fd398"
bright_yellow = "#ffd168"
bright_blue = "#68a0ff"
bright_magenta = "#9868ff"
bright_cyan = "#68d4ff"
bright_white = "#ffffff"
```

### [terminal] - Terminal Behavior

```toml
[terminal]
scrollback_lines = 10000           # Number of lines to keep in scrollback
cursor_style = "block"             # "block", "underline", or "beam"
cursor_blink = true                # Enable cursor blinking
cursor_blink_interval_ms = 530     # Cursor blink interval in milliseconds
bell_enabled = true                # Enable bell notifications
bell_style = "visual"              # "visual", "sound", "both", or "none"
bracketed_paste = true             # Enable bracketed paste mode
auto_scroll_on_output = true       # Auto-scroll to bottom on new output
```

### [keybindings] - Keyboard Shortcuts

```toml
[keybindings]
mode = "default"                   # "default", "vim", or "emacs"

# Custom keybindings (example)
[keybindings.custom]
"Ctrl+Shift+C" = "copy"
"Ctrl+Shift+V" = "paste"
"Ctrl+Shift+F" = "search"
```

### [shell] - Shell Configuration

```toml
[shell]
# program = "/bin/zsh"            # Shell program (auto-detected if not set)
# args = ["--login"]              # Shell arguments
login_shell = true                # Launch as login shell

# Environment variables to set for shell
[shell.env]
TERM = "xterm-256color"
COLORTERM = "truecolor"
```

### [mouse] - Mouse Behavior

```toml
[mouse]
enabled = true                    # Enable mouse support
reporting = true                  # Allow applications to receive mouse events
selection_mode = "character"      # "character", "word", or "line"
copy_on_select = true             # Automatically copy selection to clipboard
middle_click_paste = true         # Paste with middle mouse button
```

### [pty] - PTY Configuration

```toml
[pty]
max_sessions = 32                 # Maximum number of concurrent PTY sessions
default_cols = 120                # Default terminal columns
default_rows = 40                 # Default terminal rows
scrollback_lines = 10000          # Scrollback buffer size
```

### [tui] - TUI Settings

```toml
[tui]
target_fps = 60                   # Target frames per second
show_line_numbers = false         # Show line numbers in terminal
theme = "default"                 # TUI theme
mouse_support = true              # Enable mouse support in TUI
keybindings = "default"           # Keybinding mode
```

### [logging] - Logging Configuration

```toml
[logging]
level = "info"                    # "trace", "debug", "info", "warn", or "error"
format = "pretty"                 # "pretty", "compact", or "json"
timestamps = true                 # Show timestamps in logs
file_line = false                 # Show file and line numbers
file_output = true                # Enable file output
# file_path = "~/.local/share/agterm/logs"  # Log file directory
```

### [debug] - Debug Panel

```toml
[debug]
enabled = false                   # Enable debug panel on startup
show_fps = true                   # Show FPS in debug panel
show_pty_stats = true             # Show PTY statistics
log_buffer_size = 50              # Number of log entries to keep
```

## Default Keyboard Shortcuts

- **Cmd+T** / **Ctrl+T** - New tab
- **Cmd+W** / **Ctrl+W** - Close current tab
- **Cmd+]** / **Ctrl+]** - Next tab
- **Cmd+[** / **Ctrl+[** - Previous tab
- **Cmd+1-9** - Switch to tab 1-9
- **Cmd+Shift+D** - Duplicate tab
- **Cmd+K** / **Ctrl+K** - Clear screen
- **Cmd+D** / **F12** - Toggle debug panel
- **Cmd++** / **Ctrl++** - Increase font size
- **Cmd+-** / **Ctrl+-** - Decrease font size
- **Cmd+0** / **Ctrl+0** - Reset font size
- **Cmd+Home** - Scroll to top
- **Cmd+End** - Scroll to bottom
- **Cmd+Shift+C** - Force copy selection
- **Cmd+Shift+V** - Force paste without bracketed paste
- **Ctrl+C** - Copy (if selection) or send SIGINT (if no selection)
- **Ctrl+D** - Send EOF
- **Ctrl+Z** - Send SIGTSTP (suspend)

## Environment Variables

- **AGTERM_LOG** - Override log level (e.g., `AGTERM_LOG=agterm=debug`)
- **AGTERM_DEBUG** - Enable debug panel on startup (any value enables it)

## Example Configuration

Here's a complete example configuration with common customizations:

```toml
# ~/.config/agterm/config.toml

[general]
app_name = "agterm"

[appearance]
font_family = "D2Coding"
font_size = 16.0
theme = "default"
background_opacity = 0.95
use_ligatures = true

[terminal]
scrollback_lines = 20000
cursor_style = "block"
cursor_blink = true
cursor_blink_interval_ms = 530
bell_enabled = true
bell_style = "visual"

[shell]
login_shell = true

[shell.env]
TERM = "xterm-256color"
COLORTERM = "truecolor"

[mouse]
enabled = true
copy_on_select = true
middle_click_paste = true

[pty]
default_cols = 120
default_rows = 40

[logging]
level = "info"
format = "pretty"
file_output = true

[debug]
enabled = false
show_fps = true
```

## Creating a Project-Local Config

For project-specific terminal settings, create a `.agterm/config.toml` in your project directory:

```bash
mkdir -p .agterm
cat > .agterm/config.toml << EOF
[general]
default_working_dir = "."

[shell.env]
PROJECT_ROOT = "$(pwd)"
EOF
```

## Configuration API (Rust)

If you're using AgTerm as a library, you can load and use the configuration programmatically:

```rust
use agterm::config::AppConfig;

// Load configuration with fallback chain
let config = AppConfig::load()?;

// Access configuration values
println!("Font size: {}", config.appearance.font_size);
println!("Scrollback lines: {}", config.terminal.scrollback_lines);

// Save configuration to user config path
config.save()?;
```

## Troubleshooting

### Configuration not loading

1. Check file permissions on your config directory
2. Verify TOML syntax with a TOML validator
3. Check logs for parsing errors: `AGTERM_LOG=agterm=debug agterm`

### Reset to defaults

To reset to default configuration, rename or delete your user config:

```bash
mv ~/.config/agterm/config.toml ~/.config/agterm/config.toml.backup
```

### View active configuration

Enable debug panel with **Cmd+D** or **F12** to see runtime configuration values and logs.
