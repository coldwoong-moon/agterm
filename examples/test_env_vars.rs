//! Test environment variables in PTY sessions
//!
//! This example creates a PTY session and verifies that environment variables
//! are set correctly.

use agterm::terminal::pty::{PtyEnvironment, PtyManager};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

fn main() {
    // Initialize tracing for debug output
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    println!("Testing PTY Environment Variables");
    println!("==================================\n");

    // Test 1: Default environment
    println!("Test 1: Default environment (no custom config)");
    test_default_env();

    // Test 2: Custom environment with inheritance
    println!("\nTest 2: Custom environment with inheritance");
    test_custom_env_with_inheritance();

    // Test 3: Minimal environment without inheritance
    println!("\nTest 3: Minimal environment (no inheritance)");
    test_minimal_env();

    println!("\n✓ All tests completed!");
}

fn test_default_env() {
    let manager = PtyManager::new();
    let session_id = manager.create_session(24, 80).expect("Failed to create session");

    // Test critical environment variables
    let test_commands = vec![
        "echo TERM=$TERM",
        "echo COLORTERM=$COLORTERM",
        "echo TERM_PROGRAM=$TERM_PROGRAM",
        "echo SHELL=$SHELL",
        "echo HOME=$HOME",
        "echo USER=$USER",
        "echo PATH=$PATH",
        "exit\n",
    ];

    for cmd in test_commands {
        manager
            .write(&session_id, cmd.as_bytes())
            .expect("Failed to write");
        manager
            .write(&session_id, b"\n")
            .expect("Failed to write");
        thread::sleep(Duration::from_millis(50));
    }

    // Read all output
    thread::sleep(Duration::from_millis(200));
    let output = manager.read(&session_id).expect("Failed to read");
    let output_str = String::from_utf8_lossy(&output);

    println!("Output:\n{}", output_str);

    // Verify key variables are present
    assert!(output_str.contains("TERM=xterm-256color"), "TERM not set correctly");
    assert!(output_str.contains("COLORTERM=truecolor"), "COLORTERM not set correctly");
    assert!(output_str.contains("TERM_PROGRAM=agterm"), "TERM_PROGRAM not set correctly");
    assert!(output_str.contains("SHELL="), "SHELL not set");
    assert!(output_str.contains("HOME="), "HOME not set");

    manager.close_session(&session_id).expect("Failed to close");
    println!("✓ Default environment test passed");
}

fn test_custom_env_with_inheritance() {
    let manager = PtyManager::new();

    let mut variables = HashMap::new();
    variables.insert("CUSTOM_VAR".to_string(), "custom_value".to_string());
    variables.insert("MY_APP".to_string(), "agterm".to_string());

    let env = PtyEnvironment {
        inherit_env: true,
        variables,
        unset: Vec::new(),
    };

    let session_id = manager
        .create_session_with_env(24, 80, Some(env))
        .expect("Failed to create session");

    let test_commands = vec![
        "echo CUSTOM_VAR=$CUSTOM_VAR",
        "echo MY_APP=$MY_APP",
        "echo HOME=$HOME",
        "exit\n",
    ];

    for cmd in test_commands {
        manager
            .write(&session_id, cmd.as_bytes())
            .expect("Failed to write");
        manager
            .write(&session_id, b"\n")
            .expect("Failed to write");
        thread::sleep(Duration::from_millis(50));
    }

    thread::sleep(Duration::from_millis(200));
    let output = manager.read(&session_id).expect("Failed to read");
    let output_str = String::from_utf8_lossy(&output);

    println!("Output:\n{}", output_str);

    assert!(output_str.contains("CUSTOM_VAR=custom_value"), "Custom variable not set");
    assert!(output_str.contains("MY_APP=agterm"), "MY_APP not set");
    assert!(output_str.contains("HOME="), "HOME not inherited");

    manager.close_session(&session_id).expect("Failed to close");
    println!("✓ Custom environment with inheritance test passed");
}

fn test_minimal_env() {
    let manager = PtyManager::new();

    let env = PtyEnvironment::minimal();

    let session_id = manager
        .create_session_with_env(24, 80, Some(env))
        .expect("Failed to create session");

    let test_commands = vec![
        "echo HOME=$HOME",
        "echo USER=$USER",
        "echo SHELL=$SHELL",
        "echo PATH=$PATH",
        "exit\n",
    ];

    for cmd in test_commands {
        manager
            .write(&session_id, cmd.as_bytes())
            .expect("Failed to write");
        manager
            .write(&session_id, b"\n")
            .expect("Failed to write");
        thread::sleep(Duration::from_millis(50));
    }

    thread::sleep(Duration::from_millis(200));
    let output = manager.read(&session_id).expect("Failed to read");
    let output_str = String::from_utf8_lossy(&output);

    println!("Output:\n{}", output_str);

    // Even with minimal env, critical variables should be set
    assert!(output_str.contains("HOME="), "HOME not set in minimal env");
    assert!(output_str.contains("SHELL="), "SHELL not set in minimal env");
    assert!(output_str.contains("PATH="), "PATH not set in minimal env");

    manager.close_session(&session_id).expect("Failed to close");
    println!("✓ Minimal environment test passed");
}
