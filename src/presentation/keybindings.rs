//! Keybinding System
//!
//! Customizable keyboard shortcuts for the TUI.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Key binding action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    // Navigation
    /// Move up
    Up,
    /// Move down
    Down,
    /// Move left
    Left,
    /// Move right
    Right,
    /// Go to first item
    First,
    /// Go to last item
    Last,
    /// Page up
    PageUp,
    /// Page down
    PageDown,

    // Focus
    /// Focus next pane
    FocusNext,
    /// Focus previous pane
    FocusPrevious,
    /// Focus task tree
    FocusTree,
    /// Focus terminal
    FocusTerminal,

    // Views
    /// Toggle help view
    Help,
    /// Toggle graph view
    GraphView,
    /// Toggle archive view
    ArchiveView,
    /// Toggle MCP panel
    McpPanel,
    /// Close current view/popup
    Close,

    // Terminal
    /// Split horizontal
    SplitHorizontal,
    /// Split vertical
    SplitVertical,
    /// Close terminal
    CloseTerminal,
    /// Toggle fullscreen terminal
    ToggleFullscreen,

    // Tasks
    /// Add new task
    AddTask,
    /// Delete selected task
    DeleteTask,
    /// Cancel running task
    CancelTask,
    /// Retry failed task
    RetryTask,
    /// Select/enter
    Select,

    // Search
    /// Start search
    Search,
    /// Clear search
    ClearSearch,
    /// Next search result
    NextResult,
    /// Previous search result
    PrevResult,

    // Application
    /// Quit application
    Quit,
    /// Force quit
    ForceQuit,
    /// Refresh display
    Refresh,
    /// Toggle mouse mode
    ToggleMouse,

    // Scrolling
    /// Scroll up
    ScrollUp,
    /// Scroll down
    ScrollDown,
    /// Scroll to top
    ScrollTop,
    /// Scroll to bottom
    ScrollBottom,

    // Archive
    /// Load selected archive
    LoadArchive,
    /// Export archive
    ExportArchive,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Key combination
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyCombination {
    /// Key code
    pub code: KeyCode,
    /// Modifiers
    pub modifiers: KeyModifiers,
}

impl KeyCombination {
    /// Create a new key combination
    #[must_use]
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    /// Create a key combination without modifiers
    #[must_use]
    pub fn simple(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: KeyModifiers::NONE,
        }
    }

    /// Create a key combination with Ctrl
    #[must_use]
    pub fn ctrl(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: KeyModifiers::CONTROL,
        }
    }

    /// Create a key combination with Alt
    #[must_use]
    pub fn alt(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: KeyModifiers::ALT,
        }
    }

    /// Create a key combination with Shift
    #[must_use]
    pub fn shift(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: KeyModifiers::SHIFT,
        }
    }

    /// Check if this combination matches a key event
    #[must_use]
    pub fn matches(&self, event: &KeyEvent) -> bool {
        event.code == self.code && event.modifiers == self.modifiers
    }

    /// Parse from string representation
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('+').map(str::trim).collect();
        if parts.is_empty() {
            return None;
        }

        let mut modifiers = KeyModifiers::NONE;
        let key_str = parts.last()?;

        for part in &parts[..parts.len() - 1] {
            match part.to_lowercase().as_str() {
                "ctrl" | "control" | "c" => modifiers |= KeyModifiers::CONTROL,
                "alt" | "meta" | "m" => modifiers |= KeyModifiers::ALT,
                "shift" | "s" => modifiers |= KeyModifiers::SHIFT,
                _ => return None,
            }
        }

        let code = parse_key_code(key_str)?;
        Some(Self { code, modifiers })
    }

    /// Format as string
    #[must_use]
    pub fn to_string_repr(&self) -> String {
        let mut parts = Vec::new();

        if self.modifiers.contains(KeyModifiers::CONTROL) {
            parts.push("Ctrl");
        }
        if self.modifiers.contains(KeyModifiers::ALT) {
            parts.push("Alt");
        }
        if self.modifiers.contains(KeyModifiers::SHIFT) {
            parts.push("Shift");
        }

        let key = format_key_code(&self.code);
        parts.push(&key);

        parts.join("+")
    }
}

