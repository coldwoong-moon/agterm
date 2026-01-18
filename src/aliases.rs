//! Command alias management system for AgTerm
//!
//! Provides a powerful alias system with:
//! - Alias definitions with metadata (name, command, description, category)
//! - Parameter substitution ($1, $2, ${*}, etc.)
//! - Nested alias expansion
//! - Category-based organization
//! - Shell integration (import/export zsh/bash aliases)
//! - CRUD operations for alias management
//! - File persistence (JSON)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Represents a command alias with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Alias {
    /// Alias name (short form)
    pub name: String,
    /// Full command to execute
    pub command: String,
    /// Human-readable description
    pub description: Option<String>,
    /// Category for grouping (e.g., "git", "docker", "file")
    pub category: Option<String>,
    /// When this alias was created
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    /// Whether this alias is active
    pub enabled: bool,
}

impl Alias {
    /// Create a new alias
    pub fn new(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
            description: None,
            category: None,
            created_at: Utc::now(),
            enabled: true,
        }
    }

    /// Create an alias with description
    pub fn with_description(
        name: impl Into<String>,
        command: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
            description: Some(description.into()),
            category: None,
            created_at: Utc::now(),
            enabled: true,
        }
    }

    /// Set the category
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Set enabled state
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Manages command aliases with expansion and persistence
#[derive(Debug, Clone)]
pub struct AliasManager {
    /// All aliases stored by name
    aliases: HashMap<String, Alias>,
    /// Category index for fast lookup
    category_index: HashMap<String, Vec<String>>,
    /// Path to persistence file
    file_path: Option<PathBuf>,
    /// Maximum recursion depth for nested aliases
    max_recursion_depth: usize,
}

impl AliasManager {
    /// Create a new alias manager
    pub fn new() -> Self {
        Self {
            aliases: HashMap::new(),
            category_index: HashMap::new(),
            file_path: None,
            max_recursion_depth: 10,
        }
    }

    /// Create an alias manager with default aliases
    pub fn with_defaults() -> Self {
        let mut manager = Self::new();

        // File operations
        let _ = manager.add_alias(
            "ll",
            "ls -lah",
            Some("List all files with details"),
        );
        let _ = manager.add_alias(
            "la",
            "ls -A",
            Some("List all files including hidden"),
        );
        let _ = manager.add_alias(
            "l",
            "ls -CF",
            Some("List files with indicators"),
        );
        let _ = manager.add_alias(
            "...",
            "cd ../..",
            Some("Go up two directories"),
        );
        let _ = manager.add_alias(
            "....",
            "cd ../../..",
            Some("Go up three directories"),
        );

        // Git aliases
        let _ = manager.add_alias(
            "gs",
            "git status",
            Some("Git status"),
        );
        let _ = manager.add_alias(
            "ga",
            "git add",
            Some("Git add"),
        );
        let _ = manager.add_alias(
            "gc",
            "git commit -m",
            Some("Git commit with message"),
        );
        let _ = manager.add_alias(
            "gp",
            "git push",
            Some("Git push"),
        );
        let _ = manager.add_alias(
            "gl",
            "git pull",
            Some("Git pull"),
        );
        let _ = manager.add_alias(
            "gco",
            "git checkout",
            Some("Git checkout"),
        );
        let _ = manager.add_alias(
            "gb",
            "git branch",
            Some("Git branch"),
        );
        let _ = manager.add_alias(
            "gd",
            "git diff",
            Some("Git diff"),
        );
        let _ = manager.add_alias(
            "glog",
            "git log --oneline --graph --all",
            Some("Git log with graph"),
        );

        // Docker aliases
        let _ = manager.add_alias(
            "dps",
            "docker ps",
            Some("List running containers"),
        );
        let _ = manager.add_alias(
            "dpsa",
            "docker ps -a",
            Some("List all containers"),
        );
        let _ = manager.add_alias(
            "dim",
            "docker images",
            Some("List docker images"),
        );
        let _ = manager.add_alias(
            "drm",
            "docker rm",
            Some("Remove container"),
        );
        let _ = manager.add_alias(
            "drmi",
            "docker rmi",
            Some("Remove image"),
        );
        let _ = manager.add_alias(
            "dex",
            "docker exec -it",
            Some("Execute command in container"),
        );

        // Cargo/Rust aliases
        let _ = manager.add_alias(
            "cb",
            "cargo build",
            Some("Cargo build"),
        );
        let _ = manager.add_alias(
            "cr",
            "cargo run",
            Some("Cargo run"),
        );
        let _ = manager.add_alias(
            "ct",
            "cargo test",
            Some("Cargo test"),
        );
        let _ = manager.add_alias(
            "cc",
            "cargo check",
            Some("Cargo check"),
        );
        let _ = manager.add_alias(
            "cbr",
            "cargo build --release",
            Some("Cargo build release"),
        );

        // System aliases
        let _ = manager.add_alias(
            "h",
            "history",
            Some("Show command history"),
        );
        let _ = manager.add_alias(
            "c",
            "clear",
            Some("Clear terminal"),
        );
        let _ = manager.add_alias(
            "grep",
            "grep --color=auto",
            Some("Grep with color"),
        );

        // Set categories for defaults
        for name in &["ll", "la", "l", "...", "...."] {
            if let Some(alias) = manager.aliases.get_mut(*name) {
                alias.category = Some("file".to_string());
            }
        }
        for name in &["gs", "ga", "gc", "gp", "gl", "gco", "gb", "gd", "glog"] {
            if let Some(alias) = manager.aliases.get_mut(*name) {
                alias.category = Some("git".to_string());
            }
        }
        for name in &["dps", "dpsa", "dim", "drm", "drmi", "dex"] {
            if let Some(alias) = manager.aliases.get_mut(*name) {
                alias.category = Some("docker".to_string());
            }
        }
        for name in &["cb", "cr", "ct", "cc", "cbr"] {
            if let Some(alias) = manager.aliases.get_mut(*name) {
                alias.category = Some("rust".to_string());
            }
        }
        for name in &["h", "c", "grep"] {
            if let Some(alias) = manager.aliases.get_mut(*name) {
                alias.category = Some("system".to_string());
            }
        }

        // Rebuild category index
        manager.rebuild_category_index();

        manager
    }

