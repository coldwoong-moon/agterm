//! History View Component
//!
//! Displays command execution history with search functionality.
//! Provides an interactive UI for viewing and replaying past commands.

use floem::prelude::*;
use floem::reactive::{RwSignal, SignalGet, SignalUpdate};
use floem::style::CursorStyle;
use floem::views::{container, h_stack, label, scroll, text_input, v_stack, Decorators, dyn_container, empty};
use chrono::{Local, TimeZone};

use crate::floem_app::theme::Theme;
use crate::history::HistoryEntry;

/// Group commands by date
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryGroup {
    pub date_label: String,
    pub entries: Vec<HistoryEntryDisplay>,
}

/// Extended history entry with display metadata
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryEntryDisplay {
    pub entry: HistoryEntry,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<u64>,
    pub expanded: bool,
}

impl HistoryEntryDisplay {
    pub fn new(entry: HistoryEntry) -> Self {
        Self {
            entry,
            exit_code: Some(0), // Default to success
            duration_ms: None,
            expanded: false,
        }
    }

    pub fn with_exit_code(mut self, code: i32) -> Self {
        self.exit_code = Some(code);
        self
    }

    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }
}

/// History view state
#[derive(Clone)]
pub struct HistoryViewState {
    /// All history groups (organized by date)
    pub groups: RwSignal<Vec<HistoryGroup>>,
    /// Search query
    pub search_query: RwSignal<String>,
    /// Whether groups are expanded or collapsed
    pub expanded_groups: RwSignal<Vec<bool>>,
}

impl HistoryViewState {
    pub fn new() -> Self {
        Self {
            groups: RwSignal::new(Vec::new()),
            search_query: RwSignal::new(String::new()),
            expanded_groups: RwSignal::new(Vec::new()),
        }
    }

    /// Load history entries and group by date
    pub fn load_entries(&self, entries: Vec<HistoryEntry>) {
        let mut groups = Vec::new();
        let mut current_date = String::new();
        let mut current_entries = Vec::new();

        for entry in entries.into_iter().rev() {
            let date_label = if let Some(timestamp) = entry.timestamp {
                let dt = Local.timestamp_opt(timestamp, 0).unwrap();
                let today = Local::now().date_naive();
                let entry_date = dt.date_naive();

                if entry_date == today {
                    "Today".to_string()
                } else if entry_date == today - chrono::Duration::days(1) {
                    "Yesterday".to_string()
                } else {
                    entry_date.format("%Y-%m-%d").to_string()
                }
            } else {
                "Unknown".to_string()
            };

            if date_label != current_date {
                if !current_entries.is_empty() {
                    groups.push(HistoryGroup {
                        date_label: current_date.clone(),
                        entries: current_entries.clone(),
                    });
                    current_entries.clear();
                }
                current_date = date_label;
            }

            current_entries.push(HistoryEntryDisplay::new(entry));
        }

        if !current_entries.is_empty() {
            groups.push(HistoryGroup {
                date_label: current_date,
                entries: current_entries,
            });
        }

        let expanded = vec![true; groups.len()];
        self.expanded_groups.set(expanded);
        self.groups.set(groups);
    }

    /// Toggle expansion state for a group
    pub fn toggle_group(&self, index: usize) {
        self.expanded_groups.update(|expanded| {
            if index < expanded.len() {
                expanded[index] = !expanded[index];
            }
        });
    }

    /// Filter groups by search query
    pub fn filtered_groups(&self) -> Vec<HistoryGroup> {
        let query = self.search_query.get().to_lowercase();
        if query.is_empty() {
            return self.groups.get();
        }

        self.groups.get()
            .into_iter()
            .filter_map(|group| {
                let filtered_entries: Vec<_> = group.entries
                    .into_iter()
                    .filter(|e| e.entry.command.to_lowercase().contains(&query))
                    .collect();

                if filtered_entries.is_empty() {
                    None
                } else {
                    Some(HistoryGroup {
                        date_label: group.date_label,
                        entries: filtered_entries,
                    })
                }
            })
            .collect()
    }
}

impl Default for HistoryViewState {
    fn default() -> Self {
        Self::new()
    }
}

