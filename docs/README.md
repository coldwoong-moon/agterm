# AgTerm Documentation

Welcome to the AgTerm documentation! This directory contains comprehensive guides for users, developers, and contributors.

## Documentation Files

### For Users

- **[QUICK_START.md](QUICK_START.md)** - Quick start guide with examples
  - Getting started with AgTerm
  - Basic usage patterns
  - Keyboard shortcuts
  - Common workflows

### For Developers

- **[API_DOCUMENTATION.md](API_DOCUMENTATION.md)** - Complete API reference
  - Module documentation
  - Type definitions
  - Usage examples
  - Architecture diagrams
  - Performance characteristics

### For Contributors

- **[../DOCUMENTATION_SUMMARY.md](../DOCUMENTATION_SUMMARY.md)** - Documentation improvements summary
  - What was documented
  - Standards applied
  - Verification process

## Quick Links

### Generated Rustdoc
```bash
cargo doc --open
```

### Key Modules

| Module | Description | Documentation |
|--------|-------------|---------------|
| `terminal::screen` | Terminal buffer with VTE parser | [Rustdoc](../target/doc/agterm/terminal/screen/index.html) |
| `terminal::pty` | PTY session management | [Rustdoc](../target/doc/agterm/terminal/pty/index.html) |
| `terminal_canvas` | GPU-accelerated rendering | [Rustdoc](../target/doc/agterm/terminal_canvas/index.html) |
| Main Application | Iced GUI and state management | [Rustdoc](../target/doc/agterm/index.html) |

### Core Types

| Type | Purpose | Example |
|------|---------|---------|
| `TerminalScreen` | Screen buffer | `TerminalScreen::new(80, 24)` |
| `AnsiColor` | Color representation | `AnsiColor::Rgb(255, 0, 0)` |
| `Cell` | Terminal cell | `Cell { c: 'A', bold: true, .. }` |
| `TerminalCanvas` | Canvas program | `TerminalCanvas::new(&lines, ...)` |

## Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                   AgTerm Application                 │
│                    (Iced GUI)                        │
└───┬─────────────────────────────────────────────┬───┘
    │                                             │
    ▼                                             ▼
┌────────────────┐                         ┌────────────────┐
│  PTY Manager   │◄────────────────────────┤Terminal Canvas │
│                │  Responses (DA/DSR)     │ (GPU Rendering)│
└───┬────────────┘                         └────────────────┘
    │                                             ▲
    ▼                                             │
┌────────────────┐                         ┌────────────────┐
│   Shell I/O    │                         │ Styled Spans   │
│  (bash/zsh)    │                         │   (Rendering)  │
└───┬────────────┘                         └────────────────┘
    │                                             ▲
    ▼                                             │
┌────────────────┐                         ┌────────────────┐
│ Terminal Screen│─────────────────────────▶│ VTE Parser     │
│  (Buffer)      │   ANSI Sequences         │                │
└────────────────┘                         └────────────────┘
```

## Documentation Structure

```
agterm/
├── docs/
│   ├── README.md                  # This file
│   ├── QUICK_START.md             # Quick start guide
│   └── API_DOCUMENTATION.md       # Complete API reference
│
├── src/
│   ├── main.rs                    # Application docs (//!)
│   ├── terminal/
│   │   ├── screen.rs              # Terminal buffer docs (//!)
│   │   └── pty.rs                 # PTY management docs
│   ├── terminal_canvas.rs         # Canvas rendering docs (//!)
│   ├── debug.rs                   # Debug panel
│   └── logging.rs                 # Logging system
│
├── target/doc/agterm/             # Generated rustdoc
│   ├── index.html                 # Main documentation
│   ├── terminal/
│   │   ├── screen/
│   │   │   ├── struct.TerminalScreen.html
│   │   │   ├── enum.AnsiColor.html
│   │   │   └── ...
│   │   └── pty/
│   └── terminal_canvas/
│
└── DOCUMENTATION_SUMMARY.md       # Improvement summary
```

## Getting Started

### 1. View the Documentation
```bash
# Build and open documentation
cargo doc --open

