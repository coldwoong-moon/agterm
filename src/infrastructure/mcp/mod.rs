//! MCP (Model Context Protocol) Integration
//!
//! This module provides MCP client functionality for connecting to
//! MCP servers and invoking tools.
//!
//! # Architecture
//!
//! - `client`: MCP client wrapper for single server connections
//! - `registry`: Multi-server registry for managing connections
//! - `server_config`: Configuration types for MCP servers
//! - `tools`: Helper types for tool invocation
//!
//! # Example
//!
//! ```ignore
//! use agterm::infrastructure::mcp::{McpRegistry, McpServerConfig};
//!
//! // Create a registry
//! let mut registry = McpRegistry::new();
//!
//! // Register a stdio server
//! registry.register(
//!     McpServerConfig::stdio("git", "uvx")
//!         .with_args(vec!["mcp-server-git".to_string()])
//! );
//!
//! // Connect
//! registry.connect("git").await?;
//!
//! // Call a tool
//! let result = registry.call_tool("git", "git_status", None).await?;
//! ```

pub mod client;
pub mod registry;
pub mod server_config;
pub mod tools;

// Re-export main types
pub use client::{
    create_client_handle, ConnectionStatus, McpClient, McpClientError, McpClientHandle,
    McpClientResult,
};
pub use registry::{
    create_registry_handle, create_registry_handle_with_events, McpRegistry, McpRegistryHandle,
    RegistryEvent,
};
pub use server_config::{McpServerConfig, McpTransport};
pub use tools::{
    format_tool_result, parse_tool_arguments, ContentItem, ToolCallRequest, ToolCallResponse,
    ToolInfo,
};
