//! Quick Actions Module Demo
//!
//! This example demonstrates the functionality of the quick_actions module.
//! Run with: cargo run --example quick_actions_demo

use agterm::quick_actions::{
    ActionCategory, ActionCommand, ActionError, QuickAction, QuickActionManager,
};

fn main() {
    println!("=== AgTerm Quick Actions Demo ===\n");

    // Create a new manager with default actions
    let mut manager = QuickActionManager::new();

    println!("1. Default Actions Loaded:");
    println!("   Total actions: {}", manager.get_all().len());
    for action in manager.get_all().iter().take(5) {
        println!(
            "   - {} ({}): {}",
            action.name,
            action.category.name(),
            action.description
        );
    }

    // Register a custom action
    println!("\n2. Registering Custom Action:");
    let custom_action = QuickAction::new(
        "my_custom_action",
        "My Custom Action",
        "This is a custom action I created",
        ActionCategory::Custom,
        ActionCommand::Shell("echo Hello World".to_string()),
    )
    .with_shortcut("Ctrl+Shift+H")
    .with_icon("custom_icon");

    manager.register(custom_action);
    println!("   Registered: My Custom Action");

    // Search for actions
    println!("\n3. Fuzzy Search Demo:");
    let search_queries = vec!["tab", "new", "copy", "nt"];
    for query in search_queries {
        let results = manager.search(query);
        println!(
            "   Query '{}' found {} matches:",
            query,
            results.len()
        );
        for (i, action) in results.iter().take(3).enumerate() {
            println!("      {}. {}", i + 1, action.name);
        }
    }

    // Get actions by category
    println!("\n4. Actions by Category:");
    let categories = vec![
        ActionCategory::Tab,
        ActionCategory::Terminal,
        ActionCategory::Clipboard,
    ];
    for category in categories {
        let actions = manager.get_by_category(category);
        println!(
            "   {}: {} actions",
            category.name(),
            actions.len()
        );
    }

    // Execute actions
    println!("\n5. Executing Actions:");
    let action_ids = vec!["new_tab", "copy", "my_custom_action"];
    for id in action_ids {
        match manager.execute(id) {
            Ok(_) => println!("   ✓ Executed: {}", id),
            Err(ActionError::NotFound) => {
                println!("   ✗ Not found: {}", id)
            }
            Err(ActionError::Disabled) => {
                println!("   ✗ Disabled: {}", id)
            }
            Err(e) => println!("   ✗ Error: {}", e),
        }
    }

    // Recently used actions
    println!("\n6. Recently Used Actions:");
    let recent = manager.get_recent(5);
    for (i, action) in recent.iter().enumerate() {
        println!("   {}. {}", i + 1, action.name);
    }

    // Disable an action
    println!("\n7. Disabling an Action:");
    manager.set_enabled("new_tab", false);
    println!("   Disabled: new_tab");

    match manager.execute("new_tab") {
        Err(ActionError::Disabled) => {
            println!("   ✓ Correctly blocked disabled action")
        }
        _ => println!("   ✗ Should have been blocked"),
    }

    // Search doesn't include disabled actions
    let results = manager.search("new tab");
    println!(
        "   Search for 'new tab' found {} results (disabled excluded)",
        results.len()
    );

    // Serialization demo
    println!("\n8. Serialization:");
    match serde_json::to_string_pretty(&manager) {
        Ok(json) => {
            let preview: String = json.chars().take(200).collect();
            println!("   ✓ Serialized to JSON");
            println!("   Preview: {}...", preview);
        }
        Err(e) => println!("   ✗ Serialization failed: {}", e),
    }

    println!("\n=== Demo Complete ===");
}
