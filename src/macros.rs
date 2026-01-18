//! Input macro system for AgTerm
//!
//! This module provides a powerful macro system that allows users to:
//! - Record and replay sequences of keyboard inputs
//! - Define complex automation workflows
//! - Create reusable command templates
//! - Execute shell commands as part of macros
//! - Support conditional logic and repetition

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;

/// Errors that can occur during macro operations
#[derive(Debug, Error)]
pub enum MacroError {
    #[error("Macro not found: {0}")]
    NotFound(String),

    #[error("Macro already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid macro name: {0}")]
    InvalidName(String),

    #[error("Macro recording not in progress")]
    NotRecording,

    #[error("Macro recording already in progress")]
    AlreadyRecording,

    #[error("Empty macro: {0}")]
    EmptyMacro(String),

    #[error("Invalid action: {0}")]
    InvalidAction(String),

    #[error("Maximum recursion depth exceeded")]
    MaxRecursionDepth,
}

/// A key combination consisting of a key and modifier keys
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyCombo {
    /// The key name
    pub key: String,
    /// Modifier keys pressed
    pub modifiers: KeyModifiers,
}

/// Modifier keys for a key binding
#[derive(Debug, Clone, Hash, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub super_: bool, // Cmd on macOS, Win on Windows
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

    /// Create modifiers with Ctrl+Alt
    pub fn ctrl_alt() -> Self {
        Self {
            ctrl: true,
            alt: true,
            ..Default::default()
        }
    }
}

/// A single action that can be performed as part of a macro
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MacroAction {
    /// Send plain text to the terminal
    SendText(String),

    /// Send a sequence of key events (including modifiers)
    SendKeys(Vec<KeyEvent>),

    /// Execute a shell command
    RunCommand(String),

    /// Wait for a specified duration
    Wait(DurationMs),

    /// Repeat an action multiple times
    Repeat {
        action: Box<MacroAction>,
        count: usize,
    },

    /// Execute a sequence of actions
    Sequence(Vec<MacroAction>),

    /// Execute another macro by name
    CallMacro(String),
}

/// A key event with modifiers (serializable version of Iced's key event)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyEvent {
    /// The key pressed
    pub key: String,

    /// Modifier keys
    pub modifiers: KeyModifiers,
}

/// Duration in milliseconds (serializable)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurationMs(pub u64);

impl From<Duration> for DurationMs {
    fn from(duration: Duration) -> Self {
        Self(duration.as_millis() as u64)
    }
}

impl From<DurationMs> for Duration {
    fn from(ms: DurationMs) -> Self {
        Duration::from_millis(ms.0)
    }
}

impl KeyEvent {
    /// Create a new key event
    pub fn new(key: String, modifiers: KeyModifiers) -> Self {
        Self { key, modifiers }
    }

    /// Convert to a KeyCombo for trigger matching
    pub fn to_key_combo(&self) -> KeyCombo {
        KeyCombo {
            key: self.key.clone(),
            modifiers: self.modifiers.clone(),
        }
    }
}

/// A complete macro definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Macro {
    /// Unique name of the macro
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Optional key combination to trigger this macro
    pub trigger: Option<KeyCombo>,

    /// Sequence of actions to execute
    pub actions: Vec<MacroAction>,

    /// Whether this macro is enabled
    pub enabled: bool,
}

impl Macro {
    /// Create a new macro
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            trigger: None,
            actions: Vec::new(),
            enabled: true,
        }
    }

    /// Set the trigger key combination
    pub fn with_trigger(mut self, trigger: KeyCombo) -> Self {
        self.trigger = Some(trigger);
        self
    }

    /// Add an action to the macro
    pub fn add_action(mut self, action: MacroAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Set all actions at once
    pub fn with_actions(mut self, actions: Vec<MacroAction>) -> Self {
        self.actions = actions;
        self
    }

    /// Enable or disable the macro
    pub fn set_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Validate the macro
    pub fn validate(&self) -> Result<(), MacroError> {
        if self.name.is_empty() {
            return Err(MacroError::InvalidName("Macro name cannot be empty".into()));
        }

        if self.actions.is_empty() {
            return Err(MacroError::EmptyMacro(self.name.clone()));
        }

        Ok(())
    }
}

/// Recording state for macro creation
#[derive(Debug, Clone)]
pub struct RecordingState {
    /// Name of the macro being recorded
    pub macro_name: String,

    /// Actions recorded so far
    pub actions: Vec<MacroAction>,

