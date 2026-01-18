# AgTerm

**AI Agent Terminal - Native GPU-Accelerated Terminal Emulator**

AgTerm is a modern terminal emulator built with Rust and the Iced GUI framework, featuring native GPU acceleration, comprehensive ANSI/VTE support, and Korean/CJK input handling. Designed for both daily terminal use and AI agent workflows with advanced customization and automation capabilities.

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Features

### Core Terminal Features

- **GPU-Accelerated Rendering**: Built on Iced GUI framework with tiny-skia CPU renderer optimized for text rendering
- **Multi-Tab Support**: Create, close, duplicate, and switch between multiple terminal sessions
- **Full VTE/ANSI Support**:
  - 256-color and TrueColor (24-bit) color support
  - SGR (Select Graphic Rendition) with bold, dim, italic, underline, strikethrough
  - Cursor control and positioning
  - Device Attributes (DA, DSR, CPR) responses
  - OSC sequences (window title, hyperlinks)
  - Application cursor keys mode
- **Korean/CJK Input**: Native IME support with D2Coding font for Korean and other CJK characters
- **Wide Character Support**: Proper handling of double-width characters (Korean, Japanese, Chinese)
- **Smart Cursor Blinking**: Alacritty-style cursor with configurable blink interval
- **Text Selection & Clipboard**: Mouse-based text selection with copy/paste support
- **Scrollback Buffer**: Configurable scrollback (up to 10,000 lines) with virtual scrolling for performance
- **Terminal Bell**: Visual and audio notification support with volume control
- **URL Detection**: Automatic URL recognition and click-to-open functionality

### Advanced Features

- **Profile System**: Multiple terminal profiles with custom settings
- **Session Restoration**: Automatically save and restore tabs, working directories, and titles on startup
- **Command Snippets**: 24 built-in snippets for common commands (Git, Docker, Kubernetes, Cargo)
  - Quick access via shorthand (e.g., `/gs` → `git status`)
  - Customizable and extensible
  - Categorized by tool (git, docker, common, kubernetes, cargo)
- **Event Hook System**: Automate actions based on terminal events
  - Command completion hooks
  - Directory change triggers
  - Output pattern matching
  - Bell notifications
  - See [Hook System Documentation](docs/HOOKS.md) for details
- **Custom Keybindings**: Full keyboard customization via configuration file
- **Environment Detection**: Automatically detects and adapts to SSH sessions, containers, and tmux
  - Optimizes refresh rate and performance
  - Shows visual indicators in status bar
  - See [Environment Detection Guide](docs/ENVIRONMENT_DETECTION.md)
- **Image Protocol Support**: Basic inline image rendering (iTerm2/kitty protocols)
- **Memory Optimization**: Smart buffer management with interner for efficient string storage

### User Interface

- **Modern Dark Theme**: Warp-inspired color scheme with high contrast
- **Custom Themes**: Support for custom color schemes via configuration
- **Dynamic Tab Titles**: Custom tab titles via OSC sequences
- **Bell Notifications**: Visual and audio indicators when background tabs receive bell signals
- **Status Bar**: Shows current shell, mode, environment indicators, and active tab information
- **Debug Panel**: Real-time terminal state inspection (F12 or Cmd+D)
- **Font Size Control**: Dynamically adjust font size (8-24pt)
- **Adaptive Refresh Rate**: Dynamic tick rate based on activity (5-60 FPS) for optimal performance

### Developer Features

- **Comprehensive Configuration**: TOML-based configuration system
  - Embedded defaults with user overrides
  - Per-project configuration support (`.agterm/config.toml`)
  - Runtime configuration reloading
- **Debug Panel**: Inspect PTY state, metrics, memory usage, and logs
- **Comprehensive Logging**: Structured logging with tracing
- **Performance Metrics**: Frame timing, PTY I/O monitoring, and memory profiling
- **Dynamic Tick Rate**: Adaptive polling based on activity and environment

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/coldwoong-moon/agterm.git
cd agterm

