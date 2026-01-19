# Floem Pane Split Test

A demonstration of resizable pane layouts using the Floem GUI framework.

## Features

- **2-way horizontal split**: Main view split into left and right panes
- **Nested vertical split**: Right pane further divided vertically
- **4-pane layout**: Bottom right section split horizontally again
- **Draggable dividers**: Click and drag the gray dividers to resize panes
- **Color-coded panes**: Each pane has a distinct background color for easy identification
  - Pane 1 (Red): Top-left
  - Pane 2 (Green): Top-right
  - Pane 3 (Blue): Bottom-left
  - Pane 4 (Yellow): Bottom-right

## Running

```bash
cargo run --bin pane-test
```

## Implementation Details

### Layout Structure

The application uses Floem's flexbox-based layout system:
- `h_stack()`: Horizontal stacks for side-by-side panes
- `v_stack()`: Vertical stacks for top-bottom panes
- `flex_grow()`: Dynamic sizing based on divider position signals

### Divider Interaction

Dividers are implemented as interactive containers that:
1. Track pointer down/move/up events
2. Calculate delta from drag start position
3. Update the corresponding position signal (clamped to 0.1-0.9)
4. Change appearance on hover and during drag

### Reactive State

Uses Floem's reactive signals:
- `divider1_pos`, `divider2_pos`, `divider3_pos`: Position ratios (0.0-1.0)
- `dragging`: Boolean flag for active drag state
- Pane sizes automatically update when divider positions change

## Customization

To adjust the divider sensitivity, modify the `scale` factor in `vertical_divider()` and `horizontal_divider()`:

```rust
let scale = 0.002; // Smaller = less sensitive, larger = more sensitive
```

To change pane colors, edit the `Color::rgb8()` values in `app_view()`.

## Dependencies

- `floem = "0.2"`
