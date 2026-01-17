//! AgTerm - AI Agent Terminal Orchestrator
//!
//! A next-generation terminal emulator designed for AI agent workflows.
//! Features include:
//! - Tree-based task branching and parallel execution
//! - Real-time visualization of task progress
//! - MCP (Model Context Protocol) native support
//! - Intelligent session archiving with AI summarization
//!
//! # Architecture
//!
//! AgTerm follows a layered architecture:
//!
//! ```text
//! Presentation Layer (TUI)
//!         ↓
//! Application Layer (State, Config)
//!         ↓
//! Domain Layer (Task, Session, Memory, Event)
//!         ↓
//! Infrastructure Layer (PTY, MCP, Storage)
//! ```
//!
//! # Quick Start
//!
//! ```no_run
//! use agterm::app::{AppConfig, AppState};
//! use agterm::app::logging;
//!
//! #[tokio::main]
//! async fn main() -> agterm::error::Result<()> {
//!     // Load configuration
//!     let config = AppConfig::load()?;
//!
//!     // Initialize logging
//!     logging::init_logging(&config.logging)?;
//!
//!     // Create application state
//!     let state = AppState::new(config)?;
//!
//!     // ... run application
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod app;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod presentation;

// Re-exports for convenience
pub use app::{AppConfig, AppState};
pub use error::{AgTermError, Result};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
