//! Task Scheduler
//!
//! Dependency-aware task scheduler that manages concurrent execution.

use super::graph::TaskGraph;
use super::model::{ErrorPolicy, TaskId, TaskResult, TaskStatus};
use crate::error::{TaskError, TaskResult as TResult};
use crate::infrastructure::pty::PtyId;
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc;

/// Events that the scheduler can produce
#[derive(Debug, Clone)]
pub enum SchedulerEvent {
    /// A task is ready to be executed
    TaskReady { task_id: TaskId, command: String, args: Vec<String> },
    /// A task has completed
    TaskCompleted { task_id: TaskId, result: TaskResult },
    /// A task has failed
    TaskFailed { task_id: TaskId, error: String },
    /// A task was skipped due to dependency failure
    TaskSkipped { task_id: TaskId },
    /// All tasks are complete
    AllComplete { stats: SchedulerStats },
    /// Progress update
    Progress { completed: usize, total: usize, running: usize },
}

/// Commands that can be sent to the scheduler
#[derive(Debug)]
pub enum SchedulerCommand {
    /// Report that a task has started with a PTY
    TaskStarted { task_id: TaskId, pty_id: PtyId },
    /// Report that a task has completed
    TaskCompleted { task_id: TaskId, result: TaskResult },
    /// Cancel a specific task
    CancelTask { task_id: TaskId },
    /// Cancel all tasks
    CancelAll,
    /// Pause scheduling (don't start new tasks)
    Pause,
    /// Resume scheduling
    Resume,
}

/// Scheduler statistics
#[derive(Debug, Clone, Default)]
pub struct SchedulerStats {
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub cancelled: usize,
}

/// Scheduler configuration
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Maximum number of concurrent tasks
    pub max_concurrent: usize,
    /// Default error policy
    pub error_policy: ErrorPolicy,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 4,
            error_policy: ErrorPolicy::StopOnError,
        }
    }
}

/// Task Scheduler
///
/// Manages task execution based on dependencies and concurrency limits.
#[derive(Debug)]
pub struct TaskScheduler {
    /// The task graph
    graph: TaskGraph,
    /// Scheduler configuration
    config: SchedulerConfig,
    /// Currently running tasks (task_id -> pty_id)
    running: HashMap<TaskId, PtyId>,
    /// Tasks waiting to be started
    pending_start: HashSet<TaskId>,
    /// Whether scheduling is paused
    paused: bool,
    /// Whether all tasks should be cancelled
    cancelled: bool,
}

impl TaskScheduler {
    /// Create a new scheduler with a task graph
    pub fn new(graph: TaskGraph, config: SchedulerConfig) -> Self {
        Self {
            graph,
            config,
            running: HashMap::new(),
            pending_start: HashSet::new(),
            paused: false,
            cancelled: false,
        }
    }

    /// Create a scheduler with default configuration
    pub fn with_graph(graph: TaskGraph) -> Self {
        Self::new(graph, SchedulerConfig::default())
    }

    /// Get the underlying task graph
    pub fn graph(&self) -> &TaskGraph {
        &self.graph
    }

    /// Get mutable access to the task graph
    pub fn graph_mut(&mut self) -> &mut TaskGraph {
        &mut self.graph
    }

    /// Get the number of currently running tasks
    pub fn running_count(&self) -> usize {
        self.running.len()
    }

    /// Check if more tasks can be started
    pub fn can_start_more(&self) -> bool {
        !self.paused
            && !self.cancelled
            && self.running.len() < self.config.max_concurrent
    }

    /// Get the next batch of tasks to execute
    ///
    /// Returns tasks that are ready (dependencies satisfied) and can be started
    /// based on concurrency limits.
    pub fn get_next_tasks(&mut self) -> Vec<(TaskId, String, Vec<String>)> {
        if !self.can_start_more() {
            return Vec::new();
        }

        // Update blocked status
        self.graph.update_blocked_status();

        // Get ready tasks
        let ready = self.graph.get_ready_tasks();
        let available_slots = self.config.max_concurrent - self.running.len();

        let mut result = Vec::new();
        for task_id in ready.into_iter().take(available_slots) {
            // Skip if already pending start
            if self.pending_start.contains(&task_id) {
                continue;
            }

            if let Some(task) = self.graph.get_task(&task_id) {
                result.push((
                    task_id,
                    task.command.clone(),
                    task.args.clone(),
                ));
                self.pending_start.insert(task_id);
            }
        }

        result
    }

