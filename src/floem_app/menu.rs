//! Menu bar implementation for AgTerm
//!
//! This module provides a custom menu bar for the application with macOS-style menus.
//! Since Floem 0.2's native menu support is not fully implemented, we create a custom
//! menu bar UI that integrates with the application.

use floem::prelude::*;
use floem::views::{h_stack, v_stack, text, empty, dyn_container, Decorators};
use crate::floem_app::state::AppState;
use crate::floem_app::theme;

/// Menu item definition
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct MenuItemDef {
    pub label: String,
    pub shortcut: Option<String>,
    pub enabled: bool,
    pub action: MenuAction,
}

/// Menu actions that can be triggered
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum MenuAction {
    // File menu
    NewTab,
    NewWindow,
    CloseTab,
    CloseWindow,

    // Edit menu
    Copy,
    Paste,
    SelectAll,

    // View menu
    ZoomIn,
    ZoomOut,
    ResetZoom,
    ToggleTheme,

    // Window menu
    SplitVertically,
    SplitHorizontally,
    NextPane,
    PreviousPane,

    // Special
    Separator,
    NoOp,
}

#[allow(dead_code)]
impl MenuItemDef {
    pub fn new(label: impl Into<String>, shortcut: Option<String>, action: MenuAction) -> Self {
        Self {
            label: label.into(),
            shortcut,
            enabled: true,
            action,
        }
    }

    pub fn separator() -> Self {
        Self {
            label: String::new(),
            shortcut: None,
            enabled: false,
            action: MenuAction::Separator,
        }
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Menu definition
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct MenuDef {
    pub title: String,
    pub items: Vec<MenuItemDef>,
}

#[allow(dead_code)]
impl MenuDef {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            items: Vec::new(),
        }
    }

    pub fn item(mut self, item: MenuItemDef) -> Self {
        self.items.push(item);
        self
    }

    pub fn separator(mut self) -> Self {
        self.items.push(MenuItemDef::separator());
        self
    }
}

/// Creates the complete menu bar structure
#[allow(dead_code)]
pub fn create_menu_structure() -> Vec<MenuDef> {
    vec![
        // File Menu
        MenuDef::new("File")
            .item(MenuItemDef::new("New Tab", Some("Cmd+T".to_string()), MenuAction::NewTab))
            .item(MenuItemDef::new("New Window", Some("Cmd+N".to_string()), MenuAction::NewWindow))
            .separator()
            .item(MenuItemDef::new("Close Tab", Some("Cmd+W".to_string()), MenuAction::CloseTab))
            .item(MenuItemDef::new("Close Window", Some("Cmd+Shift+W".to_string()), MenuAction::CloseWindow)),

        // Edit Menu
        MenuDef::new("Edit")
            .item(MenuItemDef::new("Copy", Some("Cmd+C".to_string()), MenuAction::Copy))
            .item(MenuItemDef::new("Paste", Some("Cmd+V".to_string()), MenuAction::Paste))
            .separator()
            .item(MenuItemDef::new("Select All", Some("Cmd+A".to_string()), MenuAction::SelectAll)),

        // View Menu
        MenuDef::new("View")
            .item(MenuItemDef::new("Zoom In", Some("Cmd++".to_string()), MenuAction::ZoomIn))
            .item(MenuItemDef::new("Zoom Out", Some("Cmd+-".to_string()), MenuAction::ZoomOut))
            .item(MenuItemDef::new("Reset Zoom", Some("Cmd+0".to_string()), MenuAction::ResetZoom))
            .separator()
            .item(MenuItemDef::new("Toggle Theme", None, MenuAction::ToggleTheme)),

        // Window Menu
        MenuDef::new("Window")
            .item(MenuItemDef::new("Split Vertically", Some("Cmd+D".to_string()), MenuAction::SplitVertically))
            .item(MenuItemDef::new("Split Horizontally", Some("Cmd+Shift+D".to_string()), MenuAction::SplitHorizontally))
            .separator()
            .item(MenuItemDef::new("Next Pane", Some("Cmd+Tab".to_string()), MenuAction::NextPane))
            .item(MenuItemDef::new("Previous Pane", Some("Cmd+Shift+Tab".to_string()), MenuAction::PreviousPane)),
    ]
}

