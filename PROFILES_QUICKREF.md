# Profile System Quick Reference

## Quick Start

```rust
use agterm::profiles::{Profile, ProfileManager};

// Initialize manager
let mut manager = ProfileManager::new();
manager.init()?;

// Create profile
let mut profile = Profile::new("MyProfile".to_string());
profile.font.size = 16.0;
profile.colors.theme = "dracula".to_string();

// Add to manager
let id = manager.add_profile(profile)?;
manager.set_default_profile(&id)?;
```

## Common Operations

### Create Profile
```rust
let mut profile = Profile::new("Name".to_string());
profile.description = Some("Description".to_string());
```

### Configure Font
```rust
profile.font.family = "JetBrains Mono".to_string();
profile.font.size = 16.0;
profile.font.use_ligatures = true;
```

### Configure Colors
```rust
profile.colors.theme = "monokai_pro".to_string();
profile.colors.background_opacity = 0.95;
profile.colors.cursor_style = CursorStyle::BlinkingBar;
```

### Configure Shell
```rust
profile.shell.command = Some("/bin/zsh".to_string());
profile.shell.args = vec!["-l".to_string()];
profile.shell.login_shell = true;
```

### Environment Variables
```rust
profile.environment.insert("EDITOR".to_string(), "nvim".to_string());
```

### Startup Commands
```rust
profile.startup_commands = vec![
    "echo 'Hello'".to_string(),
    "clear".to_string(),
];
```

### Key Bindings
```rust
profile.keybindings.push(KeyBindingOverride {
    key: "t".to_string(),
    modifiers: KeyModifiersConfig { ctrl: true, ..Default::default() },
    action: "new_tab".to_string(),
});
```

## ProfileManager Operations

### Add
```rust
let id = manager.add_profile(profile)?;
```

### Get
```rust
let profile = manager.get_profile(&id);
let profile = manager.get_profile_by_name("Name");
```

### Update
```rust
let mut updated = manager.get_profile(&id).unwrap().clone();
updated.description = Some("New description".to_string());
manager.update_profile(&id, updated)?;
```

### Delete
```rust
manager.delete_profile(&id)?;
```

### Clone
```rust
let new_id = manager.clone_profile(&id, "New Name".to_string())?;
```

### List
```rust
let names = manager.list_profiles();
let all = manager.get_all_profiles();
```

### Default
```rust
manager.set_default_profile(&id)?;
let default = manager.get_default_profile();
```

### Import/Export
```rust
manager.export_profile(&id, Path::new("profile.toml"))?;
let imported_id = manager.import_profile(Path::new("profile.toml"))?;
```

## Built-in Profiles

- **Default**: Standard configuration
- **Developer**: Coding-optimized (Monokai, ligatures)
- **Light**: Light theme (Solarized Light)

Access: `manager.get_profile_by_name("Developer")`

## Available Themes

- `warp_dark` (default)
- `dracula`
- `solarized_dark`
- `solarized_light`
- `nord`
- `one_dark`
- `monokai_pro`
- `tokyo_night`

## Cursor Styles

- `block`, `underline`, `bar`
- `blinking_block`, `blinking_underline`, `blinking_bar`

## Bell Styles

- `none`, `audible`, `visual`, `both`

## Shell Types

- `bash`, `zsh`, `fish`, `nushell`, `powershell`, `cmd`

## Error Handling

```rust
use agterm::profiles::ProfileError;

match manager.add_profile(profile) {
    Ok(id) => println!("Created: {}", id),
    Err(ProfileError::AlreadyExists(name)) => println!("Profile {} exists", name),
    Err(e) => eprintln!("Error: {}", e),
}
```

## TOML Format

```toml
id = "uuid"
name = "Profile Name"
description = "Description"
icon = "🚀"

[font]
family = "Fira Code"
size = 16.0
line_height = 1.3

[colors]
theme = "dracula"
background_opacity = 0.95

[terminal]
scrollback_lines = 50000
scroll_on_input = true

[shell]
command = "/bin/zsh"
args = ["-l"]

[environment]
EDITOR = "nvim"

[[keybindings]]
key = "t"
action = "new_tab"
[keybindings.modifiers]
ctrl = true
```

## File Locations

