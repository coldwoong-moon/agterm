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
mod logging;
mod debug;

use debug::{DebugPanel, DebugPanelMessage};
use logging::{LogBuffer, LoggingConfig};

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

    // Prompt symbol
    pub const PROMPT: Color = Color::from_rgb(0.55, 0.36, 0.98);          // #8c5cfa (purple)

    // ANSI colors (standard 16-color palette)
    pub const ANSI_BLACK: Color = Color::from_rgb(0.0, 0.0, 0.0);
    pub const ANSI_RED: Color = Color::from_rgb(0.8, 0.2, 0.2);
    pub const ANSI_GREEN: Color = Color::from_rgb(0.2, 0.8, 0.2);
    pub const ANSI_YELLOW: Color = Color::from_rgb(0.8, 0.8, 0.2);
    pub const ANSI_BLUE: Color = Color::from_rgb(0.2, 0.2, 0.8);
    pub const ANSI_MAGENTA: Color = Color::from_rgb(0.8, 0.2, 0.8);
    pub const ANSI_CYAN: Color = Color::from_rgb(0.2, 0.8, 0.8);
    pub const ANSI_WHITE: Color = Color::from_rgb(0.8, 0.8, 0.8);
    // Bright variants
    pub const ANSI_BRIGHT_BLACK: Color = Color::from_rgb(0.5, 0.5, 0.5);
    pub const ANSI_BRIGHT_RED: Color = Color::from_rgb(1.0, 0.3, 0.3);
    pub const ANSI_BRIGHT_GREEN: Color = Color::from_rgb(0.3, 1.0, 0.3);
    pub const ANSI_BRIGHT_YELLOW: Color = Color::from_rgb(1.0, 1.0, 0.3);
    pub const ANSI_BRIGHT_BLUE: Color = Color::from_rgb(0.3, 0.3, 1.0);
    pub const ANSI_BRIGHT_MAGENTA: Color = Color::from_rgb(1.0, 0.3, 1.0);
    pub const ANSI_BRIGHT_CYAN: Color = Color::from_rgb(0.3, 1.0, 1.0);
    pub const ANSI_BRIGHT_WHITE: Color = Color::from_rgb(1.0, 1.0, 1.0);
}

// ============================================================================
// ANSI Color Parsing
// ============================================================================

/// A styled text span with optional color
#[derive(Clone, Debug)]
#[allow(dead_code)]
struct StyledSpan {
    text: String,
    color: Option<Color>,
    bold: bool,
}

/// Parse ANSI-colored text into styled spans
fn parse_ansi_text(input: &str) -> Vec<StyledSpan> {
    let mut spans = Vec::new();
    let mut current_text = String::new();
    let mut current_color: Option<Color> = None;
    let mut bold = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Save current text if any
            if !current_text.is_empty() {
                spans.push(StyledSpan {
                    text: std::mem::take(&mut current_text),
                    color: current_color,
                    bold,
                });
            }

            // Parse escape sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                let mut codes = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() || ch == ';' {
                        codes.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                // Consume the command character (usually 'm')
                if chars.peek() == Some(&'m') {
                    chars.next();
                }

                // Parse SGR codes
                for code_str in codes.split(';') {
                    if let Ok(code) = code_str.parse::<u8>() {
                        match code {
                            0 => { current_color = None; bold = false; } // Reset
                            1 => bold = true,
                            22 => bold = false,
                            30 => current_color = Some(theme::ANSI_BLACK),
                            31 => current_color = Some(theme::ANSI_RED),
                            32 => current_color = Some(theme::ANSI_GREEN),
                            33 => current_color = Some(theme::ANSI_YELLOW),
                            34 => current_color = Some(theme::ANSI_BLUE),
                            35 => current_color = Some(theme::ANSI_MAGENTA),
                            36 => current_color = Some(theme::ANSI_CYAN),
                            37 => current_color = Some(theme::ANSI_WHITE),
                            39 => current_color = None, // Default foreground
                            90 => current_color = Some(theme::ANSI_BRIGHT_BLACK),
                            91 => current_color = Some(theme::ANSI_BRIGHT_RED),
                            92 => current_color = Some(theme::ANSI_BRIGHT_GREEN),
                            93 => current_color = Some(theme::ANSI_BRIGHT_YELLOW),
                            94 => current_color = Some(theme::ANSI_BRIGHT_BLUE),
                            95 => current_color = Some(theme::ANSI_BRIGHT_MAGENTA),
                            96 => current_color = Some(theme::ANSI_BRIGHT_CYAN),
                            97 => current_color = Some(theme::ANSI_BRIGHT_WHITE),
                            _ => {} // Ignore other codes
                        }
                    }
                }
            } else if chars.peek() == Some(&']') {
                // OSC sequence - skip until BEL or ST
                chars.next();
                while let Some(&ch) = chars.peek() {
                    chars.next();
                    if ch == '\x07' || ch == '\\' {
                        break;
                    }
                }
            }
        } else if c == '\r' {
            // Skip carriage return
        } else {
            current_text.push(c);
        }
    }

    // Don't forget the last span
    if !current_text.is_empty() {
        spans.push(StyledSpan {
            text: current_text,
            color: current_color,
            bold,
        });
    }

    spans
}

