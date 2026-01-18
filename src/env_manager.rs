//! Environment Variable Manager for AgTerm
//!
//! This module provides a comprehensive environment variable management system that allows:
//! - Setting, getting, and removing environment variables with metadata
//! - Categorizing variables (PATH, LANG, CUSTOM, etc.)
//! - Tracking variable sources (System, User, Session, Profile)
//! - Marking sensitive variables (passwords, tokens) with masking support
//! - Searching and filtering variables by category, source, or query
//! - Comparing with system environment and generating diffs
//! - Persisting variables to disk with serialization
//! - Applying managed variables to the actual environment

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::Path;
use thiserror::Error;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, Error)]
pub enum EnvError {
    #[error("Environment variable not found: {0}")]
    NotFound(String),

    #[error("Invalid variable name: {0}")]
    InvalidName(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Variable is marked as sensitive and cannot be exported")]
    SensitiveVariable(String),

    #[error("System variable cannot be removed: {0}")]
    SystemVariable(String),
}

pub type EnvResult<T> = Result<T, EnvError>;

// ============================================================================
// EnvVarSource Enum
// ============================================================================

/// Source of an environment variable
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnvVarSource {
    /// System environment variable (from OS)
    System,
    /// User-defined variable (persistent across sessions)
    User,
    /// Session-specific variable (temporary)
    Session,
    /// Profile-specific variable (from terminal profile)
    Profile,
}

impl EnvVarSource {
    /// Check if this source is persistent (saved to disk)
    pub fn is_persistent(&self) -> bool {
        matches!(self, EnvVarSource::User | EnvVarSource::Profile)
    }

    /// Check if this source can be modified
    pub fn is_mutable(&self) -> bool {
        matches!(
            self,
            EnvVarSource::User | EnvVarSource::Session | EnvVarSource::Profile
        )
    }
}

// ============================================================================
// EnvVar Structure
// ============================================================================

/// An environment variable with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    /// Variable name
    pub name: String,

    /// Variable value
    pub value: String,

    /// Optional description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Variable category (e.g., PATH, LANG, CUSTOM)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Source of this variable
    pub source: EnvVarSource,

    /// Whether this is sensitive data (password, token, etc.)
    #[serde(default)]
    pub sensitive: bool,

    /// Timestamp when this variable was created/modified
    pub created_at: DateTime<Utc>,
}

impl EnvVar {
    /// Create a new environment variable
    pub fn new(name: String, value: String, source: EnvVarSource) -> Self {
        Self {
            name,
            value,
            description: None,
            category: None,
            source,
            sensitive: false,
            created_at: Utc::now(),
        }
    }

    /// Create with full configuration
    pub fn with_details(
        name: String,
        value: String,
        source: EnvVarSource,
        description: Option<String>,
        category: Option<String>,
        sensitive: bool,
    ) -> Self {
        Self {
            name,
            value,
            description,
            category,
            source,
            sensitive,
            created_at: Utc::now(),
        }
    }

    /// Get the value, masking if sensitive
    pub fn masked_value(&self) -> String {
        if self.sensitive {
            "********".to_string()
        } else {
            self.value.clone()
        }
    }

    /// Auto-detect category based on variable name
    pub fn auto_categorize(&mut self) {
        if self.category.is_some() {
            return;
        }

        let name_upper = self.name.to_uppercase();
        self.category = Some(
            if name_upper.contains("PATH") {
                "PATH"
            } else if name_upper.contains("LANG") || name_upper.starts_with("LC_") {
                "LANG"
            } else if name_upper.contains("TERM") {
                "TERM"
            } else if name_upper.contains("HOME") || name_upper.contains("USER") {
                "USER"
            } else if name_upper.contains("SHELL") {
                "SHELL"
            } else if name_upper.contains("PASSWORD")
                || name_upper.contains("TOKEN")
                || name_upper.contains("SECRET")
                || name_upper.contains("KEY")
            {
                self.sensitive = true;
                "SENSITIVE"
            } else {
                "CUSTOM"
            }
            .to_string(),
        );
    }
}

// ============================================================================
// EnvDiff Structure
// ============================================================================

/// Difference between two environment states
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnvDiff {
    /// Variables added (not in system)
    pub added: Vec<EnvVar>,

    /// Variables removed (in system but not in manager)
    pub removed: Vec<String>,

    /// Variables modified (name, old_value, new_value)
    pub modified: Vec<(String, String, String)>,
}

