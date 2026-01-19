//! Comprehensive tests for the automation module

use agterm::automation::*;
use agterm::terminal::pty::PtyManager;
use regex::Regex;
use std::time::Duration;

// Helper function to create a test engine
fn create_test_engine() -> AutomationEngine {
    let manager = PtyManager::new();
    let pty_id = manager
        .create_session(40, 120)
        .expect("Failed to create PTY session");
    AutomationEngine::new(manager, pty_id)
}

#[test]
fn test_key_conversions() {
    // Test basic keys
    assert_eq!(Key::Enter.to_bytes(), vec![b'\r']);
    assert_eq!(Key::Tab.to_bytes(), vec![b'\t']);
    assert_eq!(Key::Backspace.to_bytes(), vec![0x7F]);
    assert_eq!(Key::Escape.to_bytes(), vec![0x1B]);

    // Test arrow keys
    assert_eq!(Key::Up.to_bytes(), b"\x1B[A");
    assert_eq!(Key::Down.to_bytes(), b"\x1B[B");
    assert_eq!(Key::Right.to_bytes(), b"\x1B[C");
    assert_eq!(Key::Left.to_bytes(), b"\x1B[D");

    // Test function keys
    assert_eq!(Key::F(1).to_bytes(), b"\x1BOP");
    assert_eq!(Key::F(5).to_bytes(), b"\x1B[15~");

    // Test control keys
    assert_eq!(Key::Ctrl('c').to_bytes(), vec![3]);
    assert_eq!(Key::Ctrl('d').to_bytes(), vec![4]);
    assert_eq!(Key::Ctrl('z').to_bytes(), vec![26]);

    // Test alt keys
    assert_eq!(Key::Alt('a').to_bytes(), b"\x1Ba");
}

#[test]
fn test_key_parsing() {
    // Basic keys
    assert_eq!(Key::parse_str("ENTER"), Some(Key::Enter));
    assert_eq!(Key::parse_str("Tab"), Some(Key::Tab));
    assert_eq!(Key::parse_str("ESCAPE"), Some(Key::Escape));
    assert_eq!(Key::parse_str("ESC"), Some(Key::Escape));

    // Arrow keys
    assert_eq!(Key::parse_str("UP"), Some(Key::Up));
    assert_eq!(Key::parse_str("down"), Some(Key::Down));
    assert_eq!(Key::parse_str("LEFT"), Some(Key::Left));
    assert_eq!(Key::parse_str("RIGHT"), Some(Key::Right));

    // Function keys
    assert_eq!(Key::parse_str("F1"), Some(Key::F(1)));
    assert_eq!(Key::parse_str("F12"), Some(Key::F(12)));
    assert_eq!(Key::parse_str("F99"), None); // Out of range

    // Modified keys
    assert_eq!(Key::parse_str("CTRL+C"), Some(Key::Ctrl('C')));
    assert_eq!(Key::parse_str("ALT+A"), Some(Key::Alt('A')));

    // Characters
    assert_eq!(Key::parse_str("a"), Some(Key::Char('a')));
    assert_eq!(Key::parse_str("5"), Some(Key::Char('5')));

    // Invalid
    assert_eq!(Key::parse_str("INVALID"), None);
    assert_eq!(Key::parse_str(""), None);
}

#[test]
fn test_pattern_exact_matching() {
    let pattern = Pattern::Exact("hello".to_string());

    assert!(pattern.matches("hello"));
    assert!(pattern.matches("hello world"));
    assert!(pattern.matches("say hello there"));
    assert!(!pattern.matches("goodbye"));
    assert!(!pattern.matches("helo"));
}

#[test]
fn test_pattern_regex_matching() {
    let pattern = Pattern::Regex(Regex::new(r"\d{3}-\d{4}").unwrap());

    assert!(pattern.matches("Call 555-1234 now"));
    assert!(pattern.matches("Phone: 123-4567"));
    assert!(!pattern.matches("No phone here"));
    assert!(!pattern.matches("123-456")); // Wrong format
}

