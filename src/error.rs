//! `AgTerm` Error Types
//!
//! Centralized error handling using thiserror for type-safe errors.

use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;

/// Top-level error type for `AgTerm`
#[derive(Error, Debug)]
pub enum AgTermError {
    #[error("PTY error: {0}")]
    Pty(#[from] PtyError),

    #[error("MCP error: {0}")]
    Mcp(#[from] McpError),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Task error: {0}")]
    Task(#[from] TaskError),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("TUI error: {0}")]
    Tui(#[from] TuiError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// PTY-related errors
#[derive(Error, Debug)]
pub enum PtyError {
    #[error("Failed to spawn PTY: {reason}")]
    SpawnFailed { reason: String },

    #[error("PTY pool exhausted (max: {max}, current: {current})")]
    PoolExhausted { max: usize, current: usize },

    #[error("PTY with id '{id}' not found")]
    NotFound { id: String },

    #[error("PTY I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("PTY resize failed: {reason}")]
    ResizeFailed { reason: String },

    #[error("PTY already closed")]
    AlreadyClosed,
}

/// MCP (Model Context Protocol) errors
#[derive(Error, Debug)]
pub enum McpError {
    #[error("Connection failed to server '{server}': {reason}")]
    ConnectionFailed { server: String, reason: String },

    #[error("Tool call failed: {tool} - {error}")]
    ToolCallFailed { tool: String, error: String },

    #[error("Server '{server}' not responding (timeout: {timeout_secs}s)")]
    Timeout { server: String, timeout_secs: u64 },

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Server '{server}' not found in registry")]
    ServerNotFound { server: String },

    #[error("Invalid MCP message: {0}")]
    InvalidMessage(String),

    #[error("Authentication failed for server '{server}'")]
    AuthFailed { server: String },
}

/// Storage and archive errors
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Archive not found: {id}")]
    ArchiveNotFound { id: Uuid },

    #[error("Session not found: {id}")]
    SessionNotFound { id: Uuid },

    #[error("Failed to read file '{path}': {reason}")]
    FileReadFailed { path: PathBuf, reason: String },

    #[error("Failed to write file '{path}': {reason}")]
    FileWriteFailed { path: PathBuf, reason: String },

    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

/// Task orchestration errors
#[derive(Error, Debug)]
pub enum TaskError {
    #[error("Task '{id}' failed with exit code {exit_code}")]
    ExecutionFailed { id: Uuid, exit_code: i32 },

    #[error("Task '{id}' timed out after {timeout_secs}s")]
    Timeout { id: Uuid, timeout_secs: u64 },

    #[error("Circular dependency detected: {cycle:?}")]
    CircularDependency { cycle: Vec<Uuid> },

    #[error("Dependency '{dep_id}' failed, cannot start task '{task_id}'")]
    DependencyFailed { task_id: Uuid, dep_id: Uuid },

    #[error("Task '{id}' not found")]
    NotFound { id: Uuid },

    #[error("Task '{id}' is already running")]
    AlreadyRunning { id: Uuid },

    #[error("Task '{id}' cannot be cancelled in state '{state}'")]
    CannotCancel { id: Uuid, state: String },

    #[error("Invalid task state transition: {from} -> {to}")]
    InvalidStateTransition { from: String, to: String },
}

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: PathBuf },

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Failed to parse configuration: {0}")]
    ParseError(String),

    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Environment variable '{var}' not set")]
    EnvVarNotSet { var: String },

    #[error("Config library error: {0}")]
    ConfigLib(#[from] config::ConfigError),
}

/// TUI rendering errors
#[derive(Error, Debug)]
pub enum TuiError {
    #[error("Terminal initialization failed: {0}")]
    InitFailed(String),

    #[error("Terminal restoration failed: {0}")]
    RestoreFailed(String),

    #[error("Rendering error: {0}")]
    RenderError(String),

    #[error("Input handling error: {0}")]
    InputError(String),

    #[error("Crossterm error: {0}")]
    Crossterm(#[from] std::io::Error),
}

/// Result type alias for `AgTerm` operations
pub type Result<T> = std::result::Result<T, AgTermError>;

/// Result type alias for PTY operations
pub type PtyResult<T> = std::result::Result<T, PtyError>;

/// Result type alias for MCP operations
pub type McpResult<T> = std::result::Result<T, McpError>;

/// Result type alias for Storage operations
pub type StorageResult<T> = std::result::Result<T, StorageError>;

/// Result type alias for Task operations
pub type TaskResult<T> = std::result::Result<T, TaskError>;

/// Result type alias for Config operations
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = PtyError::PoolExhausted {
            max: 32,
            current: 32,
        };
        assert_eq!(err.to_string(), "PTY pool exhausted (max: 32, current: 32)");
    }

    #[test]
    fn test_error_conversion() {
        let pty_err = PtyError::SpawnFailed {
            reason: "permission denied".to_string(),
        };
        let agterm_err: AgTermError = pty_err.into();
        assert!(matches!(agterm_err, AgTermError::Pty(_)));
    }
}
