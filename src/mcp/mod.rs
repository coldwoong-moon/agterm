use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub mod client;
pub mod context;
pub mod error;
pub mod offline;
pub mod response;
pub mod server_config;
pub mod transport;

pub use client::{
    McpClient, McpClientError, McpRequest, McpResponse as ClientResponse,
    ServerCapabilities, ToolCall, ToolContent, ToolInfo, ToolResult,
    PromptInfo, PromptArgument, ResourceInfo,
};
pub use context::{
    ContextEnvironment, GitInfo, ProcessInfo, TerminalContext, TerminalDimensions,
};
pub use error::{McpError, McpResult};
pub use offline::{CachedResponse, OfflineHandler};
pub use response::{McpResponse, ToolCallResult};
pub use server_config::{
    McpConfig, McpConfigError, McpSettings,
    ServerConfig, ServerProfile, ServerType,
    TransportConfig, RetryConfig,
};
pub use transport::TransportType;

/// Unique identifier for an MCP server
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct McpServerId(Uuid);

impl McpServerId {
    /// Create a new unique server ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a server ID from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    /// Get the string representation
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl Default for McpServerId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for McpServerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Connection status for an MCP server
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    /// Not connected
    Disconnected,
    /// Currently connecting
    Connecting,
    /// Successfully connected
    Connected,
    /// Connection failed
    Failed,
    /// Connection was lost
    Reconnecting,
}

impl ConnectionStatus {
    /// Check if the connection is active
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Connected)
    }

    /// Check if the connection is in progress
    pub fn is_connecting(&self) -> bool {
        matches!(self, Self::Connecting | Self::Reconnecting)
    }

    /// Check if the connection is inactive
    pub fn is_inactive(&self) -> bool {
        matches!(self, Self::Disconnected | Self::Failed)
    }
}

impl std::fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Connecting => write!(f, "Connecting"),
            Self::Connected => write!(f, "Connected"),
            Self::Failed => write!(f, "Failed"),
            Self::Reconnecting => write!(f, "Reconnecting"),
        }
    }
}

/// Configuration for an MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Server ID
    pub id: McpServerId,
    /// Server name
    pub name: String,
    /// Transport configuration
    pub transport: TransportType,
    /// Whether to auto-connect on startup
    #[serde(default)]
    pub auto_connect: bool,
    /// Connection timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

fn default_timeout() -> u64 {
    30
}

impl McpServerConfig {
    /// Create a new server configuration
    pub fn new(name: impl Into<String>, transport: TransportType) -> Self {
        Self {
            id: McpServerId::new(),
            name: name.into(),
            transport,
            auto_connect: false,
            timeout_secs: default_timeout(),
            metadata: HashMap::new(),
        }
    }

    /// Set auto-connect flag
    pub fn with_auto_connect(mut self, auto_connect: bool) -> Self {
        self.auto_connect = auto_connect;
        self
    }

    /// Set connection timeout
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// MCP Manager for handling multiple server connections
#[derive(Debug)]
pub struct McpManager {
    /// Active server connections
    connections: HashMap<McpServerId, ConnectionStatus>,
    /// Server configurations
    configs: HashMap<McpServerId, McpServerConfig>,
}

impl McpManager {
    /// Create a new MCP manager
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            configs: HashMap::new(),
        }
    }

    /// Add a server configuration
    pub fn add_server(&mut self, config: McpServerConfig) -> McpServerId {
        let id = config.id;
        self.configs.insert(id, config);
        self.connections.insert(id, ConnectionStatus::Disconnected);
        id
    }

    /// Remove a server
    pub fn remove_server(&mut self, id: McpServerId) -> Option<McpServerConfig> {
        self.connections.remove(&id);
        self.configs.remove(&id)
    }

    /// Get server configuration
    pub fn get_config(&self, id: McpServerId) -> Option<&McpServerConfig> {
        self.configs.get(&id)
    }

    /// Get server connection status
    pub fn get_status(&self, id: McpServerId) -> Option<ConnectionStatus> {
        self.connections.get(&id).copied()
    }

    /// Update server connection status
    pub fn update_status(&mut self, id: McpServerId, status: ConnectionStatus) {
        if let Some(current_status) = self.connections.get_mut(&id) {
            *current_status = status;
        }
    }

    /// Get all server IDs
    pub fn server_ids(&self) -> Vec<McpServerId> {
        self.configs.keys().copied().collect()
    }

    /// Get all connected servers
    pub fn connected_servers(&self) -> Vec<McpServerId> {
        self.connections
            .iter()
            .filter(|(_, status)| status.is_active())
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get number of servers
    pub fn server_count(&self) -> usize {
        self.configs.len()
    }

    /// Get number of connected servers
    pub fn connected_count(&self) -> usize {
        self.connected_servers().len()
    }
}

