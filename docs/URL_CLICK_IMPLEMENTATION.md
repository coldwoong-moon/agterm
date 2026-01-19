# URL Click Implementation Guide for AgTerm

## Overview
This document describes how to add URL detection and clicking functionality to the AgTerm terminal emulator.

## Current State
The infrastructure is already partially in place:
- `Cell` struct in `src/terminal/screen.rs` has a `hyperlink` field (line 268)
- `detect_urls()` method exists in `TerminalScreen` (line 929)
- `LinkHandler` exists in `src/link_handler.rs` with URL opening functionality
- `open` crate is available for opening URLs in default browser

## Required Changes

### 1. Add LinkRegion struct to `src/floem_app/views/terminal.rs`

Add this after the constants (after line 25):

```rust
/// Link region for click detection
#[derive(Clone, Debug)]
struct LinkRegion {
    /// The URL text
    url: Arc<String>,
    /// Row index
    row: usize,
    /// Start column
    start_col: usize,
    /// End column
    end_col: usize,
}

impl LinkRegion {
    /// Check if a cell position is within this link
    fn contains(&self, row: usize, col: usize) -> bool {
        self.row == row && col >= self.start_col && col < self.end_col
    }
}
```

### 2. Update TerminalCanvas struct

Find the `TerminalCanvas` struct (around line 217) and add these fields:

```rust
pub struct TerminalCanvas {
    id: ViewId,
    state: TerminalState,
    app_state: AppState,
    font_family: Vec<FamilyOwned>,
    last_size: std::cell::Cell<(f64, f64)>,
    is_focused: RwSignal<bool>,
    // ADD THESE NEW FIELDS:
    /// Currently detected link regions
    link_regions: std::cell::RefCell<Vec<LinkRegion>>,
    /// Currently hovered link URL
    hovered_link: std::cell::RefCell<Option<Arc<String>>>,
    /// Last mouse position (col, row)
    mouse_pos: std::cell::RefCell<Option<(usize, usize)>>,
}
```

### 3. Update TerminalCanvas::new()

Update the constructor to initialize the new fields:

```rust
pub fn new(state: TerminalState, app_state: AppState, is_focused: RwSignal<bool>) -> Self {
    let font_family = FamilyOwned::parse_list("JetBrains Mono, Menlo, Monaco, Courier New, monospace")
        .collect::<Vec<_>>();

    Self {
        id: ViewId::new(),
        state,
        app_state,
        font_family,
        last_size: std::cell::Cell::new((0.0, 0.0)),
        is_focused,
        // ADD THESE:
        link_regions: std::cell::RefCell::new(Vec::new()),
        hovered_link: std::cell::RefCell::new(None),
        mouse_pos: std::cell::RefCell::new(None),
    }
}
```

### 4. Add helper methods to TerminalCanvas

Add these methods before the `impl View for TerminalCanvas` block:

```rust
impl TerminalCanvas {
    // ... existing methods ...

    /// Convert pixel coordinates to cell position
    fn pixel_to_cell(&self, x: f64, y: f64) -> (usize, usize) {
        let col = (x / CELL_WIDTH).floor() as usize;
        let row = (y / CELL_HEIGHT).floor() as usize;
        (col, row)
    }

    /// Find link at given cell position
    fn find_link_at(&self, row: usize, col: usize) -> Option<Arc<String>> {
        let regions = self.link_regions.borrow();
        regions.iter()
            .find(|region| region.contains(row, col))
            .map(|region| Arc::clone(&region.url))
    }

    /// Handle mouse click on a link
    fn handle_link_click(&self, url: &str) {
        tracing::info!("Opening URL: {}", url);
        
        if let Err(e) = open::that(url) {
            tracing::error!("Failed to open URL {}: {}", url, e);
        }
    }

    /// Build link regions from screen buffer
    fn build_link_regions(
        &self,
        lines: &[Vec<agterm::terminal::screen::Cell>],
        rows: usize,
        cols: usize
    ) -> Vec<LinkRegion> {
        let mut regions = Vec::new();

        for (row_idx, row) in lines.iter().enumerate().take(rows) {
            let mut current_link: Option<(Arc<String>, usize)> = None;

            for (col_idx, cell) in row.iter().enumerate().take(cols) {
                if let Some(ref url) = cell.hyperlink {
                    match &current_link {
                        Some((link_url, start_col)) => {
                            if Arc::ptr_eq(link_url, url) {
                                continue; // Same link, continue
                            } else {
                                // Different URL, close previous and start new
                                regions.push(LinkRegion {
                                    url: Arc::clone(link_url),
                                    row: row_idx,
                                    start_col: *start_col,
                                    end_col: col_idx,
                                });
                                current_link = Some((Arc::clone(url), col_idx));
                            }
                        }
                        None => {
                            current_link = Some((Arc::clone(url), col_idx));
                        }
                    }
                } else if let Some((link_url, start_col)) = current_link.take() {
                    regions.push(LinkRegion {
                        url: link_url,
                        row: row_idx,
                        start_col,
                        end_col: col_idx,
                    });
                }
            }

            // Close any remaining link at end of line
            if let Some((link_url, start_col)) = current_link {
                regions.push(LinkRegion {
                    url: link_url,
                    row: row_idx,
                    start_col,
                    end_col: cols,
                });
            }
        }

        regions
    }
}
```

