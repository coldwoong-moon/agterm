//! Search Bar Component
//!
//! Provides a search UI for finding text in the terminal buffer.

use floem::prelude::*;
use floem::reactive::{RwSignal, SignalGet, SignalUpdate};
use floem::views::{h_stack, text_input, label, container, Decorators};
use floem::keyboard::{Key, NamedKey};

use crate::floem_app::state::AppState;
use crate::floem_app::theme::colors;

/// Search bar state (future feature)
#[derive(Clone)]
pub struct SearchBarState {
    /// Whether the search bar is visible
    #[allow(dead_code)]
    pub visible: RwSignal<bool>,
    /// Current search query
    #[allow(dead_code)]
    pub query: RwSignal<String>,
    /// Current match index (1-based)
    #[allow(dead_code)]
    pub current_match: RwSignal<Option<usize>>,
    /// Total match count
    #[allow(dead_code)]
    pub match_count: RwSignal<usize>,
}

impl SearchBarState {
    pub fn new() -> Self {
        Self {
            visible: RwSignal::new(false),
            query: RwSignal::new(String::new()),
            current_match: RwSignal::new(None),
            match_count: RwSignal::new(0),
        }
    }

    /// Show the search bar
    #[allow(dead_code)]
    pub fn show(&self) {
        self.visible.set(true);
    }

    /// Hide the search bar
    #[allow(dead_code)]
    pub fn hide(&self) {
        self.visible.set(false);
        self.query.set(String::new());
        self.current_match.set(None);
        self.match_count.set(0);
    }

    /// Update match info
    #[allow(dead_code)]
    pub fn set_match_info(&self, current: Option<usize>, total: usize) {
        self.current_match.set(current);
        self.match_count.set(total);
    }
}

impl Default for SearchBarState {
    fn default() -> Self {
        Self::new()
    }
}

