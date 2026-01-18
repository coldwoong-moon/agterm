//! Clipboard History Demo
//!
//! Demonstrates the clipboard history functionality including:
//! - Adding and retrieving entries
//! - Pinning/unpinning items
//! - Content type detection
//! - Search functionality
//! - File persistence

use agterm::clipboard_history::{ClipboardHistory, ClipboardType};
use std::path::PathBuf;

fn main() {
    println!("=== Clipboard History Demo ===\n");

    // Create a new clipboard history with max 100 entries
    let mut history = ClipboardHistory::new(100);

    // Add various types of content
    println!("1. Adding clipboard entries...");
    history.add("https://github.com/rust-lang/rust".to_string(), Some("browser".to_string()));
    history.add("cargo build --release".to_string(), Some("terminal".to_string()));
    history.add("/Users/username/Documents/file.txt".to_string(), Some("finder".to_string()));
    history.add("user@example.com".to_string(), Some("mail-app".to_string()));
    history.add("fn main() { println!(\"Hello\"); }".to_string(), Some("editor".to_string()));
    history.add("Plain text content".to_string(), None);

    println!("   Added {} entries", history.len());
    println!();

    // Show content type detection
    println!("2. Content type detection:");
    for (i, entry) in history.all().iter().enumerate() {
        println!("   [{}] {:?} - {}", i, entry.content_type, entry.preview(40));
    }
    println!();

    // Pin an important entry
    println!("3. Pinning entry...");
    history.pin(0, Some("Important URL".to_string()));
    println!("   Pinned: {}", history.get(0).unwrap().content);
    println!("   Pinned count: {}", history.pinned_count());
    println!();

    // Search functionality
    println!("4. Searching for 'file':");
    let results = history.search("file");
    for result in results {
        println!("   Found: {}", result.preview(50));
    }
    println!();

    // Filter by type
    println!("5. Filtering by ClipboardType::Url:");
    let urls = history.filter_by_type(ClipboardType::Url);
    for url in urls {
        println!("   URL: {}", url.content);
    }
    println!();

    // Show recent entries
    println!("6. Recent 3 entries:");
    for entry in history.recent(3) {
        println!("   - {} (from: {:?})", entry.preview(40), entry.source);
    }
    println!();

    // Show pinned entries
    println!("7. Pinned entries:");
    for entry in history.pinned() {
        println!("   - {} [{}]", entry.preview(40), entry.label.as_ref().unwrap_or(&"(no label)".to_string()));
    }
    println!();

    // Demonstrate deduplication
    println!("8. Testing deduplication:");
    let initial_len = history.len();
    history.add("https://github.com/rust-lang/rust".to_string(), Some("browser".to_string())); // Duplicate
    println!("   Before duplicate: {}, After: {} (duplicate removed)", initial_len, history.len());
    println!();

    // File persistence
    println!("9. File persistence:");
    let temp_path = PathBuf::from("/tmp/agterm_clipboard_demo.json");
    history.load_from_file(temp_path.clone()).unwrap();
    history.save_to_file().unwrap();
    println!("   Saved to: {:?}", temp_path);

    // Load in new instance
    let mut history2 = ClipboardHistory::new(100);
    history2.load_from_file(temp_path).unwrap();
    println!("   Loaded {} entries in new instance", history2.len());
    println!();

    println!("=== Demo Complete ===");
}
