//! Example demonstrating the terminal annotation system
//!
//! Run with: cargo run --example annotations_demo

use agterm::annotations::{Annotation, AnnotationManager, AnnotationType, LineRange};

fn main() {
    println!("=== AgTerm Annotation System Demo ===\n");

    // Create a new annotation manager
    let mut manager = AnnotationManager::new();

    println!("1. Creating annotations...");

    // Add a simple note
    let note = Annotation::note(10, "This is an important command".to_string());
    let note_id = manager.add(note);
    println!("   Added note at line 10: {}", note_id);

    // Add a warning
    let warning = Annotation::warning(25, "Check memory usage here".to_string());
    let warning_id = manager.add(warning);
    println!("   Added warning at line 25: {}", warning_id);

    // Add bookmarks for navigation
    let bookmark1 = Annotation::bookmark(50, "Start of main function".to_string());
    manager.add(bookmark1);
    println!("   Added bookmark at line 50");

    let bookmark2 = Annotation::bookmark(100, "Error handling section".to_string());
    manager.add(bookmark2);
    println!("   Added bookmark at line 100");

    let bookmark3 = Annotation::bookmark(150, "Cleanup code".to_string());
    manager.add(bookmark3);
    println!("   Added bookmark at line 150");

    // Add a multi-line annotation
    let multiline = Annotation::new(
        LineRange::new(70, 75),
        "Critical section with potential race condition".to_string(),
        AnnotationType::Warning,
    );
    manager.add(multiline);
    println!("   Added multi-line warning at lines 70-75");

    // Add an annotation with tags
    let mut tagged_note = Annotation::note(
        120,
        "Review this algorithm for optimization".to_string(),
    );
    tagged_note.add_tag("performance".to_string());
    tagged_note.add_tag("todo".to_string());
    manager.add(tagged_note);
    println!("   Added tagged note at line 120");

    // Display statistics
    println!("\n2. Statistics:");
    let stats = manager.stats();
    println!("   Total annotations: {}", stats.total);
    println!("   Notes: {}", stats.notes);
    println!("   Warnings: {}", stats.warnings);
    println!("   Bookmarks: {}", stats.bookmarks);

    // Query annotations
    println!("\n3. Querying annotations:");

    // Get annotations for a specific line
    println!("   Annotations at line 10:");
    for annotation in manager.get_for_line(10) {
        println!("     - [{}] {}", annotation.annotation_type.name(), annotation.content);
    }

    println!("   Annotations at line 72 (multi-line range):");
    for annotation in manager.get_for_line(72) {
        println!(
            "     - [{}] {} (lines {}-{})",
            annotation.annotation_type.name(),
            annotation.content,
            annotation.range.start,
            annotation.range.end
        );
    }

    // Search annotations
    println!("\n4. Searching annotations:");
    let search_results = manager.search("section");
    println!("   Found {} annotations containing 'section':", search_results.len());
    for annotation in search_results {
        println!(
            "     - Line {}: {}",
            annotation.range.start, annotation.content
        );
    }

    // Search by tag
    let tagged_results = manager.search_by_tag("performance");
    println!("\n   Found {} annotations tagged 'performance':", tagged_results.len());
    for annotation in tagged_results {
        println!(
            "     - Line {}: {}",
            annotation.range.start, annotation.content
        );
    }

    // Bookmark navigation
    println!("\n5. Bookmark navigation:");
    let bookmarks = manager.get_bookmarks();
    println!("   All bookmarks (sorted by line):");
    for bookmark in bookmarks {
        println!(
            "     - Line {}: {}",
            bookmark.range.start, bookmark.content
        );
    }

    println!("\n   Navigate from line 60:");
    if let Some(next) = manager.next_bookmark(60) {
        println!("     Next bookmark: line {} - {}", next.range.start, next.content);
    }
    if let Some(prev) = manager.prev_bookmark(60) {
        println!("     Previous bookmark: line {} - {}", prev.range.start, prev.content);
    }

    // Update an annotation
    println!("\n6. Updating annotations:");
    println!("   Original note: {}", manager.get(&note_id).unwrap().content);
    manager.update_content(&note_id, "Updated: This command needs review".to_string());
    println!("   Updated note: {}", manager.get(&note_id).unwrap().content);

    // Custom colors
    println!("\n7. Custom colors:");
    manager.update_color(&warning_id, [255, 0, 0]); // Red
    if let Some(warning) = manager.get(&warning_id) {
        let color = warning.effective_color();
        println!(
            "   Warning color: RGB({}, {}, {})",
            color[0], color[1], color[2]
        );
    }

    // List all annotations sorted by line
    println!("\n8. All annotations (sorted by line):");
    for annotation in manager.all_sorted() {
        let color = annotation.effective_color();
        let line_info = if annotation.range.is_single_line() {
            format!("Line {}", annotation.range.start)
        } else {
            format!("Lines {}-{}", annotation.range.start, annotation.range.end)
        };
        println!(
            "   {} [{}] {} RGB({},{},{})",
            line_info,
            annotation.annotation_type.symbol(),
            annotation.content,
            color[0],
            color[1],
            color[2]
        );
    }

    // Persistence demo
    println!("\n9. File persistence:");
    let temp_file = std::env::temp_dir().join("agterm_annotations_demo.json");
    println!("   Saving to: {:?}", temp_file);

    // Save annotations
    let mut manager_clone = manager.clone();
    manager_clone.set_file_path(temp_file.clone());
    manager_clone.save_to_file().unwrap();
    println!("   Saved {} annotations", manager_clone.count());

    // Load into a new manager
    let mut new_manager = AnnotationManager::new();
    new_manager.load_from_file(temp_file.clone()).unwrap();
    println!("   Loaded {} annotations", new_manager.count());

    // Verify they match
    let original_stats = manager.stats();
    let loaded_stats = new_manager.stats();
    println!("   Verification:");
    println!("     Original: {} total", original_stats.total);
    println!("     Loaded: {} total", loaded_stats.total);
    println!("     Match: {}", original_stats.total == loaded_stats.total);

    // Clean up
    if let Err(e) = std::fs::remove_file(&temp_file) {
        eprintln!("   Warning: Could not remove temp file: {}", e);
    }

    // Clear by type
    println!("\n10. Clearing annotations:");
    let before = manager.count();
    manager.clear_by_type(AnnotationType::Note);
    let after = manager.count();
    println!("   Cleared all notes: {} -> {} annotations", before, after);

    // Final clear
    manager.clear();
    println!("   Cleared all annotations: {} remaining", manager.count());

    println!("\n=== Demo Complete ===");
}
