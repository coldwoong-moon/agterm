//! AgTerm - AI Agent Terminal
//!
//! Native GPU-accelerated terminal emulator with AI agent orchestration.
//! Inspired by Warp terminal's modern block-based interface.

use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Border, Color, Element, Font, Length, Subscription, Task};
use iced::widget::text_input::Id as TextInputId;
use iced::keyboard::{self, Key, Modifiers};
use std::sync::Arc;
use std::time::{Duration, Instant};
mod terminal;

use terminal::pty::{PtyManager, MAX_OUTPUT_LINES};

// ============================================================================
// Font Configuration - Embedded D2Coding for Korean/CJK support
// ============================================================================

/// D2Coding font bytes (Korean monospace font by Naver)
const D2CODING_FONT: &[u8] = include_bytes!("../assets/fonts/D2Coding.ttf");

/// Monospace font with Korean/CJK support
const MONO_FONT: Font = Font::with_name("D2Coding");

/// Maximum number of command blocks per tab
const MAX_BLOCKS_PER_TAB: usize = 500;

// ============================================================================
// Warp-inspired Dark Theme Colors
// ============================================================================

mod theme {
    use iced::Color;

    // Background colors
    pub const BG_PRIMARY: Color = Color::from_rgb(0.09, 0.09, 0.11);      // #17171c
    pub const BG_SECONDARY: Color = Color::from_rgb(0.12, 0.12, 0.15);    // #1e1e26
    pub const BG_BLOCK: Color = Color::from_rgb(0.14, 0.14, 0.18);        // #242430
    pub const BG_BLOCK_HOVER: Color = Color::from_rgb(0.18, 0.18, 0.22);  // #2d2d38
    pub const BG_INPUT: Color = Color::from_rgb(0.11, 0.11, 0.14);        // #1c1c24

    // Text colors
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.93, 0.93, 0.95);    // #edeff2
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.6, 0.62, 0.68);   // #999ead
    pub const TEXT_MUTED: Color = Color::from_rgb(0.45, 0.47, 0.52);      // #737885

    // Accent colors
    pub const ACCENT_BLUE: Color = Color::from_rgb(0.36, 0.54, 0.98);     // #5c8afa
    pub const ACCENT_GREEN: Color = Color::from_rgb(0.35, 0.78, 0.55);    // #59c78c
    pub const ACCENT_YELLOW: Color = Color::from_rgb(0.95, 0.77, 0.36);   // #f2c55c
    pub const ACCENT_RED: Color = Color::from_rgb(0.92, 0.39, 0.45);      // #eb6473

    // UI elements
    pub const BORDER: Color = Color::from_rgb(0.22, 0.22, 0.28);          // #383847
    pub const TAB_ACTIVE: Color = Color::from_rgb(0.36, 0.54, 0.98);      // #5c8afa
    pub const TAB_INACTIVE: Color = Color::from_rgb(0.18, 0.18, 0.22);    // #2d2d38

    // Prompt symbol
    pub const PROMPT: Color = Color::from_rgb(0.55, 0.36, 0.98);          // #8c5cfa (purple)
}

// ============================================================================
// Command Block - Warp-style grouped command + output
// ============================================================================

#[derive(Clone)]
struct CommandBlock {
    command: String,
    output: Vec<String>,
    timestamp: Instant,
    completed_at: Option<Instant>,
    exit_code: Option<i32>,
    is_running: bool,
}

/// Shorten path for display (replace home dir with ~)
fn shorten_path(path: &str) -> String {
    if let Some(home) = dirs::home_dir() {
        let home_str = home.display().to_string();
        if path.starts_with(&home_str) {
            return path.replacen(&home_str, "~", 1);
        }
    }
    path.to_string()
}

/// Strip ANSI escape sequences from text
fn strip_ansi(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip escape sequence
            if let Some(&'[') = chars.peek() {
                chars.next(); // consume '['
                // Skip until we hit a letter (command character)
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next.is_ascii_alphabetic() || next == 'm' || next == 'K' || next == 'H' || next == 'J' {
                        break;
                    }
                }
            } else if let Some(&']') = chars.peek() {
                // OSC sequence - skip until BEL or ST
                chars.next();
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next == '\x07' || next == '\\' {
                        break;
                    }
                }
            }
        } else if c == '\r' {
            // Skip carriage return
        } else {
            result.push(c);
        }
    }
    result
}

/// Format relative time for display (e.g., "2m ago", "just now")
fn format_relative_time(timestamp: &Instant) -> String {
    let elapsed = timestamp.elapsed().as_secs();

    if elapsed < 5 {
        "just now".to_string()
    } else if elapsed < 60 {
        format!("{}s ago", elapsed)
    } else if elapsed < 3600 {
        format!("{}m ago", elapsed / 60)
    } else if elapsed < 86400 {
        format!("{}h ago", elapsed / 3600)
    } else {
        format!("{}d ago", elapsed / 86400)
    }
}

/// Format execution duration (e.g., "0.5s", "2m 30s")
fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.as_millis();

    if millis < 1000 {
        format!("{}ms", millis)
    } else if secs < 60 {
        let decimal = millis % 1000 / 100;
        if decimal > 0 {
            format!("{}.{}s", secs, decimal)
        } else {
            format!("{}s", secs)
        }
    } else if secs < 3600 {
        let mins = secs / 60;
        let rem_secs = secs % 60;
        if rem_secs > 0 {
            format!("{}m {}s", mins, rem_secs)
        } else {
            format!("{}m", mins)
        }
    } else {
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        if mins > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}h", hours)
        }
    }
}

