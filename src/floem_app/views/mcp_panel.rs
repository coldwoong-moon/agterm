//! MCP AI Assistant Panel for Floem UI
//!
//! Provides a side panel for interacting with MCP (Model Context Protocol) servers.
//! Uses Floem's reactive signal system (RwSignal) for state management.

use floem::peniko::Color;
use floem::prelude::*;
use floem::reactive::{RwSignal, SignalGet, SignalUpdate};
use floem::style::{AlignItems, CursorStyle, FlexDirection, JustifyContent};
use floem::views::{container, dyn_container, h_stack, label, scroll, v_stack, Decorators};

use crate::floem_app::async_bridge::{AsyncCommand, AsyncResult, ToolInfo};
use crate::floem_app::theme::Theme;

/// AI Agent type for MCP integration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentType {
    /// Claude Code by Anthropic
    ClaudeCode,
    /// Gemini CLI by Google
    GeminiCli,
    /// OpenAI Codex
    OpenAICodex,
    /// Qwen Code by Alibaba
    QwenCode,
}

impl AgentType {
    /// Get agent display name
    pub fn name(&self) -> &'static str {
        match self {
            AgentType::ClaudeCode => "Claude Code",
            AgentType::GeminiCli => "Gemini CLI",
            AgentType::OpenAICodex => "OpenAI Codex",
            AgentType::QwenCode => "Qwen Code",
        }
    }

    /// Get all available agents (used in tests)
    #[cfg(test)]
    pub fn all() -> &'static [AgentType] {
        &[
            AgentType::ClaudeCode,
            AgentType::GeminiCli,
            AgentType::OpenAICodex,
            AgentType::QwenCode,
        ]
    }
}

/// MCP Panel state with reactive signals
#[derive(Clone)]
pub struct McpPanelState {
    /// Whether the panel is expanded/visible
    pub visible: RwSignal<bool>,
    /// Whether MCP is connected
    pub connected: RwSignal<bool>,
    /// Current server name
    pub server_name: RwSignal<String>,
    /// Available tools from the MCP server
    pub tools: RwSignal<Vec<ToolInfo>>,
    /// Currently selected agent
    pub selected_agent: RwSignal<AgentType>,
    /// Loading state
    pub is_loading: RwSignal<bool>,
    /// Error message (if any)
    pub error_message: RwSignal<Option<String>>,
    /// Command sender for async operations
    command_tx: Option<tokio::sync::mpsc::Sender<AsyncCommand>>,
    /// Result receiver for async operations
    result_rx: Option<std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<AsyncResult>>>>,
}

impl McpPanelState {
    /// Create a new MCP panel state without async bridge (for tests)
    pub fn new() -> Self {
        Self {
            visible: RwSignal::new(true),
            connected: RwSignal::new(false),
            server_name: RwSignal::new(String::from("No server")),
            tools: RwSignal::new(Vec::new()),
            selected_agent: RwSignal::new(AgentType::ClaudeCode),
            is_loading: RwSignal::new(false),
            error_message: RwSignal::new(None),
            command_tx: None,
            result_rx: None,
        }
    }

    /// Create a new MCP panel state with async bridge
    pub fn with_bridge(
        command_tx: tokio::sync::mpsc::Sender<AsyncCommand>,
        result_rx: std::sync::mpsc::Receiver<AsyncResult>,
    ) -> Self {
        Self {
            visible: RwSignal::new(true),
            connected: RwSignal::new(false),
            server_name: RwSignal::new(String::from("No server")),
            tools: RwSignal::new(Vec::new()),
            selected_agent: RwSignal::new(AgentType::ClaudeCode),
            is_loading: RwSignal::new(false),
            error_message: RwSignal::new(None),
            command_tx: Some(command_tx),
            result_rx: Some(std::sync::Arc::new(std::sync::Mutex::new(result_rx))),
        }
    }

    /// Toggle panel visibility
    pub fn toggle_visibility(&self) {
        self.visible.update(|v| *v = !*v);
    }

    /// Select a specific agent
    pub fn select_agent(&self, agent: AgentType) {
        self.selected_agent.set(agent);
    }

