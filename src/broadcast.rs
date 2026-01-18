//! Terminal broadcast functionality for AgTerm
//!
//! Provides the ability to broadcast input to multiple terminal sessions simultaneously.
//! This is useful for executing commands across multiple servers or terminals in parallel.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during broadcast operations
#[derive(Debug, Error)]
pub enum BroadcastError {
    #[error("Broadcast group not found: {0}")]
    GroupNotFound(String),

    #[error("Terminal not found: {0}")]
    TerminalNotFound(Uuid),

    #[error("Group already exists: {0}")]
    GroupAlreadyExists(String),

    #[error("Terminal already in group: {terminal_id} in {group_name}")]
    TerminalAlreadyInGroup {
        terminal_id: Uuid,
        group_name: String,
    },

    #[error("Terminal not in group: {terminal_id} in {group_name}")]
    TerminalNotInGroup {
        terminal_id: Uuid,
        group_name: String,
    },

    #[error("Cannot remove last terminal from active broadcast group")]
    CannotRemoveLastTerminal,

    #[error("Invalid group name: {0}")]
    InvalidGroupName(String),
}

/// Broadcast mode determines how input is sent to terminals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BroadcastMode {
    /// All input is broadcast to group members
    Full,
    /// Only specific key combinations trigger broadcasts
    Selective,
    /// Broadcast is disabled
    Disabled,
}

/// Key modifier for selective broadcast triggers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BroadcastTrigger {
    /// Require Ctrl key
    pub ctrl: bool,
    /// Require Alt key
    pub alt: bool,
    /// Require Shift key
    pub shift: bool,
    /// Require Command/Super key
    pub command: bool,
}

impl BroadcastTrigger {
    /// Create a new trigger with Ctrl+Shift (default for selective mode)
    pub fn default_selective() -> Self {
        Self {
            ctrl: true,
            alt: false,
            shift: true,
            command: false,
        }
    }

    /// Check if the given modifiers match this trigger
    pub fn matches(&self, ctrl: bool, alt: bool, shift: bool, command: bool) -> bool {
        self.ctrl == ctrl && self.alt == alt && self.shift == shift && self.command == command
    }
}

/// A group of terminals that can receive broadcast input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastGroup {
    /// Unique name for this group
    name: String,
    /// Set of terminal IDs that are members of this group
    members: HashSet<Uuid>,
    /// Whether this group is currently active for broadcasting
    active: bool,
    /// Broadcast mode for this group
    mode: BroadcastMode,
    /// Trigger configuration for selective mode
    trigger: BroadcastTrigger,
    /// Optional description of the group's purpose
    description: Option<String>,
}

impl BroadcastGroup {
    /// Create a new broadcast group
    pub fn new(name: String) -> Self {
        Self {
            name,
            members: HashSet::new(),
            active: false,
            mode: BroadcastMode::Full,
            trigger: BroadcastTrigger::default_selective(),
            description: None,
        }
    }

    /// Get the group name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the set of member terminal IDs
    pub fn members(&self) -> &HashSet<Uuid> {
        &self.members
    }

    /// Check if the group is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get the broadcast mode
    pub fn mode(&self) -> BroadcastMode {
        self.mode
    }

    /// Set the broadcast mode
    pub fn set_mode(&mut self, mode: BroadcastMode) {
        self.mode = mode;
    }

    /// Get the trigger configuration
    pub fn trigger(&self) -> &BroadcastTrigger {
        &self.trigger
    }

    /// Set the trigger configuration
    pub fn set_trigger(&mut self, trigger: BroadcastTrigger) {
        self.trigger = trigger;
    }

