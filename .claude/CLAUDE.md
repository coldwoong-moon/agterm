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

| Variable | Description | Example |
|----------|-------------|---------|
| `AGTERM_DEBUG` | Enable debug panel on start | `1` |
| `AGTERM_LOG` | Set log filter | `agterm=debug,agterm::terminal::pty=trace` |

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

## Log Files

Logs are stored in: `~/.local/share/agterm/logs/agterm.log.*`

Daily rotation is enabled by default.
