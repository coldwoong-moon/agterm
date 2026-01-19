//! Tab Bar Component

use floem::prelude::*;
use floem::views::{container, dyn_stack, h_stack, label, v_stack, Decorators};

use crate::floem_app::state::AppState;
use crate::floem_app::theme::layout;

/// Tab bar view
pub fn tab_bar(state: &AppState) -> impl IntoView {
    let state = state.clone();
    let state_style = state.clone();

    container(
        h_stack((
            // Tab buttons using dyn_stack
            tab_list(&state),

            // Spacer
            container(label(|| "")).style(|s| s.flex_grow(1.0)),

            // New tab button
            new_tab_button(&state),
        ))
        .style(|s| s.gap(layout::TAB_GAP).padding_horiz(8.0)),
    )
    .style(move |s| {
        let colors = state_style.colors();
        s.width_full()
            .height(layout::TAB_BAR_HEIGHT)
            .background(colors.bg_tab_bar)
            .border_color(colors.border_subtle)
            .border_bottom(1.0)
            .items_center()
    })
}

/// List of tab buttons using dyn_stack for dynamic rendering
fn tab_list(state: &AppState) -> impl IntoView {
    let state = state.clone();

    dyn_stack(
        move || state.tabs.get(),
        |tab| tab.id,
        move |tab| {
            let state_clone = state.clone();
            let state_close_clone = state.clone();
            let state_button = state.clone();
            let state_bell_check = state.clone();
            let tab_id = tab.id;
            let title = tab.title.get();
            let is_active = tab.is_active.get();
            
            // Check if this tab has bell notifications
            let tabs = state_bell_check.tabs.get();
            let tab_index = tabs.iter().position(|t| t.id == tab_id).unwrap_or(0);
            let has_bell = state_bell_check.tab_has_bell(tab_index);

            tab_button(
                title,
                is_active,
                has_bell,
                state_button,
                move || {
                    // Find index by id and select tab
                    let tabs = state_clone.tabs.get();
                    if let Some(idx) = tabs.iter().position(|t| t.id == tab_id) {
                        state_clone.select_tab(idx);
                    }
                },
                move || {
                    // Find index by id and close tab
                    let tabs = state_close_clone.tabs.get();
                    if let Some(idx) = tabs.iter().position(|t| t.id == tab_id) {
                        state_close_clone.close_tab(idx);
                    }
                },
            )
        },
    )
    .style(|s| s.gap(layout::TAB_GAP))
}

/// Single tab button with bottom accent line for active state and bell indicator
fn tab_button(
    title: String,
    is_active: bool,
    has_bell: bool,
    state: AppState,
    on_click: impl Fn() + 'static,
    on_close: impl Fn() + 'static,
) -> impl IntoView {
    let state_content = state.clone();
    let state_close = state.clone();
    let _state_hover = state.clone();
    let state_accent = state.clone();

    let colors = state.colors();
    let bg_color = if is_active {
        colors.bg_tab_active
    } else {
        colors.bg_tab_bar
    };

    let text_color = if is_active {
        colors.text_primary
    } else {
        colors.text_secondary
    };

    // Container with bottom accent line for active tab
    v_stack((
        // Tab content (bell indicator + title + close button)
        container(
            h_stack((
                // Bell indicator (only shown for inactive tabs with bells)
                if has_bell && !is_active {
                    label(|| "🔔")
                        .style(move |s| {
                            s.font_size(10.0)
                                .margin_right(4.0)
                        })
                        .into_any()
                } else {
                    container(label(|| ""))
                        .style(|s| s.width(0.0))
                        .into_any()
                },

                // Tab title
                label(move || title.clone())
                    .style(move |s| s.font_size(12.0).color(text_color)),

                // Close button (x) with red hover effect
                container(
                    label(|| "×")
                        .style(move |s| {
                            s.font_size(16.0)
                                .color(text_color)
                        }),
                )
                .style(move |s| {
                    let _colors = state_close.colors();
                    s.width(16.0)
                        .height(16.0)
                        .justify_center()
                        .items_center()
                        .margin_left(8.0)
                        .border_radius(2.0)
                        .cursor(floem::style::CursorStyle::Pointer)
                        // Red background on hover
                        .hover({
                            let state_close = state_close.clone();
                            move |s| {
                                let colors = state_close.colors();
                                s.background(colors.accent_red)
                                    .color(colors.text_primary)
                            }
                        })
                })
                .on_click_stop(move |_| on_close()),
            ))
            .style(|s| s.items_center().gap(4.0)),
        )
        .style(move |s| {
            let colors = state_content.colors();
            let hover_color = colors.bg_tab_hover;
            s.padding_horiz(layout::TAB_PADDING)
                .padding_top(6.0)
                .padding_bottom(4.0) // Slightly less padding for accent line
                .background(bg_color)
                .border_radius(4.0)
                .cursor(floem::style::CursorStyle::Pointer)
                .hover(move |s| {
                    if !is_active {
                        s.background(hover_color)
                    } else {
                        s
                    }
                })
        })
        .on_click_stop(move |_| on_click()),

        // Bottom accent line (only visible when active)
        container(label(|| ""))
            .style(move |s| {
                let colors = state_accent.colors();
                if is_active {
                    s.width_full()
                        .height(2.0)
                        .background(colors.accent_blue)
                } else {
                    s.width_full().height(2.0)
                }
            }),
    ))
    .style(|s| s.flex_col())
}

/// New tab button (+)
fn new_tab_button(state: &AppState) -> impl IntoView {
    let state = state.clone();
    let state_style = state.clone();
    let state_hover = state.clone();

    let colors = state.colors();

    container(
        label(|| "+")
            .style(move |s| {
                s.font_size(16.0)
                    .color(colors.text_secondary)
            }),
    )
    .style(move |s| {
        let colors = state_style.colors();
        let hover_color = colors.bg_tab_hover;
        s.width(28.0)
            .height(28.0)
            .justify_center()
            .items_center()
            .border_radius(4.0)
            .cursor(floem::style::CursorStyle::Pointer)
            .hover(move |s| s.background(hover_color))
    })
    .on_event_stop(floem::event::EventListener::PointerDown, move |_| {
        tracing::info!("New tab button PointerDown!");
        state_hover.add_tab();
    })
}
