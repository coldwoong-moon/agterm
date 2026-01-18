//! Filter System Demo
//!
//! Demonstrates the usage of AgTerm's filter system for terminal output processing.
//!
//! Run with: cargo run --example filters_demo

use agterm::filters::{Filter, FilterAction, FilterManager, FilterProcessor};

fn main() {
    println!("=== AgTerm Filter System Demo ===\n");

    // Create a filter processor
    let mut processor = FilterProcessor::new();

    // Example 1: Hide debug messages
    println!("Example 1: Hiding Debug Messages");
    println!("---");

    let hide_debug = Filter::new(
        "hide_debug".to_string(),
        "Hide Debug".to_string(),
        r"(?i)\[DEBUG\]".to_string(),
        FilterAction::Hide,
    )
    .unwrap();

    processor.manager_mut().add_filter(hide_debug).unwrap();

    let test_lines = vec![
        "[INFO] Application started",
        "[DEBUG] Loading configuration",
        "[WARN] Deprecated feature used",
        "[DEBUG] Memory usage: 45MB",
        "[ERROR] Connection failed",
    ];

    for line in &test_lines {
        let result = processor.process_line(line);
        if !result.hidden {
            println!("{}", result.text);
        }
    }

    println!();

    // Example 2: Highlight errors in red
    println!("Example 2: Highlighting Errors");
    println!("---");

    processor.manager_mut().clear();

    let highlight_errors = Filter::new(
        "highlight_errors".to_string(),
        "Highlight Errors".to_string(),
        r"(?i)\[ERROR\]|\bERROR\b".to_string(),
        FilterAction::Highlight {
            color: (255, 0, 0), // Red
            bg_color: Some((50, 0, 0)), // Dark red background
        },
    )
    .unwrap();

    processor.manager_mut().add_filter(highlight_errors).unwrap();

    for line in &test_lines {
        let result = processor.process_line(line);
        if !result.highlights.is_empty() {
            println!("{} [HIGHLIGHTED in red]", result.text);
        } else {
            println!("{}", result.text);
        }
    }

    println!();

    // Example 3: Replace sensitive information
    println!("Example 3: Masking Sensitive Data");
    println!("---");

    processor.manager_mut().clear();

    let mask_passwords = Filter::new(
        "mask_passwords".to_string(),
        "Mask Passwords".to_string(),
        r"(?i)(password|pwd|token)[:=]\s*\S+".to_string(),
        FilterAction::Replace {
            replacement: "$1: [REDACTED]".to_string(),
        },
    )
    .unwrap();

    processor.manager_mut().add_filter(mask_passwords).unwrap();

    let sensitive_lines = vec![
        "User logged in successfully",
        "Authentication: password=secret123",
        "API request with token=abc123xyz",
        "Connection established",
    ];

    for line in &sensitive_lines {
        let result = processor.process_line(line);
        println!("{}", result.text);
    }

    println!();

    // Example 4: Multiple filters with priorities
    println!("Example 4: Multiple Filters with Priorities");
    println!("---");

    processor.manager_mut().clear();

    let mut filter1 = Filter::new(
        "highlight_warn".to_string(),
        "Highlight Warnings".to_string(),
        r"(?i)\[WARN\]".to_string(),
        FilterAction::Highlight {
            color: (255, 255, 0), // Yellow
            bg_color: None,
        },
    )
    .unwrap();
    filter1.priority = 10;

    let mut filter2 = Filter::new_case_insensitive(
        "replace_deprecated".to_string(),
        "Replace Deprecated".to_string(),
        r"deprecated".to_string(),
        FilterAction::Replace {
            replacement: "LEGACY".to_string(),
        },
    )
    .unwrap();
    filter2.priority = 20; // Higher priority - runs first

    processor.manager_mut().add_filter(filter1).unwrap();
    processor.manager_mut().add_filter(filter2).unwrap();

    for line in &test_lines {
        let result = processor.process_line(line);
        let highlights_str = if !result.highlights.is_empty() {
            " [HIGHLIGHTED]"
        } else {
            ""
        };
        println!("{}{}", result.text, highlights_str);
    }

    println!();

    // Example 5: Filter groups
    println!("Example 5: Filter Groups");
    println!("---");

    processor.manager_mut().clear();

    let mut log_filter1 = Filter::new(
        "hide_info".to_string(),
        "Hide Info".to_string(),
        r"(?i)\[INFO\]".to_string(),
        FilterAction::Hide,
    )
    .unwrap();
    log_filter1.group = Some("log_levels".to_string());

    let mut log_filter2 = Filter::new(
        "hide_debug2".to_string(),
        "Hide Debug".to_string(),
        r"(?i)\[DEBUG\]".to_string(),
        FilterAction::Hide,
    )
    .unwrap();
    log_filter2.group = Some("log_levels".to_string());

    processor.manager_mut().add_filter(log_filter1).unwrap();
    processor.manager_mut().add_filter(log_filter2).unwrap();

    println!("With filters enabled:");
    for line in &test_lines {
        let result = processor.process_line(line);
        if !result.hidden {
            println!("{}", result.text);
        }
    }

    println!("\nDisabling filter group...");
    processor.manager_mut().disable_group("log_levels").unwrap();

    println!("\nWith filters disabled:");
    for line in &test_lines {
        let result = processor.process_line(line);
        if !result.hidden {
            println!("{}", result.text);
        }
    }

    println!();

    // Example 6: Statistics
    println!("Example 6: Filter Statistics");
    println!("---");

    processor.manager_mut().clear();

    let count_filter = Filter::new(
        "count_errors".to_string(),
        "Count Errors".to_string(),
        r"(?i)error".to_string(),
        FilterAction::Highlight {
            color: (255, 0, 0),
            bg_color: None,
        },
    )
    .unwrap();

    processor.manager_mut().add_filter(count_filter).unwrap();

    let many_lines = vec![
        "Starting application",
        "Error loading config",
        "Retrying connection",
        "Error: timeout",
        "Connection established",
        "Processing data",
        "Error in module A",
        "Completed successfully",
    ];

    for line in &many_lines {
        processor.process_line(line);
    }

    let stats = processor.get_stats();
    for (name, stat) in stats {
        println!("Filter '{}': {} matches", name, stat.match_count);
    }

    println!("\nTotal matches: {}", processor.manager().total_matches());

    println!();

    // Example 7: Notifications
    println!("Example 7: Notification Triggers");
    println!("---");

    processor.manager_mut().clear();

    let notify_filter = Filter::new(
        "notify_critical".to_string(),
        "Notify Critical".to_string(),
        r"(?i)critical|fatal".to_string(),
        FilterAction::Notify {
            title: "Critical Error".to_string(),
            body: Some("A critical error occurred!".to_string()),
            sound: true,
        },
    )
    .unwrap();

    processor.manager_mut().add_filter(notify_filter).unwrap();

    let critical_lines = vec![
        "Normal operation",
        "CRITICAL: Database connection lost",
        "Warning: low memory",
        "Fatal error in core module",
    ];

    for line in &critical_lines {
        let result = processor.process_line(line);
        println!("{}", result.text);
        for notification in result.notifications {
            println!(
                "  -> NOTIFICATION: {} - {}",
                notification.title,
                notification.body.unwrap_or_default()
            );
        }
    }

    println!();

    // Example 8: JSON Export/Import
    println!("Example 8: Filter Export/Import");
    println!("---");

    let mut manager = FilterManager::new();

    let filter = Filter::new(
        "export_test".to_string(),
        "Export Test Filter".to_string(),
        r"test".to_string(),
        FilterAction::Hide,
    )
    .unwrap();

    manager.add_filter(filter).unwrap();

    let json = manager.export_json().unwrap();
    println!("Exported JSON:\n{}", json);

    let mut new_manager = FilterManager::new();
    let count = new_manager.import_json(&json).unwrap();
    println!("\nImported {} filter(s)", count);

    println!("\n=== Demo Complete ===");
}