/// Execute a menu action
#[allow(dead_code)]
pub fn execute_menu_action(app_state: &AppState, action: &MenuAction) {
    use crate::floem_app::pane::NavigationDirection;

    match action {
        MenuAction::NewTab => {
            tracing::info!("Menu: New Tab");
            app_state.add_tab();
        }

        MenuAction::NewWindow => {
            tracing::info!("Menu: New Window (not implemented)");
            // TODO: Implement new window functionality
        }

        MenuAction::CloseTab => {
            tracing::info!("Menu: Close Tab");
            let active_idx = app_state.active_tab.get();
            app_state.close_tab(active_idx);
        }

        MenuAction::CloseWindow => {
            tracing::info!("Menu: Close Window (not implemented)");
            // TODO: Implement close window functionality
        }

        MenuAction::Copy => {
            tracing::info!("Menu: Copy");
            // TODO: Implement copy functionality
        }

        MenuAction::Paste => {
            tracing::info!("Menu: Paste");
            if let Some(active_tab) = app_state.active_tab_ref() {
                let tree = active_tab.pane_tree.get();
                if let Some((_, terminal_state)) = tree.get_focused_leaf() {
                    if let Some(session_id) = terminal_state.pty_session() {
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            if let Ok(text) = clipboard.get_text() {
                                let _ = app_state.pty_manager.write(&session_id, text.as_bytes());
                            }
                        }
                    }
                }
            }
        }

        MenuAction::SelectAll => {
            tracing::info!("Menu: Select All");
            // TODO: Implement select all functionality
        }

        MenuAction::ZoomIn => {
            tracing::info!("Menu: Zoom In");
            app_state.increase_font_size();
        }

        MenuAction::ZoomOut => {
            tracing::info!("Menu: Zoom Out");
            app_state.decrease_font_size();
        }

        MenuAction::ResetZoom => {
            tracing::info!("Menu: Reset Zoom");
            app_state.reset_font_size();
        }

        MenuAction::ToggleTheme => {
            tracing::info!("Menu: Toggle Theme");
            app_state.toggle_theme();
        }

        MenuAction::SplitVertically => {
            tracing::info!("Menu: Split Vertically");
            if let Some(active_tab) = app_state.active_tab_ref() {
                let mut tree = active_tab.pane_tree.get();
                if let Some((focused_id, _)) = tree.get_focused_leaf() {
                    split_pane_recursive(&mut tree, focused_id, true, &app_state.pty_manager);
                    active_tab.pane_tree.set(tree);
                }
            }
        }

        MenuAction::SplitHorizontally => {
            tracing::info!("Menu: Split Horizontally");
            if let Some(active_tab) = app_state.active_tab_ref() {
                let mut tree = active_tab.pane_tree.get();
                if let Some((focused_id, _)) = tree.get_focused_leaf() {
                    split_pane_recursive(&mut tree, focused_id, false, &app_state.pty_manager);
                    active_tab.pane_tree.set(tree);
                }
            }
        }

        MenuAction::NextPane => {
            tracing::info!("Menu: Next Pane");
            if let Some(active_tab) = app_state.active_tab_ref() {
                let tree = active_tab.pane_tree.get();
                if let Some(next_id) = tree.navigate(NavigationDirection::Next) {
                    tree.clear_focus();
                    tree.set_focus(next_id);
                    active_tab.pane_tree.set(tree);
                }
            }
        }

        MenuAction::PreviousPane => {
            tracing::info!("Menu: Previous Pane");
            if let Some(active_tab) = app_state.active_tab_ref() {
                let tree = active_tab.pane_tree.get();
                if let Some(prev_id) = tree.navigate(NavigationDirection::Previous) {
                    tree.clear_focus();
                    tree.set_focus(prev_id);
                    active_tab.pane_tree.set(tree);
                }
            }
        }

        MenuAction::Separator | MenuAction::NoOp => {}
    }
}

/// Helper function to split a specific pane in the tree
#[allow(dead_code)]
fn split_pane_recursive(
    tree: &mut crate::floem_app::pane::PaneTree,
    target_id: uuid::Uuid,
    vertical: bool,
    pty_manager: &std::sync::Arc<crate::terminal::pty::PtyManager>,
) -> bool {
    use crate::floem_app::pane::PaneTree;

    match tree {
        PaneTree::Leaf { id, .. } => {
            if *id == target_id {
                if vertical {
                    tree.split_vertical(pty_manager);
                } else {
                    tree.split_horizontal(pty_manager);
                }
                true
            } else {
                false
            }
        }
        PaneTree::Split { first, second, .. } => {
            let mut first_val = first.get();
            if split_pane_recursive(&mut first_val, target_id, vertical, pty_manager) {
                first.set(first_val);
                return true;
            }

            let mut second_val = second.get();
            if split_pane_recursive(&mut second_val, target_id, vertical, pty_manager) {
                second.set(second_val);
                return true;
            }

            false
        }
    }
}

