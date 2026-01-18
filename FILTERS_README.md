# Terminal Output Filter System

A comprehensive, production-ready filter system for real-time terminal output processing.

## Quick Start

```rust
use agterm::filters::{Filter, FilterAction, FilterProcessor};

let mut processor = FilterProcessor::new();

// Hide debug messages
let filter = Filter::new(
    "hide_debug".to_string(),
    "Hide Debug".to_string(),
    r"(?i)\[DEBUG\]".to_string(),
    FilterAction::Hide,
)?;

processor.manager_mut().add_filter(filter)?;

// Process terminal output
let result = processor.process_line("[DEBUG] Loading config");
if !result.hidden {
    println!("{}", result.text);
}
```

## Features

### ✅ Complete Implementation

- **4 Filter Actions**
  - 🚫 **Hide** - Remove matching lines
  - 🎨 **Highlight** - Color matching text
  - 🔄 **Replace** - Transform text with regex
  - 🔔 **Notify** - Desktop notifications

- **Advanced Features**
  - Full regex support with capture groups
  - Case-insensitive matching
  - Priority-based ordering
  - Filter grouping
  - Enable/disable controls
  - Statistics tracking
  - JSON export/import

- **Performance**
  - Arc-wrapped compiled patterns
  - Early exit optimization
  - Efficient matching
  - Thread-safe design

### 📊 Test Coverage

- **35+ Tests**
  - 20+ unit tests
  - 15+ integration tests
  - All features covered
  - Edge cases tested

### 📚 Documentation

- **Complete Guides**
  - [FILTERS_GUIDE.md](FILTERS_GUIDE.md) - Comprehensive usage guide
  - [FILTERS_IMPLEMENTATION_SUMMARY.md](FILTERS_IMPLEMENTATION_SUMMARY.md) - Technical details
  - API documentation in source
  - Working examples

## File Structure

```
agterm/
├── src/
│   ├── filters.rs                    # Core implementation (1400 LOC)
│   └── lib.rs                        # Module export
├── examples/
│   └── filters_demo.rs               # Demonstration (400 LOC)
├── tests/
│   └── filters_integration_test.rs   # Integration tests (400 LOC)
├── FILTERS_GUIDE.md                  # User guide (700 lines)
├── FILTERS_IMPLEMENTATION_SUMMARY.md # Technical summary (400 lines)
└── FILTERS_README.md                 # This file
```

## Usage Examples

### Hide Debug Output

```rust
let filter = Filter::new(
    "hide_debug".to_string(),
    "Hide Debug".to_string(),
    r"(?i)\[DEBUG\]".to_string(),
    FilterAction::Hide,
)?;
```

### Highlight Errors in Red

```rust
let filter = Filter::new(
    "errors".to_string(),
    "Highlight Errors".to_string(),
    r"(?i)ERROR|FAIL".to_string(),
    FilterAction::Highlight {
        color: (255, 0, 0),           // Red text
        bg_color: Some((50, 0, 0)),   // Dark red background
    },
)?;
```

### Mask Sensitive Data

```rust
let filter = Filter::new(
    "mask_passwords".to_string(),
    "Mask Passwords".to_string(),
    r"password[:=]\s*(\S+)".to_string(),
    FilterAction::Replace {
        replacement: "password: [REDACTED]".to_string(),
    },
)?;

let result = processor.process_line("Auth: password=secret123");
// Output: "Auth: password: [REDACTED]"
```

### Alert on Critical Errors

```rust
let filter = Filter::new(
    "notify_critical".to_string(),
    "Critical Alerts".to_string(),
    r"(?i)CRITICAL|FATAL".to_string(),
    FilterAction::Notify {
        title: "Critical Error".to_string(),
        body: Some("Check terminal immediately".to_string()),
        sound: true,
    },
)?;
```

## Advanced Features

### Filter Groups

```rust
let mut filter1 = Filter::new(/* ... */)?;
filter1.group = Some("log_levels".to_string());

processor.manager_mut().add_filter(filter1)?;
processor.manager_mut().disable_group("log_levels")?;
```

### Priority Ordering

```rust
let mut high_priority = Filter::new(/* ... */)?;
high_priority.priority = 100;  // Higher runs first

processor.manager_mut().add_filter(high_priority)?;
```

