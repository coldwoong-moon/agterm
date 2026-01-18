# Workspace System Architecture

## Module Structure

```
src/workspace.rs
├── Workspace              (Main workspace definition)
│   ├── version: u32
│   ├── name: String
│   ├── description: String
│   ├── created_at: SystemTime
│   ├── modified_at: SystemTime
│   ├── last_used_at: SystemTime
│   ├── layout: WorkspaceLayout
│   ├── active_tab: usize
│   ├── auto_restore: bool
│   └── metadata: HashMap<String, String>
│
├── WorkspaceLayout        (Layout configuration)
│   ├── tabs: Vec<TabLayout>
│   ├── window_size: Option<(u32, u32)>
│   └── font_size: f32
│
├── TabLayout              (Single tab configuration)
│   ├── title: Option<String>
│   ├── pane_layout: PaneLayoutType
│   ├── panes: Vec<PaneConfig>
│   └── focused_pane: usize
│
├── PaneLayoutType         (Layout type enum)
│   ├── Single
│   ├── HorizontalSplit
│   ├── VerticalSplit
│   └── Grid { rows, cols }
│
├── PaneConfig             (Single pane configuration)
│   ├── cwd: PathBuf
│   ├── shell: Option<String>
│   ├── env_vars: Vec<(String, String)>
│   ├── initial_command: Option<String>
│   └── focused: bool
│
├── WorkspaceManager       (Management operations)
│   ├── workspace_dir: PathBuf
│   ├── workspaces: HashMap<String, Workspace>
│   └── active_workspace: Option<String>
│
├── WorkspaceInfo          (Lightweight metadata)
│   ├── name: String
│   ├── description: String
│   ├── created_at: SystemTime
│   ├── modified_at: SystemTime
│   ├── last_used_at: SystemTime
│   ├── tab_count: usize
│   └── auto_restore: bool
│
└── WorkspaceError         (Error types)
    ├── IoError
    ├── TomlSerError
    ├── TomlDeError
    ├── VersionMismatch
    ├── NotFound
    ├── AlreadyExists
    ├── InvalidName
    └── Corrupted
```

## Data Flow

### Creating a Workspace

```
User Code
    │
    ├─> WorkspaceManager::new()
    │       │
    │       └─> Creates manager with default workspace directory
    │
    ├─> manager.create_workspace(name, description)
    │       │
    │       ├─> Workspace::new(name, description)
    │       │       │
    │       │       └─> Validates name
    │       │
    │       ├─> manager.save_workspace(&workspace)
    │       │       │
    │       │       ├─> workspace.validate()
    │       │       ├─> workspace.to_toml()
    │       │       ├─> Write to temp file
    │       │       ├─> Rename to final path (atomic)
    │       │       └─> Cache in memory
    │       │
    │       └─> Returns Workspace
    │
    └─> Workspace ready to use
```

### Loading a Workspace

```
User Code
    │
    ├─> manager.load_workspace(name)
    │       │
    │       ├─> Check in-memory cache
    │       │       │
    │       │       ├─> Cache Hit: Return cached workspace
    │       │       │
    │       │       └─> Cache Miss:
    │       │               │
    │       │               ├─> Read TOML file
    │       │               ├─> Workspace::from_toml(toml_str)
    │       │               │       │
    │       │               │       ├─> toml::from_str()
    │       │               │       └─> workspace.validate()
    │       │               │
    │       │               ├─> Cache workspace
    │       │               └─> Return workspace
    │       │
    │       └─> Returns Result<Workspace>
    │
    └─> Use workspace
```

### Workspace Restoration Flow

```
Application Startup
    │
    ├─> manager.get_auto_restore_workspace()
    │       │
    │       ├─> manager.list_workspaces()
    │       │
    │       ├─> For each workspace:
    │       │       │
    │       │       ├─> manager.load_workspace(name)
    │       │       └─> Check if auto_restore == true
    │       │
    │       └─> Return first auto-restore workspace (or None)
    │
    ├─> If workspace found:
    │       │
    │       ├─> Restore window size (layout.window_size)
    │       ├─> Set font size (layout.font_size)
    │       │
    │       ├─> For each tab in workspace.layout.tabs:
    │       │       │
    │       │       ├─> Create tab with title
    │       │       │
    │       │       └─> For each pane in tab.panes:
    │       │               │
    │       │               ├─> Create PTY session
    │       │               ├─> Set working directory (pane.cwd)
    │       │               ├─> Set shell (pane.shell)
    │       │               ├─> Set environment variables (pane.env_vars)
    │       │               ├─> Run initial command (pane.initial_command)
    │       │               └─> Set focus (pane.focused)
    │       │
    │       └─> Set active tab (workspace.active_tab)
    │
    └─> Application ready
```

## Layout Visualization

### Single Pane Layout
```
┌────────────────────────────────┐
│                                │
│                                │
│          Single Pane           │
│         (PaneConfig)           │
│                                │
│                                │
└────────────────────────────────┘
```

### Horizontal Split Layout
```
┌────────────────────────────────┐
│         Top Pane               │
│       (PaneConfig 0)           │
│        [FOCUSED]               │
├────────────────────────────────┤
│        Bottom Pane             │
│       (PaneConfig 1)           │
│                                │
└────────────────────────────────┘
```

### Vertical Split Layout
```
┌────────────────┬───────────────┐
│                │               │
│   Left Pane    │  Right Pane   │
│ (PaneConfig 0) │(PaneConfig 1) │
│   [FOCUSED]    │               │
│                │               │
│                │               │
└────────────────┴───────────────┘
```

