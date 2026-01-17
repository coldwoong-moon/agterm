# AgTerm Performance Benchmarks

This document describes the performance benchmarks available in AgTerm and how to run them.

## Overview

AgTerm includes comprehensive performance benchmarks using the [Criterion.rs](https://github.com/bheisler/criterion.rs) framework. The benchmarks are organized into two main suites:

1. **Rendering Benchmarks** - Measures text rendering and display performance
2. **Terminal Benchmarks** - Measures terminal emulation and sequence processing

## Running Benchmarks

### Run All Benchmarks

```bash
cargo bench
```

### Run Specific Benchmark Suite

```bash
# Rendering benchmarks only
cargo bench --bench rendering_benchmark

# Terminal processing benchmarks only
cargo bench --bench terminal_benchmark
```

### Run Specific Benchmark

```bash
# Run only large text output benchmark
cargo bench -- large_text_output

# Run only VTE parsing benchmark
cargo bench -- vte_plain_text
```

### Quick Test Run

To quickly verify benchmarks work without full statistical analysis:

```bash
cargo bench -- --test
```

## Benchmark Suites

### Rendering Benchmarks (`benches/rendering_benchmark.rs`)

#### 1. Large Text Output
Measures rendering performance for different amounts of text output.

**Sizes tested:** 100, 500, 1000, 5000 lines

```bash
cargo bench -- large_text_output
```

#### 2. Span Merging
Tests the efficiency of merging consecutive styled text spans.

**Cell counts tested:** 100, 500, 1000, 5000 cells

```bash
cargo bench -- span_merging
```

#### 3. ANSI Color Parsing
Benchmarks parsing and applying various ANSI color sequences:
- Basic 16-color palette
- 256-color palette
- RGB true color
- Mixed attributes (bold, underline, etc.)

```bash
cargo bench -- ansi_color_parsing
```

#### 4. Scrollback Buffer Management
Tests scrollback buffer performance with different history sizes.

**Line counts tested:** 100, 500, 1000, 5000, 10000 lines

```bash
cargo bench -- scrollback_buffer
```

#### 5. Full Screen Refresh
Simulates rendering large file outputs (like `cat large_file.txt`).

**File sizes tested:** 1KB, 10KB, 100KB, 500KB

```bash
cargo bench -- full_screen_refresh
```

#### 6. Wide Character Handling
Tests rendering performance with CJK (Chinese, Japanese, Korean) characters:
- ASCII only
- Mixed ASCII/Korean
- Korean only
- Japanese Kanji
- Chinese Simplified

```bash
cargo bench -- wide_characters
```

### Terminal Benchmarks (`benches/terminal_benchmark.rs`)

#### 1. VTE Plain Text
Measures VTE parser performance with plain text input.

**Sizes tested:** 100, 500, 1000, 5000 characters

```bash
cargo bench -- vte_plain_text
```

#### 2. Cursor Movement
Tests cursor positioning and movement sequences:
- Arrow keys (up, down, left, right)
- Absolute positioning
- Home
- Complex movements

```bash
cargo bench -- cursor_movement
```

#### 3. Erase Sequences
Benchmarks screen/line clearing operations:
- Erase entire display
- Erase below/above cursor
- Erase entire line
- Erase left/right of cursor

```bash
cargo bench -- erase_sequences
```

#### 4. SGR (Select Graphic Rendition)
Tests text styling sequence processing:
- Bold, dim, italic, underline
- Foreground/background colors (basic, 256-color, RGB)
- Combined attributes

```bash
cargo bench -- sgr_sequences
```

#### 5. Scrolling Operations
Measures scroll region management:
- Scroll up/down
- Insert/delete lines
- Various scroll amounts

```bash
cargo bench -- scrolling
```

#### 6. Alternate Screen Buffer
Tests alternate screen switching (used by vim, less, etc.).

```bash
cargo bench -- alternate_screen
```

#### 7. Terminal Reset
Benchmarks full terminal state reset.

```bash
cargo bench -- terminal_reset
```

#### 8. Mixed Content
Realistic terminal output simulations:
- Git status output
- Colored ls output
- Compiler errors
- Progress bars
- Vim screen

```bash
cargo bench -- mixed_content
```

#### 9. Tab Handling
Tests tab character processing and expansion.

```bash
cargo bench -- tab_handling
```

#### 10. Character Operations
Benchmarks character insertion, deletion, and erasing.

```bash
cargo bench -- char_operations
```

#### 11. Device Attributes
Tests device attribute query/response handling.

```bash
cargo bench -- device_attributes
```

#### 12. Hyperlinks
Measures OSC 8 hyperlink sequence parsing.

```bash
cargo bench -- hyperlinks
```

## Understanding Results

Criterion generates detailed HTML reports in `target/criterion/`.

### Key Metrics

- **time:** Average time per iteration
- **throughput:** Operations/bytes processed per second
- **R² goodness of fit:** Confidence in measurements (closer to 1.0 is better)

### Interpreting Output

```
large_text_output/100   time:   [123.45 µs 125.67 µs 128.90 µs]
                        thrpt:  [776.12 elem/s 795.86 elem/s 810.23 elem/s]
```

This shows:
- Average time: 125.67 µs
- Confidence interval: 123.45 - 128.90 µs
- Throughput: ~795 elements per second

### Change Detection

Criterion automatically detects performance changes:
- **Improved**: Performance got better
- **Regressed**: Performance got worse
- **No change**: Within noise threshold

## Continuous Benchmarking

### Baseline Comparison

Save current performance as baseline:

```bash
cargo bench -- --save-baseline main
```

Compare against baseline:

```bash
# After making changes
cargo bench -- --baseline main
```

### CI Integration

For CI/CD, use quick mode to avoid long runs:

```bash
cargo bench -- --quick
```

## Performance Tips

### What to Look For

1. **Rendering Performance**
   - Span merging should scale linearly
   - Large text output should be sub-linear (caching helps)
   - Wide character handling should be close to ASCII speed

2. **Terminal Processing**
   - VTE parsing should be very fast (< 1µs per character)
   - Cursor movements should be constant time
   - Screen operations should scale with affected area

### Optimization Targets

- **Critical Path**: VTE parsing, span merging
- **High Volume**: Plain text rendering, scrollback buffer
- **User Experience**: Full screen refresh, wide characters

## Adding New Benchmarks

To add a new benchmark:

1. Add benchmark function to appropriate file
2. Use `criterion` API:
   ```rust
   fn bench_new_feature(c: &mut Criterion) {
       c.bench_function("feature_name", |b| {
           b.iter(|| {
               // Code to benchmark
               black_box(expensive_operation())
           });
       });
   }
   ```
3. Add to `criterion_group!` macro
4. Document in this file

## Resources

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [AgTerm Architecture](../PLAN.md)

## Troubleshooting

### Benchmarks Won't Run

```bash
# Clean and rebuild
cargo clean
cargo bench --no-run
cargo bench
```

### Inconsistent Results

- Close other applications
- Disable CPU frequency scaling
- Use `--sample-size` for more samples:
  ```bash
  cargo bench -- --sample-size 200
  ```

### Out of Memory

Reduce benchmark parameters or run specific benchmarks:

```bash
# Instead of full suite
cargo bench -- vte_plain_text/100
```
