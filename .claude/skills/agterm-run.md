# /agterm-run - Run AgTerm Application

Build and run the AgTerm terminal application.

## Activation

This skill is invoked when user types `/agterm-run` or asks to run/start AgTerm.

## Arguments

- `/agterm-run` - Build and run in release mode
- `/agterm-run dev` - Build and run in development mode
- `/agterm-run debug` - Run with debug panel enabled
- `/agterm-run trace` - Run with trace-level logging

## Instructions

### Release Mode (Optimized)

```bash
cargo run --release 2>&1
```

### Development Mode (Fast Build)

```bash
cargo run 2>&1
```

### With Debug Panel

```bash
AGTERM_DEBUG=1 cargo run 2>&1
```

### With Trace Logging

```bash
AGTERM_LOG="agterm=trace" cargo run 2>&1
```

### With Custom Log Filter

```bash
# Debug PTY operations only
AGTERM_LOG="agterm::terminal::pty=debug" cargo run 2>&1

# Debug multiple modules
AGTERM_LOG="agterm=info,agterm::terminal=debug,agterm::debug=trace" cargo run 2>&1
```

## Keyboard Shortcuts

After launching, remind user of key shortcuts:

| Shortcut | Action |
|----------|--------|
| `Cmd+T` | New tab |
| `Cmd+W` | Close tab |
| `Cmd+]` / `Cmd+[` | Next/Prev tab |
| `Cmd+1-5` | Switch to tab N |
| `Cmd+M` | Toggle Raw/Block mode |
| `Cmd+D` / `F12` | Toggle debug panel |
| `Ctrl+C` | Send interrupt |
| `Ctrl+D` | Send EOF |
| `Ctrl+Z` | Suspend |

## Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `AGTERM_DEBUG` | Enable debug panel | `1` |
| `AGTERM_LOG` | Log filter | `agterm=debug` |
| `SHELL` | Override shell | `/bin/zsh` |

## Troubleshooting

If app fails to start:

1. **Check build:**
   ```bash
   cargo build 2>&1
   ```

2. **Check dependencies:**
   ```bash
   cargo check 2>&1
   ```

3. **Verify PTY support:**
   ```bash
   ls -la /dev/ptmx
   echo $SHELL
   ```

4. **Check logs:**
   ```bash
   cat ~/.local/share/agterm/logs/agterm.log.* 2>/dev/null | tail -20
   ```
