# AgTerm - AI Agent Terminal

Native GPU-accelerated terminal emulator with AI agent orchestration.

## Project Overview

AgTerm is a Rust-based terminal emulator using the Iced GUI framework, featuring:
- Multi-tab terminal sessions with PTY support
- Warp-inspired block-based UI (Block mode) and traditional Raw mode
- ANSI color parsing and Korean/CJK IME support
- Real-time debug panel with performance metrics
- Structured logging with tracing ecosystem

## Architecture

```
src/
├── main.rs           # Iced app, UI, message handling
├── terminal/
│   ├── mod.rs
│   └── pty.rs        # PTY session management (threaded)
├── logging/
│   ├── mod.rs        # Logging initialization
│   └── layers.rs     # Custom tracing layers
└── debug/
    ├── mod.rs        # Metrics, debug state
    └── panel.rs      # Debug UI panel
```

## Key Components

| Component | Location | Description |
|-----------|----------|-------------|
| `AgTerm` | `src/main.rs:327` | Main app state |
| `PtyManager` | `src/terminal/pty.rs:285` | Thread-safe PTY controller |
| `DebugPanel` | `src/debug/panel.rs:49` | Debug UI component |
| `LoggingConfig` | `src/logging/mod.rs:19` | Logging configuration |

## Available Skills

| Skill | Description |
|-------|-------------|
| `/agterm-run` | Build and run the application |
| `/agterm-debug` | Launch in debug mode with enhanced logging |
| `/agterm-test` | Run test suite |
| `/agterm-logs` | View and analyze application logs |
| `/agterm-profile` | Performance profiling tools |
| `/agterm-inspect` | Inspect runtime state and configuration |

## Development Commands

```bash
# Build
cargo build              # Debug build
cargo build --release    # Release build

# Run
cargo run                # Normal run
AGTERM_DEBUG=1 cargo run # With debug panel

# Test
cargo test              # All tests
cargo test pty::        # PTY tests only

# Check
cargo check             # Type check
cargo clippy            # Lints
```

## Environment Variables

### Application Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `AGTERM_DEBUG` | Enable debug panel on start | `1` |
| `AGTERM_LOG` | Advanced log filter (module-specific) | `agterm=debug,agterm::terminal::pty=trace` |
| `AGTERM_LOG_LEVEL` | Simple log level override | `debug` (trace/debug/info/warn/error) |
| `AGTERM_LOG_PATH` | Custom log directory path | `/custom/logs` |

### PTY Session Environment Variables

AgTerm automatically configures environment variables for shell sessions:

| Variable | Default Value | Purpose |
|----------|---------------|---------|
| `TERM` | `xterm-256color` | 256-color terminal support |
| `COLORTERM` | `truecolor` | 24-bit true color support |
| `TERM_PROGRAM` | `agterm` | Terminal emulator identification |
| `AGTERM_VERSION` | (version) | AgTerm version |
| `SHELL` | (auto-detected) | Shell path |
| `LANG` | `en_US.UTF-8` | UTF-8 locale |

Critical variables (`HOME`, `USER`, `PATH`) are always inherited. See [ENVIRONMENT_VARIABLES.md](../ENVIRONMENT_VARIABLES.md) for details.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+T` | New tab |
| `Cmd+W` | Close tab |
| `Cmd+M` | Toggle Raw/Block mode |
| `Cmd+D` / `F12` | Toggle debug panel |
| `Ctrl+C` | Interrupt |
| `Ctrl+D` | EOF |

## Code Conventions

- Use `tracing` macros for logging (`info!`, `debug!`, `trace!`)
- Add `#[instrument]` to public functions for tracing
- Follow Rust naming conventions
- Keep UI code in `main.rs`, business logic in modules

## Testing

All tests should pass before committing:
```bash
cargo test
# Expected: 49+ tests passing
```

## Logging System

AgTerm uses the tracing ecosystem for comprehensive structured logging.

### Log Locations

**Default paths:**
- macOS/Linux: `~/.local/share/agterm/logs/agterm.log.*`
- Windows: `%APPDATA%\agterm\logs\agterm.log.*`

**Features:**
- Daily log rotation (automatic cleanup)
- Console output during development
- In-memory buffer for debug panel
- Module-level filtering support

### Log Levels

| Level | Purpose | Use Case |
|-------|---------|----------|
| `trace` | Very detailed | Step-by-step debugging, PTY I/O |
| `debug` | Detailed | Development, troubleshooting |
| `info` | Informational | Normal operations, state changes |
| `warn` | Warnings | Non-critical issues, fallbacks |
| `error` | Errors | Failures, exceptions |

### Configuration

**In config file** (`~/.config/agterm/config.toml`):
```toml
[logging]
level = "info"
format = "pretty"
timestamps = true
file_line = false
file_output = true
file_path = "/custom/path"  # Optional
```

**Via environment variables:**
```bash
# Simple level override
AGTERM_LOG_LEVEL=debug cargo run

# Module-specific filtering
AGTERM_LOG=agterm=debug,agterm::terminal::pty=trace cargo run

# Custom log path
AGTERM_LOG_PATH=/tmp/agterm-logs cargo run
```

### What Gets Logged

**Application Lifecycle:**
- Startup and shutdown events
- Configuration loading
- Settings changes

**Terminal Operations:**
- Tab creation/closing
- Pane splits
- PTY session lifecycle
- Shell spawning

**PTY Events:**
- Session creation with retry attempts
- Read/write operations
- Session cleanup
- Errors and warnings

**Debug Information:**
- Performance metrics
- Memory usage
- Thread activity
- Adaptive polling behavior
