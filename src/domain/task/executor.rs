//! Task Executor
//!
//! Bridges the task scheduler with PTY pool for task execution.

use super::graph::TaskGraph;
use super::model::{TaskId, TaskResult};
use super::scheduler::{SchedulerConfig, SchedulerEvent, TaskScheduler};
use crate::error::{TaskError, TaskResult as TResult};
use crate::infrastructure::pty::{PtyId, PtyPool, PtyPoolConfig};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;
use tokio::sync::mpsc;

/// Task execution events
#[derive(Debug, Clone)]
pub enum ExecutorEvent {
    /// Task started execution
    TaskStarted {
        task_id: TaskId,
        pty_id: PtyId,
    },
    /// Task output received
    TaskOutput {
        task_id: TaskId,
        data: String,
    },
    /// Task completed
    TaskCompleted {
        task_id: TaskId,
        result: TaskResult,
    },
    /// Task failed
    TaskFailed {
        task_id: TaskId,
        error: String,
    },
    /// Task was skipped
    TaskSkipped {
        task_id: TaskId,
    },
    /// All tasks complete
    AllComplete {
        total: usize,
        completed: usize,
        failed: usize,
    },
    /// Progress update
    Progress {
        completed: usize,
        total: usize,
        running: usize,
    },
    /// Scheduler event passthrough
    Scheduler(SchedulerEvent),
}

/// Task executor configuration
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Maximum concurrent tasks
    pub max_concurrent: usize,
    /// Working directory for tasks
    pub working_dir: PathBuf,
    /// Shell to use for command execution
    pub shell: String,
    /// Capture task output
    pub capture_output: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 4,
            working_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            shell: std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string()),
            capture_output: true,
        }
    }
}

/// Running task information
#[derive(Debug)]
struct RunningTask {
    task_id: TaskId,
    pty_id: PtyId,
    start_time: Instant,
    output: String,
}

/// Task Executor
///
/// Manages task execution by coordinating between the scheduler and PTY pool.
pub struct TaskExecutor {
    /// The task scheduler
    scheduler: TaskScheduler,
    /// PTY pool for running tasks
    pty_pool: PtyPool,
    /// Executor configuration
    config: ExecutorConfig,
    /// Running tasks (pty_id -> running task info)
    running_tasks: HashMap<PtyId, RunningTask>,
    /// Reverse lookup (task_id -> pty_id)
    task_to_pty: HashMap<TaskId, PtyId>,
    /// Event sender
    event_tx: Option<mpsc::UnboundedSender<ExecutorEvent>>,
}

impl TaskExecutor {
    /// Create a new executor
    pub fn new(graph: TaskGraph, config: ExecutorConfig) -> Self {
        let scheduler_config = SchedulerConfig {
            max_concurrent: config.max_concurrent,
            ..Default::default()
        };
        let pty_config = PtyPoolConfig {
            max_sessions: config.max_concurrent + 2, // Allow some buffer
            ..Default::default()
        };

        Self {
            scheduler: TaskScheduler::new(graph, scheduler_config),
            pty_pool: PtyPool::new(pty_config),
            config,
            running_tasks: HashMap::new(),
            task_to_pty: HashMap::new(),
            event_tx: None,
        }
    }

    /// Create executor with event channel
    pub fn with_events(graph: TaskGraph, config: ExecutorConfig) -> (Self, mpsc::UnboundedReceiver<ExecutorEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut executor = Self::new(graph, config);
        executor.event_tx = Some(tx);
        (executor, rx)
    }

    /// Get reference to the scheduler
    pub fn scheduler(&self) -> &TaskScheduler {
        &self.scheduler
    }

    /// Get mutable reference to the scheduler
    pub fn scheduler_mut(&mut self) -> &mut TaskScheduler {
        &mut self.scheduler
    }

    /// Get reference to the PTY pool
    pub fn pty_pool(&self) -> &PtyPool {
        &self.pty_pool
    }

    /// Get mutable reference to the PTY pool
    pub fn pty_pool_mut(&mut self) -> &mut PtyPool {
        &mut self.pty_pool
    }

    /// Get running task count
    pub fn running_count(&self) -> usize {
        self.running_tasks.len()
    }

    /// Check if execution is complete
    pub fn is_complete(&self) -> bool {
        self.scheduler.is_complete()
    }

    /// Send event if channel is set
    fn send_event(&self, event: ExecutorEvent) {
        if let Some(ref tx) = self.event_tx {
            let _ = tx.send(event);
        }
    }

