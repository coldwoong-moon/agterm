//! Plugin API for AgTerm
//!
//! This module provides a comprehensive plugin system that allows extending AgTerm
//! with custom functionality through a safe, permission-based API.
//!
//! # Features
//! - Plugin lifecycle management (load, activate, deactivate, unload)
//! - Fine-grained permission system
//! - Event hooks and custom event dispatching
//! - Command registration and handling
//! - Dependency management with version checking
//! - Plugin discovery and registry
//!
//! # Example
//! ```no_run
//! use agterm::plugin_api::{Plugin, PluginMetadata, PluginContext, PluginEvent, PluginError};
//!
//! struct MyPlugin {
//!     metadata: PluginMetadata,
//! }
//!
//! impl Plugin for MyPlugin {
//!     fn metadata(&self) -> &PluginMetadata {
//!         &self.metadata
//!     }
//!
//!     fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
//!         ctx.log(LogLevel::Info, "Plugin activated");
//!         Ok(())
//!     }
//!
//!     fn deactivate(&mut self) -> Result<(), PluginError> {
//!         Ok(())
//!     }
//!
//!     fn on_event(&mut self, event: &PluginEvent) -> Result<(), PluginError> {
//!         // Handle events
//!         Ok(())
//!     }
//!
//!     fn on_command(&mut self, command: &str, args: &[String]) -> Result<String, PluginError> {
//!         Ok(format!("Executed: {}", command))
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use uuid::Uuid;

// Re-export notification type from our notification module
pub use crate::notification::NotificationManager;

/// Unique identifier for a plugin
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginId(Uuid);

impl PluginId {
    /// Create a new random plugin ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a plugin ID from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for PluginId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PluginId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Metadata describing a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique identifier for this plugin
    pub id: PluginId,
    /// Human-readable name
    pub name: String,
    /// Semantic version (e.g., "1.0.0")
    pub version: String,
    /// Plugin author
    pub author: String,
    /// Description of what the plugin does
    pub description: String,
    /// Homepage URL (optional)
    pub homepage: Option<String>,
    /// License identifier (e.g., "MIT", "Apache-2.0")
    pub license: Option<String>,
    /// Minimum AgTerm version required
    pub min_agterm_version: Option<String>,
    /// Maximum AgTerm version supported
    pub max_agterm_version: Option<String>,
    /// Plugin dependencies
    pub dependencies: Vec<PluginDependency>,
    /// Required permissions
    pub permissions: Vec<Permission>,
    /// Entry point (main file or function)
    pub entry_point: String,
}

impl PluginMetadata {
    /// Create new plugin metadata
    pub fn new(name: impl Into<String>, version: impl Into<String>, author: impl Into<String>) -> Self {
        Self {
            id: PluginId::new(),
            name: name.into(),
            version: version.into(),
            author: author.into(),
            description: String::new(),
            homepage: None,
            license: None,
            min_agterm_version: None,
            max_agterm_version: None,
            dependencies: Vec::new(),
            permissions: Vec::new(),
            entry_point: String::from("main"),
        }
    }

    /// Builder method to set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Builder method to set permissions
    pub fn with_permissions(mut self, permissions: Vec<Permission>) -> Self {
        self.permissions = permissions;
        self
    }

    /// Builder method to set entry point
    pub fn with_entry_point(mut self, entry_point: impl Into<String>) -> Self {
        self.entry_point = entry_point.into();
        self
    }
}

/// Dependency on another plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// Name of the required plugin
    pub plugin_name: String,
    /// Version requirement (semver range, e.g., "^1.0.0", ">=2.0.0")
    pub version_requirement: String,
}

impl PluginDependency {
    /// Create a new plugin dependency
    pub fn new(plugin_name: impl Into<String>, version_requirement: impl Into<String>) -> Self {
        Self {
            plugin_name: plugin_name.into(),
            version_requirement: version_requirement.into(),
        }
    }
}

