//! Context Inspector View Component
//!
//! Displays terminal context information including current directory,
//! shell environment, recent output, and environment variables.

use floem::prelude::*;
use floem::reactive::{RwSignal, SignalGet, SignalUpdate};
use floem::style::CursorStyle;
use floem::views::{container, h_stack, label, scroll, v_stack, Decorators, dyn_container, empty};

use crate::floem_app::theme::Theme;

/// Context Inspector state
#[derive(Clone)]
pub struct ContextInspectorState {
    /// Current working directory
    pub cwd: RwSignal<String>,
    /// Shell name (e.g., zsh, bash)
    pub shell: RwSignal<String>,
    /// Last executed command
    pub last_command: RwSignal<Option<String>>,
    /// Exit code of last command
    pub last_exit_code: RwSignal<Option<i32>>,
    /// Environment variables
    pub env_vars: RwSignal<Vec<(String, String)>>,
    /// Recent terminal output (last N lines)
    pub recent_output: RwSignal<String>,
    /// Whether to show all environment variables
    pub show_all_env: RwSignal<bool>,
    /// PTY session ID (if applicable)
    pub pty_session_id: RwSignal<Option<String>>,
    /// Terminal dimensions (cols x rows)
    pub terminal_size: RwSignal<(u16, u16)>,
}

impl ContextInspectorState {
    pub fn new() -> Self {
        Self {
            cwd: RwSignal::new(String::from("/")),
            shell: RwSignal::new(String::from("unknown")),
            last_command: RwSignal::new(None),
            last_exit_code: RwSignal::new(None),
            env_vars: RwSignal::new(Vec::new()),
            recent_output: RwSignal::new(String::new()),
            show_all_env: RwSignal::new(false),
            pty_session_id: RwSignal::new(None),
            terminal_size: RwSignal::new((80, 24)),
        }
    }

    /// Update current working directory
    pub fn set_cwd(&self, cwd: String) {
        self.cwd.set(cwd);
    }

    /// Update shell
    pub fn set_shell(&self, shell: String) {
        self.shell.set(shell);
    }

    /// Update last command and exit code
    pub fn set_last_command(&self, command: String, exit_code: i32) {
        self.last_command.set(Some(command));
        self.last_exit_code.set(Some(exit_code));
    }

    /// Update environment variables
    pub fn set_env_vars(&self, vars: Vec<(String, String)>) {
        self.env_vars.set(vars);
    }

    /// Update recent output
    pub fn set_recent_output(&self, output: String) {
        self.recent_output.set(output);
    }

    /// Toggle showing all environment variables
    pub fn toggle_show_all_env(&self) {
        self.show_all_env.update(|show| *show = !*show);
    }

    /// Update PTY session ID
    pub fn set_pty_session(&self, session_id: Option<String>) {
        self.pty_session_id.set(session_id);
    }

    /// Update terminal size
    pub fn set_terminal_size(&self, cols: u16, rows: u16) {
        self.terminal_size.set((cols, rows));
    }

    /// Get filtered environment variables (important ones first)
    pub fn filtered_env_vars(&self, show_all: bool) -> Vec<(String, String)> {
        let vars = self.env_vars.get();

        if show_all {
            return vars;
        }

        // Important environment variables to show by default
        let important = [
            "PATH", "HOME", "USER", "SHELL", "TERM", "LANG",
            "PWD", "OLDPWD", "EDITOR", "NODE_ENV", "VIRTUAL_ENV",
            "CARGO_HOME", "RUSTUP_HOME", "GOPATH", "JAVA_HOME",
        ];

        vars.into_iter()
            .filter(|(key, _)| important.contains(&key.as_str()))
            .collect()
    }
}

