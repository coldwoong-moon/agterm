//! Terminal Automation Example
//!
//! This example demonstrates the AgTerm automation API structure and usage.
//! Note: This is a documentation example - actual execution requires a running PTY session.

use agterm::automation::{
    AutomationCommand, AutomationScript, Condition, Key, Pattern,
};
use std::collections::HashMap;
use std::time::Duration;

fn main() {
    println!("AgTerm Terminal Automation Examples\n");
    println!("This example demonstrates the API structure without executing commands.\n");

    example_1_basic_commands();
    example_2_pattern_matching();
    example_3_variables();
    example_4_conditional_execution();
    example_5_script_dsl();
    example_6_advanced_script();
    example_7_programmatic_script();
}

/// Example 1: Basic Commands
fn example_1_basic_commands() {
    println!("=== Example 1: Basic Commands ===");

    // Send text to terminal
    let cmd = AutomationCommand::SendText {
        text: "echo 'Hello from automation!'".to_string(),
        append_newline: true,
    };
    println!("  SendText command: {:?}", cmd);

    // Send key combinations
    let cmd = AutomationCommand::SendKeys(vec![
        Key::Char('l'),
        Key::Char('s'),
        Key::Enter,
    ]);
    println!("  SendKeys command: {:?}", cmd);

    println!();
}

/// Example 2: Pattern Matching and Waiting
fn example_2_pattern_matching() {
    println!("=== Example 2: Pattern Matching ===");

    // Execute a command and wait for output
    let script = vec![
        AutomationCommand::SendText {
            text: "echo 'Starting process...'".to_string(),
            append_newline: true,
        },
        AutomationCommand::WaitFor {
            pattern: Pattern::Exact("Starting process".to_string()),
            timeout: Duration::from_secs(5),
        },
        AutomationCommand::Capture { store_in: None },
    ];

    println!("  Script with {} commands:", script.len());
    for (i, cmd) in script.iter().enumerate() {
        println!("    {}: {:?}", i + 1, cmd);
    }

    println!();
}

/// Example 3: Using Variables
fn example_3_variables() {
    println!("=== Example 3: Variables ===");

    let mut variables = HashMap::new();
    variables.insert("USERNAME".to_string(), "alice".to_string());
    variables.insert("DIRECTORY".to_string(), "/tmp".to_string());
    println!("  Initial variables: {:?}", variables);

    let script = vec![
        AutomationCommand::SendText {
            text: "echo 'User: ${USERNAME}'".to_string(),
            append_newline: true,
        },
        AutomationCommand::SendText {
            text: "cd ${DIRECTORY}".to_string(),
            append_newline: true,
        },
        AutomationCommand::SetVariable {
            name: "RESULT".to_string(),
            value: "success".to_string(),
        },
    ];

    println!("  Script using variables:");
    for (i, cmd) in script.iter().enumerate() {
        println!("    {}: {:?}", i + 1, cmd);
    }

    println!();
}

/// Example 4: Conditional Execution
fn example_4_conditional_execution() {
    println!("=== Example 4: Conditional Execution ===");

    let cmd = AutomationCommand::If {
        condition: Condition::VarEquals("STATUS".to_string(), "ok".to_string()),
        then_commands: vec![AutomationCommand::SendText {
            text: "echo 'Status is OK'".to_string(),
            append_newline: true,
        }],
        else_commands: vec![AutomationCommand::SendText {
            text: "echo 'Status is not OK'".to_string(),
            append_newline: true,
        }],
    };

    println!("  Conditional command: {:?}", cmd);

    println!();
}

/// Example 5: Using the Script DSL
fn example_5_script_dsl() {
    println!("=== Example 5: Script DSL ===");

    let script_text = r#"
        # Simple automation script
        SET USER="bob"
        SEND "echo 'Hello ${USER}'"
        SEND_KEY Enter
        WAIT_FOR "Hello" 3s
        CAPTURE
    "#;

    println!("  Script DSL:");
    for line in script_text.lines() {
        if !line.trim().is_empty() {
            println!("    {}", line.trim());
        }
    }

    println!();
}

/// Example 6: Advanced Script
fn example_6_advanced_script() {
    println!("=== Example 6: Advanced Script ===");

    let script_text = r#"
        # Advanced automation example

        # Set up variables
        SET PROJECT_DIR="/tmp/test_project"
        SET BUILD_CMD="make all"

        # Create and enter directory
        SEND "mkdir -p ${PROJECT_DIR}"
        SEND_KEY Enter
        SLEEP 500ms
        SEND "cd ${PROJECT_DIR}"
        SEND_KEY Enter
        SLEEP 500ms

        # Create a simple Makefile
        SEND "cat > Makefile << 'EOF'"
        SEND_KEY Enter
        SEND "all:"
        SEND_KEY Enter
        SEND "	@echo 'Building project...'"
        SEND_KEY Enter
        SEND "	@echo 'Build complete!'"
        SEND_KEY Enter
        SEND "EOF"
        SEND_KEY Enter
        SLEEP 500ms

        # Run build
        SEND "${BUILD_CMD}"
        SEND_KEY Enter
        WAIT_FOR "Build complete" 10s
        CAPTURE
        EXPECT "Build complete"

        # Clean up
        SEND "cd .."
        SEND_KEY Enter
        SEND "rm -rf ${PROJECT_DIR}"
        SEND_KEY Enter
    "#;

    println!("  Advanced script with variable substitution and file creation");
    println!("  Total lines: {}", script_text.lines().count());

    println!();
}

/// Example 7: Building a script programmatically
fn example_7_programmatic_script() {
    println!("=== Example 7: Programmatic Script Building ===");

    let mut script = AutomationScript::new("deployment_script")
        .with_description("Deploy application to server");

    // Add variables
    script.set_variable("SERVER", "example.com");
    script.set_variable("APP_DIR", "/var/www/app");

    // Build command sequence
    script.add_command(AutomationCommand::SendText {
        text: "ssh user@${SERVER}".to_string(),
        append_newline: true,
    });

    script.add_command(AutomationCommand::WaitFor {
        pattern: Pattern::Exact("password:".to_string()),
        timeout: Duration::from_secs(10),
    });

    script.add_command(AutomationCommand::Sleep(Duration::from_millis(500)));

    script.add_command(AutomationCommand::SendText {
        text: "cd ${APP_DIR}".to_string(),
        append_newline: true,
    });

    script.add_command(AutomationCommand::Execute {
        command: "git pull origin main".to_string(),
        wait: true,
    });

    script.add_command(AutomationCommand::Expect {
        pattern: Pattern::Exact("Already up to date".to_string()),
        message: Some("Failed to pull latest code".to_string()),
    });

    println!("  Script: {}", script.name());
    println!("  Description: {:?}", script.description());
    println!("  Commands: {}", script.commands().len());
    println!("  Variables: {:?}", script.variables());

    println!();
}
