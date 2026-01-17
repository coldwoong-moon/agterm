//! `AgTerm` - AI Agent Terminal Orchestrator
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
//! `AgTerm` follows a layered architecture:
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

// TODO: Re-enable strict lints after initial release
// #![warn(missing_docs)]
#![allow(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::let_and_return)]
#![allow(clippy::question_mark)]
#![allow(clippy::match_like_matches_macro)]
#![allow(clippy::wrong_self_convention)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::similar_names)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::fn_params_excessive_bools)]
#![allow(clippy::if_not_else)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::only_used_in_recursion)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::format_push_string)]
#![allow(clippy::match_wildcard_for_single_variants)]
#![allow(clippy::unused_self)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::float_cmp)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::missing_fields_in_debug)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::manual_flatten)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(deprecated)]
#![allow(dead_code)] // Allow during development
#![allow(unused_imports)]
#![allow(unused_variables)]

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
