//! Mouse action handling for AgTerm terminal emulator
//!
//! This module provides comprehensive mouse event handling including:
//! - Click detection (single, double, triple)
//! - Drag operations
//! - Scroll handling
//! - Mouse bindings
//! - Selection types
//! - Link detection

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Mouse button types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}

/// Mouse action types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseAction {
    Click(MouseButton),
    DoubleClick(MouseButton),
    TripleClick(MouseButton),
    Drag(MouseButton),
    Scroll(ScrollDirection),
    Release(MouseButton),
}

/// Scroll direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Mouse position in screen and terminal coordinates
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MousePosition {
    /// Screen X coordinate
    pub x: f32,
    /// Screen Y coordinate
    pub y: f32,
    /// Terminal column
    pub col: usize,
    /// Terminal row
    pub row: usize,
    /// Whether the position is in the scrollback region
    pub in_scrollback: bool,
}

impl MousePosition {
    /// Create a new mouse position
    pub fn new(x: f32, y: f32, col: usize, row: usize, in_scrollback: bool) -> Self {
        Self {
            x,
            y,
            col,
            row,
            in_scrollback,
        }
    }

    /// Calculate distance to another position
    pub fn distance_to(&self, other: &MousePosition) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Keyboard modifiers held during mouse event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MouseModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

impl MouseModifiers {
    /// Create modifiers with no keys pressed
    pub fn none() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        }
    }

    /// Check if any modifiers are active
    pub fn any(&self) -> bool {
        self.shift || self.ctrl || self.alt || self.meta
    }

    /// Check if modifiers match (considering only active ones)
    pub fn matches(&self, other: &MouseModifiers) -> bool {
        self.shift == other.shift
            && self.ctrl == other.ctrl
            && self.alt == other.alt
            && self.meta == other.meta
    }
}

/// Complete mouse event with all context
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MouseEvent {
    pub action: MouseAction,
    pub position: MousePosition,
    pub modifiers: MouseModifiers,
    #[serde(skip, default = "Instant::now")]
    pub timestamp: Instant,
    pub click_count: u8,
}

impl MouseEvent {
    /// Create a new mouse event
    pub fn new(
        action: MouseAction,
        position: MousePosition,
        modifiers: MouseModifiers,
        click_count: u8,
    ) -> Self {
        Self {
            action,
            position,
            modifiers,
            timestamp: Instant::now(),
            click_count,
        }
    }
}

/// Commands that can be triggered by mouse actions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MouseCommand {
    Select(SelectionType),
    ExtendSelection,
    OpenLink,
    ContextMenu,
    PasteSelection,
    PastePrimary,
    ScrollLines(i32),
    ScrollPages(f32),
    CopySelection,
    SearchSelection,
    OpenPath,
    Custom(String),
}

/// Types of text selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SelectionType {
    Character,
    Word,
    Line,
    Block,
}

/// Mouse binding configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MouseBinding {
    pub action: MouseAction,
    pub modifiers: MouseModifiers,
    pub command: MouseCommand,
    pub enabled: bool,
    pub description: String,
}

impl MouseBinding {
    /// Create a new mouse binding
    pub fn new(
        action: MouseAction,
        modifiers: MouseModifiers,
        command: MouseCommand,
        description: impl Into<String>,
    ) -> Self {
        Self {
            action,
            modifiers,
            command,
            enabled: true,
            description: description.into(),
        }
    }

    /// Check if this binding matches an event
    pub fn matches(&self, event: &MouseEvent) -> bool {
        self.enabled && self.action == event.action && self.modifiers.matches(&event.modifiers)
    }
}

/// Multi-click detection state
#[derive(Debug)]
pub struct ClickDetector {
    max_click_interval: Duration,
    max_click_distance: f32,
    last_click_time: Option<Instant>,
    last_click_position: Option<MousePosition>,
    last_button: Option<MouseButton>,
    click_count: u8,
}