// ============================================================================
// Command Block - Warp-style grouped command + output
// ============================================================================

#[derive(Clone)]
struct CommandBlock {
    command: String,
    output: Vec<String>,  // Raw output with ANSI codes preserved
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

/// Global log buffer for debug panel (initialized once at startup)
static LOG_BUFFER: std::sync::OnceLock<LogBuffer> = std::sync::OnceLock::new();

fn main() -> iced::Result {
    // Initialize logging system
    let logging_config = LoggingConfig::default();
    let log_buffer = logging::init_logging(&logging_config);

    // Store log buffer globally for access by DebugPanel
    LOG_BUFFER.set(log_buffer).expect("LOG_BUFFER already initialized");

    tracing::info!("AgTerm starting");

    iced::application("AgTerm - AI Agent Terminal", AgTerm::update, AgTerm::view)
        .subscription(AgTerm::subscription)
        .font(D2CODING_FONT)
        .run()
}

/// Input field ID for focusing (Block mode)
fn input_id() -> TextInputId {
    TextInputId::new("terminal_input")
}

/// Raw mode input field ID for IME support
fn raw_input_id() -> TextInputId {
    TextInputId::new("raw_terminal_input")
}

/// Main application state
struct AgTerm {
    tabs: Vec<TerminalTab>,
    active_tab: usize,
    pty_manager: Arc<PtyManager>,
    next_tab_id: usize,
    startup_focus_count: u8,
    /// Debug panel state
    debug_panel: DebugPanel,
}

impl Default for AgTerm {
    fn default() -> Self {
        tracing::debug!("Initializing AgTerm application");
        let pty_manager = Arc::new(PtyManager::new());
        let session_result = pty_manager.create_session(24, 80);
        let cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "~".to_string());

        let (session_id, error_message) = match session_result {
            Ok(id) => {
                tracing::info!(session_id = %id, "Initial PTY session created");
                (Some(id), None)
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to create initial PTY session");
                (None, Some(format!("Failed to create PTY session: {}", e)))
            }
        };

        let tab = TerminalTab {
            id: 0,
            session_id,
            blocks: Vec::new(),
            pending_output: Vec::new(),
            raw_output_buffer: String::new(),
            raw_input: String::new(),
            input: String::new(),
            cwd,
            error_message,
            history: Vec::new(),
            history_index: None,
            history_temp_input: String::new(),
            mode: TerminalMode::Raw,  // Default to Raw mode for interactive apps
        };

        let mut debug_panel = DebugPanel::new();
        // Connect log buffer to debug panel
        if let Some(log_buffer) = LOG_BUFFER.get() {
            debug_panel.set_log_buffer(log_buffer.clone());
        }

        tracing::info!("AgTerm application initialized");
        Self {
            tabs: vec![tab],
            active_tab: 0,
            pty_manager,
            next_tab_id: 1,
            startup_focus_count: 10,
            debug_panel,
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
    raw_output_buffer: String,    // Raw PTY output for Raw mode display
    raw_input: String,            // Input buffer for Raw mode (IME support)
    input: String,
    cwd: String,  // Current working directory display
    error_message: Option<String>,  // PTY error message if creation failed
    // Command history
    history: Vec<String>,
    history_index: Option<usize>,  // Current position in history (None = not browsing)
    history_temp_input: String,    // Temporary storage for current input when browsing
    // Terminal mode
    mode: TerminalMode,
}

/// Terminal input mode
#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum TerminalMode {
    /// Raw mode: all key input goes directly to PTY (for interactive apps like vim, Claude Code)
    #[default]
    Raw,
    /// Block mode: command input via text field, output in blocks (Warp-style)
    Block,
}

/// Signal types for terminal control
#[derive(Debug, Clone, Copy)]
enum SignalType {
    Interrupt,  // Ctrl+C (0x03)
    EOF,        // Ctrl+D (0x04)
    Suspend,    // Ctrl+Z (0x1A)
}

impl SignalType {
    /// Convert signal type to its corresponding byte value
    fn as_byte(self) -> u8 {
        match self {
            SignalType::Interrupt => 0x03,  // Ctrl+C
            SignalType::EOF => 0x04,         // Ctrl+D
            SignalType::Suspend => 0x1A,     // Ctrl+Z
        }
    }
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