    /// Whether to capture timing information
    pub capture_timing: bool,

    /// Timestamp of the last action (for timing capture)
    pub last_action_time: Option<std::time::Instant>,
}

/// The macro execution engine
pub struct MacroEngine {
    /// All registered macros (name -> macro)
    macros: HashMap<String, Macro>,

    /// Trigger key combinations mapped to macro names
    triggers: HashMap<KeyCombo, String>,

    /// Current recording state (if recording)
    recording: Option<RecordingState>,

    /// Maximum recursion depth for CallMacro actions
    max_recursion_depth: usize,
}

impl MacroEngine {
    /// Create a new macro engine
    pub fn new() -> Self {
        Self {
            macros: HashMap::new(),
            triggers: HashMap::new(),
            recording: None,
            max_recursion_depth: 10,
        }
    }

    /// Register a new macro
    pub fn register(&mut self, macro_def: Macro) -> Result<(), MacroError> {
        macro_def.validate()?;

        if self.macros.contains_key(&macro_def.name) {
            return Err(MacroError::AlreadyExists(macro_def.name.clone()));
        }

        // Register trigger if present
        if let Some(ref trigger) = macro_def.trigger {
            self.triggers
                .insert(trigger.clone(), macro_def.name.clone());
        }

        self.macros.insert(macro_def.name.clone(), macro_def);
        Ok(())
    }

    /// Unregister a macro by name
    pub fn unregister(&mut self, name: &str) -> Result<Macro, MacroError> {
        let macro_def = self
            .macros
            .remove(name)
            .ok_or_else(|| MacroError::NotFound(name.to_string()))?;

        // Remove trigger mapping
        if let Some(ref trigger) = macro_def.trigger {
            self.triggers.remove(trigger);
        }

        Ok(macro_def)
    }

    /// Get a macro by name
    pub fn get(&self, name: &str) -> Option<&Macro> {
        self.macros.get(name)
    }

    /// Get a mutable reference to a macro
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Macro> {
        self.macros.get_mut(name)
    }

    /// List all macro names
    pub fn list(&self) -> Vec<String> {
        self.macros.keys().cloned().collect()
    }

    /// Check if a key combination triggers a macro
    pub fn match_trigger(&self, combo: &KeyCombo) -> Option<&str> {
        self.triggers.get(combo).map(|s| s.as_str())
    }

    /// Execute a macro by name
    pub fn execute(&self, name: &str) -> Result<Vec<MacroAction>, MacroError> {
        self.execute_with_depth(name, 0)
    }

    /// Execute a macro with recursion depth tracking
    fn execute_with_depth(
        &self,
        name: &str,
        depth: usize,
    ) -> Result<Vec<MacroAction>, MacroError> {
        if depth >= self.max_recursion_depth {
            return Err(MacroError::MaxRecursionDepth);
        }

        let macro_def = self
            .macros
            .get(name)
            .ok_or_else(|| MacroError::NotFound(name.to_string()))?;

        if !macro_def.enabled {
            return Ok(Vec::new());
        }

        let mut result = Vec::new();

        for action in &macro_def.actions {
            match action {
                MacroAction::CallMacro(ref called_name) => {
                    // Recursively execute the called macro
                    let sub_actions = self.execute_with_depth(called_name, depth + 1)?;
                    result.extend(sub_actions);
                }
                MacroAction::Repeat { action, count } => {
                    // Expand repeat into multiple actions
                    for _ in 0..*count {
                        if let MacroAction::CallMacro(ref called_name) = **action {
                            let sub_actions = self.execute_with_depth(called_name, depth + 1)?;
                            result.extend(sub_actions);
                        } else {
                            result.push((**action).clone());
                        }
                    }
                }
                MacroAction::Sequence(ref seq) => {
                    // Flatten sequence
                    for seq_action in seq {
                        if let MacroAction::CallMacro(ref called_name) = seq_action {
                            let sub_actions = self.execute_with_depth(called_name, depth + 1)?;
                            result.extend(sub_actions);
                        } else {
                            result.push(seq_action.clone());
                        }
                    }
                }
                other => {
                    result.push(other.clone());
                }
            }
        }

        Ok(result)
    }

    /// Start recording a new macro
    pub fn start_recording(&mut self, name: String, capture_timing: bool) -> Result<(), MacroError> {
        if self.recording.is_some() {
            return Err(MacroError::AlreadyRecording);
        }

        if name.is_empty() {
            return Err(MacroError::InvalidName("Macro name cannot be empty".into()));
        }

        self.recording = Some(RecordingState {
            macro_name: name,
            actions: Vec::new(),
            capture_timing,
            last_action_time: None,
        });

        Ok(())
    }