/// Permission types that plugins can request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    /// File system access with specified scope
    FileSystem(PathScope),
    /// Network access with specified scope
    Network(NetworkScope),
    /// Access to clipboard
    Clipboard,
    /// Execute shell commands
    SystemShell,
    /// Read/write environment variables
    Environment,
    /// Intercept keyboard input
    KeyboardHook,
    /// Modify AgTerm configuration
    Configuration,
    /// Terminal access with specified permissions
    Terminal(TerminalPermission),
}

/// Scope of file system access
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathScope {
    /// Read-only access to specific path
    ReadOnly(PathBuf),
    /// Read-write access to specific path
    ReadWrite(PathBuf),
    /// Access to user's home directory
    Home,
    /// Access to current working directory
    Cwd,
    /// Unrestricted file system access
    Anywhere,
}

/// Scope of network access
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkScope {
    /// Only localhost connections
    LocalhostOnly,
    /// Specific hosts/domains
    Specific(Vec<String>),
    /// Unrestricted network access
    Anywhere,
}

/// Terminal-specific permissions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminalPermission {
    /// Read terminal output
    Read,
    /// Write to terminal
    Write,
    /// Control terminal (resize, etc.)
    Control,
}

/// Current state of a plugin
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginState {
    /// Plugin is loaded but not active
    Loaded,
    /// Plugin is active and running
    Active,
    /// Plugin is temporarily inactive
    Inactive,
    /// Plugin encountered an error
    Error(String),
    /// Plugin has been unloaded
    Unloaded,
}

/// Log level for plugin logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Notification to show to user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub title: String,
    pub body: String,
    pub timeout_seconds: u64,
}

impl Notification {
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            timeout_seconds: 5,
        }
    }

    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }
}

/// Command handler function type
pub type CommandHandler = Arc<dyn Fn(&[String]) -> Result<String, String> + Send + Sync>;

/// Hook handler function type
pub type HookHandler = Arc<dyn Fn(&HookContext) -> Result<(), String> + Send + Sync>;

/// Context provided to hook handlers
#[derive(Debug, Clone)]
pub struct HookContext {
    pub data: String,
    pub metadata: HashMap<String, String>,
}

