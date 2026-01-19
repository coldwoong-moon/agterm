//! Profile System for AgTerm
//!
//! This module provides a comprehensive profile management system that allows users to:
//! - Create and manage terminal profiles with custom settings
//! - Configure fonts, colors, themes, and terminal behavior per profile
//! - Override shell commands, environment variables, and key bindings
//! - Set working directories and startup commands
//! - Save/load profiles from TOML files
//! - Clone and duplicate existing profiles
//! - Set a default profile for new terminals

use crate::shell::ShellType;
#[cfg(feature = "iced-gui")]
use crate::keybind::{Action, KeyCombo, KeyModifiers};
#[cfg(feature = "iced-gui")]
use crate::theme::Theme;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("Profile not found: {0}")]
    NotFound(String),

    #[error("Profile already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid profile name: {0}")]
    InvalidName(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] toml::ser::Error),

    #[error("Deserialization error: {0}")]
    Deserialization(#[from] toml::de::Error),

    #[error("No default profile set")]
    NoDefaultProfile,
}

pub type ProfileResult<T> = Result<T, ProfileError>;

// ============================================================================
// Legacy Profile Format (for backward compatibility)
// ============================================================================

/// Legacy profile format for backward compatibility
/// This handles old profile files that used different field names and structures
#[derive(Debug, Clone, Deserialize)]
struct LegacyProfile {
    #[serde(default = "generate_profile_id")]
    id: String,
    name: String,
    #[serde(default)]
    description: Option<String>,

    // Legacy top-level fields
    #[serde(default)]
    shell: Option<String>,
    #[serde(default)]
    shell_args: Option<Vec<String>>,
    #[serde(default)]
    theme: Option<String>,
    #[serde(default)]
    font_size: Option<f32>,
    #[serde(default)]
    font_family: Option<String>,

    // Current nested structures (may be missing in legacy files)
    #[serde(default)]
    font: Option<FontSettings>,
    #[serde(default)]
    colors: Option<ColorSettings>,
    #[serde(default)]
    terminal: Option<TerminalSettings>,

    // Remaining fields (note: 'env' is sometimes used instead of 'environment')
    #[serde(default)]
    environment: HashMap<String, String>,
    #[serde(default)]
    env: Option<HashMap<String, String>>,
    #[serde(default)]
    working_directory: Option<PathBuf>,
    #[serde(default)]
    startup_commands: Vec<String>,
    #[serde(default)]
    tab_title: Option<String>,
    #[serde(default)]
    icon: Option<String>,
    #[serde(default)]
    read_only: bool,
    #[serde(default)]
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    modified_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl LegacyProfile {
    /// Migrate legacy profile to current format
    fn migrate(self) -> Profile {
        // Merge environment variables (prefer 'environment' over 'env')
        let mut environment = self.environment;
        if let Some(env) = self.env {
            for (k, v) in env {
                environment.entry(k).or_insert(v);
            }
        }

        let mut profile = Profile {
            id: if self.id.is_empty() {
                Uuid::new_v4().to_string()
            } else {
                self.id
            },
            name: self.name,
            description: self.description,
            font: self.font.unwrap_or_default(),
            colors: self.colors.unwrap_or_default(),
            terminal: self.terminal.unwrap_or_default(),
            shell: ShellSettings::default(),
            environment,
            #[cfg(feature = "iced-gui")]
            keybindings: Vec::new(),
            working_directory: self.working_directory,
            startup_commands: self.startup_commands,
            tab_title: self.tab_title,
            icon: self.icon,
            read_only: self.read_only,
            created_at: self.created_at,
            modified_at: self.modified_at,
        };

        // Apply legacy top-level shell settings
        if let Some(shell_cmd) = self.shell {
            profile.shell.command = Some(shell_cmd);
        }
        if let Some(args) = self.shell_args {
            profile.shell.args = args;
        }

        // Apply legacy top-level theme
        if let Some(theme) = self.theme {
            profile.colors.theme = theme;
        }

        // Apply legacy top-level font settings
        if let Some(size) = self.font_size {
            profile.font.size = size;
        }
        if let Some(family) = self.font_family {
            profile.font.family = family;
        }

        profile
    }
}

// ============================================================================
// Profile Structure
// ============================================================================

/// A terminal profile containing all customizable settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Unique identifier for this profile (auto-generated if missing)
    #[serde(default = "generate_profile_id")]
    pub id: String,

    /// Display name of the profile
    pub name: String,

    /// Optional description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Font configuration
    #[serde(default)]
    pub font: FontSettings,

    /// Color and theme settings
    #[serde(default)]
    pub colors: ColorSettings,

    /// Terminal behavior settings
    #[serde(default)]
    pub terminal: TerminalSettings,

