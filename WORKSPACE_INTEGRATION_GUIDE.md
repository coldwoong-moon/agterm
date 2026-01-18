# Workspace System Integration Guide

This guide shows how to integrate the workspace system into AgTerm's main application.

## Step 1: Add WorkspaceManager to Application State

```rust
// In src/main.rs or your main application state

use agterm::workspace::{WorkspaceManager, Workspace, TabLayout};

struct AgTerm {
    // Existing fields...
    tabs: Vec<TerminalTab>,
    active_tab: usize,
    font_size: f32,

    // Add workspace manager
    workspace_manager: WorkspaceManager,
    current_workspace: Option<String>,
}

impl AgTerm {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize workspace manager
        let workspace_manager = WorkspaceManager::new()?;

        // Try to load auto-restore workspace
        let mut app = Self {
            tabs: Vec::new(),
            active_tab: 0,
            font_size: 14.0,
            workspace_manager,
            current_workspace: None,
        };

        // Restore workspace if available
        if let Ok(Some(workspace)) = app.workspace_manager.get_auto_restore_workspace() {
            app.restore_workspace(workspace)?;
        } else {
            // Create default tab if no workspace
            app.create_default_tab();
        }

        Ok(app)
    }
}
```

## Step 2: Implement Workspace Restoration

```rust
impl AgTerm {
    /// Restore a workspace by creating tabs and panes from the configuration
    fn restore_workspace(&mut self, workspace: Workspace) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Restoring workspace: {}", workspace.name);

        // Clear existing tabs
        self.tabs.clear();

        // Set window configuration
        if let Some((width, height)) = workspace.layout.window_size {
            self.set_window_size(width, height);
        }
        self.font_size = workspace.layout.font_size;

        // Restore each tab
        for tab_layout in &workspace.layout.tabs {
            self.restore_tab(tab_layout)?;
        }

        // Set active tab
        if workspace.active_tab < self.tabs.len() {
            self.active_tab = workspace.active_tab;
        }

        // Update current workspace
        self.current_workspace = Some(workspace.name.clone());

        Ok(())
    }

    /// Restore a single tab from layout configuration
    fn restore_tab(&mut self, tab_layout: &TabLayout) -> Result<(), Box<dyn std::error::Error>> {
        match tab_layout.pane_layout {
            PaneLayoutType::Single => {
                // Single pane - standard tab
                if let Some(pane) = tab_layout.panes.first() {
                    let tab = self.create_tab_from_pane(pane, tab_layout.title.clone())?;
                    self.tabs.push(tab);
                }
            }

            PaneLayoutType::HorizontalSplit => {
                // Horizontal split - create tab with 2 panes (top/bottom)
                if tab_layout.panes.len() >= 2 {
                    let tab = self.create_split_tab(
                        &tab_layout.panes[0],
                        &tab_layout.panes[1],
                        tab_layout.title.clone(),
                        true, // horizontal
                    )?;
                    self.tabs.push(tab);
                }
            }

            PaneLayoutType::VerticalSplit => {
                // Vertical split - create tab with 2 panes (left/right)
                if tab_layout.panes.len() >= 2 {
                    let tab = self.create_split_tab(
                        &tab_layout.panes[0],
                        &tab_layout.panes[1],
                        tab_layout.title.clone(),
                        false, // vertical
                    )?;
                    self.tabs.push(tab);
                }
            }

            PaneLayoutType::Grid { rows, cols } => {
                // Grid layout - future implementation
                tracing::warn!("Grid layout not yet implemented, using single pane");
                if let Some(pane) = tab_layout.panes.first() {
                    let tab = self.create_tab_from_pane(pane, tab_layout.title.clone())?;
                    self.tabs.push(tab);
                }
            }
        }

        Ok(())
    }

    /// Create a terminal tab from pane configuration
    fn create_tab_from_pane(
        &mut self,
        pane: &PaneConfig,
        title: Option<String>,
    ) -> Result<TerminalTab, Box<dyn std::error::Error>> {
        // Create PTY with specified configuration
        let pty_config = self.create_pty_config(pane)?;
        let session = self.pty_manager.spawn_pty(pty_config)?;

        // Create tab
        let mut tab = TerminalTab::new(self.next_tab_id(), Some(session.id()));

        // Set custom title if provided
        if let Some(title) = title {
            tab.title = Some(title);
        }

        // Run initial command if specified
        if let Some(cmd) = &pane.initial_command {
            self.pty_manager.write(session.id(), format!("{}\n", cmd).as_bytes())?;
        }

        Ok(tab)
    }

    /// Create PTY configuration from pane config
    fn create_pty_config(&self, pane: &PaneConfig) -> Result<PtyConfig, Box<dyn std::error::Error>> {
        let mut config = PtyConfig {
            cwd: Some(pane.cwd.clone()),
            shell: pane.shell.clone(),
            env_vars: pane.env_vars.clone(),
            ..Default::default()
        };

        Ok(config)
    }
}
```

