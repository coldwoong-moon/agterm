//! Memory usage benchmarks for AgTerm
//!
//! This benchmark suite measures:
//! - Memory efficiency of screen buffer
//! - Scrollback buffer memory usage
//! - Memory compression effectiveness
//! - String interning efficiency
//! - Large buffer handling

use agterm::terminal::screen::TerminalScreen;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

/// Benchmark memory usage for different screen sizes
fn bench_screen_size_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("screen_size_memory");

    let screen_sizes = vec![
        (80, 24),   // Standard terminal
        (132, 50),  // Large terminal
        (200, 60),  // Extra large
        (300, 100), // Very large
    ];

    for (cols, rows) in screen_sizes {
        let size_name = format!("{}x{}", cols, rows);
        group.bench_function(&size_name, |b| {
            b.iter(|| {
                let screen = TerminalScreen::new(cols, rows);
                let stats = screen.memory_stats();
                black_box(stats)
            });
        });
    }

    group.finish();
}

/// Benchmark scrollback buffer memory usage
fn bench_scrollback_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("scrollback_memory");

    for line_count in [100, 500, 1000, 5000, 10000].iter() {
        group.throughput(Throughput::Elements(*line_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(line_count),
            line_count,
            |b, &lines| {
                b.iter(|| {
                    let mut screen = TerminalScreen::new(80, 24);
                    let text = b"This is a test line with some content that simulates real terminal output.\r\n";

                    // Generate scrollback
                    for _ in 0..lines {
                        screen.process(text);
                    }

                    let stats = screen.memory_stats();
                    black_box(stats)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark compression effectiveness with different content types
fn bench_compression_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_effectiveness");

    let test_cases: Vec<(&str, &[u8], usize)> = vec![
        (
            "repeated_text",
            b"Same text repeated on every line\r\n" as &[u8],
            1000,
        ),
        (
            "varied_text",
            b"Different text with numbers: " as &[u8],
            1000,
        ),
        (
            "ansi_colored",
            b"\x1b[31mRed\x1b[32mGreen\x1b[33mYellow\x1b[0m\r\n" as &[u8],
            1000,
        ),
        (
            "mixed_width",
            "ASCII and 한글 mixed content\r\n".as_bytes(),
            1000,
        ),
        (
            "long_lines",
            b"Very long line with lots of content that exceeds typical terminal width and continues...\r\n" as &[u8],
            1000,
        ),
    ];

    for (name, base_text, lines) in test_cases {
        group.bench_function(name, |b| {
            b.iter(|| {
                let mut screen = TerminalScreen::new(80, 24);

                // Generate scrollback with different content patterns
                for i in 0..lines {
                    screen.process(base_text);
                    if name == "varied_text" {
                        let num_text = format!("{}\r\n", i);
                        screen.process(num_text.as_bytes());
                    }
                }

                let stats = screen.memory_stats();
                let compression_stats = screen.compression_stats().clone();
                black_box((stats, compression_stats))
            });
        });
    }

    group.finish();
}

/// Benchmark memory usage with string interning (hyperlinks, URLs)
fn bench_string_interning_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_interning_memory");

    // Test with repeated hyperlinks (should benefit from interning)
    group.bench_function("repeated_hyperlinks", |b| {
        b.iter(|| {
            let mut screen = TerminalScreen::new(80, 24);

            // Same URL repeated many times
            for _ in 0..100 {
                screen.process(
                    b"\x1b]8;;https://github.com/coldwoong-moon/agterm\x1b\\AgTerm\x1b]8;;\x1b\\ ",
                );
            }

            let stats = screen.memory_stats();
            black_box(stats)
        });
    });

    // Test with unique hyperlinks (less benefit from interning)
    group.bench_function("unique_hyperlinks", |b| {
        b.iter(|| {
            let mut screen = TerminalScreen::new(80, 24);

            // Different URLs each time
            for i in 0..100 {
                let url = format!(
                    "\x1b]8;;https://example.com/page{}\x1b\\Link\x1b]8;;\x1b\\ ",
                    i
                );
                screen.process(url.as_bytes());
            }

            let stats = screen.memory_stats();
            black_box(stats)
        });
    });

    group.finish();
}

/// Benchmark memory usage during window resizing
fn bench_resize_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("resize_memory");

    group.bench_function("resize_with_content", |b| {
        b.iter(|| {
            let mut screen = TerminalScreen::new(80, 24);

            // Fill with content
            for _ in 0..100 {
                screen.process(b"Line with content\r\n");
            }

            let stats_before = screen.memory_stats();

            // Resize larger
            screen.resize(120, 40);
            let stats_after_grow = screen.memory_stats();

            // Resize smaller
            screen.resize(60, 20);
            let stats_after_shrink = screen.memory_stats();

            black_box((stats_before, stats_after_grow, stats_after_shrink))
        });
    });

    group.finish();
}

/// Benchmark memory usage with alternate screen buffer
fn bench_alternate_screen_memory(c: &mut Criterion) {
    c.bench_function("alternate_screen_memory", |b| {
        b.iter(|| {
            let mut screen = TerminalScreen::new(80, 24);

            // Fill main screen
            for _ in 0..50 {
                screen.process(b"Main screen content\r\n");
            }

            let stats_main = screen.memory_stats();

            // Switch to alternate screen
            screen.process(b"\x1b[?1049h");

            // Fill alternate screen
            for _ in 0..30 {
                screen.process(b"Alternate screen content\r\n");
            }

            let stats_alternate = screen.memory_stats();

            // Switch back
            screen.process(b"\x1b[?1049l");

            let stats_restored = screen.memory_stats();

            black_box((stats_main, stats_alternate, stats_restored))
        });
    });
}

/// Benchmark memory usage with large amount of styled text
fn bench_styled_text_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("styled_text_memory");

    group.bench_function("plain_text", |b| {
        b.iter(|| {
            let mut screen = TerminalScreen::new(80, 24);

            for _ in 0..1000 {
                screen.process(b"Plain text without any styling\r\n");
            }

            let stats = screen.memory_stats();
            black_box(stats)
        });
    });

    group.bench_function("heavily_styled", |b| {
        b.iter(|| {
            let mut screen = TerminalScreen::new(80, 24);

            for _ in 0..1000 {
                screen.process(b"\x1b[1;4;38;2;255;100;50mHeavily styled text\x1b[0m\r\n");
            }

            let stats = screen.memory_stats();
            black_box(stats)
        });
    });

    group.bench_function("mixed_styles", |b| {
        b.iter(|| {
            let mut screen = TerminalScreen::new(80, 24);

            for i in 0..1000 {
                if i % 4 == 0 {
                    screen.process(b"Plain text\r\n");
                } else if i % 4 == 1 {
                    screen.process(b"\x1b[1mBold\x1b[0m\r\n");
                } else if i % 4 == 2 {
                    screen.process(b"\x1b[31mColored\x1b[0m\r\n");
                } else {
                    screen.process(b"\x1b[1;4;32mMultiple\x1b[0m\r\n");
                }
            }

            let stats = screen.memory_stats();
            black_box(stats)
        });
    });

    group.finish();
}

/// Benchmark memory efficiency with CJK characters
fn bench_cjk_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("cjk_memory");

    let test_cases = vec![
        ("ascii_only", b"Hello World Test\r\n".to_vec(), 1000),
        (
            "korean_text",
            "안녕하세요 테스트\r\n".as_bytes().to_vec(),
            1000,
        ),
        (
            "japanese_text",
            "こんにちはテスト\r\n".as_bytes().to_vec(),
            1000,
        ),
        ("chinese_text", "你好世界测试\r\n".as_bytes().to_vec(), 1000),
        (
            "mixed_ascii_cjk",
            "Hello 안녕 World 世界\r\n".as_bytes().to_vec(),
            1000,
        ),
    ];

    for (name, text, lines) in test_cases {
        group.bench_function(name, |b| {
            b.iter(|| {
                let mut screen = TerminalScreen::new(80, 24);

                for _ in 0..lines {
                    screen.process(&text);
                }

                let stats = screen.memory_stats();
                black_box(stats)
            });
        });
    }

    group.finish();
}

/// Benchmark memory peak during continuous output
fn bench_continuous_output_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("continuous_output_memory");

    // Simulate continuous log output
    group.bench_function("log_stream", |b| {
        b.iter(|| {
            let mut screen = TerminalScreen::new(80, 24);

            // Simulate log streaming with timestamps
            for i in 0..10000 {
                let log_line = format!("[2025-01-18 12:34:56] INFO: Processing item {}\r\n", i);
                screen.process(log_line.as_bytes());

                // Check memory every 1000 lines
                if i % 1000 == 0 {
                    black_box(screen.memory_stats());
                }
            }

            let final_stats = screen.memory_stats();
            black_box(final_stats)
        });
    });

    // Simulate compilation output
    group.bench_function("compiler_output", |b| {
        b.iter(|| {
            let mut screen = TerminalScreen::new(80, 24);

            for i in 0..5000 {
                // Mix of plain text and ANSI colored warnings/errors
                if i % 10 == 0 {
                    screen.process(b"\x1b[1;31merror\x1b[0m: compilation failed\r\n");
                } else if i % 5 == 0 {
                    screen.process(b"\x1b[1;33mwarning\x1b[0m: unused variable\r\n");
                } else {
                    screen.process(b"   Compiling package...\r\n");
                }
            }

            let stats = screen.memory_stats();
            black_box(stats)
        });
    });

    group.finish();
}