    // Methods below are prepared for future MCP server integration.
    // Currently unused but will be called when MCP connection is implemented.

    /// Set connection status
    pub fn set_connected(&self, connected: bool, server_name: Option<String>) {
        self.connected.set(connected);
        if let Some(name) = server_name {
            self.server_name.set(name);
        }
    }

    /// Update tools list
    pub fn update_tools(&self, tools: Vec<ToolInfo>) {
        self.tools.set(tools);
    }

    /// Set loading state
    pub fn set_loading(&self, loading: bool) {
        self.is_loading.set(loading);
    }

    /// Set error message
    pub fn set_error(&self, error: Option<String>) {
        self.error_message.set(error);
    }

    /// Connect to the currently selected agent's MCP server
    pub fn connect(&self) {
        if let Some(ref tx) = self.command_tx {
            let agent_name = match self.selected_agent.get() {
                AgentType::ClaudeCode => "claude_code",
                AgentType::GeminiCli => "gemini_cli",
                AgentType::OpenAICodex => "openai_codex",
                AgentType::QwenCode => "qwen_code",
            };

            self.set_loading(true);
            self.set_error(None);

            if let Err(e) = tx.try_send(AsyncCommand::McpConnect(agent_name.to_string())) {
                tracing::error!("Failed to send connect command: {}", e);
                self.set_loading(false);
                self.set_error(Some(format!("Failed to connect: {e}")));
            } else {
                tracing::info!("Sent connect command for agent: {}", agent_name);
            }
        } else {
            tracing::warn!("No command sender available for MCP connection");
            self.set_error(Some("MCP bridge not initialized".to_string()));
        }
    }

    /// Disconnect from the current MCP server
    pub fn disconnect(&self) {
        if let Some(ref tx) = self.command_tx {
            self.set_loading(true);

            if let Err(e) = tx.try_send(AsyncCommand::McpDisconnect) {
                tracing::error!("Failed to send disconnect command: {}", e);
                self.set_loading(false);
            } else {
                tracing::info!("Sent disconnect command");
            }
        }
    }

    /// Request list of available tools
    pub fn refresh_tools(&self) {
        if let Some(ref tx) = self.command_tx {
            if !self.connected.get() {
                return;
            }

            self.set_loading(true);

            if let Err(e) = tx.try_send(AsyncCommand::McpListTools) {
                tracing::error!("Failed to send list tools command: {}", e);
                self.set_loading(false);
            } else {
                tracing::info!("Sent list tools command");
            }
        }
    }

    /// Poll for async results and update state (call this periodically)
    pub fn poll_results(&self) {
        if let Some(ref rx_arc) = self.result_rx {
            if let Ok(rx) = rx_arc.try_lock() {
                while let Ok(result) = rx.try_recv() {
                    self.handle_result(result);
                }
            }
        }
    }

    /// Handle a single async result
    fn handle_result(&self, result: AsyncResult) {
        match result {
            AsyncResult::McpConnected { server_name } => {
                tracing::info!("MCP connected to: {}", server_name);
                self.set_connected(true, Some(server_name));
                self.set_loading(false);
                self.set_error(None);
                // Automatically fetch tools after connecting
                self.refresh_tools();
            }
            AsyncResult::McpDisconnected => {
                tracing::info!("MCP disconnected");
                self.set_connected(false, Some("No server".to_string()));
                self.tools.set(Vec::new());
                self.set_loading(false);
            }
            AsyncResult::McpTools(tools) => {
                tracing::info!("Received {} tools", tools.len());
                self.update_tools(tools);
                self.set_loading(false);
            }
            AsyncResult::McpToolResult(value) => {
                tracing::info!("Tool result: {:?}", value);
                self.set_loading(false);
            }
            AsyncResult::Error(msg) => {
                tracing::error!("MCP error: {}", msg);
                self.set_error(Some(msg));
                self.set_loading(false);
            }
            AsyncResult::CommandApproved { .. } | AsyncResult::CommandBlocked { .. } => {
                // Command validation results - handled elsewhere
            }
        }
    }
}

