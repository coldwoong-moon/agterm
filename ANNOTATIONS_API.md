# Annotation System API Reference

Quick reference for the AgTerm annotation system.

## Core Types

### `AnnotationType`

```rust
pub enum AnnotationType {
    Note,      // 📝 General-purpose annotation
    Warning,   // ⚠️ Alert or important issue
    Bookmark,  // 🔖 Navigation marker
}
```

**Methods:**
- `default_color() -> [u8; 3]` - Get default RGB color
- `symbol() -> &'static str` - Get emoji symbol
- `name() -> &'static str` - Get display name

### `LineRange`

```rust
pub struct LineRange {
    pub start: usize,
    pub end: usize,
}
```

**Constructors:**
- `LineRange::single(line: usize)` - Single line
- `LineRange::new(start: usize, end: usize)` - Line range

**Methods:**
- `contains(&self, line: usize) -> bool`
- `overlaps(&self, other: &LineRange) -> bool`
- `len(&self) -> usize`
- `is_single_line(&self) -> bool`

### `Annotation`

```rust
pub struct Annotation {
    pub id: String,                     // UUID
    pub range: LineRange,               // Lines this applies to
    pub content: String,                // Annotation text
    pub annotation_type: AnnotationType,
    pub color: Option<[u8; 3]>,        // Custom color (RGB)
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub tags: Vec<String>,
}
```

**Constructors:**
```rust
// Quick creation
Annotation::note(line: usize, content: String) -> Annotation
Annotation::warning(line: usize, content: String) -> Annotation
Annotation::bookmark(line: usize, content: String) -> Annotation

// Custom creation
Annotation::new(
    range: LineRange,
    content: String,
    annotation_type: AnnotationType
) -> Annotation
```

**Methods:**
```rust
set_content(&mut self, content: String)
set_color(&mut self, color: [u8; 3])
effective_color(&self) -> [u8; 3]
add_tag(&mut self, tag: String)
remove_tag(&mut self, tag: &str) -> bool
has_tag(&self, tag: &str) -> bool
applies_to_line(&self, line: usize) -> bool
```

### `AnnotationManager`

```rust
pub struct AnnotationManager {
    // Internal fields omitted
}
```

**Constructors:**
```rust
AnnotationManager::new() -> AnnotationManager
AnnotationManager::with_max_annotations(max: usize) -> AnnotationManager
```

**Adding/Removing:**
```rust
add(&mut self, annotation: Annotation) -> String          // Returns ID
remove(&mut self, id: &str) -> Option<Annotation>
```

**Updating:**
```rust
update_content(&mut self, id: &str, content: String) -> bool
update_color(&mut self, id: &str, color: [u8; 3]) -> bool
```

**Querying:**
```rust
get(&self, id: &str) -> Option<&Annotation>
get_for_line(&self, line: usize) -> Vec<&Annotation>
get_by_type(&self, annotation_type: AnnotationType) -> Vec<&Annotation>
get_bookmarks(&self) -> Vec<&Annotation>              // Sorted by line
all_sorted(&self) -> Vec<&Annotation>                 // Sorted by line

has_annotations_at_line(&self, line: usize) -> bool
```

**Searching:**
```rust
search(&self, query: &str) -> Vec<&Annotation>        // Case-insensitive content search
search_by_tag(&self, tag: &str) -> Vec<&Annotation>
```

**Navigation:**
```rust
next_bookmark(&self, current_line: usize) -> Option<&Annotation>
prev_bookmark(&self, current_line: usize) -> Option<&Annotation>
```

**Statistics:**
```rust
count(&self) -> usize
count_by_type(&self, annotation_type: AnnotationType) -> usize
stats(&self) -> AnnotationStats
```

**Clearing:**
```rust
clear(&mut self)
clear_by_type(&mut self, annotation_type: AnnotationType)
```

**Persistence:**
```rust
load_from_file(&mut self, path: PathBuf) -> std::io::Result<()>
save_to_file(&self) -> std::io::Result<()>
```

### `AnnotationStats`

```rust
pub struct AnnotationStats {
    pub total: usize,
    pub notes: usize,
    pub warnings: usize,
    pub bookmarks: usize,
}
```

## Common Patterns

### Creating Annotations

```rust
// Simple note
let note = Annotation::note(10, "Important".to_string());

// Warning with custom color
let mut warning = Annotation::warning(20, "Check this".to_string());
warning.set_color([255, 0, 0]);

// Multi-line bookmark with tags
let mut bookmark = Annotation::new(
    LineRange::new(30, 35),
    "Function definition".to_string(),
    AnnotationType::Bookmark,
);
bookmark.add_tag("function".to_string());
```

### Managing Annotations

```rust
let mut manager = AnnotationManager::new();

// Add
let id = manager.add(annotation);

// Update
manager.update_content(&id, "New content".to_string());

// Remove
manager.remove(&id);
```

### Querying

```rust
// By line
for ann in manager.get_for_line(10) {
    println!("{}", ann.content);
}

// By type
let warnings = manager.get_by_type(AnnotationType::Warning);

// By content
let results = manager.search("error");

// By tag
let todos = manager.search_by_tag("todo");
```

### Navigation

```rust
// Forward
if let Some(next) = manager.next_bookmark(current_line) {
    jump_to_line(next.range.start);
}

// Backward
if let Some(prev) = manager.prev_bookmark(current_line) {
    jump_to_line(prev.range.start);
}

// All bookmarks
for bookmark in manager.get_bookmarks() {
    println!("Line {}: {}", bookmark.range.start, bookmark.content);
}
```

### Persistence

```rust
use std::path::PathBuf;

let mut manager = AnnotationManager::new();
let path = PathBuf::from("annotations.json");

// Load
manager.load_from_file(path.clone())?;

// ... work with annotations ...

// Save
manager.save_to_file()?;
```

