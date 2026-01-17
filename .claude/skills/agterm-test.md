# /agterm-test - AgTerm Test Runner

Run AgTerm tests with various options.

## Activation

This skill is invoked when user types `/agterm-test` or asks to run tests.

## Arguments

- `/agterm-test` - Run all tests
- `/agterm-test unit` - Run unit tests only
- `/agterm-test pty` - Run PTY-related tests
- `/agterm-test logging` - Run logging module tests
- `/agterm-test debug` - Run debug module tests
- `/agterm-test coverage` - Run tests with coverage (if tarpaulin installed)

## Instructions

### Run All Tests

```bash
cargo test 2>&1
```

### Run Specific Test Modules

```bash
# PTY tests
cargo test pty:: 2>&1

# Logging tests
cargo test logging:: 2>&1

# Debug panel tests
cargo test debug:: 2>&1

# Main app tests
cargo test tests:: 2>&1
```

### Run Single Test

```bash
# By name pattern
cargo test test_name_pattern 2>&1

# With output
cargo test test_name -- --nocapture 2>&1
```

### Run Tests with Verbose Output

```bash
cargo test -- --nocapture 2>&1
```

### Run Tests with Coverage

```bash
# Using cargo-tarpaulin (install: cargo install cargo-tarpaulin)
cargo tarpaulin --out Html 2>&1
```

## Test Categories

| Category | Pattern | Description |
|----------|---------|-------------|
| Unit | `test_*` | Pure unit tests |
| PTY | `test_pty_*` | PTY session tests |
| Logging | `logging::*` | Logging system tests |
| Debug | `debug::*` | Debug panel tests |
| Integration | `test_*_integration` | Full integration tests |

## Expected Test Output

All 49+ tests should pass:

```
running 49 tests
test debug::panel::tests::test_debug_panel_toggle ... ok
test debug::tests::test_metrics_fps ... ok
test logging::tests::test_default_config ... ok
...
test result: ok. 49 passed; 0 failed; 0 ignored
```

## Troubleshooting Failed Tests

If tests fail:

1. **Check build first:**
   ```bash
   cargo build 2>&1
   ```

2. **Run failing test with details:**
   ```bash
   cargo test failing_test_name -- --nocapture 2>&1
   ```

3. **Check for PTY permission issues (Unix only):**
   ```bash
   ls -la /dev/ptmx
   ```