## Step 3: Implement Workspace Saving

```rust
impl AgTerm {
    /// Save current terminal state as a workspace
    fn save_current_as_workspace(
        &mut self,
        name: String,
        description: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Build tab layouts from current state
        let mut tab_layouts = Vec::new();

        for tab in &self.tabs {
            let tab_layout = self.tab_to_layout(tab)?;
            tab_layouts.push(tab_layout);
        }

        // Get window size
        let window_size = Some((self.window_width, self.window_height));

        // Save workspace
        let workspace = self.workspace_manager.save_current_state(
            name.clone(),
            description,
            tab_layouts,
            self.active_tab,
            window_size,
            self.font_size,
        )?;

        self.current_workspace = Some(name);

        tracing::info!("Saved workspace: {}", workspace.name);
        Ok(())
    }

    /// Convert a terminal tab to a workspace tab layout
    fn tab_to_layout(&self, tab: &TerminalTab) -> Result<TabLayout, Box<dyn std::error::Error>> {
        match tab.pane_layout {
            PaneLayout::Single => {
                // Single pane tab
                let pane_config = self.pane_to_config(&tab, 0)?;

                Ok(TabLayout {
                    title: tab.title.clone(),
                    pane_layout: PaneLayoutType::Single,
                    panes: vec![pane_config],
                    focused_pane: 0,
                })
            }

            PaneLayout::HorizontalSplit => {
                // Horizontal split tab
                let pane0 = self.pane_to_config(&tab, 0)?;
                let pane1 = self.pane_to_config(&tab, 1)?;

                Ok(TabLayout {
                    title: tab.title.clone(),
                    pane_layout: PaneLayoutType::HorizontalSplit,
                    panes: vec![pane0, pane1],
                    focused_pane: tab.focused_pane,
                })
            }

            PaneLayout::VerticalSplit => {
                // Vertical split tab
                let pane0 = self.pane_to_config(&tab, 0)?;
                let pane1 = self.pane_to_config(&tab, 1)?;

                Ok(TabLayout {
                    title: tab.title.clone(),
                    pane_layout: PaneLayoutType::VerticalSplit,
                    panes: vec![pane0, pane1],
                    focused_pane: tab.focused_pane,
                })
            }
        }
    }

    /// Convert a pane to pane configuration
    fn pane_to_config(
        &self,
        tab: &TerminalTab,
        pane_index: usize,
    ) -> Result<PaneConfig, Box<dyn std::error::Error>> {
        // Get pane or use tab's main screen
        let pane = if pane_index < tab.panes.len() {
            &tab.panes[pane_index]
        } else {
            return Err("Pane index out of bounds".into());
        };

        // Get current working directory from PTY
        let cwd = if let Some(pty_id) = pane.pty_id {
            self.pty_manager.get_cwd(pty_id)
                .unwrap_or_else(|_| PathBuf::from("/tmp"))
        } else {
            PathBuf::from("/tmp")
        };

        // Get shell from PTY
        let shell = if let Some(pty_id) = pane.pty_id {
            self.pty_manager.get_shell(pty_id).ok()
        } else {
            None
        };

        Ok(PaneConfig {
            cwd,
            shell,
            env_vars: Vec::new(), // Could get from PTY if needed
            initial_command: None,
            focused: pane.focused,
        })
    }
}
```

