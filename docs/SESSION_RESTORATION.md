# Session Restoration Feature

AgTerm now supports automatic session restoration, allowing you to restore your previous terminal tabs and settings when you restart the application.

## Features

- **Automatic Session Saving**: Saves session state when the application exits
- **Automatic Restoration**: Restores tabs, working directories, and titles on startup
- **Persistent State**: Saves tab count, active tab, working directories, custom titles, and font size
- **Configurable**: Can be enabled/disabled via configuration

## Configuration

Session restoration is configured in your `config.toml` file:

```toml
[general.session]
restore_on_startup = true      # Restore previous session on startup
save_on_exit = true            # Save session on exit
# session_file = "~/.config/agterm/session.json"  # Custom session file path (optional)
```

### Configuration Options

- **`restore_on_startup`** (default: `true`): When enabled, AgTerm will restore the previous session on startup
- **`save_on_exit`** (default: `true`): When enabled, AgTerm will save the current session when the application exits
- **`session_file`** (default: `~/.config/agterm/session.json`): Path to the session file

## Session File Format

The session file is stored in JSON format at `~/.config/agterm/session.json` by default.

Example session file:

```json
{
  "tabs": [
    {
      "cwd": "/Users/username/project1",
      "title": "Web Server",
      "id": 0
    },
    {
      "cwd": "/Users/username/project2",
      "title": "Database",
      "id": 1
    }
  ],
  "active_tab": 1,
  "window_size": [120, 40],
  "font_size": 14.0
}
```

### Session State Fields

- **`tabs`**: Array of tab states
  - **`cwd`**: Working directory for the tab
  - **`title`**: Custom tab title (if set)
  - **`id`**: Unique tab identifier
- **`active_tab`**: Index of the active tab (0-based)
- **`window_size`**: Terminal dimensions `[cols, rows]` (optional)
- **`font_size`**: Font size in pixels

## Usage

### Normal Usage

Session restoration works automatically:

1. **Work in AgTerm**: Open multiple tabs, navigate to different directories, set custom titles
2. **Exit AgTerm**: Close the application normally (Cmd+Q on macOS)
3. **Restart AgTerm**: Your previous tabs will be restored with their working directories and titles

### Disabling Session Restoration

To start fresh without restoring the previous session:

1. Edit `~/.config/agterm/config.toml`
2. Set `restore_on_startup = false` under `[general.session]`
3. Restart AgTerm

Or delete the session file:

```bash
rm ~/.config/agterm/session.json
```

### Custom Session File Location

To use a custom location for the session file:

```toml
[general.session]
restore_on_startup = true
save_on_exit = true
session_file = "/path/to/custom/session.json"
```

## Implementation Details

### Session Saving

- Session is saved automatically when the application exits (via `Drop` trait)
- Session file is created in `~/.config/agterm/` directory (or platform-specific config directory)
- If the config directory doesn't exist, it will be created automatically

### Session Restoration

- On startup, AgTerm checks if session restoration is enabled
- If a session file exists, it loads the tab states and recreates them
- Each restored tab gets a new PTY session with the saved working directory
- If restoration fails, AgTerm starts with a single default tab

### Error Handling

- If the session file is corrupted or invalid, AgTerm logs an error and starts fresh
- If PTY creation fails for a restored tab, the tab is still created but shows an error message
- Missing or inaccessible working directories are preserved as-is (the shell will handle them)

## Limitations

- **Terminal content is not saved**: Only the tab metadata (working directory, title) is saved
- **Running processes are not restored**: Each restored tab starts with a fresh shell
- **Command history is not saved**: Each tab starts with empty command history
- **Window position/size is not fully implemented yet**: The `window_size` field is saved but not currently applied

## Future Enhancements

Potential improvements for session restoration:

1. **Command history preservation**: Save and restore command history per tab
2. **Window geometry restoration**: Restore window position and size
3. **Profile-based sessions**: Different session files for different profiles
4. **Multiple named sessions**: Switch between different saved sessions
5. **Auto-save on interval**: Periodic session saving (not just on exit)
6. **Cloud sync**: Sync sessions across multiple machines

## Troubleshooting

### Session not restoring

1. Check if `restore_on_startup` is enabled in your config
2. Verify the session file exists: `ls -la ~/.config/agterm/session.json`
3. Check logs for errors: `tail -f ~/Library/Application\ Support/agterm/logs/agterm.log` (macOS)

### Session file corrupted

If the session file becomes corrupted:

```bash
# Backup the corrupted file (for debugging)
mv ~/.config/agterm/session.json ~/.config/agterm/session.json.bak

# Start fresh
# AgTerm will create a new session file on next exit
```

### Wrong working directories

If tabs restore with incorrect working directories:

1. The shell's `$PWD` might have changed since the session was saved
2. Check if the directories still exist: `ls -ld /path/to/saved/dir`
3. The shell will typically fall back to your home directory if the saved path doesn't exist

## Examples

### Example 1: Development Workflow

You're working on multiple projects:

1. Tab 1: `/Users/you/project1` - Web server
2. Tab 2: `/Users/you/project2` - API backend
3. Tab 3: `/Users/you/project3` - Database monitoring

When you exit and restart AgTerm, all three tabs are restored with their working directories intact.

### Example 2: Custom Titles

You set custom titles for your tabs:

1. Tab 1: "Production Server" at `/var/www/production`
2. Tab 2: "Staging Server" at `/var/www/staging`
3. Tab 3: "Local Dev" at `~/dev/app`

On restart, both the titles and working directories are restored.

### Example 3: Disabling for Testing

When testing a new configuration:

```bash
# Temporarily disable session restoration
echo -e "[general.session]\nrestore_on_startup = false" >> ~/.config/agterm/config.toml

# Start AgTerm with fresh tabs
agterm

# Re-enable when done
# Edit config.toml and set restore_on_startup = true
```

## See Also

- [Configuration Guide](../README.md#configuration)
- [Keybindings Reference](../README.md#keybindings)
- [Tab Management](../README.md#tab-management)