impl EnvDiff {
    /// Check if there are any differences
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.modified.is_empty()
    }

    /// Get total number of changes
    pub fn total_changes(&self) -> usize {
        self.added.len() + self.removed.len() + self.modified.len()
    }

    /// Get a human-readable summary
    pub fn summary(&self) -> String {
        if self.is_empty() {
            return "No changes".to_string();
        }

        let mut parts = Vec::new();
        if !self.added.is_empty() {
            parts.push(format!("+{} added", self.added.len()));
        }
        if !self.removed.is_empty() {
            parts.push(format!("-{} removed", self.removed.len()));
        }
        if !self.modified.is_empty() {
            parts.push(format!("~{} modified", self.modified.len()));
        }

        parts.join(", ")
    }
}

// ============================================================================
// EnvManager
// ============================================================================

/// Environment variable manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvManager {
    /// Managed environment variables
    variables: HashMap<String, EnvVar>,

    /// Default category for new variables
    #[serde(default = "default_category")]
    pub default_category: String,

    /// Whether to auto-categorize new variables
    #[serde(default = "default_true")]
    pub auto_categorize: bool,

    /// Whether to mask sensitive variables in exports
    #[serde(default = "default_true")]
    pub mask_sensitive: bool,
}

fn default_category() -> String {
    "CUSTOM".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for EnvManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvManager {
    /// Create a new environment manager
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            default_category: default_category(),
            auto_categorize: true,
            mask_sensitive: true,
        }
    }

    /// Create and populate from system environment
    pub fn from_system() -> Self {
        let mut manager = Self::new();
        for (key, value) in env::vars() {
            let mut var = EnvVar::new(key, value, EnvVarSource::System);
            if manager.auto_categorize {
                var.auto_categorize();
            }
            manager.variables.insert(var.name.clone(), var);
        }
        manager
    }

    /// Set an environment variable
    pub fn set_var(
        &mut self,
        name: String,
        value: String,
        source: EnvVarSource,
    ) -> EnvResult<()> {
        if name.is_empty() {
            return Err(EnvError::InvalidName("Variable name cannot be empty".to_string()));
        }

        if name.contains('=') {
            return Err(EnvError::InvalidName(
                "Variable name cannot contain '='".to_string(),
            ));
        }

        let mut var = EnvVar::new(name.clone(), value, source);
        if self.auto_categorize {
            var.auto_categorize();
        } else {
            var.category = Some(self.default_category.clone());
        }

        self.variables.insert(name, var);
        Ok(())
    }

    /// Set with full details
    pub fn set_var_detailed(
        &mut self,
        name: String,
        value: String,
        source: EnvVarSource,
        description: Option<String>,
        category: Option<String>,
        sensitive: bool,
    ) -> EnvResult<()> {
        if name.is_empty() {
            return Err(EnvError::InvalidName("Variable name cannot be empty".to_string()));
        }

        if name.contains('=') {
            return Err(EnvError::InvalidName(
                "Variable name cannot contain '='".to_string(),
            ));
        }

        let var = EnvVar::with_details(name.clone(), value, source, description, category, sensitive);
        self.variables.insert(name, var);
        Ok(())
    }

    /// Get an environment variable
    pub fn get_var(&self, name: &str) -> Option<&EnvVar> {
        self.variables.get(name)
    }

    /// Get a mutable reference to a variable
    pub fn get_var_mut(&mut self, name: &str) -> Option<&mut EnvVar> {
        self.variables.get_mut(name)
    }

    /// Remove an environment variable
    pub fn remove_var(&mut self, name: &str) -> EnvResult<EnvVar> {
        let var = self
            .variables
            .get(name)
            .ok_or_else(|| EnvError::NotFound(name.to_string()))?;

        // Prevent removal of system variables
        if var.source == EnvVarSource::System {
            return Err(EnvError::SystemVariable(name.to_string()));
        }

        self.variables
            .remove(name)
            .ok_or_else(|| EnvError::NotFound(name.to_string()))
    }

    /// List all variables
    pub fn list_vars(&self) -> Vec<&EnvVar> {
        self.variables.values().collect()
    }

    /// List variables by category
    pub fn list_by_category(&self, category: &str) -> Vec<&EnvVar> {
        self.variables
            .values()
            .filter(|v| {
                v.category
                    .as_ref()
                    .map(|c| c.eq_ignore_ascii_case(category))
                    .unwrap_or(false)
            })
            .collect()
    }

    /// List variables by source
    pub fn list_by_source(&self, source: EnvVarSource) -> Vec<&EnvVar> {
        self.variables
            .values()
            .filter(|v| v.source == source)
            .collect()
    }

    /// Search variables by query (matches name, value, or description)
    pub fn search_vars(&self, query: &str) -> Vec<&EnvVar> {
        let query_lower = query.to_lowercase();
        self.variables
            .values()
            .filter(|v| {
                v.name.to_lowercase().contains(&query_lower)
                    || v.value.to_lowercase().contains(&query_lower)
                    || v.description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .collect()
    }

    /// Export variables for shell (as HashMap)
    pub fn export_for_shell(&self) -> HashMap<String, String> {
        self.variables
            .iter()
            .filter(|(_, v)| !v.sensitive || !self.mask_sensitive)
            .map(|(k, v)| (k.clone(), v.value.clone()))
            .collect()
    }

    /// Export variables with masking option
    pub fn export_with_masking(&self, mask: bool) -> HashMap<String, String> {
        self.variables
            .iter()
            .map(|(k, v)| {
                let value = if mask && v.sensitive {
                    v.masked_value()
                } else {
                    v.value.clone()
                };
                (k.clone(), value)
            })
            .collect()
    }

    /// Compare with system environment and generate diff
    pub fn diff_with_system(&self) -> EnvDiff {
        let mut diff = EnvDiff::default();
        let system_vars: HashMap<String, String> = env::vars().collect();

        // Find added and modified variables
        for (name, var) in &self.variables {
            if let Some(system_value) = system_vars.get(name) {
                // Variable exists in both - check if modified
                if &var.value != system_value {
                    diff.modified
                        .push((name.clone(), system_value.clone(), var.value.clone()));
                }
            } else if var.source != EnvVarSource::System {
                // Variable added (not from system)
                diff.added.push(var.clone());
            }
        }

        // Find removed variables (in system but not in manager)
        for name in system_vars.keys() {
            if !self.variables.contains_key(name) {
                diff.removed.push(name.clone());
            }
        }

        diff
    }

    /// Apply managed variables to the actual environment
    pub fn apply_to_env(&self) -> EnvResult<usize> {
        let mut count = 0;

        for (name, var) in &self.variables {
            // Skip system variables
            if var.source == EnvVarSource::System {
                continue;
            }

            // Skip sensitive variables if masking is enabled
            if var.sensitive && self.mask_sensitive {
                return Err(EnvError::SensitiveVariable(name.clone()));
            }

            env::set_var(name, &var.value);
            count += 1;
        }

        Ok(count)
    }

    /// Apply specific variables to environment
    pub fn apply_vars(&self, names: &[String]) -> EnvResult<usize> {
        let mut count = 0;

        for name in names {
            let var = self.get_var(name).ok_or_else(|| EnvError::NotFound(name.clone()))?;

            // Check if sensitive and masking is enabled
            if var.sensitive && self.mask_sensitive {
                return Err(EnvError::SensitiveVariable(name.clone()));
            }

            env::set_var(name, &var.value);
            count += 1;
        }

        Ok(count)
    }

    /// Remove variables from system environment
    pub fn remove_from_env(&self, names: &[String]) -> usize {
        let mut count = 0;

        for name in names {
            env::remove_var(name);
            count += 1;
        }

        count
    }

    /// Get all unique categories
    pub fn get_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self
            .variables
            .values()
            .filter_map(|v| v.category.clone())
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }

    /// Get statistics about managed variables
    pub fn stats(&self) -> EnvStats {
        let mut stats = EnvStats::default();

        for var in self.variables.values() {
            stats.total += 1;

            match var.source {
                EnvVarSource::System => stats.system += 1,
                EnvVarSource::User => stats.user += 1,
                EnvVarSource::Session => stats.session += 1,
                EnvVarSource::Profile => stats.profile += 1,
            }

            if var.sensitive {
                stats.sensitive += 1;
            }
        }

        stats
    }

    /// Save to JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> EnvResult<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load from JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> EnvResult<Self> {
        let contents = std::fs::read_to_string(path)?;
        let manager = serde_json::from_str(&contents)?;
        Ok(manager)
    }

    /// Merge another manager's variables into this one
    pub fn merge(&mut self, other: &EnvManager, overwrite: bool) {
        for (name, var) in &other.variables {
            if overwrite || !self.variables.contains_key(name) {
                self.variables.insert(name.clone(), var.clone());
            }
        }
    }

    /// Clear all non-system variables
    pub fn clear_user_vars(&mut self) {
        self.variables.retain(|_, v| v.source == EnvVarSource::System);
    }

    /// Import from system environment with filter
    pub fn import_from_system<F>(&mut self, filter: F)
    where
        F: Fn(&str) -> bool,
    {
        for (key, value) in env::vars() {
            if filter(&key) {
                let mut var = EnvVar::new(key.clone(), value, EnvVarSource::System);
                if self.auto_categorize {
                    var.auto_categorize();
                }
                self.variables.insert(key, var);
            }
        }
    }
}