    /// Shell configuration
    #[serde(default)]
    pub shell: ShellSettings,

    /// Environment variables
    #[serde(default)]
    pub environment: HashMap<String, String>,

    /// Key binding overrides
    #[serde(default)]
    #[cfg(feature = "iced-gui")]
    pub keybindings: Vec<KeyBindingOverride>,

    /// Working directory
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<PathBuf>,

    /// Startup commands to run after shell launch
    #[serde(default)]
    pub startup_commands: Vec<String>,

    /// Tab title template (can use variables like {cwd}, {shell}, {command})
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tab_title: Option<String>,

    /// Icon or emoji for this profile
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,

    /// Whether this profile is read-only (built-in)
    #[serde(default)]
    pub read_only: bool,

    /// Creation timestamp
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Last modified timestamp
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Font configuration for a profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSettings {
    /// Font family name
    #[serde(default = "default_font_family")]
    pub family: String,

    /// Font size in points
    #[serde(default = "default_font_size")]
    pub size: f32,

    /// Line height multiplier (e.g., 1.2 = 120% of font size)
    #[serde(default = "default_line_height")]
    pub line_height: f32,

    /// Whether to use bold font for bright colors
    #[serde(default = "default_true")]
    pub bold_as_bright: bool,

    /// Use ligatures for programming fonts
    #[serde(default = "default_true")]
    pub use_ligatures: bool,

    /// Use thin strokes on macOS
    #[serde(default)]
    pub use_thin_strokes: bool,
}

/// Color and theme settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorSettings {
    /// Theme name (references a built-in or custom theme)
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Background opacity (0.0 = transparent, 1.0 = opaque)
    #[serde(default = "default_opacity")]
    pub background_opacity: f32,

    /// Cursor style
    #[serde(default)]
    pub cursor_style: CursorStyle,

    /// Cursor blink rate in milliseconds (0 = no blink)
    #[serde(default)]
    pub cursor_blink_interval: u64,

    /// Custom color overrides (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_colors: Option<CustomColors>,
}

/// Custom color overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomColors {
    pub foreground: Option<String>,
    pub background: Option<String>,
    pub cursor: Option<String>,
    pub selection: Option<String>,
    pub ansi: Option<HashMap<u8, String>>,
}

/// Cursor style options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum CursorStyle {
    #[default]
    Block,
    Underline,
    Bar,
    BlinkingBlock,
    BlinkingUnderline,
    BlinkingBar,
}


/// Terminal behavior settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSettings {
    /// Scrollback buffer size (lines)
    #[serde(default = "default_scrollback")]
    pub scrollback_lines: usize,

    /// Whether to scroll to bottom on input
    #[serde(default = "default_true")]
    pub scroll_on_input: bool,

    /// Bell behavior
    #[serde(default)]
    pub bell_style: BellStyle,

    /// Enable audible bell sound
    #[serde(default)]
    pub audible_bell: bool,

    /// Enable visual bell flash
    #[serde(default = "default_true")]
    pub visual_bell: bool,

    /// Copy on selection
    #[serde(default)]
    pub copy_on_select: bool,

    /// Paste on middle click
    #[serde(default = "default_true")]
    pub paste_on_middle_click: bool,

    /// Word separators for double-click selection
    #[serde(default = "default_word_separators")]
    pub word_separators: String,
}

/// Bell notification style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum BellStyle {
    None,
    Audible,
    #[default]
    Visual,
    Both,
}


/// Shell configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct ShellSettings {
    /// Shell command (path to shell executable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    /// Shell arguments
    #[serde(default)]
    pub args: Vec<String>,

    /// Shell type hint
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shell_type: Option<String>,

    /// Whether to run as login shell
    #[serde(default)]
    pub login_shell: bool,
}

/// Key binding override
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindingOverride {
    /// Key combination (e.g., "t")
    pub key: String,

    /// Modifier keys
    #[serde(default)]
    pub modifiers: KeyModifiersConfig,

    /// Action to perform
    pub action: String,
}

/// Modifier keys configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeyModifiersConfig {
    #[serde(default)]
    pub ctrl: bool,

    #[serde(default)]
    pub alt: bool,

    #[serde(default)]
    pub shift: bool,

    #[serde(default)]
    pub cmd: bool,
}

// ============================================================================
// Profile Manager
// ============================================================================

/// Manages a collection of terminal profiles
pub struct ProfileManager {
    /// All available profiles (keyed by profile ID)
    profiles: HashMap<String, Profile>,

    /// Name to ID mapping for quick lookup
    name_index: HashMap<String, String>,

    /// Default profile ID
    default_profile_id: Option<String>,

    /// Directory where profiles are stored
    storage_dir: PathBuf,
}

