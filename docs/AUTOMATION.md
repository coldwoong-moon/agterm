# Terminal Automation API

AgTerm provides a comprehensive terminal automation API that allows you to programmatically control terminal sessions through scripts and commands.

## Features

- **Command System**: Send text, keys, and control sequences
- **Pattern Matching**: Wait for and expect specific output patterns
- **Screen Capture**: Capture terminal output at any time
- **Scripting DSL**: Simple domain-specific language for automation
- **Variable Support**: Use variables and environment variables in scripts
- **Conditional Execution**: Execute commands based on conditions

## Quick Start

```rust
use agterm::automation::{AutomationEngine, AutomationScript};
use agterm::terminal::pty::PtyManager;

// Create PTY and automation engine
let manager = PtyManager::new();
let pty_id = manager.create_session(40, 120).unwrap();
let mut engine = AutomationEngine::new(manager, pty_id);

// Execute a simple script
let script_text = r#"
    SEND "echo hello"
    SEND_KEY Enter
    WAIT_FOR "hello" 5s
"#;

engine.execute_script_str(script_text).unwrap();
```

## Script DSL Reference

### Basic Commands

#### SEND
Send text to the terminal with a newline.

```
SEND "echo hello world"
SEND "ls -la"
```

#### SEND_TEXT
Send text without appending a newline.

```
SEND_TEXT "username"
```

#### SEND_KEY
Send a specific key or key combination.

```
SEND_KEY Enter
SEND_KEY Tab
SEND_KEY CTRL+C
SEND_KEY F1
```

Supported keys:
- Basic: `Enter`, `Tab`, `Backspace`, `Escape`, `Space`
- Arrows: `Up`, `Down`, `Left`, `Right`
- Navigation: `Home`, `End`, `PageUp`, `PageDown`, `Insert`, `Delete`
- Function: `F1` through `F12`
- Modifiers: `CTRL+<key>`, `ALT+<key>`
- Characters: Any single character

### Pattern Matching

#### WAIT_FOR
Wait for a pattern to appear in the output, with a timeout.

```
WAIT_FOR "password:" 10s
WAIT_FOR "Build complete" 5s
```

Timeout formats:
- `ms` - milliseconds (e.g., `500ms`)
- `s` - seconds (e.g., `5s`)
- `m` - minutes (e.g., `2m`)

#### EXPECT
Expect a pattern in the output. Fails the script if not found.

```
EXPECT "Success"
EXPECT "Connection established"
```

#### CAPTURE
Capture the current terminal output.

```
CAPTURE
```

The captured output is stored and can be used with pattern matching conditions.

### Variables

#### SET
Define or update a variable.

```
SET USER="alice"
SET DIR="/home/alice"
SET COUNT="42"
```

#### Variable Expansion
Use variables in commands with `${VAR}` or `$VAR` syntax.

```
SET USER="bob"
SEND "echo Hello ${USER}"
SEND "cd $HOME"
```

#### Environment Variables
Access environment variables with the `ENV:` prefix.

```
SEND "echo ${ENV:HOME}"
SEND "cd ${ENV:PWD}"
```

### Control Flow

#### SLEEP
Pause execution for a specified duration.

```
SLEEP 1s
SLEEP 500ms
SLEEP 2m
```

#### CLEAR
Clear the terminal screen.

```
CLEAR
```

#### EXECUTE
Execute a command and optionally wait for completion.

```
EXECUTE "ls -la"
```

### Comments

Lines starting with `#` are treated as comments.

```
# This is a comment
SEND "echo hello"  # This command sends text
```

## Programmatic API

### AutomationCommand

The core command enum for automation operations:

```rust
use agterm::automation::{AutomationCommand, Key, Pattern};
use std::time::Duration;

// Send text
let cmd = AutomationCommand::SendText {
    text: "echo hello".to_string(),
    append_newline: true,
};

// Send keys
let cmd = AutomationCommand::SendKeys(vec![
    Key::Char('l'),
    Key::Char('s'),
    Key::Enter,
]);

// Wait for pattern
let cmd = AutomationCommand::WaitFor {
    pattern: Pattern::Exact("password:".to_string()),
    timeout: Duration::from_secs(10),
};

// Capture output
let cmd = AutomationCommand::Capture {
    store_in: Some("output".to_string()),
};

// Conditional execution
let cmd = AutomationCommand::If {
    condition: Condition::VarEquals("STATUS".to_string(), "ok".to_string()),
    then_commands: vec![/* commands if true */],
    else_commands: vec![/* commands if false */],
};
```

### Pattern Types