    /// Mark a task as started
    pub fn task_started(&mut self, task_id: TaskId, pty_id: PtyId) -> TResult<()> {
        self.pending_start.remove(&task_id);
        self.graph.start_task(&task_id, pty_id)?;
        self.running.insert(task_id, pty_id);
        Ok(())
    }

    /// Mark a task as completed and process the result
    ///
    /// Returns events generated by this completion (e.g., newly ready tasks, skipped tasks)
    pub fn task_completed(&mut self, task_id: TaskId, result: TaskResult) -> TResult<Vec<SchedulerEvent>> {
        let mut events = Vec::new();

        // Remove from running
        self.running.remove(&task_id);
        self.pending_start.remove(&task_id);

        // Update task status
        let success = result.is_success();
        self.graph.complete_task(&task_id, result.clone())?;

        // If failed, handle based on error policy
        if !success {
            let task = self.graph.get_task(&task_id);
            let policy = task.map(|t| t.error_policy).unwrap_or(self.config.error_policy);

            match policy {
                ErrorPolicy::StopOnError => {
                    // Skip all dependent tasks
                    let dependents = self.get_all_dependents(&task_id);
                    for dep_id in dependents {
                        if let Ok(()) = self.graph.skip_task(&dep_id) {
                            events.push(SchedulerEvent::TaskSkipped { task_id: dep_id });
                        }
                    }
                }
                ErrorPolicy::ContinueOnError => {
                    // Just skip direct dependents
                    let dependents = self.graph.get_dependents(&task_id);
                    for dep_id in dependents {
                        if let Ok(()) = self.graph.skip_task(&dep_id) {
                            events.push(SchedulerEvent::TaskSkipped { task_id: dep_id });
                        }
                    }
                }
                ErrorPolicy::RetryThenStop { max_retries } => {
                    if let Some(task) = self.graph.get_task_mut(&task_id) {
                        if task.retry_count < max_retries {
                            task.retry_count += 1;
                            task.status = TaskStatus::Pending;
                            // Will be picked up in next get_next_tasks call
                        } else {
                            // Max retries reached, treat as StopOnError
                            let dependents = self.get_all_dependents(&task_id);
                            for dep_id in dependents {
                                if let Ok(()) = self.graph.skip_task(&dep_id) {
                                    events.push(SchedulerEvent::TaskSkipped { task_id: dep_id });
                                }
                            }
                        }
                    }
                }
            }

            events.push(SchedulerEvent::TaskFailed {
                task_id,
                error: result.stderr.clone(),
            });
        } else {
            events.push(SchedulerEvent::TaskCompleted { task_id, result });
        }

        // Update blocked status for remaining tasks
        self.graph.update_blocked_status();

        // Check for newly ready tasks
        for (task_id, command, args) in self.get_next_tasks() {
            events.push(SchedulerEvent::TaskReady { task_id, command, args });
        }

        // Check if all done
        if self.is_complete() {
            events.push(SchedulerEvent::AllComplete {
                stats: self.statistics(),
            });
        } else {
            // Send progress update
            let stats = self.graph.statistics();
            events.push(SchedulerEvent::Progress {
                completed: stats.completed + stats.failed + stats.skipped + stats.cancelled,
                total: stats.total,
                running: stats.running,
            });
        }

        Ok(events)
    }

    /// Cancel a specific task
    pub fn cancel_task(&mut self, task_id: TaskId) -> TResult<Vec<SchedulerEvent>> {
        let mut events = Vec::new();

        self.running.remove(&task_id);
        self.pending_start.remove(&task_id);
        self.graph.cancel_task(&task_id)?;

        // Skip dependents
        let dependents = self.get_all_dependents(&task_id);
        for dep_id in dependents {
            if let Ok(()) = self.graph.skip_task(&dep_id) {
                events.push(SchedulerEvent::TaskSkipped { task_id: dep_id });
            }
        }

        Ok(events)
    }

    /// Cancel all tasks
    pub fn cancel_all(&mut self) -> Vec<SchedulerEvent> {
        self.cancelled = true;
        let mut events = Vec::new();

        // Cancel all running tasks
        let running_ids: Vec<TaskId> = self.running.keys().copied().collect();
        for task_id in running_ids {
            if let Ok(evts) = self.cancel_task(task_id) {
                events.extend(evts);
            }
        }

        // Cancel all pending tasks
        let pending_ids = self.graph.get_tasks_by_status(TaskStatus::Pending);
        for task_id in pending_ids {
            let _ = self.graph.cancel_task(&task_id);
        }

        events
    }

