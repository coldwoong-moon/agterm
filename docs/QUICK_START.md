# AgTerm Quick Start Guide

## Documentation Overview

AgTerm now has comprehensive rustdoc documentation. Access it in three ways:

### 1. View Online Documentation
```bash
cargo doc --open
```

This will build and open the documentation in your browser at `target/doc/agterm/index.html`

### 2. Key Documentation Files

#### API Reference
- **target/doc/agterm/terminal/screen/index.html** - Terminal screen buffer
- **target/doc/agterm/terminal_canvas/index.html** - GPU rendering
- **docs/API_DOCUMENTATION.md** - Comprehensive API guide

#### Module Documentation
- **src/terminal/screen.rs** - VTE parser and buffer management
- **src/terminal_canvas.rs** - Canvas rendering with virtual scrolling
- **src/main.rs** - Application architecture and message flow

### 3. Code Examples

#### Create a Terminal Screen
```rust
use agterm::terminal::screen::TerminalScreen;

// Create 80x24 terminal
let mut screen = TerminalScreen::new(80, 24);

// Process ANSI output
screen.process(b"\x1b[31mHello\x1b[0m World\n");

// Get rendered lines
let all_lines = screen.get_all_lines();
let (cursor_row, cursor_col) = screen.cursor_position();
```

#### Color Handling
```rust
use agterm::terminal::screen::AnsiColor;

// Standard ANSI color
let red = AnsiColor::Indexed(1);

// 256-color palette
let orange = AnsiColor::Palette256(208);

// True color RGB
let custom = AnsiColor::Rgb(123, 45, 67);

// Convert to Iced color for rendering
let iced_color = red.to_color();
```

#### Canvas Rendering
```rust
use agterm::terminal_canvas::{TerminalCanvas, CursorState, CursorStyle};

let cursor = CursorState {
    row: 10,
    col: 5,
    style: CursorStyle::Block,
    visible: true,
    blink_on: true,
};

let canvas = TerminalCanvas::new(
    &styled_lines,
    content_version,
    default_color,
    mono_font,
)
.with_cursor(cursor)
.with_font_size(14.0);
```

## Architecture Overview

```
User Input → Message → AgTerm State
                            ↓
                       PTY Manager → Shell
                            ↑
                       PTY Output
                            ↓
                       VTE Parser
                            ↓
                    Terminal Screen (Buffer)
                            ↓
                    StyledSpans (Rendering)
                            ↓
                    Terminal Canvas
                            ↓
                    GPU Rendering (Iced)
```

## Key Concepts

### Terminal Screen
- **Buffer**: Current visible lines (Vec<Vec<Cell>>)
- **Scrollback**: Historical lines (VecDeque<Vec<Cell>>)
- **Alternate Buffer**: For fullscreen apps (vim, less)
- **VTE Parser**: Processes ANSI escape sequences

### Canvas Rendering
- **Virtual Scrolling**: Only renders visible lines
- **Geometry Caching**: Reuses cached text geometry
- **Streaming Mode**: Bypasses cache during rapid output
- **Text Selection**: Mouse-based selection support

### Color System
1. **Indexed (0-15)**: Basic ANSI colors
2. **Palette256 (0-255)**: Extended palette
3. **RGB**: 24-bit true color

### Wide Characters
- Korean/CJK characters occupy 2 cells
- First cell: `wide = true`, contains character
- Second cell: `placeholder = true`, maintains alignment

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+T` | New tab |
| `Cmd+W` | Close tab |
| `Cmd+[` / `]` | Previous/Next tab |
| `Cmd+1-5` | Jump to tab |
| `Cmd+C` | Copy selection (or send interrupt if no selection) |
| `Cmd+V` | Paste |
| `Cmd+D` / `F12` | Toggle debug panel |
| `Cmd+` / `-` / `0` | Increase/Decrease/Reset font size |
| `Ctrl+C` | Send SIGINT (interrupt) |
| `Ctrl+D` | Send EOF |
| `Ctrl+Z` | Send SIGTSTP (suspend) |

## Development Workflow

### Building
```bash
cargo build --release
```

### Running
```bash
cargo run
```

### Testing
```bash
cargo test
```

### Documentation
```bash
# Build docs
cargo doc --no-deps

