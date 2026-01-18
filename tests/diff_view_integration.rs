//! Integration tests for the diff viewer module

use agterm::diff_view::{
    diff_strings, DiffLine, DiffLineType, DiffResult, DiffViewMode, DiffViewer, MyersDiff,
};

#[test]
fn test_simple_diff() {
    let old = "line1\nline2\nline3";
    let new = "line1\nmodified\nline3";

    let old_lines: Vec<String> = old.lines().map(|s| s.to_string()).collect();
    let new_lines: Vec<String> = new.lines().map(|s| s.to_string()).collect();

    let diff = MyersDiff::new(old_lines, new_lines);
    let result = diff.compute();

    assert_eq!(result.stats.unchanged, 2);
    assert_eq!(result.stats.modified, 1);
}

#[test]
fn test_all_operations() {
    let old = vec![
        "keep".to_string(),
        "remove".to_string(),
        "modify_old".to_string(),
    ];
    let new = vec![
        "keep".to_string(),
        "modify_new".to_string(),
        "add".to_string(),
    ];

    let diff = MyersDiff::new(old, new);
    let result = diff.compute();

    // Should have: keep unchanged, remove deleted, modify changed, add added
    assert_eq!(result.stats.unchanged, 1); // "keep"
    assert!(result.stats.added >= 1); // "add" and possibly part of modify
    assert!(result.stats.removed >= 1); // "remove" and possibly part of modify
}

#[test]
fn test_diff_viewer_modes() {
    let lines = vec![
        DiffLine::unchanged("line1".to_string(), 1, 1),
        DiffLine::added("line2".to_string(), 2),
    ];

    let result = DiffResult::new(lines);
    let mut viewer = DiffViewer::new(result, 80);

    // Test side-by-side
    viewer.set_mode(DiffViewMode::SideBySide);
    let output = viewer.render();
    assert!(output.contains("Old"));
    assert!(output.contains("New"));
    assert!(output.contains("|"));

    // Test unified
    viewer.set_mode(DiffViewMode::Unified);
    let output = viewer.render();
    assert!(output.contains("Unified Diff"));
}

#[test]
fn test_navigation() {
    let lines = vec![
        DiffLine::unchanged("line1".to_string(), 1, 1),
        DiffLine::added("line2".to_string(), 2),
        DiffLine::unchanged("line3".to_string(), 2, 3),
        DiffLine::removed("line4".to_string(), 3),
    ];

    let result = DiffResult::new(lines);
    let mut viewer = DiffViewer::new(result, 80);

    assert_eq!(viewer.current_line(), 0);
    assert!(viewer.next_change());
    assert_eq!(viewer.current_line(), 1);
    assert!(viewer.next_change());
    assert_eq!(viewer.current_line(), 3);
    assert!(!viewer.next_change()); // No more changes
    assert!(viewer.prev_change());
    assert_eq!(viewer.current_line(), 1);
}

#[test]
fn test_diff_strings_convenience() {
    let old = "Hello\nWorld";
    let new = "Hello\nRust";

    let output = diff_strings(old, new, 80);

    // Should contain the expected content
    assert!(output.contains("Hello"));
    assert!(output.contains("World") || output.contains("Rust"));
}

#[test]
fn test_empty_inputs() {
    let diff = MyersDiff::new(vec![], vec![]);
    let result = diff.compute();

    assert_eq!(result.stats.total_lines(), 0);
    assert_eq!(result.stats.total_changes(), 0);
}

#[test]
fn test_large_diff() {
    let old: Vec<String> = (0..100).map(|i| format!("line{}", i)).collect();
    let mut new = old.clone();
    new[50] = "modified".to_string();
    new.push("added".to_string());

    let diff = MyersDiff::new(old, new);
    let result = diff.compute();

    assert_eq!(result.stats.unchanged, 99);
    assert_eq!(result.stats.modified, 1);
    assert_eq!(result.stats.added, 1);
}

#[test]
fn test_change_indices() {
    let lines = vec![
        DiffLine::unchanged("a".to_string(), 1, 1),
        DiffLine::added("b".to_string(), 2),
        DiffLine::unchanged("c".to_string(), 2, 3),
        DiffLine::removed("d".to_string(), 3),
        DiffLine::modified("e".to_string(), "f".to_string(), 4, 4),
    ];

    let result = DiffResult::new(lines);
    let changes = result.change_indices();

    assert_eq!(changes, vec![1, 3, 4]);
}

#[test]
fn test_diff_result_navigation() {
    let lines = vec![
        DiffLine::unchanged("a".to_string(), 1, 1),
        DiffLine::added("b".to_string(), 2),
        DiffLine::unchanged("c".to_string(), 2, 3),
        DiffLine::removed("d".to_string(), 3),
    ];

    let result = DiffResult::new(lines);

    assert_eq!(result.next_change(0), Some(1));
    assert_eq!(result.next_change(1), Some(3));
    assert_eq!(result.next_change(3), None);

    assert_eq!(result.prev_change(3), Some(1));
    assert_eq!(result.prev_change(1), None);
}

#[test]
fn test_line_numbers() {
    let old = vec!["a".to_string(), "b".to_string()];
    let new = vec!["a".to_string(), "c".to_string()];

    let diff = MyersDiff::new(old, new);
    let result = diff.compute();

    for line in &result.lines {
        match line.line_type {
            DiffLineType::Unchanged => {
                assert!(line.left_line_num.is_some());
                assert!(line.right_line_num.is_some());
            }
            DiffLineType::Added => {
                assert!(line.left_line_num.is_none());
                assert!(line.right_line_num.is_some());
            }
            DiffLineType::Removed => {
                assert!(line.left_line_num.is_some());
                assert!(line.right_line_num.is_none());
            }
            DiffLineType::Modified => {
                assert!(line.left_line_num.is_some());
                assert!(line.right_line_num.is_some());
            }
        }
    }
}

#[test]
fn test_viewer_set_current_line() {
    let lines = vec![
        DiffLine::unchanged("a".to_string(), 1, 1),
        DiffLine::added("b".to_string(), 2),
        DiffLine::unchanged("c".to_string(), 2, 3),
    ];

    let result = DiffResult::new(lines);
    let mut viewer = DiffViewer::new(result, 80);

    viewer.set_current_line(2);
    assert_eq!(viewer.current_line(), 2);

    // Should not set beyond bounds
    viewer.set_current_line(100);
    assert_eq!(viewer.current_line(), 2); // Unchanged
}

#[test]
fn test_consecutive_changes() {
    let old = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let new = vec!["x".to_string(), "y".to_string(), "z".to_string()];

    let diff = MyersDiff::new(old, new);
    let result = diff.compute();

    // Due to Myers algorithm behavior with all-different content,
    // it processes as deletions + insertions with modification pairing
    // Total changes = removed + added + modified
    // Exact count depends on algorithm's pairing of delete/insert operations
    assert!(result.stats.total_changes() >= 3); // At least all original lines are changed
    assert_eq!(result.stats.unchanged, 0);
}

#[test]
fn test_real_world_code_diff() {
    let old_code = "\
fn hello() {
    println!(\"Hello\");
}

fn main() {
    hello();
}";

    let new_code = "\
fn hello(name: &str) {
    println!(\"Hello, {}!\", name);
}

fn main() {
    hello(\"World\");
}";

    let output = diff_strings(old_code, new_code, 100);

    // Check that the diff was generated
    assert!(!output.is_empty());
    assert!(output.contains("fn"));
    assert!(output.contains("main"));
}