// ============================================================================
// EnvStats
// ============================================================================

/// Statistics about environment variables
#[derive(Debug, Clone, Default)]
pub struct EnvStats {
    pub total: usize,
    pub system: usize,
    pub user: usize,
    pub session: usize,
    pub profile: usize,
    pub sensitive: usize,
}

impl EnvStats {
    /// Format as human-readable string
    pub fn format(&self) -> String {
        format!(
            "Total: {}, System: {}, User: {}, Session: {}, Profile: {}, Sensitive: {}",
            self.total, self.system, self.user, self.session, self.profile, self.sensitive
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_var_creation() {
        let var = EnvVar::new("TEST_VAR".to_string(), "test_value".to_string(), EnvVarSource::User);

        assert_eq!(var.name, "TEST_VAR");
        assert_eq!(var.value, "test_value");
        assert_eq!(var.source, EnvVarSource::User);
        assert!(!var.sensitive);
    }

    #[test]
    fn test_env_var_masking() {
        let mut var = EnvVar::new("PASSWORD".to_string(), "secret123".to_string(), EnvVarSource::User);
        var.sensitive = true;

        assert_eq!(var.masked_value(), "********");
        assert_eq!(var.value, "secret123");
    }

    #[test]
    fn test_auto_categorize() {
        let mut var = EnvVar::new("PATH".to_string(), "/usr/bin".to_string(), EnvVarSource::System);
        var.auto_categorize();
        assert_eq!(var.category, Some("PATH".to_string()));

        let mut var = EnvVar::new("LC_ALL".to_string(), "en_US.UTF-8".to_string(), EnvVarSource::System);
        var.auto_categorize();
        assert_eq!(var.category, Some("LANG".to_string()));

        let mut var = EnvVar::new("API_TOKEN".to_string(), "secret".to_string(), EnvVarSource::User);
        var.auto_categorize();
        assert_eq!(var.category, Some("SENSITIVE".to_string()));
        assert!(var.sensitive);
    }

    #[test]
    fn test_env_manager_set_get() {
        let mut manager = EnvManager::new();

        manager.set_var("TEST".to_string(), "value".to_string(), EnvVarSource::User).unwrap();

        let var = manager.get_var("TEST").unwrap();
        assert_eq!(var.name, "TEST");
        assert_eq!(var.value, "value");
    }

    #[test]
    fn test_env_manager_remove() {
        let mut manager = EnvManager::new();

        manager.set_var("TEST".to_string(), "value".to_string(), EnvVarSource::User).unwrap();
        assert!(manager.get_var("TEST").is_some());

        manager.remove_var("TEST").unwrap();
        assert!(manager.get_var("TEST").is_none());
    }

    #[test]
    fn test_cannot_remove_system_var() {
        let mut manager = EnvManager::new();

        manager.set_var("SYSTEM_VAR".to_string(), "value".to_string(), EnvVarSource::System).unwrap();

        let result = manager.remove_var("SYSTEM_VAR");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EnvError::SystemVariable(_)));
    }

