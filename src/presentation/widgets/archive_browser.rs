//! Archive Browser Widget
//!
//! TUI widget for browsing and searching session archives.

use crate::domain::session::{CompressionLevel, SessionArchive};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget, Wrap,
    },
};

/// Archive browser state
#[derive(Debug, Default)]
pub struct ArchiveBrowserState {
    /// Selected archive index
    pub selected: usize,
    /// List state for scrolling
    pub list_state: ListState,
    /// Scrollbar state
    pub scrollbar_state: ScrollbarState,
    /// Search query
    pub search_query: String,
    /// Whether search input is focused
    pub search_focused: bool,
    /// Detail view open
    pub detail_open: bool,
    /// Archives list
    archives: Vec<SessionArchive>,
    /// Filtered archives (indices into main list)
    filtered_indices: Vec<usize>,
}

impl ArchiveBrowserState {
    /// Create new state
    pub fn new() -> Self {
        Self::default()
    }

    /// Set archives list
    pub fn set_archives(&mut self, archives: Vec<SessionArchive>) {
        self.archives = archives;
        self.filtered_indices = (0..self.archives.len()).collect();
        self.selected = 0;
        self.update_list_state();
    }

    /// Get all archives
    pub fn archives(&self) -> &[SessionArchive] {
        &self.archives
    }

    /// Get filtered archives
    pub fn filtered_archives(&self) -> Vec<&SessionArchive> {
        self.filtered_indices
            .iter()
            .filter_map(|&i| self.archives.get(i))
            .collect()
    }

    /// Get currently selected archive
    pub fn selected_archive(&self) -> Option<&SessionArchive> {
        self.filtered_indices
            .get(self.selected)
            .and_then(|&i| self.archives.get(i))
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.update_list_state();
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.selected + 1 < self.filtered_indices.len() {
            self.selected += 1;
            self.update_list_state();
        }
    }

    /// Move to first item
    pub fn select_first(&mut self) {
        self.selected = 0;
        self.update_list_state();
    }

    /// Move to last item
    pub fn select_last(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected = self.filtered_indices.len() - 1;
            self.update_list_state();
        }
    }

    /// Update list state to match selection
    fn update_list_state(&mut self) {
        self.list_state.select(Some(self.selected));
        self.scrollbar_state = self
            .scrollbar_state
            .content_length(self.filtered_indices.len())
            .position(self.selected);
    }

    /// Set search query and filter
    pub fn set_search(&mut self, query: String) {
        self.search_query = query.to_lowercase();
        self.apply_filter();
    }

    /// Apply current filter
    fn apply_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_indices = (0..self.archives.len()).collect();
        } else {
            self.filtered_indices = self
                .archives
                .iter()
                .enumerate()
                .filter(|(_, archive)| {
                    archive.summary.to_lowercase().contains(&self.search_query)
                        || archive
                            .tags
                            .iter()
                            .any(|t| t.to_lowercase().contains(&self.search_query))
                        || archive
                            .working_dir
                            .to_string_lossy()
                            .to_lowercase()
                            .contains(&self.search_query)
                })
                .map(|(i, _)| i)
                .collect();
        }

        // Reset selection if out of bounds
        if self.selected >= self.filtered_indices.len() {
            self.selected = self.filtered_indices.len().saturating_sub(1);
        }
        self.update_list_state();
    }

    /// Toggle detail view
    pub fn toggle_detail(&mut self) {
        self.detail_open = !self.detail_open;
    }

    /// Toggle search focus
    pub fn toggle_search(&mut self) {
        self.search_focused = !self.search_focused;
    }

    /// Clear search
    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.apply_filter();
    }
}

/// Archive browser widget
pub struct ArchiveBrowser<'a> {
    /// Widget title
    title: &'a str,
    /// Border style
    border_style: Style,
    /// Selected item style
    selected_style: Style,
    /// Show search bar
    show_search: bool,
}

impl<'a> Default for ArchiveBrowser<'a> {
    fn default() -> Self {
        Self {
            title: "Archives",
            border_style: Style::default(),
            selected_style: Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            show_search: true,
        }
    }
}

impl<'a> ArchiveBrowser<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    pub fn selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    pub fn show_search(mut self, show: bool) -> Self {
        self.show_search = show;
        self
    }
}

