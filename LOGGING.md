# AgTerm Logging System

AgTerm uses the `tracing` ecosystem for structured, hierarchical logging with support for multiple output formats and filtering.

## Architecture

### Components

1. **tracing** - Core instrumentation library for structured logging
2. **tracing-subscriber** - Composable layers for log processing
3. **tracing-appender** - File rotation and async writing
4. **LogBuffer** - In-memory ring buffer for debug panel display

### Log Levels

- **TRACE** - Detailed I/O operations, byte-level data (PTY reads/writes with previews)
- **DEBUG** - Application flow, tab operations, state changes
- **INFO** - Important events (session creation, configuration loading)
- **WARN** - Recoverable errors, degraded functionality
- **ERROR** - Critical failures requiring attention

## Configuration

### Environment Variables

```bash
# Set log level for entire application
export AGTERM_LOG="info"

# Set per-module log levels
export AGTERM_LOG="agterm=debug,agterm::terminal::pty=trace"

# Enable debug panel on startup
export AGTERM_DEBUG=1
```

### Config File (`~/.config/agterm/config.toml`)

```toml
[logging]
level = "info"                   # trace, debug, info, warn, error
format = "pretty"                # pretty, compact, json
timestamps = true
file_line = false                # Include file:line in logs
file_output = true               # Write logs to file
# file_path = auto               # Auto-determined by platform
```

### Platform-Specific Log Paths

- **macOS**: `~/Library/Application Support/agterm/logs/agterm-YYYY-MM-DD.log`
- **Linux**: `~/.local/share/agterm/logs/agterm-YYYY-MM-DD.log`
- **Windows**: `%APPDATA%\agterm\logs\agterm-YYYY-MM-DD.log`

Log files rotate daily and are named with the date.

## Logged Events

### Application Lifecycle

```rust
tracing::info!("AgTerm starting");
tracing::info!("Configuration loaded from default + user overrides");
tracing::info!("AgTerm application initialized");
```

### PTY Sessions

```rust
// Session creation
tracing::info!(session_id = %id, "PTY session created");

// Session termination  
tracing::info!(session_id = %id, "Closing PTY session");

// Resize operations
tracing::debug!(session_id = %id, rows = 40, cols = 120, "Resizing PTY");
```

### PTY I/O (TRACE level)

```rust
// Input to PTY with preview
tracing::trace!(
    session_id = %id, 
    bytes = data.len(), 
    preview = %String::from_utf8_lossy(&data[..64]), 
    "PTY input"
);

// Output from PTY with preview
tracing::trace!(
    session_id = %id, 
    bytes = result.len(), 
    preview = %String::from_utf8_lossy(&result[..64]), 
    "PTY output"
);
```

### Tab Operations

```rust
// Creating new tab
tracing::debug!(tab_id = id, "Creating new tab");
tracing::info!(tab_id = id, session_id = %sid, cwd = %cwd, "New tab created");

// Closing tab
tracing::debug!(tab_index = index, tab_id = tab.id, "Closing tab");
tracing::info!(tab_index = index, session_id = %session_id, "Closing PTY session for tab");
tracing::debug!(remaining_tabs = self.tabs.len(), new_active = self.active_tab, "Tab closed");

// Tab switching
tracing::debug!(from_tab = old_tab, to_tab = index, "Tab switched");

// Duplicating tab
tracing::debug!(new_tab_id = id, source_tab = self.active_tab, "Duplicating tab");
tracing::info!(tab_id = id, session_id = %sid, cwd = %cwd, "Tab duplicated");
```

### Configuration Loading

```rust
tracing::info!("Loaded user config from {:?}", user_config_path);
tracing::warn!("Failed to load user config: {}", e);
tracing::info!("Loaded project config from {:?}", project_config_path);
```

## Debug Panel

The debug panel (toggle with `Cmd+D` or `F12`) displays:
- Real-time log stream (last 50 entries by default)
- Filterable by log level
- Searchable by message content
- Color-coded by severity

## Performance Considerations

### Trace Level I/O Logging

PTY I/O logging at TRACE level includes content previews. This can generate significant log volume during active terminal usage:

- Logs show first 64 bytes of data
- Longer messages are truncated with "... (N bytes)"
- Only enabled when `AGTERM_LOG` includes `trace` level
- Minimal overhead when not enabled (guard clauses check `tracing::enabled!()`)

### Log File Rotation

- Daily rotation prevents unbounded growth
- Old log files remain until manually deleted
- Consider implementing log retention policy for long-running systems

## Debugging

### Common Scenarios

1. **PTY issues**: Set `AGTERM_LOG="agterm::terminal::pty=trace"` to see all I/O
2. **Tab management**: Set `AGTERM_LOG="agterm=debug"` to see tab operations
3. **Configuration problems**: Check INFO level for config loading messages
4. **General troubleshooting**: Use `AGTERM_LOG="agterm=debug"` for overview

### Reading Logs

```bash
# Follow live log (macOS)
tail -f ~/Library/Application\ Support/agterm/logs/agterm-$(date +%Y-%m-%d).log

# Search for errors
grep ERROR ~/Library/Application\ Support/agterm/logs/*.log

# PTY session tracking
grep "session_id=" ~/Library/Application\ Support/agterm/logs/*.log
```

## Implementation Details

### Instrumentation Macros

Functions can be instrumented with `#[instrument]` attribute:

```rust
#[instrument(skip(self), fields(session_id = %id, rows = rows, cols = cols))]
pub fn create_session(&self, rows: u16, cols: u16) -> Result<PtyId, PtyError> {
    // Function body automatically logs entry/exit with structured fields
}
```

### Structured Fields

Always use structured fields for searchability:

```rust
// Good - structured
tracing::info!(tab_id = id, session_id = %sid, "Tab created");

// Bad - string interpolation
tracing::info!("Tab {} created with session {}", id, sid);
```

### Field Formatting

- `%` - Display formatting
- `?` - Debug formatting  
- No prefix - Default (usually Debug)

```rust
tracing::info!(
    bytes = data.len(),          // No prefix = raw value
    path = %path,                // Display formatting
    error = ?err,                // Debug formatting
    "Message"
);
```

## Future Enhancements

### Planned Features

1. **Metrics Integration**: Export structured metrics (session count, I/O throughput)
2. **Distributed Tracing**: OpenTelemetry integration for MCP protocol tracing
3. **Log Summarization**: AI-powered log summarization for debugging
4. **Performance Profiling**: Tracing-based performance profiling integration

### Log Retention Policy

Currently, log files accumulate indefinitely. Consider implementing:
- Automatic deletion of logs older than N days
- Compression of old log files
- Size-based rotation (in addition to daily rotation)

## References

- [tracing Documentation](https://docs.rs/tracing/)
- [tracing-subscriber Documentation](https://docs.rs/tracing-subscriber/)
- [tracing-appender Documentation](https://docs.rs/tracing-appender/)
