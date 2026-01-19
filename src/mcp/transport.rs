use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Transport type for MCP connections
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TransportType {
    /// Standard input/output transport (spawns a child process)
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

    /// HTTP transport
    Http {
        /// Base URL for the HTTP endpoint
        url: String,
        /// Optional HTTP headers
        #[serde(default)]
        headers: HashMap<String, String>,
    },

    /// Server-Sent Events (SSE) transport
    Sse {
        /// SSE endpoint URL
        url: String,
    },
}

impl TransportType {
    /// Create a new Stdio transport
    pub fn stdio(command: impl Into<String>) -> Self {
        Self::Stdio {
            command: command.into(),
            args: Vec::new(),
            env: HashMap::new(),
        }
    }

    /// Create a new Stdio transport with arguments
    pub fn stdio_with_args(command: impl Into<String>, args: Vec<String>) -> Self {
        Self::Stdio {
            command: command.into(),
            args,
            env: HashMap::new(),
        }
    }

    /// Create a new Stdio transport with environment variables
    pub fn stdio_with_env(
        command: impl Into<String>,
        args: Vec<String>,
        env: HashMap<String, String>,
    ) -> Self {
        Self::Stdio {
            command: command.into(),
            args,
            env,
        }
    }

    /// Create a new HTTP transport
    pub fn http(url: impl Into<String>) -> Self {
        Self::Http {
            url: url.into(),
            headers: HashMap::new(),
        }
    }

    /// Create a new HTTP transport with headers
    pub fn http_with_headers(url: impl Into<String>, headers: HashMap<String, String>) -> Self {
        Self::Http {
            url: url.into(),
            headers,
        }
    }

    /// Create a new SSE transport
    pub fn sse(url: impl Into<String>) -> Self {
        Self::Sse {
            url: url.into(),
        }
    }

    /// Get a human-readable description of the transport
    pub fn description(&self) -> String {
        match self {
            Self::Stdio { command, args, .. } => {
                if args.is_empty() {
                    format!("stdio: {}", command)
                } else {
                    format!("stdio: {} {}", command, args.join(" "))
                }
            }
            Self::Http { url, .. } => format!("http: {}", url),
            Self::Sse { url } => format!("sse: {}", url),
        }
    }

    /// Check if this transport requires network access
    pub fn requires_network(&self) -> bool {
        matches!(self, Self::Http { .. } | Self::Sse { .. })
    }

    /// Check if this transport spawns a local process
    pub fn is_local_process(&self) -> bool {
        matches!(self, Self::Stdio { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdio_creation() {
        let transport = TransportType::stdio("python");
        match transport {
            TransportType::Stdio { command, args, env } => {
                assert_eq!(command, "python");
                assert!(args.is_empty());
                assert!(env.is_empty());
            }
            _ => panic!("Expected Stdio transport"),
        }
    }

    #[test]
    fn test_stdio_with_args() {
        let transport = TransportType::stdio_with_args(
            "python",
            vec!["-m".to_string(), "my_server".to_string()],
        );
        match transport {
            TransportType::Stdio { command, args, .. } => {
                assert_eq!(command, "python");
                assert_eq!(args, vec!["-m", "my_server"]);
            }
            _ => panic!("Expected Stdio transport"),
        }
    }

    #[test]
    fn test_stdio_with_env() {
        let mut env = HashMap::new();
        env.insert("API_KEY".to_string(), "secret".to_string());

        let transport = TransportType::stdio_with_env("node", vec!["server.js".to_string()], env.clone());
        match transport {
            TransportType::Stdio { command, args, env: transport_env } => {
                assert_eq!(command, "node");
                assert_eq!(args, vec!["server.js"]);
                assert_eq!(transport_env.get("API_KEY").unwrap(), "secret");
            }
            _ => panic!("Expected Stdio transport"),
        }
    }

    #[test]
    fn test_http_creation() {
        let transport = TransportType::http("http://localhost:8080");
        match transport {
            TransportType::Http { url, headers } => {
                assert_eq!(url, "http://localhost:8080");
                assert!(headers.is_empty());
            }
            _ => panic!("Expected Http transport"),
        }
    }

    #[test]
    fn test_http_with_headers() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token".to_string());

        let transport = TransportType::http_with_headers("https://api.example.com", headers.clone());
        match transport {
            TransportType::Http { url, headers: transport_headers } => {
                assert_eq!(url, "https://api.example.com");
                assert_eq!(transport_headers.get("Authorization").unwrap(), "Bearer token");
            }
            _ => panic!("Expected Http transport"),
        }
    }

    #[test]
    fn test_sse_creation() {
        let transport = TransportType::sse("https://events.example.com/stream");
        match transport {
            TransportType::Sse { url } => {
                assert_eq!(url, "https://events.example.com/stream");
            }
            _ => panic!("Expected Sse transport"),
        }
    }

    #[test]
    fn test_description() {
        let stdio = TransportType::stdio("python");
        assert_eq!(stdio.description(), "stdio: python");

        let stdio_args = TransportType::stdio_with_args("python", vec!["-m".to_string(), "server".to_string()]);
        assert_eq!(stdio_args.description(), "stdio: python -m server");

        let http = TransportType::http("http://localhost:8080");
        assert_eq!(http.description(), "http: http://localhost:8080");

        let sse = TransportType::sse("https://events.example.com");
        assert_eq!(sse.description(), "sse: https://events.example.com");
    }

    #[test]
    fn test_requires_network() {
        let stdio = TransportType::stdio("python");
        assert!(!stdio.requires_network());

        let http = TransportType::http("http://localhost:8080");
        assert!(http.requires_network());

        let sse = TransportType::sse("https://events.example.com");
        assert!(sse.requires_network());
    }

    #[test]
    fn test_is_local_process() {
        let stdio = TransportType::stdio("python");
        assert!(stdio.is_local_process());

        let http = TransportType::http("http://localhost:8080");
        assert!(!http.is_local_process());

        let sse = TransportType::sse("https://events.example.com");
        assert!(!sse.is_local_process());
    }

    #[test]
    fn test_serialization() {
        let stdio = TransportType::stdio_with_args("python", vec!["-m".to_string(), "server".to_string()]);
        let json = serde_json::to_string(&stdio).unwrap();
        let deserialized: TransportType = serde_json::from_str(&json).unwrap();
        assert_eq!(stdio, deserialized);

        let http = TransportType::http("http://localhost:8080");
        let json = serde_json::to_string(&http).unwrap();
        let deserialized: TransportType = serde_json::from_str(&json).unwrap();
        assert_eq!(http, deserialized);

        let sse = TransportType::sse("https://events.example.com");
        let json = serde_json::to_string(&sse).unwrap();
        let deserialized: TransportType = serde_json::from_str(&json).unwrap();
        assert_eq!(sse, deserialized);
    }

    #[test]
    fn test_clone() {
        let original = TransportType::stdio_with_args("node", vec!["server.js".to_string()]);
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }
}
