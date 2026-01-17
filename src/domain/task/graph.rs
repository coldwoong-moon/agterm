//! Task Graph
//!
//! Directed acyclic graph for task dependencies using petgraph.

use super::model::{TaskEdge, TaskId, TaskNode, TaskResult, TaskStatus};
use crate::error::{TaskError, TaskResult as TResult};
use crate::infrastructure::pty::PtyId;
use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use std::collections::HashMap;

/// Task graph - manages tasks and their dependencies
#[derive(Debug, Clone)]
pub struct TaskGraph {
    /// The underlying directed graph
    graph: DiGraph<TaskNode, TaskEdge>,
    /// Map from `TaskId` to `NodeIndex` for fast lookup
    id_to_index: HashMap<TaskId, NodeIndex>,
}

impl TaskGraph {
    /// Create a new empty task graph
    #[must_use]
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            id_to_index: HashMap::new(),
        }
    }

    /// Get the number of tasks
    #[must_use]
    pub fn task_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Check if the graph is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.graph.node_count() == 0
    }

    /// Add a task to the graph
    pub fn add_task(&mut self, task: TaskNode) -> TaskId {
        let id = task.id;
        let index = self.graph.add_node(task);
        self.id_to_index.insert(id, index);
        id
    }

    /// Remove a task from the graph
    pub fn remove_task(&mut self, task_id: &TaskId) -> Option<TaskNode> {
        if let Some(&index) = self.id_to_index.get(task_id) {
            self.id_to_index.remove(task_id);
            self.graph.remove_node(index)
        } else {
            None
        }
    }

    /// Get a reference to a task
    #[must_use]
    pub fn get_task(&self, task_id: &TaskId) -> Option<&TaskNode> {
        self.id_to_index
            .get(task_id)
            .and_then(|&index| self.graph.node_weight(index))
    }

    /// Get a mutable reference to a task
    pub fn get_task_mut(&mut self, task_id: &TaskId) -> Option<&mut TaskNode> {
        if let Some(&index) = self.id_to_index.get(task_id) {
            self.graph.node_weight_mut(index)
        } else {
            None
        }
    }

    /// Add a dependency edge (from -> to means "to" depends on "from")
    pub fn add_dependency(
        &mut self,
        from: &TaskId,
        to: &TaskId,
        edge_type: TaskEdge,
    ) -> TResult<()> {
        let from_idx = self
            .id_to_index
            .get(from)
            .ok_or(TaskError::NotFound { id: *from })?;
        let to_idx = self
            .id_to_index
            .get(to)
            .ok_or(TaskError::NotFound { id: *to })?;

        self.graph.add_edge(*from_idx, *to_idx, edge_type);

        // Check for cycles
        if self.has_cycle() {
            // Remove the edge we just added
            if let Some(edge) = self.graph.find_edge(*from_idx, *to_idx) {
                self.graph.remove_edge(edge);
            }
            return Err(TaskError::CircularDependency {
                cycle: vec![*from, *to],
            });
        }

        Ok(())
    }

    /// Check if the graph has a cycle
    #[must_use]
    pub fn has_cycle(&self) -> bool {
        toposort(&self.graph, None).is_err()
    }

    /// Get topological order of tasks
    pub fn topological_order(&self) -> TResult<Vec<TaskId>> {
        match toposort(&self.graph, None) {
            Ok(indices) => Ok(indices
                .into_iter()
                .filter_map(|idx| self.graph.node_weight(idx).map(|t| t.id))
                .collect()),
            Err(cycle) => {
                let task_id = self
                    .graph
                    .node_weight(cycle.node_id())
                    .map(|t| t.id)
                    .unwrap_or_default();
                Err(TaskError::CircularDependency {
                    cycle: vec![task_id],
                })
            }
        }
    }

    /// Get direct dependencies of a task (tasks that must complete before this one)
    #[must_use]
    pub fn get_dependencies(&self, task_id: &TaskId) -> Vec<TaskId> {
        let Some(&index) = self.id_to_index.get(task_id) else {
            return Vec::new();
        };

        self.graph
            .edges_directed(index, Direction::Incoming)
            .filter_map(|edge| self.graph.node_weight(edge.source()).map(|task| task.id))
            .collect()
    }

    /// Get direct dependents of a task (tasks that depend on this one)
    #[must_use]
    pub fn get_dependents(&self, task_id: &TaskId) -> Vec<TaskId> {
        let Some(&index) = self.id_to_index.get(task_id) else {
            return Vec::new();
        };

        self.graph
            .edges_directed(index, Direction::Outgoing)
            .filter_map(|edge| self.graph.node_weight(edge.target()).map(|task| task.id))
            .collect()
    }

    /// Check if all dependencies of a task are completed
    #[must_use]
    pub fn dependencies_satisfied(&self, task_id: &TaskId) -> bool {
        let deps = self.get_dependencies(task_id);
        deps.iter().all(|dep_id| {
            self.get_task(dep_id)
                .is_some_and(|t| t.status == TaskStatus::Completed)
        })
    }

    /// Check if any dependency of a task has failed
    #[must_use]
    pub fn has_failed_dependency(&self, task_id: &TaskId) -> bool {
        let Some(&index) = self.id_to_index.get(task_id) else {
            return false;
        };

        self.graph
            .edges_directed(index, Direction::Incoming)
            .any(|edge| {
                // Only check hard dependencies
                if *edge.weight() == TaskEdge::SoftDependsOn {
                    return false;
                }
                self.graph.node_weight(edge.source()).is_some_and(|t| {
                    t.status == TaskStatus::Failed || t.status == TaskStatus::Cancelled
                })
            })
    }

    /// Get all tasks that are ready to run (pending with satisfied dependencies)
    #[must_use]
    pub fn get_ready_tasks(&self) -> Vec<TaskId> {
        self.graph
            .node_indices()
            .filter_map(|idx| {
                let task = self.graph.node_weight(idx)?;
                if task.status == TaskStatus::Pending && self.dependencies_satisfied(&task.id) {
                    Some(task.id)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all tasks with a specific status
    #[must_use]
    pub fn get_tasks_by_status(&self, status: TaskStatus) -> Vec<TaskId> {
        self.graph
            .node_indices()
            .filter_map(|idx| {
                let task = self.graph.node_weight(idx)?;
                if task.status == status {
                    Some(task.id)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all tasks
    #[must_use]
    pub fn all_tasks(&self) -> Vec<&TaskNode> {
        self.graph
            .node_indices()
            .filter_map(|idx| self.graph.node_weight(idx))
            .collect()
    }

    /// Get all task IDs
    #[must_use]
    pub fn all_task_ids(&self) -> Vec<TaskId> {
        self.graph
            .node_indices()
            .filter_map(|idx| self.graph.node_weight(idx).map(|t| t.id))
            .collect()
    }

    /// Start a task (update status and set PTY ID)
    pub fn start_task(&mut self, task_id: &TaskId, pty_id: PtyId) -> TResult<()> {
        let task = self
            .get_task_mut(task_id)
            .ok_or(TaskError::NotFound { id: *task_id })?;

        if !task.status.is_runnable() {
            return Err(TaskError::InvalidStateTransition {
                from: task.status.to_string(),
                to: TaskStatus::Running.to_string(),
            });
        }

        task.start(pty_id);
        Ok(())
    }

    /// Complete a task with result
    pub fn complete_task(&mut self, task_id: &TaskId, result: TaskResult) -> TResult<()> {
        let task = self
            .get_task_mut(task_id)
            .ok_or(TaskError::NotFound { id: *task_id })?;

        task.complete(result);
        Ok(())
    }

    /// Cancel a task
    pub fn cancel_task(&mut self, task_id: &TaskId) -> TResult<()> {
        let task = self
            .get_task_mut(task_id)
            .ok_or(TaskError::NotFound { id: *task_id })?;

        if task.status.is_terminal() {
            return Err(TaskError::CannotCancel {
                id: *task_id,
                state: task.status.to_string(),
            });
        }

        task.cancel();
        Ok(())
    }

    /// Skip a task (due to dependency failure)
    pub fn skip_task(&mut self, task_id: &TaskId) -> TResult<()> {
        let task = self
            .get_task_mut(task_id)
            .ok_or(TaskError::NotFound { id: *task_id })?;

        task.skip();
        Ok(())
    }

    /// Update blocked/unblocked status based on dependencies
    pub fn update_blocked_status(&mut self) {
        let task_ids: Vec<TaskId> = self.all_task_ids();

        for task_id in task_ids {
            let should_block = !self.dependencies_satisfied(&task_id);
            let has_failed_dep = self.has_failed_dependency(&task_id);

            if let Some(task) = self.get_task_mut(&task_id) {
                match task.status {
                    TaskStatus::Pending if should_block => task.block(),
                    TaskStatus::Blocked if !should_block && !has_failed_dep => task.unblock(),
                    TaskStatus::Pending | TaskStatus::Blocked if has_failed_dep => task.skip(),
                    _ => {}
                }
            }
        }
    }

    /// Get root tasks (tasks with no dependencies)
    #[must_use]
    pub fn get_root_tasks(&self) -> Vec<TaskId> {
        self.graph
            .node_indices()
            .filter(|&idx| self.graph.edges_directed(idx, Direction::Incoming).count() == 0)
            .filter_map(|idx| self.graph.node_weight(idx).map(|t| t.id))
            .collect()
    }

    /// Get leaf tasks (tasks with no dependents)
    #[must_use]
    pub fn get_leaf_tasks(&self) -> Vec<TaskId> {
        self.graph
            .node_indices()
            .filter(|&idx| self.graph.edges_directed(idx, Direction::Outgoing).count() == 0)
            .filter_map(|idx| self.graph.node_weight(idx).map(|t| t.id))
            .collect()
    }

    /// Check if all tasks are complete (terminal state)
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.graph.node_indices().all(|idx| {
            self.graph
                .node_weight(idx)
                .map_or(true, |t| t.status.is_terminal())
        })
    }

    /// Get completion statistics
    #[must_use]
    pub fn statistics(&self) -> TaskStatistics {
        let mut stats = TaskStatistics::default();
        for idx in self.graph.node_indices() {
            if let Some(task) = self.graph.node_weight(idx) {
                stats.total += 1;
                match task.status {
                    TaskStatus::Pending => stats.pending += 1,
                    TaskStatus::Blocked => stats.blocked += 1,
                    TaskStatus::Running => stats.running += 1,
                    TaskStatus::Completed => stats.completed += 1,
                    TaskStatus::Failed => stats.failed += 1,
                    TaskStatus::Cancelled => stats.cancelled += 1,
                    TaskStatus::Skipped => stats.skipped += 1,
                }
            }
        }
        stats
    }
}

impl Default for TaskGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Task completion statistics
#[derive(Debug, Clone, Default)]
pub struct TaskStatistics {
    /// Total number of tasks
    pub total: usize,
    /// Pending tasks
    pub pending: usize,
    /// Blocked tasks
    pub blocked: usize,
    /// Running tasks
    pub running: usize,
    /// Completed tasks
    pub completed: usize,
    /// Failed tasks
    pub failed: usize,
    /// Cancelled tasks
    pub cancelled: usize,
    /// Skipped tasks
    pub skipped: usize,
}

impl TaskStatistics {
    /// Get progress as percentage (0-100)
    #[must_use]
    pub fn progress_percent(&self) -> f32 {
        if self.total == 0 {
            return 100.0;
        }
        let terminal = self.completed + self.failed + self.cancelled + self.skipped;
        (terminal as f32 / self.total as f32) * 100.0
    }

    /// Check if all tasks are done
    #[must_use]
    pub fn is_done(&self) -> bool {
        self.pending == 0 && self.blocked == 0 && self.running == 0
    }

    /// Check if there are any failures
    #[must_use]
    pub fn has_failures(&self) -> bool {
        self.failed > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_creation() {
        let graph = TaskGraph::new();
        assert!(graph.is_empty());
        assert_eq!(graph.task_count(), 0);
    }

    #[test]
    fn test_add_task() {
        let mut graph = TaskGraph::new();
        let task = TaskNode::new("Build", "npm run build");
        let id = graph.add_task(task);

        assert_eq!(graph.task_count(), 1);
        assert!(graph.get_task(&id).is_some());
    }

    #[test]
    fn test_add_dependency() {
        let mut graph = TaskGraph::new();

        let build = TaskNode::new("Build", "npm run build");
        let test = TaskNode::new("Test", "npm test");

        let build_id = graph.add_task(build);
        let test_id = graph.add_task(test);

        // Test depends on Build
        graph
            .add_dependency(&build_id, &test_id, TaskEdge::DependsOn)
            .unwrap();

        let deps = graph.get_dependencies(&test_id);
        assert_eq!(deps, vec![build_id]);

        let dependents = graph.get_dependents(&build_id);
        assert_eq!(dependents, vec![test_id]);
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = TaskGraph::new();

        let a = TaskNode::new("A", "echo A");
        let b = TaskNode::new("B", "echo B");

        let a_id = graph.add_task(a);
        let b_id = graph.add_task(b);

        // A -> B
        graph
            .add_dependency(&a_id, &b_id, TaskEdge::DependsOn)
            .unwrap();

        // B -> A (would create cycle)
        let result = graph.add_dependency(&b_id, &a_id, TaskEdge::DependsOn);
        assert!(result.is_err());
    }

    #[test]
    fn test_topological_order() {
        let mut graph = TaskGraph::new();

        let a = TaskNode::new("A", "echo A");
        let b = TaskNode::new("B", "echo B");
        let c = TaskNode::new("C", "echo C");

        let a_id = graph.add_task(a);
        let b_id = graph.add_task(b);
        let c_id = graph.add_task(c);

        // A -> B -> C
        graph
            .add_dependency(&a_id, &b_id, TaskEdge::DependsOn)
            .unwrap();
        graph
            .add_dependency(&b_id, &c_id, TaskEdge::DependsOn)
            .unwrap();

        let order = graph.topological_order().unwrap();
        let a_pos = order.iter().position(|&id| id == a_id).unwrap();
        let b_pos = order.iter().position(|&id| id == b_id).unwrap();
        let c_pos = order.iter().position(|&id| id == c_id).unwrap();

        assert!(a_pos < b_pos);
        assert!(b_pos < c_pos);
    }

    #[test]
    fn test_ready_tasks() {
        let mut graph = TaskGraph::new();

        let a = TaskNode::new("A", "echo A");
        let b = TaskNode::new("B", "echo B");

        let a_id = graph.add_task(a);
        let b_id = graph.add_task(b);

        // B depends on A
        graph
            .add_dependency(&a_id, &b_id, TaskEdge::DependsOn)
            .unwrap();

        // Initially, only A is ready
        let ready = graph.get_ready_tasks();
        assert_eq!(ready, vec![a_id]);

        // Complete A
        graph
            .complete_task(&a_id, TaskResult::success("done".to_string(), 100))
            .unwrap();

        // Now B is ready
        let ready = graph.get_ready_tasks();
        assert_eq!(ready, vec![b_id]);
    }

    #[test]
    fn test_statistics() {
        let mut graph = TaskGraph::new();

        let a = TaskNode::new("A", "echo A");
        let b = TaskNode::new("B", "echo B");
        let c = TaskNode::new("C", "echo C");

        let a_id = graph.add_task(a);
        graph.add_task(b);
        graph.add_task(c);

        graph
            .complete_task(&a_id, TaskResult::success("done".to_string(), 100))
            .unwrap();

        let stats = graph.statistics();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.completed, 1);
        assert_eq!(stats.pending, 2);
    }
}
