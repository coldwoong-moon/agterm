//! AgTerm - Floem GUI Entry Point
//!
//! This module serves as the main entry point for the Floem-based GPU-accelerated GUI.
//!
//! # Building and Running
//!
//! ```bash
//! # Build the Floem GUI binary
//! cargo build --bin agterm-floem --features floem-gui --no-default-features
//!
//! # Run with debug logging
//! AGTERM_LOG=agterm=debug cargo run --bin agterm-floem --features floem-gui --no-default-features
//!
//! # Release build with optimizations
//! cargo build --release --bin agterm-floem --features floem-gui --no-default-features
//! ```
//!
//! # Features
//!
//! The Floem GUI provides:
//! - **GPU-accelerated rendering** via wgpu
//! - **Reactive state management** with Floem's signal system
//! - **Tab system** with independent PTY sessions
//! - **Pane splitting** (horizontal/vertical)
//! - **Theme support** (Dark/Light)
//! - **Persistent settings** (font size, theme, shell)
//! - **Keyboard shortcuts** for productivity
//! - **IME support** for international input
//!
//! # Architecture
//!
//! The application follows this structure:
//! - **floem_app::app_view**: Main application view composition
//! - **floem_app::state**: Global application state and tab management
//! - **floem_app::views**: UI components (terminal, tab bar, status bar, panes)
//! - **floem_app::theme**: Theme system with color palettes
//! - **floem_app::settings**: Persistent configuration management
//!
//! # Configuration
//!
//! Settings are stored in `~/.config/agterm/config.toml`:
//! ```toml
//! font_size = 14.0
//! theme_name = "Ghostty Dark"
//! shell = "/bin/zsh"
//! default_cols = 80
//! default_rows = 24
//! ```
//!
//! # Keyboard Shortcuts
//!
//! - **Cmd +**: Increase font size
//! - **Cmd -**: Decrease font size
//! - **Cmd T**: Toggle theme (Dark/Light)
//! - **Ctrl+Shift+D**: Split pane vertically
//! - **Ctrl+Shift+E**: Split pane horizontally
//! - **Ctrl+Shift+W**: Close focused pane
//! - **Ctrl+Tab**: Navigate to next pane
//! - **Ctrl+Shift+Tab**: Navigate to previous pane

fn main() {
    // Initialize logging system with default configuration
    let log_config = agterm::logging::LoggingConfig::default();
    agterm::logging::init_logging(&log_config);

    // Create Tokio runtime for async operations (MCP, etc.)
    let rt = tokio::runtime::Runtime::new()
        .expect("Failed to create Tokio runtime");
    let _guard = rt.enter();

    tracing::info!("Starting AgTerm (Floem GUI) with Tokio runtime");

    // Launch the Floem application
    // This is a blocking call that runs the event loop
    floem::launch(agterm::floem_app::app_view);
}
