/// Example: Using the AgTerm Snippet System
///
/// This example demonstrates how to use the snippet/macro system in AgTerm.
/// Snippets allow users to define text shortcuts that expand to longer commands.
///
/// Run with: cargo run --example snippet_usage
use agterm::config::{ConfigError, Snippet};

fn main() -> Result<(), ConfigError> {
    println!("=== AgTerm Snippet System Example ===\n");

    // 1. Load default snippets
    println!("1. Loading default snippets...");
    let snippets = Snippet::default_snippets();
    println!("   Loaded {} default snippets\n", snippets.len());

    // 2. List all categories
    println!("2. Available categories:");
    let categories = Snippet::get_categories(&snippets);
    for category in &categories {
        let count = Snippet::find_by_category(&snippets, category).len();
        println!("   - {} ({} snippets)", category, count);
    }
    println!();

    // 3. Find a snippet by trigger
    println!("3. Finding snippet by trigger '/gs':");
    if let Some(snippet) = Snippet::find_by_trigger(&snippets, "/gs") {
        println!("   Name: {}", snippet.name);
        println!("   Trigger: {}", snippet.trigger);
        println!("   Content: {}", snippet.content);
        println!("   Category: {}", snippet.category);
    }
    println!();

    // 4. List all git snippets
    println!("4. Git snippets:");
    let git_snippets = Snippet::find_by_category(&snippets, "git");
    for snippet in git_snippets {
        println!("   {} → {}", snippet.trigger, snippet.content);
    }
    println!();

    // 5. Create a custom snippet
    println!("5. Creating custom snippet...");
    let custom = Snippet::new(
        "My Custom Command".to_string(),
        "/mycmd".to_string(),
        "echo 'Hello from AgTerm!'".to_string(),
        "custom".to_string(),
    );
    println!("   Created: {} → {}\n", custom.trigger, custom.content);

    // 6. Save snippets to file
    println!("6. Save/Load example:");
    println!("   To save snippets to ~/.config/agterm/snippets.toml:");
    println!("   Snippet::save_to_file(&snippets)?;\n");
    println!("   To load snippets from file:");
    println!("   let loaded = Snippet::load_from_file()?;\n");

    // 7. Initialize default snippets file
    println!("7. Initialize default snippets file:");
    println!("   This creates ~/.config/agterm/snippets.toml if it doesn't exist:");
    println!("   Snippet::initialize_default_file()?;\n");

    // 8. Show some useful snippets
    println!("8. Most useful snippets:");
    let useful_triggers = vec!["/gs", "/ga", "/gc", "/gp", "/dps", "/ll", "/cb", "/cr"];
    for trigger in useful_triggers {
        if let Some(snippet) = Snippet::find_by_trigger(&snippets, trigger) {
            println!(
                "   {} → {} ({})",
                snippet.trigger, snippet.content, snippet.category
            );
        }
    }
    println!();

    println!("=== Example Complete ===");
    println!("\nTo use snippets in AgTerm:");
    println!("1. Type a trigger (e.g., '/gs')");
    println!("2. Press space or tab to expand");
    println!("3. Edit ~/.config/agterm/snippets.toml to add custom snippets");

    Ok(())
}
