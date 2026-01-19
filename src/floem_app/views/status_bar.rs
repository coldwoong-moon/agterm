//! Status Bar Component

use floem::prelude::*;
use floem::reactive::SignalGet;
use floem::views::{container, h_stack, label, Decorators};
use std::env;

use crate::floem_app::state::AppState;
use crate::floem_app::theme::layout;

/// Get the current working directory from environment
fn get_current_directory() -> String {
    env::current_dir()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "~".to_string())
}

/// Get git branch for the current directory (if in a git repo)
fn get_git_branch() -> Option<String> {
    use std::process::Command;

    Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}

/// Check if current directory is a git repository
fn is_git_repo() -> bool {
    use std::process::Command;

    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Status bar view with enhanced information display
pub fn status_bar(state: &AppState) -> impl IntoView {
    let state_dir = state.clone();
    let state_git = state.clone();
    let state_sep1 = state.clone();
    let state_sep2 = state.clone();
    let state_sep3 = state.clone();
    let state_sep4 = state.clone();
    let state_sep5 = state.clone();
    let state_sep6 = state.clone();
    let state_tabs = state.clone();
    let state_panes = state.clone();
    let state_panes_style = state.clone();
    let state_terminal_size = state.clone();
    let state_terminal_size_style = state.clone();
    let state_font = state.clone();
    let state_theme = state.clone();
    let state_style = state.clone();

    container(
        h_stack((
            // Current working directory
            label(move || {
                let dir = get_current_directory();
                // Abbreviate home directory
                let home = env::var("HOME").unwrap_or_default();
                if !home.is_empty() && dir.starts_with(&home) {
                    dir.replace(&home, "~")
                } else {
                    dir
                }
            })
            .style(move |s| {
                let colors = state_dir.colors();
                s.font_size(11.0).color(colors.text_secondary)
            }),

            // Git branch (if in a git repo)
            label(move || {
                if is_git_repo() {
                    if let Some(branch) = get_git_branch() {
                        format!(" [{}]", branch)
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            })
            .style(move |s| {
                let colors = state_git.colors();
                s.font_size(11.0).color(colors.accent_green)
            }),

            // Separator
            label(|| " | ")
                .style(move |s| {
                    let colors = state_sep1.colors();
                    s.font_size(11.0).color(colors.text_muted).padding_horiz(4.0)
                }),

            // Tab info (current / total)
            label(move || {
                let tabs = state_tabs.tabs.get();
                let active = state_tabs.active_tab.get();
                format!("Tab {}/{}", active + 1, tabs.len())
            })
            .style(move |s| {
                let colors = state_tabs.colors();
                s.font_size(11.0).color(colors.text_secondary)
            }),

            // Separator
            label(|| " | ")
                .style(move |s| {
                    let colors = state_sep2.colors();
                    s.font_size(11.0).color(colors.text_muted).padding_horiz(4.0)
                }),

            // Pane info (active / total in current tab)
            label(move || {
                if let Some(tab) = state_panes.active_tab_ref() {
                    let pane_count = tab.pane_tree.get().count_leaves();
                    let focused = tab.pane_tree.get()
                        .get_all_leaf_ids()
                        .iter()
                        .position(|id| {
                            if let Some((focused_id, _)) = tab.pane_tree.get().get_focused_leaf() {
                                *id == focused_id
                            } else {
                                false
                            }
                        })
                        .map(|i| i + 1)
                        .unwrap_or(1);

                    format!("Pane {}/{}", focused, pane_count)
                } else {
                    "No panes".to_string()
                }
            })
            .style(move |s| {
                let colors = state_panes_style.colors();
                s.font_size(11.0).color(colors.text_secondary)
            }),

            // Separator
            label(|| " | ")
                .style(move |s| {
                    let colors = state_sep3.colors();
                    s.font_size(11.0).color(colors.text_muted).padding_horiz(4.0)
                }),

            // Keyboard shortcuts hint
            label(|| "⌘D:Split ⌘T:NewTab ⌘W:Close ⌘+/-:Font ⇧⌘T:Theme")
                .style(move |s| {
                    let colors = state_sep4.colors();
                    s.font_size(10.0).color(colors.text_muted)
                }),

            // Spacer (pushes right-side items to the right)
            container(label(|| "")).style(|s| s.flex_grow(1.0)),

            // Terminal size (from active pane)
            label(move || {
                if let Some(tab) = state_terminal_size.active_tab_ref() {
                    if let Some((_, terminal_state)) = tab.pane_tree.get().get_focused_leaf() {
                        let (cols, rows) = terminal_state.dimensions();
                        format!("{}x{}", cols, rows)
                    } else {
                        "80x24".to_string()
                    }
                } else {
                    "80x24".to_string()
                }
            })
            .style(move |s| {
                let colors = state_terminal_size_style.colors();
                s.font_size(11.0).color(colors.text_muted)
            }),

            // Separator
            label(|| " | ")
                .style(move |s| {
                    let colors = state_sep6.colors();
                    s.font_size(11.0).color(colors.text_muted).padding_horiz(4.0)
                }),

            // Font size
            label(move || format!("{:.0}pt", state_font.font_size.get()))
                .style(move |s| {
                    let colors = state_font.colors();
                    s.font_size(11.0).color(colors.text_muted)
                }),

            // Separator
            label(|| " | ")
                .style(move |s| {
                    let colors = state_sep5.colors();
                    s.font_size(11.0).color(colors.text_muted).padding_horiz(4.0)
                }),

            // Theme
            label(move || state_theme.theme.get().name())
                .style(move |s| {
                    let colors = state_theme.colors();
                    s.font_size(11.0).color(colors.text_muted)
                }),
        ))
        .style(|s| s.padding_horiz(12.0).items_center().width_full()),
    )
    .style(move |s| {
        let colors = state_style.colors();
        s.width_full()
            .height(layout::STATUS_BAR_HEIGHT)
            .background(colors.bg_status)
            .border_color(colors.border_subtle)
            .border_top(1.0)
            .items_center()
    })
}
