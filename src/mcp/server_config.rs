//! MCP Server Configuration
//!
//! Provides configuration structures for MCP server connections including:
//! - Server profiles with connection settings
//! - Retry and timeout configurations
//! - Global MCP settings with offline fallback and caching
//! - TOML file persistence

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Server profile containing connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerProfile {
    /// Display name for the server
    pub name: String,

    /// Optional description
    #[serde(default)]
    pub description: String,

    /// Server configuration
    pub config: ServerConfig,

    /// Whether this server is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Auto-connect on startup
    #[serde(default)]
    pub auto_connect: bool,
}

/// Server configuration details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Unique server name
    pub name: String,

    /// Server type
    pub server_type: ServerType,

    /// Transport configuration
    pub transport: TransportConfig,

    /// Request timeout in milliseconds
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,

    /// Retry configuration
    #[serde(default)]
    pub retry: RetryConfig,

    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Type of MCP server
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServerType {
    /// Claude AI server
    Claude,

    /// GitHub Copilot server
    GitHubCopilot,

    /// Local LLM server
    LocalLLM,

    /// Custom server implementation
    Custom,
}

/// Transport configuration for server connection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TransportConfig {
    /// Standard I/O transport (child process)
    Stdio {
        /// Command to execute
        command: String,

        /// Command arguments
        #[serde(default)]
        args: Vec<String>,

        /// Environment variables
        #[serde(default)]
        env: HashMap<String, String>,
    },

    /// HTTP/SSE transport
    Http {
        /// Server URL
        url: String,

        /// Optional authentication headers
        #[serde(default)]
        headers: HashMap<String, String>,
    },

    /// WebSocket transport
    WebSocket {
        /// WebSocket URL
        url: String,

        /// Optional authentication
        #[serde(default)]
        auth_token: Option<String>,
    },
}

/// Retry configuration for failed requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Base delay between retries in milliseconds
    #[serde(default = "default_base_delay")]
    pub base_delay_ms: u64,

    /// Exponential backoff factor
    #[serde(default = "default_backoff_factor")]
    pub backoff_factor: f64,
}

/// Global MCP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Available server profiles
    #[serde(default)]
    pub servers: Vec<ServerProfile>,

    /// Default server to use
    #[serde(default)]
    pub default_server: Option<String>,

    /// Global settings
    #[serde(default)]
    pub settings: McpSettings,
}

/// Global MCP settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSettings {
    /// Use offline fallback when server unavailable
    #[serde(default = "default_true")]
    pub offline_fallback: bool,

    /// Cache server responses
    #[serde(default = "default_true")]
    pub cache_responses: bool,

    /// Cache TTL in seconds
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_seconds: u64,

    /// Include context in requests
    #[serde(default = "default_true")]
    pub include_context: bool,

    /// Number of context lines to include
    #[serde(default = "default_context_lines")]
    pub context_lines: usize,
}

// Default value functions
fn default_true() -> bool {
    true
}

fn default_timeout() -> u64 {
    30_000 // 30 seconds
}

fn default_max_retries() -> u32 {
    3
}

fn default_base_delay() -> u64 {
    1_000 // 1 second
}

fn default_backoff_factor() -> f64 {
    2.0
}

fn default_cache_ttl() -> u64 {
    3600 // 1 hour
}

fn default_context_lines() -> usize {
    100
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: default_max_retries(),
            base_delay_ms: default_base_delay(),
            backoff_factor: default_backoff_factor(),
        }
    }
}

impl Default for McpSettings {
    fn default() -> Self {
        Self {
            offline_fallback: default_true(),
            cache_responses: default_true(),
            cache_ttl_seconds: default_cache_ttl(),
            include_context: default_true(),
            context_lines: default_context_lines(),
        }
    }
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            servers: vec![Self::default_claude_server()],
            default_server: Some("claude".to_string()),
            settings: McpSettings::default(),
        }
    }
}

