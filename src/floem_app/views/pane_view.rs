//! Pane View Component - Recursive rendering of split panes
//!
//! This module provides the view rendering for the PaneTree structure,
//! recursively building split layouts with draggable dividers.

use floem::prelude::*;
use floem::reactive::{create_signal, SignalGet, SignalUpdate};
use floem::views::{container, h_stack, label, v_stack, Decorators};
use floem::style::CursorStyle;
use floem::event::EventListener;
use std::sync::Arc;

use crate::floem_app::pane::{PaneTree, SplitDirection};
use crate::floem_app::state::AppState;
use crate::floem_app::views::terminal::{TerminalCanvas, TerminalState};

/// Render a pane tree recursively
pub fn pane_tree_view(
    tree_signal: RwSignal<PaneTree>,
    app_state: AppState,
) -> impl IntoView {
    // Create a dynamic view that rebuilds when tree changes
    let app_state_clone = app_state.clone();
    let app_state_style = app_state.clone();

    container(
        dyn_container(
            move || tree_signal.get(),
            move |tree| {
                render_pane_tree(tree, app_state_clone.clone())
            }
        )
    )
    .style(move |s| {
        let colors = app_state_style.colors();
        s.width_full()
            .height_full()
            .background(colors.bg_primary)
    })
}

/// Recursively render the pane tree structure
fn render_pane_tree(tree: PaneTree, app_state: AppState) -> impl IntoView {
    match tree {
        PaneTree::Leaf { id, terminal_state, is_focused } => {
            render_leaf_pane(id, terminal_state, is_focused, app_state).into_any()
        }
        PaneTree::Split { direction, first, second, ratio } => {
            render_split_pane(direction, first, second, ratio, app_state).into_any()
        }
    }
}

