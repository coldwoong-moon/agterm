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

pub mod terminal;

pub use tab_bar::tab_bar;
pub use terminal::terminal_area;
pub use status_bar::status_bar;
pub use pane_view::pane_tree_view;
pub use search::SearchBarState;
pub use settings_view::settings_panel;
pub use mcp_panel::{mcp_panel, McpPanelState};

// Re-exports for future AI/MCP integration (currently unused)
#[allow(unused_imports)]
pub use mcp_panel::AgentType;
#[allow(unused_imports)]
pub use ai_block::{ai_blocks_view, AiBlock, AiBlockState, AiBlockType, CommandRiskLevel};
