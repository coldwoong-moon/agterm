//! MCP Client Implementation
//!
//! Provides client functionality for connecting to MCP servers including:
//! - Connection management with multiple transport types
//! - Capability discovery (tools, prompts, resources)
//! - Message sending and tool invocation
//! - Error handling and retry logic

use super::server_config::{ServerConfig, ServerProfile, TransportConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// MCP Client for communicating with MCP servers
pub struct McpClient {
    /// Server configuration
    config: ServerConfig,

    /// Connection state
    state: ClientState,

    /// Discovered server capabilities
    capabilities: Option<ServerCapabilities>,
}

/// Client connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClientState {
    /// Not connected
    Disconnected,

    /// Currently connecting
    Connecting,

    /// Connected and ready
    Connected,

    /// Connection failed
    #[allow(dead_code)]
    Failed,
}

/// Server capabilities discovered during connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Available tools
    pub tools: Vec<ToolInfo>,

    /// Available prompts
    pub prompts: Vec<PromptInfo>,

    /// Available resources
    pub resources: Vec<ResourceInfo>,

    /// Server metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Information about an available tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    /// Tool name
    pub name: String,

    /// Tool description
    pub description: String,

    /// Input schema (JSON Schema)
    pub input_schema: serde_json::Value,

    /// Whether the tool is deprecated
    #[serde(default)]
    pub deprecated: bool,
}

/// Information about an available prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptInfo {
    /// Prompt name
    pub name: String,

    /// Prompt description
    pub description: String,

    /// Required arguments
    #[serde(default)]
    pub arguments: Vec<PromptArgument>,
}

/// Prompt argument definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    /// Argument name
    pub name: String,

    /// Argument description
    pub description: String,

    /// Whether the argument is required
    #[serde(default)]
    pub required: bool,
}

/// Information about an available resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    /// Resource URI
    pub uri: String,

    /// Resource name
    pub name: String,

    /// Resource description
    pub description: String,

    /// MIME type
    #[serde(default)]
    pub mime_type: Option<String>,
}

/// Request message to send to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    /// Request method
    pub method: String,

    /// Request parameters
    #[serde(default)]
    pub params: serde_json::Value,

    /// Request ID (for tracking)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Response message from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    /// Response result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,

    /// Error if request failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,

    /// Response ID (matching request)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Error response from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    /// Error code
    pub code: i32,

    /// Error message
    pub message: String,

    /// Additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Tool call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool name
    pub name: String,

    /// Tool arguments
    #[serde(default)]
    pub arguments: serde_json::Value,
}

/// Tool call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Tool output content
    pub content: Vec<ToolContent>,

    /// Whether the call failed
    #[serde(default)]
    pub is_error: bool,
}

/// Tool output content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ToolContent {
    /// Text content
    Text {
        /// Text content
        text: String,
    },

    /// Image content
    Image {
        /// Image data (base64)
        data: String,

        /// MIME type
        mime_type: String,
    },

    /// Resource reference
    Resource {
        /// Resource URI
        uri: String,
    },
}

impl McpClient {
    /// Create a new MCP client from server profile
    pub fn new(profile: &ServerProfile) -> Self {
        Self {
            config: profile.config.clone(),
            state: ClientState::Disconnected,
            capabilities: None,
        }
    }

    /// Create a new MCP client from server config
    pub fn from_config(config: ServerConfig) -> Self {
        Self {
            config,
            state: ClientState::Disconnected,
            capabilities: None,
        }
    }

    /// Connect to the MCP server
    pub async fn connect(&mut self) -> Result<(), McpClientError> {
        if self.state == ClientState::Connected {
            return Ok(());
        }

        self.state = ClientState::Connecting;

        // Clone transport to avoid borrow checker issues
        let transport = self.config.transport.clone();

        // Implement connection logic based on transport type
        match transport {
            TransportConfig::Stdio { command, args, env } => {
                self.connect_stdio(&command, &args, &env).await?;
            }
            TransportConfig::Http { url, headers } => {
                self.connect_http(&url, &headers).await?;
            }
            TransportConfig::WebSocket { url, auth_token } => {
                self.connect_websocket(&url, auth_token.as_deref()).await?;
            }
        }

        self.state = ClientState::Connected;
        Ok(())
    }

