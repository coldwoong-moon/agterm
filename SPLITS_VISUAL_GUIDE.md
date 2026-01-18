# Visual Guide to Terminal Splits

## Split Operations

### Starting State: Single Pane
```
┌─────────────────────┐
│                     │
│                     │
│      Pane 0         │
│   (Focused)         │
│                     │
│                     │
└─────────────────────┘
```

### After Horizontal Split
```rust
container.split_focused(SplitDirection::Horizontal);
```
```
┌─────────────────────┐
│                     │
│      Pane 0         │
│                     │
├─────────────────────┤  ← Split line (50/50 ratio)
│                     │
│      Pane 1         │
│   (Focused)         │
└─────────────────────┘
```

### After Vertical Split on Pane 1
```rust
container.split_focused(SplitDirection::Vertical);
```
```
┌─────────────────────┐
│                     │
│      Pane 0         │
│                     │
├──────────┬──────────┤
│          │          │
│ Pane 1   │ Pane 2   │
│          │(Focused) │
└──────────┴──────────┘
```

### Complex 4-Pane Layout
```rust
// Starting from scratch
let mut container = SplitContainer::new();
container.split_focused(SplitDirection::Horizontal); // Create pane 1 below 0
container.set_focused_id(0);                         // Focus pane 0
container.split_focused(SplitDirection::Vertical);   // Create pane 2 right of 0
container.split_focused(SplitDirection::Horizontal); // Create pane 3 below 2
```
```
┌──────────┬──────────┐
│          │  Pane 2  │
│  Pane 0  ├──────────┤
│          │  Pane 3  │
├──────────┴──────────┤
│      Pane 1         │
└─────────────────────┘
```

## Tree Structure

### Visual Representation
The tree structure for the 4-pane layout above:

```
                 [Horizontal Split]
                /                  \
          [Vertical Split]        Pane 1
          /              \
      Pane 0      [Horizontal Split]
                  /              \
              Pane 2          Pane 3
```

### Internal Representation
```rust
Split {
    direction: Horizontal,
    ratio: 0.5,
    first: Split {
        direction: Vertical,
        ratio: 0.5,
        first: Leaf { id: 0 },
        second: Split {
            direction: Horizontal,
            ratio: 0.5,
            first: Leaf { id: 2 },
            second: Leaf { id: 3 },
        }
    },
    second: Leaf { id: 1 }
}
```

## Navigation Examples

### Horizontal Navigation
```
┌──────────┬──────────┬──────────┐
│          │          │          │
│ Pane 0   │ Pane 1   │ Pane 2   │
│          │(Focused) │          │
└──────────┴──────────┴──────────┘

navigate_focus(Left)   → Focus moves to Pane 0
navigate_focus(Right)  → Focus moves to Pane 2
navigate_focus(Up)     → No change (no pane above)
navigate_focus(Down)   → No change (no pane below)
```

### Vertical Navigation
```
┌─────────────────────┐
│      Pane 0         │
├─────────────────────┤
│      Pane 1         │
│   (Focused)         │
├─────────────────────┤
│      Pane 2         │
└─────────────────────┘

navigate_focus(Up)     → Focus moves to Pane 1
navigate_focus(Down)   → Focus moves to Pane 2
navigate_focus(Left)   → No change (no pane to left)
navigate_focus(Right)  → No change (no pane to right)
```

### Complex Navigation
```
┌──────────┬──────────┐
│          │  Pane 2  │
│  Pane 0  ├──────────┤
│          │  Pane 3  │
│(Focused) │          │
├──────────┴──────────┤
│      Pane 1         │
└─────────────────────┘

From Pane 0:
  Right → Pane 2 (nearest to the right)
  Down  → Pane 1 (directly below)
  Up    → No change
  Left  → No change

From Pane 2:
  Left  → Pane 0 (nearest to the left)
  Down  → Pane 3 (directly below)
  Up    → No change
  Right → No change
```

## Resize Operations

### Initial State
```
┌──────────┬──────────┐
│          │          │
│ Pane 0   │ Pane 1   │
│  50%     │   50%    │
│          │          │
└──────────┴──────────┘
```

### After resize_split(0, 0.2)
Increase Pane 0's ratio by 0.2 (20%)
```
┌────────────────┬─────┐
│                │     │
│    Pane 0      │ P1  │
│     70%        │ 30% │
│                │     │
└────────────────┴─────┘
```

### After resize_split(0, -0.3)
Decrease Pane 0's ratio by 0.3 (30%)
```
┌─────┬────────────────┐
│     │                │
│ P0  │    Pane 1      │
│ 40% │     60%        │
│     │                │
└─────┴────────────────┘
```

