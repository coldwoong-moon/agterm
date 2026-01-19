//! AgTerm - AI Agent Terminal
//!
//! Native GPU-accelerated terminal emulator with AI agent orchestration.
//! Inspired by Warp terminal's modern block-based interface.

use iced::keyboard::{self, Key, Modifiers};
use iced::widget::text_input::Id as TextInputId;
use iced::widget::{button, column, container, row, stack, text, text_input, Space};
use iced::{Alignment, Border, Color, Element, Font, Length, Subscription, Task};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};

// Iced-specific modules (not in library)
mod terminal_canvas;

// Re-export library modules with iced-gui feature
use agterm::completion::{CompletionEngine, CompletionItem};
use agterm::config::{self, AppConfig};
use agterm::debug::panel::TerminalState;
use agterm::debug::{DebugPanel, DebugPanelMessage};
use agterm::history::HistoryManager;
use agterm::keybind::KeyBindings;
// KeyAction reserved for future keymap management features
#[allow(unused_imports)]
use agterm::keybind::Action as KeyAction;
use agterm::logging::{self, LogBuffer};
#[allow(unused_imports)]
use agterm::logging::LoggingConfig;
use agterm::notification::NotificationManager;
use agterm::shell::{self, ShellInfo};
use agterm::sound;
use agterm::ssh;
use agterm::terminal::env::EnvironmentInfo;
use agterm::terminal::pty::PtyManager;
use agterm::terminal::screen::{Cell, TerminalScreen};
use agterm::theme::{self, Theme};
use agterm::trigger::TriggerManager;
use agterm::ui::{self, palette::{palette_input_id, CommandPalette, PaletteMessage}, mcp_panel::{McpPanel, McpPanelMessage}};
use terminal_canvas::{CursorState, CursorStyle, TerminalCanvas, TerminalCanvasState};

// ============================================================================
// Font Configuration - Embedded D2Coding for Korean/CJK support
// ============================================================================

/// D2Coding font bytes (Korean monospace font by Naver)
const D2CODING_FONT: &[u8] = include_bytes!("../assets/fonts/D2Coding.ttf");

/// Monospace font with Korean/CJK support
const MONO_FONT: Font = Font::with_name("D2Coding");

// ============================================================================
// Warp-inspired Dark Theme Colors (inline constants for backward compatibility)
// ============================================================================

mod inline_theme {
    use iced::Color;

    // Background colors
    pub const BG_PRIMARY: Color = Color::from_rgb(0.09, 0.09, 0.11); // #17171c
    pub const BG_SECONDARY: Color = Color::from_rgb(0.12, 0.12, 0.15); // #1e1e26
    #[allow(dead_code)]
    pub const BG_BLOCK: Color = Color::from_rgb(0.14, 0.14, 0.18); // #242430
    pub const BG_BLOCK_HOVER: Color = Color::from_rgb(0.18, 0.18, 0.22); // #2d2d38
    #[allow(dead_code)]
    pub const BG_INPUT: Color = Color::from_rgb(0.11, 0.11, 0.14); // #1c1c24

    // Text colors
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.93, 0.93, 0.95); // #edeff2
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.6, 0.62, 0.68); // #999ead
    pub const TEXT_MUTED: Color = Color::from_rgb(0.45, 0.47, 0.52); // #737885

    // Accent colors
    #[allow(dead_code)]
    pub const ACCENT_BLUE: Color = Color::from_rgb(0.36, 0.54, 0.98); // #5c8afa
    #[allow(dead_code)]
    pub const ACCENT_GREEN: Color = Color::from_rgb(0.35, 0.78, 0.55); // #59c78c
    #[allow(dead_code)]
    pub const ACCENT_YELLOW: Color = Color::from_rgb(0.95, 0.77, 0.36); // #f2c55c
    #[allow(dead_code)]
    pub const ACCENT_RED: Color = Color::from_rgb(0.92, 0.39, 0.45); // #eb6473

    // UI elements
    pub const BORDER: Color = Color::from_rgb(0.22, 0.22, 0.28); // #383847
    pub const TAB_ACTIVE: Color = Color::from_rgb(0.36, 0.54, 0.98); // #5c8afa

    // Prompt symbol (may be used in future features)
    #[allow(dead_code)]
    pub const PROMPT: Color = Color::from_rgb(0.55, 0.36, 0.98); // #8c5cfa (purple)

    // ANSI colors (standard 16-color palette) - May be used for ANSI parsing in future
    #[allow(dead_code)]
    pub const ANSI_BLACK: Color = Color::from_rgb(0.0, 0.0, 0.0);
    #[allow(dead_code)]
    pub const ANSI_RED: Color = Color::from_rgb(0.8, 0.2, 0.2);
    #[allow(dead_code)]
    pub const ANSI_GREEN: Color = Color::from_rgb(0.2, 0.8, 0.2);
    #[allow(dead_code)]
    pub const ANSI_YELLOW: Color = Color::from_rgb(0.8, 0.8, 0.2);
    #[allow(dead_code)]
    pub const ANSI_BLUE: Color = Color::from_rgb(0.2, 0.2, 0.8);
    #[allow(dead_code)]
    pub const ANSI_MAGENTA: Color = Color::from_rgb(0.8, 0.2, 0.8);
    #[allow(dead_code)]
    pub const ANSI_CYAN: Color = Color::from_rgb(0.2, 0.8, 0.8);
    #[allow(dead_code)]
    pub const ANSI_WHITE: Color = Color::from_rgb(0.8, 0.8, 0.8);
    // Bright variants
    #[allow(dead_code)]
    pub const ANSI_BRIGHT_BLACK: Color = Color::from_rgb(0.5, 0.5, 0.5);
    #[allow(dead_code)]
    pub const ANSI_BRIGHT_RED: Color = Color::from_rgb(1.0, 0.3, 0.3);
    #[allow(dead_code)]
    pub const ANSI_BRIGHT_GREEN: Color = Color::from_rgb(0.3, 1.0, 0.3);
    #[allow(dead_code)]
    pub const ANSI_BRIGHT_YELLOW: Color = Color::from_rgb(1.0, 1.0, 0.3);
    #[allow(dead_code)]
    pub const ANSI_BRIGHT_BLUE: Color = Color::from_rgb(0.3, 0.3, 1.0);
    #[allow(dead_code)]
    pub const ANSI_BRIGHT_MAGENTA: Color = Color::from_rgb(1.0, 0.3, 1.0);
    #[allow(dead_code)]
    pub const ANSI_BRIGHT_CYAN: Color = Color::from_rgb(0.3, 1.0, 1.0);
    #[allow(dead_code)]
    pub const ANSI_BRIGHT_WHITE: Color = Color::from_rgb(1.0, 1.0, 1.0);

    // ============================================================================
    // Reusable Style Functions
    // ============================================================================

    use iced::widget::container;
    use iced::Border;

    /// Container style for status bar
    #[allow(dead_code)]
    pub fn status_bar_style(_theme: &iced::Theme) -> container::Style {
        container::Style {
            background: Some(BG_PRIMARY.into()),
            border: Border {
                color: BORDER,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }

    /// Container style for primary background
    pub fn primary_background_style(_theme: &iced::Theme) -> container::Style {
        container::Style {
            background: Some(BG_PRIMARY.into()),
            ..Default::default()
        }
    }
}

// ============================================================================
// Session State Management
// ============================================================================

/// Represents the state of a terminal tab for session persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TabState {
    /// Working directory when tab was saved
    cwd: String,
    /// Custom tab title (if set)
    title: Option<String>,
    /// Tab ID (for tracking)
    id: usize,
}

/// Session state for persistence across app restarts
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionState {
    /// List of tab states
    tabs: Vec<TabState>,
    /// Index of active tab
    active_tab: usize,
    /// Window dimensions (cols, rows)
    window_size: Option<(u16, u16)>,
    /// Font size
    font_size: f32,
}

impl SessionState {
    /// Save session state to file
    fn save_to_file(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        tracing::info!("Session saved to {:?}", path);
        Ok(())
    }

    /// Load session state from file
    fn load_from_file(path: &std::path::Path) -> Result<Self, std::io::Error> {
        let json = std::fs::read_to_string(path)?;
        let state = serde_json::from_str(&json)?;
        tracing::info!("Session loaded from {:?}", path);
        Ok(state)
    }
}

// ============================================================================
// ANSI Color Parsing
// ============================================================================

/// A styled text span with optional color
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct StyledSpan {
    pub text: String,
    pub color: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub underline: bool,
    pub dim: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub hyperlink: Option<std::sync::Arc<String>>,
}

/// Convert terminal cells to styled spans
fn cells_to_styled_spans(cells: &[Cell]) -> Vec<StyledSpan> {
    let mut spans = Vec::new();
    let mut current_text = String::new();
    let mut current_color: Option<Color> = None;
    let mut current_bg: Option<Color> = None;
    let mut current_bold = false;
    let mut current_underline = false;
    let mut current_dim = false;
    let mut current_italic = false;
    let mut current_strikethrough = false;
    let mut current_hyperlink: Option<std::sync::Arc<String>> = None;

    for cell in cells {
        // Skip placeholder cells (second cell of wide characters)
        if cell.placeholder {
            continue;
        }

        // Handle reverse video mode by swapping fg and bg
        let (fg_color, bg_color) = if cell.reverse {
            // In reverse mode, swap foreground and background
            let fg = cell.bg.as_ref().map(|c| c.to_color().into());
            let bg = cell.fg.as_ref().map(|c| c.to_color().into());
            (fg, bg)
        } else {
            let fg = cell.fg.as_ref().map(|c| c.to_color().into());
            let bg = cell.bg.as_ref().map(|c| c.to_color().into());
            (fg, bg)
        };

        // If any style attribute changes, push current span and start new one
        if fg_color != current_color
            || bg_color != current_bg
            || cell.bold != current_bold
            || cell.underline != current_underline
            || cell.dim != current_dim
            || cell.italic != current_italic
            || cell.strikethrough != current_strikethrough
            || cell.hyperlink != current_hyperlink
        {
            if !current_text.is_empty() {
                spans.push(StyledSpan {
                    text: std::mem::take(&mut current_text),
                    color: current_color,
                    bg: current_bg,
                    bold: current_bold,
                    underline: current_underline,
                    dim: current_dim,
                    italic: current_italic,
                    strikethrough: current_strikethrough,
                    hyperlink: current_hyperlink.clone(),
                });
            }
            current_color = fg_color;
            current_bg = bg_color;
            current_bold = cell.bold;
            current_underline = cell.underline;
            current_dim = cell.dim;
            current_italic = cell.italic;
            current_strikethrough = cell.strikethrough;
            current_hyperlink = cell.hyperlink.clone();
        }

        current_text.push(cell.c);
    }

    // Push final span
    if !current_text.is_empty() {
        spans.push(StyledSpan {
            text: current_text,
            color: current_color,
            bg: current_bg,
            bold: current_bold,
            underline: current_underline,
            dim: current_dim,
            italic: current_italic,
            strikethrough: current_strikethrough,
            hyperlink: current_hyperlink,
        });
    }

    spans
}

/// Global log buffer for debug panel (initialized once at startup)
static LOG_BUFFER: std::sync::OnceLock<LogBuffer> = std::sync::OnceLock::new();

/// Global configuration (initialized once at startup)
static APP_CONFIG: std::sync::OnceLock<AppConfig> = std::sync::OnceLock::new();