/// Format duration for display
fn format_duration(duration_ms: u64) -> String {
    if duration_ms < 1000 {
        format!("{}ms", duration_ms)
    } else {
        format!("{:.1}s", duration_ms as f64 / 1000.0)
    }
}

/// Format timestamp for display
fn format_timestamp(timestamp: i64) -> String {
    if let Some(dt) = Local.timestamp_opt(timestamp, 0).single() {
        dt.format("%H:%M").to_string()
    } else {
        "??:??".to_string()
    }
}

/// Create a history entry row
fn history_entry_row<F>(
    entry: &HistoryEntryDisplay,
    theme: Theme,
    on_replay: F,
) -> impl IntoView
where
    F: Fn(String) + 'static + Clone,
{
    let colors = theme.colors();
    let command = entry.entry.command.clone();
    let exit_code = entry.exit_code.unwrap_or(0);
    let duration = entry.duration_ms;
    let timestamp = entry.entry.timestamp;

    let command_for_click = command.clone();
    let on_replay_clone = on_replay.clone();

    container(
        h_stack((
            // Timestamp
            label(move || {
                timestamp.map(format_timestamp).unwrap_or_else(|| "--:--".to_string())
            })
            .style(move |s| {
                s.font_size(11.0)
                    .color(colors.text_muted)
                    .width(50.0)
            }),

            // Separator
            label(|| " │ ")
                .style(move |s| s.font_size(11.0).color(colors.border)),

            // Command
            label(move || {
                let cmd = command.clone();
                if cmd.len() > 50 {
                    format!("{}...", &cmd[..50])
                } else {
                    cmd
                }
            })
            .style(move |s| {
                s.font_size(12.0)
                    .color(colors.text_primary)
                    .flex_grow(1.0)
                    .min_width(200.0)
            }),

            // Exit code
            label(move || {
                if exit_code == 0 {
                    "✅ 0".to_string()
                } else {
                    format!("❌ {}", exit_code)
                }
            })
            .style(move |s| {
                let color = if exit_code == 0 {
                    colors.accent_green
                } else {
                    colors.accent_red
                };
                s.font_size(11.0).color(color).width(60.0)
            }),

            // Duration
            label(move || {
                duration.map(format_duration).unwrap_or_else(|| "--".to_string())
            })
            .style(move |s| {
                s.font_size(11.0)
                    .color(colors.text_muted)
                    .width(60.0)
            }),

            // Replay button
            container(label(|| "▶"))
                .on_click_stop(move |_| {
                    let cmd = command_for_click.clone();
                    on_replay_clone(cmd);
                })
                .style(move |s| {
                    s.font_size(14.0)
                        .color(colors.accent_blue)
                        .padding(4.0)
                        .border_radius(4.0)
                        .cursor(CursorStyle::Pointer)
                        .hover(|s| s.background(colors.bg_tab_hover))
                }),
        ))
        .style(move |s| {
            s.padding(8.0)
                .width_full()
                .items_center()
                .gap(8.0)
        }),
    )
    .style(move |s| {
        s.width_full()
            .border_bottom(1.0)
            .border_color(colors.border_subtle)
            .hover(|s| s.background(colors.bg_tab_hover))
    })
}