    /// Stop recording and create the macro
    pub fn stop_recording(&mut self, description: String, trigger: Option<KeyCombo>) -> Result<Macro, MacroError> {
        let recording = self
            .recording
            .take()
            .ok_or(MacroError::NotRecording)?;

        if recording.actions.is_empty() {
            return Err(MacroError::EmptyMacro(recording.macro_name.clone()));
        }

        let mut macro_def = Macro::new(recording.macro_name, description)
            .with_actions(recording.actions);

        if let Some(trigger) = trigger {
            macro_def = macro_def.with_trigger(trigger);
        }

        Ok(macro_def)
    }

    /// Cancel recording without creating a macro
    pub fn cancel_recording(&mut self) -> Result<(), MacroError> {
        if self.recording.is_none() {
            return Err(MacroError::NotRecording);
        }

        self.recording = None;
        Ok(())
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        self.recording.is_some()
    }

    /// Get the current recording state
    pub fn recording_state(&self) -> Option<&RecordingState> {
        self.recording.as_ref()
    }

    /// Record a key event (if recording)
    pub fn record_key(&mut self, key_event: KeyEvent) {
        if let Some(ref mut recording) = self.recording {
            let now = std::time::Instant::now();

            // Add timing wait if capture_timing is enabled
            if recording.capture_timing {
                if let Some(last_time) = recording.last_action_time {
                    let elapsed = now.duration_since(last_time);
                    if elapsed.as_millis() > 100 {
                        // Only record significant delays
                        recording
                            .actions
                            .push(MacroAction::Wait(elapsed.into()));
                    }
                }
            }

            recording
                .actions
                .push(MacroAction::SendKeys(vec![key_event]));
            recording.last_action_time = Some(now);
        }
    }

    /// Record text input (if recording)
    pub fn record_text(&mut self, text: String) {
        if let Some(ref mut recording) = self.recording {
            let now = std::time::Instant::now();

            // Add timing wait if capture_timing is enabled
            if recording.capture_timing {
                if let Some(last_time) = recording.last_action_time {
                    let elapsed = now.duration_since(last_time);
                    if elapsed.as_millis() > 100 {
                        recording
                            .actions
                            .push(MacroAction::Wait(elapsed.into()));
                    }
                }
            }

            recording.actions.push(MacroAction::SendText(text));
            recording.last_action_time = Some(now);
        }
    }

    /// Load macros from a configuration
    pub fn load_from_config(&mut self, macros: Vec<Macro>) -> Result<(), MacroError> {
        for macro_def in macros {
            self.register(macro_def)?;
        }
        Ok(())
    }

    /// Export all macros to a serializable format
    pub fn export_all(&self) -> Vec<Macro> {
        self.macros.values().cloned().collect()
    }

    /// Clear all macros
    pub fn clear(&mut self) {
        self.macros.clear();
        self.triggers.clear();
    }

    /// Set maximum recursion depth
    pub fn set_max_recursion_depth(&mut self, depth: usize) {
        self.max_recursion_depth = depth;
    }
}

impl Default for MacroEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create common macro actions
pub mod builders {
    use super::*;

    /// Send a line of text (with Enter at the end)
    pub fn send_line(text: impl Into<String>) -> MacroAction {
        let mut text = text.into();
        text.push('\n');
        MacroAction::SendText(text)
    }

    /// Send text without Enter
    pub fn send_text(text: impl Into<String>) -> MacroAction {
        MacroAction::SendText(text.into())
    }

    /// Run a shell command
    pub fn run_command(cmd: impl Into<String>) -> MacroAction {
        MacroAction::RunCommand(cmd.into())
    }

    /// Wait for milliseconds
    pub fn wait_ms(ms: u64) -> MacroAction {
        MacroAction::Wait(DurationMs(ms))
    }

    /// Wait for seconds
    pub fn wait_secs(secs: u64) -> MacroAction {
        MacroAction::Wait(DurationMs(secs * 1000))
    }

    /// Repeat an action
    pub fn repeat(action: MacroAction, count: usize) -> MacroAction {
        MacroAction::Repeat {
            action: Box::new(action),
            count,
        }
    }

    /// Create a sequence of actions
    pub fn sequence(actions: Vec<MacroAction>) -> MacroAction {
        MacroAction::Sequence(actions)
    }