    /// Add a new alias
    pub fn add_alias(
        &mut self,
        name: impl Into<String>,
        command: impl Into<String>,
        description: Option<impl Into<String>>,
    ) -> Result<(), AliasError> {
        let name = name.into();
        let command = command.into();

        if name.is_empty() {
            return Err(AliasError::InvalidName("Alias name cannot be empty".into()));
        }

        if command.is_empty() {
            return Err(AliasError::InvalidCommand("Command cannot be empty".into()));
        }

        if self.aliases.contains_key(&name) {
            return Err(AliasError::AlreadyExists(name));
        }

        let mut alias = if let Some(desc) = description {
            Alias::with_description(&name, &command, desc.into())
        } else {
            Alias::new(&name, &command)
        };

        // Auto-detect category if not set
        if alias.category.is_none() {
            alias.category = Self::detect_category(&command);
        }

        // Update category index
        if let Some(ref category) = alias.category {
            self.category_index
                .entry(category.clone())
                .or_default()
                .push(name.clone());
        }

        self.aliases.insert(name, alias);
        Ok(())
    }

    /// Remove an alias by name
    pub fn remove_alias(&mut self, name: &str) -> Result<Alias, AliasError> {
        let alias = self
            .aliases
            .remove(name)
            .ok_or_else(|| AliasError::NotFound(name.to_string()))?;

        // Remove from category index
        if let Some(ref category) = alias.category {
            if let Some(names) = self.category_index.get_mut(category) {
                names.retain(|n| n != name);
            }
        }

        Ok(alias)
    }

    /// Update an existing alias
    pub fn update_alias(
        &mut self,
        name: &str,
        new_command: impl Into<String>,
    ) -> Result<(), AliasError> {
        let new_command = new_command.into();

        if new_command.is_empty() {
            return Err(AliasError::InvalidCommand("Command cannot be empty".into()));
        }

        let alias = self
            .aliases
            .get_mut(name)
            .ok_or_else(|| AliasError::NotFound(name.to_string()))?;

        alias.command = new_command;
        Ok(())
    }

