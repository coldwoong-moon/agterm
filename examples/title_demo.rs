//! Demo of dynamic terminal title functionality
//!
//! This example demonstrates how to use OSC sequences to set terminal titles.
//!
//! # Usage
//!
//! Run this in AgTerm and watch the tab title change:
//!
//! ```bash
//! cargo run --example title_demo
//! ```

use std::io::{self, Write};
use std::thread;
use std::time::Duration;

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    println!("=== AgTerm Dynamic Title Demo ===\n");
    println!("Watch the tab title change as different OSC sequences are sent.\n");

    // Example 1: OSC 0 - Set both icon and window title
    writeln!(handle, "Sending OSC 0 (both icon and window title)...")?;
    write!(handle, "\x1b]0;Example 1: Both Titles\x07")?;
    handle.flush()?;
    thread::sleep(Duration::from_secs(3));

    // Example 2: OSC 2 - Set window title only
    writeln!(handle, "\nSending OSC 2 (window title only)...")?;
    write!(handle, "\x1b]2;Example 2: Window Title\x07")?;
    handle.flush()?;
    thread::sleep(Duration::from_secs(3));

    // Example 3: OSC 1 - Set icon title only
    writeln!(handle, "\nSending OSC 1 (icon title only)...")?;
    write!(handle, "\x1b]1;Icon\x07")?;
    handle.flush()?;
    thread::sleep(Duration::from_secs(3));

    // Example 4: Simulating editor
    writeln!(handle, "\nSimulating vim editor...")?;
    write!(handle, "\x1b]0;vim ~/.bashrc\x07")?;
    handle.flush()?;
    thread::sleep(Duration::from_secs(3));

    // Example 5: Simulating command
    writeln!(handle, "\nSimulating cargo build...")?;
    write!(handle, "\x1b]0;cargo build --release\x07")?;
    handle.flush()?;
    thread::sleep(Duration::from_secs(3));

    // Example 6: Directory path
    writeln!(handle, "\nShowing directory path...")?;
    write!(handle, "\x1b]0;~/projects/agterm\x07")?;
    handle.flush()?;
    thread::sleep(Duration::from_secs(3));

    // Reset to default
    writeln!(handle, "\nResetting to default title...")?;
    write!(handle, "\x1b]0;\x07")?;
    handle.flush()?;

    println!("\nDemo complete!");
    println!("Press Enter to exit...");

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(())
}