#[test]
fn test_pattern_any_of() {
    let pattern = Pattern::AnyOf(vec![
        Pattern::Exact("success".to_string()),
        Pattern::Exact("completed".to_string()),
        Pattern::Exact("done".to_string()),
    ]);

    assert!(pattern.matches("success"));
    assert!(pattern.matches("Task completed"));
    assert!(pattern.matches("Job done!"));
    assert!(!pattern.matches("failed"));
}

#[test]
fn test_pattern_all_of() {
    let pattern = Pattern::AllOf(vec![
        Pattern::Exact("build".to_string()),
        Pattern::Exact("successful".to_string()),
    ]);

    assert!(pattern.matches("build successful"));
    assert!(pattern.matches("successful build process"));
    assert!(!pattern.matches("build failed"));
    assert!(!pattern.matches("successful test"));
}

#[test]
fn test_pattern_extract() {
    // Exact pattern
    let pattern = Pattern::Exact("hello".to_string());
    assert_eq!(pattern.extract("hello world"), Some("hello".to_string()));
    assert_eq!(pattern.extract("goodbye"), None);

    // Regex pattern
    let pattern = Pattern::Regex(Regex::new(r"\d{3}-\d{4}").unwrap());
    assert_eq!(
        pattern.extract("Phone: 555-1234"),
        Some("555-1234".to_string())
    );
    assert_eq!(pattern.extract("No phone"), None);
}

#[test]
fn test_execution_context_variable_expansion() {
    let mut context = ExecutionContext::default();
    context
        .variables
        .insert("USER".to_string(), "alice".to_string());
    context
        .variables
        .insert("AGE".to_string(), "30".to_string());
    context
        .variables
        .insert("CITY".to_string(), "NYC".to_string());

    // Test ${VAR} format
    assert_eq!(
        context.expand_variables("Hello ${USER}"),
        "Hello alice"
    );

    // Test $VAR format
    assert_eq!(
        context.expand_variables("User $USER is $AGE years old"),
        "User alice is 30 years old"
    );

    // Test multiple variables
    assert_eq!(
        context.expand_variables("${USER} from ${CITY} is ${AGE}"),
        "alice from NYC is 30"
    );

    // Test non-existent variable (should remain unchanged)
    assert_eq!(
        context.expand_variables("Hello ${NONEXISTENT}"),
        "Hello ${NONEXISTENT}"
    );
}

#[test]
fn test_execution_context_env_expansion() {
    std::env::set_var("TEST_AUTOMATION_VAR", "test_value");

    let context = ExecutionContext::default();

    assert_eq!(
        context.expand_variables("Value: ${ENV:TEST_AUTOMATION_VAR}"),
        "Value: test_value"
    );

    // Non-existent env var should remain unchanged
    assert!(context
        .expand_variables("${ENV:NONEXISTENT_VAR}")
        .contains("NONEXISTENT_VAR"));
}

#[test]
fn test_execution_context_buffer_management() {
    let mut context = ExecutionContext::default();
    context.max_buffer_size = 100; // Small buffer for testing

    // Add output that fits
    context.append_output("Hello ");
    context.append_output("World");
    assert_eq!(context.output_buffer, "Hello World");

    // Add output that exceeds buffer
    let large_text = "x".repeat(200);
    context.append_output(&large_text);
    assert!(context.output_buffer.len() <= context.max_buffer_size);
}

#[test]
fn test_condition_var_equals() {
    let mut context = ExecutionContext::default();
    context
        .variables
        .insert("STATUS".to_string(), "ok".to_string());
    context
        .variables
        .insert("COUNT".to_string(), "42".to_string());

    let cond = Condition::VarEquals("STATUS".to_string(), "ok".to_string());
    assert!(cond.evaluate(&context));

    let cond = Condition::VarEquals("STATUS".to_string(), "error".to_string());
    assert!(!cond.evaluate(&context));

    let cond = Condition::VarEquals("COUNT".to_string(), "42".to_string());
    assert!(cond.evaluate(&context));

    let cond = Condition::VarEquals("NONEXISTENT".to_string(), "value".to_string());
    assert!(!cond.evaluate(&context));
}