impl Default for McpPanelState {
    fn default() -> Self {
        Self::new()
    }
}

/// Create the MCP panel view
pub fn mcp_panel(state: McpPanelState, theme: RwSignal<Theme>) -> impl IntoView {
    let visible = state.visible;

    dyn_container(
        move || visible.get(),
        move |is_visible| {
            if !is_visible {
                return container(label(|| "")).style(|s| s.display(floem::style::Display::None));
            }

            // Panel container
            container(
                v_stack((
                    // Header section
                    header_view(state.clone(), theme),
                    // Agent selector buttons
                    agent_selector_view(state.clone(), theme),
                    // Connection status
                    connection_status_view(state.clone(), theme),
                    // Tools list (scrollable)
                    tools_list_view(state.clone(), theme),
                ))
                .style(move |s| {
                    s.flex_direction(FlexDirection::Column)
                        .width(350.0)
                        .height_full()
                }),
            )
            .style(move |s| {
                let colors = theme.get().colors();
                s.width(350.0)
                    .height_full()
                    .background(colors.bg_primary)
                    .border_left(1.0)
                    .border_color(colors.border)
            })
        },
    )
}

/// Header with title and collapse button
fn header_view(state: McpPanelState, theme: RwSignal<Theme>) -> impl IntoView {
    let state_clone = state.clone();

    h_stack((
        label(|| "AI Agents".to_string()).style(move |s| {
            let colors = theme.get().colors();
            s.font_size(16.0)
                .font_weight(floem::text::Weight::BOLD)
                .color(colors.text_primary)
                .flex_grow(1.0)
        }),
        // Collapse button
        container(label(|| "×".to_string()))
            .on_click_stop(move |_| {
                state_clone.toggle_visibility();
            })
            .style(move |s| {
                let colors = theme.get().colors();
                s.padding(4.0)
                    .font_size(20.0)
                    .color(colors.text_primary)
                    .cursor(CursorStyle::Pointer)
                    .hover(|s| s.background(colors.bg_tab_hover))
                    .border_radius(4.0)
            }),
    ))
    .style(move |s| {
        let colors = theme.get().colors();
        s.width_full()
            .padding(12.0)
            .background(colors.bg_secondary)
            .border_bottom(1.0)
            .border_color(colors.border)
            .justify_content(JustifyContent::SpaceBetween)
            .align_items(AlignItems::Center)
    })
}

/// Agent selector buttons (2x2 grid layout)
fn agent_selector_view(
    state: McpPanelState,
    theme: RwSignal<Theme>,
) -> impl IntoView {
    let state_clone = state.clone();
    let state_clone2 = state.clone();

    // Create agent button helper
    let create_agent_button = move |agent: AgentType, state: McpPanelState, theme: RwSignal<Theme>| {
        let selected = state.selected_agent;
        container(label(move || agent.name().to_string()))
            .on_click_stop(move |_| {
                tracing::info!("Agent button clicked: {:?}", agent);
                state.select_agent(agent);
            })
            .style(move |s| {
                let colors = theme.get().colors();
                let is_selected = selected.get() == agent;
                let base = s
                    .padding(10.0)
                    .font_size(11.0)
                    .border(1.0)
                    .border_radius(6.0)
                    .cursor(CursorStyle::Pointer)
                    .flex_grow(1.0)
                    .justify_content(JustifyContent::Center)
                    .align_items(AlignItems::Center);

                if is_selected {
                    base.background(colors.accent_blue)
                        .border_color(colors.accent_blue)
                        .color(Color::WHITE)
                } else {
                    base.background(colors.bg_secondary)
                        .border_color(colors.border)
                        .color(colors.text_primary)
                        .hover(|s| s.background(colors.bg_tab_hover))
                }
            })
    };

    // 2x2 Grid layout
    v_stack((
        // Row 1: Claude Code, Gemini CLI
        h_stack((
            create_agent_button(AgentType::ClaudeCode, state_clone.clone(), theme),
            create_agent_button(AgentType::GeminiCli, state_clone.clone(), theme),
        ))
        .style(|s| s.width_full().gap(6.0)),
        // Row 2: OpenAI Codex, Qwen Code
        h_stack((
            create_agent_button(AgentType::OpenAICodex, state_clone2.clone(), theme),
            create_agent_button(AgentType::QwenCode, state_clone2.clone(), theme),
        ))
        .style(|s| s.width_full().gap(6.0)),
    ))
    .style(move |s| {
        let colors = theme.get().colors();
        s.width_full()
            .padding(12.0)
            .gap(6.0)
            .background(colors.bg_primary)
    })
}

