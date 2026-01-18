# AgTerm Workspace System

The workspace system provides persistent layout management for AgTerm, allowing you to save and restore complex terminal configurations with multiple tabs and pane splits.

## Features

- **Named Workspaces**: Create and manage multiple workspace configurations with descriptive names
- **Tab Management**: Save and restore multiple tabs with custom titles
- **Pane Layouts**: Support for single pane, horizontal split, vertical split, and grid layouts
- **Terminal State**: Preserve working directory, shell, and environment variables for each pane
- **Auto-restore**: Automatically load a workspace on startup
- **TOML Format**: Human-readable configuration files that can be edited manually

## Basic Usage

### Creating a Workspace

```rust
use agterm::workspace::{Workspace, WorkspaceManager};
use std::path::PathBuf;

// Create a workspace manager
let mut manager = WorkspaceManager::new()?;

// Create a simple workspace
let workspace = Workspace::with_basic_layout(
    "my-project".to_string(),
    "Development environment for my project".to_string(),
    PathBuf::from("/path/to/project"),
)?;

// Save the workspace
manager.save_workspace(&workspace)?;
```

### Adding Tabs and Panes

```rust
use agterm::workspace::{TabLayout, PaneConfig};

// Create a tab with a single pane
let single_pane_tab = TabLayout::single_pane(
    PathBuf::from("/tmp"),
    Some("Terminal".to_string()),
);

// Create a tab with horizontal split (top/bottom)
let split_tab = TabLayout::horizontal_split(
    PathBuf::from("/path/to/server"),
    PathBuf::from("/path/to/logs"),
    Some("Server & Logs".to_string()),
);

// Add tabs to workspace
workspace.add_tab(single_pane_tab);
workspace.add_tab(split_tab);
```

### Advanced Pane Configuration

```rust
// Configure a pane with custom settings
let pane = PaneConfig::new(PathBuf::from("/tmp"))
    .with_shell("/bin/zsh".to_string())
    .with_env_var("NODE_ENV".to_string(), "development".to_string())
    .with_command("npm run dev".to_string())
    .with_focus(true);
```

### Loading and Switching Workspaces

```rust
// Load a workspace
let workspace = manager.load_workspace("my-project")?;

// Switch to a workspace (loads and marks as active)
let workspace = manager.switch_workspace("my-project")?;

// List all available workspaces
let names = manager.list_workspaces()?;
for name in names {
    let info = manager.get_workspace_info(&name)?;
    println!("{}: {} tabs", info.name, info.tab_count);
}
```

### Auto-restore

```rust
// Set a workspace to auto-restore on startup
manager.set_auto_restore("my-project", true)?;

// Get the auto-restore workspace
if let Some(workspace) = manager.get_auto_restore_workspace()? {
    println!("Auto-restoring: {}", workspace.name);
}
```

### Saving Current State

```rust
// Save your current terminal state as a new workspace
let workspace = manager.save_current_state(
    "current-session".to_string(),
    "Auto-saved session".to_string(),
    current_tabs,      // Vec<TabLayout>
    active_tab_index,  // usize
    Some((1920, 1080)), // window size
    14.0,              // font size
)?;
```

## Pane Layout Types

The workspace system supports several layout types:

### Single Pane
```rust
PaneLayoutType::Single
```
One full-screen terminal pane.

### Horizontal Split
```rust
PaneLayoutType::HorizontalSplit
```
Two panes stacked vertically (top and bottom).

### Vertical Split
```rust
PaneLayoutType::VerticalSplit
```
Two panes side by side (left and right).

### Grid Layout (Future)
```rust
PaneLayoutType::Grid { rows: 2, cols: 2 }
```
Multiple panes arranged in a grid.

## TOML Configuration Format

Workspaces are saved as TOML files in `~/.local/share/agterm/workspaces/`. Here's an example:

```toml
version = 1
name = "development"
description = "Main development environment"
created_at = 1705612800
modified_at = 1705612800
last_used_at = 1705612800
active_tab = 0
auto_restore = false

[layout]
font_size = 14.0
window_size = [1920, 1080]

[[layout.tabs]]
title = "Editor"
pane_layout = "single"
focused_pane = 0

[[layout.tabs.panes]]
cwd = "/home/user/project"
shell = "/bin/zsh"
focused = true
env_vars = [
    ["NODE_ENV", "development"],
]

[[layout.tabs]]
title = "Server & Logs"
pane_layout = "horizontal_split"
focused_pane = 0

[[layout.tabs.panes]]
cwd = "/home/user/project"
initial_command = "npm run dev"
focused = true

[[layout.tabs.panes]]
cwd = "/home/user/project"
initial_command = "tail -f logs/app.log"
focused = false
```

## Example Workspaces

### Web Development Workspace