# Build and open in browser
cargo doc --open

# Check for missing docs
cargo rustdoc -- -D missing_docs
```

## Important Modules

### `terminal::screen`
- `TerminalScreen` - Main terminal buffer
- `AnsiColor` - Color representation
- `Cell` - Single terminal cell
- `MouseMode` / `MouseEncoding` - Mouse reporting

### `terminal::pty`
- `PtyManager` - Manages PTY sessions
- Platform-specific PTY spawning

### `terminal_canvas`
- `TerminalCanvas` - Canvas rendering program
- `TerminalCanvasState` - Canvas state and caching
- `Selection` - Text selection state
- `CursorState` - Cursor rendering

### Main Application
- `AgTerm` - Main app state
- `TerminalTab` - Individual terminal tab
- `Message` - Event messages
- `theme` - Color theme

## Performance Features

### Optimizations
1. **Virtual Scrolling**: Only visible lines rendered
2. **Geometry Caching**: Cached text geometry reused
3. **Span Merging**: Consecutive identical styles merged
4. **Streaming Detection**: Automatic cache bypass during rapid output
5. **Dynamic Tick**: Variable update frequency (60fps → 5fps)

### Monitoring
- Press `F12` or `Cmd+D` to open debug panel
- Shows:
  - FPS (frames per second)
  - PTY read metrics
  - Input state
  - Performance graphs

## Common Patterns

### Processing Terminal Output
```rust
// Read from PTY
let output = pty_manager.read(&session_id)?;

// Process through screen
screen.process(&output);

// Get responses to send back (DA, DSR, etc.)
let responses = screen.take_pending_responses();
for response in responses {
    pty_manager.write(&session_id, response.as_bytes())?;
}

// Render
let lines = screen.get_all_lines();
let styled_spans = lines_to_styled_spans(&lines);
```

### Handling Resize
```rust
// User resizes window
let new_cols = (width / char_width) as usize;
let new_rows = (height / line_height) as usize;

// Resize PTY
pty_manager.resize(&session_id, new_rows as u16, new_cols as u16)?;

// Resize screen buffer (content-preserving)
screen.resize(new_cols, new_rows);
```

### Text Selection
```rust
// Start selection on mouse down
let selection = Selection::new(row, col);

// Update on drag
selection.end = (new_row, new_col);

// Extract text
let text = get_selected_text(&lines, &selection);

// Copy to clipboard
clipboard.set_text(text)?;
```

## Troubleshooting

### Documentation Not Building
```bash
# Clean and rebuild
cargo clean
cargo doc --no-deps
```

### Missing Type Documentation
Check that rustdoc comments use `///` (not `//`):
```rust
/// This is a doc comment
pub struct MyType;

// This is a regular comment (not shown in docs)
```

### Broken Links
```bash
# Check for unresolved links
cargo doc --no-deps 2>&1 | grep "unresolved link"
```

## Resources

- **Rustdoc Book**: https://doc.rust-lang.org/rustdoc/
- **Iced Documentation**: https://docs.rs/iced/
- **VTE Spec**: https://invisible-island.net/xterm/ctlseqs/ctlseqs.html
- **ANSI Escape Codes**: https://en.wikipedia.org/wiki/ANSI_escape_code

## Next Steps

1. Explore the generated documentation: `cargo doc --open`
2. Read the API documentation: `docs/API_DOCUMENTATION.md`
3. Try the code examples above
4. Check out the source code with inline documentation
5. Contribute improvements following the documented patterns

---

Happy hacking! 🚀
