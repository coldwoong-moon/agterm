//! Workspace system demonstration
//!
//! This example demonstrates how to use the AgTerm workspace system to:
//! - Create and manage workspaces
//! - Save and load workspace configurations
//! - Define tab and pane layouts
//! - Switch between workspaces

use agterm::workspace::{
    PaneConfig, PaneLayoutType, TabLayout, Workspace, WorkspaceManager,
};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AgTerm Workspace System Demo ===\n");

    // Create a workspace manager
    let temp_dir = std::env::temp_dir().join("agterm_workspace_demo");
    std::fs::create_dir_all(&temp_dir)?;

    let mut manager = WorkspaceManager::with_directory(temp_dir.clone())?;
    println!("Created workspace manager at: {:?}\n", temp_dir);

    // Example 1: Create a development workspace with multiple tabs
    println!("--- Example 1: Development Workspace ---");
    let mut dev_workspace = Workspace::new(
        "development".to_string(),
        "Main development environment with editor, terminal, and tests".to_string(),
    )?;

    // Tab 1: Editor (single pane)
    let editor_tab = TabLayout::single_pane(
        PathBuf::from("/tmp"),
        Some("Editor".to_string()),
    );
    dev_workspace.add_tab(editor_tab);

    // Tab 2: Terminal (horizontal split - top: server, bottom: logs)
    let server_pane = PaneConfig::new(PathBuf::from("/tmp"))
        .with_command("npm run dev".to_string())
        .with_focus(true);

    let logs_pane = PaneConfig::new(PathBuf::from("/tmp"))
        .with_command("tail -f /var/log/system.log".to_string());

    let terminal_tab = TabLayout {
        title: Some("Server & Logs".to_string()),
        pane_layout: PaneLayoutType::HorizontalSplit,
        panes: vec![server_pane, logs_pane],
        focused_pane: 0,
    };
    dev_workspace.add_tab(terminal_tab);

    // Tab 3: Tests (vertical split - left: unit tests, right: integration tests)
    let unit_tests_tab = TabLayout::vertical_split(
        PathBuf::from("/tmp"),
        PathBuf::from("/tmp"),
        Some("Tests".to_string()),
    );
    dev_workspace.add_tab(unit_tests_tab);

    dev_workspace.layout.font_size = 14.0;
    dev_workspace.layout.window_size = Some((1920, 1080));

    manager.save_workspace(&dev_workspace)?;
    println!("Created and saved 'development' workspace with {} tabs", dev_workspace.layout.tabs.len());
    println!("  - Tab 1: Editor (single pane)");
    println!("  - Tab 2: Server & Logs (horizontal split)");
    println!("  - Tab 3: Tests (vertical split)\n");

    // Example 2: Create a DevOps workspace
    println!("--- Example 2: DevOps Workspace ---");
    let mut devops_workspace = Workspace::new(
        "devops".to_string(),
        "Infrastructure and deployment monitoring".to_string(),
    )?;

    // Tab 1: Kubernetes (single pane with kubectl)
    let k8s_pane = PaneConfig::new(PathBuf::from("/tmp"))
        .with_command("kubectl get pods --watch".to_string())
        .with_env_var("KUBECONFIG".to_string(), "/path/to/kubeconfig".to_string());

    let k8s_tab = TabLayout {
        title: Some("Kubernetes".to_string()),
        pane_layout: PaneLayoutType::Single,
        panes: vec![k8s_pane],
        focused_pane: 0,
    };
    devops_workspace.add_tab(k8s_tab);

    // Tab 2: Monitoring (grid layout - future feature, using horizontal split for now)
    let monitoring_tab = TabLayout::horizontal_split(
        PathBuf::from("/tmp"),
        PathBuf::from("/tmp"),
        Some("Monitoring".to_string()),
    );
    devops_workspace.add_tab(monitoring_tab);

    devops_workspace.auto_restore = true; // Auto-restore this workspace on startup
    manager.save_workspace(&devops_workspace)?;
    println!("Created and saved 'devops' workspace with {} tabs", devops_workspace.layout.tabs.len());
    println!("  - Auto-restore enabled");
    println!("  - Tab 1: Kubernetes monitoring");
    println!("  - Tab 2: System monitoring\n");

    // Example 3: Create a simple workspace with current directory
    println!("--- Example 3: Quick Workspace ---");
    let quick_workspace = Workspace::with_basic_layout(
        "quick".to_string(),
        "Quick terminal workspace".to_string(),
        PathBuf::from("."),
    )?;
    manager.save_workspace(&quick_workspace)?;
    println!("Created 'quick' workspace with basic layout\n");

    // List all workspaces
    println!("--- Listing All Workspaces ---");
    let workspaces = manager.list_workspaces()?;
    println!("Available workspaces: {}", workspaces.len());
    for name in &workspaces {
        let info = manager.get_workspace_info(name)?;
        println!("  - {}: {} ({}  tabs)", info.name, info.description, info.tab_count);
        if info.auto_restore {
            println!("    [AUTO-RESTORE]");
        }
    }
    println!();

    // Switch between workspaces
    println!("--- Switching Workspaces ---");
    let loaded = manager.switch_workspace("development")?;
    println!("Switched to: {}", loaded.name);
    println!("  Description: {}", loaded.description);
    println!("  Tabs: {}", loaded.layout.tabs.len());
    println!("  Active tab: {}", loaded.active_tab);
    println!();

    // Get auto-restore workspace
    println!("--- Auto-Restore Workspace ---");
    if let Some(auto_ws) = manager.get_auto_restore_workspace()? {
        println!("Found auto-restore workspace: {}", auto_ws.name);
        println!("  Description: {}", auto_ws.description);
    }
    println!();

    // Save current state as a workspace
    println!("--- Saving Current State ---");
    let current_tabs = vec![
        TabLayout::single_pane(PathBuf::from("/tmp"), Some("Current Work".to_string())),
    ];
    let current_ws = manager.save_current_state(
        "current-session".to_string(),
        "Auto-saved current session".to_string(),
        current_tabs,
        0,
        Some((1920, 1080)),
        16.0,
    )?;
    println!("Saved current state as workspace: {}", current_ws.name);
    println!();

    // Demonstrate TOML serialization
    println!("--- TOML Serialization Example ---");
    let toml_output = quick_workspace.to_toml()?;
    println!("Workspace serialized to TOML:");
    println!("{}", &toml_output[..toml_output.len().min(500)]);
    if toml_output.len() > 500 {
        println!("... (truncated)");
    }
    println!();

    // Cleanup
    println!("--- Cleanup ---");
    manager.delete_workspace("quick")?;
    println!("Deleted 'quick' workspace");
    println!("Remaining workspaces: {}", manager.list_workspaces()?.len());

    println!("\n=== Demo Complete ===");
    println!("Workspace files saved to: {:?}", temp_dir);

    Ok(())
}
