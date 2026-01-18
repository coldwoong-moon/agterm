# Terminal Annotation System Guide

The AgTerm annotation system provides a powerful way to mark, comment, and bookmark terminal output lines for later reference.

## Features

### 1. Annotation Types

The system supports three types of annotations:

- **Notes** (📝): General-purpose annotations for any kind of information
- **Warnings** (⚠️): Alerts for important issues or things to watch out for
- **Bookmarks** (🔖): Quick navigation markers for important locations

Each type has its own default color:
- Notes: Cornflower blue (RGB: 100, 149, 237)
- Warnings: Orange (RGB: 255, 165, 0)
- Bookmarks: Lime green (RGB: 50, 205, 50)

### 2. Line Ranges

Annotations can apply to:
- **Single lines**: Mark a specific line
- **Line ranges**: Mark multiple consecutive lines

Example:
```rust
// Single line
let note = Annotation::note(10, "Important command".to_string());

// Line range
let warning = Annotation::new(
    LineRange::new(20, 25),
    "Critical section".to_string(),
    AnnotationType::Warning,
);
```

### 3. Tags

Annotations can have multiple tags for categorization:

```rust
let mut annotation = Annotation::note(50, "Review this".to_string());
annotation.add_tag("performance".to_string());
annotation.add_tag("todo".to_string());
annotation.add_tag("high-priority".to_string());
```

### 4. Custom Colors

Override default colors with custom RGB values:

```rust
annotation.set_color([255, 0, 0]); // Red
```

### 5. Persistence

Annotations are automatically saved to disk and restored on session start:

```rust
let mut manager = AnnotationManager::new();
manager.load_from_file(PathBuf::from("annotations.json"))?;

// ... add/modify annotations ...

manager.save_to_file()?;
```

## Usage

### Creating Annotations

#### Quick Creation Methods

```rust
use agterm::annotations::{Annotation, AnnotationType};

// Create a note
let note = Annotation::note(10, "This is a note".to_string());

// Create a warning
let warning = Annotation::warning(20, "Watch out here".to_string());

// Create a bookmark
let bookmark = Annotation::bookmark(30, "Important spot".to_string());
```

#### Custom Creation

```rust
use agterm::annotations::{Annotation, AnnotationType, LineRange};

let annotation = Annotation::new(
    LineRange::new(40, 45),
    "Multi-line annotation".to_string(),
    AnnotationType::Warning,
);
```

### Managing Annotations

```rust
use agterm::annotations::AnnotationManager;

let mut manager = AnnotationManager::new();

// Add an annotation
let id = manager.add(annotation);

// Get an annotation
if let Some(ann) = manager.get(&id) {
    println!("Content: {}", ann.content);
}

// Update content
manager.update_content(&id, "New content".to_string());

// Update color
manager.update_color(&id, [0, 255, 0]);

// Remove an annotation
manager.remove(&id);
```

### Querying Annotations

#### By Line Number

```rust
// Get all annotations for a specific line
let annotations = manager.get_for_line(10);
for ann in annotations {
    println!("{}: {}", ann.annotation_type.name(), ann.content);
}

// Check if a line has annotations
if manager.has_annotations_at_line(10) {
    println!("Line 10 has annotations");
}
```

#### By Type

```rust
// Get all notes
let notes = manager.get_by_type(AnnotationType::Note);

// Get all warnings
let warnings = manager.get_by_type(AnnotationType::Warning);

// Get all bookmarks
let bookmarks = manager.get_by_type(AnnotationType::Bookmark);
```

#### By Content Search

```rust
// Case-insensitive content search
let results = manager.search("error");
for ann in results {
    println!("Line {}: {}", ann.range.start, ann.content);
}
```

#### By Tag

```rust
// Find all annotations with a specific tag
let tagged = manager.search_by_tag("todo");
for ann in tagged {
    println!("Line {}: {}", ann.range.start, ann.content);
}
```

### Bookmark Navigation

Bookmarks provide quick navigation through important locations:

```rust
// Get all bookmarks sorted by line number
let bookmarks = manager.get_bookmarks();

// Navigate forward from current line
if let Some(next) = manager.next_bookmark(current_line) {
    println!("Next bookmark at line {}", next.range.start);
}

// Navigate backward from current line
if let Some(prev) = manager.prev_bookmark(current_line) {
    println!("Previous bookmark at line {}", prev.range.start);
}
```

### Statistics

```rust
let stats = manager.stats();
println!("Total: {}", stats.total);
println!("Notes: {}", stats.notes);
println!("Warnings: {}", stats.warnings);
println!("Bookmarks: {}", stats.bookmarks);

// Or count by type directly
let note_count = manager.count_by_type(AnnotationType::Note);
```

### Clearing Annotations

```rust
// Clear all annotations
manager.clear();

// Clear annotations of a specific type
manager.clear_by_type(AnnotationType::Note);
```

## Architecture

### Core Components

