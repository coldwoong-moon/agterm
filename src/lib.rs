//! AgTerm Library
//!
//! This library provides terminal emulation functionality including:
//! - PTY (Pseudo-Terminal) management
//! - ANSI escape sequence parsing and rendering
//! - Screen buffer management
//! - Theme system with popular presets
//! - Configuration management
//! - Character encoding support

pub mod config;
pub mod encoding;
pub mod notification;
pub mod session;
pub mod shell;
pub mod ssh;
pub mod terminal;
pub mod theme;
pub mod trigger;
pub mod ui;
