# AgTerm Diff Viewer

A comprehensive terminal-based diff viewer implementing the Myers diff algorithm for efficient text comparison.

## Features

- **Myers Diff Algorithm**: Efficient O(ND) difference algorithm for computing the shortest edit script
- **Multiple View Modes**: Side-by-side and unified diff views
- **Change Navigation**: Navigate between changes with next/previous functionality
- **Syntax Highlighting**: Color-coded output for different change types
- **Statistics**: Detailed change statistics (added, removed, modified, unchanged)
- **Terminal-Friendly**: Proper line truncation and formatting for terminal display

## Quick Start

```rust
use agterm::diff_view::{diff_strings, MyersDiff, DiffViewer, DiffViewMode};

// Simple string comparison
let old = "Hello World\nThis is a test";
let new = "Hello World\nThis is modified";

let output = diff_strings(old, new, 80);
println!("{}", output);
```

## Core Components

### DiffLine

Represents a single line in a diff with its type and content.

```rust
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub left_content: Option<String>,
    pub right_content: Option<String>,
    pub left_line_num: Option<usize>,
    pub right_line_num: Option<usize>,
}
```

**Line Types:**
- `Added`: Line exists only in the new version
- `Removed`: Line exists only in the old version
- `Modified`: Line changed between versions
- `Unchanged`: Line is identical in both versions

### DiffResult

Contains the complete diff with statistics.

```rust
pub struct DiffResult {
    pub lines: Vec<DiffLine>,
    pub stats: DiffStats,
}
```

**Statistics:**
```rust
pub struct DiffStats {
    pub added: usize,
    pub removed: usize,
    pub modified: usize,
    pub unchanged: usize,
}
```

### MyersDiff

Implements the Myers diff algorithm.

```rust
let old_lines = vec!["line1".to_string(), "line2".to_string()];
let new_lines = vec!["line1".to_string(), "modified".to_string()];

let diff = MyersDiff::new(old_lines, new_lines);
let result = diff.compute();

println!("Added: {}", result.stats.added);
println!("Removed: {}", result.stats.removed);
println!("Modified: {}", result.stats.modified);
```

### DiffViewer

Renders diffs with navigation support.

```rust
let mut viewer = DiffViewer::new(result, 100);

// Switch view modes
viewer.set_mode(DiffViewMode::SideBySide);
println!("{}", viewer.render());

viewer.set_mode(DiffViewMode::Unified);
println!("{}", viewer.render());

// Navigate changes
viewer.next_change();  // Move to next change
viewer.prev_change();  // Move to previous change
```

## View Modes

### Side-by-Side View

Displays old and new versions in parallel columns.

```
        Old                 |         New
================================================================================
   1 Hello World            |    1 Hello World
   2 This is a test         |    2 This is a modified test
   3 Some unchanged line    |    3 Some unchanged line
   4 -Old line to remove    |
                            |    4 +New line added
   5 Another line           |    5 Another line
================================================================================
+1 -1 ~1 (Total: 3 changes)
```

### Unified View

Displays changes in a single column with markers.

```
Unified Diff
================================================================================
   1    1   Hello World
   2    2 ~ This is a test
        2 + This is a modified test
   3    3   Some unchanged line
   4      - Old line to remove
        4 + New line added
   5    5   Another line
================================================================================
+1 -1 ~1 (Total: 3 changes)
```

## Color Coding

- **Green** (`\x1b[32m`): Added lines (prefix: `+`)
- **Red** (`\x1b[31m`): Removed lines (prefix: `-`)
- **Yellow** (`\x1b[33m`): Modified lines (prefix: `~`)
- **Default**: Unchanged lines (no prefix)

## Navigation

```rust
let mut viewer = DiffViewer::new(result, 80);

// Get current position
let current = viewer.current_line();

// Navigate to next change
if viewer.next_change() {
    println!("Moved to line {}", viewer.current_line());
}

// Navigate to previous change
if viewer.prev_change() {
    println!("Moved to line {}", viewer.current_line());
}

// Get all change indices
let changes = viewer.result().change_indices();
println!("Changes at lines: {:?}", changes);

// Jump to specific line
viewer.set_current_line(5);
```

## Myers Algorithm

The implementation uses Eugene W. Myers' O(ND) difference algorithm, which finds the shortest edit script (SES) between two sequences.

