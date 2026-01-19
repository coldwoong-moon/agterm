//! Search Bar Component
//!
//! Provides a search UI for finding text in the terminal buffer.

use floem::prelude::*;
use floem::reactive::{RwSignal, SignalGet, SignalUpdate};
use floem::views::{h_stack, text_input, label, container, Decorators};
use floem::keyboard::{Key, NamedKey};

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

/// Create the search bar view (future feature)
#[allow(dead_code)]
pub fn search_bar<F1, F2, F3>(
    state: SearchBarState,
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
            // Search label
            label(|| "Find:".to_string())
                .style(|s| {
                    s.padding(5.0)
                        .color(colors::TEXT_PRIMARY)
                        .font_size(14.0)
                }),

            // Search input
            text_input(query)
                .placeholder("Search...")
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
                .style(|s| {
                    s.width(300.0)
                        .padding(5.0)
                        .border(1.0)
                        .border_color(colors::BORDER)
                        .border_radius(4.0)
                        .background(colors::BG_SECONDARY)
                        .color(colors::TEXT_PRIMARY)
                        .font_size(14.0)
                }),

            // Match counter
            label(move || {
                let total = match_count.get();
                if total == 0 {
                    "No matches".to_string()
                } else if let Some(current) = current_match.get() {
                    format!("{}/{}", current, total)
                } else {
                    format!("{} matches", total)
                }
            })
            .style(|s| {
                s.padding(5.0)
                    .color(colors::TEXT_SECONDARY)
                    .font_size(13.0)
                    .min_width(100.0)
            }),

            // Previous button
            container(
                label(|| "↑".to_string())
                    .style(|s| {
                        s.color(colors::TEXT_PRIMARY)
                            .font_size(16.0)
                    })
            )
            .on_click_stop({
                let on_prev = on_prev.clone();
                move |_| {
                    on_prev();
                }
            })
            .style(|s| {
                s.padding(8.0)
                    .border(1.0)
                    .border_color(colors::BORDER)
                    .border_radius(4.0)
                    .background(colors::BG_SECONDARY)
                    .cursor(floem::style::CursorStyle::Pointer)
                    .hover(|s| s.background(colors::BG_HOVER))
            }),

            // Next button
            container(
                label(|| "↓".to_string())
                    .style(|s| {
                        s.color(colors::TEXT_PRIMARY)
                            .font_size(16.0)
                    })
            )
            .on_click_stop({
                let on_next = on_next.clone();
                move |_| {
                    on_next();
                }
            })
            .style(|s| {
                s.padding(8.0)
                    .border(1.0)
                    .border_color(colors::BORDER)
                    .border_radius(4.0)
                    .background(colors::BG_SECONDARY)
                    .cursor(floem::style::CursorStyle::Pointer)
                    .hover(|s| s.background(colors::BG_HOVER))
            }),

            // Close button
            container(
                label(|| "✕".to_string())
                    .style(|s| {
                        s.color(colors::TEXT_PRIMARY)
                            .font_size(16.0)
                    })
            )
            .on_click_stop({
                let state = state.clone();
                move |_| {
                    state.hide();
                }
            })
            .style(|s| {
                s.padding(8.0)
                    .margin_left(10.0)
                    .border(1.0)
                    .border_color(colors::BORDER)
                    .border_radius(4.0)
                    .background(colors::BG_SECONDARY)
                    .cursor(floem::style::CursorStyle::Pointer)
                    .hover(|s| s.background(colors::BG_HOVER))
            }),
        ))
        .style(|s| {
            s.gap(10.0)
                .items_center()
        })
    )
    .style(move |s| {
        let visible = state.visible.get();
        s.width_full()
            .padding(10.0)
            .background(colors::BG_PRIMARY)
            .border_bottom(1.0)
            .border_color(colors::BORDER)
            .display(if visible {
                floem::style::Display::Flex
            } else {
                floem::style::Display::None
            })
    })
}