/// Connection status display
fn connection_status_view(
    state: McpPanelState,
    theme: RwSignal<Theme>,
) -> impl IntoView {
    let connected = state.connected;
    let server_name = state.server_name;
    let error_message = state.error_message;
    let is_loading = state.is_loading;
    let state_for_button = state.clone();

    v_stack((
        // Connection indicator and button row
        h_stack((
            // Status indicator
            h_stack((
                label(move || {
                    if connected.get() {
                        "●".to_string()
                    } else {
                        "○".to_string()
                    }
                })
                .style(move |s| {
                    let colors = theme.get().colors();
                    let color = if connected.get() {
                        colors.accent_green
                    } else {
                        colors.text_muted
                    };
                    s.font_size(14.0).color(color)
                }),
                label(move || server_name.get()).style(move |s| {
                    let colors = theme.get().colors();
                    s.font_size(12.0).color(colors.text_secondary).margin_left(6.0)
                }),
            ))
            .style(|s| s.align_items(AlignItems::Center).flex_grow(1.0)),
            // Connect/Disconnect button
            dyn_container(
                move || (connected.get(), is_loading.get()),
                move |(is_connected, loading)| {
                    let state_clone = state_for_button.clone();
                    let button_text = if loading {
                        "...".to_string()
                    } else if is_connected {
                        "Disconnect".to_string()
                    } else {
                        "Connect".to_string()
                    };

                    container(label(move || button_text.clone()))
                        .on_click_stop(move |_| {
                            if loading {
                                return;
                            }
                            if is_connected {
                                state_clone.disconnect();
                            } else {
                                state_clone.connect();
                            }
                        })
                        .style(move |s| {
                            let colors = theme.get().colors();
                            let base = s
                                .padding_horiz(12.0)
                                .padding_vert(6.0)
                                .font_size(11.0)
                                .border(1.0)
                                .border_radius(4.0);

                            if loading {
                                base.background(colors.bg_secondary)
                                    .border_color(colors.border)
                                    .color(colors.text_muted)
                                    .cursor(CursorStyle::Default)
                            } else if is_connected {
                                base.background(colors.bg_secondary)
                                    .border_color(colors.accent_red)
                                    .color(colors.accent_red)
                                    .cursor(CursorStyle::Pointer)
                                    .hover(|s| s.background(Color::rgba8(235, 100, 115, 30)))
                            } else {
                                base.background(colors.accent_green)
                                    .border_color(colors.accent_green)
                                    .color(Color::WHITE)
                                    .cursor(CursorStyle::Pointer)
                                    .hover(|s| s.background(Color::rgba8(80, 200, 120, 255)))
                            }
                        })
                },
            ),
        ))
        .style(|s| s.width_full().align_items(AlignItems::Center).justify_content(JustifyContent::SpaceBetween)),
        // Error message (if any)
        dyn_container(
            move || error_message.get(),
            move |error_opt| {
                if let Some(error_text) = error_opt {
                    container(label(move || error_text.clone())).style(move |s| {
                        let colors = theme.get().colors();
                        s.font_size(11.0)
                            .color(colors.accent_red)
                            .margin_top(4.0)
                            .padding(6.0)
                            .background(Color::rgba8(235, 100, 115, 20))
                            .border_radius(4.0)
                    })
                } else {
                    container(label(|| "")).style(|s| s.display(floem::style::Display::None))
                }
            },
        ),
    ))
    .style(move |s| {
        let colors = theme.get().colors();
        s.width_full()
            .padding(12.0)
            .background(colors.bg_secondary)
            .border_bottom(1.0)
            .border_color(colors.border)
    })
}

