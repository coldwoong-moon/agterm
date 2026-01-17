//! Task Model
//!
//! Core data structures for task management.

use crate::infrastructure::pty::PtyId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Task identifier
pub type TaskId = Uuid;

/// Task execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is waiting to be executed
    Pending,
    /// Task is waiting for dependencies to complete
    Blocked,
    /// Task is currently running
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed with an error
    Failed,
    /// Task was cancelled
    Cancelled,
    /// Task was skipped (due to dependency failure)
    Skipped,
}

impl TaskStatus {
    /// Check if the task is in a terminal state
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            TaskStatus::Completed
                | TaskStatus::Failed
                | TaskStatus::Cancelled
                | TaskStatus::Skipped
        )
    }

    /// Check if the task is runnable
    #[must_use]
    pub fn is_runnable(&self) -> bool {
        matches!(self, TaskStatus::Pending)
    }

    /// Check if the task is active (running or blocked)
    #[must_use]
    pub fn is_active(&self) -> bool {
        matches!(self, TaskStatus::Running | TaskStatus::Blocked)
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "⏳ Pending"),
            TaskStatus::Blocked => write!(f, "🔒 Blocked"),
            TaskStatus::Running => write!(f, "▶️ Running"),
            TaskStatus::Completed => write!(f, "✅ Completed"),
            TaskStatus::Failed => write!(f, "❌ Failed"),
            TaskStatus::Cancelled => write!(f, "⏹️ Cancelled"),
            TaskStatus::Skipped => write!(f, "⏭️ Skipped"),
        }
    }
}

/// Task execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Exit code (0 = success)
    pub exit_code: i32,
    /// Stdout output (possibly truncated)
    pub stdout: String,
    /// Stderr output (possibly truncated)
    pub stderr: String,
    /// Full output log path
    pub log_path: Option<PathBuf>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
}

impl TaskResult {
    /// Create a successful result
    #[must_use]
    pub fn success(stdout: String, duration_ms: u64) -> Self {
        Self {
            exit_code: 0,
            stdout,
            stderr: String::new(),
            log_path: None,
            duration_ms,
        }
    }

    /// Create a failed result
    #[must_use]
    pub fn failure(exit_code: i32, stderr: String, duration_ms: u64) -> Self {
        Self {
            exit_code,
            stdout: String::new(),
            stderr,
            log_path: None,
            duration_ms,
        }
    }

    /// Check if the result indicates success
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }
}

/// Error propagation policy for task dependencies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ErrorPolicy {
    /// Stop all dependent tasks on failure
    #[default]
    StopOnError,
    /// Continue with other tasks, skip dependents
    ContinueOnError,
    /// Retry the failed task N times before stopping
    RetryThenStop { max_retries: u32 },
}

/// Task node in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNode {
    /// Unique task identifier
    pub id: TaskId,
    /// Human-readable task name
    pub name: String,
    /// Command to execute
    pub command: String,
    /// Arguments
    pub args: Vec<String>,
    /// Working directory
    pub working_dir: PathBuf,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Current status
    pub status: TaskStatus,
    /// Execution result (if completed)
    pub result: Option<TaskResult>,
    /// Error policy
    pub error_policy: ErrorPolicy,
    /// Associated PTY session ID (when running)
    pub pty_id: Option<PtyId>,
    /// Parent task ID (for tree structure)
    pub parent_id: Option<TaskId>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Start timestamp (when execution began)
    pub started_at: Option<DateTime<Utc>>,
    /// Completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Number of retry attempts
    pub retry_count: u32,
    /// User-defined metadata
    pub metadata: HashMap<String, String>,
}

