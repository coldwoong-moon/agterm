# AgTerm Terminal Annotation System

> Mark, comment, and bookmark terminal output lines for easy reference and navigation.

## Quick Start

```rust
use agterm::annotations::{Annotation, AnnotationManager};

// Create manager
let mut manager = AnnotationManager::new();

// Add a bookmark
manager.add(Annotation::bookmark(50, "Important section".to_string()));

// Navigate to bookmarks
if let Some(next) = manager.next_bookmark(current_line) {
    // Jump to next.range.start
}
```

## Features

- **Three Annotation Types**: Notes (📝), Warnings (⚠️), Bookmarks (🔖)
- **Line Ranges**: Annotate single lines or line ranges
- **Rich Search**: Search by content, type, tag, or line number
- **Navigation**: Quick bookmark navigation (next/previous)
- **Persistence**: Auto-save/load annotations to disk
- **Tags**: Categorize annotations with custom tags
- **Custom Colors**: Override default colors with RGB values

## Documentation

| Document | Description |
|----------|-------------|
| [ANNOTATIONS_GUIDE.md](./ANNOTATIONS_GUIDE.md) | Complete user guide with examples |
| [ANNOTATIONS_API.md](./ANNOTATIONS_API.md) | API reference for developers |
| [ANNOTATIONS_IMPLEMENTATION_SUMMARY.md](./ANNOTATIONS_IMPLEMENTATION_SUMMARY.md) | Technical implementation details |

## Examples

### Add Annotations

```rust
// Simple note
let note = Annotation::note(10, "Remember to check this".to_string());
manager.add(note);

// Warning with custom color
let mut warning = Annotation::warning(20, "Potential issue".to_string());
warning.set_color([255, 0, 0]); // Red
manager.add(warning);

// Multi-line annotation
let annotation = Annotation::new(
    LineRange::new(30, 35),
    "Critical section".to_string(),
    AnnotationType::Warning,
);
manager.add(annotation);
```

### Search and Query

```rust
// Get annotations for a line
let annotations = manager.get_for_line(10);

// Search by content
let results = manager.search("error");

// Get all bookmarks
let bookmarks = manager.get_bookmarks();

// Search by tag
let todos = manager.search_by_tag("todo");
```

### Bookmark Navigation

```rust
// Navigate forward
if let Some(next) = manager.next_bookmark(current_line) {
    println!("Next bookmark at line {}", next.range.start);
}

// Navigate backward
if let Some(prev) = manager.prev_bookmark(current_line) {
    println!("Previous bookmark at line {}", prev.range.start);
}
```

### Persistence

```rust
use std::path::PathBuf;

// Load from file
let path = PathBuf::from("~/.config/agterm/annotations.json");
manager.load_from_file(path.clone())?;

// Work with annotations...

// Save to file
manager.save_to_file()?;
```

## Demo Application

Run the demo to see all features in action:

```bash
cargo run --example annotations_demo
```

Output:
```
=== AgTerm Annotation System Demo ===

1. Creating annotations...
   Added note at line 10: 550e8400-e29b-41d4-a716-446655440000
   Added warning at line 25: 550e8400-e29b-41d4-a716-446655440001
   Added bookmark at line 50
   ...

2. Statistics:
   Total annotations: 7
   Notes: 3
   Warnings: 2
   Bookmarks: 2
   ...
```

## Testing

```bash
# Run all annotation tests
cargo test annotations

# Run unit tests
cargo test annotations::tests

# Run integration tests
cargo test --test annotations_integration_test

# Run with output
cargo test annotations -- --nocapture
```

## Architecture

### Core Components

```rust
AnnotationType    // Note, Warning, Bookmark
LineRange         // Single line or range
Annotation        // Individual annotation with metadata
AnnotationManager // Central management system
```

### Storage

- **Main Storage**: `HashMap<String, Annotation>` for O(1) ID lookup
- **Line Index**: `HashMap<usize, Vec<String>>` for O(1) line queries
- **File Format**: JSON Lines for persistence

### Performance