/// Creates a visual menu bar view (for platforms where native menus aren't available)
#[allow(dead_code)]
pub fn menu_bar_view(app_state: &AppState) -> impl IntoView {
    let menus = create_menu_structure();
    let app_state_clone = app_state.clone();

    h_stack((
        menus.into_iter().map(move |menu| {
            menu_view(menu, app_state_clone.clone())
        }).collect::<Vec<_>>(),
    ))
    .style(|s| {
        s.width_full()
            .height(30.0)
            .background(theme::colors::BG_SECONDARY)
            .border_bottom(1.0)
            .border_color(theme::colors::BORDER)
            .align_items(floem::style::AlignItems::Center)
            .padding_left(10.0)
            .padding_right(10.0)
            .gap(10.0)
    })
}

/// Creates a single menu button with dropdown
#[allow(dead_code)]
fn menu_view(menu: MenuDef, app_state: AppState) -> impl IntoView {
    let is_open = RwSignal::new(false);
    let menu_clone = menu.clone();
    let app_state_clone = app_state.clone();
    let is_open_for_dropdown = is_open.clone();

    v_stack((
        // Menu button
        text(menu.title.clone())
            .on_click_stop(move |_| {
                is_open.update(|open| *open = !*open);
            })
            .style(move |s| {
                s.padding(5.0)
                    .padding_left(10.0)
                    .padding_right(10.0)
                    .border_radius(4.0)
                    .cursor(floem::style::CursorStyle::Pointer)
                    .apply_if(is_open.get(), |s| {
                        s.background(theme::colors::SURFACE_HOVER)
                    })
                    .hover(|s| {
                        s.background(theme::colors::SURFACE_HOVER)
                    })
            }),

        // Dropdown menu (shown when open)
        {
            let menu = menu_clone;
            let app_state = app_state_clone;

            dyn_container(
                move || is_open_for_dropdown.get(),
                move |is_open| {
                    if is_open {
                        v_stack((
                            menu.items.iter().map(|item| {
                                menu_item_view(item.clone(), app_state.clone(), is_open_for_dropdown)
                            }).collect::<Vec<_>>(),
                        ))
                        .style(|s| {
                            s.position(floem::style::Position::Absolute)
                                .inset_top(30.0)
                                .inset_left(0.0)
                                .z_index(1000)
                                .min_width(250.0)
                                .background(theme::colors::BG_SECONDARY)
                                .border(1.0)
                                .border_color(theme::colors::BORDER)
                                .border_radius(4.0)
                                .box_shadow_blur(10.0)
                        })
                        .into_any()
                    } else {
                        empty()
                            .style(|s| s.display(floem::style::Display::None))
                            .into_any()
                    }
                },
            )
        },
    ))
    .style(|s| {
        s.position(floem::style::Position::Relative)
    })
}

/// Creates a menu item view
#[allow(dead_code)]
fn menu_item_view(item: MenuItemDef, app_state: AppState, menu_open: RwSignal<bool>) -> impl IntoView {
    if matches!(item.action, MenuAction::Separator) {
        return empty()
            .style(|s| {
                s.width_full()
                    .height(1.0)
                    .background(theme::colors::BORDER)
                    .margin_top(4.0)
                    .margin_bottom(4.0)
            })
            .into_any();
    }

    let item_clone = item.clone();
    let app_state_clone = app_state.clone();

    h_stack((
        text(item.label.clone())
            .style(|s| {
                s.flex_grow(1.0)
            }),
        item.shortcut.map(|shortcut| {
            text(shortcut)
                .style(|s| {
                    s.color(theme::colors::TEXT_SECONDARY)
                        .font_size(11.0)
                })
                .into_any()
        }).unwrap_or_else(|| empty().into_any()),
    ))
    .on_click_stop(move |_| {
        if item_clone.enabled {
            execute_menu_action(&app_state_clone, &item_clone.action);
            menu_open.set(false);
        }
    })
    .style(move |s| {
        s.width_full()
            .padding(8.0)
            .padding_left(12.0)
            .padding_right(12.0)
            .cursor(floem::style::CursorStyle::Pointer)
            .apply_if(!item.enabled, |s| {
                s.color(theme::colors::TEXT_DISABLED)
                    .cursor(floem::style::CursorStyle::Default)
            })
            .apply_if(item.enabled, |s| {
                s.hover(|s| {
                    s.background(theme::colors::SURFACE_HOVER)
                })
            })
    })
    .into_any()
}

/// Try to use native macOS menu if available
///
/// Note: As of Floem 0.2, native menu support for macOS is not fully implemented.
/// This function is a placeholder for future native menu integration.
#[allow(dead_code)]
pub fn try_setup_native_menu(_app_state: &AppState) -> bool {
    // Check if we're on macOS
    #[cfg(target_os = "macos")]
    {
        tracing::info!("Native macOS menu support is not yet available in Floem 0.2");
        // When Floem implements native menu support, we'll use it here
        // For now, we'll use the custom menu bar
        false
    }

    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}
