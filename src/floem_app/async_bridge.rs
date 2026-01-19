//! Async bridge between Floem UI and Tokio runtime
//!
//! This module provides a bridge between Floem's synchronous UI and Tokio's
//! asynchronous runtime. It enables the UI to send async commands and receive
//! results without blocking the UI thread.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
        };

        (bridge, worker)
    }

    /// Send an async command
    pub fn send_command(&self, command: AsyncCommand) -> Result<(), String> {
        self.command_tx
            .try_send(command)
            .map_err(|e| format!("Failed to send command: {}", e))
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
}

impl Default for AsyncBridge {
    fn default() -> Self {
        Self::new().0
    }
}

/// Worker that processes async commands
pub struct BridgeWorker {
    /// Receiver for async commands
    command_rx: tokio::sync::mpsc::Receiver<AsyncCommand>,

    /// Sender for async results
    result_tx: std::sync::mpsc::Sender<AsyncResult>,
}

impl BridgeWorker {
    /// Run the worker (should be called in a Tokio runtime)
    pub async fn run(mut self) {
        tracing::info!("AsyncBridge worker started");

        while let Some(command) = self.command_rx.recv().await {
            tracing::debug!(?command, "Processing async command");

            let result = self.process_command(command).await;

            if let Err(e) = self.result_tx.send(result) {
                tracing::error!("Failed to send result back to UI: {}", e);
                break;
            }
        }

        tracing::info!("AsyncBridge worker stopped");
    }

    /// Process a single command
    async fn process_command(&self, command: AsyncCommand) -> AsyncResult {
        match command {
            AsyncCommand::McpConnect(server_name) => {
                // TODO: Implement MCP connection logic
                tracing::info!("Connecting to MCP server: {}", server_name);
                AsyncResult::McpConnected { server_name }
            }

            AsyncCommand::McpDisconnect => {
                // TODO: Implement MCP disconnection logic
                tracing::info!("Disconnecting from MCP server");
                AsyncResult::McpDisconnected
            }

            AsyncCommand::McpListTools => {
                // TODO: Implement tool listing logic
                tracing::info!("Listing MCP tools");
                AsyncResult::McpTools(vec![])
            }

            AsyncCommand::McpCallTool(name, params) => {
                // TODO: Implement tool calling logic
                tracing::info!("Calling MCP tool: {} with params: {:?}", name, params);
                AsyncResult::McpToolResult(serde_json::Value::Null)
            }

            AsyncCommand::ExecuteCommand {
                command,
                terminal_id,
                risk_level,
            } => {
                // TODO: Implement command execution logic with risk assessment
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
                        reason: format!(
                            "Command blocked due to {:?} risk level",
                            risk_level
                        ),
                    },
                }
            }
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

    #[tokio::test]
    async fn test_worker_processes_commands() {
        let (bridge, worker) = AsyncBridge::new();

        // Spawn worker in background
        tokio::spawn(async move {
            worker.run().await;
        });

        // Send a command
        bridge
            .send_command(AsyncCommand::McpConnect("test-server".to_string()))
            .unwrap();

        // Give the worker time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Check for result
        let result = bridge.try_recv_result();
        assert!(result.is_some());

        if let Some(AsyncResult::McpConnected { server_name }) = result {
            assert_eq!(server_name, "test-server");
        } else {
            panic!("Expected McpConnected result");
        }
    }
}
