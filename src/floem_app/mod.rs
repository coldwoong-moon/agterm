//! Floem-based GUI Application for AgTerm

mod state;
mod views;
pub mod theme;
pub mod pane;
pub mod settings;
pub mod menu;
pub mod async_bridge;
pub mod mcp_client;
pub mod command_validator;

use floem::prelude::*;
use floem::views::{v_stack, h_stack, Decorators, stack, dyn_container};
use floem::keyboard::{Key, NamedKey};
use std::time::Duration;

pub use state::AppState;
use views::McpPanelState;
use async_bridge::AsyncBridge;

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

    // Create AsyncBridge for MCP communication
    let (bridge, worker) = AsyncBridge::new();

    // Spawn the worker on the Tokio runtime
    tokio::spawn(async move {
        worker.run().await;
    });

    // MCP panel state with bridge connection
    let mcp_panel_state = McpPanelState::with_bridge(
        bridge.command_tx().clone(),
        bridge.into_result_rx(),
    );

    // Clone for polling in idle callback
    let mcp_panel_for_polling = mcp_panel_state.clone();

    // Set up periodic polling for MCP results using a timer
    // This spawns a background task that triggers result polling
    let poll_trigger = RwSignal::new(0u64);
    {
        let poll_trigger = poll_trigger;
        let mcp_state = mcp_panel_for_polling.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(100)).await;
                // Trigger a poll by incrementing the counter
                // The poll happens in the main thread via the effect below
                poll_trigger.update(|v| *v = v.wrapping_add(1));
                // Also poll directly here to process results
                mcp_state.poll_results();
            }
        });
    }

    // Enable IME input
    floem::action::set_ime_allowed(true);

    stack((
        // Main application view
        v_stack((
            views::tab_bar(&app_state),
            // Horizontal layout: terminal area + MCP panel
            h_stack((
                views::terminal_area(&app_state)
                    .style(|s| s.flex_grow(1.0).height_full()),
                views::mcp_panel(mcp_panel_state.clone(), app_state.theme),
            ))
            .style(|s| s.width_full().height_full().flex_grow(1.0)),
            views::status_bar(&app_state),
        ))
        .style(|s| {
            s.width_full()
                .height_full()
                .background(theme::colors::BG_PRIMARY)
        }),

        // Settings overlay (shown conditionally)
        // IMPORTANT: Only render overlay when visible to avoid blocking mouse events
        dyn_container(
            move || settings_visible.get(),
            move |is_visible| {
                if is_visible {
                    views::settings_panel(&app_state_settings, settings_visible)
                        .style(|s| {
                            s.position(floem::style::Position::Absolute)
                                .inset(0.0)
                        })
                        .into_any()
                } else {
                    // Return empty view with display:none to not intercept events
                    floem::views::empty()
                        .style(|s| s.display(floem::style::Display::None))
                        .into_any()
                }
            }
        ),
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
                // Handle Shift+Cmd+T for theme toggle
                if ch.as_str() == "t" && key_event.modifiers.meta() && key_event.modifiers.shift() {
                    tracing::info!("Theme toggle shortcut triggered (Shift+Cmd+T)");
                    app_state_keyboard.toggle_theme();
                    return floem::event::EventPropagation::Stop;
                }
                // Handle Cmd+T for new tab
                if ch.as_str() == "t" && key_event.modifiers.meta() && !key_event.modifiers.shift() {
                    tracing::info!("New tab shortcut triggered (Cmd+T)");
                    app_state_keyboard.new_tab();
                    return floem::event::EventPropagation::Stop;
                }
                // Handle Cmd+Shift+W to close pane (before Cmd+W for tab)
                if ch.as_str() == "w" && key_event.modifiers.meta() && key_event.modifiers.shift() {
                    tracing::info!("Close pane shortcut triggered (Cmd+Shift+W)");
                    app_state_keyboard.close_focused_pane();
                    return floem::event::EventPropagation::Stop;
                }
                // Handle Cmd+W to close tab
                if ch.as_str() == "w" && key_event.modifiers.meta() && !key_event.modifiers.shift() {
                    tracing::info!("Close tab shortcut triggered (Cmd+W)");
                    app_state_keyboard.close_active_tab();
                    return floem::event::EventPropagation::Stop;
                }
                // Handle Cmd+D for vertical split (top/bottom)
                if ch.as_str() == "d" && key_event.modifiers.meta() && !key_event.modifiers.shift() {
                    tracing::info!("Split pane vertical shortcut triggered (Cmd+D)");
                    app_state_keyboard.split_pane_vertical();
                    return floem::event::EventPropagation::Stop;
                }
                // Handle Cmd+Shift+D for horizontal split (left/right)
                if ch.as_str() == "d" && key_event.modifiers.meta() && key_event.modifiers.shift() {
                    tracing::info!("Split pane horizontal shortcut triggered (Cmd+Shift+D)");
                    app_state_keyboard.split_pane_horizontal();
                    return floem::event::EventPropagation::Stop;
                }
                // Handle Cmd+] for next pane
                if ch.as_str() == "]" && key_event.modifiers.meta() {
                    tracing::info!("Next pane shortcut triggered (Cmd+])");
                    app_state_keyboard.next_pane();
                    return floem::event::EventPropagation::Stop;
                }
                // Handle Cmd+[ for previous pane
                if ch.as_str() == "[" && key_event.modifiers.meta() {
                    tracing::info!("Previous pane shortcut triggered (Cmd+[)");
                    app_state_keyboard.previous_pane();
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