impl McpConfig {
    /// Create a default Claude server configuration
    pub fn default_claude_server() -> ServerProfile {
        ServerProfile {
            name: "claude".to_string(),
            description: "Claude AI MCP Server".to_string(),
            config: ServerConfig {
                name: "claude".to_string(),
                server_type: ServerType::Claude,
                transport: TransportConfig::Stdio {
                    command: "npx".to_string(),
                    args: vec!["-y".to_string(), "@anthropic-ai/mcp-server".to_string()],
                    env: HashMap::new(),
                },
                timeout_ms: default_timeout(),
                retry: RetryConfig::default(),
                metadata: HashMap::new(),
            },
            enabled: true,
            auto_connect: false,
        }
    }

    /// Load configuration from TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, McpConfigError> {
        let contents = fs::read_to_string(path.as_ref())
            .map_err(|e| McpConfigError::IoError(e.to_string()))?;

        toml::from_str(&contents)
            .map_err(|e| McpConfigError::ParseError(e.to_string()))
    }

    /// Save configuration to TOML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), McpConfigError> {
        let contents = toml::to_string_pretty(self)
            .map_err(|e| McpConfigError::SerializeError(e.to_string()))?;

        // Ensure parent directory exists
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)
                .map_err(|e| McpConfigError::IoError(e.to_string()))?;
        }

        fs::write(path.as_ref(), contents)
            .map_err(|e| McpConfigError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Get default config file path
    pub fn default_config_path() -> Result<PathBuf, McpConfigError> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| McpConfigError::IoError("Cannot determine config directory".to_string()))?;

        Ok(config_dir.join("agterm").join("mcp_config.toml"))
    }

    /// Load configuration from default path or create default
    pub fn load_or_default() -> Result<Self, McpConfigError> {
        let path = Self::default_config_path()?;

        if path.exists() {
            Self::load_from_file(&path)
        } else {
            let config = Self::default();
            config.save_to_file(&path)?;
            Ok(config)
        }
    }

    /// Find server profile by name
    pub fn find_server(&self, name: &str) -> Option<&ServerProfile> {
        self.servers.iter().find(|s| s.name == name)
    }

    /// Get the default server profile
    pub fn default_server_profile(&self) -> Option<&ServerProfile> {
        self.default_server
            .as_ref()
            .and_then(|name| self.find_server(name))
    }

    /// Add or update a server profile
    pub fn upsert_server(&mut self, profile: ServerProfile) {
        if let Some(existing) = self.servers.iter_mut().find(|s| s.name == profile.name) {
            *existing = profile;
        } else {
            self.servers.push(profile);
        }
    }

    /// Remove a server profile by name
    pub fn remove_server(&mut self, name: &str) -> bool {
        if let Some(pos) = self.servers.iter().position(|s| s.name == name) {
            self.servers.remove(pos);

            // Clear default if it was the removed server
            if self.default_server.as_deref() == Some(name) {
                self.default_server = None;
            }

            true
        } else {
            false
        }
    }
}

/// MCP configuration errors
#[derive(Debug, thiserror::Error)]
pub enum McpConfigError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Serialize error: {0}")]
    SerializeError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = McpConfig::default();
        assert_eq!(config.servers.len(), 1);
        assert_eq!(config.default_server, Some("claude".to_string()));
        assert!(config.settings.offline_fallback);
    }

    #[test]
    fn test_server_management() {
        let mut config = McpConfig::default();

        // Add new server
        let custom_server = ServerProfile {
            name: "custom".to_string(),
            description: "Custom server".to_string(),
            config: ServerConfig {
                name: "custom".to_string(),
                server_type: ServerType::Custom,
                transport: TransportConfig::Http {
                    url: "http://localhost:8080".to_string(),
                    headers: HashMap::new(),
                },
                timeout_ms: 5000,
                retry: RetryConfig::default(),
                metadata: HashMap::new(),
            },
            enabled: true,
            auto_connect: false,
        };

        config.upsert_server(custom_server);
        assert_eq!(config.servers.len(), 2);

        // Find server
        assert!(config.find_server("custom").is_some());

        // Remove server
        assert!(config.remove_server("custom"));
        assert_eq!(config.servers.len(), 1);
    }

    #[test]
    fn test_serialization() {
        let config = McpConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: McpConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(config.servers.len(), parsed.servers.len());
        assert_eq!(config.default_server, parsed.default_server);
    }
}
