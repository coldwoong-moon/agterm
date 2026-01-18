# AgTerm Hook System

The AgTerm hook system allows you to define custom actions that are triggered by specific terminal events. This enables automation, notifications, and custom workflows based on what happens in your terminal.

## Table of Contents

- [Overview](#overview)
- [Configuration](#configuration)
- [Hook Structure](#hook-structure)
- [Event Types](#event-types)
- [Action Types](#action-types)
- [Examples](#examples)
- [API Usage](#api-usage)

## Overview

Hooks consist of three main components:

1. **Event Type**: What terminal event triggers the hook
2. **Action**: What happens when the hook is triggered
3. **Enabled State**: Whether the hook is active

Hooks are configured in `~/.config/agterm/hooks.toml`.

## Configuration

### Location

Hooks are stored in:
```
~/.config/agterm/hooks.toml
```

### Basic Structure

```toml
[[hooks]]
name = "Hook Name"
enabled = true

[hooks.event_type]
type = "EventType"
# event-specific data

[hooks.action]
type = "ActionType"
# action-specific data
```

## Hook Structure

### Hook Fields

- `name` (string, required): Unique identifier for the hook
- `enabled` (boolean, default: true): Whether the hook is active
- `event_type` (object, required): The event that triggers this hook
- `action` (object, required): The action to perform when triggered

## Event Types

### 1. CommandComplete

Triggered when a command execution finishes.

**Fields:**
- `command_pattern` (string, optional): Regex pattern to match command
- `exit_code` (integer, optional): Exit code to match (0 = success, non-zero = error)

**Examples:**

```toml
# Match any command
[hooks.event_type]
type = "CommandComplete"

# Match git commands with exit code 0
[hooks.event_type]
type = "CommandComplete"
data = { command_pattern = "git.*", exit_code = 0 }

# Match failed commands
[hooks.event_type]
type = "CommandComplete"
data = { exit_code = 1 }
```

### 2. DirectoryChange

Triggered when the working directory changes.

**Fields:**
- `directory_pattern` (string, optional): Pattern to match directory path

**Examples:**

```toml
# Match any directory change
[hooks.event_type]
type = "DirectoryChange"

# Match specific directory
[hooks.event_type]
type = "DirectoryChange"
data = { directory_pattern = "/var/www" }

# Match home directory
[hooks.event_type]
type = "DirectoryChange"
data = { directory_pattern = "~" }
```

### 3. OutputMatch

Triggered when terminal output matches a pattern.

**Fields:**
- `pattern` (string, required): Regex pattern to match in output

**Examples:**

```toml
# Match error messages (case-insensitive)
[hooks.event_type]
type = "OutputMatch"
data = { pattern = "(?i)(error|fail|fatal)" }

# Match success messages
[hooks.event_type]
type = "OutputMatch"
data = { pattern = "(?i)success|✓|passed" }
```

### 4. Bell

Triggered when the terminal bell character is received.

**Examples:**

```toml
[hooks.event_type]
type = "Bell"
```

## Action Types

### 1. Notify

Send a desktop notification.

**Fields:**
- `title` (string, required): Notification title
- `message` (string, required): Notification message

**Example:**

```toml
[hooks.action]
type = "Notify"
data = { title = "Command Complete", message = "Your command has finished" }
```

### 2. RunCommand

Execute a shell command.

**Fields:**
- `command` (string, required): Command to execute
- `args` (array of strings, optional): Command arguments

**Example:**

```toml
[hooks.action]
type = "RunCommand"
data = { command = "notify-send", args = ["Test", "Message"] }
```

### 3. PlaySound

Play a sound file.

**Fields:**
- `path` (string, required): Path to sound file
- `volume` (float, optional, default: 0.5): Volume level (0.0 to 1.0)

**Example:**

```toml
[hooks.action]
type = "PlaySound"
data = { path = "/System/Library/Sounds/Glass.aiff", volume = 0.7 }
```

### 4. Custom

Execute a custom registered action.

**Fields:**
- `id` (string, required): Custom action identifier
- `params` (object, optional): Action-specific parameters

**Example:**

```toml
[hooks.action]
type = "Custom"
data = { id = "send_slack_message", params = { channel = "#builds", message = "Build complete" } }
```

## Examples

### Example 1: Notify on Git Success

```toml
[[hooks]]
name = "Git Success"
enabled = true

[hooks.event_type]
type = "CommandComplete"
data = { command_pattern = "git.*", exit_code = 0 }

[hooks.action]
type = "Notify"
data = { title = "Git Success", message = "Git command completed successfully" }
```

### Example 2: Sound on Build Complete

```toml
[[hooks]]
name = "Build Complete Sound"
enabled = true

[hooks.event_type]
type = "CommandComplete"
data = { command_pattern = "cargo build|npm run build", exit_code = 0 }

[hooks.action]
type = "PlaySound"
data = { path = "/System/Library/Sounds/Glass.aiff", volume = 0.5 }
```

### Example 3: Warn on Production Directory

```toml
[[hooks]]
name = "Production Warning"
enabled = true

[hooks.event_type]
type = "DirectoryChange"
data = { directory_pattern = "/var/www/production" }

[hooks.action]
type = "Notify"
data = { title = "⚠️  Production Environment", message = "You are in production!" }
```

### Example 4: Alert on Error Output

```toml
[[hooks]]
name = "Error Alert"
enabled = true

[hooks.event_type]
type = "OutputMatch"
data = { pattern = "(?i)error|fatal|panic" }

[hooks.action]
type = "Notify"
data = { title = "Error Detected", message = "Check terminal output" }
```

### Example 5: Multiple Hooks Chain

```toml
# Notify on test start
[[hooks]]
name = "Test Start"
enabled = true

[hooks.event_type]
type = "CommandComplete"
data = { command_pattern = "cargo test|npm test" }

[hooks.action]
type = "Notify"
data = { title = "Tests Running", message = "Test execution started" }

# Play sound on test success
[[hooks]]
name = "Test Success"
enabled = true

[hooks.event_type]
type = "OutputMatch"
data = { pattern = "test result: ok" }

[hooks.action]
type = "PlaySound"
data = { path = "/System/Library/Sounds/Glass.aiff", volume = 0.6 }

# Alert on test failure
[[hooks]]
name = "Test Failure"
enabled = true

[hooks.event_type]
type = "OutputMatch"
data = { pattern = "test result: FAILED" }

[hooks.action]
type = "Notify"
data = { title = "❌ Tests Failed", message = "Some tests failed!" }
```

## API Usage

### In Rust Code

```rust
use agterm::config::{Hook, HookEvent, HookAction, HookManager};

// Create a hook manager
let mut manager = HookManager::new();

// Create a custom hook
let hook = Hook::new(
    "My Hook".to_string(),
    HookEvent::Bell,
    HookAction::Notify {
        title: "Bell".to_string(),
        message: "Terminal bell received".to_string(),
    },
);

// Add the hook
manager.add_hook(hook);

// Process an event
manager.process_event(&HookEvent::Bell);

// Disable a hook
manager.set_hook_enabled("My Hook", false);

// Remove a hook
manager.remove_hook("My Hook");

// Save hooks to file
manager.save().unwrap();

// Reload hooks from file
manager.reload().unwrap();
```

### Event Processing

```rust
// Command completion
manager.process_event(&HookEvent::CommandComplete {
    command_pattern: Some("git status".to_string()),
    exit_code: Some(0),
});

// Directory change
manager.process_event(&HookEvent::DirectoryChange {
    directory_pattern: Some("/home/user".to_string()),
});

// Output matching
manager.process_event(&HookEvent::OutputMatch {
    pattern: "Error: something failed".to_string(),
});

// Terminal bell
manager.process_event(&HookEvent::Bell);
```

## Advanced Usage

### Pattern Matching Tips

1. **Case-Insensitive Matching**: Use `(?i)` prefix
   ```toml
   pattern = "(?i)error"  # Matches: error, Error, ERROR
   ```

2. **Multiple Patterns**: Use `|` (OR)
   ```toml
   pattern = "error|fail|fatal"  # Matches any of these
   ```

3. **Word Boundaries**: Use `\b`
   ```toml
   pattern = "\\berror\\b"  # Matches "error" but not "errors"
   ```

4. **Capture Groups**: For complex matching
   ```toml
   pattern = "test (\\w+): (FAILED|ok)"
   ```

### Performance Considerations

1. **Disable Unused Hooks**: Set `enabled = false` for hooks you don't need
2. **Pattern Complexity**: Simpler patterns are faster
3. **Hook Count**: Limit the number of active hooks for better performance

### Debugging

To see when hooks are triggered, check the AgTerm logs:

```bash
tail -f ~/.config/agterm/agterm.log
```

Look for entries like:
```
INFO Hook 'Git Success' triggered notification: Git Success - Git command completed successfully
```

## Best Practices

1. **Use Descriptive Names**: Make hook names clear and specific
2. **Start with Disabled**: Create new hooks as disabled, then enable after testing
3. **Test Patterns**: Use regex testing tools to verify patterns work correctly
4. **Group Related Hooks**: Use naming conventions (e.g., "Git: Success", "Git: Failure")
5. **Document Custom Hooks**: Add comments in hooks.toml for complex configurations
6. **Backup Configuration**: Keep a backup of your hooks.toml file

## Troubleshooting

### Hook Not Triggering

1. Check if the hook is enabled
2. Verify the event pattern matches your use case
3. Check the AgTerm logs for error messages
4. Test the regex pattern separately

### Performance Issues

1. Reduce the number of active hooks
2. Simplify regex patterns
3. Disable hooks that match very frequently

### Regex Not Matching

1. Test regex at [regex101.com](https://regex101.com)
2. Remember to escape special characters in TOML strings
3. Use raw strings if needed: `pattern = '''raw\string'''`

## Future Enhancements

Planned features for the hook system:

- [ ] Conditional execution based on environment variables
- [ ] Hook chaining (trigger multiple actions)
- [ ] Time-based filtering (only trigger during work hours)
- [ ] Rate limiting (prevent notification spam)
- [ ] Web hook integration
- [ ] Script execution with environment variables
- [ ] Hook templates and presets
- [ ] GUI hook configuration

## See Also

- [Configuration Guide](../README.md#configuration)
- [Examples](../examples/hooks.toml.example)
- [API Documentation](https://docs.rs/agterm)

## Contributing

Found a bug or have a feature request? Please open an issue on [GitHub](https://github.com/yourusername/agterm).
