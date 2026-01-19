# AgTerm

AI Agent Terminal - Native GPU-accelerated terminal emulator with AI orchestration and modern features.

AgTerm is a next-generation terminal emulator built in Rust with GPU-accelerated rendering. It combines the power of a traditional terminal with modern productivity features like session management, command automation, and AI agent integration via the Model Context Protocol (MCP).

## Screenshots

Desktop screenshots and demo GIFs are available in `/assets/screenshots/` directory.

## Features

### Core Terminal
- **GPU-accelerated rendering** with choice of GUI frameworks
  - Iced: Traditional, fully-featured tab-based interface
  - Floem: Modern, reactive pane-based layout (experimental)
- **Full Unicode support** including Korean/CJK (D2Coding font)
- **True color** (24-bit) and 256-color ANSI support
- **OSC 8 hyperlinks** with Ctrl+Click to open
- **Terminal bell** with sound and visual notifications
- **IME input** for Korean/CJK text in Raw mode

### Session Management
- **Multi-tab interface** with drag-to-reorder
- **Session persistence** with automatic save/restore
- **Workspace system** for session organization
- **Tab groups** with collapse/expand functionality
- **Split panes** (horizontal/vertical with resize)

### Productivity Features
- **Command palette** (Cmd+Shift+P on macOS, Ctrl+Shift+P on Linux/Windows)
- **Fuzzy finder** for quick action search
- **Command completion** with history suggestions
- **Input macros** for command automation
- **Snippets** with template variables
- **Bookmarks** for frequently used commands
- **Directory history** with frecency ranking

### Shell Integration
- **OSC 7 parsing** for automatic working directory tracking
- **Integration scripts** for bash, zsh, and fish shells
- **Git status** display in prompts
- **Automatic session restoration** from previous state

### Advanced Features
- **Terminal recording** with playback functionality
- **Diff viewer** using Myers algorithm for output comparison
- **Real-time output filters** for text processing
- **Terminal broadcast** to multiple sessions simultaneously
- **Link handler** for URL detection and opening

### Customization & Themes
- **8 built-in themes** (Warp Dark, Dracula, Nord, Solarized, etc.)
- **Theme editor** for creating custom color schemes
- **Profile system** with inheritance and templates
- **Configurable keyboard shortcuts** and bindings

### Accessibility
- **WCAG AA/AAA** contrast compliance
- **Screen reader** support
- **High contrast** theme options
- **Reduced motion** mode
- **Keyboard-only** navigation

### Internationalization
- **6 languages** (English, Korean, Japanese, Chinese, German, French)
- **RTL support** for Arabic and Hebrew text
- **Locale-aware** number and date formatting

### Developer Tools
- **Debug panel** (Cmd+D on macOS, Ctrl+D on Linux/Windows)
- **Performance monitor** with rendering statistics
- **tracing** integration for structured logging
- **Statistics dashboard** for terminal metrics
- **MCP (Model Context Protocol)** client for AI agent integration

## Requirements

- macOS 10.15+ / Linux / Windows 10+
- Rust 1.75+

## Building

### Prerequisites

- Rust 1.75 or later
- macOS 10.15+, Linux (most distributions), or Windows 10+

For Linux systems, you may need to install development headers:

```bash
# Ubuntu/Debian
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev

# Fedora
sudo dnf install libxcb-devel libxkbcommon-devel

# Arch
sudo pacman -S libxcb xcb-util
```

### Default Build (Iced GUI)

Iced provides the main, fully-featured tab-based terminal interface with comprehensive feature support.

```bash
# Debug build
cargo build

# Release build with optimizations
cargo build --release

# Run application
cargo run --release

# Run with debug panel enabled
AGTERM_DEBUG=1 cargo run --release
```

### Alternative Build (Floem GUI)

Floem is a modern, reactive GUI framework providing an experimental pane-based interface with advanced layout capabilities.

```bash
# Build Floem version
cargo build --bin agterm-floem --features floem-gui --no-default-features

# Run Floem version
cargo run --bin agterm-floem --features floem-gui --no-default-features

# Release build with optimizations
cargo build --release --bin agterm-floem --features floem-gui --no-default-features

# Run with debug panel enabled
AGTERM_DEBUG=1 cargo run --release --bin agterm-floem --features floem-gui --no-default-features
```

### Testing

```bash
# Run full test suite
cargo test --lib

# Run specific test module
cargo test pty::

# Run tests with output
cargo test --lib -- --nocapture

# Run benchmarks
cargo bench
```

### Installation

```bash
# Install to ~/.cargo/bin/
cargo install --path .

# Run installed binary
agterm                                    # Iced GUI (default)
agterm-floem                              # Floem GUI

# For bash/zsh, add to ~/.bashrc or ~/.zshrc:
# export PATH="$HOME/.cargo/bin:$PATH"
```

### macOS App Bundle

