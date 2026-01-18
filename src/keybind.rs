//! Key binding system for AgTerm
//!
//! This module provides a flexible key mapping system that allows users to:
//! - Define custom key bindings
//! - Map key combinations to actions
//! - Load bindings from configuration files
//! - Support multiple modifier keys (Ctrl, Shift, Alt, Cmd)

use std::collections::HashMap;
use iced::keyboard::{Key, Modifiers};
use crate::config::{KeyBinding as ConfigKeyBinding, KeyModifiers as ConfigKeyModifiers};

/// A key combination consisting of a key and modifier keys
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeyCombo {
    /// The key name (normalized to lowercase for consistency)
    pub key: String,
    /// Modifier keys pressed
    pub modifiers: KeyModifiers,
}

/// Modifier keys for a key binding
#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub super_: bool, // Cmd on macOS, Win on Windows
}

/// Actions that can be triggered by key bindings
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    // Tab management
    NewTab,
    CloseTab,
    NextTab,
    PrevTab,
    SelectTab(usize),
    DuplicateTab,

    // Splitting
    SplitHorizontal,
    SplitVertical,

    // Clipboard
    CopySelection,
    Paste,
    ForceCopy,
    ForcePaste,

    // Font size
    IncreaseFontSize,
    DecreaseFontSize,
    ResetFontSize,

    // Scrolling
    ScrollUp(usize),
    ScrollDown(usize),
    ScrollPageUp,
    ScrollPageDown,
    ScrollToTop,
    ScrollToBottom,

    // Terminal
    ClearScreen,
    ClearScrollback,

    // Debug/UI
    ToggleDebugPanel,
    OpenCommandPalette,

    // Search
    ReverseSearch,

    // Custom action (for extensibility)
    Custom(String),
}

impl Action {
    /// Parse an action string into an Action enum
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "new_tab" => Some(Action::NewTab),
            "close_tab" => Some(Action::CloseTab),
            "next_tab" => Some(Action::NextTab),
            "prev_tab" => Some(Action::PrevTab),
            "duplicate_tab" => Some(Action::DuplicateTab),

            "split_horizontal" => Some(Action::SplitHorizontal),
            "split_vertical" => Some(Action::SplitVertical),

            "copy" => Some(Action::CopySelection),
            "paste" => Some(Action::Paste),
            "force_copy" => Some(Action::ForceCopy),
            "force_paste" => Some(Action::ForcePaste),

            "increase_font_size" => Some(Action::IncreaseFontSize),
            "decrease_font_size" => Some(Action::DecreaseFontSize),
            "reset_font_size" => Some(Action::ResetFontSize),

            "scroll_up" => Some(Action::ScrollUp(1)),
            "scroll_down" => Some(Action::ScrollDown(1)),
            "scroll_page_up" => Some(Action::ScrollPageUp),
            "scroll_page_down" => Some(Action::ScrollPageDown),
            "scroll_to_top" => Some(Action::ScrollToTop),
            "scroll_to_bottom" => Some(Action::ScrollToBottom),

            "clear_screen" => Some(Action::ClearScreen),
            "clear_scrollback" => Some(Action::ClearScrollback),

            "toggle_debug_panel" => Some(Action::ToggleDebugPanel),
            "command_palette" => Some(Action::OpenCommandPalette),

            "reverse_search" => Some(Action::ReverseSearch),

            // Handle select_tab_N actions
            s if s.starts_with("select_tab_") => {
                s.strip_prefix("select_tab_")
                    .and_then(|n| n.parse::<usize>().ok())
                    .map(|n| Action::SelectTab(n.saturating_sub(1))) // Convert 1-based to 0-based
            }

            // Unknown action - store as custom
            _ => Some(Action::Custom(s.to_string())),
        }
    }

    /// Convert action to string representation
    pub fn to_string(&self) -> String {
        match self {
            Action::NewTab => "new_tab".to_string(),
            Action::CloseTab => "close_tab".to_string(),
            Action::NextTab => "next_tab".to_string(),
            Action::PrevTab => "prev_tab".to_string(),
            Action::SelectTab(n) => format!("select_tab_{}", n + 1),
            Action::DuplicateTab => "duplicate_tab".to_string(),
            Action::SplitHorizontal => "split_horizontal".to_string(),
            Action::SplitVertical => "split_vertical".to_string(),
            Action::CopySelection => "copy".to_string(),
            Action::Paste => "paste".to_string(),
            Action::ForceCopy => "force_copy".to_string(),
            Action::ForcePaste => "force_paste".to_string(),
            Action::IncreaseFontSize => "increase_font_size".to_string(),
            Action::DecreaseFontSize => "decrease_font_size".to_string(),
            Action::ResetFontSize => "reset_font_size".to_string(),
            Action::ScrollUp(n) => format!("scroll_up_{n}"),
            Action::ScrollDown(n) => format!("scroll_down_{n}"),
            Action::ScrollPageUp => "scroll_page_up".to_string(),
            Action::ScrollPageDown => "scroll_page_down".to_string(),
            Action::ScrollToTop => "scroll_to_top".to_string(),
            Action::ScrollToBottom => "scroll_to_bottom".to_string(),
            Action::ClearScreen => "clear_screen".to_string(),
            Action::ClearScrollback => "clear_scrollback".to_string(),
            Action::ToggleDebugPanel => "toggle_debug_panel".to_string(),
            Action::OpenCommandPalette => "command_palette".to_string(),
            Action::ReverseSearch => "reverse_search".to_string(),
            Action::Custom(s) => s.clone(),
        }
    }
}