### 5. Update the paint() method

In the `paint()` method (around line 454), after getting the lines:

```rust
// After getting lines...
// Build link regions for click detection
let link_regions = self.build_link_regions(&lines, rows, cols);
*self.link_regions.borrow_mut() = link_regions;

// Get hovered link
let hovered_link = self.hovered_link.borrow().clone();
```

Then in the cell rendering loop, add hover detection and rendering:

```rust
for (row_idx, row) in lines.iter().enumerate().take(rows) {
    for (col_idx, cell) in row.iter().enumerate().take(cols) {
        let x = col_idx as f64 * CELL_WIDTH;
        let y = row_idx as f64 * CELL_HEIGHT;

        // Check if this cell is part of a hovered link
        let is_hovered_link = if let Some(ref hover_url) = hovered_link {
            cell.hyperlink.as_ref().map_or(false, |url| Arc::ptr_eq(url, hover_url))
        } else {
            false
        };

        // ... existing rendering code ...

        // AFTER drawing the cell background, ADD hover background:
        if is_hovered_link {
            let hover_rect = Rect::new(x, y, x + CELL_WIDTH, y + CELL_HEIGHT);
            cx.fill(&hover_rect, Color::rgba(0.3, 0.5, 0.8, 0.1), 0.0);
        }

        // ... existing text rendering ...

        // AFTER drawing text, ADD underline for hovered links:
        if is_hovered_link {
            let underline_rect = Rect::new(
                x,
                y + CELL_HEIGHT - 1.0,
                x + CELL_WIDTH,
                y + CELL_HEIGHT,
            );
            cx.fill(&underline_rect, colors.accent_blue, 0.0);
        }
    }
}
```

### 6. Call detect_urls() after processing output

In `TerminalState::process_output()` method (around line 90):

```rust
pub fn process_output(&self, data: &[u8]) {
    match self.screen.lock() {
        Ok(mut screen) => {
            screen.process(data);
            // ADD THIS LINE:
            screen.detect_urls();
            self.content_version.update(|v| *v += 1);
        }
        Err(e) => {
            tracing::error!("Failed to lock terminal screen for output processing: {}", e);
        }
    }
}
```

### 7. Add mouse event handlers in pane_view.rs

In `src/floem_app/views/pane_view.rs`, the terminal canvas container needs mouse event handlers.

Find where the terminal canvas is wrapped in a container (around line 101-138) and add:

```rust
container(/* ... terminal canvas ... */)
    // ADD THESE EVENT HANDLERS:
    .on_event_stop(EventListener::PointerMove, {
        let terminal_state_clone = terminal_state.clone();
        move |event| {
            if let floem::event::Event::PointerMove(pointer) = event {
                // Convert pixel to cell coords
                let col = (pointer.pos.x / CELL_WIDTH).floor() as usize;
                let row = (pointer.pos.y / CELL_HEIGHT).floor() as usize;
                
                // Update hovered link
                // (This requires accessing the TerminalCanvas instance - see note below)
            }
        }
    })
    .on_event_stop(EventListener::PointerDown, {
        let terminal_state_clone = terminal_state.clone();
        move |event| {
            if let floem::event::Event::PointerDown(pointer) = event {
                // Check for Cmd+click (macOS) or Ctrl+click (other platforms)
                #[cfg(target_os = "macos")]
                let modifier_pressed = pointer.modifiers.meta();
                #[cfg(not(target_os = "macos"))]
                let modifier_pressed = pointer.modifiers.control();

                if modifier_pressed {
                    let col = (pointer.pos.x / CELL_WIDTH).floor() as usize;
                    let row = (pointer.pos.y / CELL_HEIGHT).floor() as usize;
                    
                    // Find and open link
                    // (This requires accessing the TerminalCanvas instance)
                }
            }
        }
    })
    .style(/* ... */)
```

## Implementation Note

The challenge with the current architecture is that `TerminalCanvas` is created inside the view rendering and doesn't have a signal-based API for mouse events. 

**Recommended approach:**

1. Add link interaction signals to `TerminalState`:
   ```rust
   pub struct TerminalState {
       // ... existing fields ...
       pub hovered_cell: RwSignal<Option<(usize, usize)>>,
   }
   ```

2. Use these signals in the event handlers

3. Let the paint() method read these signals and update hover state

This maintains Floem's reactive design pattern.

## Testing

1. Build: `cargo build`
2. Run: `cargo run`
3. In terminal, output a URL: `echo "Visit https://github.com"`
4. Hover over the URL - should see underline
5. Cmd+Click the URL - should open in browser

## Files Modified

- `src/floem_app/views/terminal.rs` - Main implementation
- `src/floem_app/views/pane_view.rs` - Mouse event handlers
- `src/terminal/screen.rs` - Already has detect_urls()

## Dependencies

All required dependencies are already in Cargo.toml:
- `open` crate for opening URLs
- `regex` for URL pattern matching
- `floem` for UI

