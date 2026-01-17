//! Graph View Widget
//!
//! A full-screen ASCII art visualization of the task dependency graph.

use crate::domain::task::{TaskGraph, TaskId, TaskNode, TaskStatistics, TaskStatus};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};
use std::collections::HashMap;

/// Node layout information
#[derive(Debug, Clone)]
struct NodeLayout {
    /// Task ID
    task_id: TaskId,
    /// X position (column)
    x: u16,
    /// Y position (row)
    y: u16,
    /// Width of the node
    width: u16,
    /// Height of the node
    height: u16,
    /// Layer (depth) in the graph
    layer: usize,
}

/// Graph view widget for visualizing task flow
pub struct GraphView<'a> {
    /// Reference to the task graph
    graph: &'a TaskGraph,
    /// Optional block for borders/title
    block: Option<Block<'a>>,
    /// Currently selected task
    selected: Option<TaskId>,
    /// Node width
    node_width: u16,
    /// Node height
    node_height: u16,
    /// Horizontal spacing between nodes
    h_spacing: u16,
    /// Vertical spacing between layers
    v_spacing: u16,
}

impl<'a> GraphView<'a> {
    /// Create a new graph view
    pub fn new(graph: &'a TaskGraph) -> Self {
        Self {
            graph,
            block: None,
            selected: None,
            node_width: 20,
            node_height: 5,
            h_spacing: 4,
            v_spacing: 2,
        }
    }

    /// Set the block for borders/title
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set the selected task
    pub fn selected(mut self, task_id: Option<TaskId>) -> Self {
        self.selected = task_id;
        self
    }

    /// Get status icon
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

    /// Get status color
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

    /// Compute node layouts using layered graph layout
    fn compute_layout(&self, area: Rect) -> Vec<NodeLayout> {
        let mut layouts = Vec::new();

        // Get topological order
        let topo_order = match self.graph.topological_order() {
            Ok(order) => order,
            Err(_) => return layouts,
        };

        if topo_order.is_empty() {
            return layouts;
        }

        // Assign layers based on longest path from root
        let mut layers: HashMap<TaskId, usize> = HashMap::new();

        for task_id in &topo_order {
            let deps = self.graph.get_dependencies(task_id);
            let layer = if deps.is_empty() {
                0
            } else {
                deps.iter()
                    .filter_map(|d| layers.get(d))
                    .max()
                    .map(|m| m + 1)
                    .unwrap_or(0)
            };
            layers.insert(*task_id, layer);
        }

        // Group tasks by layer
        let max_layer = layers.values().max().copied().unwrap_or(0);
        let mut layer_tasks: Vec<Vec<TaskId>> = vec![Vec::new(); max_layer + 1];

        for (task_id, layer) in &layers {
            layer_tasks[*layer].push(*task_id);
        }

        // Calculate positions
        let total_height = (max_layer + 1) as u16 * (self.node_height + self.v_spacing);
        let start_y = area.y + (area.height.saturating_sub(total_height)) / 2;

        for (layer_idx, tasks) in layer_tasks.iter().enumerate() {
            let layer_width = tasks.len() as u16 * (self.node_width + self.h_spacing);
            let start_x = area.x + (area.width.saturating_sub(layer_width)) / 2;

            for (task_idx, task_id) in tasks.iter().enumerate() {
                let x = start_x + task_idx as u16 * (self.node_width + self.h_spacing);
                let y = start_y + layer_idx as u16 * (self.node_height + self.v_spacing);

                layouts.push(NodeLayout {
                    task_id: *task_id,
                    x,
                    y,
                    width: self.node_width,
                    height: self.node_height,
                    layer: layer_idx,
                });
            }
        }

        layouts
    }

    /// Render a single task node
    fn render_node(&self, task: &TaskNode, layout: &NodeLayout, area: Rect, buf: &mut Buffer) {
        // Check bounds
        if layout.x >= area.x + area.width || layout.y >= area.y + area.height {
            return;
        }

        let node_rect = Rect {
            x: layout.x,
            y: layout.y,
            width: layout.width.min(area.x + area.width - layout.x),
            height: layout.height.min(area.y + area.height - layout.y),
        };

        let is_selected = self.selected == Some(task.id);
        let status_color = Self::status_color(task.status);

        let border_style = if is_selected {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(status_color)
        };

        // Draw border
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);

