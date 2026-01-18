//! Integration tests for the macro system

use agterm::macros::*;
use agterm::macros::builders::*;

#[test]
fn test_macro_system_basic() {
    let mut engine = MacroEngine::new();

    // Create a simple macro
    let macro_def = Macro::new("hello".to_string(), "Say hello".to_string())
        .add_action(MacroAction::SendText("hello world".to_string()));

    // Register it
    assert!(engine.register(macro_def).is_ok());

    // Execute it
    let actions = engine.execute("hello").unwrap();
    assert_eq!(actions.len(), 1);
    assert!(matches!(actions[0], MacroAction::SendText(ref s) if s == "hello world"));
}

#[test]
fn test_macro_with_trigger() {
    let mut engine = MacroEngine::new();

    let trigger = KeyCombo {
        key: "h".to_string(),
        modifiers: KeyModifiers::ctrl_alt(),
    };

    let macro_def = Macro::new("hello".to_string(), "Say hello".to_string())
        .with_trigger(trigger.clone())
        .add_action(MacroAction::SendText("hello".to_string()));

    engine.register(macro_def).unwrap();

    // Check trigger matching
    assert_eq!(engine.match_trigger(&trigger), Some("hello"));
}

#[test]
fn test_macro_recording() {
    let mut engine = MacroEngine::new();

    // Start recording
    engine.start_recording("recorded".to_string(), false).unwrap();
    assert!(engine.is_recording());

    // Record some actions
    engine.record_text("ls -la".to_string());
    engine.record_text("\n".to_string());

    // Stop recording
    let macro_def = engine
        .stop_recording("Recorded macro".to_string(), None)
        .unwrap();

    assert_eq!(macro_def.name, "recorded");
    assert_eq!(macro_def.actions.len(), 2);

    // Register and execute
    engine.register(macro_def).unwrap();
    let actions = engine.execute("recorded").unwrap();
    assert_eq!(actions.len(), 2);
}

#[test]
fn test_macro_builders() {
    let mut engine = MacroEngine::new();

    let macro_def = Macro::new("complex".to_string(), "Complex macro".to_string())
        .add_action(send_line("git status"))
        .add_action(wait_ms(500))
        .add_action(send_line("git diff"))
        .add_action(repeat(send_text("x"), 3));

    engine.register(macro_def).unwrap();

    let actions = engine.execute("complex").unwrap();
    assert!(actions.len() >= 4); // 3 individual x's + other actions
}

#[test]
fn test_macro_call() {
    let mut engine = MacroEngine::new();

    // First macro
    let macro1 = Macro::new("base".to_string(), "Base macro".to_string())
        .add_action(send_text("base"));

    // Second macro that calls the first
    let macro2 = Macro::new("caller".to_string(), "Caller macro".to_string())
        .add_action(call_macro("base"))
        .add_action(send_text(" extended"));

    engine.register(macro1).unwrap();
    engine.register(macro2).unwrap();

    let actions = engine.execute("caller").unwrap();
    assert_eq!(actions.len(), 2);
}

#[test]
fn test_macro_sequence() {
    let mut engine = MacroEngine::new();

    let macro_def = Macro::new("seq".to_string(), "Sequence macro".to_string())
        .add_action(sequence(vec![
            send_text("a"),
            send_text("b"),
            send_text("c"),
        ]));

    engine.register(macro_def).unwrap();

    let actions = engine.execute("seq").unwrap();
    assert_eq!(actions.len(), 3);
}

#[test]
fn test_macro_export_import() {
    let mut engine1 = MacroEngine::new();

    let macro1 = Macro::new("test1".to_string(), "Test 1".to_string())
        .add_action(send_text("hello"));

    let macro2 = Macro::new("test2".to_string(), "Test 2".to_string())
        .add_action(send_text("world"));

    engine1.register(macro1).unwrap();
    engine1.register(macro2).unwrap();

    // Export
    let exported = engine1.export_all();
    assert_eq!(exported.len(), 2);

    // Import to new engine
    let mut engine2 = MacroEngine::new();
    engine2.load_from_config(exported).unwrap();

    assert!(engine2.get("test1").is_some());
    assert!(engine2.get("test2").is_some());
}

#[test]
fn test_disabled_macro() {
    let mut engine = MacroEngine::new();

    let macro_def = Macro::new("disabled".to_string(), "Disabled macro".to_string())
        .add_action(send_text("should not execute"))
        .set_enabled(false);

    engine.register(macro_def).unwrap();

    let actions = engine.execute("disabled").unwrap();
    assert_eq!(actions.len(), 0); // Disabled macro returns no actions
}

#[test]
fn test_recursion_protection() {
    let mut engine = MacroEngine::new();
    engine.set_max_recursion_depth(3);

    let macro_def = Macro::new("recursive".to_string(), "Recursive macro".to_string())
        .add_action(call_macro("recursive"));

    engine.register(macro_def).unwrap();

    let result = engine.execute("recursive");
    assert!(matches!(result, Err(MacroError::MaxRecursionDepth)));
}

#[test]
fn test_key_event_creation() {
    let key_event = KeyEvent::new(
        "c".to_string(),
        KeyModifiers::ctrl(),
    );

    assert_eq!(key_event.key, "c");
    assert!(key_event.modifiers.ctrl);
    assert!(!key_event.modifiers.alt);
}

#[test]
fn test_duration_conversion() {
    let duration = std::time::Duration::from_millis(500);
    let duration_ms: DurationMs = duration.into();
    assert_eq!(duration_ms.0, 500);

    let back: std::time::Duration = duration_ms.into();
    assert_eq!(back.as_millis(), 500);
}
