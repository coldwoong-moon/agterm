//! Demonstration of the snippet system functionality
//!
//! Run with: cargo run --example snippets_demo

use agterm::snippets::{Snippet, SnippetManager};
use std::collections::HashMap;

fn main() {
    println!("=== AgTerm Snippet System Demo ===\n");

    // Create a snippet manager with default snippets
    let manager = SnippetManager::with_defaults();

    // Show available categories
    println!("Available categories:");
    for category in manager.get_categories() {
        let count = manager.get_by_category(&category).len();
        println!("  - {}: {} snippets", category, count);
    }
    println!();

    // Demonstrate trigger-based search
    println!("Searching for snippets with trigger 'fn':");
    if let Some(snippet) = manager.find_exact_trigger("fn") {
        println!("  Found: {}", snippet.name);
        println!("  Description: {}", snippet.description);
        println!("  Template: {}", snippet.template.replace('\n', "\\n"));
        println!();
    }

    // Demonstrate template expansion with sequential placeholders
    println!("=== Sequential Placeholders Example ===");
    let template = "Hello $1, welcome to $2!";
    let mut values = HashMap::new();
    values.insert("1".to_string(), "World".to_string());
    values.insert("2".to_string(), "AgTerm".to_string());

    let (result, cursor) = manager.expand_template(template, &values);
    println!("Template: {}", template);
    println!("Expanded: {}", result);
    println!("Cursor position: {:?}", cursor);
    println!();

    // Demonstrate named placeholders with defaults
    println!("=== Named Placeholders with Defaults Example ===");
    let template = "fn ${name}() -> ${type:Result<(), Error>} { $0 }";
    let mut values = HashMap::new();
    values.insert("name".to_string(), "process_data".to_string());
    // Note: 'type' is not provided, so it will use the default "Result<(), Error>"

    let (result, cursor) = manager.expand_template(template, &values);
    println!("Template: {}", template);
    println!("Expanded: {}", result);
    println!("Cursor position: {:?}", cursor);
    println!();

    // Demonstrate creating a custom snippet
    println!("=== Custom Snippet Creation ===");
    let custom = Snippet::new(
        "Custom Loop",
        "A custom for loop template",
        "forr",
        "for ${var} in ${start}..${end} {\n    $0\n}",
        "rust",
    )
    .with_tag("loop")
    .with_tag("custom");

    println!("Created custom snippet:");
    println!("  Name: {}", custom.name);
    println!("  Trigger: {}", custom.trigger);
    println!("  Category: {}", custom.category);
    println!("  Tags: {:?}", custom.tags);
    println!();

    // Demonstrate template parsing
    println!("=== Template Parsing ===");
    let parsed = manager.parse_template("fn ${name}($1) -> $2 {\n    $0\n}");
    println!("Template parts: {} parts", parsed.parts.len());
    println!("Placeholders: {} placeholders", parsed.placeholders.len());
    println!("Has final position: {}", parsed.final_position.is_some());
    println!();

    // Show rust category snippets
    println!("=== Rust Category Snippets ===");
    let rust_snippets = manager.get_by_category("rust");
    for snippet in rust_snippets {
        println!("  {} (trigger: '{}')", snippet.name, snippet.trigger);
    }
    println!();

    // Show git category snippets
    println!("=== Git Category Snippets ===");
    let git_snippets = manager.get_by_category("git");
    for snippet in git_snippets {
        println!("  {} (trigger: '{}')", snippet.name, snippet.trigger);
    }
    println!();

    println!("=== Demo Complete ===");
}
