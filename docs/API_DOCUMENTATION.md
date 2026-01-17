# AgTerm API Documentation

## Core Modules

### `terminal::screen` - Terminal Screen Buffer

The heart of AgTerm's terminal emulation, providing VTE-compliant ANSI escape sequence parsing.

#### Key Types

##### `TerminalScreen`

Main terminal screen buffer with VTE parser integration.

**Creating a Terminal:**
```rust
let mut screen = TerminalScreen::new(80, 24);
```

**Processing Data:**
```rust
// Process raw PTY output
screen.process(b"\x1b[31mHello\x1b[0m World\n");

// Get rendered lines
let lines = screen.get_all_lines();
let (row, col) = screen.cursor_position();
```

**Resizing:**
```rust
// Intelligently handles content reflow
screen.resize(120, 30);
```

##### `AnsiColor`

Represents terminal colors in multiple formats:
- `Indexed(u8)`: 16-color ANSI palette (0-15)
- `Palette256(u8)`: 256-color palette (0-255)
- `Rgb(r, g, b)`: 24-bit true color

**Example:**
```rust
use terminal::screen::AnsiColor;

let red = AnsiColor::Indexed(1);
let orange = AnsiColor::Palette256(208);
let custom = AnsiColor::Rgb(123, 45, 67);

// Convert to rendering color
let iced_color = red.to_color();
```

##### `Cell`

A single terminal cell with character and styling.

**Properties:**
- `c: char` - The character
- `fg: Option<AnsiColor>` - Foreground color
- `bold`, `underline`, `italic`, `dim`, `strikethrough` - Text attributes
- `wide`, `placeholder` - Wide character handling (CJK, emoji)

**Wide Character Handling:**
```
Korean "가" (width=2):
Cell 0: { c: '가', wide: true, ... }
Cell 1: { c: ' ', placeholder: true, ... }
```

##### `MouseMode` / `MouseEncoding`

Controls mouse reporting for interactive applications (vim, tmux, etc.):
- `MouseMode::None` - No reporting (default)
- `MouseMode::X10` - Click events only
- `MouseMode::ButtonEvent` - Press, release, drag
- `MouseMode::AnyEvent` - All movements

---

### `terminal_canvas` - GPU-Accelerated Rendering

Virtual scrolling canvas with hardware acceleration via Iced.

#### Key Types

##### `TerminalCanvas`

Canvas program for rendering terminal output.

**Creating:**
```rust
let canvas = TerminalCanvas::new(
    &lines,           // Vec<Vec<StyledSpan>>
    content_version,  // u64 cache key
    default_color,    // Color
    mono_font,        // Font
)
.with_cursor(cursor_state)
.with_font_size(14.0);
```

**Features:**
- **Virtual Scrolling**: Only renders visible lines
- **Geometry Caching**: Reuses cached geometry when content unchanged
- **Streaming Mode**: Bypasses cache during rapid updates (streaming output)
- **Text Selection**: Mouse-based text selection support

##### `TerminalCanvasState`

Manages canvas state including scroll position and caching.

**Methods:**
- `scroll_to_bottom()` - Auto-scroll to latest output
- `invalidate()` - Clear geometry cache (force redraw)

##### `CursorState`

Cursor rendering state:
```rust
CursorState {
    row: 10,
    col: 5,
    style: CursorStyle::Block,  // or Underline, Bar
    visible: true,
    blink_on: true,
}
```

#### Configuration

```rust
use terminal_canvas::config;

// Font sizing (scales proportionally)
let line_height = config::line_height(font_size);
let char_width = config::char_width(font_size);

// Streaming mode detection
const STREAMING_THRESHOLD_MS: u64 = 50;  // Gap between updates
const STREAMING_COUNT_THRESHOLD: u8 = 3;  // Rapid updates to enter mode
```

#### Performance

- **Cache Invalidation**: Smart cache clearing based on change type
- **Span Merging**: Consecutive cells with identical styles merged before rendering
- **Streaming Detection**: Automatic cache bypass during rapid updates (e.g., `cat large_file.txt`)

---

### `main` - Application Architecture

Iced-based GUI application with PTY management.

#### Application Structure

```
AgTerm
├── tabs: Vec<TerminalTab>        # Multiple terminal tabs
├── pty_manager: PtyManager        # PTY session management
├── debug_panel: DebugPanel        # F12 debug overlay
└── font_size: f32                 # Adjustable font (Cmd+/-)
```

#### Message Flow

```
User Input → Message → AgTerm::update() → PTY Write
PTY Output → Tick → VTE Parser → Screen Buffer → Canvas Render
```

#### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+T` | New tab |
| `Cmd+W` | Close tab |
| `Cmd+[`/`]` | Previous/Next tab |
| `Cmd+1-5` | Jump to tab |
| `Cmd+C` | Copy selection (or send interrupt) |
| `Cmd+V` | Paste |
| `Cmd+D` / `F12` | Toggle debug panel |
| `Cmd+`/`-` | Adjust font size |
| `Ctrl+C` | Send interrupt signal (^C) |
| `Ctrl+D` | Send EOF signal (^D) |
| `Ctrl+Z` | Send suspend signal (^Z) |

