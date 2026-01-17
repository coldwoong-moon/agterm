//! Rendering performance benchmarks for AgTerm
//!
//! This benchmark suite measures:
//! - Large text output rendering speed
//! - Span merging efficiency
//! - ANSI color parsing and rendering
//! - Scrollback buffer rendering

use agterm::terminal::screen::{Cell, TerminalScreen};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

/// Create a terminal screen with sample content
fn create_test_screen(rows: usize, cols: usize) -> TerminalScreen {
    let mut screen = TerminalScreen::new(cols, rows);

    // Fill with sample text
    let sample_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ";
    let mut text_bytes = sample_text.as_bytes().to_vec();
    text_bytes.extend(b"\r\n");

    for _ in 0..rows {
        screen.process(&text_bytes);
    }

    screen
}

/// Create cells with various ANSI colors
fn create_colored_cells(count: usize) -> Vec<Cell> {
    let colors: Vec<&[u8]> = vec![
        b"\x1b[31m", // Red
        b"\x1b[32m", // Green
        b"\x1b[33m", // Yellow
        b"\x1b[34m", // Blue
        b"\x1b[35m", // Magenta
        b"\x1b[36m", // Cyan
        b"\x1b[0m",  // Reset
    ];

    let mut screen = TerminalScreen::new(80, 24);

    for i in 0..count {
        let color = colors[i % colors.len()];
        screen.process(color);
        screen.process(b"X");
    }

    let lines = screen.get_all_lines();
    if !lines.is_empty() {
        lines[0].clone()
    } else {
        Vec::new()
    }
}

/// Benchmark large text output rendering
fn bench_large_text_output(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_text_output");

    for size in [100, 500, 1000, 5000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let screen = create_test_screen(size, 80);
                black_box(screen.get_all_lines())
            });
        });
    }

    group.finish();
}

/// Benchmark span merging efficiency
fn bench_span_merging(c: &mut Criterion) {
    let mut group = c.benchmark_group("span_merging");

    for cell_count in [100, 500, 1000, 5000].iter() {
        group.throughput(Throughput::Elements(*cell_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(cell_count),
            cell_count,
            |b, &count| {
                let cells = create_colored_cells(count);
                b.iter(|| {
                    // Convert cells to styled spans (simulating rendering)
                    black_box(cells_to_styled_spans(&cells))
                });
            },
        );
    }

    group.finish();
}

/// Helper function to convert cells to styled spans (simplified version)
fn cells_to_styled_spans(cells: &[Cell]) -> Vec<(String, Option<(f32, f32, f32)>)> {
    let mut spans = Vec::new();
    let mut current_text = String::new();
    let mut current_color: Option<(f32, f32, f32)> = None;

    for cell in cells {
        if cell.placeholder {
            continue;
        }

        let color = if let Some(fg) = &cell.fg {
            let c = fg.to_color();
            Some((c.r, c.g, c.b))
        } else {
            None
        };

        // If color changes, push current span and start new one
        if color != current_color {
            if !current_text.is_empty() {
                spans.push((std::mem::take(&mut current_text), current_color));
            }
            current_color = color;
        }

        current_text.push(cell.c);
    }

    // Push final span
    if !current_text.is_empty() {
        spans.push((current_text, current_color));
    }

    spans
}

/// Benchmark ANSI color parsing
fn bench_ansi_color_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("ansi_color_parsing");

    let test_sequences = vec![
        (
            "basic_colors",
            b"\x1b[31mRed\x1b[32mGreen\x1b[33mYellow\x1b[0m".to_vec(),
        ),
        (
            "256_colors",
            b"\x1b[38;5;196mBright Red\x1b[38;5;46mBright Green\x1b[0m".to_vec(),
        ),
        (
            "rgb_colors",
            b"\x1b[38;2;255;0;0mRGB Red\x1b[38;2;0;255;0mRGB Green\x1b[0m".to_vec(),
        ),
        (
            "mixed_attributes",
            b"\x1b[1;4;31mBold Underline Red\x1b[0m\x1b[2;32mDim Green\x1b[0m".to_vec(),
        ),
    ];

    for (name, sequence) in test_sequences {
        group.bench_function(name, |b| {
            b.iter(|| {
                let mut screen = TerminalScreen::new(80, 24);
                screen.process(black_box(&sequence));
                black_box(screen.get_all_lines())
            });
        });
    }

    group.finish();
}

/// Benchmark scrollback buffer management
fn bench_scrollback_buffer(c: &mut Criterion) {
    let mut group = c.benchmark_group("scrollback_buffer");

    for lines in [100, 500, 1000, 5000, 10000].iter() {
        group.throughput(Throughput::Elements(*lines as u64));
        group.bench_with_input(BenchmarkId::from_parameter(lines), lines, |b, &lines| {
            b.iter(|| {
                let mut screen = TerminalScreen::new(80, 24);
                let text = b"Test line with some content\r\n";

                for _ in 0..lines {
                    screen.process(text);
                }

                black_box(screen.scrollback_size())
            });
        });
    }

    group.finish();
}

/// Benchmark full screen refresh (like cat-ing a large file)
fn bench_full_screen_refresh(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_screen_refresh");

    // Simulate different file sizes
    for kb in [1, 10, 100, 500].iter() {
        let bytes = *kb * 1024;
        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(BenchmarkId::from_parameter(kb), kb, |b, &kb| {
            // Create test data
            let line = b"This is a test line with some content to simulate real output.\n";
            let mut data = Vec::new();
            let target_size = kb * 1024;
            while data.len() < target_size {
                data.extend_from_slice(line);
            }
            data.truncate(target_size);

            b.iter(|| {
                let mut screen = TerminalScreen::new(80, 24);
                screen.process(black_box(&data));
                black_box(screen.get_all_lines())
            });
        });
    }

    group.finish();
}

/// Benchmark wide character (CJK) handling
fn bench_wide_characters(c: &mut Criterion) {
    let mut group = c.benchmark_group("wide_characters");

    let test_cases = vec![
        ("ascii_only", b"Hello World! This is a test.\n".to_vec()),
        (
            "mixed_ascii_korean",
            "Hello 안녕하세요 World 세계!\n".as_bytes().to_vec(),
        ),
        (
            "korean_only",
            "안녕하세요 여러분 반갑습니다!\n".as_bytes().to_vec(),
        ),
        (
            "japanese_kanji",
            "こんにちは世界！日本語テスト。\n".as_bytes().to_vec(),
        ),
        (
            "chinese_simplified",
            "你好世界！这是一个测试。\n".as_bytes().to_vec(),
        ),
    ];

    for (name, text) in test_cases {
        group.bench_function(name, |b| {
            b.iter(|| {
                let mut screen = TerminalScreen::new(80, 24);
                screen.process(black_box(&text));
                black_box(screen.get_all_lines())
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_large_text_output,
    bench_span_merging,
    bench_ansi_color_parsing,
    bench_scrollback_buffer,
    bench_full_screen_refresh,
    bench_wide_characters
);
criterion_main!(benches);
