//! Desktop notification system for AgTerm
//!
//! This module provides desktop notifications for terminal events such as:
//! - Bell events in background tabs
//! - Command completion (future feature)
//! - Custom notifications

use notify_rust::{Notification, Timeout};

// Import NotificationConfig from config module
use crate::config::NotificationConfig;

/// Manages desktop notifications for terminal events
pub struct NotificationManager {
    config: NotificationConfig,
}

impl NotificationManager {
    /// Create a new notification manager with the given configuration
    pub fn new(config: NotificationConfig) -> Self {
        Self { config }
    }

    /// Update notification configuration
    pub fn update_config(&mut self, config: NotificationConfig) {
        self.config = config;
    }

    /// Show notification for terminal bell event
    ///
    /// Only shows if notifications are enabled and on_bell is true.
    /// Typically called for background tabs only.
    pub fn notify_bell(&self, tab_title: &str) {
        if !self.config.enabled || !self.config.on_bell {
            return;
        }

        let result = Notification::new()
            .summary("AgTerm")
            .body(&format!("Bell in: {tab_title}"))
            .timeout(Timeout::Milliseconds(
                (self.config.timeout_seconds * 1000) as u32
            ))
            .show();

        if let Err(e) = result {
            tracing::warn!("Failed to show bell notification: {}", e);
        }
    }

    /// Show notification for command completion (future feature)
    ///
    /// # Arguments
    /// * `command` - The command that completed
    /// * `exit_code` - Exit code of the command
    pub fn notify_command_complete(&self, command: &str, exit_code: i32) {
        if !self.config.enabled || !self.config.on_command_complete {
            return;
        }

        let status = if exit_code == 0 { "✓" } else { "✗" };
        let summary = format!("{status} Command Complete");

        let result = Notification::new()
            .summary(&summary)
            .body(command)
            .timeout(Timeout::Milliseconds(
                (self.config.timeout_seconds * 1000) as u32
            ))
            .show();

        if let Err(e) = result {
            tracing::warn!("Failed to show command completion notification: {}", e);
        }
    }

    /// Show custom notification
    ///
    /// # Arguments
    /// * `title` - Notification title
    /// * `body` - Notification body text
    pub fn notify_custom(&self, title: &str, body: &str) {
        if !self.config.enabled {
            return;
        }

        let result = Notification::new()
            .summary(title)
            .body(body)
            .timeout(Timeout::Milliseconds(
                (self.config.timeout_seconds * 1000) as u32
            ))
            .show();

        if let Err(e) = result {
            tracing::warn!("Failed to show custom notification: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NotificationConfig;

    #[test]
    fn test_notification_config_default() {
        let config = NotificationConfig::default();
        assert!(config.enabled);
        assert!(config.on_bell);
        assert!(!config.on_command_complete);
        assert_eq!(config.timeout_seconds, 5);
    }

    #[test]
    fn test_notification_manager_creation() {
        let config = NotificationConfig::default();
        let manager = NotificationManager::new(config.clone());
        assert_eq!(manager.config.enabled, config.enabled);
    }

    #[test]
    fn test_notification_config_update() {
        let config = NotificationConfig::default();
        let mut manager = NotificationManager::new(config);

        let new_config = NotificationConfig {
            enabled: false,
            on_bell: false,
            on_command_complete: true,
            timeout_seconds: 10,
        };

        manager.update_config(new_config.clone());
        assert_eq!(manager.config.enabled, false);
        assert_eq!(manager.config.timeout_seconds, 10);
    }

    #[test]
    fn test_disabled_notifications() {
        let config = NotificationConfig {
            enabled: false,
            ..Default::default()
        };
        let manager = NotificationManager::new(config);

        // These should not panic even if notifications are disabled
        manager.notify_bell("Test Tab");
        manager.notify_command_complete("ls", 0);
        manager.notify_custom("Test", "Body");
    }

    #[test]
    fn test_bell_disabled() {
        let config = NotificationConfig {
            enabled: true,
            on_bell: false,
            ..Default::default()
        };
        let manager = NotificationManager::new(config);

        // Should not show notification when on_bell is false
        manager.notify_bell("Test Tab");
    }

    #[test]
    fn test_command_complete_disabled() {
        let config = NotificationConfig {
            enabled: true,
            on_command_complete: false,
            ..Default::default()
        };
        let manager = NotificationManager::new(config);

        // Should not show notification when on_command_complete is false
        manager.notify_command_complete("ls", 0);
    }
}
