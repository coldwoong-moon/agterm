//! Comprehensive usage examples for the AgTerm bookmark system
//!
//! This file demonstrates all major features and common use cases.

use agterm::bookmarks::{Bookmark, BookmarkManager, BookmarkUpdate};
use std::path::PathBuf;

fn main() {
    example_basic_usage();
    example_with_defaults();
    example_search_and_filter();
    example_usage_tracking();
    example_updates();
    example_persistence();
}

/// Basic bookmark creation and retrieval
fn example_basic_usage() {
    println!("=== Basic Usage ===\n");

    let mut manager = BookmarkManager::new();

    // Add a simple bookmark
    let id = manager
        .add_bookmark(
            "List Files",
            "ls -la",
            None,
            vec!["files".to_string()],
            Some("List all files with details"),
        )
        .expect("Failed to add bookmark");

    println!("✓ Created bookmark with ID: {}", id);

    // Retrieve the bookmark
    if let Some(bookmark) = manager.get_bookmark(id) {
        println!("Name: {}", bookmark.name);
        println!("Command: {}", bookmark.command);
        println!("Tags: {:?}", bookmark.tags);
    }

    println!();
}

/// Using default bookmarks
fn example_with_defaults() {
    println!("=== Using Default Bookmarks ===\n");

    let manager = BookmarkManager::with_defaults();

    println!("Loaded {} default bookmarks", manager.len());

    // List all git-related bookmarks
    println!("\nGit bookmarks:");
    for bookmark in manager.get_by_tag("git") {
        println!("  - {}: {}", bookmark.name, bookmark.command);
    }

    // List all available tags
    println!("\nAvailable tags:");
    for tag in manager.get_all_tags() {
        let count = manager.get_by_tag(&tag).len();
        println!("  - {} ({} bookmarks)", tag, count);
    }

    println!();
}

/// Search and filter operations
fn example_search_and_filter() {
    println!("=== Search and Filter ===\n");

    let mut manager = BookmarkManager::new();

    // Add several bookmarks
    manager
        .add_bookmark(
            "Git Status",
            "git status",
            None,
            vec!["git".to_string(), "status".to_string()],
            None::<String>,
        )
        .unwrap();

    manager
        .add_bookmark(
            "Git Log",
            "git log --oneline",
            None,
            vec!["git".to_string(), "log".to_string()],
            None::<String>,
        )
        .unwrap();

    manager
        .add_bookmark(
            "Docker PS",
            "docker ps -a",
            None,
            vec!["docker".to_string()],
            None::<String>,
        )
        .unwrap();

    // Search by keyword
    println!("Search results for 'git':");
    for bookmark in manager.search_bookmarks("git") {
        println!("  - {}: {}", bookmark.name, bookmark.command);
    }

    // Filter by tag
    println!("\nBookmarks tagged 'docker':");
    for bookmark in manager.get_by_tag("docker") {
        println!("  - {}: {}", bookmark.name, bookmark.command);
    }

    // List all bookmarks (sorted by name)
    println!("\nAll bookmarks:");
    for bookmark in manager.list_bookmarks() {
        println!("  - {}", bookmark.name);
    }

    println!();
}

/// Usage tracking and analytics
fn example_usage_tracking() {
    println!("=== Usage Tracking ===\n");

    let mut manager = BookmarkManager::new();

    // Add bookmarks
    let id1 = manager
        .add_bookmark("Command A", "echo a", None, vec![], None::<String>)
        .unwrap();
    let id2 = manager
        .add_bookmark("Command B", "echo b", None, vec![], None::<String>)
        .unwrap();
    let id3 = manager
        .add_bookmark("Command C", "echo c", None, vec![], None::<String>)
        .unwrap();

    // Record usage
    manager.record_use(id1).unwrap();
    manager.record_use(id1).unwrap();
    manager.record_use(id1).unwrap();
    manager.record_use(id2).unwrap();
    manager.record_use(id2).unwrap();
    manager.record_use(id3).unwrap();

    println!("Most frequently used bookmarks:");
    for bookmark in manager.get_most_used(5) {
        println!(
            "  - {} (used {} times)",
            bookmark.name, bookmark.use_count
        );
    }

    println!("\nMost recently used bookmarks:");
    for bookmark in manager.get_recent(5) {
        println!("  - {} (last used: {:?})", bookmark.name, bookmark.last_used);
    }

    println!();
}

