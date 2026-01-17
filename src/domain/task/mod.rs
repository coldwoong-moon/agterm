//! Task Management
//!
//! Task model, graph operations, and scheduling.

pub mod executor;
pub mod graph;
pub mod model;
pub mod scheduler;

pub use executor::{ExecutorConfig, ExecutorEvent, TaskExecutor};
pub use graph::{TaskGraph, TaskStatistics};
pub use model::{ErrorPolicy, TaskEdge, TaskId, TaskNode, TaskResult, TaskStatus};
pub use scheduler::{
    create_scheduler, SchedulerCommand, SchedulerConfig, SchedulerEvent, SchedulerRunner,
    SchedulerStats, TaskScheduler,
};
