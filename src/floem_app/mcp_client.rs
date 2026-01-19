//! MCP Client for connecting to external MCP servers
//!
//! This module provides a client implementation for connecting to MCP (Model Context Protocol)
//! servers. It enables AgTerm to communicate with AI agents like Claude Code, Gemini CLI, etc.
//!
//! # Protocol
//!
//! The MCP protocol uses JSON-RPC 2.0 over stdio (stdin/stdout) for communication.
//! The client spawns an external process and communicates via pipes.
//!
//! # Example
//!
//! ```ignore
//! let client = McpClient::connect_stdio("claude", &["--mcp-server"]).await?;
//! let tools = client.list_tools().await?;
//! let result = client.call_tool("run_command", json!({"command": "ls"})).await?;
//! client.disconnect().await?;
//! ```

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: Option<u64>,
    result: Option<Value>,
    error: Option<JsonRpcErrorResponse>,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Deserialize)]
struct JsonRpcErrorResponse {
    code: i64,
    message: String,
    #[allow(dead_code)]
    data: Option<Value>,
}

/// JSON-RPC 2.0 Notification (no id, no response expected)
#[derive(Debug, Clone, Serialize)]
struct JsonRpcNotification {
    jsonrpc: &'static str,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

/// Server capabilities from initialize response
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ServerCapabilities {
    /// Tool capabilities
    #[serde(default)]
    pub tools: Option<ToolsCapability>,
    /// Prompt capabilities
    #[serde(default)]
    pub prompts: Option<PromptsCapability>,
    /// Resource capabilities
    #[serde(default)]
    pub resources: Option<ResourcesCapability>,
}

/// Tool capability details
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ToolsCapability {
    /// Whether list_changed notifications are supported
    #[serde(default, rename = "listChanged")]
    pub list_changed: bool,
}

/// Prompt capability details
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PromptsCapability {
    /// Whether list_changed notifications are supported
    #[serde(default, rename = "listChanged")]
    pub list_changed: bool,
}

/// Resource capability details
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ResourcesCapability {
    /// Whether subscribe is supported
    #[serde(default)]
    pub subscribe: bool,
    /// Whether list_changed notifications are supported
    #[serde(default, rename = "listChanged")]
    pub list_changed: bool,
}

/// Tool information from the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    /// Tool name
    pub name: String,
    /// Tool description
    #[serde(default)]
    pub description: Option<String>,
    /// Input schema (JSON Schema)
    #[serde(default, rename = "inputSchema")]
    pub input_schema: Option<Value>,
}

/// Tool list response
#[derive(Debug, Clone, Deserialize)]
struct ToolsListResponse {
    tools: Vec<McpTool>,
}

/// Tool call result
#[derive(Debug, Clone, Deserialize)]
pub struct ToolCallResult {
    /// Result content
    pub content: Vec<ToolResultContent>,
    /// Whether the call resulted in an error
    #[serde(default)]
    pub is_error: bool,
}

/// Content item in tool result
#[derive(Debug, Clone, Deserialize)]
pub struct ToolResultContent {
    /// Content type (usually "text")
    #[serde(rename = "type")]
    pub content_type: String,
    /// Text content
    pub text: Option<String>,
}

/// MCP Client errors
#[derive(Debug)]
pub enum McpError {
    /// IO error (process spawn, read, write)
    Io(std::io::Error),
    /// JSON serialization/deserialization error
    Json(serde_json::Error),
    /// Protocol error (invalid response, unexpected state)
    Protocol(String),
    /// Server returned an error
    ServerError { code: i64, message: String },
    /// Connection is not established
    NotConnected,
    /// Timeout waiting for response
    Timeout,
}

impl std::fmt::Display for McpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpError::Io(e) => write!(f, "IO error: {e}"),
            McpError::Json(e) => write!(f, "JSON error: {e}"),
            McpError::Protocol(msg) => write!(f, "Protocol error: {msg}"),
            McpError::ServerError { code, message } => {
                write!(f, "Server error ({code}): {message}")
            }
            McpError::NotConnected => write!(f, "Not connected to MCP server"),
            McpError::Timeout => write!(f, "Timeout waiting for response"),
        }
    }
}

impl std::error::Error for McpError {}

impl From<std::io::Error> for McpError {
    fn from(e: std::io::Error) -> Self {
        McpError::Io(e)
    }
}

impl From<serde_json::Error> for McpError {
    fn from(e: serde_json::Error) -> Self {
        McpError::Json(e)
    }
}

