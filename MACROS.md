# AgTerm Macro System

The AgTerm macro system provides powerful automation capabilities for terminal interactions. You can record, define, and execute complex sequences of actions to streamline your workflow.

## Features

- **Record and Replay**: Record your keyboard inputs and replay them later
- **Complex Workflows**: Chain multiple actions together with timing controls
- **Key Triggers**: Bind macros to keyboard shortcuts
- **Composition**: Call macros from other macros
- **Repetition**: Repeat actions multiple times
- **Timing Control**: Add delays between actions
- **Serialization**: Export and import macros as JSON

## Quick Start

```rust
use agterm::macros::*;
use agterm::macros::builders::*;

// Create a macro engine
let mut engine = MacroEngine::new();

// Define a simple macro
let macro_def = Macro::new("hello".to_string(), "Say hello".to_string())
    .add_action(send_line("echo 'Hello, World!'"));

// Register it
engine.register(macro_def).unwrap();

// Execute it
let actions = engine.execute("hello").unwrap();
```

## Core Concepts

### MacroAction

The building blocks of macros. Available actions:

- `SendText(String)` - Send plain text to the terminal
- `SendKeys(Vec<KeyEvent>)` - Send a sequence of key events
- `RunCommand(String)` - Execute a shell command
- `Wait(Duration)` - Wait for a specified duration
- `Repeat { action, count }` - Repeat an action multiple times
- `Sequence(Vec<MacroAction>)` - Execute a sequence of actions
- `CallMacro(String)` - Execute another macro by name

### Macro

A complete macro definition with:
- `name` - Unique identifier
- `description` - Human-readable description
- `trigger` - Optional keyboard shortcut (KeyCombo)
- `actions` - List of actions to execute
- `enabled` - Whether the macro is active

### MacroEngine

The execution engine that:
- Manages macro registration and storage
- Handles trigger matching
- Executes macros with recursion protection
- Supports macro recording
- Exports/imports macro configurations

## Builder Functions

The `builders` module provides convenient functions for creating actions:

```rust
use agterm::macros::builders::*;

// Text operations
send_line("ls -la")      // Send text + Enter
send_text("hello")       // Send text without Enter
send_enter()             // Send Enter key
send_escape()            // Send Escape key
send_ctrl_c()            // Send Ctrl+C

// Timing
wait_ms(500)             // Wait 500 milliseconds
wait_secs(2)             // Wait 2 seconds

// Control flow
repeat(send_text("x"), 5)              // Repeat action 5 times
sequence(vec![action1, action2])        // Run actions in sequence
call_macro("other_macro")               // Call another macro

// Shell commands
run_command("git status")               // Execute shell command
```

## Examples

### 1. Simple Command Macro

```rust
let git_status = Macro::new("gs".to_string(), "Git status".to_string())
    .add_action(send_line("git status"));

engine.register(git_status).unwrap();
```

### 2. Macro with Keyboard Trigger

```rust
let trigger = KeyCombo {
    key: "g".to_string(),
    modifiers: KeyModifiers {
        ctrl: true,
        alt: true,
        shift: false,
        super_: false,
    },
};

let git_macro = Macro::new("quick_git".to_string(), "Quick git status".to_string())
    .with_trigger(trigger)
    .add_action(send_line("git status"))
    .add_action(wait_ms(500))
    .add_action(send_line("git diff --stat"));

engine.register(git_macro).unwrap();

// Later, check if a key combination triggers a macro
if let Some(macro_name) = engine.match_trigger(&some_key_combo) {
    engine.execute(macro_name).unwrap();
}
```

### 3. Complex Workflow

```rust
let deploy_macro = Macro::new("deploy".to_string(), "Deploy application".to_string())
    .add_action(send_line("echo 'Starting deployment...'"))
    .add_action(send_line("git pull origin main"))
    .add_action(wait_secs(2))
    .add_action(send_line("cargo build --release"))
    .add_action(wait_secs(5))
    .add_action(send_line("./deploy.sh"))
    .add_action(send_line("echo 'Deployment complete!'"));

engine.register(deploy_macro).unwrap();
```

### 4. Macro Composition

```rust
// Base macros
let draw_line = Macro::new("line".to_string(), "Draw a line".to_string())
    .add_action(repeat(send_text("-"), 40))
    .add_action(send_enter());

let header = Macro::new("header".to_string(), "Section header".to_string())
    .add_action(call_macro("line"))
    .add_action(send_line("echo 'SECTION TITLE'"))
    .add_action(call_macro("line"));

engine.register(draw_line).unwrap();
engine.register(header).unwrap();

// Composite macro
let report = Macro::new("report".to_string(), "Generate report".to_string())
    .add_action(call_macro("header"))
    .add_action(send_line("date"))
    .add_action(send_line("uptime"))
    .add_action(call_macro("line"));

engine.register(report).unwrap();
```

### 5. Recording Macros

```rust
// Start recording
engine.start_recording("my_macro".to_string(), false).unwrap();

// Simulate user actions
engine.record_text("cd /tmp".to_string());
engine.record_text("\n".to_string());
engine.record_text("ls -la".to_string());
engine.record_text("\n".to_string());

// Stop recording and get the macro
let recorded_macro = engine.stop_recording(
    "Navigate to tmp and list files".to_string(),
    None  // Optional trigger
).unwrap();

// Register it
engine.register(recorded_macro).unwrap();

// Execute it
engine.execute("my_macro").unwrap();
```