    // Terminal input (Block mode)
    InputChanged(String),
    SubmitInput,
    #[allow(dead_code)]
    FocusInput,

    // Raw input (Raw mode)
    RawInput(String),
    RawInputChanged(String),
    RawInputSubmit,

    // Mode toggle
    ToggleMode,

    // History navigation
    HistoryPrevious,
    HistoryNext,

    // Keyboard events
    KeyPressed(Key, Modifiers),

    // Signal sending
    SendSignal(SignalType),

    // Clipboard
    CopyToClipboard(String),
    ClipboardContent(Option<String>),

    // Window resize
    WindowResized { width: u32, height: u32 },

    // Tick for PTY polling
    Tick,

    // Debug panel
    ToggleDebugPanel,
    #[allow(dead_code)]
    DebugPanelMessage(DebugPanelMessage),
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
                    raw_output_buffer: String::new(),
                    raw_input: String::new(),
                    input: String::new(),
                    cwd,
                    error_message,
                    history: Vec::new(),
                    history_index: None,
                    history_temp_input: String::new(),
                    mode: TerminalMode::Raw,
                };
                self.tabs.push(tab);
                self.active_tab = self.tabs.len() - 1;
                text_input::focus(raw_input_id())
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

            Message::RawInput(input) => {
                // Send raw input directly to PTY (Raw mode)
                if let Some(tab) = self.tabs.get(self.active_tab) {
                    if let Some(session_id) = &tab.session_id {
                        let _ = self.pty_manager.write(session_id, input.as_bytes());
                    }
                }
                Task::none()
            }

