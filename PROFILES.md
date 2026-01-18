# AgTerm Profile System

The AgTerm profile system provides powerful customization capabilities for terminal sessions. Each profile contains a complete set of terminal settings including fonts, colors, themes, shell configuration, environment variables, key bindings, and more.

## Table of Contents

- [Overview](#overview)
- [Profile Structure](#profile-structure)
- [ProfileManager](#profilemanager)
- [Creating Profiles](#creating-profiles)
- [Managing Profiles](#managing-profiles)
- [Profile Configuration](#profile-configuration)
- [File Format](#file-format)
- [API Reference](#api-reference)
- [Examples](#examples)

## Overview

Profiles in AgTerm allow you to:

- Customize fonts, colors, and themes per profile
- Configure shell commands and environment variables
- Override key bindings for specific workflows
- Set working directories and startup commands
- Create multiple profiles for different use cases (development, ops, light mode, etc.)
- Clone and duplicate existing profiles
- Import/export profiles as TOML files
- Set a default profile for new terminals

## Profile Structure

Each profile contains the following sections:

### Basic Information
- `id`: Unique identifier (UUID)
- `name`: Display name
- `description`: Optional description
- `icon`: Optional emoji or icon
- `read_only`: Whether the profile is built-in (immutable)
- `created_at`: Creation timestamp
- `modified_at`: Last modification timestamp

### Font Settings
- `family`: Font family name (e.g., "Fira Code", "SF Mono")
- `size`: Font size in points
- `line_height`: Line height multiplier
- `bold_as_bright`: Use bold font for bright colors
- `use_ligatures`: Enable programming ligatures
- `use_thin_strokes`: Use thin strokes on macOS

### Color Settings
- `theme`: Theme name (references built-in themes)
- `background_opacity`: Background opacity (0.0-1.0)
- `cursor_style`: Cursor appearance (block, bar, underline, blinking variants)
- `cursor_blink_interval`: Cursor blink rate in milliseconds
- `custom_colors`: Optional color overrides

### Terminal Settings
- `scrollback_lines`: Scrollback buffer size
- `scroll_on_input`: Scroll to bottom on input
- `bell_style`: Bell notification style (none, audible, visual, both)
- `audible_bell`: Enable audible bell sound
- `visual_bell`: Enable visual bell flash
- `copy_on_select`: Copy text on selection
- `paste_on_middle_click`: Paste on middle mouse click
- `word_separators`: Characters that separate words

### Shell Settings
- `command`: Shell command path
- `args`: Shell arguments
- `shell_type`: Shell type hint (bash, zsh, fish, etc.)
- `login_shell`: Run as login shell

### Environment & Startup
- `environment`: Environment variables (key-value map)
- `startup_commands`: Commands to run after shell launch
- `working_directory`: Initial working directory

### Key Bindings
- `keybindings`: Key binding overrides

## ProfileManager

The `ProfileManager` manages all profiles and provides CRUD operations.

### Initialization

```rust
use agterm::profiles::ProfileManager;

// Create with default storage directory (~/.config/agterm/profiles)
let mut manager = ProfileManager::new();
manager.init()?;

// Or specify custom directory
let mut manager = ProfileManager::with_storage_dir(custom_path);
manager.init()?;
```

### Built-in Profiles

AgTerm comes with three built-in profiles:

1. **Default** - Standard AgTerm configuration
2. **Developer** - Optimized for development (Monokai theme, ligatures, large scrollback)
3. **Light** - Light theme for daytime use (Solarized Light)

Built-in profiles are read-only and cannot be modified or deleted.

## Creating Profiles

### Basic Profile

```rust
use agterm::profiles::Profile;

let mut profile = Profile::new("My Profile".to_string());
profile.description = Some("Custom profile for my workflow".to_string());
profile.icon = Some("🚀".to_string());
```

### Configuring Font

```rust
profile.font.family = "JetBrains Mono".to_string();
profile.font.size = 16.0;
profile.font.use_ligatures = true;
```

### Configuring Colors

```rust
profile.colors.theme = "dracula".to_string();
profile.colors.background_opacity = 0.95;
profile.colors.cursor_style = CursorStyle::BlinkingBar;
```

### Configuring Shell

```rust
profile.shell.command = Some("/bin/zsh".to_string());
profile.shell.args = vec!["-l".to_string()];
profile.shell.shell_type = Some("zsh".to_string());
profile.shell.login_shell = true;
```

### Adding Environment Variables

```rust
profile.environment.insert("EDITOR".to_string(), "nvim".to_string());
profile.environment.insert("PAGER".to_string(), "less -R".to_string());
```

### Adding Startup Commands

```rust
profile.startup_commands = vec![
    "echo 'Welcome!'".to_string(),
    "source ~/.custom_profile".to_string(),
];
```

### Adding Key Bindings

```rust
use agterm::profiles::KeyBindingOverride;

profile.keybindings.push(KeyBindingOverride {
    key: "t".to_string(),
    modifiers: KeyModifiersConfig {
        ctrl: true,
        ..Default::default()
    },
    action: "new_tab".to_string(),
});
```

## Managing Profiles

### Add Profile

```rust
let profile_id = manager.add_profile(profile)?;
```

### Get Profile

```rust
// By ID
let profile = manager.get_profile(&profile_id);

// By name
let profile = manager.get_profile_by_name("My Profile");
```

### Update Profile

```rust
let mut profile = manager.get_profile(&profile_id).unwrap().clone();
profile.description = Some("Updated description".to_string());
manager.update_profile(&profile_id, profile)?;
```

### Delete Profile

```rust
manager.delete_profile(&profile_id)?;
```

### Clone Profile

```rust
let new_id = manager.clone_profile(&profile_id, "New Name".to_string())?;
```

### Set Default Profile

```rust
manager.set_default_profile(&profile_id)?;

// Or by name
manager.set_default_profile_by_name("My Profile")?;
```

### List Profiles

```rust
// List names
let names = manager.list_profiles();

// Get all profiles
let profiles = manager.get_all_profiles();
```

## Profile Configuration

### Available Themes

- `warp_dark` (default)
- `dracula`
- `solarized_dark`
- `solarized_light`
- `nord`
- `one_dark`
- `monokai_pro`
- `tokyo_night`

### Cursor Styles

- `block` - Solid block cursor
- `underline` - Underline cursor
- `bar` - Vertical bar cursor
- `blinking_block` - Blinking block
- `blinking_underline` - Blinking underline
- `blinking_bar` - Blinking bar

### Bell Styles

- `none` - No bell
- `audible` - Sound only
- `visual` - Visual flash only
- `both` - Both sound and visual

### Shell Types

- `bash`
- `zsh`
- `fish`
- `nushell` / `nu`
- `powershell` / `pwsh`
- `cmd`

## File Format

Profiles are stored as TOML files in `~/.config/agterm/profiles/`.

Example profile file (`my-profile.toml`):

```toml
id = "550e8400-e29b-41d4-a716-446655440000"
name = "My Profile"
description = "Custom development profile"
icon = "🚀"
read_only = false

[font]
family = "Fira Code"
size = 16.0
line_height = 1.4
bold_as_bright = true
use_ligatures = true
use_thin_strokes = false

[colors]
theme = "monokai_pro"
background_opacity = 0.95
cursor_style = "blinking_bar"
cursor_blink_interval = 500

[terminal]
scrollback_lines = 50000
scroll_on_input = true
bell_style = "visual"
audible_bell = false
visual_bell = true
copy_on_select = false
paste_on_middle_click = true
word_separators = " ,│`|:\"'()[]{}<>"

[shell]
command = "/bin/zsh"
args = ["-l"]
shell_type = "zsh"
login_shell = true

[environment]
EDITOR = "nvim"
PAGER = "less -R"

[[keybindings]]
key = "t"
action = "new_tab"

[keybindings.modifiers]
ctrl = true
alt = false
shift = false
cmd = false

startup_commands = [
    "echo 'Welcome to My Profile!'",
    "clear"
]
```

## API Reference

### Profile

```rust
pub struct Profile {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub font: FontSettings,
    pub colors: ColorSettings,
    pub terminal: TerminalSettings,
    pub shell: ShellSettings,
    pub environment: HashMap<String, String>,
    pub keybindings: Vec<KeyBindingOverride>,
    pub working_directory: Option<PathBuf>,
    pub startup_commands: Vec<String>,
    pub tab_title: Option<String>,
    pub icon: Option<String>,
    pub read_only: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub modified_at: Option<DateTime<Utc>>,
}

impl Profile {
    pub fn new(name: String) -> Self;
    pub fn load_from_file(path: &Path) -> ProfileResult<Self>;
    pub fn save_to_file(&self, path: &Path) -> ProfileResult<()>;
    pub fn load_theme(&self) -> Option<Theme>;
    pub fn apply_keybindings(&self) -> Vec<(KeyCombo, Action)>;
    pub fn get_shell_type(&self) -> Option<ShellType>;
}
```

### ProfileManager

```rust
pub struct ProfileManager {
    // Internal fields
}

impl ProfileManager {
    pub fn new() -> Self;
    pub fn with_storage_dir(dir: PathBuf) -> Self;
    pub fn init(&mut self) -> ProfileResult<()>;

    pub fn add_profile(&mut self, profile: Profile) -> ProfileResult<String>;
    pub fn get_profile(&self, id: &str) -> Option<&Profile>;
    pub fn get_profile_by_name(&self, name: &str) -> Option<&Profile>;
    pub fn get_default_profile(&self) -> Option<&Profile>;

    pub fn set_default_profile(&mut self, id: &str) -> ProfileResult<()>;
    pub fn set_default_profile_by_name(&mut self, name: &str) -> ProfileResult<()>;

    pub fn update_profile(&mut self, id: &str, profile: Profile) -> ProfileResult<()>;
    pub fn delete_profile(&mut self, id: &str) -> ProfileResult<()>;
    pub fn clone_profile(&mut self, id: &str, new_name: String) -> ProfileResult<String>;

    pub fn list_profiles(&self) -> Vec<String>;
    pub fn get_all_profiles(&self) -> Vec<&Profile>;

    pub fn export_profile(&self, id: &str, path: &Path) -> ProfileResult<()>;
    pub fn import_profile(&mut self, path: &Path) -> ProfileResult<String>;
}
```

## Examples

### Example 1: Developer Profile

```rust
let mut dev_profile = Profile::new("Developer".to_string());
dev_profile.description = Some("Optimized for coding".to_string());
dev_profile.icon = Some("💻".to_string());

dev_profile.font = FontSettings {
    family: "JetBrains Mono".to_string(),
    size: 14.0,
    line_height: 1.3,
    use_ligatures: true,
    ..Default::default()
};

dev_profile.colors.theme = "monokai_pro".to_string();
dev_profile.terminal.scrollback_lines = 100000;

dev_profile.environment.insert("EDITOR".to_string(), "code".to_string());
dev_profile.startup_commands = vec!["clear".to_string()];

let id = manager.add_profile(dev_profile)?;
manager.set_default_profile(&id)?;
```

### Example 2: Light Mode Profile

```rust
let mut light_profile = Profile::new("Light Mode".to_string());
light_profile.icon = Some("☀️".to_string());

light_profile.colors = ColorSettings {
    theme: "solarized_light".to_string(),
    background_opacity: 1.0,
    ..Default::default()
};

manager.add_profile(light_profile)?;
```

### Example 3: Import/Export

```rust
// Export
manager.export_profile(&profile_id, Path::new("my-profile.toml"))?;

// Import
let imported_id = manager.import_profile(Path::new("my-profile.toml"))?;
```

### Example 4: Clone and Modify

```rust
// Clone existing profile
let work_id = manager.clone_profile(&dev_id, "Work".to_string())?;

// Modify cloned profile
let mut work_profile = manager.get_profile(&work_id).unwrap().clone();
work_profile.environment.insert("PROJECT".to_string(), "/work".to_string());
manager.update_profile(&work_id, work_profile)?;
```

## Best Practices

1. **Name profiles descriptively** - Use clear names like "Development", "Production", "Light Mode"
2. **Use icons** - Emojis help quickly identify profiles
3. **Document changes** - Use the description field
4. **Clone before modifying** - Clone built-in profiles rather than creating from scratch
5. **Export important profiles** - Back up custom profiles
6. **Use environment variables** - Configure tools via environment instead of startup commands
7. **Test profiles** - Verify settings work before setting as default

## Troubleshooting

### Profile not found
- Check profile name spelling
- Use `list_profiles()` to see available profiles

### Cannot modify profile
- Built-in profiles are read-only
- Clone the profile first, then modify the clone

### Profile not persisting
- Check storage directory permissions
- Verify profile is not marked as read-only

### Theme not loading
- Verify theme name is correct
- Use `Theme::available_themes()` to list valid themes

## Future Enhancements

Planned features for the profile system:

- [ ] Profile templates
- [ ] Profile inheritance
- [ ] Per-tab profile switching
- [ ] Profile quick-switch hotkeys
- [ ] Profile auto-detection based on directory
- [ ] Remote profile synchronization
- [ ] Profile marketplace/sharing