```rust
let mut workspace = Workspace::new(
    "web-dev".to_string(),
    "Full-stack web development".to_string(),
)?;

// Tab 1: Editor
workspace.add_tab(TabLayout::single_pane(
    PathBuf::from("~/projects/webapp"),
    Some("Editor".to_string()),
));

// Tab 2: Frontend Dev Server
let frontend_pane = PaneConfig::new(PathBuf::from("~/projects/webapp/frontend"))
    .with_command("npm run dev".to_string());

workspace.add_tab(TabLayout {
    title: Some("Frontend".to_string()),
    pane_layout: PaneLayoutType::Single,
    panes: vec![frontend_pane],
    focused_pane: 0,
});

// Tab 3: Backend & Database
workspace.add_tab(TabLayout::horizontal_split(
    PathBuf::from("~/projects/webapp/backend"),
    PathBuf::from("~/projects/webapp/backend"),
    Some("Backend & DB".to_string()),
));
```

### DevOps Monitoring Workspace

```rust
let mut workspace = Workspace::new(
    "devops".to_string(),
    "Infrastructure monitoring".to_string(),
)?;

// Kubernetes monitoring
let k8s_pane = PaneConfig::new(PathBuf::from("/tmp"))
    .with_command("kubectl get pods --watch".to_string())
    .with_env_var("KUBECONFIG".to_string(), "~/.kube/config".to_string());

workspace.add_tab(TabLayout {
    title: Some("Kubernetes".to_string()),
    pane_layout: PaneLayoutType::Single,
    panes: vec![k8s_pane],
    focused_pane: 0,
});

// Log monitoring with split view
workspace.add_tab(TabLayout::vertical_split(
    PathBuf::from("/var/log"),
    PathBuf::from("/var/log"),
    Some("Logs".to_string()),
));
```

## API Reference

### Workspace

- `new(name, description)` - Create a new workspace
- `with_basic_layout(name, description, cwd)` - Create a workspace with a single tab and pane
- `add_tab(tab)` - Add a tab to the workspace
- `remove_tab(index)` - Remove a tab from the workspace
- `validate()` - Validate workspace integrity
- `to_toml()` - Serialize to TOML string
- `from_toml(toml_str)` - Deserialize from TOML string

### WorkspaceManager

- `new()` - Create a new workspace manager with default directory
- `with_directory(path)` - Create a manager with custom directory
- `save_workspace(workspace)` - Save a workspace to disk
- `load_workspace(name)` - Load a workspace from disk
- `create_workspace(name, description)` - Create and save a new workspace
- `delete_workspace(name)` - Delete a workspace
- `list_workspaces()` - List all available workspace names
- `get_workspace_info(name)` - Get metadata about a workspace
- `switch_workspace(name)` - Switch to a different workspace
- `get_auto_restore_workspace()` - Get the workspace marked for auto-restore
- `set_auto_restore(name, enabled)` - Enable/disable auto-restore for a workspace
- `save_current_state(...)` - Save current terminal state as a workspace

### TabLayout

- `single_pane(cwd, title)` - Create a single pane tab
- `horizontal_split(top_cwd, bottom_cwd, title)` - Create a horizontal split tab
- `vertical_split(left_cwd, right_cwd, title)` - Create a vertical split tab

### PaneConfig

- `new(cwd)` - Create a new pane configuration
- `with_shell(shell)` - Set the shell for the pane
- `with_env_var(key, value)` - Add an environment variable
- `with_command(command)` - Set an initial command to run
- `with_focus(focused)` - Set whether this pane is focused

## Testing

Run the workspace tests:

```bash
cargo test workspace --lib
```

Run the example demo:

```bash
cargo run --example workspace_demo
```

## File Locations

- **Workspaces**: `~/.local/share/agterm/workspaces/*.toml`
- **Session data**: `~/.local/share/agterm/session.json` (different from workspaces)

On macOS: `~/Library/Application Support/agterm/workspaces/`
On Windows: `%LOCALAPPDATA%\agterm\workspaces\`

## Best Practices

1. **Use descriptive names**: Make workspace names and descriptions clear and meaningful
2. **Organize by project**: Create one workspace per major project
3. **Leverage auto-restore**: Set your most-used workspace to auto-restore
4. **Back up manually**: Workspace files are plain TOML and can be version-controlled
5. **Keep it simple**: Start with basic layouts and add complexity as needed

## Troubleshooting

### Workspace won't load

Check that:
- The workspace file exists in the workspaces directory
- The TOML syntax is valid
- All directories in pane configurations exist
- The version number matches the current format version

### Validation errors

Run `workspace.validate()` to get detailed error information about what's wrong with the workspace configuration.

### Missing directories

If a pane's working directory doesn't exist, it will generate a warning but won't fail validation. Ensure all paths are valid before loading.

## Future Enhancements

- Grid layout support for more than 2 panes
- Dynamic pane resizing
- Nested splits (splits within splits)
- Workspace templates
- Import/export functionality
- Cloud sync support