/// MCP Client for connecting to external MCP servers
pub struct McpClient {
    /// Child process handle
    process: Option<Child>,
    /// Stdin writer (wrapped in Mutex for thread safety)
    stdin: Option<Arc<Mutex<tokio::process::ChildStdin>>>,
    /// Stdout reader (wrapped in Mutex for thread safety)
    stdout: Option<Arc<Mutex<BufReader<tokio::process::ChildStdout>>>>,
    /// Request ID counter
    request_id: AtomicU64,
    /// Server capabilities (set after initialize)
    capabilities: Option<ServerCapabilities>,
    /// Server name (for display)
    server_name: String,
}

impl McpClient {
    /// Create a new unconnected client
    pub fn new() -> Self {
        Self {
            process: None,
            stdin: None,
            stdout: None,
            request_id: AtomicU64::new(0),
            capabilities: None,
            server_name: String::new(),
        }
    }

    /// Connect to an MCP server via stdio
    ///
    /// # Arguments
    ///
    /// * `command` - The command to run (e.g., "claude", "gemini")
    /// * `args` - Command line arguments (e.g., &["--mcp-server"])
    ///
    /// # Returns
    ///
    /// A connected McpClient instance
    pub async fn connect_stdio(command: &str, args: &[&str]) -> Result<Self, McpError> {
        tracing::info!("Connecting to MCP server: {} {:?}", command, args);

        let mut process = Command::new(command)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let stdin = process
            .stdin
            .take()
            .ok_or_else(|| McpError::Protocol("Failed to capture stdin".into()))?;

        let stdout = process
            .stdout
            .take()
            .ok_or_else(|| McpError::Protocol("Failed to capture stdout".into()))?;

        let mut client = Self {
            process: Some(process),
            stdin: Some(Arc::new(Mutex::new(stdin))),
            stdout: Some(Arc::new(Mutex::new(BufReader::new(stdout)))),
            request_id: AtomicU64::new(0),
            capabilities: None,
            server_name: command.to_string(),
        };

        // Perform MCP initialization handshake
        client.initialize().await?;

        tracing::info!("Connected to MCP server: {}", command);

        Ok(client)
    }

    /// Initialize the MCP connection (handshake)
    async fn initialize(&mut self) -> Result<(), McpError> {
        // Send initialize request
        let init_params = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "agterm",
                "version": env!("CARGO_PKG_VERSION")
            }
        });

        let response = self
            .send_request("initialize", Some(init_params))
            .await?;

        // Parse capabilities from response
        if let Some(caps) = response.get("capabilities") {
            self.capabilities = Some(serde_json::from_value(caps.clone()).unwrap_or_default());
        }

        // Send initialized notification
        self.send_notification("notifications/initialized", None)
            .await?;

        Ok(())
    }

    /// List available tools from the MCP server
    pub async fn list_tools(&mut self) -> Result<Vec<McpTool>, McpError> {
        let response = self.send_request("tools/list", None).await?;

        let tools_response: ToolsListResponse = serde_json::from_value(response)?;

        Ok(tools_response.tools)
    }

    /// Call a tool on the MCP server
    ///
    /// # Arguments
    ///
    /// * `name` - Tool name
    /// * `arguments` - Tool arguments as JSON
    ///
    /// # Returns
    ///
    /// The tool call result
    pub async fn call_tool(
        &mut self,
        name: &str,
        arguments: Value,
    ) -> Result<ToolCallResult, McpError> {
        let params = serde_json::json!({
            "name": name,
            "arguments": arguments
        });

        let response = self.send_request("tools/call", Some(params)).await?;

        let result: ToolCallResult = serde_json::from_value(response)?;

        Ok(result)
    }

    /// Disconnect from the MCP server
    pub async fn disconnect(&mut self) -> Result<(), McpError> {
        if let Some(mut process) = self.process.take() {
            tracing::info!("Disconnecting from MCP server: {}", self.server_name);

            // Drop stdin/stdout to close pipes
            self.stdin = None;
            self.stdout = None;

            // Kill the process
            let _ = process.kill().await;
        }

        self.capabilities = None;

        Ok(())
    }

    /// Check if connected to an MCP server
    pub fn is_connected(&self) -> bool {
        self.process.is_some()
    }

    /// Get the server name
    pub fn server_name(&self) -> &str {
        &self.server_name
    }

    /// Get server capabilities
    pub fn capabilities(&self) -> Option<&ServerCapabilities> {
        self.capabilities.as_ref()
    }

    /// Get the next request ID
    fn next_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Send a JSON-RPC request and wait for response
    async fn send_request(
        &mut self,
        method: &str,
        params: Option<Value>,
    ) -> Result<Value, McpError> {
        let stdin = self
            .stdin
            .as_ref()
            .ok_or(McpError::NotConnected)?
            .clone();

        let stdout = self
            .stdout
            .as_ref()
            .ok_or(McpError::NotConnected)?
            .clone();

        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id: self.next_id(),
            method: method.to_string(),
            params,
        };

        let request_json = serde_json::to_string(&request)?;
        tracing::debug!("MCP Request: {}", request_json);

        // Write request
        {
            let mut stdin_guard = stdin.lock().await;
            stdin_guard
                .write_all(request_json.as_bytes())
                .await?;
            stdin_guard.write_all(b"\n").await?;
            stdin_guard.flush().await?;
        }

        // Read response with timeout
        let response_json = {
            let mut stdout_guard = stdout.lock().await;
            let mut line = String::new();

            // TODO: Add proper timeout handling
            stdout_guard.read_line(&mut line).await?;

            line
        };

        tracing::debug!("MCP Response: {}", response_json.trim());

        // Parse response
        let response: JsonRpcResponse = serde_json::from_str(&response_json)?;

        // Check for errors
        if let Some(error) = response.error {
            return Err(McpError::ServerError {
                code: error.code,
                message: error.message,
            });
        }

        response.result.ok_or_else(|| {
            McpError::Protocol("Response has neither result nor error".into())
        })
    }

    /// Send a JSON-RPC notification (no response expected)
    async fn send_notification(
        &mut self,
        method: &str,
        params: Option<Value>,
    ) -> Result<(), McpError> {
        let stdin = self
            .stdin
            .as_ref()
            .ok_or(McpError::NotConnected)?
            .clone();

        let notification = JsonRpcNotification {
            jsonrpc: "2.0",
            method: method.to_string(),
            params,
        };

        let notification_json = serde_json::to_string(&notification)?;
        tracing::debug!("MCP Notification: {}", notification_json);

        // Write notification
        {
            let mut stdin_guard = stdin.lock().await;
            stdin_guard
                .write_all(notification_json.as_bytes())
                .await?;
            stdin_guard.write_all(b"\n").await?;
            stdin_guard.flush().await?;
        }

        Ok(())
    }
}