    /// Get an alias by name
    pub fn get_alias(&self, name: &str) -> Option<&Alias> {
        self.aliases.get(name)
    }

    /// Get a mutable alias by name
    pub fn get_alias_mut(&mut self, name: &str) -> Option<&mut Alias> {
        self.aliases.get_mut(name)
    }

    /// List all aliases
    pub fn list_aliases(&self) -> Vec<&Alias> {
        let mut aliases: Vec<_> = self.aliases.values().collect();
        aliases.sort_by(|a, b| a.name.cmp(&b.name));
        aliases
    }

    /// List only enabled aliases
    pub fn list_enabled_aliases(&self) -> Vec<&Alias> {
        let mut aliases: Vec<_> = self.aliases.values().filter(|a| a.enabled).collect();
        aliases.sort_by(|a, b| a.name.cmp(&b.name));
        aliases
    }

    /// Get aliases by category
    pub fn get_by_category(&self, category: &str) -> Vec<&Alias> {
        self.category_index
            .get(category)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|name| self.aliases.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all categories
    pub fn get_categories(&self) -> Vec<String> {
        let mut categories: Vec<_> = self.category_index.keys().cloned().collect();
        categories.sort();
        categories
    }

    /// Toggle an alias on/off
    pub fn toggle_alias(&mut self, name: &str) -> Result<bool, AliasError> {
        let alias = self
            .aliases
            .get_mut(name)
            .ok_or_else(|| AliasError::NotFound(name.to_string()))?;

        alias.enabled = !alias.enabled;
        Ok(alias.enabled)
    }

    /// Expand a command by replacing aliases and substituting parameters
    pub fn expand_command(&self, input: &str) -> String {
        self.expand_command_recursive(input, 0)
    }

    /// Expand command with recursion depth tracking
    fn expand_command_recursive(&self, input: &str, depth: usize) -> String {
        if depth >= self.max_recursion_depth {
            tracing::warn!("Max recursion depth reached while expanding aliases");
            return input.to_string();
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return input.to_string();
        }

        let first_word = parts[0];
        let args = &parts[1..];

        // Check if first word is an alias
        if let Some(alias) = self.aliases.get(first_word) {
            if !alias.enabled {
                return input.to_string();
            }

            // Expand the alias command
            let expanded = self.substitute_parameters(&alias.command, args);

            // Recursively expand in case the expanded command contains more aliases
            self.expand_command_recursive(&expanded, depth + 1)
        } else {
            input.to_string()
        }
    }

    /// Substitute parameters ($1, $2, ${*}, etc.) in a command
    fn substitute_parameters(&self, command: &str, args: &[&str]) -> String {
        let mut result = command.to_string();

        // Replace ${*} with all arguments
        if result.contains("${*}") {
            result = result.replace("${*}", &args.join(" "));
        }

        // Replace $* with all arguments (alternative syntax)
        if result.contains("$*") {
            result = result.replace("$*", &args.join(" "));
        }

        // Replace ${@} with all arguments
        if result.contains("${@}") {
            result = result.replace("${@}", &args.join(" "));
        }

        // Replace $@ with all arguments (alternative syntax)
        if result.contains("$@") {
            result = result.replace("$@", &args.join(" "));
        }

        // Replace $# with argument count
        let arg_count = args.len().to_string();
        result = result.replace("$#", &arg_count);

        // Replace numbered parameters ($1, $2, etc.)
        for (i, arg) in args.iter().enumerate() {
            let param_num = i + 1;
            result = result.replace(&format!("${{{}}}", param_num), arg);
            result = result.replace(&format!("${}", param_num), arg);
        }

        // Append remaining arguments if no parameter substitution occurred
        if !command.contains('$') && !args.is_empty() {
            result.push(' ');
            result.push_str(&args.join(" "));
        }

        result
    }

    /// Save aliases to file
    pub fn save_to_file(&mut self, path: impl Into<PathBuf>) -> Result<(), AliasError> {
        let path = path.into();

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AliasError::IoError(format!("Failed to create directory: {e}")))?;
        }