/// Key bindings manager
pub struct KeyBindings {
    bindings: HashMap<KeyCombo, Action>,
}

impl KeyBindings {
    /// Create a new KeyBindings instance with default bindings
    pub fn default() -> Self {
        let mut kb = Self {
            bindings: HashMap::new(),
        };
        kb.load_defaults();
        kb
    }

    /// Load default key bindings
    fn load_defaults(&mut self) {
        // Tab management
        self.bind_str("t", KeyModifiers::cmd(), Action::NewTab);
        self.bind_str("w", KeyModifiers::cmd(), Action::CloseTab);
        self.bind_str("]", KeyModifiers::cmd(), Action::NextTab);
        self.bind_str("[", KeyModifiers::cmd(), Action::PrevTab);
        self.bind_str("d", KeyModifiers::cmd_shift(), Action::DuplicateTab);

        // Tab selection (Cmd+1 through Cmd+9)
        for i in 1..=9 {
            self.bind_str(&i.to_string(), KeyModifiers::cmd(), Action::SelectTab(i - 1));
        }

        // Splitting
        self.bind_str("h", KeyModifiers::cmd_shift(), Action::SplitHorizontal);
        self.bind_str("|", KeyModifiers::cmd_shift(), Action::SplitVertical);

        // Clipboard
        self.bind_str("c", KeyModifiers::cmd_shift(), Action::ForceCopy);
        self.bind_str("v", KeyModifiers::cmd(), Action::Paste);
        self.bind_str("v", KeyModifiers::cmd_shift(), Action::ForcePaste);

        // Font size
        self.bind_str("+", KeyModifiers::cmd(), Action::IncreaseFontSize);
        self.bind_str("=", KeyModifiers::cmd(), Action::IncreaseFontSize);
        self.bind_str("-", KeyModifiers::cmd(), Action::DecreaseFontSize);
        self.bind_str("0", KeyModifiers::cmd(), Action::ResetFontSize);

        // Scrolling
        self.bind_str("Home", KeyModifiers::cmd(), Action::ScrollToTop);
        self.bind_str("End", KeyModifiers::cmd(), Action::ScrollToBottom);
        self.bind_str("PageUp", KeyModifiers::none(), Action::ScrollPageUp);
        self.bind_str("PageDown", KeyModifiers::none(), Action::ScrollPageDown);

        // Terminal
        self.bind_str("k", KeyModifiers::cmd(), Action::ClearScreen);

        // Debug
        self.bind_str("d", KeyModifiers::cmd(), Action::ToggleDebugPanel);
        self.bind_str("F12", KeyModifiers::none(), Action::ToggleDebugPanel);
    }

    /// Create KeyBindings from configuration
    pub fn from_config(bindings: &[ConfigKeyBinding]) -> Self {
        let mut kb = Self::default();

        // Override defaults with config bindings
        for binding in bindings {
            let modifiers = KeyModifiers {
                ctrl: binding.modifiers.ctrl,
                alt: binding.modifiers.alt,
                shift: binding.modifiers.shift,
                super_: binding.modifiers.cmd,
            };

            if let Some(action) = Action::from_string(&binding.action) {
                kb.bind_str(&binding.key, modifiers, action);
            }
        }

        kb
    }

    /// Get action for a key combination
    pub fn get_action(&self, combo: &KeyCombo) -> Option<&Action> {
        self.bindings.get(combo)
    }

    /// Bind a key combination to an action
    pub fn bind(&mut self, combo: KeyCombo, action: Action) {
        self.bindings.insert(combo, action);
    }

    /// Helper to bind a string key with modifiers to an action
    fn bind_str(&mut self, key: &str, modifiers: KeyModifiers, action: Action) {
        let combo = KeyCombo {
            key: normalize_key_name(key),
            modifiers,
        };
        self.bind(combo, action);
    }

    /// Unbind a key combination
    pub fn unbind(&mut self, combo: &KeyCombo) -> Option<Action> {
        self.bindings.remove(combo)
    }

    /// Convert Iced key event to KeyCombo
    pub fn from_iced_key(key: &Key, modifiers: &Modifiers) -> Option<KeyCombo> {
        let key_str = match key.as_ref() {
            Key::Character(c) => c.to_string(),
            Key::Named(named) => format!("{named:?}"),
            Key::Unidentified => return None,
        };

        Some(KeyCombo {
            key: normalize_key_name(&key_str),
            modifiers: KeyModifiers {
                ctrl: modifiers.control(),
                alt: modifiers.alt(),
                shift: modifiers.shift(),
                super_: modifiers.command(),
            },
        })
    }
}