/// Benchmark memory cleanup and garbage collection
fn bench_memory_cleanup(c: &mut Criterion) {
    c.bench_function("clear_screen_memory", |b| {
        b.iter(|| {
            let mut screen = TerminalScreen::new(80, 24);

            // Fill with lots of content
            for _ in 0..1000 {
                screen.process(b"\x1b[1;31mStyled text with colors\x1b[0m\r\n");
            }

            let stats_before = screen.memory_stats();

            // Clear screen (should free memory)
            screen.process(b"\x1b[2J\x1b[H");

            let stats_after = screen.memory_stats();

            black_box((stats_before, stats_after))
        });
    });

    c.bench_function("clear_scrollback_memory", |b| {
        b.iter(|| {
            let mut screen = TerminalScreen::new(80, 24);

            // Generate large scrollback
            for _ in 0..5000 {
                screen.process(b"Scrollback line\r\n");
            }

            let stats_before = screen.memory_stats();

            // Clear scrollback (CSI 3 J)
            screen.process(b"\x1b[3J");

            let stats_after = screen.memory_stats();

            black_box((stats_before, stats_after))
        });
    });
}

criterion_group!(
    benches,
    bench_screen_size_memory,
    bench_scrollback_memory,
    bench_compression_effectiveness,
    bench_string_interning_memory,
    bench_resize_memory,
    bench_alternate_screen_memory,
    bench_styled_text_memory,
    bench_cjk_memory,
    bench_continuous_output_memory,
    bench_memory_cleanup
);
criterion_main!(benches);