## Step 4: Add UI Commands

```rust
// Add to your Message enum
#[derive(Debug, Clone)]
enum Message {
    // Existing messages...

    // Workspace management
    SaveWorkspace { name: String, description: String },
    LoadWorkspace(String),
    DeleteWorkspace(String),
    ListWorkspaces,
    SetAutoRestore(String),

    // Workspace UI
    ShowWorkspaceMenu,
    HideWorkspaceMenu,
    WorkspaceSelected(String),
}

impl AgTerm {
    fn update(&mut self, message: Message) {
        match message {
            // Existing message handlers...

            Message::SaveWorkspace { name, description } => {
                if let Err(e) = self.save_current_as_workspace(name, description) {
                    tracing::error!("Failed to save workspace: {}", e);
                }
            }

            Message::LoadWorkspace(name) => {
                match self.workspace_manager.load_workspace(&name) {
                    Ok(workspace) => {
                        if let Err(e) = self.restore_workspace(workspace) {
                            tracing::error!("Failed to restore workspace: {}", e);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to load workspace {}: {}", name, e);
                    }
                }
            }

            Message::DeleteWorkspace(name) => {
                if let Err(e) = self.workspace_manager.delete_workspace(&name) {
                    tracing::error!("Failed to delete workspace: {}", e);
                }
            }

            Message::ListWorkspaces => {
                match self.workspace_manager.list_workspaces() {
                    Ok(names) => {
                        tracing::info!("Available workspaces: {:?}", names);
                        // Update UI with workspace list
                    }
                    Err(e) => {
                        tracing::error!("Failed to list workspaces: {}", e);
                    }
                }
            }

            Message::SetAutoRestore(name) => {
                if let Err(e) = self.workspace_manager.set_auto_restore(&name, true) {
                    tracing::error!("Failed to set auto-restore: {}", e);
                }
            }

            _ => {}
        }
    }
}
```

## Step 5: Add Keyboard Shortcuts

```rust
use iced::keyboard::{Key, Modifiers};

impl AgTerm {
    fn handle_keyboard_event(&mut self, key: Key, modifiers: Modifiers) -> Option<Message> {
        match (key, modifiers) {
            // Existing shortcuts...

            // Cmd/Ctrl + S: Save current workspace
            (Key::Character('s'), m) if m.command() => {
                Some(Message::SaveWorkspace {
                    name: "current".to_string(),
                    description: "Auto-saved".to_string(),
                })
            }

            // Cmd/Ctrl + Shift + W: Show workspace menu
            (Key::Character('w'), m) if m.command() && m.shift() => {
                Some(Message::ShowWorkspaceMenu)
            }

            // Cmd/Ctrl + 1-9: Quick switch to workspace
            (Key::Character(c), m) if m.command() && ('1'..='9').contains(&c) => {
                let index = c.to_digit(10).unwrap() as usize - 1;
                // Get workspace name from index and switch
                None // Implement workspace quick-switch
            }

            _ => None
        }
    }
}
```

## Step 6: Add Workspace Menu UI

```rust
use iced::widget::{button, column, container, row, text, text_input, Column};

impl AgTerm {
    fn workspace_menu(&self) -> Element<Message> {
        let mut content = Column::new()
            .spacing(10)
            .padding(20);

        // Title
        content = content.push(
            text("Workspaces")
                .size(24)
        );

        // List existing workspaces
        if let Ok(names) = self.workspace_manager.list_workspaces() {
            for name in names {
                if let Ok(info) = self.workspace_manager.get_workspace_info(&name) {
                    let workspace_row = row![
                        text(&info.name).width(Length::Fill),
                        text(format!("{} tabs", info.tab_count)),
                        button("Load").on_press(Message::LoadWorkspace(name.clone())),
                        button("Delete").on_press(Message::DeleteWorkspace(name.clone())),
                    ]
                    .spacing(10);

                    content = content.push(workspace_row);
                }
            }
        }

        // Save current workspace
        content = content.push(
            row![
                text_input("Workspace name", &self.new_workspace_name)
                    .on_input(Message::WorkspaceNameChanged),
                button("Save Current")
                    .on_press(Message::SaveWorkspace {
                        name: self.new_workspace_name.clone(),
                        description: "Saved from UI".to_string(),
                    }),
            ]
            .spacing(10)
        );

        // Close button
        content = content.push(
            button("Close").on_press(Message::HideWorkspaceMenu)
        );

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
```

