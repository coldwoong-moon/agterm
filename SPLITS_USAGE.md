# Terminal Split Management

The `splits` module provides a tree-based split management system for terminal panes in AgTerm.

## Overview

The split system allows you to:
- Split terminal panes horizontally (top/bottom) or vertically (left/right)
- Navigate between panes using direction keys
- Resize splits dynamically
- Close panes while maintaining the tree structure
- Serialize/deserialize split layouts for session persistence

## Core Components

### `SplitDirection`

Defines the direction of a split:
- `Horizontal`: Divides the pane into top and bottom sections
- `Vertical`: Divides the pane into left and right sections

### `SplitNode`

A recursive tree structure representing the split layout:
- `Leaf`: Terminal pane with a unique ID
- `Split`: Container with two child nodes and a split ratio

### `SplitContainer`

The main container managing the split tree:
- Tracks the focused pane
- Manages pane IDs
- Provides methods for splitting, navigation, and resizing

## Usage Examples

### Basic Splitting

```rust
use agterm::splits::{SplitContainer, SplitDirection};

// Create a new container with a single pane
let mut container = SplitContainer::new();

// Split the focused pane horizontally
let new_pane_id = container.split_focused(SplitDirection::Horizontal);

// Split again vertically
let another_pane_id = container.split_focused(SplitDirection::Vertical);
```

### Navigation

```rust
use agterm::splits::NavigationDirection;

// Navigate to the pane above
if container.navigate_focus(NavigationDirection::Up) {
    println!("Moved focus up");
}

// Navigate to the pane on the right
if container.navigate_focus(NavigationDirection::Right) {
    println!("Moved focus right");
}
```

### Resizing

```rust
// Increase the size of the focused pane
let focused_id = container.focused_id();
container.resize_split(focused_id, 0.1); // +10%

// Decrease the size
container.resize_split(focused_id, -0.1); // -10%
```

### Pane Bounds

Get the bounding box for rendering:

```rust
if let Some((x, y, width, height)) = container.get_pane_bounds(pane_id) {
    // Coordinates are normalized (0.0 to 1.0)
    // Multiply by actual window dimensions for pixel coordinates
    let pixel_x = x * window_width;
    let pixel_y = y * window_height;
    let pixel_width = width * window_width;
    let pixel_height = height * window_height;
}
```

### Closing Panes

```rust
// Close a specific pane
if container.close_pane(pane_id) {
    println!("Pane closed successfully");
} else {
    println!("Cannot close the last pane");
}
```

## Layout Examples

### Two Horizontal Panes

```
┌─────────────┐
│   Pane 0    │
├─────────────┤
│   Pane 1    │
└─────────────┘
```

```rust
let mut container = SplitContainer::new();
container.split_focused(SplitDirection::Horizontal);
```

### Two Vertical Panes

```
┌──────┬──────┐
│      │      │
│ P0   │ P1   │
│      │      │
└──────┴──────┘
```

```rust
let mut container = SplitContainer::new();
container.split_focused(SplitDirection::Vertical);
```

### Complex 4-Pane Layout

```
┌──────┬──────┐
│      │  P2  │
│  P0  ├──────┤
│      │  P3  │
├──────┴──────┤
│     P1      │
└─────────────┘
```

```rust
let mut container = SplitContainer::new();
container.split_focused(SplitDirection::Horizontal); // Split 0 -> [0, 1]
container.set_focused_id(0);
container.split_focused(SplitDirection::Vertical);   // Split 0 -> [0, 2]
container.split_focused(SplitDirection::Horizontal); // Split 2 -> [2, 3]
```

## Integration with AgTerm

To integrate the split system with the main application:

1. **Store the container**: Add a `SplitContainer` to your tab state
2. **Map pane IDs to PTY sessions**: Maintain a HashMap<usize, PtySession>
3. **Render each pane**: Use `get_pane_bounds()` to position each terminal
4. **Handle keyboard input**: Map keybindings to split/navigate/resize operations

### Example Integration

```rust
struct TerminalTab {
    splits: SplitContainer,
    pty_sessions: HashMap<usize, PtySession>,
    // ... other fields
}

impl TerminalTab {
    fn split_horizontal(&mut self) {
        let new_id = self.splits.split_focused(SplitDirection::Horizontal);
        let pty = create_new_pty_session();
        self.pty_sessions.insert(new_id, pty);
    }

    fn render(&self, window_width: f32, window_height: f32) {
        for pane_id in self.splits.get_all_ids() {
            if let Some((x, y, w, h)) = self.splits.get_pane_bounds(pane_id) {
                let bounds = (
                    x * window_width,
                    y * window_height,
                    w * window_width,
                    h * window_height,
                );

                if let Some(pty) = self.pty_sessions.get(&pane_id) {
                    render_terminal(pty, bounds);
                }
            }
        }
    }
}
```

## Advanced Features

### Session Persistence

The split container supports serialization:

```rust
use serde_json;

// Serialize
let json = serde_json::to_string(&container)?;

// Deserialize
let container: SplitContainer = serde_json::from_str(&json)?;
```

### Split Ratio Constraints

Split ratios are automatically clamped to prevent too-small panes:
- Minimum ratio: 0.1 (10%)
- Maximum ratio: 0.9 (90%)

This ensures all panes remain usable.

### Focus Management

When closing a pane:
- If the focused pane is closed, focus automatically moves to another pane
- Cannot close the last remaining pane

## Testing

The module includes comprehensive tests covering:
- Basic splitting operations
- Navigation between panes
- Pane closing and focus management
- Bounds calculation
- Resizing with constraints
- Complex nested splits
- Serialization/deserialization

Run tests with:

```bash
cargo test --lib splits
```

## API Reference

### `SplitContainer`

- `new() -> Self` - Create a new container with a single pane
- `split_focused(direction: SplitDirection) -> usize` - Split the focused pane
- `focused_id() -> usize` - Get the currently focused pane ID
- `set_focused_id(id: usize) -> bool` - Set the focused pane
- `get_all_ids() -> Vec<usize>` - Get all pane IDs
- `pane_count() -> usize` - Get the total number of panes
- `close_pane(id: usize) -> bool` - Close a specific pane
- `navigate_focus(direction: NavigationDirection) -> bool` - Navigate focus
- `get_pane_bounds(id: usize) -> Option<(f32, f32, f32, f32)>` - Get pane bounds
- `resize_split(pane_id: usize, delta: f32) -> bool` - Resize a split
- `root() -> &SplitNode` - Get the root node for inspection

### `NavigationDirection`

- `Up` - Navigate to the pane above
- `Down` - Navigate to the pane below
- `Left` - Navigate to the pane on the left
- `Right` - Navigate to the pane on the right

## Performance Considerations

- Tree operations are O(n) where n is the number of panes
- Bounds calculation is O(h) where h is the tree height
- The tree structure naturally limits depth in practice
- All operations are fast enough for interactive use

## Future Enhancements

Potential improvements:
- Custom split ratios on creation
- Named panes for easier reference
- Pane swapping
- Tab-like pane switching (Ctrl+1, Ctrl+2, etc.)
- Save/load named layouts
- Zoom mode (maximize a single pane temporarily)
- Visual split indicators for better UX