## Performance Notes

| Operation | Complexity | Notes |
|-----------|------------|-------|
| `add()` | O(n) | n = lines in range |
| `remove()` | O(n) | n = lines in range |
| `get()` | O(1) | ID lookup |
| `get_for_line()` | O(1) + O(m) | m = annotations on line |
| `search()` | O(n) | n = total annotations |
| `next_bookmark()` | O(log n) | Due to sorting |
| `prev_bookmark()` | O(log n) | Due to sorting |

## Examples

### Example 1: Basic Usage

```rust
use agterm::annotations::{Annotation, AnnotationManager};

fn main() {
    let mut manager = AnnotationManager::new();

    // Add a note
    let id = manager.add(Annotation::note(10, "Important".to_string()));

    // Query it
    if let Some(ann) = manager.get(&id) {
        println!("Note: {}", ann.content);
    }

    // Update it
    manager.update_content(&id, "Very important".to_string());
}
```

### Example 2: Bookmark Navigation

```rust
use agterm::annotations::{Annotation, AnnotationManager};

fn setup_bookmarks(manager: &mut AnnotationManager) {
    manager.add(Annotation::bookmark(10, "Start".to_string()));
    manager.add(Annotation::bookmark(50, "Middle".to_string()));
    manager.add(Annotation::bookmark(100, "End".to_string()));
}

fn navigate(manager: &AnnotationManager, current: usize) {
    if let Some(next) = manager.next_bookmark(current) {
        println!("Next: line {}", next.range.start);
    }
}
```

### Example 3: Tagged Search

```rust
use agterm::annotations::{Annotation, AnnotationManager};

fn mark_todos(manager: &mut AnnotationManager, lines: &[usize]) {
    for &line in lines {
        let mut ann = Annotation::note(line, "TODO".to_string());
        ann.add_tag("todo".to_string());
        ann.add_tag("priority-high".to_string());
        manager.add(ann);
    }
}

fn find_todos(manager: &AnnotationManager) -> Vec<usize> {
    manager.search_by_tag("todo")
        .iter()
        .map(|ann| ann.range.start)
        .collect()
}
```

### Example 4: Persistence

```rust
use agterm::annotations::{Annotation, AnnotationManager};
use std::path::PathBuf;

fn save_session(manager: &AnnotationManager, path: PathBuf) -> std::io::Result<()> {
    let mut manager_clone = manager.clone();
    manager_clone.file_path = Some(path);
    manager_clone.save_to_file()
}

fn load_session(path: PathBuf) -> std::io::Result<AnnotationManager> {
    let mut manager = AnnotationManager::new();
    manager.load_from_file(path)?;
    Ok(manager)
}
```

### Example 5: Statistics Dashboard

```rust
use agterm::annotations::{AnnotationManager, AnnotationType};

fn print_dashboard(manager: &AnnotationManager) {
    let stats = manager.stats();

    println!("=== Annotation Dashboard ===");
    println!("Total: {}", stats.total);
    println!("Notes: {} ({}%)", stats.notes,
        stats.notes * 100 / stats.total.max(1));
    println!("Warnings: {} ({}%)", stats.warnings,
        stats.warnings * 100 / stats.total.max(1));
    println!("Bookmarks: {} ({}%)", stats.bookmarks,
        stats.bookmarks * 100 / stats.total.max(1));

    println!("\nRecent bookmarks:");
    for bookmark in manager.get_bookmarks().iter().rev().take(5) {
        println!("  Line {}: {}", bookmark.range.start, bookmark.content);
    }
}
```

## Error Handling

All I/O operations return `std::io::Result`:

```rust
use std::path::PathBuf;

fn handle_persistence(manager: &mut AnnotationManager) {
    let path = PathBuf::from("annotations.json");

    match manager.load_from_file(path.clone()) {
        Ok(_) => println!("Loaded successfully"),
        Err(e) => eprintln!("Failed to load: {}", e),
    }

    match manager.save_to_file() {
        Ok(_) => println!("Saved successfully"),
        Err(e) => eprintln!("Failed to save: {}", e),
    }
}
```

## Integration with AgTerm

To integrate annotations into AgTerm's terminal UI:

```rust
use agterm::annotations::{AnnotationManager, AnnotationType};

// In your terminal state
struct TerminalState {
    annotations: AnnotationManager,
    current_line: usize,
    // ... other fields
}

impl TerminalState {
    fn render_line(&self, line_num: usize, content: &str) {
        // Get annotations for this line
        let annotations = self.annotations.get_for_line(line_num);

        if !annotations.is_empty() {
            // Render annotation indicators
            for ann in annotations {
                let color = ann.effective_color();
                let symbol = ann.annotation_type.symbol();
                // Render symbol with color in the gutter
            }
        }

        // Render line content
    }

    fn handle_annotation_keybind(&mut self, key: Key) {
        match key {
            Key::AddNote => {
                let ann = Annotation::note(self.current_line, "".to_string());
                self.annotations.add(ann);
            }
            Key::NextBookmark => {
                if let Some(next) = self.annotations.next_bookmark(self.current_line) {
                    self.current_line = next.range.start;
                }
            }
            // ... other keybinds
        }
    }
}
```

## Testing

Run tests with:

```bash
# Unit tests
cargo test annotations::tests

# Integration tests
cargo test --test annotations_integration_test

# With output
cargo test annotations -- --nocapture
```

## Thread Safety

`AnnotationManager` is not thread-safe by default. For multi-threaded use:

```rust
use std::sync::{Arc, Mutex};

type SharedAnnotations = Arc<Mutex<AnnotationManager>>;

fn share_manager(manager: AnnotationManager) -> SharedAnnotations {
    Arc::new(Mutex::new(manager))
}
```