impl Default for McpManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Message types for MCP manager communication
#[derive(Debug, Clone)]
pub enum McpManagerMessage {
    /// Connect to a server
    Connect(McpServerId),
    /// Disconnect from a server
    Disconnect(McpServerId),
    /// Add a new server
    AddServer(McpServerConfig),
    /// Remove a server
    RemoveServer(McpServerId),
    /// Server status changed
    StatusChanged(McpServerId, ConnectionStatus),
    /// Request server list
    ListServers,
    /// Server list response
    ServerList(Vec<(McpServerId, String, ConnectionStatus)>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_id_creation() {
        let id1 = McpServerId::new();
        let id2 = McpServerId::new();
        assert_ne!(id1, id2);

        let uuid = Uuid::new_v4();
        let id3 = McpServerId::from_uuid(uuid);
        assert_eq!(id3.as_uuid(), uuid);
    }

    #[test]
    fn test_server_id_display() {
        let id = McpServerId::new();
        let display = format!("{}", id);
        let as_str = id.as_str();
        assert_eq!(display, as_str);
    }

    #[test]
    fn test_connection_status() {
        assert!(ConnectionStatus::Connected.is_active());
        assert!(!ConnectionStatus::Disconnected.is_active());

        assert!(ConnectionStatus::Connecting.is_connecting());
        assert!(ConnectionStatus::Reconnecting.is_connecting());
        assert!(!ConnectionStatus::Connected.is_connecting());

        assert!(ConnectionStatus::Disconnected.is_inactive());
        assert!(ConnectionStatus::Failed.is_inactive());
        assert!(!ConnectionStatus::Connected.is_inactive());
    }

    #[test]
    fn test_connection_status_display() {
        assert_eq!(ConnectionStatus::Connected.to_string(), "Connected");
        assert_eq!(ConnectionStatus::Disconnected.to_string(), "Disconnected");
        assert_eq!(ConnectionStatus::Connecting.to_string(), "Connecting");
        assert_eq!(ConnectionStatus::Failed.to_string(), "Failed");
        assert_eq!(ConnectionStatus::Reconnecting.to_string(), "Reconnecting");
    }

    #[test]
    fn test_server_config() {
        let transport = TransportType::stdio("python");
        let config = McpServerConfig::new("Test Server", transport.clone())
            .with_auto_connect(true)
            .with_timeout(60)
            .with_metadata("key".to_string(), "value".to_string());

        assert_eq!(config.name, "Test Server");
        assert_eq!(config.transport, transport);
        assert!(config.auto_connect);
        assert_eq!(config.timeout_secs, 60);
        assert_eq!(config.metadata.get("key").unwrap(), "value");
    }

    #[test]
    fn test_mcp_manager() {
        let mut manager = McpManager::new();
        assert_eq!(manager.server_count(), 0);

        let transport = TransportType::stdio("python");
        let config = McpServerConfig::new("Test Server", transport);
        let id = manager.add_server(config.clone());

        assert_eq!(manager.server_count(), 1);
        assert_eq!(manager.connected_count(), 0);

        let retrieved_config = manager.get_config(id).unwrap();
        assert_eq!(retrieved_config.name, "Test Server");

        let status = manager.get_status(id).unwrap();
        assert_eq!(status, ConnectionStatus::Disconnected);

        manager.update_status(id, ConnectionStatus::Connected);
        let new_status = manager.get_status(id).unwrap();
        assert_eq!(new_status, ConnectionStatus::Connected);
        assert_eq!(manager.connected_count(), 1);

        let connected = manager.connected_servers();
        assert_eq!(connected.len(), 1);
        assert_eq!(connected[0], id);

        let removed = manager.remove_server(id);
        assert!(removed.is_some());
        assert_eq!(manager.server_count(), 0);
    }

    #[test]
    fn test_manager_default() {
        let manager = McpManager::default();
        assert_eq!(manager.server_count(), 0);
    }

    #[test]
    fn test_server_ids() {
        let mut manager = McpManager::new();

        let config1 = McpServerConfig::new("Server 1", TransportType::stdio("python"));
        let config2 = McpServerConfig::new("Server 2", TransportType::http("http://localhost:8080"));

        let id1 = manager.add_server(config1);
        let id2 = manager.add_server(config2);

        let ids = manager.server_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[test]
    fn test_mcp_manager_message_clone() {
        let msg = McpManagerMessage::Connect(McpServerId::new());
        let cloned = msg.clone();
        // Just verify it compiles and can be cloned
        drop(cloned);
    }
}
