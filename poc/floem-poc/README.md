# Floem POC - Canvas Rendering Performance Test

This is a proof-of-concept project testing canvas rendering performance in Floem framework.

## Canvas Test

Location: `src/canvas_test.rs`

### Features

- **Terminal Grid Simulation**: Renders an 80x24 grid (1,920 cells total)
- **Random Colors**: Each cell has randomly generated foreground and background colors
- **Performance Metrics**: Displays frame count and FPS counter
- **SVG Rendering**: Uses SVG for efficient primitive rendering

### Technical Details

- **Grid Dimensions**: 80 columns × 24 rows = 1,920 cells
- **Cell Size**: 10px width × 20px height
- **Rendering**: 3,840 rectangles per frame (2 per cell: background + indicator)
- **Approach**: Static SVG rendering for baseline performance measurement

### Running the Test

```bash
cargo run --bin canvas-test
```

### What's Measured

1. **Frame Count**: Total frames rendered since startup
2. **FPS**: Frames per second (target: 60 FPS)
3. **Render Time**: Time taken to construct the scene

### Current Implementation

This version uses **static SVG rendering** as a simplified approach because:
- Floem 0.2's canvas API is limited for dynamic rendering
- SVG provides good baseline for measuring primitive rendering performance
- Easy to count exact number of primitives (rectangles) rendered

### Limitations

- Character glyphs not rendered (colored indicators used instead)
- Static rendering (no animation or updates)
- Simplified for performance testing purposes

### Next Steps

For a real terminal implementation, you would need:
1. Dynamic canvas with text rendering
2. Proper glyph cache and rendering
3. Scrollback buffer management
4. Input event handling
5. Selection and copy/paste support

## Build

```bash
# Build all binaries
cargo build

# Build release mode
cargo build --release

# Build specific test
cargo build --bin canvas-test
```

## Project Structure

```
floem-poc/
├── Cargo.toml
├── src/
│   ├── main.rs          # Default entry point
│   ├── canvas_test.rs   # Canvas rendering performance test
│   ├── ime_test.rs      # (planned) IME input test
│   └── pane_test.rs     # (planned) Layout pane test
└── README.md
```

## Dependencies

- **floem** 0.2 - Native Rust UI framework with fine-grained reactivity
  - Includes peniko for 2D graphics primitives
  - Reactive signals for state management
  - Declarative view composition

## Notes

This is a proof-of-concept for evaluating Floem's suitability for building a terminal emulator. The focus is on:

1. **Rendering Performance**: Can it handle 1,920+ cells at 60 FPS?
2. **API Ergonomics**: How easy is it to work with Floem's APIs?
3. **Reactivity**: How well does reactive state management work for terminal updates?

Current results indicate that SVG rendering handles the primitive count well, but real-world terminal performance would depend on:
- Text layout and glyph rendering performance
- Dynamic updates and partial redraws
- Memory usage with large scrollback buffers
- Input latency and event handling