    /// Connect via stdio transport (child process)
    async fn connect_stdio(
        &mut self,
        command: &str,
        args: &[String],
        _env: &HashMap<String, String>,
    ) -> Result<(), McpClientError> {
        // In a full implementation, this would:
        // 1. Spawn the child process with command and args
        // 2. Set up stdio pipes for communication
        // 3. Initialize the MCP protocol handshake
        // 4. Store the process handle for later communication

        // Placeholder implementation
        log::info!("Connecting to MCP server via stdio: {command} {args:?}");

        // Simulate connection delay
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(())
    }

    /// Connect via HTTP/SSE transport
    async fn connect_http(
        &mut self,
        url: &str,
        _headers: &HashMap<String, String>,
    ) -> Result<(), McpClientError> {
        // In a full implementation, this would:
        // 1. Create HTTP client with appropriate headers
        // 2. Establish SSE connection for server events
        // 3. Send initial handshake request
        // 4. Set up event stream handler

        // Placeholder implementation
        log::info!("Connecting to MCP server via HTTP: {url}");

        // Simulate connection delay
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(())
    }

    /// Connect via WebSocket transport
    async fn connect_websocket(
        &mut self,
        url: &str,
        _auth_token: Option<&str>,
    ) -> Result<(), McpClientError> {
        // In a full implementation, this would:
        // 1. Create WebSocket client
        // 2. Add authentication if provided
        // 3. Establish WebSocket connection
        // 4. Send initial handshake
        // 5. Set up message handler

        // Placeholder implementation
        log::info!("Connecting to MCP server via WebSocket: {url}");

        // Simulate connection delay
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(())
    }

    /// Disconnect from the server
    pub async fn disconnect(&mut self) -> Result<(), McpClientError> {
        if self.state == ClientState::Disconnected {
            return Ok(());
        }

        // Close connection based on transport type
        log::info!("Disconnecting from MCP server");

        self.state = ClientState::Disconnected;
        self.capabilities = None;

        Ok(())
    }

    /// Discover server capabilities
    pub async fn discover_capabilities(&mut self) -> Result<ServerCapabilities, McpClientError> {
        if self.state != ClientState::Connected {
            return Err(McpClientError::NotConnected);
        }

        // If already discovered, return cached capabilities
        if let Some(ref caps) = self.capabilities {
            return Ok(caps.clone());
        }

        // Request capabilities from server
        let tools = self.list_tools().await?;
        let prompts = self.list_prompts().await?;
        let resources = self.list_resources().await?;

        let capabilities = ServerCapabilities {
            tools,
            prompts,
            resources,
            metadata: HashMap::new(),
        };

        self.capabilities = Some(capabilities.clone());
        Ok(capabilities)
    }

    /// List available tools
    async fn list_tools(&self) -> Result<Vec<ToolInfo>, McpClientError> {
        let request = McpRequest {
            method: "tools/list".to_string(),
            params: serde_json::Value::Null,
            id: Some(uuid::Uuid::new_v4().to_string()),
        };

        let response = self.send_request(&request).await?;

        // Parse tools from response
        let tools = response
            .result
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        Ok(tools)
    }

    /// List available prompts
    async fn list_prompts(&self) -> Result<Vec<PromptInfo>, McpClientError> {
        let request = McpRequest {
            method: "prompts/list".to_string(),
            params: serde_json::Value::Null,
            id: Some(uuid::Uuid::new_v4().to_string()),
        };

        let response = self.send_request(&request).await?;

        // Parse prompts from response
        let prompts = response
            .result
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        Ok(prompts)
    }

    /// List available resources
    async fn list_resources(&self) -> Result<Vec<ResourceInfo>, McpClientError> {
        let request = McpRequest {
            method: "resources/list".to_string(),
            params: serde_json::Value::Null,
            id: Some(uuid::Uuid::new_v4().to_string()),
        };

        let response = self.send_request(&request).await?;

        // Parse resources from response
        let resources = response
            .result
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        Ok(resources)
    }

