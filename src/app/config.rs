//! Configuration Management
//!
//! Loads and manages application configuration using config-rs.
//! Follows XDG specification for config file locations.

use crate::error::{ConfigError, ConfigResult};
use config::{Config, Environment, File};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// General settings
    #[serde(default)]
    pub general: GeneralConfig,

    /// PTY pool settings
    #[serde(default)]
    pub pty: PtyConfig,

    /// MCP settings
    #[serde(default)]
    pub mcp: McpConfig,

    /// Storage settings
    #[serde(default)]
    pub storage: StorageConfig,

    /// TUI settings
    #[serde(default)]
    pub tui: TuiConfig,

    /// Logging settings
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Application name (used for directory paths)
    #[serde(default = "default_app_name")]
    pub app_name: String,

    /// Default shell to use
    #[serde(default = "default_shell")]
    pub default_shell: String,

    /// Default working directory
    pub default_working_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtyConfig {
    /// Maximum number of concurrent PTY sessions
    #[serde(default = "default_max_pty_sessions")]
    pub max_sessions: usize,

    /// Default terminal columns
    #[serde(default = "default_cols")]
    pub default_cols: u16,

    /// Default terminal rows
    #[serde(default = "default_rows")]
    pub default_rows: u16,

    /// Scrollback buffer size (lines)
    #[serde(default = "default_scrollback")]
    pub scrollback_lines: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// List of MCP servers to connect to
    #[serde(default)]
    pub servers: Vec<McpServerConfig>,

    /// Connection timeout in seconds
    #[serde(default = "default_mcp_timeout")]
    pub timeout_secs: u64,

    /// Retry attempts for failed connections
    #[serde(default = "default_mcp_retries")]
    pub retry_attempts: u32,

    /// Retry delay in seconds
    #[serde(default = "default_mcp_retry_delay")]
    pub retry_delay_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Server name (identifier)
    pub name: String,

    /// Transport type: "stdio" or "sse"
    #[serde(default = "default_transport")]
    pub transport: String,

    /// Command to spawn (for stdio transport)
    pub command: Option<String>,

    /// Arguments for the command
    #[serde(default)]
    pub args: Vec<String>,

    /// URL for SSE transport
    pub url: Option<String>,

    /// Environment variables to set
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,

    /// Whether to auto-connect on startup
    #[serde(default = "default_true")]
    pub auto_connect: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Database file path (`SQLite`)
    pub database_path: Option<PathBuf>,

    /// Log files directory
    pub logs_dir: Option<PathBuf>,

    /// Archive compression level
    #[serde(default = "default_compression_level")]
    pub compression_level: String,

    /// Maximum archive age in days (for cleanup)
    #[serde(default = "default_archive_retention_days")]
    pub archive_retention_days: u32,

    /// Enable AI summarization for archives
    #[serde(default)]
    pub ai_summarization: bool,

    /// AI provider for summarization ("ollama", "claude", etc.)
    pub ai_provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiConfig {
    /// Target frames per second
    #[serde(default = "default_fps")]
    pub target_fps: u32,

    /// Show line numbers in terminal output
    #[serde(default)]
    pub show_line_numbers: bool,

    /// Theme name
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Enable mouse support
    #[serde(default = "default_true")]
    pub mouse_support: bool,

    /// Key bindings (vim, emacs, or custom)
    #[serde(default = "default_keybindings")]
    pub keybindings: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level: trace, debug, info, warn, error
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Log to file
    #[serde(default)]
    pub file: Option<PathBuf>,

    /// Log format: "pretty", "json", "compact"
    #[serde(default = "default_log_format")]
    pub format: String,

    /// Include timestamps
    #[serde(default = "default_true")]
    pub timestamps: bool,

    /// Include file/line info
    #[serde(default)]
    pub file_line: bool,
}

// Default value functions
fn default_app_name() -> String {
    "agterm".to_string()
}

fn default_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
}

fn default_max_pty_sessions() -> usize {
    32
}

fn default_cols() -> u16 {
    120
}

fn default_rows() -> u16 {
    40
}

fn default_scrollback() -> usize {
    10000
}

fn default_mcp_timeout() -> u64 {
    30
}

fn default_mcp_retries() -> u32 {
    3
}

fn default_mcp_retry_delay() -> u64 {
    5
}

fn default_transport() -> String {
    "stdio".to_string()
}

fn default_true() -> bool {
    true
}

fn default_compression_level() -> String {
    "compacted".to_string()
}

fn default_archive_retention_days() -> u32 {
    90
}

fn default_fps() -> u32 {
    60
}