impl TaskNode {
    /// Create a new task
    pub fn new(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            command: command.into(),
            args: Vec::new(),
            working_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            env: HashMap::new(),
            status: TaskStatus::Pending,
            result: None,
            error_policy: ErrorPolicy::default(),
            pty_id: None,
            parent_id: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            retry_count: 0,
            metadata: HashMap::new(),
        }
    }

    /// Builder: set arguments
    #[must_use]
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Builder: set working directory
    #[must_use]
    pub fn with_working_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = dir;
        self
    }

    /// Builder: set environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Builder: set error policy
    #[must_use]
    pub fn with_error_policy(mut self, policy: ErrorPolicy) -> Self {
        self.error_policy = policy;
        self
    }

    /// Builder: set parent task
    #[must_use]
    pub fn with_parent(mut self, parent_id: TaskId) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Builder: set metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Mark task as running
    pub fn start(&mut self, pty_id: PtyId) {
        self.status = TaskStatus::Running;
        self.pty_id = Some(pty_id);
        self.started_at = Some(Utc::now());
    }

    /// Mark task as completed
    pub fn complete(&mut self, result: TaskResult) {
        self.status = if result.is_success() {
            TaskStatus::Completed
        } else {
            TaskStatus::Failed
        };
        self.result = Some(result);
        self.completed_at = Some(Utc::now());
    }

    /// Mark task as cancelled
    pub fn cancel(&mut self) {
        self.status = TaskStatus::Cancelled;
        self.completed_at = Some(Utc::now());
    }

    /// Mark task as skipped
    pub fn skip(&mut self) {
        self.status = TaskStatus::Skipped;
        self.completed_at = Some(Utc::now());
    }

    /// Mark task as blocked
    pub fn block(&mut self) {
        self.status = TaskStatus::Blocked;
    }

    /// Unblock task (set to pending)
    pub fn unblock(&mut self) {
        if self.status == TaskStatus::Blocked {
            self.status = TaskStatus::Pending;
        }
    }

    /// Get execution duration in milliseconds
    #[must_use]
    pub fn duration_ms(&self) -> Option<u64> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => {
                let duration = end.signed_duration_since(start);
                Some(duration.num_milliseconds().max(0) as u64)
            }
            (Some(start), None) => {
                let duration = Utc::now().signed_duration_since(start);
                Some(duration.num_milliseconds().max(0) as u64)
            }
            _ => None,
        }
    }

    /// Format duration as human-readable string
    #[must_use]
    pub fn duration_str(&self) -> String {
        match self.duration_ms() {
            Some(ms) if ms < 1000 => format!("{ms}ms"),
            Some(ms) if ms < 60000 => format!("{:.1}s", ms as f64 / 1000.0),
            Some(ms) => format!("{}m {}s", ms / 60000, (ms % 60000) / 1000),
            None => "N/A".to_string(),
        }
    }

    /// Get the full command string
    #[must_use]
    pub fn full_command(&self) -> String {
        if self.args.is_empty() {
            self.command.clone()
        } else {
            format!("{} {}", self.command, self.args.join(" "))
        }
    }
}

/// Edge type in the task graph (dependency relationship)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TaskEdge {
    /// Standard dependency (target waits for source)
    #[default]
    DependsOn,
    /// Soft dependency (target can proceed even if source fails)
    SoftDependsOn,
    /// Parent-child relationship (for tree visualization)
    ParentOf,
}

impl std::fmt::Display for TaskEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskEdge::DependsOn => write!(f, "depends on"),
            TaskEdge::SoftDependsOn => write!(f, "soft depends on"),
            TaskEdge::ParentOf => write!(f, "parent of"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = TaskNode::new("Build", "npm run build");
        assert_eq!(task.name, "Build");
        assert_eq!(task.command, "npm run build");
        assert_eq!(task.status, TaskStatus::Pending);
        assert!(task.result.is_none());
    }

    #[test]
    fn test_task_builder() {
        let task = TaskNode::new("Test", "cargo test")
            .with_args(vec!["--release".to_string()])
            .with_env("RUST_LOG", "debug")
            .with_error_policy(ErrorPolicy::ContinueOnError);

        assert_eq!(task.args, vec!["--release"]);
        assert_eq!(task.env.get("RUST_LOG"), Some(&"debug".to_string()));
        assert_eq!(task.error_policy, ErrorPolicy::ContinueOnError);
    }

    #[test]
    fn test_task_lifecycle() {
        let mut task = TaskNode::new("Test", "echo hello");
        let pty_id = Uuid::new_v4();

        // Start
        task.start(pty_id);
        assert_eq!(task.status, TaskStatus::Running);
        assert_eq!(task.pty_id, Some(pty_id));
        assert!(task.started_at.is_some());

        // Complete
        let result = TaskResult::success("hello".to_string(), 100);
        task.complete(result);
        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.completed_at.is_some());
        assert!(task.result.is_some());
    }

    #[test]
    fn test_status_checks() {
        assert!(TaskStatus::Completed.is_terminal());
        assert!(TaskStatus::Failed.is_terminal());
        assert!(!TaskStatus::Running.is_terminal());

        assert!(TaskStatus::Pending.is_runnable());
        assert!(!TaskStatus::Blocked.is_runnable());

        assert!(TaskStatus::Running.is_active());
        assert!(TaskStatus::Blocked.is_active());
    }
}
