//! MCP Client
//!
//! MCP client wrapper for connecting to MCP servers.

use crate::infrastructure::mcp::server_config::{McpServerConfig, McpTransport};
use rmcp::{
    model::{
        CallToolRequestParam, CallToolResult, ClientInfo, ListToolsResult,
        PaginatedRequestParam, ServerInfo, Tool,
    },
    service::{Peer, RunningService},
    transport::TokioChildProcess,
    RoleClient, ServiceExt,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::process::Command;
use tokio::sync::RwLock;

/// MCP Client errors
#[derive(Debug, Error)]
pub enum McpClientError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Not connected")]
    NotConnected,

    #[error("Tool call failed: {0}")]
    ToolCallFailed(String),

    #[error("Transport error: {0}")]
    TransportError(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Timeout")]
    Timeout,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// MCP Client result type
pub type McpClientResult<T> = Result<T, McpClientError>;

/// Connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// Not connected
    Disconnected,
    /// Currently connecting
    Connecting,
    /// Connected and ready
    Connected,
    /// Connection failed
    Failed,
}

/// MCP Client wrapper
///
/// Note: Currently only stdio transport is supported. SSE transport
/// will be added in a future version.
pub struct McpClient {
    /// Server configuration
    config: McpServerConfig,
    /// Connection status
    status: ConnectionStatus,
    /// Running service (stdio only for now)
    service: Option<RunningService<RoleClient, ClientInfo>>,
    /// Cached server info
    server_info: Option<ServerInfo>,
    /// Cached tools list
    cached_tools: Vec<Tool>,
}

impl McpClient {
    /// Create a new MCP client (not connected)
    pub fn new(config: McpServerConfig) -> Self {
        Self {
            config,
            status: ConnectionStatus::Disconnected,
            service: None,
            server_info: None,
            cached_tools: Vec::new(),
        }
    }

    /// Get the server configuration
    pub fn config(&self) -> &McpServerConfig {
        &self.config
    }

    /// Get the connection status
    pub fn status(&self) -> ConnectionStatus {
        self.status
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.status == ConnectionStatus::Connected && self.service.is_some()
    }

    /// Get server info (if connected)
    pub fn server_info(&self) -> Option<&ServerInfo> {
        self.server_info.as_ref()
    }

    /// Get cached tools
    pub fn tools(&self) -> &[Tool] {
        &self.cached_tools
    }

    /// Get the peer for making requests
    fn peer(&self) -> McpClientResult<&Peer<RoleClient>> {
        self.service
            .as_ref()
            .map(|s| s.peer())
            .ok_or(McpClientError::NotConnected)
    }

    /// Connect to the server
    pub async fn connect(&mut self) -> McpClientResult<()> {
        if self.is_connected() {
            return Ok(());
        }

        self.status = ConnectionStatus::Connecting;

        // Clone transport config to avoid borrow issues
        let transport = self.config.transport.clone();

        let result = match transport {
            McpTransport::Stdio {
                command,
                args,
                working_dir,
                env,
            } => {
                self.connect_stdio(&command, &args, working_dir.as_ref(), &env)
                    .await
            }
            McpTransport::Sse { url, .. } => {
                // SSE transport not yet fully supported in this version
                Err(McpClientError::InvalidConfig(format!(
                    "SSE transport not yet supported. URL: {}",
                    url
                )))
            }
        };

        match result {
            Ok(()) => {
                self.status = ConnectionStatus::Connected;
                // Refresh tools cache
                let _ = self.refresh_tools().await;
                Ok(())
            }
            Err(e) => {
                self.status = ConnectionStatus::Failed;
                Err(e)
            }
        }
    }

    /// Connect via stdio transport
    async fn connect_stdio(
        &mut self,
        command: &str,
        args: &[String],
        working_dir: Option<&PathBuf>,
        env: &HashMap<String, String>,
    ) -> McpClientResult<()> {
        let mut cmd = Command::new(command);
        cmd.args(args);

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        for (key, value) in env {
            cmd.env(key, value);
        }

        // TokioChildProcess::new takes ownership of Command
        let transport = TokioChildProcess::new(cmd)
            .map_err(|e| McpClientError::TransportError(e.to_string()))?;

        let client_info = ClientInfo::default();

        let service = client_info
            .serve(transport)
            .await
            .map_err(|e| McpClientError::ConnectionFailed(e.to_string()))?;

        // Get server info from peer
        if let Some(info) = service.peer().peer_info() {
            self.server_info = Some(info.clone());
        }

        self.service = Some(service);

        Ok(())
    }

    /// Disconnect from the server
    pub async fn disconnect(&mut self) -> McpClientResult<()> {
        if let Some(service) = self.service.take() {
            service
                .cancel()
                .await
                .map_err(|e| McpClientError::ServerError(e.to_string()))?;
        }

        self.status = ConnectionStatus::Disconnected;
        self.server_info = None;
        self.cached_tools.clear();

        Ok(())
    }

    /// List available tools
    pub async fn list_tools(&self) -> McpClientResult<ListToolsResult> {
        let peer = self.peer()?;

        peer.list_tools(Some(PaginatedRequestParam { cursor: None }))
            .await
            .map_err(|e| McpClientError::ServerError(e.to_string()))
    }

    /// List all tools (handling pagination)
    pub async fn list_all_tools(&self) -> McpClientResult<Vec<Tool>> {
        let peer = self.peer()?;

        peer.list_all_tools()
            .await
            .map_err(|e| McpClientError::ServerError(e.to_string()))
    }

    /// Refresh the cached tools list
    pub async fn refresh_tools(&mut self) -> McpClientResult<()> {
        let tools = self.list_all_tools().await?;
        self.cached_tools = tools;
        Ok(())
    }

    /// Call a tool
    pub async fn call_tool(
        &self,
        name: impl Into<String>,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> McpClientResult<CallToolResult> {
        let peer = self.peer()?;

        let params = CallToolRequestParam {
            name: name.into().into(),
            arguments,
            task: None, // Not using task scheduling
        };

        peer.call_tool(params)
            .await
            .map_err(|e| McpClientError::ToolCallFailed(e.to_string()))
    }

    /// Call a tool with JSON arguments
    pub async fn call_tool_json(
        &self,
        name: impl Into<String>,
        arguments: serde_json::Value,
    ) -> McpClientResult<CallToolResult> {
        let args = arguments.as_object().cloned();
        self.call_tool(name, args).await
    }
}

/// Thread-safe MCP client handle
pub type McpClientHandle = Arc<RwLock<McpClient>>;

/// Create a new client handle
pub fn create_client_handle(config: McpServerConfig) -> McpClientHandle {
    Arc::new(RwLock::new(McpClient::new(config)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let config = McpServerConfig::stdio("test", "echo");
        let client = McpClient::new(config);

        assert_eq!(client.status(), ConnectionStatus::Disconnected);
        assert!(!client.is_connected());
        assert!(client.server_info().is_none());
    }
}
