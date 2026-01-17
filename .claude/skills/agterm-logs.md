# /agterm-logs - AgTerm Log Viewer

View and analyze AgTerm application logs.

## Activation

This skill is invoked when user types `/agterm-logs` or asks to view AgTerm logs.

## Arguments

- `/agterm-logs` - Show recent logs (last 50 lines)
- `/agterm-logs tail` - Follow logs in real-time
- `/agterm-logs error` - Show only error logs
- `/agterm-logs pty` - Show PTY-related logs
- `/agterm-logs clear` - Clear log files

## Instructions

### View Recent Logs

```bash
# Determine log directory based on platform
if [[ "$OSTYPE" == "darwin"* ]]; then
    LOG_DIR="$HOME/Library/Application Support/agterm/logs"
else
    LOG_DIR="$HOME/.local/share/agterm/logs"
fi

# Find the latest log file
LATEST_LOG=$(ls -t "$LOG_DIR"/agterm.log.* 2>/dev/null | head -1)

if [ -n "$LATEST_LOG" ]; then
    tail -50 "$LATEST_LOG"
else
    echo "No log files found in $LOG_DIR"
fi
```

### Follow Logs in Real-time

```bash
if [[ "$OSTYPE" == "darwin"* ]]; then
    LOG_DIR="$HOME/Library/Application Support/agterm/logs"
else
    LOG_DIR="$HOME/.local/share/agterm/logs"
fi
tail -f "$LOG_DIR"/agterm.log.* 2>/dev/null
```

### Filter by Log Level

```bash
# Error logs only
grep -i "ERROR" "$LOG_DIR"/agterm.log.*

# Warning and above
grep -iE "WARN|ERROR" "$LOG_DIR"/agterm.log.*

# Debug level (verbose)
grep -i "DEBUG\|TRACE" "$LOG_DIR"/agterm.log.*
```

### Filter by Module

```bash
# PTY logs
grep "pty\|PTY" "$LOG_DIR"/agterm.log.*

# Terminal logs
grep "terminal" "$LOG_DIR"/agterm.log.*

# Main app logs
grep "agterm::" "$LOG_DIR"/agterm.log.*
```

### Clear Logs

```bash
if [[ "$OSTYPE" == "darwin"* ]]; then
    rm -f "$HOME/Library/Application Support/agterm/logs"/agterm.log.*
else
    rm -f "$HOME/.local/share/agterm/logs"/agterm.log.*
fi
echo "Log files cleared"
```

## Log Format

AgTerm logs use the following format:
```
TIMESTAMP LEVEL TARGET: message field1=value1 field2=value2
```

Example:
```
2024-01-15T10:30:45.123Z INFO agterm::terminal::pty: PTY session created session_id=abc123
```

## Analysis Tips

When analyzing logs:
1. Look for ERROR and WARN entries first
2. Check PTY session lifecycle (create → read/write → close)
3. Monitor FPS drops correlating with specific operations
4. Track memory-related warnings
