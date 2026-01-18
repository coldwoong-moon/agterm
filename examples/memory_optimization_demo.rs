//! Demo program showing memory optimization features
//!
//! This example demonstrates:
//! 1. String interning for URLs
//! 2. Memory usage tracking
//! 3. Automatic cleanup of unused interned strings

use agterm::terminal::screen::TerminalScreen;

fn main() {
    println!("AgTerm Memory Optimization Demo");
    println!("================================\n");

    // Create a terminal screen
    let mut screen = TerminalScreen::new(80, 24);

    // Initial memory stats
    println!("Initial state:");
    println!("{}\n", screen.memory_usage_string());

    // Simulate writing some content with URLs
    let test_data = vec![
        "Visit https://github.com/coldwoong-moon/agterm for more info\n",
        "Documentation at https://docs.rs/agterm\n",
        "Same URL again: https://github.com/coldwoong-moon/agterm\n",
        "And again: https://github.com/coldwoong-moon/agterm\n",
        "Another site: https://www.rust-lang.org\n",
    ];

    for data in test_data {
        screen.process(data.as_bytes());
    }

    // Detect URLs (this will trigger string interning)
    screen.detect_urls();

    println!("After processing text with URLs:");
    println!("{}\n", screen.memory_usage_string());

    let stats = screen.memory_stats();
    println!("Detailed statistics:");
    println!("  Buffer lines: {}", stats.buffer_lines);
    println!("  Scrollback lines: {}", stats.scrollback_lines);
    println!("  Buffer memory: {} bytes", stats.buffer_bytes);
    println!("  Scrollback memory: {} bytes", stats.scrollback_bytes);
    println!("  String interner: {} bytes", stats.interner_bytes);
    println!("  Interned strings: {}", stats.interned_strings);
    println!("  Interner hits: {}", stats.interner_hits);
    println!("  Interner misses: {}", stats.interner_misses);

    if stats.interned_strings > 0 {
        let hit_rate = (stats.interner_hits as f64)
            / ((stats.interner_hits + stats.interner_misses) as f64)
            * 100.0;
        println!("  Hit rate: {:.1}%", hit_rate);
        println!("\nString interning is working!");
        println!(
            "The same URL 'https://github.com/coldwoong-moon/agterm' appeared {} times",
            stats.interner_hits + 1
        );
        println!(
            "but is only stored {} time(s) in memory.",
            stats.interned_strings
        );
    }

    // Fill scrollback with more data
    println!("\n\nFilling scrollback buffer...");
    for i in 0..100 {
        let line = format!("Line {} with URL https://example.com/{}\n", i, i % 5);
        screen.process(line.as_bytes());
    }
    screen.detect_urls();

    println!("After filling scrollback:");
    println!("{}\n", screen.memory_usage_string());

    // Manual cleanup
    screen.cleanup_interner();
    println!("After manual interner cleanup:");
    println!("{}\n", screen.memory_usage_string());

    println!("\nMemory optimization features demonstrated:");
    println!("  1. String interning reduces memory for repeated URLs");
    println!("  2. Memory tracking provides visibility into usage");
    println!("  3. Automatic periodic cleanup prevents unbounded growth");
    println!("  4. Manual cleanup available when needed");
}
