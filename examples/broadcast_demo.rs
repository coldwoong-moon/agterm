//! Broadcast demonstration example
//!
//! This example demonstrates the terminal broadcast functionality,
//! showing how to create groups, add terminals, and broadcast input.

use agterm::broadcast::{BroadcastManager, BroadcastMode, BroadcastTrigger};
use uuid::Uuid;

fn main() {
    println!("=== AgTerm Broadcast Demo ===\n");

    // Create a broadcast manager
    let mut manager = BroadcastManager::new();

    // Simulate creating three terminal sessions
    println!("Creating terminals...");
    let server1 = Uuid::new_v4();
    let server2 = Uuid::new_v4();
    let server3 = Uuid::new_v4();

    manager.register_terminal(server1);
    manager.register_terminal(server2);
    manager.register_terminal(server3);

    println!("  Terminal 1: {}", server1);
    println!("  Terminal 2: {}", server2);
    println!("  Terminal 3: {}", server3);
    println!();

    // Create a broadcast group
    println!("Creating broadcast group 'webservers'...");
    manager
        .create_group("webservers".to_string())
        .expect("Failed to create group");

    // Add terminals to the group
    println!("Adding terminals to group...");
    manager
        .add_to_group("webservers", server1)
        .expect("Failed to add server1");
    manager
        .add_to_group("webservers", server2)
        .expect("Failed to add server2");
    manager
        .add_to_group("webservers", server3)
        .expect("Failed to add server3");

    // Set group description
    {
        let group = manager.get_group_mut("webservers").unwrap();
        group.set_description(Some("Production web servers".to_string()));
    }

    println!("  Group 'webservers' created with 3 terminals");
    println!();

    // Demo 1: Full broadcast mode
    println!("=== Demo 1: Full Broadcast Mode ===");
    println!("All input is broadcast to all terminals in the group\n");

    manager
        .activate_group("webservers")
        .expect("Failed to activate group");

    let input = "systemctl status nginx";
    println!("User types in Terminal 1: {}", input);

    if let Some(targets) = manager.get_broadcast_targets(&server1, false, false, false, false) {
        println!("Broadcasting to {} terminals:", targets.len());
        for target in targets {
            println!("  → {}", target);
        }
    }
    println!();

    // Demo 2: Selective broadcast mode
    println!("=== Demo 2: Selective Broadcast Mode ===");
    println!("Only Ctrl+Shift+<key> combinations are broadcast\n");

    {
        let group = manager.get_group_mut("webservers").unwrap();
        group.set_mode(BroadcastMode::Selective);
        group.set_trigger(BroadcastTrigger::default_selective());
    }

    // Regular input - not broadcast
    let input = "ls -la";
    println!("User types in Terminal 1: {} (no modifiers)", input);
    if let Some(_targets) = manager.get_broadcast_targets(&server1, false, false, false, false) {
        println!("  → Broadcast (should not happen in selective mode)");
    } else {
        println!("  → Local only (correct!)");
    }
    println!();

    // Ctrl+Shift input - broadcast
    let input = "sudo systemctl restart nginx";
    println!(
        "User types in Terminal 1 with Ctrl+Shift: {}",
        input
    );
    if let Some(targets) = manager.get_broadcast_targets(&server1, true, false, true, false) {
        println!("  → Broadcasting to {} terminals:", targets.len());
        for target in targets {
            println!("     {}", target);
        }
    } else {
        println!("  → Local only");
    }
    println!();

    // Demo 3: Multiple groups
    println!("=== Demo 3: Multiple Groups ===");
    println!("Terminals can belong to different groups\n");

    manager
        .create_group("staging".to_string())
        .expect("Failed to create staging group");

    let staging1 = Uuid::new_v4();
    let staging2 = Uuid::new_v4();
    manager.register_terminal(staging1);
    manager.register_terminal(staging2);

    manager
        .add_to_group("staging", staging1)
        .expect("Failed to add staging1");
    manager
        .add_to_group("staging", staging2)
        .expect("Failed to add staging2");

    println!("Created group 'staging' with 2 terminals");
    println!("  Staging 1: {}", staging1);
    println!("  Staging 2: {}", staging2);
    println!();

    // Switch active group
    println!("Switching to 'staging' group...");
    manager
        .activate_group("staging")
        .expect("Failed to activate staging");

    {
        let group = manager.get_group_mut("staging").unwrap();
        group.set_mode(BroadcastMode::Full);
    }

    let input = "git pull";
    println!("User types in Staging 1: {}", input);

    if let Some(targets) = manager.get_broadcast_targets(&staging1, false, false, false, false) {
        println!("Broadcasting to {} terminals:", targets.len());
        for target in targets {
            println!("  → {}", target);
        }
    }
    println!();

    // Demo 4: Group management
    println!("=== Demo 4: Group Management ===");
    println!("Finding groups for terminals\n");

    let groups = manager.find_groups_for_terminal(&server1);
    println!("Terminal {} belongs to groups:", server1);
    for group in groups {
        println!("  - {}", group);
    }
    println!();

    // Demo 5: Statistics
    println!("=== Demo 5: Statistics ===");
    let stats = manager.stats();
    println!("Total groups: {}", stats.total_groups);
    println!("Active groups: {}", stats.active_groups);
    println!("Total registered terminals: {}", stats.total_terminals);
    println!(
        "Terminals in at least one group: {}",
        stats.terminals_in_groups
    );
    println!();

    // Demo 6: Deactivation
    println!("=== Demo 6: Deactivation ===");
    println!("Deactivating broadcast...\n");

    manager
        .deactivate_current()
        .expect("Failed to deactivate");
    println!("Broadcast deactivated");
    println!(
        "Active group: {}",
        manager.active_group_name().unwrap_or("None")
    );
    println!();

    // Demo 7: Cleanup
    println!("=== Demo 7: Cleanup ===");
    println!("Removing terminal from group...\n");

    manager
        .remove_from_group("webservers", &server1)
        .expect("Failed to remove terminal");

    let group = manager.get_group("webservers").unwrap();
    println!("Removed server1 from 'webservers'");
    println!("Group now has {} members", group.member_count());
    println!();

    println!("Unregistering terminal...");
    manager.unregister_terminal(&server1);
    println!("Server1 unregistered and removed from all groups");
    println!();

    // Final statistics
    println!("=== Final Statistics ===");
    let stats = manager.stats();
    println!("Total groups: {}", stats.total_groups);
    println!("Total terminals: {}", stats.total_terminals);
    println!(
        "Terminals in groups: {}",
        stats.terminals_in_groups
    );
    println!();

    println!("=== Demo Complete ===");
}