impl Default for McpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        // Try to kill the process if it's still running
        if let Some(mut process) = self.process.take() {
            let _ = process.start_kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_client_new() {
        let client = McpClient::new();
        assert!(!client.is_connected());
        assert!(client.server_name().is_empty());
        assert!(client.capabilities().is_none());
    }

    #[test]
    fn test_mcp_error_display() {
        let io_err = McpError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "test",
        ));
        assert!(io_err.to_string().contains("IO error"));

        let protocol_err = McpError::Protocol("test error".into());
        assert!(protocol_err.to_string().contains("Protocol error"));

        let server_err = McpError::ServerError {
            code: -32600,
            message: "Invalid request".into(),
        };
        assert!(server_err.to_string().contains("-32600"));

        let not_connected = McpError::NotConnected;
        assert!(not_connected.to_string().contains("Not connected"));

        let timeout = McpError::Timeout;
        assert!(timeout.to_string().contains("Timeout"));
    }

    #[test]
    fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "test".to_string(),
            params: Some(serde_json::json!({"key": "value"})),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"method\":\"test\""));
    }

    #[test]
    fn test_json_rpc_request_without_params() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "test".to_string(),
            params: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(!json.contains("params"));
    }

    #[test]
    fn test_json_rpc_response_parsing() {
        let json = r#"{"jsonrpc":"2.0","id":1,"result":{"status":"ok"}}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[test]
    fn test_json_rpc_error_response_parsing() {
        let json =
            r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32600,"message":"Invalid Request"}}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32600);
        assert_eq!(error.message, "Invalid Request");
    }

    #[test]
    fn test_mcp_tool_parsing() {
        let json = r#"{
            "name": "test_tool",
            "description": "A test tool",
            "inputSchema": {"type": "object"}
        }"#;
        let tool: McpTool = serde_json::from_str(json).unwrap();
        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.description, Some("A test tool".into()));
        assert!(tool.input_schema.is_some());
    }

    #[test]
    fn test_tools_list_response_parsing() {
        let json = r#"{"tools":[
            {"name":"tool1","description":"First tool"},
            {"name":"tool2"}
        ]}"#;
        let response: ToolsListResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.tools.len(), 2);
        assert_eq!(response.tools[0].name, "tool1");
        assert_eq!(response.tools[1].description, None);
    }

    #[test]
    fn test_server_capabilities_parsing() {
        let json = r#"{
            "tools": {"listChanged": true},
            "prompts": {"listChanged": false},
            "resources": {"subscribe": true, "listChanged": true}
        }"#;
        let caps: ServerCapabilities = serde_json::from_str(json).unwrap();
        assert!(caps.tools.is_some());
        assert!(caps.tools.as_ref().unwrap().list_changed);
        assert!(caps.resources.is_some());
        assert!(caps.resources.as_ref().unwrap().subscribe);
    }
}
