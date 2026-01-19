//! View Components
//!
//! UI components for the Floem application.

mod tab_bar;
mod status_bar;
mod pane_view;
mod search;
mod settings_view;
mod mcp_panel;
mod ai_block;
mod streaming_view;
mod agent_dashboard;
mod tool_form;
mod command_queue;
mod history_view;
mod context_inspector;

pub mod terminal;

pub use tab_bar::tab_bar;
pub use terminal::terminal_area;
pub use status_bar::status_bar;
pub use pane_view::pane_tree_view;
pub use search::SearchBarState;
pub use settings_view::settings_panel;
pub use mcp_panel::{mcp_panel, McpPanelState};
pub use agent_dashboard::{agent_dashboard, AgentDashboardState, AgentRegistry, AgentConfig, ConnectionState, SessionInfo};
pub use tool_form::{tool_form, ToolFormState, FormParameter, ParameterType};
pub use history_view::{history_view, history_panel, HistoryViewState, HistoryGroup, HistoryEntryDisplay};
pub use context_inspector::{context_inspector, context_inspector_panel, ContextInspectorState};

// Re-exports for future AI/MCP integration (currently unused)
#[allow(unused_imports)]
pub use mcp_panel::AgentType;
#[allow(unused_imports)]
pub use ai_block::{ai_blocks_view, AiBlock, AiBlockState, AiBlockType, CommandRiskLevel};
#[allow(unused_imports)]
pub use streaming_view::{streaming_view, StreamingState};

// Command queue view exports
pub use command_queue::command_queue_view;