- Default storage: `~/.config/agterm/profiles/`
- Profile files: `{profile-id}.toml`
- Custom directory: `ProfileManager::with_storage_dir(path)`

## Integration Methods

### Load Theme
```rust
let theme = profile.load_theme();
```

### Apply Keybindings
```rust
let bindings = profile.apply_keybindings();
for (combo, action) in bindings {
    keybind_manager.bind(combo, action);
}
```

### Get Shell Type
```rust
let shell_type = profile.get_shell_type();
```

## Tips

1. **Clone before modifying built-ins**
   ```rust
   let id = manager.clone_profile(&builtin_id, "My Custom".to_string())?;
   ```

2. **Use descriptive names**
   ```rust
   Profile::new("Development - Python".to_string())
   ```

3. **Add icons for visual identification**
   ```rust
   profile.icon = Some("🐍".to_string());
   ```

4. **Export important profiles**
   ```rust
   manager.export_profile(&id, Path::new("backup.toml"))?;
   ```

5. **Check read-only status**
   ```rust
   if !profile.read_only {
       manager.update_profile(&id, profile)?;
   }
   ```

## Common Patterns

### Developer Profile
```rust
let mut dev = Profile::new("Dev".to_string());
dev.font.family = "JetBrains Mono".to_string();
dev.font.use_ligatures = true;
dev.colors.theme = "monokai_pro".to_string();
dev.terminal.scrollback_lines = 100000;
dev.environment.insert("EDITOR".to_string(), "code".to_string());
```

### Light Mode
```rust
let mut light = Profile::new("Light".to_string());
light.colors.theme = "solarized_light".to_string();
light.colors.background_opacity = 1.0;
```

### Production SSH
```rust
let mut prod = Profile::new("Production".to_string());
prod.shell.command = Some("/usr/bin/ssh".to_string());
prod.shell.args = vec!["prod-server".to_string()];
prod.terminal.audible_bell = true;
prod.colors.theme = "nord".to_string();
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_profile() {
        let mut manager = ProfileManager::new();
        let profile = Profile::new("Test".to_string());
        let id = manager.add_profile(profile).unwrap();
        assert!(manager.get_profile(&id).is_some());
    }
}
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Profile not found | Use `list_profiles()` to verify name |
| Cannot modify | Check if `read_only == true`, clone first |
| Theme not loading | Verify theme name with `Theme::available_themes()` |
| Permission denied | Check storage directory permissions |
| Duplicate name | Use different name or delete existing |

## Example: Complete Custom Profile

```rust
let mut manager = ProfileManager::new();
manager.init()?;

let mut profile = Profile::new("Ultimate Dev".to_string());

// Identity
profile.description = Some("My perfect dev environment".to_string());
profile.icon = Some("⚡".to_string());

// Font
profile.font = FontSettings {
    family: "Cascadia Code".to_string(),
    size: 15.0,
    line_height: 1.4,
    bold_as_bright: true,
    use_ligatures: true,
    use_thin_strokes: false,
};

// Colors
profile.colors = ColorSettings {
    theme: "tokyo_night".to_string(),
    background_opacity: 0.92,
    cursor_style: CursorStyle::BlinkingBar,
    cursor_blink_interval: 500,
    custom_colors: None,
};

// Terminal
profile.terminal = TerminalSettings {
    scrollback_lines: 100000,
    scroll_on_input: true,
    bell_style: BellStyle::Visual,
    copy_on_select: false,
    paste_on_middle_click: true,
    ..Default::default()
};

// Shell
profile.shell = ShellSettings {
    command: Some("/opt/homebrew/bin/fish".to_string()),
    args: vec![],
    shell_type: Some("fish".to_string()),
    login_shell: true,
};

// Environment
profile.environment.insert("EDITOR".to_string(), "nvim".to_string());
profile.environment.insert("VISUAL".to_string(), "code".to_string());
profile.environment.insert("PAGER".to_string(), "bat".to_string());

// Startup
profile.startup_commands = vec![
    "set -gx PATH ~/bin $PATH".to_string(),
    "clear".to_string(),
];

// Working directory
profile.working_directory = Some(PathBuf::from("~/projects"));

// Add and set as default
let id = manager.add_profile(profile)?;
manager.set_default_profile(&id)?;

println!("Created and set profile: {}", id);
```