impl HookContext {
    pub fn new(data: impl Into<String>) -> Self {
        Self {
            data: data.into(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Key pattern for keyboard hooks
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyPattern {
    pub key: String,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

impl KeyPattern {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
        }
    }

    pub fn with_ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    pub fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }

    pub fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub fn with_meta(mut self) -> Self {
        self.meta = true;
        self
    }
}

/// Types of hooks that plugins can register
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookType {
    /// Before a command is executed
    BeforeCommand(String),
    /// After a command is executed
    AfterCommand(String),
    /// When output matches a pattern
    OnOutput(String), // Regex pattern as string for serialization
    /// When a key is pressed
    OnKey(KeyPattern),
    /// When terminal tab changes
    OnTabChange,
    /// When terminal is resized
    OnResize,
    /// When window focus changes
    OnFocus,
    /// When application starts
    OnStartup,
    /// When application is closing
    OnShutdown,
}

/// Events that plugins can emit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginEvent {
    /// Custom event with arbitrary data
    Custom(String, serde_json::Value),
    /// Command execution result
    CommandResult(String, bool),
    /// Output processing result
    OutputProcessed(String),
}

/// Context provided to plugins for interacting with AgTerm
pub struct PluginContext {
    terminal_writer: Arc<Mutex<Vec<u8>>>,
    terminal_reader: Arc<Mutex<Vec<u8>>>,
    config_store: Arc<Mutex<HashMap<String, String>>>,
    logger: Arc<dyn Fn(LogLevel, &str) + Send + Sync>,
    notifier: Arc<Mutex<Option<NotificationManager>>>,
    command_registry: Arc<Mutex<HashMap<String, CommandHandler>>>,
    hook_registry: Arc<Mutex<HashMap<String, Vec<HookHandler>>>>,
    selection: Arc<Mutex<Option<String>>>,
    cwd: Arc<Mutex<PathBuf>>,
    event_emitter: Arc<Mutex<Vec<PluginEvent>>>,
}

impl PluginContext {
    /// Create a new plugin context
    pub fn new() -> Self {
        Self {
            terminal_writer: Arc::new(Mutex::new(Vec::new())),
            terminal_reader: Arc::new(Mutex::new(Vec::new())),
            config_store: Arc::new(Mutex::new(HashMap::new())),
            logger: Arc::new(|level, msg| {
                match level {
                    LogLevel::Trace => tracing::trace!("{}", msg),
                    LogLevel::Debug => tracing::debug!("{}", msg),
                    LogLevel::Info => tracing::info!("{}", msg),
                    LogLevel::Warn => tracing::warn!("{}", msg),
                    LogLevel::Error => tracing::error!("{}", msg),
                }
            }),
            notifier: Arc::new(Mutex::new(None)),
            command_registry: Arc::new(Mutex::new(HashMap::new())),
            hook_registry: Arc::new(Mutex::new(HashMap::new())),
            selection: Arc::new(Mutex::new(None)),
            cwd: Arc::new(Mutex::new(PathBuf::from("."))),
            event_emitter: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Write data to terminal
    pub fn terminal_write(&self, data: &[u8]) {
        if let Ok(mut writer) = self.terminal_writer.lock() {
            writer.extend_from_slice(data);
        }
    }

    /// Read data from terminal
    pub fn terminal_read(&self) -> Vec<u8> {
        self.terminal_reader
            .lock()
            .map(|reader| reader.clone())
            .unwrap_or_default()
    }

    /// Get configuration value
    pub fn get_config(&self, key: &str) -> Option<String> {
        self.config_store
            .lock()
            .ok()
            .and_then(|store| store.get(key).cloned())
    }

    /// Set configuration value
    pub fn set_config(&self, key: &str, value: &str) {
        if let Ok(mut store) = self.config_store.lock() {
            store.insert(key.to_string(), value.to_string());
        }
    }

    /// Log a message
    pub fn log(&self, level: LogLevel, message: &str) {
        (self.logger)(level, message);
    }

    /// Show a notification
    pub fn show_notification(&self, notification: Notification) {
        if let Ok(notifier) = self.notifier.lock() {
            if let Some(nm) = notifier.as_ref() {
                nm.notify_custom(&notification.title, &notification.body);
            }
        }
    }

    /// Register a command handler
    pub fn register_command(&self, name: &str, handler: CommandHandler) {
        if let Ok(mut registry) = self.command_registry.lock() {
            registry.insert(name.to_string(), handler);
        }
    }

    /// Register a hook handler
    pub fn register_hook(&self, hook: HookType, handler: HookHandler) {
        if let Ok(mut registry) = self.hook_registry.lock() {
            let hook_key = format!("{:?}", hook);
            registry.entry(hook_key).or_insert_with(Vec::new).push(handler);
        }
    }

    /// Get current selection
    pub fn get_selection(&self) -> Option<String> {
        self.selection.lock().ok().and_then(|sel| sel.clone())
    }

    /// Get current working directory
    pub fn get_cwd(&self) -> PathBuf {
        self.cwd.lock().map(|cwd| cwd.clone()).unwrap_or_else(|_| PathBuf::from("."))
    }

    /// Emit a plugin event
    pub fn emit_event(&self, event: PluginEvent) {
        if let Ok(mut emitter) = self.event_emitter.lock() {
            emitter.push(event);
        }
    }

    /// Get emitted events (for testing)
    pub fn get_events(&self) -> Vec<PluginEvent> {
        self.event_emitter
            .lock()
            .map(|events| events.clone())
            .unwrap_or_default()
    }
}

impl Default for PluginContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Error types for plugin operations
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(PluginId),

    #[error("Failed to load plugin: {0}")]
    LoadError(String),

    #[error("Failed to activate plugin: {0}")]
    ActivationError(String),

    #[error("Permission denied: {0:?}")]
    PermissionDenied(Permission),

    #[error("Missing dependency: {0}")]
    DependencyMissing(String),

    #[error("Version mismatch: required {required}, actual {actual}")]
    VersionMismatch { required: String, actual: String },

    #[error("Plugin execution error: {0}")]
    ExecutionError(String),

    #[error("Invalid plugin state: expected {expected}, got {actual}")]
    InvalidState { expected: String, actual: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Trait that all plugins must implement
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Activate the plugin
    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError>;

    /// Deactivate the plugin
    fn deactivate(&mut self) -> Result<(), PluginError>;

    /// Handle an event
    fn on_event(&mut self, event: &PluginEvent) -> Result<(), PluginError>;

    /// Handle a command
    fn on_command(&mut self, command: &str, args: &[String]) -> Result<String, PluginError>;
}

/// Manages all plugins
pub struct PluginManager {
    plugins: HashMap<PluginId, Box<dyn Plugin>>,
    states: HashMap<PluginId, PluginState>,
    registry: PluginRegistry,
    context: PluginContext,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            states: HashMap::new(),
            registry: PluginRegistry::new(),
            context: PluginContext::new(),
        }
    }