        let aliases: Vec<_> = self.aliases.values().collect();
        let json = serde_json::to_string_pretty(&aliases)
            .map_err(|e| AliasError::SerializationError(e.to_string()))?;

        fs::write(&path, json)
            .map_err(|e| AliasError::IoError(format!("Failed to write file: {e}")))?;

        self.file_path = Some(path.clone());
        tracing::info!("Saved {} aliases to {:?}", aliases.len(), path);
        Ok(())
    }

    /// Save to the previously loaded file
    pub fn save(&self) -> Result<(), AliasError> {
        if let Some(ref path) = self.file_path {
            // Create parent directory if needed
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| AliasError::IoError(format!("Failed to create directory: {e}")))?;
            }

            let aliases: Vec<_> = self.aliases.values().collect();
            let json = serde_json::to_string_pretty(&aliases)
                .map_err(|e| AliasError::SerializationError(e.to_string()))?;

            fs::write(path, json)
                .map_err(|e| AliasError::IoError(format!("Failed to write file: {e}")))?;

            tracing::info!("Saved {} aliases to {:?}", aliases.len(), path);
            Ok(())
        } else {
            Err(AliasError::IoError("No file path set".to_string()))
        }
    }

    /// Load aliases from file
    pub fn load_from_file(&mut self, path: impl Into<PathBuf>) -> Result<(), AliasError> {
        let path = path.into();

        if !path.exists() {
            return Err(AliasError::IoError(format!("File not found: {:?}", path)));
        }

        let json = fs::read_to_string(&path)
            .map_err(|e| AliasError::IoError(format!("Failed to read file: {e}")))?;

        let aliases: Vec<Alias> = serde_json::from_str(&json)
            .map_err(|e| AliasError::SerializationError(e.to_string()))?;

        for alias in aliases {
            let name = alias.name.clone();
            if let Some(ref category) = alias.category {
                self.category_index
                    .entry(category.clone())
                    .or_default()
                    .push(name.clone());
            }
            self.aliases.insert(name, alias);
        }

        self.file_path = Some(path.clone());
        tracing::info!("Loaded {} aliases from {:?}", self.aliases.len(), path);
        Ok(())
    }

    /// Export aliases in shell format (bash/zsh compatible)
    pub fn export_to_shell(&self) -> String {
        let mut output = String::new();
        output.push_str("# AgTerm Aliases\n");
        output.push_str("# Generated by AgTerm alias manager\n\n");

        let mut aliases: Vec<_> = self.aliases.values().collect();
        aliases.sort_by(|a, b| {
            match (&a.category, &b.category) {
                (Some(ca), Some(cb)) => ca.cmp(cb).then_with(|| a.name.cmp(&b.name)),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.name.cmp(&b.name),
            }
        });

        let mut current_category: Option<&String> = None;
        for alias in aliases {
            if !alias.enabled {
                continue;
            }

            // Add category header
            if alias.category.as_ref() != current_category {
                if current_category.is_some() {
                    output.push('\n');
                }
                if let Some(ref category) = alias.category {
                    output.push_str(&format!("# {} aliases\n", category));
                }
                current_category = alias.category.as_ref();
            }

            // Add description if available
            if let Some(ref desc) = alias.description {
                output.push_str(&format!("# {}\n", desc));
            }

            // Add alias definition
            output.push_str(&format!("alias {}='{}'\n", alias.name, alias.command));
        }

        output
    }

    /// Import aliases from shell format (bash/zsh)
    pub fn import_from_shell(&mut self, shell_content: &str) -> Result<usize, AliasError> {
        let mut imported = 0;
        let mut current_description: Option<String> = None;

        for line in shell_content.lines() {
            let line = line.trim();

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Parse comments for descriptions
            if let Some(comment) = line.strip_prefix('#') {
                let comment = comment.trim();
                if !comment.is_empty()
                    && !comment.contains("aliases")
                    && !comment.starts_with("Generated")
                {
                    current_description = Some(comment.to_string());
                }
                continue;
            }

            // Parse alias definition
            if let Some(alias_def) = line.strip_prefix("alias ") {
                if let Some((name, command)) = Self::parse_shell_alias(alias_def) {
                    let result = self.add_alias(name, command, current_description.as_ref());
                    if result.is_ok() {
                        imported += 1;
                    }
                    current_description = None;
                }
            }
        }

        tracing::info!("Imported {} aliases from shell", imported);
        Ok(imported)
    }

    /// Parse a shell alias definition (e.g., "name='command'")
    fn parse_shell_alias(definition: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = definition.splitn(2, '=').collect();
        if parts.len() != 2 {
            return None;
        }

        let name = parts[0].trim();
        let command = parts[1].trim();

        // Remove quotes from command
        let command = command
            .strip_prefix('\'')
            .and_then(|s| s.strip_suffix('\''))
            .or_else(|| {
                command
                    .strip_prefix('"')
                    .and_then(|s| s.strip_suffix('"'))
            })
            .unwrap_or(command);

        Some((name.to_string(), command.to_string()))
    }

    /// Detect category based on command content
    fn detect_category(command: &str) -> Option<String> {
        if command.starts_with("git ") {
            Some("git".to_string())
        } else if command.starts_with("docker ") || command.starts_with("docker-compose ") {
            Some("docker".to_string())
        } else if command.starts_with("cargo ") {
            Some("rust".to_string())
        } else if command.starts_with("npm ") || command.starts_with("yarn ") || command.starts_with("pnpm ") {
            Some("nodejs".to_string())
        } else if command.starts_with("cd ") || command.starts_with("ls ") {
            Some("file".to_string())
        } else {
            Some("system".to_string())
        }
    }

    /// Rebuild the category index from scratch
    fn rebuild_category_index(&mut self) {
        self.category_index.clear();
        for (name, alias) in &self.aliases {
            if let Some(ref category) = alias.category {
                self.category_index
                    .entry(category.clone())
                    .or_default()
                    .push(name.clone());
            }
        }
    }

    /// Check if an alias exists
    pub fn contains(&self, name: &str) -> bool {
        self.aliases.contains_key(name)
    }

    /// Get the number of aliases
    pub fn len(&self) -> usize {
        self.aliases.len()
    }

    /// Check if the manager has no aliases
    pub fn is_empty(&self) -> bool {
        self.aliases.is_empty()
    }

    /// Clear all aliases
    pub fn clear(&mut self) {
        self.aliases.clear();
        self.category_index.clear();
    }
}

