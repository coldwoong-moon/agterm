//! Settings View
//!
//! A modal settings panel that provides UI for adjusting application settings.
//! Activated via Cmd+, keyboard shortcut.

use floem::prelude::*;
use floem::views::{v_stack, h_stack, label, Decorators, container, scroll};
use floem::keyboard::{Key, NamedKey};
use floem::text::Weight;

use crate::floem_app::state::AppState;
use crate::floem_app::theme::Theme;

/// Settings panel view
///
/// Displays a modal overlay with:
/// - Font size slider
/// - Theme dropdown/toggle
/// - Save button (auto-saves on change)
pub fn settings_panel(app_state: &AppState, is_visible: RwSignal<bool>) -> impl IntoView {
    let app_state_dec_style = app_state.clone();
    let app_state_dec_click = app_state.clone();
    let app_state_inc_style = app_state.clone();
    let app_state_inc_click = app_state.clone();
    let app_state_reset_style = app_state.clone();
    let app_state_reset_click = app_state.clone();
    let app_state_dark_label = app_state.clone();
    let app_state_dark_style = app_state.clone();
    let app_state_dark_click = app_state.clone();
    let app_state_light_label = app_state.clone();
    let app_state_light_style = app_state.clone();
    let app_state_light_click = app_state.clone();
    let app_state_info = app_state.clone();
    let app_state_modal = app_state.clone();
    let app_state_close = app_state.clone();

    // Get current settings
    let font_size = app_state.font_size;
    let theme = app_state.theme;

    container(
        v_stack((
            // Header
            h_stack((
                label(|| "Settings".to_string())
                    .style(|s| {
                        s.font_size(20.0)
                            .font_weight(Weight::BOLD)
                            .margin_bottom(4.0)
                    }),
            ))
            .style(|s| {
                s.width_full()
                    .padding(16.0)
                    .border_bottom(1.0)
            }),

            // Content area
            scroll(
                v_stack((
                    // Font size section
                    v_stack((
                        label(|| "Font Size".to_string())
                            .style(|s| {
                                s.font_size(14.0)
                                    .font_weight(Weight::SEMIBOLD)
                                    .margin_bottom(8.0)
                            }),

                        // Font size display and controls
                        h_stack((
                            label(move || format!("{:.1} pt", font_size.get()))
                                .style(|s| {
                                    s.font_size(12.0)
                                        .padding(4.0)
                                        .margin_right(12.0)
                                        .min_width(60.0)
                                }),

                            // Decrease button
                            container(
                                label(|| "-".to_string())
                                    .style(|s| {
                                        s.font_size(16.0)
                                            .font_weight(Weight::BOLD)
                                    })
                            )
                            .style(move |s| {
                                let colors = app_state_dec_style.colors();
                                s.padding(8.0)
                                    .margin_right(8.0)
                                    .border_radius(4.0)
                                    .background(colors.bg_secondary)
                                    .border(1.0)
                                    .border_color(colors.border)
                                    .hover(|s| {
                                        s.background(colors.bg_tab_hover)
                                            .border_color(colors.border.multiply_alpha(1.5))
                                    })
                            })
                            .on_click_stop({
                                let app_state = app_state_dec_click.clone();
                                move |_| {
                                    app_state.decrease_font_size();
                                }
                            }),

                            // Increase button
                            container(
                                label(|| "+".to_string())
                                    .style(|s| {
                                        s.font_size(16.0)
                                            .font_weight(Weight::BOLD)
                                    })
                            )
                            .style(move |s| {
                                let colors = app_state_inc_style.colors();
                                s.padding(8.0)
                                    .margin_right(8.0)
                                    .border_radius(4.0)
                                    .background(colors.bg_secondary)
                                    .border(1.0)
                                    .border_color(colors.border)
                                    .hover(|s| {
                                        s.background(colors.bg_tab_hover)
                                            .border_color(colors.border.multiply_alpha(1.5))
                                    })
                            })
                            .on_click_stop({
                                let app_state = app_state_inc_click.clone();
                                move |_| {
                                    app_state.increase_font_size();
                                }
                            }),

                            // Reset button
                            container(
                                label(|| "Reset".to_string())
                                    .style(|s| {
                                        s.font_size(12.0)
                                    })
                            )
                            .style(move |s| {
                                let colors = app_state_reset_style.colors();
                                s.padding(6.0)
                                    .padding_horiz(12.0)
                                    .border_radius(4.0)
                                    .background(colors.bg_secondary)
                                    .border(1.0)
                                    .border_color(colors.border)
                                    .hover(|s| {
                                        s.background(colors.bg_tab_hover)
                                            .border_color(colors.border.multiply_alpha(1.5))
                                    })
                            })
                            .on_click_stop({
                                let app_state = app_state_reset_click.clone();
                                move |_| {
                                    app_state.reset_font_size();
                                }
                            }),
                        ))
                        .style(|s| {
                            s.width_full()
                                .items_center()
                        }),
                    ))
                    .style(|s| {
                        s.width_full()
                            .padding(16.0)
                            .border_bottom(1.0)
                    }),

                    // Theme section
                    v_stack((
                        label(|| "Theme".to_string())
                            .style(|s| {
                                s.font_size(14.0)
                                    .font_weight(Weight::SEMIBOLD)
                                    .margin_bottom(8.0)
                            }),

                        // Theme toggle
                        h_stack((
                            // Dark theme button
                            container(
                                label(|| "Dark".to_string())
                                    .style(move |s| {
                                        let is_dark = theme.get() == Theme::GhosttyDark;
                                        let colors = app_state_dark_label.colors();
                                        s.font_size(12.0)
                                            .color(if is_dark { colors.text_primary } else { colors.text_secondary })
                                    })
                            )
                            .style(move |s| {
                                let is_dark = theme.get() == Theme::GhosttyDark;
                                let colors = app_state_dark_style.colors();
                                s.padding(8.0)
                                    .padding_horiz(16.0)
                                    .margin_right(8.0)
                                    .border_radius(4.0)
                                    .background(if is_dark { colors.accent_blue.multiply_alpha(0.2) } else { colors.bg_secondary })
                                    .border(1.0)
                                    .border_color(if is_dark { colors.accent_blue } else { colors.border })
                                    .hover(|s| {
                                        if !is_dark {
                                            s.background(colors.bg_tab_hover)
                                                .border_color(colors.border.multiply_alpha(1.5))
                                        } else {
                                            s
                                        }
                                    })
                            })
                            .on_click_stop({
                                let app_state = app_state_dark_click.clone();
                                move |_| {
                                    if theme.get() != Theme::GhosttyDark {
                                        app_state.theme.set(Theme::GhosttyDark);
                                        app_state.save_settings();
                                    }
                                }
                            }),

                            // Light theme button
                            container(
                                label(|| "Light".to_string())
                                    .style(move |s| {
                                        let is_light = theme.get() == Theme::GhosttyLight;
                                        let colors = app_state_light_label.colors();
                                        s.font_size(12.0)
                                            .color(if is_light { colors.text_primary } else { colors.text_secondary })
                                    })
                            )
                            .style(move |s| {
                                let is_light = theme.get() == Theme::GhosttyLight;
                                let colors = app_state_light_style.colors();
                                s.padding(8.0)
                                    .padding_horiz(16.0)
                                    .border_radius(4.0)
                                    .background(if is_light { colors.accent_blue.multiply_alpha(0.2) } else { colors.bg_secondary })
                                    .border(1.0)
                                    .border_color(if is_light { colors.accent_blue } else { colors.border })
                                    .hover(|s| {
                                        if !is_light {
                                            s.background(colors.bg_tab_hover)
                                                .border_color(colors.border.multiply_alpha(1.5))
                                        } else {
                                            s
                                        }
                                    })
                            })
                            .on_click_stop({
                                let app_state = app_state_light_click.clone();
                                move |_| {
                                    if theme.get() != Theme::GhosttyLight {
                                        app_state.theme.set(Theme::GhosttyLight);
                                        app_state.save_settings();
                                    }
                                }
                            }),
                        ))
                        .style(|s| {
                            s.width_full()
                                .items_center()
                        }),
                    ))
                    .style(|s| {
                        s.width_full()
                            .padding(16.0)
                            .border_bottom(1.0)
                    }),

                    // Info section
                    v_stack((
                        label(|| "Settings are automatically saved".to_string())
                            .style(move |s| {
                                let colors = app_state_info.colors();
                                s.font_size(11.0)
                                    .color(colors.text_muted)
                            }),
                    ))
                    .style(|s| {
                        s.width_full()
                            .padding(16.0)
                    }),
                ))
                .style(|s| {
                    s.width_full()
                        .flex_col()
                })
            )
            .style(|s| {
                s.width_full()
                    .flex_grow(1.0)
            }),

            // Footer with close button
            h_stack((
                container(
                    label(|| "Close".to_string())
                        .style(|s| {
                            s.font_size(12.0)
                        })
                )
                .style(move |s| {
                    let colors = app_state_close.colors();
                    s.padding(8.0)
                        .padding_horiz(20.0)
                        .border_radius(4.0)
                        .background(colors.accent_blue)
                        .hover(|s| {
                            s.background(colors.accent_blue.multiply_alpha(1.2))
                        })
                })
                .on_click_stop(move |_| {
                    is_visible.set(false);
                }),
            ))
            .style(|s| {
                s.width_full()
                    .padding(16.0)
                    .justify_end()
                    .border_top(1.0)
            }),
        ))
        .style(move |s| {
            let colors = app_state_modal.colors();
            s.width(500.0)
                .max_height_pct(80.0)
                .background(colors.bg_primary)
                .border_radius(8.0)
                .border(1.0)
                .border_color(colors.border)
                .box_shadow_blur(20.0)
        })
    )
    .style(|s| {
        s.width_full()
            .height_full()
            .items_center()
            .justify_center()
            .background(floem::peniko::Color::rgba8(0, 0, 0, 128))
    })
    .on_event_stop(floem::event::EventListener::KeyDown, move |event| {
        if let floem::event::Event::KeyDown(key_event) = event {
            match &key_event.key.logical_key {
                Key::Named(NamedKey::Escape) => {
                    is_visible.set(false);
                }
                Key::Character(ch) if ch.as_str() == "," && key_event.modifiers.meta() => {
                    is_visible.set(false);
                }
                _ => {}
            }
        }
    })
    .on_click_stop(move |_| {
        // Close when clicking outside the modal
        is_visible.set(false);
    })
}