    /// Start execution
    ///
    /// Returns initial events (tasks ready to start).
    pub fn start(&mut self) -> Vec<ExecutorEvent> {
        let scheduler_events = self.scheduler.start();
        let mut events = Vec::new();

        for event in scheduler_events {
            match event {
                SchedulerEvent::TaskReady { task_id, command, args } => {
                    // Start the task
                    match self.start_task(task_id, &command, &args) {
                        Ok(pty_id) => {
                            events.push(ExecutorEvent::TaskStarted { task_id, pty_id });
                        }
                        Err(e) => {
                            events.push(ExecutorEvent::TaskFailed {
                                task_id,
                                error: e.to_string(),
                            });
                        }
                    }
                }
                SchedulerEvent::Progress { completed, total, running } => {
                    events.push(ExecutorEvent::Progress { completed, total, running });
                }
                other => {
                    events.push(ExecutorEvent::Scheduler(other));
                }
            }
        }

        // Send events
        for event in &events {
            self.send_event(event.clone());
        }

        events
    }

    /// Start a specific task
    fn start_task(&mut self, task_id: TaskId, command: &str, args: &[String]) -> TResult<PtyId> {
        // Build the full command
        let full_command = if args.is_empty() {
            command.to_string()
        } else {
            format!("{} {}", command, args.join(" "))
        };

        // Build shell command
        let shell_args = vec!["-c".to_string(), full_command];

        // Spawn PTY using sync wrapper
        let pty_id = self.pty_pool.spawn_with_command_sync(
            &self.config.shell,
            &shell_args,
            &self.config.working_dir,
        ).map_err(|_e| TaskError::ExecutionFailed {
            id: task_id,
            exit_code: -1,
        })?;

        // Register with scheduler
        self.scheduler.task_started(task_id, pty_id)?;

        // Track running task
        let running = RunningTask {
            task_id,
            pty_id,
            start_time: Instant::now(),
            output: String::new(),
        };
        self.running_tasks.insert(pty_id, running);
        self.task_to_pty.insert(task_id, pty_id);

        Ok(pty_id)
    }

    /// Poll for task completions and process events
    ///
    /// This should be called periodically to check for task completions.
    pub async fn poll(&mut self) -> Vec<ExecutorEvent> {
        let mut events = Vec::new();

        // Read output from all PTYs
        let outputs = self.pty_pool.read_all_outputs().await;

        // Process outputs
        for (pty_id, data) in outputs {
            if let Some(running) = self.running_tasks.get_mut(&pty_id) {
                // Append output
                running.output.push_str(&data);

                // Send output event
                events.push(ExecutorEvent::TaskOutput {
                    task_id: running.task_id,
                    data,
                });
            }
        }

        // Send events
        for event in &events {
            self.send_event(event.clone());
        }

        events
    }

    /// Mark a task as completed (called externally when PTY exits)
    pub fn task_exited(&mut self, pty_id: PtyId, exit_code: i32) -> Vec<ExecutorEvent> {
        let mut events = Vec::new();

        if let Some(running) = self.running_tasks.remove(&pty_id) {
            self.task_to_pty.remove(&running.task_id);

            let duration_ms = running.start_time.elapsed().as_millis() as u64;
            let result = if exit_code == 0 {
                TaskResult::success(running.output.clone(), duration_ms)
            } else {
                TaskResult::failure(exit_code, running.output.clone(), duration_ms)
            };

            // Notify scheduler
            if let Ok(scheduler_events) = self.scheduler.task_completed(running.task_id, result.clone()) {
                events.extend(self.process_scheduler_events(scheduler_events));
            }

            events.push(ExecutorEvent::TaskCompleted {
                task_id: running.task_id,
                result,
            });

            // Kill the PTY session using sync wrapper
            let _ = self.pty_pool.kill_sync(&pty_id);
        }

        // Send events
        for event in &events {
            self.send_event(event.clone());
        }

        events
    }

    /// Append output to a running task
    pub fn append_output(&mut self, pty_id: &PtyId, data: &str) {
        let task_id = if let Some(running) = self.running_tasks.get_mut(pty_id) {
            running.output.push_str(data);
            Some(running.task_id)
        } else {
            None
        };

        if let Some(task_id) = task_id {
            self.send_event(ExecutorEvent::TaskOutput {
                task_id,
                data: data.to_string(),
            });
        }
    }

