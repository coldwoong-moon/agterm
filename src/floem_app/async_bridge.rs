//! Async bridge between Floem UI and Tokio runtime
//!
//! This module provides a bridge between Floem's synchronous UI and Tokio's
//! asynchronous runtime. It enables the UI to send async commands and receive
//! results without blocking the UI thread.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::floem_app::mcp_client::McpClient;

/// Commands that can be sent from the UI to the async worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AsyncCommand {
    /// Connect to an MCP server
    McpConnect(String),

    /// Disconnect from the current MCP server
    McpDisconnect,

    /// List available tools from the MCP server
    McpListTools,

    /// Call a tool on the MCP server
    McpCallTool(String, serde_json::Value),

    /// Execute a command with risk assessment
    ExecuteCommand {
        command: String,
        terminal_id: Uuid,
        risk_level: RiskLevel,
    },
}

/// Results from async operations sent back to the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AsyncResult {
    /// Successfully connected to MCP server
    McpConnected { server_name: String },

    /// Disconnected from MCP server
    McpDisconnected,

    /// List of available tools
    McpTools(Vec<ToolInfo>),

    /// Result from calling a tool
    McpToolResult(serde_json::Value),

    /// Command was approved for execution
    CommandApproved {
        command: String,
        terminal_id: Uuid,
    },

    /// Command was blocked
    CommandBlocked {
        command: String,
        reason: String,
    },

    /// An error occurred
    Error(String),
}

/// Risk level for command execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk - safe commands (ls, cat, etc.)
    Low,

    /// Medium risk - commands that modify files
    Medium,

    /// High risk - commands that affect system state
    High,

    /// Critical risk - dangerous commands (rm -rf, etc.)
    Critical,
}

/// Information about an MCP tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    /// Tool name
    pub name: String,

    /// Tool description (optional)
    pub description: Option<String>,
}

/// Bridge for sending async commands from the UI
pub struct AsyncBridge {
    /// Sender for async commands
    command_tx: tokio::sync::mpsc::Sender<AsyncCommand>,

    /// Receiver for async results
    result_rx: std::sync::mpsc::Receiver<AsyncResult>,
}

impl AsyncBridge {
    /// Create a new async bridge
    ///
    /// Returns the bridge (for the UI) and a worker (for the async runtime)
    pub fn new() -> (Self, BridgeWorker) {
        let (command_tx, command_rx) = tokio::sync::mpsc::channel(32);
        let (result_tx, result_rx) = std::sync::mpsc::channel();

        let bridge = AsyncBridge {
            command_tx,
            result_rx,
        };

        let worker = BridgeWorker {
            command_rx,
            result_tx,
            mcp_client: None,
        };

        (bridge, worker)
    }

    /// Send an async command
    pub fn send_command(&self, command: AsyncCommand) -> Result<(), String> {
        self.command_tx
            .try_send(command)
            .map_err(|e| format!("Failed to send command: {e}"))
    }

    /// Try to receive a result (non-blocking)
    pub fn try_recv_result(&self) -> Option<AsyncResult> {
        self.result_rx.try_recv().ok()
    }

    /// Receive all pending results
    pub fn recv_all_results(&self) -> Vec<AsyncResult> {
        let mut results = Vec::new();
        while let Ok(result) = self.result_rx.try_recv() {
            results.push(result);
        }
        results
    }

    /// Get a clone of the command sender
    pub fn command_tx(&self) -> &tokio::sync::mpsc::Sender<AsyncCommand> {
        &self.command_tx
    }

    /// Take ownership of the result receiver (consumes the bridge)
    pub fn into_result_rx(self) -> std::sync::mpsc::Receiver<AsyncResult> {
        self.result_rx
    }
}

impl Default for AsyncBridge {
    fn default() -> Self {
        Self::new().0
    }
}

/// Agent configuration for MCP connection
struct AgentConfig {
    command: &'static str,
    args: &'static [&'static str],
}

/// Get agent configuration by name
fn get_agent_config(agent_name: &str) -> Option<AgentConfig> {
    match agent_name {
        "claude_code" | "ClaudeCode" => Some(AgentConfig {
            command: "claude",
            args: &["mcp", "serve"],
        }),
        "gemini_cli" | "GeminiCli" => Some(AgentConfig {
            command: "gemini",
            args: &["mcp"],
        }),
        "openai_codex" | "OpenAICodex" => Some(AgentConfig {
            command: "openai",
            args: &["mcp"],
        }),
        "qwen_code" | "QwenCode" => Some(AgentConfig {
            command: "qwen",
            args: &["mcp"],
        }),
        // Allow custom server commands (format: "cmd:arg1:arg2")
        _ if agent_name.contains(':') => {
            // Custom format not yet supported, return None
            None
        }
        _ => None,
    }
}