/// Parse a key code from string
fn parse_key_code(s: &str) -> Option<KeyCode> {
    let s = s.to_lowercase();
    match s.as_str() {
        "enter" | "return" => Some(KeyCode::Enter),
        "esc" | "escape" => Some(KeyCode::Esc),
        "space" => Some(KeyCode::Char(' ')),
        "tab" => Some(KeyCode::Tab),
        "backspace" | "bs" => Some(KeyCode::Backspace),
        "delete" | "del" => Some(KeyCode::Delete),
        "insert" | "ins" => Some(KeyCode::Insert),
        "home" => Some(KeyCode::Home),
        "end" => Some(KeyCode::End),
        "pageup" | "pgup" => Some(KeyCode::PageUp),
        "pagedown" | "pgdn" => Some(KeyCode::PageDown),
        "up" => Some(KeyCode::Up),
        "down" => Some(KeyCode::Down),
        "left" => Some(KeyCode::Left),
        "right" => Some(KeyCode::Right),
        "f1" => Some(KeyCode::F(1)),
        "f2" => Some(KeyCode::F(2)),
        "f3" => Some(KeyCode::F(3)),
        "f4" => Some(KeyCode::F(4)),
        "f5" => Some(KeyCode::F(5)),
        "f6" => Some(KeyCode::F(6)),
        "f7" => Some(KeyCode::F(7)),
        "f8" => Some(KeyCode::F(8)),
        "f9" => Some(KeyCode::F(9)),
        "f10" => Some(KeyCode::F(10)),
        "f11" => Some(KeyCode::F(11)),
        "f12" => Some(KeyCode::F(12)),
        _ => {
            if s.len() == 1 {
                Some(KeyCode::Char(s.chars().next()?))
            } else {
                None
            }
        }
    }
}

/// Format a key code as string
fn format_key_code(code: &KeyCode) -> String {
    match code {
        KeyCode::Char(' ') => "Space".to_string(),
        KeyCode::Char(c) => c.to_uppercase().to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Insert => "Insert".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::F(n) => format!("F{n}"),
        _ => "?".to_string(),
    }
}

/// Keybinding configuration
#[derive(Debug, Clone)]
pub struct Keybindings {
    /// Name of the keybinding set
    pub name: String,
    /// Key to action mappings
    bindings: HashMap<KeyCombination, Action>,
    /// Action to key mappings (for help display)
    reverse: HashMap<Action, Vec<KeyCombination>>,
}

impl Default for Keybindings {
    fn default() -> Self {
        Self::vim()
    }
}

