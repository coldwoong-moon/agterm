# Floem Terminal Rendering - Phase 2 Learnings

## Key Insights

### 1. Floem Renderer Architecture

**Discovery**: Floem uses a trait-based renderer abstraction (`floem_renderer::Renderer`)
- The trait defines rendering primitives: `fill`, `stroke`, `draw_text`, `draw_svg`, etc.
- `PaintCx` derefs to the concrete `Renderer<W: wgpu::WindowHandle>` type
- Methods are only accessible when the trait is in scope

**Implication**: Need to either:
- Import `floem_renderer` as a dependency (chosen solution)
- Use only the higher-level methods exposed through floem's API

### 2. Feature Flags and Dependencies

**Problem**: `floem_renderer` is a transitive dependency of `floem` but not re-exported
**Solution**: Added as explicit optional dependency

```toml
[dependencies]
floem_renderer = { version = "0.2", optional = true }

[features]
floem-gui = ["dep:floem", "dep:floem_renderer"]
```

**Why this works**: Cargo allows accessing transitive dependencies if explicitly declared

### 3. Custom View Implementation

**Pattern for custom Floem views**:
```rust
pub struct CustomView {
    id: ViewId,          // Required for view identification
    state: MyState,      // Your view's state
}

impl View for CustomView {
    fn id(&self) -> ViewId { self.id }

    fn paint(&mut self, cx: &mut PaintCx) {
        // Rendering logic here
        // cx derefs to Renderer trait methods
    }

    fn update(&mut self, _cx: &mut UpdateCx, _state: Box<dyn Any>) {
        // React to state changes
        self.id.request_paint();
    }
}
```

### 4. Reactive State Management

**Pattern**: Separate reactive signals from core data structures
```rust
pub struct TerminalState {
    screen: Arc<Mutex<TerminalScreen>>,  // Shared, mutable state
    content_version: RwSignal<u64>,       // Reactive change tracker
}
```

**Benefits**:
- Thread-safe access to terminal buffer
- Efficient change detection
- Automatic view updates on signal changes

### 5. Color Conversion Between Frameworks

**Challenge**: Converting between `agterm::color::Color` and `floem::peniko::Color`

**Solution**: Framework-agnostic color type with conversion functions
```rust
fn ansi_to_floem_color(ansi: &AnsiColor) -> Color {
    let agterm_color = ansi.to_color();
    Color::rgba(
        agterm_color.r as f64,
        agterm_color.g as f64,
        agterm_color.b as f64,
        agterm_color.a as f64,
    )
}
```

### 6. Text Rendering Complexity

**Discovery**: Floem's text rendering uses cosmic-text + TextLayout
- Not as simple as drawing strings at positions
- Requires text shaping, glyph caching, font management
- TextLayout is a prepared, cacheable text object

**Deferred**: Full text rendering to Phase 3
**Current**: Placeholder rectangles to validate grid layout and colors

### 7. Layout System Integration

**Floem's layout flow**: Taffy → ViewId layout storage → PaintCx access
```rust
let layout = self.id.get_layout().unwrap_or_default();
let rect = Rect::new(0.0, 0.0,
    layout.size.width as f64,
    layout.size.height as f64
);
```

### 8. Performance Considerations

**For 80x24 terminal**:
- 1,920 cells
- Each cell: background rect + character rendering
- Minimal: ~3,840 drawing operations per frame

**Optimization strategies** (for Phase 3+):
1. Dirty tracking (only repaint changed cells)
2. Texture atlas for common glyphs
3. Batch rendering calls
4. GPU acceleration (already provided by wgpu backend)

### 9. Debugging Compilation Errors

**Useful patterns**:
1. Compile library separately: `cargo check --lib`
2. Check specific binary: `cargo check --bin name`
3. Read trait method signatures carefully (parameters, lifetimes)
4. Look at working examples in the same codebase (e.g., svg.rs)

### 10. Documentation is Sparse

**Reality**: Floem 0.2 is relatively new, documentation is limited
**Approach**:
- Read source code directly (cargo registry cache)
- Study existing views (svg, label, container, etc.)
- Experiment and iterate
- Document your findings for others

## Gotchas

1. **Trait methods not in scope**: Import the trait, not just the type
2. **Color type confusion**: Multiple Color types (floem::peniko::Color, agterm::color::Color)
3. **Closure move semantics**: Signals are Copy, but wrapped types need Arc/Rc
4. **View lifecycle**: Views are created once, not recreated on every update
5. **PaintCx deref**: Methods you want might be on the dereferenced type

## What Worked Well

1. ✅ Reusing existing TerminalScreen buffer (no duplication)
2. ✅ Arc<Mutex<>> for thread-safe state sharing
3. ✅ RwSignal for reactive change tracking
4. ✅ Custom View trait implementation
5. ✅ Color conversion abstraction
6. ✅ Placeholder approach for incomplete features

## What Needs Improvement

1. ⚠️ Text rendering (currently just colored rectangles)
2. ⚠️ No dirty tracking yet (repaints entire grid every frame)
3. ⚠️ No IME integration
4. ⚠️ No PTY connection yet
5. ⚠️ No keyboard input handling
6. ⚠️ No scrollback support in UI

## Code Quality Notes

- **Compilation warnings**: 34 warnings, all "unused code" (acceptable for WIP)
- **Error handling**: Uses `Result` unwrapping in some places (needs improvement)
- **Thread safety**: Proper use of Arc/Mutex
- **Reactivity**: Correct signal usage patterns

## Time Estimates for Phase 3

Based on Phase 2 experience:
- Text rendering integration: 3-4 hours
- IME + keyboard input: 2-3 hours
- PTY connection: 1-2 hours
- Basic testing + bugfixes: 2-3 hours

**Total Phase 3 estimate**: 8-12 hours

## References Used

- Floem source: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/floem-0.2.0/`
- Floem views: `src/views/*.rs`
- Terminal screen: `agterm/src/terminal/screen.rs`
- POC examples: `poc/floem-poc/src/*.rs`