**Algorithm Complexity:**
- Time: O((N+M)D) where N and M are sequence lengths, D is the edit distance
- Space: O(N+M)

**Key Concepts:**
1. **Edit Graph**: Represents possible edit operations as a graph
2. **Diagonals**: Paths with matching elements
3. **Snake**: Maximal diagonal segment with matching elements
4. **D-paths**: Paths with exactly D edit operations

**Advantages:**
- Optimal: Always finds the shortest edit script
- Efficient: Fast for common cases with few differences
- Clean output: Produces minimal, intuitive diffs

## Use Cases

### 1. File Comparison

```rust
use std::fs;

let old_file = fs::read_to_string("old.txt")?;
let new_file = fs::read_to_string("new.txt")?;

let output = diff_strings(&old_file, &new_file, 120);
println!("{}", output);
```

### 2. Git-Style Diff

```rust
let old_lines: Vec<String> = old_content.lines().map(String::from).collect();
let new_lines: Vec<String> = new_content.lines().map(String::from).collect();

let diff = MyersDiff::new(old_lines, new_lines);
let result = diff.compute();

for line in &result.lines {
    match line.line_type {
        DiffLineType::Added => println!("+{}", line.right_content.as_ref().unwrap()),
        DiffLineType::Removed => println!("-{}", line.left_content.as_ref().unwrap()),
        DiffLineType::Modified => {
            println!("-{}", line.left_content.as_ref().unwrap());
            println!("+{}", line.right_content.as_ref().unwrap());
        }
        DiffLineType::Unchanged => println!(" {}", line.left_content.as_ref().unwrap()),
    }
}
```

### 3. Configuration Diff

```rust
let old_config = load_config("old_config.toml");
let new_config = load_config("new_config.toml");

let diff = MyersDiff::new(old_config, new_config);
let result = diff.compute();

if result.stats.total_changes() > 0 {
    println!("Configuration changes detected:");
    println!("  Added: {}", result.stats.added);
    println!("  Removed: {}", result.stats.removed);
    println!("  Modified: {}", result.stats.modified);
}
```

### 4. Code Review

```rust
let mut viewer = DiffViewer::new(result, terminal_width);
viewer.set_mode(DiffViewMode::SideBySide);

// Show only lines with changes
for idx in viewer.result().change_indices() {
    viewer.set_current_line(idx);
    println!("{}", viewer.render());
}
```

## Advanced Features

### Custom Filtering

```rust
// Filter to show only specific change types
let added_lines: Vec<_> = result.lines
    .iter()
    .filter(|line| line.line_type == DiffLineType::Added)
    .collect();

println!("Added {} new lines", added_lines.len());
```

### Change Statistics

```rust
let stats = &result.stats;

println!("Diff Statistics:");
println!("  Total lines: {}", stats.total_lines());
println!("  Total changes: {}", stats.total_changes());
println!("  Change ratio: {:.1}%",
    (stats.total_changes() as f64 / stats.total_lines() as f64) * 100.0);
```

### Context Extraction

```rust
// Extract context around changes (N lines before and after)
fn extract_context(result: &DiffResult, change_idx: usize, context_lines: usize) -> Vec<&DiffLine> {
    let start = change_idx.saturating_sub(context_lines);
    let end = (change_idx + context_lines + 1).min(result.lines.len());

    result.lines[start..end].iter().collect()
}
```

## Testing

The module includes comprehensive tests covering:

- Line creation and types
- Statistics calculation
- Myers algorithm correctness
- Edge cases (empty, single line, all additions/removals)
- Navigation functionality
- View mode rendering
- Line truncation

Run tests:
```bash
cargo test diff_view
```

Run the demo:
```bash
cargo run --example diff_viewer_demo
```

## Performance Considerations

1. **Line Length**: Long lines are automatically truncated to fit terminal width
2. **Large Files**: For very large files, consider processing in chunks
3. **Memory**: The algorithm stores the entire diff in memory
4. **Rendering**: ANSI escape codes add minimal overhead

## References

- [An O(ND) Difference Algorithm and Its Variations](http://www.xmailserver.org/diff2.pdf) by Eugene W. Myers
- [Understanding the Myers Diff Algorithm](https://blog.jcoglan.com/2017/02/12/the-myers-diff-algorithm-part-1/)

## License

MIT License - See main project LICENSE file.