impl<'a> StatefulWidget for ArchiveBrowser<'a> {
    type State = ArchiveBrowserState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Layout
        let chunks = if self.show_search {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0)])
                .split(area)
        };

        let (search_area, list_area) = if self.show_search {
            (Some(chunks[0]), chunks[1])
        } else {
            (None, chunks[0])
        };

        // Render search bar
        if let Some(search_area) = search_area {
            let search_style = if state.search_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            let search_text = if state.search_query.is_empty() && !state.search_focused {
                "Press / to search..."
            } else {
                &state.search_query
            };

            let search = Paragraph::new(search_text)
                .style(search_style)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Search")
                        .border_style(if state.search_focused {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default()
                        }),
                );

            search.render(search_area, buf);
        }

        // Build list items
        let items: Vec<ListItem> = state
            .filtered_archives()
            .iter()
            .map(|archive| {
                let compression_icon = match archive.compression_level {
                    CompressionLevel::Raw => "●",
                    CompressionLevel::Compacted => "◐",
                    CompressionLevel::Summarized => "○",
                    CompressionLevel::Rolled => "◎",
                };

                let date = archive.period.0.format("%Y-%m-%d %H:%M");
                let dir = archive
                    .working_dir
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| archive.working_dir.to_string_lossy().to_string());

                // Truncate summary
                let summary: String = archive
                    .summary
                    .chars()
                    .take(50)
                    .collect::<String>()
                    .replace('\n', " ");

                let line = Line::from(vec![
                    Span::styled(
                        format!("{} ", compression_icon),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(format!("{} ", date), Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("[{}] ", dir), Style::default().fg(Color::Green)),
                    Span::raw(summary),
                ]);

                ListItem::new(line)
            })
            .collect();

        // Render list
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(
                        "{} ({}/{})",
                        self.title,
                        state.filtered_indices.len(),
                        state.archives.len()
                    ))
                    .border_style(self.border_style),
            )
            .highlight_style(self.selected_style)
            .highlight_symbol("▶ ");

        StatefulWidget::render(list, list_area, buf, &mut state.list_state);

        // Render scrollbar
        if state.filtered_indices.len() > list_area.height as usize - 2 {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            let scrollbar_area = Rect {
                x: list_area.x + list_area.width - 1,
                y: list_area.y + 1,
                width: 1,
                height: list_area.height - 2,
            };

            StatefulWidget::render(scrollbar, scrollbar_area, buf, &mut state.scrollbar_state);
        }

        // Render detail popup if open
        if state.detail_open {
            if let Some(archive) = state.selected_archive() {
                render_archive_detail(archive, area, buf);
            }
        }
    }
}

/// Render archive detail popup
fn render_archive_detail(archive: &SessionArchive, area: Rect, buf: &mut Buffer) {
    // Calculate popup area (centered, 80% width, 70% height)
    let popup_width = (area.width as f32 * 0.8) as u16;
    let popup_height = (area.height as f32 * 0.7) as u16;
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear background
    Clear.render(popup_area, buf);

    // Build content
    let compression_str = match archive.compression_level {
        CompressionLevel::Raw => "Raw",
        CompressionLevel::Compacted => "Compacted",
        CompressionLevel::Summarized => "Summarized",
        CompressionLevel::Rolled => "Rolled",
    };

    let duration = archive.duration_secs();
    let duration_str = if duration < 60.0 {
        format!("{:.1}s", duration)
    } else if duration < 3600.0 {
        format!("{:.1}m", duration / 60.0)
    } else {
        format!("{:.1}h", duration / 3600.0)
    };

    let tags_str = if archive.tags.is_empty() {
        "None".to_string()
    } else {
        archive.tags.join(", ")
    };

    let metrics = &archive.metrics;
    let success_rate = if metrics.total_tasks > 0 {
        (metrics.successful_tasks as f64 / metrics.total_tasks as f64 * 100.0) as u32
    } else {
        0
    };

    let content = vec![
        Line::from(vec![
            Span::styled("Session ID: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(archive.session_id.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Working Dir: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(archive.working_dir.to_string_lossy()),
        ]),
        Line::from(vec![
            Span::styled("Period: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!(
                "{} → {} ({})",
                archive.period.0.format("%Y-%m-%d %H:%M"),
                archive.period.1.format("%H:%M"),
                duration_str
            )),
        ]),
        Line::from(vec![
            Span::styled("Compression: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(compression_str),
        ]),
        Line::from(vec![
            Span::styled("Tags: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(tags_str, Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "─── Metrics ───",
            Style::default().fg(Color::Cyan),
        )]),
        Line::from(vec![
            Span::styled("Tasks: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!(
                "{} total ({} success, {} failed, {} cancelled)",
                metrics.total_tasks,
                metrics.successful_tasks,
                metrics.failed_tasks,
                metrics.cancelled_tasks
            )),
        ]),
        Line::from(vec![
            Span::styled("Success Rate: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{}%", success_rate),
                if success_rate >= 80 {
                    Style::default().fg(Color::Green)
                } else if success_rate >= 50 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Red)
                },
            ),
        ]),
        Line::from(vec![
            Span::styled("Compression Ratio: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{:.1}%", metrics.compression_ratio() * 100.0)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "─── Summary ───",
            Style::default().fg(Color::Cyan),
        )]),
    ];

    // Split content and summary
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(popup_area);

    let inner_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(content.len() as u16),
            Constraint::Min(3),
        ])
        .split(layout[0]);

    // Render popup border
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Archive Details")
        .title_bottom("Press Enter or q to close")
        .border_style(Style::default().fg(Color::Cyan));

    block.render(popup_area, buf);

    // Render metadata
    let metadata = Paragraph::new(content);
    metadata.render(inner_layout[0], buf);

    // Render summary with wrapping
    let summary = Paragraph::new(archive.summary.as_str())
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::White));
    summary.render(inner_layout[1], buf);
}

