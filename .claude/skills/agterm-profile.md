# /agterm-profile - Performance Profiling

Profile AgTerm performance and identify bottlenecks.

## Activation

This skill is invoked when user types `/agterm-profile` or asks about performance analysis.

## Arguments

- `/agterm-profile` - Show current metrics (from debug panel)
- `/agterm-profile cpu` - Profile CPU usage
- `/agterm-profile memory` - Profile memory usage
- `/agterm-profile flamegraph` - Generate flamegraph

## Instructions

### Using Debug Panel Metrics

1. Launch with debug enabled:
   ```bash
   AGTERM_DEBUG=1 cargo run 2>&1 &
   ```

2. Open debug panel (`Cmd+D` or `F12`)

3. Monitor metrics:
   - **FPS**: Should be ~60 for smooth UI
   - **Frame time**: Should be <16.7ms for 60fps
   - **Message time**: Processing time per event
   - **PTY bytes/sec**: I/O throughput

### CPU Profiling with Instruments (macOS)

```bash
# Build release with debug symbols
cargo build --release

# Run with Instruments
xcrun xctrace record --template 'Time Profiler' --launch -- ./target/release/agterm
```

### Memory Profiling

```bash
# Using heaptrack (install: brew install heaptrack)
heaptrack ./target/release/agterm

# Analyze results
heaptrack_gui heaptrack.agterm.*.gz
```

### Flamegraph Generation

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph (requires dtrace permissions on macOS)
sudo cargo flamegraph --bin agterm
```

### Built-in Tracing Analysis

```bash
# Enable detailed tracing
AGTERM_LOG="agterm=trace" cargo run 2>&1 | tee trace.log

# Analyze trace output
grep "duration\|elapsed\|took" trace.log
```

## Performance Targets

| Metric | Target | Warning | Critical |
|--------|--------|---------|----------|
| FPS | 60 | <55 | <30 |
| Frame time | <16ms | >20ms | >50ms |
| PTY latency | <10ms | >50ms | >100ms |
| Memory | <100MB | >200MB | >500MB |

## Common Performance Issues

1. **Low FPS:**
   - Check PTY read frequency
   - Look for excessive re-renders
   - Monitor ANSI parsing complexity

2. **High memory:**
   - Check output buffer sizes
   - Monitor block history length
   - Look for memory leaks in log buffer

3. **Input lag:**
   - Check message queue depth
   - Monitor IME processing time
   - Verify PTY write latency

## Optimization Tips

```rust
// In pty.rs - batch reads
const BATCH_SIZE: usize = 4096;

// In main.rs - limit re-renders
const TICK_INTERVAL: Duration = Duration::from_millis(100);

// In debug/panel.rs - limit log buffer
const MAX_LOG_ENTRIES: usize = 50;
```