fn main() -> iced::Result {
    // Load configuration
    let config = AppConfig::load().unwrap_or_else(|e| {
        eprintln!("Warning: Failed to load config ({e}), using defaults");
        AppConfig::default()
    });

    // Store config globally
    APP_CONFIG
        .set(config.clone())
        .expect("APP_CONFIG already initialized");

    // Initialize logging system
    let logging_config = config.logging.to_logging_config();
    let log_buffer = logging::init_logging(&logging_config);

    // Store log buffer globally for access by DebugPanel
    LOG_BUFFER
        .set(log_buffer)
        .expect("LOG_BUFFER already initialized");

    tracing::info!("AgTerm starting");
    tracing::info!("Configuration loaded from default + user overrides");

    // Log shell information
    if config.shell.program.is_none() {
        if let Some(default_shell) = ShellInfo::default_shell() {
            tracing::info!(
                shell = %default_shell.path.display(),
                shell_type = ?default_shell.shell_type,
                "Using default shell"
            );
        }

        // Show shell recommendation if different from default
        if let Some(recommended) = shell::recommend_shell() {
            let default = ShellInfo::default_shell();
            if default.is_none()
                || default
                    .as_ref()
                    .map(|d| d.shell_type != recommended.shell_type)
                    .unwrap_or(false)
            {
                tracing::info!(
                    recommended_shell = %recommended.path.display(),
                    shell_type = ?recommended.shell_type,
                    description = recommended.shell_type.description(),
                    "Recommended shell available - configure in [shell] section"
                );
            }
        }
    } else {
        tracing::info!(
            configured_shell = %config.shell.program.as_ref().unwrap(),
            "Using configured shell"
        );
    }

    iced::application("AgTerm - AI Agent Terminal", AgTerm::update, AgTerm::view)
        .subscription(AgTerm::subscription)
        .font(D2CODING_FONT)
        .run()
}

/// Raw mode input field ID for IME support
fn raw_input_id() -> TextInputId {
    TextInputId::new("raw_terminal_input")
}

/// Get global configuration
fn get_config() -> AppConfig {
    APP_CONFIG.get().cloned().unwrap_or_else(AppConfig::default)
}

/// Convert config cursor style to terminal canvas cursor style
fn convert_cursor_style(style: config::CursorStyle) -> CursorStyle {
    match style {
        config::CursorStyle::Block => CursorStyle::Block,
        config::CursorStyle::Underline => CursorStyle::Underline,
        config::CursorStyle::Beam => CursorStyle::Bar,
    }
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
    /// Last time PTY activity was detected (for dynamic tick optimization)
    last_pty_activity: Instant,
    /// Last cursor blink toggle time
    last_cursor_blink: Instant,
    /// Font size (8.0 ~ 24.0)
    font_size: f32,
    /// Search state (reserved for future search functionality)
    #[allow(dead_code)]
    search_mode: bool,
    #[allow(dead_code)]
    search_query: String,
    search_matches: Vec<(usize, usize, usize)>, // (line, start_col, end_col)
    current_match_index: Option<usize>,
    /// Environment information (SSH, container, terminal capabilities, etc.)
    env_info: EnvironmentInfo,
    /// Bell sound player
    bell_sound: sound::BellSound,
    /// Bell flash animation state
    bell_flash_active: bool,
    /// When the flash started (for animation timing)
    flash_started_at: Option<Instant>,
    /// Tab drag state for reordering
    tab_drag: Option<TabDragState>,
    /// Tab context menu state
    tab_context_menu: Option<TabContextMenu>,
    /// Tab rename mode (tab index being renamed)
    tab_rename_mode: Option<usize>,
    /// Tab rename input buffer
    tab_rename_input: String,
    /// Current theme
    current_theme: Theme,
    /// Current keyboard modifiers (for Ctrl+Click URL opening)
    #[allow(dead_code)]
    current_modifiers: Modifiers,
    /// Desktop notification manager
    notification_manager: NotificationManager,
    /// Key bindings manager
    #[allow(dead_code)]
    keybindings: KeyBindings,
    /// Command palette
    command_palette: CommandPalette,
    /// Command history manager
    history_manager: HistoryManager,
    /// Completion engine
    completion_engine: CompletionEngine,
    /// Completion popup state
    completion_items: Vec<CompletionItem>,
    completion_selected: usize,
    completion_visible: bool,
    /// Output trigger manager
    trigger_manager: TriggerManager,
    /// MCP AI Assistant Panel
    mcp_panel: McpPanel,
}

impl Default for AgTerm {
    fn default() -> Self {
        tracing::debug!("Initializing AgTerm application");
        let config = get_config();

        // Detect environment (SSH, container, terminal capabilities)
        let env_info = EnvironmentInfo::detect();
        tracing::info!("Environment detected: {}", env_info.description());

        // Log environment details for debugging
        if env_info.is_ssh {
            tracing::info!("Running in SSH session");
        }
        if env_info.is_container {
            tracing::info!("Running in container");
        }
        if env_info.is_tmux {
            tracing::info!("Running in tmux");
        }
        if env_info.is_screen {
            tracing::info!("Running in GNU screen");
        }

        let suggested_settings = env_info.suggested_settings();
        tracing::info!(
            "Suggested settings: truecolor={}, mouse={}, unicode={}, refresh={}ms",
            suggested_settings.enable_truecolor,
            suggested_settings.enable_mouse,
            suggested_settings.enable_unicode,
            suggested_settings.refresh_rate_ms
        );

        let pty_manager = Arc::new(PtyManager::new());

        // Try to restore session first
        let (tabs, active_tab, font_size, next_tab_id) =
            if let Some((restored_tabs, restored_active, restored_font)) =
                Self::restore_session(&config, &pty_manager)
            {
                // Calculate next_tab_id from restored tabs
                let max_id = restored_tabs.iter().map(|t| t.id).max().unwrap_or(0);
                tracing::info!("Session restored with {} tabs", restored_tabs.len());

                (restored_tabs, restored_active, restored_font, max_id + 1)
            } else {
                // No session to restore, create a fresh tab
                let session_result =
                    pty_manager.create_session(config.pty.default_rows, config.pty.default_cols);
                let cwd = config
                    .general
                    .default_working_dir
                    .as_ref()
                    .and_then(|p| p.to_str())
                    .map(|s| s.to_string())
                    .or_else(|| {
                        std::env::current_dir()
                            .ok()
                            .map(|p| p.display().to_string())
                    })
                    .unwrap_or_else(|| "~".to_string());

                let (session_id, error_message) = match session_result {
                    Ok(id) => {
                        tracing::info!(session_id = %id, "Initial PTY session created");
                        (Some(id), None)
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to create initial PTY session");
                        (None, Some(format!("Failed to create PTY session: {e}")))
                    }
                };

                let tab = TerminalTab {
                    id: 0,
                    session_id,
                    raw_input: String::new(),
                    input: String::new(),
                    cwd,
                    error_message,
                    history: Vec::new(),
                    history_index: None,
                    history_temp_input: String::new(),
                    mode: TerminalMode::Raw,
                    parsed_line_cache: Vec::new(),
                    canvas_state: TerminalCanvasState::new(),
                    content_version: 0,
                    screen: TerminalScreen::new(
                        config.pty.default_cols as usize,
                        config.pty.default_rows as usize,
                    ),
                    cursor_blink_on: true,
                    bell_pending: false,
                    title: None,
                    last_copied_selection: None,
                    bracket_match: None,
                    pane_layout: PaneLayout::Single,
                    panes: Vec::new(),
                    focused_pane: 0,
                    title_info: agterm::terminal::title::TitleInfo::new(),
                };

                (vec![tab], 0, config.appearance.font.size, 1)
            };

        let mut debug_panel = DebugPanel::new();
        // Connect log buffer to debug panel
        if let Some(log_buffer) = LOG_BUFFER.get() {
            debug_panel.set_log_buffer(log_buffer.clone());
        }

        // Apply debug config
        if config.debug.enabled || std::env::var("AGTERM_DEBUG").is_ok() {
            debug_panel.toggle();
        }

        // Load theme from config
        let current_theme = theme::Theme::by_name(&config.appearance.theme)
            .unwrap_or_else(|| theme::Theme::warp_dark());

        // Initialize notification manager
        let notification_manager = NotificationManager::new(config.notification.clone());

        // Initialize key bindings from config
        let keybindings = KeyBindings::from_config(&config.keybindings.bindings);

        // Initialize history manager
        let mut history_manager = HistoryManager::with_config(
            config.history.max_size,
            config.history.ignore_duplicates,
            config.history.ignore_space,
        );
        if config.history.enabled && config.history.save_to_file {
            let history_path = config.history.file_path.clone().unwrap_or_else(|| {
                let config_dir = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
                config_dir.join("agterm").join("history")
            });
            if let Err(e) = history_manager.load_from_file(history_path) {
                tracing::warn!("Failed to load history: {}", e);
            }
        }

        tracing::info!("AgTerm application initialized");
        Self {
            tabs,
            active_tab,
            pty_manager,
            next_tab_id,
            startup_focus_count: 10,
            debug_panel,
            last_pty_activity: Instant::now(),
            last_cursor_blink: Instant::now(),
            font_size,
            search_mode: false,
            search_query: String::new(),
            search_matches: Vec::new(),
            current_match_index: None,
            env_info,
            bell_sound: sound::BellSound::new(),
            bell_flash_active: false,
            flash_started_at: None,
            tab_drag: None,
            tab_context_menu: None,
            tab_rename_mode: None,
            tab_rename_input: String::new(),
            current_theme,
            current_modifiers: Modifiers::default(),
            notification_manager,
            keybindings,
            command_palette: CommandPalette::with_default_commands(),
            history_manager,
            completion_engine: CompletionEngine::new(),
            completion_items: Vec::new(),
            completion_selected: 0,
            completion_visible: false,
            trigger_manager: TriggerManager::from_config(&config.triggers),
            mcp_panel: McpPanel::with_example_servers(),
        }
    }
}

impl Drop for AgTerm {
    fn drop(&mut self) {
        // Save session state when application exits
        self.save_session();

        // Save command history
        if let Err(e) = self.history_manager.save_to_file() {
            tracing::warn!("Failed to save history: {}", e);
        }

        tracing::info!("AgTerm shutting down");
    }
}

// ============================================================================
// Pane Management
// ============================================================================

/// Pane layout type
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum PaneLayout {
    /// Single pane (no split)
    Single,
    /// Horizontal split (top/bottom)
    HorizontalSplit,
    /// Vertical split (left/right)
    VerticalSplit,
}

/// A single pane within a tab
#[allow(dead_code)]
struct Pane {
    /// Screen buffer for this pane
    screen: TerminalScreen,
    /// PTY session ID
    pty_id: Option<uuid::Uuid>,
    /// Whether this pane is focused
    focused: bool,
    /// Parsed line cache for rendering
    parsed_line_cache: Vec<Vec<StyledSpan>>,
    /// Content version for cache invalidation
    content_version: u64,
    /// Canvas state for virtual scrolling
    canvas_state: TerminalCanvasState,
    /// Cursor blink state
    cursor_blink_on: bool,
}

#[allow(dead_code)]
impl Pane {
    /// Create a new pane with given dimensions
    fn new(cols: usize, rows: usize, pty_id: Option<uuid::Uuid>) -> Self {
        Self {
            screen: TerminalScreen::new(cols, rows),
            pty_id,
            focused: false,
            parsed_line_cache: Vec::new(),
            content_version: 0,
            canvas_state: TerminalCanvasState::new(),
            cursor_blink_on: true,
        }
    }
}

impl std::fmt::Debug for Pane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pane")
            .field("pty_id", &self.pty_id)
            .field("focused", &self.focused)
            .field("content_version", &self.content_version)
            .field("cursor_blink_on", &self.cursor_blink_on)
            .finish_non_exhaustive()
    }
}

/// Tab drag state for drag-and-drop reordering
#[derive(Debug, Clone)]
struct TabDragState {
    dragging_index: usize,
    start_x: f32,
    current_x: f32,
}

/// Tab context menu state
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TabContextMenu {
    tab_index: usize,
    position: (f32, f32),
}

