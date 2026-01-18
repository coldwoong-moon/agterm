//! Integration tests for the annotation system
//!
//! Run with: cargo test --test annotations_integration_test

use agterm::annotations::{Annotation, AnnotationManager, AnnotationType, LineRange};
use tempfile::NamedTempFile;

#[test]
fn test_full_annotation_workflow() {
    let mut manager = AnnotationManager::new();

    // Create various annotations
    let note_id = manager.add(Annotation::note(10, "Test note".to_string()));
    let warning_id = manager.add(Annotation::warning(20, "Test warning".to_string()));
    let bookmark_id = manager.add(Annotation::bookmark(30, "Test bookmark".to_string()));

    // Verify they exist
    assert_eq!(manager.count(), 3);
    assert!(manager.get(&note_id).is_some());
    assert!(manager.get(&warning_id).is_some());
    assert!(manager.get(&bookmark_id).is_some());

    // Update content
    assert!(manager.update_content(&note_id, "Updated note".to_string()));
    assert_eq!(manager.get(&note_id).unwrap().content, "Updated note");

    // Query by line
    let line10_annotations = manager.get_for_line(10);
    assert_eq!(line10_annotations.len(), 1);
    assert_eq!(line10_annotations[0].content, "Updated note");

    // Remove one
    assert!(manager.remove(&warning_id).is_some());
    assert_eq!(manager.count(), 2);
    assert!(manager.get(&warning_id).is_none());
}

#[test]
fn test_multi_line_annotations() {
    let mut manager = AnnotationManager::new();

    let annotation = Annotation::new(
        LineRange::new(10, 15),
        "Multi-line annotation".to_string(),
        AnnotationType::Warning,
    );
    manager.add(annotation);

    // All lines in range should have the annotation
    for line in 10..=15 {
        assert!(manager.has_annotations_at_line(line));
        let annotations = manager.get_for_line(line);
        assert_eq!(annotations.len(), 1);
        assert_eq!(annotations[0].content, "Multi-line annotation");
    }

    // Lines outside range should not have it
    assert!(!manager.has_annotations_at_line(9));
    assert!(!manager.has_annotations_at_line(16));
}

#[test]
fn test_bookmark_navigation() {
    let mut manager = AnnotationManager::new();

    // Add bookmarks at various lines
    manager.add(Annotation::bookmark(10, "First".to_string()));
    manager.add(Annotation::bookmark(30, "Second".to_string()));
    manager.add(Annotation::bookmark(50, "Third".to_string()));

    // Test navigation forward
    let next = manager.next_bookmark(5);
    assert_eq!(next.unwrap().range.start, 10);

    let next = manager.next_bookmark(10);
    assert_eq!(next.unwrap().range.start, 30);

    let next = manager.next_bookmark(50);
    assert!(next.is_none()); // No more bookmarks after 50

    // Test navigation backward
    let prev = manager.prev_bookmark(60);
    assert_eq!(prev.unwrap().range.start, 50);

    let prev = manager.prev_bookmark(30);
    assert_eq!(prev.unwrap().range.start, 10);

    let prev = manager.prev_bookmark(10);
    assert!(prev.is_none()); // No bookmarks before 10
}

#[test]
fn test_search_functionality() {
    let mut manager = AnnotationManager::new();

    manager.add(Annotation::note(10, "git commit".to_string()));
    manager.add(Annotation::note(20, "git push".to_string()));
    manager.add(Annotation::note(30, "docker build".to_string()));
    manager.add(Annotation::note(40, "docker run".to_string()));

    // Case-insensitive search
    let git_results = manager.search("GIT");
    assert_eq!(git_results.len(), 2);

    let docker_results = manager.search("docker");
    assert_eq!(docker_results.len(), 2);

    let no_results = manager.search("kubernetes");
    assert_eq!(no_results.len(), 0);
}

#[test]
fn test_tag_functionality() {
    let mut manager = AnnotationManager::new();

    // Create annotations with tags
    let mut ann1 = Annotation::note(10, "Note 1".to_string());
    ann1.add_tag("todo".to_string());
    ann1.add_tag("important".to_string());
    manager.add(ann1);

    let mut ann2 = Annotation::note(20, "Note 2".to_string());
    ann2.add_tag("todo".to_string());
    manager.add(ann2);

    let mut ann3 = Annotation::note(30, "Note 3".to_string());
    ann3.add_tag("important".to_string());
    manager.add(ann3);

    // Search by tag
    let todo_annotations = manager.search_by_tag("todo");
    assert_eq!(todo_annotations.len(), 2);

    let important_annotations = manager.search_by_tag("important");
    assert_eq!(important_annotations.len(), 2);

    let nonexistent = manager.search_by_tag("nonexistent");
    assert_eq!(nonexistent.len(), 0);
}

#[test]
fn test_persistence() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();

    // Create manager and add annotations
    let mut manager1 = AnnotationManager::new();
    manager1.add(Annotation::note(10, "Note 1".to_string()));
    manager1.add(Annotation::warning(20, "Warning 1".to_string()));
    manager1.add(Annotation::bookmark(30, "Bookmark 1".to_string()));

    // Save to file
    manager1.set_file_path(path.clone());
    manager1.save_to_file().unwrap();

    // Load into new manager
    let mut manager2 = AnnotationManager::new();
    manager2.load_from_file(path).unwrap();

    // Verify they match
    assert_eq!(manager1.count(), manager2.count());
    assert_eq!(
        manager1.count_by_type(AnnotationType::Note),
        manager2.count_by_type(AnnotationType::Note)
    );
    assert_eq!(
        manager1.count_by_type(AnnotationType::Warning),
        manager2.count_by_type(AnnotationType::Warning)
    );
    assert_eq!(
        manager1.count_by_type(AnnotationType::Bookmark),
        manager2.count_by_type(AnnotationType::Bookmark)
    );

    // Verify line mappings work
    assert!(manager2.has_annotations_at_line(10));
    assert!(manager2.has_annotations_at_line(20));
    assert!(manager2.has_annotations_at_line(30));
}