#[test]
fn test_condition_var_contains() {
    let mut context = ExecutionContext::default();
    context.variables.insert(
        "OUTPUT".to_string(),
        "Error: file not found".to_string(),
    );

    let cond = Condition::VarContains("OUTPUT".to_string(), "Error".to_string());
    assert!(cond.evaluate(&context));

    let cond = Condition::VarContains("OUTPUT".to_string(), "file".to_string());
    assert!(cond.evaluate(&context));

    let cond = Condition::VarContains("OUTPUT".to_string(), "Success".to_string());
    assert!(!cond.evaluate(&context));
}

#[test]
fn test_condition_var_matches() {
    let mut context = ExecutionContext::default();
    context
        .variables
        .insert("VERSION".to_string(), "1.2.3".to_string());

    let cond = Condition::VarMatches(
        "VERSION".to_string(),
        Regex::new(r"\d+\.\d+\.\d+").unwrap(),
    );
    assert!(cond.evaluate(&context));

    let cond = Condition::VarMatches("VERSION".to_string(), Regex::new(r"^2\.").unwrap());
    assert!(!cond.evaluate(&context));
}

#[test]
fn test_condition_env_exists() {
    std::env::set_var("TEST_ENV_VAR", "value");

    let context = ExecutionContext::default();

    let cond = Condition::EnvExists("TEST_ENV_VAR".to_string());
    assert!(cond.evaluate(&context));

    let cond = Condition::EnvExists("NONEXISTENT_ENV_VAR".to_string());
    assert!(!cond.evaluate(&context));
}

#[test]
fn test_condition_pattern_matches() {
    let mut context = ExecutionContext::default();
    context.last_capture = Some("Build successful".to_string());

    let cond = Condition::PatternMatches(Pattern::Exact("successful".to_string()));
    assert!(cond.evaluate(&context));

    let cond = Condition::PatternMatches(Pattern::Exact("failed".to_string()));
    assert!(!cond.evaluate(&context));

    // No capture
    context.last_capture = None;
    let cond = Condition::PatternMatches(Pattern::Exact("anything".to_string()));
    assert!(!cond.evaluate(&context));
}

#[test]
fn test_condition_logical_and() {
    let mut context = ExecutionContext::default();
    context
        .variables
        .insert("A".to_string(), "1".to_string());
    context
        .variables
        .insert("B".to_string(), "2".to_string());

    let cond_a = Condition::VarEquals("A".to_string(), "1".to_string());
    let cond_b = Condition::VarEquals("B".to_string(), "2".to_string());
    let cond_c = Condition::VarEquals("C".to_string(), "3".to_string());

    // Both true
    let cond = Condition::And(Box::new(cond_a.clone()), Box::new(cond_b.clone()));
    assert!(cond.evaluate(&context));

    // One false
    let cond = Condition::And(Box::new(cond_a.clone()), Box::new(cond_c.clone()));
    assert!(!cond.evaluate(&context));

    // Both false
    let cond = Condition::And(Box::new(cond_c.clone()), Box::new(cond_c.clone()));
    assert!(!cond.evaluate(&context));
}

#[test]
fn test_condition_logical_or() {
    let mut context = ExecutionContext::default();
    context
        .variables
        .insert("A".to_string(), "1".to_string());

    let cond_a = Condition::VarEquals("A".to_string(), "1".to_string());
    let cond_b = Condition::VarEquals("B".to_string(), "2".to_string());

    // At least one true
    let cond = Condition::Or(Box::new(cond_a.clone()), Box::new(cond_b.clone()));
    assert!(cond.evaluate(&context));

    // Both false
    let cond = Condition::Or(Box::new(cond_b.clone()), Box::new(cond_b.clone()));
    assert!(!cond.evaluate(&context));
}