| Operation | Complexity |
|-----------|------------|
| Add/Remove | O(n) where n = lines in range |
| Get by ID | O(1) |
| Get by line | O(1) + O(m) where m = annotations on line |
| Search | O(n) where n = total annotations |
| Next/Prev bookmark | O(log n) |

## API Overview

### Creating Annotations

```rust
Annotation::note(line, content)
Annotation::warning(line, content)
Annotation::bookmark(line, content)
Annotation::new(range, content, type)
```

### Managing Annotations

```rust
manager.add(annotation) -> String
manager.remove(&id) -> Option<Annotation>
manager.update_content(&id, content) -> bool
manager.update_color(&id, color) -> bool
```

### Querying

```rust
manager.get(&id) -> Option<&Annotation>
manager.get_for_line(line) -> Vec<&Annotation>
manager.get_by_type(type) -> Vec<&Annotation>
manager.search(query) -> Vec<&Annotation>
manager.search_by_tag(tag) -> Vec<&Annotation>
```

### Navigation

```rust
manager.next_bookmark(current_line) -> Option<&Annotation>
manager.prev_bookmark(current_line) -> Option<&Annotation>
manager.get_bookmarks() -> Vec<&Annotation>
```

### Statistics

```rust
manager.count() -> usize
manager.count_by_type(type) -> usize
manager.stats() -> AnnotationStats
```

### Persistence

```rust
manager.load_from_file(path) -> Result<()>
manager.save_to_file() -> Result<()>
```

## Configuration

```rust
// Default configuration
let manager = AnnotationManager::new(); // max 10,000 annotations

// Custom max annotations
let manager = AnnotationManager::with_max_annotations(5000);
```

## Integration with AgTerm

### Suggested Keybindings

- `Ctrl+N`: Add note to current line
- `Ctrl+W`: Add warning to current line
- `Ctrl+B`: Add bookmark to current line
- `F2`: Next bookmark
- `Shift+F2`: Previous bookmark
- `Ctrl+A`: Show all annotations
- `Ctrl+F`: Search annotations

### UI Elements

1. **Gutter Indicators**: Show annotation symbols with colors
2. **Hover Tooltips**: Display annotation content on hover
3. **Annotation Panel**: List all annotations with filtering
4. **Search Panel**: Search by content or tags
5. **Context Menu**: Right-click menu for annotation operations

### Session Integration

```rust
struct TerminalSession {
    annotations: AnnotationManager,
    // ... other fields
}

impl TerminalSession {
    fn new() -> Self {
        let mut annotations = AnnotationManager::new();

        // Load from session-specific file
        let path = get_session_annotations_path();
        let _ = annotations.load_from_file(path);

        Self { annotations }
    }

    fn save(&self) {
        let _ = self.annotations.save_to_file();
    }
}
```

## File Format

Annotations are stored as JSON Lines (one JSON object per line):

```json
{"id":"550e8400-...","range":{"start":10,"end":10},"content":"Important","annotation_type":"Note","color":null,"created_at":"2024-01-15T10:30:00Z","modified_at":"2024-01-15T10:30:00Z","tags":["important"]}
```

## Best Practices

### 1. Use Appropriate Types

- **Notes**: General information and documentation
- **Warnings**: Issues requiring attention
- **Bookmarks**: Navigation points

### 2. Keep Content Concise

```rust
// Good
"Check return value"

// Too verbose
"This function might return an error so we should..."
```

### 3. Use Tags for Organization

```rust
annotation.add_tag("security");
annotation.add_tag("performance");
annotation.add_tag("todo");
```

### 4. Save Periodically

```rust
// Auto-save after changes
manager.add(annotation);
manager.save_to_file()?;
```

### 5. Clean Up Old Annotations

```rust
// Remove specific annotations
manager.remove(&old_id);

// Clear by type
manager.clear_by_type(AnnotationType::Note);
```

## License

MIT - Part of the AgTerm project

## Related

- [AgTerm README](./README.md)
- [AgTerm Documentation](./docs/)
- [Contributing Guidelines](./CONTRIBUTING.md)

## Support

For issues, questions, or contributions:
- GitHub Issues: https://github.com/coldwoong-moon/agterm/issues
- Discussions: https://github.com/coldwoong-moon/agterm/discussions
