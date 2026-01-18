# AgTerm

AI Agent Terminal - Native GPU-accelerated terminal emulator with modern features.

## Features

### Core Terminal
- **GPU-accelerated rendering** with Iced framework
- **Full Unicode support** including Korean/CJK (D2Coding font)
- **True color** (24-bit) and 256-color support
- **OSC 8 hyperlinks** with Ctrl+Click to open
- **Terminal bell** with sound and visual flash
- **IME input** for Korean/CJK in Raw mode

### Session Management
- **Multiple tabs** with drag-to-reorder
- **Session persistence** across restarts
- **Workspace system** for session organization
- **Tab groups** with collapse/expand

### Productivity
- **Command palette** (Cmd/Ctrl+Shift+P)
- **Fuzzy finder** for quick actions
- **Command completion** with history
- **Input macros** for automation
- **Snippets** with template expansion
- **Bookmarks** for frequent commands

### Shell Integration
- **OSC parsing** for cwd tracking
- **bash/zsh/fish** integration scripts
- **Git status** in prompt
- **Directory history** with frecency

### Advanced Features
- **Split panes** (horizontal/vertical)
- **Terminal recording** and playback
- **Diff viewer** with Myers algorithm
- **Output filters** for real-time processing
- **Terminal broadcast** to multiple sessions

### Customization
- **Theme system** with 8 presets
- **Theme editor** for custom themes
- **Profile system** with inheritance
- **Plugin API** with permissions

### Accessibility
- **WCAG AA/AAA** contrast compliance
- **Screen reader** support
- **High contrast** themes
- **Reduced motion** mode
- **Keyboard-only** navigation

### Internationalization
- **6 languages** (en, ko, ja, zh, de, fr)
- **RTL support** for Arabic/Hebrew
- **Locale-aware** formatting

### Developer Tools
- **Debug panel** (Cmd+D or F12)
- **Performance monitor**
- **tracing** logging integration
- **Statistics dashboard**

## Requirements

- macOS 10.15+ / Linux / Windows 10+
- Rust 1.75+

## Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test --lib

# Run application
cargo run --release
```

## Configuration

Configuration file: `~/.config/agterm/config.toml`

```toml
[general]
default_shell = "/bin/zsh"

[appearance]
font_size = 14.0
theme = "warp_dark"

[keybindings]
new_tab = "Cmd+T"
close_tab = "Cmd+W"
```

## Keyboard Shortcuts

| Action | macOS | Linux/Windows |
|--------|-------|---------------|
| New Tab | Cmd+T | Ctrl+T |
| Close Tab | Cmd+W | Ctrl+W |
| Next Tab | Cmd+] | Ctrl+Tab |
| Previous Tab | Cmd+[ | Ctrl+Shift+Tab |
| Command Palette | Cmd+Shift+P | Ctrl+Shift+P |
| Debug Panel | Cmd+D | Ctrl+D |
| Font Size + | Cmd+= | Ctrl+= |
| Font Size - | Cmd+- | Ctrl+- |

## Project Stats

- **100,000+** lines of Rust code
- **69** source files
- **1,078** tests
- **49** feature modules

## License

MIT License - See LICENSE file for details.