    /// Call another macro
    pub fn call_macro(name: impl Into<String>) -> MacroAction {
        MacroAction::CallMacro(name.into())
    }

    /// Send Ctrl+C
    pub fn send_ctrl_c() -> MacroAction {
        MacroAction::SendKeys(vec![KeyEvent {
            key: "c".to_string(),
            modifiers: KeyModifiers {
                ctrl: true,
                alt: false,
                shift: false,
                super_: false,
            },
        }])
    }

    /// Send Enter key
    pub fn send_enter() -> MacroAction {
        MacroAction::SendKeys(vec![KeyEvent {
            key: "Enter".to_string(),
            modifiers: KeyModifiers::default(),
        }])
    }

    /// Send Escape key
    pub fn send_escape() -> MacroAction {
        MacroAction::SendKeys(vec![KeyEvent {
            key: "Escape".to_string(),
            modifiers: KeyModifiers::default(),
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_creation() {
        let macro_def = Macro::new("test".to_string(), "Test macro".to_string())
            .add_action(MacroAction::SendText("hello".to_string()))
            .add_action(MacroAction::Wait(DurationMs(100)));

        assert_eq!(macro_def.name, "test");
        assert_eq!(macro_def.actions.len(), 2);
        assert!(macro_def.validate().is_ok());
    }

    #[test]
    fn test_empty_macro_validation() {
        let macro_def = Macro::new("empty".to_string(), "Empty macro".to_string());
        assert!(macro_def.validate().is_err());
    }

    #[test]
    fn test_macro_registration() {
        let mut engine = MacroEngine::new();

        let macro_def = Macro::new("test".to_string(), "Test".to_string())
            .add_action(MacroAction::SendText("hello".to_string()));

        assert!(engine.register(macro_def).is_ok());
        assert!(engine.get("test").is_some());
    }

    #[test]
    fn test_duplicate_registration() {
        let mut engine = MacroEngine::new();

        let macro_def = Macro::new("test".to_string(), "Test".to_string())
            .add_action(MacroAction::SendText("hello".to_string()));

        engine.register(macro_def.clone()).unwrap();
        assert!(engine.register(macro_def).is_err());
    }

    #[test]
    fn test_macro_unregistration() {
        let mut engine = MacroEngine::new();

        let macro_def = Macro::new("test".to_string(), "Test".to_string())
            .add_action(MacroAction::SendText("hello".to_string()));

        engine.register(macro_def).unwrap();
        assert!(engine.unregister("test").is_ok());
        assert!(engine.get("test").is_none());
    }

    #[test]
    fn test_trigger_matching() {
        let mut engine = MacroEngine::new();

        let trigger = KeyCombo {
            key: "t".to_string(),
            modifiers: KeyModifiers {
                ctrl: true,
                alt: true,
                shift: false,
                super_: false,
            },
        };

        let macro_def = Macro::new("test".to_string(), "Test".to_string())
            .with_trigger(trigger.clone())
            .add_action(MacroAction::SendText("hello".to_string()));

        engine.register(macro_def).unwrap();

        assert_eq!(engine.match_trigger(&trigger), Some("test"));
    }

    #[test]
    fn test_macro_execution() {
        let mut engine = MacroEngine::new();

        let macro_def = Macro::new("test".to_string(), "Test".to_string())
            .add_action(MacroAction::SendText("hello".to_string()))
            .add_action(MacroAction::Wait(DurationMs(100)));

        engine.register(macro_def).unwrap();

        let actions = engine.execute("test").unwrap();
        assert_eq!(actions.len(), 2);
    }

    #[test]
    fn test_repeat_action() {
        let mut engine = MacroEngine::new();

        let macro_def = Macro::new("test".to_string(), "Test".to_string()).add_action(
            MacroAction::Repeat {
                action: Box::new(MacroAction::SendText("x".to_string())),
                count: 3,
            },
        );

        engine.register(macro_def).unwrap();

        let actions = engine.execute("test").unwrap();
        assert_eq!(actions.len(), 3);
        assert!(matches!(actions[0], MacroAction::SendText(ref s) if s == "x"));
    }

    #[test]
    fn test_call_macro() {
        let mut engine = MacroEngine::new();

        let macro1 = Macro::new("macro1".to_string(), "First macro".to_string())
            .add_action(MacroAction::SendText("hello".to_string()));

        let macro2 = Macro::new("macro2".to_string(), "Second macro".to_string())
            .add_action(MacroAction::CallMacro("macro1".to_string()))
            .add_action(MacroAction::SendText("world".to_string()));

        engine.register(macro1).unwrap();
        engine.register(macro2).unwrap();

        let actions = engine.execute("macro2").unwrap();
        assert_eq!(actions.len(), 2);
    }

    #[test]
    fn test_recursion_limit() {
        let mut engine = MacroEngine::new();
        engine.set_max_recursion_depth(3);

        let macro_def = Macro::new("recursive".to_string(), "Recursive".to_string())
            .add_action(MacroAction::CallMacro("recursive".to_string()));

        engine.register(macro_def).unwrap();

        let result = engine.execute("recursive");
        assert!(matches!(result, Err(MacroError::MaxRecursionDepth)));
    }

    #[test]
    fn test_recording() {
        let mut engine = MacroEngine::new();

        engine.start_recording("test".to_string(), false).unwrap();
        assert!(engine.is_recording());

        engine.record_text("hello".to_string());

        let macro_def = engine
            .stop_recording("Test macro".to_string(), None)
            .unwrap();

        assert_eq!(macro_def.name, "test");
        assert_eq!(macro_def.actions.len(), 1);
    }

    #[test]
    fn test_recording_with_timing() {
        let mut engine = MacroEngine::new();

        engine.start_recording("test".to_string(), true).unwrap();
        engine.record_text("hello".to_string());

        std::thread::sleep(Duration::from_millis(150));
        engine.record_text("world".to_string());

        let macro_def = engine
            .stop_recording("Test macro".to_string(), None)
            .unwrap();

        // Should have: SendText, Wait, SendText
        assert!(macro_def.actions.len() >= 2);
    }

    #[test]
    fn test_cancel_recording() {
        let mut engine = MacroEngine::new();

        engine.start_recording("test".to_string(), false).unwrap();
        engine.record_text("hello".to_string());
        engine.cancel_recording().unwrap();

        assert!(!engine.is_recording());
        assert!(engine.get("test").is_none());
    }

    #[test]
    fn test_disabled_macro() {
        let mut engine = MacroEngine::new();

        let macro_def = Macro::new("test".to_string(), "Test".to_string())
            .add_action(MacroAction::SendText("hello".to_string()))
            .set_enabled(false);

        engine.register(macro_def).unwrap();

        let actions = engine.execute("test").unwrap();
        assert_eq!(actions.len(), 0);
    }

    #[test]
    fn test_builders() {
        use builders::*;

        let actions = vec![
            send_line("ls -la"),
            wait_ms(100),
            send_ctrl_c(),
            repeat(send_text("x"), 5),
            sequence(vec![send_text("a"), send_text("b")]),
        ];

        assert_eq!(actions.len(), 5);
    }

    #[test]
    fn test_key_event_from_modifiers() {
        let key_event = KeyEvent {
            key: "c".to_string(),
            modifiers: KeyModifiers {
                ctrl: true,
                alt: false,
                shift: false,
                super_: false,
            },
        };

        let combo = key_event.to_key_combo();
        assert_eq!(combo.key, "c");
        assert!(combo.modifiers.ctrl);
    }

    #[test]
    fn test_export_import() {
        let mut engine = MacroEngine::new();

        let macro1 = Macro::new("test1".to_string(), "Test 1".to_string())
            .add_action(MacroAction::SendText("hello".to_string()));

        let macro2 = Macro::new("test2".to_string(), "Test 2".to_string())
            .add_action(MacroAction::SendText("world".to_string()));

        engine.register(macro1).unwrap();
        engine.register(macro2).unwrap();

        let exported = engine.export_all();
        assert_eq!(exported.len(), 2);

        let mut new_engine = MacroEngine::new();
        new_engine.load_from_config(exported).unwrap();

        assert!(new_engine.get("test1").is_some());
        assert!(new_engine.get("test2").is_some());
    }

    #[test]
    fn test_sequence_action() {
        let mut engine = MacroEngine::new();

        let macro_def = Macro::new("test".to_string(), "Test".to_string()).add_action(
            MacroAction::Sequence(vec![
                MacroAction::SendText("a".to_string()),
                MacroAction::SendText("b".to_string()),
                MacroAction::SendText("c".to_string()),
            ]),
        );

        engine.register(macro_def).unwrap();

        let actions = engine.execute("test").unwrap();
        assert_eq!(actions.len(), 3);
    }
}