### Statistics

```rust
let stats = processor.get_stats();
for (name, stat) in stats {
    println!("{}: {} matches", name, stat.match_count);
}

let total = processor.manager().total_matches();
println!("Total: {}", total);
```

### Persistence

```rust
// Export
let json = processor.manager().export_json()?;
std::fs::write("filters.json", json)?;

// Import
let json = std::fs::read_to_string("filters.json")?;
processor.manager_mut().import_json(&json)?;
```

## Running Tests

```bash
# Run unit tests
cargo test filters --lib

# Run integration tests
cargo test --test filters_integration_test

# Run demo
cargo run --example filters_demo
```

## API Overview

### Core Types

- **`Filter`** - Individual filter with pattern and action
- **`FilterManager`** - Manages collection of filters
- **`FilterProcessor`** - Applies filters to text
- **`FilterAction`** - Action to perform on match
- **`ProcessedLine`** - Result of processing a line

### Key Methods

```rust
// Filter
Filter::new(id, name, pattern, action) -> Result<Filter>
filter.matches(text: &str) -> bool
filter.apply(text: &str) -> FilterResult

// Manager
manager.add_filter(filter) -> Result<()>
manager.enable_filter(id) -> Result<()>
manager.disable_group(group) -> Result<()>

// Processor
processor.process_line(line: &str) -> ProcessedLine
processor.get_stats() -> HashMap<String, FilterStats>
```

## Performance

- **Fast Matching**: Compiled regex patterns cached in Arc
- **Early Exit**: Hide action stops processing immediately
- **Efficient**: O(1) filter lookup, O(n log n) priority sorting
- **Thread-Safe**: Safe for concurrent use

## Integration Points

### Terminal Rendering

```rust
for line in terminal_output {
    let result = processor.process_line(&line);

    if result.hidden { continue; }

    // Apply highlights
    for highlight in result.highlights {
        apply_colors(highlight.color, highlight.bg_color, highlight.ranges);
    }

    // Send notifications
    for notification in result.notifications {
        send_notification(notification);
    }

    render_line(result.text);
}
```

### Configuration

```rust
// In config.rs
pub struct AppConfig {
    pub filters: String,  // JSON of filters
    // ...
}

// Save
config.filters = processor.manager().export_json()?;

// Load
processor.manager_mut().import_json(&config.filters)?;
```

## Documentation

- **[FILTERS_GUIDE.md](FILTERS_GUIDE.md)** - Complete user guide with examples
- **[FILTERS_IMPLEMENTATION_SUMMARY.md](FILTERS_IMPLEMENTATION_SUMMARY.md)** - Technical details
- **[examples/filters_demo.rs](examples/filters_demo.rs)** - Working demonstrations
- **Source code** - Fully documented API

## Dependencies

All required dependencies are already in `Cargo.toml`:
- `regex` - Pattern matching
- `serde` + `serde_json` - Serialization
- `thiserror` - Error handling

## Status

✅ **Production Ready**

- Complete implementation
- Comprehensive testing
- Full documentation
- Performance optimized
- Thread-safe
- Zero unsafe code

## Next Steps

To integrate into AgTerm:

1. ✅ Core implementation
2. ✅ Testing
3. ✅ Documentation
4. ⬜ Configuration file support
5. ⬜ UI integration (command palette)
6. ⬜ Keyboard shortcuts
7. ⬜ Visual filter editor
8. ⬜ Filter profiles

## Examples

Run the comprehensive demo:

```bash
cargo run --example filters_demo
```

Output shows 8 practical examples:
1. Hiding debug messages
2. Highlighting errors
3. Masking sensitive data
4. Multiple filters with priorities
5. Filter groups
6. Statistics tracking
7. Notification triggers
8. JSON export/import

## Support

- See [FILTERS_GUIDE.md](FILTERS_GUIDE.md) for complete usage guide
- See [FILTERS_IMPLEMENTATION_SUMMARY.md](FILTERS_IMPLEMENTATION_SUMMARY.md) for technical details
- Run `cargo doc --open` for API documentation

## License

Same as AgTerm project (MIT)
