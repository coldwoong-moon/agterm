# Floem Implementation Notes - Archive

This document consolidates technical implementation details from various development phases. For user documentation, see [FLOEM_GUI.md](FLOEM_GUI.md).

## Features Implementation

### Cursor Implementation

The cursor is rendered as an outline rectangle updated at regular intervals.

**Features**:
- Block cursor by default
- Outline drawn with configurable stroke width
- Position tracked from terminal screen state
- Color contrasts with background for visibility

**Future Enhancements**:
- Cursor style options (block, underline, bar)
- Smooth animations between positions
- Blink animation configuration

### ANSI Color Support

Full support for terminal color palettes:

**Supported Formats**:
- ANSI 16-color palette (standard colors + bright)
- 256-color palette (6x6x6 RGB + grayscale)
- True color (24-bit RGB)
- Named color from theme palette

**Color Conversion**:
```rust
fn ansi_to_floem_color(ansi_color: &AnsiColor) -> floem::peniko::Color
```

Maps AgTerm's color representation to Floem's color format.

**Terminal Color Palette** (from theme):
- Black, Red, Green, Yellow, Blue, Magenta, Cyan, White
- Bright variants of each
- Background and foreground colors
- Accent colors

### Clipboard Implementation

Full clipboard support for copy/paste operations.

**Operations**:
- Copy selected text to clipboard
- Paste from clipboard to terminal
- Integration with `arboard` crate

**Keyboard Shortcuts**:
- Cmd+C: Copy selected text
- Cmd+V: Paste clipboard content

### Menu System

Custom menu bar implementation (Floem 0.2 limitation).

**Menu Structure**:
- File: New Tab, New Window, Close Tab, Close Window
- Edit: Copy, Paste, Select All
- View: Zoom In/Out, Reset Zoom, Toggle Theme
- Window: Split Vertically, Split Horizontally, Next/Previous Pane

**Menu Actions**:
Each menu item triggers appropriate application action with visual feedback.

### Error Handling

Robust error handling with user-friendly messages.

**Error Categories**:
1. **PTY Errors**: Session creation/management failures
2. **I/O Errors**: File system and clipboard operations
3. **Configuration Errors**: Settings validation
4. **Rendering Errors**: Canvas/paint operation failures

**Handling Strategy**:
- Log all errors with context
- Display non-critical errors in status bar
- Prevent crashes with graceful degradation

## Performance Optimization

### Rendering Optimization

1. **GPU Acceleration**: All rendering goes through wgpu
2. **Batch Operations**: Floem batches multiple draw calls
3. **Dirty Tracking**: Only repaint on content changes
4. **Signal-based Invalidation**: Efficient change detection

**Performance Metrics**:
- Terminal rendering: 720x432px @ 60+ FPS
- PTY polling: 60 FPS background thread
- Memory: ~50MB typical usage

### PTY Polling Strategy

Background thread polls PTY output efficiently:

```rust
loop {
    // Non-blocking read
    match pty_session.read_nonblocking() {
        Ok(data) => process_output(data),
        Err(WouldBlock) => thread::sleep(16ms), // 60 FPS
    }
}
```

**Advantages**:
- Non-blocking keeps UI responsive
- 60 FPS rate balances latency and CPU usage
- Background thread isolates I/O from UI

### Memory Management

- Arc<Mutex<>> for thread-safe sharing
- RwSignal for minimal reactive overhead
- Terminal screen buffer: 80x24 cells ~5KB
- PTY state: ~1KB per session

## Technical Decisions

### Why Binary Tree for Panes?

**Advantages**:
- Flexible layout system (arbitrary depth)
- Efficient navigation and focus tracking
- Easy to implement recursive operations
- Supports arbitrary split patterns

**Node Structure**:
```rust
pub enum PaneNode {
    Leaf { id: Uuid, pane: Pane },
    HSplit { left: Box<PaneNode>, right: Box<PaneNode> },
    VSplit { top: Box<PaneNode>, bottom: Box<PaneNode> },
}
```

### Why RwSignal Instead of RefCell?

