# Automation API Quick Start Guide

Get started with AgTerm's terminal automation in 5 minutes.

## Installation

The automation API is built into AgTerm. No additional dependencies needed!

```rust
use agterm::automation::*;
use agterm::terminal::pty::PtyManager;
```

## Basic Usage

### 1. Create an Automation Engine

```rust
// Initialize PTY manager
let manager = PtyManager::new();
let pty_id = manager.create_session(40, 120).unwrap();

// Create automation engine
let mut engine = AutomationEngine::new(manager, pty_id);
```

### 2. Execute a Simple Script

```rust
let script = r#"
    SEND "echo Hello, World!"
    SEND_KEY Enter
    WAIT_FOR "Hello" 5s
"#;

engine.execute_script_str(script).unwrap();
```

### 3. Use Variables

```rust
let script = r#"
    SET USER="alice"
    SET DIR="/home/alice"

    SEND "cd ${DIR}"
    SEND_KEY Enter

    SEND "echo Welcome ${USER}"
    SEND_KEY Enter
"#;

engine.execute_script_str(script).unwrap();
```

## Common Patterns

### SSH Connection
```rust
let script = r#"
    SET HOST="example.com"
    SET USER="admin"

    SEND "ssh ${USER}@${HOST}"
    SEND_KEY Enter
    WAIT_FOR "password:" 10s
"#;
```

### Build Process
```rust
let script = r#"
    SEND "make clean"
    SEND_KEY Enter
    WAIT_FOR "done" 5s

    SEND "make all"
    SEND_KEY Enter
    WAIT_FOR "Build successful" 60s
    EXPECT "Build successful"
"#;
```

### Git Workflow
```rust
let script = r#"
    SET BRANCH="feature/new-feature"

    SEND "git checkout -b ${BRANCH}"
    SEND_KEY Enter
    EXPECT "Switched to"

    SEND "git add ."
    SEND_KEY Enter

    SEND "git commit -m 'Add feature'"
    SEND_KEY Enter
"#;
```

## Programmatic API

For more control, use the programmatic API:

```rust
use agterm::automation::*;

// Create script
let mut script = AutomationScript::new("my_script");
script.set_variable("SERVER", "example.com");

// Add commands
script.add_command(AutomationCommand::SendText {
    text: "echo Hello".to_string(),
    append_newline: true,
});

script.add_command(AutomationCommand::WaitFor {
    pattern: Pattern::Exact("Hello".to_string()),
    timeout: Duration::from_secs(5),
});

// Execute
let results = engine.execute_script(&script).unwrap();
```

## Error Handling

Always handle errors appropriately:

```rust
match engine.execute_script_str(script) {
    Ok(results) => {
        println!("Success! {} commands executed", results.len());
    }
    Err(AutomationError::Timeout(msg)) => {
        eprintln!("Timeout: {}", msg);
    }
    Err(AutomationError::ExpectationFailed(msg)) => {
        eprintln!("Expected pattern not found: {}", msg);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

## Available Commands

| Command | Description | Example |
|---------|-------------|---------|
| `SEND` | Send text with newline | `SEND "ls -la"` |
| `SEND_TEXT` | Send text without newline | `SEND_TEXT "password"` |
| `SEND_KEY` | Send specific key | `SEND_KEY Enter` |
| `WAIT_FOR` | Wait for pattern | `WAIT_FOR "done" 5s` |
| `EXPECT` | Expect pattern (fail if missing) | `EXPECT "success"` |
| `CAPTURE` | Capture output | `CAPTURE` |
| `SET` | Define variable | `SET VAR="value"` |
| `SLEEP` | Pause execution | `SLEEP 500ms` |
| `CLEAR` | Clear screen | `CLEAR` |
| `EXECUTE` | Execute command | `EXECUTE "ls"` |

## Key Names

- Basic: `Enter`, `Tab`, `Backspace`, `Escape`
- Arrows: `Up`, `Down`, `Left`, `Right`
- Navigation: `Home`, `End`, `PageUp`, `PageDown`
- Function: `F1`, `F2`, ..., `F12`
- Control: `CTRL+C`, `CTRL+D`, etc.
- Alt: `ALT+A`, `ALT+B`, etc.

## Duration Format

- `100ms` - 100 milliseconds
- `5s` - 5 seconds
- `2m` - 2 minutes

## Variable Expansion

```rust
SET NAME="Alice"
SEND "Hello ${NAME}"        # Use ${VAR}
SEND "Hello $NAME"          # Use $VAR
SEND "Path: ${ENV:HOME}"    # Use ${ENV:VAR}
```

## Next Steps

1. **Full Documentation**: See [AUTOMATION.md](./AUTOMATION.md)
2. **Quick Reference**: See [AUTOMATION_DSL_REFERENCE.md](./AUTOMATION_DSL_REFERENCE.md)
3. **Examples**: See [examples/automation_example.rs](../examples/automation_example.rs)
4. **Tests**: See [tests/automation_tests.rs](../tests/automation_tests.rs)

## Tips

1. **Start Simple**: Begin with basic commands, add complexity gradually
2. **Use Comments**: Document your scripts with `# comment`
3. **Set Timeouts**: Always provide reasonable timeouts for `WAIT_FOR`
4. **Add Delays**: Use `SLEEP` between commands when needed
5. **Capture Output**: Use `CAPTURE` for debugging
6. **Test Incrementally**: Test each section before combining

## Complete Example

```rust
use agterm::automation::*;
use agterm::terminal::pty::PtyManager;

fn main() {
    // Setup
    let manager = PtyManager::new();
    let pty_id = manager.create_session(40, 120).unwrap();
    let mut engine = AutomationEngine::new(manager, pty_id);

    // Script
    let script = r#"
        # Configure environment
        SET PROJECT_DIR="/tmp/test"
        SET BUILD_CMD="make all"

        # Navigate and build
        SEND "cd ${PROJECT_DIR}"
        SEND_KEY Enter
        SLEEP 500ms

        SEND "${BUILD_CMD}"
        SEND_KEY Enter
        WAIT_FOR "Build complete" 30s
        EXPECT "Build complete"

        # Verify
        CAPTURE
        SEND "echo Done"
        SEND_KEY Enter
    "#;

    // Execute
    match engine.execute_script_str(script) {
        Ok(results) => {
            println!("✓ Automation completed successfully");
            println!("  {} commands executed", results.len());
        }
        Err(e) => {
            eprintln!("✗ Automation failed: {}", e);
        }
    }
}
```

## Help

- **Issues**: Check error messages for detailed information
- **Debugging**: Add `CAPTURE` commands to inspect output
- **Performance**: Use appropriate timeouts and sleeps
- **Documentation**: Refer to comprehensive docs for advanced features

Happy Automating! 🚀
