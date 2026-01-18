# Automation DSL Quick Reference

## Command Syntax

### Text Input Commands

```
SEND "text"              # Send text with newline
SEND_TEXT "text"         # Send text without newline
```

### Key Commands

```
SEND_KEY <key>           # Send a single key
```

**Supported Keys:**
- Basic: `Enter`, `Tab`, `Backspace`, `Escape`, `Space`
- Navigation: `Up`, `Down`, `Left`, `Right`, `Home`, `End`, `PageUp`, `PageDown`
- Editing: `Insert`, `Delete`
- Function: `F1`, `F2`, ..., `F12`
- Control: `CTRL+A`, `CTRL+C`, `CTRL+D`, etc.
- Alt: `ALT+A`, `ALT+B`, etc.
- Characters: Any single character (e.g., `a`, `5`, `!`)

### Pattern Matching

```
WAIT_FOR "pattern" <timeout>    # Wait for pattern with timeout
EXPECT "pattern"                # Expect pattern (fail if not found)
CAPTURE                         # Capture current output
```

**Timeout Format:**
- `100ms` - milliseconds
- `5s` - seconds
- `2m` - minutes

### Variables

```
SET NAME="value"                # Define a variable
${NAME}                         # Use variable (braces)
$NAME                           # Use variable (no braces)
${ENV:VAR}                      # Access environment variable
```

### Control Flow

```
SLEEP <duration>                # Pause execution
CLEAR                           # Clear screen
EXECUTE "command"               # Execute command
```

### Comments

```
# This is a comment
SEND "command"  # Inline comment
```

## Complete Example

```bash
# SSH Connection and Command Execution
SET HOST="server.example.com"
SET USER="admin"
SET TIMEOUT="30s"

# Connect
SEND "ssh ${USER}@${HOST}"
SEND_KEY Enter
WAIT_FOR "password:" ${TIMEOUT}

# Execute commands
SEND "uptime"
SEND_KEY Enter
WAIT_FOR "load average" 10s
CAPTURE

SEND "df -h"
SEND_KEY Enter
WAIT_FOR "Filesystem" 5s
EXPECT "Filesystem"

# Disconnect
SEND "exit"
SEND_KEY Enter
```

## Common Patterns

### Login Automation
```bash
SEND "ssh user@host"
SEND_KEY Enter
WAIT_FOR "password:" 10s
SEND_TEXT "${PASSWORD}"
SEND_KEY Enter
WAIT_FOR "$" 5s
```

### Build Process
```bash
SET BUILD_DIR="/path/to/project"
SEND "cd ${BUILD_DIR}"
SEND_KEY Enter
SLEEP 500ms

SEND "make clean"
SEND_KEY Enter
WAIT_FOR "done" 5s

SEND "make all"
SEND_KEY Enter
WAIT_FOR "Build successful" 60s
EXPECT "Build successful"
```

### Interactive Menu Navigation
```bash
SEND "app-menu"
SEND_KEY Enter
WAIT_FOR "Select option" 2s

SEND_KEY Down
SEND_KEY Down
SEND_KEY Enter

WAIT_FOR "Processing" 5s
CAPTURE
```

### Database Operations
```bash
SET DB="mydb"
SET QUERY="SELECT COUNT(*) FROM users"

SEND "psql ${DB}"
SEND_KEY Enter
WAIT_FOR "${DB}=#" 3s

SEND "${QUERY}"
SEND_KEY Enter
WAIT_FOR "(1 row)" 5s
CAPTURE

SEND "\\q"
SEND_KEY Enter
```

### Git Workflow
```bash
SET BRANCH="feature/new-feature"

SEND "git checkout -b ${BRANCH}"
SEND_KEY Enter
EXPECT "Switched to"

SEND "git add ."
SEND_KEY Enter
SLEEP 500ms

SEND "git commit -m 'Add feature'"
SEND_KEY Enter
WAIT_FOR "changed" 5s

SEND "git push origin ${BRANCH}"
SEND_KEY Enter
WAIT_FOR "new branch" 30s
```

### Testing and Verification
```bash
# Run tests and verify
SEND "npm test"
SEND_KEY Enter
WAIT_FOR "Test Suites:" 60s
CAPTURE

# Check results
EXPECT "passed"
EXPECT "0 failed"
```