impl Default for AliasManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during alias operations
#[derive(Debug, thiserror::Error)]
pub enum AliasError {
    #[error("Alias not found: {0}")]
    NotFound(String),
    #[error("Alias already exists: {0}")]
    AlreadyExists(String),
    #[error("Invalid alias name: {0}")]
    InvalidName(String),
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_alias_creation() {
        let alias = Alias::new("ll", "ls -la");
        assert_eq!(alias.name, "ll");
        assert_eq!(alias.command, "ls -la");
        assert!(alias.enabled);
        assert!(alias.description.is_none());
        assert!(alias.category.is_none());
    }

    #[test]
    fn test_alias_with_description() {
        let alias = Alias::with_description("ll", "ls -la", "List all files");
        assert_eq!(alias.description, Some("List all files".to_string()));
    }

    #[test]
    fn test_alias_with_category() {
        let alias = Alias::new("ll", "ls -la").with_category("file");
        assert_eq!(alias.category, Some("file".to_string()));
    }

    #[test]
    fn test_add_alias() {
        let mut manager = AliasManager::new();
        assert!(manager
            .add_alias("ll", "ls -la", Some("List all files"))
            .is_ok());
        assert_eq!(manager.len(), 1);

        let alias = manager.get_alias("ll").unwrap();
        assert_eq!(alias.name, "ll");
        assert_eq!(alias.command, "ls -la");
    }

    #[test]
    fn test_duplicate_alias() {
        let mut manager = AliasManager::new();
        manager.add_alias("ll", "ls -la", None::<String>).unwrap();
        let result = manager.add_alias("ll", "ls -lah", None::<String>);
        assert!(matches!(result, Err(AliasError::AlreadyExists(_))));
    }

