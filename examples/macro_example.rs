//! Example demonstrating the macro system
//!
//! Run with: cargo run --example macro_example

use agterm::macros::*;
use agterm::macros::builders::*;

fn main() {
    println!("=== AgTerm Macro System Example ===\n");

    // Create a macro engine
    let mut engine = MacroEngine::new();

    // Example 1: Simple text macro
    println!("1. Creating a simple text macro...");
    let simple_macro = Macro::new("hello".to_string(), "Send hello message".to_string())
        .add_action(send_line("echo 'Hello from AgTerm!'"));

    engine.register(simple_macro).unwrap();
    println!("   Registered 'hello' macro");

    // Example 2: Macro with key trigger
    println!("\n2. Creating a macro with keyboard trigger...");
    let trigger = KeyCombo {
        key: "h".to_string(),
        modifiers: KeyModifiers::ctrl_alt(),
    };

    let triggered_macro = Macro::new("git_status".to_string(), "Quick git status".to_string())
        .with_trigger(trigger.clone())
        .add_action(send_line("git status"))
        .add_action(wait_ms(500))
        .add_action(send_line("git diff --stat"));

    engine.register(triggered_macro).unwrap();
    println!("   Registered 'git_status' macro with Ctrl+Alt+H trigger");

    // Example 3: Complex macro with repetition
    println!("\n3. Creating a macro with repetition...");
    let repeat_macro = Macro::new("draw_line".to_string(), "Draw a line".to_string())
        .add_action(repeat(send_text("-"), 40))
        .add_action(send_enter());

    engine.register(repeat_macro).unwrap();
    println!("   Registered 'draw_line' macro");

    // Example 4: Macro calling another macro
    println!("\n4. Creating a macro that calls other macros...");
    let composite_macro = Macro::new("status_report".to_string(), "Full status report".to_string())
        .add_action(call_macro("draw_line"))
        .add_action(send_line("echo 'Status Report'"))
        .add_action(call_macro("draw_line"))
        .add_action(call_macro("git_status"));

    engine.register(composite_macro).unwrap();
    println!("   Registered 'status_report' macro");

    // Example 5: Recording a macro
    println!("\n5. Demonstrating macro recording...");
    engine.start_recording("recorded_test".to_string(), false).unwrap();
    engine.record_text("cd /tmp".to_string());
    engine.record_text("\n".to_string());
    engine.record_text("ls -la".to_string());
    engine.record_text("\n".to_string());

    let recorded = engine.stop_recording("Recorded navigation".to_string(), None).unwrap();
    engine.register(recorded).unwrap();
    println!("   Recorded and registered 'recorded_test' macro");

    // List all macros
    println!("\n6. Listing all registered macros:");
    let macros = engine.list();
    for macro_name in &macros {
        if let Some(m) = engine.get(macro_name) {
            println!("   - {}: {}", m.name, m.description);
            if let Some(ref trigger) = m.trigger {
                println!("     Trigger: {} (Ctrl:{} Alt:{} Shift:{} Cmd:{})",
                    trigger.key,
                    trigger.modifiers.ctrl,
                    trigger.modifiers.alt,
                    trigger.modifiers.shift,
                    trigger.modifiers.super_
                );
            }
        }
    }

    // Execute a macro
    println!("\n7. Executing 'status_report' macro:");
    match engine.execute("status_report") {
        Ok(actions) => {
            println!("   Macro expanded to {} actions:", actions.len());
            for (i, action) in actions.iter().enumerate() {
                match action {
                    MacroAction::SendText(text) => {
                        println!("     {}. Send text: {:?}", i + 1, text.chars().take(50).collect::<String>());
                    }
                    MacroAction::SendKeys(keys) => {
                        println!("     {}. Send {} key(s)", i + 1, keys.len());
                    }
                    MacroAction::Wait(duration) => {
                        println!("     {}. Wait {} ms", i + 1, duration.0);
                    }
                    _ => {
                        println!("     {}. {:?}", i + 1, action);
                    }
                }
            }
        }
        Err(e) => {
            println!("   Error executing macro: {}", e);
        }
    }

    // Export macros
    println!("\n8. Exporting all macros...");
    let exported = engine.export_all();
    println!("   Exported {} macros", exported.len());

    // Create a new engine and import
    println!("\n9. Creating new engine and importing macros...");
    let mut new_engine = MacroEngine::new();
    new_engine.load_from_config(exported).unwrap();
    println!("   Imported {} macros", new_engine.list().len());

    // Test trigger matching
    println!("\n10. Testing trigger matching...");
    if let Some(macro_name) = new_engine.match_trigger(&trigger) {
        println!("   Ctrl+Alt+H triggers: '{}'", macro_name);
    }

    println!("\n=== Example Complete ===");
}