/// Worker that processes async commands
pub struct BridgeWorker {
    /// Receiver for async commands
    command_rx: tokio::sync::mpsc::Receiver<AsyncCommand>,

    /// Sender for async results
    result_tx: std::sync::mpsc::Sender<AsyncResult>,

    /// MCP client instance
    mcp_client: Option<McpClient>,
}

impl BridgeWorker {
    /// Run the worker (should be called in a Tokio runtime)
    pub async fn run(mut self) {
        tracing::info!("AsyncBridge worker started");

        while let Some(command) = self.command_rx.recv().await {
            tracing::debug!(?command, "Processing async command");

            let result = self.process_command(command).await;

            if let Err(e) = self.result_tx.send(result) {
                tracing::error!("Failed to send result back to UI: {e}");
                break;
            }
        }

        // Clean up MCP client on shutdown
        if let Some(mut client) = self.mcp_client.take() {
            let _ = client.disconnect().await;
        }

        tracing::info!("AsyncBridge worker stopped");
    }

    /// Process a single command
    async fn process_command(&mut self, command: AsyncCommand) -> AsyncResult {
        match command {
            AsyncCommand::McpConnect(agent_name) => {
                self.handle_mcp_connect(&agent_name).await
            }

            AsyncCommand::McpDisconnect => {
                self.handle_mcp_disconnect().await
            }

            AsyncCommand::McpListTools => {
                self.handle_mcp_list_tools().await
            }

            AsyncCommand::McpCallTool(name, params) => {
                self.handle_mcp_call_tool(&name, params).await
            }

            AsyncCommand::ExecuteCommand {
                command,
                terminal_id,
                risk_level,
            } => {
                self.handle_execute_command(command, terminal_id, risk_level).await
            }
        }
    }

    /// Handle MCP connect command
    async fn handle_mcp_connect(&mut self, agent_name: &str) -> AsyncResult {
        tracing::info!("Connecting to MCP server: {}", agent_name);

        // Disconnect existing client if any
        if let Some(mut client) = self.mcp_client.take() {
            let _ = client.disconnect().await;
        }

        // Get agent configuration
        let config = match get_agent_config(agent_name) {
            Some(cfg) => cfg,
            None => {
                return AsyncResult::Error(format!(
                    "Unknown agent: {}. Supported agents: claude_code, gemini_cli, openai_codex, qwen_code",
                    agent_name
                ));
            }
        };

        // Try to connect
        match McpClient::connect_stdio(config.command, config.args).await {
            Ok(client) => {
                let server_name = client.server_name().to_string();
                self.mcp_client = Some(client);
                tracing::info!("Successfully connected to MCP server: {}", server_name);
                AsyncResult::McpConnected { server_name }
            }
            Err(e) => {
                tracing::error!("Failed to connect to MCP server: {}", e);
                AsyncResult::Error(format!("Connection failed: {e}"))
            }
        }
    }

    /// Handle MCP disconnect command
    async fn handle_mcp_disconnect(&mut self) -> AsyncResult {
        tracing::info!("Disconnecting from MCP server");

        if let Some(mut client) = self.mcp_client.take() {
            match client.disconnect().await {
                Ok(_) => {
                    tracing::info!("Successfully disconnected from MCP server");
                    AsyncResult::McpDisconnected
                }
                Err(e) => {
                    tracing::error!("Error during disconnect: {}", e);
                    // Still mark as disconnected even if there was an error
                    AsyncResult::McpDisconnected
                }
            }
        } else {
            AsyncResult::McpDisconnected
        }
    }

    /// Handle MCP list tools command
    async fn handle_mcp_list_tools(&mut self) -> AsyncResult {
        tracing::info!("Listing MCP tools");

        let client = match self.mcp_client.as_mut() {
            Some(c) => c,
            None => {
                return AsyncResult::Error("Not connected to MCP server".to_string());
            }
        };

        match client.list_tools().await {
            Ok(tools) => {
                let tool_infos: Vec<ToolInfo> = tools
                    .into_iter()
                    .map(|t| ToolInfo {
                        name: t.name,
                        description: t.description,
                    })
                    .collect();

                tracing::info!("Found {} tools", tool_infos.len());
                AsyncResult::McpTools(tool_infos)
            }
            Err(e) => {
                tracing::error!("Failed to list tools: {}", e);
                AsyncResult::Error(format!("Failed to list tools: {e}"))
            }
        }
    }