    #[test]
    fn test_remove_alias() {
        let mut manager = AliasManager::new();
        manager.add_alias("ll", "ls -la", None::<String>).unwrap();
        assert_eq!(manager.len(), 1);

        let removed = manager.remove_alias("ll");
        assert!(removed.is_ok());
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn test_update_alias() {
        let mut manager = AliasManager::new();
        manager.add_alias("ll", "ls -la", None::<String>).unwrap();

        manager.update_alias("ll", "ls -lah").unwrap();
        let alias = manager.get_alias("ll").unwrap();
        assert_eq!(alias.command, "ls -lah");
    }

    #[test]
    fn test_list_aliases() {
        let mut manager = AliasManager::new();
        manager.add_alias("ll", "ls -la", None::<String>).unwrap();
        manager.add_alias("la", "ls -A", None::<String>).unwrap();
        manager.add_alias("gs", "git status", None::<String>).unwrap();

        let aliases = manager.list_aliases();
        assert_eq!(aliases.len(), 3);
        // Check sorting
        assert_eq!(aliases[0].name, "gs");
        assert_eq!(aliases[1].name, "la");
        assert_eq!(aliases[2].name, "ll");
    }

    #[test]
    fn test_toggle_alias() {
        let mut manager = AliasManager::new();
        manager.add_alias("ll", "ls -la", None::<String>).unwrap();

        assert!(manager.get_alias("ll").unwrap().enabled);

        let new_state = manager.toggle_alias("ll").unwrap();
        assert!(!new_state);
        assert!(!manager.get_alias("ll").unwrap().enabled);

        let new_state = manager.toggle_alias("ll").unwrap();
        assert!(new_state);
    }

    #[test]
    fn test_expand_simple_alias() {
        let mut manager = AliasManager::new();
        manager.add_alias("ll", "ls -la", None::<String>).unwrap();

        let expanded = manager.expand_command("ll");
        assert_eq!(expanded, "ls -la");
    }

    #[test]
    fn test_expand_alias_with_args() {
        let mut manager = AliasManager::new();
        manager.add_alias("ll", "ls -la", None::<String>).unwrap();

        let expanded = manager.expand_command("ll /tmp");
        assert_eq!(expanded, "ls -la /tmp");
    }

    #[test]
    fn test_expand_with_parameter_substitution() {
        let mut manager = AliasManager::new();
        manager
            .add_alias("greet", "echo Hello $1, welcome to $2!", None::<String>)
            .unwrap();

        let expanded = manager.expand_command("greet Alice AgTerm");
        assert_eq!(expanded, "echo Hello Alice, welcome to AgTerm!");
    }

    #[test]
    fn test_expand_with_all_args() {
        let mut manager = AliasManager::new();
        manager
            .add_alias("mygrep", "grep --color=auto ${*}", None::<String>)
            .unwrap();

        let expanded = manager.expand_command("mygrep pattern file1.txt file2.txt");
        assert_eq!(expanded, "grep --color=auto pattern file1.txt file2.txt");
    }

    #[test]
    fn test_expand_with_dollar_star() {
        let mut manager = AliasManager::new();
        manager
            .add_alias("mygrep", "grep --color=auto $*", None::<String>)
            .unwrap();

        let expanded = manager.expand_command("mygrep pattern file.txt");
        assert_eq!(expanded, "grep --color=auto pattern file.txt");
    }

    #[test]
    fn test_expand_nested_alias() {
        let mut manager = AliasManager::new();
        manager.add_alias("ll", "ls -la", None::<String>).unwrap();
        manager.add_alias("lll", "ll", None::<String>).unwrap();

        let expanded = manager.expand_command("lll /tmp");
        assert_eq!(expanded, "ls -la /tmp");
    }

    #[test]
    fn test_disabled_alias() {
        let mut manager = AliasManager::new();
        manager.add_alias("ll", "ls -la", None::<String>).unwrap();
        manager.toggle_alias("ll").unwrap();

        let expanded = manager.expand_command("ll");
        assert_eq!(expanded, "ll"); // Not expanded because disabled
    }

    #[test]
    fn test_category_filtering() {
        let mut manager = AliasManager::new();
        manager.add_alias("ll", "ls -la", None::<String>).unwrap();
        manager.add_alias("gs", "git status", None::<String>).unwrap();
        manager.add_alias("gp", "git push", None::<String>).unwrap();

        // Manually set categories
        manager.get_alias_mut("ll").unwrap().category = Some("file".to_string());
        manager.get_alias_mut("gs").unwrap().category = Some("git".to_string());
        manager.get_alias_mut("gp").unwrap().category = Some("git".to_string());
        manager.rebuild_category_index();

        let git_aliases = manager.get_by_category("git");
        assert_eq!(git_aliases.len(), 2);

        let file_aliases = manager.get_by_category("file");
        assert_eq!(file_aliases.len(), 1);
    }

    #[test]
    fn test_get_categories() {
        let mut manager = AliasManager::new();
        manager.add_alias("ll", "ls -la", None::<String>).unwrap();
        manager.add_alias("gs", "git status", None::<String>).unwrap();

        manager.get_alias_mut("ll").unwrap().category = Some("file".to_string());
        manager.get_alias_mut("gs").unwrap().category = Some("git".to_string());
        manager.rebuild_category_index();

        let categories = manager.get_categories();
        assert_eq!(categories.len(), 2);
        assert!(categories.contains(&"file".to_string()));
        assert!(categories.contains(&"git".to_string()));
    }

    #[test]
    fn test_file_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("aliases.json");

        // Create and save
        let mut manager = AliasManager::new();
        manager.add_alias("ll", "ls -la", Some("List all")).unwrap();
        manager.add_alias("gs", "git status", Some("Git status")).unwrap();
        manager.save_to_file(&file_path).unwrap();

        // Load into new manager
        let mut new_manager = AliasManager::new();
        new_manager.load_from_file(&file_path).unwrap();

        assert_eq!(new_manager.len(), 2);
        assert!(new_manager.contains("ll"));
        assert!(new_manager.contains("gs"));
    }

