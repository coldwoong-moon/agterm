//! Task Detail Popup Widget
//!
//! A popup widget for displaying detailed information about a task.

use crate::domain::task::{TaskNode, TaskResult, TaskStatus};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

/// Task detail popup widget
pub struct TaskDetail<'a> {
    /// Task to display
    task: &'a TaskNode,
    /// Optional block for borders/title
    block: Option<Block<'a>>,
}

impl<'a> TaskDetail<'a> {
    /// Create a new task detail popup
    pub fn new(task: &'a TaskNode) -> Self {
        Self { task, block: None }
    }

    /// Set the block for borders/title
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
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

    /// Build the content lines
    fn build_content(&self) -> Vec<Line<'a>> {
        let mut lines = Vec::new();
        let task = self.task;

        // Task name and status
        let status_color = Self::status_color(task.status);
        lines.push(Line::from(vec![
            Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&task.name),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(format!("{}", task.status), Style::default().fg(status_color)),
        ]));

        lines.push(Line::default()); // Empty line

        // Command
        lines.push(Line::from(vec![
            Span::styled("Command: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(task.full_command(), Style::default().fg(Color::Cyan)),
        ]));

        // Working directory
        lines.push(Line::from(vec![
            Span::styled("Working Dir: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(task.working_dir.display().to_string()),
        ]));

        lines.push(Line::default()); // Empty line

        // Timing information
        lines.push(Line::from(Span::styled(
            "Timing",
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )));

        lines.push(Line::from(vec![
            Span::raw("  Created: "),
            Span::styled(
                task.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                Style::default().fg(Color::Gray),
            ),
        ]));

        if let Some(started) = task.started_at {
            lines.push(Line::from(vec![
                Span::raw("  Started: "),
                Span::styled(
                    started.format("%Y-%m-%d %H:%M:%S").to_string(),
                    Style::default().fg(Color::Gray),
                ),
            ]));
        }

        if let Some(completed) = task.completed_at {
            lines.push(Line::from(vec![
                Span::raw("  Completed: "),
                Span::styled(
                    completed.format("%Y-%m-%d %H:%M:%S").to_string(),
                    Style::default().fg(Color::Gray),
                ),
            ]));
        }

        lines.push(Line::from(vec![
            Span::raw("  Duration: "),
            Span::styled(task.duration_str(), Style::default().fg(Color::Yellow)),
        ]));

        // Retry count
        if task.retry_count > 0 {
            lines.push(Line::from(vec![
                Span::raw("  Retries: "),
                Span::styled(
                    format!("{}", task.retry_count),
                    Style::default().fg(Color::Red),
                ),
            ]));
        }

        lines.push(Line::default()); // Empty line

        // Result (if available)
        if let Some(ref result) = task.result {
            lines.push(Line::from(Span::styled(
                "Result",
                Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )));

            lines.push(Line::from(vec![
                Span::raw("  Exit Code: "),
                Span::styled(
                    format!("{}", result.exit_code),
                    Style::default().fg(if result.is_success() {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                ),
            ]));

            lines.push(Line::from(vec![
                Span::raw("  Execution Time: "),
                Span::styled(
                    format!("{}ms", result.duration_ms),
                    Style::default().fg(Color::Gray),
                ),
            ]));

            // Output preview (first few lines)
            if !result.stdout.is_empty() {
                lines.push(Line::default());
                lines.push(Line::from(Span::styled(
                    "Output (preview):",
                    Style::default().add_modifier(Modifier::BOLD),
                )));

                for line in result.stdout.lines().take(5) {
                    lines.push(Line::from(Span::styled(
                        format!("  {}", line),
                        Style::default().fg(Color::Gray),
                    )));
                }

                if result.stdout.lines().count() > 5 {
                    lines.push(Line::from(Span::styled(
                        "  ... (truncated)",
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }

            // Error output
            if !result.stderr.is_empty() {
                lines.push(Line::default());
                lines.push(Line::from(Span::styled(
                    "Errors:",
                    Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
                )));

                for line in result.stderr.lines().take(5) {
                    lines.push(Line::from(Span::styled(
                        format!("  {}", line),
                        Style::default().fg(Color::Red),
                    )));
                }
            }
        }

        // Metadata
        if !task.metadata.is_empty() {
            lines.push(Line::default());
            lines.push(Line::from(Span::styled(
                "Metadata",
                Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )));

            for (key, value) in &task.metadata {
                lines.push(Line::from(vec![
                    Span::raw(format!("  {}: ", key)),
                    Span::styled(value, Style::default().fg(Color::Gray)),
                ]));
            }
        }

        lines
    }
}

impl<'a> Widget for TaskDetail<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear the area first (for popup effect)
        Widget::render(Clear, area, buf);

        let content = self.build_content();
        let paragraph = Paragraph::new(content).wrap(Wrap { trim: true });

        let paragraph = if let Some(block) = self.block {
            paragraph.block(block)
        } else {
            paragraph
        };

        Widget::render(paragraph, area, buf);
    }
}

/// Timer display widget
pub struct TaskTimer<'a> {
    /// Task to display timer for
    task: &'a TaskNode,
    /// Style for the timer
    style: Style,
}

impl<'a> TaskTimer<'a> {
    /// Create a new timer display
    pub fn new(task: &'a TaskNode) -> Self {
        Self {
            task,
            style: Style::default(),
        }
    }

    /// Set the style
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Format duration as HH:MM:SS
    fn format_duration(ms: u64) -> String {
        let total_secs = ms / 1000;
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;

        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{:02}:{:02}", minutes, seconds)
        }
    }
}

impl<'a> Widget for TaskTimer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let duration_ms = self.task.duration_ms().unwrap_or(0);
        let duration_str = Self::format_duration(duration_ms);

        let style = match self.task.status {
            TaskStatus::Running => self.style.fg(Color::Yellow),
            TaskStatus::Completed => self.style.fg(Color::Green),
            TaskStatus::Failed => self.style.fg(Color::Red),
            _ => self.style.fg(Color::Gray),
        };

        buf.set_string(area.x, area.y, &duration_str, style);
    }
}