#### Terminal Modes

**Raw Mode** (default):
- All input goes directly to PTY
- Full streaming terminal support
- IME support for Korean/CJK input
- Application cursor keys (vim, less)

#### Theme System

Warp-inspired dark theme with configurable colors in `theme` module:
- Background: `BG_PRIMARY`, `BG_SECONDARY`
- Text: `TEXT_PRIMARY`, `TEXT_SECONDARY`, `TEXT_MUTED`
- Accents: `ACCENT_BLUE`, `ACCENT_GREEN`, `ACCENT_RED`

---

## Usage Examples

### Basic Terminal Emulation

```rust
use agterm::terminal::screen::TerminalScreen;

// Create terminal
let mut screen = TerminalScreen::new(80, 24);

// Process ANSI sequences
screen.process(b"Hello \x1b[1;31mRed Bold\x1b[0m World\n");

// Get output
let lines = screen.get_all_lines();
let (cursor_row, cursor_col) = screen.cursor_position();
```

### Rendering with Canvas

```rust
use agterm::terminal_canvas::{TerminalCanvas, CursorState, CursorStyle};

let cursor = CursorState {
    row: cursor_row,
    col: cursor_col,
    style: CursorStyle::Block,
    visible: true,
    blink_on: true,
};

let canvas = TerminalCanvas::new(&styled_lines, version, color, font)
    .with_cursor(cursor)
    .with_font_size(14.0);
```

### PTY Integration

```rust
use agterm::terminal::pty::PtyManager;

let pty = PtyManager::new();
let session_id = pty.create_session(24, 80)?;

// Write input
pty.write(&session_id, b"ls -la\n")?;

// Read output
let output = pty.read(&session_id)?;
screen.process(&output);
```

---

## Architecture Diagrams

### Data Flow

```
┌─────────────┐
│ User Input  │
└──────┬──────┘
       │
       ▼
┌─────────────┐     ┌──────────────┐
│ Iced Events │────▶│ AgTerm State │
└─────────────┘     └──────┬───────┘
                           │
                           ▼
                    ┌─────────────┐
                    │ PTY Manager │
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
                    │   PTY I/O   │
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
                    │ VTE Parser  │
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
                    │TerminalScreen│
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
                    │StyledSpans  │
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
                    │   Canvas    │
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
                    │GPU Rendering│
                    └─────────────┘
```

### Screen Buffer Structure

```
┌──────────────────────────────┐
│      Scrollback Buffer       │ ← Historical lines (VecDeque)
│  ┌─────────────────────────┐ │
│  │ Line 1 (oldest)         │ │
│  │ Line 2                  │ │
│  │ ...                     │ │
│  │ Line N (newest)         │ │
│  └─────────────────────────┘ │
└──────────────────────────────┘
┌──────────────────────────────┐
│      Visible Buffer          │ ← Current screen (Vec<Vec<Cell>>)
│  ┌─────────────────────────┐ │
│  │ Row 0                   │ │
│  │ Row 1                   │ │
│  │ ...                     │ │
│  │ Row (rows-1)            │ │
│  │     ▲ Cursor            │ │
│  └─────────────────────────┘ │
└──────────────────────────────┘
┌──────────────────────────────┐
│   Alternate Screen (Option)  │ ← For fullscreen apps
│  (vim, less, htop, etc.)     │
└──────────────────────────────┘
```

---

## Performance Characteristics

### Terminal Screen
- **Scrollback**: O(1) push/pop, O(n) for full render
- **Resize**: O(rows × cols) with intelligent content preservation
- **VTE Parsing**: O(bytes) linear processing

### Canvas Rendering
- **Virtual Scrolling**: Only renders visible lines (O(visible))
- **Geometry Caching**: Reuses cached geometry (O(1) when unchanged)
- **Streaming Mode**: Bypasses cache during rapid updates
- **Span Merging**: Reduces draw calls by merging consecutive identical styles

### Dynamic Tick Optimization
- Active (< 500ms): 60 FPS (16ms tick)
- Medium (< 2s): 20 FPS (50ms tick)
- Idle: 5 FPS (200ms tick)

---

## Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Test specific module
cargo test terminal::screen

# Generate documentation
cargo doc --open
```

---

## Future Enhancements

### Planned Features
- Background color rendering (currently only foreground)
- Reverse video support
- Mouse reporting to PTY
- Bracketed paste mode
- Clipboard integration (OSC 52)
- Hyperlink support (OSC 8)
- Sixel graphics

### Optimization Opportunities
- Incremental rendering (dirty regions)
- GPU-based text rasterization
- Shared glyph atlas
- Zero-copy span rendering

---

## Contributing

When adding features:
1. Add rustdoc comments for all public APIs
2. Include examples in documentation
3. Update this API documentation
4. Add tests for new functionality
5. Run `cargo doc` to verify documentation builds

## License

MIT OR Apache-2.0