    #[test]
    fn test_list_by_category() {
        let mut manager = EnvManager::new();

        manager.set_var_detailed(
            "VAR1".to_string(),
            "value1".to_string(),
            EnvVarSource::User,
            None,
            Some("TEST".to_string()),
            false,
        ).unwrap();

        manager.set_var_detailed(
            "VAR2".to_string(),
            "value2".to_string(),
            EnvVarSource::User,
            None,
            Some("TEST".to_string()),
            false,
        ).unwrap();

        manager.set_var_detailed(
            "VAR3".to_string(),
            "value3".to_string(),
            EnvVarSource::User,
            None,
            Some("OTHER".to_string()),
            false,
        ).unwrap();

        let test_vars = manager.list_by_category("TEST");
        assert_eq!(test_vars.len(), 2);

        let other_vars = manager.list_by_category("OTHER");
        assert_eq!(other_vars.len(), 1);
    }

    #[test]
    fn test_list_by_source() {
        let mut manager = EnvManager::new();

        manager.set_var("USER1".to_string(), "val1".to_string(), EnvVarSource::User).unwrap();
        manager.set_var("USER2".to_string(), "val2".to_string(), EnvVarSource::User).unwrap();
        manager.set_var("SESSION1".to_string(), "val3".to_string(), EnvVarSource::Session).unwrap();

        let user_vars = manager.list_by_source(EnvVarSource::User);
        assert_eq!(user_vars.len(), 2);

        let session_vars = manager.list_by_source(EnvVarSource::Session);
        assert_eq!(session_vars.len(), 1);
    }