    /// Send a message to the server
    pub async fn send_message(&self, message: &str) -> Result<McpResponse, McpClientError> {
        if self.state != ClientState::Connected {
            return Err(McpClientError::NotConnected);
        }

        let request = McpRequest {
            method: "message/send".to_string(),
            params: serde_json::json!({ "content": message }),
            id: Some(uuid::Uuid::new_v4().to_string()),
        };

        self.send_request(&request).await
    }

    /// Call a tool on the server
    pub async fn call_tool(&self, tool_call: &ToolCall) -> Result<ToolResult, McpClientError> {
        if self.state != ClientState::Connected {
            return Err(McpClientError::NotConnected);
        }

        let request = McpRequest {
            method: "tools/call".to_string(),
            params: serde_json::to_value(tool_call)
                .map_err(|e| McpClientError::SerializationError(e.to_string()))?,
            id: Some(uuid::Uuid::new_v4().to_string()),
        };

        let response = self.send_request_with_retry(&request).await?;

        if let Some(error) = response.error {
            return Err(McpClientError::ServerError(error.message));
        }

        let result = response
            .result
            .ok_or_else(|| McpClientError::InvalidResponse("Missing result".to_string()))?;

        serde_json::from_value(result)
            .map_err(|e| McpClientError::InvalidResponse(e.to_string()))
    }

    /// Send a request to the server
    async fn send_request(&self, request: &McpRequest) -> Result<McpResponse, McpClientError> {
        // In a full implementation, this would:
        // 1. Serialize the request
        // 2. Send via the appropriate transport
        // 3. Wait for response with timeout
        // 4. Deserialize and return response

        // Placeholder implementation - simulate request/response
        log::debug!("Sending request: {request:?}");

        // Simulate network delay
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Simulate successful response
        Ok(McpResponse {
            result: Some(serde_json::json!({"status": "ok"})),
            error: None,
            id: request.id.clone(),
        })
    }

    /// Send a request with retry logic
    async fn send_request_with_retry(
        &self,
        request: &McpRequest,
    ) -> Result<McpResponse, McpClientError> {
        let retry_config = &self.config.retry;
        let mut last_error = None;

        for attempt in 0..=retry_config.max_retries {
            if attempt > 0 {
                let delay = Duration::from_millis(
                    retry_config.base_delay_ms * (retry_config.backoff_factor.powi(attempt as i32 - 1) as u64)
                );
                log::debug!("Retrying request after {delay:?} (attempt {attempt})");
                tokio::time::sleep(delay).await;
            }

            match self.send_request(request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    log::warn!("Request failed (attempt {}): {}", attempt + 1, e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or(McpClientError::MaxRetriesExceeded))
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.state == ClientState::Connected
    }

    /// Get cached capabilities
    pub fn capabilities(&self) -> Option<&ServerCapabilities> {
        self.capabilities.as_ref()
    }

    /// Get server config
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }
}

/// MCP client errors
#[derive(Debug, thiserror::Error)]
pub enum McpClientError {
    #[error("Not connected to server")]
    NotConnected,

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Maximum retries exceeded")]
    MaxRetriesExceeded,

    #[error("Timeout")]
    Timeout,

    #[error("IO error: {0}")]
    IoError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::server_config::{RetryConfig, ServerProfile};

    #[tokio::test]
    async fn test_client_creation() {
        let profile = ServerProfile {
            name: "test".to_string(),
            description: "Test server".to_string(),
            config: ServerConfig {
                name: "test".to_string(),
                server_type: crate::mcp::server_config::ServerType::Custom,
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

        let client = McpClient::new(&profile);
        assert!(!client.is_connected());
        assert!(client.capabilities().is_none());
    }

    #[tokio::test]
    async fn test_connect_disconnect() {
        let profile = ServerProfile {
            name: "test".to_string(),
            description: "Test server".to_string(),
            config: ServerConfig {
                name: "test".to_string(),
                server_type: crate::mcp::server_config::ServerType::Custom,
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

        let mut client = McpClient::new(&profile);

        // Connect
        client.connect().await.unwrap();
        assert!(client.is_connected());

        // Disconnect
        client.disconnect().await.unwrap();
        assert!(!client.is_connected());
    }
}
