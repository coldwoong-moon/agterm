//! Profile System Demo
//!
//! This example demonstrates how to use the AgTerm profile system.

use agterm::profiles::{Profile, ProfileManager, FontSettings, ColorSettings, TerminalSettings, ShellSettings};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AgTerm Profile System Demo ===\n");

    // Create a new profile manager
    let mut manager = ProfileManager::new();
    manager.init()?;

    println!("1. Listing built-in profiles:");
    for profile_name in manager.list_profiles() {
        if let Some(profile) = manager.get_profile_by_name(&profile_name) {
            println!("  - {} ({})", profile.name, if profile.read_only { "built-in" } else { "custom" });
            if let Some(desc) = &profile.description {
                println!("    {}", desc);
            }
        }
    }

    // Create a custom profile
    println!("\n2. Creating a custom 'Coding' profile:");
    let mut coding_profile = Profile::new("Coding".to_string());
    coding_profile.description = Some("Optimized for programming".to_string());
    coding_profile.icon = Some("🚀".to_string());

    // Configure font
    coding_profile.font = FontSettings {
        family: "Fira Code".to_string(),
        size: 16.0,
        line_height: 1.4,
        bold_as_bright: true,
        use_ligatures: true,
        use_thin_strokes: false,
    };

    // Configure colors
    coding_profile.colors = ColorSettings {
        theme: "monokai_pro".to_string(),
        background_opacity: 0.95,
        ..Default::default()
    };

    // Configure terminal settings
    coding_profile.terminal = TerminalSettings {
        scrollback_lines: 50000,
        scroll_on_input: true,
        copy_on_select: false,
        ..Default::default()
    };

    // Configure shell
    coding_profile.shell = ShellSettings {
        command: Some("/bin/zsh".to_string()),
        args: vec!["-l".to_string()],
        shell_type: Some("zsh".to_string()),
        login_shell: true,
    };

    // Add environment variables
    coding_profile.environment.insert("EDITOR".to_string(), "nvim".to_string());
    coding_profile.environment.insert("PAGER".to_string(), "less -R".to_string());

    // Add startup commands
    coding_profile.startup_commands = vec![
        "echo 'Welcome to Coding Profile!'".to_string(),
        "clear".to_string(),
    ];

    coding_profile.working_directory = Some(std::env::current_dir()?);

    let coding_id = manager.add_profile(coding_profile)?;
    println!("  Created profile with ID: {}", coding_id);

    // Set as default
    println!("\n3. Setting 'Coding' as default profile:");
    manager.set_default_profile(&coding_id)?;
    if let Some(default) = manager.get_default_profile() {
        println!("  Default profile: {}", default.name);
    }

    // Clone the profile
    println!("\n4. Cloning 'Coding' profile to 'Coding (Work)':");
    let cloned_id = manager.clone_profile(&coding_id, "Coding (Work)".to_string())?;
    if let Some(cloned) = manager.get_profile(&cloned_id) {
        println!("  Cloned profile: {} (ID: {})", cloned.name, cloned.id);
    }

    // List all profiles
    println!("\n5. All profiles:");
    for profile in manager.get_all_profiles() {
        println!("  - {} {} ({})",
            profile.icon.as_ref().unwrap_or(&"".to_string()),
            profile.name,
            if profile.read_only { "built-in" } else { "custom" }
        );
    }

    // Save to file
    println!("\n6. Saving custom profile to file:");
    let temp_dir = std::env::temp_dir();
    let export_path = temp_dir.join("coding_profile.toml");
    manager.export_profile(&coding_id, &export_path)?;
    println!("  Exported to: {:?}", export_path);

    // Show profile details
    if let Some(profile) = manager.get_profile(&coding_id) {
        println!("\n7. Profile details:");
        println!("  Name: {}", profile.name);
        println!("  Description: {}", profile.description.as_ref().unwrap_or(&"None".to_string()));
        println!("  Font: {} @ {}pt", profile.font.family, profile.font.size);
        println!("  Theme: {}", profile.colors.theme);
        println!("  Scrollback: {} lines", profile.terminal.scrollback_lines);
        if let Some(shell) = &profile.shell.command {
            println!("  Shell: {}", shell);
        }
        println!("  Environment variables: {}", profile.environment.len());
        println!("  Startup commands: {}", profile.startup_commands.len());
    }

    println!("\n=== Demo Complete ===");
    Ok(())
}
