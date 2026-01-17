//! Terminal sequence processing benchmarks for AgTerm
//!
//! This benchmark suite measures:
//! - VTE parsing speed for various ANSI sequences
//! - Terminal emulator control sequence handling
//! - Cursor movement operations
//! - Screen manipulation commands

use agterm::terminal::screen::TerminalScreen;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

/// Benchmark basic VTE parsing with plain text
fn bench_vte_plain_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("vte_plain_text");

    for size in [100, 500, 1000, 5000].iter() {
        let text = "a".repeat(*size);
        let bytes = text.as_bytes();
        group.throughput(Throughput::Bytes(bytes.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut screen = TerminalScreen::new(80, 24);
                screen.process(black_box(bytes));
            });
        });
    }

    group.finish();
}

/// Benchmark cursor movement sequences
fn bench_cursor_movement(c: &mut Criterion) {
    let mut group = c.benchmark_group("cursor_movement");

    let test_sequences = vec![
        ("cursor_up", b"\x1b[A".to_vec()),
        ("cursor_down", b"\x1b[B".to_vec()),
        ("cursor_forward", b"\x1b[C".to_vec()),
        ("cursor_backward", b"\x1b[D".to_vec()),
        ("cursor_position", b"\x1b[10;20H".to_vec()),
        ("cursor_home", b"\x1b[H".to_vec()),
        (
            "cursor_complex",
            b"\x1b[10;20HText\x1b[A\x1b[5CMore\x1b[B".to_vec(),
        ),
    ];

    for (name, sequence) in test_sequences {
        group.bench_function(name, |b| {
            b.iter(|| {
                let mut screen = TerminalScreen::new(80, 24);
                for _ in 0..100 {
                    screen.process(black_box(&sequence));
                }
            });
        });
    }

    group.finish();
}

/// Benchmark erase sequences (clear screen, clear line, etc.)
fn bench_erase_sequences(c: &mut Criterion) {
    let mut group = c.benchmark_group("erase_sequences");

    let test_sequences = vec![
        ("erase_in_display_all", b"\x1b[2J".to_vec()),
        ("erase_in_display_below", b"\x1b[0J".to_vec()),
        ("erase_in_display_above", b"\x1b[1J".to_vec()),
        ("erase_in_line_all", b"\x1b[2K".to_vec()),
        ("erase_in_line_right", b"\x1b[0K".to_vec()),
        ("erase_in_line_left", b"\x1b[1K".to_vec()),
    ];

    for (name, sequence) in test_sequences {
        group.bench_function(name, |b| {
            b.iter(|| {
                let mut screen = TerminalScreen::new(80, 24);
                // Fill screen first
                for _ in 0..24 {
                    screen.process(b"Test line with content\r\n");
                }
                // Then erase
                screen.process(black_box(&sequence));
            });
        });
    }

    group.finish();
}

/// Benchmark SGR (Select Graphic Rendition) sequences
fn bench_sgr_sequences(c: &mut Criterion) {
    let mut group = c.benchmark_group("sgr_sequences");

    let test_sequences = vec![
        ("bold", b"\x1b[1mBold Text\x1b[0m".to_vec()),
        ("dim", b"\x1b[2mDim Text\x1b[0m".to_vec()),
        ("italic", b"\x1b[3mItalic Text\x1b[0m".to_vec()),
        ("underline", b"\x1b[4mUnderline Text\x1b[0m".to_vec()),
        ("blink", b"\x1b[5mBlink Text\x1b[0m".to_vec()),
        ("reverse", b"\x1b[7mReverse Text\x1b[0m".to_vec()),
        ("strikethrough", b"\x1b[9mStrike Text\x1b[0m".to_vec()),
        (
            "combined",
            b"\x1b[1;4;31mBold Underline Red\x1b[0m".to_vec(),
        ),
        ("fg_color_basic", b"\x1b[31mRed\x1b[0m".to_vec()),
        ("bg_color_basic", b"\x1b[41mRed BG\x1b[0m".to_vec()),
        ("fg_color_256", b"\x1b[38;5;196mBright Red\x1b[0m".to_vec()),
        ("bg_color_256", b"\x1b[48;5;21mBlue BG\x1b[0m".to_vec()),
        (
            "fg_color_rgb",
            b"\x1b[38;2;255;0;0mRGB Red\x1b[0m".to_vec(),
        ),
        (
            "bg_color_rgb",
            b"\x1b[48;2;0;0;255mRGB Blue BG\x1b[0m".to_vec(),
        ),
    ];

    for (name, sequence) in test_sequences {
        group.bench_function(name, |b| {
            b.iter(|| {
                let mut screen = TerminalScreen::new(80, 24);
                for _ in 0..100 {
                    screen.process(black_box(&sequence));
                }
            });
        });
    }

    group.finish();
}