impl ClickDetector {
    /// Create a new click detector with default settings
    pub fn new() -> Self {
        Self {
            max_click_interval: Duration::from_millis(500),
            max_click_distance: 5.0,
            last_click_time: None,
            last_click_position: None,
            last_button: None,
            click_count: 0,
        }
    }

    /// Detect click count for an event
    pub fn detect(&mut self, event: &MouseEvent) -> u8 {
        let button = match &event.action {
            MouseAction::Click(btn) => *btn,
            _ => {
                return 1;
            }
        };

        let should_increment = if let (Some(last_time), Some(last_pos), Some(last_btn)) = (
            self.last_click_time,
            self.last_click_position,
            self.last_button,
        ) {
            let time_ok = event.timestamp.duration_since(last_time) <= self.max_click_interval;
            let distance_ok = event.position.distance_to(&last_pos) <= self.max_click_distance;
            let button_ok = button == last_btn;

            time_ok && distance_ok && button_ok
        } else {
            false
        };

        if should_increment {
            self.click_count = self.click_count.saturating_add(1).min(3);
        } else {
            self.click_count = 1;
        }

        self.last_click_time = Some(event.timestamp);
        self.last_click_position = Some(event.position);
        self.last_button = Some(button);

        self.click_count
    }

    /// Reset detection state
    pub fn reset(&mut self) {
        self.last_click_time = None;
        self.last_click_position = None;
        self.last_button = None;
        self.click_count = 0;
    }

    /// Configure max interval between clicks
    pub fn set_max_interval(&mut self, duration: Duration) {
        self.max_click_interval = duration;
    }

    /// Configure max distance between clicks
    pub fn set_max_distance(&mut self, distance: f32) {
        self.max_click_distance = distance;
    }
}

impl Default for ClickDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Drag operation state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DragState {
    pub start_position: MousePosition,
    pub current_position: MousePosition,
    pub button: MouseButton,
    #[serde(skip, default = "Instant::now")]
    pub start_time: Instant,
}

impl DragState {
    /// Create a new drag state
    pub fn new(start_position: MousePosition, button: MouseButton) -> Self {
        Self {
            start_position,
            current_position: start_position,
            button,
            start_time: Instant::now(),
        }
    }

    /// Update current position
    pub fn update_position(&mut self, position: MousePosition) {
        self.current_position = position;
    }

    /// Calculate distance dragged
    pub fn distance(&self) -> f32 {
        self.start_position.distance_to(&self.current_position)
    }

    /// Calculate drag duration
    pub fn duration(&self) -> Duration {
        Instant::now().duration_since(self.start_time)
    }
}

/// Main mouse event handler
#[derive(Debug)]
pub struct MouseHandler {
    bindings: Vec<MouseBinding>,
    click_detector: ClickDetector,
    drag_state: Option<DragState>,
}

impl MouseHandler {
    /// Create a new mouse handler with default bindings
    pub fn new() -> Self {
        let mut handler = Self {
            bindings: Vec::new(),
            click_detector: ClickDetector::new(),
            drag_state: None,
        };
        handler.add_default_bindings();
        handler
    }