    #[test]
    fn test_search_vars() {
        let mut manager = EnvManager::new();

        manager.set_var("SEARCH_TEST".to_string(), "value".to_string(), EnvVarSource::User).unwrap();
        manager.set_var("OTHER".to_string(), "search_value".to_string(), EnvVarSource::User).unwrap();
        manager.set_var("UNRELATED".to_string(), "data".to_string(), EnvVarSource::User).unwrap();

        let results = manager.search_vars("search");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_export_for_shell() {
        let mut manager = EnvManager::new();

        manager.set_var("VAR1".to_string(), "value1".to_string(), EnvVarSource::User).unwrap();
        manager.set_var("VAR2".to_string(), "value2".to_string(), EnvVarSource::User).unwrap();

        let exported = manager.export_for_shell();
        assert_eq!(exported.get("VAR1"), Some(&"value1".to_string()));
        assert_eq!(exported.get("VAR2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_export_masks_sensitive() {
        let mut manager = EnvManager::new();
        manager.mask_sensitive = true;

        manager.set_var_detailed(
            "PASSWORD".to_string(),
            "secret123".to_string(),
            EnvVarSource::User,
            None,
            None,
            true,
        ).unwrap();

        let exported = manager.export_for_shell();
        assert!(!exported.contains_key("PASSWORD"));
    }

    #[test]
    fn test_export_with_masking() {
        let mut manager = EnvManager::new();

        manager.set_var("PUBLIC".to_string(), "value".to_string(), EnvVarSource::User).unwrap();
        manager.set_var_detailed(
            "SECRET".to_string(),
            "password".to_string(),
            EnvVarSource::User,
            None,
            None,
            true,
        ).unwrap();

        let exported = manager.export_with_masking(true);
        assert_eq!(exported.get("PUBLIC"), Some(&"value".to_string()));
        assert_eq!(exported.get("SECRET"), Some(&"********".to_string()));
    }

    #[test]
    fn test_env_diff() {
        let diff = EnvDiff {
            added: vec![EnvVar::new("NEW".to_string(), "value".to_string(), EnvVarSource::User)],
            removed: vec!["OLD".to_string()],
            modified: vec![("MOD".to_string(), "old".to_string(), "new".to_string())],
        };

        assert!(!diff.is_empty());
        assert_eq!(diff.total_changes(), 3);
        assert!(diff.summary().contains("1 added"));
        assert!(diff.summary().contains("1 removed"));
        assert!(diff.summary().contains("1 modified"));
    }

    #[test]
    fn test_env_stats() {
        let mut manager = EnvManager::new();

        manager.set_var("SYS1".to_string(), "val".to_string(), EnvVarSource::System).unwrap();
        manager.set_var("USER1".to_string(), "val".to_string(), EnvVarSource::User).unwrap();
        manager.set_var("USER2".to_string(), "val".to_string(), EnvVarSource::User).unwrap();
        manager.set_var_detailed(
            "SECRET".to_string(),
            "val".to_string(),
            EnvVarSource::User,
            None,
            None,
            true,
        ).unwrap();

        let stats = manager.stats();
        assert_eq!(stats.total, 4);
        assert_eq!(stats.system, 1);
        assert_eq!(stats.user, 3);
        assert_eq!(stats.sensitive, 1);
    }

    #[test]
    fn test_save_load_file() {
        let mut manager = EnvManager::new();
        manager.set_var("TEST".to_string(), "value".to_string(), EnvVarSource::User).unwrap();

        let temp_file = std::env::temp_dir().join("env_manager_test.json");
        manager.save_to_file(&temp_file).unwrap();

        let loaded = EnvManager::load_from_file(&temp_file).unwrap();
        assert_eq!(loaded.get_var("TEST").unwrap().value, "value");

        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_merge() {
        let mut manager1 = EnvManager::new();
        manager1.set_var("VAR1".to_string(), "value1".to_string(), EnvVarSource::User).unwrap();

        let mut manager2 = EnvManager::new();
        manager2.set_var("VAR2".to_string(), "value2".to_string(), EnvVarSource::User).unwrap();

        manager1.merge(&manager2, false);

        assert!(manager1.get_var("VAR1").is_some());
        assert!(manager1.get_var("VAR2").is_some());
    }

    #[test]
    fn test_merge_overwrite() {
        let mut manager1 = EnvManager::new();
        manager1.set_var("VAR".to_string(), "old".to_string(), EnvVarSource::User).unwrap();

        let mut manager2 = EnvManager::new();
        manager2.set_var("VAR".to_string(), "new".to_string(), EnvVarSource::User).unwrap();

        manager1.merge(&manager2, true);
        assert_eq!(manager1.get_var("VAR").unwrap().value, "new");

        let mut manager3 = EnvManager::new();
        manager3.set_var("VAR".to_string(), "old".to_string(), EnvVarSource::User).unwrap();
        manager3.merge(&manager2, false);
        assert_eq!(manager3.get_var("VAR").unwrap().value, "old");
    }

    #[test]
    fn test_clear_user_vars() {
        let mut manager = EnvManager::new();

        manager.set_var("SYSTEM".to_string(), "val".to_string(), EnvVarSource::System).unwrap();
        manager.set_var("USER".to_string(), "val".to_string(), EnvVarSource::User).unwrap();
        manager.set_var("SESSION".to_string(), "val".to_string(), EnvVarSource::Session).unwrap();

        manager.clear_user_vars();

        assert!(manager.get_var("SYSTEM").is_some());
        assert!(manager.get_var("USER").is_none());
        assert!(manager.get_var("SESSION").is_none());
    }

    #[test]
    fn test_get_categories() {
        let mut manager = EnvManager::new();

        manager.set_var_detailed(
            "VAR1".to_string(),
            "val".to_string(),
            EnvVarSource::User,
            None,
            Some("CAT1".to_string()),
            false,
        ).unwrap();

        manager.set_var_detailed(
            "VAR2".to_string(),
            "val".to_string(),
            EnvVarSource::User,
            None,
            Some("CAT2".to_string()),
            false,
        ).unwrap();

        manager.set_var_detailed(
            "VAR3".to_string(),
            "val".to_string(),
            EnvVarSource::User,
            None,
            Some("CAT1".to_string()),
            false,
        ).unwrap();

        let categories = manager.get_categories();
        assert_eq!(categories.len(), 2);
        assert!(categories.contains(&"CAT1".to_string()));
        assert!(categories.contains(&"CAT2".to_string()));
    }

    #[test]
    fn test_invalid_variable_names() {
        let mut manager = EnvManager::new();

        // Empty name
        let result = manager.set_var("".to_string(), "value".to_string(), EnvVarSource::User);
        assert!(result.is_err());

        // Name with equals sign
        let result = manager.set_var("VAR=NAME".to_string(), "value".to_string(), EnvVarSource::User);
        assert!(result.is_err());
    }

    #[test]
    fn test_env_var_source_properties() {
        assert!(EnvVarSource::User.is_persistent());
        assert!(EnvVarSource::Profile.is_persistent());
        assert!(!EnvVarSource::System.is_persistent());
        assert!(!EnvVarSource::Session.is_persistent());

        assert!(EnvVarSource::User.is_mutable());
        assert!(EnvVarSource::Session.is_mutable());
        assert!(EnvVarSource::Profile.is_mutable());
        assert!(!EnvVarSource::System.is_mutable());
    }

    #[test]
    fn test_import_from_system_with_filter() {
        let mut manager = EnvManager::new();

        // Import only PATH-related variables
        manager.import_from_system(|name| name.contains("PATH"));

        let vars = manager.list_vars();
        for var in vars {
            assert!(var.name.contains("PATH"));
        }
    }
}
