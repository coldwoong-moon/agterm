//! Session Model
//!
//! Core data structures for session and archive management.

use crate::domain::task::TaskGraph;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Session ID type
pub type SessionId = Uuid;

/// MCP connection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConnectionState {
    /// Server name
    pub name: String,
    /// Whether connected
    pub connected: bool,
    /// Available tools count
    pub tools_count: usize,
    /// Last error (if any)
    pub last_error: Option<String>,
}

/// Session - a single AgTerm execution unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session ID
    pub id: SessionId,
    /// Working directory
    pub working_dir: PathBuf,
    /// Session start time
    pub started_at: DateTime<Utc>,
    /// Session end time (if ended)
    pub ended_at: Option<DateTime<Utc>>,
    /// Session status
    pub status: SessionStatus,
    /// Task graph (serializable representation)
    #[serde(skip)]
    pub task_graph: Option<TaskGraph>,
    /// MCP connection states
    pub mcp_connections: Vec<McpConnectionState>,
    /// Custom metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    /// Session is active
    Active,
    /// Session completed normally
    Completed,
    /// Session crashed
    Crashed,
}

impl Session {
    /// Create a new session
    pub fn new(working_dir: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            working_dir,
            started_at: Utc::now(),
            ended_at: None,
            status: SessionStatus::Active,
            task_graph: Some(TaskGraph::new()),
            mcp_connections: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// End the session
    pub fn end(&mut self) {
        self.ended_at = Some(Utc::now());
        self.status = SessionStatus::Completed;
    }

    /// Mark session as crashed
    pub fn mark_crashed(&mut self) {
        self.ended_at = Some(Utc::now());
        self.status = SessionStatus::Crashed;
    }

    /// Get session duration
    pub fn duration(&self) -> chrono::Duration {
        let end = self.ended_at.unwrap_or_else(Utc::now);
        end - self.started_at
    }

    /// Check if session is active
    pub fn is_active(&self) -> bool {
        self.status == SessionStatus::Active
    }
}

/// Session metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionMetrics {
    /// Total number of tasks
    pub total_tasks: usize,
    /// Successfully completed tasks
    pub successful_tasks: usize,
    /// Failed tasks
    pub failed_tasks: usize,
    /// Cancelled tasks
    pub cancelled_tasks: usize,
    /// Total duration in seconds
    pub total_duration_secs: f64,
    /// Average task duration in seconds
    pub avg_task_duration_secs: f64,
    /// Maximum parallelism (concurrent tasks)
    pub max_parallelism: usize,
    /// Total output bytes
    pub total_output_bytes: usize,
    /// Compressed output bytes
    pub compressed_output_bytes: usize,
}

impl SessionMetrics {
    /// Calculate compression ratio
    pub fn compression_ratio(&self) -> f64 {
        if self.total_output_bytes == 0 {
            1.0
        } else {
            self.compressed_output_bytes as f64 / self.total_output_bytes as f64
        }
    }
}

/// Compression level for archives
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionLevel {
    /// Raw output (recent sessions)
    Raw,
    /// Compacted (references only)
    Compacted,
    /// AI summarized
    Summarized,
    /// Hierarchical rolling summary
    Rolled,
}

impl Default for CompressionLevel {
    fn default() -> Self {
        Self::Raw
    }
}

/// Session archive - stored session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionArchive {
    /// Archive ID
    pub id: Uuid,
    /// Original session ID
    pub session_id: SessionId,
    /// Working directory
    pub working_dir: PathBuf,
    /// Time period (start, end)
    pub period: (DateTime<Utc>, DateTime<Utc>),
    /// AI-generated summary
    pub summary: String,
    /// Search tags
    pub tags: Vec<String>,
    /// Session metrics
    pub metrics: SessionMetrics,
    /// Compression level
    pub compression_level: CompressionLevel,
    /// Archive creation time
    pub created_at: DateTime<Utc>,
}

impl SessionArchive {
    /// Create a new archive from a session
    pub fn from_session(session: &Session, summary: String, tags: Vec<String>) -> Self {
        let end_time = session.ended_at.unwrap_or_else(Utc::now);

        Self {
            id: Uuid::new_v4(),
            session_id: session.id,
            working_dir: session.working_dir.clone(),
            period: (session.started_at, end_time),
            summary,
            tags,
            metrics: SessionMetrics::default(),
            compression_level: CompressionLevel::Raw,
            created_at: Utc::now(),
        }
    }

    /// Set metrics
    pub fn with_metrics(mut self, metrics: SessionMetrics) -> Self {
        self.metrics = metrics;
        self
    }

    /// Set compression level
    pub fn with_compression_level(mut self, level: CompressionLevel) -> Self {
        self.compression_level = level;
        self
    }

    /// Get duration in seconds
    pub fn duration_secs(&self) -> f64 {
        (self.period.1 - self.period.0).num_milliseconds() as f64 / 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new(PathBuf::from("/tmp/test"));

        assert!(session.is_active());
        assert!(session.ended_at.is_none());
        assert_eq!(session.working_dir, PathBuf::from("/tmp/test"));
    }

    #[test]
    fn test_session_end() {
        let mut session = Session::new(PathBuf::from("/tmp/test"));
        session.end();

        assert!(!session.is_active());
        assert!(session.ended_at.is_some());
        assert_eq!(session.status, SessionStatus::Completed);
    }

    #[test]
    fn test_archive_creation() {
        let session = Session::new(PathBuf::from("/tmp/test"));
        let archive = SessionArchive::from_session(
            &session,
            "Test summary".to_string(),
            vec!["test".to_string()],
        );

        assert_eq!(archive.session_id, session.id);
        assert_eq!(archive.summary, "Test summary");
        assert_eq!(archive.tags, vec!["test"]);
    }

    #[test]
    fn test_compression_ratio() {
        let metrics = SessionMetrics {
            total_output_bytes: 1000,
            compressed_output_bytes: 250,
            ..Default::default()
        };

        assert_eq!(metrics.compression_ratio(), 0.25);
    }
}
