# Filter System Quick Reference

## Basic Setup

```rust
use agterm::filters::{Filter, FilterAction, FilterProcessor};

let mut processor = FilterProcessor::new();
```

## Creating Filters

### Hide

```rust
Filter::new(
    "id".to_string(),
    "Name".to_string(),
    r"pattern".to_string(),
    FilterAction::Hide,
)?
```

### Highlight

```rust
Filter::new(
    "id".to_string(),
    "Name".to_string(),
    r"pattern".to_string(),
    FilterAction::Highlight {
        color: (255, 0, 0),           // RGB foreground
        bg_color: Some((50, 0, 0)),   // Optional RGB background
    },
)?
```

### Replace

```rust
Filter::new(
    "id".to_string(),
    "Name".to_string(),
    r"(pattern)".to_string(),
    FilterAction::Replace {
        replacement: "new $1 text".to_string(),  // $1, $2 for capture groups
    },
)?
```

### Notify

```rust
Filter::new(
    "id".to_string(),
    "Name".to_string(),
    r"pattern".to_string(),
    FilterAction::Notify {
        title: "Alert".to_string(),
        body: Some("Message".to_string()),
        sound: true,
    },
)?
```

## Case Insensitive

```rust
Filter::new_case_insensitive(
    "id".to_string(),
    "Name".to_string(),
    r"error".to_string(),  // Matches ERROR, Error, error
    FilterAction::Hide,
)?
```

## Managing Filters

```rust
// Add
processor.manager_mut().add_filter(filter)?;

// Remove
processor.manager_mut().remove_filter("id");

// Get
let filter = processor.manager().get_filter("id");
let filter = processor.manager_mut().get_filter_mut("id");

// Enable/Disable
processor.manager_mut().enable_filter("id")?;
processor.manager_mut().disable_filter("id")?;
processor.manager_mut().toggle_filter("id")?;

// Clear all
processor.manager_mut().clear();
```

## Groups

```rust
// Create with group
let mut filter = Filter::new(/* ... */)?;
filter.group = Some("group_name".to_string());

// Group operations
processor.manager_mut().enable_group("group_name")?;
processor.manager_mut().disable_group("group_name")?;

// Iterate group
for filter in processor.manager().filters_in_group("group_name") {
    // ...
}
```

## Priority

```rust
let mut filter = Filter::new(/* ... */)?;
filter.priority = 100;  // Higher runs first (default: 0)

// Or set after adding
processor.manager_mut().set_priority("id", 100)?;
```

## Processing

```rust
// Single line
let result = processor.process_line("text");

if result.hidden {
    // Don't display
} else {
    println!("{}", result.text);  // Display (possibly modified)

    // Apply highlights
    for highlight in result.highlights {
        // highlight.color: (u8, u8, u8)
        // highlight.bg_color: Option<(u8, u8, u8)>
        // highlight.ranges: Vec<(usize, usize)>
    }

    // Send notifications
    for notification in result.notifications {
        // notification.title: String
        // notification.body: Option<String>
        // notification.sound: bool
    }
}

// Multiple lines
let results = processor.process_lines(&lines);
```

## Processor Control

```rust
// Enable/Disable entire processor
processor.enable();
processor.disable();
processor.toggle();
let enabled = processor.is_enabled();
```

## Statistics

```rust
// Per-filter stats
let stats = processor.get_stats();
for (name, stat) in stats {
    println!("{}: {} matches", name, stat.match_count);
    if let Some(ts) = stat.last_match {
        println!("Last: {}", ts);
    }
}

// Total across all filters
let total = processor.manager().total_matches();

// Reset
processor.manager_mut().reset_stats();
```

## Persistence

```rust
// Export
let json = processor.manager().export_json()?;
std::fs::write("filters.json", json)?;

// Import
let json = std::fs::read_to_string("filters.json")?;
let count = processor.manager_mut().import_json(&json)?;
```

## Common Patterns

### Hide log levels

```rust
r"(?i)\[(DEBUG|TRACE|VERBOSE)\]"
```

### Highlight errors/warnings

```rust
r"(?i)\b(ERROR|FAIL|FATAL)\b"     // Errors (red)
r"(?i)\b(WARN|WARNING)\b"         // Warnings (yellow)
r"(?i)\b(SUCCESS|OK|PASS)\b"      // Success (green)
```

### Mask sensitive data

```rust
r"(?i)(password|pwd|token|api[_-]?key)[:=]\s*\S+"
r"\b\d{3}-\d{2}-\d{4}\b"          // SSN
r"\b\d{16}\b"                     // Credit card
r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b"  // Email
```

### URL detection

```rust
r"https?://[^\s<>\"\'\]\)]+'"
```

### IP addresses

```rust
r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b"
```

### Timestamps

```rust
r"\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}"  // ISO 8601
```

### File paths

```rust
r"(?:/[^/\s]+)+"                  // Unix
r"[A-Z]:\\(?:[^\\]+\\)*[^\\]+"    // Windows
```

## Regex Tips

