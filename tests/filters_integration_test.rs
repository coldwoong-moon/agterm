//! Integration tests for the filter system

use agterm::filters::{Filter, FilterAction, FilterManager, FilterProcessor};

#[test]
fn test_basic_filter_workflow() {
    let mut processor = FilterProcessor::new();

    let filter = Filter::new(
        "test".to_string(),
        "Test Filter".to_string(),
        r"error".to_string(),
        FilterAction::Hide,
    )
    .unwrap();

    processor.manager_mut().add_filter(filter).unwrap();

    let result = processor.process_line("This has error in it");
    assert!(result.hidden);

    let result = processor.process_line("This is fine");
    assert!(!result.hidden);
}

#[test]
fn test_highlight_filter() {
    let mut processor = FilterProcessor::new();

    let filter = Filter::new(
        "highlight".to_string(),
        "Highlight Test".to_string(),
        r"important".to_string(),
        FilterAction::Highlight {
            color: (255, 0, 0),
            bg_color: None,
        },
    )
    .unwrap();

    processor.manager_mut().add_filter(filter).unwrap();

    let result = processor.process_line("This is important information");
    assert!(!result.hidden);
    assert_eq!(result.highlights.len(), 1);
    assert_eq!(result.highlights[0].color, (255, 0, 0));
}

#[test]
fn test_replace_filter() {
    let mut processor = FilterProcessor::new();

    let filter = Filter::new(
        "replace".to_string(),
        "Replace Test".to_string(),
        r"secret".to_string(),
        FilterAction::Replace {
            replacement: "***".to_string(),
        },
    )
    .unwrap();

    processor.manager_mut().add_filter(filter).unwrap();

    let result = processor.process_line("password=secret");
    assert!(!result.hidden);
    assert!(result.text.contains("***"));
    assert!(!result.text.contains("secret"));
}

#[test]
fn test_filter_priority() {
    let mut processor = FilterProcessor::new();

    let mut filter1 = Filter::new(
        "low".to_string(),
        "Low Priority".to_string(),
        r"test".to_string(),
        FilterAction::Replace {
            replacement: "LOW".to_string(),
        },
    )
    .unwrap();
    filter1.priority = 1;

    let mut filter2 = Filter::new(
        "high".to_string(),
        "High Priority".to_string(),
        r"test".to_string(),
        FilterAction::Replace {
            replacement: "HIGH".to_string(),
        },
    )
    .unwrap();
    filter2.priority = 10;

    processor.manager_mut().add_filter(filter1).unwrap();
    processor.manager_mut().add_filter(filter2).unwrap();

    let result = processor.process_line("test");
    // High priority filter runs first: "test" -> "HIGH"
    // Then low priority filter doesn't match "HIGH" (pattern is "test")
    // So final result is "HIGH"
    assert_eq!(result.text, "HIGH");
}

#[test]
fn test_filter_groups() {
    let mut processor = FilterProcessor::new();

    let mut filter1 = Filter::new(
        "f1".to_string(),
        "Filter 1".to_string(),
        r"a".to_string(),
        FilterAction::Hide,
    )
    .unwrap();
    filter1.group = Some("group1".to_string());

    let mut filter2 = Filter::new(
        "f2".to_string(),
        "Filter 2".to_string(),
        r"b".to_string(),
        FilterAction::Hide,
    )
    .unwrap();
    filter2.group = Some("group1".to_string());

    processor.manager_mut().add_filter(filter1).unwrap();
    processor.manager_mut().add_filter(filter2).unwrap();

    let result = processor.process_line("has a");
    assert!(result.hidden);

    processor.manager_mut().disable_group("group1").unwrap();

    let result = processor.process_line("has a");
    assert!(!result.hidden);
}

#[test]
fn test_case_insensitive() {
    let filter = Filter::new_case_insensitive(
        "ci".to_string(),
        "Case Insensitive".to_string(),
        r"error".to_string(),
        FilterAction::Hide,
    )
    .unwrap();

    assert!(filter.matches("ERROR"));
    assert!(filter.matches("error"));
    assert!(filter.matches("Error"));
}

#[test]
fn test_statistics() {
    let mut processor = FilterProcessor::new();

    let filter = Filter::new(
        "stats".to_string(),
        "Stats Test".to_string(),
        r"error".to_string(),
        FilterAction::Highlight {
            color: (255, 0, 0),
            bg_color: None,
        },
    )
    .unwrap();

    processor.manager_mut().add_filter(filter).unwrap();

    processor.process_line("error 1");
    processor.process_line("ok");
    processor.process_line("error 2");
    processor.process_line("error 3");

    assert_eq!(processor.manager().total_matches(), 3);
}

