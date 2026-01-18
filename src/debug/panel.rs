//! Debug UI Panel
//!
//! A toggleable panel that displays:
//! - PTY session status
//! - Input debugging information
//! - Performance metrics
//! - Recent log entries

use super::{InputDebugState, Metrics, PtyDebugInfo};
use crate::logging::layers::LogBuffer;
use iced::widget::{column, container, row, scrollable, text, Space};
use iced::{Alignment, Border, Element, Font, Length};
use std::time::Instant;
use tracing::Level;

/// Monospace font for debug panel
const MONO_FONT: Font = Font::with_name("D2Coding");

/// Debug panel theme colors
mod colors {
    use iced::Color;

    pub const BG_PANEL: Color = Color::from_rgba(0.05, 0.05, 0.08, 0.95);
    pub const BG_SECTION: Color = Color::from_rgba(0.1, 0.1, 0.13, 0.9);
    pub const BORDER: Color = Color::from_rgb(0.25, 0.25, 0.3);
    pub const TEXT_TITLE: Color = Color::from_rgb(0.8, 0.85, 1.0);
    pub const TEXT_LABEL: Color = Color::from_rgb(0.6, 0.65, 0.7);
    pub const TEXT_VALUE: Color = Color::from_rgb(0.9, 0.9, 0.95);
    pub const ACCENT_GREEN: Color = Color::from_rgb(0.4, 0.9, 0.5);
    pub const ACCENT_YELLOW: Color = Color::from_rgb(0.95, 0.8, 0.3);
    pub const ACCENT_RED: Color = Color::from_rgb(0.95, 0.4, 0.4);
    pub const ACCENT_BLUE: Color = Color::from_rgb(0.4, 0.6, 1.0);
    pub const ACCENT_CYAN: Color = Color::from_rgb(0.4, 0.85, 0.9);
}

/// Get current memory usage in MB (platform-specific)
pub fn get_memory_usage_mb() -> f64 {
    // TODO: Implement memory tracking
    // Memory tracking via mach2 API is complex and needs proper testing
    0.0
}

/// Debug panel messages
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum DebugPanelMessage {
    /// Toggle panel visibility
    Toggle,
    /// Set log level filter
    SetLogFilter(Level),
    /// Clear log buffer
    ClearLogs,
    /// Search logs
    SearchLogs(String),
}

/// Terminal state information
#[derive(Debug, Clone, Default)]
pub struct TerminalState {
    /// Terminal dimensions (cols x rows)
    pub cols: usize,
    pub rows: usize,
    /// Cursor position (row, col)
    pub cursor_row: usize,
    pub cursor_col: usize,
    /// Scrollback buffer size
    pub scrollback_size: usize,
    /// Total line count (visible + scrollback)
    pub total_lines: usize,
}

/// Debug panel state
pub struct DebugPanel {
    /// Panel visibility
    pub visible: bool,
    /// Performance metrics
    pub metrics: Metrics,
    /// Input debug state
    pub input_state: InputDebugState,
    /// PTY session info
    pub pty_sessions: Vec<PtyDebugInfo>,
    /// Terminal state
    pub terminal_state: TerminalState,
    /// Log buffer handle (optional, set after initialization)
    pub log_buffer: Option<LogBuffer>,
    /// Current log level filter
    pub log_filter: Level,
    /// Log search query
    pub log_search: String,
    /// Panel creation time
    pub created_at: Instant,
}

impl Default for DebugPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugPanel {
    /// Create a new debug panel
    pub fn new() -> Self {
        Self {
            visible: std::env::var("AGTERM_DEBUG")
                .map(|v| v == "1")
                .unwrap_or(false),
            metrics: Metrics::default(),
            input_state: InputDebugState::default(),
            pty_sessions: Vec::new(),
            terminal_state: TerminalState::default(),
            log_buffer: None,
            log_filter: Level::DEBUG,
            log_search: String::new(),
            created_at: Instant::now(),
        }
    }