### 6. Recording with Timing

```rust
// Start recording with timing capture
engine.start_recording("timed_macro".to_string(), true).unwrap();

engine.record_text("first command".to_string());
std::thread::sleep(Duration::from_millis(200));  // Delay will be captured
engine.record_text("second command".to_string());

let macro_with_timing = engine.stop_recording(
    "Macro with timing".to_string(),
    None
).unwrap();

// The macro will include Wait actions between commands
```

### 7. Export and Import

```rust
// Export all macros to JSON
let exported = engine.export_all();
let json = serde_json::to_string_pretty(&exported).unwrap();
std::fs::write("macros.json", json).unwrap();

// Load macros from JSON
let json = std::fs::read_to_string("macros.json").unwrap();
let macros: Vec<Macro> = serde_json::from_str(&json).unwrap();

let mut new_engine = MacroEngine::new();
new_engine.load_from_config(macros).unwrap();
```

## Advanced Features

### Recursion Protection

The engine protects against infinite recursion:

```rust
engine.set_max_recursion_depth(10);  // Default is 10

// This will fail after 10 levels
let recursive = Macro::new("recursive".to_string(), "Self-calling".to_string())
    .add_action(call_macro("recursive"));

engine.register(recursive).unwrap();

// Returns Err(MacroError::MaxRecursionDepth)
let result = engine.execute("recursive");
```

### Disabling Macros

```rust
let macro_def = Macro::new("temp".to_string(), "Temporary macro".to_string())
    .add_action(send_line("echo 'test'"))
    .set_enabled(false);  // Disabled

engine.register(macro_def).unwrap();

// Returns empty action list (no-op)
let actions = engine.execute("temp").unwrap();
assert_eq!(actions.len(), 0);

// Re-enable later
if let Some(m) = engine.get_mut("temp") {
    m.enabled = true;
}
```

### Dynamic Macro Modification

```rust
// Get mutable reference and modify
if let Some(macro_def) = engine.get_mut("my_macro") {
    macro_def.description = "Updated description".to_string();
    macro_def.actions.push(send_line("new action"));
}
```

## Integration with AgTerm

The macro system is designed to integrate seamlessly with AgTerm's terminal emulation:

1. **Keyboard Input**: Connect macro triggers to the keybind system
2. **PTY Output**: Send macro actions directly to the PTY
3. **Configuration**: Store macros in AgTerm's config file
4. **UI**: Display available macros in the command palette

Example integration:

```rust
// In your terminal event handler
use agterm::keybind::KeyBindings;
use agterm::macros::MacroEngine;

fn handle_key_press(
    key: &Key,
    modifiers: &Modifiers,
    keybindings: &KeyBindings,
    macro_engine: &MacroEngine,
) {
    // First check if it's a macro trigger
    if let Some(combo) = KeyBindings::from_iced_key(key, modifiers) {
        if let Some(macro_name) = macro_engine.match_trigger(&combo) {
            if let Ok(actions) = macro_engine.execute(macro_name) {
                // Execute each action
                for action in actions {
                    match action {
                        MacroAction::SendText(text) => {
                            // Send to PTY
                            pty.write(text.as_bytes()).unwrap();
                        }
                        MacroAction::Wait(duration) => {
                            std::thread::sleep(duration.into());
                        }
                        // Handle other action types...
                        _ => {}
                    }
                }
                return;
            }
        }
    }

    // Fall back to normal key binding handling
    // ...
}
```

## Configuration File Format

Macros can be stored in JSON format:

```json
[
  {
    "name": "gs",
    "description": "Quick git status",
    "trigger": {
      "key": "g",
      "modifiers": {
        "ctrl": true,
        "alt": true,
        "shift": false,
        "super_": false
      }
    },
    "actions": [
      {
        "SendText": "git status\n"
      }
    ],
    "enabled": true
  }
]
```

## Error Handling

The macro system provides comprehensive error types:

```rust
match engine.execute("nonexistent") {
    Ok(actions) => { /* ... */ }
    Err(MacroError::NotFound(name)) => {
        eprintln!("Macro '{}' not found", name);
    }
    Err(MacroError::MaxRecursionDepth) => {
        eprintln!("Macro recursion limit exceeded");
    }
    Err(e) => {
        eprintln!("Macro error: {}", e);
    }
}
```

## Best Practices

1. **Naming**: Use descriptive, short names for macros
2. **Triggers**: Avoid conflicting with system shortcuts
3. **Timing**: Add appropriate delays for long-running commands
4. **Composition**: Break complex workflows into smaller, reusable macros
5. **Testing**: Test macros thoroughly before binding to triggers
6. **Documentation**: Use clear descriptions for all macros
7. **Export**: Regularly export your macros for backup

## Running the Example

```bash
# Run the comprehensive example
cargo run --example macro_example

# Run the tests
cargo test --lib macros
```

## API Reference

See the inline documentation in `src/macros.rs` for complete API details:

```bash
cargo doc --open --package agterm
```

## Future Enhancements

Potential future features:
- Conditional execution based on command output
- Variable substitution in macro actions
- Macro templates with parameters
- Visual macro editor in the UI
- Macro marketplace/sharing
- Performance profiling for macros
- Macro debugging mode

## License

Part of the AgTerm project, licensed under MIT.