/// Create the search bar view with dynamic theming
#[allow(dead_code)]
pub fn search_bar<F1, F2, F3>(
    state: SearchBarState,
    app_state: AppState,
    on_query_change: F1,
    on_next: F2,
    on_prev: F3,
) -> impl IntoView
where
    F1: Fn(String) + Clone + 'static,
    F2: Fn() + Clone + 'static,
    F3: Fn() + Clone + 'static,
{
    let query = state.query;
    let current_match = state.current_match;
    let match_count = state.match_count;

    let on_next_clone = on_next.clone();
    let on_prev_clone = on_prev.clone();

    // Clone AppState for different style closures
    let app_state_label = app_state.clone();
    let app_state_input = app_state.clone();
    let app_state_counter = app_state.clone();
    let app_state_prev_label = app_state.clone();
    let app_state_prev_btn = app_state.clone();
    let app_state_next_label = app_state.clone();
    let app_state_next_btn = app_state.clone();
    let app_state_close_label = app_state.clone();
    let app_state_close_btn = app_state.clone();
    let app_state_container = app_state.clone();

    // Set up reactive query change monitoring
    {
        let on_query_change = on_query_change.clone();
        floem::reactive::create_effect(move |_| {
            let q = query.get();
            on_query_change(q);
        });
    }

    container(
        h_stack((
            // Search icon and label
            label(|| "🔍 Find:".to_string())
                .style(move |s| {
                    let colors = app_state_label.colors();
                    s.padding(5.0)
                        .color(colors.text_primary)
                        .font_size(14.0)
                }),

            // Search input
            text_input(query)
                .placeholder("Search... (Enter: next, Shift+Enter: prev)")
                .on_event(floem::event::EventListener::KeyDown, move |event| {
                    if let floem::event::Event::KeyDown(key_event) = event {
                        match &key_event.key.logical_key {
                            Key::Named(NamedKey::Enter) => {
                                // Check if Shift is pressed
                                if key_event.modifiers.shift() {
                                    on_prev_clone();
                                } else {
                                    on_next_clone();
                                }
                                return floem::event::EventPropagation::Stop;
                            }
                            Key::Named(NamedKey::Escape) => {
                                // Escape handled by parent
                                return floem::event::EventPropagation::Continue;
                            }
                            _ => {}
                        }
                    }
                    floem::event::EventPropagation::Continue
                })
                .style(move |s| {
                    let colors = app_state_input.colors();
                    s.width(300.0)
                        .padding(8.0)
                        .border(1.0)
                        .border_color(colors.border)
                        .border_radius(6.0)
                        .background(colors.bg_secondary)
                        .color(colors.text_primary)
                        .font_size(14.0)
                        .focus(move |s| {
                            s.border_color(colors::ACCENT_BLUE)
                                .border(2.0)
                        })
                }),

            // Match counter with highlighting
            label(move || {
                let total = match_count.get();
                if total == 0 {
                    "No matches".to_string()
                } else if let Some(current) = current_match.get() {
                    format!("{current} of {total}")
                } else {
                    format!("{total} matches")
                }
            })
            .style(move |s| {
                let colors = app_state_counter.colors();
                let total = match_count.get();
                let text_color = if total == 0 {
                    colors.text_muted
                } else {
                    colors.accent_blue
                };
                s.padding(5.0)
                    .color(text_color)
                    .font_size(13.0)
                    .min_width(100.0)
            }),

            // Previous button
            container(
                label(|| "▲".to_string())
                    .style(move |s| {
                        let colors = app_state_prev_label.colors();
                        s.color(colors.text_primary)
                            .font_size(12.0)
                    })
            )
            .on_click_stop({
                let on_prev = on_prev.clone();
                move |_| {
                    on_prev();
                }
            })
            .style(move |s| {
                let colors = app_state_prev_btn.colors();
                s.padding_horiz(12.0)
                    .padding_vert(6.0)
                    .border(1.0)
                    .border_color(colors.border)
                    .border_radius(4.0)
                    .background(colors.bg_secondary)
                    .cursor(floem::style::CursorStyle::Pointer)
                    .hover(|s| s.background(colors::BG_HOVER))
            }),

            // Next button
            container(
                label(|| "▼".to_string())
                    .style(move |s| {
                        let colors = app_state_next_label.colors();
                        s.color(colors.text_primary)
                            .font_size(12.0)
                    })
            )
            .on_click_stop({
                let on_next = on_next.clone();
                move |_| {
                    on_next();
                }
            })
            .style(move |s| {
                let colors = app_state_next_btn.colors();
                s.padding_horiz(12.0)
                    .padding_vert(6.0)
                    .border(1.0)
                    .border_color(colors.border)
                    .border_radius(4.0)
                    .background(colors.bg_secondary)
                    .cursor(floem::style::CursorStyle::Pointer)
                    .hover(|s| s.background(colors::BG_HOVER))
            }),

            // Close button
            container(
                label(|| "✕".to_string())
                    .style(move |s| {
                        let colors = app_state_close_label.colors();
                        s.color(colors.text_secondary)
                            .font_size(14.0)
                    })
            )
            .on_click_stop({
                let state = state.clone();
                move |_| {
                    state.hide();
                }
            })
            .style(move |s| {
                let colors = app_state_close_btn.colors();
                s.padding(6.0)
                    .margin_left(8.0)
                    .border_radius(4.0)
                    .cursor(floem::style::CursorStyle::Pointer)
                    .hover(|s| s.background(colors::BG_HOVER).color(colors.text_primary))
            }),
        ))
        .style(|s| {
            s.gap(8.0)
                .items_center()
        })
    )
    .style(move |s| {
        let visible = state.visible.get();
        let colors = app_state_container.colors();
        s.width_full()
            .padding(10.0)
            .background(colors.bg_primary)
            .border_bottom(1.0)
            .border_color(colors.border)
            .display(if visible {
                floem::style::Display::Flex
            } else {
                floem::style::Display::None
            })
    })
}