    /// Handle MCP call tool command
    async fn handle_mcp_call_tool(&mut self, name: &str, params: serde_json::Value) -> AsyncResult {
        tracing::info!("Calling MCP tool: {} with params: {:?}", name, params);

        let client = match self.mcp_client.as_mut() {
            Some(c) => c,
            None => {
                return AsyncResult::Error("Not connected to MCP server".to_string());
            }
        };

        match client.call_tool(name, params).await {
            Ok(result) => {
                // Convert result to JSON value
                let result_value = serde_json::json!({
                    "content": result.content.iter().map(|c| {
                        serde_json::json!({
                            "type": c.content_type,
                            "text": c.text
                        })
                    }).collect::<Vec<_>>(),
                    "is_error": result.is_error
                });

                tracing::info!("Tool call completed successfully");
                AsyncResult::McpToolResult(result_value)
            }
            Err(e) => {
                tracing::error!("Failed to call tool: {}", e);
                AsyncResult::Error(format!("Failed to call tool: {e}"))
            }
        }
    }

    /// Handle command execution with risk assessment
    async fn handle_execute_command(
        &self,
        command: String,
        terminal_id: Uuid,
        risk_level: RiskLevel,
    ) -> AsyncResult {
        tracing::info!(
            "Executing command: {} (risk: {:?}) on terminal: {}",
            command,
            risk_level,
            terminal_id
        );

        // For now, approve all low/medium risk commands
        match risk_level {
            RiskLevel::Low | RiskLevel::Medium => AsyncResult::CommandApproved {
                command,
                terminal_id,
            },
            RiskLevel::High | RiskLevel::Critical => AsyncResult::CommandBlocked {
                command,
                reason: format!("Command blocked due to {risk_level:?} risk level"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_bridge_creation() {
        let (bridge, _worker) = AsyncBridge::new();
        assert!(bridge.try_recv_result().is_none());
    }

    #[test]
    fn test_send_command() {
        let (bridge, _worker) = AsyncBridge::new();
        let result = bridge.send_command(AsyncCommand::McpDisconnect);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_agent_config() {
        assert!(get_agent_config("claude_code").is_some());
        assert!(get_agent_config("ClaudeCode").is_some());
        assert!(get_agent_config("gemini_cli").is_some());
        assert!(get_agent_config("unknown_agent").is_none());
    }

    #[tokio::test]
    async fn test_worker_processes_disconnect() {
        let (bridge, worker) = AsyncBridge::new();

        // Spawn worker in background
        tokio::spawn(async move {
            worker.run().await;
        });

        // Send disconnect command (should work even without connection)
        bridge
            .send_command(AsyncCommand::McpDisconnect)
            .unwrap();

        // Give the worker time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Check for result
        let result = bridge.try_recv_result();
        assert!(result.is_some());

        if let Some(AsyncResult::McpDisconnected) = result {
            // Expected
        } else {
            panic!("Expected McpDisconnected result");
        }
    }

    #[tokio::test]
    async fn test_worker_processes_list_tools_without_connection() {
        let (bridge, worker) = AsyncBridge::new();

        // Spawn worker in background
        tokio::spawn(async move {
            worker.run().await;
        });

        // Send list tools command without connection
        bridge
            .send_command(AsyncCommand::McpListTools)
            .unwrap();

        // Give the worker time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Check for result (should be an error)
        let result = bridge.try_recv_result();
        assert!(result.is_some());

        if let Some(AsyncResult::Error(msg)) = result {
            assert!(msg.contains("Not connected"));
        } else {
            panic!("Expected Error result");
        }
    }

    #[tokio::test]
    async fn test_worker_processes_connect_unknown_agent() {
        let (bridge, worker) = AsyncBridge::new();

        // Spawn worker in background
        tokio::spawn(async move {
            worker.run().await;
        });

        // Send connect command with unknown agent
        bridge
            .send_command(AsyncCommand::McpConnect("unknown_agent".to_string()))
            .unwrap();

        // Give the worker time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Check for result (should be an error)
        let result = bridge.try_recv_result();
        assert!(result.is_some());

        if let Some(AsyncResult::Error(msg)) = result {
            assert!(msg.contains("Unknown agent"));
        } else {
            panic!("Expected Error result");
        }
    }
}
