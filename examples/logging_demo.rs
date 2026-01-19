//! Logging System Demo
//!
//! Demonstrates the enhanced logging capabilities of AgTerm.
//!
//! Run with:
//! ```bash
//! # Default info level
//! cargo run --example logging_demo
//!
//! # Debug level via environment
//! AGTERM_LOG_LEVEL=debug cargo run --example logging_demo
//!
//! # Custom log path
//! AGTERM_LOG_PATH=/tmp/demo-logs cargo run --example logging_demo
//!
//! # Module-specific filtering
//! AGTERM_LOG=agterm=trace cargo run --example logging_demo
//! ```

use agterm::logging::{LoggingConfig, init_logging};

fn main() {
    // Initialize logging with default config
    let config = LoggingConfig::default();
    let _log_buffer = init_logging(&config);

    // Log at different levels
    tracing::info!("Application started");
    tracing::debug!("Debug information: system initialized");
    tracing::trace!("Trace information: detailed step-by-step");

    // Structured logging with fields
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        build_type = "demo",
        "AgTerm logging demo"
    );

    // Simulate some operations
    simulate_tab_operations();
    simulate_pty_operations();
    simulate_error_handling();

    tracing::info!("Application completed successfully");
}

fn simulate_tab_operations() {
    tracing::info!("=== Tab Operations Demo ===");

    let tab_id = uuid::Uuid::new_v4();
    tracing::debug!("Creating new tab 'Terminal 1' with ID: {}", tab_id);
    tracing::info!("Tab 'Terminal 1' created successfully with ID: {}", tab_id);

    tracing::info!("Switching to tab 0");
    tracing::debug!("Tab 0 activated");

    tracing::info!("Closing tab at index 0");
    tracing::debug!("Active tab updated to index 0");
}

fn simulate_pty_operations() {
    tracing::info!("=== PTY Operations Demo ===");

    let pane_id = uuid::Uuid::new_v4();
    let session_id = uuid::Uuid::new_v4();

    tracing::debug!("Creating new pane with ID: {}", pane_id);
    tracing::info!(
        "Created PTY session {} for pane {} (attempt 1)",
        session_id,
        pane_id
    );
    tracing::debug!("Starting adaptive PTY polling thread for pane {}", pane_id);

    tracing::info!("PTY session created successfully");

    tracing::info!("Closing PTY session");
    tracing::debug!("Cleaning up PTY session {} for pane {}", session_id, pane_id);
    tracing::info!("Closed PTY session {} for pane {}", session_id, pane_id);
}

fn simulate_error_handling() {
    tracing::info!("=== Error Handling Demo ===");

    tracing::warn!("Non-critical warning: using fallback behavior");
    tracing::error!("Simulated error: operation failed");

    // Structured error logging
    tracing::error!(
        error_code = 500,
        retry_count = 3,
        "Failed to complete operation after retries"
    );
}