# Build and install
cargo install --path .
```

### Using Cargo

```bash
cargo install agterm
```

### Requirements

- Rust 1.75 or later
- macOS, Linux, or Windows
- A system shell (bash, zsh, fish, etc.)

## Quick Start

```bash
# Start AgTerm
agterm

# The terminal will open with a single tab ready to use
```

## Keyboard Shortcuts

### Tab Management

| Shortcut | Action |
|----------|--------|
| `Cmd+T` | New tab |
| `Cmd+W` | Close current tab |
| `Cmd+Shift+D` | Duplicate current tab |
| `Cmd+[` | Previous tab |
| `Cmd+]` | Next tab |
| `Cmd+1-9` | Switch to tab 1-9 |

### Terminal Control

| Shortcut | Action |
|----------|--------|
| `Cmd+K` | Clear screen |
| `Cmd+Home` | Scroll to top |
| `Cmd+End` | Scroll to bottom |
| `Ctrl+C` | Send interrupt signal (or copy if text selected) |
| `Ctrl+D` | Send EOF signal |
| `Ctrl+Z` | Send suspend signal |

### Clipboard

| Shortcut | Action |
|----------|--------|
| `Cmd+C` | Copy selection (if text selected) |
| `Cmd+V` | Paste with bracketed paste mode |
| `Cmd+Shift+C` | Force copy selection |
| `Cmd+Shift+V` | Force paste (without bracketed paste) |

### Font Size

| Shortcut | Action |
|----------|--------|
| `Cmd++` or `Cmd+=` | Increase font size |
| `Cmd+-` | Decrease font size |
| `Cmd+0` | Reset font size to default |

### Debug

| Shortcut | Action |
|----------|--------|
| `Cmd+D` or `F12` | Toggle debug panel |

### Custom Keybindings

Create `~/.config/agterm/keybindings.toml` to customize shortcuts. See [Configuration](#configuration) for details.

## Configuration

### Config File Location

AgTerm uses a layered configuration system:

1. **Embedded defaults**: Built into the binary (`default_config.toml`)
2. **User config**: `~/.config/agterm/config.toml` (macOS/Linux) or `%APPDATA%\agterm\config.toml` (Windows)
3. **Project config**: `./.agterm/config.toml` (optional, overrides user config)

### Complete Configuration Example

```toml
[general]
app_name = "agterm"
default_shell = "/bin/zsh"
# default_working_dir = "~"

# Session management
[general.session]
restore_on_startup = true      # Restore previous tabs on startup
save_on_exit = true            # Save session when closing
# session_file = "~/.config/agterm/session.json"

# ============================================================================
# Appearance Settings
# ============================================================================

[appearance]
font_family = "D2Coding"
font_size = 14.0
theme = "default"              # default, dracula, nord, gruvbox, etc.
background_opacity = 1.0
use_ligatures = true

# Optional: Custom color scheme (overrides theme)
[appearance.color_scheme]
background = "#17171c"
foreground = "#edeff2"
cursor = "#5c8afa"
selection = "#383847"
# ANSI colors (optional)
black = "#000000"
red = "#eb6473"
green = "#59c78c"
yellow = "#f2c55c"
blue = "#5c8afa"
magenta = "#8c5cfa"
cyan = "#5cc8fa"
white = "#cccccc"

# ============================================================================
# Terminal Behavior
# ============================================================================

[terminal]
scrollback_lines = 10000
cursor_style = "block"           # block, underline, beam
cursor_blink = true
cursor_blink_interval_ms = 530
bell_enabled = true
bell_style = "visual"            # visual, sound, both, none
bell_volume = 0.5                # 0.0 to 1.0 (50% by default)
bracketed_paste = true
auto_scroll_on_output = true

# Image protocol support
[terminal.images]
enabled = true
max_width = 1920
max_height = 1080

