//! Environment Manager Demo
//!
//! This example demonstrates how to use the EnvManager API to manage
//! environment variables with metadata, categorization, and persistence.

use agterm::env_manager::{EnvManager, EnvVarSource};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AgTerm Environment Manager Demo ===\n");

    // Create a new environment manager
    let mut manager = EnvManager::new();
    println!("✓ Created new EnvManager");

    // Set some basic variables
    manager.set_var(
        "MY_API_URL".to_string(),
        "https://api.example.com".to_string(),
        EnvVarSource::User,
    )?;

    manager.set_var(
        "DEBUG_MODE".to_string(),
        "true".to_string(),
        EnvVarSource::Session,
    )?;

    println!("✓ Added 2 variables\n");

    // Set a detailed variable with all metadata
    manager.set_var_detailed(
        "DATABASE_PASSWORD".to_string(),
        "super_secret_password".to_string(),
        EnvVarSource::User,
        Some("Production database password".to_string()),
        Some("SENSITIVE".to_string()),
        true, // Mark as sensitive
    )?;

    println!("✓ Added sensitive variable (will be masked)\n");

    // List all variables
    println!("All variables:");
    for var in manager.list_vars() {
        println!("  {} = {} (source: {:?})", var.name, var.masked_value(), var.source);
    }
    println!();

    // Search for variables
    println!("Search results for 'API':");
    for var in manager.search_vars("API") {
        println!("  {} = {}", var.name, var.masked_value());
    }
    println!();

    // List by category
    println!("Variables by category 'SENSITIVE':");
    for var in manager.list_by_category("SENSITIVE") {
        println!("  {} = {} (sensitive: {})", var.name, var.masked_value(), var.sensitive);
    }
    println!();

    // Export for shell (sensitive vars excluded)
    println!("Export for shell:");
    let exported = manager.export_for_shell();
    for (name, value) in exported.iter() {
        println!("  export {}={}", name, value);
    }
    println!();

    // Get statistics
    let stats = manager.stats();
    println!("Statistics:");
    println!("  {}", stats.format());
    println!();

    // Save to file
    let temp_file = std::env::temp_dir().join("agterm_env_demo.json");
    manager.save_to_file(&temp_file)?;
    println!("✓ Saved to: {}", temp_file.display());

    // Load from file
    let loaded = EnvManager::load_from_file(&temp_file)?;
    println!("✓ Loaded {} variables from file", loaded.list_vars().len());

    // Clean up
    std::fs::remove_file(&temp_file)?;
    println!("✓ Cleaned up temp file\n");

    // Demonstrate auto-categorization
    let mut manager2 = EnvManager::new();
    manager2.auto_categorize = true;

    manager2.set_var("PATH".to_string(), "/usr/bin:/bin".to_string(), EnvVarSource::User)?;
    manager2.set_var("LC_ALL".to_string(), "en_US.UTF-8".to_string(), EnvVarSource::User)?;
    manager2.set_var("SECRET_TOKEN".to_string(), "token123".to_string(), EnvVarSource::User)?;

    println!("Auto-categorized variables:");
    for var in manager2.list_vars() {
        println!(
            "  {} -> category: {}, sensitive: {}",
            var.name,
            var.category.as_ref().unwrap_or(&"None".to_string()),
            var.sensitive
        );
    }
    println!();

    // Get all categories
    println!("Available categories:");
    for category in manager2.get_categories() {
        println!("  - {}", category);
    }
    println!();

    println!("=== Demo Complete ===");

    Ok(())
}