### Conditional Logic (Programmatic)
```rust
// In Rust code
let cmd = AutomationCommand::If {
    condition: Condition::VarEquals("ENV".to_string(), "prod".to_string()),
    then_commands: vec![
        AutomationCommand::SendText {
            text: "echo 'Production environment'".to_string(),
            append_newline: true,
        }
    ],
    else_commands: vec![
        AutomationCommand::SendText {
            text: "echo 'Development environment'".to_string(),
            append_newline: true,
        }
    ],
};
```

## Error Handling

### Timeouts
When `WAIT_FOR` times out, the script fails:
```bash
WAIT_FOR "success" 5s  # Fails after 5 seconds if not found
```

### Expectations
When `EXPECT` doesn't match, the script fails:
```bash
EXPECT "Build successful"  # Fails if not found in output
```

### Recovery Strategies
```rust
// In Rust code
match engine.execute_script_str(script) {
    Ok(results) => println!("Success"),
    Err(AutomationError::Timeout(msg)) => {
        eprintln!("Timeout: {}", msg);
        // Retry logic here
    }
    Err(AutomationError::ExpectationFailed(msg)) => {
        eprintln!("Expectation failed: {}", msg);
        // Handle failure
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Best Practices

1. **Use appropriate timeouts**: Consider command execution time
   ```bash
   WAIT_FOR "password:" 10s    # Login might be slow
   WAIT_FOR ">" 2s             # Prompt should be fast
   ```

2. **Add delays between commands**: Let terminal process
   ```bash
   SEND "command1"
   SEND_KEY Enter
   SLEEP 500ms                 # Wait for output
   SEND "command2"
   SEND_KEY Enter
   ```

3. **Capture for debugging**: Store output for inspection
   ```bash
   SEND "command"
   SEND_KEY Enter
   WAIT_FOR "done" 10s
   CAPTURE                     # Save output
   ```

4. **Use variables for maintainability**:
   ```bash
   SET HOST="server.com"
   SET PORT="22"
   SEND "ssh -p ${PORT} user@${HOST}"
   ```

5. **Comment your scripts**:
   ```bash
   # Connect to production server
   SET HOST="prod.example.com"

   # Execute health check
   SEND "curl localhost:8080/health"
   SEND_KEY Enter
   ```

## Advanced Usage

### Multi-step Workflows
```bash
# Step 1: Setup
SET PROJECT="myapp"
SEND "cd /opt/${PROJECT}"
SEND_KEY Enter
SLEEP 500ms

# Step 2: Backup
SEND "tar czf backup.tar.gz *"
SEND_KEY Enter
WAIT_FOR "backup.tar.gz" 30s
EXPECT "backup.tar.gz"

# Step 3: Deploy
SEND "git pull"
SEND_KEY Enter
WAIT_FOR "Already up to date" 10s

# Step 4: Restart
SEND "systemctl restart ${PROJECT}"
SEND_KEY Enter
SLEEP 2s

# Step 5: Verify
SEND "systemctl status ${PROJECT}"
SEND_KEY Enter
WAIT_FOR "active (running)" 5s
EXPECT "active (running)"
```

### Environment-specific Scripts
```bash
# Load environment
SET ENV="${ENV:ENVIRONMENT}"

# Production-specific commands
# (Use programmatic If command for conditional logic)

SEND "echo Running in ${ENV} environment"
SEND_KEY Enter
```

### Parameterized Scripts
```bash
# Script accepts parameters via variables
# Set these before running: USER, HOST, COMMAND

SEND "ssh ${USER}@${HOST}"
SEND_KEY Enter
WAIT_FOR "password:" 10s

SEND "${COMMAND}"
SEND_KEY Enter
WAIT_FOR "$" 30s
CAPTURE
```

## Debugging Tips

1. **Use CAPTURE frequently**: See what the terminal outputs
   ```bash
   SEND "command"
   SEND_KEY Enter
   SLEEP 1s
   CAPTURE                     # Check output
   ```

2. **Increase timeouts during debugging**: Avoid false failures
   ```bash
   WAIT_FOR "done" 60s         # Generous timeout
   ```

3. **Add sleep between commands**: Ensure output is complete
   ```bash
   SEND "command"
   SEND_KEY Enter
   SLEEP 1s                    # Let output settle
   ```

4. **Check expectations**: Verify patterns match actual output
   ```bash
   EXPECT "expected text"      # Make sure this appears
   ```

5. **Test incrementally**: Build scripts step by step
   ```bash
   # Test each section separately
   SEND "ls"
   SEND_KEY Enter
   CAPTURE
   # Verify this works before adding more
   ```