1. **Annotation**: Represents a single annotation with:
   - Unique ID (UUID)
   - Line range
   - Content text
   - Type (note/warning/bookmark)
   - Optional custom color
   - Creation and modification timestamps
   - Optional tags

2. **AnnotationManager**: Manages all annotations with:
   - HashMap for fast ID-based lookup
   - Line index for fast line-based queries
   - File persistence
   - Automatic trimming when max annotations exceeded

3. **LineRange**: Represents a range of lines with:
   - Start and end line numbers
   - Methods for overlap detection
   - Single-line vs multi-line detection

### Data Structures

```rust
// Annotation storage
HashMap<String, Annotation>  // ID -> Annotation

// Line index for fast queries
HashMap<usize, Vec<String>>  // Line number -> Annotation IDs
```

### Performance

- **Add**: O(n) where n is the number of lines in the range
- **Remove**: O(n) where n is the number of lines in the range
- **Get by ID**: O(1)
- **Get by line**: O(1) + O(m) where m is the number of annotations on that line
- **Search**: O(n) where n is the total number of annotations
- **Next/Prev bookmark**: O(log n) due to sorting

### File Format

Annotations are stored as JSON Lines (one JSON object per line):

```json
{"id":"550e8400-e29b-41d4-a716-446655440000","range":{"start":10,"end":10},"content":"Important note","annotation_type":"Note","color":null,"created_at":"2024-01-15T10:30:00Z","modified_at":"2024-01-15T10:30:00Z","tags":["important"]}
{"id":"550e8400-e29b-41d4-a716-446655440001","range":{"start":20,"end":25},"content":"Warning section","annotation_type":"Warning","color":[255,0,0],"created_at":"2024-01-15T10:31:00Z","modified_at":"2024-01-15T10:31:00Z","tags":[]}
```

## Configuration

### Maximum Annotations

Limit the number of annotations to prevent unbounded growth:

```rust
let manager = AnnotationManager::with_max_annotations(5000);
```

When the limit is exceeded, the oldest annotations are automatically removed.

## Best Practices

### 1. Use Appropriate Types

- Use **Notes** for general information and documentation
- Use **Warnings** for potential issues or things requiring attention
- Use **Bookmarks** for navigation points you'll return to frequently

### 2. Tag Strategically

Use tags for cross-cutting concerns:
```rust
annotation.add_tag("security".to_string());
annotation.add_tag("performance".to_string());
annotation.add_tag("todo".to_string());
```

### 3. Keep Content Concise

Make annotation content clear but brief:
```rust
// Good
"Check return value"

// Too verbose
"This function might return an error so we should check the return value and handle it appropriately otherwise we might have issues"
```

### 4. Save Periodically

For interactive applications, save annotations periodically to prevent data loss:
```rust
// Save after significant changes
manager.save_to_file()?;
```

### 5. Clean Up Old Annotations

Regularly remove annotations that are no longer relevant:
```rust
// Remove specific annotations
manager.remove(&old_id);

// Or clear by type
manager.clear_by_type(AnnotationType::Note);
```

## Example: Interactive Session

Here's a complete example of using annotations in an interactive terminal session:

```rust
use agterm::annotations::{Annotation, AnnotationManager, AnnotationType};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize manager with persistence
    let mut manager = AnnotationManager::new();
    let annotations_file = dirs::config_dir()
        .unwrap()
        .join("agterm")
        .join("annotations.json");

    manager.load_from_file(annotations_file)?;

    // User runs a command at line 100
    // They want to mark it as important
    let note = Annotation::bookmark(
        100,
        "Initial setup command".to_string()
    );
    manager.add(note);

    // Later, they see an error at line 150
    let mut warning = Annotation::warning(
        150,
        "Connection timeout - check network".to_string()
    );
    warning.add_tag("error".to_string());
    warning.add_tag("network".to_string());
    manager.add(warning);

    // They want to navigate back to their bookmarks
    println!("Bookmarks:");
    for bookmark in manager.get_bookmarks() {
        println!("  Line {}: {}", bookmark.range.start, bookmark.content);
    }

    // Search for network-related annotations
    let network_issues = manager.search_by_tag("network");
    println!("\nNetwork-related issues: {}", network_issues.len());

    // Save changes
    manager.save_to_file()?;

    Ok(())
}
```

## Testing

The annotation system includes comprehensive tests covering:

- Annotation creation and modification
- Line range operations
- Manager operations (add, remove, update)
- Queries (by line, type, content, tag)
- Bookmark navigation
- File persistence
- Maximum annotations trimming

Run tests with:
```bash
cargo test annotations::tests
```

## Future Enhancements

Possible future improvements:

1. **Shared annotations**: Synchronize annotations across team members
2. **Annotation history**: Track changes to annotations over time
3. **Rich text**: Support formatting in annotation content
4. **Attachments**: Link files or screenshots to annotations
5. **Export**: Export annotations to various formats (Markdown, HTML, etc.)
6. **Smart search**: Fuzzy matching and regex support in searches
7. **Annotation groups**: Organize related annotations together
8. **Time-based queries**: Find annotations created/modified in a time range