/// Render a leaf pane (terminal)
fn render_leaf_pane(
    _id: uuid::Uuid,
    terminal_state: TerminalState,
    is_focused: RwSignal<bool>,
    app_state: AppState,
) -> impl IntoView {
    // Get PTY session info
    let session_info = if let Some(sid) = terminal_state.pty_session() {
        format!("PTY: {}", &sid.to_string()[..8])
    } else {
        "No PTY".to_string()
    };

    let terminal_state_clone = terminal_state.clone();
    let terminal_state_ime_label = terminal_state.clone();
    let terminal_state_ime_label_style = terminal_state.clone();
    let app_state_canvas = app_state.clone();
    let app_state_header = app_state.clone();
    let app_state_label = app_state.clone();
    let app_state_ime_label = app_state.clone();
    let app_state_ime_style = app_state.clone();
    let app_state_canvas_bg = app_state.clone();
    let app_state_style = app_state.clone();

    // Create terminal canvas directly (repaint is triggered via ViewId.request_paint())
    let terminal_canvas_view = TerminalCanvas::new(terminal_state_clone.clone(), app_state_canvas.clone(), is_focused);

    container(
        v_stack((
            // Header bar with session info
            container(
                label(move || session_info.clone())
                    .style(move |s| {
                        let colors = app_state_label.colors();
                        s.font_size(11.0)
                            .font_family("monospace".to_string())
                            .color(colors.text_secondary)
                            .padding(4.0)
                    })
            )
            .style(move |s| {
                let colors = app_state_header.colors();
                s.width_full()
                    .background(colors.bg_secondary)
                    .border_bottom(1.0)
                    .border_color(colors.border)
            }),

            // Terminal canvas area with keyboard and IME input handling
            container(
                v_stack((
                    terminal_canvas_view,
                    // IME composing text overlay
                    container(
                        label(move || {
                            let composing = terminal_state_ime_label.ime_composing.get();
                            if composing.is_empty() {
                                "".to_string()
                            } else {
                                format!("IME: {}", composing)
                            }
                        })
                        .style(move |s| {
                            let colors = app_state_ime_label.colors();
                            s.font_size(12.0)
                                .font_family("monospace".to_string())
                                .color(colors.accent_blue)
                                .padding(4.0)
                        })
                    )
                    .style(move |s| {
                        let composing = terminal_state_ime_label_style.ime_composing.get();
                        let colors = app_state_ime_style.colors();
                        if composing.is_empty() {
                            s.display(floem::style::Display::None)
                        } else {
                            s.position(floem::style::Position::Absolute)
                                .inset_left(10.0)
                                .inset_bottom(10.0)
                                .background(colors.bg_secondary)
                                .border(1.0)
                                .border_color(colors.accent_blue)
                                .border_radius(4.0)
                        }
                    }),
                ))
            )
                // NOTE: Keyboard events are handled at the app level (mod.rs)
                // to ensure consistent routing to the active terminal
                // IME events
                .on_event_stop(EventListener::ImeEnabled, {
                    let terminal_state_ime = terminal_state.clone();
                    move |_event| {
                        if is_focused.get() {
                            tracing::debug!("IME enabled");
                            terminal_state_ime.ime_composing.set(String::new());
                        }
                    }
                })
                .on_event_stop(EventListener::ImePreedit, {
                    let terminal_state_ime = terminal_state.clone();
                    move |event| {
                        if !is_focused.get() {
                            return;
                        }
                        if let floem::event::Event::ImePreedit { text, cursor } = event {
                            tracing::debug!("IME preedit: text='{}', cursor={:?}", text, cursor);
                            terminal_state_ime.ime_composing.set(text.clone());
                        }
                    }
                })
                .on_event_stop(EventListener::ImeCommit, {
                    let terminal_state_ime = terminal_state.clone();
                    let app_state_ime = app_state.clone();
                    move |event| {
                        if !is_focused.get() {
                            return;
                        }
                        if let floem::event::Event::ImeCommit(text) = event {
                            tracing::info!("IME commit: '{}'", text);

                            // Clear composing text
                            terminal_state_ime.ime_composing.set(String::new());

                            // Send committed text to PTY
                            if let Some(session_id) = terminal_state_ime.pty_session() {
                                let bytes = text.as_bytes();
                                if let Err(e) = app_state_ime.pty_manager.write(&session_id, bytes) {
                                    tracing::error!("Failed to write IME commit to PTY: {}", e);
                                } else {
                                    tracing::debug!("Sent {} bytes (IME) to PTY: {:?}", bytes.len(), text);
                                }
                            }
                        }
                    }
                })
                .on_event_stop(EventListener::ImeDisabled, {
                    let terminal_state_ime = terminal_state.clone();
                    move |_event| {
                        if is_focused.get() {
                            tracing::debug!("IME disabled");
                            terminal_state_ime.ime_composing.set(String::new());
                        }
                    }
                })
                // IMPORTANT: Make view keyboard navigable to receive keyboard events
                .keyboard_navigable()
                // Request focus when clicked
                .on_click_stop(move |_| {
                    tracing::debug!("Terminal container clicked - requesting focus");
                    is_focused.set(true);
                })
                .style(move |s| {
                    let colors = app_state_canvas_bg.colors();
                    s.flex_grow(1.0)
                        .width_full()
                        .height_full()
                        .background(colors.bg_primary)
                })
        ))
    )
    .style(move |s| {
        let colors = app_state_style.colors();
        let mut style = s
            .width_full()
            .height_full()
            .flex_col()
            .background(colors.bg_primary);

        // Add focus border
        if is_focused.get() {
            style = style
                .border(2.0)
                .border_color(colors.accent_blue);
        } else {
            style = style
                .border(1.0)
                .border_color(colors.border);
        }

        style
    })
}

/// Render a split pane (horizontal or vertical)
fn render_split_pane(
    direction: SplitDirection,
    first: Arc<RwSignal<PaneTree>>,
    second: Arc<RwSignal<PaneTree>>,
    ratio: RwSignal<f64>,
    app_state: AppState,
) -> impl IntoView {
    match direction {
        SplitDirection::Horizontal => {
            render_horizontal_split(first, second, ratio, app_state).into_any()
        }
        SplitDirection::Vertical => {
            render_vertical_split(first, second, ratio, app_state).into_any()
        }
    }
}

/// Render horizontal split (left | right)
fn render_horizontal_split(
    first: Arc<RwSignal<PaneTree>>,
    second: Arc<RwSignal<PaneTree>>,
    ratio: RwSignal<f64>,
    app_state: AppState,
) -> impl IntoView {
    let app_state_first = app_state.clone();
    let app_state_second = app_state.clone();
    let app_state_divider = app_state.clone();

    h_stack((
        // Left pane
        container(
            dyn_container(
                move || first.get(),
                move |tree| {
                    render_pane_tree(tree, app_state_first.clone())
                }
            )
        )
        .style(move |s| {
            s.flex_basis(0.0)
                .flex_grow(ratio.get() as f32)
                .height_full()
        }),

        // Vertical divider
        vertical_divider(ratio, app_state_divider),

        // Right pane
        container(
            dyn_container(
                move || second.get(),
                move |tree| {
                    render_pane_tree(tree, app_state_second.clone())
                }
            )
        )
        .style(move |s| {
            s.flex_basis(0.0)
                .flex_grow((1.0 - ratio.get()) as f32)
                .height_full()
        }),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
    })
}