/// Tools list with scrolling
fn tools_list_view(
    state: McpPanelState,
    theme: RwSignal<Theme>,
) -> impl IntoView {
    let tools = state.tools;
    let is_loading = state.is_loading;

    scroll(
        v_stack((
            // Loading indicator
            dyn_container(
                move || is_loading.get(),
                move |loading| {
                    if loading {
                        container(label(|| "Loading tools...".to_string())).style(move |s| {
                            let colors = theme.get().colors();
                            s.font_size(12.0)
                                .color(colors.text_muted)
                                .padding(12.0)
                                .width_full()
                                .justify_content(JustifyContent::Center)
                        })
                    } else {
                        container(label(|| "")).style(|s| s.display(floem::style::Display::None))
                    }
                },
            ),
            // Tools list
            dyn_container(
                move || (tools.get(), is_loading.get()),
                move |(tool_list, loading)| {
                    if tool_list.is_empty() && !loading {
                        container(
                            v_stack((
                                label(|| "🔌 No MCP Server Connected".to_string())
                                    .style(move |s| {
                                        let colors = theme.get().colors();
                                        s.font_size(14.0)
                                            .font_weight(floem::text::Weight::MEDIUM)
                                            .color(colors.text_secondary)
                                            .margin_bottom(8.0)
                                    }),
                                label(|| "Select an AI agent above to connect".to_string())
                                    .style(move |s| {
                                        let colors = theme.get().colors();
                                        s.font_size(12.0)
                                            .color(colors.text_muted)
                                    }),
                                label(|| "to an MCP server and access tools.".to_string())
                                    .style(move |s| {
                                        let colors = theme.get().colors();
                                        s.font_size(12.0)
                                            .color(colors.text_muted)
                                    }),
                            ))
                            .style(move |s| {
                                s.padding(24.0)
                                    .width_full()
                                    .align_items(AlignItems::Center)
                                    .justify_content(JustifyContent::Center)
                            }),
                        )
                    } else {
                        container(
                            v_stack((
                                tool_list
                                    .iter()
                                    .map(|tool| tool_item_view(tool.clone(), theme))
                                    .collect::<Vec<_>>(),
                            ))
                            .style(|s| s.width_full().gap(8.0)),
                        )
                    }
                },
            ),
        ))
        .style(move |s| {
            s.width_full()
                .padding(12.0)
                .gap(8.0)
                .flex_direction(FlexDirection::Column)
        }),
    )
    .style(move |s| {
        let colors = theme.get().colors();
        s.width_full()
            .flex_grow(1.0)
            .background(colors.bg_primary)
    })
}

