# Clipboard History - Usage Guide

## Overview

The `clipboard_history` module provides comprehensive clipboard history management with the following features:

1. **ClipboardEntry** - Individual clipboard items with metadata
2. **ClipboardHistory** - History manager with search, filter, and persistence
3. **Content Type Detection** - Automatic detection of URLs, paths, emails, code, etc.
4. **Pin Functionality** - Pin frequently used items to keep them persistent
5. **Deduplication** - Optional removal of duplicate entries
6. **Search & Filter** - Find entries by content, label, or type
7. **File Persistence** - Save and load history from disk

## File Location

**Module:** `/Users/yunwoopc/SIDE-PROJECT/agterm/src/clipboard_history.rs`

**Declaration:** Added to `/Users/yunwoopc/SIDE-PROJECT/agterm/src/lib.rs` as `pub mod clipboard_history;`

## Basic Usage

### Creating a History Manager

```rust
use agterm::clipboard_history::{ClipboardHistory, ClipboardType};

// Create with default settings (max 100 entries, deduplication enabled)
let mut history = ClipboardHistory::new(100);

// Create with custom settings
let mut history = ClipboardHistory::with_config(
    50,      // max_size
    true,    // deduplicate
    1,       // min_length
    1_000_000 // max_length (1MB)
);
```

### Adding Entries

```rust
// Add clipboard content
history.add("Hello, World!".to_string(), Some("terminal-1".to_string()));

// Returns false if content doesn't meet criteria
let added = history.add("x".to_string(), None); // Too short if min_length > 1
```

### Content Type Detection

The module automatically detects content types:

```rust
use agterm::clipboard_history::ClipboardType;

// Detected automatically when adding
history.add("https://example.com".to_string(), None);
history.add("/home/user/file.txt".to_string(), None);
history.add("user@example.com".to_string(), None);
history.add("fn main() {}".to_string(), None);

// Filter by type
let urls = history.filter_by_type(ClipboardType::Url);
let paths = history.filter_by_type(ClipboardType::Path);
let emails = history.filter_by_type(ClipboardType::Email);
let code = history.filter_by_type(ClipboardType::Code);
let text = history.filter_by_type(ClipboardType::Text);
```

### Pin Functionality

Pin important entries to keep them from being removed:

```rust
// Pin an entry by index
history.pin(0, Some("Important command".to_string()));

// Unpin an entry
history.unpin(0);

// Toggle pin status
history.toggle_pin(0, Some("My label".to_string()));

// Get all pinned entries
let pinned = history.pinned();

// Get pinned count
let count = history.pinned_count();
```

**Note:** Pinned entries are:
- Never removed when trimming to max_size
- Always saved first during persistence
- Can have optional labels for organization

### Search and Filter

```rust
// Search by content (case-insensitive)
let results = history.search("git");

// Search also matches labels and sources
let results = history.search("terminal");

// Filter by content type
let urls = history.filter_by_type(ClipboardType::Url);

// Get recent entries
let recent_5 = history.recent(5);

// Get all entries
let all = history.all();
```

### File Persistence

```rust
use std::path::PathBuf;

// Set file path and load existing entries
let path = PathBuf::from("/path/to/clipboard_history.json");
history.load_from_file(path)?;

// Save to file
history.save_to_file()?;
```

**Persistence Features:**
- Pinned entries are always saved
- Most recent unpinned entries are saved (up to max_size)
- JSON format for robust serialization
- Automatic directory creation

### Entry Management

```rust
// Get entry by index
if let Some(entry) = history.get(0) {
    println!("{}", entry.content);
    println!("Type: {:?}", entry.content_type);
    println!("Pinned: {}", entry.pinned);
}

// Get mutable entry
if let Some(entry) = history.get_mut(0) {
    entry.label = Some("Updated label".to_string());
}

// Remove entry
let removed = history.remove(0);

// Clear unpinned entries only
history.clear_unpinned();

// Clear all (including pinned)
history.clear();
```

### Entry Previews

```rust
let entry = history.get(0).unwrap();

// Get preview (first line, truncated to max length)
let preview = entry.preview(50);
println!("{}", preview); // "This is the first line..."

// Check if entry matches query
if entry.matches("keyword") {
    println!("Match found!");
}
```

## Data Structures

### ClipboardEntry

```rust
pub struct ClipboardEntry {
    pub content: String,              // The clipboard content
    pub timestamp: DateTime<Utc>,     // When copied
    pub source: Option<String>,       // Source identifier
    pub content_type: ClipboardType,  // Detected type
    pub pinned: bool,                 // Pin status
    pub label: Option<String>,        // Optional label
}
```

### ClipboardType Enum

```rust
pub enum ClipboardType {
    Text,   // Plain text
    Path,   // File path (/, ~/, C:\)
    Url,    // URL (http://, https://, ftp://, file://)
    Email,  // Email address (user@domain.com)
    Code,   // Code snippet (detected by keywords)
}
```

### ClipboardHistory

```rust
pub struct ClipboardHistory {
    entries: Vec<ClipboardEntry>,
    file_path: Option<PathBuf>,
    max_size: usize,
    deduplicate: bool,
    min_length: usize,
    max_length: usize,
}
```

## Configuration Options

### Deduplication

When enabled, duplicate content is removed (keeping the most recent):