        Widget::render(block, node_rect, buf);

        // Draw content inside the border
        let inner_rect = Rect {
            x: node_rect.x + 1,
            y: node_rect.y + 1,
            width: node_rect.width.saturating_sub(2),
            height: node_rect.height.saturating_sub(2),
        };

        if inner_rect.width == 0 || inner_rect.height == 0 {
            return;
        }

        // Line 1: Task name (truncated)
        let name = if task.name.len() > inner_rect.width as usize - 2 {
            format!("{}…", &task.name[..inner_rect.width as usize - 3])
        } else {
            task.name.clone()
        };

        let icon = Self::status_icon(task.status);
        let name_line = format!("{} {}", icon, name);

        if inner_rect.y < area.y + area.height {
            let name_style = Style::default()
                .fg(status_color)
                .add_modifier(if task.status == TaskStatus::Running {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                });

            buf.set_string(inner_rect.x, inner_rect.y, &name_line, name_style);
        }

        // Line 2: Progress bar (for running tasks)
        if inner_rect.height > 1 && inner_rect.y + 1 < area.y + area.height {
            let progress_y = inner_rect.y + 1;

            if task.status == TaskStatus::Running {
                // Animated progress bar
                let bar_width = inner_rect.width as usize;
                let filled = (bar_width / 2).min(bar_width); // Placeholder progress
                let bar: String = "█".repeat(filled) + &"░".repeat(bar_width - filled);
                buf.set_string(inner_rect.x, progress_y, &bar, Style::default().fg(Color::Yellow));
            } else {
                // Duration
                let duration = task.duration_str();
                buf.set_string(
                    inner_rect.x,
                    progress_y,
                    &duration,
                    Style::default().fg(Color::DarkGray),
                );
            }
        }
    }

    /// Render connection lines between nodes
    fn render_connections(
        &self,
        layouts: &[NodeLayout],
        area: Rect,
        buf: &mut Buffer,
    ) {
        let layout_map: HashMap<TaskId, &NodeLayout> =
            layouts.iter().map(|l| (l.task_id, l)).collect();

        for layout in layouts {
            let deps = self.graph.get_dependencies(&layout.task_id);

            for dep_id in deps {
                if let Some(dep_layout) = layout_map.get(&dep_id) {
                    self.render_connection(dep_layout, layout, area, buf);
                }
            }
        }
    }

    /// Render a single connection line
    fn render_connection(
        &self,
        from: &NodeLayout,
        to: &NodeLayout,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let from_x = from.x + from.width / 2;
        let from_y = from.y + from.height;
        let to_x = to.x + to.width / 2;
        let to_y = to.y;

        let line_style = Style::default().fg(Color::DarkGray);

        // Draw vertical line from source
        if from_y < to_y {
            let mid_y = (from_y + to_y) / 2;

            // Vertical from source to mid
            for y in from_y..=mid_y.min(area.y + area.height - 1) {
                if y >= area.y && from_x >= area.x && from_x < area.x + area.width {
                    buf.set_string(from_x, y, "│", line_style);
                }
            }

            // Horizontal at mid level
            let (min_x, max_x) = if from_x < to_x {
                (from_x, to_x)
            } else {
                (to_x, from_x)
            };

            if mid_y >= area.y && mid_y < area.y + area.height {
                for x in min_x..=max_x {
                    if x >= area.x && x < area.x + area.width {
                        buf.set_string(x, mid_y, "─", line_style);
                    }
                }

                // Corners
                if from_x != to_x {
                    if from_x >= area.x && from_x < area.x + area.width {
                        let corner = if from_x < to_x { "└" } else { "┘" };
                        buf.set_string(from_x, mid_y, corner, line_style);
                    }
                    if to_x >= area.x && to_x < area.x + area.width {
                        let corner = if from_x < to_x { "┐" } else { "┌" };
                        buf.set_string(to_x, mid_y, corner, line_style);
                    }
                }
            }

            // Vertical from mid to target
            for y in mid_y..to_y {
                if y >= area.y && y < area.y + area.height && to_x >= area.x && to_x < area.x + area.width {
                    buf.set_string(to_x, y, "│", line_style);
                }
            }

            // Arrow at target
            if to_y > 0 && to_y - 1 >= area.y && to_y - 1 < area.y + area.height
                && to_x >= area.x && to_x < area.x + area.width
            {
                buf.set_string(to_x, to_y - 1, "▼", line_style);
            }
        }
    }

    /// Render the statistics footer
    fn render_stats(&self, area: Rect, buf: &mut Buffer) {
        let stats = self.graph.statistics();

        let stats_text = format!(
            "Nodes: {} │ Running: {} │ Completed: {} │ Failed: {} │ Pending: {}",
            stats.total, stats.running, stats.completed, stats.failed, stats.pending
        );

        let style = Style::default().fg(Color::Gray);
        buf.set_string(area.x, area.y, &stats_text, style);
    }
}

