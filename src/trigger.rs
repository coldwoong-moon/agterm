//! Output Trigger System
//!
//! This module implements a pattern-matching trigger system for terminal output.
//! Triggers can respond to specific patterns in terminal output with actions like
//! notifications, highlighting, sounds, or custom commands.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a single trigger rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    /// Unique name for this trigger
    pub name: String,
    /// Regular expression pattern to match against terminal output
    pub pattern: String,
    /// Action to perform when pattern matches
    pub action: TriggerAction,
    /// Whether this trigger is currently enabled
    pub enabled: bool,
}

/// Action to perform when a trigger matches
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TriggerAction {
    /// Show a desktop notification
    Notify {
        title: String,
        body: String,
    },
    /// Highlight the matched text with a color
    Highlight {
        color: String, // Hex color like "#FF0000"
    },
    /// Play a sound file
    PlaySound {
        file: Option<String>, // None = use default bell sound
    },
    /// Execute a shell command (future feature)
    RunCommand {
        command: String,
    },
    /// Log a message (for debugging/auditing)
    Log {
        message: String,
    },
}

/// Manages all triggers and handles pattern matching
pub struct TriggerManager {
    /// List of configured triggers
    triggers: Vec<Trigger>,
    /// Compiled regex patterns with their trigger indices
    /// Using Vec for stable ordering, could be optimized with RegexSet for large numbers
    compiled: Vec<(usize, Regex)>,
}

impl TriggerManager {
    /// Create a new empty trigger manager
    pub fn new() -> Self {
        Self {
            triggers: Vec::new(),
            compiled: Vec::new(),
        }
    }

    /// Add a new trigger
    ///
    /// Returns an error if the pattern is invalid regex
    pub fn add(&mut self, trigger: Trigger) -> Result<(), String> {
        // Validate regex pattern
        let regex = Regex::new(&trigger.pattern)
            .map_err(|e| format!("Invalid regex pattern '{}': {}", trigger.pattern, e))?;

        // Add trigger
        let index = self.triggers.len();
        self.triggers.push(trigger);
        self.compiled.push((index, regex));

        Ok(())
    }

    /// Check text against all enabled triggers
    ///
    /// Returns a list of (trigger_index, trigger) pairs for all matches
    pub fn check(&self, text: &str) -> Vec<(usize, &Trigger)> {
        let mut matches = Vec::new();

        for (index, regex) in &self.compiled {
            let trigger = &self.triggers[*index];

            // Skip disabled triggers
            if !trigger.enabled {
                continue;
            }

            // Check if pattern matches
            if regex.is_match(text) {
                matches.push((*index, trigger));
            }
        }

        matches
    }

    /// Enable or disable a trigger by name
    ///
    /// Returns true if trigger was found and updated
    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> bool {
        for trigger in &mut self.triggers {
            if trigger.name == name {
                trigger.enabled = enabled;
                return true;
            }
        }
        false
    }

    /// Remove a trigger by name
    ///
    /// Returns true if trigger was found and removed
    pub fn remove(&mut self, name: &str) -> bool {
        if let Some(pos) = self.triggers.iter().position(|t| t.name == name) {
            self.triggers.remove(pos);
            // Rebuild compiled list
            self.rebuild_compiled();
            return true;
        }
        false
    }

    /// Get all triggers
    pub fn triggers(&self) -> &[Trigger] {
        &self.triggers
    }

    /// Get a trigger by name
    pub fn get(&self, name: &str) -> Option<&Trigger> {
        self.triggers.iter().find(|t| t.name == name)
    }

    /// Rebuild compiled regex list (internal helper)
    fn rebuild_compiled(&mut self) {
        self.compiled.clear();
        for (index, trigger) in self.triggers.iter().enumerate() {
            if let Ok(regex) = Regex::new(&trigger.pattern) {
                self.compiled.push((index, regex));
            } else {
                tracing::warn!("Failed to compile regex for trigger '{}': {}",
                              trigger.name, trigger.pattern);
            }
        }
    }

    /// Create trigger manager from configuration
    pub fn from_config(triggers: &[TriggerConfig]) -> Self {
        let mut manager = Self::new();

        for config in triggers {
            // Convert TriggerConfig to Trigger
            let action = match config.action.as_str() {
                "notify" => {
                    let title = config.params.get("title")
                        .cloned()
                        .unwrap_or_else(|| "AgTerm Alert".to_string());
                    let body = config.params.get("body")
                        .cloned()
                        .unwrap_or_else(|| "Pattern matched in terminal".to_string());
                    TriggerAction::Notify { title, body }
                }
                "highlight" => {
                    let color = config.params.get("color")
                        .cloned()
                        .unwrap_or_else(|| "#FFFF00".to_string());
                    TriggerAction::Highlight { color }
                }
                "sound" => {
                    let file = config.params.get("file").cloned();
                    TriggerAction::PlaySound { file }
                }
                "command" => {
                    let command = config.params.get("command")
                        .cloned()
                        .unwrap_or_default();
                    TriggerAction::RunCommand { command }
                }
                "log" => {
                    let message = config.params.get("message")
                        .cloned()
                        .unwrap_or_else(|| "Trigger matched".to_string());
                    TriggerAction::Log { message }
                }
                _ => {
                    tracing::warn!("Unknown trigger action '{}' for trigger '{}'",
                                  config.action, config.name);
                    continue;
                }
            };

            let trigger = Trigger {
                name: config.name.clone(),
                pattern: config.pattern.clone(),
                action,
                enabled: config.enabled,
            };

            if let Err(e) = manager.add(trigger) {
                tracing::error!("Failed to add trigger '{}': {}", config.name, e);
            } else {
                tracing::info!("Loaded trigger: {}", config.name);
            }
        }

        manager
    }
}