impl Keybindings {
    /// Create empty keybindings
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bindings: HashMap::new(),
            reverse: HashMap::new(),
        }
    }

    /// Create vim-style keybindings
    #[must_use]
    pub fn vim() -> Self {
        let mut kb = Self::new("vim");

        // Navigation (vim-style)
        kb.bind(KeyCombination::simple(KeyCode::Char('j')), Action::Down);
        kb.bind(KeyCombination::simple(KeyCode::Char('k')), Action::Up);
        kb.bind(KeyCombination::simple(KeyCode::Char('h')), Action::Left);
        kb.bind(KeyCombination::simple(KeyCode::Char('l')), Action::Right);
        kb.bind(KeyCombination::simple(KeyCode::Char('g')), Action::First);
        kb.bind(KeyCombination::shift(KeyCode::Char('G')), Action::Last);
        kb.bind(KeyCombination::ctrl(KeyCode::Char('u')), Action::PageUp);
        kb.bind(KeyCombination::ctrl(KeyCode::Char('d')), Action::PageDown);

        // Arrow keys as well
        kb.bind(KeyCombination::simple(KeyCode::Up), Action::Up);
        kb.bind(KeyCombination::simple(KeyCode::Down), Action::Down);
        kb.bind(KeyCombination::simple(KeyCode::Left), Action::Left);
        kb.bind(KeyCombination::simple(KeyCode::Right), Action::Right);

        // Focus
        kb.bind(KeyCombination::simple(KeyCode::Tab), Action::FocusNext);
        kb.bind(
            KeyCombination::shift(KeyCode::BackTab),
            Action::FocusPrevious,
        );
        kb.bind(KeyCombination::ctrl(KeyCode::Char('t')), Action::FocusTree);
        kb.bind(
            KeyCombination::ctrl(KeyCode::Char('p')),
            Action::FocusTerminal,
        );

        // Views
        kb.bind(KeyCombination::simple(KeyCode::F(1)), Action::Help);
        kb.bind(KeyCombination::simple(KeyCode::Char('?')), Action::Help);
        kb.bind(KeyCombination::simple(KeyCode::F(4)), Action::GraphView);
        kb.bind(KeyCombination::simple(KeyCode::F(6)), Action::ArchiveView);
        kb.bind(KeyCombination::simple(KeyCode::F(5)), Action::McpPanel);
        kb.bind(KeyCombination::simple(KeyCode::Esc), Action::Close);
        kb.bind(KeyCombination::simple(KeyCode::Char('q')), Action::Close);

        // Terminal
        kb.bind(KeyCombination::simple(KeyCode::F(3)), Action::SplitVertical);
        kb.bind(
            KeyCombination::shift(KeyCode::F(3)),
            Action::SplitHorizontal,
        );
        kb.bind(
            KeyCombination::ctrl(KeyCode::Char('w')),
            Action::CloseTerminal,
        );
        kb.bind(
            KeyCombination::simple(KeyCode::F(11)),
            Action::ToggleFullscreen,
        );

        // Tasks
        kb.bind(KeyCombination::simple(KeyCode::Char('a')), Action::AddTask);
        kb.bind(
            KeyCombination::simple(KeyCode::Char('d')),
            Action::DeleteTask,
        );
        kb.bind(KeyCombination::ctrl(KeyCode::Char('c')), Action::CancelTask);
        kb.bind(
            KeyCombination::simple(KeyCode::Char('r')),
            Action::RetryTask,
        );
        kb.bind(KeyCombination::simple(KeyCode::Enter), Action::Select);

        // Search
        kb.bind(KeyCombination::simple(KeyCode::Char('/')), Action::Search);
        kb.bind(
            KeyCombination::simple(KeyCode::Char('n')),
            Action::NextResult,
        );
        kb.bind(
            KeyCombination::shift(KeyCode::Char('N')),
            Action::PrevResult,
        );

        // Application
        kb.bind(KeyCombination::ctrl(KeyCode::Char('q')), Action::Quit);
        kb.bind(KeyCombination::ctrl(KeyCode::Char('l')), Action::Refresh);

        // Scrolling
        kb.bind(KeyCombination::ctrl(KeyCode::Char('e')), Action::ScrollDown);
        kb.bind(KeyCombination::ctrl(KeyCode::Char('y')), Action::ScrollUp);
        kb.bind(
            KeyCombination::shift(KeyCode::Char('G')),
            Action::ScrollBottom,
        );

        kb
    }

    /// Create emacs-style keybindings
    #[must_use]
    pub fn emacs() -> Self {
        let mut kb = Self::new("emacs");

        // Navigation (emacs-style)
        kb.bind(KeyCombination::ctrl(KeyCode::Char('n')), Action::Down);
        kb.bind(KeyCombination::ctrl(KeyCode::Char('p')), Action::Up);
        kb.bind(KeyCombination::ctrl(KeyCode::Char('b')), Action::Left);
        kb.bind(KeyCombination::ctrl(KeyCode::Char('f')), Action::Right);
        kb.bind(KeyCombination::alt(KeyCode::Char('<')), Action::First);
        kb.bind(KeyCombination::alt(KeyCode::Char('>')), Action::Last);
        kb.bind(KeyCombination::alt(KeyCode::Char('v')), Action::PageUp);
        kb.bind(KeyCombination::ctrl(KeyCode::Char('v')), Action::PageDown);

        // Arrow keys
        kb.bind(KeyCombination::simple(KeyCode::Up), Action::Up);
        kb.bind(KeyCombination::simple(KeyCode::Down), Action::Down);
        kb.bind(KeyCombination::simple(KeyCode::Left), Action::Left);
        kb.bind(KeyCombination::simple(KeyCode::Right), Action::Right);

        // Focus
        kb.bind(KeyCombination::ctrl(KeyCode::Char('o')), Action::FocusNext);
        kb.bind(
            KeyCombination::alt(KeyCode::Char('o')),
            Action::FocusPrevious,
        );

        // Views
        kb.bind(KeyCombination::simple(KeyCode::F(1)), Action::Help);
        kb.bind(KeyCombination::ctrl(KeyCode::Char('h')), Action::Help);
        kb.bind(KeyCombination::simple(KeyCode::F(4)), Action::GraphView);
        kb.bind(KeyCombination::simple(KeyCode::F(6)), Action::ArchiveView);
        kb.bind(KeyCombination::simple(KeyCode::Esc), Action::Close);
        kb.bind(KeyCombination::ctrl(KeyCode::Char('g')), Action::Close);

        // Tasks
        kb.bind(KeyCombination::ctrl(KeyCode::Char('a')), Action::AddTask);
        kb.bind(KeyCombination::ctrl(KeyCode::Char('d')), Action::DeleteTask);
        kb.bind(KeyCombination::ctrl(KeyCode::Char('c')), Action::CancelTask);
        kb.bind(KeyCombination::simple(KeyCode::Enter), Action::Select);

        // Search
        kb.bind(KeyCombination::ctrl(KeyCode::Char('s')), Action::Search);
        kb.bind(KeyCombination::ctrl(KeyCode::Char('r')), Action::PrevResult);

        // Application
        kb.bind(KeyCombination::ctrl(KeyCode::Char('x')), Action::Quit);
        kb.bind(KeyCombination::ctrl(KeyCode::Char('l')), Action::Refresh);

        kb
    }

    /// Create arrow-based keybindings (for beginners)
    #[must_use]
    pub fn arrows() -> Self {
        let mut kb = Self::new("arrows");

        // Navigation (arrow-based)
        kb.bind(KeyCombination::simple(KeyCode::Up), Action::Up);
        kb.bind(KeyCombination::simple(KeyCode::Down), Action::Down);
        kb.bind(KeyCombination::simple(KeyCode::Left), Action::Left);
        kb.bind(KeyCombination::simple(KeyCode::Right), Action::Right);
        kb.bind(KeyCombination::simple(KeyCode::Home), Action::First);
        kb.bind(KeyCombination::simple(KeyCode::End), Action::Last);
        kb.bind(KeyCombination::simple(KeyCode::PageUp), Action::PageUp);
        kb.bind(KeyCombination::simple(KeyCode::PageDown), Action::PageDown);

        // Focus
        kb.bind(KeyCombination::simple(KeyCode::Tab), Action::FocusNext);
        kb.bind(
            KeyCombination::shift(KeyCode::BackTab),
            Action::FocusPrevious,
        );

        // Views
        kb.bind(KeyCombination::simple(KeyCode::F(1)), Action::Help);
        kb.bind(KeyCombination::simple(KeyCode::F(4)), Action::GraphView);
        kb.bind(KeyCombination::simple(KeyCode::F(6)), Action::ArchiveView);
        kb.bind(KeyCombination::simple(KeyCode::F(5)), Action::McpPanel);
        kb.bind(KeyCombination::simple(KeyCode::Esc), Action::Close);

        // Terminal
        kb.bind(KeyCombination::simple(KeyCode::F(3)), Action::SplitVertical);
        kb.bind(KeyCombination::ctrl(KeyCode::F(4)), Action::CloseTerminal);

        // Tasks
        kb.bind(KeyCombination::simple(KeyCode::Insert), Action::AddTask);
        kb.bind(KeyCombination::simple(KeyCode::Delete), Action::DeleteTask);
        kb.bind(KeyCombination::ctrl(KeyCode::Char('c')), Action::CancelTask);
        kb.bind(KeyCombination::simple(KeyCode::Enter), Action::Select);

        // Search
        kb.bind(KeyCombination::ctrl(KeyCode::Char('f')), Action::Search);
        kb.bind(KeyCombination::simple(KeyCode::F(3)), Action::NextResult);

        // Application
        kb.bind(KeyCombination::alt(KeyCode::F(4)), Action::Quit);
        kb.bind(KeyCombination::simple(KeyCode::F(5)), Action::Refresh);

        kb
    }

    /// Get keybindings by name
    #[must_use]
    pub fn by_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "vim" => Self::vim(),
            "emacs" => Self::emacs(),
            "arrows" | "arrow" | "simple" => Self::arrows(),
            _ => Self::vim(),
        }
    }

    /// Bind a key combination to an action
    pub fn bind(&mut self, key: KeyCombination, action: Action) {
        self.bindings.insert(key.clone(), action);
        self.reverse.entry(action).or_default().push(key);
    }

    /// Unbind a key combination
    pub fn unbind(&mut self, key: &KeyCombination) {
        if let Some(action) = self.bindings.remove(key) {
            if let Some(keys) = self.reverse.get_mut(&action) {
                keys.retain(|k| k != key);
            }
        }
    }

    /// Get the action for a key event
    #[must_use]
    pub fn get_action(&self, event: &KeyEvent) -> Option<Action> {
        let key = KeyCombination::new(event.code, event.modifiers);
        self.bindings.get(&key).copied()
    }

    /// Get all key combinations for an action
    #[must_use]
    pub fn get_keys(&self, action: Action) -> Vec<&KeyCombination> {
        self.reverse
            .get(&action)
            .map(|keys| keys.iter().collect())
            .unwrap_or_default()
    }

    /// Get a formatted string for an action's keys
    #[must_use]
    pub fn format_keys(&self, action: Action) -> String {
        let keys = self.get_keys(action);
        if keys.is_empty() {
            "unbound".to_string()
        } else {
            keys.iter()
                .map(|k| k.to_string_repr())
                .collect::<Vec<_>>()
                .join(" / ")
        }
    }

    /// Get all bindings for help display
    #[must_use]
    pub fn all_bindings(&self) -> Vec<(Action, String)> {
        let mut result: Vec<(Action, String)> = self
            .reverse
            .iter()
            .map(|(action, keys)| {
                let keys_str = keys
                    .iter()
                    .map(KeyCombination::to_string_repr)
                    .collect::<Vec<_>>()
                    .join(" / ");
                (*action, keys_str)
            })
            .collect();

        result.sort_by(|a, b| format!("{:?}", a.0).cmp(&format!("{:?}", b.0)));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_combination_parse() {
        let key = KeyCombination::parse("Ctrl+c").unwrap();
        assert_eq!(key.code, KeyCode::Char('c'));
        assert!(key.modifiers.contains(KeyModifiers::CONTROL));

        let key = KeyCombination::parse("Enter").unwrap();
        assert_eq!(key.code, KeyCode::Enter);
        assert_eq!(key.modifiers, KeyModifiers::NONE);

        let key = KeyCombination::parse("Shift+F3").unwrap();
        assert_eq!(key.code, KeyCode::F(3));
        assert!(key.modifiers.contains(KeyModifiers::SHIFT));
    }

    #[test]
    fn test_vim_keybindings() {
        let kb = Keybindings::vim();

        let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert_eq!(kb.get_action(&event), Some(Action::Down));

        let event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert_eq!(kb.get_action(&event), Some(Action::Up));

        let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        assert_eq!(kb.get_action(&event), Some(Action::Quit));
    }

    #[test]
    fn test_emacs_keybindings() {
        let kb = Keybindings::emacs();

        let event = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL);
        assert_eq!(kb.get_action(&event), Some(Action::Down));

        let event = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL);
        assert_eq!(kb.get_action(&event), Some(Action::Up));
    }

    #[test]
    fn test_bind_unbind() {
        let mut kb = Keybindings::new("test");

        let key = KeyCombination::simple(KeyCode::Char('x'));
        kb.bind(key.clone(), Action::Quit);

        let event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert_eq!(kb.get_action(&event), Some(Action::Quit));

        kb.unbind(&key);
        assert_eq!(kb.get_action(&event), None);
    }

    #[test]
    fn test_format_keys() {
        let kb = Keybindings::vim();
        let keys = kb.format_keys(Action::Quit);
        assert!(keys.contains("Ctrl"));
    }
}