    /// Add default mouse bindings
    fn add_default_bindings(&mut self) {
        // Left click: Start character selection
        self.add_binding(MouseBinding::new(
            MouseAction::Click(MouseButton::Left),
            MouseModifiers::none(),
            MouseCommand::Select(SelectionType::Character),
            "Start character selection",
        ));

        // Left double-click: Select word
        self.add_binding(MouseBinding::new(
            MouseAction::DoubleClick(MouseButton::Left),
            MouseModifiers::none(),
            MouseCommand::Select(SelectionType::Word),
            "Select word",
        ));

        // Left triple-click: Select line
        self.add_binding(MouseBinding::new(
            MouseAction::TripleClick(MouseButton::Left),
            MouseModifiers::none(),
            MouseCommand::Select(SelectionType::Line),
            "Select line",
        ));

        // Left drag: Extend selection
        self.add_binding(MouseBinding::new(
            MouseAction::Drag(MouseButton::Left),
            MouseModifiers::none(),
            MouseCommand::ExtendSelection,
            "Extend selection",
        ));

        // Right click: Context menu
        self.add_binding(MouseBinding::new(
            MouseAction::Click(MouseButton::Right),
            MouseModifiers::none(),
            MouseCommand::ContextMenu,
            "Open context menu",
        ));

        // Middle click: Paste selection
        self.add_binding(MouseBinding::new(
            MouseAction::Click(MouseButton::Middle),
            MouseModifiers::none(),
            MouseCommand::PasteSelection,
            "Paste selection",
        ));

        // Ctrl+Click: Open link
        self.add_binding(MouseBinding::new(
            MouseAction::Click(MouseButton::Left),
            MouseModifiers {
                shift: false,
                ctrl: true,
                alt: false,
                meta: false,
            },
            MouseCommand::OpenLink,
            "Open link or path",
        ));

        // Scroll up: Scroll 3 lines up
        self.add_binding(MouseBinding::new(
            MouseAction::Scroll(ScrollDirection::Up),
            MouseModifiers::none(),
            MouseCommand::ScrollLines(-3),
            "Scroll up",
        ));

        // Scroll down: Scroll 3 lines down
        self.add_binding(MouseBinding::new(
            MouseAction::Scroll(ScrollDirection::Down),
            MouseModifiers::none(),
            MouseCommand::ScrollLines(3),
            "Scroll down",
        ));

        // Shift+Scroll up: Scroll left
        self.add_binding(MouseBinding::new(
            MouseAction::Scroll(ScrollDirection::Up),
            MouseModifiers {
                shift: true,
                ctrl: false,
                alt: false,
                meta: false,
            },
            MouseCommand::ScrollLines(-3),
            "Scroll left",
        ));

        // Shift+Scroll down: Scroll right
        self.add_binding(MouseBinding::new(
            MouseAction::Scroll(ScrollDirection::Down),
            MouseModifiers {
                shift: true,
                ctrl: false,
                alt: false,
                meta: false,
            },
            MouseCommand::ScrollLines(3),
            "Scroll right",
        ));
    }

    /// Handle a mouse event and return the associated command
    pub fn handle_event(&mut self, mut event: MouseEvent) -> Option<MouseCommand> {
        // Update click count
        if matches!(event.action, MouseAction::Click(_)) {
            event.click_count = self.click_detector.detect(&event);

            // Update action based on click count
            if event.click_count >= 3 {
                if let MouseAction::Click(btn) = event.action {
                    event.action = MouseAction::TripleClick(btn);
                }
            } else if event.click_count == 2 {
                if let MouseAction::Click(btn) = event.action {
                    event.action = MouseAction::DoubleClick(btn);
                }
            }
        }

        // Update drag state
        match &event.action {
            MouseAction::Click(btn) => {
                self.drag_state = Some(DragState::new(event.position, *btn));
            }
            MouseAction::Drag(btn) => {
                if let Some(ref mut drag) = self.drag_state {
                    if drag.button == *btn {
                        drag.update_position(event.position);
                    }
                }
            }
            MouseAction::Release(_) => {
                self.drag_state = None;
            }
            _ => {}
        }

        // Find matching binding
        self.bindings
            .iter()
            .find(|binding| binding.matches(&event))
            .map(|binding| binding.command.clone())
    }

    /// Add a mouse binding
    pub fn add_binding(&mut self, binding: MouseBinding) {
        self.bindings.push(binding);
    }

    /// Remove a mouse binding
    pub fn remove_binding(&mut self, action: &MouseAction, mods: &MouseModifiers) {
        self.bindings
            .retain(|b| &b.action != action || !b.modifiers.matches(mods));
    }