/// Benchmark scrolling operations
fn bench_scrolling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scrolling");

    let test_sequences = vec![
        ("scroll_up_1", b"\x1b[S".to_vec()),
        ("scroll_down_1", b"\x1b[T".to_vec()),
        ("scroll_up_5", b"\x1b[5S".to_vec()),
        ("scroll_down_5", b"\x1b[5T".to_vec()),
        ("insert_line", b"\x1b[L".to_vec()),
        ("delete_line", b"\x1b[M".to_vec()),
    ];

    for (name, sequence) in test_sequences {
        group.bench_function(name, |b| {
            b.iter(|| {
                let mut screen = TerminalScreen::new(80, 24);
                // Fill screen first
                for _ in 0..24 {
                    screen.process(b"Test line\r\n");
                }
                // Then scroll
                for _ in 0..10 {
                    screen.process(black_box(&sequence));
                }
            });
        });
    }

    group.finish();
}

/// Benchmark alternate screen buffer switching
fn bench_alternate_screen(c: &mut Criterion) {
    c.bench_function("alternate_screen_switch", |b| {
        b.iter(|| {
            let mut screen = TerminalScreen::new(80, 24);
            // Fill main screen
            for _ in 0..24 {
                screen.process(b"Main screen content\r\n");
            }
            // Switch to alternate screen
            screen.process(b"\x1b[?1049h");
            // Fill alternate screen
            for _ in 0..24 {
                screen.process(b"Alternate screen content\r\n");
            }
            // Switch back
            screen.process(b"\x1b[?1049l");
        });
    });
}

/// Benchmark terminal reset
fn bench_terminal_reset(c: &mut Criterion) {
    c.bench_function("terminal_reset", |b| {
        b.iter(|| {
            let mut screen = TerminalScreen::new(80, 24);
            // Fill with styled content
            for _ in 0..24 {
                screen.process(b"\x1b[1;31mBold Red Text\x1b[0m\r\n");
            }
            // Reset
            screen.process(b"\x1bc"); // RIS - Reset to Initial State
        });
    });
}

/// Benchmark mixed content processing (realistic terminal output)
fn bench_mixed_content(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_content");

    // Simulate various realistic terminal outputs
    let test_cases = vec![
        (
            "git_status",
            b"\x1b[1mOn branch main\x1b[0m\n\
              \x1b[1mYour branch is up to date with 'origin/main'.\x1b[0m\n\n\
              \x1b[1mChanges not staged for commit:\x1b[0m\n\
              \x1b[31m  modified:   src/main.rs\x1b[0m\n\
              \x1b[31m  modified:   Cargo.toml\x1b[0m\n"
                .to_vec(),
        ),
        (
            "ls_color",
            b"\x1b[1;34mdir1\x1b[0m  \x1b[1;32mscript.sh\x1b[0m  \
              file.txt  \x1b[1;35marchive.zip\x1b[0m\n"
                .to_vec(),
        ),
        (
            "compiler_error",
            b"\x1b[1;31merror\x1b[0m\x1b[1m: cannot find value `foo` in this scope\x1b[0m\n\
              \x1b[1;34m  --> \x1b[0msrc/main.rs:10:5\n\
              \x1b[1;34m   |\x1b[0m\n\
              \x1b[1;34m10 |\x1b[0m     foo();\n\
              \x1b[1;34m   |     \x1b[0m\x1b[1;31m^^^ not found in this scope\x1b[0m\n"
                .to_vec(),
        ),
        (
            "progress_bar",
            b"\rDownloading [===========>           ] 55%"
                .to_vec(),
        ),
        (
            "vim_screen",
            b"\x1b[?1049h\x1b[H\x1b[2J\x1b[1;1H\
              \x1b[7m file.txt                              \x1b[0m\n\
              Hello World\n\
              \x1b[24;1H\x1b[7m INSERT \x1b[0m"
                .to_vec(),
        ),
    ];

    for (name, content) in test_cases {
        group.bench_function(name, |b| {
            b.iter(|| {
                let mut screen = TerminalScreen::new(80, 24);
                screen.process(black_box(&content));
            });
        });
    }

    group.finish();
}

