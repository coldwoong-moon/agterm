//! Integration tests for the broadcast module
//!
//! These tests verify the broadcast functionality works correctly
//! in real-world scenarios.

use agterm::broadcast::{BroadcastError, BroadcastManager, BroadcastMode, BroadcastTrigger};
use uuid::Uuid;

#[test]
fn test_basic_workflow() {
    let mut manager = BroadcastManager::new();

    // Create terminals
    let term1 = Uuid::new_v4();
    let term2 = Uuid::new_v4();
    let term3 = Uuid::new_v4();

    manager.register_terminal(term1);
    manager.register_terminal(term2);
    manager.register_terminal(term3);

    // Create group
    manager.create_group("servers".to_string()).unwrap();
    manager.add_to_group("servers", term1).unwrap();
    manager.add_to_group("servers", term2).unwrap();
    manager.add_to_group("servers", term3).unwrap();

    // Activate group
    manager.activate_group("servers").unwrap();

    // Test broadcasting
    let targets = manager
        .get_broadcast_targets(&term1, false, false, false, false)
        .unwrap();

    assert_eq!(targets.len(), 2);
    assert!(!targets.contains(&term1));
    assert!(targets.contains(&term2));
    assert!(targets.contains(&term3));
}

#[test]
fn test_multiple_groups() {
    let mut manager = BroadcastManager::new();

    let prod1 = Uuid::new_v4();
    let prod2 = Uuid::new_v4();
    let staging1 = Uuid::new_v4();

    manager.register_terminal(prod1);
    manager.register_terminal(prod2);
    manager.register_terminal(staging1);

    // Create production group
    manager.create_group("production".to_string()).unwrap();
    manager.add_to_group("production", prod1).unwrap();
    manager.add_to_group("production", prod2).unwrap();

    // Create staging group
    manager.create_group("staging".to_string()).unwrap();
    manager.add_to_group("staging", staging1).unwrap();

    // Activate production
    manager.activate_group("production").unwrap();

    // Production broadcast works
    let targets = manager
        .get_broadcast_targets(&prod1, false, false, false, false)
        .unwrap();
    assert_eq!(targets.len(), 1);
    assert!(targets.contains(&prod2));

    // Staging terminal doesn't broadcast (not in active group)
    let targets = manager.get_broadcast_targets(&staging1, false, false, false, false);
    assert!(targets.is_none());

    // Switch to staging
    manager.activate_group("staging").unwrap();

    // Production no longer broadcasts
    let targets = manager.get_broadcast_targets(&prod1, false, false, false, false);
    assert!(targets.is_none());

    // Staging terminal now broadcasts (but to no one since it's alone)
    let targets = manager
        .get_broadcast_targets(&staging1, false, false, false, false)
        .unwrap();
    assert_eq!(targets.len(), 0);
}

#[test]
fn test_selective_mode() {
    let mut manager = BroadcastManager::new();

    let term1 = Uuid::new_v4();
    let term2 = Uuid::new_v4();

    manager.register_terminal(term1);
    manager.register_terminal(term2);

    manager.create_group("test".to_string()).unwrap();
    manager.add_to_group("test", term1).unwrap();
    manager.add_to_group("test", term2).unwrap();

    // Set selective mode
    {
        let group = manager.get_group_mut("test").unwrap();
        group.set_mode(BroadcastMode::Selective);
        group.set_trigger(BroadcastTrigger {
            ctrl: true,
            alt: false,
            shift: true,
            command: false,
        });
    }

    manager.activate_group("test").unwrap();

    // Regular input - no broadcast
    assert!(manager
        .get_broadcast_targets(&term1, false, false, false, false)
        .is_none());

    // Ctrl only - no broadcast
    assert!(manager
        .get_broadcast_targets(&term1, true, false, false, false)
        .is_none());

    // Shift only - no broadcast
    assert!(manager
        .get_broadcast_targets(&term1, false, false, true, false)
        .is_none());

    // Ctrl+Shift - broadcast!
    let targets = manager
        .get_broadcast_targets(&term1, true, false, true, false)
        .unwrap();
    assert_eq!(targets.len(), 1);
    assert!(targets.contains(&term2));
}

#[test]
fn test_terminal_lifecycle() {
    let mut manager = BroadcastManager::new();

    let term1 = Uuid::new_v4();
    let term2 = Uuid::new_v4();

    manager.register_terminal(term1);
    manager.register_terminal(term2);

    manager.create_group("test".to_string()).unwrap();
    manager.add_to_group("test", term1).unwrap();
    manager.add_to_group("test", term2).unwrap();

    // Verify membership
    let groups = manager.find_groups_for_terminal(&term1);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0], "test");

    // Unregister terminal removes it from groups
    manager.unregister_terminal(&term1);

    let groups = manager.find_groups_for_terminal(&term1);
    assert_eq!(groups.len(), 0);

    let group = manager.get_group("test").unwrap();
    assert!(!group.contains(&term1));
    assert_eq!(group.member_count(), 1);
}

#[test]
fn test_error_conditions() {
    let mut manager = BroadcastManager::new();

    // Cannot add unregistered terminal
    let term1 = Uuid::new_v4();
    manager.create_group("test".to_string()).unwrap();

    let result = manager.add_to_group("test", term1);
    assert!(matches!(result, Err(BroadcastError::TerminalNotFound(_))));

    // Cannot create duplicate group
    let result = manager.create_group("test".to_string());
    assert!(matches!(
        result,
        Err(BroadcastError::GroupAlreadyExists(_))
    ));

    // Cannot activate non-existent group
    let result = manager.activate_group("nonexistent");
    assert!(matches!(result, Err(BroadcastError::GroupNotFound(_))));
}

#[test]
fn test_group_descriptions() {
    let mut manager = BroadcastManager::new();

    manager.create_group("servers".to_string()).unwrap();

    {
        let group = manager.get_group_mut("servers").unwrap();
        assert!(group.description().is_none());

        group.set_description(Some("Production web servers".to_string()));
        assert_eq!(group.description(), Some("Production web servers"));

        group.set_description(None);
        assert!(group.description().is_none());
    }
}

#[test]
fn test_deactivation() {
    let mut manager = BroadcastManager::new();

    let term1 = Uuid::new_v4();
    let term2 = Uuid::new_v4();

    manager.register_terminal(term1);
    manager.register_terminal(term2);

    manager.create_group("test".to_string()).unwrap();
    manager.add_to_group("test", term1).unwrap();
    manager.add_to_group("test", term2).unwrap();

    manager.activate_group("test").unwrap();
    assert!(manager.active_group_name().is_some());

    // Broadcasting works when active
    assert!(manager
        .get_broadcast_targets(&term1, false, false, false, false)
        .is_some());

    // Deactivate
    manager.deactivate_current().unwrap();
    assert!(manager.active_group_name().is_none());

    // Broadcasting doesn't work when deactivated
    assert!(manager
        .get_broadcast_targets(&term1, false, false, false, false)
        .is_none());
}

#[test]
fn test_statistics() {
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

    manager.activate_group("group1").unwrap();

    let stats = manager.stats();
    assert_eq!(stats.total_groups, 2);
    assert_eq!(stats.active_groups, 1);
    assert_eq!(stats.total_terminals, 3);
    assert_eq!(stats.terminals_in_groups, 2); // term1 and term2
}
