# Tab Drag and Reordering Implementation

## Overview
This document describes the tab drag-and-drop reordering feature implementation for AgTerm terminal emulator.

## Implementation Status: Infrastructure Complete ✅

### What Was Implemented

#### 1. Core Data Structures (src/main.rs)

**TabDragState** (line 585-591)
```rust
struct TabDragState {
    dragging_index: usize,
    start_x: f32,
    current_x: f32,
}
```
- Tracks which tab is being dragged
- Stores drag start and current positions for calculating movement

**TabContextMenu** (line 593-597)
```rust
struct TabContextMenu {
    tab_index: usize,
    position: (f32, f32),
}
```
- Stores context menu state for right-click actions
- Position for rendering menu at mouse location

**AgTerm State Extensions** (line 380-387)
```rust
tab_drag: Option<TabDragState>,
tab_context_menu: Option<TabContextMenu>,
tab_rename_mode: Option<usize>,
tab_rename_input: String,
```

#### 2. Message Types (src/main.rs, line 762-773)

**Drag and Drop Messages:**
- `TabDragStart(usize)` - Initiates drag on tab index
- `TabDragMove(f32)` - Updates drag position (x coordinate)
- `TabDragEnd` - Completes drag operation

**Context Menu Messages:**
- `TabContextMenu(usize, f32, f32)` - Opens context menu at position
- `CloseContextMenu` - Closes context menu

**Tab Rename Messages:**
- `TabRenameStart(usize)` - Enters rename mode for tab
- `TabRenameInput(String)` - Updates rename input buffer
- `TabRenameSubmit` - Applies new tab name
- `TabRenameCancel` - Cancels rename operation

#### 3. Message Handlers (src/main.rs, line 1720-1815)

**TabDragStart Handler:**
- Initializes drag state with starting index and position
- Prepares for tracking movement

**TabDragMove Handler:**
- Calculates target index based on horizontal movement
- Uses 150px as approximate tab width
- Swaps tabs when movement crosses threshold
- Updates active tab index if necessary
- Maintains drag state consistency

**TabDragEnd Handler:**
- Clears drag state
- Finalizes tab reordering

**Tab Rename Handlers:**
- `TabRenameStart`: Loads current tab title into input buffer
- `TabRenameInput`: Updates input as user types
- `TabRenameSubmit`: Saves new title to tab, clears empty titles
- `TabRenameCancel`: Discards changes

**Context Menu Handlers:**
- Simple state management for showing/hiding menu
- Stores clicked tab index and mouse position

## How It Works

### Tab Reordering Algorithm

```rust
// Calculate how many tab positions to move
let tab_width = 150.0;
let offset = current_x - start_x;
let position_change = (offset / tab_width).round() as i32;

// Calculate target index (clamped to valid range)
let target_index = (dragging_index as i32 + position_change)
    .max(0)
    .min(tabs.len() as i32 - 1) as usize;

// Swap tabs if position changed
if target_index != dragging_index {
    tabs.swap(dragging_index, target_index);
    // Update active tab tracking
    // Update drag state to new position
}
```

### Tab Renaming Flow

1. User triggers `TabRenameStart(index)` (e.g., double-click or context menu)
2. Current tab title loaded into `tab_rename_input`
3. `tab_rename_mode` set to `Some(index)`
4. User types, sending `TabRenameInput` messages
5. On submit: `TabRenameSubmit` saves to `tab.title`
6. On cancel: `TabRenameCancel` discards changes

## What's Missing (View Layer)

The **business logic is complete**, but the **UI interactions** need to be implemented in the view layer:

### Required View Updates

1. **Tab Bar Event Handling** (view_tab_bar function)
   - Add mouse event listeners to tab buttons
   - Detect drag start (mouse down + move)
   - Track drag movement
   - Detect drag end (mouse up)

2. **Context Menu Rendering**
   - Right-click detection on tabs
   - Menu popup with options:
     - Duplicate Tab (already functional via `Message::DuplicateTab`)
     - Rename Tab
     - Close Tab
   - Menu positioning at cursor

3. **Rename Input Field**
   - Replace tab label with text input when in rename mode
   - Handle Enter key (submit) and Esc key (cancel)
   - Focus management

### Why View Implementation is Challenging

Iced's `button` widget doesn't support:
- Mouse drag events (only click events)
- Right-click detection
- Custom mouse event handling