impl Default for ContextInspectorState {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a labeled info row
fn info_row(label_text: String, value: String, theme: Theme) -> impl IntoView {
    let colors = theme.colors();

    container(
        h_stack((
            label(move || label_text.clone())
                .style(move |s| {
                    s.font_size(11.0)
                        .color(colors.text_muted)
                        .width(120.0)
                        .font_weight(floem::text::Weight::BOLD)
                }),
            label(move || value.clone())
                .style(move |s| {
                    s.font_size(11.0)
                        .color(colors.text_primary)
                        .flex_grow(1.0)
                }),
        ))
        .style(|s| s.width_full().items_center().gap(8.0))
    )
    .style(move |s| {
        s.width_full()
            .padding_vert(6.0)
            .padding_horiz(12.0)
            .border_bottom(1.0)
            .border_color(colors.border_subtle)
    })
}

/// Create environment variables section
fn env_vars_section(
    state: ContextInspectorState,
    theme: Theme,
) -> impl IntoView {
    let colors = theme.colors();

    let state_vars = state.clone();
    let state_toggle = state.clone();

    v_stack((
        // Section header
        container(
            h_stack((
                label(|| "Environment Variables")
                    .style(move |s| {
                        s.font_size(13.0)
                            .color(colors.text_primary)
                            .font_weight(floem::text::Weight::BOLD)
                    }),

                // Spacer
                container(label(|| "")).style(|s| s.flex_grow(1.0)),

                // Toggle button
                container(
                    label(move || {
                        if state_toggle.show_all_env.get() {
                            "Show Less"
                        } else {
                            "Show All"
                        }
                    })
                )
                .on_click_stop(move |_| {
                    state_toggle.toggle_show_all_env();
                })
                .style(move |s| {
                    s.font_size(11.0)
                        .color(colors.accent_blue)
                        .padding(4.0)
                        .border_radius(4.0)
                        .cursor(CursorStyle::Pointer)
                        .hover(|s| s.background(colors.bg_tab_hover))
                }),
            ))
            .style(|s| s.width_full().items_center())
        )
        .style(move |s| {
            s.padding(12.0)
                .width_full()
                .border_bottom(1.0)
                .border_color(colors.border)
        }),

        // Environment variables list
        dyn_container(
            move || (state_vars.filtered_env_vars(state_vars.show_all_env.get()), state_vars.show_all_env.get()),
            move |(vars, _show_all)| {
                if vars.is_empty() {
                    container(
                        label(|| "No environment variables")
                            .style(move |s| {
                                s.font_size(11.0)
                                    .color(colors.text_muted)
                                    .padding(12.0)
                            })
                    )
                    .style(|s| s.width_full())
                    .into_any()
                } else {
                    scroll(
                        container(
                            empty()
                        )
                        .style(|s| s.width_full())
                    )
                    .style(|s| s.width_full().max_height(200.0))
                    .into_any()
                }
            }
        ),
    ))
    .style(|s| s.width_full())
}

/// Create recent output section
fn recent_output_section(
    state: ContextInspectorState,
    theme: Theme,
) -> impl IntoView {
    let colors = theme.colors();

    v_stack((
        // Section header
        container(
            label(|| "Recent Output (last 50 lines)")
                .style(move |s| {
                    s.font_size(13.0)
                        .color(colors.text_primary)
                        .font_weight(floem::text::Weight::BOLD)
                })
        )
        .style(move |s| {
            s.padding(12.0)
                .width_full()
                .border_bottom(1.0)
                .border_color(colors.border)
        }),

        // Output content
        scroll(
            container(
                label(move || {
                    let output = state.recent_output.get();
                    if output.is_empty() {
                        "(No recent output)".to_string()
                    } else {
                        output
                    }
                })
                .style(move |s| {
                    s.font_size(10.0)
                        .color(if state.recent_output.get().is_empty() {
                            colors.text_muted
                        } else {
                            colors.text_primary
                        })
                        .font_family("monospace".to_string())
                        .line_height(1.4)
                })
            )
            .style(move |s| {
                s.padding(12.0)
                    .width_full()
                    .background(colors.bg_secondary)
                    .border_radius(4.0)
            })
        )
        .style(|s| s.width_full().flex_grow(1.0).max_height(300.0))
        .style(move |s| s.padding(12.0).background(colors.bg_primary)),
    ))
    .style(|s| s.width_full())
}

/// Main context inspector view
pub fn context_inspector(
    state: ContextInspectorState,
    theme: Theme,
) -> impl IntoView {
    let colors = theme.colors();

    let state_cwd = state.clone();
    let state_shell = state.clone();
    let state_cmd = state.clone();
    let state_exit = state.clone();
    let state_pty = state.clone();
    let state_size = state.clone();

    container(
        v_stack((
            // Header
            container(
                label(|| "🔍 Context Inspector")
                    .style(move |s| {
                        s.font_size(16.0)
                            .color(colors.text_primary)
                            .font_weight(floem::text::Weight::BOLD)
                    })
            )
            .style(move |s| {
                s.padding(16.0)
                    .width_full()
                    .border_bottom(2.0)
                    .border_color(colors.border)
            }),

            // Scrollable content
            scroll(
                v_stack((
                    // Terminal State section
                    v_stack((
                        container(
                            label(|| "Terminal State")
                                .style(move |s| {
                                    s.font_size(13.0)
                                        .color(colors.text_primary)
                                        .font_weight(floem::text::Weight::BOLD)
                                })
                        )
                        .style(move |s| {
                            s.padding(12.0)
                                .width_full()
                                .background(colors.bg_secondary)
                                .border_bottom(1.0)
                                .border_color(colors.border)
                        }),

                        info_row("CWD:".to_string(), state_cwd.cwd.get(), theme),
                        info_row("Shell:".to_string(), state_shell.shell.get(), theme),
                        info_row("Last Command:".to_string(), state_cmd.last_command.get().unwrap_or_else(|| "(none)".to_string()), theme),
                        info_row("Exit Code:".to_string(),
                            state_exit.last_exit_code.get()
                                .map(|code| {
                                    if code == 0 {
                                        format!("✅ {}", code)
                                    } else {
                                        format!("❌ {}", code)
                                    }
                                })
                                .unwrap_or_else(|| "(none)".to_string()),
                            theme
                        ),
                        info_row("PTY Session:".to_string(),
                            state_pty.pty_session_id.get()
                                .unwrap_or_else(|| "(none)".to_string()),
                            theme
                        ),
                        info_row("Terminal Size:".to_string(),
                            {
                                let (cols, rows) = state_size.terminal_size.get();
                                format!("{}x{}", cols, rows)
                            },
                            theme
                        ),
                    ))
                    .style(|s| s.width_full()),

                    // Spacer
                    container(label(|| "")).style(|s| s.height(16.0)),

                    // Environment section
                    env_vars_section(state.clone(), theme),

                    // Spacer
                    container(label(|| "")).style(|s| s.height(16.0)),

                    // Recent output section
                    recent_output_section(state.clone(), theme),
                ))
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

/// Create a standalone context inspector panel with default styling
pub fn context_inspector_panel(
    state: ContextInspectorState,
    theme: Theme,
) -> impl IntoView {
    container(context_inspector(state, theme))
        .style(move |s| {
            let colors = theme.colors();
            s.width(350.0)
                .height_full()
                .border_left(1.0)
                .border_color(colors.border)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_inspector_state_new() {
        let state = ContextInspectorState::new();
        assert_eq!(state.cwd.get(), "/");
        assert_eq!(state.shell.get(), "unknown");
        assert_eq!(state.last_command.get(), None);
        assert_eq!(state.last_exit_code.get(), None);
        assert_eq!(state.show_all_env.get(), false);
        assert_eq!(state.terminal_size.get(), (80, 24));
    }

    #[test]
    fn test_set_cwd() {
        let state = ContextInspectorState::new();
        state.set_cwd("/home/user".to_string());
        assert_eq!(state.cwd.get(), "/home/user");
    }

    #[test]
    fn test_set_last_command() {
        let state = ContextInspectorState::new();
        state.set_last_command("ls -la".to_string(), 0);
        assert_eq!(state.last_command.get(), Some("ls -la".to_string()));
        assert_eq!(state.last_exit_code.get(), Some(0));
    }

    #[test]
    fn test_toggle_show_all_env() {
        let state = ContextInspectorState::new();
        assert_eq!(state.show_all_env.get(), false);
        state.toggle_show_all_env();
        assert_eq!(state.show_all_env.get(), true);
        state.toggle_show_all_env();
        assert_eq!(state.show_all_env.get(), false);
    }

    #[test]
    fn test_filtered_env_vars() {
        let state = ContextInspectorState::new();

        let vars = vec![
            ("PATH".to_string(), "/usr/bin".to_string()),
            ("HOME".to_string(), "/home/user".to_string()),
            ("RANDOM_VAR".to_string(), "value".to_string()),
            ("NODE_ENV".to_string(), "development".to_string()),
        ];

        state.set_env_vars(vars);

        // With show_all = false, should only show important vars
        let filtered = state.filtered_env_vars(false);
        assert_eq!(filtered.len(), 3);
        assert!(filtered.iter().any(|(k, _)| k == "PATH"));
        assert!(filtered.iter().any(|(k, _)| k == "HOME"));
        assert!(filtered.iter().any(|(k, _)| k == "NODE_ENV"));
        assert!(!filtered.iter().any(|(k, _)| k == "RANDOM_VAR"));

        // With show_all = true, should show all vars
        let all_vars = state.filtered_env_vars(true);
        assert_eq!(all_vars.len(), 4);
    }

    #[test]
    fn test_set_terminal_size() {
        let state = ContextInspectorState::new();
        state.set_terminal_size(120, 40);
        assert_eq!(state.terminal_size.get(), (120, 40));
    }
}