```rust
use agterm::automation::Pattern;
use regex::Regex;

// Exact string match
let pattern = Pattern::Exact("hello world".to_string());

// Regular expression
let pattern = Pattern::Regex(Regex::new(r"\d{3}-\d{4}").unwrap());

// Match any of multiple patterns
let pattern = Pattern::AnyOf(vec![
    Pattern::Exact("success".to_string()),
    Pattern::Exact("completed".to_string()),
]);

// Match all patterns
let pattern = Pattern::AllOf(vec![
    Pattern::Exact("build".to_string()),
    Pattern::Exact("successful".to_string()),
]);
```

### Conditions

```rust
use agterm::automation::Condition;
use regex::Regex;

// Variable equals value
let cond = Condition::VarEquals("STATUS".to_string(), "ok".to_string());

// Variable contains substring
let cond = Condition::VarContains("OUTPUT".to_string(), "error".to_string());

// Variable matches regex
let cond = Condition::VarMatches(
    "VERSION".to_string(),
    Regex::new(r"\d+\.\d+\.\d+").unwrap()
);

// Environment variable exists
let cond = Condition::EnvExists("HOME".to_string());

// Pattern matches last captured output
let cond = Condition::PatternMatches(Pattern::Exact("success".to_string()));

// Logical operations
let cond = Condition::And(
    Box::new(Condition::VarEquals("A".to_string(), "1".to_string())),
    Box::new(Condition::VarEquals("B".to_string(), "2".to_string())),
);

let cond = Condition::Or(
    Box::new(Condition::VarEquals("X".to_string(), "1".to_string())),
    Box::new(Condition::VarEquals("Y".to_string(), "2".to_string())),
);

let cond = Condition::Not(
    Box::new(Condition::VarEquals("Z".to_string(), "0".to_string()))
);
```

### Building Scripts Programmatically

```rust
use agterm::automation::{AutomationScript, AutomationCommand};
use std::time::Duration;

let mut script = AutomationScript::new("my_script")
    .with_description("Example automation script");

// Add variables
script.set_variable("SERVER", "example.com");
script.set_variable("PORT", "22");

// Add commands
script.add_command(AutomationCommand::SendText {
    text: "ssh user@${SERVER}".to_string(),
    append_newline: true,
});

script.add_command(AutomationCommand::Sleep(Duration::from_secs(1)));

script.add_command(AutomationCommand::WaitFor {
    pattern: Pattern::Exact("password:".to_string()),
    timeout: Duration::from_secs(10),
});

// Execute script
let mut engine = AutomationEngine::new(manager, pty_id);
let results = engine.execute_script(&script).unwrap();
```

### Execution Context

The execution context maintains state during script execution:

```rust
use agterm::automation::ExecutionContext;
use std::collections::HashMap;

// Create context with initial variables
let mut variables = HashMap::new();
variables.insert("USER".to_string(), "alice".to_string());
let mut context = ExecutionContext::new(variables);

// Add output to buffer
context.append_output("Build successful\n");

// Expand variables in text
let expanded = context.expand_variables("Hello ${USER}");
assert_eq!(expanded, "Hello alice");

// Access last captured output
if let Some(capture) = &context.last_capture {
    println!("Last capture: {}", capture);
}
```

## Complete Examples

### Example 1: Simple Command Sequence

```
# Simple automation
SEND "cd /tmp"
SEND_KEY Enter
SLEEP 500ms
SEND "ls -la"
SEND_KEY Enter
WAIT_FOR "total" 3s
CAPTURE
```

### Example 2: SSH Connection

```
SET HOST="example.com"
SET USER="admin"

# Connect to server
SEND "ssh ${USER}@${HOST}"
SEND_KEY Enter

# Wait for password prompt
WAIT_FOR "password:" 10s

# Note: In real usage, handle password securely
SLEEP 1s

# Execute commands on server
SEND "uptime"
SEND_KEY Enter
WAIT_FOR "load average" 5s
CAPTURE

# Disconnect
SEND "exit"
SEND_KEY Enter
```

### Example 3: Build Process Automation

```
SET PROJECT_DIR="/path/to/project"
SET BUILD_TYPE="release"

# Navigate to project
SEND "cd ${PROJECT_DIR}"
SEND_KEY Enter
SLEEP 500ms

# Clean previous build
SEND "make clean"
SEND_KEY Enter
WAIT_FOR "done" 5s

# Start build
SEND "make ${BUILD_TYPE}"
SEND_KEY Enter
WAIT_FOR "Build successful" 60s

# Verify build
EXPECT "Build successful"
CAPTURE

# Run tests
SEND "make test"
SEND_KEY Enter
WAIT_FOR "All tests passed" 30s
EXPECT "All tests passed"
```

