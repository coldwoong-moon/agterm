//! TUI Widgets
//!
//! Custom ratatui widgets for AgTerm.

pub mod archive_browser;
pub mod graph_view;
pub mod status_bar;
pub mod task_detail;
pub mod task_tree;
pub mod terminal_pane;

pub use archive_browser::{ArchiveBrowser, ArchiveBrowserState, CompressionIndicator};
pub use graph_view::{GraphView, TaskProgressBar};
pub use status_bar::StatusBar;
pub use task_detail::{centered_rect, TaskDetail, TaskEta, TaskTimer};
pub use task_tree::{StatefulTaskTree, TaskTree};
pub use terminal_pane::{SimpleTerminalOutput, TerminalPane};