/// ETA calculator for task completion
pub struct TaskEta {
    /// Total tasks
    total: usize,
    /// Completed tasks
    completed: usize,
    /// Average duration per task in milliseconds
    avg_duration_ms: u64,
}

impl TaskEta {
    /// Create a new ETA calculator
    pub fn new(total: usize, completed: usize, avg_duration_ms: u64) -> Self {
        Self {
            total,
            completed,
            avg_duration_ms,
        }
    }

    /// Calculate ETA in milliseconds
    pub fn eta_ms(&self) -> u64 {
        let remaining = self.total.saturating_sub(self.completed);
        remaining as u64 * self.avg_duration_ms
    }

    /// Format ETA as human-readable string
    pub fn format(&self) -> String {
        let eta_ms = self.eta_ms();
        let total_secs = eta_ms / 1000;

        if total_secs < 60 {
            format!("~{}s", total_secs)
        } else if total_secs < 3600 {
            format!("~{}m {}s", total_secs / 60, total_secs % 60)
        } else {
            format!(
                "~{}h {}m",
                total_secs / 3600,
                (total_secs % 3600) / 60
            )
        }
    }
}

impl Widget for TaskEta {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let eta_str = format!("ETA: {}", self.format());
        let style = Style::default().fg(Color::Cyan);

        buf.set_string(area.x, area.y, &eta_str, style);
    }
}

/// Calculate centered popup rect
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::TaskNode;

    #[test]
    fn test_task_detail_creation() {
        let task = TaskNode::new("Test", "echo hello");
        let _detail = TaskDetail::new(&task);
    }

    #[test]
    fn test_duration_format() {
        assert_eq!(TaskTimer::format_duration(0), "00:00");
        assert_eq!(TaskTimer::format_duration(30000), "00:30");
        assert_eq!(TaskTimer::format_duration(90000), "01:30");
        assert_eq!(TaskTimer::format_duration(3661000), "01:01:01");
    }

    #[test]
    fn test_eta_calculation() {
        let eta = TaskEta::new(10, 5, 60000); // 10 tasks, 5 done, 1 min avg
        assert_eq!(eta.eta_ms(), 300000); // 5 remaining * 60000ms = 5 min
    }

    #[test]
    fn test_centered_rect() {
        let area = Rect::new(0, 0, 100, 50);
        let popup = centered_rect(60, 60, area);

        assert!(popup.x > 0);
        assert!(popup.y > 0);
        assert!(popup.width < area.width);
        assert!(popup.height < area.height);
    }
}
