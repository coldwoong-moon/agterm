//! Presentation Layer (TUI)
//!
//! Terminal user interface components and widgets.

pub mod keybindings;
pub mod layout;
pub mod theme;
pub mod tui;
pub mod widgets;

pub use keybindings::{Action, KeyCombination, Keybindings};
pub use layout::{ComputedLayout, LayoutManager, LayoutNode, NavigateDirection, SplitDirection};
pub use theme::{Theme, ThemeColor, ThemeColors, ThemeManager};
pub use tui::{init, install_panic_hook, restore, Tui};
pub use widgets::{
    centered_rect, ArchiveBrowser, ArchiveBrowserState, CompressionIndicator, GraphView,
    SimpleTerminalOutput, StatefulTaskTree, StatusBar, TaskDetail, TaskEta, TaskProgressBar,
    TaskTimer, TaskTree, TerminalPane,
};