```rust
// Case insensitive
r"(?i)pattern"

// Word boundaries
r"\berror\b"

// Capture groups
r"(\w+)@(\w+)"  // Use $1, $2 in replacement

// Non-capturing group
r"(?:pattern)"

// Lookahead/Lookbehind
r"error(?! ignored)"   // error but not "error ignored"
r"(?<=User: )\w+"      // word after "User: "

// Multiple options
r"(ERROR|WARN|INFO)"

// Character classes
r"\d+"     // digits
r"\w+"     // word characters
r"\s+"     // whitespace
r"[a-z]+"  // lowercase letters
```

## Error Handling

```rust
use agterm::filters::FilterError;

match filter_result {
    Ok(filter) => { /* ... */ },
    Err(FilterError::InvalidPattern(e)) => {
        eprintln!("Bad regex: {}", e);
    },
    Err(FilterError::FilterNotFound(id)) => {
        eprintln!("No filter: {}", id);
    },
    Err(FilterError::GroupNotFound(group)) => {
        eprintln!("No group: {}", group);
    },
    Err(e) => eprintln!("Error: {}", e),
}
```

## Performance Tips

1. **Simple patterns are faster**
   ```rust
   r"ERROR"              // Fast
   r"(?i)ERROR"          // Slower (case insensitive)
   r"(E|e)(R|r)(R|r).*"  // Slowest (complex)
   ```

2. **Use anchors when possible**
   ```rust
   r"^ERROR"   // Only check start of line
   r"ERROR$"   // Only check end of line
   ```

3. **Avoid excessive backtracking**
   ```rust
   r".*error.*"    // Bad (backtracking)
   r"error"        // Good (no backtracking)
   ```

4. **Disable unused filters**
   ```rust
   processor.manager_mut().disable_filter("unused")?;
   ```

5. **Use groups to manage many filters**
   ```rust
   processor.manager_mut().disable_group("verbose_logs")?;
   ```

## Common Use Cases

### Development

```rust
// Hide debug noise
Filter::new("hide_debug", "Hide Debug",
    r"(?i)\[DEBUG\]", FilterAction::Hide)?

// Highlight test failures
Filter::new("test_fail", "Test Failures",
    r"FAILED|test.*failed",
    FilterAction::Highlight { color: (255, 0, 0), bg_color: None })?
```

### Log Analysis

```rust
// Hide health checks
Filter::new("hide_health", "Hide Health Checks",
    r"/health|/ping", FilterAction::Hide)?

// Mask API keys
Filter::new("mask_api", "Mask API Keys",
    r"api[_-]?key[:=]\s*(\S+)",
    FilterAction::Replace { replacement: "api_key: [REDACTED]".to_string() })?
```

### Security Monitoring

```rust
// Alert on failed logins
Filter::new("failed_login", "Failed Login",
    r"(?i)authentication failed|login failed",
    FilterAction::Notify {
        title: "Security Alert".to_string(),
        body: Some("Failed login attempt".to_string()),
        sound: true
    })?
```

## Testing

```rust
#[test]
fn test_my_filter() {
    let mut processor = FilterProcessor::new();
    let filter = Filter::new(
        "test".to_string(),
        "Test".to_string(),
        r"pattern".to_string(),
        FilterAction::Hide,
    ).unwrap();

    processor.manager_mut().add_filter(filter).unwrap();

    let result = processor.process_line("text with pattern");
    assert!(result.hidden);
}
```

## Full Example

```rust
use agterm::filters::{Filter, FilterAction, FilterProcessor};

fn setup_filters() -> Result<FilterProcessor, Box<dyn std::error::Error>> {
    let mut processor = FilterProcessor::new();

    // Hide debug
    let mut hide_debug = Filter::new(
        "hide_debug".to_string(),
        "Hide Debug".to_string(),
        r"(?i)\[DEBUG\]".to_string(),
        FilterAction::Hide,
    )?;
    hide_debug.group = Some("log_levels".to_string());
    hide_debug.priority = 10;

    // Highlight errors
    let highlight_errors = Filter::new(
        "errors".to_string(),
        "Highlight Errors".to_string(),
        r"(?i)ERROR|FAIL".to_string(),
        FilterAction::Highlight {
            color: (255, 0, 0),
            bg_color: Some((50, 0, 0)),
        },
    )?;

    // Mask passwords
    let mask_passwords = Filter::new(
        "mask_pwd".to_string(),
        "Mask Passwords".to_string(),
        r"(?i)password[:=]\s*(\S+)".to_string(),
        FilterAction::Replace {
            replacement: "password: [REDACTED]".to_string(),
        },
    )?;

    processor.manager_mut().add_filter(hide_debug)?;
    processor.manager_mut().add_filter(highlight_errors)?;
    processor.manager_mut().add_filter(mask_passwords)?;

    Ok(processor)
}

fn process_terminal_output(processor: &mut FilterProcessor, line: &str) {
    let result = processor.process_line(line);

    if result.hidden {
        return;  // Skip hidden lines
    }

    // Apply highlights to terminal rendering
    for highlight in result.highlights {
        // Set colors for highlighted ranges
    }

    // Send notifications
    for notification in result.notifications {
        // Send desktop notification
    }

    // Display line (possibly modified)
    println!("{}", result.text);
}
```

## Resources

- **Full Guide**: `FILTERS_GUIDE.md`
- **Implementation Details**: `FILTERS_IMPLEMENTATION_SUMMARY.md`
- **Demo**: `cargo run --example filters_demo`
- **Tests**: `cargo test filters`