# ============================================================================
# Keybindings
# ============================================================================

[keybindings]
mode = "default"                 # default, vim, emacs

# Keyboard repeat settings
[keybindings.keyboard]
repeat_delay_ms = 500            # Initial delay before repeat starts
repeat_rate_ms = 30              # Interval between repeats (30ms ≈ 33 keys/sec)

# Custom keybindings example (create ~/.config/agterm/keybindings.toml):
# [[bindings]]
# key = "t"
# modifiers = { cmd = true }
# action = "new_tab"
# description = "Open a new tab"

# ============================================================================
# Shell Configuration
# ============================================================================

[shell]
# program = "/bin/zsh"
# args = ["--login"]
login_shell = true

# Environment variables
[shell.env]
TERM = "xterm-256color"
COLORTERM = "truecolor"

# ============================================================================
# Mouse Configuration
# ============================================================================

[mouse]
enabled = true
reporting = true                 # Allow applications to receive mouse events
selection_mode = "character"     # character, word, line
copy_on_select = true
middle_click_paste = true

# ============================================================================
# PTY Configuration
# ============================================================================

[pty]
max_sessions = 32
default_cols = 120
default_rows = 40
scrollback_lines = 10000

# ============================================================================
# TUI Settings
# ============================================================================

[tui]
target_fps = 60
show_line_numbers = false
theme = "default"
mouse_support = true
keybindings = "default"          # default, vim, emacs

# ============================================================================
# Logging
# ============================================================================

[logging]
level = "info"                   # trace, debug, info, warn, error
format = "pretty"                # pretty, compact, json
timestamps = true
file_line = false
file_output = true
# file_path = auto               # Platform-specific default location

# ============================================================================
# Debug Panel
# ============================================================================

[debug]
enabled = false                  # Enable debug panel on startup (or set AGTERM_DEBUG=1)
show_fps = true
show_pty_stats = true
log_buffer_size = 50
```

### Session Restoration

AgTerm automatically saves your terminal session (tabs, working directories, titles) when you exit and restores it on startup. This feature is enabled by default.

**How it works:**

1. When you close AgTerm, it saves the current state to `~/.config/agterm/session.json`
2. On next startup, it restores all tabs with their working directories and custom titles
3. Each tab gets a fresh PTY session in the saved directory

**Configuration:**

```toml
[general.session]
restore_on_startup = true      # Enable/disable restoration
save_on_exit = true            # Enable/disable saving
# session_file = "~/.config/agterm/session.json"  # Custom location
```

See [Session Restoration Guide](docs/SESSION_RESTORATION.md) for detailed documentation.

### Command Snippets

AgTerm includes 24 built-in command snippets for quick access to common commands:

**Git Commands** (`/gs`, `/ga`, `/gc`, `/gp`, `/gpl`, `/gl`, `/gd`, `/gb`, `/gco`)
**Docker Commands** (`/dps`, `/di`, `/dcu`, `/dcd`, `/dlogs`)
**Kubernetes Commands** (`/kgp`, `/kdesc`, `/klogs`)
**Cargo Commands** (`/cb`, `/cr`, `/ct`, `/cc`)
**Common Commands** (`/ll`, `/ff`, `/gr`)

Type a snippet trigger (e.g., `/gs`) and press Tab or Enter to expand it.

**Custom Snippets**: Create `~/.config/agterm/snippets.toml`:

```toml
[[snippets]]
name = "SSH to Production"
trigger = "/sshprod"
expansion = "ssh user@production.example.com"
category = "ssh"
```

### Event Hooks

Automate actions based on terminal events. Create `~/.config/agterm/hooks.toml`:

```toml
[[hooks]]
name = "Build Success Notification"
enabled = true

[hooks.event_type]
type = "CommandComplete"
data = { command_pattern = "cargo build", exit_code = 0 }