            Message::RawInputChanged(new_input) => {
                // Handle text input in Raw mode (for IME/Korean support)
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    let old_len = tab.raw_input.chars().count();
                    let new_len = new_input.chars().count();

                    if let Some(session_id) = &tab.session_id {
                        if new_len > old_len {
                            // Characters were added - send only the new chars to PTY
                            let added: String = new_input.chars().skip(old_len).collect();
                            let _ = self.pty_manager.write(session_id, added.as_bytes());
                        } else if new_len < old_len {
                            // Characters were deleted - send backspace
                            let deleted_count = old_len - new_len;
                            for _ in 0..deleted_count {
                                let _ = self.pty_manager.write(session_id, &[0x7f]); // Backspace
                            }
                        }
                    }
                    tab.raw_input = new_input;
                }
                Task::none()
            }

            Message::RawInputSubmit => {
                // Enter key in Raw mode - send newline and clear input
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    if let Some(session_id) = &tab.session_id {
                        let _ = self.pty_manager.write(session_id, b"\r");
                    }
                    tab.raw_input.clear();
                }
                text_input::focus(raw_input_id())
            }

            Message::ToggleMode => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    tab.mode = match tab.mode {
                        TerminalMode::Raw => TerminalMode::Block,
                        TerminalMode::Block => TerminalMode::Raw,
                    };
                }
                Task::none()
            }

            Message::KeyPressed(key, modifiers) => {
                // Get current mode
                let current_mode = self.tabs.get(self.active_tab)
                    .map(|t| t.mode)
                    .unwrap_or(TerminalMode::Raw);

                // Handle Ctrl key signals (both modes)
                if modifiers.control() {
                    match key.as_ref() {
                        Key::Character("c") => return self.update(Message::SendSignal(SignalType::Interrupt)),
                        Key::Character("d") => return self.update(Message::SendSignal(SignalType::EOF)),
                        Key::Character("z") => return self.update(Message::SendSignal(SignalType::Suspend)),
                        _ => {}
                    }
                }

                // Handle keyboard shortcuts (Cmd key - both modes)
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
                        Key::Character("v") => return iced::clipboard::read().map(Message::ClipboardContent),
                        Key::Character("m") => return self.update(Message::ToggleMode),  // Toggle mode
                        Key::Character("d") => return self.update(Message::ToggleDebugPanel),  // Toggle debug panel
                        _ => {}
                    }
                }

                // F12 to toggle debug panel (no modifier needed)
                if matches!(key.as_ref(), Key::Named(keyboard::key::Named::F12)) {
                    return self.update(Message::ToggleDebugPanel);
                }

                // Raw mode: send special keys directly to PTY
                // NOTE: Regular characters are handled by text_input (for IME support)
                if current_mode == TerminalMode::Raw && !modifiers.command() {
                    let input = match key.as_ref() {
                        // Only handle special/named keys here
                        // Regular characters go through text_input for IME support
                        Key::Named(keyboard::key::Named::Escape) => Some("\x1b".to_string()),
                        Key::Named(keyboard::key::Named::ArrowUp) => Some("\x1b[A".to_string()),
                        Key::Named(keyboard::key::Named::ArrowDown) => Some("\x1b[B".to_string()),
                        Key::Named(keyboard::key::Named::ArrowRight) => Some("\x1b[C".to_string()),
                        Key::Named(keyboard::key::Named::ArrowLeft) => Some("\x1b[D".to_string()),
                        Key::Named(keyboard::key::Named::Home) => Some("\x1b[H".to_string()),
                        Key::Named(keyboard::key::Named::End) => Some("\x1b[F".to_string()),
                        Key::Named(keyboard::key::Named::PageUp) => Some("\x1b[5~".to_string()),
                        Key::Named(keyboard::key::Named::PageDown) => Some("\x1b[6~".to_string()),
                        Key::Named(keyboard::key::Named::Delete) => Some("\x1b[3~".to_string()),
                        Key::Named(keyboard::key::Named::Insert) => Some("\x1b[2~".to_string()),
                        // Tab key - send directly (text_input uses it for focus)
                        Key::Named(keyboard::key::Named::Tab) => Some("\t".to_string()),
                        _ => None,
                    };

                    if let Some(input_str) = input {
                        return self.update(Message::RawInput(input_str));
                    }
                }

                // Block mode: history navigation with arrow keys
                if current_mode == TerminalMode::Block {
                    match key.as_ref() {
                        Key::Named(keyboard::key::Named::ArrowUp) => {
                            return self.update(Message::HistoryPrevious);
                        }
                        Key::Named(keyboard::key::Named::ArrowDown) => {
                            return self.update(Message::HistoryNext);
                        }
                        _ => {}
                    }
                }

                Task::none()
            }

            Message::SendSignal(signal_type) => {
                // Send signal to active PTY session
                if let Some(tab) = self.tabs.get(self.active_tab) {
                    if let Some(session_id) = &tab.session_id {
                        let signal_byte = signal_type.as_byte();
                        let _ = self.pty_manager.write(session_id, &[signal_byte]);
                    }
                }
                Task::none()
            }

            Message::CopyToClipboard(content) => {
                iced::clipboard::write(content)
            }

            Message::ClipboardContent(clipboard_opt) => {
                if let Some(content) = clipboard_opt {
                    if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                        match tab.mode {
                            TerminalMode::Raw => {
                                // In Raw mode, send clipboard content directly to PTY
                                if let Some(session_id) = &tab.session_id {
                                    let _ = self.pty_manager.write(session_id, content.as_bytes());
                                }
                            }
                            TerminalMode::Block => {
                                // In Block mode, append to input field
                                tab.input.push_str(&content);
                            }
                        }
                    }
                }
                Task::none()
            }

            Message::WindowResized { width, height } => {
                // Calculate terminal dimensions based on approximate character size
                // D2Coding at ~13px = roughly 8px width, 18px height per character
                let cols = ((width as f32 / 8.0).max(80.0)) as u16;
                let rows = ((height as f32 / 18.0).max(24.0)) as u16;

                // Resize all active PTY sessions
                for tab in &self.tabs {
                    if let Some(session_id) = &tab.session_id {
                        let _ = self.pty_manager.resize(session_id, rows, cols);
                    }
                }
                Task::none()
            }

            Message::ToggleDebugPanel => {
                self.debug_panel.toggle();
                Task::none()
            }

            Message::DebugPanelMessage(msg) => {
                self.debug_panel.update(msg);
                Task::none()
            }

            Message::Tick => {
                // Record frame for metrics
                self.debug_panel.metrics.record_frame();

                // Update input debug state
                if let Some(tab) = self.tabs.get(self.active_tab) {
                    self.debug_panel.input_state.raw_mode = tab.mode == TerminalMode::Raw;
                }

                // Auto-focus on raw input for IME support
                let focus_task = if self.startup_focus_count > 0 {
                    self.startup_focus_count -= 1;
                    if let Some(tab) = self.tabs.get(self.active_tab) {
                        match tab.mode {
                            TerminalMode::Raw => text_input::focus(raw_input_id()),
                            TerminalMode::Block => text_input::focus(input_id()),
                        }
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                };

                // Poll PTY output for all tabs
                for tab in &mut self.tabs {
                    if let Some(session_id) = &tab.session_id {
                        if let Ok(data) = self.pty_manager.read(session_id) {
                            if !data.is_empty() {
                                let raw_output = String::from_utf8_lossy(&data);

                                // Record PTY read metrics
                                self.debug_panel.metrics.record_pty_read(data.len());

                                match tab.mode {
                                    TerminalMode::Raw => {
                                        // In Raw mode, append to raw_output_buffer
                                        tab.raw_output_buffer.push_str(&raw_output);
                                        // Limit buffer size (keep last 100KB)
                                        const MAX_RAW_BUFFER: usize = 100 * 1024;
                                        if tab.raw_output_buffer.len() > MAX_RAW_BUFFER {
                                            let excess = tab.raw_output_buffer.len() - MAX_RAW_BUFFER;
                                            tab.raw_output_buffer.drain(0..excess);
                                        }
                                    }
                                    TerminalMode::Block => {
                                        // In Block mode, use existing block-based logic
                                        let output = &raw_output;
                                        let stripped = strip_ansi(&raw_output);

                                        if let Some(block) = tab.blocks.iter_mut().rev().find(|b| b.is_running) {
                                            for line in output.lines() {
                                                let trimmed_stripped = strip_ansi(line).trim().to_string();
                                                if !trimmed_stripped.is_empty() && trimmed_stripped != block.command {
                                                    block.output.push(line.to_string());
                                                    if block.output.len() > MAX_OUTPUT_LINES {
                                                        block.output.drain(0..1000);
                                                    }
                                                }
                                            }
                                            if block.timestamp.elapsed() > Duration::from_millis(500) && stripped.is_empty() {
                                                if block.is_running {
                                                    block.completed_at = Some(Instant::now());
                                                }
                                                block.is_running = false;
                                            }
                                        } else {
                                            for line in output.lines() {
                                                let trimmed = line.trim();
                                                if !trimmed.is_empty() {
                                                    tab.pending_output.push(trimmed.to_string());
                                                    if tab.pending_output.len() > 1000 {
                                                        tab.pending_output.drain(0..100);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else if tab.mode == TerminalMode::Block {
                                // Block mode: mark running blocks as complete
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

                    // Enforce maximum blocks per tab (Block mode)
                    if tab.mode == TerminalMode::Block && tab.blocks.len() > MAX_BLOCKS_PER_TAB {
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
                    let can_close = self.tabs.len() > 1;

                    let icon_color = if is_active { theme::TAB_ACTIVE } else { theme::TEXT_MUTED };
                    let label_color = if is_active { theme::TEXT_PRIMARY } else { theme::TEXT_SECONDARY };

                    // Tab label button (clickable to select)
                    let tab_label_button = button(
                        row![
                            text("▶").size(11).color(icon_color),
                            Space::with_width(8),
                            text(label.clone()).size(13).color(label_color)
                        ]
                        .align_y(Alignment::Center)
                    )
                    .padding([8, 12])
                    .style(move |_, status| {
                        let bg = match status {
                            button::Status::Hovered => {
                                if is_active { theme::BG_SECONDARY } else { theme::BG_BLOCK_HOVER }
                            }
                            _ => {
                                if is_active { theme::BG_SECONDARY } else { theme::BG_PRIMARY }
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
                                    top_right: 0.0,
                                    bottom_left: 0.0,
                                    bottom_right: 0.0,
                                },
                            },
                            ..Default::default()
                        }
                    })
                    .on_press(Message::SelectTab(i));

                    // Close button (separate, clickable to close)
                    let close_button = button(
                        text("×").size(14)
                    )
                    .padding([8, 10])
                    .style(move |_, status| {
                        let (bg, text_color) = match status {
                            button::Status::Hovered => {
                                (theme::BG_BLOCK_HOVER, theme::ACCENT_RED)
                            }
                            _ => {
                                let bg = if is_active { theme::BG_SECONDARY } else { theme::BG_PRIMARY };
                                (bg, theme::TEXT_MUTED)
                            }
                        };
                        button::Style {
                            background: Some(bg.into()),
                            text_color,
                            border: Border {
                                color: Color::TRANSPARENT,
                                width: 0.0,
                                radius: iced::border::Radius {
                                    top_left: 0.0,
                                    top_right: 6.0,
                                    bottom_left: 0.0,
                                    bottom_right: 0.0,
                                },
                            },
                            ..Default::default()
                        }
                    })
                    .on_press_maybe(if can_close { Some(Message::CloseTab(i)) } else { None });

                    // Tab content with accent line
                    let tab_content = column![
                        row![tab_label_button, close_button],
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
                    ];

                    container(tab_content).into()
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
            match tab.mode {
                TerminalMode::Raw => {
                    // ========== Raw Mode: Full terminal view ==========
                    let terminal_output = self.render_raw_terminal(&tab.raw_output_buffer);

                    // ========== Input Area (Raw Mode - for IME/Korean support) ==========
                    let prompt_symbol = text("❯").size(16).font(MONO_FONT).color(theme::PROMPT);
                    let raw_input_field = text_input("", &tab.raw_input)
                        .id(raw_input_id())
                        .on_input(Message::RawInputChanged)
                        .on_submit(Message::RawInputSubmit)
                        .padding([10, 14])
                        .size(14)
                        .font(MONO_FONT)
                        .style(|_, _| text_input::Style {
                            background: theme::BG_INPUT.into(),
                            border: Border {
                                color: theme::BORDER,
                                width: 1.0,
                                radius: 6.0.into(),
                            },
                            icon: theme::TEXT_MUTED,
                            placeholder: theme::TEXT_MUTED,
                            value: theme::TEXT_PRIMARY,
                            selection: theme::ACCENT_BLUE,
                        });

                    let input_row: Element<Message> = container(
                        row![
                            prompt_symbol,
                            Space::with_width(10),
                            raw_input_field
                        ]
                        .align_y(Alignment::Center)
                        .width(Length::Fill),
                    )
                    .padding([8, 12])
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

                    // ========== Status Bar (Raw Mode) ==========
                    let shell_name = self.get_shell_name();
                    let mode_indicator = text("RAW")
                        .size(11)
                        .color(theme::ACCENT_GREEN);
                    let tab_info = format!("Tab {} of {}", self.active_tab + 1, self.tabs.len());

                    let status_left = row![
                        text(shell_name).size(12).color(theme::TEXT_MUTED),
                        Space::with_width(12),
                        mode_indicator
                    ];

                    let status_center = text(tab_info)
                        .size(12)
                        .color(theme::TEXT_MUTED);

                    let status_right = text("⌘M Toggle | ⌘T New | ⌘W Close")
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
                        container(terminal_output)
                            .padding([8, 12])
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .style(|_| container::Style {
                                background: Some(theme::BG_BLOCK.into()),
                                ..Default::default()
                            }),
                        input_row,
                        status_bar
                    ]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
                }
                TerminalMode::Block => {
                    // ========== Block Mode: Command blocks view ==========
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

                    // ========== Input Area (Block Mode) ==========
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

                    // ========== Status Bar (Block Mode) ==========
                    let shell_name = self.get_shell_name();
                    let mode_indicator = text("BLOCK")
                        .size(11)
                        .color(theme::ACCENT_BLUE);
                    let tab_info = format!("Tab {} of {}", self.active_tab + 1, self.tabs.len());

                    let status_left = row![
                        text(shell_name).size(12).color(theme::TEXT_MUTED),
                        Space::with_width(12),
                        mode_indicator
                    ];

                    let status_center = text(tab_info)
                        .size(12)
                        .color(theme::TEXT_MUTED);

                    let status_right = text("⌘M Toggle | ⌘T New | ⌘W Close")
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
                }
            }
        } else {
            column![text("No terminal open").color(theme::TEXT_PRIMARY)].into()
        };

        // ========== Main Layout ==========
        let terminal_area = column![
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
        ]
        .width(Length::Fill);

        // Main content with optional debug panel
        let main_content: Element<Message> = if self.debug_panel.visible {
            let debug_panel_view: Element<Message> = self.debug_panel.view();
            row![
                terminal_area,
                debug_panel_view
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            terminal_area.height(Length::Fill).into()
        };

        container(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(theme::BG_PRIMARY.into()),
                ..Default::default()
            })
            .into()
    }

    /// Render raw terminal output (for Raw mode)
    fn render_raw_terminal(&self, buffer: &str) -> Element<Message> {
        // Parse the buffer and render with ANSI colors
        // Handle different line endings: \r\n, \n, or standalone \r
        let normalized = buffer
            .replace("\r\n", "\n")
            .replace('\r', "\n");

        // Split into lines and get the last N lines
        let lines: Vec<&str> = normalized.lines().collect();
        let display_lines: Vec<&str> = if lines.len() > 100 {
            lines[lines.len() - 100..].to_vec()
        } else {
            lines
        };

        let content: Element<Message> = if display_lines.is_empty() {
            // Show cursor indicator when empty
            text("█")
                .size(14)
                .font(MONO_FONT)
                .color(theme::TEXT_PRIMARY)
                .into()
        } else {
            column(
                display_lines
                    .iter()
                    .map(|line| {
                        // Parse ANSI codes for each line
                        let spans = parse_ansi_text(line);
                        if spans.is_empty() {
                            text("").size(14).font(MONO_FONT).into()
                        } else if spans.len() == 1 && spans[0].color.is_none() {
                            text(spans[0].text.clone())
                                .size(14)
                                .font(MONO_FONT)
                                .color(theme::TEXT_PRIMARY)
                                .into()
                        } else {
                            row(
                                spans.into_iter().map(|span| {
                                    let color = span.color.unwrap_or(theme::TEXT_PRIMARY);
                                    text(span.text)
                                        .size(14)
                                        .font(MONO_FONT)
                                        .color(color)
                                        .into()
                                }).collect::<Vec<Element<Message>>>()
                            ).into()
                        }
                    })
                    .collect::<Vec<Element<Message>>>(),
            )
            .spacing(2)
            .into()
        };

        scrollable(content)
            .height(Length::Fill)
            .width(Length::Fill)
            .style(|_, _| scrollable::Style {
                container: container::Style::default(),
                vertical_rail: scrollable::Rail {
                    background: Some(theme::BG_BLOCK.into()),
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
                    background: Some(theme::BG_BLOCK.into()),
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
                        // Parse ANSI codes and render with colors
                        let spans = parse_ansi_text(line);
                        if spans.is_empty() {
                            Space::with_height(0).into()
                        } else if spans.len() == 1 && spans[0].color.is_none() {
                            // Simple case: no colors, just text (clone to own)
                            text(spans[0].text.clone())
                                .size(13)
                                .font(MONO_FONT)
                                .color(theme::TEXT_SECONDARY)
                                .into()
                        } else {
                            // Multiple spans with colors
                            row(
                                spans.into_iter().map(|span| {
                                    let color = span.color.unwrap_or(theme::TEXT_SECONDARY);
                                    text(span.text)
                                        .size(13)
                                        .font(MONO_FONT)
                                        .color(color)
                                        .into()
                                }).collect::<Vec<Element<Message>>>()
                            ).into()
                        }
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

        // Listen for window resize events
        let window_events = iced::event::listen_with(|event, _status, _id| {
            if let iced::Event::Window(iced::window::Event::Resized(size)) = event {
                Some(Message::WindowResized {
                    width: size.width as u32,
                    height: size.height as u32,
                })
            } else {
                None
            }
        });

        Subscription::batch([timer, keyboard, window_events])
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
            raw_output_buffer: String::new(),
            raw_input: String::new(),
            input: String::new(),
            cwd: "/test/path".to_string(),
            error_message: None,
            history: Vec::new(),
            history_index: None,
            history_temp_input: String::new(),
            mode: TerminalMode::Block, // Use Block mode for tests to maintain compatibility
        };

        AgTerm {
            tabs: vec![tab],
            active_tab: 0,
            pty_manager,
            next_tab_id: 1,
            startup_focus_count: 0,
            debug_panel: DebugPanel::new(),
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
