//! AgTerm Library
//!
//! This library provides terminal emulation functionality including:
//! - PTY (Pseudo-Terminal) management
//! - ANSI escape sequence parsing and rendering
//! - Screen buffer management
//! - Theme system with popular presets
//! - Configuration management
//! - Character encoding support
//! - Input macro system for automation
//! - Clipboard history with pin functionality
//! - Code snippet system with template expansion
//! - Output filter system for real-time processing
//! - Terminal session recording and playback
//! - Terminal broadcast for multi-session input
//! - Terminal annotation system for marking and bookmarking lines
//! - Comprehensive link detection and handling
//! - Terminal image protocol support (iTerm2, Kitty, SIXEL)
//! - Terminal automation API with scripting DSL
//! - Diff viewer with Myers algorithm for text comparison
//! - Command completion engine with file and command suggestions
//! - Debug panel with performance metrics and event logging
//! - Command history management with persistence
//! - Structured logging with tracing ecosystem
//! - Terminal bell sound system
//! - Environment variable manager with categorization and security

//! - Session tagging and organization system
//! - Bookmark system for frequently used commands
//! - Command alias system with shell integration
//! - Statistics dashboard for usage analytics and productivity tracking

pub mod aliases;
pub mod annotations;
pub mod automation;
pub mod bookmarks;
pub mod broadcast;
pub mod clipboard_history;
pub mod completion;
pub mod config;
pub mod debug;
pub mod diff_view;
pub mod encoding;
pub mod env_manager;

pub mod filters;
pub mod highlighting;
pub mod history;
pub mod image_protocol;
pub mod keybind;
pub mod link_handler;
pub mod logging;
pub mod macros;
pub mod markdown;
pub mod notification;
pub mod pipeline;
pub mod profiles;
pub mod recording;
pub mod render_cache;
pub mod session;
pub mod session_tags;
pub mod shell;
pub mod snippets;
pub mod sound;
pub mod splits;
pub mod ssh;
pub mod statistics;
pub mod terminal;
pub mod timer;

pub mod theme;
pub mod theme_editor;
pub mod trigger;
pub mod ui;
pub mod workspace;