/// Benchmark tab handling
fn bench_tab_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("tab_handling");

    let test_cases = vec![
        ("single_tab", b"\tText".to_vec()),
        ("multiple_tabs", b"\t\t\tText".to_vec()),
        ("mixed_tabs_spaces", b"  \t  \t  Text".to_vec()),
        ("tab_at_column_boundary", b"12345678\tNext".to_vec()),
    ];

    for (name, content) in test_cases {
        group.bench_function(name, |b| {
            b.iter(|| {
                let mut screen = TerminalScreen::new(80, 24);
                for _ in 0..100 {
                    screen.process(black_box(&content));
                    screen.process(b"\r\n");
                }
            });
        });
    }

    group.finish();
}

/// Benchmark character insertion and deletion
fn bench_char_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("char_operations");

    let test_sequences = vec![
        ("insert_char", b"\x1b[@".to_vec()),
        ("insert_5_chars", b"\x1b[5@".to_vec()),
        ("delete_char", b"\x1b[P".to_vec()),
        ("delete_5_chars", b"\x1b[5P".to_vec()),
        ("erase_char", b"\x1b[X".to_vec()),
        ("erase_5_chars", b"\x1b[5X".to_vec()),
    ];

    for (name, sequence) in test_sequences {
        group.bench_function(name, |b| {
            b.iter(|| {
                let mut screen = TerminalScreen::new(80, 24);
                // Fill with content first
                screen.process(b"This is a test line with content");
                // Perform operation
                for _ in 0..100 {
                    screen.process(black_box(&sequence));
                }
            });
        });
    }

    group.finish();
}

/// Benchmark device attributes response
fn bench_device_attributes(c: &mut Criterion) {
    c.bench_function("device_attributes_query", |b| {
        b.iter(|| {
            let mut screen = TerminalScreen::new(80, 24);
            // Primary DA
            screen.process(b"\x1b[c");
            let _ = screen.take_pending_responses();
            // Secondary DA
            screen.process(b"\x1b[>c");
            let _ = screen.take_pending_responses();
        });
    });
}

/// Benchmark hyperlink OSC sequences
fn bench_hyperlinks(c: &mut Criterion) {
    c.bench_function("hyperlink_parsing", |b| {
        b.iter(|| {
            let mut screen = TerminalScreen::new(80, 24);
            // Set hyperlink
            screen.process(b"\x1b]8;;https://example.com\x1b\\Link Text\x1b]8;;\x1b\\");
            // Multiple hyperlinks
            for i in 0..10 {
                let url = format!("\x1b]8;;https://example.com/{}\x1b\\Link{}\x1b]8;;\x1b\\", i, i);
                screen.process(url.as_bytes());
            }
        });
    });
}

criterion_group!(
    benches,
    bench_vte_plain_text,
    bench_cursor_movement,
    bench_erase_sequences,
    bench_sgr_sequences,
    bench_scrolling,
    bench_alternate_screen,
    bench_terminal_reset,
    bench_mixed_content,
    bench_tab_handling,
    bench_char_operations,
    bench_device_attributes,
    bench_hyperlinks
);
criterion_main!(benches);