impl ProfileManager {
    /// Create a new ProfileManager
    pub fn new() -> Self {
        let storage_dir = Self::default_storage_dir();
        Self {
            profiles: HashMap::new(),
            name_index: HashMap::new(),
            default_profile_id: None,
            storage_dir,
        }
    }

    /// Create a ProfileManager with custom storage directory
    pub fn with_storage_dir(dir: PathBuf) -> Self {
        Self {
            profiles: HashMap::new(),
            name_index: HashMap::new(),
            default_profile_id: None,
            storage_dir: dir,
        }
    }

    /// Get the default storage directory for profiles
    pub fn default_storage_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("agterm")
            .join("profiles")
    }

    /// Initialize the profile manager with built-in profiles
    pub fn init(&mut self) -> ProfileResult<()> {
        // Create storage directory if it doesn't exist
        std::fs::create_dir_all(&self.storage_dir)?;

        // Load built-in profiles
        self.load_builtin_profiles();

        // Load user profiles from disk
        self.load_profiles_from_disk()?;

        // If no profiles exist, create a default one
        if self.profiles.is_empty() {
            let default_profile = Profile::default();
            self.add_profile(default_profile)?;
        }

        // Set default profile if not already set
        if self.default_profile_id.is_none() {
            if let Some(profile) = self.profiles.values().next() {
                self.default_profile_id = Some(profile.id.clone());
            }
        }

        Ok(())
    }

    /// Load built-in profiles
    fn load_builtin_profiles(&mut self) {
        // Default profile
        let mut default = Profile::default();
        default.read_only = true;
        default.description = Some("Default AgTerm profile".to_string());
        let _ = self.add_profile_internal(default);

        // Developer profile with Monokai theme
        let mut dev = Profile::default();
        dev.id = Uuid::new_v4().to_string();
        dev.name = "Developer".to_string();
        dev.description = Some("Profile optimized for development work".to_string());
        dev.colors.theme = "monokai_pro".to_string();
        dev.font.family = "JetBrains Mono".to_string();
        dev.font.use_ligatures = true;
        dev.terminal.scrollback_lines = 100000;
        dev.read_only = true;
        dev.icon = Some("💻".to_string());
        let _ = self.add_profile_internal(dev);

        // Light mode profile
        let mut light = Profile::default();
        light.id = Uuid::new_v4().to_string();
        light.name = "Light".to_string();
        light.description = Some("Light theme profile for daytime use".to_string());
        light.colors.theme = "solarized_light".to_string();
        light.colors.background_opacity = 1.0;
        light.read_only = true;
        light.icon = Some("☀️".to_string());
        let _ = self.add_profile_internal(light);
    }

    /// Load profiles from disk
    fn load_profiles_from_disk(&mut self) -> ProfileResult<()> {
        if !self.storage_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(&self.storage_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                match Profile::load_from_file(&path) {
                    Ok(profile) => {
                        let _ = self.add_profile_internal(profile);
                    }
                    Err(e) => {
                        eprintln!("Failed to load profile from {path:?}: {e}");
                    }
                }
            }
        }

        Ok(())
    }

    /// Add a profile (returns error if name already exists)
    pub fn add_profile(&mut self, profile: Profile) -> ProfileResult<String> {
        if self.name_index.contains_key(&profile.name) {
            return Err(ProfileError::AlreadyExists(profile.name.clone()));
        }

        self.add_profile_internal(profile)
    }

    /// Internal method to add profile without name conflict checking
    fn add_profile_internal(&mut self, mut profile: Profile) -> ProfileResult<String> {
        // Update timestamps
        let now = chrono::Utc::now();
        if profile.created_at.is_none() {
            profile.created_at = Some(now);
        }
        profile.modified_at = Some(now);

        let id = profile.id.clone();
        let name = profile.name.clone();

        // Save to disk if not read-only
        if !profile.read_only {
            self.save_profile_to_disk(&profile)?;
        }

        self.profiles.insert(id.clone(), profile);
        self.name_index.insert(name, id.clone());

        Ok(id)
    }

    /// Get a profile by ID
    pub fn get_profile(&self, id: &str) -> Option<&Profile> {
        self.profiles.get(id)
    }

    /// Get a profile by name
    pub fn get_profile_by_name(&self, name: &str) -> Option<&Profile> {
        self.name_index
            .get(name)
            .and_then(|id| self.profiles.get(id))
    }

    /// Get the default profile
    pub fn get_default_profile(&self) -> Option<&Profile> {
        self.default_profile_id
            .as_ref()
            .and_then(|id| self.profiles.get(id))
    }

    /// Set the default profile by ID
    pub fn set_default_profile(&mut self, id: &str) -> ProfileResult<()> {
        if !self.profiles.contains_key(id) {
            return Err(ProfileError::NotFound(id.to_string()));
        }
        self.default_profile_id = Some(id.to_string());
        Ok(())
    }

    /// Set the default profile by name
    pub fn set_default_profile_by_name(&mut self, name: &str) -> ProfileResult<()> {
        let id = self
            .name_index
            .get(name)
            .ok_or_else(|| ProfileError::NotFound(name.to_string()))?
            .clone();
        self.set_default_profile(&id)
    }

    /// Update an existing profile
    pub fn update_profile(&mut self, id: &str, mut profile: Profile) -> ProfileResult<()> {
        if !self.profiles.contains_key(id) {
            return Err(ProfileError::NotFound(id.to_string()));
        }

        let existing = self.profiles.get(id).unwrap();

        // Prevent updating read-only profiles
        if existing.read_only {
            return Err(ProfileError::InvalidName(
                "Cannot update read-only profile".to_string(),
            ));
        }

        // Preserve ID and creation time
        profile.id = id.to_string();
        profile.created_at = existing.created_at;

        // Update modified time
        profile.modified_at = Some(chrono::Utc::now());

        // Update name index if name changed
        if existing.name != profile.name {
            self.name_index.remove(&existing.name);
            if self.name_index.contains_key(&profile.name) {
                return Err(ProfileError::AlreadyExists(profile.name.clone()));
            }
            self.name_index.insert(profile.name.clone(), id.to_string());
        }

        // Save to disk
        self.save_profile_to_disk(&profile)?;

        self.profiles.insert(id.to_string(), profile);
        Ok(())
    }

    /// Delete a profile by ID
    pub fn delete_profile(&mut self, id: &str) -> ProfileResult<()> {
        let profile = self
            .profiles
            .get(id)
            .ok_or_else(|| ProfileError::NotFound(id.to_string()))?;

        // Prevent deleting read-only profiles
        if profile.read_only {
            return Err(ProfileError::InvalidName(
                "Cannot delete read-only profile".to_string(),
            ));
        }

        // Remove from disk
        let filename = format!("{id}.toml");
        let path = self.storage_dir.join(filename);
        if path.exists() {
            std::fs::remove_file(path)?;
        }

        // Remove from memory
        let profile = self.profiles.remove(id).unwrap();
        self.name_index.remove(&profile.name);

        // If this was the default, clear default
        if self.default_profile_id.as_ref() == Some(&id.to_string()) {
            self.default_profile_id = None;
        }

        Ok(())
    }

    /// Clone an existing profile with a new name
    pub fn clone_profile(&mut self, id: &str, new_name: String) -> ProfileResult<String> {
        let source = self
            .profiles
            .get(id)
            .ok_or_else(|| ProfileError::NotFound(id.to_string()))?
            .clone();

        if self.name_index.contains_key(&new_name) {
            return Err(ProfileError::AlreadyExists(new_name));
        }

        let mut cloned = source;
        cloned.id = Uuid::new_v4().to_string();
        cloned.name = new_name;
        cloned.read_only = false;
        cloned.created_at = Some(chrono::Utc::now());
        cloned.modified_at = Some(chrono::Utc::now());

        self.add_profile(cloned)
    }

    /// List all profile names
    pub fn list_profiles(&self) -> Vec<String> {
        let mut names: Vec<_> = self.name_index.keys().cloned().collect();
        names.sort();
        names
    }

    /// List all profiles
    pub fn get_all_profiles(&self) -> Vec<&Profile> {
        self.profiles.values().collect()
    }

    /// Save a profile to disk
    fn save_profile_to_disk(&self, profile: &Profile) -> ProfileResult<()> {
        let filename = format!("{}.toml", profile.id);
        let path = self.storage_dir.join(filename);
        profile.save_to_file(&path)?;
        Ok(())
    }

    /// Export a profile to a TOML file
    pub fn export_profile(&self, id: &str, path: &Path) -> ProfileResult<()> {
        let profile = self
            .profiles
            .get(id)
            .ok_or_else(|| ProfileError::NotFound(id.to_string()))?;
        profile.save_to_file(path)?;
        Ok(())
    }

    /// Import a profile from a TOML file
    pub fn import_profile(&mut self, path: &Path) -> ProfileResult<String> {
        let mut profile = Profile::load_from_file(path)?;

        // Generate new ID and make non-read-only
        profile.id = Uuid::new_v4().to_string();
        profile.read_only = false;

        // Ensure unique name
        let mut name = profile.name.clone();
        let mut counter = 1;
        while self.name_index.contains_key(&name) {
            name = format!("{} ({})", profile.name, counter);
            counter += 1;
        }
        profile.name = name;

        self.add_profile(profile)
    }
}

