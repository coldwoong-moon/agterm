//! Example: SSH Profile Management
//!
//! This example demonstrates how to create, manage, and use SSH profiles in AgTerm.
//!
//! Run with: cargo run --example ssh_profiles

use agterm::ssh::{SshProfile, SshProfileManager};
use std::path::PathBuf;

fn main() {
    println!("=== AgTerm SSH Profile Management Example ===\n");

    // Example 1: Create a basic SSH profile
    println!("1. Creating basic SSH profile:");
    let basic = SshProfile::new("myserver".to_string(), "example.com".to_string());
    println!("   Name: {}", basic.name);
    println!("   Connection: {}", basic.connection_string());
    println!("   Command: {:?}", basic.to_command());
    println!();

    // Example 2: Create a profile with all options
    println!("2. Creating advanced SSH profile:");
    let mut advanced = SshProfile::new(
        "production".to_string(),
        "prod.example.com".to_string(),
    );
    advanced.user = Some("admin".to_string());
    advanced.port = 2222;
    advanced.identity_file = Some(PathBuf::from("/home/user/.ssh/id_rsa"));
    advanced.forward_agent = true;
    advanced.proxy_jump = Some("bastion.example.com".to_string());
    advanced
        .extra_options
        .push("StrictHostKeyChecking=no".to_string());

    println!("   Name: {}", advanced.name);
    println!("   Connection: {}", advanced.connection_string());
    println!("   Command: {:?}", advanced.to_command());
    println!();

    // Example 3: Managing profiles with ProfileManager
    println!("3. Using SshProfileManager:");
    let mut manager = SshProfileManager::new();

    // Add profiles
    manager.add(basic.clone());
    manager.add(advanced.clone());

    println!("   Total profiles: {}", manager.list().len());

    // Retrieve a profile
    if let Some(profile) = manager.get("production") {
        println!("   Retrieved profile: {}", profile.name);
        println!("   Host: {}", profile.host);
    }

    // List all profiles
    println!("\n   All profiles:");
    for profile in manager.list() {
        println!("     - {}: {}", profile.name, profile.connection_string());
    }
    println!();

    // Example 4: Load from SSH config (if available)
    println!("4. Loading from ~/.ssh/config:");
    let config_manager = SshProfileManager::load_from_ssh_config();
    println!(
        "   Found {} profiles in SSH config",
        config_manager.list().len()
    );

    if !config_manager.list().is_empty() {
        println!("\n   Profiles from SSH config:");
        for profile in config_manager.list() {
            println!("     - {}: {}", profile.name, profile.connection_string());
        }
    } else {
        println!("   (No SSH config file found or no hosts defined)");
    }
    println!();

    // Example 5: Load specific host from SSH config
    println!("5. Loading specific host from SSH config:");
    match SshProfile::from_ssh_config("github.com") {
        Some(profile) => {
            println!("   Found profile for github.com:");
            println!("     Host: {}", profile.host);
            println!("     User: {:?}", profile.user);
            println!("     Port: {}", profile.port);
        }
        None => {
            println!("   No profile found for 'github.com' in SSH config");
        }
    }
    println!();

    // Example 6: Demonstrate connection strings
    println!("6. Connection string formats:");
    let examples = vec![
        SshProfile::new("simple".to_string(), "example.com".to_string()),
        {
            let mut p = SshProfile::new("with-user".to_string(), "example.com".to_string());
            p.user = Some("alice".to_string());
            p
        },
        {
            let mut p = SshProfile::new("custom-port".to_string(), "example.com".to_string());
            p.user = Some("bob".to_string());
            p.port = 2222;
            p
        },
    ];

    for profile in examples {
        println!(
            "   {}: {}",
            profile.name,
            profile.connection_string()
        );
    }
    println!();

    // Example 7: Remove a profile
    println!("7. Removing a profile:");
    manager.remove("myserver");
    println!(
        "   Profiles after removal: {}",
        manager.list().len()
    );
    println!();

    println!("=== Example Complete ===");
}