/// Create history group section (date header + entries)
fn history_group_view<F>(
    group: &HistoryGroup,
    index: usize,
    is_expanded: bool,
    theme: Theme,
    on_toggle: impl Fn(usize) + 'static + Clone,
    on_replay: F,
) -> impl IntoView
where
    F: Fn(String) + 'static + Clone,
{
    let colors = theme.colors();
    let entry_count = group.entries.len();
    let date_label = group.date_label.clone();
    let entries = group.entries.clone();

    let on_toggle_clone = on_toggle.clone();

    v_stack((
        // Date header
        container(
            h_stack((
                label(move || if is_expanded { "▼" } else { "▶" })
                    .style(move |s| {
                        s.font_size(12.0)
                            .color(colors.text_muted)
                            .width(20.0)
                    }),
                label(move || date_label.clone())
                    .style(move |s| {
                        s.font_size(14.0)
                            .color(colors.text_primary)
                            .font_weight(floem::text::Weight::BOLD)
                    }),
                label(move || format!("({} commands)", entry_count))
                    .style(move |s| {
                        s.font_size(11.0)
                            .color(colors.text_muted)
                            .padding_left(8.0)
                    }),
            ))
            .style(|s| s.items_center().gap(4.0))
        )
        .on_click_stop(move |_| {
            on_toggle_clone(index);
        })
        .style(move |s| {
            s.padding(12.0)
                .width_full()
                .cursor(CursorStyle::Pointer)
                .background(colors.bg_secondary)
                .border_bottom(1.0)
                .border_color(colors.border)
                .hover(|s| s.background(colors.bg_tab_hover))
        }),

        // Entries (only shown when expanded)
        dyn_container(
            move || is_expanded,
            move |expanded| {
                if expanded {
                    container(
                        empty()
                    )
                    .style(|s| s.width_full())
                    .into_any()
                } else {
                    container(empty()).style(|s| s.display(floem::style::Display::None)).into_any()
                }
            }
        ),
    ))
    .style(|s| s.width_full())
}

/// Main history view
pub fn history_view<F>(
    state: HistoryViewState,
    theme: Theme,
    on_replay: F,
) -> impl IntoView
where
    F: Fn(String) + 'static + Clone,
{
    let colors = theme.colors();

    let state_search = state.clone();
    let state_groups = state.clone();
    let state_expanded = state.clone();

    container(
        v_stack((
            // Header with search bar
            container(
                h_stack((
                    label(|| "📜 History")
                        .style(move |s| {
                            s.font_size(16.0)
                                .color(colors.text_primary)
                                .font_weight(floem::text::Weight::BOLD)
                        }),

                    // Spacer
                    container(label(|| "")).style(|s| s.flex_grow(1.0)),

                    // Search input
                    text_input(state_search.search_query)
                        .placeholder("🔍 Search...")
                        .style(move |s| {
                            s.width(200.0)
                                .padding(6.0)
                                .border_radius(4.0)
                                .border(1.0)
                                .border_color(colors.border)
                                .background(colors.bg_primary)
                                .color(colors.text_primary)
                                .font_size(12.0)
                                .focus(|s| s.border_color(colors.accent_blue))
                        }),
                ))
                .style(|s| s.width_full().items_center().gap(12.0))
            )
            .style(move |s| {
                s.padding(16.0)
                    .width_full()
                    .border_bottom(2.0)
                    .border_color(colors.border)
            }),

            // Scrollable history groups
            scroll(
                container(
                    empty()
                )
                .style(|s| s.width_full())
            )
            .style(|s| s.flex_grow(1.0).width_full()),
        ))
        .style(|s| s.width_full().height_full())
    )
    .style(move |s| {
        s.width_full()
            .height_full()
            .background(colors.bg_primary)
    })
}

/// Create a standalone history panel with default styling
pub fn history_panel<F>(
    state: HistoryViewState,
    theme: Theme,
    on_replay: F,
) -> impl IntoView
where
    F: Fn(String) + 'static + Clone,
{
    container(history_view(state, theme, on_replay))
        .style(move |s| {
            let colors = theme.colors();
            s.width(400.0)
                .height_full()
                .border_left(1.0)
                .border_color(colors.border)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_view_state_new() {
        let state = HistoryViewState::new();
        assert_eq!(state.groups.get().len(), 0);
        assert_eq!(state.search_query.get(), "");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(50), "50ms");
        assert_eq!(format_duration(999), "999ms");
        assert_eq!(format_duration(1000), "1.0s");
        assert_eq!(format_duration(1500), "1.5s");
        assert_eq!(format_duration(12345), "12.3s");
    }

    #[test]
    fn test_history_entry_display() {
        let entry = HistoryEntry::new("ls -la".to_string(), Some("/home".to_string()));
        let display = HistoryEntryDisplay::new(entry)
            .with_exit_code(0)
            .with_duration(123);

        assert_eq!(display.exit_code, Some(0));
        assert_eq!(display.duration_ms, Some(123));
    }
}