```rust
let mut history = ClipboardHistory::with_config(100, true, 1, 1_000_000);
history.add("duplicate".to_string(), None);
history.add("unique".to_string(), None);
history.add("duplicate".to_string(), None);

assert_eq!(history.len(), 2); // Only one "duplicate" entry
```

### Length Limits

Set minimum and maximum content length:

```rust
let mut history = ClipboardHistory::with_config(
    100,        // max entries
    true,       // deduplicate
    5,          // min 5 characters
    1_000_000   // max 1MB
);

history.add("abc".to_string(), None);      // Rejected: too short
history.add("valid text".to_string(), None); // Accepted
```

### Max Size Behavior

The `max_size` only applies to unpinned entries:

```rust
let mut history = ClipboardHistory::new(2); // Max 2 unpinned

history.add("item1".to_string(), None);
history.add("item2".to_string(), None);
history.pin(0, Some("Keep".to_string())); // Pin first

history.add("item3".to_string(), None);
history.add("item4".to_string(), None);

// Result: item1 (pinned), item3, item4
// Total: 3 entries (1 pinned + 2 unpinned)
```

## Test Coverage

The module includes comprehensive tests for:

1. **Content Type Detection** - All types (URL, Path, Email, Code, Text)
2. **Add and Retrieve** - Basic operations
3. **Deduplication** - With and without
4. **Length Limits** - Min/max enforcement
5. **Max Size** - Trimming behavior
6. **Pin Functionality** - Pin/unpin/toggle
7. **Pin Persistence** - Pins survive trimming
8. **Search** - Content and label search
9. **Filter by Type** - Type-specific filtering
10. **Entry Previews** - Truncation and multiline
11. **Clear Operations** - Clear all vs unpinned
12. **Remove Entry** - Individual removal
13. **File Persistence** - Save and load
14. **Pinned Persistence** - Pinned entries always saved

## Integration Example

Here's how you might integrate this into AgTerm:

```rust
use agterm::clipboard_history::ClipboardHistory;
use std::path::PathBuf;
use dirs;

// Initialize in your app state
struct AppState {
    clipboard_history: ClipboardHistory,
    // ... other fields
}

impl AppState {
    fn new() -> Self {
        let mut history = ClipboardHistory::with_config(
            1000,    // Keep last 1000 entries
            true,    // Remove duplicates
            1,       // Min 1 char
            100_000  // Max 100KB
        );

        // Load from config directory
        if let Some(config_dir) = dirs::config_dir() {
            let path = config_dir.join("agterm/clipboard_history.json");
            let _ = history.load_from_file(path);
        }

        Self {
            clipboard_history: history,
        }
    }

    // When clipboard changes
    fn on_clipboard_change(&mut self, content: String, source: String) {
        self.clipboard_history.add(content, Some(source));
        let _ = self.clipboard_history.save_to_file();
    }

    // Show clipboard history UI
    fn show_clipboard_history(&self) {
        for entry in self.clipboard_history.recent(20) {
            println!("{} - {}",
                entry.preview(50),
                entry.timestamp.format("%Y-%m-%d %H:%M")
            );
        }
    }

    // Pin current clipboard
    fn pin_current(&mut self, label: String) {
        let latest_idx = self.clipboard_history.len() - 1;
        self.clipboard_history.pin(latest_idx, Some(label));
        let _ = self.clipboard_history.save_to_file();
    }
}
```

## Benefits

1. **Automatic Type Detection** - No manual classification needed
2. **Smart Deduplication** - Keeps history clean
3. **Pinned Items** - Never lose important snippets
4. **Efficient Search** - Find entries quickly
5. **Persistence** - Survives restarts
6. **Flexible Configuration** - Adapts to different use cases
7. **Memory Safe** - Automatic size limits
8. **Type Safe** - Strong Rust types throughout

## API Summary

### ClipboardHistory Methods

| Method | Description |
|--------|-------------|
| `new(max_size)` | Create with default settings |
| `with_config(...)` | Create with custom settings |
| `load_from_file(path)` | Load from JSON file |
| `save_to_file()` | Save to JSON file |
| `add(content, source)` | Add new entry |
| `pin(index, label)` | Pin entry |
| `unpin(index)` | Unpin entry |
| `toggle_pin(index, label)` | Toggle pin status |
| `search(query)` | Search entries |
| `filter_by_type(type)` | Filter by content type |
| `recent(n)` | Get recent n entries |
| `pinned()` | Get all pinned entries |
| `unpinned()` | Get all unpinned entries |
| `get(index)` | Get entry by index |
| `get_mut(index)` | Get mutable entry |
| `remove(index)` | Remove entry |
| `clear_unpinned()` | Clear unpinned only |
| `clear()` | Clear all entries |
| `len()` | Total entry count |
| `is_empty()` | Check if empty |
| `pinned_count()` | Count pinned entries |

### ClipboardEntry Methods

| Method | Description |
|--------|-------------|
| `new(content, source)` | Create new entry |
| `pinned(content, label)` | Create pinned entry |
| `preview(max_len)` | Get preview string |
| `matches(query)` | Check if matches query |

### ClipboardType Methods

| Method | Description |
|--------|-------------|
| `detect(content)` | Detect content type |

## Dependencies

Added to `Cargo.toml`:

```toml
chrono = { version = "0.4", features = ["serde"] }
```

All other dependencies were already present in the project.
