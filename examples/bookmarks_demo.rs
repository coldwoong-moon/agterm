//! Demonstration of the bookmark system functionality

use agterm::bookmarks::{Bookmark, BookmarkManager, BookmarkUpdate};
use std::path::PathBuf;

fn main() {
    println!("=== AgTerm Bookmark System Demo ===\n");

    // Create a new bookmark manager
    let mut manager = BookmarkManager::new();
    println!("✓ Created empty bookmark manager\n");

    // Add some bookmarks
    println!("Adding bookmarks...");
    let id1 = manager
        .add_bookmark(
            "Git Status",
            "git status",
            Some(PathBuf::from("/tmp")),
            vec!["git".to_string(), "vcs".to_string()],
            Some("Check repository status"),
        )
        .unwrap();
    println!("  ✓ Added 'Git Status' bookmark");

    let id2 = manager
        .add_bookmark(
            "Docker PS",
            "docker ps -a",
            None,
            vec!["docker".to_string(), "containers".to_string()],
            Some("List all containers"),
        )
        .unwrap();
    println!("  ✓ Added 'Docker PS' bookmark");

    let id3 = manager
        .add_bookmark(
            "Cargo Build",
            "cargo build --release",
            None,
            vec!["rust".to_string(), "build".to_string()],
            Some("Build in release mode"),
        )
        .unwrap();
    println!("  ✓ Added 'Cargo Build' bookmark\n");

    // List all bookmarks
    println!("All bookmarks:");
    for bookmark in manager.list_bookmarks() {
        println!(
            "  - {} ({})",
            bookmark.name, bookmark.command
        );
    }
    println!();

    // Search bookmarks
    println!("Searching for 'git':");
    for bookmark in manager.search_bookmarks("git") {
        println!("  - {}: {}", bookmark.name, bookmark.command);
    }
    println!();

    // Get bookmarks by tag
    println!("Bookmarks with 'docker' tag:");
    for bookmark in manager.get_by_tag("docker") {
        println!("  - {}: {}", bookmark.name, bookmark.command);
    }
    println!();

    // Record usage
    println!("Recording bookmark usage...");
    manager.record_use(id1).unwrap();
    manager.record_use(id1).unwrap();
    manager.record_use(id2).unwrap();
    println!("  ✓ Used 'Git Status' 2 times");
    println!("  ✓ Used 'Docker PS' 1 time\n");

    // Get most used
    println!("Most used bookmarks:");
    for bookmark in manager.get_most_used(5) {
        println!(
            "  - {} (used {} times)",
            bookmark.name, bookmark.use_count
        );
    }
    println!();

    // Update a bookmark
    println!("Updating bookmark...");
    let update = BookmarkUpdate::new()
        .name("Git Status Extended")
        .command("git status -sb");
    manager.update_bookmark(id1, update).unwrap();
    println!("  ✓ Updated 'Git Status' bookmark\n");

    // Get updated bookmark
    if let Some(bookmark) = manager.get_bookmark(id1) {
        println!("Updated bookmark:");
        println!("  Name: {}", bookmark.name);
        println!("  Command: {}", bookmark.command);
        println!("  Use count: {}", bookmark.use_count);
        println!();
    }

    // Create manager with defaults
    println!("Creating manager with default bookmarks...");
    let default_manager = BookmarkManager::with_defaults();
    println!("  ✓ Created with {} default bookmarks", default_manager.len());
    println!();

    println!("Available tags:");
    for tag in default_manager.get_all_tags() {
        let count = default_manager.get_by_tag(&tag).len();
        println!("  - {}: {} bookmarks", tag, count);
    }
    println!();

    // File persistence demo
    println!("Testing file persistence...");
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("agterm_bookmarks_demo.json");

    // Save to file
    manager.save_to_file(file_path.clone()).unwrap();
    println!("  ✓ Saved bookmarks to {}", file_path.display());

    // Load from file
    let mut loaded_manager = BookmarkManager::new();
    loaded_manager.load_from_file(file_path.clone()).unwrap();
    println!("  ✓ Loaded {} bookmarks from file", loaded_manager.len());

    // Clean up
    std::fs::remove_file(file_path).ok();
    println!("  ✓ Cleaned up demo file\n");

    println!("=== Demo Complete ===");
}