/// Individual tool item
fn tool_item_view(tool: ToolInfo, theme: RwSignal<Theme>) -> impl IntoView {
    let tool_name = tool.name.clone();
    let tool_desc = tool.description.clone();
    let has_desc = tool.description.is_some();

    v_stack((
        label(move || tool_name.clone()).style(move |s| {
            let colors = theme.get().colors();
            s.font_size(13.0)
                .font_weight(floem::text::Weight::SEMIBOLD)
                .color(colors.text_primary)
        }),
        container(label(move || {
            tool_desc
                .clone()
                .unwrap_or_else(|| "No description".to_string())
        }))
        .style(move |s| {
            let colors = theme.get().colors();
            if has_desc {
                s.font_size(11.0)
                    .color(colors.text_secondary)
                    .margin_top(4.0)
            } else {
                s.font_size(11.0)
                    .color(colors.text_muted)
                    .margin_top(4.0)
                    .font_style(floem::text::Style::Italic)
            }
        }),
    ))
    .style(move |s| {
        let colors = theme.get().colors();
        s.width_full()
            .padding(10.0)
            .background(colors.bg_secondary)
            .border(1.0)
            .border_color(colors.border_subtle)
            .border_radius(6.0)
            .cursor(CursorStyle::Pointer)
            .hover(|s| {
                s.background(colors.bg_tab_hover)
                    .border_color(colors.border)
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type_name() {
        assert_eq!(AgentType::ClaudeCode.name(), "Claude Code");
        assert_eq!(AgentType::GeminiCli.name(), "Gemini CLI");
        assert_eq!(AgentType::OpenAICodex.name(), "OpenAI Codex");
        assert_eq!(AgentType::QwenCode.name(), "Qwen Code");
    }

    #[test]
    fn test_agent_type_all() {
        let all_agents = AgentType::all();
        assert_eq!(all_agents.len(), 4);
        assert_eq!(all_agents[0], AgentType::ClaudeCode);
        assert_eq!(all_agents[1], AgentType::GeminiCli);
        assert_eq!(all_agents[2], AgentType::OpenAICodex);
        assert_eq!(all_agents[3], AgentType::QwenCode);
    }

    #[test]
    fn test_mcp_panel_state_new() {
        let state = McpPanelState::new();
        assert!(state.visible.get());
        assert!(!state.connected.get());
        assert_eq!(state.server_name.get(), "No server");
        assert!(state.tools.get().is_empty());
        assert_eq!(state.selected_agent.get(), AgentType::ClaudeCode);
        assert!(!state.is_loading.get());
        assert!(state.error_message.get().is_none());
    }

    #[test]
    fn test_mcp_panel_state_toggle_visibility() {
        let state = McpPanelState::new();
        assert!(state.visible.get());

        state.toggle_visibility();
        assert!(!state.visible.get());

        state.toggle_visibility();
        assert!(state.visible.get());
    }

    #[test]
    fn test_mcp_panel_state_select_agent() {
        let state = McpPanelState::new();
        assert_eq!(state.selected_agent.get(), AgentType::ClaudeCode);

        state.select_agent(AgentType::GeminiCli);
        assert_eq!(state.selected_agent.get(), AgentType::GeminiCli);

        state.select_agent(AgentType::QwenCode);
        assert_eq!(state.selected_agent.get(), AgentType::QwenCode);
    }

    #[test]
    fn test_mcp_panel_state_set_connected() {
        let state = McpPanelState::new();
        assert!(!state.connected.get());

        state.set_connected(true, Some("test-server".to_string()));
        assert!(state.connected.get());
        assert_eq!(state.server_name.get(), "test-server");

        state.set_connected(false, None);
        assert!(!state.connected.get());
        // Server name should remain from previous set
        assert_eq!(state.server_name.get(), "test-server");
    }

    #[test]
    fn test_mcp_panel_state_update_tools() {
        let state = McpPanelState::new();
        assert!(state.tools.get().is_empty());

        let tools = vec![
            ToolInfo {
                name: "tool1".to_string(),
                description: Some("Description 1".to_string()),
            },
            ToolInfo {
                name: "tool2".to_string(),
                description: None,
            },
        ];

        state.update_tools(tools.clone());
        assert_eq!(state.tools.get().len(), 2);
        assert_eq!(state.tools.get()[0].name, "tool1");
        assert_eq!(state.tools.get()[1].name, "tool2");
    }

    #[test]
    fn test_mcp_panel_state_set_loading() {
        let state = McpPanelState::new();
        assert!(!state.is_loading.get());

        state.set_loading(true);
        assert!(state.is_loading.get());

        state.set_loading(false);
        assert!(!state.is_loading.get());
    }

    #[test]
    fn test_mcp_panel_state_set_error() {
        let state = McpPanelState::new();
        assert!(state.error_message.get().is_none());

        state.set_error(Some("Test error".to_string()));
        assert_eq!(state.error_message.get(), Some("Test error".to_string()));

        state.set_error(None);
        assert!(state.error_message.get().is_none());
    }

    #[test]
    fn test_agent_type_equality() {
        assert_eq!(AgentType::ClaudeCode, AgentType::ClaudeCode);
        assert_ne!(AgentType::ClaudeCode, AgentType::GeminiCli);
    }

    #[test]
    fn test_mcp_panel_state_default() {
        let state = McpPanelState::default();
        assert!(state.visible.get());
        assert!(!state.connected.get());
    }
}
