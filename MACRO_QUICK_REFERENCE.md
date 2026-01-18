# AgTerm Macro System - Quick Reference

## Import
```rust
use agterm::macros::*;
use agterm::macros::builders::*;
```

## Create Engine
```rust
let mut engine = MacroEngine::new();
```

## Define Macro

### Basic
```rust
let m = Macro::new("name".to_string(), "description".to_string())
    .add_action(send_line("command"));
```

### With Trigger
```rust
let m = Macro::new("name".to_string(), "description".to_string())
    .with_trigger(KeyCombo {
        key: "g".to_string(),
        modifiers: KeyModifiers::ctrl_alt(),
    })
    .add_action(send_line("command"));
```

## Register & Execute
```rust
engine.register(macro_def)?;
let actions = engine.execute("name")?;
```

## Recording
```rust
// Start
engine.start_recording("name".to_string(), false)?;

// Record
engine.record_text("text".to_string());

// Stop
let m = engine.stop_recording("description".to_string(), None)?;
engine.register(m)?;
```

## Actions

### Text
```rust
send_line("text")          // Text + Enter
send_text("text")          // Text only
send_enter()               // Enter key
send_escape()              // Escape key
send_ctrl_c()              // Ctrl+C
```

### Timing
```rust
wait_ms(500)               // Wait 500ms
wait_secs(2)               // Wait 2 seconds
```

### Control Flow
```rust
repeat(action, 5)          // Repeat 5 times
sequence(vec![a1, a2])     // Sequential
call_macro("other")        // Call macro
```

### Shell
```rust
run_command("ls -la")      // Execute command
```

## Trigger Matching
```rust
if let Some(name) = engine.match_trigger(&combo) {
    engine.execute(name)?;
}
```

## Export/Import
```rust
// Export
let macros = engine.export_all();
let json = serde_json::to_string(&macros)?;

// Import
let macros: Vec<Macro> = serde_json::from_str(&json)?;
engine.load_from_config(macros)?;
```

## Error Handling
```rust
match engine.execute("name") {
    Ok(actions) => { /* ... */ }
    Err(MacroError::NotFound(n)) => { /* ... */ }
    Err(MacroError::MaxRecursionDepth) => { /* ... */ }
    Err(e) => { /* ... */ }
}
```

## KeyModifiers
```rust
KeyModifiers::none()       // No modifiers
KeyModifiers::ctrl()       // Ctrl
KeyModifiers::alt()        // Alt
KeyModifiers::shift()      // Shift
KeyModifiers::cmd()        // Cmd/Super
KeyModifiers::ctrl_alt()   // Ctrl+Alt
```

## Management
```rust
engine.get("name")             // Get macro
engine.get_mut("name")         // Get mutable
engine.unregister("name")?     // Remove
engine.list()                  // List all names
engine.clear()                 // Remove all
```

## Configuration
```rust
engine.set_max_recursion_depth(10);
```

## Complete Example
```rust
use agterm::macros::*;
use agterm::macros::builders::*;

let mut engine = MacroEngine::new();

let m = Macro::new("hello".to_string(), "Say hello".to_string())
    .with_trigger(KeyCombo {
        key: "h".to_string(),
        modifiers: KeyModifiers::ctrl_alt(),
    })
    .add_action(send_line("echo 'Hello, World!'"))
    .add_action(wait_ms(500))
    .add_action(send_line("date"));

engine.register(m)?;

// Execute by name
engine.execute("hello")?;

// Or by trigger
let combo = KeyCombo {
    key: "h".to_string(),
    modifiers: KeyModifiers::ctrl_alt(),
};
if let Some(name) = engine.match_trigger(&combo) {
    engine.execute(name)?;
}
```

## Testing
```bash
# Run tests
cargo test macros

# Run example
cargo run --example macro_example
```

## Documentation
- Full guide: `MACROS.md`
- Implementation details: `MACRO_IMPLEMENTATION_SUMMARY.md`
- Delivery summary: `MACRO_SYSTEM_DELIVERY.md`
- API docs: `cargo doc --open`
