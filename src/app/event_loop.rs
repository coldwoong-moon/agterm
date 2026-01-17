//! Main Event Loop
//!
//! Handles the main application loop including input, PTY I/O, and rendering.

use crate::app::state::AppState;
use crate::error::Result;
use crate::infrastructure::pty::{PtyId, PtyPool, PtyPoolConfig, PtySessionConfig};
use crate::presentation::layout::{LayoutManager, NavigateDirection, SplitDirection};
use crate::presentation::{StatusBar, TerminalPane, Tui};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use portable_pty::PtySize;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Tabs},
};
use std::time::Duration;

/// Event loop tick rate in milliseconds
const TICK_RATE_MS: u64 = 16; // ~60 FPS

/// Main application runner
pub struct App {
    /// Application state
    state: AppState,
    /// PTY pool
    pty_pool: PtyPool,
    /// Layout manager for split panes
    layout: LayoutManager,
    /// Whether to exit
    should_quit: bool,
    /// Current split direction for new terminals
    next_split_direction: SplitDirection,
}

impl App {
    /// Create a new application
    #[must_use]
    pub fn new(state: AppState) -> Self {
        let pool_config = PtyPoolConfig {
            max_sessions: state.config.pty.max_sessions,
            default_size: (24, 80),
            idle_timeout_secs: 0,
        };

        Self {
            state,
            pty_pool: PtyPool::new(pool_config),
            layout: LayoutManager::new(),
            should_quit: false,
            next_split_direction: SplitDirection::Horizontal,
        }
    }

    /// Spawn a new terminal session and add to layout
    async fn spawn_terminal(&mut self, direction: Option<SplitDirection>) -> Result<PtyId> {
        let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
        let session_num = self.pty_pool.session_count().await + 1;

        let config = PtySessionConfig {
            shell: self.state.config.general.default_shell.clone(),
            args: Vec::new(),
            working_dir: self.state.working_dir.clone(),
            env: Vec::new(),
            size: PtySize {
                rows: rows.saturating_sub(3),
                cols,
                pixel_width: 0,
                pixel_height: 0,
            },
        };

        let label = format!("Shell {session_num}");
        let id = self.pty_pool.spawn(config, label).await?;

        // Add to layout
        self.layout.add_terminal(id, direction, false);

        // Sync focus with pool
        if let Some(focused) = self.layout.focused_pty() {
            let _ = self.pty_pool.set_focus(focused).await;
        }

        tracing::info!(pty_id = %id, "PTY session created");
        Ok(id)
    }

    /// Close the focused terminal
    async fn close_focused(&mut self) -> Result<()> {
        if let Some(focused) = self.layout.focused_pty() {
            // Remove from layout first
            self.layout.remove_terminal(&focused);

            // Kill the PTY
            if let Err(e) = self.pty_pool.kill(&focused).await {
                tracing::warn!(error = %e, "Failed to kill PTY session");
            }

            // Sync focus with pool
            if let Some(new_focus) = self.layout.focused_pty() {
                let _ = self.pty_pool.set_focus(new_focus).await;
            }
        }
        Ok(())
    }

    /// Run the main event loop
    pub async fn run(&mut self, terminal: &mut Tui) -> Result<()> {
        // Initialize first PTY session
        self.spawn_terminal(None).await?;

        self.state.start();

        while !self.should_quit {
            // Read PTY output for all sessions
            self.pty_pool.read_all_outputs().await;

            // Check for exited sessions
            let exited = self.pty_pool.cleanup_exited().await;
            for id in &exited {
                self.layout.remove_terminal(id);
                tracing::info!(pty_id = %id, "PTY session exited and removed");
            }

            // If no sessions left, quit
            if self.layout.is_empty() {
                tracing::info!("All PTY sessions exited, quitting");
                self.should_quit = true;
                break;
            }

            // Draw UI
            terminal.draw(|frame| self.render(frame))?;

            // Handle events
            if self.handle_events().await? {
                break;
            }
        }

        // Cleanup
        self.pty_pool.kill_all().await?;
        self.state.request_shutdown();
        self.state.stop();

        Ok(())
    }

