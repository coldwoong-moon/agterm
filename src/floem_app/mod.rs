//! Floem-based GUI Application for AgTerm

mod state;
mod views;
pub mod theme;
pub mod pane;
pub mod settings;
pub mod menu;
pub mod async_bridge;

use floem::prelude::*;
use floem::views::{v_stack, h_stack, Decorators, stack, dyn_container};
use floem::keyboard::{Key, NamedKey};

pub use state::AppState;
use views::McpPanelState;

/// Convert keyboard input to bytes for PTY
fn convert_key_to_bytes(key: &Key, modifiers: &floem::keyboard::Modifiers) -> Option<Vec<u8>> {
    match key {
        // Named keys
        Key::Named(named) => match named {
            NamedKey::Enter => Some(b"\r".to_vec()),
            NamedKey::Backspace => Some(b"\x7f".to_vec()),
            NamedKey::Tab => Some(b"\t".to_vec()),
            NamedKey::Escape => Some(b"\x1b".to_vec()),
            NamedKey::ArrowUp => Some(b"\x1b[A".to_vec()),
            NamedKey::ArrowDown => Some(b"\x1b[B".to_vec()),
            NamedKey::ArrowRight => Some(b"\x1b[C".to_vec()),
            NamedKey::ArrowLeft => Some(b"\x1b[D".to_vec()),
            NamedKey::Home => Some(b"\x1b[H".to_vec()),
            NamedKey::End => Some(b"\x1b[F".to_vec()),
            NamedKey::PageUp => Some(b"\x1b[5~".to_vec()),
            NamedKey::PageDown => Some(b"\x1b[6~".to_vec()),
            NamedKey::Delete => Some(b"\x1b[3~".to_vec()),
            NamedKey::Space => Some(b" ".to_vec()),
            _ => None,
        },
        // Character keys
        Key::Character(ch) => {
            let ch_str = ch.as_str();

            // Skip if Cmd/Super modifier is pressed (reserved for shortcuts)
            if modifiers.meta() {
                return None;
            }

            // Handle Ctrl combinations
            if modifiers.control() {
                if let Some(c) = ch_str.chars().next() {
                    match c.to_ascii_lowercase() {
                        'a'..='z' => {
                            // Ctrl+A = 0x01, Ctrl+B = 0x02, etc.
                            let ctrl_byte = (c.to_ascii_lowercase() as u8) - b'a' + 1;
                            Some(vec![ctrl_byte])
                        }
                        '[' => Some(b"\x1b".to_vec()),  // Ctrl+[
                        ']' => Some(b"\x1d".to_vec()),  // Ctrl+]
                        '\\' => Some(b"\x1c".to_vec()), // Ctrl+\
                        _ => None,
                    }
                } else {
                    None
                }
            } else {
                // Regular character input
                Some(ch_str.as_bytes().to_vec())
            }
        }
        _ => None,
    }
}

/// Main application view
pub fn app_view() -> impl IntoView {
    tracing::info!("Initializing Floem application view");
    let app_state = AppState::new();

    // Clone app_state for different closures
    let app_state_settings = app_state.clone();
    let app_state_keyboard = app_state.clone();

    // Settings panel visibility state
    let settings_visible = RwSignal::new(false);

    // MCP panel state
    let mcp_panel_state = McpPanelState::new();

    // Enable IME input
    floem::action::set_ime_allowed(true);

    stack((
        // Main application view
        v_stack((
            views::tab_bar(&app_state),
            // Horizontal layout: terminal area + MCP panel
            h_stack((
                views::terminal_area(&app_state)
                    .style(|s| s.flex_grow(1.0)),
                views::mcp_panel(mcp_panel_state.clone(), app_state.theme.get()),
            ))
            .style(|s| s.width_full().flex_grow(1.0)),
            views::status_bar(&app_state),
        ))
        .style(|s| {
            s.width_full()
                .height_full()
                .background(theme::colors::BG_PRIMARY)
        }),

        // Settings overlay (shown conditionally)
        dyn_container(
            move || settings_visible.get(),
            move |is_visible| {
                if is_visible {
                    views::settings_panel(&app_state_settings, settings_visible).into_any()
                } else {
                    floem::views::empty().into_any()
                }
            }
        )
        .style(|s| {
            s.position(floem::style::Position::Absolute)
                .inset(0.0)
        }),
    ))
    .on_event(floem::event::EventListener::KeyDown, move |event| {
        if let floem::event::Event::KeyDown(key_event) = event {
            tracing::info!("App-level KeyDown: {:?}", key_event.key.logical_key);

            // Handle Cmd+, to toggle settings
            if let Key::Character(ch) = &key_event.key.logical_key {
                if ch.as_str() == "," && key_event.modifiers.meta() {
                    tracing::info!("Settings shortcut triggered (Cmd+,)");
                    settings_visible.set(!settings_visible.get());
                    return floem::event::EventPropagation::Stop;
                }
                // Handle Cmd+M to toggle MCP panel
                if ch.as_str() == "m" && key_event.modifiers.meta() {
                    tracing::info!("MCP panel toggle triggered (Cmd+M)");
                    mcp_panel_state.toggle_visibility();
                    return floem::event::EventPropagation::Stop;
                }
                // Handle Cmd+T for new tab
                if ch.as_str() == "t" && key_event.modifiers.meta() {
                    tracing::info!("New tab shortcut triggered (Cmd+T)");
                    app_state_keyboard.new_tab();
                    return floem::event::EventPropagation::Stop;
                }
                // Handle Cmd+W to close tab
                if ch.as_str() == "w" && key_event.modifiers.meta() {
                    tracing::info!("Close tab shortcut triggered (Cmd+W)");
                    app_state_keyboard.close_active_tab();
                    return floem::event::EventPropagation::Stop;
                }
            }

            // Skip if settings panel is visible
            if settings_visible.get() {
                return floem::event::EventPropagation::Continue;
            }

            // Forward all other key events to active terminal
            let bytes = convert_key_to_bytes(&key_event.key.logical_key, &key_event.modifiers);
            if let Some(bytes) = bytes {
                // Get active terminal's PTY session and send bytes
                if let Some(active_tab) = app_state_keyboard.active_tab_ref() {
                    if let Some(pty_session) = active_tab.get_focused_pty_session() {
                        if let Err(e) = app_state_keyboard.pty_manager.write(&pty_session, &bytes) {
                            tracing::error!("Failed to write to PTY: {}", e);
                        } else {
                            tracing::trace!("Sent {:?} to PTY {:?}", bytes, pty_session);
                        }
                        return floem::event::EventPropagation::Stop;
                    }
                }
            }
        }
        floem::event::EventPropagation::Continue
    })
    .style(|s| {
        s.width_full()
            .height_full()
    })
}
