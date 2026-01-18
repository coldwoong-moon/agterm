# Diff Viewer Visual Examples

This document shows what the diff viewer output actually looks like.

## Side-by-Side View

```
                    Old                 |                   New
================================================================================
   1 Hello World                        |    1 Hello World
   2 This is a test                     |    2 This is a modified test
   3 Some unchanged line                |    3 Some unchanged line
   4 -Old line to be removed            |
                                        |    4 +New line that was added
   5 Another unchanged line             |    5 Another unchanged line
================================================================================
+1 -1 ~1 (Total: 3 changes)
```

**Key:**
- Lines with `-` prefix in red: Removed from old version
- Lines with `+` prefix in green: Added to new version
- Lines with `~` prefix in yellow: Modified between versions
- Lines with no prefix: Unchanged in both versions

## Unified View

```
Unified Diff
================================================================================
   1    1   Hello World
   2      - This is a test
        2 + This is a modified test
   3    3   Some unchanged line
   4      - Old line to be removed
        4 + New line that was added
   5    5   Another unchanged line
================================================================================
+1 -1 ~1 (Total: 3 changes)
```

**Format:**
- First column: Line number in old version (blank for additions)
- Second column: Line number in new version (blank for deletions)
- Third column: Change indicator (-, +, or space)
- Fourth column: Line content

## Real Code Comparison Example

### Old Code:
```rust
fn main() {
    let x = 5;
    println!("Hello");
    let y = 10;
    println!("x = {}", x);
}
```

### New Code:
```rust
fn main() {
    let x = 10;
    println!("Hello, World!");
    let y = 10;
    println!("x = {}, y = {}", x, y);
}
```

### Diff Output (Side-by-Side):
```
                    Old                 |                   New
================================================================================
   1 fn main() {                        |    1 fn main() {
   2 ~    let x = 5;                    |    2 ~    let x = 10;
   3 ~    println!("Hello");            |    3 ~    println!("Hello, World!");
   4     let y = 10;                    |    4     let y = 10;
   5 ~    println!("x = {}", x);        |    5 ~    println!("x = {}, y = {}", x, y);
   6 }                                  |    6 }
================================================================================
+0 -0 ~3 (Total: 3 changes)
```

## Configuration File Comparison

### Old Config:
```yaml
server:
  host: localhost
  port: 8080
  timeout: 30
database:
  name: mydb
  user: admin
```

### New Config:
```yaml
server:
  host: 0.0.0.0
  port: 3000
  timeout: 30
  ssl: true
database:
  name: mydb
  user: admin
  password: secret
```

### Diff Output (Unified):
```
Unified Diff
================================================================================
   1    1   server:
   2      - host: localhost
        2 + host: 0.0.0.0
   3      - port: 8080
        3 + port: 3000
   4    4   timeout: 30
        5 + ssl: true
   5    6   database:
   6    7   name: mydb
   7    8   user: admin
        9 + password: secret
================================================================================
+3 -2 ~0 (Total: 5 changes)
```

## Edge Cases

### Empty File Comparison
```
                    Old                 |                   New
================================================================================
                                        |    1 +New content added
                                        |    2 +Another line
================================================================================
+2 -0 ~0 (Total: 2 changes)
```

### Identical Files
```
                    Old                 |                   New
================================================================================
   1 Line 1                             |    1 Line 1
   2 Line 2                             |    2 Line 2
   3 Line 3                             |    3 Line 3
================================================================================
+0 -0 ~0 (Total: 0 changes)

Files are identical
```

### All Lines Changed
```
Unified Diff
================================================================================
   1      - Old line 1
        1 + New line 1
   2      - Old line 2
        2 + New line 2
   3      - Old line 3
        3 + New line 3
================================================================================
+3 -3 ~0 (Total: 6 changes)
```

## Color Coding (ANSI Terminal)

When displayed in a terminal with color support:

🟢 **Green** - Added lines
```
   4 +New line that was added
```

🔴 **Red** - Removed lines
```
   4 -Old line to be removed
```

🟡 **Yellow** - Modified lines
```
   2 ~    let x = 10;
```

⚪ **Default** - Unchanged lines
```
   1     let y = 10;
```

## Navigation Indicators

When navigating, the current line is marked with `>`:

```
Unified Diff
================================================================================
   1    1   Hello World
>  2      - This is a test            <- Current position
        2 + This is a modified test
   3    3   Some unchanged line
   4      - Old line to be removed
        4 + New line that was added
   5    5   Another unchanged line
================================================================================
```

## Statistics Display

At the bottom of each diff:

```
+5 -3 ~2 (Total: 10 changes)
```

Breakdown:
- `+5`: 5 lines added (green)
- `-3`: 3 lines removed (red)
- `~2`: 2 lines modified (yellow)
- `(Total: 10 changes)`: Total number of changes

## Long Line Truncation

When lines exceed terminal width, they are automatically truncated:

```
                    Old                 |                   New
================================================================================
   1 This is a very long line that ex...|    1 This is a very long line that ex...
   2 Normal line                        |    2 Normal line
================================================================================
```

## Multi-Line Changes

When multiple consecutive lines are changed:

```
Unified Diff
================================================================================
   1    1   Header
   2      - Old line A
   3      - Old line B
   4      - Old line C
        2 + New line X
        3 + New line Y
        4 + New line Z
   5    5   Footer
================================================================================
+3 -3 ~0 (Total: 6 changes)
```

## Command-Line Usage Example

```bash
$ cargo run --example diff_files old.txt new.txt

Comparing: old.txt vs new.txt

                    Old                 |                   New
================================================================================
   1 Hello World                        |    1 Hello World
   2 This is a test                     |    2 This is a modified test
================================================================================
+0 -0 ~1 (Total: 1 changes)

Summary:
  Files differ in 1 location(s)
  ~1 line(s) modified
```

## Interactive Navigation Example

When using the navigation API:

```rust
let mut viewer = DiffViewer::new(result, 80);

// Move to first change
viewer.next_change();
println!("At line: {}", viewer.current_line()); // Output: At line: 1

// Move to next change
viewer.next_change();
println!("At line: {}", viewer.current_line()); // Output: At line: 5

// Move back
viewer.prev_change();
println!("At line: {}", viewer.current_line()); // Output: At line: 1
```

---

These examples demonstrate the visual output of the diff viewer in various scenarios. The actual terminal output includes proper ANSI color codes for enhanced readability.