# Or build only
cargo doc --no-deps
```

### 2. Read the Quick Start
Start with [QUICK_START.md](QUICK_START.md) for practical examples.

### 3. Explore the API
Dive into [API_DOCUMENTATION.md](API_DOCUMENTATION.md) for comprehensive reference.

### 4. Check the Source
Read the source code - it's well-documented with inline comments!

## Key Features Documented

### Terminal Screen Module
✅ VTE parser integration
✅ ANSI color system (16/256/RGB)
✅ Wide character handling (CJK/emoji)
✅ Mouse reporting modes
✅ Alternate screen buffer
✅ OSC sequences (title, CWD, clipboard)
✅ Scrollback management
✅ Resize handling

### Terminal Canvas Module
✅ Virtual scrolling
✅ Geometry caching
✅ Streaming mode detection
✅ Text selection
✅ Cursor rendering
✅ Font scaling
✅ Performance optimizations

### Main Application
✅ Tab management
✅ Keyboard shortcuts
✅ Control signals
✅ Message flow
✅ Theme system
✅ Debug panel

## Code Examples

### Basic Terminal Usage
```rust
use agterm::terminal::screen::TerminalScreen;

// Create terminal
let mut screen = TerminalScreen::new(80, 24);

// Process output
screen.process(b"\x1b[31mRed text\x1b[0m\n");

// Get content
let lines = screen.get_all_lines();
```

### Color Handling
```rust
use agterm::terminal::screen::AnsiColor;

// Different color formats
let colors = vec![
    AnsiColor::Indexed(1),           // ANSI red
    AnsiColor::Palette256(208),      // Orange
    AnsiColor::Rgb(123, 45, 67),     // Custom RGB
];

// Convert to Iced color
for color in colors {
    let iced_color = color.to_color();
    // Use for rendering...
}
```

### Canvas Rendering
```rust
use agterm::terminal_canvas::*;

let canvas = TerminalCanvas::new(&lines, version, color, font)
    .with_cursor(CursorState {
        row: 10,
        col: 5,
        style: CursorStyle::Block,
        visible: true,
        blink_on: true,
    })
    .with_font_size(14.0);
```

## Performance

### Optimizations Documented
- Virtual scrolling (only visible lines)
- Geometry caching (text reuse)
- Span merging (reduce draw calls)
- Streaming detection (cache bypass)
- Dynamic tick rate (60fps → 5fps)

### Monitoring
Press `F12` to open the debug panel and view:
- Frame rate (FPS)
- PTY read statistics
- Input state
- Performance metrics

## Contributing

When contributing to AgTerm:

1. **Add rustdoc comments** for public APIs (`///`)
2. **Include examples** in documentation
3. **Update this documentation** when adding features
4. **Run `cargo doc`** to verify documentation builds
5. **Check for warnings** with `cargo rustdoc -- -D missing_docs`

### Documentation Style

```rust
/// Brief one-line description.
///
/// More detailed explanation of what this does,
/// how it works, and when to use it.
///
/// # Examples
///
/// ```
/// use agterm::terminal::screen::TerminalScreen;
///
/// let screen = TerminalScreen::new(80, 24);
/// ```
///
/// # Panics
///
/// Panics if... (if applicable)
pub fn documented_function() {
    // implementation
}
```

## Troubleshooting

### Documentation Won't Build
```bash
# Clean and rebuild
cargo clean
cargo doc --no-deps
```

### Missing Documentation
```bash
# Check for warnings
cargo rustdoc -- -D missing_docs
```

### Broken Links
```bash
# Find unresolved links
cargo doc --no-deps 2>&1 | grep "unresolved link"
```

## Resources

### External Documentation
- [Rustdoc Book](https://doc.rust-lang.org/rustdoc/)
- [Iced GUI Framework](https://docs.rs/iced/)
- [VTE Specification](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html)
- [ANSI Escape Codes](https://en.wikipedia.org/wiki/ANSI_escape_code)

### Related Projects
- [Alacritty](https://github.com/alacritty/alacritty) - GPU-accelerated terminal
- [Warp](https://www.warp.dev/) - Modern terminal (inspiration)
- [vte crate](https://docs.rs/vte/) - VTE parser library

## Future Documentation

### Planned Additions
- **ARCHITECTURE.md** - Deep dive into system design
- **PERFORMANCE.md** - Benchmarks and optimization guide
- **TESTING.md** - Testing strategies and patterns
- **CONTRIBUTING.md** - Full contribution guidelines

### Enhancement Ideas
- Video tutorials
- Interactive examples
- Architecture decision records (ADRs)
- API stability guarantees

## Questions?

For questions about the documentation:
1. Check the [QUICK_START.md](QUICK_START.md) guide
2. Read the [API_DOCUMENTATION.md](API_DOCUMENTATION.md) reference
3. Explore the generated rustdoc (`cargo doc --open`)
4. Read the inline source code documentation

## License

Documentation is licensed under the same terms as AgTerm:
MIT OR Apache-2.0

---

**Last Updated:** 2026-01-18
**Documentation Version:** 1.0
**AgTerm Version:** 1.0.0
