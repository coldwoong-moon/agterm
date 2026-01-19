//! Example: Using the Menu System
//!
//! This example demonstrates how to integrate the custom menu bar
//! into the AgTerm application.
//!
//! Run with: cargo run --example menu_example --features floem-gui --no-default-features

#[cfg(feature = "floem-gui")]
fn main() {
    use agterm::logging::LoggingConfig;

    // Initialize logging
    let log_config = LoggingConfig::default();
    agterm::logging::init_logging(&log_config);

    tracing::info!("Starting AgTerm Menu Example");

    // To enable the menu bar in the application:
    // 1. Import the menu module
    // 2. Call menu_bar_view() in the app_view() function
    // 3. Add it to the v_stack before the tab bar

    println!("\n=== AgTerm Menu System Example ===\n");
    println!("The menu system has been implemented with the following structure:\n");

    println!("File Menu:");
    println!("  - New Tab (Cmd+T)");
    println!("  - New Window (Cmd+N)");
    println!("  - Close Tab (Cmd+W)");
    println!("  - Close Window (Cmd+Shift+W)");
    println!();

    println!("Edit Menu:");
    println!("  - Copy (Cmd+C)");
    println!("  - Paste (Cmd+V)");
    println!("  - Select All (Cmd+A)");
    println!();

    println!("View Menu:");
    println!("  - Zoom In (Cmd++)");
    println!("  - Zoom Out (Cmd+-)");
    println!("  - Reset Zoom (Cmd+0)");
    println!("  - Toggle Theme");
    println!();

    println!("Window Menu:");
    println!("  - Split Vertically (Cmd+D)");
    println!("  - Split Horizontally (Cmd+Shift+D)");
    println!("  - Next Pane (Cmd+Tab)");
    println!("  - Previous Pane (Cmd+Shift+Tab)");
    println!();

    println!("To enable the visual menu bar:");
    println!("  1. Edit src/floem_app/mod.rs");
    println!("  2. Import: use menu::menu_bar_view;");
    println!("  3. Add menu_bar_view(&app_state) to v_stack");
    println!();

    println!("All keyboard shortcuts are already working in the main application!");
    println!();

    println!("For more details, see: MENU_IMPLEMENTATION.md");
}

#[cfg(not(feature = "floem-gui"))]
fn main() {
    eprintln!("This example requires the 'floem-gui' feature.");
    eprintln!("Run with: cargo run --example menu_example --features floem-gui --no-default-features");
    std::process::exit(1);
}