/// Update operations
fn example_updates() {
    println!("=== Update Operations ===\n");

    let mut manager = BookmarkManager::new();

    let id = manager
        .add_bookmark(
            "Old Name",
            "old command",
            None,
            vec!["old-tag".to_string()],
            None::<String>,
        )
        .unwrap();

    println!("Original bookmark:");
    if let Some(bookmark) = manager.get_bookmark(id) {
        println!("  Name: {}", bookmark.name);
        println!("  Command: {}", bookmark.command);
        println!("  Tags: {:?}", bookmark.tags);
    }

    // Update multiple fields
    let update = BookmarkUpdate::new()
        .name("New Name")
        .command("new command")
        .tags(vec!["new-tag".to_string(), "updated".to_string()])
        .description(Some("Updated description".to_string()));

    manager.update_bookmark(id, update).unwrap();

    println!("\nUpdated bookmark:");
    if let Some(bookmark) = manager.get_bookmark(id) {
        println!("  Name: {}", bookmark.name);
        println!("  Command: {}", bookmark.command);
        println!("  Tags: {:?}", bookmark.tags);
        println!("  Description: {:?}", bookmark.description);
    }

    // Partial update (only name)
    let update = BookmarkUpdate::new().name("Another Name");
    manager.update_bookmark(id, update).unwrap();

    println!("\nAfter partial update:");
    if let Some(bookmark) = manager.get_bookmark(id) {
        println!("  Name: {} (updated)", bookmark.name);
        println!("  Command: {} (unchanged)", bookmark.command);
    }

    println!();
}

/// File persistence
fn example_persistence() {
    println!("=== File Persistence ===\n");

    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("agterm_bookmarks_example.json");

    // Create and save bookmarks
    {
        let mut manager = BookmarkManager::new();

        manager
            .add_bookmark(
                "Saved Bookmark 1",
                "echo hello",
                None,
                vec!["test".to_string()],
                Some("A saved bookmark"),
            )
            .unwrap();

        manager
            .add_bookmark(
                "Saved Bookmark 2",
                "echo world",
                Some(PathBuf::from("/tmp")),
                vec!["test".to_string()],
                None::<String>,
            )
            .unwrap();

        manager.save_to_file(file_path.clone()).unwrap();
        println!("✓ Saved {} bookmarks to {}", manager.len(), file_path.display());
    }

    // Load from file
    {
        let mut manager = BookmarkManager::new();
        manager.load_from_file(file_path.clone()).unwrap();

        println!("✓ Loaded {} bookmarks from file", manager.len());

        println!("\nLoaded bookmarks:");
        for bookmark in manager.list_bookmarks() {
            println!("  - {}: {}", bookmark.name, bookmark.command);
        }
    }

    // Load and save with configured path
    {
        let mut manager = BookmarkManager::new();
        manager.load_from_file(file_path.clone()).unwrap();

        // Add another bookmark
        manager
            .add_bookmark("New Bookmark", "echo new", None, vec![], None::<String>)
            .unwrap();

        // Save using configured path
        manager.save().unwrap();
        println!("\n✓ Saved using configured path");
    }

    // Clean up
    std::fs::remove_file(file_path).ok();
    println!("✓ Cleaned up example file\n");
}

/// Advanced usage patterns
#[allow(dead_code)]
fn example_advanced_patterns() {
    println!("=== Advanced Patterns ===\n");

    let mut manager = BookmarkManager::with_defaults();

    // Fuzzy search across multiple fields
    let query = "build";
    let results: Vec<_> = manager
        .list_bookmarks()
        .into_iter()
        .filter(|b| {
            b.name.to_lowercase().contains(query)
                || b.command.to_lowercase().contains(query)
                || b.tags.iter().any(|t| t.to_lowercase().contains(query))
        })
        .collect();

    println!("Fuzzy search for '{}': {} results", query, results.len());

    // Get bookmarks by multiple tags (OR operation)
    let tags = vec!["git", "docker"];
    let mut bookmarks_by_tags = Vec::new();
    for tag in tags {
        bookmarks_by_tags.extend(manager.get_by_tag(tag));
    }
    println!("Bookmarks with any of the tags: {}", bookmarks_by_tags.len());

    // Get most used bookmark in a specific category
    let git_bookmarks = manager.get_by_tag("git");
    let most_used_git = git_bookmarks
        .iter()
        .max_by_key(|b| b.use_count)
        .map(|b| b.name.as_str())
        .unwrap_or("none");
    println!("Most used git bookmark: {}", most_used_git);

    // Filter by working directory
    let with_workdir: Vec<_> = manager
        .list_bookmarks()
        .into_iter()
        .filter(|b| b.working_dir.is_some())
        .collect();
    println!("Bookmarks with working directory: {}", with_workdir.len());

    println!();
}
