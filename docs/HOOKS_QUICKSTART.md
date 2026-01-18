# Hook System Quick Start

## 5-Minute Integration Guide

### 1. Import the Hook System

```rust
use agterm::config::{HookManager, HookEvent};
```

### 2. Initialize Hook Manager

```rust
// In your main application state
struct AppState {
    hook_manager: HookManager,
    // ... other fields
}

impl AppState {
    fn new() -> Self {
        Self {
            hook_manager: HookManager::new(), // Automatically loads from file
            // ... other fields
        }
    }
}
```

### 3. Trigger Hooks on Events

#### Command Completion
```rust
// When a command finishes
let exit_code = process.wait().unwrap().code().unwrap_or(1);
self.hook_manager.process_event(&HookEvent::CommandComplete {
    command_pattern: Some(command.clone()),
    exit_code: Some(exit_code),
});
```

#### Directory Change
```rust
// When directory changes
if current_dir != previous_dir {
    self.hook_manager.process_event(&HookEvent::DirectoryChange {
        directory_pattern: Some(current_dir.to_string_lossy().to_string()),
    });
}
```

#### Output Pattern
```rust
// When processing terminal output
let output_text = String::from_utf8_lossy(&output);
self.hook_manager.process_event(&HookEvent::OutputMatch {
    pattern: output_text.to_string(),
});
```

#### Terminal Bell
```rust
// When bell character is received (typically in ANSI escape handler)
self.hook_manager.process_event(&HookEvent::Bell);
```

## Common Integration Points

### In PTY Handler
```rust
// src/terminal/pty.rs
pub fn read(&self, id: &PtyId) -> Result<Vec<u8>, PtyError> {
    let output = /* ... read from PTY ... */;

    // Trigger hook on output
    if let Some(hook_manager) = &self.hook_manager {
        let text = String::from_utf8_lossy(&output);
        hook_manager.process_event(&HookEvent::OutputMatch {
            pattern: text.to_string(),
        });
    }

    Ok(output)
}
```

### In ANSI Escape Handler
```rust
// src/terminal/screen.rs - In your Perform implementation
fn execute(&mut self, byte: u8) {
    match byte {
        0x07 => {  // BEL (bell)
            if let Some(hook_manager) = &self.hook_manager {
                hook_manager.process_event(&HookEvent::Bell);
            }
            // ... existing bell handling ...
        }
        // ... other cases ...
    }
}
```

### In Shell Integration
```rust
// Detect command completion from shell integration OSC sequences
fn handle_osc_sequence(&mut self, params: &[&[u8]]) {
    if params[0] == b"633" && params[1] == b"D" {  // Command finished
        let exit_code = params[2].parse().ok();

        if let Some(hook_manager) = &self.hook_manager {
            hook_manager.process_event(&HookEvent::CommandComplete {
                command_pattern: self.last_command.clone(),
                exit_code,
            });
        }
    }
}
```

## User Configuration

Users configure hooks in `~/.config/agterm/hooks.toml`:

```toml
[[hooks]]
name = "Build Success"
enabled = true

[hooks.event_type]
type = "CommandComplete"
data = { command_pattern = "cargo build", exit_code = 0 }

[hooks.action]
type = "Notify"
data = { title = "Build Complete", message = "Cargo build succeeded!" }
```

## Example: Complete Integration

```rust
use agterm::config::{HookManager, HookEvent};

pub struct Terminal {
    hook_manager: HookManager,
    current_dir: PathBuf,
    last_command: Option<String>,
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            hook_manager: HookManager::new(),
            current_dir: std::env::current_dir().unwrap(),
            last_command: None,
        }
    }

    pub fn process_output(&mut self, output: &[u8]) {
        // Process terminal output and trigger hooks
        let text = String::from_utf8_lossy(output);
        self.hook_manager.process_event(&HookEvent::OutputMatch {
            pattern: text.to_string(),
        });
    }

    pub fn on_command_complete(&mut self, command: String, exit_code: i32) {
        self.hook_manager.process_event(&HookEvent::CommandComplete {
            command_pattern: Some(command),
            exit_code: Some(exit_code),
        });
    }

    pub fn on_directory_change(&mut self, new_dir: PathBuf) {
        if new_dir != self.current_dir {
            self.hook_manager.process_event(&HookEvent::DirectoryChange {
                directory_pattern: Some(new_dir.to_string_lossy().to_string()),
            });
            self.current_dir = new_dir;
        }
    }

    pub fn on_bell(&mut self) {
        self.hook_manager.process_event(&HookEvent::Bell);
    }
}
```

## Testing Your Integration

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_integration() {
        let mut terminal = Terminal::new();

        // Test command completion hook
        terminal.on_command_complete("git status".to_string(), 0);

        // Test directory change hook
        terminal.on_directory_change(PathBuf::from("/tmp"));

        // Test output match hook
        terminal.process_output(b"Error: something failed");

        // Test bell hook
        terminal.on_bell();
    }
}
```

## Debugging

Enable logging to see hook execution:

```rust
// In your main.rs
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .init();
```

Look for log messages like:
```
INFO Hook 'Build Success' triggered notification: Build Complete - Cargo build succeeded!
```

## Performance Tips

1. **Lazy Processing**: Only process hooks when events actually occur
2. **Pattern Caching**: Regex patterns are compiled once and cached
3. **Selective Matching**: Disabled hooks are skipped early
4. **Batch Events**: Consider debouncing high-frequency events

## Next Steps

1. ✅ Add hook_manager field to your application state
2. ✅ Call process_event() at appropriate integration points
3. ✅ Test with example hooks from `examples/hooks.toml.example`
4. ⏳ Implement concrete action handlers (notifications, sounds, commands)
5. ⏳ Add UI for hook management

## Need Help?

- See full documentation: [docs/HOOKS.md](./HOOKS.md)
- Check examples: [examples/hooks.toml.example](../examples/hooks.toml.example)
- Run demo: `cargo run --example hook_demo`