fn main() -> iced::Result {
    iced::application("AgTerm - AI Agent Terminal", AgTerm::update, AgTerm::view)
        .subscription(AgTerm::subscription)
        .font(D2CODING_FONT)
        .run()
}

/// Input field ID for focusing
fn input_id() -> TextInputId {
    TextInputId::new("terminal_input")
}

/// Main application state
struct AgTerm {
    tabs: Vec<TerminalTab>,
    active_tab: usize,
    pty_manager: Arc<PtyManager>,
    next_tab_id: usize,
    startup_focus_count: u8,
}

impl Default for AgTerm {
    fn default() -> Self {
        let pty_manager = Arc::new(PtyManager::new());
        let session_result = pty_manager.create_session(24, 80);
        let cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "~".to_string());

        let (session_id, error_message) = match session_result {
            Ok(id) => (Some(id), None),
            Err(e) => (None, Some(format!("Failed to create PTY session: {}", e))),
        };

        let tab = TerminalTab {
            id: 0,
            session_id,
            blocks: Vec::new(),
            pending_output: vec!["Welcome to AgTerm - AI Agent Terminal".to_string()],
            input: String::new(),
            cwd,
            error_message,
            history: Vec::new(),
            history_index: None,
            history_temp_input: String::new(),
        };

        Self {
            tabs: vec![tab],
            active_tab: 0,
            pty_manager,
            next_tab_id: 1,
            startup_focus_count: 10,
        }
    }
}

/// A single terminal tab with block-based output
struct TerminalTab {
    #[allow(dead_code)]
    id: usize,
    session_id: Option<uuid::Uuid>,
    blocks: Vec<CommandBlock>,
    pending_output: Vec<String>,  // Output before first command
    input: String,
    cwd: String,  // Current working directory display
    error_message: Option<String>,  // PTY error message if creation failed
    // Command history
    history: Vec<String>,
    history_index: Option<usize>,  // Current position in history (None = not browsing)
    history_temp_input: String,    // Temporary storage for current input when browsing
}

#[derive(Debug, Clone)]
enum Message {
    // Tab management
    NewTab,
    CloseTab(usize),
    CloseCurrentTab,
    SelectTab(usize),
    NextTab,
    PrevTab,

    // Terminal input
    InputChanged(String),
    SubmitInput,
    FocusInput,

    // History navigation
    HistoryPrevious,
    HistoryNext,

    // Keyboard events
    KeyPressed(Key, Modifiers),

    // Clipboard
    CopyToClipboard(String),

    // Tick for PTY polling
    Tick,
}

impl AgTerm {
    /// Get the current shell name (e.g., "zsh", "bash")
    fn get_shell_name(&self) -> String {
        std::env::var("SHELL")
            .ok()
            .and_then(|path| {
                std::path::Path::new(&path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "shell".to_string())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NewTab => {
                let id = self.next_tab_id;
                self.next_tab_id += 1;

                let session_result = self.pty_manager.create_session(24, 80);
                let cwd = std::env::current_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| "~".to_string());

                let (session_id, error_message) = match session_result {
                    Ok(id) => (Some(id), None),
                    Err(e) => (None, Some(format!("Failed to create PTY session: {}", e))),
                };

                let tab = TerminalTab {
                    id,
                    session_id,
                    blocks: Vec::new(),
                    pending_output: Vec::new(),
                    input: String::new(),
                    cwd,
                    error_message,
                    history: Vec::new(),
                    history_index: None,
                    history_temp_input: String::new(),
                };
                self.tabs.push(tab);
                self.active_tab = self.tabs.len() - 1;
                text_input::focus(input_id())
            }

            Message::CloseTab(index) => {
                if self.tabs.len() > 1 {
                    if let Some(tab) = self.tabs.get(index) {
                        if let Some(session_id) = &tab.session_id {
                            let _ = self.pty_manager.close_session(session_id);
                        }
                    }
                    self.tabs.remove(index);
                    if self.active_tab >= self.tabs.len() {
                        self.active_tab = self.tabs.len() - 1;
                    }
                }
                Task::none()
            }

            Message::SelectTab(index) => {
                if index < self.tabs.len() {
                    self.active_tab = index;
                }
                text_input::focus(input_id())
            }

            Message::InputChanged(input) => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    tab.input = input;
                    // Reset history browsing when user types
                    tab.history_index = None;
                }
                Task::none()
            }