    /// Create a plugin manager with a specific plugin directory
    pub fn with_plugins_dir(plugins_dir: PathBuf) -> Self {
        Self {
            plugins: HashMap::new(),
            states: HashMap::new(),
            registry: PluginRegistry::with_dir(plugins_dir),
            context: PluginContext::new(),
        }
    }

    /// Load a plugin from a path
    pub fn load_plugin(&mut self, _path: &Path) -> Result<PluginId, PluginError> {
        // In a real implementation, this would load a dynamic library or script
        // For now, return an error indicating dynamic loading is not yet implemented
        Err(PluginError::LoadError("Dynamic plugin loading not yet implemented".to_string()))
    }

    /// Register a plugin directly (for builtin plugins)
    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) -> PluginId {
        let id = plugin.metadata().id;
        self.states.insert(id, PluginState::Loaded);
        self.plugins.insert(id, plugin);
        id
    }

    /// Unload a plugin
    pub fn unload_plugin(&mut self, id: PluginId) -> Result<(), PluginError> {
        // Deactivate first if active
        if let Some(state) = self.states.get(&id) {
            if *state == PluginState::Active {
                self.deactivate_plugin(id)?;
            }
        }

        self.plugins.remove(&id);
        self.states.insert(id, PluginState::Unloaded);
        Ok(())
    }

    /// Activate a plugin
    pub fn activate_plugin(&mut self, id: PluginId) -> Result<(), PluginError> {
        let state = self.states.get(&id).cloned();

        match state {
            Some(PluginState::Loaded) | Some(PluginState::Inactive) => {
                // First, collect metadata without holding mutable reference
                let (permissions, dependencies) = if let Some(plugin) = self.plugins.get(&id) {
                    let metadata = plugin.metadata();
                    (metadata.permissions.clone(), metadata.dependencies.clone())
                } else {
                    return Err(PluginError::NotFound(id));
                };

                // Check permissions
                for permission in &permissions {
                    if !self.check_permissions(id, permission) {
                        return Err(PluginError::PermissionDenied(permission.clone()));
                    }
                }

                // Check dependencies
                for dep in &dependencies {
                    if !self.is_dependency_satisfied(dep) {
                        return Err(PluginError::DependencyMissing(dep.plugin_name.clone()));
                    }
                }

                // Now activate the plugin
                if let Some(plugin) = self.plugins.get_mut(&id) {
                    plugin.activate(&self.context)?;
                    self.states.insert(id, PluginState::Active);
                    Ok(())
                } else {
                    Err(PluginError::NotFound(id))
                }
            }
            Some(PluginState::Active) => Ok(()), // Already active
            Some(state) => Err(PluginError::InvalidState {
                expected: "Loaded or Inactive".to_string(),
                actual: format!("{:?}", state),
            }),
            None => Err(PluginError::NotFound(id)),
        }
    }

