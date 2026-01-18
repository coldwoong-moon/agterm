# Broadcast Quick Start Guide

## 5-Minute Tutorial

### Step 1: Create a Manager

```rust
use agterm::broadcast::BroadcastManager;

let mut manager = BroadcastManager::new();
```

### Step 2: Register Terminals

```rust
use uuid::Uuid;

let server1 = Uuid::new_v4();
let server2 = Uuid::new_v4();
let server3 = Uuid::new_v4();

manager.register_terminal(server1);
manager.register_terminal(server2);
manager.register_terminal(server3);
```

### Step 3: Create a Group

```rust
manager.create_group("production".to_string()).unwrap();

manager.add_to_group("production", server1).unwrap();
manager.add_to_group("production", server2).unwrap();
manager.add_to_group("production", server3).unwrap();
```

### Step 4: Activate Broadcasting

```rust
manager.activate_group("production").unwrap();
```

### Step 5: Broadcast Input

```rust
// User types in server1
let input = "ls -la\n";

// Get broadcast targets
if let Some(targets) = manager.get_broadcast_targets(
    &server1,
    false, // ctrl
    false, // alt
    false, // shift
    false  // command
) {
    // Send to all targets (server2 and server3)
    for target_id in targets {
        // Your PTY write implementation
        println!("Sending to {}: {}", target_id, input);
    }
}

// Send to source terminal too
println!("Sending to {}: {}", server1, input);
```

## Common Commands

### Create Group
```rust
manager.create_group("group-name".to_string())?;
```

### Add Terminal
```rust
manager.add_to_group("group-name", terminal_id)?;
```

### Activate Group
```rust
manager.activate_group("group-name")?;
```

### Change Mode
```rust
use agterm::broadcast::BroadcastMode;

let group = manager.get_group_mut("group-name")?;
group.set_mode(BroadcastMode::Selective);
```

### Deactivate
```rust
manager.deactivate_current()?;
```

### Remove Terminal
```rust
manager.remove_from_group("group-name", &terminal_id)?;
```

### Delete Group
```rust
manager.delete_group("group-name")?;
```

## Broadcast Modes Cheat Sheet

| Mode | Behavior | Use Case |
|------|----------|----------|
| `Full` | All input broadcasts | Execute same commands everywhere |
| `Selective` | Only Ctrl+Shift+<key> broadcasts | Mixed local/broadcast usage |
| `Disabled` | No broadcasting | Group exists but inactive |

## Error Handling Pattern

```rust
match manager.activate_group("servers") {
    Ok(_) => println!("Broadcasting to servers"),
    Err(BroadcastError::GroupNotFound(name)) => {
        println!("Group '{}' doesn't exist", name);
    }
    Err(e) => println!("Error: {}", e),
}
```

## Complete Example

```rust
use agterm::broadcast::{BroadcastManager, BroadcastMode};
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = BroadcastManager::new();

    // Setup terminals
    let term1 = Uuid::new_v4();
    let term2 = Uuid::new_v4();

    manager.register_terminal(term1);
    manager.register_terminal(term2);

    // Create and configure group
    manager.create_group("demo".to_string())?;
    manager.add_to_group("demo", term1)?;
    manager.add_to_group("demo", term2)?;

    // Activate
    manager.activate_group("demo")?;

    // Simulate input
    let input = "echo 'hello'";

    if let Some(targets) = manager.get_broadcast_targets(
        &term1, false, false, false, false
    ) {
        println!("Broadcasting '{}' to {} terminals", input, targets.len());
        for target in targets {
            println!("  -> {}", target);
        }
    }

    Ok(())
}
```

## Run the Demo

```bash
cargo run --example broadcast_demo
```

## For More Information

- **Implementation Details:** See `BROADCAST_IMPLEMENTATION.md`
- **User Guide:** See `BROADCAST_USAGE.md`
- **Full Summary:** See `BROADCAST_SUMMARY.md`
- **Example Code:** See `examples/broadcast_demo.rs`
- **Tests:** See `tests/broadcast_integration.rs`