### Clamping Behavior
```rust
// Trying to make pane too small
resize_split(0, -0.9);  // Would result in 0%

// Result: Clamped to minimum 10%
┌──┬──────────────────┐
│P0│     Pane 1       │
│  │      90%         │
│10│                  │
└──┴──────────────────┘
```

## Closing Panes

### Before: 3 Panes
```
┌──────────┬──────────┐
│          │          │
│ Pane 0   │ Pane 2   │
│          │          │
├──────────┴──────────┤
│      Pane 1         │
│   (Focused)         │
└─────────────────────┘
```

### After: close_pane(1)
```
┌──────────┬──────────┐
│          │          │
│          │          │
│ Pane 0   │ Pane 2   │
│          │ (Focused)│
│          │          │
│          │          │
└──────────┴──────────┘
```

The tree automatically adjusts to remove the closed pane.

## Bounds Calculation

### Normalized Coordinates (0.0 to 1.0)
```
┌──────────┬──────────┐
│          │          │
│ Pane 0   │ Pane 1   │
│          │          │
└──────────┴──────────┘

Pane 0: (x=0.0, y=0.0, width=0.5, height=1.0)
Pane 1: (x=0.5, y=0.0, width=0.5, height=1.0)
```

### Converting to Pixels
```rust
let (x, y, w, h) = container.get_pane_bounds(pane_id)?;
let window_width = 1920.0;
let window_height = 1080.0;

let pixel_x = x * window_width;      // 0.5 * 1920 = 960
let pixel_y = y * window_height;     // 0.0 * 1080 = 0
let pixel_w = w * window_width;      // 0.5 * 1920 = 960
let pixel_h = h * window_height;     // 1.0 * 1080 = 1080
```

## Common Layouts

### IDE-Style (Code + Terminal)
```
┌─────────────────────┐
│                     │
│    Code Editor      │
│     (Pane 0)        │
│                     │
├─────────────────────┤
│    Terminal         │
│     (Pane 1)        │
└─────────────────────┘

Code: 70%, Terminal: 30%
```

### Tmux-Style (3-Column)
```
┌──────┬──────────┬──────┐
│      │          │      │
│  P0  │   P1     │  P2  │
│ 25%  │   50%    │ 25%  │
│      │          │      │
└──────┴──────────┴──────┘
```

### Quad Split (Even)
```
┌──────────┬──────────┐
│          │          │
│ Pane 0   │ Pane 2   │
│          │          │
├──────────┼──────────┤
│          │          │
│ Pane 1   │ Pane 3   │
│          │          │
└──────────┴──────────┘

All panes: 25% each
```

### Main + Side (Golden Ratio)
```
┌──────────────┬──────┐
│              │      │
│              │ P1   │
│    Main      │      │
│   (Pane 0)   ├──────┤
│     62%      │      │
│              │ P2   │
│              │      │
└──────────────┴──────┘
                 38%
```

## Focus Indicators

### Visual Feedback
```
┌──────────┬══════════┐  ← Double border = focused
│          ║          ║
│ Pane 0   ║ Pane 1   ║
│          ║(Focused) ║
└──────────┴══════════┘
```

### Status Display
```
┌─────────────────────┐
│ Pane 1/3 [Split V]  │  ← Status bar shows current pane
├─────────────────────┤
│                     │
│   Terminal content  │
│                     │
└─────────────────────┘
```

## Keyboard Workflow Example

Starting from single pane, creating IDE layout:

```
1. Initial
┌─────────────┐
│   Pane 0    │
└─────────────┘

2. Ctrl+B H (split horizontal)
┌─────────────┐
│   Pane 0    │
├─────────────┤
│   Pane 1    │
└─────────────┘

3. Ctrl+B ↑ (navigate up)
┌═════════════┐
║   Pane 0    ║
├─────────────┤
│   Pane 1    │
└─────────────┘

4. Ctrl+B V (split vertical)
┌──────┬──────┐
║ P0   │ P2   ║
├──────┴──────┤
│   Pane 1    │
└─────────────┘

5. Ctrl+B + (resize larger)
┌─────────┬───┐
║  P0     │P2 ║
├─────────┴───┤
│   Pane 1    │
└─────────────┘

Final layout: Code on left, docs on right, terminal below
```

## Split Ratio Visualization

### Equal Split (0.5)
```
┌──────────┬──────────┐
│    50%   │   50%    │
└──────────┴──────────┘
```

### Unequal Split (0.7)
```
┌──────────────┬──────┐
│     70%      │ 30%  │
└──────────────┴──────┘
```

### Minimum Clamped (0.1)
```
┌──┬──────────────────┐
│10│       90%        │
└──┴──────────────────┘
```

### Maximum Clamped (0.9)
```
┌──────────────────┬──┐
│       90%        │10│
└──────────────────┴──┘
```
