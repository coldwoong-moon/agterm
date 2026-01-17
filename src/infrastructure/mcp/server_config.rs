//! MCP Server Configuration
//!
//! Configuration types for MCP server connections.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Transport type for MCP server connection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum McpTransport {
    /// Standard I/O transport (child process)
    Stdio {
        /// Command to execute
        command: String,
        /// Command arguments
        #[serde(default)]
        args: Vec<String>,
        /// Working directory
        #[serde(default)]
        working_dir: Option<PathBuf>,
        /// Environment variables
        #[serde(default)]
        env: HashMap<String, String>,
    },
    /// Server-Sent Events transport (HTTP)
    Sse {
        /// SSE endpoint URL
        url: String,
        /// Optional authorization header
        #[serde(default)]
        auth_token: Option<String>,
    },
}

/// MCP Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Unique server name/identifier
    pub name: String,
    /// Human-readable description
    #[serde(default)]
    pub description: Option<String>,
    /// Transport configuration
    pub transport: McpTransport,
    /// Whether to auto-connect on startup
    #[serde(default)]
    pub auto_connect: bool,
    /// Connection timeout in milliseconds
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    /// Custom metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

fn default_timeout() -> u64 {
    30000 // 30 seconds
}

impl McpServerConfig {
    /// Create a new stdio server configuration
    pub fn stdio(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            transport: McpTransport::Stdio {
                command: command.into(),
                args: Vec::new(),
                working_dir: None,
                env: HashMap::new(),
            },
            auto_connect: false,
            timeout_ms: default_timeout(),
            metadata: HashMap::new(),
        }
    }

    /// Create a new SSE server configuration
    pub fn sse(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            transport: McpTransport::Sse {
                url: url.into(),
                auth_token: None,
            },
            auto_connect: false,
            timeout_ms: default_timeout(),
            metadata: HashMap::new(),
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set auto-connect
    pub fn with_auto_connect(mut self, auto_connect: bool) -> Self {
        self.auto_connect = auto_connect;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Add command arguments (for stdio transport)
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        if let McpTransport::Stdio { args: ref mut a, .. } = self.transport {
            *a = args;
        }
        self
    }

    /// Set working directory (for stdio transport)
    pub fn with_working_dir(mut self, dir: PathBuf) -> Self {
        if let McpTransport::Stdio { working_dir: ref mut w, .. } = self.transport {
            *w = Some(dir);
        }
        self
    }

    /// Add environment variable (for stdio transport)
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        if let McpTransport::Stdio { env: ref mut e, .. } = self.transport {
            e.insert(key.into(), value.into());
        }
        self
    }

    /// Set auth token (for SSE transport)
    pub fn with_auth_token(mut self, token: impl Into<String>) -> Self {
        if let McpTransport::Sse { auth_token: ref mut t, .. } = self.transport {
            *t = Some(token.into());
        }
        self
    }

    /// Check if this is a stdio transport
    pub fn is_stdio(&self) -> bool {
        matches!(self.transport, McpTransport::Stdio { .. })
    }

    /// Check if this is an SSE transport
    pub fn is_sse(&self) -> bool {
        matches!(self.transport, McpTransport::Sse { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdio_config() {
        let config = McpServerConfig::stdio("git-server", "uvx")
            .with_args(vec!["mcp-server-git".to_string()])
            .with_description("Git MCP server")
            .with_auto_connect(true);

        assert_eq!(config.name, "git-server");
        assert!(config.is_stdio());
        assert!(config.auto_connect);

        if let McpTransport::Stdio { command, args, .. } = &config.transport {
            assert_eq!(command, "uvx");
            assert_eq!(args, &vec!["mcp-server-git".to_string()]);
        }
    }

    #[test]
    fn test_sse_config() {
        let config = McpServerConfig::sse("remote-server", "http://localhost:8000/sse")
            .with_auth_token("secret-token")
            .with_timeout(60000);

        assert_eq!(config.name, "remote-server");
        assert!(config.is_sse());
        assert_eq!(config.timeout_ms, 60000);

        if let McpTransport::Sse { url, auth_token } = &config.transport {
            assert_eq!(url, "http://localhost:8000/sse");
            assert_eq!(auth_token, &Some("secret-token".to_string()));
        }
    }

    #[test]
    fn test_serialization() {
        let config = McpServerConfig::stdio("test", "echo")
            .with_args(vec!["hello".to_string()]);

        let json = serde_json::to_string(&config).unwrap();
        let parsed: McpServerConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, config.name);
    }
}
