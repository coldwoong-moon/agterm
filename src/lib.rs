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
//! - Shell integration for bash, zsh, and fish with OSC support
//! - Accessibility features with WCAG compliance and screen reader support
//! - Plugin API for extensibility with permission-based security
//! - Internationalization (i18n) with multi-language support
//! - MCP (Model Context Protocol) integration for AI agent orchestration
//! - Command validator for AI-generated command risk analysis and safety

// Core modules (framework-independent)
pub mod aliases;
pub mod annotations;
pub mod automation;
pub mod bookmarks;
pub mod broadcast;
pub mod clipboard_history;
pub mod color;
pub mod command_validator;
pub mod completion;
pub mod config;
pub mod encoding;
pub mod env_manager;
pub mod filters;
pub mod highlighting;
pub mod history;
pub mod i18n;
pub mod image_protocol;
#[cfg(feature = "iced-gui")]
pub mod keybind;
pub mod link_handler;
pub mod logging;
pub mod macros;
pub mod mcp;
pub mod mouse_actions;
pub mod notification;
pub mod performance_monitor;
pub mod pipeline;
pub mod plugin_api;
#[cfg(any(feature = "iced-gui", feature = "floem-gui"))]
pub mod profiles;
pub mod quick_actions;
pub mod recording;
pub mod session;
pub mod session_tags;
pub mod shell;
pub mod shell_integration;
pub mod snippets;
pub mod sound;
pub mod splits;
pub mod ssh;
pub mod statistics;
pub mod tab_manager;
pub mod terminal;
pub mod timer;
pub mod trigger;
pub mod workspace;

// Iced-specific modules (require iced-gui feature)
#[cfg(feature = "iced-gui")]
pub mod accessibility;
#[cfg(feature = "iced-gui")]
pub mod debug;
#[cfg(feature = "iced-gui")]
pub mod diff_view;
#[cfg(feature = "iced-gui")]
pub mod markdown;
#[cfg(feature = "iced-gui")]
pub mod render_cache;
#[cfg(feature = "iced-gui")]
pub mod theme;
#[cfg(feature = "iced-gui")]
pub mod theme_editor;
#[cfg(feature = "iced-gui")]
pub mod ui;

// Floem-specific modules (require floem-gui feature)
#[cfg(feature = "floem-gui")]
pub mod floem_app;