    /// Handle input events
    async fn handle_events(&mut self) -> Result<bool> {
        if event::poll(Duration::from_millis(TICK_RATE_MS))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind != KeyEventKind::Press {
                        return Ok(false);
                    }

                    // Handle special key combinations
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        match key.code {
                            KeyCode::Char('c') => {
                                // Send SIGINT to focused PTY
                                let _ = self.pty_pool.write_to_focused(&[0x03]).await;
                            }
                            KeyCode::Char('d') => {
                                // Send EOF
                                let _ = self.pty_pool.write_to_focused(&[0x04]).await;
                            }
                            KeyCode::Char('q') => {
                                // Quit application
                                self.should_quit = true;
                                return Ok(true);
                            }
                            KeyCode::Char('t') => {
                                // New terminal (no split, tab-like)
                                if let Err(e) = self.spawn_terminal(None).await {
                                    tracing::error!(error = %e, "Failed to spawn new session");
                                }
                            }
                            KeyCode::Char('\\') => {
                                // Horizontal split (side by side)
                                if let Err(e) =
                                    self.spawn_terminal(Some(SplitDirection::Horizontal)).await
                                {
                                    tracing::error!(error = %e, "Failed to spawn new session");
                                }
                            }
                            KeyCode::Char('-') => {
                                // Vertical split (stacked)
                                if let Err(e) =
                                    self.spawn_terminal(Some(SplitDirection::Vertical)).await
                                {
                                    tracing::error!(error = %e, "Failed to spawn new session");
                                }
                            }
                            KeyCode::Char('w') => {
                                // Close current terminal
                                if let Err(e) = self.close_focused().await {
                                    tracing::error!(error = %e, "Failed to close session");
                                }
                            }
                            KeyCode::Tab => {
                                // Next terminal
                                if let Some(pty_id) = self.layout.focus_next() {
                                    let _ = self.pty_pool.set_focus(pty_id).await;
                                }
                            }
                            KeyCode::Char('h') => {
                                // Navigate left (vim-style)
                                if let Some(pty_id) = self.layout.navigate(NavigateDirection::Left)
                                {
                                    let _ = self.pty_pool.set_focus(pty_id).await;
                                }
                            }
                            KeyCode::Char('j') => {
                                // Navigate down (vim-style)
                                if let Some(pty_id) = self.layout.navigate(NavigateDirection::Down)
                                {
                                    let _ = self.pty_pool.set_focus(pty_id).await;
                                }
                            }
                            KeyCode::Char('k') => {
                                // Navigate up (vim-style)
                                if let Some(pty_id) = self.layout.navigate(NavigateDirection::Up) {
                                    let _ = self.pty_pool.set_focus(pty_id).await;
                                }
                            }
                            KeyCode::Char('l') => {
                                // Navigate right (vim-style)
                                if let Some(pty_id) = self.layout.navigate(NavigateDirection::Right)
                                {
                                    let _ = self.pty_pool.set_focus(pty_id).await;
                                }
                            }
                            KeyCode::Left => {
                                // Resize split left (shrink)
                                self.layout.resize_focused(-0.05);
                            }
                            KeyCode::Right => {
                                // Resize split right (grow)
                                self.layout.resize_focused(0.05);
                            }
                            KeyCode::Up => {
                                // Resize split up (shrink)
                                self.layout.resize_focused(-0.05);
                            }
                            KeyCode::Down => {
                                // Resize split down (grow)
                                self.layout.resize_focused(0.05);
                            }
                            _ => {
                                // Pass through to PTY
                                self.send_key_to_focused_pty(key.code, key.modifiers).await;
                            }
                        }
                    } else if key.modifiers.contains(KeyModifiers::SHIFT)
                        && key.code == KeyCode::BackTab
                    {
                        // Previous terminal (Shift+Tab)
                        if let Some(pty_id) = self.layout.focus_prev() {
                            let _ = self.pty_pool.set_focus(pty_id).await;
                        }
                    } else if key.modifiers.contains(KeyModifiers::ALT) {
                        // Alt+number to switch by index
                        match key.code {
                            KeyCode::Char('1') => self.focus_by_index(0).await,
                            KeyCode::Char('2') => self.focus_by_index(1).await,
                            KeyCode::Char('3') => self.focus_by_index(2).await,
                            KeyCode::Char('4') => self.focus_by_index(3).await,
                            KeyCode::Char('5') => self.focus_by_index(4).await,
                            KeyCode::Char('6') => self.focus_by_index(5).await,
                            KeyCode::Char('7') => self.focus_by_index(6).await,
                            KeyCode::Char('8') => self.focus_by_index(7).await,
                            KeyCode::Char('9') => self.focus_by_index(8).await,
                            _ => {
                                self.send_key_to_focused_pty(key.code, key.modifiers).await;
                            }
                        }
                    } else {
                        // Send key to PTY
                        self.send_key_to_focused_pty(key.code, key.modifiers).await;
                    }
                }
                Event::Resize(cols, rows) => {
                    // Resize all PTYs
                    let pty_rows = rows.saturating_sub(3);
                    if let Err(e) = self.pty_pool.resize_all(pty_rows, cols).await {
                        tracing::warn!(error = %e, "Failed to resize PTY pool");
                    }
                    // Invalidate layout cache
                    self.layout.invalidate_cache();
                }
                Event::FocusGained => {}
                Event::FocusLost => {}
                Event::Mouse(_) => {}
                Event::Paste(text) => {
                    let _ = self.pty_pool.write_to_focused(text.as_bytes()).await;
                }
            }
        }

        Ok(false)
    }

    /// Focus session by index
    async fn focus_by_index(&mut self, index: usize) {
        let ids = self.layout.all_pty_ids();
        if let Some(id) = ids.get(index) {
            self.layout.set_focus(*id);
            let _ = self.pty_pool.set_focus(*id).await;
        }
    }

    /// Send a key to the focused PTY
    async fn send_key_to_focused_pty(&self, code: KeyCode, modifiers: KeyModifiers) {
        let data: Vec<u8> = match code {
            KeyCode::Char(c) => {
                if modifiers.contains(KeyModifiers::CONTROL) {
                    let ctrl_char = (c.to_ascii_lowercase() as u8)
                        .wrapping_sub(b'a')
                        .wrapping_add(1);
                    vec![ctrl_char]
                } else if modifiers.contains(KeyModifiers::ALT) {
                    vec![0x1b, c as u8]
                } else {
                    c.to_string().into_bytes()
                }
            }
            KeyCode::Enter => vec![b'\r'],
            KeyCode::Backspace => vec![0x7f],
            KeyCode::Tab => vec![b'\t'],
            KeyCode::Esc => vec![0x1b],
            KeyCode::Up => b"\x1b[A".to_vec(),
            KeyCode::Down => b"\x1b[B".to_vec(),
            KeyCode::Right => b"\x1b[C".to_vec(),
            KeyCode::Left => b"\x1b[D".to_vec(),
            KeyCode::Home => b"\x1b[H".to_vec(),
            KeyCode::End => b"\x1b[F".to_vec(),
            KeyCode::PageUp => b"\x1b[5~".to_vec(),
            KeyCode::PageDown => b"\x1b[6~".to_vec(),
            KeyCode::Insert => b"\x1b[2~".to_vec(),
            KeyCode::Delete => b"\x1b[3~".to_vec(),
            KeyCode::F(n) => match n {
                1 => b"\x1bOP".to_vec(),
                2 => b"\x1bOQ".to_vec(),
                3 => b"\x1bOR".to_vec(),
                4 => b"\x1bOS".to_vec(),
                5 => b"\x1b[15~".to_vec(),
                6 => b"\x1b[17~".to_vec(),
                7 => b"\x1b[18~".to_vec(),
                8 => b"\x1b[19~".to_vec(),
                9 => b"\x1b[20~".to_vec(),
                10 => b"\x1b[21~".to_vec(),
                11 => b"\x1b[23~".to_vec(),
                12 => b"\x1b[24~".to_vec(),
                _ => return,
            },
            _ => return,
        };

        let _ = self.pty_pool.write_to_focused(&data).await;
    }

    /// Render the UI
    fn render(&mut self, frame: &mut ratatui::Frame) {
        let size = frame.area();

        // Create layout: tabs + terminal area + status bar
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Tab bar
                Constraint::Min(0),    // Terminal area
                Constraint::Length(1), // Status bar
            ])
            .split(size);

        // Render tab bar
        self.render_tabs(frame, chunks[0]);

        // Render split terminals
        self.render_terminals(frame, chunks[1]);

        // Render status bar
        self.render_status_bar(frame, chunks[2]);
    }

    /// Render the tab bar
    fn render_tabs(&self, frame: &mut ratatui::Frame, area: Rect) {
        let pty_ids = self.layout.all_pty_ids();
        let focused_pty = self.layout.focused_pty();

        let titles: Vec<Line> = pty_ids
            .iter()
            .enumerate()
            .map(|(idx, pty_id)| {
                let is_focused = focused_pty == Some(*pty_id);
                let style = if is_focused {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };

                // Get label from pool
                let label = futures::executor::block_on(async {
                    self.pty_pool
                        .with_session(pty_id, |s| s.label.clone())
                        .await
                        .unwrap_or_else(|_| format!("Terminal {}", idx + 1))
                });

                let prefix = if is_focused { "● " } else { "○ " };
                Line::from(vec![Span::styled(format!("{prefix}{label}"), style)])
            })
            .collect();

        let selected = pty_ids
            .iter()
            .position(|id| focused_pty == Some(*id))
            .unwrap_or(0);

        let tabs = Tabs::new(titles)
            .select(selected)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .divider(Span::raw(" │ "));

        frame.render_widget(tabs, area);
    }

    /// Render split terminal panes
    fn render_terminals(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        let focused_pty = self.layout.focused_pty();

        // Compute layout
        let computed = self.layout.compute(area);

        // Get all PTY rects
        for (pty_id, rect) in &computed.pty_rects {
            let is_focused = focused_pty == Some(*pty_id);

            // Get screen data
            let result = futures::executor::block_on(self.pty_pool.get_screen_data(pty_id));

            if let Ok((screen, label)) = result {
                let border_color = if is_focused {
                    Color::Cyan
                } else {
                    Color::DarkGray
                };

                let terminal_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .title(format!(" {label} "));

                let terminal_pane = TerminalPane::new(&screen)
                    .block(terminal_block)
                    .focused(is_focused)
                    .show_cursor(is_focused);

                frame.render_widget(terminal_pane, *rect);
            }
        }

        // Render split separators
        self.render_separators(frame, area);
    }

    /// Render split separators
    fn render_separators(&self, frame: &mut ratatui::Frame, area: Rect) {
        if let Some(root) = self.layout.root() {
            self.render_node_separators(frame, root, area);
        }
    }

    fn render_node_separators(
        &self,
        frame: &mut ratatui::Frame,
        node: &crate::presentation::layout::LayoutNode,
        area: Rect,
    ) {
        use crate::presentation::layout::LayoutNode;

        if let LayoutNode::Split {
            direction,
            first,
            second,
            ratio,
            ..
        } = node
        {
            match direction {
                SplitDirection::Horizontal => {
                    let first_width = (f32::from(area.width) * ratio) as u16;
                    let sep_x = area.x + first_width;

                    // Draw vertical separator
                    for y in area.y..area.y + area.height {
                        if sep_x < area.x + area.width {
                            let cell = frame.buffer_mut().cell_mut((sep_x, y));
                            if let Some(cell) = cell {
                                cell.set_char('│').set_fg(Color::DarkGray);
                            }
                        }
                    }

                    // Recurse
                    let first_area = Rect {
                        x: area.x,
                        y: area.y,
                        width: first_width,
                        height: area.height,
                    };
                    let second_area = Rect {
                        x: sep_x + 1,
                        y: area.y,
                        width: area.width.saturating_sub(first_width + 1),
                        height: area.height,
                    };
                    self.render_node_separators(frame, first, first_area);
                    self.render_node_separators(frame, second, second_area);
                }
                SplitDirection::Vertical => {
                    let first_height = (f32::from(area.height) * ratio) as u16;
                    let sep_y = area.y + first_height;

                    // Draw horizontal separator
                    for x in area.x..area.x + area.width {
                        if sep_y < area.y + area.height {
                            let cell = frame.buffer_mut().cell_mut((x, sep_y));
                            if let Some(cell) = cell {
                                cell.set_char('─').set_fg(Color::DarkGray);
                            }
                        }
                    }

                    // Recurse
                    let first_area = Rect {
                        x: area.x,
                        y: area.y,
                        width: area.width,
                        height: first_height,
                    };
                    let second_area = Rect {
                        x: area.x,
                        y: sep_y + 1,
                        width: area.width,
                        height: area.height.saturating_sub(first_height + 1),
                    };
                    self.render_node_separators(frame, first, first_area);
                    self.render_node_separators(frame, second, second_area);
                }
            }
        }
    }

    /// Render the status bar
    fn render_status_bar(&self, frame: &mut ratatui::Frame, area: Rect) {
        let terminal_count = self.layout.terminal_count();
        let session_msg = format!("Panes: {} │ {}", terminal_count, self.state.session_id);

        let status_bar = StatusBar::new()
            .hints(vec![
                ("^\\", "HSplit"),
                ("^-", "VSplit"),
                ("^W", "Close"),
                ("^hjkl", "Nav"),
                ("^Q", "Quit"),
            ])
            .message(&session_msg);

        frame.render_widget(status_bar, area);
    }
}
