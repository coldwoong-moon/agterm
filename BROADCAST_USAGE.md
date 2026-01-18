# Terminal Broadcast Usage Guide

The broadcast module provides functionality to send input to multiple terminal sessions simultaneously. This is particularly useful when managing multiple servers or executing commands across several terminals at once.

## Overview

The broadcast system consists of three main components:

1. **BroadcastGroup** - A named group of terminal sessions that can receive broadcast input
2. **BroadcastManager** - Manages all groups and routing of broadcast input
3. **BroadcastMode** - Controls how input is broadcast (Full, Selective, or Disabled)

## Basic Usage

### Creating a Broadcast Group

```rust
use agterm::broadcast::{BroadcastManager, BroadcastMode};
use uuid::Uuid;

let mut manager = BroadcastManager::new();

// Create a group for production servers
manager.create_group("production".to_string()).unwrap();
```

### Adding Terminals to a Group

```rust
// Register terminals with the manager
let terminal1 = Uuid::new_v4();
let terminal2 = Uuid::new_v4();
let terminal3 = Uuid::new_v4();

manager.register_terminal(terminal1);
manager.register_terminal(terminal2);
manager.register_terminal(terminal3);

// Add terminals to the production group
manager.add_to_group("production", terminal1).unwrap();
manager.add_to_group("production", terminal2).unwrap();
manager.add_to_group("production", terminal3).unwrap();
```

### Activating a Group

```rust
// Activate the production group for broadcasting
manager.activate_group("production").unwrap();
```

### Broadcasting Input

```rust
// Get broadcast targets for input from terminal1
// In Full mode, all other terminals in the group receive the input
let targets = manager.get_broadcast_targets(
    &terminal1,
    false, // ctrl
    false, // alt
    false, // shift
    false  // command
);

if let Some(targets) = targets {
    // Send input to all targets
    for target_id in targets {
        // Send the input to each target terminal
        // (Implementation depends on your PTY manager)
    }
}
```

## Broadcast Modes

### Full Mode (Default)

All input is broadcast to all terminals in the active group.

```rust
let group = manager.get_group_mut("production").unwrap();
group.set_mode(BroadcastMode::Full);
```

**Use case:** When you need to execute the same commands on all servers simultaneously.

### Selective Mode

Input is only broadcast when specific key modifiers are pressed (default: Ctrl+Shift).

```rust
use agterm::broadcast::BroadcastTrigger;

let group = manager.get_group_mut("production").unwrap();
group.set_mode(BroadcastMode::Selective);

// Customize the trigger (e.g., Ctrl+Alt+Shift)
let trigger = BroadcastTrigger {
    ctrl: true,
    alt: true,
    shift: true,
    command: false,
};
group.set_trigger(trigger);
```

**Use case:** When you want to type normally but occasionally broadcast specific commands.

### Disabled Mode

Broadcasting is disabled for this group.

```rust
let group = manager.get_group_mut("production").unwrap();
group.set_mode(BroadcastMode::Disabled);
```

## Advanced Features

### Multiple Groups

You can create multiple groups for different purposes:

```rust
manager.create_group("production".to_string()).unwrap();
manager.create_group("staging".to_string()).unwrap();
manager.create_group("development".to_string()).unwrap();

// Terminals can belong to multiple groups
manager.add_to_group("production", server1).unwrap();
manager.add_to_group("staging", server1).unwrap();
manager.add_to_group("staging", server2).unwrap();
```

### Group Descriptions

Add descriptions to help identify group purposes:

```rust
let group = manager.get_group_mut("production").unwrap();
group.set_description(Some("All production web servers".to_string()));
```

### Finding Terminal Groups

Discover which groups a terminal belongs to:

```rust
let groups = manager.find_groups_for_terminal(&terminal_id);
for group_name in groups {
    println!("Terminal is in group: {}", group_name);
}
```

### Statistics

Get overview statistics about broadcast usage:

```rust
let stats = manager.stats();
println!("Total groups: {}", stats.total_groups);
println!("Active groups: {}", stats.active_groups);
println!("Total terminals: {}", stats.total_terminals);
println!("Terminals in groups: {}", stats.terminals_in_groups);
```

## Integration Example

Here's a complete example of integrating broadcast into a terminal application:

```rust
use agterm::broadcast::{BroadcastManager, BroadcastMode, BroadcastError};
use uuid::Uuid;

struct TerminalApp {
    broadcast_manager: BroadcastManager,
    terminals: Vec<Uuid>,
}

impl TerminalApp {
    fn new() -> Self {
        Self {
            broadcast_manager: BroadcastManager::new(),
            terminals: Vec::new(),
        }
    }

    fn add_terminal(&mut self) -> Uuid {
        let id = Uuid::new_v4();
        self.broadcast_manager.register_terminal(id);
        self.terminals.push(id);
        id
    }

    fn setup_server_group(&mut self, name: &str, terminals: &[Uuid]) -> Result<(), BroadcastError> {
        self.broadcast_manager.create_group(name.to_string())?;

        for &terminal_id in terminals {
            self.broadcast_manager.add_to_group(name, terminal_id)?;
        }

        Ok(())
    }

    fn handle_input(&mut self, terminal_id: &Uuid, input: &str, ctrl: bool, alt: bool, shift: bool, cmd: bool) {
        // Check if we should broadcast this input
        if let Some(targets) = self.broadcast_manager.get_broadcast_targets(
            terminal_id,
            ctrl,
            alt,
            shift,
            cmd
        ) {
            // Broadcast to all targets
            for target_id in targets {
                self.send_to_terminal(&target_id, input);
            }
        }

        // Always send to the originating terminal
        self.send_to_terminal(terminal_id, input);
    }

    fn send_to_terminal(&self, terminal_id: &Uuid, input: &str) {
        // Your PTY write implementation here
        println!("Sending to {}: {}", terminal_id, input);
    }
}

fn main() {
    let mut app = TerminalApp::new();

    // Create three server terminals
    let server1 = app.add_terminal();
    let server2 = app.add_terminal();
    let server3 = app.add_terminal();

    // Setup broadcast group
    app.setup_server_group("prod-servers", &[server1, server2, server3]).unwrap();
    app.broadcast_manager.activate_group("prod-servers").unwrap();

    // Simulate input
    app.handle_input(&server1, "ls -la\n", false, false, false, false);
    // This input goes to server1, server2, and server3
}
```

## Safety Considerations

1. **Last Terminal Protection**: You cannot remove the last terminal from an active broadcast group. Deactivate the group first.

2. **Terminal Validation**: The manager validates that terminals are registered before adding them to groups.

3. **Source Exclusion**: The source terminal is automatically excluded from broadcast targets to prevent duplicate input.

## Common Patterns

### Temporary Broadcast

```rust
// Enable broadcasting temporarily
manager.activate_group("servers").unwrap();
// ... perform operations ...
manager.deactivate_current().unwrap();
```

### Per-Command Broadcast

Use Selective mode for broadcasting specific commands:

```rust
let group = manager.get_group_mut("servers").unwrap();
group.set_mode(BroadcastMode::Selective);

// Now only Ctrl+Shift+<key> combinations are broadcast
// Regular typing stays local to each terminal
```

### Dynamic Group Management

```rust
// Create groups on-the-fly based on user needs
let group_name = format!("session-{}", session_id);
manager.create_group(group_name.clone()).unwrap();

// Add relevant terminals
for terminal in relevant_terminals {
    manager.add_to_group(&group_name, terminal).unwrap();
}

// Activate for this session
manager.activate_group(&group_name).unwrap();

// Clean up when done
manager.deactivate_current().unwrap();
manager.delete_group(&group_name).unwrap();
```

## Error Handling

All operations return `Result<T, BroadcastError>`. Common errors:

- `GroupNotFound`: The specified group doesn't exist
- `TerminalNotFound`: The terminal isn't registered
- `GroupAlreadyExists`: Attempting to create a duplicate group
- `TerminalAlreadyInGroup`: Terminal is already a member
- `CannotRemoveLastTerminal`: Cannot remove the last terminal from an active group
- `InvalidGroupName`: Group name is empty or too long (max 64 characters)

```rust
match manager.activate_group("nonexistent") {
    Ok(_) => println!("Group activated"),
    Err(BroadcastError::GroupNotFound(name)) => {
        println!("Group '{}' not found", name);
    }
    Err(e) => println!("Error: {}", e),
}
```

## Serialization

Both `BroadcastManager` and `BroadcastGroup` support serde serialization, allowing you to save and restore broadcast configurations:

```rust
use std::fs;

// Save configuration
let json = serde_json::to_string_pretty(&manager)?;
fs::write("broadcast_config.json", json)?;

// Load configuration
let json = fs::read_to_string("broadcast_config.json")?;
let manager: BroadcastManager = serde_json::from_str(&json)?;
```

## Performance Notes

- Group membership checks use `HashSet` for O(1) lookups
- Broadcasting to N terminals has O(N) complexity
- Memory usage scales linearly with the number of groups and terminals
- All operations are synchronous and non-blocking

## Future Enhancements

Potential features that could be added:

- Regex-based input filtering (broadcast only certain commands)
- Time-based broadcast scheduling
- Recording of broadcast sessions
- Broadcast history and replay
- Terminal-specific broadcast overrides
- Conditional broadcasting based on terminal state