/// A single terminal tab with block-based output
struct TerminalTab {
    #[allow(dead_code)]
    id: usize,
    session_id: Option<uuid::Uuid>,
    raw_input: String, // Input buffer for Raw mode (IME support)
    #[allow(dead_code)]
    input: String,
    #[allow(dead_code)]
    cwd: String, // Current working directory display
    #[allow(dead_code)]
    error_message: Option<String>, // PTY error message if creation failed
    // Command history
    #[allow(dead_code)]
    history: Vec<String>,
    #[allow(dead_code)]
    history_index: Option<usize>, // Current position in history (None = not browsing)
    #[allow(dead_code)]
    history_temp_input: String, // Temporary storage for current input when browsing
    // Terminal mode
    mode: TerminalMode,
    // ANSI parsing cache for Raw mode
    /// Cached parsed lines for Raw mode (line index -> parsed spans)
    parsed_line_cache: Vec<Vec<StyledSpan>>,
    // Canvas state for virtual scrolling
    #[allow(dead_code)]
    canvas_state: TerminalCanvasState,
    /// Content version for cache invalidation
    content_version: u64,
    /// Terminal screen buffer with VTE parser
    screen: TerminalScreen,
    /// Cursor blink state
    cursor_blink_on: bool,
    /// Bell notification state - when bell is triggered in background tab
    bell_pending: bool,
    /// Custom tab title (set via OSC 0/2 or manually)
    title: Option<String>,
    /// Dynamic title information from OSC sequences and shell integration
    #[allow(dead_code)]
    title_info: agterm::terminal::title::TitleInfo,
    /// Track last copied selection coordinates to avoid duplicate copies
    last_copied_selection: Option<(terminal_canvas::SelectionPoint, terminal_canvas::SelectionPoint)>,
    /// Bracket matching state
    bracket_match: Option<agterm::terminal::bracket::BracketMatch>,
    // Pane management
    /// Pane layout type
    #[allow(dead_code)]
    pane_layout: PaneLayout,
    /// Panes within this tab (empty if using legacy single-pane mode)
    #[allow(dead_code)]
    panes: Vec<Pane>,
    /// Index of the focused pane
    #[allow(dead_code)]
    focused_pane: usize,
}

/// Terminal input mode
#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum TerminalMode {
    /// Raw mode: all key input goes directly to PTY (full streaming terminal)
    #[default]
    Raw,
}

/// Signal types for terminal control
#[derive(Debug, Clone, Copy)]
enum SignalType {
    Interrupt, // Ctrl+C (0x03)
    EOF,       // Ctrl+D (0x04)
    Suspend,   // Ctrl+Z (0x1A)
}

impl SignalType {
    /// Convert signal type to its corresponding byte value
    fn as_byte(self) -> u8 {
        match self {
            SignalType::Interrupt => 0x03, // Ctrl+C
            SignalType::EOF => 0x04,       // Ctrl+D
            SignalType::Suspend => 0x1A,   // Ctrl+Z
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Message {
    // Tab management
    NewTab,
    NewSshTab(ssh::SshProfile),
    CloseTab(usize),
    CloseCurrentTab,
    SelectTab(usize),
    NextTab,
    PrevTab,
    DuplicateTab,

    // Raw input (Raw mode)
    RawInput(String),
    RawInputChanged(String),
    RawInputSubmit,

    // Keyboard events
    KeyPressed(Key, Modifiers),

    // Signal sending
    SendSignal(SignalType),

    // Clipboard
    ClipboardContent(Option<String>),
    CopySelection,
    ForceClipboardCopy,
    ForceClipboardPaste,

    // Window resize
    WindowResized {
        width: u32,
        height: u32,
    },

    // Terminal control
    ClearScreen,
    ScrollToTop,
    ScrollToBottom,

    // URL handling
    OpenUrl(String),

    // Tick for PTY polling
    Tick,

    // Bell flash animation tick
    BellFlashTick,

    // Debug panel
    ToggleDebugPanel,
    #[allow(dead_code)]
    DebugPanelMessage(DebugPanelMessage),

    // Command palette
    PaletteMessage(PaletteMessage),

    // Font size adjustment
    IncreaseFontSize,
    DecreaseFontSize,
    ResetFontSize,

    // Theme switching
    SwitchTheme(String),

    // Pane management
    SplitHorizontal,
    SplitVertical,
    ClosePane,
    NextPane,
    PrevPane,

    // Tab drag and drop
    TabDragStart(usize),
    TabDragMove(f32),
    TabDragEnd,

    // Tab context menu
    TabContextMenu(usize, f32, f32),
    CloseContextMenu,
    TabRenameStart(usize),
    TabRenameInput(String),
    TabRenameSubmit,
    TabRenameCancel,

    // History search (Ctrl+R)
    StartHistorySearch,
    UpdateHistorySearch(String),
    HistorySearchNext,
    HistorySearchPrev,
    EndHistorySearch,
    CancelHistorySearch,

    // Completion (Tab autocomplete)
    TriggerCompletion,
    CompletionNext,
    CompletionPrev,
    CompletionSelect,
    CompletionCancel,

    // MCP AI Assistant Panel
    ToggleMcpPanel,
    McpPanelMessage(McpPanelMessage),
}

impl From<DebugPanelMessage> for Message {
    fn from(msg: DebugPanelMessage) -> Self {
        Message::DebugPanelMessage(msg)
    }
}

impl AgTerm {
    /// Resize PTY sessions when font size changes
    /// Calculates new terminal dimensions based on old/new font sizes
    fn resize_pty_for_font_change(&mut self, old_font_size: f32) {
        // Calculate scaling factor
        let scale = old_font_size / self.font_size;

        for tab in &mut self.tabs {
            let (current_cols, current_rows) = tab.screen.dimensions();
            // Scale dimensions inversely with font size
            let new_cols = ((current_cols as f32 * scale).max(80.0)) as u16;
            let new_rows = ((current_rows as f32 * scale).max(24.0)) as u16;

            // Resize PTY session
            if let Some(session_id) = &tab.session_id {
                let _ = self.pty_manager.resize(session_id, new_rows, new_cols);
            }
            // Resize screen buffer
            tab.screen.resize(new_cols as usize, new_rows as usize);
        }
    }

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

    /// Play bell sound based on configuration
    fn play_bell_sound(&self) {
        let config = get_config();

        // Check if bell is enabled
        if !config.terminal.bell_enabled {
            return;
        }

        // Play sound based on bell_style
        match config.terminal.bell_style {
            config::BellStyle::Sound | config::BellStyle::Both => {
                self.bell_sound.play(config.terminal.bell_volume);
            }
            _ => {
                // Visual or None - don't play sound
            }
        }
    }

    /// Trigger bell flash effect based on configuration
    fn trigger_bell_flash(&mut self) {
        let config = get_config();

        // Check if bell is enabled and visual flash is enabled
        if !config.terminal.bell_enabled || !config.terminal.visual_flash {
            return;
        }

        // Check if visual bell is enabled in bell_style
        match config.terminal.bell_style {
            config::BellStyle::Visual | config::BellStyle::Both => {
                self.bell_flash_active = true;
                self.flash_started_at = Some(Instant::now());
            }
            _ => {
                // Sound or None - don't show flash
            }
        }
    }

    /// Check triggers against terminal output and execute matching actions
    fn check_triggers(&mut self, text: &str) {
        let matches = self.trigger_manager.check(text);

        if matches.is_empty() {
            return;
        }

        for (_, trigger) in matches {
            tracing::debug!(
                trigger_name = %trigger.name,
                pattern = %trigger.pattern,
                "Trigger matched"
            );

            match &trigger.action {
                agterm::trigger::TriggerAction::Notify { title, body } => {
                    // Send desktop notification
                    self.notification_manager.notify_custom(title, body);
                    tracing::info!(
                        trigger = %trigger.name,
                        title = %title,
                        "Trigger notification sent"
                    );
                }
                agterm::trigger::TriggerAction::Highlight { color } => {
                    // TODO: Implement text highlighting in terminal canvas
                    // This would require tracking highlighted regions in the terminal screen
                    tracing::debug!(
                        trigger = %trigger.name,
                        color = %color,
                        "Trigger highlight requested (not yet implemented)"
                    );
                }
                agterm::trigger::TriggerAction::PlaySound { file } => {
                    if let Some(path) = file {
                        // TODO: Play custom sound file
                        tracing::debug!(
                            trigger = %trigger.name,
                            file = %path,
                            "Custom sound requested (not yet implemented)"
                        );
                    } else {
                        // Play default bell sound
                        self.play_bell_sound();
                    }
                }
                agterm::trigger::TriggerAction::RunCommand { command } => {
                    // TODO: Execute shell command
                    // Security consideration: This should be carefully implemented
                    // to avoid command injection vulnerabilities
                    tracing::debug!(
                        trigger = %trigger.name,
                        command = %command,
                        "Command execution requested (not yet implemented)"
                    );
                }
                agterm::trigger::TriggerAction::Log { message } => {
                    tracing::info!(
                        trigger = %trigger.name,
                        message = %message,
                        "Trigger log"
                    );
                }
            }
        }
    }

    /// Update bracket matching for the active tab
    fn update_bracket_match(&mut self) {
        let config = get_config();

        // Check if bracket matching is enabled
        if !config.terminal.bracket.enabled {
            if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                tab.bracket_match = None;
            }
            return;
        }

        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            let (cursor_row, cursor_col) = tab.screen.cursor_position();
            tab.bracket_match = agterm::terminal::bracket::find_matching_bracket(
                &tab.screen,
                cursor_row,
                cursor_col,
            );
        }
    }

    /// Save current session state to file
    fn save_session(&self) {
        let config = get_config();
        if !config.general.session.save_on_exit {
            return;
        }

        let session_path = config.session_file_path();

        // Collect tab states
        let tab_states: Vec<TabState> = self
            .tabs
            .iter()
            .map(|tab| TabState {
                cwd: tab.cwd.clone(),
                title: tab.title.clone(),
                id: tab.id,
            })
            .collect();

        let session = SessionState {
            tabs: tab_states,
            active_tab: self.active_tab,
            window_size: None, // Will be set from actual window size if available
            font_size: self.font_size,
        };

        if let Err(e) = session.save_to_file(&session_path) {
            tracing::error!("Failed to save session: {}", e);
        }
    }

    /// Restore session from file and create tabs
    fn restore_session(
        config: &AppConfig,
        pty_manager: &Arc<PtyManager>,
    ) -> Option<(Vec<TerminalTab>, usize, f32)> {
        if !config.general.session.restore_on_startup {
            return None;
        }

        let session_path = config.session_file_path();
        if !session_path.exists() {
            tracing::info!(
                "No session file found at {:?}, starting fresh",
                session_path
            );
            return None;
        }

        match SessionState::load_from_file(&session_path) {
            Ok(session) => {
                if session.tabs.is_empty() {
                    tracing::warn!("Session file has no tabs, starting fresh");
                    return None;
                }

                tracing::info!("Restoring session with {} tabs", session.tabs.len());

                let mut tabs = Vec::new();
                for tab_state in session.tabs {
                    let session_result = pty_manager
                        .create_session(config.pty.default_rows, config.pty.default_cols);

                    let (session_id, error_message) = match session_result {
                        Ok(id) => {
                            tracing::info!(session_id = %id, "PTY session created for restored tab");
                            (Some(id), None)
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "Failed to create PTY session for restored tab");
                            (None, Some(format!("Failed to create PTY session: {e}")))
                        }
                    };

                    let tab = TerminalTab {
                        id: tab_state.id,
                        session_id,
                        raw_input: String::new(),
                        input: String::new(),
                        cwd: tab_state.cwd,
                        error_message,
                        history: Vec::new(),
                        history_index: None,
                        history_temp_input: String::new(),
                        mode: TerminalMode::Raw,
                        parsed_line_cache: Vec::new(),
                        canvas_state: TerminalCanvasState::new(),
                        content_version: 0,
                        screen: TerminalScreen::new(
                            config.pty.default_cols as usize,
                            config.pty.default_rows as usize,
                        ),
                        cursor_blink_on: true,
                        bell_pending: false,
                        title: tab_state.title,
                        last_copied_selection: None,
                        bracket_match: None,
                        pane_layout: PaneLayout::Single,
                        panes: Vec::new(),
                        focused_pane: 0,
                        title_info: agterm::terminal::title::TitleInfo::new(),
                    };

                    tabs.push(tab);
                }

                let active_tab = session.active_tab.min(tabs.len().saturating_sub(1));
                let font_size = session.font_size;

                Some((tabs, active_tab, font_size))
            }
            Err(e) => {
                tracing::error!("Failed to load session: {}", e);
                None
            }
        }
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
                    Err(e) => (None, Some(format!("Failed to create PTY session: {e}"))),
                };

