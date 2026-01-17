//! Application Layer
//!
//! Contains application-level components including configuration,
//! state management, and the main event loop.

pub mod config;
pub mod event_loop;
pub mod logging;
pub mod state;

pub use config::AppConfig;
pub use event_loop::App;
pub use state::AppState;