impl Default for TriggerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration format for triggers (used in TOML config)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerConfig {
    /// Trigger name
    pub name: String,
    /// Regex pattern
    pub pattern: String,
    /// Action type: "notify", "highlight", "sound", "command", "log"
    pub action: String,
    /// Action parameters (depends on action type)
    pub params: HashMap<String, String>,
    /// Whether trigger is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_add_and_check() {
        let mut manager = TriggerManager::new();

        let trigger = Trigger {
            name: "error_detection".to_string(),
            pattern: r"(?i)error|fail".to_string(),
            action: TriggerAction::Notify {
                title: "Error Detected".to_string(),
                body: "An error was found in the output".to_string(),
            },
            enabled: true,
        };

        assert!(manager.add(trigger).is_ok());

        // Test matching
        let matches = manager.check("Something went wrong: Error 404");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].1.name, "error_detection");

        // Test case insensitive
        let matches = manager.check("Build FAILED");
        assert_eq!(matches.len(), 1);

        // Test non-matching
        let matches = manager.check("All good");
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_trigger_enable_disable() {
        let mut manager = TriggerManager::new();

        let trigger = Trigger {
            name: "test_trigger".to_string(),
            pattern: "test".to_string(),
            action: TriggerAction::Log {
                message: "test".to_string(),
            },
            enabled: true,
        };

        manager.add(trigger).unwrap();

        // Enabled - should match
        assert_eq!(manager.check("test string").len(), 1);

        // Disable
        assert!(manager.set_enabled("test_trigger", false));
        assert_eq!(manager.check("test string").len(), 0);

        // Re-enable
        assert!(manager.set_enabled("test_trigger", true));
        assert_eq!(manager.check("test string").len(), 1);
    }

    #[test]
    fn test_trigger_remove() {
        let mut manager = TriggerManager::new();

        let trigger = Trigger {
            name: "remove_me".to_string(),
            pattern: "test".to_string(),
            action: TriggerAction::Log {
                message: "test".to_string(),
            },
            enabled: true,
        };

        manager.add(trigger).unwrap();
        assert_eq!(manager.triggers().len(), 1);

        assert!(manager.remove("remove_me"));
        assert_eq!(manager.triggers().len(), 0);

        // Try removing non-existent trigger
        assert!(!manager.remove("nonexistent"));
    }

    #[test]
    fn test_invalid_regex() {
        let mut manager = TriggerManager::new();

        let trigger = Trigger {
            name: "bad_regex".to_string(),
            pattern: "[invalid(".to_string(), // Invalid regex
            action: TriggerAction::Log {
                message: "test".to_string(),
            },
            enabled: true,
        };

        assert!(manager.add(trigger).is_err());
    }

    #[test]
    fn test_multiple_triggers() {
        let mut manager = TriggerManager::new();

        manager.add(Trigger {
            name: "error".to_string(),
            pattern: "error".to_string(),
            action: TriggerAction::Log { message: "error".to_string() },
            enabled: true,
        }).unwrap();

        manager.add(Trigger {
            name: "warning".to_string(),
            pattern: "warning".to_string(),
            action: TriggerAction::Log { message: "warning".to_string() },
            enabled: true,
        }).unwrap();

        // Text with both patterns
        let matches = manager.check("error and warning detected");
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_from_config() {
        let configs = vec![
            TriggerConfig {
                name: "notify_test".to_string(),
                pattern: "test".to_string(),
                action: "notify".to_string(),
                params: {
                    let mut map = HashMap::new();
                    map.insert("title".to_string(), "Test".to_string());
                    map.insert("body".to_string(), "Test body".to_string());
                    map
                },
                enabled: true,
            },
            TriggerConfig {
                name: "log_test".to_string(),
                pattern: "log".to_string(),
                action: "log".to_string(),
                params: {
                    let mut map = HashMap::new();
                    map.insert("message".to_string(), "Log message".to_string());
                    map
                },
                enabled: true,
            },
        ];

        let manager = TriggerManager::from_config(&configs);
        assert_eq!(manager.triggers().len(), 2);

        let matches = manager.check("test");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].1.name, "notify_test");
    }
}