### Example 4: Interactive Debugging Session

```
SET PROGRAM="./my_app"
SET BREAKPOINT="main.c:42"

# Start debugger
SEND "gdb ${PROGRAM}"
SEND_KEY Enter
WAIT_FOR "(gdb)" 3s

# Set breakpoint
SEND "break ${BREAKPOINT}"
SEND_KEY Enter
WAIT_FOR "Breakpoint" 2s
EXPECT "Breakpoint"

# Run program
SEND "run"
SEND_KEY Enter
WAIT_FOR "Breakpoint 1" 5s

# Inspect variables
SEND "print myvar"
SEND_KEY Enter
WAIT_FOR "$1 =" 2s
CAPTURE

# Continue execution
SEND "continue"
SEND_KEY Enter
WAIT_FOR "exited normally" 10s

# Quit debugger
SEND "quit"
SEND_KEY Enter
```

### Example 5: Git Operations

```
SET BRANCH="feature/new-feature"
SET REMOTE="origin"

# Check status
SEND "git status"
SEND_KEY Enter
WAIT_FOR "On branch" 2s
CAPTURE

# Create and switch to new branch
SEND "git checkout -b ${BRANCH}"
SEND_KEY Enter
WAIT_FOR "Switched to" 2s
EXPECT "Switched to"

# Stage changes
SEND "git add ."
SEND_KEY Enter
SLEEP 500ms

# Commit changes
SEND "git commit -m 'Add new feature'"
SEND_KEY Enter
WAIT_FOR "changed" 3s

# Push to remote
SEND "git push ${REMOTE} ${BRANCH}"
SEND_KEY Enter
WAIT_FOR "new branch" 10s
EXPECT "new branch"
```

## Error Handling

The automation API provides detailed error types:

```rust
use agterm::automation::AutomationError;

match engine.execute_command(&cmd) {
    Ok(result) => {
        println!("Success! Duration: {:?}", result.duration);
        if let Some(output) = result.output {
            println!("Output: {}", output);
        }
    }
    Err(AutomationError::Timeout(msg)) => {
        eprintln!("Timeout: {}", msg);
    }
    Err(AutomationError::ExpectationFailed(msg)) => {
        eprintln!("Expected pattern not found: {}", msg);
    }
    Err(AutomationError::PtyError(e)) => {
        eprintln!("PTY error: {}", e);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

## Best Practices

1. **Use timeouts wisely**: Set appropriate timeouts for WAIT_FOR commands based on expected operation duration.

2. **Handle errors gracefully**: Always check command results and handle errors appropriately.

3. **Use variables**: Define frequently used values as variables for maintainability.

4. **Add delays**: Use SLEEP between commands when needed to allow the terminal to process output.

5. **Capture output**: Use CAPTURE to store output for later verification or debugging.

6. **Test incrementally**: Build and test automation scripts incrementally rather than writing large scripts at once.

7. **Comment your scripts**: Use comments to document complex automation sequences.

8. **Validate expectations**: Use EXPECT to validate critical outcomes in your automation flow.

## Integration with AgTerm

The automation API integrates seamlessly with AgTerm's PTY management:

```rust
use agterm::terminal::pty::PtyManager;
use agterm::automation::AutomationEngine;

// Get PTY manager from AgTerm application
let manager = pty_manager.clone();
let pty_id = active_terminal_id;

// Create automation engine
let mut engine = AutomationEngine::new(manager, pty_id);

// Execute automation
let script = load_script_from_file("automation.txt");
match engine.execute_script_str(&script) {
    Ok(results) => {
        println!("Automation completed: {} commands executed", results.len());
    }
    Err(e) => {
        eprintln!("Automation failed: {}", e);
    }
}
```

## Future Enhancements

Planned features for future releases:

- **Async/await support**: Non-blocking automation execution
- **Recording and playback**: Record terminal sessions as automation scripts
- **Script debugging**: Step-through debugging for automation scripts
- **Advanced pattern matching**: XPath-like selectors for structured output
- **Script templates**: Reusable script components with parameters
- **Parallel execution**: Run multiple automation scripts concurrently
- **Event-driven automation**: Trigger scripts based on terminal events

## See Also

- [examples/automation_example.rs](../examples/automation_example.rs) - Complete working examples
- [src/automation.rs](../src/automation.rs) - API implementation
- [Terminal PTY Documentation](./PTY.md) - PTY management details
