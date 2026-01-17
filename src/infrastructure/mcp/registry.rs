//! MCP Server Registry
//!
//! Manages multiple MCP server connections.

use crate::infrastructure::mcp::client::{
    create_client_handle, ConnectionStatus, McpClient, McpClientError, McpClientHandle,
    McpClientResult,
};
use crate::infrastructure::mcp::server_config::McpServerConfig;
use rmcp::model::Tool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Event emitted by the registry
#[derive(Debug, Clone)]
pub enum RegistryEvent {
    /// Server added to registry
    ServerAdded(String),
    /// Server removed from registry
    ServerRemoved(String),
    /// Server connection status changed
    StatusChanged {
        server_name: String,
        old_status: ConnectionStatus,
        new_status: ConnectionStatus,
    },
    /// Tools list updated
    ToolsUpdated(String),
}

/// MCP Server Registry
///
/// Manages multiple MCP server connections and provides
/// unified access to all available tools.
pub struct McpRegistry {
    /// Registered servers (name -> client handle)
    servers: HashMap<String, McpClientHandle>,
    /// Event sender (optional)
    event_tx: Option<tokio::sync::mpsc::UnboundedSender<RegistryEvent>>,
}

impl McpRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
            event_tx: None,
        }
    }

    /// Create a new registry with event channel
    pub fn with_events() -> (Self, tokio::sync::mpsc::UnboundedReceiver<RegistryEvent>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let registry = Self {
            servers: HashMap::new(),
            event_tx: Some(tx),
        };
        (registry, rx)
    }

    /// Send an event (if event channel is configured)
    fn send_event(&self, event: RegistryEvent) {
        if let Some(ref tx) = self.event_tx {
            let _ = tx.send(event);
        }
    }

    /// Register a server configuration
    pub fn register(&mut self, config: McpServerConfig) -> McpClientHandle {
        let name = config.name.clone();
        let handle = create_client_handle(config);
        self.servers.insert(name.clone(), handle.clone());
        self.send_event(RegistryEvent::ServerAdded(name));
        handle
    }

    /// Unregister a server
    pub fn unregister(&mut self, name: &str) -> Option<McpClientHandle> {
        let handle = self.servers.remove(name);
        if handle.is_some() {
            self.send_event(RegistryEvent::ServerRemoved(name.to_string()));
        }
        handle
    }

    /// Get a server handle by name
    pub fn get(&self, name: &str) -> Option<&McpClientHandle> {
        self.servers.get(name)
    }

    /// Get all server names
    pub fn server_names(&self) -> Vec<String> {
        self.servers.keys().cloned().collect()
    }

    /// Get number of registered servers
    pub fn server_count(&self) -> usize {
        self.servers.len()
    }

    /// Check if a server is registered
    pub fn contains(&self, name: &str) -> bool {
        self.servers.contains_key(name)
    }

    /// Connect to a specific server
    pub async fn connect(&self, name: &str) -> McpClientResult<()> {
        let handle = self
            .servers
            .get(name)
            .ok_or_else(|| McpClientError::InvalidConfig(format!("Server not found: {}", name)))?;

        let old_status = handle.read().await.status();
        handle.write().await.connect().await?;
        let new_status = handle.read().await.status();

        self.send_event(RegistryEvent::StatusChanged {
            server_name: name.to_string(),
            old_status,
            new_status,
        });

        Ok(())
    }

    /// Disconnect from a specific server
    pub async fn disconnect(&self, name: &str) -> McpClientResult<()> {
        let handle = self
            .servers
            .get(name)
            .ok_or_else(|| McpClientError::InvalidConfig(format!("Server not found: {}", name)))?;

        let old_status = handle.read().await.status();
        handle.write().await.disconnect().await?;
        let new_status = handle.read().await.status();

        self.send_event(RegistryEvent::StatusChanged {
            server_name: name.to_string(),
            old_status,
            new_status,
        });

        Ok(())
    }

    /// Connect to all servers that have auto_connect enabled
    pub async fn connect_auto(&self) -> Vec<(String, McpClientResult<()>)> {
        let mut results = Vec::new();

        for (name, handle) in &self.servers {
            let should_connect = handle.read().await.config().auto_connect;
            if should_connect {
                let result = self.connect(name).await;
                results.push((name.clone(), result));
            }
        }

        results
    }

    /// Connect to all servers
    pub async fn connect_all(&self) -> Vec<(String, McpClientResult<()>)> {
        let mut results = Vec::new();

        for name in self.servers.keys() {
            let result = self.connect(name).await;
            results.push((name.clone(), result));
        }

        results
    }

    /// Disconnect from all servers
    pub async fn disconnect_all(&self) -> Vec<(String, McpClientResult<()>)> {
        let mut results = Vec::new();

        for name in self.servers.keys() {
            let result = self.disconnect(name).await;
            results.push((name.clone(), result));
        }

        results
    }

    /// Get connection status for all servers
    pub async fn get_all_status(&self) -> HashMap<String, ConnectionStatus> {
        let mut status_map = HashMap::new();

        for (name, handle) in &self.servers {
            let status = handle.read().await.status();
            status_map.insert(name.clone(), status);
        }

        status_map
    }

    /// Get all available tools from all connected servers
    pub async fn get_all_tools(&self) -> HashMap<String, Vec<Tool>> {
        let mut tools_map = HashMap::new();

        for (name, handle) in &self.servers {
            let client = handle.read().await;
            if client.is_connected() {
                tools_map.insert(name.clone(), client.tools().to_vec());
            }
        }

        tools_map
    }

    /// Get a flat list of all tools with server prefixes
    pub async fn get_all_tools_flat(&self) -> Vec<(String, Tool)> {
        let mut tools = Vec::new();

        for (name, handle) in &self.servers {
            let client = handle.read().await;
            if client.is_connected() {
                for tool in client.tools() {
                    tools.push((name.clone(), tool.clone()));
                }
            }
        }

        tools
    }

    /// Find a tool by name across all connected servers
    pub async fn find_tool(&self, tool_name: &str) -> Option<(String, Tool)> {
        for (server_name, handle) in &self.servers {
            let client = handle.read().await;
            if client.is_connected() {
                if let Some(tool) = client.tools().iter().find(|t| t.name == tool_name) {
                    return Some((server_name.clone(), tool.clone()));
                }
            }
        }
        None
    }

    /// Call a tool on a specific server
    pub async fn call_tool(
        &self,
        server_name: &str,
        tool_name: impl Into<String>,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> McpClientResult<rmcp::model::CallToolResult> {
        let handle = self.servers.get(server_name).ok_or_else(|| {
            McpClientError::InvalidConfig(format!("Server not found: {}", server_name))
        })?;

        let client = handle.read().await;
        client.call_tool(tool_name, arguments).await
    }

    /// Refresh tools for all connected servers
    pub async fn refresh_all_tools(&self) -> Vec<(String, McpClientResult<()>)> {
        let mut results = Vec::new();

        for (name, handle) in &self.servers {
            let is_connected = handle.read().await.is_connected();
            if is_connected {
                let result = handle.write().await.refresh_tools().await;
                if result.is_ok() {
                    self.send_event(RegistryEvent::ToolsUpdated(name.clone()));
                }
                results.push((name.clone(), result));
            }
        }

        results
    }
}

