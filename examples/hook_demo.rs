//! Demonstration of the AgTerm Hook System
//!
//! Run with: cargo run --example hook_demo

use agterm::config::{Hook, HookAction, HookEvent, HookManager};

fn main() {
    println!("=== AgTerm Hook System Demo ===\n");

    // Create a hook manager
    let mut manager = HookManager::new();

    println!("Default hooks loaded: {}", manager.hooks().len());
    for hook in manager.hooks() {
        println!(
            "  - {} ({})",
            hook.name,
            if hook.enabled { "enabled" } else { "disabled" }
        );
    }

    println!("\n--- Creating Custom Hooks ---");

    // Create a custom hook for git commands
    let git_hook = Hook::new(
        "Git Success Notifier".to_string(),
        HookEvent::CommandComplete {
            command_pattern: Some("git.*".to_string()),
            exit_code: Some(0),
        },
        HookAction::Notify {
            title: "Git Command".to_string(),
            message: "Git command completed successfully!".to_string(),
        },
    );
    manager.add_hook(git_hook);
    println!("✓ Added Git Success Notifier hook");

    // Create a hook for directory changes
    let dir_hook = Hook::new(
        "Directory Tracker".to_string(),
        HookEvent::DirectoryChange {
            directory_pattern: Some("/tmp".to_string()),
        },
        HookAction::Notify {
            title: "Directory Change".to_string(),
            message: "Entered /tmp directory".to_string(),
        },
    );
    manager.add_hook(dir_hook);
    println!("✓ Added Directory Tracker hook");

    // Create a hook for error patterns
    let error_hook = Hook::new(
        "Error Detector".to_string(),
        HookEvent::OutputMatch {
            pattern: "(?i)error|fail".to_string(),
        },
        HookAction::PlaySound {
            path: "/System/Library/Sounds/Funk.aiff".to_string(),
            volume: 0.5,
        },
    );
    manager.add_hook(error_hook);
    println!("✓ Added Error Detector hook");

    println!("\nTotal hooks: {}", manager.hooks().len());

    println!("\n--- Testing Event Matching ---");

    // Test command completion event
    println!("\nTesting: Git command completed with exit code 0");
    manager.process_event(&HookEvent::CommandComplete {
        command_pattern: Some("git status".to_string()),
        exit_code: Some(0),
    });

    // Test directory change event
    println!("\nTesting: Directory changed to /tmp");
    manager.process_event(&HookEvent::DirectoryChange {
        directory_pattern: Some("/tmp".to_string()),
    });

    // Test output match event
    println!("\nTesting: Error output detected");
    manager.process_event(&HookEvent::OutputMatch {
        pattern: "Error: something went wrong".to_string(),
    });

    // Test bell event
    println!("\nTesting: Terminal bell");
    manager.process_event(&HookEvent::Bell);

    println!("\n--- Hook Management ---");

    // Disable a hook
    println!("\nDisabling 'Git Success Notifier' hook...");
    manager.set_hook_enabled("Git Success Notifier", false);

    println!("Testing git command again (should not trigger):");
    manager.process_event(&HookEvent::CommandComplete {
        command_pattern: Some("git push".to_string()),
        exit_code: Some(0),
    });

    // Re-enable it
    println!("\nRe-enabling 'Git Success Notifier' hook...");
    manager.set_hook_enabled("Git Success Notifier", true);

    println!("Testing git command again (should trigger):");
    manager.process_event(&HookEvent::CommandComplete {
        command_pattern: Some("git commit".to_string()),
        exit_code: Some(0),
    });

    // Remove a hook
    println!("\n--- Removing Hooks ---");
    println!("Removing 'Directory Tracker' hook...");
    if manager.remove_hook("Directory Tracker") {
        println!("✓ Successfully removed");
    }

    println!("\nFinal hook count: {}", manager.hooks().len());

    println!("\n=== Demo Complete ===");
    println!("\nNOTE: Actual notifications, sounds, and commands are placeholders.");
    println!("Check the logs to see when hooks are triggered.");
}