    /// Process scheduler events and convert to executor events
    fn process_scheduler_events(&mut self, events: Vec<SchedulerEvent>) -> Vec<ExecutorEvent> {
        let mut result = Vec::new();

        for event in events {
            match event {
                SchedulerEvent::TaskReady { task_id, command, args } => {
                    // Start the task
                    match self.start_task(task_id, &command, &args) {
                        Ok(pty_id) => {
                            result.push(ExecutorEvent::TaskStarted { task_id, pty_id });
                        }
                        Err(e) => {
                            result.push(ExecutorEvent::TaskFailed {
                                task_id,
                                error: e.to_string(),
                            });
                        }
                    }
                }
                SchedulerEvent::TaskSkipped { task_id } => {
                    result.push(ExecutorEvent::TaskSkipped { task_id });
                }
                SchedulerEvent::AllComplete { stats } => {
                    result.push(ExecutorEvent::AllComplete {
                        total: stats.total,
                        completed: stats.completed,
                        failed: stats.failed,
                    });
                }
                SchedulerEvent::Progress { completed, total, running } => {
                    result.push(ExecutorEvent::Progress { completed, total, running });
                }
                other => {
                    result.push(ExecutorEvent::Scheduler(other));
                }
            }
        }

        result
    }

    /// Cancel a specific task
    pub fn cancel_task(&mut self, task_id: TaskId) -> Vec<ExecutorEvent> {
        let mut events = Vec::new();

        // Kill the PTY if running
        if let Some(&pty_id) = self.task_to_pty.get(&task_id) {
            self.running_tasks.remove(&pty_id);
            self.task_to_pty.remove(&task_id);
            let _ = self.pty_pool.kill_sync(&pty_id);
        }

        // Cancel in scheduler
        if let Ok(scheduler_events) = self.scheduler.cancel_task(task_id) {
            events.extend(self.process_scheduler_events(scheduler_events));
        }

        // Send events
        for event in &events {
            self.send_event(event.clone());
        }

        events
    }

    /// Cancel all tasks
    pub fn cancel_all(&mut self) -> Vec<ExecutorEvent> {
        let mut events = Vec::new();

        // Kill all PTYs
        let pty_ids: Vec<_> = self.running_tasks.keys().copied().collect();
        for pty_id in pty_ids {
            let _ = self.pty_pool.kill_sync(&pty_id);
        }
        self.running_tasks.clear();
        self.task_to_pty.clear();

        // Cancel in scheduler
        let scheduler_events = self.scheduler.cancel_all();
        events.extend(self.process_scheduler_events(scheduler_events));

        // Send events
        for event in &events {
            self.send_event(event.clone());
        }

        events
    }

    /// Pause execution (don't start new tasks)
    pub fn pause(&mut self) {
        self.scheduler.pause();
    }

    /// Resume execution
    pub fn resume(&mut self) -> Vec<ExecutorEvent> {
        self.scheduler.resume();

        // Check for newly ready tasks
        let mut events = Vec::new();
        for (task_id, command, args) in self.scheduler.get_next_tasks() {
            match self.start_task(task_id, &command, &args) {
                Ok(pty_id) => {
                    events.push(ExecutorEvent::TaskStarted { task_id, pty_id });
                }
                Err(e) => {
                    events.push(ExecutorEvent::TaskFailed {
                        task_id,
                        error: e.to_string(),
                    });
                }
            }
        }

        // Send events
        for event in &events {
            self.send_event(event.clone());
        }

        events
    }

    /// Get task by PTY ID
    pub fn get_task_by_pty(&self, pty_id: &PtyId) -> Option<TaskId> {
        self.running_tasks.get(pty_id).map(|r| r.task_id)
    }

    /// Get PTY by task ID
    pub fn get_pty_by_task(&self, task_id: &TaskId) -> Option<PtyId> {
        self.task_to_pty.get(task_id).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::model::{TaskEdge, TaskNode};

    fn create_test_graph() -> TaskGraph {
        let mut graph = TaskGraph::new();

        let task1 = TaskNode::new("Echo 1", "echo")
            .with_args(vec!["Hello".to_string()]);
        let task2 = TaskNode::new("Echo 2", "echo")
            .with_args(vec!["World".to_string()]);

        let id1 = graph.add_task(task1);
        let id2 = graph.add_task(task2);

        // task2 depends on task1
        graph.add_dependency(&id1, &id2, TaskEdge::DependsOn).unwrap();

        graph
    }

    #[test]
    fn test_executor_creation() {
        let graph = create_test_graph();
        let config = ExecutorConfig::default();
        let executor = TaskExecutor::new(graph, config);

        assert_eq!(executor.running_count(), 0);
        assert!(!executor.is_complete());
    }

    #[test]
    fn test_executor_with_events() {
        let graph = create_test_graph();
        let config = ExecutorConfig::default();
        let (executor, _rx) = TaskExecutor::with_events(graph, config);

        assert_eq!(executor.running_count(), 0);
    }
}