            Message::SubmitInput => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    if !tab.input.is_empty() {
                        // Add to history (avoid duplicates of last command)
                        let cmd = tab.input.clone();
                        if tab.history.last() != Some(&cmd) {
                            tab.history.push(cmd.clone());
                        }

                        // Reset history navigation
                        tab.history_index = None;
                        tab.history_temp_input.clear();

                        // Create a new command block
                        let block = CommandBlock {
                            command: tab.input.clone(),
                            output: Vec::new(),
                            timestamp: Instant::now(),
                            completed_at: None,
                            exit_code: None,
                            is_running: true,
                        };
                        tab.blocks.push(block);

                        // Send command to PTY
                        if let Some(session_id) = &tab.session_id {
                            let input = format!("{}\n", tab.input);
                            let _ = self.pty_manager.write(session_id, input.as_bytes());
                        }
                        tab.input.clear();
                    }
                }
                text_input::focus(input_id())
            }

            Message::FocusInput => {
                text_input::focus(input_id())
            }

            Message::HistoryPrevious => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    if tab.history.is_empty() {
                        return Task::none();
                    }

                    match tab.history_index {
                        None => {
                            // Start browsing history - save current input
                            tab.history_temp_input = tab.input.clone();
                            tab.history_index = Some(tab.history.len() - 1);
                            tab.input = tab.history[tab.history.len() - 1].clone();
                        }
                        Some(idx) if idx > 0 => {
                            // Move to older command
                            tab.history_index = Some(idx - 1);
                            tab.input = tab.history[idx - 1].clone();
                        }
                        _ => {
                            // Already at oldest - do nothing
                        }
                    }
                }
                Task::none()
            }

            Message::HistoryNext => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    match tab.history_index {
                        Some(idx) if idx < tab.history.len() - 1 => {
                            // Move to newer command
                            tab.history_index = Some(idx + 1);
                            tab.input = tab.history[idx + 1].clone();
                        }
                        Some(_) => {
                            // At newest history entry - restore temp input
                            tab.history_index = None;
                            tab.input = tab.history_temp_input.clone();
                            tab.history_temp_input.clear();
                        }
                        None => {
                            // Not browsing history - do nothing
                        }
                    }
                }
                Task::none()
            }

            Message::CloseCurrentTab => {
                if self.tabs.len() > 1 {
                    if let Some(tab) = self.tabs.get(self.active_tab) {
                        if let Some(session_id) = &tab.session_id {
                            let _ = self.pty_manager.close_session(session_id);
                        }
                    }
                    self.tabs.remove(self.active_tab);
                    if self.active_tab >= self.tabs.len() {
                        self.active_tab = self.tabs.len() - 1;
                    }
                }
                text_input::focus(input_id())
            }

            Message::NextTab => {
                if !self.tabs.is_empty() {
                    self.active_tab = (self.active_tab + 1) % self.tabs.len();
                }
                text_input::focus(input_id())
            }

            Message::PrevTab => {
                if !self.tabs.is_empty() {
                    self.active_tab = if self.active_tab == 0 {
                        self.tabs.len() - 1
                    } else {
                        self.active_tab - 1
                    };
                }
                text_input::focus(input_id())
            }

            Message::KeyPressed(key, modifiers) => {
                // Handle keyboard shortcuts
                if modifiers.command() {
                    match key.as_ref() {
                        Key::Character("t") => return self.update(Message::NewTab),
                        Key::Character("w") => return self.update(Message::CloseCurrentTab),
                        Key::Character("]") => return self.update(Message::NextTab),
                        Key::Character("[") => return self.update(Message::PrevTab),
                        Key::Character("1") => return self.update(Message::SelectTab(0)),
                        Key::Character("2") => return self.update(Message::SelectTab(1)),
                        Key::Character("3") => return self.update(Message::SelectTab(2)),
                        Key::Character("4") => return self.update(Message::SelectTab(3)),
                        Key::Character("5") => return self.update(Message::SelectTab(4)),
                        _ => {}
                    }
                }

                // History navigation with arrow keys (no modifiers needed)
                match key.as_ref() {
                    Key::Named(keyboard::key::Named::ArrowUp) => {
                        return self.update(Message::HistoryPrevious);
                    }
                    Key::Named(keyboard::key::Named::ArrowDown) => {
                        return self.update(Message::HistoryNext);
                    }
                    _ => {}
                }

                Task::none()
            }

            Message::CopyToClipboard(content) => {
                iced::clipboard::write(content)
            }

            Message::Tick => {
                // Auto-focus input on startup
                let focus_task = if self.startup_focus_count > 0 {
                    self.startup_focus_count -= 1;
                    text_input::focus(input_id())
                } else {
                    Task::none()
                };

                // Poll PTY output for all tabs
                for tab in &mut self.tabs {
                    if let Some(session_id) = &tab.session_id {
                        if let Ok(data) = self.pty_manager.read(session_id) {
                            if !data.is_empty() {
                                let raw_output = String::from_utf8_lossy(&data);
                                let output = strip_ansi(&raw_output);

                                // Find the last running block or use pending_output
                                if let Some(block) = tab.blocks.iter_mut().rev().find(|b| b.is_running) {
                                    // Add output to the current running block
                                    for line in output.lines() {
                                        let trimmed = line.trim();
                                        // Skip the echo of the command itself
                                        if !trimmed.is_empty() && trimmed != block.command {
                                            block.output.push(trimmed.to_string());
                                            // Enforce output line limit per block
                                            if block.output.len() > MAX_OUTPUT_LINES {
                                                block.output.drain(0..1000);  // Remove first 1000 lines
                                            }
                                        }
                                    }
                                    // Mark block as complete after a brief pause
                                    // (PTY typically sends output quickly, so if we're getting data, command is running)
                                    if block.timestamp.elapsed() > Duration::from_millis(500) && output.is_empty() {
                                        if block.is_running {
                                            block.completed_at = Some(Instant::now());
                                        }
                                        block.is_running = false;
                                    }
                                } else {
                                    // No running block, add to pending output
                                    for line in output.lines() {
                                        let trimmed = line.trim();
                                        if !trimmed.is_empty() {
                                            tab.pending_output.push(trimmed.to_string());
                                            // Limit pending output as well
                                            if tab.pending_output.len() > 1000 {
                                                tab.pending_output.drain(0..100);
                                            }
                                        }
                                    }
                                }
                            } else {
                                // No new output - mark running blocks as complete if enough time passed
                                for block in &mut tab.blocks {
                                    if block.is_running && block.timestamp.elapsed() > Duration::from_millis(300) {
                                        if block.completed_at.is_none() {
                                            block.completed_at = Some(Instant::now());
                                        }
                                        block.is_running = false;
                                    }
                                }
                            }
                        }
                    }

                    // Enforce maximum blocks per tab
                    if tab.blocks.len() > MAX_BLOCKS_PER_TAB {
                        let excess = tab.blocks.len() - MAX_BLOCKS_PER_TAB;
                        tab.blocks.drain(0..excess);
                    }
                }
                focus_task
            }
        }
    }

    fn view(&self) -> Element<Message> {
        if self.tabs.is_empty() {
            return container(text("No terminal open").color(theme::TEXT_PRIMARY))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(|_| container::Style {
                    background: Some(theme::BG_PRIMARY.into()),
                    ..Default::default()
                })
                .into();
        }

        // ========== Tab Bar ==========
        let tab_bar: Element<Message> = row(
            self.tabs
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    let is_active = i == self.active_tab;
                    let label = format!("Terminal {}", i + 1);

                    let icon_color = if is_active { theme::TAB_ACTIVE } else { theme::TEXT_MUTED };
                    let label_color = if is_active { theme::TEXT_PRIMARY } else { theme::TEXT_SECONDARY };

                    let tab_content = column![
                        // Tab content with icon
                        container(
                            row![
                                text("▶").size(11).color(icon_color),
                                Space::with_width(8),
                                text(label.clone()).size(13).color(label_color),
                                Space::with_width(8),
                                text("×").size(14).color(theme::TEXT_MUTED)
                            ]
                            .align_y(Alignment::Center)
                        ),
                        // Active tab bottom accent line
                        container(Space::with_height(0))
                            .width(Length::Fill)
                            .height(2)
                            .style(move |_| container::Style {
                                background: if is_active {
                                    Some(theme::TAB_ACTIVE.into())
                                } else {
                                    None
                                },
                                ..Default::default()
                            })
                    ]
                    .spacing(6);

                    let tab_button: Element<Message> = button(
                        container(tab_content)
                            .padding([8, 16])
                    )
                        .style(move |_, status| {
                            let bg = match status {
                                button::Status::Hovered => {
                                    if is_active {
                                        theme::BG_SECONDARY
                                    } else {
                                        theme::BG_BLOCK_HOVER
                                    }
                                }
                                _ => {
                                    if is_active {
                                        theme::BG_SECONDARY
                                    } else {
                                        theme::BG_PRIMARY
                                    }
                                }
                            };
                            button::Style {
                                background: Some(bg.into()),
                                text_color: theme::TEXT_PRIMARY,
                                border: Border {
                                    color: Color::TRANSPARENT,
                                    width: 0.0,
                                    radius: iced::border::Radius {
                                        top_left: 6.0,
                                        top_right: 6.0,
                                        bottom_left: 0.0,
                                        bottom_right: 0.0,
                                    },
                                },
                                ..Default::default()
                            }
                        })
                        .on_press(Message::SelectTab(i))
                        .into();

                    tab_button
                })
                .collect::<Vec<Element<Message>>>(),
        )
        .spacing(2)
        .push(Space::with_width(8))
        .push(
            button(text("+").size(16).color(theme::TEXT_SECONDARY))
                .padding([8, 14])
                .style(|_, status| {
                    let bg = match status {
                        button::Status::Hovered => theme::BG_BLOCK_HOVER,
                        _ => theme::BG_BLOCK,
                    };
                    button::Style {
                        background: Some(bg.into()),
                        text_color: theme::TEXT_SECONDARY,
                        border: Border {
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                })
                .on_press(Message::NewTab),
        )
        .into();

        // ========== Terminal Content ==========
        let content: Element<Message> = if let Some(tab) = self.tabs.get(self.active_tab) {
            let mut blocks_column: Vec<Element<Message>> = Vec::new();

            // Show error message if PTY creation failed
            if let Some(error_msg) = &tab.error_message {
                let error_block = self.render_error_block(error_msg);
                blocks_column.push(error_block);
            }

            // Show pending output (before first command)
            if !tab.pending_output.is_empty() {
                let welcome_block = self.render_welcome_block(&tab.pending_output);
                blocks_column.push(welcome_block);
            }

            // Render command blocks
            for block in &tab.blocks {
                let block_element = self.render_command_block(block);
                blocks_column.push(block_element);
            }

            // Add some spacing at the bottom
            blocks_column.push(Space::with_height(20).into());

            let terminal_content: Element<Message> = scrollable(
                column(blocks_column)
                    .spacing(12)
                    .width(Length::Fill),
            )
            .height(Length::Fill)
            .style(|_, _| scrollable::Style {
                container: container::Style::default(),
                vertical_rail: scrollable::Rail {
                    background: Some(theme::BG_PRIMARY.into()),
                    border: Border::default(),
                    scroller: scrollable::Scroller {
                        color: theme::BG_BLOCK_HOVER,
                        border: Border {
                            radius: 4.0.into(),
                            ..Default::default()
                        },
                    },
                },
                horizontal_rail: scrollable::Rail {
                    background: Some(theme::BG_PRIMARY.into()),
                    border: Border::default(),
                    scroller: scrollable::Scroller {
                        color: theme::BG_BLOCK_HOVER,
                        border: Border {
                            radius: 4.0.into(),
                            ..Default::default()
                        },
                    },
                },
                gap: None,
            })
            .into();

            // ========== Input Area ==========
            let prompt_symbol = text("❯").size(16).font(MONO_FONT).color(theme::PROMPT);
            let cwd_display = text(shorten_path(&tab.cwd))
                .size(12)
                .font(MONO_FONT)
                .color(theme::TEXT_MUTED);

            let input_field = text_input("Type a command...", &tab.input)
                .id(input_id())
                .on_input(Message::InputChanged)
                .on_submit(Message::SubmitInput)
                .padding([12, 16])
                .size(14)
                .font(MONO_FONT)
                .style(|_, _| text_input::Style {
                    background: theme::BG_INPUT.into(),
                    border: Border {
                        color: theme::BORDER,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    icon: theme::TEXT_MUTED,
                    placeholder: theme::TEXT_MUTED,
                    value: theme::TEXT_PRIMARY,
                    selection: theme::ACCENT_BLUE,
                });

            let input_row: Element<Message> = container(
                column![
                    cwd_display,
                    Space::with_height(6),
                    row![
                        prompt_symbol,
                        Space::with_width(12),
                        input_field
                    ]
                    .align_y(Alignment::Center)
                ]
                .width(Length::Fill),
            )
            .padding([16, 20])
            .style(|_| container::Style {
                background: Some(theme::BG_SECONDARY.into()),
                border: Border {
                    color: theme::BORDER,
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .into();

            // ========== Status Bar ==========
            let shell_name = self.get_shell_name();
            let tab_info = format!("Tab {} of {}", self.active_tab + 1, self.tabs.len());

            let status_left = text(shell_name)
                .size(12)
                .color(theme::TEXT_MUTED);

            let status_center = text(tab_info)
                .size(12)
                .color(theme::TEXT_MUTED);

            let status_right = text("⌘T New | ⌘W Close")
                .size(12)
                .color(theme::TEXT_MUTED);

            let status_bar: Element<Message> = container(
                row![
                    status_left,
                    Space::with_width(Length::Fill),
                    status_center,
                    Space::with_width(Length::Fill),
                    status_right
                ]
                .align_y(Alignment::Center)
                .width(Length::Fill)
            )
            .padding([4, 12])
            .width(Length::Fill)
            .style(|_| container::Style {
                background: Some(theme::BG_PRIMARY.into()),
                border: Border {
                    color: theme::BORDER,
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .into();

            column![
                container(terminal_content)
                    .padding([16, 20])
                    .width(Length::Fill)
                    .height(Length::Fill),
                input_row,
                status_bar
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            column![text("No terminal open").color(theme::TEXT_PRIMARY)].into()
        };

        // ========== Main Layout ==========
        let main_content = column![
            container(tab_bar)
                .padding([8, 12])
                .width(Length::Fill)
                .style(|_| container::Style {
                    background: Some(theme::BG_PRIMARY.into()),
                    border: Border {
                        color: theme::BORDER,
                        width: 0.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                }),
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_| container::Style {
                    background: Some(theme::BG_SECONDARY.into()),
                    ..Default::default()
                })
        ];

        container(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(theme::BG_PRIMARY.into()),
                ..Default::default()
            })
            .into()
    }

    /// Render welcome/info block (for initial output)
    fn render_welcome_block<'a>(&self, lines: &'a [String]) -> Element<'a, Message> {
        let content = column(
            lines
                .iter()
                .map(|line| {
                    text(line)
                        .size(13)
                        .font(MONO_FONT)
                        .color(theme::TEXT_SECONDARY)
                        .into()
                })
                .collect::<Vec<Element<Message>>>(),
        )
        .spacing(4);

        container(content)
            .padding([12, 16])
            .width(Length::Fill)
            .style(|_| container::Style {
                background: Some(theme::BG_BLOCK.into()),
                border: Border {
                    color: theme::BORDER,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            })
            .into()
    }

    /// Render error block for PTY failures
    fn render_error_block<'a>(&self, message: &'a str) -> Element<'a, Message> {
        container(
            text(format!("⚠ Error: {}", message))
                .size(13)
                .font(MONO_FONT)
                .color(theme::ACCENT_RED)
        )
        .padding([12, 16])
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Color::from_rgb(0.2, 0.1, 0.1).into()),
            border: Border {
                color: theme::ACCENT_RED,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
        .into()
    }

    /// Render a command block (Warp-style)
    fn render_command_block<'a>(&self, block: &'a CommandBlock) -> Element<'a, Message> {
        // Command header with status indicator
        let status_color = if block.is_running {
            theme::ACCENT_YELLOW
        } else if block.exit_code.unwrap_or(0) == 0 {
            theme::ACCENT_GREEN
        } else {
            theme::ACCENT_RED
        };

        let status_indicator = text(if block.is_running { "●" } else { "●" })
            .size(10)
            .color(status_color);

        let command_text = text(&block.command)
            .size(14)
            .font(MONO_FONT)
            .color(theme::TEXT_PRIMARY);

        // Timestamp display
        let timestamp_text = text(format_relative_time(&block.timestamp))
            .size(11)
            .color(theme::TEXT_MUTED);

        // Execution time display (if completed)
        let execution_time_element: Element<Message> = if let Some(completed_at) = block.completed_at {
            let duration = completed_at.duration_since(block.timestamp);
            let duration_str = format_duration(duration);
            row![
                text(" • ").size(11).color(theme::TEXT_MUTED),
                text(duration_str).size(11).color(theme::ACCENT_GREEN)
            ].into()
        } else {
            Space::with_width(0).into()
        };

        // Copy button
        let copy_button = button(
            text("Copy").size(11).color(theme::TEXT_MUTED)
        )
        .padding([4, 8])
        .style(|_, _| button::Style {
            background: Some(theme::BG_SECONDARY.into()),
            text_color: theme::TEXT_MUTED,
            border: Border {
                color: theme::BORDER,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .on_press(Message::CopyToClipboard(block.command.clone()));

        // Header row: status + command + metadata + copy button
        let command_header = row![
            status_indicator,
            Space::with_width(10),
            text("$").size(14).font(MONO_FONT).color(theme::PROMPT),
            Space::with_width(8),
            command_text,
            Space::with_width(12),
            timestamp_text,
            execution_time_element,
            Space::with_width(Length::Fill),
            copy_button
        ]
        .align_y(Alignment::Center);

        // Output lines
        let output_content: Element<Message> = if block.output.is_empty() {
            if block.is_running {
                text("Running...").size(12).font(MONO_FONT).color(theme::TEXT_MUTED).into()
            } else {
                Space::with_height(0).into()
            }
        } else {
            column(
                block
                    .output
                    .iter()
                    .map(|line| {
                        text(line)
                            .size(13)
                            .font(MONO_FONT)
                            .color(theme::TEXT_SECONDARY)
                            .into()
                    })
                    .collect::<Vec<Element<Message>>>(),
            )
            .spacing(2)
            .into()
        };

        let block_content = column![
            command_header,
            Space::with_height(8),
            output_content
        ]
        .width(Length::Fill);

        container(block_content)
            .padding([12, 16])
            .width(Length::Fill)
            .style(|_| container::Style {
                background: Some(theme::BG_BLOCK.into()),
                border: Border {
                    color: theme::BORDER,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            })
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        // Poll PTY output every 100ms (balance between responsiveness and CPU usage)
        let timer = iced::time::every(Duration::from_millis(100)).map(|_| Message::Tick);

        let keyboard = keyboard::on_key_press(|key, modifiers| {
            Some(Message::KeyPressed(key, modifiers))
        });

        Subscription::batch([timer, keyboard])
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // ========== Utility Function Tests ==========

    #[test]
    fn test_strip_ansi_basic() {
        // Test basic ANSI color codes
        let input = "\x1b[31mRed Text\x1b[0m";
        let result = strip_ansi(input);
        assert_eq!(result, "Red Text");
    }

    #[test]
    fn test_strip_ansi_multiple_codes() {
        let input = "\x1b[1m\x1b[32mBold Green\x1b[0m Normal";
        let result = strip_ansi(input);
        assert_eq!(result, "Bold Green Normal");
    }

    #[test]
    fn test_strip_ansi_cursor_movement() {
        let input = "\x1b[2K\x1b[1GLine content";
        let result = strip_ansi(input);
        assert_eq!(result, "Line content");
    }

    #[test]
    fn test_strip_ansi_osc_sequence() {
        // OSC sequence (e.g., terminal title)
        let input = "\x1b]0;Terminal Title\x07Content";
        let result = strip_ansi(input);
        assert_eq!(result, "Content");
    }

    #[test]
    fn test_strip_ansi_carriage_return() {
        let input = "Line1\rLine2";
        let result = strip_ansi(input);
        assert_eq!(result, "Line1Line2");
    }

    #[test]
    fn test_strip_ansi_no_sequences() {
        let input = "Plain text without ANSI";
        let result = strip_ansi(input);
        assert_eq!(result, "Plain text without ANSI");
    }

    #[test]
    fn test_strip_ansi_korean() {
        let input = "\x1b[32m한글 테스트\x1b[0m";
        let result = strip_ansi(input);
        assert_eq!(result, "한글 테스트");
    }

    #[test]
    fn test_shorten_path_with_home() {
        if let Some(home) = dirs::home_dir() {
            let home_str = home.display().to_string();
            let path = format!("{}/projects/test", home_str);
            let result = shorten_path(&path);
            assert_eq!(result, "~/projects/test");
        }
    }

    #[test]
    fn test_shorten_path_without_home() {
        let path = "/usr/local/bin";
        let result = shorten_path(path);
        assert_eq!(result, "/usr/local/bin");
    }

    #[test]
    fn test_format_duration_milliseconds() {
        let duration = Duration::from_millis(500);
        assert_eq!(format_duration(duration), "500ms");
    }

    #[test]
    fn test_format_duration_seconds() {
        let duration = Duration::from_millis(2500);
        assert_eq!(format_duration(duration), "2.5s");
    }

    #[test]
    fn test_format_duration_minutes() {
        let duration = Duration::from_secs(90);
        assert_eq!(format_duration(duration), "1m 30s");
    }

    #[test]
    fn test_format_duration_hours() {
        let duration = Duration::from_secs(3700);
        assert_eq!(format_duration(duration), "1h 1m");
    }

    // ========== Tab Management Tests ==========

    /// Create a mock AgTerm instance for testing (without PTY)
    fn create_test_app() -> AgTerm {
        let pty_manager = Arc::new(PtyManager::new());

        let tab = TerminalTab {
            id: 0,
            session_id: None, // No actual PTY for tests
            blocks: Vec::new(),
            pending_output: vec!["Test Welcome".to_string()],
            input: String::new(),
            cwd: "/test/path".to_string(),
            error_message: None,
            history: Vec::new(),
            history_index: None,
            history_temp_input: String::new(),
        };

        AgTerm {
            tabs: vec![tab],
            active_tab: 0,
            pty_manager,
            next_tab_id: 1,
            startup_focus_count: 0,
        }
    }

    #[test]
    fn test_initial_state() {
        let app = create_test_app();
        assert_eq!(app.tabs.len(), 1);
        assert_eq!(app.active_tab, 0);
        assert_eq!(app.tabs[0].cwd, "/test/path");
    }

    #[test]
    fn test_new_tab() {
        let mut app = create_test_app();
        let initial_count = app.tabs.len();

        let _ = app.update(Message::NewTab);

        assert_eq!(app.tabs.len(), initial_count + 1);
        assert_eq!(app.active_tab, initial_count); // Should switch to new tab
    }

    #[test]
    fn test_close_current_tab_with_multiple_tabs() {
        let mut app = create_test_app();
        let _ = app.update(Message::NewTab); // Now we have 2 tabs
        assert_eq!(app.tabs.len(), 2);

        let _ = app.update(Message::CloseCurrentTab);

        assert_eq!(app.tabs.len(), 1);
    }

    #[test]
    fn test_close_tab_preserves_minimum() {
        let mut app = create_test_app();
        assert_eq!(app.tabs.len(), 1);

        let _ = app.update(Message::CloseCurrentTab);

        // Should not close the last tab
        assert_eq!(app.tabs.len(), 1);
    }

    #[test]
    fn test_select_tab() {
        let mut app = create_test_app();
        let _ = app.update(Message::NewTab);
        let _ = app.update(Message::NewTab);
        assert_eq!(app.tabs.len(), 3);

        let _ = app.update(Message::SelectTab(0));
        assert_eq!(app.active_tab, 0);

        let _ = app.update(Message::SelectTab(2));
        assert_eq!(app.active_tab, 2);
    }

    #[test]
    fn test_select_invalid_tab() {
        let mut app = create_test_app();
        let _ = app.update(Message::SelectTab(999));

        // Should not change active tab if index is invalid
        assert_eq!(app.active_tab, 0);
    }

    #[test]
    fn test_next_tab_cycling() {
        let mut app = create_test_app();
        let _ = app.update(Message::NewTab);
        let _ = app.update(Message::NewTab);
        // 3 tabs: 0, 1, 2; active = 2

        let _ = app.update(Message::NextTab);
        assert_eq!(app.active_tab, 0); // Should cycle back to 0
    }

    #[test]
    fn test_prev_tab_cycling() {
        let mut app = create_test_app();
        let _ = app.update(Message::NewTab);
        let _ = app.update(Message::NewTab);
        let _ = app.update(Message::SelectTab(0));

        let _ = app.update(Message::PrevTab);
        assert_eq!(app.active_tab, 2); // Should cycle to last tab
    }

    // ========== Input Management Tests ==========

    #[test]
    fn test_input_changed() {
        let mut app = create_test_app();

        let _ = app.update(Message::InputChanged("ls -la".to_string()));

        assert_eq!(app.tabs[0].input, "ls -la");
    }

    #[test]
    fn test_submit_input_creates_block() {
        let mut app = create_test_app();

        let _ = app.update(Message::InputChanged("echo hello".to_string()));
        let _ = app.update(Message::SubmitInput);

        assert_eq!(app.tabs[0].blocks.len(), 1);
        assert_eq!(app.tabs[0].blocks[0].command, "echo hello");
        assert!(app.tabs[0].blocks[0].is_running);
        assert_eq!(app.tabs[0].input, ""); // Input should be cleared
    }

    #[test]
    fn test_submit_empty_input_no_block() {
        let mut app = create_test_app();

        let _ = app.update(Message::SubmitInput);

        assert_eq!(app.tabs[0].blocks.len(), 0);
    }

    #[test]
    fn test_submit_adds_to_history() {
        let mut app = create_test_app();

        let _ = app.update(Message::InputChanged("cmd1".to_string()));
        let _ = app.update(Message::SubmitInput);
        let _ = app.update(Message::InputChanged("cmd2".to_string()));
        let _ = app.update(Message::SubmitInput);

        assert_eq!(app.tabs[0].history.len(), 2);
        assert_eq!(app.tabs[0].history[0], "cmd1");
        assert_eq!(app.tabs[0].history[1], "cmd2");
    }

    #[test]
    fn test_submit_no_duplicate_history() {
        let mut app = create_test_app();

        let _ = app.update(Message::InputChanged("cmd1".to_string()));
        let _ = app.update(Message::SubmitInput);
        let _ = app.update(Message::InputChanged("cmd1".to_string()));
        let _ = app.update(Message::SubmitInput);

        // Should not add duplicate consecutive commands
        assert_eq!(app.tabs[0].history.len(), 1);
    }

    // ========== History Navigation Tests ==========

    #[test]
    fn test_history_previous() {
        let mut app = create_test_app();

        // Add some history
        let _ = app.update(Message::InputChanged("cmd1".to_string()));
        let _ = app.update(Message::SubmitInput);
        let _ = app.update(Message::InputChanged("cmd2".to_string()));
        let _ = app.update(Message::SubmitInput);

        // Navigate back
        let _ = app.update(Message::HistoryPrevious);
        assert_eq!(app.tabs[0].input, "cmd2");

        let _ = app.update(Message::HistoryPrevious);
        assert_eq!(app.tabs[0].input, "cmd1");
    }

    #[test]
    fn test_history_next() {
        let mut app = create_test_app();

        // Add history
        let _ = app.update(Message::InputChanged("cmd1".to_string()));
        let _ = app.update(Message::SubmitInput);
        let _ = app.update(Message::InputChanged("cmd2".to_string()));
        let _ = app.update(Message::SubmitInput);

        // Type something new
        let _ = app.update(Message::InputChanged("new cmd".to_string()));

        // Navigate back
        let _ = app.update(Message::HistoryPrevious);
        let _ = app.update(Message::HistoryPrevious);

        // Navigate forward
        let _ = app.update(Message::HistoryNext);
        assert_eq!(app.tabs[0].input, "cmd2");

        let _ = app.update(Message::HistoryNext);
        assert_eq!(app.tabs[0].input, "new cmd"); // Should restore temp input
    }

    #[test]
    fn test_history_empty() {
        let mut app = create_test_app();

        // Should not crash on empty history
        let _ = app.update(Message::HistoryPrevious);
        let _ = app.update(Message::HistoryNext);

        assert_eq!(app.tabs[0].input, "");
    }

    #[test]
    fn test_input_change_resets_history_index() {
        let mut app = create_test_app();

        let _ = app.update(Message::InputChanged("cmd1".to_string()));
        let _ = app.update(Message::SubmitInput);

        let _ = app.update(Message::HistoryPrevious);
        assert!(app.tabs[0].history_index.is_some());

        let _ = app.update(Message::InputChanged("new".to_string()));
        assert!(app.tabs[0].history_index.is_none());
    }

    // ========== Command Block Tests ==========

    #[test]
    fn test_command_block_creation() {
        let block = CommandBlock {
            command: "test".to_string(),
            output: vec!["line1".to_string(), "line2".to_string()],
            timestamp: Instant::now(),
            completed_at: None,
            exit_code: None,
            is_running: true,
        };

        assert_eq!(block.command, "test");
        assert_eq!(block.output.len(), 2);
        assert!(block.is_running);
        assert!(block.completed_at.is_none());
    }

    // ========== Theme Tests ==========

    #[test]
    fn test_theme_colors_defined() {
        // Verify all theme colors are accessible
        let _ = theme::BG_PRIMARY;
        let _ = theme::BG_SECONDARY;
        let _ = theme::BG_BLOCK;
        let _ = theme::BG_BLOCK_HOVER;
        let _ = theme::BG_INPUT;
        let _ = theme::TEXT_PRIMARY;
        let _ = theme::TEXT_SECONDARY;
        let _ = theme::TEXT_MUTED;
        let _ = theme::ACCENT_BLUE;
        let _ = theme::ACCENT_GREEN;
        let _ = theme::ACCENT_YELLOW;
        let _ = theme::ACCENT_RED;
        let _ = theme::BORDER;
        let _ = theme::TAB_ACTIVE;
        let _ = theme::PROMPT;
    }

    // ========== Integration Tests (with actual PTY) ==========

    #[test]
    #[cfg(unix)] // PTY tests only work on Unix
    fn test_pty_session_creation() {
        let pty_manager = PtyManager::new();
        let result = pty_manager.create_session(24, 80);

        assert!(result.is_ok(), "PTY session should be created successfully");

        let session_id = result.unwrap();
        let close_result = pty_manager.close_session(&session_id);
        assert!(close_result.is_ok());
    }

    #[test]
    #[cfg(unix)]
    fn test_pty_write_read() {
        let pty_manager = PtyManager::new();
        let session_id = pty_manager.create_session(24, 80).unwrap();

        // Write a simple command
        let write_result = pty_manager.write(&session_id, b"echo test\n");
        assert!(write_result.is_ok());

        // Give PTY time to process
        std::thread::sleep(Duration::from_millis(200));

        // Read output
        let read_result = pty_manager.read(&session_id);
        assert!(read_result.is_ok());

        let _ = pty_manager.close_session(&session_id);
    }

    #[test]
    fn test_max_blocks_limit() {
        let mut app = create_test_app();

        // Add more than MAX_BLOCKS_PER_TAB blocks
        for i in 0..(MAX_BLOCKS_PER_TAB + 50) {
            app.tabs[0].blocks.push(CommandBlock {
                command: format!("cmd{}", i),
                output: Vec::new(),
                timestamp: Instant::now(),
                completed_at: Some(Instant::now()),
                exit_code: Some(0),
                is_running: false,
            });
        }

        // Trigger tick to enforce limit
        let _ = app.update(Message::Tick);

        assert!(app.tabs[0].blocks.len() <= MAX_BLOCKS_PER_TAB);
    }
}