**Possible Solutions:**

1. **Mouse Event Subscription** (Recommended)
   ```rust
   fn subscription(&self) -> Subscription<Message> {
       iced::event::listen_with(|event, status| {
           match event {
               Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                   // Check if over tab, send TabDragStart
               },
               Event::Mouse(mouse::Event::CursorMoved { position }) => {
                   // If dragging, send TabDragMove
               },
               Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                   // Send TabDragEnd
               },
               Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                   // Send TabContextMenu
               },
               _ => None,
           }
       })
   }
   ```

2. **Custom Tab Widget**
   - Implement `Widget` trait for tabs
   - Handle mouse events in `on_event` method
   - Full control over interaction

3. **Gesture Library**
   - Use a third-party gesture detection library
   - Map gestures to messages

## Testing

### Build Status
```bash
$ cargo build 2>&1
   Compiling agterm v1.0.0
   Finished `dev` profile [unoptimized + debuginfo] target(s)
```
✅ Compiles without errors (50 warnings, all pre-existing)

### Test Status
```bash
$ cargo test --lib
test result: ok. 219 passed; 0 failed; 0 ignored
```
✅ All existing tests pass

### Release Build
```bash
$ cargo build --release
   Finished `release` profile [optimized] target(s) in 1m 17s
```
✅ Release build successful

## Usage Example (Once View Layer Complete)

### Drag and Drop
1. Click and hold on a tab
2. Drag left or right
3. Tab swaps position when crossing threshold
4. Release to finalize

### Context Menu
1. Right-click on a tab
2. Menu appears with options:
   - Duplicate Tab
   - Rename Tab
   - Close Tab
3. Click option or click elsewhere to close

### Tab Rename
1. Double-click tab title, OR
2. Right-click → "Rename Tab"
3. Type new name
4. Press Enter to save, Esc to cancel

## Current Workarounds

Until view layer is implemented, users can:
- **Duplicate Tab**: Cmd+Shift+D (already functional)
- **Close Tab**: Click X button or Cmd+W
- **Switch Tabs**: Cmd+[ / Cmd+] or Cmd+1-9
- **New Tab**: Cmd+T or click + button

Tab titles can be set via OSC escape sequences:
```bash
echo -e "\033]0;My Custom Title\007"
```

## Next Steps

1. **Implement mouse event subscription** in `AgTerm::subscription()`
2. **Add hit testing** to determine which tab was clicked
3. **Render context menu** when `tab_context_menu` is `Some`
4. **Replace tab label with text input** when `tab_rename_mode` is `Some`
5. **Add visual feedback** for drag state (opacity, shadow, cursor)

## Architecture Benefits

The current implementation separates concerns:
- **State management**: Complete ✅
- **Business logic**: Complete ✅
- **UI interaction**: To be implemented

This makes it easy to:
- Test state changes independently
- Swap UI frameworks if needed
- Add keyboard shortcuts for tab reordering
- Implement touch gestures on mobile

## Files Modified

- `/Users/yunwoopc/SIDE-PROJECT/agterm/src/main.rs`
  - Lines 585-597: Data structures
  - Lines 380-387: AgTerm state fields
  - Lines 524-527: State initialization
  - Lines 762-773: Message enum variants
  - Lines 1720-1815: Message handlers

## Commit Recommendation

```bash
git add src/main.rs
git commit -m "feat: add tab drag reordering infrastructure

- Add TabDragState and TabContextMenu structs
- Add drag, context menu, and rename message types
- Implement message handlers for tab operations
- Add state tracking to AgTerm
- Tab swapping logic based on drag position
- Rename mode with input buffer

Note: View layer interactions not yet implemented.
Mouse event handling required for full functionality."
```

## Related Features

This infrastructure also enables:
- **Tab pinning** (prevent close/reorder)
- **Tab grouping** (color coding, collapse/expand)
- **Tab history** (undo close, reopen)
- **Keyboard shortcuts** for tab reordering (Cmd+Shift+[ / ])

## Documentation

See existing tab features:
- Tab creation: `Message::NewTab` (line 942)
- Tab duplication: `Message::DuplicateTab` (line 984)
- Tab closing: `Message::CloseTab` (line 1033)
- Tab switching: `Message::SelectTab` (line 1048)

Tab state is persisted in sessions (see `SessionState` struct and `save_to_file` method).