    /// Pause scheduling
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume scheduling
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Check if scheduling is paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Check if all tasks are complete
    pub fn is_complete(&self) -> bool {
        self.running.is_empty() && self.pending_start.is_empty() && self.graph.is_complete()
    }

    /// Get scheduler statistics
    pub fn statistics(&self) -> SchedulerStats {
        let graph_stats = self.graph.statistics();
        SchedulerStats {
            total: graph_stats.total,
            completed: graph_stats.completed,
            failed: graph_stats.failed,
            skipped: graph_stats.skipped,
            cancelled: graph_stats.cancelled,
        }
    }

    /// Get all transitive dependents of a task
    fn get_all_dependents(&self, task_id: &TaskId) -> Vec<TaskId> {
        let mut all_dependents = Vec::new();
        let mut to_process = vec![*task_id];
        let mut seen = HashSet::new();

        while let Some(current) = to_process.pop() {
            if seen.contains(&current) {
                continue;
            }
            seen.insert(current);

            for dep in self.graph.get_dependents(&current) {
                if !seen.contains(&dep) {
                    all_dependents.push(dep);
                    to_process.push(dep);
                }
            }
        }

        all_dependents
    }

    /// Start scheduling and return initial tasks to run
    pub fn start(&mut self) -> Vec<SchedulerEvent> {
        let mut events = Vec::new();

        // Update blocked status
        self.graph.update_blocked_status();

        // Get initial ready tasks
        for (task_id, command, args) in self.get_next_tasks() {
            events.push(SchedulerEvent::TaskReady { task_id, command, args });
        }

        // Initial progress
        let stats = self.graph.statistics();
        events.push(SchedulerEvent::Progress {
            completed: 0,
            total: stats.total,
            running: 0,
        });

        events
    }
}

/// Async scheduler runner
///
/// Runs the scheduler in a background task and communicates via channels.
pub struct SchedulerRunner {
    scheduler: TaskScheduler,
    event_tx: mpsc::UnboundedSender<SchedulerEvent>,
    command_rx: mpsc::UnboundedReceiver<SchedulerCommand>,
}

impl SchedulerRunner {
    /// Create a new scheduler runner
    pub fn new(
        scheduler: TaskScheduler,
        event_tx: mpsc::UnboundedSender<SchedulerEvent>,
        command_rx: mpsc::UnboundedReceiver<SchedulerCommand>,
    ) -> Self {
        Self {
            scheduler,
            event_tx,
            command_rx,
        }
    }

    /// Run the scheduler until all tasks are complete or cancelled
    pub async fn run(mut self) {
        // Start scheduling
        let events = self.scheduler.start();
        for event in events {
            let _ = self.event_tx.send(event);
        }

        // Process commands
        while let Some(cmd) = self.command_rx.recv().await {
            let events = match cmd {
                SchedulerCommand::TaskStarted { task_id, pty_id } => {
                    match self.scheduler.task_started(task_id, pty_id) {
                        Ok(()) => Vec::new(),
                        Err(e) => {
                            vec![SchedulerEvent::TaskFailed {
                                task_id,
                                error: e.to_string(),
                            }]
                        }
                    }
                }
                SchedulerCommand::TaskCompleted { task_id, result } => {
                    self.scheduler.task_completed(task_id, result).unwrap_or_default()
                }
                SchedulerCommand::CancelTask { task_id } => {
                    self.scheduler.cancel_task(task_id).unwrap_or_default()
                }
                SchedulerCommand::CancelAll => {
                    self.scheduler.cancel_all()
                }
                SchedulerCommand::Pause => {
                    self.scheduler.pause();
                    Vec::new()
                }
                SchedulerCommand::Resume => {
                    self.scheduler.resume();
                    // Get newly ready tasks after resume
                    self.scheduler.get_next_tasks()
                        .into_iter()
                        .map(|(task_id, command, args)| {
                            SchedulerEvent::TaskReady { task_id, command, args }
                        })
                        .collect()
                }
            };

            // Send events
            for event in events {
                if self.event_tx.send(event).is_err() {
                    // Receiver dropped, stop
                    break;
                }
            }

            // Check if complete
            if self.scheduler.is_complete() {
                break;
            }
        }
    }
}

