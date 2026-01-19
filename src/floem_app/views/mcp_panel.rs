//! MCP AI Assistant Panel for Floem UI
//!
//! Provides a side panel for interacting with MCP (Model Context Protocol) servers.
//! Uses Floem's reactive signal system (RwSignal) for state management.

use floem::peniko::Color;
use floem::prelude::*;
use floem::reactive::{RwSignal, SignalGet, SignalUpdate};
use floem::style::{AlignItems, CursorStyle, FlexDirection, JustifyContent};
use floem::views::{container, dyn_container, h_stack, label, scroll, v_stack, Decorators};

use crate::floem_app::async_bridge::{AsyncCommand, AsyncResult, ToolInfo, RiskLevel};
use crate::floem_app::theme::Theme;
use crate::floem_app::state::AppState;
use crate::floem_app::execution_pipeline::ExecutionPipeline;
use super::ai_block::{AiBlock, AiBlockState, ai_blocks_view};
use super::command_queue::command_queue_view;
use super::context_inspector::{ContextInspectorState, context_inspector};
use super::history_view::{HistoryViewState, history_view};

/// Panel tab selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PanelTab {
    /// Tools and AI responses (default)
    #[default]
    Tools,
    /// Command execution queue
    Queue,
    /// Execution history
    History,
    /// Terminal context inspector
    Context,
}

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
    /// AI block state for displaying tool results
    pub ai_block_state: AiBlockState,
    /// Command sender for async operations
    command_tx: Option<tokio::sync::mpsc::Sender<AsyncCommand>>,
    /// Result receiver for async operations
    result_rx: Option<std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<AsyncResult>>>>,
    /// App state for accessing current terminal
    app_state: Option<AppState>,
    /// Panel width in pixels
    pub width: RwSignal<f64>,
    /// Whether the divider is being dragged
    pub is_dragging: RwSignal<bool>,
    /// Active tab in the panel
    pub active_tab: RwSignal<PanelTab>,
    /// Execution pipeline state for command queue
    pub execution_pipeline: ExecutionPipeline,
    /// History view state
    pub history_state: HistoryViewState,
    /// Context inspector state
    pub context_state: ContextInspectorState,
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
            ai_block_state: AiBlockState::new(),
            command_tx: None,
            result_rx: None,
            app_state: None,
            width: RwSignal::new(350.0),
            is_dragging: RwSignal::new(false),
            active_tab: RwSignal::new(PanelTab::Tools),
            execution_pipeline: ExecutionPipeline::new(),
            history_state: HistoryViewState::new(),
            context_state: ContextInspectorState::new(),
        }
    }

    /// Create a new MCP panel state with async bridge
    pub fn with_bridge(
        command_tx: tokio::sync::mpsc::Sender<AsyncCommand>,
        result_rx: std::sync::mpsc::Receiver<AsyncResult>,
        app_state: AppState,
    ) -> Self {
        Self {
            visible: RwSignal::new(true),
            connected: RwSignal::new(false),
            server_name: RwSignal::new(String::from("No server")),
            tools: RwSignal::new(Vec::new()),
            selected_agent: RwSignal::new(AgentType::ClaudeCode),
            is_loading: RwSignal::new(false),
            error_message: RwSignal::new(None),
            ai_block_state: AiBlockState::new(),
            command_tx: Some(command_tx),
            result_rx: Some(std::sync::Arc::new(std::sync::Mutex::new(result_rx))),
            app_state: Some(app_state),
            width: RwSignal::new(350.0),
            is_dragging: RwSignal::new(false),
            active_tab: RwSignal::new(PanelTab::Tools),
            execution_pipeline: ExecutionPipeline::new(),
            history_state: HistoryViewState::new(),
            context_state: ContextInspectorState::new(),
        }
    }

    /// Set the active tab
    pub fn set_active_tab(&self, tab: PanelTab) {
        self.active_tab.set(tab);
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

    /// Call a tool with given parameters
    pub fn call_tool(&self, tool_name: String, params: serde_json::Value) {
        if let Some(ref tx) = self.command_tx {
            if !self.connected.get() {
                self.set_error(Some("Not connected to MCP server".to_string()));
                return;
            }

            self.set_loading(true);
            self.set_error(None);

            // Add a "thinking" block to show that we're calling the tool
            let block_id = uuid::Uuid::new_v4().to_string();
            self.ai_block_state.add_block(AiBlock::thinking(
                block_id.clone(),
                format!("Calling tool: {}", tool_name),
            ));

            if let Err(e) = tx.try_send(AsyncCommand::McpCallTool(tool_name.clone(), params)) {
                tracing::error!("Failed to send call tool command: {}", e);
                self.set_loading(false);
                self.set_error(Some(format!("Failed to call tool: {e}")));
                // Remove thinking block and add error block
                self.ai_block_state.remove_block(&block_id);
                self.ai_block_state.add_block(AiBlock::error(
                    uuid::Uuid::new_v4().to_string(),
                    format!("Failed to call {}: {}", tool_name, e),
                ));
            } else {
                tracing::info!("Sent call tool command for: {}", tool_name);
            }
        } else {
            self.set_error(Some("MCP bridge not initialized".to_string()));
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
                // Clear AI blocks when disconnecting
                self.ai_block_state.clear();
            }
            AsyncResult::McpTools(tools) => {
                tracing::info!("Received {} tools", tools.len());
                self.update_tools(tools);
                self.set_loading(false);
            }
            AsyncResult::McpToolResult(value) => {
                tracing::info!("Tool result: {:?}", value);
                self.set_loading(false);

                // Remove any "thinking" blocks
                let thinking_blocks: Vec<String> = self.ai_block_state.blocks.get()
                    .iter()
                    .filter(|b| b.block_type == super::ai_block::AiBlockType::Thinking)
                    .map(|b| b.id.clone())
                    .collect();
                for id in thinking_blocks {
                    self.ai_block_state.remove_block(&id);
                }

                // Convert the result to an AI block
                let block_id = uuid::Uuid::new_v4().to_string();

                // Check if the result is an error
                let is_error = value.get("is_error").and_then(|v| v.as_bool()).unwrap_or(false);

                if is_error {
                    // Display as error block
                    let error_text = value.get("content")
                        .and_then(|c| c.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|item| item.get("text"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("Unknown error");

                    self.ai_block_state.add_block(AiBlock::error(
                        block_id,
                        error_text.to_string(),
                    ));
                } else {
                    // Display as response block
                    let content_text = value.get("content")
                        .and_then(|c| c.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
                                .collect::<Vec<_>>()
                                .join("\n")
                        })
                        .unwrap_or_else(|| {
                            // Fallback: show the raw JSON
                            serde_json::to_string_pretty(&value).unwrap_or_else(|_| "Empty result".to_string())
                        });

                    self.ai_block_state.add_block(AiBlock::response(
                        block_id,
                        content_text,
                    ));
                }
            }
            AsyncResult::Error(msg) => {
                tracing::error!("MCP error: {}", msg);
                self.set_error(Some(msg.clone()));
                self.set_loading(false);

                // Remove any "thinking" blocks and add error block
                let thinking_blocks: Vec<String> = self.ai_block_state.blocks.get()
                    .iter()
                    .filter(|b| b.block_type == super::ai_block::AiBlockType::Thinking)
                    .map(|b| b.id.clone())
                    .collect();
                for id in thinking_blocks {
                    self.ai_block_state.remove_block(&id);
                }

                self.ai_block_state.add_block(AiBlock::error(
                    uuid::Uuid::new_v4().to_string(),
                    msg,
                ));
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
    let width = state.width;

    dyn_container(
        move || visible.get(),
        move |is_visible| {
            if !is_visible {
                return h_stack((label(|| ""),))
                    .style(|s| s.display(floem::style::Display::None));
            }

            // Panel container with divider
            h_stack((
                // Draggable divider
                divider_view(state.clone(), theme),
                // Panel content
                container(
                    v_stack((
                        // Header section
                        header_view(state.clone(), theme),
                        // Tab bar
                        panel_tab_bar(state.clone(), theme),
                        // Tab content (dynamic based on active tab)
                        panel_tab_content(state.clone(), theme),
                    ))
                    .style(move |s| {
                        s.flex_direction(FlexDirection::Column)
                            .width_full()
                            .height_full()
                    }),
                )
                .style(move |s| {
                    let colors = theme.get().colors();
                    s.width_full()
                        .height_full()
                        .background(colors.bg_primary)
                }),
            ))
            .style(move |s| {
                s.width(width.get())
                    .height_full()
            })
        },
    )
}

/// Draggable divider for resizing the panel
fn divider_view(state: McpPanelState, theme: RwSignal<Theme>) -> impl IntoView {
    let is_dragging = state.is_dragging;
    let width = state.width;
    let is_hovering = RwSignal::new(false);

    // Track the initial mouse position and width when drag starts
    let drag_start_x = RwSignal::new(0.0);
    let drag_start_width = RwSignal::new(width.get());

    container(label(|| ""))
        .on_event(floem::event::EventListener::PointerDown, move |event| {
            if let floem::event::Event::PointerDown(pointer_event) = event {
                tracing::info!("Divider drag started at x={}", pointer_event.pos.x);
                is_dragging.set(true);
                drag_start_x.set(pointer_event.pos.x);
                drag_start_width.set(width.get());
                return floem::event::EventPropagation::Stop;
            }
            floem::event::EventPropagation::Continue
        })
        .on_event(floem::event::EventListener::PointerMove, move |event| {
            if let floem::event::Event::PointerMove(pointer_event) = event {
                if is_dragging.get() {
                    let delta = drag_start_x.get() - pointer_event.pos.x;
                    let new_width = drag_start_width.get() + delta;

                    // Constrain width between min (200) and max (600)
                    let constrained_width = new_width.max(200.0).min(600.0);

                    width.set(constrained_width);
                    tracing::trace!("Panel width updated to: {}", constrained_width);
                    return floem::event::EventPropagation::Stop;
                }
            }
            floem::event::EventPropagation::Continue
        })
        .on_event(floem::event::EventListener::PointerUp, move |_event| {
            if is_dragging.get() {
                tracing::info!("Divider drag ended, final width: {}", width.get());
                is_dragging.set(false);
                return floem::event::EventPropagation::Stop;
            }
            floem::event::EventPropagation::Continue
        })
        .on_event(floem::event::EventListener::PointerEnter, move |_event| {
            is_hovering.set(true);
            floem::event::EventPropagation::Continue
        })
        .on_event(floem::event::EventListener::PointerLeave, move |_event| {
            is_hovering.set(false);
            floem::event::EventPropagation::Continue
        })
        .style(move |s| {
            let colors = theme.get().colors();
            let is_dragging_val = is_dragging.get();
            let is_hovering_val = is_hovering.get();

            let bg_color = if is_dragging_val {
                colors.accent_blue
            } else if is_hovering_val {
                colors.border
            } else {
                colors.border_subtle
            };

            s.width(5.0)
                .height_full()
                .background(bg_color)
                .cursor(CursorStyle::ColResize)
        })
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
                                    .map(|tool| tool_item_view(tool.clone(), state.clone(), theme))
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
            .max_height(250.0)
            .background(colors.bg_primary)
    })
}

/// Individual tool item
fn tool_item_view(tool: ToolInfo, state: McpPanelState, theme: RwSignal<Theme>) -> impl IntoView {
    let tool_name = tool.name.clone();
    let tool_name_for_click = tool.name.clone();
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
    .on_click_stop(move |_| {
        tracing::info!("Tool clicked: {}", tool_name_for_click);
        // Call the tool with empty parameters (for now)
        // In the future, we could show a dialog to input parameters
        state.call_tool(tool_name_for_click.clone(), serde_json::json!({}));
    })
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

/// AI blocks section view
fn ai_blocks_section_view(
    state: McpPanelState,
    theme: RwSignal<Theme>,
) -> impl IntoView {
    let ai_blocks = state.ai_block_state.blocks;

    dyn_container(
        move || ai_blocks.get().len(),
        move |block_count| {
            if block_count == 0 {
                // Empty state - no blocks to show
                container(label(|| "")).style(|s| s.display(floem::style::Display::None))
            } else {
                let state_for_execute = state.clone();
                let state_for_cancel = state.clone();

                // Execute callback - send command to PTY
                let on_execute = move |command: String| {
                    tracing::info!("Execute callback invoked for command: {}", command);

                    // Get the focused pane's terminal ID
                    let terminal_id = if let Some(ref app_state) = state_for_execute.app_state {
                        if let Some(active_tab) = app_state.active_tab_ref() {
                            if let Some((pane_id, _)) = active_tab.pane_tree.get().get_focused_leaf() {
                                Some(pane_id)
                            } else {
                                tracing::warn!("No focused pane found");
                                None
                            }
                        } else {
                            tracing::warn!("No active tab found");
                            None
                        }
                    } else {
                        tracing::warn!("No app state available");
                        None
                    };

                    if let Some(tid) = terminal_id {
                        if let Some(ref tx) = state_for_execute.command_tx {
                            // For now, use Medium risk level - in the future, this should come from the block
                            if let Err(e) = tx.try_send(AsyncCommand::ExecuteCommand {
                                command: command.clone(),
                                terminal_id: tid,
                                risk_level: RiskLevel::Medium,
                            }) {
                                tracing::error!("Failed to send execute command: {}", e);
                            } else {
                                tracing::info!("Command sent for execution: {}", command);
                            }
                        }
                    }
                };

                // Edit callback - placeholder for now
                let on_edit = move |command: String| {
                    tracing::info!("Edit callback invoked for command: {}", command);
                    // TODO: In the future, open an edit dialog or insert into input field
                };

                // Cancel callback - remove the block
                let on_cancel = move |block_id: String| {
                    tracing::info!("Cancel callback invoked for block: {}", block_id);
                    state_for_cancel.ai_block_state.remove_block(&block_id);
                };

                // Copy callback - placeholder for clipboard
                let on_copy = move |content: String| {
                    tracing::info!("Copy callback invoked, content length: {}", content.len());
                    // TODO: Implement clipboard copy in the future
                };

                container(
                    scroll(
                        container(ai_blocks_view(
                            &state.ai_block_state,
                            on_execute,
                            on_edit,
                            on_cancel,
                            on_copy,
                        ))
                            .style(|s| s.width_full())
                    )
                    .style(move |s| {
                        let colors = theme.get().colors();
                        s.width_full()
                            .flex_grow(1.0)
                            .background(colors.bg_primary)
                    })
                )
                .style(move |s| {
                    let colors = theme.get().colors();
                    s.width_full()
                        .flex_grow(1.0)
                        .border_top(1.0)
                        .border_color(colors.border)
                })
            }
        },
    )
}

/// Tab bar for panel navigation
fn panel_tab_bar(state: McpPanelState, theme: RwSignal<Theme>) -> impl IntoView {
    let active_tab = state.active_tab;

    let create_tab_button = move |tab: PanelTab, icon: &'static str, label_text: &'static str| {
        let state_clone = state.clone();
        container(
            h_stack((
                label(move || icon).style(move |s| s.font_size(12.0)),
                label(move || label_text).style(move |s| {
                    let colors = theme.get().colors();
                    s.font_size(11.0).color(colors.text_primary).margin_left(4.0)
                }),
            ))
            .style(|s| s.items_center()),
        )
        .on_click_stop(move |_| {
            state_clone.set_active_tab(tab);
        })
        .style(move |s| {
            let colors = theme.get().colors();
            let is_active = active_tab.get() == tab;

            let base = s
                .padding_horiz(10.0)
                .padding_vert(6.0)
                .cursor(CursorStyle::Pointer)
                .border_bottom(2.0);

            if is_active {
                base.background(colors.bg_secondary)
                    .border_color(colors.accent_blue)
            } else {
                base.border_color(Color::TRANSPARENT)
                    .hover(|s| s.background(colors.bg_tab_hover))
            }
        })
    };

    h_stack((
        create_tab_button(PanelTab::Tools, "🔧", "Tools"),
        create_tab_button(PanelTab::Queue, "📋", "Queue"),
        create_tab_button(PanelTab::History, "📜", "History"),
        create_tab_button(PanelTab::Context, "🔍", "Context"),
    ))
    .style(move |s| {
        let colors = theme.get().colors();
        s.width_full()
            .background(colors.bg_primary)
            .border_bottom(1.0)
            .border_color(colors.border)
    })
}

/// Tab content based on active tab
fn panel_tab_content(state: McpPanelState, theme: RwSignal<Theme>) -> impl IntoView {
    let active_tab = state.active_tab;

    dyn_container(
        move || active_tab.get(),
        move |tab| {
            match tab {
                PanelTab::Tools => {
                    // Tools tab: Agent selector + connection status + tools list + AI blocks
                    v_stack((
                        agent_selector_view(state.clone(), theme),
                        connection_status_view(state.clone(), theme),
                        tools_list_view(state.clone(), theme),
                        ai_blocks_section_view(state.clone(), theme),
                    ))
                    .style(|s| s.width_full().height_full().flex_col())
                    .into_any()
                }
                PanelTab::Queue => {
                    // Queue tab: Command execution queue
                    let pipeline = state.execution_pipeline.clone();
                    let on_approve = {
                        let pipeline = pipeline.clone();
                        move |id: uuid::Uuid| {
                            let _ = pipeline.approve(id);
                        }
                    };
                    let on_edit = {
                        let pipeline = pipeline.clone();
                        move |id: uuid::Uuid, new_command: String| {
                            let _ = pipeline.approve_modified(id, &new_command);
                        }
                    };
                    let on_reject = {
                        let pipeline = pipeline.clone();
                        move |id: uuid::Uuid| {
                            let _ = pipeline.reject(id);
                        }
                    };

                    command_queue_view(pipeline, theme, on_approve, on_edit, on_reject)
                        .style(|s| s.width_full().height_full())
                        .into_any()
                }
                PanelTab::History => {
                    // History tab: Execution history
                    let history_state = state.history_state.clone();
                    let on_replay = |command: String| {
                        tracing::info!("Replay command: {}", command);
                    };
                    history_view(history_state, theme.get(), on_replay)
                        .style(|s| s.width_full().height_full())
                        .into_any()
                }
                PanelTab::Context => {
                    // Context tab: Terminal context inspector
                    let context_state = state.context_state.clone();
                    context_inspector(context_state, theme.get())
                        .style(|s| s.width_full().height_full())
                        .into_any()
                }
            }
        },
    )
    .style(|s| s.width_full().height_full().flex_grow(1.0))
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
