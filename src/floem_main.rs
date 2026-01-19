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
//!
//! # Run as MCP server (for AI agent integration)
//! cargo run --features floem-gui -- --mcp-server
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
//! - **MCP Server mode** for AI agent control
//!
//! # Architecture
//!
//! The application follows this structure:
//! - **floem_app::app_view**: Main application view composition
//! - **floem_app::state**: Global application state and tab management
//! - **floem_app::views**: UI components (terminal, tab bar, status bar, panes)
//! - **floem_app::theme**: Theme system with color palettes
//! - **floem_app::settings**: Persistent configuration management
//! - **mcp_server**: MCP protocol server for AI agent orchestration
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
//!
//! # MCP Server Mode
//!
//! Run AgTerm as an MCP server for AI agents like Claude Code:
//! ```bash
//! agterm --mcp-server
//! ```
//!
//! Available MCP tools:
//! - `create_session`: Create a new terminal session
//! - `run_command`: Execute a command in a session
//! - `get_output`: Get output from a session
//! - `list_sessions`: List all active sessions
//! - `close_session`: Close a terminal session
//! - `resize_session`: Resize a terminal session

fn main() {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let mcp_server_mode = args.iter().any(|arg| arg == "--mcp-server");
    let test_server_mode = args.iter().any(|arg| arg == "--test-server");

    // Initialize logging system with default configuration
    let log_config = agterm::logging::LoggingConfig::default();
    agterm::logging::init_logging(&log_config);

    if mcp_server_mode {
        // MCP server mode is always headless (no GUI)
        // Use open_gui MCP tool to launch GUI when needed
        tracing::info!("Starting AgTerm in headless MCP server mode");
        run_mcp_server_headless();
    } else {
        // Run as GUI application
        run_gui(test_server_mode);
    }
}

fn run_gui(test_server: bool) {
    // Create Tokio runtime for async operations (MCP, etc.)
    let rt = tokio::runtime::Runtime::new()
        .expect("Failed to create Tokio runtime");
    let _guard = rt.enter();

    tracing::info!("Starting AgTerm (Floem GUI) with Tokio runtime");

    // Start test server if requested
    if test_server {
        tracing::info!("Starting UI test server on port {}", agterm::floem_app::test_server::TEST_SERVER_PORT);
        let test_srv = agterm::floem_app::test_server::TestServer::new();
        if let Err(e) = test_srv.start() {
            tracing::error!("Failed to start test server: {}", e);
        }
        // Note: AppState will be set after app_view initializes
    }

    // Configure window with appropriate size
    let window_config = floem::window::WindowConfig::default()
        .size(floem::kurbo::Size::new(1200.0, 800.0))
        .title("AgTerm")
        .resizable(true);

    // Launch the Floem application with window configuration
    floem::Application::new()
        .window(|_| agterm::floem_app::app_view(), Some(window_config))
        .run();
}

fn run_mcp_server_headless() {
    // Create Tokio runtime for async MCP server
    let rt = tokio::runtime::Runtime::new()
        .expect("Failed to create Tokio runtime");

    rt.block_on(async {
        let server = agterm::mcp_server::StandaloneMcpServer::new();
        server.run().await;
    });
}