#[test]
fn test_export_import() {
    let mut manager = FilterManager::new();

    let filter = Filter::new(
        "export_test".to_string(),
        "Export Test".to_string(),
        r"test".to_string(),
        FilterAction::Hide,
    )
    .unwrap();

    manager.add_filter(filter).unwrap();

    let json = manager.export_json().unwrap();

    let mut new_manager = FilterManager::new();
    let count = new_manager.import_json(&json).unwrap();

    assert_eq!(count, 1);
    assert!(new_manager.get_filter("export_test").is_some());
}

#[test]
fn test_processor_toggle() {
    let mut processor = FilterProcessor::new();

    let filter = Filter::new(
        "toggle".to_string(),
        "Toggle Test".to_string(),
        r"hide".to_string(),
        FilterAction::Hide,
    )
    .unwrap();

    processor.manager_mut().add_filter(filter).unwrap();

    let result = processor.process_line("hide this");
    assert!(result.hidden);

    processor.disable();

    let result = processor.process_line("hide this");
    assert!(!result.hidden);

    processor.enable();

    let result = processor.process_line("hide this");
    assert!(result.hidden);
}

#[test]
fn test_multiple_highlights() {
    let mut processor = FilterProcessor::new();

    let filter = Filter::new(
        "numbers".to_string(),
        "Numbers".to_string(),
        r"\d+".to_string(),
        FilterAction::Highlight {
            color: (0, 255, 0),
            bg_color: None,
        },
    )
    .unwrap();

    processor.manager_mut().add_filter(filter).unwrap();

    let result = processor.process_line("Found 3 errors and 5 warnings");
    assert_eq!(result.highlights.len(), 1);
    assert_eq!(result.highlights[0].ranges.len(), 2); // Two numbers matched
}

#[test]
fn test_regex_capture_groups() {
    let mut processor = FilterProcessor::new();

    let filter = Filter::new(
        "capture".to_string(),
        "Capture Test".to_string(),
        r"(\w+)@(\w+)\.com".to_string(),
        FilterAction::Replace {
            replacement: "[$2] $1".to_string(),
        },
    )
    .unwrap();

    processor.manager_mut().add_filter(filter).unwrap();

    let result = processor.process_line("Contact: john@example.com");
    assert!(result.text.contains("[example] john"));
}

#[test]
fn test_notification_action() {
    let mut processor = FilterProcessor::new();

    let filter = Filter::new(
        "notify".to_string(),
        "Notify Test".to_string(),
        r"critical".to_string(),
        FilterAction::Notify {
            title: "Alert".to_string(),
            body: Some("Critical event".to_string()),
            sound: true,
        },
    )
    .unwrap();

    processor.manager_mut().add_filter(filter).unwrap();

    let result = processor.process_line("critical error occurred");
    assert_eq!(result.notifications.len(), 1);
    assert_eq!(result.notifications[0].title, "Alert");
    assert!(result.notifications[0].sound);
}

#[test]
fn test_process_multiple_lines() {
    let mut processor = FilterProcessor::new();

    let filter = Filter::new(
        "multi".to_string(),
        "Multi Test".to_string(),
        r"skip".to_string(),
        FilterAction::Hide,
    )
    .unwrap();

    processor.manager_mut().add_filter(filter).unwrap();

    let lines = vec![
        "line 1".to_string(),
        "line 2 skip".to_string(),
        "line 3".to_string(),
        "line 4 skip".to_string(),
    ];

    let results = processor.process_lines(&lines);
    assert_eq!(results.len(), 4);
    assert!(!results[0].hidden);
    assert!(results[1].hidden);
    assert!(!results[2].hidden);
    assert!(results[3].hidden);
}

#[test]
fn test_filter_manager_operations() {
    let mut manager = FilterManager::new();

    let filter = Filter::new(
        "test".to_string(),
        "Test".to_string(),
        r"test".to_string(),
        FilterAction::Hide,
    )
    .unwrap();

    manager.add_filter(filter).unwrap();
    assert_eq!(manager.filter_count(), 1);

    manager.toggle_filter("test").unwrap();
    assert!(!manager.get_filter("test").unwrap().enabled);

    manager.toggle_filter("test").unwrap();
    assert!(manager.get_filter("test").unwrap().enabled);

    let removed = manager.remove_filter("test");
    assert!(removed.is_some());
    assert_eq!(manager.filter_count(), 0);
}