#[test]
fn test_condition_logical_not() {
    let mut context = ExecutionContext::default();
    context
        .variables
        .insert("A".to_string(), "1".to_string());

    let cond = Condition::VarEquals("A".to_string(), "1".to_string());
    let not_cond = Condition::Not(Box::new(cond));
    assert!(!not_cond.evaluate(&context));

    let cond = Condition::VarEquals("B".to_string(), "2".to_string());
    let not_cond = Condition::Not(Box::new(cond));
    assert!(not_cond.evaluate(&context));
}

// Note: parse_duration and unquote are private helper methods
// They are indirectly tested through parse_script tests

#[test]
fn test_parse_duration_via_script() {
    // Test duration parsing indirectly through script parsing
    let script_text = r#"
        SLEEP 100ms
        SLEEP 5s
        SLEEP 2m
        WAIT_FOR "test" 10s
    "#;

    let script = AutomationEngine::parse_script(script_text).unwrap();
    assert_eq!(script.commands.len(), 4);
}

#[test]
fn test_unquote_via_script() {
    // Test string unquoting indirectly through script parsing
    let script_text = r#"
        SEND "hello"
        SEND 'world'
        SET VAR="value"
    "#;

    let script = AutomationEngine::parse_script(script_text).unwrap();
    assert_eq!(script.commands.len(), 3);
}

#[test]
fn test_parse_script_basic_commands() {
    let script_text = r#"
# Comment line
SEND "echo hello"
SEND_TEXT "username"
SEND_KEY Enter
    "#;

    let script = AutomationEngine::parse_script(script_text).unwrap();
    assert_eq!(script.commands.len(), 3);
}

#[test]
fn test_parse_script_with_variables() {
    let script_text = r#"
SET USER="alice"
SET AGE="30"
SEND "Hello ${USER}"
    "#;

    let script = AutomationEngine::parse_script(script_text).unwrap();
    assert_eq!(script.commands.len(), 3);
}

#[test]
fn test_parse_script_wait_and_expect() {
    let script_text = r#"
WAIT_FOR "password:" 10s
EXPECT "success"
CAPTURE
    "#;

    let script = AutomationEngine::parse_script(script_text).unwrap();
    assert_eq!(script.commands.len(), 3);
}

#[test]
fn test_parse_script_sleep_and_clear() {
    let script_text = r#"
SLEEP 500ms
CLEAR
SLEEP 2s
    "#;

    let script = AutomationEngine::parse_script(script_text).unwrap();
    assert_eq!(script.commands.len(), 3);
}

#[test]
fn test_parse_script_execute() {
    let script_text = r#"
EXECUTE "ls -la"
EXECUTE "pwd"
    "#;

    let script = AutomationEngine::parse_script(script_text).unwrap();
    assert_eq!(script.commands.len(), 2);
}

#[test]
fn test_parse_script_with_comments() {
    let script_text = r#"
# This is a comment
SEND "command1"
# Another comment
SEND "command2"
    "#;

    let script = AutomationEngine::parse_script(script_text).unwrap();
    assert_eq!(script.commands.len(), 2);
}

#[test]
fn test_parse_script_empty_lines() {
    let script_text = r#"
SEND "command1"


SEND "command2"

    "#;

    let script = AutomationEngine::parse_script(script_text).unwrap();
    assert_eq!(script.commands.len(), 2);
}

#[test]
fn test_automation_script_builder() {
    let mut script = AutomationScript::new("test_script")
        .with_description("A test automation script");

    assert_eq!(script.name, "test_script");
    assert_eq!(
        script.description,
        Some("A test automation script".to_string())
    );

    script.set_variable("USER", "bob");
    script.set_variable("PORT", "8080");

    assert_eq!(script.variables.get("USER"), Some(&"bob".to_string()));
    assert_eq!(script.variables.get("PORT"), Some(&"8080".to_string()));

    script.add_command(AutomationCommand::SendText {
        text: "test".to_string(),
        append_newline: true,
    });

    script.add_command(AutomationCommand::Sleep(Duration::from_millis(100)));

    assert_eq!(script.commands.len(), 2);
}