impl Default for ProfileManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Profile Implementation
// ============================================================================

impl Profile {
    /// Create a new profile with default settings
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: None,
            font: FontSettings::default(),
            colors: ColorSettings::default(),
            terminal: TerminalSettings::default(),
            shell: ShellSettings::default(),
            environment: HashMap::new(),
            #[cfg(feature = "iced-gui")]
            keybindings: Vec::new(),
            working_directory: None,
            startup_commands: Vec::new(),
            tab_title: None,
            icon: None,
            read_only: false,
            created_at: Some(chrono::Utc::now()),
            modified_at: Some(chrono::Utc::now()),
        }
    }

    /// Load a profile from a TOML file
    pub fn load_from_file(path: &Path) -> ProfileResult<Self> {
        let content = std::fs::read_to_string(path)?;

        // Try to load as the current format first
        match toml::from_str::<Profile>(&content) {
            Ok(mut profile) => {
                // Ensure the profile has an ID
                if profile.id.is_empty() {
                    profile.id = Uuid::new_v4().to_string();
                }
                Ok(profile)
            }
            Err(e) => {
                // Try to load as legacy format
                #[cfg(debug_assertions)]
                {
                    eprintln!("Failed to load profile as current format from {path:?}: {e}");
                    eprintln!("Attempting to load as legacy format...");
                }

                match toml::from_str::<LegacyProfile>(&content) {
                    Ok(legacy) => {
                        #[cfg(debug_assertions)]
                        eprintln!("Successfully loaded as legacy format, migrating...");

                        let migrated = legacy.migrate();

                        // Save the migrated profile back to disk in the new format
                        if let Err(_save_err) = migrated.save_to_file(path) {
                            #[cfg(debug_assertions)]
                            eprintln!("Warning: Failed to save migrated profile to {path:?}: {_save_err}");
                        } else {
                            #[cfg(debug_assertions)]
                            eprintln!("Migrated profile saved to {path:?}");
                        }

                        Ok(migrated)
                    }
                    Err(_legacy_err) => {
                        #[cfg(debug_assertions)]
                        eprintln!("Failed to load as legacy format: {_legacy_err}");
                        Err(e.into())
                    }
                }
            }
        }
    }

    /// Save the profile to a TOML file
    pub fn save_to_file(&self, path: &Path) -> ProfileResult<()> {
        let toml_str = toml::to_string_pretty(self)?;
        std::fs::write(path, toml_str)?;
        Ok(())
    }

    /// Load the theme for this profile
    #[cfg(feature = "iced-gui")]
    pub fn load_theme(&self) -> Option<Theme> {
        Theme::by_name(&self.colors.theme)
    }

    /// Apply this profile's key bindings
    #[cfg(feature = "iced-gui")]
    pub fn apply_keybindings(&self) -> Vec<(KeyCombo, Action)> {
        self.keybindings
            .iter()
            .filter_map(|kb| {
                let action = Action::from_string(&kb.action)?;
                let modifiers = KeyModifiers {
                    ctrl: kb.modifiers.ctrl,
                    alt: kb.modifiers.alt,
                    shift: kb.modifiers.shift,
                    super_: kb.modifiers.cmd,
                };
                let combo = KeyCombo {
                    key: kb.key.clone(),
                    modifiers,
                };
                Some((combo, action))
            })
            .collect()
    }

    /// Get shell type
    pub fn get_shell_type(&self) -> Option<ShellType> {
        self.shell.shell_type.as_ref().and_then(|s| match s.as_str() {
            "bash" => Some(ShellType::Bash),
            "zsh" => Some(ShellType::Zsh),
            "fish" => Some(ShellType::Fish),
            "nu" | "nushell" => Some(ShellType::Nushell),
            "powershell" | "pwsh" => Some(ShellType::PowerShell),
            "cmd" => Some(ShellType::Cmd),
            _ => None,
        })
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self::new("Default".to_string())
    }
}