    /// Deactivate a plugin
    pub fn deactivate_plugin(&mut self, id: PluginId) -> Result<(), PluginError> {
        let state = self.states.get(&id).cloned();

        match state {
            Some(PluginState::Active) => {
                if let Some(plugin) = self.plugins.get_mut(&id) {
                    plugin.deactivate()?;
                    self.states.insert(id, PluginState::Inactive);
                    Ok(())
                } else {
                    Err(PluginError::NotFound(id))
                }
            }
            Some(PluginState::Inactive) => Ok(()), // Already inactive
            Some(state) => Err(PluginError::InvalidState {
                expected: "Active".to_string(),
                actual: format!("{:?}", state),
            }),
            None => Err(PluginError::NotFound(id)),
        }
    }

    /// Get a reference to a plugin
    pub fn get_plugin(&self, id: PluginId) -> Option<&dyn Plugin> {
        self.plugins.get(&id).map(|p| &**p)
    }

    /// Get plugin state
    pub fn get_state(&self, id: PluginId) -> Option<&PluginState> {
        self.states.get(&id)
    }

    /// List all plugins
    pub fn list_plugins(&self) -> Vec<&PluginMetadata> {
        self.plugins.values().map(|p| p.metadata()).collect()
    }

    /// Dispatch an event to all active plugins
    pub fn dispatch_event(&mut self, event: PluginEvent) {
        let active_ids: Vec<PluginId> = self
            .states
            .iter()
            .filter(|(_, state)| **state == PluginState::Active)
            .map(|(id, _)| *id)
            .collect();

        for id in active_ids {
            if let Some(plugin) = self.plugins.get_mut(&id) {
                if let Err(e) = plugin.on_event(&event) {
                    tracing::error!("Plugin {} event error: {}", id, e);
                    self.states.insert(id, PluginState::Error(e.to_string()));
                }
            }
        }
    }

    /// Dispatch a command to a specific plugin
    pub fn dispatch_command(
        &mut self,
        plugin_id: PluginId,
        command: &str,
        args: &[String],
    ) -> Result<String, PluginError> {
        let state = self.states.get(&plugin_id).cloned();

        match state {
            Some(PluginState::Active) => {
                if let Some(plugin) = self.plugins.get_mut(&plugin_id) {
                    plugin.on_command(command, args)
                } else {
                    Err(PluginError::NotFound(plugin_id))
                }
            }
            Some(state) => Err(PluginError::InvalidState {
                expected: "Active".to_string(),
                actual: format!("{:?}", state),
            }),
            None => Err(PluginError::NotFound(plugin_id)),
        }
    }

    /// Check if a permission is granted
    pub fn check_permissions(&self, _id: PluginId, _permission: &Permission) -> bool {
        // In a real implementation, this would check against a permission database
        // For now, allow all permissions (permissive for development)
        true
    }

    /// Check if a dependency is satisfied
    fn is_dependency_satisfied(&self, dep: &PluginDependency) -> bool {
        self.plugins.values().any(|plugin| {
            let metadata = plugin.metadata();
            metadata.name == dep.plugin_name
                && version_matches(&metadata.version, &dep.version_requirement)
        })
    }

    /// Get the plugin context
    pub fn context(&self) -> &PluginContext {
        &self.context
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry for discovering plugins
pub struct PluginRegistry {
    plugins_dir: PathBuf,
}

impl PluginRegistry {
    /// Create a new registry with default plugins directory
    pub fn new() -> Self {
        let plugins_dir = dirs::config_dir()
            .map(|d| d.join("agterm").join("plugins"))
            .unwrap_or_else(|| PathBuf::from("plugins"));

        Self { plugins_dir }
    }

    /// Create a registry with a specific plugins directory
    pub fn with_dir(plugins_dir: PathBuf) -> Self {
        Self { plugins_dir }
    }

    /// Scan for available plugins
    pub fn scan(&self) -> Vec<PluginMetadata> {
        let mut plugins = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&self.plugins_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = self.load_metadata(entry.path()) {
                    plugins.push(metadata);
                }
            }
        }

        plugins
    }

