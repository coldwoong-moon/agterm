//! Task Tree Widget
//!
//! A tree view widget for displaying task hierarchy in the sidebar.

use crate::domain::task::{TaskGraph, TaskId, TaskStatus};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, StatefulWidget, Widget},
};

/// Task tree widget for sidebar display
pub struct TaskTree<'a> {
    /// Reference to the task graph
    graph: &'a TaskGraph,
    /// Optional block for borders/title
    block: Option<Block<'a>>,
    /// Style for the widget
    style: Style,
    /// Highlight style for selected item
    highlight_style: Style,
    /// Currently focused task ID
    focused_task: Option<TaskId>,
}

impl<'a> TaskTree<'a> {
    /// Create a new task tree widget
    #[must_use]
    pub fn new(graph: &'a TaskGraph) -> Self {
        Self {
            graph,
            block: None,
            style: Style::default(),
            highlight_style: Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
            focused_task: None,
        }
    }

    /// Set the block for borders/title
    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set the base style
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the highlight style for selected items
    #[must_use]
    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    /// Set the focused task
    #[must_use]
    pub fn focused_task(mut self, task_id: Option<TaskId>) -> Self {
        self.focused_task = task_id;
        self
    }

    /// Get status icon for a task
    fn status_icon(status: TaskStatus) -> &'static str {
        match status {
            TaskStatus::Pending => "○",
            TaskStatus::Blocked => "◐",
            TaskStatus::Running => "●",
            TaskStatus::Completed => "✓",
            TaskStatus::Failed => "✗",
            TaskStatus::Cancelled => "⊘",
            TaskStatus::Skipped => "⊖",
        }
    }

    /// Get status color for a task
    fn status_color(status: TaskStatus) -> Color {
        match status {
            TaskStatus::Pending => Color::Gray,
            TaskStatus::Blocked => Color::DarkGray,
            TaskStatus::Running => Color::Yellow,
            TaskStatus::Completed => Color::Green,
            TaskStatus::Failed => Color::Red,
            TaskStatus::Cancelled => Color::Magenta,
            TaskStatus::Skipped => Color::DarkGray,
        }
    }

    /// Build list items from the task graph
    fn build_items(&self) -> Vec<ListItem<'a>> {
        let mut items = Vec::new();

        // Get root tasks (no dependencies)
        let roots = self.graph.get_root_tasks();

        for root_id in roots {
            self.build_subtree(&root_id, 0, &mut items);
        }

        items
    }

    /// Recursively build subtree items
    fn build_subtree(&self, task_id: &TaskId, depth: usize, items: &mut Vec<ListItem<'a>>) {
        let Some(task) = self.graph.get_task(task_id) else {
            return;
        };

        // Build the line for this task
        let indent = "  ".repeat(depth);
        let prefix = if depth > 0 { "├─" } else { "" };
        let icon = Self::status_icon(task.status);
        let color = Self::status_color(task.status);

        let duration_str = if task.status == TaskStatus::Running || task.status.is_terminal() {
            format!(" ({})", task.duration_str())
        } else {
            String::new()
        };

        let line = Line::from(vec![
            Span::raw(indent),
            Span::raw(prefix),
            Span::styled(format!("{icon} "), Style::default().fg(color)),
            Span::styled(
                task.name.clone(),
                Style::default().fg(if task.status == TaskStatus::Running {
                    Color::White
                } else {
                    Color::Gray
                }),
            ),
            Span::styled(duration_str, Style::default().fg(Color::DarkGray)),
        ]);

        items.push(ListItem::new(line));

        // Get dependents (children in the tree view)
        let dependents = self.graph.get_dependents(task_id);
        for dep_id in dependents {
            self.build_subtree(&dep_id, depth + 1, items);
        }
    }
}

impl Widget for TaskTree<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items = self.build_items();

        let list = List::new(items)
            .style(self.style)
            .highlight_style(self.highlight_style);

        let list = if let Some(block) = self.block {
            list.block(block)
        } else {
            list
        };

        Widget::render(list, area, buf);
    }
}

/// Stateful task tree widget that supports selection
pub struct StatefulTaskTree<'a> {
    /// The inner task tree
    inner: TaskTree<'a>,
    /// List of task IDs in display order
    task_ids: Vec<TaskId>,
}

impl<'a> StatefulTaskTree<'a> {
    /// Create a new stateful task tree
    #[must_use]
    pub fn new(graph: &'a TaskGraph) -> Self {
        let inner = TaskTree::new(graph);

        // Build task ID list in display order
        let mut task_ids = Vec::new();
        let roots = graph.get_root_tasks();
        for root_id in roots {
            Self::collect_ids(graph, &root_id, &mut task_ids);
        }

        Self { inner, task_ids }
    }

    /// Collect task IDs recursively
    fn collect_ids(graph: &TaskGraph, task_id: &TaskId, ids: &mut Vec<TaskId>) {
        ids.push(*task_id);
        let dependents = graph.get_dependents(task_id);
        for dep_id in dependents {
            Self::collect_ids(graph, &dep_id, ids);
        }
    }

    /// Set the block
    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.inner = self.inner.block(block);
        self
    }

    /// Get task ID at the given index
    #[must_use]
    pub fn task_at(&self, index: usize) -> Option<TaskId> {
        self.task_ids.get(index).copied()
    }

    /// Get the number of tasks
    #[must_use]
    pub fn len(&self) -> usize {
        self.task_ids.len()
    }

    /// Check if empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.task_ids.is_empty()
    }
}

impl StatefulWidget for StatefulTaskTree<'_> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let items = self.inner.build_items();

        let list = List::new(items)
            .style(self.inner.style)
            .highlight_style(self.inner.highlight_style);

        let list = if let Some(block) = self.inner.block {
            list.block(block)
        } else {
            list
        };

        StatefulWidget::render(list, area, buf, state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::{TaskEdge, TaskNode};

    fn create_test_graph() -> TaskGraph {
        let mut graph = TaskGraph::new();

        let build = TaskNode::new("Build", "npm run build");
        let test = TaskNode::new("Test", "npm test");
        let lint = TaskNode::new("Lint", "npm run lint");

        let build_id = graph.add_task(build);
        let test_id = graph.add_task(test);
        let lint_id = graph.add_task(lint);

        // Test and Lint depend on Build
        graph
            .add_dependency(&build_id, &test_id, TaskEdge::DependsOn)
            .unwrap();
        graph
            .add_dependency(&build_id, &lint_id, TaskEdge::DependsOn)
            .unwrap();

        graph
    }

    #[test]
    fn test_task_tree_creation() {
        let graph = create_test_graph();
        let tree = TaskTree::new(&graph);
        let items = tree.build_items();
        assert!(!items.is_empty());
    }

    #[test]
    fn test_status_icons() {
        assert_eq!(TaskTree::status_icon(TaskStatus::Running), "●");
        assert_eq!(TaskTree::status_icon(TaskStatus::Completed), "✓");
        assert_eq!(TaskTree::status_icon(TaskStatus::Failed), "✗");
    }
}
