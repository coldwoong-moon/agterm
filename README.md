# AgTerm

**AI Agent Terminal - Native GPU-Accelerated Terminal Emulator**

AgTerm is a modern terminal emulator built with Rust and the Iced GUI framework, featuring native GPU acceleration, comprehensive ANSI/VTE support, and Korean/CJK input handling. Designed for both daily terminal use and AI agent workflows.

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
- **Smart Cursor Blinking**: Alacritty-style cursor with 530ms blink interval
- **Text Selection & Clipboard**: Mouse-based text selection with copy/paste support
- **Scrollback Buffer**: Unlimited scrollback with virtual scrolling for performance
- **Terminal Bell**: Visual notification for background tabs

### User Interface

- **Modern Dark Theme**: Warp-inspired color scheme with high contrast
- **Dynamic Tab Titles**: Custom tab titles via OSC sequences
- **Bell Notifications**: Visual indicators when background tabs receive bell signals
- **Status Bar**: Shows current shell, mode, and active tab information
- **Debug Panel**: Real-time terminal state inspection (F12 or Cmd+D)
- **Font Size Control**: Dynamically adjust font size (8-24pt)

### Developer Features

- **Debug Panel**: Inspect PTY state, metrics, and logs
- **Comprehensive Logging**: Structured logging with tracing
- **Performance Metrics**: Frame timing and PTY I/O monitoring
- **Dynamic Tick Rate**: Adaptive polling based on activity (5-60 FPS)

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

## Configuration

### Config File Location

AgTerm looks for configuration files in the following locations:

- macOS/Linux: `~/.config/agterm/config.toml`
- Windows: `%APPDATA%\agterm\config.toml`

### Example Configuration

```toml
[general]
default_shell = "/bin/zsh"
font_size = 14.0

[terminal]
scrollback_lines = 10000
cursor_blink_interval_ms = 530

[theme]
name = "dark"

[logging]
level = "info"
log_file = "~/.config/agterm/logs/agterm.log"
```

Note: Configuration system is currently planned. Current version uses hardcoded defaults.

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
- **Dynamic Tick Rate**: Adjusts polling frequency based on activity
- **Content Versioning**: Invalidates cache only when content changes

## Terminal Compatibility

AgTerm aims for high compatibility with modern terminal applications:

- **TERM**: Set to `xterm-256color` by default
- **Shell Integration**: Works with bash, zsh, fish, and other common shells
- **Interactive Apps**: Supports vim, emacs, htop, tmux, and other full-screen applications
- **Color Support**: 256-color palette and 24-bit TrueColor
- **Mouse Reporting**: Basic mouse events for applications that support it

### Known Limitations

- MCP (Model Context Protocol) support is planned but not yet implemented
- Configuration file system is planned but not yet implemented
- Search functionality (Cmd+F) is planned but not yet implemented
- Split panes are not yet implemented
- Custom themes are not yet configurable

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
│   ├── terminal/
│   │   ├── mod.rs
│   │   ├── pty.rs           # PTY management
│   │   ├── screen.rs        # Terminal screen buffer and VTE
│   │   └── ansi_color.rs    # ANSI color parsing
│   ├── terminal_canvas.rs   # GPU-accelerated canvas rendering
│   ├── debug/
│   │   ├── mod.rs
│   │   ├── panel.rs         # Debug panel UI
│   │   └── metrics.rs       # Performance metrics
│   └── logging.rs           # Logging configuration
├── assets/
│   └── fonts/
│       └── D2Coding.ttf     # Embedded Korean font
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

- Configuration file system (TOML-based)
- Search functionality (find in terminal output)
- Split panes (horizontal/vertical splits)
- Custom themes and color schemes
- Performance optimizations
- Additional ANSI sequence support
- MCP (Model Context Protocol) integration

## Roadmap

### Version 1.1 (Current Development)

- [ ] Configuration file support (TOML)
- [ ] Search functionality (Cmd+F)
- [ ] Custom color themes
- [ ] Font family selection

### Version 1.2 (Planned)

- [ ] Split panes (horizontal/vertical)
- [ ] Tab groups
- [ ] Session persistence
- [ ] Hyperlink support (OSC 8)

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

Made with ❤️ by the AgTerm team