/// Render vertical split (top / bottom)
fn render_vertical_split(
    first: Arc<RwSignal<PaneTree>>,
    second: Arc<RwSignal<PaneTree>>,
    ratio: RwSignal<f64>,
    app_state: AppState,
) -> impl IntoView {
    let app_state_first = app_state.clone();
    let app_state_second = app_state.clone();
    let app_state_divider = app_state.clone();

    v_stack((
        // Top pane
        container(
            dyn_container(
                move || first.get(),
                move |tree| {
                    render_pane_tree(tree, app_state_first.clone())
                }
            )
        )
        .style(move |s| {
            s.flex_basis(0.0)
                .flex_grow(ratio.get() as f32)
                .width_full()
        }),

        // Horizontal divider
        horizontal_divider(ratio, app_state_divider),

        // Bottom pane
        container(
            dyn_container(
                move || second.get(),
                move |tree| {
                    render_pane_tree(tree, app_state_second.clone())
                }
            )
        )
        .style(move |s| {
            s.flex_basis(0.0)
                .flex_grow((1.0 - ratio.get()) as f32)
                .width_full()
        }),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
    })
}

/// Vertical divider (for horizontal splits)
fn vertical_divider(ratio: RwSignal<f64>, app_state: AppState) -> impl IntoView {
    let (dragging, set_dragging) = create_signal(false);
    let (drag_start_x, set_drag_start_x) = create_signal(0.0);
    let (drag_start_ratio, set_drag_start_ratio) = create_signal(0.5);

    container(label(|| "".to_string()))
        .style(move |s| {
            let colors = app_state.colors();
            s.width(6.0)
                .height_full()
                .background(if dragging.get() {
                    colors.accent_blue
                } else {
                    colors.border
                })
                .cursor(CursorStyle::ColResize)
                .hover({
                    let app_state = app_state.clone();
                    move |s| {
                        let colors = app_state.colors();
                        s.background(colors.border.multiply_alpha(1.5))
                    }
                })
        })
        .on_event_stop(floem::event::EventListener::PointerDown, move |event| {
            if let floem::event::Event::PointerDown(e) = event {
                set_dragging.set(true);
                set_drag_start_x.set(e.pos.x);
                set_drag_start_ratio.set(ratio.get());
            }
        })
        .on_event_stop(floem::event::EventListener::PointerMove, move |event| {
            if let floem::event::Event::PointerMove(e) = event {
                if dragging.get() {
                    let delta = e.pos.x - drag_start_x.get();
                    // Scale factor for ratio adjustment
                    let scale = 0.002;
                    let new_ratio = (drag_start_ratio.get() + delta * scale).clamp(0.1, 0.9);
                    ratio.set(new_ratio);
                }
            }
        })
        .on_event_stop(floem::event::EventListener::PointerUp, move |_event| {
            set_dragging.set(false);
        })
}

/// Horizontal divider (for vertical splits)
fn horizontal_divider(ratio: RwSignal<f64>, app_state: AppState) -> impl IntoView {
    let (dragging, set_dragging) = create_signal(false);
    let (drag_start_y, set_drag_start_y) = create_signal(0.0);
    let (drag_start_ratio, set_drag_start_ratio) = create_signal(0.5);

    container(label(|| "".to_string()))
        .style(move |s| {
            let colors = app_state.colors();
            s.width_full()
                .height(6.0)
                .background(if dragging.get() {
                    colors.accent_blue
                } else {
                    colors.border
                })
                .cursor(CursorStyle::RowResize)
                .hover({
                    let app_state = app_state.clone();
                    move |s| {
                        let colors = app_state.colors();
                        s.background(colors.border.multiply_alpha(1.5))
                    }
                })
        })
        .on_event_stop(floem::event::EventListener::PointerDown, move |event| {
            if let floem::event::Event::PointerDown(e) = event {
                set_dragging.set(true);
                set_drag_start_y.set(e.pos.y);
                set_drag_start_ratio.set(ratio.get());
            }
        })
        .on_event_stop(floem::event::EventListener::PointerMove, move |event| {
            if let floem::event::Event::PointerMove(e) = event {
                if dragging.get() {
                    let delta = e.pos.y - drag_start_y.get();
                    // Scale factor for ratio adjustment
                    let scale = 0.002;
                    let new_ratio = (drag_start_ratio.get() + delta * scale).clamp(0.1, 0.9);
                    ratio.set(new_ratio);
                }
            }
        })
        .on_event_stop(floem::event::EventListener::PointerUp, move |_event| {
            set_dragging.set(false);
        })
}
