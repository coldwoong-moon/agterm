# AgTerm Filter System Guide

The AgTerm Filter System provides powerful real-time processing of terminal output, allowing you to hide, highlight, replace, or get notified about specific patterns in your terminal sessions.

## Table of Contents

- [Overview](#overview)
- [Core Concepts](#core-concepts)
- [Filter Actions](#filter-actions)
- [Usage Examples](#usage-examples)
- [Advanced Features](#advanced-features)
- [API Reference](#api-reference)

## Overview

The filter system consists of three main components:

1. **Filter** - Individual filter definitions with regex patterns and actions
2. **FilterManager** - Manages collections of filters with grouping and ordering
3. **FilterProcessor** - Applies filters to terminal output in real-time

## Core Concepts

### Filter Definition

A filter consists of:

- **ID**: Unique identifier
- **Name**: Human-readable name
- **Pattern**: Regular expression to match
- **Action**: What to do when pattern matches (hide, highlight, replace, notify)
- **Priority**: Higher priority filters run first
- **Group**: Optional grouping for batch operations
- **Enabled**: Whether the filter is active

### Filter Actions

Four types of actions are supported:

1. **Hide** - Remove matching lines from output
2. **Highlight** - Color matching text with foreground/background colors
3. **Replace** - Transform matching text using regex replacement
4. **Notify** - Send desktop notifications when pattern matches

## Filter Actions

### 1. Hide Action

Hide lines matching a pattern:

```rust
use agterm::filters::{Filter, FilterAction};

let filter = Filter::new(
    "hide_debug".to_string(),
    "Hide Debug Messages".to_string(),
    r"(?i)\[DEBUG\]".to_string(),
    FilterAction::Hide,
)?;
```

**Use Cases:**
- Hide verbose debug output
- Filter out noise from logs
- Remove specific error messages you want to ignore

### 2. Highlight Action

Highlight matching text with colors:

```rust
let filter = Filter::new(
    "highlight_errors".to_string(),
    "Highlight Errors".to_string(),
    r"(?i)ERROR|FAIL".to_string(),
    FilterAction::Highlight {
        color: (255, 0, 0),           // Red text
        bg_color: Some((50, 0, 0)),   // Dark red background
    },
)?;
```

**Use Cases:**
- Make errors stand out in red
- Highlight successful operations in green
- Mark warnings in yellow
- Emphasize important keywords

### 3. Replace Action

Transform text using regex replacement:

```rust
let filter = Filter::new(
    "mask_passwords".to_string(),
    "Mask Sensitive Data".to_string(),
    r"password[:=]\s*(\S+)".to_string(),
    FilterAction::Replace {
        replacement: "password: [REDACTED]".to_string(),
    },
)?;
```

**Capture Groups:**
You can use `$1`, `$2`, etc. to reference regex capture groups:

```rust
// Extract and reformat user information
let filter = Filter::new(
    "format_user".to_string(),
    "Format User Info".to_string(),
    r"User: (\w+), Role: (\w+)".to_string(),
    FilterAction::Replace {
        replacement: "[$2] $1".to_string(),  // Outputs: [role] username
    },
)?;
```

**Use Cases:**
- Mask sensitive information (passwords, tokens, emails)
- Reformat log messages
- Normalize output format
- Shorten verbose messages

### 4. Notify Action

Send desktop notifications when patterns match:

```rust
let filter = Filter::new(
    "notify_critical".to_string(),
    "Alert on Critical Errors".to_string(),
    r"(?i)CRITICAL|FATAL".to_string(),
    FilterAction::Notify {
        title: "Critical Error Detected".to_string(),
        body: Some("Check terminal immediately!".to_string()),
        sound: true,  // Play system notification sound
    },
)?;
```

**Use Cases:**
- Alert on critical errors in long-running processes
- Notify when build/test completes
- Alert on security events
- Monitor for specific events while multitasking

## Usage Examples

### Basic Usage

```rust
use agterm::filters::{Filter, FilterAction, FilterProcessor};

// Create processor
let mut processor = FilterProcessor::new();

// Add a filter
let filter = Filter::new(
    "my_filter".to_string(),
    "My First Filter".to_string(),
    r"pattern".to_string(),
    FilterAction::Hide,
)?;

processor.manager_mut().add_filter(filter)?;

// Process output
let result = processor.process_line("Some output with pattern here");
if !result.hidden {
    println!("{}", result.text);
}
```

### Case-Insensitive Matching

```rust
// Case-insensitive filter
let filter = Filter::new_case_insensitive(
    "ignore_case".to_string(),
    "Case Insensitive".to_string(),
    r"error".to_string(),  // Matches ERROR, Error, error, etc.
    FilterAction::Hide,
)?;
```

### Multiple Filters with Priority

```rust
let mut high_priority = Filter::new(
    "critical".to_string(),
    "Critical Filter".to_string(),
    r"CRITICAL".to_string(),
    FilterAction::Hide,
)?;
high_priority.priority = 100;  // Higher priority runs first

let mut low_priority = Filter::new(
    "debug".to_string(),
    "Debug Filter".to_string(),
    r"DEBUG".to_string(),
    FilterAction::Hide,
)?;
low_priority.priority = 1;

processor.manager_mut().add_filter(high_priority)?;
processor.manager_mut().add_filter(low_priority)?;
```

### Filter Groups

Organize filters into groups for batch operations:

```rust
// Create filters in a group
let mut filter1 = Filter::new(
    "log1".to_string(),
    "Filter 1".to_string(),
    r"pattern1".to_string(),
    FilterAction::Hide,
)?;
filter1.group = Some("my_group".to_string());

let mut filter2 = Filter::new(
    "log2".to_string(),
    "Filter 2".to_string(),
    r"pattern2".to_string(),
    FilterAction::Hide,
)?;
filter2.group = Some("my_group".to_string());

let manager = processor.manager_mut();
manager.add_filter(filter1)?;
manager.add_filter(filter2)?;

// Enable/disable entire group
manager.disable_group("my_group")?;
manager.enable_group("my_group")?;

// Iterate group filters
for filter in manager.filters_in_group("my_group") {
    println!("Group filter: {}", filter.name);
}
```

### Toggle Filters

```rust
// Toggle individual filter
let enabled = processor.manager_mut().toggle_filter("my_filter")?;
println!("Filter is now: {}", if enabled { "enabled" } else { "disabled" });

// Disable/enable processor entirely
processor.disable();
processor.enable();
let is_enabled = processor.is_enabled();
```

### Statistics Tracking

Each filter tracks how many times it matched:

```rust
// Process some lines
processor.process_line("error 1");
processor.process_line("error 2");
processor.process_line("success");

// Get statistics
let stats = processor.get_stats();
for (name, stat) in stats {
    println!("{}: {} matches", name, stat.match_count);
    if let Some(timestamp) = stat.last_match {
        println!("  Last match: {}", timestamp);
    }
}

// Total matches across all filters
let total = processor.manager().total_matches();
println!("Total matches: {}", total);

// Reset statistics
processor.manager_mut().reset_stats();
```

### Export/Import Filters

Save and load filter configurations:

```rust
// Export to JSON
let json = processor.manager().export_json()?;
std::fs::write("filters.json", json)?;

// Import from JSON
let json = std::fs::read_to_string("filters.json")?;
let count = processor.manager_mut().import_json(&json)?;
println!("Imported {} filters", count);
```

## Advanced Features

### Complex Regex Patterns

The filter system supports full regex syntax:

```rust
// Match multiple patterns
let filter = Filter::new(
    "multi".to_string(),
    "Multiple Patterns".to_string(),
    r"(ERROR|WARN|FAIL)".to_string(),
    FilterAction::Highlight { color: (255, 0, 0), bg_color: None },
)?;

// Word boundaries
let filter = Filter::new(
    "exact".to_string(),
    "Exact Word".to_string(),
    r"\berror\b".to_string(),  // Matches "error" but not "errors" or "suberror"
    FilterAction::Hide,
)?;

// Negative lookahead
let filter = Filter::new(
    "exclude".to_string(),
    "Exclude Pattern".to_string(),
    r"error(?! ignored)".to_string(),  // Matches "error" except "error ignored"
    FilterAction::Hide,
)?;

// Capture groups for complex replacements
let filter = Filter::new(
    "reformat".to_string(),
    "Reformat Timestamps".to_string(),
    r"(\d{4})-(\d{2})-(\d{2})".to_string(),
    FilterAction::Replace {
        replacement: "$2/$3/$1".to_string(),  // Convert YYYY-MM-DD to MM/DD/YYYY
    },
)?;
```

### Processing Multiple Lines

```rust
let lines = vec![
    "Line 1 with error".to_string(),
    "Line 2 normal".to_string(),
    "Line 3 with warning".to_string(),
];

let results = processor.process_lines(&lines);
for result in results {
    if !result.hidden {
        println!("{}", result.text);
    }
}
```

### Handling Highlights

```rust
let result = processor.process_line("Error in line 42");

for highlight in result.highlights {
    println!("Highlight color: RGB({}, {}, {})",
             highlight.color.0, highlight.color.1, highlight.color.2);

    if let Some(bg) = highlight.bg_color {
        println!("Background: RGB({}, {}, {})", bg.0, bg.1, bg.2);
    }

    for (start, end) in highlight.ranges {
        println!("Highlighted range: {}-{}", start, end);
    }
}
```

### Handling Notifications

```rust
let result = processor.process_line("CRITICAL: System failure");

for notification in result.notifications {
    println!("Notification: {}", notification.title);
    if let Some(body) = notification.body {
        println!("Body: {}", body);
    }
    if notification.sound {
        println!("Play notification sound");
    }
}
```

## API Reference

### Filter

```rust
// Create new filter
Filter::new(id: String, name: String, pattern: String, action: FilterAction) -> Result<Filter>

// Create case-insensitive filter
Filter::new_case_insensitive(id: String, name: String, pattern: String, action: FilterAction) -> Result<Filter>

// Check if pattern matches
filter.matches(text: &str) -> bool

// Apply filter to text
filter.apply(text: &str) -> FilterResult

// Recompile pattern
filter.compile() -> Result<()>

// Reset statistics
filter.reset_stats()
```

### FilterManager

```rust
// Create new manager
FilterManager::new() -> FilterManager

// Add/remove filters
manager.add_filter(filter: Filter) -> Result<()>
manager.remove_filter(id: &str) -> Option<Filter>

// Get filters
manager.get_filter(id: &str) -> Option<&Filter>
manager.get_filter_mut(id: &str) -> Option<&mut Filter>
manager.filters() -> impl Iterator<Item = &Filter>

// Groups
manager.filters_in_group(group: &str) -> impl Iterator<Item = &Filter>
manager.groups() -> impl Iterator<Item = &String>

// Enable/disable
manager.enable_filter(id: &str) -> Result<()>
manager.disable_filter(id: &str) -> Result<()>
manager.toggle_filter(id: &str) -> Result<bool>
manager.enable_group(group: &str) -> Result<()>
manager.disable_group(group: &str) -> Result<()>

// Priority
manager.set_priority(id: &str, priority: i32) -> Result<()>

// Statistics
manager.total_matches() -> u64
manager.reset_stats()

// Export/Import
manager.export_json() -> Result<String>
manager.import_json(json: &str) -> Result<usize>

// Utility
manager.clear()
manager.filter_count() -> usize
```

### FilterProcessor

```rust
// Create new processor
FilterProcessor::new() -> FilterProcessor
FilterProcessor::with_manager(manager: FilterManager) -> FilterProcessor

// Get manager
processor.manager() -> &FilterManager
processor.manager_mut() -> &mut FilterManager

// Enable/disable
processor.enable()
processor.disable()
processor.toggle() -> bool
processor.is_enabled() -> bool

// Process output
processor.process_line(line: &str) -> ProcessedLine
processor.process_lines(lines: &[String]) -> Vec<ProcessedLine>

// Statistics
processor.get_stats() -> HashMap<String, FilterStats>
```

### ProcessedLine

```rust
pub struct ProcessedLine {
    pub text: String,                      // Final text after processing
    pub hidden: bool,                      // Should this line be hidden?
    pub highlights: Vec<HighlightInfo>,    // Highlight information
    pub notifications: Vec<NotificationInfo>, // Notifications to trigger
}
```

### HighlightInfo

```rust
pub struct HighlightInfo {
    pub color: (u8, u8, u8),              // RGB foreground color
    pub bg_color: Option<(u8, u8, u8)>,   // RGB background color
    pub ranges: Vec<(usize, usize)>,      // Character ranges to highlight
}
```

### NotificationInfo

```rust
pub struct NotificationInfo {
    pub title: String,              // Notification title
    pub body: Option<String>,       // Notification body
    pub sound: bool,                // Play sound?
}
```

## Common Use Cases

### 1. Development Workflow

```rust
// Hide verbose framework output
let hide_verbose = Filter::new_case_insensitive(
    "hide_verbose".to_string(),
    "Hide Verbose".to_string(),
    r"\[VERBOSE\]|\[TRACE\]".to_string(),
    FilterAction::Hide,
)?;

// Highlight test failures
let highlight_fails = Filter::new(
    "test_fails".to_string(),
    "Test Failures".to_string(),
    r"FAILED|FAIL:|test.*failed".to_string(),
    FilterAction::Highlight {
        color: (255, 0, 0),
        bg_color: Some((60, 0, 0)),
    },
)?;

// Notify on build completion
let notify_build = Filter::new(
    "build_done".to_string(),
    "Build Complete".to_string(),
    r"Build (succeeded|failed)".to_string(),
    FilterAction::Notify {
        title: "Build Complete".to_string(),
        body: None,
        sound: true,
    },
)?;
```

### 2. Log Analysis

```rust
// Hide health checks
let hide_health = Filter::new(
    "hide_health".to_string(),
    "Hide Health Checks".to_string(),
    r"/health|/ping|/metrics".to_string(),
    FilterAction::Hide,
)?;

// Mask API keys
let mask_keys = Filter::new(
    "mask_keys".to_string(),
    "Mask API Keys".to_string(),
    r"(api[_-]?key|token)[:=]\s*['\"]?([^'\" ]+)".to_string(),
    FilterAction::Replace {
        replacement: "$1: [REDACTED]".to_string(),
    },
)?;

// Highlight slow queries
let highlight_slow = Filter::new(
    "slow_queries".to_string(),
    "Slow Queries".to_string(),
    r"query time: (\d+)ms".to_string(),
    FilterAction::Highlight {
        color: (255, 165, 0), // Orange
        bg_color: None,
    },
)?;
```

### 3. Security Monitoring

```rust
// Notify on failed login attempts
let notify_failed_login = Filter::new(
    "failed_login".to_string(),
    "Failed Login".to_string(),
    r"authentication failed|invalid credentials|login failed".to_string(),
    FilterAction::Notify {
        title: "Security Alert".to_string(),
        body: Some("Failed login attempt detected".to_string()),
        sound: true,
    },
)?;

// Highlight permission errors
let highlight_perms = Filter::new(
    "perms".to_string(),
    "Permission Errors".to_string(),
    r"permission denied|access denied|unauthorized".to_string(),
    FilterAction::Highlight {
        color: (255, 0, 0),
        bg_color: Some((80, 0, 0)),
    },
)?;
```

## Best Practices

1. **Use Specific Patterns**: Make patterns as specific as possible to avoid false matches
2. **Set Priorities Carefully**: Higher priority filters run first, so order matters
3. **Group Related Filters**: Use groups to manage related filters together
4. **Monitor Performance**: Complex regex patterns can impact performance on high-volume output
5. **Test Patterns**: Use tools like regex101.com to test patterns before adding them
6. **Export Configurations**: Save your filter setups for different projects/scenarios
7. **Use Case-Insensitive**: When appropriate, use case-insensitive matching for more reliable matches

## Performance Tips

- Simple patterns (like literal strings) are faster than complex regex
- Use `^` and `$` anchors when matching entire lines
- Avoid excessive backtracking in regex patterns
- Disable filters you're not actively using
- Use filter groups to quickly disable multiple filters
- Monitor match statistics to identify frequently-triggered filters

## Troubleshooting

### Pattern Not Matching

1. Test your regex pattern separately
2. Check if case-insensitive mode is needed
3. Verify you're using Rust regex syntax (not PCRE or JavaScript regex)
4. Use raw strings (`r"pattern"`) to avoid escape sequence issues

### Performance Issues

1. Simplify complex regex patterns
2. Disable unused filters
3. Check if patterns are causing excessive backtracking
4. Consider using multiple simple filters instead of one complex filter

### Unexpected Behavior

1. Check filter priorities - wrong order can cause unexpected results
2. Verify filters are enabled
3. Check if processor is enabled
4. Review filter statistics to see which filters are matching

## Examples

See `examples/filters_demo.rs` for a complete working demonstration:

```bash
cargo run --example filters_demo
```

## Integration with AgTerm

The filter system integrates seamlessly with AgTerm's terminal output processing. Filters can be configured through:

1. Configuration files (TOML)
2. Runtime API calls
3. Command palette / UI controls
4. Keyboard shortcuts

Future enhancements may include:
- Visual filter editor
- Filter templates library
- Pattern testing interface
- Real-time filter statistics in UI
- Filter profiles for different scenarios