impl Default for McpRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe registry handle
pub type McpRegistryHandle = Arc<RwLock<McpRegistry>>;

/// Create a new registry handle
pub fn create_registry_handle() -> McpRegistryHandle {
    Arc::new(RwLock::new(McpRegistry::new()))
}

/// Create a new registry handle with events
pub fn create_registry_handle_with_events(
) -> (McpRegistryHandle, tokio::sync::mpsc::UnboundedReceiver<RegistryEvent>) {
    let (registry, rx) = McpRegistry::with_events();
    (Arc::new(RwLock::new(registry)), rx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = McpRegistry::new();
        assert_eq!(registry.server_count(), 0);
    }

    #[test]
    fn test_register_server() {
        let mut registry = McpRegistry::new();
        let config = McpServerConfig::stdio("test-server", "echo");

        registry.register(config);

        assert_eq!(registry.server_count(), 1);
        assert!(registry.contains("test-server"));
    }

    #[test]
    fn test_unregister_server() {
        let mut registry = McpRegistry::new();
        let config = McpServerConfig::stdio("test-server", "echo");

        registry.register(config);
        let removed = registry.unregister("test-server");

        assert!(removed.is_some());
        assert_eq!(registry.server_count(), 0);
        assert!(!registry.contains("test-server"));
    }

    #[test]
    fn test_server_names() {
        let mut registry = McpRegistry::new();

        registry.register(McpServerConfig::stdio("server-a", "echo"));
        registry.register(McpServerConfig::stdio("server-b", "echo"));
        registry.register(McpServerConfig::sse("server-c", "http://localhost:8000/sse"));

        let names = registry.server_names();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"server-a".to_string()));
        assert!(names.contains(&"server-b".to_string()));
        assert!(names.contains(&"server-c".to_string()));
    }

    #[tokio::test]
    async fn test_registry_with_events() {
        let (mut registry, mut rx) = McpRegistry::with_events();

        registry.register(McpServerConfig::stdio("test", "echo"));

        // Should receive ServerAdded event
        let event = rx.try_recv().unwrap();
        match event {
            RegistryEvent::ServerAdded(name) => assert_eq!(name, "test"),
            _ => panic!("Expected ServerAdded event"),
        }
    }
}