    /// Get metadata for a specific plugin by name
    pub fn get_metadata(&self, name: &str) -> Option<PluginMetadata> {
        self.scan().into_iter().find(|m| m.name == name)
    }

    /// Load metadata from a plugin directory
    fn load_metadata(&self, path: PathBuf) -> Result<PluginMetadata, PluginError> {
        let manifest_path = path.join("plugin.json");
        let content = std::fs::read_to_string(manifest_path)?;
        let metadata: PluginMetadata = serde_json::from_str(&content)?;
        Ok(metadata)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a version matches a requirement
fn version_matches(version: &str, requirement: &str) -> bool {
    // Simplified version matching - in production, use semver crate
    // For now, just check exact match or >= for caret requirements
    if requirement.starts_with('^') {
        let req_version = requirement.trim_start_matches('^');
        version >= req_version
    } else if requirement.starts_with(">=") {
        let req_version = requirement.trim_start_matches(">=").trim();
        version >= req_version
    } else {
        version == requirement
    }
}

/// Example builtin plugin demonstrating the Plugin trait
pub struct BuiltinPlugin {
    metadata: PluginMetadata,
    active: bool,
}

impl BuiltinPlugin {
    /// Create a new builtin plugin
    pub fn new() -> Self {
        let metadata = PluginMetadata::new("builtin-example", "1.0.0", "AgTerm Team")
            .with_description("Example builtin plugin demonstrating the Plugin trait")
            .with_permissions(vec![
                Permission::Terminal(TerminalPermission::Read),
                Permission::Terminal(TerminalPermission::Write),
            ])
            .with_entry_point("builtin");

        Self {
            metadata,
            active: false,
        }
    }
}

impl Default for BuiltinPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for BuiltinPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.log(LogLevel::Info, "Builtin plugin activated");

        // Register a sample command
        let cmd_handler: CommandHandler = Arc::new(|args| {
            Ok(format!("Builtin command executed with {} args", args.len()))
        });
        ctx.register_command("builtin-test", cmd_handler);

        // Register a sample hook
        let hook_handler: HookHandler = Arc::new(|hook_ctx| {
            tracing::debug!("Hook triggered: {}", hook_ctx.data);
            Ok(())
        });
        ctx.register_hook(HookType::OnStartup, hook_handler);

        self.active = true;
        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), PluginError> {
        self.active = false;
        Ok(())
    }

    fn on_event(&mut self, event: &PluginEvent) -> Result<(), PluginError> {
        match event {
            PluginEvent::Custom(name, _) => {
                tracing::debug!("Received custom event: {}", name);
            }
            PluginEvent::CommandResult(cmd, success) => {
                tracing::debug!("Command '{}' {}", cmd, if *success { "succeeded" } else { "failed" });
            }
            PluginEvent::OutputProcessed(output) => {
                tracing::trace!("Output processed: {} bytes", output.len());
            }
        }
        Ok(())
    }