#[test]
fn test_statistics() {
    let mut manager = AnnotationManager::new();

    manager.add(Annotation::note(10, "Note 1".to_string()));
    manager.add(Annotation::note(11, "Note 2".to_string()));
    manager.add(Annotation::warning(20, "Warning 1".to_string()));
    manager.add(Annotation::bookmark(30, "Bookmark 1".to_string()));

    let stats = manager.stats();
    assert_eq!(stats.total, 4);
    assert_eq!(stats.notes, 2);
    assert_eq!(stats.warnings, 1);
    assert_eq!(stats.bookmarks, 1);

    // Verify count methods match stats
    assert_eq!(manager.count(), stats.total);
    assert_eq!(
        manager.count_by_type(AnnotationType::Note),
        stats.notes
    );
    assert_eq!(
        manager.count_by_type(AnnotationType::Warning),
        stats.warnings
    );
    assert_eq!(
        manager.count_by_type(AnnotationType::Bookmark),
        stats.bookmarks
    );
}

#[test]
fn test_clear_operations() {
    let mut manager = AnnotationManager::new();

    manager.add(Annotation::note(10, "Note 1".to_string()));
    manager.add(Annotation::note(11, "Note 2".to_string()));
    manager.add(Annotation::warning(20, "Warning 1".to_string()));
    manager.add(Annotation::bookmark(30, "Bookmark 1".to_string()));

    // Clear by type
    assert_eq!(manager.count(), 4);
    manager.clear_by_type(AnnotationType::Note);
    assert_eq!(manager.count(), 2);
    assert_eq!(manager.count_by_type(AnnotationType::Note), 0);

    // Clear all
    manager.clear();
    assert_eq!(manager.count(), 0);
    assert!(!manager.has_annotations_at_line(20));
    assert!(!manager.has_annotations_at_line(30));
}

#[test]
fn test_max_annotations_trimming() {
    let mut manager = AnnotationManager::with_max_annotations(3);

    // Add more than max
    manager.add(Annotation::note(10, "Note 1".to_string()));
    std::thread::sleep(std::time::Duration::from_millis(10));
    manager.add(Annotation::note(11, "Note 2".to_string()));
    std::thread::sleep(std::time::Duration::from_millis(10));
    manager.add(Annotation::note(12, "Note 3".to_string()));
    std::thread::sleep(std::time::Duration::from_millis(10));

    assert_eq!(manager.count(), 3);

    // Adding one more should trigger trimming
    manager.add(Annotation::note(13, "Note 4".to_string()));
    assert_eq!(manager.count(), 3);

    // Oldest should be removed
    assert!(!manager.has_annotations_at_line(10));
    assert!(manager.has_annotations_at_line(11));
    assert!(manager.has_annotations_at_line(12));
    assert!(manager.has_annotations_at_line(13));
}

#[test]
fn test_sorted_output() {
    let mut manager = AnnotationManager::new();

    // Add in random order
    manager.add(Annotation::note(50, "Third".to_string()));
    manager.add(Annotation::note(10, "First".to_string()));
    manager.add(Annotation::note(30, "Second".to_string()));

    // Should be sorted by line number
    let sorted = manager.all_sorted();
    assert_eq!(sorted.len(), 3);
    assert_eq!(sorted[0].range.start, 10);
    assert_eq!(sorted[1].range.start, 30);
    assert_eq!(sorted[2].range.start, 50);
}

#[test]
fn test_annotation_colors() {
    let mut manager = AnnotationManager::new();

    let mut note = Annotation::note(10, "Test".to_string());
    let default_color = note.effective_color();
    assert_eq!(default_color, AnnotationType::Note.default_color());

    // Set custom color
    let custom_color = [255, 0, 0];
    note.set_color(custom_color);
    let id = manager.add(note);

    // Update color through manager
    manager.update_color(&id, [0, 255, 0]);
    let updated = manager.get(&id).unwrap();
    assert_eq!(updated.effective_color(), [0, 255, 0]);
}

#[test]
fn test_line_range_operations() {
    // Single line
    let single = LineRange::single(5);
    assert!(single.is_single_line());
    assert_eq!(single.len(), 1);
    assert!(single.contains(5));
    assert!(!single.contains(4));
    assert!(!single.contains(6));

    // Multi-line
    let range = LineRange::new(10, 20);
    assert!(!range.is_single_line());
    assert_eq!(range.len(), 11);
    assert!(range.contains(10));
    assert!(range.contains(15));
    assert!(range.contains(20));
    assert!(!range.contains(9));
    assert!(!range.contains(21));

    // Overlap detection
    let range1 = LineRange::new(10, 20);
    let range2 = LineRange::new(15, 25);
    let range3 = LineRange::new(30, 40);

    assert!(range1.overlaps(&range2));
    assert!(range2.overlaps(&range1));
    assert!(!range1.overlaps(&range3));
    assert!(!range3.overlaps(&range1));
}
