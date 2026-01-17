# AgTerm Benchmarks

Performance benchmarks for AgTerm terminal emulator using Criterion.rs.

## Quick Start

```bash
# Run all benchmarks
cargo bench

# Run specific suite
cargo bench --bench rendering_benchmark
cargo bench --bench terminal_benchmark

# Test benchmarks (fast mode)
cargo bench -- --test
```

## Benchmark Suites

### `rendering_benchmark.rs`
Measures rendering and display performance:
- Large text output speed
- Span merging efficiency
- ANSI color parsing
- Scrollback buffer management
- Full screen refresh
- Wide character (CJK) handling

### `terminal_benchmark.rs`
Measures terminal emulation performance:
- VTE parsing speed
- Cursor movement operations
- Screen erase sequences
- SGR (text styling) processing
- Scrolling operations
- Alternate screen buffer
- Character operations
- Device attributes
- Hyperlink parsing

## Documentation

See [docs/BENCHMARKS.md](../docs/BENCHMARKS.md) for detailed documentation.

## Results

Criterion generates HTML reports in `target/criterion/` with:
- Performance graphs
- Statistical analysis
- Change detection
- Regression warnings

## Performance Targets

### Critical Benchmarks
- **VTE parsing**: < 1µs per character
- **Span merging**: O(n) linear scaling
- **Scrollback**: < 10ms for 10k lines

### User Experience
- **Full screen refresh**: < 50ms for 100KB
- **Wide characters**: < 2x ASCII time
- **Color parsing**: < 100ns per sequence

## Adding Benchmarks

1. Add function to appropriate file
2. Use `criterion::black_box()` to prevent optimization
3. Set appropriate `Throughput` for scaling metrics
4. Add to `criterion_group!` macro
5. Document in `docs/BENCHMARKS.md`

## CI Integration

```bash
# Quick mode for CI
cargo bench -- --quick

# Save baseline
cargo bench -- --save-baseline main

# Compare against baseline
cargo bench -- --baseline main
```