// ============================================================================
// Default Value Functions
// ============================================================================

fn generate_profile_id() -> String {
    Uuid::new_v4().to_string()
}

fn default_font_family() -> String {
    if cfg!(target_os = "macos") {
        "SF Mono".to_string()
    } else if cfg!(target_os = "windows") {
        "Consolas".to_string()
    } else {
        "Monospace".to_string()
    }
}

fn default_font_size() -> f32 {
    14.0
}

fn default_line_height() -> f32 {
    1.2
}

fn default_theme() -> String {
    "warp_dark".to_string()
}

fn default_opacity() -> f32 {
    0.95
}

fn default_scrollback() -> usize {
    10000
}

fn default_word_separators() -> String {
    " ,│`|:\"'()[]{}<>".to_string()
}

fn default_true() -> bool {
    true
}

// ============================================================================
// Default Implementations
// ============================================================================

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            family: default_font_family(),
            size: default_font_size(),
            line_height: default_line_height(),
            bold_as_bright: true,
            use_ligatures: true,
            use_thin_strokes: false,
        }
    }
}

impl Default for ColorSettings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            background_opacity: default_opacity(),
            cursor_style: CursorStyle::default(),
            cursor_blink_interval: 500,
            custom_colors: None,
        }
    }
}

impl Default for TerminalSettings {
    fn default() -> Self {
        Self {
            scrollback_lines: default_scrollback(),
            scroll_on_input: true,
            bell_style: BellStyle::default(),
            audible_bell: false,
            visual_bell: true,
            copy_on_select: false,
            paste_on_middle_click: true,
            word_separators: default_word_separators(),
        }
    }
}


// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_profile_creation() {
        let profile = Profile::new("Test Profile".to_string());
        assert_eq!(profile.name, "Test Profile");
        assert!(!profile.read_only);
        assert!(profile.created_at.is_some());
        assert!(profile.modified_at.is_some());
    }

    #[test]
    fn test_profile_default() {
        let profile = Profile::default();
        assert_eq!(profile.name, "Default");
        assert_eq!(profile.font.family, default_font_family());
        assert_eq!(profile.font.size, default_font_size());
        assert_eq!(profile.colors.theme, default_theme());
    }

    #[test]
    fn test_profile_serialization() {
        let profile = Profile::new("Serialize Test".to_string());
        let toml = toml::to_string(&profile).unwrap();
        assert!(toml.contains("name = \"Serialize Test\""));
        assert!(toml.contains("[font]"));
        assert!(toml.contains("[colors]"));
        assert!(toml.contains("[terminal]"));
    }

    #[test]
    fn test_profile_deserialization() {
        let toml = r#"
            id = "test-123"
            name = "Test Profile"
            description = "A test profile"
            read_only = false

            [font]
            family = "Monaco"
            size = 16.0
            line_height = 1.3
            bold_as_bright = true
            use_ligatures = false

            [colors]
            theme = "dracula"
            background_opacity = 0.9
            cursor_style = "bar"
            cursor_blink_interval = 600

            [terminal]
            scrollback_lines = 50000
            scroll_on_input = true
            bell_style = "both"
            audible_bell = true
            visual_bell = true

            [shell]
            command = "/bin/zsh"
            args = ["-l"]
            shell_type = "zsh"
            login_shell = true
        "#;

        let profile: Profile = toml::from_str(toml).unwrap();
        assert_eq!(profile.name, "Test Profile");
        assert_eq!(profile.font.family, "Monaco");
        assert_eq!(profile.font.size, 16.0);
        assert_eq!(profile.colors.theme, "dracula");
        assert_eq!(profile.colors.cursor_style, CursorStyle::Bar);
        assert_eq!(profile.terminal.scrollback_lines, 50000);
        assert_eq!(profile.shell.command, Some("/bin/zsh".to_string()));
    }

    #[test]
    fn test_profile_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let profile_path = temp_dir.path().join("test_profile.toml");

        let mut profile = Profile::new("Save/Load Test".to_string());
        profile.description = Some("Testing save and load".to_string());
        profile.font.size = 18.0;

        // Save
        profile.save_to_file(&profile_path).unwrap();
        assert!(profile_path.exists());

        // Load
        let loaded = Profile::load_from_file(&profile_path).unwrap();
        assert_eq!(loaded.name, profile.name);
        assert_eq!(loaded.description, profile.description);
        assert_eq!(loaded.font.size, 18.0);
    }

    #[test]
    fn test_profile_manager_creation() {
        let manager = ProfileManager::new();
        assert_eq!(manager.profiles.len(), 0);
    }

    #[test]
    fn test_profile_manager_add_profile() {
        let mut manager = ProfileManager::new();
        let profile = Profile::new("Test".to_string());

        let id = manager.add_profile(profile).unwrap();
        assert!(manager.get_profile(&id).is_some());
        assert!(manager.get_profile_by_name("Test").is_some());
    }

    #[test]
    fn test_profile_manager_duplicate_name() {
        let mut manager = ProfileManager::new();
        let profile1 = Profile::new("Test".to_string());
        let profile2 = Profile::new("Test".to_string());

        manager.add_profile(profile1).unwrap();
        let result = manager.add_profile(profile2);

        assert!(matches!(result, Err(ProfileError::AlreadyExists(_))));
    }

    #[test]
    fn test_profile_manager_update_profile() {
        let mut manager = ProfileManager::new();
        let mut profile = Profile::new("Original".to_string());
        profile.read_only = false;

        let id = manager.add_profile(profile).unwrap();

        let mut updated = manager.get_profile(&id).unwrap().clone();
        updated.description = Some("Updated description".to_string());

        manager.update_profile(&id, updated).unwrap();

        let retrieved = manager.get_profile(&id).unwrap();
        assert_eq!(
            retrieved.description,
            Some("Updated description".to_string())
        );
    }

    #[test]
    fn test_profile_manager_delete_profile() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ProfileManager::with_storage_dir(temp_dir.path().to_path_buf());

        let mut profile = Profile::new("Delete Test".to_string());
        profile.read_only = false;

        let id = manager.add_profile(profile).unwrap();
        assert!(manager.get_profile(&id).is_some());

        manager.delete_profile(&id).unwrap();
        assert!(manager.get_profile(&id).is_none());
    }

    #[test]
    fn test_profile_manager_cannot_delete_readonly() {
        let mut manager = ProfileManager::new();
        let mut profile = Profile::new("ReadOnly".to_string());
        profile.read_only = true;

        let id = manager.add_profile_internal(profile).unwrap();

        let result = manager.delete_profile(&id);
        assert!(matches!(result, Err(ProfileError::InvalidName(_))));
    }

    #[test]
    fn test_profile_manager_clone_profile() {
        let mut manager = ProfileManager::new();
        let mut profile = Profile::new("Original".to_string());
        profile.description = Some("Original profile".to_string());
        profile.font.size = 20.0;

        let original_id = manager.add_profile(profile).unwrap();

        let cloned_id = manager
            .clone_profile(&original_id, "Cloned".to_string())
            .unwrap();

        let cloned = manager.get_profile(&cloned_id).unwrap();
        assert_eq!(cloned.name, "Cloned");
        assert_eq!(cloned.font.size, 20.0);
        assert!(!cloned.read_only);
        assert_ne!(cloned.id, original_id);
    }

    #[test]
    fn test_profile_manager_default_profile() {
        let mut manager = ProfileManager::new();
        let profile = Profile::new("Default Test".to_string());

        let id = manager.add_profile(profile).unwrap();
        manager.set_default_profile(&id).unwrap();

        let default = manager.get_default_profile().unwrap();
        assert_eq!(default.name, "Default Test");
    }

    #[test]
    fn test_profile_manager_list_profiles() {
        let mut manager = ProfileManager::new();

        manager
            .add_profile(Profile::new("Alpha".to_string()))
            .unwrap();
        manager
            .add_profile(Profile::new("Beta".to_string()))
            .unwrap();
        manager
            .add_profile(Profile::new("Gamma".to_string()))
            .unwrap();

        let names = manager.list_profiles();
        assert_eq!(names, vec!["Alpha", "Beta", "Gamma"]);
    }

    #[test]
    #[cfg(feature = "iced-gui")]
    fn test_profile_load_theme() {
        let mut profile = Profile::default();
        profile.colors.theme = "dracula".to_string();

        let theme = profile.load_theme();
        assert!(theme.is_some());
        assert_eq!(theme.unwrap().name, "Dracula");
    }

    #[test]
    #[cfg(feature = "iced-gui")]
    fn test_profile_keybindings() {
        let mut profile = Profile::new("Keybind Test".to_string());

        profile.keybindings.push(KeyBindingOverride {
            key: "t".to_string(),
            modifiers: KeyModifiersConfig {
                ctrl: true,
                ..Default::default()
            },
            action: "new_tab".to_string(),
        });

        let bindings = profile.apply_keybindings();
        assert_eq!(bindings.len(), 1);

        let (combo, action) = &bindings[0];
        assert_eq!(combo.key, "t");
        assert!(combo.modifiers.ctrl);
        assert!(matches!(action, Action::NewTab));
    }

    #[test]
    fn test_profile_shell_type() {
        let mut profile = Profile::default();
        profile.shell.shell_type = Some("zsh".to_string());

        let shell_type = profile.get_shell_type();
        assert!(matches!(shell_type, Some(ShellType::Zsh)));
    }

    #[test]
    fn test_cursor_styles() {
        // Test cursor styles through a simple wrapper struct (as they're actually used)
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct StyleWrapper {
            cursor: CursorStyle,
        }

        let styles = vec![
            CursorStyle::Block,
            CursorStyle::Underline,
            CursorStyle::Bar,
            CursorStyle::BlinkingBlock,
            CursorStyle::BlinkingUnderline,
            CursorStyle::BlinkingBar,
        ];

        for style in styles {
            let wrapper = StyleWrapper { cursor: style };
            let toml_str = toml::to_string(&wrapper).unwrap();
            let deserialized: StyleWrapper = toml::from_str(&toml_str).unwrap();
            assert_eq!(wrapper, deserialized);
        }
    }

    #[test]
    fn test_bell_styles() {
        // Test bell styles through a simple wrapper struct (as they're actually used)
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct StyleWrapper {
            bell: BellStyle,
        }

        let styles = vec![
            BellStyle::None,
            BellStyle::Audible,
            BellStyle::Visual,
            BellStyle::Both,
        ];

        for style in styles {
            let wrapper = StyleWrapper { bell: style };
            let toml_str = toml::to_string(&wrapper).unwrap();
            let deserialized: StyleWrapper = toml::from_str(&toml_str).unwrap();
            assert_eq!(wrapper, deserialized);
        }
    }

    #[test]
    fn test_profile_environment_variables() {
        let mut profile = Profile::new("Env Test".to_string());
        profile.environment.insert("PATH".to_string(), "/custom/path".to_string());
        profile.environment.insert("EDITOR".to_string(), "vim".to_string());

        let toml = toml::to_string(&profile).unwrap();
        let deserialized: Profile = toml::from_str(&toml).unwrap();

        assert_eq!(deserialized.environment.len(), 2);
        assert_eq!(
            deserialized.environment.get("PATH"),
            Some(&"/custom/path".to_string())
        );
    }

    #[test]
    fn test_profile_startup_commands() {
        let mut profile = Profile::new("Startup Test".to_string());
        profile.startup_commands = vec![
            "echo 'Welcome!'".to_string(),
            "source ~/.profile".to_string(),
        ];

        let toml = toml::to_string(&profile).unwrap();
        let deserialized: Profile = toml::from_str(&toml).unwrap();

        assert_eq!(deserialized.startup_commands.len(), 2);
        assert_eq!(deserialized.startup_commands[0], "echo 'Welcome!'");
    }

    #[test]
    fn test_profile_manager_init() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ProfileManager::with_storage_dir(temp_dir.path().to_path_buf());

        manager.init().unwrap();

        // Should have built-in profiles
        let profiles = manager.get_all_profiles();
        assert!(profiles.len() >= 3); // Default, Developer, Light

        // Should have a default profile set
        assert!(manager.get_default_profile().is_some());
    }

    #[test]
    fn test_profile_manager_export_import() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ProfileManager::with_storage_dir(temp_dir.path().to_path_buf());

        let mut profile = Profile::new("Export Test".to_string());
        profile.description = Some("Test export/import".to_string());
        profile.font.size = 22.0;

        let id = manager.add_profile(profile).unwrap();

        // Export
        let export_path = temp_dir.path().join("exported.toml");
        manager.export_profile(&id, &export_path).unwrap();
        assert!(export_path.exists());

        // Import
        let imported_id = manager.import_profile(&export_path).unwrap();
        let imported = manager.get_profile(&imported_id).unwrap();

        // Should have different ID
        assert_ne!(imported.id, id);
        // But same settings
        assert_eq!(imported.font.size, 22.0);
    }

    #[test]
    fn test_legacy_profile_loading() {
        let temp_dir = TempDir::new().unwrap();
        let legacy_path = temp_dir.path().join("legacy.toml");

        // Write a legacy format profile
        let legacy_content = r#"
name = "legacy_test"
shell = "/bin/bash"
shell_args = ["-l"]
theme = "dracula"
font_size = 16.0
font_family = "Monaco"
startup_commands = ["echo 'hello'"]
read_only = false
        "#;

        std::fs::write(&legacy_path, legacy_content).unwrap();

        // Load the legacy profile
        let profile = Profile::load_from_file(&legacy_path).unwrap();

        // Verify migration
        assert_eq!(profile.name, "legacy_test");
        assert!(!profile.id.is_empty()); // Should have auto-generated ID
        assert_eq!(profile.shell.command, Some("/bin/bash".to_string()));
        assert_eq!(profile.shell.args, vec!["-l"]);
        assert_eq!(profile.colors.theme, "dracula");
        assert_eq!(profile.font.size, 16.0);
        assert_eq!(profile.font.family, "Monaco");
        assert_eq!(profile.startup_commands, vec!["echo 'hello'"]);
        assert!(!profile.read_only);
    }

    #[test]
    fn test_legacy_profile_without_id() {
        let temp_dir = TempDir::new().unwrap();
        let legacy_path = temp_dir.path().join("no_id.toml");

        // Write a legacy format profile without ID
        let legacy_content = r#"
name = "no_id_profile"
shell = "/bin/zsh"
theme = "solarized_light"
        "#;

        std::fs::write(&legacy_path, legacy_content).unwrap();

        // Load the profile
        let profile = Profile::load_from_file(&legacy_path).unwrap();

        // Verify ID was auto-generated
        assert_eq!(profile.name, "no_id_profile");
        assert!(!profile.id.is_empty());
        assert_eq!(profile.shell.command, Some("/bin/zsh".to_string()));
        assert_eq!(profile.colors.theme, "solarized_light");
    }
}