### Grid Layout (2x2 - Future)
```
┌────────────────┬───────────────┐
│  Pane 0        │  Pane 1       │
│  [FOCUSED]     │               │
├────────────────┼───────────────┤
│  Pane 2        │  Pane 3       │
│                │               │
└────────────────┴───────────────┘
```

## Component Hierarchy

### Workspace Organization
```
Workspace "development"
│
├─ Tab 0: "Editor"
│  │
│  └─ Layout: Single
│     │
│     └─ Pane 0: [FOCUSED]
│        ├─ cwd: /home/user/project
│        ├─ shell: /bin/zsh
│        └─ env: []
│
├─ Tab 1: "Server & Logs"  [ACTIVE]
│  │
│  └─ Layout: HorizontalSplit
│     │
│     ├─ Pane 0: [FOCUSED]
│     │  ├─ cwd: /home/user/project
│     │  ├─ command: npm run dev
│     │  └─ env: [("NODE_ENV", "development")]
│     │
│     └─ Pane 1:
│        ├─ cwd: /home/user/project
│        └─ command: tail -f logs/app.log
│
└─ Tab 2: "Tests"
   │
   └─ Layout: VerticalSplit
      │
      ├─ Pane 0: [FOCUSED]
      │  ├─ cwd: /home/user/project
      │  └─ command: npm run test:watch
      │
      └─ Pane 1:
         ├─ cwd: /home/user/project
         └─ command: npm run test:e2e
```

## File System Layout

```
~/.local/share/agterm/
│
└─ workspaces/
   ├─ development.toml        (Development workspace)
   ├─ devops.toml             (DevOps workspace)
   ├─ personal.toml           (Personal workspace)
   └─ quick-debug.toml        (Quick debugging workspace)
```

## State Management

### Workspace Manager State
```
WorkspaceManager
├─ workspace_dir: PathBuf
│  └─ ~/.local/share/agterm/workspaces/
│
├─ workspaces: HashMap<String, Workspace>
│  ├─ "development" -> Workspace { ... }
│  ├─ "devops" -> Workspace { ... }
│  └─ "personal" -> Workspace { ... }
│
└─ active_workspace: Option<String>
   └─ Some("development")
```

## API Call Flow Examples

### Example 1: Create a Development Workspace

```rust
// 1. Create manager
let mut manager = WorkspaceManager::new()?;

// 2. Create workspace
let mut workspace = Workspace::new(
    "dev".to_string(),
    "Development environment".to_string()
)?;

// 3. Add editor tab
let editor_tab = TabLayout::single_pane(
    PathBuf::from("~/project"),
    Some("Editor".to_string())
);
workspace.add_tab(editor_tab);

// 4. Add split terminal tab
let terminal_tab = TabLayout::horizontal_split(
    PathBuf::from("~/project"),
    PathBuf::from("~/project"),
    Some("Terminal".to_string())
);
workspace.add_tab(terminal_tab);

// 5. Save workspace
manager.save_workspace(&workspace)?;

// Result: workspace saved to ~/.local/share/agterm/workspaces/dev.toml
```

### Example 2: Load and Switch Workspace

```rust
// 1. Load workspace
let workspace = manager.load_workspace("dev")?;

// 2. Inspect workspace
println!("Tabs: {}", workspace.layout.tabs.len());
for (i, tab) in workspace.layout.tabs.iter().enumerate() {
    println!("  Tab {}: {:?} ({} panes)",
        i, tab.title, tab.panes.len());
}

// 3. Switch to workspace
manager.switch_workspace("dev")?;

// 4. Workspace is now active
assert_eq!(manager.get_active_workspace(), Some(&"dev".to_string()));
```

## Error Handling Flow

```
Operation
    │
    ├─> Success
    │   └─> Returns Ok(result)
    │
    └─> Failure
        │
        ├─> IoError
        │   └─> File system errors (read/write/delete)
        │
        ├─> TomlSerError / TomlDeError
        │   └─> Serialization/deserialization errors
        │
        ├─> VersionMismatch
        │   └─> Workspace file version != current version
        │
        ├─> NotFound
        │   └─> Workspace file doesn't exist
        │
        ├─> AlreadyExists
        │   └─> Workspace name already taken
        │
        ├─> InvalidName
        │   └─> Empty or contains invalid characters
        │
        └─> Corrupted
            └─> Invalid structure (bad indices, layout mismatch)
```

## Performance Considerations

### Caching Strategy
```
First Load:
    Read file → Parse TOML → Validate → Cache → Return
    (Slower: ~5-10ms)

Subsequent Loads:
    Check cache → Return cached
    (Faster: <1ms)

Memory Usage:
    Each workspace: ~1-5 KB in memory
    Cache 10 workspaces: ~10-50 KB total
```

### Atomic Saves
```
Save Operation:
    1. Serialize to TOML string
    2. Write to temp file (.tmp extension)
    3. Rename temp file to final name
       └─> Atomic operation (no partial writes)
    4. Update in-memory cache

Benefits:
    - No corrupted files from crashes
    - No partial writes visible to readers
    - Fast recovery from failures
```

## Integration with AgTerm

The workspace system integrates with AgTerm's existing architecture:

```
AgTerm Application
│
├─ GUI Layer (Iced)
│  └─ Workspace selection UI
│     ├─ List workspaces
│     ├─ Switch workspace
│     └─ Create/edit/delete
│
├─ Terminal Management
│  ├─ Tab creation from TabLayout
│  ├─ PTY spawning for each pane
│  └─ Environment setup
│
├─ Session Management (src/session.rs)
│  ├─ Crash recovery (different purpose)
│  └─ Complementary to workspaces
│
└─ Workspace System (src/workspace.rs)
   ├─ Save/load layouts
   ├─ Auto-restore
   └─ Workspace switching
```