## Step 7: Auto-save on Exit

```rust
impl AgTerm {
    fn on_exit(&mut self) {
        // Auto-save current workspace if named
        if let Some(name) = &self.current_workspace {
            tracing::info!("Auto-saving workspace: {}", name);

            if let Err(e) = self.save_current_as_workspace(
                name.clone(),
                format!("Auto-saved on {}", chrono::Local::now()),
            ) {
                tracing::error!("Failed to auto-save workspace: {}", e);
            }
        }
    }
}
```

## Complete Integration Example

```rust
// src/main.rs

use agterm::workspace::{WorkspaceManager, Workspace, TabLayout, PaneConfig};

fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Run application
    iced::application("AgTerm", AgTerm::update, AgTerm::view)
        .subscription(AgTerm::subscription)
        .run_with(|| {
            // Initialize with workspace restoration
            match AgTerm::new() {
                Ok(app) => (app, Task::none()),
                Err(e) => {
                    eprintln!("Failed to initialize: {}", e);
                    std::process::exit(1);
                }
            }
        })
}

// Application state with workspace support
struct AgTerm {
    tabs: Vec<TerminalTab>,
    active_tab: usize,
    workspace_manager: WorkspaceManager,
    current_workspace: Option<String>,
    show_workspace_menu: bool,
    new_workspace_name: String,
}

impl AgTerm {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut workspace_manager = WorkspaceManager::new()?;
        let mut tabs = Vec::new();
        let mut current_workspace = None;

        // Try to restore auto-restore workspace
        if let Ok(Some(workspace)) = workspace_manager.get_auto_restore_workspace() {
            tracing::info!("Auto-restoring workspace: {}", workspace.name);
            current_workspace = Some(workspace.name.clone());

            // Restore tabs from workspace
            for tab_layout in &workspace.layout.tabs {
                // Create tabs based on layout
                // (implementation in Step 2)
            }
        }

        // Create default tab if no workspace
        if tabs.is_empty() {
            tabs.push(TerminalTab::new(0, None));
        }

        Ok(Self {
            tabs,
            active_tab: 0,
            workspace_manager,
            current_workspace,
            show_workspace_menu: false,
            new_workspace_name: String::new(),
        })
    }
}
```

## Testing Integration

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_restoration() {
        let mut app = AgTerm::new().unwrap();

        // Create some tabs
        app.tabs.push(TerminalTab::new(1, None));
        app.tabs.push(TerminalTab::new(2, None));

        // Save as workspace
        app.save_current_as_workspace(
            "test".to_string(),
            "Test workspace".to_string(),
        ).unwrap();

        // Clear tabs
        app.tabs.clear();

        // Restore workspace
        let workspace = app.workspace_manager.load_workspace("test").unwrap();
        app.restore_workspace(workspace).unwrap();

        // Verify tabs restored
        assert_eq!(app.tabs.len(), 2);
    }
}
```

## Summary

This integration guide shows how to:
1. ✅ Add WorkspaceManager to application state
2. ✅ Restore workspaces on startup (auto-restore)
3. ✅ Save current state as workspace
4. ✅ Handle workspace UI messages
5. ✅ Add keyboard shortcuts
6. ✅ Create workspace menu UI
7. ✅ Auto-save on exit

The workspace system integrates cleanly with AgTerm's existing architecture and provides a powerful way to manage complex terminal layouts.