/// Create a scheduler with channels for communication
pub fn create_scheduler(
    graph: TaskGraph,
    config: SchedulerConfig,
) -> (
    mpsc::UnboundedSender<SchedulerCommand>,
    mpsc::UnboundedReceiver<SchedulerEvent>,
    SchedulerRunner,
) {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let (command_tx, command_rx) = mpsc::unbounded_channel();
    let scheduler = TaskScheduler::new(graph, config);
    let runner = SchedulerRunner::new(scheduler, event_tx, command_rx);
    (command_tx, event_rx, runner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::model::{TaskNode, TaskEdge};

    fn create_test_graph() -> TaskGraph {
        let mut graph = TaskGraph::new();

        let a = TaskNode::new("A", "echo");
        let b = TaskNode::new("B", "echo");
        let c = TaskNode::new("C", "echo");

        let a_id = graph.add_task(a.with_args(vec!["A".to_string()]));
        let b_id = graph.add_task(b.with_args(vec!["B".to_string()]));
        let c_id = graph.add_task(c.with_args(vec!["C".to_string()]));

        // B depends on A, C depends on B
        graph.add_dependency(&a_id, &b_id, TaskEdge::DependsOn).unwrap();
        graph.add_dependency(&b_id, &c_id, TaskEdge::DependsOn).unwrap();

        graph
    }

    #[test]
    fn test_scheduler_creation() {
        let graph = create_test_graph();
        let scheduler = TaskScheduler::with_graph(graph);
        assert_eq!(scheduler.running_count(), 0);
        assert!(!scheduler.is_complete());
    }

    #[test]
    fn test_get_next_tasks() {
        let graph = create_test_graph();
        let mut scheduler = TaskScheduler::with_graph(graph);

        let tasks = scheduler.get_next_tasks();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].1, "echo");
        assert_eq!(tasks[0].2, vec!["A".to_string()]);
    }

    #[test]
    fn test_task_lifecycle() {
        let graph = create_test_graph();
        let mut scheduler = TaskScheduler::with_graph(graph);

        // Get first task
        let tasks = scheduler.get_next_tasks();
        let (task_id, _, _) = tasks.into_iter().next().unwrap();

        // Start it
        let pty_id = uuid::Uuid::new_v4();
        scheduler.task_started(task_id, pty_id).unwrap();
        assert_eq!(scheduler.running_count(), 1);

        // Complete it
        let result = TaskResult::success("A".to_string(), 100);
        let events = scheduler.task_completed(task_id, result).unwrap();

        // Should have: TaskCompleted, TaskReady (for B), Progress
        assert!(events.iter().any(|e| matches!(e, SchedulerEvent::TaskCompleted { .. })));
        assert!(events.iter().any(|e| matches!(e, SchedulerEvent::TaskReady { .. })));
    }

    #[test]
    fn test_concurrency_limit() {
        let mut graph = TaskGraph::new();

        // Create 5 independent tasks
        for i in 0..5 {
            let task = TaskNode::new(format!("Task{}", i), "echo");
            graph.add_task(task);
        }

        let config = SchedulerConfig {
            max_concurrent: 2,
            ..Default::default()
        };
        let mut scheduler = TaskScheduler::new(graph, config);

        let tasks = scheduler.get_next_tasks();
        assert_eq!(tasks.len(), 2); // Limited by max_concurrent
    }

    #[test]
    fn test_pause_resume() {
        let graph = create_test_graph();
        let mut scheduler = TaskScheduler::with_graph(graph);

        // Pause
        scheduler.pause();
        let tasks = scheduler.get_next_tasks();
        assert!(tasks.is_empty());

        // Resume
        scheduler.resume();
        let tasks = scheduler.get_next_tasks();
        assert_eq!(tasks.len(), 1);
    }

    #[test]
    fn test_error_propagation() {
        let graph = create_test_graph();
        let mut scheduler = TaskScheduler::with_graph(graph);

        // Get and start first task
        let tasks = scheduler.get_next_tasks();
        let (task_id, _, _) = tasks.into_iter().next().unwrap();
        let pty_id = uuid::Uuid::new_v4();
        scheduler.task_started(task_id, pty_id).unwrap();

        // Fail it
        let result = TaskResult::failure(1, "error".to_string(), 100);
        let events = scheduler.task_completed(task_id, result).unwrap();

        // Should skip dependents (B and C)
        let skipped: Vec<_> = events.iter()
            .filter(|e| matches!(e, SchedulerEvent::TaskSkipped { .. }))
            .collect();
        assert_eq!(skipped.len(), 2);
    }
}