    #[test]
    fn test_save_without_path() {
        let mut manager = AliasManager::new();
        manager.add_alias("ll", "ls -la", None::<String>).unwrap();

        // Should fail because no file path is set
        let result = manager.save();
        assert!(matches!(result, Err(AliasError::IoError(_))));

        // After loading from file, save() should work
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("aliases.json");
        manager.save_to_file(&file_path).unwrap();

        // Add another alias and save using save()
        manager.add_alias("gs", "git status", None::<String>).unwrap();
        assert!(manager.save().is_ok());

        // Verify it was saved
        let mut new_manager = AliasManager::new();
        new_manager.load_from_file(&file_path).unwrap();
        assert_eq!(new_manager.len(), 2);
    }

    #[test]
    fn test_export_to_shell() {
        let mut manager = AliasManager::new();
        manager.add_alias("ll", "ls -la", Some("List all")).unwrap();
        manager.add_alias("gs", "git status", Some("Git status")).unwrap();

        let shell_output = manager.export_to_shell();
        assert!(shell_output.contains("alias ll='ls -la'"));
        assert!(shell_output.contains("alias gs='git status'"));
        assert!(shell_output.contains("# List all"));
        assert!(shell_output.contains("# Git status"));
    }

    #[test]
    fn test_import_from_shell() {
        let shell_content = r#"
# My aliases
# List all files
alias ll='ls -la'
# Git status
alias gs='git status'
alias gp='git push'
"#;

        let mut manager = AliasManager::new();
        let imported = manager.import_from_shell(shell_content).unwrap();

        assert_eq!(imported, 3);
        assert!(manager.contains("ll"));
        assert!(manager.contains("gs"));
        assert!(manager.contains("gp"));
        assert_eq!(
            manager.get_alias("ll").unwrap().description,
            Some("List all files".to_string())
        );
    }

