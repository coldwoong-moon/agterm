# Changelog

All notable changes to AgTerm will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-01-18

### Added

#### Terminal Emulation Core
- VTE-based ANSI escape code parser for enhanced terminal support
- Alternate screen buffer support for full-screen applications
- Cursor rendering with customizable styles (block, underline, beam)
- Wide character (CJK) support with proper width calculation
- Unicode width handling for international text
- SGR (Select Graphic Rendition) terminal sequences
- Device attribute reporting (DA, CSI c)
- Full streaming terminal mode with PTY integration
- Virtual scrolling with Canvas widget for performance

#### Input & Interaction
- IME (Input Method Editor) support for Korean/CJK characters in Raw mode
- Raw mode for interactive CLI applications
- Key repeat system with configurable timing (500ms delay, 30ms rate)
- Comprehensive keyboard shortcut system (28 default keybindings)
- Mouse reporting support (button clicks, motion, scroll)
- Mouse wheel scrolling in terminal buffer
- Customizable keymap system with conflict detection
- ~/.config/agterm/keybindings.toml configuration support

#### Advanced Features
- Event hook system with 4 event types (CommandComplete, DirectoryChange, OutputMatch, Bell)
- Hook actions: Notify, RunCommand, PlaySound, Custom
- HookManager for lifecycle management with ~/.config/agterm/hooks.toml
- Profile system with shell, environment, theme, and font settings
- Session restoration on startup with automatic save on exit
- Snippet system with 24 built-in snippets (git, docker, k8s, cargo)
- ~/.config/agterm/snippets.toml storage
- Environment detection (SSH, containers, terminal multiplexers)
- Pane system foundation with horizontal/vertical splits
- Pane navigation with Cmd+Shift+H/| shortcuts

#### UI/UX
- Multi-tab management with create, close, and navigation
- Tab titles from shell working directory
- Visual bell notification system
- Sound-based bell with rodio audio playback (800Hz tone)
- Configurable bell style (visual/sound/both/none)
- Search functionality with highlighted results
- Scrollbar with visual indicators
- Debug panel with performance metrics
- Performance graphs (FPS, memory, PTY I/O rate)
- Unicode sparkline visualization for metrics
- Log viewer in debug panel

#### Hyperlinks & URLs
- OSC 8 hyperlink support (clickable terminal hyperlinks)
- Regex-based URL detection (http/https/file)
- Click-to-open URLs in default browser
- Cyan underline highlighting for detected URLs

#### Configuration & Theming
- Comprehensive config system with TOML format
- Theme support with color customization
- Font configuration with size and family settings
- Scrollback buffer size configuration
- Refresh rate and adaptive settings
- Config persistence in ~/.config/agterm/

#### Clipboard
- System clipboard integration with arboard
- Copy/paste support with keyboard shortcuts
- Shell integration for clipboard operations

#### Logging & Debugging
- Tracing-based structured logging system
- PTY I/O content previews at TRACE level
- AGTERM_LOG environment variable control
- Daily log rotation with tracing-appender
- Cross-platform log path support

#### Image Protocol Foundation
- ImageProtocol enum (Sixel, Kitty, ITerm2)
- ImageData struct with dimensions and protocol support
- Cell image field for future rendering
- Configuration options: images.enabled, images.max_size

#### Build & Distribution
- Comprehensive release packaging for macOS, Linux, Windows
- Package manager support (Homebrew, cargo-binstall)
- macOS .app bundle configuration
- Windows and Linux icon support
- CI/CD pipeline with GitHub Actions
- Cross-platform build automation

#### Testing & Benchmarks
- Comprehensive test suite with 368+ tests
- Environment detection tests (22)
- Snippet system tests (8)
- Profile management tests (6)
- URL detection tests (8)
- Pane layout tests (6)
- Integration tests for clipboard and shell integration
- Rendering benchmarks with criterion
- Terminal emulation benchmarks

### Changed

#### Architecture
- Complete architecture overhaul with Iced GUI framework
- Migration from wgpu to tiny-skia CPU renderer for better text performance
- Switched to full streaming terminal mode
- Refactored terminal emulator with VTE parser

#### Performance
- Rendering performance improved from 11fps to 66fps (6x improvement)
- Optimized repetitive style closures
- Pre-allocated Vec buffers to reduce allocations
- String interning for URLs with Arc<String> (25-48% memory reduction)
- Memory statistics monitoring
- Automatic cleanup of unused strings
- Virtual scrolling implementation

#### Input Handling
- All keyboard input now sent directly to PTY
- Improved Raw mode for interactive applications

### Fixed

- Platform-appropriate shell selection for Windows compatibility
- Clippy warnings and formatting issues resolved
- Unknown lints allowed for cross-version clippy compatibility
- GitHub Actions rust-toolchain action name corrected
- PTY spawning tests skip on Windows CI
- Cross-platform log path documentation
- Cursor visibility control issues
- Terminal resize handling improvements

### Performance

- Rendering optimization achieving 6x performance improvement (11fps → 66fps)
- Memory usage reduced by 25-48% through string interning
- Virtual scrolling with Canvas widget for efficient rendering
- Optimized Vec allocations with pre-allocation
- Repetitive closure optimizations
- Adaptive settings based on environment detection (SSH, containers, multiplexers)

### Documentation

- Comprehensive README with installation instructions
- API documentation for terminal sequences
- Benchmarking results and renderer choice documentation (tiny-skia vs wgpu)
- Cross-platform log path documentation
- Hook system documentation
- Keymap customization guide
- Mouse reporting documentation
- Alternate screen buffer usage

---

## Project Philosophy

AgTerm is designed to be a native, GPU-accelerated terminal emulator with AI agent orchestration capabilities. Version 1.0.0 establishes the foundation with:

- **Robust Terminal Emulation**: VTE parser, ANSI sequences, wide character support
- **Modern UI Framework**: Iced with tiny-skia renderer for optimal performance
- **Performance Focus**: 66fps rendering, memory optimization (25-48% reduction)
- **Extensibility**: Hooks, snippets, profiles, keymaps, panes
- **Cross-Platform Support**: macOS, Linux, Windows with native builds
- **Comprehensive Testing**: 368+ tests covering all major features
- **Developer Experience**: Debug panel, performance graphs, structured logging

### Future Roadmap

Planned features for upcoming releases:

- Async operations with tokio for improved responsiveness
- MCP (Model Context Protocol) support for AI agent integration
- Image rendering (Sixel, Kitty, ITerm2 protocols)
- Advanced pane management (tree layouts, saved layouts)
- Plugin system for custom extensions
- Remote session management

---

## Statistics

- **Total Tests**: 368+
- **Performance**: 66fps rendering (6x improvement)
- **Memory Efficiency**: 25-48% reduction through optimization
- **Supported Platforms**: macOS, Linux, Windows
- **Built-in Snippets**: 24 (git, docker, k8s, cargo, common)
- **Default Keybindings**: 28
- **Hook Event Types**: 4
- **Hook Action Types**: 4

---

[1.0.0]: https://github.com/coldwoong-moon/agterm/releases/tag/v1.0.0