[hooks.action]
type = "Notify"
data = { title = "Build Complete", message = "Cargo build succeeded!" }
```

**Available Events:**
- `CommandComplete`: Trigger on command completion with optional pattern and exit code
- `DirectoryChange`: Trigger on directory change with optional path pattern
- `OutputMatch`: Trigger on output matching a pattern
- `Bell`: Trigger on terminal bell

**Available Actions:**
- `Notify`: Desktop notification
- `Sound`: Play sound effect
- `Command`: Execute shell command
- `Log`: Write to log file

See [Hook System Documentation](docs/HOOKS.md) for complete guide.

## Technical Architecture

### Framework Stack

- **GUI Framework**: [Iced](https://iced.rs/) - Cross-platform GUI with GPU acceleration
- **PTY Backend**: [portable-pty](https://docs.rs/portable-pty) - Cross-platform PTY management
- **VTE Parser**: [vte](https://docs.rs/vte) - ANSI escape sequence parsing
- **Clipboard**: [arboard](https://docs.rs/arboard) - Cross-platform clipboard access

### Rendering Pipeline

```
PTY Output → VTE Parser → Screen Buffer → Styled Spans → GPU Canvas
```

1. **PTY Output**: Raw bytes from shell process
2. **VTE Parser**: Parses ANSI escape sequences
3. **Screen Buffer**: Grid-based terminal state (cells, cursor, attributes)
4. **Styled Spans**: Optimized text spans with color and style attributes
5. **GPU Canvas**: Hardware-accelerated rendering via Iced canvas

### Performance Optimizations

- **Virtual Scrolling**: Only renders visible lines
- **Cached Text Layout**: Reuses styled spans between frames
- **Dynamic Tick Rate**: Adjusts polling frequency based on activity and environment
- **Content Versioning**: Invalidates cache only when content changes
- **String Interning**: Deduplicates repeated strings in terminal buffer
- **Memory Profiling**: Built-in memory usage tracking and optimization
- **Adaptive Refresh**: SSH/Container environments use lower refresh rates (20 FPS vs 60 FPS)

### Memory Usage

Typical memory footprint:
- **Base application**: ~15-20 MB
- **Per tab**: ~2-5 MB (depends on scrollback)
- **With 10,000 line scrollback**: ~3-4 MB per tab
- **String interner savings**: 30-50% reduction in repeated string storage

View detailed memory statistics in the debug panel (F12).

## Terminal Compatibility

AgTerm aims for high compatibility with modern terminal applications:

- **TERM**: Set to `xterm-256color` by default
- **Shell Integration**: Works with bash, zsh, fish, and other common shells
- **Interactive Apps**: Supports vim, emacs, htop, tmux, and other full-screen applications
- **Color Support**: 256-color palette and 24-bit TrueColor
- **Mouse Reporting**: Basic mouse events for applications that support it
- **Environment Adaptation**: Automatically detects SSH, containers, and tmux

### Known Limitations

- MCP (Model Context Protocol) support is planned but not yet implemented
- Search functionality (Cmd+F) is planned but not yet implemented
- Split panes are not yet implemented
- Image protocol support is basic (experimental)

## Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/coldwoong-moon/agterm.git
cd agterm

# Run in development mode
cargo run

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run

# Build release binary
cargo build --release
```

### Project Structure

```
agterm/
├── src/
│   ├── main.rs              # Application entry point and UI
│   ├── config/
│   │   └── mod.rs           # Configuration system
│   ├── terminal/
│   │   ├── mod.rs
│   │   ├── pty.rs           # PTY management
│   │   ├── screen.rs        # Terminal screen buffer and VTE
│   │   ├── ansi_color.rs    # ANSI color parsing
│   │   └── env.rs           # Environment detection
│   ├── terminal_canvas.rs   # GPU-accelerated canvas rendering
│   ├── debug/
│   │   ├── mod.rs
│   │   ├── panel.rs         # Debug panel UI
│   │   └── metrics.rs       # Performance metrics
│   ├── theme.rs             # Color schemes and themes
│   └── logging.rs           # Logging configuration
├── assets/
│   └── fonts/
│       └── D2Coding.ttf     # Embedded Korean font
├── docs/                    # Documentation
├── default_config.toml      # Embedded default configuration
└── Cargo.toml
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_tab_management

# Run only unit tests (skip PTY integration tests)
cargo test --lib
```