    #[test]
    fn test_parse_shell_alias() {
        let test_cases = vec![
            ("ll='ls -la'", Some(("ll".to_string(), "ls -la".to_string()))),
            ("gs=\"git status\"", Some(("gs".to_string(), "git status".to_string()))),
            ("gp=git push", Some(("gp".to_string(), "git push".to_string()))),
            ("invalid", None),
        ];

        for (input, expected) in test_cases {
            let result = AliasManager::parse_shell_alias(input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_detect_category() {
        let test_cases = vec![
            ("git status", Some("git")),
            ("docker ps", Some("docker")),
            ("cargo build", Some("rust")),
            ("npm install", Some("nodejs")),
            ("ls -la", Some("file")),
            ("echo hello", Some("system")),
        ];

        for (command, expected_category) in test_cases {
            let detected = AliasManager::detect_category(command);
            assert_eq!(
                detected.as_deref(),
                expected_category,
                "Failed for command: {}",
                command
            );
        }
    }

    #[test]
    fn test_default_aliases() {
        let manager = AliasManager::with_defaults();

        // Check that we have default aliases
        assert!(!manager.is_empty());

        // Check for some expected aliases
        assert!(manager.contains("ll"));
        assert!(manager.contains("gs"));
        assert!(manager.contains("gp"));
        assert!(manager.contains("cb"));

        // Check categories
        let categories = manager.get_categories();
        assert!(categories.contains(&"git".to_string()));
        assert!(categories.contains(&"docker".to_string()));
        assert!(categories.contains(&"rust".to_string()));
    }

    #[test]
    fn test_parameter_substitution_complex() {
        let mut manager = AliasManager::new();
        manager
            .add_alias("deploy", "docker run -d --name $1 -p $2:$2 ${3}", None::<String>)
            .unwrap();

        let expanded = manager.expand_command("deploy myapp 8080 nginx");
        assert_eq!(expanded, "docker run -d --name myapp -p 8080:8080 nginx");
    }

    #[test]
    fn test_parameter_arg_count() {
        let mut manager = AliasManager::new();
        manager
            .add_alias("count", "echo You provided $# arguments", None::<String>)
            .unwrap();

        let expanded = manager.expand_command("count a b c");
        assert_eq!(expanded, "echo You provided 3 arguments");
    }

    #[test]
    fn test_max_recursion_protection() {
        let mut manager = AliasManager::new();
        manager.max_recursion_depth = 3;

        // Create circular reference
        manager.add_alias("a", "b", None::<String>).unwrap();
        manager.add_alias("b", "c", None::<String>).unwrap();
        manager.add_alias("c", "d", None::<String>).unwrap();
        manager.add_alias("d", "e", None::<String>).unwrap();
        manager.add_alias("e", "f", None::<String>).unwrap();

        // Should stop expanding after max depth
        let expanded = manager.expand_command("a");
        // Due to depth limit, it won't expand all the way
        assert!(expanded.len() > 0);
    }

    #[test]
    fn test_empty_command_validation() {
        let mut manager = AliasManager::new();
        let result = manager.add_alias("empty", "", None::<String>);
        assert!(matches!(result, Err(AliasError::InvalidCommand(_))));
    }

    #[test]
    fn test_empty_name_validation() {
        let mut manager = AliasManager::new();
        let result = manager.add_alias("", "ls -la", None::<String>);
        assert!(matches!(result, Err(AliasError::InvalidName(_))));
    }

    #[test]
    fn test_list_enabled_only() {
        let mut manager = AliasManager::new();
        manager.add_alias("a", "cmd1", None::<String>).unwrap();
        manager.add_alias("b", "cmd2", None::<String>).unwrap();
        manager.add_alias("c", "cmd3", None::<String>).unwrap();

        manager.toggle_alias("b").unwrap(); // Disable b

        let all = manager.list_aliases();
        assert_eq!(all.len(), 3);

        let enabled = manager.list_enabled_aliases();
        assert_eq!(enabled.len(), 2);
        assert!(enabled.iter().all(|a| a.name != "b"));
    }

    #[test]
    fn test_clear() {
        let mut manager = AliasManager::new();
        manager.add_alias("a", "cmd1", None::<String>).unwrap();
        manager.add_alias("b", "cmd2", None::<String>).unwrap();

        assert_eq!(manager.len(), 2);
        manager.clear();
        assert_eq!(manager.len(), 0);
        assert!(manager.is_empty());
    }
}
