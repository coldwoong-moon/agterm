# AgTerm Diff Viewer

A powerful, terminal-friendly diff viewer implementing the Myers diff algorithm.

## Quick Start

```rust
use agterm::diff_view::diff_strings;

let old = "Hello World\nThis is a test";
let new = "Hello World\nThis is modified";

println!("{}", diff_strings(old, new, 80));
```

## Features

✨ **Myers Diff Algorithm** - Optimal O(ND) difference computation
🎨 **Multiple Views** - Side-by-side and unified modes
🧭 **Smart Navigation** - Jump between changes easily
📊 **Statistics** - Track added, removed, and modified lines
🎯 **Terminal-Optimized** - ANSI colors and proper line wrapping
✅ **Well-Tested** - 33 comprehensive tests

## Installation

The diff viewer is included in AgTerm. Simply use:

```rust
use agterm::diff_view::*;
```

## Usage Examples

### Basic String Comparison

```rust
use agterm::diff_view::diff_strings;

let output = diff_strings("old text", "new text", 80);
println!("{}", output);
```

### Advanced Usage with Custom Viewer

```rust
use agterm::diff_view::{MyersDiff, DiffViewer, DiffViewMode};

let old_lines: Vec<String> = old_text.lines().map(String::from).collect();
let new_lines: Vec<String> = new_text.lines().map(String::from).collect();

let diff = MyersDiff::new(old_lines, new_lines);
let result = diff.compute();

let mut viewer = DiffViewer::new(result, 100);
viewer.set_mode(DiffViewMode::SideBySide);

println!("{}", viewer.render());
```

### Navigating Changes

```rust
let mut viewer = DiffViewer::new(result, 80);

// Jump to next change
while viewer.next_change() {
    println!("Change at line {}", viewer.current_line());
}

// Jump back
while viewer.prev_change() {
    println!("Previous change at {}", viewer.current_line());
}
```

### Getting Statistics

```rust
let stats = result.stats;
println!("Added: {}", stats.added);
println!("Removed: {}", stats.removed);
println!("Modified: {}", stats.modified);
println!("Total changes: {}", stats.total_changes());
```

## Command-Line Tools

### Compare Files

```bash
cargo run --example diff_files old.txt new.txt
cargo run --example diff_files old.txt new.txt --unified
cargo run --example diff_files old.txt new.txt --width 120
```

### Run Demo

```bash
cargo run --example diff_viewer_demo
```

## View Modes

### Side-by-Side

```
        Old                 |         New
======================================================
   1 Hello World            |    1 Hello World
   2 This is a test         |    2 This is modified
   3 -Removed line          |
                            |    3 +Added line
======================================================
+1 -1 ~1 (Total: 3 changes)
```

### Unified

```
Unified Diff
======================================================
   1    1   Hello World
   2      - This is a test
        2 + This is modified
   3      - Removed line
        3 + Added line
======================================================
+1 -1 ~1 (Total: 3 changes)
```

## Color Legend

- 🟢 **Green (+)**: Added lines
- 🔴 **Red (-)**: Removed lines
- 🟡 **Yellow (~)**: Modified lines
- ⚪ **White**: Unchanged lines

## API Reference

### Types

```rust
pub enum DiffLineType {
    Added,      // New line
    Removed,    // Deleted line
    Modified,   // Changed line
    Unchanged,  // Same in both
}

pub struct DiffLine {
    pub line_type: DiffLineType,
    pub left_content: Option<String>,
    pub right_content: Option<String>,
    pub left_line_num: Option<usize>,
    pub right_line_num: Option<usize>,
}

pub struct DiffStats {
    pub added: usize,
    pub removed: usize,
    pub modified: usize,
    pub unchanged: usize,
}

pub struct DiffResult {
    pub lines: Vec<DiffLine>,
    pub stats: DiffStats,
}

pub enum DiffViewMode {
    SideBySide,
    Unified,
}
```

### Main Functions

```rust
// Convenience function
pub fn diff_strings(old: &str, new: &str, terminal_width: usize) -> String

// Myers algorithm
impl MyersDiff {
    pub fn new(old: Vec<String>, new: Vec<String>) -> Self
    pub fn compute(&self) -> DiffResult
}

// Viewer
impl DiffViewer {
    pub fn new(result: DiffResult, terminal_width: usize) -> Self
    pub fn render(&self) -> String
    pub fn set_mode(&mut self, mode: DiffViewMode)
    pub fn next_change(&mut self) -> bool
    pub fn prev_change(&mut self) -> bool
    pub fn current_line(&self) -> usize
    pub fn set_current_line(&mut self, line: usize)
    pub fn result(&self) -> &DiffResult
}

// Navigation helpers
impl DiffResult {
    pub fn change_indices(&self) -> Vec<usize>
    pub fn next_change(&self, current_idx: usize) -> Option<usize>
    pub fn prev_change(&self, current_idx: usize) -> Option<usize>
}
```

## Testing

```bash
# Run all diff_view tests
cargo test diff_view

# Run integration tests
cargo test --test diff_view_integration

# Run specific test
cargo test test_myers_diff_complex
```

## Performance

- **Time**: O((N+M)D) where N, M are lengths, D is edit distance
- **Space**: O(N+M)
- **Best case**: O(N+M) for identical or very similar files
- **Worst case**: O((N+M)²) for completely different files

## Examples in the Wild

### File Comparison
```rust
let old = fs::read_to_string("version1.rs")?;
let new = fs::read_to_string("version2.rs")?;
println!("{}", diff_strings(&old, &new, 120));
```

### Configuration Changes
```rust
let old_config = load_config("old.toml");
let new_config = load_config("new.toml");

let diff = MyersDiff::new(old_config, new_config);
let result = diff.compute();

if result.stats.total_changes() > 0 {
    println!("Config changed!");
}
```

### Code Review Tool
```rust
fn review_changes(before: &str, after: &str) {
    let old: Vec<_> = before.lines().map(String::from).collect();
    let new: Vec<_> = after.lines().map(String::from).collect();

    let diff = MyersDiff::new(old, new);
    let result = diff.compute();

    let mut viewer = DiffViewer::new(result, 100);

    // Show each change
    while viewer.next_change() {
        println!("\n--- Change at line {} ---", viewer.current_line());
        // Display surrounding context
    }
}
```

## Documentation

- **Full API Docs**: See `docs/DIFF_VIEWER.md`
- **Implementation Details**: See `DIFF_VIEWER_IMPLEMENTATION.md`
- **Algorithm Reference**: [Myers Diff Paper](http://www.xmailserver.org/diff2.pdf)

## Contributing

Improvements welcome! Areas for enhancement:

- Word-level diffing
- Ignore whitespace option
- Patch generation
- Three-way merge
- Syntax highlighting

## License

MIT License - Same as AgTerm project

## Credits

Algorithm: Eugene W. Myers
Implementation: AgTerm Team
Inspired by: Git, diff-match-patch

---

**Questions?** Check the examples or open an issue!