impl KeyModifiers {
    /// Create modifiers with no keys pressed
    pub fn none() -> Self {
        Self::default()
    }

    /// Create modifiers with Ctrl
    pub fn ctrl() -> Self {
        Self {
            ctrl: true,
            ..Default::default()
        }
    }

    /// Create modifiers with Alt
    pub fn alt() -> Self {
        Self {
            alt: true,
            ..Default::default()
        }
    }

    /// Create modifiers with Shift
    pub fn shift() -> Self {
        Self {
            shift: true,
            ..Default::default()
        }
    }

    /// Create modifiers with Cmd/Super
    pub fn cmd() -> Self {
        Self {
            super_: true,
            ..Default::default()
        }
    }

    /// Create modifiers with Ctrl+Shift
    pub fn ctrl_shift() -> Self {
        Self {
            ctrl: true,
            shift: true,
            ..Default::default()
        }
    }

    /// Create modifiers with Cmd+Shift
    pub fn cmd_shift() -> Self {
        Self {
            super_: true,
            shift: true,
            ..Default::default()
        }
    }

    /// Create modifiers with Alt+Shift
    pub fn alt_shift() -> Self {
        Self {
            alt: true,
            shift: true,
            ..Default::default()
        }
    }

    /// Convert from config modifiers
    pub fn from_config(config: &ConfigKeyModifiers) -> Self {
        Self {
            ctrl: config.ctrl,
            alt: config.alt,
            shift: config.shift,
            super_: config.cmd,
        }
    }

    /// Check if any modifier is pressed
    pub fn is_any(&self) -> bool {
        self.ctrl || self.alt || self.shift || self.super_
    }
}

/// Normalize key name to a consistent format
fn normalize_key_name(key: &str) -> String {
    // Handle special key names
    match key {
        // Named keys should maintain their case
        "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" => key.to_string(),
        "Enter" | "Escape" | "Tab" | "Backspace" | "Delete" | "Insert" => key.to_string(),
        "Home" | "End" | "PageUp" | "PageDown" => key.to_string(),
        "F1" | "F2" | "F3" | "F4" | "F5" | "F6" | "F7" | "F8" | "F9" | "F10" | "F11" | "F12" => {
            key.to_string()
        }
        "Space" => " ".to_string(),

        // Character keys - keep as-is for proper matching
        _ => key.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_combo_equality() {
        let combo1 = KeyCombo {
            key: "t".to_string(),
            modifiers: KeyModifiers::cmd(),
        };

        let combo2 = KeyCombo {
            key: "t".to_string(),
            modifiers: KeyModifiers::cmd(),
        };

        assert_eq!(combo1, combo2);
    }

    #[test]
    fn test_default_bindings() {
        let kb = KeyBindings::default();

        // Test Cmd+T -> NewTab
        let combo = KeyCombo {
            key: "t".to_string(),
            modifiers: KeyModifiers::cmd(),
        };
        assert!(matches!(kb.get_action(&combo), Some(Action::NewTab)));

        // Test Cmd+W -> CloseTab
        let combo = KeyCombo {
            key: "w".to_string(),
            modifiers: KeyModifiers::cmd(),
        };
        assert!(matches!(kb.get_action(&combo), Some(Action::CloseTab)));
    }

    #[test]
    fn test_action_parsing() {
        assert!(matches!(Action::from_string("new_tab"), Some(Action::NewTab)));
        assert!(matches!(Action::from_string("close_tab"), Some(Action::CloseTab)));
        assert!(matches!(Action::from_string("select_tab_1"), Some(Action::SelectTab(0))));
        assert!(matches!(Action::from_string("select_tab_5"), Some(Action::SelectTab(4))));
    }

    #[test]
    fn test_bind_and_unbind() {
        let mut kb = KeyBindings::default();

        let combo = KeyCombo {
            key: "x".to_string(),
            modifiers: KeyModifiers::ctrl(),
        };

        // Bind custom action
        kb.bind(combo.clone(), Action::Custom("test".to_string()));
        assert!(kb.get_action(&combo).is_some());

        // Unbind
        let removed = kb.unbind(&combo);
        assert!(removed.is_some());
        assert!(kb.get_action(&combo).is_none());
    }

    #[test]
    fn test_normalize_key_name() {
        assert_eq!(normalize_key_name("Enter"), "Enter");
        assert_eq!(normalize_key_name("F12"), "F12");
        assert_eq!(normalize_key_name("ArrowUp"), "ArrowUp");
        assert_eq!(normalize_key_name("Space"), " ");
    }

    #[test]
    fn test_modifier_helpers() {
        assert!(KeyModifiers::none().ctrl == false);
        assert!(KeyModifiers::ctrl().ctrl == true);
        assert!(KeyModifiers::cmd().super_ == true);
        assert!(KeyModifiers::ctrl_shift().ctrl == true);
        assert!(KeyModifiers::ctrl_shift().shift == true);
    }
}