/// Compression level indicator widget
pub struct CompressionIndicator {
    level: CompressionLevel,
}

impl CompressionIndicator {
    pub fn new(level: CompressionLevel) -> Self {
        Self { level }
    }
}

impl Widget for CompressionIndicator {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (icon, color) = match self.level {
            CompressionLevel::Raw => ("●", Color::Green),
            CompressionLevel::Compacted => ("◐", Color::Yellow),
            CompressionLevel::Summarized => ("○", Color::Cyan),
            CompressionLevel::Rolled => ("◎", Color::Magenta),
        };

        let span = Span::styled(icon, Style::default().fg(color));
        buf.set_span(area.x, area.y, &span, area.width);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::session::Session;
    use std::path::PathBuf;

    fn create_test_archives() -> Vec<SessionArchive> {
        vec![
            SessionArchive::from_session(
                &Session::new(PathBuf::from("/home/user/project1")),
                "Implemented new feature".to_string(),
                vec!["rust".to_string(), "feature".to_string()],
            ),
            SessionArchive::from_session(
                &Session::new(PathBuf::from("/home/user/project2")),
                "Fixed bug in parser".to_string(),
                vec!["bugfix".to_string()],
            ),
            SessionArchive::from_session(
                &Session::new(PathBuf::from("/home/user/project1")),
                "Refactored database layer".to_string(),
                vec!["refactor".to_string(), "database".to_string()],
            ),
        ]
    }

    #[test]
    fn test_archive_browser_state() {
        let mut state = ArchiveBrowserState::new();
        state.set_archives(create_test_archives());

        assert_eq!(state.archives().len(), 3);
        assert_eq!(state.filtered_archives().len(), 3);
        assert!(state.selected_archive().is_some());
    }

    #[test]
    fn test_navigation() {
        let mut state = ArchiveBrowserState::new();
        state.set_archives(create_test_archives());

        assert_eq!(state.selected, 0);

        state.select_next();
        assert_eq!(state.selected, 1);

        state.select_next();
        assert_eq!(state.selected, 2);

        state.select_next(); // Should not go past end
        assert_eq!(state.selected, 2);

        state.select_previous();
        assert_eq!(state.selected, 1);

        state.select_first();
        assert_eq!(state.selected, 0);

        state.select_last();
        assert_eq!(state.selected, 2);
    }

    #[test]
    fn test_search_filter() {
        let mut state = ArchiveBrowserState::new();
        state.set_archives(create_test_archives());

        state.set_search("rust".to_string());
        assert_eq!(state.filtered_archives().len(), 1);

        state.set_search("project1".to_string());
        assert_eq!(state.filtered_archives().len(), 2);

        state.clear_search();
        assert_eq!(state.filtered_archives().len(), 3);
    }

    #[test]
    fn test_toggle_detail() {
        let mut state = ArchiveBrowserState::new();

        assert!(!state.detail_open);

        state.toggle_detail();
        assert!(state.detail_open);

        state.toggle_detail();
        assert!(!state.detail_open);
    }
}