fn default_theme() -> String {
    "default".to_string()
}

fn default_keybindings() -> String {
    "vim".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "pretty".to_string()
}

// Default implementations
impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            app_name: default_app_name(),
            default_shell: default_shell(),
            default_working_dir: None,
        }
    }
}

impl Default for PtyConfig {
    fn default() -> Self {
        Self {
            max_sessions: default_max_pty_sessions(),
            default_cols: default_cols(),
            default_rows: default_rows(),
            scrollback_lines: default_scrollback(),
        }
    }
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            servers: Vec::new(),
            timeout_secs: default_mcp_timeout(),
            retry_attempts: default_mcp_retries(),
            retry_delay_secs: default_mcp_retry_delay(),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            database_path: None,
            logs_dir: None,
            compression_level: default_compression_level(),
            archive_retention_days: default_archive_retention_days(),
            ai_summarization: false,
            ai_provider: None,
        }
    }
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            target_fps: default_fps(),
            show_line_numbers: false,
            theme: default_theme(),
            mouse_support: true,
            keybindings: default_keybindings(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            file: None,
            format: default_log_format(),
            timestamps: true,
            file_line: false,
        }
    }
}

impl AppConfig {
    /// Load configuration from files and environment
    ///
    /// Configuration is loaded in the following order (later overrides earlier):
    /// 1. Default values
    /// 2. System config: /etc/agterm/config.toml
    /// 3. User config: ~/.config/agterm/config.toml (XDG)
    /// 4. Local config: ./.agterm/config.toml
    /// 5. Environment variables: AGTERM_*
    pub fn load() -> ConfigResult<Self> {
        let mut builder = Config::builder();

        // 1. Start with defaults
        builder = builder.add_source(
            config::File::from_str(
                include_str!("../../default_config.toml"),
                config::FileFormat::Toml,
            )
            .required(false),
        );

        // 2. System config (optional)
        #[cfg(unix)]
        {
            builder = builder.add_source(File::with_name("/etc/agterm/config").required(false));
        }

        // 3. User config (XDG)
        if let Some(proj_dirs) = ProjectDirs::from("com", "agterm", "agterm") {
            let config_path = proj_dirs.config_dir().join("config");
            builder = builder
                .add_source(File::with_name(config_path.to_str().unwrap_or("")).required(false));
        }

        // 4. Local config
        builder = builder.add_source(File::with_name(".agterm/config").required(false));

        // 5. Environment variables (AGTERM_*)
        builder = builder.add_source(
            Environment::with_prefix("AGTERM")
                .separator("__")
                .try_parsing(true),
        );

        // Build and deserialize
        let config = builder.build()?;
        let app_config: AppConfig = config
            .try_deserialize()
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        Ok(app_config)
    }

    /// Load configuration with a custom config file path
    pub fn load_with_file(path: &PathBuf) -> ConfigResult<Self> {
        if !path.exists() {
            return Err(ConfigError::FileNotFound { path: path.clone() });
        }

        let mut builder = Config::builder();

        // Start with defaults
        builder = builder.add_source(
            config::File::from_str(
                include_str!("../../default_config.toml"),
                config::FileFormat::Toml,
            )
            .required(false),
        );

        // Add custom file
        builder = builder.add_source(File::from(path.clone()).required(true));

        // Environment variables
        builder = builder.add_source(
            Environment::with_prefix("AGTERM")
                .separator("__")
                .try_parsing(true),
        );

        let config = builder.build()?;
        let app_config: AppConfig = config
            .try_deserialize()
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        Ok(app_config)
    }

    /// Get the data directory path
    #[must_use]
    pub fn data_dir(&self) -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "agterm", &self.general.app_name) {
            proj_dirs.data_dir().to_path_buf()
        } else {
            PathBuf::from(".agterm/data")
        }
    }

    /// Get the database path
    #[must_use]
    pub fn database_path(&self) -> PathBuf {
        self.storage
            .database_path
            .clone()
            .unwrap_or_else(|| self.data_dir().join("agterm.db"))
    }

    /// Get the logs directory
    #[must_use]
    pub fn logs_dir(&self) -> PathBuf {
        self.storage
            .logs_dir
            .clone()
            .unwrap_or_else(|| self.data_dir().join("logs"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.general.app_name, "agterm");
        assert_eq!(config.pty.max_sessions, 32);
        assert_eq!(config.tui.target_fps, 60);
    }

    #[test]
    fn test_config_paths() {
        let config = AppConfig::default();
        let db_path = config.database_path();
        assert!(db_path.to_string_lossy().contains("agterm.db"));
    }
}