    fn on_command(&mut self, command: &str, args: &[String]) -> Result<String, PluginError> {
        match command {
            "echo" => Ok(args.join(" ")),
            "status" => Ok(if self.active { "active" } else { "inactive" }.to_string()),
            _ => Err(PluginError::ExecutionError(format!("Unknown command: {}", command))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_id_creation() {
        let id1 = PluginId::new();
        let id2 = PluginId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_plugin_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let id = PluginId::from_uuid(uuid);
        assert_eq!(id.as_uuid(), uuid);
    }

    #[test]
    fn test_plugin_metadata_builder() {
        let metadata = PluginMetadata::new("test", "1.0.0", "author")
            .with_description("Test plugin")
            .with_entry_point("main.rs");

        assert_eq!(metadata.name, "test");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.author, "author");
        assert_eq!(metadata.description, "Test plugin");
        assert_eq!(metadata.entry_point, "main.rs");
    }

    #[test]
    fn test_plugin_dependency_creation() {
        let dep = PluginDependency::new("other-plugin", "^1.0.0");
        assert_eq!(dep.plugin_name, "other-plugin");
        assert_eq!(dep.version_requirement, "^1.0.0");
    }

    #[test]
    fn test_permission_equality() {
        let perm1 = Permission::Clipboard;
        let perm2 = Permission::Clipboard;
        let perm3 = Permission::SystemShell;

        assert_eq!(perm1, perm2);
        assert_ne!(perm1, perm3);
    }

    #[test]
    fn test_path_scope_equality() {
        let scope1 = PathScope::ReadOnly(PathBuf::from("/tmp"));
        let scope2 = PathScope::ReadOnly(PathBuf::from("/tmp"));
        let scope3 = PathScope::ReadWrite(PathBuf::from("/tmp"));

        assert_eq!(scope1, scope2);
        assert_ne!(scope1, scope3);
    }

    #[test]
    fn test_network_scope() {
        let localhost = NetworkScope::LocalhostOnly;
        let specific = NetworkScope::Specific(vec!["example.com".to_string()]);
        let anywhere = NetworkScope::Anywhere;

        assert_ne!(localhost, specific);
        assert_ne!(localhost, anywhere);
    }

    #[test]
    fn test_plugin_state() {
        let state1 = PluginState::Loaded;
        let state2 = PluginState::Active;
        let state3 = PluginState::Error("test error".to_string());

        assert_ne!(state1, state2);
        assert_ne!(state2, state3);
    }

    #[test]
    fn test_key_pattern_builder() {
        let pattern = KeyPattern::new("a")
            .with_ctrl()
            .with_shift();

        assert_eq!(pattern.key, "a");
        assert!(pattern.ctrl);
        assert!(pattern.shift);
        assert!(!pattern.alt);
        assert!(!pattern.meta);
    }

    #[test]
    fn test_notification_builder() {
        let notif = Notification::new("Title", "Body").with_timeout(10);
        assert_eq!(notif.title, "Title");
        assert_eq!(notif.body, "Body");
        assert_eq!(notif.timeout_seconds, 10);
    }

    #[test]
    fn test_plugin_context_creation() {
        let ctx = PluginContext::new();
        assert!(ctx.get_config("nonexistent").is_none());
    }

    #[test]
    fn test_plugin_context_config() {
        let ctx = PluginContext::new();
        ctx.set_config("key", "value");
        assert_eq!(ctx.get_config("key"), Some("value".to_string()));
    }

    #[test]
    fn test_plugin_context_terminal_write() {
        let ctx = PluginContext::new();
        ctx.terminal_write(b"hello");
        // In real implementation, this would write to terminal
    }

    #[test]
    fn test_plugin_context_logging() {
        let ctx = PluginContext::new();
        ctx.log(LogLevel::Info, "test message");
        // Should not panic
    }

    #[test]
    fn test_plugin_context_event_emission() {
        let ctx = PluginContext::new();
        let event = PluginEvent::Custom("test".to_string(), serde_json::json!({}));
        ctx.emit_event(event.clone());

        let events = ctx.get_events();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert_eq!(manager.list_plugins().len(), 0);
    }

    #[test]
    fn test_builtin_plugin_creation() {
        let plugin = BuiltinPlugin::new();
        assert_eq!(plugin.metadata().name, "builtin-example");
        assert_eq!(plugin.metadata().version, "1.0.0");
    }

    #[test]
    fn test_builtin_plugin_lifecycle() {
        let mut plugin = BuiltinPlugin::new();
        let ctx = PluginContext::new();

        assert!(!plugin.active);
        assert!(plugin.activate(&ctx).is_ok());
        assert!(plugin.active);
        assert!(plugin.deactivate().is_ok());
        assert!(!plugin.active);
    }

    #[test]
    fn test_builtin_plugin_commands() {
        let mut plugin = BuiltinPlugin::new();
        let ctx = PluginContext::new();
        plugin.activate(&ctx).unwrap();

        let result = plugin.on_command("echo", &["hello".to_string(), "world".to_string()]);
        assert_eq!(result.unwrap(), "hello world");

        let result = plugin.on_command("status", &[]);
        assert_eq!(result.unwrap(), "active");

        let result = plugin.on_command("unknown", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_builtin_plugin_events() {
        let mut plugin = BuiltinPlugin::new();
        let event = PluginEvent::CommandResult("test".to_string(), true);
        assert!(plugin.on_event(&event).is_ok());
    }

    #[test]
    fn test_plugin_manager_register() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(BuiltinPlugin::new());
        let id = plugin.metadata().id;

        manager.register_plugin(plugin);
        assert_eq!(manager.list_plugins().len(), 1);
        assert_eq!(manager.get_state(id), Some(&PluginState::Loaded));
    }

    #[test]
    fn test_plugin_manager_activate() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(BuiltinPlugin::new());
        let id = plugin.metadata().id;

        manager.register_plugin(plugin);
        assert!(manager.activate_plugin(id).is_ok());
        assert_eq!(manager.get_state(id), Some(&PluginState::Active));
    }

    #[test]
    fn test_plugin_manager_deactivate() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(BuiltinPlugin::new());
        let id = plugin.metadata().id;

        manager.register_plugin(plugin);
        manager.activate_plugin(id).unwrap();
        assert!(manager.deactivate_plugin(id).is_ok());
        assert_eq!(manager.get_state(id), Some(&PluginState::Inactive));
    }

    #[test]
    fn test_plugin_manager_unload() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(BuiltinPlugin::new());
        let id = plugin.metadata().id;

        manager.register_plugin(plugin);
        assert!(manager.unload_plugin(id).is_ok());
        assert_eq!(manager.get_state(id), Some(&PluginState::Unloaded));
    }

