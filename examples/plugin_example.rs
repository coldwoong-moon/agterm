//! Example of creating and using plugins in AgTerm
//!
//! This example demonstrates:
//! - Creating a custom plugin
//! - Registering commands and hooks
//! - Plugin lifecycle management
//! - Event handling
//! - Permission-based access control

use agterm::plugin_api::{
    HookContext, HookHandler, HookType, LogLevel, Notification, Permission, Plugin, PluginContext,
    PluginDependency, PluginError, PluginEvent, PluginManager, PluginMetadata,
    TerminalPermission,
};
use std::sync::Arc;

/// Example plugin that provides git integration
struct GitPlugin {
    metadata: PluginMetadata,
    active: bool,
    command_count: usize,
}

impl GitPlugin {
    fn new() -> Self {
        let metadata = PluginMetadata::new("git-helper", "1.0.0", "Example Developer")
            .with_description("Git integration plugin for AgTerm")
            .with_permissions(vec![
                Permission::Terminal(TerminalPermission::Read),
                Permission::Terminal(TerminalPermission::Write),
                Permission::SystemShell,
            ])
            .with_entry_point("git_plugin::activate");

        Self {
            metadata,
            active: false,
            command_count: 0,
        }
    }
}

impl Plugin for GitPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.log(LogLevel::Info, "Git Helper plugin activated");

        // Register git status command
        let status_handler = Arc::new(|_args: &[String]| -> Result<String, String> {
            Ok("Git status command (would execute: git status)".to_string())
        });
        ctx.register_command("git-status", status_handler);

        // Register git commit command
        let commit_handler = Arc::new(|args: &[String]| -> Result<String, String> {
            let message = args.join(" ");
            Ok(format!("Git commit command (would execute: git commit -m \"{}\")", message))
        });
        ctx.register_command("git-commit", commit_handler);

        // Register hook for git command completion
        let completion_hook: HookHandler = Arc::new(|hook_ctx: &HookContext| -> Result<(), String> {
            println!("Git command completed: {}", hook_ctx.data);
            Ok(())
        });
        ctx.register_hook(HookType::AfterCommand("git".to_string()), completion_hook);

        // Register hook for detecting merge conflicts
        let conflict_hook: HookHandler = Arc::new(|hook_ctx: &HookContext| -> Result<(), String> {
            if hook_ctx.data.contains("CONFLICT") {
                println!("Merge conflict detected!");
            }
            Ok(())
        });
        ctx.register_hook(HookType::OnOutput("CONFLICT".to_string()), conflict_hook);

        self.active = true;
        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), PluginError> {
        self.active = false;
        Ok(())
    }

    fn on_event(&mut self, event: &PluginEvent) -> Result<(), PluginError> {
        match event {
            PluginEvent::CommandResult(cmd, success) => {
                if cmd.starts_with("git") {
                    println!("Git command '{}' {}", cmd, if *success { "succeeded" } else { "failed" });
                }
            }
            PluginEvent::Custom(name, data) => {
                if name == "git-branch-changed" {
                    println!("Branch changed: {:?}", data);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn on_command(&mut self, command: &str, args: &[String]) -> Result<String, PluginError> {
        self.command_count += 1;

        match command {
            "branch" => {
                Ok("Current branches (example):\n* main\n  develop\n  feature/new".to_string())
            }
            "log" => {
                let count = args.get(0).and_then(|s| s.parse::<usize>().ok()).unwrap_or(5);
                Ok(format!("Last {} commits (example)", count))
            }
            "stats" => {
                Ok(format!("Git plugin executed {} commands", self.command_count))
            }
            _ => Err(PluginError::ExecutionError(format!("Unknown command: {}", command))),
        }
    }
}

/// Example plugin that provides syntax highlighting
struct HighlightPlugin {
    metadata: PluginMetadata,
}

impl HighlightPlugin {
    fn new() -> Self {
        let mut metadata = PluginMetadata::new("syntax-highlight", "2.0.0", "Example Developer")
            .with_description("Syntax highlighting for terminal output")
            .with_permissions(vec![
                Permission::Terminal(TerminalPermission::Read),
                Permission::Terminal(TerminalPermission::Write),
            ]);

        // This plugin depends on the git plugin
        metadata.dependencies.push(PluginDependency::new("git-helper", "^1.0.0"));

        Self { metadata }
    }
}

impl Plugin for HighlightPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.log(LogLevel::Info, "Syntax Highlighting plugin activated");

        // Show a notification
        let notification = Notification::new(
            "Plugin Activated",
            "Syntax highlighting is now enabled",
        ).with_timeout(3);
        ctx.show_notification(notification);

        // Register output processing hook
        let highlight_hook: HookHandler = Arc::new(|hook_ctx: &HookContext| -> Result<(), String> {
            // In a real implementation, this would apply syntax highlighting
            if hook_ctx.data.contains("error") || hook_ctx.data.contains("ERROR") {
                println!("Would highlight error in red: {}", hook_ctx.data);
            }
            Ok(())
        });
        ctx.register_hook(HookType::OnOutput(".*".to_string()), highlight_hook);

        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn on_event(&mut self, _event: &PluginEvent) -> Result<(), PluginError> {
        Ok(())
    }

    fn on_command(&mut self, command: &str, args: &[String]) -> Result<String, PluginError> {
        match command {
            "themes" => {
                Ok("Available themes: Monokai, Solarized, Dracula, Nord".to_string())
            }
            "set-theme" => {
                if let Some(theme) = args.get(0) {
                    Ok(format!("Theme set to: {}", theme))
                } else {
                    Err(PluginError::ExecutionError("No theme specified".to_string()))
                }
            }
            _ => Err(PluginError::ExecutionError(format!("Unknown command: {}", command))),
        }
    }
}

fn main() {
    println!("AgTerm Plugin System Example\n");

    // Create plugin manager
    let mut manager = PluginManager::new();
    println!("Created plugin manager");

    // Register git plugin
    let git_plugin = Box::new(GitPlugin::new());
    let git_id = manager.register_plugin(git_plugin);
    println!("Registered git plugin: {}", git_id);

    // Register highlight plugin
    let highlight_plugin = Box::new(HighlightPlugin::new());
    let highlight_id = manager.register_plugin(highlight_plugin);
    println!("Registered highlight plugin: {}", highlight_id);

    // List all plugins
    println!("\nAvailable plugins:");
    for plugin_metadata in manager.list_plugins() {
        println!("  - {} v{} by {}",
            plugin_metadata.name,
            plugin_metadata.version,
            plugin_metadata.author
        );
        println!("    Description: {}", plugin_metadata.description);
        println!("    Permissions: {} required", plugin_metadata.permissions.len());
        if !plugin_metadata.dependencies.is_empty() {
            println!("    Dependencies:");
            for dep in &plugin_metadata.dependencies {
                println!("      - {} ({})", dep.plugin_name, dep.version_requirement);
            }
        }
    }

    // Activate git plugin
    println!("\nActivating git plugin...");
    match manager.activate_plugin(git_id) {
        Ok(_) => println!("Git plugin activated successfully"),
        Err(e) => println!("Failed to activate git plugin: {}", e),
    }

    // Try to activate highlight plugin (depends on git)
    println!("\nActivating highlight plugin...");
    match manager.activate_plugin(highlight_id) {
        Ok(_) => println!("Highlight plugin activated successfully"),
        Err(e) => println!("Failed to activate highlight plugin: {}", e),
    }

    // Execute commands on git plugin
    println!("\nExecuting git plugin commands:");
    match manager.dispatch_command(git_id, "branch", &[]) {
        Ok(result) => println!("branch command result:\n{}", result),
        Err(e) => println!("Error: {}", e),
    }

    match manager.dispatch_command(git_id, "log", &["3".to_string()]) {
        Ok(result) => println!("log command result: {}", result),
        Err(e) => println!("Error: {}", e),
    }

    match manager.dispatch_command(git_id, "stats", &[]) {
        Ok(result) => println!("stats command result: {}", result),
        Err(e) => println!("Error: {}", e),
    }

    // Execute commands on highlight plugin
    println!("\nExecuting highlight plugin commands:");
    match manager.dispatch_command(highlight_id, "themes", &[]) {
        Ok(result) => println!("themes command result: {}", result),
        Err(e) => println!("Error: {}", e),
    }

    match manager.dispatch_command(highlight_id, "set-theme", &["Monokai".to_string()]) {
        Ok(result) => println!("set-theme command result: {}", result),
        Err(e) => println!("Error: {}", e),
    }

    // Dispatch events to plugins
    println!("\nDispatching events:");
    manager.dispatch_event(PluginEvent::CommandResult("git status".to_string(), true));
    manager.dispatch_event(PluginEvent::Custom(
        "git-branch-changed".to_string(),
        serde_json::json!({"from": "main", "to": "develop"}),
    ));

    // Deactivate plugins
    println!("\nDeactivating plugins:");
    manager.deactivate_plugin(highlight_id).unwrap();
    println!("Highlight plugin deactivated");

    manager.deactivate_plugin(git_id).unwrap();
    println!("Git plugin deactivated");

    // Unload plugins
    println!("\nUnloading plugins:");
    manager.unload_plugin(highlight_id).unwrap();
    println!("Highlight plugin unloaded");

    manager.unload_plugin(git_id).unwrap();
    println!("Git plugin unloaded");

    println!("\nExample completed successfully!");
}
