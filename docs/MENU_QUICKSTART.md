# Menu Bar Quick Start Guide

## Overview

AgTerm includes a custom menu bar implementation for macOS-style menus. This guide shows you how to use it.

## Quick Enable

To add the menu bar to your application:

### Step 1: Edit `src/floem_app/mod.rs`

```rust
pub fn app_view() -> impl IntoView {
    let app_state = AppState::new();
    let app_state_clone = app_state.clone();

    v_stack((
        // Add this line:
        menu::menu_bar_view(&app_state),

        // Existing views:
        views::tab_bar(&app_state),
        views::terminal_area(&app_state),
        views::status_bar(&app_state),
    ))
    .on_event(floem::event::EventListener::KeyDown, move |event| {
        // ... existing code
    })
    .style(|s| {
        s.width_full()
            .height_full()
            .background(theme::colors::BG_PRIMARY)
    })
}
```

### Step 2: Run

```bash
cargo run --bin agterm-floem --features floem-gui --no-default-features
```

## Available Menus

| Menu | Items |
|------|-------|
| **File** | New Tab, New Window, Close Tab, Close Window |
| **Edit** | Copy, Paste, Select All |
| **View** | Zoom In, Zoom Out, Reset Zoom, Toggle Theme |
| **Window** | Split Vertically, Split Horizontally, Next Pane, Previous Pane |

## Keyboard Shortcuts

All shortcuts work whether or not the menu bar is visible:

### File
- `Cmd+T` - New Tab
- `Cmd+N` - New Window (coming soon)
- `Cmd+W` - Close Tab
- `Cmd+Shift+W` - Close Window (coming soon)

### Edit
- `Cmd+C` - Copy (coming soon)
- `Cmd+V` - Paste
- `Cmd+A` - Select All (coming soon)

### View
- `Cmd++` - Zoom In
- `Cmd+-` - Zoom Out
- `Cmd+0` - Reset Zoom
- Toggle Theme (via menu)

### Window
- `Cmd+D` - Split Vertically
- `Cmd+Shift+D` - Split Horizontally
- `Cmd+Tab` - Next Pane
- `Cmd+Shift+Tab` - Previous Pane

## Customization

### Adding a New Menu Item

1. **Add Action**: Edit `src/floem_app/menu.rs`

```rust
#[derive(Clone, Debug)]
pub enum MenuAction {
    // ... existing actions
    MyNewAction,
}
```

2. **Add to Menu**: In `create_menu_structure()`

```rust
MenuDef::new("File")
    .item(MenuItemDef::new(
        "My Action",
        Some("Cmd+X".to_string()),
        MenuAction::MyNewAction
    ))
```

3. **Implement Handler**: In `execute_menu_action()`

```rust
match action {
    // ... existing matches
    MenuAction::MyNewAction => {
        tracing::info!("My action triggered");
        // Your implementation here
    }
}
```

### Changing Menu Colors

Edit `src/floem_app/theme.rs`:

```rust
pub mod colors {
    // Menu background
    pub const BG_SECONDARY: Color = Color::rgb8(30, 30, 38);

    // Menu hover
    pub const SURFACE_HOVER: Color = Color::rgb8(40, 40, 50);

    // Disabled items
    pub const TEXT_DISABLED: Color = Color::rgb8(80, 83, 95);
}
```

## API Reference

### Menu Creation

```rust
use agterm::floem_app::menu;

// Create menu structure
let menus = menu::create_menu_structure();

// Create menu bar view
let menu_bar = menu::menu_bar_view(&app_state);

// Execute a menu action manually
menu::execute_menu_action(&app_state, &menu::MenuAction::NewTab);
```

### Menu Types

```rust
// Define a menu
pub struct MenuDef {
    pub title: String,
    pub items: Vec<MenuItemDef>,
}

// Define a menu item
pub struct MenuItemDef {
    pub label: String,
    pub shortcut: Option<String>,
    pub enabled: bool,
    pub action: MenuAction,
}
```

## Troubleshooting

### Menu doesn't appear

1. Check that you added `menu::menu_bar_view(&app_state)` to the v_stack
2. Verify you're using the floem-gui feature: `--features floem-gui --no-default-features`
3. Check console for errors: `RUST_LOG=debug cargo run ...`

### Shortcuts don't work

Shortcuts are independent of the menu bar. They're handled in `handle_global_shortcuts()` in `mod.rs`. Check:

1. Is your keyboard using Cmd (macOS) or Super (Linux/Windows)?
2. Check logs: `tracing::info!("Shortcut triggered")`

### Menu items don't do anything

1. Check `execute_menu_action()` implementation
2. Verify the action is not a placeholder (e.g., Copy, Select All)
3. Check logs for error messages

## Examples

See `examples/menu_example.rs` for a working example:

```bash
cargo run --example menu_example --features floem-gui --no-default-features
```

## Further Reading

- [Full Implementation Details](../MENU_IMPLEMENTATION.md)
- [Implementation Summary](../MENU_IMPLEMENTATION_SUMMARY.md)
- [Floem Documentation](https://docs.rs/floem)

## Contributing

To contribute menu improvements:

1. Add/modify actions in `menu.rs`
2. Test with `cargo check` and `cargo run`
3. Update documentation
4. Submit PR with examples

## License

Same as AgTerm (MIT)