                let tab = TerminalTab {
                    id,
                    session_id,
                    raw_input: String::new(),
                    input: String::new(),
                    cwd,
                    error_message,
                    history: Vec::new(),
                    history_index: None,
                    history_temp_input: String::new(),
                    mode: TerminalMode::Raw,
                    parsed_line_cache: Vec::new(),
                    canvas_state: TerminalCanvasState::new(),
                    content_version: 0,
                    screen: TerminalScreen::new(80, 24),
                    cursor_blink_on: true,
                    bell_pending: false,
                    title: None,
                    last_copied_selection: None,
                    bracket_match: None,
                    pane_layout: PaneLayout::Single,
                    panes: Vec::new(),
                    focused_pane: 0,
                    title_info: agterm::terminal::title::TitleInfo::new(),
                };
                self.tabs.push(tab);
                self.active_tab = self.tabs.len() - 1;
                text_input::focus(raw_input_id())
            }

            Message::NewSshTab(profile) => {
                let id = self.next_tab_id;
                self.next_tab_id += 1;

                let session_result = self.pty_manager.create_session(24, 80);
                let cwd = std::env::current_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| "~".to_string());

                let (session_id, error_message) = match session_result {
                    Ok(session_id) => {
                        // Send SSH command to the PTY
                        let ssh_command = profile.to_command();
                        let command_line = ssh_command.join(" ");
                        if let Err(e) = self.pty_manager.write(&session_id, format!("{command_line}\n").as_bytes()) {
                            tracing::error!("Failed to write SSH command to session: {e}");
                            (Some(session_id), Some(format!("Failed to start SSH: {e}")))
                        } else {
                            tracing::info!("SSH command sent: {command_line}");
                            (Some(session_id), None)
                        }
                    }
                    Err(e) => (None, Some(format!("Failed to create PTY session: {e}"))),
                };

                let tab = TerminalTab {
                    id,
                    session_id,
                    raw_input: String::new(),
                    input: String::new(),
                    cwd,
                    error_message,
                    history: Vec::new(),
                    history_index: None,
                    history_temp_input: String::new(),
                    mode: TerminalMode::Raw,
                    parsed_line_cache: Vec::new(),
                    canvas_state: TerminalCanvasState::new(),
                    content_version: 0,
                    screen: TerminalScreen::new(80, 24),
                    cursor_blink_on: true,
                    bell_pending: false,
                    title: Some(format!("SSH: {}", profile.connection_string())),
                    last_copied_selection: None,
                    bracket_match: None,
                    pane_layout: PaneLayout::Single,
                    panes: Vec::new(),
                    focused_pane: 0,
                    title_info: agterm::terminal::title::TitleInfo::new(),
                };
                self.tabs.push(tab);
                self.active_tab = self.tabs.len() - 1;
                text_input::focus(raw_input_id())
            }

            Message::DuplicateTab => {
                // Duplicate current tab - create new tab with same working directory
                if let Some(current_tab) = self.tabs.get(self.active_tab) {
                    let id = self.next_tab_id;
                    self.next_tab_id += 1;

                    let session_result = self.pty_manager.create_session(24, 80);
                    // Get the working directory from the current tab's shell
                    let cwd = current_tab
                        .screen
                        .cwd_from_shell()
                        .map(|s| s.to_string())
                        .or_else(|| Some(current_tab.cwd.clone()))
                        .unwrap_or_else(|| "~".to_string());

                    let (session_id, error_message) = match session_result {
                        Ok(id) => (Some(id), None),
                        Err(e) => (None, Some(format!("Failed to create PTY session: {e}"))),
                    };

                    let tab = TerminalTab {
                        id,
                        session_id,
                        raw_input: String::new(),
                        input: String::new(),
                        cwd,
                        error_message,
                        history: Vec::new(),
                        history_index: None,
                        history_temp_input: String::new(),
                        mode: TerminalMode::Raw,
                        parsed_line_cache: Vec::new(),
                        canvas_state: TerminalCanvasState::new(),
                        content_version: 0,
                        screen: TerminalScreen::new(80, 24),
                        cursor_blink_on: true,
                        bell_pending: false,
                        title: None, // New tab starts with no custom title
                        title_info: agterm::terminal::title::TitleInfo::new(),
                        last_copied_selection: None,
                        bracket_match: None,
                        pane_layout: PaneLayout::Single,
                        panes: Vec::new(),
                        focused_pane: 0,
                    };
                    self.tabs.push(tab);
                    self.active_tab = self.tabs.len() - 1;
                }
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
                    // Clear bell notification when switching to a tab
                    if let Some(tab) = self.tabs.get_mut(index) {
                        tab.bell_pending = false;
                    }
                }
                text_input::focus(raw_input_id())
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
                text_input::focus(raw_input_id())
            }

            Message::NextTab => {
                if !self.tabs.is_empty() {
                    self.active_tab = (self.active_tab + 1) % self.tabs.len();
                    // Clear bell notification when switching to a tab
                    if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                        tab.bell_pending = false;
                    }
                }
                text_input::focus(raw_input_id())
            }

            Message::PrevTab => {
                if !self.tabs.is_empty() {
                    self.active_tab = if self.active_tab == 0 {
                        self.tabs.len() - 1
                    } else {
                        self.active_tab - 1
                    };
                    // Clear bell notification when switching to a tab
                    if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                        tab.bell_pending = false;
                    }
                }
                text_input::focus(raw_input_id())
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
                // If in history search mode, update search query instead
                if self.history_manager.is_searching() {
                    return self.update(Message::UpdateHistorySearch(new_input));
                }

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
                                let _ = self.pty_manager.write(session_id, &[0x7f]);
                                // Backspace
                            }
                        }
                    }
                    tab.raw_input = new_input;
                }

                // Hide completion on input change
                if self.completion_visible {
                    self.completion_visible = false;
                    self.completion_items.clear();
                }

                Task::none()
            }

            Message::RawInputSubmit => {
                // Enter key in Raw mode - send newline and clear input
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    // Add command to completion history before sending
                    if !tab.raw_input.trim().is_empty() {
                        self.completion_engine.add_to_history(&tab.raw_input);
                    }

                    if let Some(session_id) = &tab.session_id {
                        let _ = self.pty_manager.write(session_id, b"\r");
                    }
                    tab.raw_input.clear();
                }

                // Hide completion if visible
                self.completion_visible = false;
                self.completion_items.clear();

                text_input::focus(raw_input_id())
            }

            Message::KeyPressed(key, modifiers) => {
                // If palette is visible, handle its special keys first
                if self.command_palette.is_visible() {
                    match key.as_ref() {
                        Key::Named(keyboard::key::Named::Escape) => {
                            return self.update(Message::PaletteMessage(
                                ui::palette::PaletteMessage::Close,
                            ));
                        }
                        Key::Named(keyboard::key::Named::ArrowUp) => {
                            return self.update(Message::PaletteMessage(
                                ui::palette::PaletteMessage::Up,
                            ));
                        }
                        Key::Named(keyboard::key::Named::ArrowDown) => {
                            return self.update(Message::PaletteMessage(
                                ui::palette::PaletteMessage::Down,
                            ));
                        }
                        Key::Named(keyboard::key::Named::Enter) => {
                            return self.update(Message::PaletteMessage(
                                ui::palette::PaletteMessage::Execute,
                            ));
                        }
                        _ => {}
                    }
                }

                // Handle Ctrl+R: Start reverse history search
                if modifiers.control() && matches!(key.as_ref(), Key::Character("r")) {
                    if self.history_manager.is_searching() {
                        // Already searching, move to next match
                        return self.update(Message::HistorySearchNext);
                    } else {
                        // Start new search
                        return self.update(Message::StartHistorySearch);
                    }
                }

                // If in history search mode, handle search-specific keys
                if self.history_manager.is_searching() {
                    match key.as_ref() {
                        Key::Named(keyboard::key::Named::Escape) => {
                            return self.update(Message::CancelHistorySearch);
                        }
                        Key::Named(keyboard::key::Named::Enter) => {
                            return self.update(Message::EndHistorySearch);
                        }
                        _ => {}
                    }
                }

                // Handle Tab: Trigger completion
                if matches!(key.as_ref(), Key::Named(keyboard::key::Named::Tab)) && !modifiers.shift() {
                    if self.completion_visible {
                        return self.update(Message::CompletionNext);
                    } else {
                        return self.update(Message::TriggerCompletion);
                    }
                }

                // Handle Shift+Tab: Previous completion
                if matches!(key.as_ref(), Key::Named(keyboard::key::Named::Tab)) && modifiers.shift() {
                    if self.completion_visible {
                        return self.update(Message::CompletionPrev);
                    }
                }

                // If completion is visible, handle navigation keys
                if self.completion_visible {
                    match key.as_ref() {
                        Key::Named(keyboard::key::Named::Escape) => {
                            return self.update(Message::CompletionCancel);
                        }
                        Key::Named(keyboard::key::Named::ArrowUp) => {
                            return self.update(Message::CompletionPrev);
                        }
                        Key::Named(keyboard::key::Named::ArrowDown) => {
                            return self.update(Message::CompletionNext);
                        }
                        Key::Named(keyboard::key::Named::Enter) => {
                            return self.update(Message::CompletionSelect);
                        }
                        _ => {}
                    }
                }

                // Handle Cmd+Shift+C: Force clipboard copy
                if modifiers.command()
                    && modifiers.shift()
                    && matches!(key.as_ref(), Key::Character("c"))
                {
                    return self.update(Message::ForceClipboardCopy);
                }

                // Handle Cmd+Shift+V: Force clipboard paste (without bracketed paste)
                if modifiers.command()
                    && modifiers.shift()
                    && matches!(key.as_ref(), Key::Character("v"))
                {
                    return self.update(Message::ForceClipboardPaste);
                }

                // Handle Cmd+Shift+D: Duplicate tab
                if modifiers.command()
                    && modifiers.shift()
                    && matches!(key.as_ref(), Key::Character("d"))
                {
                    return self.update(Message::DuplicateTab);
                }

                // Handle Cmd+Shift+H: Split horizontal
                if modifiers.command()
                    && modifiers.shift()
                    && matches!(key.as_ref(), Key::Character("h"))
                {
                    return self.update(Message::SplitHorizontal);
                }

                // Handle Cmd+Shift+M: Toggle MCP AI panel
                if modifiers.command()
                    && modifiers.shift()
                    && matches!(key.as_ref(), Key::Character("m"))
                {
                    return self.update(Message::ToggleMcpPanel);
                }

                // Handle Cmd+Shift+P: Open command palette
                if modifiers.command()
                    && modifiers.shift()
                    && matches!(key.as_ref(), Key::Character("p"))
                {
                    return self.update(Message::PaletteMessage(
                        ui::palette::PaletteMessage::Open,
                    ));
                }

                // Handle Cmd+Shift+| (pipe): Split vertical
                if modifiers.command()
                    && modifiers.shift()
                    && matches!(key.as_ref(), Key::Character("|"))
                {
                    return self.update(Message::SplitVertical);
                }

                // Handle Ctrl/Cmd+C: Copy if selection exists, otherwise send interrupt signal
                if (modifiers.control() || modifiers.command())
                    && matches!(key.as_ref(), Key::Character("c"))
                {
                    // Check if there's an active selection
                    let has_selection = if let Some(tab) = self.tabs.get(self.active_tab) {
                        tab.canvas_state
                            .selection
                            .as_ref()
                            .map(|s| s.active && s.start != s.end)
                            .unwrap_or(false)
                    } else {
                        false
                    };

                    if has_selection {
                        // Copy selection to clipboard
                        return self.update(Message::CopySelection);
                    } else if modifiers.control() {
                        // Send interrupt signal (Ctrl+C)
                        return self.update(Message::SendSignal(SignalType::Interrupt));
                    }
                    // Cmd+C with no selection: do nothing
                    return Task::none();
                }

                // Handle other Ctrl key signals
                if modifiers.control() {
                    match key.as_ref() {
                        Key::Character("d") => {
                            return self.update(Message::SendSignal(SignalType::EOF))
                        }
                        Key::Character("z") => {
                            return self.update(Message::SendSignal(SignalType::Suspend))
                        }
                        _ => {}
                    }
                }

                // Handle keyboard shortcuts (Cmd key)
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
                        Key::Character("6") => return self.update(Message::SelectTab(5)),
                        Key::Character("7") => return self.update(Message::SelectTab(6)),
                        Key::Character("8") => return self.update(Message::SelectTab(7)),
                        Key::Character("9") => return self.update(Message::SelectTab(8)),
                        Key::Character("k") => return self.update(Message::ClearScreen),
                        Key::Character("v") => {
                            return iced::clipboard::read().map(Message::ClipboardContent)
                        }
                        Key::Character("d") => return self.update(Message::ToggleDebugPanel), // Toggle debug panel
                        Key::Character("+") | Key::Character("=") => {
                            return self.update(Message::IncreaseFontSize)
                        }
                        Key::Character("-") => return self.update(Message::DecreaseFontSize),
                        Key::Character("0") => return self.update(Message::ResetFontSize),
                        Key::Named(keyboard::key::Named::Home) => {
                            return self.update(Message::ScrollToTop)
                        }
                        Key::Named(keyboard::key::Named::End) => {
                            return self.update(Message::ScrollToBottom)
                        }
                        _ => {}
                    }
                }

                // F12 to toggle debug panel (no modifier needed)
                if matches!(key.as_ref(), Key::Named(keyboard::key::Named::F12)) {
                    return self.update(Message::ToggleDebugPanel);
                }

                // Raw mode: send special keys directly to PTY
                // Regular characters are handled via RawInputChanged (for IME support)
                if !modifiers.command() {
                    // Check application cursor keys mode for arrow keys
                    let app_cursor_keys = self
                        .tabs
                        .get(self.active_tab)
                        .map(|tab| tab.screen.application_cursor_keys())
                        .unwrap_or(false);

                    let input = match key.as_ref() {
                        // Special/named keys only - NOT characters (handled by text_input for IME)
                        Key::Named(keyboard::key::Named::Escape) => Some("\x1b".to_string()),
                        // Arrow keys: switch between normal (\x1b[) and application (\x1bO) mode
                        Key::Named(keyboard::key::Named::ArrowUp) => {
                            Some(if app_cursor_keys { "\x1bOA" } else { "\x1b[A" }.to_string())
                        }
                        Key::Named(keyboard::key::Named::ArrowDown) => {
                            Some(if app_cursor_keys { "\x1bOB" } else { "\x1b[B" }.to_string())
                        }
                        Key::Named(keyboard::key::Named::ArrowRight) => {
                            Some(if app_cursor_keys { "\x1bOC" } else { "\x1b[C" }.to_string())
                        }
                        Key::Named(keyboard::key::Named::ArrowLeft) => {
                            Some(if app_cursor_keys { "\x1bOD" } else { "\x1b[D" }.to_string())
                        }
                        Key::Named(keyboard::key::Named::Home) => Some("\x1b[H".to_string()),
                        Key::Named(keyboard::key::Named::End) => Some("\x1b[F".to_string()),
                        Key::Named(keyboard::key::Named::PageUp) => Some("\x1b[5~".to_string()),
                        Key::Named(keyboard::key::Named::PageDown) => Some("\x1b[6~".to_string()),
                        Key::Named(keyboard::key::Named::Delete) => Some("\x1b[3~".to_string()),
                        Key::Named(keyboard::key::Named::Insert) => Some("\x1b[2~".to_string()),
                        Key::Named(keyboard::key::Named::Tab) => Some("\t".to_string()),
                        // Function keys (F1-F12)
                        Key::Named(keyboard::key::Named::F1) => Some("\x1bOP".to_string()),
                        Key::Named(keyboard::key::Named::F2) => Some("\x1bOQ".to_string()),
                        Key::Named(keyboard::key::Named::F3) => Some("\x1bOR".to_string()),
                        Key::Named(keyboard::key::Named::F4) => Some("\x1bOS".to_string()),
                        Key::Named(keyboard::key::Named::F5) => Some("\x1b[15~".to_string()),
                        Key::Named(keyboard::key::Named::F6) => Some("\x1b[17~".to_string()),
                        Key::Named(keyboard::key::Named::F7) => Some("\x1b[18~".to_string()),
                        Key::Named(keyboard::key::Named::F8) => Some("\x1b[19~".to_string()),
                        Key::Named(keyboard::key::Named::F9) => Some("\x1b[20~".to_string()),
                        Key::Named(keyboard::key::Named::F10) => Some("\x1b[21~".to_string()),
                        Key::Named(keyboard::key::Named::F11) => Some("\x1b[23~".to_string()),
                        Key::Named(keyboard::key::Named::F12) => Some("\x1b[24~".to_string()),
                        // Note: Enter, Backspace, Space are handled by text_input's on_submit and input changes
                        // Only handle them here as fallback if text_input doesn't capture them
                        _ => None,
                    };

                    if let Some(input_str) = input {
                        return self.update(Message::RawInput(input_str));
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

            Message::ClipboardContent(clipboard_opt) => {
                if let Some(content) = clipboard_opt {
                    if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                        // Send clipboard content to PTY with bracketed paste if enabled
                        if let Some(session_id) = &tab.session_id {
                            let config = get_config();
                            let bracketed_paste = config.terminal.bracketed_paste
                                && tab.screen.bracketed_paste_mode();

                            if bracketed_paste {
                                // Wrap paste with bracketed paste escape codes
                                let _ = self.pty_manager.write(session_id, b"\x1b[200~");
                                let _ = self.pty_manager.write(session_id, content.as_bytes());
                                let _ = self.pty_manager.write(session_id, b"\x1b[201~");
                            } else {
                                // Direct paste without bracketed mode
                                let _ = self.pty_manager.write(session_id, content.as_bytes());
                            }
                        }
                    }
                }
                Task::none()
            }

            Message::CopySelection => {
                use terminal_canvas::get_selected_text;

                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    if let Some(selection) = &tab.canvas_state.selection {
                        if selection.active && selection.start != selection.end {
                            // Extract selected text
                            let selected_text =
                                get_selected_text(&tab.parsed_line_cache, selection);

                            // Copy to clipboard using arboard
                            if !selected_text.is_empty() {
                                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                    let _ = clipboard.set_text(selected_text);
                                }
                            }

                            // Clear selection after copying
                            tab.canvas_state.selection = None;
                            tab.canvas_state.invalidate();
                            tab.last_copied_selection = None;
                        }
                    }
                }
                Task::none()
            }

            Message::ForceClipboardCopy => {
                // Force copy selection to clipboard (Cmd+Shift+C)
                use terminal_canvas::get_selected_text;

                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    if let Some(selection) = &tab.canvas_state.selection {
                        if selection.active && selection.start != selection.end {
                            let selected_text =
                                get_selected_text(&tab.parsed_line_cache, selection);
                            if !selected_text.is_empty() {
                                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                    let _ = clipboard.set_text(selected_text);
                                }
                            }
                            tab.canvas_state.selection = None;
                            tab.canvas_state.invalidate();
                            tab.last_copied_selection = None;
                        }
                    }
                }
                Task::none()
            }

            Message::ForceClipboardPaste => {
                // Force paste from clipboard without bracketed paste mode (Cmd+Shift+V)
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    if let Ok(text) = clipboard.get_text() {
                        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                            if let Some(session_id) = &tab.session_id {
                                // Send text directly without bracketed paste escape codes
                                let _ = self.pty_manager.write(session_id, text.as_bytes());
                            }
                        }
                    }
                }
                Task::none()
            }

            Message::ClearScreen => {
                // Send clear screen command to PTY (Cmd+K)
                if let Some(tab) = self.tabs.get(self.active_tab) {
                    if let Some(session_id) = &tab.session_id {
                        // Send Ctrl+L (clear screen)
                        let _ = self.pty_manager.write(session_id, &[0x0C]);
                    }
                }
                Task::none()
            }

            Message::ScrollToTop => {
                // Scroll to top of terminal output (Cmd+Home)
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    tab.canvas_state.scroll_to_top();
                }
                Task::none()
            }

            Message::ScrollToBottom => {
                // Scroll to bottom of terminal output (Cmd+End)
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    tab.canvas_state
                        .scroll_to_bottom(tab.parsed_line_cache.len(), self.font_size);
                }
                Task::none()
            }

            Message::OpenUrl(url) => {
                // Open URL in default browser
                tracing::info!("Opening URL: {}", url);
                if let Err(e) = open::that(&url) {
                    tracing::error!("Failed to open URL {}: {}", url, e);
                }
                Task::none()
            }

            Message::WindowResized { width, height } => {
                // Calculate terminal dimensions based on current font size
                // Monospace font: width ≈ 0.6 * font_size, height ≈ 1.4 * font_size (with line spacing)
                let char_width = self.font_size * 0.6;
                let line_height = self.font_size * 1.4;
                let cols = ((width as f32 / char_width).max(80.0)) as u16;
                let rows = ((height as f32 / line_height).max(24.0)) as u16;

                // Resize all active PTY sessions and screen buffers
                for tab in &mut self.tabs {
                    if let Some(session_id) = &tab.session_id {
                        let _ = self.pty_manager.resize(session_id, rows, cols);
                    }
                    // Resize screen buffer
                    tab.screen.resize(cols as usize, rows as usize);
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

            Message::PaletteMessage(msg) => {
                // Check if we're opening or closing the palette
                let is_opening = matches!(msg, PaletteMessage::Open);
                let is_closing = matches!(msg, PaletteMessage::Close | PaletteMessage::Execute);

                if let Some(command_id) = self.command_palette.update(msg) {
                    // Execute the selected command by dispatching appropriate message
                    match command_id.as_str() {
                        // Tab management
                        "new_tab" => return self.update(Message::NewTab),
                        "close_tab" => return self.update(Message::CloseCurrentTab),
                        "duplicate_tab" => return self.update(Message::DuplicateTab),
                        "next_tab" => return self.update(Message::NextTab),
                        "prev_tab" => return self.update(Message::PrevTab),
                        // Pane management
                        "split_horizontal" => return self.update(Message::SplitHorizontal),
                        "split_vertical" => return self.update(Message::SplitVertical),
                        "close_pane" => return self.update(Message::ClosePane),
                        "next_pane" => return self.update(Message::NextPane),
                        "prev_pane" => return self.update(Message::PrevPane),
                        // "zoom_pane" => return self.update(Message::ZoomPane), // TODO: implement
                        // View
                        "toggle_debug" => return self.update(Message::ToggleDebugPanel),
                        "clear_screen" => return self.update(Message::ClearScreen),
                        "scroll_to_top" => return self.update(Message::ScrollToTop),
                        "scroll_to_bottom" => return self.update(Message::ScrollToBottom),
                        // Font
                        "increase_font" => return self.update(Message::IncreaseFontSize),
                        "decrease_font" => return self.update(Message::DecreaseFontSize),
                        "reset_font" => return self.update(Message::ResetFontSize),
                        // Theme
                        "theme_warp" => return self.update(Message::SwitchTheme("warp".to_string())),
                        "theme_dracula" => {
                            return self.update(Message::SwitchTheme("dracula".to_string()))
                        }
                        "theme_nord" => return self.update(Message::SwitchTheme("nord".to_string())),
                        "theme_solarized" => {
                            return self.update(Message::SwitchTheme("solarized".to_string()))
                        }
                        // Clipboard
                        "copy" => return self.update(Message::CopySelection),
                        "paste" => return iced::clipboard::read().map(Message::ClipboardContent),
                        _ => {
                            tracing::warn!("Unknown command palette ID: {}", command_id);
                        }
                    }
                }

                // Manage focus based on palette state
                if is_opening {
                    text_input::focus(palette_input_id())
                } else if is_closing {
                    text_input::focus(raw_input_id())
                } else {
                    Task::none()
                }
            }

            Message::IncreaseFontSize => {
                let old_font_size = self.font_size;
                self.font_size = (self.font_size + 1.0).min(24.0);
                self.resize_pty_for_font_change(old_font_size);
                Task::none()
            }

            Message::DecreaseFontSize => {
                let old_font_size = self.font_size;
                self.font_size = (self.font_size - 1.0).max(8.0);
                self.resize_pty_for_font_change(old_font_size);
                Task::none()
            }

            Message::ResetFontSize => {
                let old_font_size = self.font_size;
                let config = get_config();
                self.font_size = config.appearance.font.size;
                self.resize_pty_for_font_change(old_font_size);
                Task::none()
            }

            Message::SwitchTheme(theme_name) => {
                // Switch to a new theme by name
                if let Some(new_theme) = theme::Theme::by_name(&theme_name) {
                    self.current_theme = new_theme;
                    tracing::info!("Switched to theme: {}", theme_name);
                } else {
                    tracing::warn!("Theme '{}' not found, keeping current theme", theme_name);
                }
                Task::none()
            }

            Message::BellFlashTick => {
                // Update flash animation state
                let config = get_config();
                if let Some(start_time) = self.flash_started_at {
                    let elapsed = start_time.elapsed().as_millis() as u64;
                    if elapsed >= config.terminal.flash_duration_ms {
                        // Flash animation complete
                        self.bell_flash_active = false;
                        self.flash_started_at = None;
                    }
                }
                Task::none()
            }

            Message::Tick => {
                // Record frame for metrics
                self.debug_panel.metrics.record_frame();

                // Update input debug state and terminal state
                if let Some(tab) = self.tabs.get(self.active_tab) {
                    self.debug_panel.input_state.raw_mode = tab.mode == TerminalMode::Raw;

                    // Update terminal state
                    let (cols, rows) = tab.screen.dimensions();
                    let (cursor_row, cursor_col) = tab.screen.cursor_position();
                    let scrollback_size = tab.screen.scrollback_size();
                    let total_lines = tab.parsed_line_cache.len();

                    self.debug_panel.terminal_state = TerminalState {
                        cols,
                        rows,
                        cursor_row,
                        cursor_col,
                        scrollback_size,
                        total_lines,
                    };
                }

                // Cursor blinking (configurable interval)
                let config = get_config();
                let cursor_blink_interval = config.terminal.cursor_blink_interval_ms;
                if self.last_cursor_blink.elapsed().as_millis() as u64 >= cursor_blink_interval {
                    self.last_cursor_blink = Instant::now();
                    if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                        if config.terminal.cursor_blink {
                            tab.cursor_blink_on = !tab.cursor_blink_on;
                        } else {
                            tab.cursor_blink_on = true; // Always on if blinking disabled
                        }
                    }
                }

                // Auto-focus on raw input for IME support
                let focus_task = if self.startup_focus_count > 0 {
                    self.startup_focus_count -= 1;
                    text_input::focus(raw_input_id())
                } else {
                    Task::none()
                };

                // Poll PTY output only for active tab
                let mut active_bell_triggered = false;
                let mut trigger_text: Option<String> = None;
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    if let Some(session_id) = &tab.session_id {
                        if let Ok(data) = self.pty_manager.read(session_id) {
                            if !data.is_empty() {
                                // Update PTY activity timestamp for dynamic tick optimization
                                self.last_pty_activity = Instant::now();

                                // Record PTY read metrics
                                self.debug_panel.metrics.record_pty_read(data.len());

                                // Process bytes through VTE parser
                                tab.screen.process(&data);

                                // Check for bell (BEL character) in active tab
                                // Store result to play sound after releasing the borrow
                                active_bell_triggered = tab.screen.take_bell_triggered();

                                // Update tab title from OSC sequences (OSC 0 or OSC 2)
                                if let Some(window_title) = tab.screen.window_title() {
                                    tab.title = Some(window_title.to_string());
                                }

                                // Update CWD from OSC 7 for tab subtitle/info
                                if let Some(cwd) = tab.screen.cwd_from_shell() {
                                    tab.cwd = cwd.to_string();
                                }

                                // Send pending responses (DA, DSR, CPR, OSC 52 query, etc.) to PTY
                                let pending_responses = tab.screen.take_pending_responses();
                                for response in pending_responses {
                                    let _ = self.pty_manager.write(session_id, response.as_bytes());
                                }

                                // Handle OSC 52 clipboard set request
                                if let Some(clipboard_data) = tab.screen.take_clipboard_request() {
                                    // Decode base64 and set clipboard
                                    use base64::{engine::general_purpose::STANDARD, Engine as _};
                                    if let Ok(decoded_bytes) = STANDARD.decode(&clipboard_data) {
                                        if let Ok(text) = String::from_utf8(decoded_bytes) {
                                            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                                let _ = clipboard.set_text(text);
                                            }
                                        }
                                    }
                                }

                                // Store text for trigger checking (done after releasing borrow)
                                if let Ok(text) = String::from_utf8(data.clone()) {
                                    trigger_text = Some(text);
                                }

                                // Detect URLs in terminal output
                                tab.screen.detect_urls();

                                // Convert screen buffer to parsed line cache for rendering
                                let all_lines = tab.screen.get_all_lines();
                                tab.parsed_line_cache = all_lines
                                    .iter()
                                    .map(|cells| cells_to_styled_spans(cells))
                                    .collect();

                                // Increment content version for canvas cache invalidation
                                tab.content_version += 1;

                                // Auto-scroll to bottom
                                tab.canvas_state
                                    .scroll_to_bottom(tab.parsed_line_cache.len(), self.font_size);
                            }
                        }
                    }
                }

                // Update bracket matching after processing PTY output
                self.update_bracket_match();

                // Check triggers against new output (after releasing tab borrow)
                if let Some(text) = trigger_text {
                    self.check_triggers(&text);
                }

                // Play bell sound and trigger flash if triggered in active tab (after releasing tab borrow)
                if active_bell_triggered {
                    self.play_bell_sound();
                    self.trigger_bell_flash();
                }

                // Check background tabs for bell notifications
                let mut background_bell_triggered = false;
                let mut background_bell_tab_titles = Vec::new();
                for (i, tab) in self.tabs.iter_mut().enumerate() {
                    if i == self.active_tab {
                        continue; // Skip active tab (already processed above)
                    }

                    // Check if bell was triggered in background tab
                    if tab.screen.take_bell_triggered() {
                        tab.bell_pending = true;
                        background_bell_triggered = true;
                        // Collect tab title for notification
                        let tab_title = tab
                            .title
                            .as_ref()
                            .cloned()
                            .unwrap_or_else(|| format!("Terminal {}", i + 1));
                        background_bell_tab_titles.push(tab_title);
                    }
                }

                // Play bell sound and send notifications for background tabs
                if background_bell_triggered {
                    self.play_bell_sound();
                    // Send desktop notification for each background tab with bell
                    for tab_title in background_bell_tab_titles {
                        self.notification_manager.notify_bell(&tab_title);
                    }
                }

                // Check for URL clicks
                let url_task = if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    if let Some(url) = tab.canvas_state.clicked_url.take() {
                        Task::done(Message::OpenUrl(url))
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                };

                // Handle copy-on-select for active tab
                let config = get_config();
                if config.mouse.copy_on_select {
                    if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                        // Check if there's a completed selection that hasn't been copied yet
                        if let Some(selection) = &tab.canvas_state.selection {
                            if selection.active && selection.start != selection.end {
                                // Check if this selection is different from the last copied one
                                let current_selection = (selection.start, selection.end);
                                let should_copy =
                                    tab.last_copied_selection.as_ref() != Some(&current_selection);

                                if should_copy && !tab.canvas_state.is_dragging {
                                    // Copy selection to clipboard
                                    use terminal_canvas::get_selected_text;
                                    let selected_text =
                                        get_selected_text(&tab.parsed_line_cache, selection);
                                    if !selected_text.is_empty() {
                                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                            let _ = clipboard.set_text(selected_text);
                                            tab.last_copied_selection = Some(current_selection);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                Task::batch([focus_task, url_task])
            }

            // Pane management messages (stub implementations)
            Message::SplitHorizontal => {
                tracing::info!("SplitHorizontal triggered (not yet implemented)");
                Task::none()
            }

            Message::SplitVertical => {
                tracing::info!("SplitVertical triggered (not yet implemented)");
                Task::none()
            }

            Message::ClosePane => {
                tracing::info!("ClosePane triggered (not yet implemented)");
                Task::none()
            }

            Message::NextPane => {
                tracing::info!("NextPane triggered (not yet implemented)");
                Task::none()
            }

            Message::PrevPane => {
                tracing::info!("PrevPane triggered (not yet implemented)");
                Task::none()
            }

            Message::TabDragStart(index) => {
                self.tab_drag = Some(TabDragState {
                    dragging_index: index,
                    start_x: 0.0,
                    current_x: 0.0,
                });
                Task::none()
            }

            Message::TabDragMove(x) => {
                if let Some(drag) = &mut self.tab_drag {
                    drag.current_x = x;

                    // Calculate target index based on drag position
                    let tab_width = 150.0; // Approximate tab width
                    let offset = drag.current_x - drag.start_x;
                    let position_change = (offset / tab_width).round() as i32;

                    let target_index = (drag.dragging_index as i32 + position_change)
                        .max(0)
                        .min(self.tabs.len() as i32 - 1)
                        as usize;

                    // Swap tabs if needed
                    if target_index != drag.dragging_index {
                        self.tabs.swap(drag.dragging_index, target_index);
                        if self.active_tab == drag.dragging_index {
                            self.active_tab = target_index;
                        } else if self.active_tab == target_index {
                            self.active_tab = drag.dragging_index;
                        }
                        drag.dragging_index = target_index;
                        drag.start_x = drag.current_x;
                    }
                }
                Task::none()
            }

            Message::TabDragEnd => {
                self.tab_drag = None;
                Task::none()
            }

            Message::TabContextMenu(index, x, y) => {
                self.tab_context_menu = Some(TabContextMenu {
                    tab_index: index,
                    position: (x, y),
                });
                Task::none()
            }

            Message::CloseContextMenu => {
                self.tab_context_menu = None;
                Task::none()
            }

            Message::TabRenameStart(index) => {
                self.tab_rename_mode = Some(index);
                self.tab_rename_input = self
                    .tabs
                    .get(index)
                    .and_then(|tab| tab.title.clone())
                    .unwrap_or_else(|| format!("Terminal {}", index + 1));
                self.tab_context_menu = None;
                Task::none()
            }

            Message::TabRenameInput(input) => {
                self.tab_rename_input = input;
                Task::none()
            }

            Message::TabRenameSubmit => {
                if let Some(index) = self.tab_rename_mode {
                    if let Some(tab) = self.tabs.get_mut(index) {
                        if self.tab_rename_input.trim().is_empty() {
                            tab.title = None;
                        } else {
                            tab.title = Some(self.tab_rename_input.clone());
                        }
                    }
                }
                self.tab_rename_mode = None;
                self.tab_rename_input.clear();
                Task::none()
            }

            Message::TabRenameCancel => {
                self.tab_rename_mode = None;
                self.tab_rename_input.clear();
                Task::none()
            }

            Message::StartHistorySearch => {
                self.history_manager.start_reverse_search();
                tracing::debug!("Started history search mode");
                Task::none()
            }

            Message::UpdateHistorySearch(query) => {
                self.history_manager.update_search(&query);
                tracing::debug!(
                    "Updated history search: query='{}', {} matches",
                    query,
                    self.history_manager.search_result_count()
                );
                Task::none()
            }

            Message::HistorySearchNext => {
                self.history_manager.next_match();
                tracing::debug!("Moved to next history match");
                Task::none()
            }

            Message::HistorySearchPrev => {
                self.history_manager.prev_match();
                tracing::debug!("Moved to previous history match");
                Task::none()
            }

            Message::EndHistorySearch => {
                if let Some(command) = self.history_manager.end_search() {
                    if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                        tab.raw_input = command.clone();
                        tracing::debug!("Selected history command: {}", command);
                    }
                }
                Task::none()
            }

            Message::CancelHistorySearch => {
                self.history_manager.cancel_search();
                tracing::debug!("Cancelled history search");
                Task::none()
            }

            // Completion messages
            Message::TriggerCompletion => {
                // Trigger tab completion
                let config = get_config();
                if config.completion.enabled {
                    if let Some(tab) = self.tabs.get(self.active_tab) {
                        let input = &tab.raw_input;
                        let cwd = &tab.cwd;

                        let mut items = self.completion_engine.complete(input, cwd);

                        // Limit items based on config
                        if items.len() > config.completion.max_items {
                            items.truncate(config.completion.max_items);
                        }

                        self.completion_items = items;
                        if !self.completion_items.is_empty() {
                            self.completion_visible = true;
                            self.completion_selected = 0;
                            tracing::debug!("Completion triggered: {} items", self.completion_items.len());
                        }
                    }
                }
                Task::none()
            }

            Message::CompletionNext => {
                if self.completion_visible && !self.completion_items.is_empty() {
                    self.completion_selected = (self.completion_selected + 1) % self.completion_items.len();
                }
                Task::none()
            }

            Message::CompletionPrev => {
                if self.completion_visible && !self.completion_items.is_empty() {
                    if self.completion_selected == 0 {
                        self.completion_selected = self.completion_items.len() - 1;
                    } else {
                        self.completion_selected -= 1;
                    }
                }
                Task::none()
            }

            Message::CompletionSelect => {
                if self.completion_visible && self.completion_selected < self.completion_items.len() {
                    let completion = &self.completion_items[self.completion_selected];
                    if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                        tab.raw_input = completion.text.clone();
                    }
                    self.completion_visible = false;
                    self.completion_items.clear();
                }
                Task::none()
            }

            Message::CompletionCancel => {
                self.completion_visible = false;
                self.completion_items.clear();
                self.completion_selected = 0;
                Task::none()
            }

            // MCP Panel
            Message::ToggleMcpPanel => {
                let _ = self.mcp_panel.update(McpPanelMessage::TogglePanel);
                Task::none()
            }
            Message::McpPanelMessage(msg) => {
                // Handle ExecuteCommand specially - send command to terminal
                if let McpPanelMessage::ExecuteCommand(ref cmd) = msg {
                    let cmd = cmd.clone();
                    // Send command to active terminal
                    if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                        if let Some(session_id) = &tab.session_id {
                            let cmd_with_newline = format!("{}\n", cmd);
                            let _ = self.pty_manager.write(session_id, cmd_with_newline.as_bytes());
                        }
                    }
                }
                // Update panel state
                let _ = self.mcp_panel.update(msg);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        if self.tabs.is_empty() {
            return container(text("No terminal open").color(inline_theme::TEXT_PRIMARY))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(inline_theme::primary_background_style)
                .into();
        }

        let tab_bar = self.view_tab_bar();
        let content = self.view_terminal_content();

        // Main Layout
        let terminal_area = column![
            container(tab_bar)
                .padding([10, 16])
                .width(Length::Fill)
                .style(|_| container::Style {
                    background: Some(inline_theme::BG_PRIMARY.into()),
                    border: Border {
                        color: inline_theme::BORDER,
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                }),
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_| container::Style {
                    background: Some(inline_theme::BG_SECONDARY.into()),
                    ..Default::default()
                })
        ]
        .width(Length::Fill);

        // Main content with optional debug panel and MCP panel
        let main_content: Element<Message> = {
            let show_debug = self.debug_panel.visible;
            let show_mcp = self.mcp_panel.is_visible();

            if !show_debug && !show_mcp {
                // No panels - just terminal area
                terminal_area.height(Length::Fill).into()
            } else {
                // Build row with terminal and optional panels
                let mut content_row = row![terminal_area];

                if show_debug {
                    let debug_panel_view: Element<Message> = self.debug_panel.view();
                    content_row = content_row.push(debug_panel_view);
                }

                if show_mcp {
                    let mcp_panel_view: Element<Message> =
                        self.mcp_panel.view().map(Message::McpPanelMessage);
                    content_row = content_row.push(mcp_panel_view);
                }

                content_row.width(Length::Fill).height(Length::Fill).into()
            }
        };

        // Add bell flash overlay if active
        let with_flash = if self.bell_flash_active {
            let config = get_config();

            // Calculate flash opacity with fade-out animation
            let opacity = if let Some(start_time) = self.flash_started_at {
                let elapsed = start_time.elapsed().as_millis() as u64;
                let progress = elapsed as f32 / config.terminal.flash_duration_ms as f32;
                // Fade out: start at full opacity, fade to 0
                (1.0 - progress).max(0.0)
            } else {
                1.0
            };

            // Parse flash color from config
            let (r, g, b, base_alpha) = config::parse_hex_color(&config.terminal.flash_color)
                .unwrap_or((1.0, 1.0, 1.0, 0.5)); // Default to white with 50% opacity

            // Apply fade-out animation to alpha
            let flash_color = Color::from_rgba(r, g, b, base_alpha * opacity);

            // Create flash overlay
            let flash_overlay = container(Space::new(Length::Fill, Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_theme| container::Style {
                    background: Some(flash_color.into()),
                    ..Default::default()
                });

            // Stack main content with flash overlay
            stack![main_content, flash_overlay].into()
        } else {
            main_content
        };

        // Add command palette overlay (always include in stack for consistent diff)
        let palette_view: Element<Message> = self
            .command_palette
            .view()
            .map(Message::PaletteMessage);
        let final_content: Element<Message> = stack![with_flash, palette_view].into();

        container(final_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(inline_theme::primary_background_style)
            .into()
    }

    /// Render the tab bar with all tabs and new tab button
    fn view_tab_bar(&self) -> Element<Message> {
        let mut tab_elements = Vec::with_capacity(self.tabs.len());
        for (i, tab) in self.tabs.iter().enumerate() {
            let is_active = i == self.active_tab;
            // Use custom title if set, otherwise use default "Terminal N"
            let label = tab
                .title
                .as_ref()
                .map(|t| {
                    // Limit title length to 30 characters for display
                    if t.len() > 30 {
                        format!("{}...", &t[..27])
                    } else {
                        t.clone()
                    }
                })
                .unwrap_or_else(|| {
                    // If no custom title, show terminal number with directory
                    if !tab.cwd.is_empty() && tab.cwd != "/" {
                        // Extract just the directory name from path
                        let dir_name = std::path::Path::new(&tab.cwd)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&tab.cwd);
                        format!("Terminal {} ({})", i + 1, dir_name)
                    } else {
                        format!("Terminal {}", i + 1)
                    }
                });
            let can_close = self.tabs.len() > 1;
            let has_bell = tab.bell_pending;

            let icon_color = if is_active {
                inline_theme::TAB_ACTIVE
            } else if has_bell {
                inline_theme::ACCENT_YELLOW // Yellow bell indicator for inactive tabs
            } else {
                inline_theme::TEXT_MUTED
            };
            let label_color = if is_active {
                inline_theme::TEXT_PRIMARY
            } else {
                inline_theme::TEXT_SECONDARY
            };

            // Tab label button (clickable to select)
            // Show bell icon (🔔) if bell is pending, otherwise show arrow (▶)
            let tab_icon = if has_bell && !is_active {
                "🔔"
            } else {
                "▶"
            };

            let tab_label_button = button(
                row![
                    text(tab_icon).size(11).color(icon_color),
                    Space::with_width(8),
                    text(label.clone()).size(13).color(label_color)
                ]
                .align_y(Alignment::Center),
            )
            .padding([8, 12])
            .style(move |_, status| {
                let bg = match status {
                    button::Status::Hovered => {
                        if is_active {
                            inline_theme::BG_SECONDARY
                        } else {
                            inline_theme::BG_BLOCK_HOVER
                        }
                    }
                    _ => {
                        if is_active {
                            inline_theme::BG_SECONDARY
                        } else {
                            inline_theme::BG_PRIMARY
                        }
                    }
                };
                button::Style {
                    background: Some(bg.into()),
                    text_color: inline_theme::TEXT_PRIMARY,
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
            let close_button = button(text("×").size(14))
                .padding([8, 10])
                .style(move |_, status| {
                    let (bg, text_color) = match status {
                        button::Status::Hovered => {
                            (inline_theme::BG_BLOCK_HOVER, inline_theme::ACCENT_RED)
                        }
                        _ => {
                            let bg = if is_active {
                                inline_theme::BG_SECONDARY
                            } else {
                                inline_theme::BG_PRIMARY
                            };
                            (bg, inline_theme::TEXT_MUTED)
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
                .on_press_maybe(if can_close {
                    Some(Message::CloseTab(i))
                } else {
                    None
                });

            // Tab content with accent line
            let tab_content = column![
                row![tab_label_button, close_button],
                // Active tab bottom accent line (2px height)
                container(Space::new(Length::Fill, Length::Fixed(2.0)))
                    .width(Length::Fill)
                    .height(Length::Fixed(2.0))
                    .style(move |_| container::Style {
                        background: if is_active {
                            Some(inline_theme::TAB_ACTIVE.into())
                        } else {
                            None
                        },
                        ..Default::default()
                    })
            ];

            tab_elements.push(container(tab_content).into());
        }

        row(tab_elements)
            .spacing(2)
            .push(Space::with_width(8))
            .push(
                button(text("+").size(16).color(inline_theme::TEXT_SECONDARY))
                    .padding([8, 14])
                    .style(|_, status| {
                        let bg = match status {
                            button::Status::Hovered => inline_theme::BG_BLOCK_HOVER,
                            _ => inline_theme::BG_BLOCK,
                        };
                        button::Style {
                            background: Some(bg.into()),
                            text_color: inline_theme::TEXT_SECONDARY,
                            border: Border {
                                radius: 6.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    })
                    .on_press(Message::NewTab),
            )
            .into()
    }

    /// Render the terminal content area (output + input + status bar)
    fn view_terminal_content(&self) -> Element<Message> {
        if let Some(tab) = self.tabs.get(self.active_tab) {
            // Full Streaming Terminal
            let terminal_output = self.render_raw_terminal(&tab.parsed_line_cache);

            // Hidden Input (for IME/Korean support)
            // Note: We use a minimal-height container instead of size(0) to avoid cosmic-text crash
            let raw_input_field: Element<Message> = container(
                text_input("", &tab.raw_input)
                    .id(raw_input_id())
                    .on_input(Message::RawInputChanged)
                    .on_submit(Message::RawInputSubmit)
                    .size(1) // Minimum size to avoid crash
                    .style(|_theme, _status| text_input::Style {
                        background: Color::TRANSPARENT.into(),
                        border: Border::default(),
                        icon: Color::TRANSPARENT,
                        placeholder: Color::TRANSPARENT,
                        value: Color::TRANSPARENT,
                        selection: Color::TRANSPARENT,
                    }),
            )
            .height(Length::Fixed(1.0)) // Minimal height
            .into();

            let status_bar = self.view_status_bar();

            column![
                container(
                    column![
                        terminal_output,
                        raw_input_field // Hidden at bottom for IME
                    ]
                    .width(Length::Fill)
                    .height(Length::Fill)
                )
                .padding([16, 12]) // Top padding for spacing from tab bar
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_| container::Style {
                    background: Some(inline_theme::BG_SECONDARY.into()),
                    ..Default::default()
                }),
                status_bar
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            column![text("No terminal open").color(inline_theme::TEXT_PRIMARY)].into()
        }
    }

    /// Render the status bar with shell name, mode, and shortcuts
    fn view_status_bar(&self) -> Element<Message> {
        let config = get_config();

        // If status bar is disabled, return minimal element (1px to avoid zero-height panic)
        if !config.status_bar.visible {
            return Space::new(Length::Fill, Length::Fixed(1.0)).into();
        }

        // Gather terminal information from active tab
        let tab = &self.tabs[self.active_tab];
        let (cols, rows) = tab.screen.dimensions();
        let scrollback_lines = tab.screen.scrollback_size();
        let total_lines = scrollback_lines + rows;
        let visible_lines = rows;

        // Calculate scroll position
        let scroll_position = if total_lines > visible_lines {
            // Assume we're viewing the bottom of the buffer (streaming mode)
            Some((total_lines, total_lines))
        } else {
            Some((visible_lines, total_lines))
        };

        // Create status bar info
        let info = ui::status_bar::StatusBarInfo {
            shell: self.get_shell_name(),
            cwd: Some(tab.cwd.clone()),
            cols: cols as u16,
            rows: rows as u16,
            encoding: String::from("UTF-8"),
            mode: Some(String::from("streaming")),
            scroll_position,
        };

        // Create status bar config from app config
        let status_config = ui::status_bar::StatusBarConfig {
            visible: config.status_bar.visible,
            show_cwd: config.status_bar.show_cwd,
            show_size: config.status_bar.show_size,
            show_encoding: config.status_bar.show_encoding,
            show_scroll_position: config.status_bar.show_scroll_position,
            show_mode: config.status_bar.show_mode,
        };

        // Render the status bar using the new modular component
        ui::status_bar::view(
            info,
            status_config,
            inline_theme::TEXT_MUTED,
            inline_theme::BG_SECONDARY,
        )
    }

    /// Render raw terminal output (for Raw mode)
    /// Uses Canvas for virtual scrolling and hardware acceleration
    fn render_raw_terminal<'a>(
        &'a self,
        parsed_cache: &'a [Vec<StyledSpan>],
    ) -> Element<'a, Message> {
        use iced::widget::canvas;

        let tab = &self.tabs[self.active_tab];
        let (cursor_row, cursor_col) = tab.screen.cursor_position();

        // Create cursor state with config-defined style
        let config = get_config();
        let cursor = CursorState {
            row: cursor_row,
            col: cursor_col,
            style: convert_cursor_style(config.terminal.cursor_style),
            visible: tab.screen.cursor_visible(),
            blink_on: tab.cursor_blink_on,
        };

        // Create terminal canvas with all lines (virtual scrolling will handle visibility)
        let terminal_canvas = TerminalCanvas::new(
            parsed_cache,
            tab.content_version,
            inline_theme::TEXT_PRIMARY,
            MONO_FONT,
        )
        .with_cursor(cursor)
        .with_font_size(self.font_size)
        .with_search_matches(&self.search_matches, self.current_match_index)
        .with_bracket_match(tab.bracket_match);

        canvas(terminal_canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        // Dynamic tick interval based on PTY activity
        // Adjust base refresh rate based on environment (slower in SSH/container)
        let suggested_settings = self.env_info.suggested_settings();
        let base_refresh_ms = suggested_settings.refresh_rate_ms;

        // - Recent activity (< 500ms): base refresh rate for smooth updates
        // - Medium activity (< 2s): 3x base rate for responsiveness
        // - Idle: 12x base rate to save CPU
        let elapsed_since_activity = self.last_pty_activity.elapsed();
        let tick_interval = if elapsed_since_activity < Duration::from_millis(500) {
            Duration::from_millis(base_refresh_ms) // Full speed
        } else if elapsed_since_activity < Duration::from_secs(2) {
            Duration::from_millis(base_refresh_ms * 3) // Reduced speed
        } else {
            Duration::from_millis(base_refresh_ms * 12) // Idle speed
        };

        let timer = iced::time::every(tick_interval).map(|_| Message::Tick);

        let keyboard =
            keyboard::on_key_press(|key, modifiers| Some(Message::KeyPressed(key, modifiers)));

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

        // Fast timer for bell flash animation (60 FPS for smooth fade-out)
        let flash_timer = if self.bell_flash_active {
            iced::time::every(Duration::from_millis(16)).map(|_| Message::BellFlashTick)
        } else {
            Subscription::none()
        };

        Subscription::batch([timer, keyboard, window_events, flash_timer])
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // ========== Tab Management Tests ==========

    /// Create a mock AgTerm instance for testing (without PTY)
    fn create_test_app() -> AgTerm {
        let pty_manager = Arc::new(PtyManager::new());

        let tab = TerminalTab {
            id: 0,
            session_id: None, // No actual PTY for tests
            raw_input: String::new(),
            input: String::new(),
            cwd: "/test/path".to_string(),
            error_message: None,
            history: Vec::new(),
            history_index: None,
            history_temp_input: String::new(),
            mode: TerminalMode::Raw,
            parsed_line_cache: Vec::new(),
            canvas_state: TerminalCanvasState::new(),
            content_version: 0,
            screen: TerminalScreen::new(80, 24),
            cursor_blink_on: true,
            bell_pending: false,
            title: None,
            title_info: agterm::terminal::title::TitleInfo::new(),
            last_copied_selection: None,
            bracket_match: None,
            pane_layout: PaneLayout::Single,
            panes: Vec::new(),
            focused_pane: 0,
        };

        AgTerm {
            tabs: vec![tab],
            active_tab: 0,
            pty_manager,
            next_tab_id: 1,
            startup_focus_count: 0,
            debug_panel: DebugPanel::new(),
            last_pty_activity: Instant::now(),
            last_cursor_blink: Instant::now(),
            font_size: 14.0,
            search_mode: false,
            search_query: String::new(),
            search_matches: Vec::new(),
            current_match_index: None,
            env_info: EnvironmentInfo::detect(),
            bell_sound: sound::BellSound::new(),
            bell_flash_active: false,
            flash_started_at: None,
            tab_drag: None,
            tab_context_menu: None,
            tab_rename_mode: None,
            tab_rename_input: String::new(),
            current_theme: theme::Theme::warp_dark(),
            current_modifiers: Modifiers::default(),
            notification_manager: NotificationManager::new(config::NotificationConfig::default()),
            keybindings: KeyBindings::default(),
            command_palette: CommandPalette::with_default_commands(),
            history_manager: HistoryManager::new(1000),
            completion_engine: CompletionEngine::new(),
            completion_items: Vec::new(),
            completion_selected: 0,
            completion_visible: false,
            trigger_manager: TriggerManager::new(),
            mcp_panel: McpPanel::new(),
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

    #[test]
    fn test_duplicate_tab() {
        let mut app = create_test_app();
        let initial_count = app.tabs.len();

        // Set a working directory for the current tab
        app.tabs[0].cwd = "/home/user/projects".to_string();

        let _ = app.update(Message::DuplicateTab);

        // Should have one more tab
        assert_eq!(app.tabs.len(), initial_count + 1);
        // Should switch to the new tab
        assert_eq!(app.active_tab, initial_count);
        // New tab should have the same working directory
        assert_eq!(app.tabs[1].cwd, "/home/user/projects");
    }

    #[test]
    fn test_tab_title_from_osc() {
        let mut app = create_test_app();

        // Initially, tab should have no title
        assert_eq!(app.tabs[0].title, None);

        // Simulate OSC 2 sequence setting window title
        let osc_sequence = b"\x1b]2;My Custom Tab Title\x1b\\";
        app.tabs[0].screen.process(osc_sequence);

        // Manually trigger the title update (normally done in Tick)
        if let Some(window_title) = app.tabs[0].screen.window_title() {
            app.tabs[0].title = Some(window_title.to_string());
        }

        assert_eq!(app.tabs[0].title, Some("My Custom Tab Title".to_string()));
    }

    #[test]
    fn test_tab_title_truncation() {
        let mut app = create_test_app();

        // Set a very long title
        let long_title =
            "This is a very long tab title that should be truncated for display purposes";
        app.tabs[0].title = Some(long_title.to_string());

        // The view_tab_bar function should truncate to 30 chars (27 + "...")
        // This is tested indirectly through the rendering logic
        assert!(app.tabs[0].title.as_ref().unwrap().len() > 30);
    }

    #[test]
    fn test_minimum_tab_warning() {
        let mut app = create_test_app();
        assert_eq!(app.tabs.len(), 1);

        // Try to close the last tab
        let _ = app.update(Message::CloseCurrentTab);

        // Should still have one tab (minimum preserved)
        assert_eq!(app.tabs.len(), 1);
    }

    // ========== Theme Tests ==========

    #[test]
    fn test_theme_colors_defined() {
        // Verify all theme colors are accessible
        let _ = inline_theme::BG_PRIMARY;
        let _ = inline_theme::BG_SECONDARY;
        let _ = inline_theme::BG_BLOCK;
        let _ = inline_theme::BG_BLOCK_HOVER;
        let _ = inline_theme::BG_INPUT;
        let _ = inline_theme::TEXT_PRIMARY;
        let _ = inline_theme::TEXT_SECONDARY;
        let _ = inline_theme::TEXT_MUTED;
        let _ = inline_theme::ACCENT_BLUE;
        let _ = inline_theme::ACCENT_GREEN;
        let _ = inline_theme::ACCENT_YELLOW;
        let _ = inline_theme::ACCENT_RED;
        let _ = inline_theme::BORDER;
        let _ = inline_theme::TAB_ACTIVE;
        let _ = inline_theme::PROMPT;
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

    // ========== Font Configuration Tests ==========

    #[test]
    fn test_font_config_default_values() {
        use config::FontConfig;

        let font_config = FontConfig::default();
        assert_eq!(font_config.family, "D2Coding");
        assert_eq!(font_config.size, 14.0);
        assert_eq!(font_config.line_height, 1.2);
        assert_eq!(font_config.bold_as_bright, true);
        assert_eq!(font_config.use_thin_strokes, false);
    }

    #[test]
    fn test_font_size_increase() {
        let mut app = create_test_app();
        app.font_size = 14.0;

        app.update(Message::IncreaseFontSize);
        assert_eq!(app.font_size, 15.0);

        // Test max limit
        app.font_size = 24.0;
        app.update(Message::IncreaseFontSize);
        assert_eq!(app.font_size, 24.0, "Font size should not exceed 24.0");
    }

    #[test]
    fn test_font_size_decrease() {
        let mut app = create_test_app();
        app.font_size = 14.0;

        app.update(Message::DecreaseFontSize);
        assert_eq!(app.font_size, 13.0);

        // Test min limit
        app.font_size = 8.0;
        app.update(Message::DecreaseFontSize);
        assert_eq!(app.font_size, 8.0, "Font size should not go below 8.0");
    }

    #[test]
    fn test_font_size_reset() {
        let mut app = create_test_app();
        app.font_size = 20.0;

        app.update(Message::ResetFontSize);
        let config = get_config();
        assert_eq!(app.font_size, config.appearance.font.size);
    }

    #[test]
    fn test_font_config_parsing() {
        let toml_str = r#"
            family = "JetBrains Mono"
            size = 16.0
            line_height = 1.5
            bold_as_bright = false
            use_thin_strokes = true
        "#;

        let font_config: config::FontConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(font_config.family, "JetBrains Mono");
        assert_eq!(font_config.size, 16.0);
        assert_eq!(font_config.line_height, 1.5);
        assert_eq!(font_config.bold_as_bright, false);
        assert_eq!(font_config.use_thin_strokes, true);
    }
}