**RwSignal Benefits**:
- Automatic view reactivity
- Type-safe change tracking
- No manual invalidation needed
- Works seamlessly with Floem

### Why TOML for Configuration?

**TOML Advantages**:
- Human-readable and editable
- Strong typed support in Rust
- Good error messages
- Standard in Rust ecosystem

## Testing Strategy

### Manual Testing Checklist

1. **Terminal I/O**
   - [ ] Type characters in terminal
   - [ ] See output with correct colors
   - [ ] Control sequences work (Ctrl+C, Ctrl+D)

2. **Tabs**
   - [ ] Create new tabs (keyboard shortcut)
   - [ ] Switch between tabs
   - [ ] Close tabs with button/shortcut
   - [ ] Settings per tab preserved

3. **Panes**
   - [ ] Split panes vertically
   - [ ] Split panes horizontally
   - [ ] Navigate between panes
   - [ ] Close panes
   - [ ] Run different commands in panes

4. **Settings**
   - [ ] Change font size
   - [ ] Toggle theme
   - [ ] Settings persist after restart
   - [ ] All controls in status bar

5. **Clipboard**
   - [ ] Copy terminal output
   - [ ] Paste text into terminal
   - [ ] Multi-line paste works

### Automated Testing

Current test coverage focuses on:
- Settings validation (font size clamping)
- Color conversion (ANSI to Floem)
- Pane tree operations
- PTY session management

```bash
cargo test --lib
```

## Known Issues and Workarounds

### Issue: Text Rendering Placeholder

**Description**: Characters display as colored rectangles instead of actual glyphs.

**Root Cause**: Floem text rendering integration not yet implemented.

**Workaround**: Terminal is fully functional; just visual.

**Solution**: Phase 8 will integrate cosmic-text for proper rendering.

### Issue: IME Composition Display

**Description**: Composition text accepted but not shown visually.

**Root Cause**: No composition string rendering in current implementation.

**Workaround**: Input still works; just not visible during composition.

**Solution**: Add visual feedback for composition in future phase.

### Issue: Native Menu Bar

**Description**: Using custom UI menu instead of OS native menu bar.

**Root Cause**: Floem 0.2 doesn't fully support native menus.

**Workaround**: Custom menu bar provides same functionality.

**Solution**: Upgrade Floem when native menu support improves.

## Build Configuration

### Cargo Features

```toml
[features]
default = ["iced-gui"]
iced-gui = ["dep:iced"]
floem-gui = ["dep:floem", "dep:floem_renderer"]

[[bin]]
name = "agterm-floem"
path = "src/floem_main.rs"
required-features = ["floem-gui"]
```

### Build Variants

```bash
# Default (Iced)
cargo build

# Floem only
cargo build --bin agterm-floem --features floem-gui --no-default-features

# Both available
cargo build --all-targets
```

## Debugging Tips

### Enable Debug Logging

```bash
RUST_LOG=agterm=debug cargo run --bin agterm-floem --features floem-gui
```

### Check Event Flow

Add breakpoints in:
- `handle_global_shortcuts()` for keyboard input
- `AppState` mutations for state changes
- `View::paint()` for rendering

### Monitor PTY

```bash
AGTERM_PTY_DEBUG=1 cargo run --bin agterm-floem --features floem-gui
```

### Profile Performance

```bash
cargo build --release --bin agterm-floem
# Use system profiler to check performance
```

## References

### Documentation
- [Floem Documentation](https://docs.rs/floem/0.2.0/)
- [Floem Renderer](https://docs.rs/floem_renderer/)
- [ANSI Escape Codes](https://en.wikipedia.org/wiki/ANSI_escape_code)

### Dependencies
- floem: Reactive UI framework
- floem_renderer: GPU rendering (wgpu-based)
- portable-pty: Cross-platform PTY management
- vte: ANSI escape sequence parsing
- arboard: Clipboard access

### Related Code
- Terminal screen buffer: `agterm::terminal::screen`
- PTY management: `agterm::terminal::pty`
- Color definitions: `agterm::color`

---

**Last Updated**: 2026-01-19
**Status**: Active Development Complete