    /// Set or clear the description
    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
    }

    /// Get the description
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Add a terminal to this group
    pub fn add_terminal(&mut self, terminal_id: Uuid) -> Result<(), BroadcastError> {
        if self.members.contains(&terminal_id) {
            return Err(BroadcastError::TerminalAlreadyInGroup {
                terminal_id,
                group_name: self.name.clone(),
            });
        }
        self.members.insert(terminal_id);
        Ok(())
    }

    /// Remove a terminal from this group
    pub fn remove_terminal(&mut self, terminal_id: &Uuid) -> Result<(), BroadcastError> {
        if !self.members.contains(terminal_id) {
            return Err(BroadcastError::TerminalNotInGroup {
                terminal_id: *terminal_id,
                group_name: self.name.clone(),
            });
        }

        // Prevent removing the last terminal from an active group
        if self.active && self.members.len() == 1 {
            return Err(BroadcastError::CannotRemoveLastTerminal);
        }

        self.members.remove(terminal_id);
        Ok(())
    }

    /// Check if a terminal is a member of this group
    pub fn contains(&self, terminal_id: &Uuid) -> bool {
        self.members.contains(terminal_id)
    }

    /// Get the number of members
    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    /// Activate this group for broadcasting
    pub fn activate(&mut self) {
        self.active = true;
    }

    /// Deactivate this group
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Check if input should be broadcast based on mode and modifiers
    pub fn should_broadcast(&self, ctrl: bool, alt: bool, shift: bool, command: bool) -> bool {
        if !self.active {
            return false;
        }

        match self.mode {
            BroadcastMode::Full => true,
            BroadcastMode::Selective => self.trigger.matches(ctrl, alt, shift, command),
            BroadcastMode::Disabled => false,
        }
    }
}

/// Manager for broadcast groups and operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastManager {
    /// Map of group names to broadcast groups
    groups: HashMap<String, BroadcastGroup>,
    /// Currently active group (if any)
    active_group: Option<String>,
    /// Set of all known terminal IDs for validation
    known_terminals: HashSet<Uuid>,
}

