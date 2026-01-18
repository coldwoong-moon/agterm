//! Demonstration of the terminal split functionality
//!
//! This example shows how to use the SplitContainer to manage
//! terminal panes with tree-based splits.

use agterm::splits::{NavigationDirection, SplitContainer, SplitDirection};

fn main() {
    println!("Terminal Split Management Demo\n");

    // Create a new split container with a single pane
    let mut container = SplitContainer::new();
    println!("1. Created container with {} pane", container.pane_count());
    println!("   Focused pane: {}\n", container.focused_id());

    // Split horizontally (top/bottom)
    let pane1 = container.split_focused(SplitDirection::Horizontal);
    println!("2. Split horizontally - new pane: {}", pane1);
    println!("   Total panes: {}", container.pane_count());
    println!("   Focused pane: {}\n", container.focused_id());

    // Split the new pane vertically (left/right)
    let pane2 = container.split_focused(SplitDirection::Vertical);
    println!("3. Split vertically - new pane: {}", pane2);
    println!("   Total panes: {}", container.pane_count());
    println!("   Focused pane: {}\n", container.focused_id());

    // Show pane bounds
    println!("4. Pane bounds (normalized coordinates 0.0-1.0):");
    for id in container.get_all_ids() {
        if let Some((x, y, w, h)) = container.get_pane_bounds(id) {
            println!("   Pane {}: x={:.2}, y={:.2}, width={:.2}, height={:.2}", id, x, y, w, h);
        }
    }
    println!();

    // Navigate focus
    println!("5. Navigation:");
    println!("   Current focus: {}", container.focused_id());

    if container.navigate_focus(NavigationDirection::Up) {
        println!("   Navigated up to: {}", container.focused_id());
    }

    if container.navigate_focus(NavigationDirection::Left) {
        println!("   Navigated left to: {}\n", container.focused_id());
    }

    // Resize a split
    println!("6. Resizing:");
    let focused = container.focused_id();
    println!("   Before resize - Pane {} bounds:", focused);
    if let Some((x, y, w, h)) = container.get_pane_bounds(focused) {
        println!("   x={:.2}, y={:.2}, width={:.2}, height={:.2}", x, y, w, h);
    }

    container.resize_split(focused, 0.1);
    println!("   After resize +0.1 - Pane {} bounds:", focused);
    if let Some((x, y, w, h)) = container.get_pane_bounds(focused) {
        println!("   x={:.2}, y={:.2}, width={:.2}, height={:.2}\n", x, y, w, h);
    }

    // Close a pane
    println!("7. Closing panes:");
    println!("   Current pane count: {}", container.pane_count());

    if container.close_pane(pane2) {
        println!("   Closed pane {}", pane2);
        println!("   New pane count: {}", container.pane_count());
        println!("   New focused pane: {}\n", container.focused_id());
    }

    // Show final layout
    println!("8. Final layout:");
    println!("   Remaining panes: {:?}", container.get_all_ids());
    println!("   Focused pane: {}", container.focused_id());
}
