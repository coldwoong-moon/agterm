//! Diff Viewer Demo
//!
//! This example demonstrates how to use the diff viewer functionality.
//!
//! Run with: cargo run --example diff_viewer_demo --features iced-gui
//!
//! Requires the `iced-gui` feature to be enabled.

#[cfg(not(feature = "iced-gui"))]
fn main() {
    eprintln!("This example requires the `iced-gui` feature. Run with:");
    eprintln!("  cargo run --example diff_viewer_demo --features iced-gui");
}

#[cfg(feature = "iced-gui")]
mod demo {
    use agterm::diff_view::{diff_strings, DiffViewMode, DiffViewer, MyersDiff};

    pub fn run() {
        println!("=== AgTerm Diff Viewer Demo ===\n");

        // Example 1: Simple text diff
        println!("Example 1: Simple Text Comparison");
        println!("{}", "=".repeat(80));

        let old_text = "\
Hello World
This is a test
Some unchanged line
Old line to be removed
Another unchanged line";

        let new_text = "\
Hello World
This is a modified test
Some unchanged line
New line that was added
Another unchanged line";

        let output = diff_strings(old_text, new_text, 80);
        println!("{}\n", output);

        // Example 2: Code diff with side-by-side view
        println!("Example 2: Code Comparison (Side-by-Side)");
        println!("{}", "=".repeat(80));

        let old_code = "\
fn main() {
    let x = 5;
    println!(\"Hello\");
    let y = 10;
    println!(\"x = {}\", x);
}";

        let new_code = "\
fn main() {
    let x = 10;
    println!(\"Hello, World!\");
    let y = 10;
    println!(\"x = {}, y = {}\", x, y);
}";

        let old_lines: Vec<String> = old_code.lines().map(|s| s.to_string()).collect();
        let new_lines: Vec<String> = new_code.lines().map(|s| s.to_string()).collect();

        let diff = MyersDiff::new(old_lines, new_lines);
        let result = diff.compute();

        let viewer = DiffViewer::new(result, 100);
        println!("{}\n", viewer.render());

        // Example 3: Unified diff view
        println!("Example 3: Unified Diff View");
        println!("{}", "=".repeat(80));

        let old_config = "\
server:
  host: localhost
  port: 8080
  timeout: 30
database:
  name: mydb
  user: admin";

        let new_config = "\
server:
  host: 0.0.0.0
  port: 3000
  timeout: 30
  ssl: true
database:
  name: mydb
  user: admin
  password: secret";

        let old_lines: Vec<String> = old_config.lines().map(|s| s.to_string()).collect();
        let new_lines: Vec<String> = new_config.lines().map(|s| s.to_string()).collect();

        let diff = MyersDiff::new(old_lines, new_lines);
        let result = diff.compute();

        let mut viewer = DiffViewer::new(result, 100);
        viewer.set_mode(DiffViewMode::Unified);
        println!("{}\n", viewer.render());

        // Example 4: Navigation between changes
        println!("Example 4: Change Navigation");
        println!("{}", "=".repeat(80));

        let old_text = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6";
        let new_text = "Line 1\nModified Line 2\nLine 3\nLine 4\nNew Line 5\nLine 6";

        let old_lines: Vec<String> = old_text.lines().map(|s| s.to_string()).collect();
        let new_lines: Vec<String> = new_text.lines().map(|s| s.to_string()).collect();

        let diff = MyersDiff::new(old_lines, new_lines);
        let result = diff.compute();

        println!("Total changes: {}", result.stats.total_changes());
        println!("Added: {}", result.stats.added);
        println!("Removed: {}", result.stats.removed);
        println!("Modified: {}", result.stats.modified);
        println!("Unchanged: {}", result.stats.unchanged);
        println!("\nChange locations: {:?}", result.change_indices());

        let mut viewer = DiffViewer::new(result, 100);
        viewer.set_mode(DiffViewMode::Unified);

        println!("\n--- Initial view (line 0) ---");
        println!("Current line: {}", viewer.current_line());

        if viewer.next_change() {
            println!("\n--- After next_change() ---");
            println!("Current line: {}", viewer.current_line());
        }

        if viewer.next_change() {
            println!("\n--- After another next_change() ---");
            println!("Current line: {}", viewer.current_line());
        }

        if viewer.prev_change() {
            println!("\n--- After prev_change() ---");
            println!("Current line: {}", viewer.current_line());
        }

        // Example 5: Empty and single line diffs
        println!("\n\nExample 5: Edge Cases");
        println!("{}", "=".repeat(80));

        // Empty diff
        let empty_diff = MyersDiff::new(vec![], vec![]);
        let empty_result = empty_diff.compute();
        println!(
            "Empty diff - Total lines: {}",
            empty_result.stats.total_lines()
        );

        // Single line change
        let single_old = vec!["old".to_string()];
        let single_new = vec!["new".to_string()];
        let single_diff = MyersDiff::new(single_old, single_new);
        let single_result = single_diff.compute();
        println!("Single line diff - Modified: {}", single_result.stats.modified);

        // All additions
        let add_diff = MyersDiff::new(vec![], vec!["line1".to_string(), "line2".to_string()]);
        let add_result = add_diff.compute();
        println!("All additions - Added: {}", add_result.stats.added);

        // All removals
        let remove_diff = MyersDiff::new(vec!["line1".to_string(), "line2".to_string()], vec![]);
        let remove_result = remove_diff.compute();
        println!("All removals - Removed: {}", remove_result.stats.removed);

        println!("\n=== Demo Complete ===");
    }
}

#[cfg(feature = "iced-gui")]
fn main() {
    demo::run();
}