    #[test]
    fn test_plugin_manager_dispatch_event() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(BuiltinPlugin::new());
        let id = plugin.metadata().id;

        manager.register_plugin(plugin);
        manager.activate_plugin(id).unwrap();

        let event = PluginEvent::Custom("test".to_string(), serde_json::json!({"data": "value"}));
        manager.dispatch_event(event);
        // Should not panic
    }

    #[test]
    fn test_plugin_manager_dispatch_command() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(BuiltinPlugin::new());
        let id = plugin.metadata().id;

        manager.register_plugin(plugin);
        manager.activate_plugin(id).unwrap();

        let result = manager.dispatch_command(id, "echo", &["test".to_string()]);
        assert_eq!(result.unwrap(), "test");
    }

    #[test]
    fn test_plugin_manager_dispatch_command_inactive() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(BuiltinPlugin::new());
        let id = plugin.metadata().id;

        manager.register_plugin(plugin);

        let result = manager.dispatch_command(id, "echo", &["test".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_plugin_registry_creation() {
        let registry = PluginRegistry::new();
        // Should not panic
        let _ = registry.scan();
    }

    #[test]
    fn test_version_matches_exact() {
        assert!(version_matches("1.0.0", "1.0.0"));
        assert!(!version_matches("1.0.0", "2.0.0"));
    }

    #[test]
    fn test_version_matches_caret() {
        assert!(version_matches("1.5.0", "^1.0.0"));
        assert!(version_matches("1.0.0", "^1.0.0"));
        assert!(!version_matches("0.9.0", "^1.0.0"));
    }

    #[test]
    fn test_version_matches_gte() {
        assert!(version_matches("2.0.0", ">=1.0.0"));
        assert!(version_matches("1.0.0", ">=1.0.0"));
        assert!(!version_matches("0.9.0", ">=1.0.0"));
    }

    #[test]
    fn test_hook_context_creation() {
        let ctx = HookContext::new("test data")
            .with_metadata("key", "value");

        assert_eq!(ctx.data, "test data");
        assert_eq!(ctx.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_plugin_error_display() {
        let error = PluginError::NotFound(PluginId::new());
        assert!(error.to_string().contains("Plugin not found"));

        let error = PluginError::LoadError("test".to_string());
        assert!(error.to_string().contains("Failed to load plugin"));

        let error = PluginError::PermissionDenied(Permission::Clipboard);
        assert!(error.to_string().contains("Permission denied"));
    }
}