### Code Quality

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run clippy lints
cargo clippy --all-targets

# Check for issues
cargo clippy -- -D warnings
```

## Documentation

- [Session Restoration Guide](docs/SESSION_RESTORATION.md)
- [Hook System Documentation](docs/HOOKS.md)
- [Hook System Quick Start](docs/HOOKS_QUICKSTART.md)
- [Environment Detection Guide](docs/ENVIRONMENT_DETECTION.md)
- [Theme Customization](docs/THEMES.md)
- [Quick Start Guide](docs/QUICK_START.md)
- [API Documentation](docs/API_DOCUMENTATION.md)
- [Performance Benchmarks](docs/BENCHMARKS.md)

## Screenshots

> Screenshots coming soon

## Contributing

Contributions are welcome! Please follow these guidelines:

1. **Fork the repository** and create a feature branch
2. **Write tests** for new functionality
3. **Follow Rust conventions** (use `cargo fmt` and `cargo clippy`)
4. **Update documentation** as needed
5. **Submit a pull request** with a clear description

### Areas for Contribution

- Search functionality (find in terminal output)
- Split panes (horizontal/vertical splits)
- Performance optimizations
- Additional ANSI sequence support
- MCP (Model Context Protocol) integration
- Image protocol improvements
- More built-in themes
- Plugin system

## Roadmap

### Version 1.1 (Current Development)

- [x] Configuration file support (TOML)
- [x] Session restoration
- [x] Command snippets
- [x] Event hook system
- [x] Custom keybindings
- [x] Environment detection
- [ ] Search functionality (Cmd+F)
- [ ] Custom color themes UI
- [ ] Font family selection UI

### Version 1.2 (Planned)

- [ ] Split panes (horizontal/vertical)
- [ ] Tab groups
- [ ] Hyperlink support (OSC 8)
- [ ] Advanced image protocol support
- [ ] Plugin system

### Version 2.0 (Future)

- [ ] MCP (Model Context Protocol) integration
- [ ] AI agent orchestration features
- [ ] Task tree visualization
- [ ] Session archiving with AI summarization

## License

AgTerm is released under the [MIT License](LICENSE).

```
MIT License

Copyright (c) 2026 AgTerm Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction...
```

See [LICENSE](LICENSE) for full text.

## Acknowledgments

Built with amazing open-source projects:

- [Iced](https://iced.rs/) - Cross-platform GUI framework
- [portable-pty](https://docs.rs/portable-pty) - Cross-platform PTY implementation
- [vte](https://docs.rs/vte) - VTE/ANSI escape sequence parser
- [arboard](https://docs.rs/arboard) - Cross-platform clipboard
- [D2Coding](https://github.com/naver/d2codingfont) - Korean monospace font by Naver

Inspired by modern terminals:

- [Warp](https://www.warp.dev/) - Modern terminal with AI features
- [Alacritty](https://alacritty.org/) - GPU-accelerated terminal emulator
- [iTerm2](https://iterm2.com/) - macOS terminal emulator
- [Kitty](https://sw.kovidgoyal.net/kitty/) - Fast, feature-rich terminal

## Community

- **Issues**: [GitHub Issues](https://github.com/coldwoong-moon/agterm/issues)
- **Discussions**: [GitHub Discussions](https://github.com/coldwoong-moon/agterm/discussions)
- **Repository**: [github.com/coldwoong-moon/agterm](https://github.com/coldwoong-moon/agterm)

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history and release notes.

---

Made with love by the AgTerm team