    /// Toggle panel visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        tracing::debug!(visible = self.visible, "Debug panel toggled");
    }

    /// Set the log buffer handle
    pub fn set_log_buffer(&mut self, buffer: LogBuffer) {
        self.log_buffer = Some(buffer);
    }

    /// Update PTY session info
    #[allow(dead_code)]
    pub fn update_pty_session(&mut self, info: PtyDebugInfo) {
        if let Some(existing) = self
            .pty_sessions
            .iter_mut()
            .find(|s| s.session_id == info.session_id)
        {
            *existing = info;
        } else {
            self.pty_sessions.push(info);
        }
    }

    /// Remove a PTY session
    #[allow(dead_code)]
    pub fn remove_pty_session(&mut self, session_id: &str) {
        self.pty_sessions.retain(|s| s.session_id != session_id);
    }

    /// Handle debug panel messages
    pub fn update(&mut self, message: DebugPanelMessage) {
        match message {
            DebugPanelMessage::Toggle => self.toggle(),
            DebugPanelMessage::SetLogFilter(level) => {
                self.log_filter = level;
            }
            DebugPanelMessage::ClearLogs => {
                if let Some(ref buffer) = self.log_buffer {
                    buffer.clear();
                }
            }
            DebugPanelMessage::SearchLogs(query) => {
                self.log_search = query;
            }
        }
    }

    /// Render the debug panel
    pub fn view<'a, M: 'a + Clone>(&'a self) -> Element<'a, M> {
        if !self.visible {
            return Space::new(0, 0).into();
        }

        // Performance section
        let perf_section: Element<'a, M> = self.render_performance_section();

        // Terminal section
        let terminal_section: Element<'a, M> = self.render_terminal_section();

        // PTY section
        let pty_section: Element<'a, M> = self.render_pty_section();

        // Input section
        let input_section: Element<'a, M> = self.render_input_section();

        // Log section
        let log_section: Element<'a, M> = self.render_log_section();

        // Header
        let header: Element<'a, M> = self.render_header();

        // Main panel content
        let content = column![
            header,
            Space::with_height(8),
            perf_section,
            Space::with_height(8),
            terminal_section,
            Space::with_height(8),
            pty_section,
            Space::with_height(8),
            input_section,
            Space::with_height(8),
            log_section,
        ]
        .spacing(4)
        .padding(12)
        .width(Length::Fixed(350.0));

        container(scrollable(content).height(Length::Fill))
            .style(|_| container::Style {
                background: Some(colors::BG_PANEL.into()),
                border: Border {
                    color: colors::BORDER,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            })
            .height(Length::Fill)
            .into()
    }

    fn render_header<'a, M: 'a>(&'a self) -> Element<'a, M> {
        let uptime = self.created_at.elapsed();
        let uptime_str = format!(
            "{}:{:02}:{:02}",
            uptime.as_secs() / 3600,
            (uptime.as_secs() % 3600) / 60,
            uptime.as_secs() % 60
        );

        row![
            text("Debug Panel").size(16).color(colors::TEXT_TITLE),
            Space::with_width(Length::Fill),
            text(uptime_str).size(12).color(colors::TEXT_LABEL),
        ]
        .align_y(Alignment::Center)
        .into()
    }

    fn render_performance_section<'a, M: 'a>(&'a self) -> Element<'a, M> {
        let fps = self.metrics.fps();
        let fps_color = if fps >= 55.0 {
            colors::ACCENT_GREEN
        } else if fps >= 30.0 {
            colors::ACCENT_YELLOW
        } else {
            colors::ACCENT_RED
        };

        let frame_time = self.metrics.avg_frame_time_ms();
        let render_time = self.metrics.avg_render_time_ms();
        let msg_time = self.metrics.avg_message_time_us();

        // Get memory usage (estimate based on Rust's allocator)
        let memory_mb = get_memory_usage_mb();

        let mut content = column![
            text("Performance").size(13).color(colors::TEXT_TITLE),
            Space::with_height(4),
            row![
                text("FPS:").size(11).color(colors::TEXT_LABEL),
                Space::with_width(8),
                text(format!("{:.1}", fps))
                    .size(11)
                    .font(MONO_FONT)
                    .color(fps_color),
            ],
        ]
        .spacing(2);

        // FPS History Graph (sparkline)
        let fps_history = self.metrics.fps_history();
        if !fps_history.is_empty() {
            let fps_data: Vec<f64> = fps_history.iter().copied().collect();
            let sparkline = render_sparkline(&fps_data, 40);
            content = content.push(row![
                text("   ").size(11),
                text(sparkline)
                    .size(11)
                    .font(MONO_FONT)
                    .color(colors::ACCENT_CYAN),
            ]);

            // Show min/max range
            if let (Some(&min_fps), Some(&max_fps)) = (
                fps_data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()),
                fps_data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()),
            ) {
                content = content.push(row![text(format!(
                    "   {:.0}-{:.0} (60s)",
                    min_fps, max_fps
                ))
                .size(9)
                .color(colors::TEXT_LABEL),]);
            }
        }

        content = content.push(Space::with_height(4));

        content = content
            .push(row![
                text("Frame:").size(11).color(colors::TEXT_LABEL),
                Space::with_width(8),
                text(format!("{:.2}ms", frame_time))
                    .size(11)
                    .font(MONO_FONT)
                    .color(colors::TEXT_VALUE),
            ])
            .push(row![
                text("Render:").size(11).color(colors::TEXT_LABEL),
                Space::with_width(8),
                text(format!("{:.2}ms", render_time))
                    .size(11)
                    .font(MONO_FONT)
                    .color(colors::TEXT_VALUE),
            ])
            .push(row![
                text("Msg:").size(11).color(colors::TEXT_LABEL),
                Space::with_width(8),
                text(format!("{:.1}\u{00B5}s", msg_time))
                    .size(11)
                    .font(MONO_FONT)
                    .color(colors::TEXT_VALUE),
            ])
            .push(Space::with_height(4))
            .push(row![
                text("Memory:").size(11).color(colors::TEXT_LABEL),
                Space::with_width(8),
                text(format!("{:.1}MB", memory_mb))
                    .size(11)
                    .font(MONO_FONT)
                    .color(colors::TEXT_VALUE),
            ]);

        // Memory History Graph (sparkline)
        let memory_history = self.metrics.memory_history();
        if !memory_history.is_empty() {
            let mem_data: Vec<f64> = memory_history.iter().copied().collect();
            let sparkline = render_sparkline(&mem_data, 40);
            content = content.push(row![
                text("   ").size(11),
                text(sparkline)
                    .size(11)
                    .font(MONO_FONT)
                    .color(colors::ACCENT_YELLOW),
            ]);
        }

        self.section_container(content)
    }

    fn render_terminal_section<'a, M: 'a>(&'a self) -> Element<'a, M> {
        let content = column![
            text("Terminal").size(13).color(colors::TEXT_TITLE),
            Space::with_height(4),
            row![
                text("Size:").size(11).color(colors::TEXT_LABEL),
                Space::with_width(8),
                text(format!(
                    "{}x{}",
                    self.terminal_state.cols, self.terminal_state.rows
                ))
                .size(11)
                .font(MONO_FONT)
                .color(colors::ACCENT_CYAN),
            ],
            row![
                text("Cursor:").size(11).color(colors::TEXT_LABEL),
                Space::with_width(8),
                text(format!(
                    "({}, {})",
                    self.terminal_state.cursor_row, self.terminal_state.cursor_col
                ))
                .size(11)
                .font(MONO_FONT)
                .color(colors::TEXT_VALUE),
            ],
            row![
                text("Lines:").size(11).color(colors::TEXT_LABEL),
                Space::with_width(8),
                text(format!("{}", self.terminal_state.total_lines))
                    .size(11)
                    .font(MONO_FONT)
                    .color(colors::TEXT_VALUE),
            ],
            row![
                text("Scrollback:").size(11).color(colors::TEXT_LABEL),
                Space::with_width(8),
                text(format!("{} lines", self.terminal_state.scrollback_size))
                    .size(11)
                    .font(MONO_FONT)
                    .color(colors::TEXT_VALUE),
            ],
        ]
        .spacing(2);

        self.section_container(content)
    }

    fn render_pty_section<'a, M: 'a>(&'a self) -> Element<'a, M> {
        let sessions_count = self.pty_sessions.len();
        let total_read = self.metrics.total_pty_bytes_read();
        let total_written = self.metrics.total_pty_bytes_written();
        let read_rate = self.metrics.pty_read_bytes_per_sec();

        let mut content = column![
            text("PTY Sessions").size(13).color(colors::TEXT_TITLE),
            Space::with_height(4),
            row![
                text("Active:").size(11).color(colors::TEXT_LABEL),
                Space::with_width(8),
                text(format!("{}", sessions_count))
                    .size(11)
                    .font(MONO_FONT)
                    .color(colors::ACCENT_GREEN),
            ],
            row![
                text("Read:").size(11).color(colors::TEXT_LABEL),
                Space::with_width(8),
                text(format_bytes(total_read))
                    .size(11)
                    .font(MONO_FONT)
                    .color(colors::TEXT_VALUE),
                text(format!(" ({}/s)", format_bytes(read_rate as usize)))
                    .size(10)
                    .color(colors::TEXT_LABEL),
            ],
        ]
        .spacing(2);

        // PTY I/O Rate Graph (sparkline)
        let pty_io_history = self.metrics.pty_io_history();
        if !pty_io_history.is_empty() {
            let io_data: Vec<f64> = pty_io_history.iter().copied().collect();
            let sparkline = render_sparkline(&io_data, 40);
            content = content.push(row![
                text("   ").size(11),
                text(sparkline)
                    .size(11)
                    .font(MONO_FONT)
                    .color(colors::ACCENT_GREEN),
            ]);

            // Show max rate
            if let Some(&max_rate) = io_data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
                content = content.push(row![text(format!(
                    "   max: {}/s (60s)",
                    format_bytes(max_rate as usize)
                ))
                .size(9)
                .color(colors::TEXT_LABEL),]);
            }
        }

        content = content.push(Space::with_height(4));

        content = content.push(row![
            text("Write:").size(11).color(colors::TEXT_LABEL),
            Space::with_width(8),
            text(format_bytes(total_written))
                .size(11)
                .font(MONO_FONT)
                .color(colors::TEXT_VALUE),
        ]);

        // Show individual session info
        if !self.pty_sessions.is_empty() {
            content = content.push(Space::with_height(4));
            for session in &self.pty_sessions {
                let status_color = if session.active {
                    colors::ACCENT_GREEN
                } else {
                    colors::ACCENT_RED
                };
                let id_short = &session.session_id[..8.min(session.session_id.len())];
                content = content.push(row![
                    text(format!("  {}", id_short))
                        .size(10)
                        .font(MONO_FONT)
                        .color(colors::TEXT_LABEL),
                    Space::with_width(4),
                    text(if session.active { "●" } else { "○" })
                        .size(10)
                        .color(status_color),
                    Space::with_width(4),
                    text(format_bytes(session.buffer_size))
                        .size(10)
                        .font(MONO_FONT)
                        .color(colors::TEXT_VALUE),
                ]);
            }
        }

        self.section_container(content)
    }

    fn render_input_section<'a, M: 'a>(&'a self) -> Element<'a, M> {
        let key_display = self.input_state.last_key.as_deref().unwrap_or("-");
        let mods_display = self.input_state.last_modifiers.as_deref().unwrap_or("-");
        let ime_status = if self.input_state.ime_composing {
            format!("Composing: {}", self.input_state.ime_preedit)
        } else {
            "Idle".to_string()
        };
        let mode_color = if self.input_state.raw_mode {
            colors::ACCENT_CYAN
        } else {
            colors::ACCENT_BLUE
        };
        let mode_str = if self.input_state.raw_mode {
            "RAW"
        } else {
            "BLOCK"
        };

        let content = column![
            text("Input").size(13).color(colors::TEXT_TITLE),
            Space::with_height(4),
            row![
                text("Mode:").size(11).color(colors::TEXT_LABEL),
                Space::with_width(8),
                text(mode_str).size(11).font(MONO_FONT).color(mode_color),
            ],
            row![
                text("Key:").size(11).color(colors::TEXT_LABEL),
                Space::with_width(8),
                text(key_display)
                    .size(11)
                    .font(MONO_FONT)
                    .color(colors::TEXT_VALUE),
                Space::with_width(8),
                text(format!("[{}]", mods_display))
                    .size(10)
                    .color(colors::TEXT_LABEL),
            ],
            row![
                text("IME:").size(11).color(colors::TEXT_LABEL),
                Space::with_width(8),
                text(ime_status)
                    .size(11)
                    .font(MONO_FONT)
                    .color(colors::TEXT_VALUE),
            ],
        ]
        .spacing(2);

        self.section_container(content)
    }

    fn render_log_section<'a, M: 'a>(&'a self) -> Element<'a, M> {
        let mut content = column![
            text("Logs").size(13).color(colors::TEXT_TITLE),
            Space::with_height(4),
        ]
        .spacing(1);

        // Get log entries if buffer is available
        if let Some(ref buffer) = self.log_buffer {
            let entries = if self.log_search.is_empty() {
                buffer.get_recent(30)
            } else {
                buffer.search(&self.log_search)
            };

            let filtered: Vec<_> = entries
                .into_iter()
                .filter(|e| e.level <= self.log_filter)
                .collect();

            if filtered.is_empty() {
                content = content.push(text("No logs").size(10).color(colors::TEXT_LABEL));
            } else {
                for entry in filtered.iter().rev().take(20) {
                    let level_color = match entry.level {
                        Level::ERROR => colors::ACCENT_RED,
                        Level::WARN => colors::ACCENT_YELLOW,
                        Level::INFO => colors::ACCENT_GREEN,
                        Level::DEBUG => colors::ACCENT_BLUE,
                        Level::TRACE => colors::TEXT_LABEL,
                    };

                    let elapsed = entry.timestamp.elapsed();
                    let time_str = if elapsed.as_secs() < 60 {
                        format!("{}s", elapsed.as_secs())
                    } else {
                        format!("{}m", elapsed.as_secs() / 60)
                    };

                    let target_short = entry.target.split("::").last().unwrap_or(&entry.target);
                    let msg_truncated = if entry.message.len() > 40 {
                        format!("{}...", &entry.message[..37])
                    } else {
                        entry.message.clone()
                    };

                    content = content.push(row![
                        text(time_str)
                            .size(9)
                            .font(MONO_FONT)
                            .color(colors::TEXT_LABEL),
                        Space::with_width(4),
                        text(entry.level_str())
                            .size(9)
                            .font(MONO_FONT)
                            .color(level_color),
                        Space::with_width(4),
                        text(format!("[{}]", target_short))
                            .size(9)
                            .color(colors::TEXT_LABEL),
                        Space::with_width(4),
                        text(msg_truncated)
                            .size(9)
                            .font(MONO_FONT)
                            .color(colors::TEXT_VALUE),
                    ]);
                }
            }
        } else {
            content = content.push(
                text("Log buffer not initialized")
                    .size(10)
                    .color(colors::ACCENT_YELLOW),
            );
        }

        self.section_container(content)
    }

    fn section_container<'a, M: 'a>(
        &'a self,
        content: iced::widget::Column<'a, M>,
    ) -> Element<'a, M> {
        container(content.padding(8))
            .width(Length::Fill)
            .style(|_| container::Style {
                background: Some(colors::BG_SECTION.into()),
                border: Border {
                    color: colors::BORDER,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
            .into()
    }
}

/// Format bytes for display
fn format_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Render a text-based ASCII bar graph
///
/// Creates a simple bar graph from a data series, fitting within the specified width.
/// Each bar represents one data point scaled to the height range.
fn render_ascii_graph(data: &[f64], width: usize, height: usize) -> Vec<String> {
    if data.is_empty() || width == 0 || height == 0 {
        return vec![" ".repeat(width); height];
    }

    // Find min and max for scaling
    let max_val = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let min_val = data.iter().copied().fold(f64::INFINITY, f64::min);
    let range = (max_val - min_val).max(0.001); // Avoid division by zero

    // Characters for vertical bars (from empty to full)
    const BLOCKS: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    // Build the graph from bottom to top
    let mut lines = vec![String::new(); height];

    // Sample data to fit width
    let samples: Vec<f64> = if data.len() <= width {
        // Pad with zeros if data is shorter than width
        let mut padded = vec![0.0; width - data.len()];
        padded.extend_from_slice(data);
        padded
    } else {
        // Downsample if data is longer than width
        (0..width)
            .map(|i| {
                let idx = (i * data.len()) / width;
                data[idx]
            })
            .collect()
    };

    // Draw bars
    for (_x, &value) in samples.iter().enumerate() {
        // Normalize value to 0-1 range
        let normalized = if range > 0.0 {
            ((value - min_val) / range).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Calculate bar height in character units
        let bar_height = normalized * (height as f64);
        let full_blocks = bar_height.floor() as usize;
        let partial = ((bar_height - bar_height.floor()) * 8.0) as usize;

        // Fill from bottom up
        for y in 0..height {
            let row_from_bottom = height - 1 - y;
            let ch = if row_from_bottom < full_blocks {
                '█'
            } else if row_from_bottom == full_blocks && partial > 0 {
                BLOCKS[partial]
            } else {
                ' '
            };
            lines[y].push(ch);
        }
    }

    lines
}

/// Render a sparkline graph (single line with Unicode block characters)
fn render_sparkline(data: &[f64], width: usize) -> String {
    if data.is_empty() || width == 0 {
        return " ".repeat(width);
    }

    // Find min and max for scaling
    let max_val = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let min_val = data.iter().copied().fold(f64::INFINITY, f64::min);
    let range = (max_val - min_val).max(0.001);

    // Sparkline characters (8 levels)
    const SPARKS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    // Sample data to fit width
    let samples: Vec<f64> = if data.len() <= width {
        let mut padded = vec![0.0; width - data.len()];
        padded.extend_from_slice(data);
        padded
    } else {
        (0..width)
            .map(|i| {
                let idx = (i * data.len()) / width;
                data[idx]
            })
            .collect()
    };

    samples
        .iter()
        .map(|&value| {
            let normalized = if range > 0.0 {
                ((value - min_val) / range).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let level = (normalized * 7.0).round() as usize;
            SPARKS[level.min(7)]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(100), "100B");
        assert_eq!(format_bytes(1024), "1.0KB");
        assert_eq!(format_bytes(1536), "1.5KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0MB");
    }

    #[test]
    fn test_debug_panel_toggle() {
        let mut panel = DebugPanel::new();
        let initial = panel.visible;
        panel.toggle();
        assert_ne!(panel.visible, initial);
        panel.toggle();
        assert_eq!(panel.visible, initial);
    }

    #[test]
    fn test_debug_panel_env_var() {
        // This test depends on environment state, so we just verify the struct is created
        let panel = DebugPanel::new();
        // visible state depends on AGTERM_DEBUG env var
        assert!(panel.metrics.fps() >= 0.0);
    }

    #[test]
    fn test_render_sparkline() {
        let data = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let sparkline = render_sparkline(&data, 5);
        // Use chars().count() for Unicode character count (not byte count)
        assert_eq!(sparkline.chars().count(), 5);
        // Should contain Unicode sparkline characters
        assert!(sparkline.chars().all(|c| c >= '▁' && c <= '█' || c == ' '));
    }

    #[test]
    fn test_render_sparkline_empty() {
        let data: Vec<f64> = vec![];
        let sparkline = render_sparkline(&data, 10);
        assert_eq!(sparkline.chars().count(), 10);
        assert_eq!(sparkline, "          ");
    }

    #[test]
    fn test_render_ascii_graph() {
        let data = vec![1.0, 2.0, 3.0, 2.0, 1.0];
        let graph = render_ascii_graph(&data, 5, 5);
        assert_eq!(graph.len(), 5); // Height of 5 lines
                                    // Use chars().count() for Unicode character count
        assert!(graph.iter().all(|line| line.chars().count() == 5)); // Width of 5 chars
    }

    #[test]
    fn test_render_ascii_graph_empty() {
        let data: Vec<f64> = vec![];
        let graph = render_ascii_graph(&data, 10, 5);
        assert_eq!(graph.len(), 5);
        assert!(graph.iter().all(|line| line == "          "));
    }
}
