# Command Validator

The Command Validator module provides risk analysis and validation for AI-generated commands, helping protect users from potentially destructive operations.

## Overview

The validator categorizes commands into four risk levels and determines whether they can be auto-approved or require user confirmation.

## Risk Levels

| Level | Symbol | Description | Auto-Approvable |
|-------|--------|-------------|-----------------|
| **Low** | ✓ | Safe read-only operations | Yes |
| **Medium** | ⚠ | State modifications, generally safe | Yes |
| **High** | ⚠⚠ | Potentially dangerous, needs confirmation | No |
| **Critical** | 🛑 | Destructive operations, never auto-approve | No |

## Usage

### Basic Usage

```rust
use agterm::command_validator::get_validator;

let validator = get_validator();
let result = validator.validate("rm -rf /");

if result.auto_approved {
    // Execute command
} else {
    // Show confirmation dialog
    println!("Risk: {:?}", result.risk_level);
    println!("Reason: {}", result.reason);
}
```

### Risk Level Methods

```rust
use agterm::command_validator::RiskLevel;

// Check if auto-approvable
if RiskLevel::Low.is_auto_approvable() {
    println!("Safe to execute");
}

// Get description
println!("{}", RiskLevel::High.description());

// Get symbol
println!("{}", RiskLevel::Critical.symbol());

// Compare levels
assert!(RiskLevel::Low < RiskLevel::Critical);
```

## Pattern Categories

### Critical (Never Auto-Approve)

Commands that can cause irreversible system damage:

- `rm -rf /` - Remove root filesystem
- `rm -rf ~` - Remove home directory
- `> /etc/*` - Modify system configuration
- `chmod 777` - World-writable permissions on system paths
- `mkfs`, `dd` - Disk formatting operations
- `shutdown`, `reboot` - System power operations

### High Risk (User Confirmation Required)

Commands that require explicit user approval:

- `rm -rf <path>` - Recursive file removal
- `sudo <cmd>` - Privileged operations
- `curl | bash` - Download and execute scripts
- `chmod +x && ./` - Make executable and run
- `apt remove` - Package removal
- `kill -9 1` - Kill init process

### Medium Risk (Auto-Approved with Warning)

Commands that modify state but are generally safe:

- `git push --force` - Force push to remote
- `npm publish` - Publish package
- `docker system prune` - Docker cleanup
- `git reset --hard` - Hard reset repository
- `DROP DATABASE` - Database operations

### Low Risk (Always Auto-Approved)

Safe read-only operations:

- File navigation: `ls`, `pwd`, `cd`, `cat`, `grep`
- Git read-only: `git status`, `git log`, `git diff`
- Package info: `npm list`, `cargo check`
- System info: `uname`, `whoami`, `ps`

## Pattern Matching Logic

The validator checks patterns in this order:

1. **Critical patterns** - Checked first to catch dangerous operations
2. **High-risk patterns** - Checked second
3. **Medium-risk patterns** - Checked third
4. **Whitelist patterns** - Checked last (after dangerous patterns)
5. **Unknown commands** - Default to Medium risk

This ensures that dangerous patterns like `> /etc/passwd` are caught even if they start with whitelisted commands like `echo`.

## Validation Result

The `ValidationResult` struct contains:

```rust
pub struct ValidationResult {
    pub risk_level: RiskLevel,
    pub matched_pattern: Option<String>,
    pub reason: String,
    pub auto_approved: bool,
}
```

## Examples

### Safe Commands (Auto-Approved)

```rust
let result = validator.validate("ls -la");
assert_eq!(result.risk_level, RiskLevel::Low);
assert!(result.auto_approved);

let result = validator.validate("git status");
assert_eq!(result.risk_level, RiskLevel::Low);
assert!(result.auto_approved);
```

### Medium Risk Commands (Auto-Approved)

```rust
let result = validator.validate("git push --force");
assert_eq!(result.risk_level, RiskLevel::Medium);
assert!(result.auto_approved);

let result = validator.validate("npm publish");
assert_eq!(result.risk_level, RiskLevel::Medium);
assert!(result.auto_approved);
```

### High Risk Commands (Requires Confirmation)

```rust
let result = validator.validate("rm -rf ./node_modules");
assert_eq!(result.risk_level, RiskLevel::High);
assert!(!result.auto_approved);

let result = validator.validate("sudo apt update");
assert_eq!(result.risk_level, RiskLevel::High);
assert!(!result.auto_approved);
```

### Critical Commands (Never Auto-Approve)

```rust
let result = validator.validate("rm -rf /");
assert_eq!(result.risk_level, RiskLevel::Critical);
assert!(!result.auto_approved);

let result = validator.validate("mkfs.ext4 /dev/sda1");
assert_eq!(result.risk_level, RiskLevel::Critical);
assert!(!result.auto_approved);
```

## Integration

### Terminal UI Integration

```rust
use agterm::command_validator::get_validator;

fn execute_ai_command(command: &str) -> Result<(), String> {
    let validator = get_validator();
    let result = validator.validate(command);

    if result.auto_approved {
        // Execute directly
        execute_command(command)?;
    } else {
        // Show confirmation dialog
        show_confirmation_dialog(
            command,
            &result.risk_level,
            &result.reason,
        )?;
    }

    Ok(())
}
```

### Logging

```rust
use tracing::warn;

let result = validator.validate(command);

match result.risk_level {
    RiskLevel::High | RiskLevel::Critical => {
        warn!(
            command = %command,
            risk_level = ?result.risk_level,
            reason = %result.reason,
            "High-risk command detected"
        );
    }
    _ => {}
}
```

## Testing

Run the comprehensive test suite:

```bash
cargo test command_validator
```

Run the interactive demo:

```bash
cargo run --example command_validator_demo
```

## Design Principles

1. **Security First**: Critical patterns are checked before whitelist
2. **Fail Safe**: Unknown commands default to Medium risk
3. **Transparency**: All matches include the pattern and reason
4. **Composability**: Easy to extend with new patterns
5. **Performance**: Patterns compiled once at initialization

## Future Enhancements

Potential improvements:

- User-configurable patterns
- Command history tracking
- Machine learning risk scoring
- Context-aware validation (e.g., current directory)
- Integration with shell-specific features
- Audit logging for high-risk commands
