//! Test scrollback buffer compression effectiveness
//!
//! This example demonstrates the compression ratio achieved by the
//! scrollback buffer compression system.

use agterm::terminal::screen::TerminalScreen;

fn main() {
    let mut screen = TerminalScreen::new(80, 24);

    println!("Testing AgTerm Scrollback Buffer Compression");
    println!("=============================================\n");

    // Test 1: Generate lots of scrollback with typical terminal output
    println!("Test 1: Simulating typical terminal output...");
    for i in 0..1000 {
        let line = format!(
            "Line {}: This is a typical terminal line with some text\n",
            i
        );
        screen.process(line.as_bytes());
    }

    let stats = screen.compression_stats();
    println!("  Lines in scrollback: {}", screen.scrollback_size());
    println!("  Total lines compressed: {}", stats.total_lines);
    println!(
        "  Original size: {} bytes ({:.2} KB)",
        stats.total_uncompressed,
        stats.total_uncompressed as f64 / 1024.0
    );
    println!(
        "  Compressed size: {} bytes ({:.2} KB)",
        stats.total_compressed,
        stats.total_compressed as f64 / 1024.0
    );
    println!(
        "  Space saved: {} bytes ({:.2} KB)",
        stats.space_saved(),
        stats.space_saved() as f64 / 1024.0
    );
    println!("  Compression ratio: {:.1}%", stats.space_saved_percent());
    println!(
        "  Average compression: {:.2}x smaller\n",
        1.0 / stats.avg_ratio
    );

    // Test 2: Best case - lots of empty lines
    let mut screen2 = TerminalScreen::new(80, 24);
    println!("Test 2: Best case (empty lines)...");
    for _ in 0..1000 {
        screen2.process(b"\n");
    }

    let stats2 = screen2.compression_stats();
    println!("  Lines in scrollback: {}", screen2.scrollback_size());
    println!(
        "  Original size: {} bytes ({:.2} KB)",
        stats2.total_uncompressed,
        stats2.total_uncompressed as f64 / 1024.0
    );
    println!(
        "  Compressed size: {} bytes ({:.2} KB)",
        stats2.total_compressed,
        stats2.total_compressed as f64 / 1024.0
    );
    println!(
        "  Space saved: {} bytes ({:.2} KB)",
        stats2.space_saved(),
        stats2.space_saved() as f64 / 1024.0
    );
    println!("  Compression ratio: {:.1}%", stats2.space_saved_percent());
    println!(
        "  Average compression: {:.2}x smaller\n",
        1.0 / stats2.avg_ratio
    );

    // Test 3: Worst case - random characters
    let mut screen3 = TerminalScreen::new(80, 24);
    println!("Test 3: Worst case (dense random content)...");
    for i in 0..1000 {
        let line = format!(
            "{:80}\n",
            format!("Random{}{}{}{}", i, i * 2, i * 3, i * 4)
                .chars()
                .cycle()
                .take(80)
                .collect::<String>()
        );
        screen3.process(line.as_bytes());
    }

    let stats3 = screen3.compression_stats();
    println!("  Lines in scrollback: {}", screen3.scrollback_size());
    println!(
        "  Original size: {} bytes ({:.2} KB)",
        stats3.total_uncompressed,
        stats3.total_uncompressed as f64 / 1024.0
    );
    println!(
        "  Compressed size: {} bytes ({:.2} KB)",
        stats3.total_compressed,
        stats3.total_compressed as f64 / 1024.0
    );
    println!(
        "  Space saved: {} bytes ({:.2} KB)",
        stats3.space_saved(),
        stats3.space_saved() as f64 / 1024.0
    );
    println!("  Compression ratio: {:.1}%", stats3.space_saved_percent());
    println!(
        "  Average compression: {:.2}x smaller\n",
        1.0 / stats3.avg_ratio
    );

    println!("\nSummary:");
    println!("========");
    println!("The run-length encoding compression works best with:");
    println!("  - Lines with trailing spaces (typical terminal output)");
    println!("  - Empty or sparse lines");
    println!("  - Repeated patterns");
    println!("\nFor typical terminal usage, expect 70-90% space savings.");
}