    /// Get a binding by action and modifiers
    pub fn get_binding(
        &self,
        action: &MouseAction,
        mods: &MouseModifiers,
    ) -> Option<&MouseBinding> {
        self.bindings
            .iter()
            .find(|b| &b.action == action && b.modifiers.matches(mods))
    }

    /// Set binding enabled state
    pub fn set_enabled(&mut self, action: &MouseAction, mods: &MouseModifiers, enabled: bool) {
        if let Some(binding) = self
            .bindings
            .iter_mut()
            .find(|b| &b.action == action && b.modifiers.matches(mods))
        {
            binding.enabled = enabled;
        }
    }

    /// Get current drag state
    pub fn drag_state(&self) -> Option<&DragState> {
        self.drag_state.as_ref()
    }

    /// Reset click detector
    pub fn reset_click_detector(&mut self) {
        self.click_detector.reset();
    }
}

impl Default for MouseHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn create_test_position() -> MousePosition {
        MousePosition::new(100.0, 200.0, 10, 20, false)
    }

    fn create_test_event(action: MouseAction) -> MouseEvent {
        MouseEvent::new(action, create_test_position(), MouseModifiers::none(), 1)
    }

    #[test]
    fn test_mouse_position_distance() {
        let pos1 = MousePosition::new(0.0, 0.0, 0, 0, false);
        let pos2 = MousePosition::new(3.0, 4.0, 1, 1, false);
        assert_eq!(pos1.distance_to(&pos2), 5.0);
    }

    #[test]
    fn test_modifiers_none() {
        let mods = MouseModifiers::none();
        assert!(!mods.shift);
        assert!(!mods.ctrl);
        assert!(!mods.alt);
        assert!(!mods.meta);
        assert!(!mods.any());
    }

    #[test]
    fn test_modifiers_any() {
        let mut mods = MouseModifiers::none();
        assert!(!mods.any());

        mods.ctrl = true;
        assert!(mods.any());
    }

    #[test]
    fn test_modifiers_matches() {
        let mods1 = MouseModifiers {
            shift: true,
            ctrl: false,
            alt: false,
            meta: false,
        };
        let mods2 = MouseModifiers {
            shift: true,
            ctrl: false,
            alt: false,
            meta: false,
        };
        let mods3 = MouseModifiers::none();

        assert!(mods1.matches(&mods2));
        assert!(!mods1.matches(&mods3));
    }

    #[test]
    fn test_click_detector_single_click() {
        let mut detector = ClickDetector::new();
        let event = create_test_event(MouseAction::Click(MouseButton::Left));

        let count = detector.detect(&event);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_click_detector_double_click() {
        let mut detector = ClickDetector::new();
        let event1 = create_test_event(MouseAction::Click(MouseButton::Left));
        let event2 = create_test_event(MouseAction::Click(MouseButton::Left));

        detector.detect(&event1);
        let count = detector.detect(&event2);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_click_detector_triple_click() {
        let mut detector = ClickDetector::new();
        let event1 = create_test_event(MouseAction::Click(MouseButton::Left));
        let event2 = create_test_event(MouseAction::Click(MouseButton::Left));
        let event3 = create_test_event(MouseAction::Click(MouseButton::Left));

        detector.detect(&event1);
        detector.detect(&event2);
        let count = detector.detect(&event3);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_click_detector_timeout() {
        let mut detector = ClickDetector::new();
        detector.set_max_interval(Duration::from_millis(50));

        let event1 = create_test_event(MouseAction::Click(MouseButton::Left));
        detector.detect(&event1);

        thread::sleep(Duration::from_millis(100));

        let event2 = create_test_event(MouseAction::Click(MouseButton::Left));
        let count = detector.detect(&event2);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_click_detector_distance() {
        let mut detector = ClickDetector::new();
        detector.set_max_distance(5.0);

        let pos1 = MousePosition::new(0.0, 0.0, 0, 0, false);
        let pos2 = MousePosition::new(100.0, 100.0, 10, 10, false);

        let event1 = MouseEvent::new(
            MouseAction::Click(MouseButton::Left),
            pos1,
            MouseModifiers::none(),
            1,
        );
        let event2 = MouseEvent::new(
            MouseAction::Click(MouseButton::Left),
            pos2,
            MouseModifiers::none(),
            1,
        );

        detector.detect(&event1);
        let count = detector.detect(&event2);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_click_detector_different_buttons() {
        let mut detector = ClickDetector::new();

        let event1 = create_test_event(MouseAction::Click(MouseButton::Left));
        let event2 = create_test_event(MouseAction::Click(MouseButton::Right));

        detector.detect(&event1);
        let count = detector.detect(&event2);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_click_detector_reset() {
        let mut detector = ClickDetector::new();

        let event = create_test_event(MouseAction::Click(MouseButton::Left));
        detector.detect(&event);

        detector.reset();
        assert_eq!(detector.click_count, 0);
        assert!(detector.last_click_time.is_none());
    }

    #[test]
    fn test_drag_state_creation() {
        let pos = create_test_position();
        let drag = DragState::new(pos, MouseButton::Left);

        assert_eq!(drag.start_position, pos);
        assert_eq!(drag.current_position, pos);
        assert_eq!(drag.button, MouseButton::Left);
        assert_eq!(drag.distance(), 0.0);
    }

    #[test]
    fn test_drag_state_update() {
        let pos1 = MousePosition::new(0.0, 0.0, 0, 0, false);
        let pos2 = MousePosition::new(3.0, 4.0, 1, 1, false);

        let mut drag = DragState::new(pos1, MouseButton::Left);
        drag.update_position(pos2);

        assert_eq!(drag.current_position, pos2);
        assert_eq!(drag.distance(), 5.0);
    }

    #[test]
    fn test_mouse_binding_matches() {
        let binding = MouseBinding::new(
            MouseAction::Click(MouseButton::Left),
            MouseModifiers::none(),
            MouseCommand::Select(SelectionType::Character),
            "Test",
        );

        let event = create_test_event(MouseAction::Click(MouseButton::Left));
        assert!(binding.matches(&event));

        let event2 = create_test_event(MouseAction::Click(MouseButton::Right));
        assert!(!binding.matches(&event2));
    }

    #[test]
    fn test_mouse_handler_creation() {
        let handler = MouseHandler::new();
        assert!(!handler.bindings.is_empty());
    }

    #[test]
    fn test_mouse_handler_add_binding() {
        let mut handler = MouseHandler::new();
        let initial_count = handler.bindings.len();

        handler.add_binding(MouseBinding::new(
            MouseAction::Click(MouseButton::Back),
            MouseModifiers::none(),
            MouseCommand::Custom("test".to_string()),
            "Test binding",
        ));

        assert_eq!(handler.bindings.len(), initial_count + 1);
    }

    #[test]
    fn test_mouse_handler_remove_binding() {
        let mut handler = MouseHandler::new();
        let action = MouseAction::Click(MouseButton::Left);
        let mods = MouseModifiers::none();

        let initial_count = handler.bindings.len();
        handler.remove_binding(&action, &mods);

        assert!(handler.bindings.len() < initial_count);
    }

    #[test]
    fn test_mouse_handler_get_binding() {
        let handler = MouseHandler::new();
        let action = MouseAction::Click(MouseButton::Left);
        let mods = MouseModifiers::none();

        let binding = handler.get_binding(&action, &mods);
        assert!(binding.is_some());
    }

    #[test]
    fn test_mouse_handler_set_enabled() {
        let mut handler = MouseHandler::new();
        let action = MouseAction::Click(MouseButton::Left);
        let mods = MouseModifiers::none();

        handler.set_enabled(&action, &mods, false);

        let binding = handler.get_binding(&action, &mods);
        assert!(binding.is_some());
        assert!(!binding.unwrap().enabled);
    }

    #[test]
    fn test_mouse_handler_event_handling() {
        let mut handler = MouseHandler::new();
        let event = create_test_event(MouseAction::Click(MouseButton::Left));

        let command = handler.handle_event(event);
        assert!(command.is_some());
        assert_eq!(
            command.unwrap(),
            MouseCommand::Select(SelectionType::Character)
        );
    }

    #[test]
    fn test_mouse_handler_double_click_detection() {
        let mut handler = MouseHandler::new();
        let event1 = create_test_event(MouseAction::Click(MouseButton::Left));
        let event2 = create_test_event(MouseAction::Click(MouseButton::Left));

        handler.handle_event(event1);
        let command = handler.handle_event(event2);

        assert!(command.is_some());
        assert_eq!(command.unwrap(), MouseCommand::Select(SelectionType::Word));
    }

    #[test]
    fn test_mouse_handler_triple_click_detection() {
        let mut handler = MouseHandler::new();
        let event1 = create_test_event(MouseAction::Click(MouseButton::Left));
        let event2 = create_test_event(MouseAction::Click(MouseButton::Left));
        let event3 = create_test_event(MouseAction::Click(MouseButton::Left));

        handler.handle_event(event1);
        handler.handle_event(event2);
        let command = handler.handle_event(event3);

        assert!(command.is_some());
        assert_eq!(command.unwrap(), MouseCommand::Select(SelectionType::Line));
    }

    #[test]
    fn test_mouse_handler_drag_state() {
        let mut handler = MouseHandler::new();
        let event = create_test_event(MouseAction::Click(MouseButton::Left));

        handler.handle_event(event);
        assert!(handler.drag_state().is_some());

        let release_event = create_test_event(MouseAction::Release(MouseButton::Left));
        handler.handle_event(release_event);
        assert!(handler.drag_state().is_none());
    }

    #[test]
    fn test_selection_types() {
        let types = vec![
            SelectionType::Character,
            SelectionType::Word,
            SelectionType::Line,
            SelectionType::Block,
        ];

        for t in types {
            let cmd = MouseCommand::Select(t);
            assert!(matches!(cmd, MouseCommand::Select(_)));
        }
    }

    #[test]
    fn test_scroll_commands() {
        let handler = MouseHandler::new();

        // Test that scroll bindings exist
        let up_binding = handler
            .bindings
            .iter()
            .find(|b| b.action == MouseAction::Scroll(ScrollDirection::Up));
        let down_binding = handler
            .bindings
            .iter()
            .find(|b| b.action == MouseAction::Scroll(ScrollDirection::Down));

        assert!(up_binding.is_some());
        assert!(down_binding.is_some());
    }

    #[test]
    fn test_ctrl_click_open_link() {
        let mut handler = MouseHandler::new();
        let mods = MouseModifiers {
            shift: false,
            ctrl: true,
            alt: false,
            meta: false,
        };
        let event = MouseEvent::new(
            MouseAction::Click(MouseButton::Left),
            create_test_position(),
            mods,
            1,
        );

        let command = handler.handle_event(event);
        assert!(command.is_some());
        assert_eq!(command.unwrap(), MouseCommand::OpenLink);
    }

    #[test]
    fn test_middle_click_paste() {
        let mut handler = MouseHandler::new();
        let event = create_test_event(MouseAction::Click(MouseButton::Middle));

        let command = handler.handle_event(event);
        assert!(command.is_some());
        assert_eq!(command.unwrap(), MouseCommand::PasteSelection);
    }

    #[test]
    fn test_right_click_context_menu() {
        let mut handler = MouseHandler::new();
        let event = create_test_event(MouseAction::Click(MouseButton::Right));

        let command = handler.handle_event(event);
        assert!(command.is_some());
        assert_eq!(command.unwrap(), MouseCommand::ContextMenu);
    }
}
