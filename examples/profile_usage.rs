//! Example: Using Terminal Profiles
//!
//! This example demonstrates how to use the profile system in AgTerm.
//!
//! Run with: cargo run --example profile_usage

use agterm::config::{Profile, AppConfig};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AgTerm Profile System Demo ===\n");

    // 1. Create default profiles
    println!("Creating default profiles...");
    match Profile::create_default_profiles() {
        Ok(_) => println!("Default profiles created successfully!"),
        Err(e) => println!("Note: {}", e),
    }
    println!();

    // 2. List all available profiles
    println!("Available profiles:");
    match Profile::list() {
        Ok(profiles) => {
            for name in &profiles {
                println!("  - {}", name);
            }
            if profiles.is_empty() {
                println!("  (no profiles found)");
            }
        }
        Err(e) => println!("Error listing profiles: {}", e),
    }
    println!();

    // 3. Create a custom profile
    println!("Creating custom development profile...");
    let mut dev_env = HashMap::new();
    dev_env.insert("EDITOR".to_string(), "vim".to_string());
    dev_env.insert("RUST_BACKTRACE".to_string(), "1".to_string());

    let dev_profile = Profile {
        name: "development".to_string(),
        shell: Some("/bin/zsh".to_string()),
        shell_args: vec!["-l".to_string()],
        env: dev_env,
        theme: Some("dark".to_string()),
        font_size: Some(16.0),
        working_dir: None,
        color_scheme: None,
    };

    match dev_profile.save() {
        Ok(_) => println!("Development profile saved!"),
        Err(e) => println!("Error saving profile: {}", e),
    }
    println!();

    // 4. Load a profile
    println!("Loading development profile...");
    match Profile::load("development") {
        Ok(profile) => {
            println!("Profile loaded successfully:");
            println!("  Name: {}", profile.name);
            println!("  Shell: {:?}", profile.shell);
            println!("  Args: {:?}", profile.shell_args);
            println!("  Theme: {:?}", profile.theme);
            println!("  Font Size: {:?}", profile.font_size);
            println!("  Environment Variables:");
            for (key, value) in &profile.env {
                println!("    {}={}", key, value);
            }
        }
        Err(e) => println!("Error loading profile: {}", e),
    }
    println!();

    // 5. Apply profile to configuration
    println!("Applying profile to configuration...");
    let mut config = AppConfig::default();
    println!("Original font size: {}", config.appearance.font_size);

    if let Ok(profile) = Profile::load("development") {
        profile.apply_to_config(&mut config);
        println!("Updated font size: {}", config.appearance.font_size);
        println!("Updated shell: {:?}", config.shell.program);
    }
    println!();

    // 6. Create a profile with custom colors
    println!("Creating Catppuccin theme profile...");
    let catppuccin_profile = Profile {
        name: "catppuccin_demo".to_string(),
        shell: None,
        shell_args: Vec::new(),
        env: HashMap::new(),
        theme: Some("catppuccin".to_string()),
        font_size: Some(14.0),
        working_dir: None,
        color_scheme: Some(agterm::config::ColorScheme {
            background: "#1e1e2e".to_string(),
            foreground: "#cdd6f4".to_string(),
            cursor: "#f5e0dc".to_string(),
            selection: Some("#585b70".to_string()),
            black: Some("#45475a".to_string()),
            red: Some("#f38ba8".to_string()),
            green: Some("#a6e3a1".to_string()),
            yellow: Some("#f9e2af".to_string()),
            blue: Some("#89b4fa".to_string()),
            magenta: Some("#f5c2e7".to_string()),
            cyan: Some("#94e2d5".to_string()),
            white: Some("#bac2de".to_string()),
            bright_black: Some("#585b70".to_string()),
            bright_red: Some("#f38ba8".to_string()),
            bright_green: Some("#a6e3a1".to_string()),
            bright_yellow: Some("#f9e2af".to_string()),
            bright_blue: Some("#89b4fa".to_string()),
            bright_magenta: Some("#f5c2e7".to_string()),
            bright_cyan: Some("#94e2d5".to_string()),
            bright_white: Some("#a6adc8".to_string()),
        }),
    };

    match catppuccin_profile.save() {
        Ok(_) => println!("Catppuccin profile saved with custom colors!"),
        Err(e) => println!("Error saving profile: {}", e),
    }
    println!();

    // 7. Final profile list
    println!("Final profile list:");
    match Profile::list() {
        Ok(profiles) => {
            for name in &profiles {
                println!("  - {}", name);
            }
        }
        Err(e) => println!("Error listing profiles: {}", e),
    }
    println!();

    println!("Profile system demo complete!");
    println!("\nProfiles are stored in: {:?}", Profile::profiles_dir());

    Ok(())
}