Create a native macOS application bundle:

```bash
cargo install cargo-bundle

# Create .app bundle for Iced
cargo bundle --release

# Create .app bundle for Floem
cargo bundle --release --bin agterm-floem

# The bundle will be in target/release/bundle/osx/
```

## Configuration

Configuration files are stored in `~/.config/agterm/`:

- `config.toml` - Main application configuration
- `themes/` - Custom theme files
- `profiles/` - Terminal profile configurations
- `keybindings.toml` - Custom keyboard shortcut mappings

### Basic Configuration

Example `~/.config/agterm/config.toml`:

```toml
[general]
default_shell = "/bin/zsh"
save_session_on_exit = true
restore_session_on_startup = true

[appearance]
font_size = 14.0
font_family = "D2Coding"
theme = "warp_dark"

[features]
enable_recording = true
enable_search = true
ime_support = true

[keybindings]
new_tab = "Cmd+T"
close_tab = "Cmd+W"
command_palette = "Cmd+Shift+P"
debug_panel = "Cmd+D"
```

For advanced configuration options, see [API Documentation](docs/API_DOCUMENTATION.md).

## Keyboard Shortcuts

### Iced GUI (Default)

| Action | macOS | Linux/Windows |
|--------|-------|---------------|
| **Tab Management** | | |
| New Tab | Cmd+T | Ctrl+T |
| Close Tab | Cmd+W | Ctrl+W |
| Next Tab | Cmd+] | Ctrl+Tab |
| Previous Tab | Cmd+[ | Ctrl+Shift+Tab |
| **Search & Input** | | |
| Command Palette | Cmd+Shift+P | Ctrl+Shift+P |
| Find | Cmd+F | Ctrl+F |
| Find Previous | Cmd+G | Ctrl+Shift+G |
| Find Next | Cmd+Shift+G | Ctrl+G |
| **Appearance** | | |
| Font Size Increase | Cmd+= | Ctrl+= |
| Font Size Decrease | Cmd+- | Ctrl+- |
| Font Size Reset | Cmd+0 | Ctrl+0 |
| **Debug & Tools** | | |
| Debug Panel | Cmd+D | Ctrl+D |
| Clear Terminal | Cmd+K | Ctrl+K |
| **Terminal Control** | | |
| Copy Selection | Cmd+C | Ctrl+Shift+C |
| Paste from Clipboard | Cmd+V | Ctrl+Shift+V |

### Floem GUI (Alternative)

The Floem GUI provides enhanced pane management with the following shortcuts:

| Action | macOS | Linux/Windows |
|--------|-------|---------------|
| **Pane Management** | | |
| Split Pane Vertically | Cmd+D | Ctrl+D |
| Split Pane Horizontally | Cmd+E | Ctrl+E |
| Close Focused Pane | Cmd+Shift+W | Ctrl+Shift+W |
| Next Pane | Cmd+] | Ctrl+Tab |
| Previous Pane | Cmd+[ | Ctrl+Shift+Tab |
| **Appearance** | | |
| Increase Font Size | Cmd+= | Ctrl+= |
| Decrease Font Size | Cmd+- | Ctrl+- |
| Reset Font Size | Cmd+0 | Ctrl+0 |
| Toggle Theme | Cmd+T | Ctrl+T |
| **Search** | | |
| Find | Cmd+F | Ctrl+F |
| Find Previous | Cmd+Shift+G | Ctrl+Shift+G |
| Find Next | Cmd+G | Ctrl+G |

All keyboard shortcuts can be customized in `~/.config/agterm/keybindings.toml`.

For complete Floem documentation, see [Floem GUI Guide](docs/FLOEM_GUI.md).

## Documentation

Complete documentation is available in the `docs/` directory:

### User Guides
- [Quick Start Guide](docs/QUICK_START.md) - Getting started in 5 minutes
- [Floem GUI Guide](docs/FLOEM_GUI.md) - Comprehensive Floem interface guide
- [Theme System](docs/THEMES.md) - Customizing appearance with themes
- [Keyboard Shortcuts Reference](docs/KEYBOARD_SHORTCUTS.md) - All available shortcuts

### Features
- [Session Restoration](docs/SESSION_RESTORATION.md) - Session persistence and recovery
- [Terminal Recording](docs/RECORDING.md) - Recording and playback functionality
- [Automation System](docs/AUTOMATION.md) - Automate terminal workflows
- [Diff Viewer](docs/DIFF_VIEWER.md) - Compare terminal outputs
- [Shell Integration](docs/SHELL_INTEGRATION.md) - Setup integration with bash/zsh/fish
- [Link Handler](docs/LINK_HANDLER.md) - URL detection and handling

### Developer Resources
- [API Documentation](docs/API_DOCUMENTATION.md) - Complete API reference
- [Architecture & Implementation](docs/FLOEM_IMPLEMENTATION_NOTES.md) - Technical architecture
- [Development Phases](docs/FLOEM_DEVELOPMENT_PHASES.md) - Development history and phases
- [Documentation Index](docs/INDEX.md) - Master index of all documentation

### Quick References
- [Recording Quick Reference](docs/RECORDING_QUICKREF.md) - Quick recording tips
- [Automation Quick Start](docs/AUTOMATION_QUICKSTART.md) - Automation examples
- [Hooks Quick Start](docs/HOOKS_QUICKSTART.md) - Event hook system
- [Menu Quick Start](docs/MENU_QUICKSTART.md) - Menu configuration

### Performance & Debugging
- [Benchmarks](docs/BENCHMARKS.md) - Performance metrics and results
- [Environment Detection](docs/ENVIRONMENT_DETECTION.md) - Shell environment setup

For more information, see [Documentation Index](docs/INDEX.md).

## Project Stats

- **100,000+** lines of Rust code
- **69** source files
- **1,078** tests passing
- **49** feature modules
- **8** built-in themes
- **6** language support

## GUI Framework Comparison

### Iced GUI (Default)
- **Status**: Stable and production-ready
- **Rendering**: CPU-based (tiny-skia)
- **Interface**: Tab-based traditional layout
- **Performance**: Excellent for most use cases
- **Features**: Complete feature set
- **Recommendation**: Use this for daily work

### Floem GUI (Alternative)
- **Status**: Experimental/Advanced
- **Rendering**: GPU-accelerated
- **Interface**: Pane-based modern layout
- **Performance**: Optimized for large workloads
- **Features**: Expanding feature set
- **Recommendation**: Test for advanced workflows

Choose the GUI that best fits your workflow:
- **Traditional workflows**: Use Iced (default)
- **Pane-heavy workflows**: Try Floem
- **Testing/Development**: Try both

## Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `AGTERM_DEBUG` | Enable debug panel on startup | `AGTERM_DEBUG=1` |
| `AGTERM_LOG` | Set log filter level | `AGTERM_LOG=agterm=debug` |
| `RUST_LOG` | Set Rust tracing filter | `RUST_LOG=agterm=trace,iced=debug` |

## Troubleshooting

### Application won't start
1. Check if all dependencies are installed: `cargo build`
2. Try resetting configuration: `rm -rf ~/.config/agterm/`
3. Enable debug logging: `AGTERM_LOG=debug cargo run --release`

### Rendering issues
- On Linux: Install required development headers (see Prerequisites)
- Try switching GUI frameworks (Iced ↔ Floem)
- Check your GPU drivers are up to date

### IME/Input problems
- Ensure IME is enabled: Check `config.toml` `ime_support = true`
- Try toggling Raw mode with the UI menu
- Check shell environment: `echo $LANG`

For more help, see [Quick Start Guide](docs/QUICK_START.md).

## Roadmap & Future Plans

### Short Term (Next Release)
- Stabilize Floem GUI with additional features
- Add search/find functionality to Floem GUI
- Improve IME input handling across platforms
- Performance optimizations for large scrollback buffers

### Medium Term (Future Releases)
- **Plugin System**: Extensible plugin API for custom features
- **Web Version**: Browser-based terminal using WebAssembly
- **Network Features**: SSH session management and synchronization
- **Advanced Automation**: Conditional execution and branching in automation scripts
- **Enhanced Theming**: Theme preview and live editing

### Long Term Vision
- **Cloud Integration**: Session sync across devices
- **AI Assistant Integration**: Improved MCP client with command suggestions
- **Collaborative Sessions**: Real-time collaboration between terminals
- **Mobile Support**: iOS/Android terminal client
- **Terminal Multiplexing**: Native tmux-like features

### Known Limitations
- Floem GUI is experimental; some features may not be complete
- Windows support is available but less tested than macOS/Linux
- Audio bell requires audio device; may need configuration on some systems
- Plugin system API is still being designed

### Contributing

We welcome contributions! Areas we're looking for help:
- **Platform Support**: Testing and fixing platform-specific issues
- **Performance**: Benchmarking and optimization
- **Documentation**: Improving guides and API documentation
- **Features**: Implementing requested features
- **Testing**: Writing and improving test coverage

See [Development Phases](docs/FLOEM_DEVELOPMENT_PHASES.md) for technical details.

## Support

- GitHub Issues: Report bugs and request features
- Documentation: See [docs/](docs/) directory for guides
- Quick Start: [Getting started in 5 minutes](docs/QUICK_START.md)
- API Reference: [Full API documentation](docs/API_DOCUMENTATION.md)

## License

MIT License - See LICENSE file for details.

### Copyright

Copyright (c) 2024 coldwoong-moon

AgTerm is free software: you can redistribute it and/or modify it under the
terms of the MIT License.

---

**Last Updated**: January 2026
**Current Version**: 1.0.0
**Status**: Actively Maintained
