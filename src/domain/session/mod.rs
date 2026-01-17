//! Session Management
//!
//! Session lifecycle and archiving.

pub mod model;

pub use model::{
    CompressionLevel, McpConnectionState, Session, SessionArchive, SessionId, SessionMetrics,
    SessionStatus,
};