impl BroadcastManager {
    /// Create a new broadcast manager
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
            active_group: None,
            known_terminals: HashSet::new(),
        }
    }

    /// Register a terminal with the manager
    pub fn register_terminal(&mut self, terminal_id: Uuid) {
        self.known_terminals.insert(terminal_id);
    }

    /// Unregister a terminal (removes it from all groups)
    pub fn unregister_terminal(&mut self, terminal_id: &Uuid) {
        self.known_terminals.remove(terminal_id);

        // Remove from all groups
        for group in self.groups.values_mut() {
            let _ = group.remove_terminal(terminal_id);
        }
    }

    /// Check if a terminal is registered
    pub fn is_terminal_registered(&self, terminal_id: &Uuid) -> bool {
        self.known_terminals.contains(terminal_id)
    }

    /// Create a new broadcast group
    pub fn create_group(&mut self, name: String) -> Result<(), BroadcastError> {
        if name.is_empty() || name.len() > 64 {
            return Err(BroadcastError::InvalidGroupName(name));
        }

        if self.groups.contains_key(&name) {
            return Err(BroadcastError::GroupAlreadyExists(name));
        }

        self.groups.insert(name.clone(), BroadcastGroup::new(name));
        Ok(())
    }

    /// Delete a broadcast group
    pub fn delete_group(&mut self, name: &str) -> Result<(), BroadcastError> {
        if !self.groups.contains_key(name) {
            return Err(BroadcastError::GroupNotFound(name.to_string()));
        }

        // Deactivate if this is the active group
        if self.active_group.as_deref() == Some(name) {
            self.active_group = None;
        }

        self.groups.remove(name);
        Ok(())
    }

    /// Get a reference to a group
    pub fn get_group(&self, name: &str) -> Result<&BroadcastGroup, BroadcastError> {
        self.groups
            .get(name)
            .ok_or_else(|| BroadcastError::GroupNotFound(name.to_string()))
    }

    /// Get a mutable reference to a group
    pub fn get_group_mut(&mut self, name: &str) -> Result<&mut BroadcastGroup, BroadcastError> {
        self.groups
            .get_mut(name)
            .ok_or_else(|| BroadcastError::GroupNotFound(name.to_string()))
    }

    /// List all group names
    pub fn list_groups(&self) -> Vec<&str> {
        self.groups.keys().map(|s| s.as_str()).collect()
    }

    /// Add a terminal to a group
    pub fn add_to_group(
        &mut self,
        group_name: &str,
        terminal_id: Uuid,
    ) -> Result<(), BroadcastError> {
        if !self.is_terminal_registered(&terminal_id) {
            return Err(BroadcastError::TerminalNotFound(terminal_id));
        }

        let group = self.get_group_mut(group_name)?;
        group.add_terminal(terminal_id)
    }

    /// Remove a terminal from a group
    pub fn remove_from_group(
        &mut self,
        group_name: &str,
        terminal_id: &Uuid,
    ) -> Result<(), BroadcastError> {
        let group = self.get_group_mut(group_name)?;
        group.remove_terminal(terminal_id)
    }

    /// Activate a group for broadcasting
    pub fn activate_group(&mut self, name: &str) -> Result<(), BroadcastError> {
        // Deactivate current group if any
        if let Some(active_name) = self.active_group.take() {
            if let Some(group) = self.groups.get_mut(&active_name) {
                group.deactivate();
            }
        }

        // Activate new group
        let group = self.get_group_mut(name)?;
        group.activate();
        self.active_group = Some(name.to_string());
        Ok(())
    }

    /// Deactivate the current broadcast group
    pub fn deactivate_current(&mut self) -> Result<(), BroadcastError> {
        if let Some(active_name) = self.active_group.take() {
            let group = self.get_group_mut(&active_name)?;
            group.deactivate();
        }
        Ok(())
    }

    /// Get the currently active group name
    pub fn active_group_name(&self) -> Option<&str> {
        self.active_group.as_deref()
    }

    /// Get the currently active group
    pub fn active_group(&self) -> Option<&BroadcastGroup> {
        self.active_group
            .as_ref()
            .and_then(|name| self.groups.get(name))
    }

    /// Get broadcast targets for the given input context
    ///
    /// Returns a list of terminal IDs that should receive the input,
    /// or None if no broadcasting should occur.
    pub fn get_broadcast_targets(
        &self,
        source_terminal: &Uuid,
        ctrl: bool,
        alt: bool,
        shift: bool,
        command: bool,
    ) -> Option<Vec<Uuid>> {
        let group = self.active_group()?;

        // Check if source terminal is in the active group
        if !group.contains(source_terminal) {
            return None;
        }

        // Check if input should be broadcast
        if !group.should_broadcast(ctrl, alt, shift, command) {
            return None;
        }

        // Return all members except the source terminal
        Some(
            group
                .members()
                .iter()
                .filter(|id| *id != source_terminal)
                .copied()
                .collect(),
        )
    }

    /// Get statistics about broadcast groups
    pub fn stats(&self) -> BroadcastStats {
        BroadcastStats {
            total_groups: self.groups.len(),
            active_groups: self.groups.values().filter(|g| g.is_active()).count(),
            total_terminals: self.known_terminals.len(),
            terminals_in_groups: self
                .groups
                .values()
                .flat_map(|g| g.members())
                .collect::<HashSet<_>>()
                .len(),
        }
    }

    /// Find which groups contain a specific terminal
    pub fn find_groups_for_terminal(&self, terminal_id: &Uuid) -> Vec<&str> {
        self.groups
            .iter()
            .filter_map(|(name, group)| {
                if group.contains(terminal_id) {
                    Some(name.as_str())
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for BroadcastManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about broadcast groups
#[derive(Debug, Clone, Copy)]
pub struct BroadcastStats {
    /// Total number of groups
    pub total_groups: usize,
    /// Number of active groups
    pub active_groups: usize,
    /// Total registered terminals
    pub total_terminals: usize,
    /// Terminals that are members of at least one group
    pub terminals_in_groups: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broadcast_group_creation() {
        let group = BroadcastGroup::new("test-group".to_string());
        assert_eq!(group.name(), "test-group");
        assert_eq!(group.member_count(), 0);
        assert!(!group.is_active());
        assert_eq!(group.mode(), BroadcastMode::Full);
    }

    #[test]
    fn test_add_remove_terminals() {
        let mut group = BroadcastGroup::new("test".to_string());
        let term1 = Uuid::new_v4();
        let term2 = Uuid::new_v4();

        // Add terminals
        assert!(group.add_terminal(term1).is_ok());
        assert!(group.add_terminal(term2).is_ok());
        assert_eq!(group.member_count(), 2);

        // Try to add duplicate
        assert!(group.add_terminal(term1).is_err());

        // Remove terminal
        assert!(group.remove_terminal(&term1).is_ok());
        assert_eq!(group.member_count(), 1);
        assert!(!group.contains(&term1));
        assert!(group.contains(&term2));

        // Try to remove non-existent terminal
        assert!(group.remove_terminal(&term1).is_err());
    }

    #[test]
    fn test_cannot_remove_last_terminal_from_active_group() {
        let mut group = BroadcastGroup::new("test".to_string());
        let term1 = Uuid::new_v4();

        group.add_terminal(term1).unwrap();
        group.activate();

        // Should fail to remove the last terminal
        assert!(matches!(
            group.remove_terminal(&term1),
            Err(BroadcastError::CannotRemoveLastTerminal)
        ));

        // Should succeed after deactivating
        group.deactivate();
        assert!(group.remove_terminal(&term1).is_ok());
    }

    #[test]
    fn test_broadcast_modes() {
        let mut group = BroadcastGroup::new("test".to_string());
        group.activate();

        // Full mode - should always broadcast
        group.set_mode(BroadcastMode::Full);
        assert!(group.should_broadcast(false, false, false, false));
        assert!(group.should_broadcast(true, false, false, false));

        // Disabled mode - should never broadcast
        group.set_mode(BroadcastMode::Disabled);
        assert!(!group.should_broadcast(false, false, false, false));
        assert!(!group.should_broadcast(true, true, true, true));

        // Selective mode - should only broadcast with matching trigger
        group.set_mode(BroadcastMode::Selective);
        let trigger = BroadcastTrigger::default_selective(); // Ctrl+Shift
        group.set_trigger(trigger);

        assert!(!group.should_broadcast(false, false, false, false));
        assert!(!group.should_broadcast(true, false, false, false));
        assert!(group.should_broadcast(true, false, true, false)); // Ctrl+Shift
    }

    #[test]
    fn test_broadcast_manager_creation() {
        let mut manager = BroadcastManager::new();

        assert!(manager.create_group("group1".to_string()).is_ok());
        assert!(manager.create_group("group2".to_string()).is_ok());

        // Duplicate name should fail
        assert!(matches!(
            manager.create_group("group1".to_string()),
            Err(BroadcastError::GroupAlreadyExists(_))
        ));

        // Invalid names should fail
        assert!(matches!(
            manager.create_group("".to_string()),
            Err(BroadcastError::InvalidGroupName(_))
        ));
        assert!(matches!(
            manager.create_group("a".repeat(65)),
            Err(BroadcastError::InvalidGroupName(_))
        ));
    }

    #[test]
    fn test_broadcast_manager_terminal_registration() {
        let mut manager = BroadcastManager::new();
        let term1 = Uuid::new_v4();
        let term2 = Uuid::new_v4();

        manager.register_terminal(term1);
        manager.register_terminal(term2);

        assert!(manager.is_terminal_registered(&term1));
        assert!(manager.is_terminal_registered(&term2));

        manager.unregister_terminal(&term1);
        assert!(!manager.is_terminal_registered(&term1));
        assert!(manager.is_terminal_registered(&term2));
    }

    #[test]
    fn test_broadcast_manager_group_operations() {
        let mut manager = BroadcastManager::new();
        let term1 = Uuid::new_v4();
        let term2 = Uuid::new_v4();
        let term3 = Uuid::new_v4();

        manager.register_terminal(term1);
        manager.register_terminal(term2);
        manager.register_terminal(term3);

        manager.create_group("servers".to_string()).unwrap();
        manager.create_group("local".to_string()).unwrap();

        // Add terminals to groups
        manager.add_to_group("servers", term1).unwrap();
        manager.add_to_group("servers", term2).unwrap();
        manager.add_to_group("local", term3).unwrap();

        // Check membership
        let servers = manager.get_group("servers").unwrap();
        assert_eq!(servers.member_count(), 2);
        assert!(servers.contains(&term1));
        assert!(servers.contains(&term2));

        let local = manager.get_group("local").unwrap();
        assert_eq!(local.member_count(), 1);
        assert!(local.contains(&term3));

        // Remove terminal
        manager.remove_from_group("servers", &term1).unwrap();
        let servers = manager.get_group("servers").unwrap();
        assert_eq!(servers.member_count(), 1);
        assert!(!servers.contains(&term1));
    }

    #[test]
    fn test_broadcast_manager_activation() {
        let mut manager = BroadcastManager::new();
        let term1 = Uuid::new_v4();

        manager.register_terminal(term1);
        manager.create_group("group1".to_string()).unwrap();
        manager.create_group("group2".to_string()).unwrap();
        manager.add_to_group("group1", term1).unwrap();

        // Activate group1
        manager.activate_group("group1").unwrap();
        assert_eq!(manager.active_group_name(), Some("group1"));
        assert!(manager.get_group("group1").unwrap().is_active());
        assert!(!manager.get_group("group2").unwrap().is_active());

        // Switch to group2
        manager.activate_group("group2").unwrap();
        assert_eq!(manager.active_group_name(), Some("group2"));
        assert!(!manager.get_group("group1").unwrap().is_active());
        assert!(manager.get_group("group2").unwrap().is_active());

        // Deactivate
        manager.deactivate_current().unwrap();
        assert_eq!(manager.active_group_name(), None);
        assert!(!manager.get_group("group1").unwrap().is_active());
        assert!(!manager.get_group("group2").unwrap().is_active());
    }

    #[test]
    fn test_broadcast_targets() {
        let mut manager = BroadcastManager::new();
        let term1 = Uuid::new_v4();
        let term2 = Uuid::new_v4();
        let term3 = Uuid::new_v4();

        manager.register_terminal(term1);
        manager.register_terminal(term2);
        manager.register_terminal(term3);

        manager.create_group("broadcast".to_string()).unwrap();
        manager.add_to_group("broadcast", term1).unwrap();
        manager.add_to_group("broadcast", term2).unwrap();
        manager.add_to_group("broadcast", term3).unwrap();
        manager.activate_group("broadcast").unwrap();

        // Get broadcast targets from term1 (Full mode)
        let targets = manager
            .get_broadcast_targets(&term1, false, false, false, false)
            .unwrap();
        assert_eq!(targets.len(), 2);
        assert!(targets.contains(&term2));
        assert!(targets.contains(&term3));
        assert!(!targets.contains(&term1)); // Source not included

        // Change to selective mode
        manager
            .get_group_mut("broadcast")
            .unwrap()
            .set_mode(BroadcastMode::Selective);

        // Without matching trigger, should return None
        assert!(manager
            .get_broadcast_targets(&term1, false, false, false, false)
            .is_none());

        // With matching trigger (Ctrl+Shift), should return targets
        let targets = manager
            .get_broadcast_targets(&term1, true, false, true, false)
            .unwrap();
        assert_eq!(targets.len(), 2);
    }

    #[test]
    fn test_unregister_terminal_removes_from_groups() {
        let mut manager = BroadcastManager::new();
        let term1 = Uuid::new_v4();

        manager.register_terminal(term1);
        manager.create_group("test".to_string()).unwrap();
        manager.add_to_group("test", term1).unwrap();

        assert_eq!(manager.get_group("test").unwrap().member_count(), 1);

        // Unregister should remove from all groups
        manager.unregister_terminal(&term1);
        assert_eq!(manager.get_group("test").unwrap().member_count(), 0);
        assert!(!manager.is_terminal_registered(&term1));
    }

    #[test]
    fn test_find_groups_for_terminal() {
        let mut manager = BroadcastManager::new();
        let term1 = Uuid::new_v4();

        manager.register_terminal(term1);
        manager.create_group("group1".to_string()).unwrap();
        manager.create_group("group2".to_string()).unwrap();
        manager.create_group("group3".to_string()).unwrap();

        manager.add_to_group("group1", term1).unwrap();
        manager.add_to_group("group3", term1).unwrap();

        let groups = manager.find_groups_for_terminal(&term1);
        assert_eq!(groups.len(), 2);
        assert!(groups.contains(&"group1"));
        assert!(groups.contains(&"group3"));
        assert!(!groups.contains(&"group2"));
    }

    #[test]
    fn test_broadcast_stats() {
        let mut manager = BroadcastManager::new();
        let term1 = Uuid::new_v4();
        let term2 = Uuid::new_v4();
        let term3 = Uuid::new_v4();

        manager.register_terminal(term1);
        manager.register_terminal(term2);
        manager.register_terminal(term3);

        manager.create_group("group1".to_string()).unwrap();
        manager.create_group("group2".to_string()).unwrap();

        manager.add_to_group("group1", term1).unwrap();
        manager.add_to_group("group1", term2).unwrap();
        manager.add_to_group("group2", term2).unwrap();
        // term3 is registered but not in any group

        manager.activate_group("group1").unwrap();

        let stats = manager.stats();
        assert_eq!(stats.total_groups, 2);
        assert_eq!(stats.active_groups, 1);
        assert_eq!(stats.total_terminals, 3);
        assert_eq!(stats.terminals_in_groups, 2); // term1 and term2
    }

    #[test]
    fn test_delete_group() {
        let mut manager = BroadcastManager::new();
        let term1 = Uuid::new_v4();

        manager.register_terminal(term1);
        manager.create_group("temp".to_string()).unwrap();
        manager.add_to_group("temp", term1).unwrap();
        manager.activate_group("temp").unwrap();

        // Delete active group should deactivate it
        manager.delete_group("temp").unwrap();
        assert!(manager.active_group_name().is_none());
        assert!(manager.get_group("temp").is_err());

        // Deleting non-existent group should fail
        assert!(matches!(
            manager.delete_group("nonexistent"),
            Err(BroadcastError::GroupNotFound(_))
        ));
    }

    #[test]
    fn test_broadcast_trigger_matching() {
        let trigger = BroadcastTrigger {
            ctrl: true,
            alt: false,
            shift: true,
            command: false,
        };

        // Exact match
        assert!(trigger.matches(true, false, true, false));

        // Mismatches
        assert!(!trigger.matches(false, false, true, false)); // No Ctrl
        assert!(!trigger.matches(true, false, false, false)); // No Shift
        assert!(!trigger.matches(true, true, true, false)); // Extra Alt
        assert!(!trigger.matches(true, false, true, true)); // Extra Command
    }

    #[test]
    fn test_group_description() {
        let mut group = BroadcastGroup::new("test".to_string());
        assert!(group.description().is_none());

        group.set_description(Some("Production servers".to_string()));
        assert_eq!(group.description(), Some("Production servers"));

        group.set_description(None);
        assert!(group.description().is_none());
    }
}
