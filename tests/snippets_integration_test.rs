//! Integration tests for the snippet system

use agterm::snippets::{Snippet, SnippetManager, SnippetError};
use std::collections::HashMap;
use tempfile::tempdir;

#[test]
fn test_full_workflow() {
    // Create a manager
    let mut manager = SnippetManager::new();

    // Add a snippet
    let snippet = Snippet::new(
        "Test Function",
        "Creates a test function",
        "test",
        "#[test]\nfn ${name}() {\n    $0\n}",
        "rust",
    )
    .with_tag("test")
    .with_tag("function");

    let id = snippet.id.clone();
    assert!(manager.add_snippet(snippet).is_ok());

    // Find by trigger
    let found = manager.find_exact_trigger("test");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Test Function");

    // Expand template
    let mut values = HashMap::new();
    values.insert("name".to_string(), "my_test".to_string());

    let snippet = manager.get_snippet(&id).unwrap();
    let (expanded, cursor) = manager.expand_template(&snippet.template, &values);

    assert!(expanded.contains("fn my_test()"));
    assert!(cursor.is_some());

    // Remove snippet
    let removed = manager.remove_snippet(&id);
    assert!(removed.is_ok());
    assert!(manager.get_snippet(&id).is_none());
}

#[test]
fn test_persistence_workflow() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("snippets.json");

    // Create and save
    {
        let mut manager = SnippetManager::new();
        manager
            .add_snippet(Snippet::new(
                "Function",
                "desc",
                "fn",
                "fn ${name}() {}",
                "rust",
            ))
            .unwrap();

        assert!(manager.save_to_file(&file_path).is_ok());
    }

    // Load in new manager
    {
        let mut manager = SnippetManager::new();
        assert!(manager.load_from_file(&file_path).is_ok());

        let snippets = manager.get_all_snippets();
        assert_eq!(snippets.len(), 1);
        assert_eq!(snippets[0].name, "Function");
    }
}

#[test]
fn test_category_organization() {
    let mut manager = SnippetManager::new();

    // Add snippets in different categories
    manager
        .add_snippet(Snippet::new("R1", "d", "r1", "t", "rust"))
        .unwrap();
    manager
        .add_snippet(Snippet::new("R2", "d", "r2", "t", "rust"))
        .unwrap();
    manager
        .add_snippet(Snippet::new("B1", "d", "b1", "t", "bash"))
        .unwrap();

    let categories = manager.get_categories();
    assert_eq!(categories.len(), 2);
    assert!(categories.contains(&"rust".to_string()));
    assert!(categories.contains(&"bash".to_string()));

    let rust_snippets = manager.get_by_category("rust");
    assert_eq!(rust_snippets.len(), 2);

    let bash_snippets = manager.get_by_category("bash");
    assert_eq!(bash_snippets.len(), 1);
}

#[test]
fn test_autocomplete_workflow() {
    let mut manager = SnippetManager::new();

    // Add several snippets with similar triggers
    manager
        .add_snippet(Snippet::new("T1", "d", "test", "t", "rust"))
        .unwrap();
    manager
        .add_snippet(Snippet::new("T2", "d", "test2", "t", "rust"))
        .unwrap();
    manager
        .add_snippet(Snippet::new("F1", "d", "fn", "t", "rust"))
        .unwrap();

    // User types "te" - should match test and test2
    let matches = manager.find_by_trigger("te");
    assert_eq!(matches.len(), 2);
    assert!(matches.iter().all(|s| s.trigger.starts_with("te")));

    // User types "test" - should match test and test2
    let matches = manager.find_by_trigger("test");
    assert_eq!(matches.len(), 2);

    // User types "fn" - should match fn exactly
    let matches = manager.find_by_trigger("fn");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].trigger, "fn");

    // Exact match
    let exact = manager.find_exact_trigger("test");
    assert!(exact.is_some());
    assert_eq!(exact.unwrap().name, "T1");
}

