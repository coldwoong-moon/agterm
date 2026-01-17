# /agterm-inspect - Runtime State Inspection

Inspect AgTerm's internal state and configuration.

## Activation

This skill is invoked when user types `/agterm-inspect` or asks to inspect app state.

## Arguments

- `/agterm-inspect` - Overview of current state
- `/agterm-inspect config` - Show configuration
- `/agterm-inspect pty` - PTY session details
- `/agterm-inspect env` - Environment variables
- `/agterm-inspect code <module>` - Inspect source code

## Instructions

### Configuration Inspection

```bash
# Show default config
cat default_config.toml

# Show user config (if exists)
cat ~/.config/agterm/config.toml 2>/dev/null || echo "No user config"

# Show project-local config
cat .agterm/config.toml 2>/dev/null || echo "No local config"
```

### Environment Check

```bash
echo "=== AgTerm Environment ==="
echo "SHELL: $SHELL"
echo "TERM: $TERM"
echo "AGTERM_DEBUG: ${AGTERM_DEBUG:-not set}"
echo "AGTERM_LOG: ${AGTERM_LOG:-not set}"
echo ""
echo "=== System Info ==="
echo "OS: $(uname -s)"
echo "Arch: $(uname -m)"
echo "User: $USER"
echo "Home: $HOME"
```

### PTY State Analysis

Check current PTY sessions from logs:
```bash
LOG_DIR="$HOME/.local/share/agterm/logs"
echo "=== Recent PTY Activity ==="
grep -i "pty\|session" "$LOG_DIR"/agterm.log.* 2>/dev/null | tail -20
```

### Source Code Inspection

Use Read tool to inspect relevant modules:

| Module | Path | Purpose |
|--------|------|---------|
| Main | `src/main.rs` | App entry, UI, state |
| PTY | `src/terminal/pty.rs` | PTY management |
| Logging | `src/logging/mod.rs` | Logging config |
| Debug | `src/debug/panel.rs` | Debug UI |

### Dependency Check

```bash
echo "=== Cargo Dependencies ==="
cargo tree --depth 1 2>/dev/null || cat Cargo.toml
```

### Build Information

```bash
echo "=== Build Info ==="
cargo --version
rustc --version
echo ""
echo "=== Project Metadata ==="
grep -E "^name|^version|^edition" Cargo.toml
```

## State Diagram

```
AgTerm State Flow:
┌────────────────────────────────────────────────┐
│                    AgTerm                       │
│  ┌────────┐  ┌────────┐  ┌────────────────┐   │
│  │ Tabs[] │──│ PTY    │──│ Debug Panel    │   │
│  │        │  │ Manager│  │ - Metrics      │   │
│  │ [Tab0] │  │        │  │ - Logs         │   │
│  │ [Tab1] │  │ [Sess] │  │ - PTY Stats    │   │
│  └────────┘  └────────┘  └────────────────┘   │
│       │           │              │             │
│       ▼           ▼              ▼             │
│   [Input]     [Output]      [Tracing]         │
└────────────────────────────────────────────────┘
```

## Quick Health Check

```bash
echo "=== AgTerm Health Check ==="

# 1. Source exists
[ -f "src/main.rs" ] && echo "✓ Source files present" || echo "✗ Missing source"

# 2. Build works
cargo check 2>/dev/null && echo "✓ Build check passed" || echo "✗ Build errors"

# 3. Tests pass
cargo test --quiet 2>/dev/null && echo "✓ Tests passing" || echo "✗ Test failures"

# 4. Log dir accessible
LOG_DIR="$HOME/.local/share/agterm/logs"
[ -d "$LOG_DIR" ] || mkdir -p "$LOG_DIR"
[ -w "$LOG_DIR" ] && echo "✓ Log directory writable" || echo "✗ Log directory not writable"

echo ""
echo "Health check complete"
```