#[test]
fn test_send_text_command() {
    let mut engine = create_test_engine();

    let cmd = AutomationCommand::SendText {
        text: "echo test".to_string(),
        append_newline: true,
    };

    let result = engine.execute_command(&cmd);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.success);
}

#[test]
fn test_send_keys_command() {
    let mut engine = create_test_engine();

    let cmd = AutomationCommand::SendKeys(vec![
        Key::Char('l'),
        Key::Char('s'),
        Key::Enter,
    ]);

    let result = engine.execute_command(&cmd);
    assert!(result.is_ok());
}

#[test]
fn test_sleep_command() {
    let mut engine = create_test_engine();

    let cmd = AutomationCommand::Sleep(Duration::from_millis(10));

    let start = std::time::Instant::now();
    let result = engine.execute_command(&cmd);
    let duration = start.elapsed();

    assert!(result.is_ok());
    assert!(duration >= Duration::from_millis(10));
}

#[test]
fn test_set_variable_command() {
    let mut engine = create_test_engine();

    let cmd = AutomationCommand::SetVariable {
        name: "TEST_VAR".to_string(),
        value: "test_value".to_string(),
    };

    let result = engine.execute_command(&cmd);
    assert!(result.is_ok());

    assert_eq!(
        engine.context().variables.get("TEST_VAR"),
        Some(&"test_value".to_string())
    );
}

#[test]
fn test_clear_command() {
    let mut engine = create_test_engine();

    let cmd = AutomationCommand::Clear;

    let result = engine.execute_command(&cmd);
    assert!(result.is_ok());
}

#[test]
fn test_variable_expansion_in_commands() {
    let mut engine = create_test_engine();

    // Set a variable
    engine
        .context_mut()
        .variables
        .insert("NAME".to_string(), "Alice".to_string());

    // Use variable in command
    let cmd = AutomationCommand::SendText {
        text: "Hello ${NAME}".to_string(),
        append_newline: true,
    };

    let result = engine.execute_command(&cmd);
    assert!(result.is_ok());
}

#[test]
fn test_automation_script_with_variables() {
    let mut engine = create_test_engine();

    let mut script = AutomationScript::new("var_test");
    script.set_variable("USER", "bob");
    script.set_variable("DIR", "/tmp");

    script.add_command(AutomationCommand::SendText {
        text: "cd ${DIR}".to_string(),
        append_newline: true,
    });

    let result = engine.execute_script(&script);
    assert!(result.is_ok());
}

#[test]
fn test_parse_invalid_command() {
    let script_text = "INVALID_COMMAND arg";

    let result = AutomationEngine::parse_script(script_text);
    assert!(result.is_err());

    if let Err(AutomationError::ParseError { line, message }) = result {
        assert_eq!(line, 1);
        assert!(message.contains("Unknown command"));
    }
}

#[test]
fn test_parse_missing_argument() {
    let script_text = "SEND";

    let result = AutomationEngine::parse_script(script_text);
    assert!(result.is_err());
}

#[test]
fn test_condition_complex_logical_expression() {
    let mut context = ExecutionContext::default();
    context
        .variables
        .insert("A".to_string(), "1".to_string());
    context
        .variables
        .insert("B".to_string(), "2".to_string());
    context
        .variables
        .insert("C".to_string(), "3".to_string());

    // (A=1 AND B=2) OR C=99
    let cond = Condition::Or(
        Box::new(Condition::And(
            Box::new(Condition::VarEquals("A".to_string(), "1".to_string())),
            Box::new(Condition::VarEquals("B".to_string(), "2".to_string())),
        )),
        Box::new(Condition::VarEquals("C".to_string(), "99".to_string())),
    );

    assert!(cond.evaluate(&context));
}

#[test]
fn test_multiple_script_executions() {
    let mut engine = create_test_engine();

    let script1 = AutomationScript::new("script1");
    let result1 = engine.execute_script(&script1);
    assert!(result1.is_ok());

    let script2 = AutomationScript::new("script2");
    let result2 = engine.execute_script(&script2);
    assert!(result2.is_ok());
}