#[test]
fn test_complex_template_expansion() {
    let manager = SnippetManager::new();

    let template = r#"fn ${name}($1) -> ${return:Result<(), Error>} {
    // TODO: ${name}
    $2
    $0
}"#;

    let mut values = HashMap::new();
    values.insert("name".to_string(), "process".to_string());
    values.insert("1".to_string(), "data: &str".to_string());
    values.insert("2".to_string(), "println!(\"data: {}\", data);".to_string());
    // Note: 'return' is not provided, should use default

    let (expanded, cursor) = manager.expand_template(template, &values);

    assert!(expanded.contains("fn process("));
    assert!(expanded.contains("data: &str"));
    assert!(expanded.contains("Result<(), Error>"));
    assert!(expanded.contains("TODO: process"));
    assert!(expanded.contains("println!"));
    assert!(cursor.is_some());
}

#[test]
fn test_default_snippets_quality() {
    let manager = SnippetManager::with_defaults();

    // Should have multiple categories
    let categories = manager.get_categories();
    assert!(categories.len() >= 3); // rust, bash, git, docker

    // Each category should have snippets
    for category in categories {
        let snippets = manager.get_by_category(&category);
        assert!(
            !snippets.is_empty(),
            "Category {} should have snippets",
            category
        );

        // Each snippet should be well-formed
        for snippet in snippets {
            assert!(!snippet.name.is_empty());
            assert!(!snippet.trigger.is_empty());
            assert!(!snippet.template.is_empty());
            assert!(!snippet.description.is_empty());
        }
    }

    // Test specific important triggers
    assert!(manager.find_exact_trigger("fn").is_some());
    assert!(manager.find_exact_trigger("test").is_some());
    assert!(manager.find_exact_trigger("if").is_some());
    assert!(manager.find_exact_trigger("for").is_some());
}

#[test]
fn test_duplicate_trigger_prevention() {
    let mut manager = SnippetManager::new();

    let snippet1 = Snippet::new("S1", "d", "dup", "t", "cat");
    let snippet2 = Snippet::new("S2", "d", "dup", "t", "cat");

    assert!(manager.add_snippet(snippet1).is_ok());

    match manager.add_snippet(snippet2) {
        Err(SnippetError::DuplicateTrigger(trigger)) => {
            assert_eq!(trigger, "dup");
        }
        _ => panic!("Should have returned DuplicateTrigger error"),
    }
}

#[test]
fn test_update_preserves_id() {
    let mut manager = SnippetManager::new();

    let original = Snippet::new("Original", "d", "orig", "t", "cat");
    let id = original.id.clone();

    manager.add_snippet(original).unwrap();

    // Update with different data
    let updated = Snippet::new("Updated", "new", "new", "new", "new");
    manager.update_snippet(&id, updated).unwrap();

    // ID should remain the same
    let retrieved = manager.get_snippet(&id);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, id);
    assert_eq!(retrieved.unwrap().name, "Updated");
}

#[test]
fn test_placeholder_edge_cases() {
    let manager = SnippetManager::new();

    // Empty template
    let (result, cursor) = manager.expand_template("", &HashMap::new());
    assert_eq!(result, "");
    assert_eq!(cursor, Some(0));

    // Only text, no placeholders
    let (result, cursor) = manager.expand_template("Hello World", &HashMap::new());
    assert_eq!(result, "Hello World");
    assert_eq!(cursor, Some(11));

    // Dollar sign at end
    let (result, _) = manager.expand_template("Price: $", &HashMap::new());
    assert_eq!(result, "Price: $");

    // Multiple $0 placeholders (last one is used due to parse overwrite)
    let (result, cursor) = manager.expand_template("Start $0 middle $0 end", &HashMap::new());
    assert!(cursor.is_some());
    // Result is "Start  middle  end" (both $0 removed, cursor at last $0 position)
    // "Start  middle " = 14 characters
    assert_eq!(result, "Start  middle  end");
    assert_eq!(cursor.unwrap(), 14);
}