impl<'a> Widget for GraphView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render block if present
        let inner_area = if let Some(ref block) = self.block {
            let inner = block.inner(area);
            Widget::render(block.clone(), area, buf);
            inner
        } else {
            area
        };

        if inner_area.width < 10 || inner_area.height < 5 {
            return;
        }

        // Reserve space for stats footer
        let graph_area = Rect {
            x: inner_area.x,
            y: inner_area.y,
            width: inner_area.width,
            height: inner_area.height.saturating_sub(1),
        };

        let stats_area = Rect {
            x: inner_area.x,
            y: inner_area.y + inner_area.height - 1,
            width: inner_area.width,
            height: 1,
        };

        // Compute layout
        let layouts = self.compute_layout(graph_area);

        // Render connections first (behind nodes)
        self.render_connections(&layouts, graph_area, buf);

        // Render nodes
        for layout in &layouts {
            if let Some(task) = self.graph.get_task(&layout.task_id) {
                self.render_node(task, layout, graph_area, buf);
            }
        }

        // Render statistics
        self.render_stats(stats_area, buf);
    }
}

/// Progress bar widget for individual tasks
pub struct TaskProgressBar<'a> {
    /// Task reference
    task: &'a TaskNode,
    /// Style for the filled portion
    filled_style: Style,
    /// Style for the unfilled portion
    unfilled_style: Style,
    /// Progress ratio (0.0 - 1.0), None for indeterminate
    progress: Option<f32>,
}

impl<'a> TaskProgressBar<'a> {
    /// Create a new progress bar
    pub fn new(task: &'a TaskNode) -> Self {
        Self {
            task,
            filled_style: Style::default().fg(Color::Green),
            unfilled_style: Style::default().fg(Color::DarkGray),
            progress: None,
        }
    }

    /// Set the progress (0.0 - 1.0)
    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = Some(progress.clamp(0.0, 1.0));
        self
    }

    /// Set filled style
    pub fn filled_style(mut self, style: Style) -> Self {
        self.filled_style = style;
        self
    }
}

impl<'a> Widget for TaskProgressBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let width = area.width as usize;

        let bar = match self.progress {
            Some(p) => {
                let filled = (width as f32 * p) as usize;
                format!("{}{}", "█".repeat(filled), "░".repeat(width - filled))
            }
            None => {
                // Indeterminate: show animated pattern based on time
                let pattern: String = (0..width)
                    .map(|i| if i % 3 == 0 { '█' } else { '░' })
                    .collect();
                pattern
            }
        };

        let style = if self.task.status == TaskStatus::Running {
            self.filled_style.fg(Color::Yellow)
        } else if self.task.status == TaskStatus::Completed {
            self.filled_style.fg(Color::Green)
        } else if self.task.status == TaskStatus::Failed {
            self.filled_style.fg(Color::Red)
        } else {
            self.unfilled_style
        };

        buf.set_string(area.x, area.y, &bar, style);
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

        graph.add_dependency(&build_id, &test_id, TaskEdge::DependsOn).unwrap();
        graph.add_dependency(&build_id, &lint_id, TaskEdge::DependsOn).unwrap();

        graph
    }

    #[test]
    fn test_graph_view_creation() {
        let graph = create_test_graph();
        let _view = GraphView::new(&graph);
    }

    #[test]
    fn test_layout_computation() {
        let graph = create_test_graph();
        let view = GraphView::new(&graph);
        let area = Rect::new(0, 0, 80, 24);
        let layouts = view.compute_layout(area);
        assert_eq!(layouts.len(), 3);
    }
}
