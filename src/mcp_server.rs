//! MCP Server for AgTerm
//!
//! This module implements an MCP (Model Context Protocol) server that allows
//! AI agents like Claude Code to control AgTerm terminals programmatically.
//!
//! ## Usage
//!
//! Run the MCP server:
//! ```bash
//! agterm --mcp-server
//! ```
//!
//! ## Available Tools
//!
//! - `create_tab`: Create a new terminal tab
//! - `split_pane`: Split the current pane
//! - `run_command`: Execute a command in a terminal
//! - `get_output`: Get output from a terminal
//! - `list_tabs`: List all tabs
//! - `list_panes`: List all panes in a tab

use crate::terminal::pty::{PtyId, PtyManager};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Strip ANSI escape codes from terminal output
fn strip_ansi_codes(input: &str) -> String {
    // Match ANSI escape sequences: ESC [ ... (letter or ~)
    let ansi_regex = Regex::new(r"\x1b\[[0-9;]*[a-zA-Z~]|\x1b\][^\x07]*\x07|\x1b[()][AB012]|\x1b[>=]").unwrap();
    let result = ansi_regex.replace_all(input, "");
    // Also remove other control characters except newline and tab
    result.chars()
        .filter(|c| *c == '\n' || *c == '\t' || *c == '\r' || !c.is_control())
        .collect()
}

/// MCP Server instance
pub struct McpServer {
    /// Channel to send commands to the main app
    command_tx: mpsc::Sender<McpCommand>,
    /// Channel to receive responses from the main app
    response_rx: std::sync::mpsc::Receiver<McpResponse>,
}

/// Commands sent from MCP server to the main app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpCommand {
    CreateTab { name: Option<String> },
    SplitPane { direction: String, tab_index: Option<usize> },
    RunCommand { command: String, tab_index: Option<usize>, pane_index: Option<usize> },
    GetOutput { tab_index: Option<usize>, pane_index: Option<usize>, lines: Option<usize> },
    ListTabs,
    ListPanes { tab_index: Option<usize> },
    FocusTab { index: usize },
    FocusPane { index: usize },
    ClosePane { tab_index: Option<usize>, pane_index: Option<usize> },
}

/// Responses from the main app to MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpResponse {
    Success { data: Value },
    Error { message: String },
}

/// JSON-RPC request structure
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

/// JSON-RPC response structure
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new() -> (Self, McpCommandReceiver) {
        let (command_tx, command_rx) = mpsc::channel(32);
        let (response_tx, response_rx) = std::sync::mpsc::channel();

        let server = McpServer {
            command_tx,
            response_rx,
        };

        let receiver = McpCommandReceiver {
            command_rx,
            response_tx,
        };

        (server, receiver)
    }

    /// Run the MCP server (blocking, reads from stdin)
    pub async fn run(&self) {
        let stdin = std::io::stdin();
        let reader = BufReader::new(stdin.lock());
        let mut stdout = std::io::stdout();

        tracing::info!("MCP server started, waiting for requests...");

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    tracing::error!("Error reading stdin: {}", e);
                    break;
                }
            };

            if line.is_empty() {
                continue;
            }

            let response = self.handle_request(&line).await;
            let response_json = serde_json::to_string(&response).unwrap_or_else(|e| {
                json!({
                    "jsonrpc": "2.0",
                    "error": {"code": -32603, "message": format!("Serialization error: {}", e)}
                }).to_string()
            });

            if let Err(e) = writeln!(stdout, "{}", response_json) {
                tracing::error!("Error writing to stdout: {}", e);
                break;
            }
            let _ = stdout.flush();
        }

        tracing::info!("MCP server stopped");
    }

    async fn handle_request(&self, line: &str) -> JsonRpcResponse {
        let request: JsonRpcRequest = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
            }
        };

        let result = self.handle_method(&request.method, request.params).await;

        match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(value),
                error: None,
            },
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(e),
            },
        }
    }

    async fn handle_method(&self, method: &str, params: Option<Value>) -> Result<Value, JsonRpcError> {
        match method {
            // MCP standard methods
            "initialize" => Ok(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "agterm",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),

            "tools/list" => Ok(json!({
                "tools": [
                    {
                        "name": "create_tab",
                        "description": "Create a new terminal tab",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "name": {"type": "string", "description": "Tab name (optional)"}
                            }
                        }
                    },
                    {
                        "name": "split_pane",
                        "description": "Split the current pane horizontally or vertically",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "direction": {"type": "string", "enum": ["horizontal", "vertical"]},
                                "tab_index": {"type": "integer", "description": "Tab index (optional, defaults to active)"}
                            },
                            "required": ["direction"]
                        }
                    },
                    {
                        "name": "run_command",
                        "description": "Execute a command in a terminal pane",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "command": {"type": "string", "description": "Command to execute"},
                                "tab_index": {"type": "integer", "description": "Tab index (optional)"},
                                "pane_index": {"type": "integer", "description": "Pane index (optional)"}
                            },
                            "required": ["command"]
                        }
                    },
                    {
                        "name": "get_output",
                        "description": "Get recent output from a terminal pane",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "tab_index": {"type": "integer"},
                                "pane_index": {"type": "integer"},
                                "lines": {"type": "integer", "description": "Number of lines to retrieve (default: 50)"}
                            }
                        }
                    },
                    {
                        "name": "list_tabs",
                        "description": "List all terminal tabs",
                        "inputSchema": {"type": "object", "properties": {}}
                    },
                    {
                        "name": "list_panes",
                        "description": "List all panes in a tab",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "tab_index": {"type": "integer", "description": "Tab index (optional, defaults to active)"}
                            }
                        }
                    },
                    {
                        "name": "focus_tab",
                        "description": "Focus a specific tab",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "index": {"type": "integer"}
                            },
                            "required": ["index"]
                        }
                    },
                    {
                        "name": "focus_pane",
                        "description": "Focus a specific pane in the active tab",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "index": {"type": "integer"}
                            },
                            "required": ["index"]
                        }
                    }
                ]
            })),

            "tools/call" => {
                let params = params.ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: "Missing params".to_string(),
                    data: None,
                })?;

                let tool_name = params.get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| JsonRpcError {
                        code: -32602,
                        message: "Missing tool name".to_string(),
                        data: None,
                    })?;

                let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

                self.call_tool(tool_name, arguments).await
            }

            _ => Err(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", method),
                data: None,
            }),
        }
    }

    async fn call_tool(&self, name: &str, args: Value) -> Result<Value, JsonRpcError> {
        let command = match name {
            "create_tab" => McpCommand::CreateTab {
                name: args.get("name").and_then(|v| v.as_str()).map(String::from),
            },
            "split_pane" => McpCommand::SplitPane {
                direction: args.get("direction")
                    .and_then(|v| v.as_str())
                    .unwrap_or("vertical")
                    .to_string(),
                tab_index: args.get("tab_index").and_then(|v| v.as_u64()).map(|v| v as usize),
            },
            "run_command" => McpCommand::RunCommand {
                command: args.get("command")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| JsonRpcError {
                        code: -32602,
                        message: "Missing command".to_string(),
                        data: None,
                    })?
                    .to_string(),
                tab_index: args.get("tab_index").and_then(|v| v.as_u64()).map(|v| v as usize),
                pane_index: args.get("pane_index").and_then(|v| v.as_u64()).map(|v| v as usize),
            },
            "get_output" => McpCommand::GetOutput {
                tab_index: args.get("tab_index").and_then(|v| v.as_u64()).map(|v| v as usize),
                pane_index: args.get("pane_index").and_then(|v| v.as_u64()).map(|v| v as usize),
                lines: args.get("lines").and_then(|v| v.as_u64()).map(|v| v as usize),
            },
            "list_tabs" => McpCommand::ListTabs,
            "list_panes" => McpCommand::ListPanes {
                tab_index: args.get("tab_index").and_then(|v| v.as_u64()).map(|v| v as usize),
            },
            "focus_tab" => McpCommand::FocusTab {
                index: args.get("index")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| JsonRpcError {
                        code: -32602,
                        message: "Missing index".to_string(),
                        data: None,
                    })? as usize,
            },
            "focus_pane" => McpCommand::FocusPane {
                index: args.get("index")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| JsonRpcError {
                        code: -32602,
                        message: "Missing index".to_string(),
                        data: None,
                    })? as usize,
            },
            _ => {
                return Err(JsonRpcError {
                    code: -32602,
                    message: format!("Unknown tool: {}", name),
                    data: None,
                });
            }
        };

        // Send command to main app
        self.command_tx.send(command).await.map_err(|e| JsonRpcError {
            code: -32603,
            message: format!("Failed to send command: {}", e),
            data: None,
        })?;

        // Wait for response
        let response = self.response_rx.recv().map_err(|e| JsonRpcError {
            code: -32603,
            message: format!("Failed to receive response: {}", e),
            data: None,
        })?;

        match response {
            McpResponse::Success { data } => Ok(json!({
                "content": [{"type": "text", "text": data.to_string()}]
            })),
            McpResponse::Error { message } => Err(JsonRpcError {
                code: -32603,
                message,
                data: None,
            }),
        }
    }
}

/// Receiver for MCP commands in the main app
pub struct McpCommandReceiver {
    command_rx: mpsc::Receiver<McpCommand>,
    response_tx: std::sync::mpsc::Sender<McpResponse>,
}

impl McpCommandReceiver {
    /// Try to receive a command (non-blocking)
    pub fn try_recv(&mut self) -> Option<McpCommand> {
        self.command_rx.try_recv().ok()
    }

    /// Send a response back to the MCP server
    pub fn send_response(&self, response: McpResponse) -> Result<(), String> {
        self.response_tx.send(response).map_err(|e| e.to_string())
    }
}

/// Session metadata for tracking
#[derive(Debug, Clone)]
struct SessionInfo {
    id: PtyId,
    #[allow(dead_code)]
    name: String,
    rows: u16,
    cols: u16,
    #[allow(dead_code)]
    created_at: std::time::Instant,
}

/// Standalone MCP Server that manages PTY sessions directly
///
/// This server is used when running AgTerm in headless MCP mode (--mcp-server).
/// It provides terminal control for AI agents without requiring a GUI.
pub struct StandaloneMcpServer {
    pty_manager: Arc<PtyManager>,
    sessions: Arc<Mutex<HashMap<String, SessionInfo>>>,
    active_session: Arc<Mutex<Option<String>>>,
}

impl StandaloneMcpServer {
    /// Create a new standalone MCP server
    pub fn new() -> Self {
        Self {
            pty_manager: Arc::new(PtyManager::new()),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            active_session: Arc::new(Mutex::new(None)),
        }
    }

    /// Run the standalone MCP server (blocking, reads from stdin)
    pub async fn run(&self) {
        let stdin = std::io::stdin();
        let reader = BufReader::new(stdin.lock());
        let mut stdout = std::io::stdout();

        tracing::info!("Standalone MCP server started, waiting for requests...");

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    tracing::error!("Error reading stdin: {}", e);
                    break;
                }
            };

            if line.is_empty() {
                continue;
            }

            let response = self.handle_request(&line);
            let response_json = serde_json::to_string(&response).unwrap_or_else(|e| {
                json!({
                    "jsonrpc": "2.0",
                    "error": {"code": -32603, "message": format!("Serialization error: {}", e)}
                }).to_string()
            });

            if let Err(e) = writeln!(stdout, "{}", response_json) {
                tracing::error!("Error writing to stdout: {}", e);
                break;
            }
            let _ = stdout.flush();
        }

        tracing::info!("Standalone MCP server stopped");
    }

    fn handle_request(&self, line: &str) -> JsonRpcResponse {
        let request: JsonRpcRequest = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
            }
        };

        let result = self.handle_method(&request.method, request.params);

        match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(value),
                error: None,
            },
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(e),
            },
        }
    }

    fn handle_method(&self, method: &str, params: Option<Value>) -> Result<Value, JsonRpcError> {
        match method {
            // MCP standard methods
            "initialize" => Ok(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "agterm",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),

            "notifications/initialized" => Ok(json!({})),

            "tools/list" => Ok(json!({
                "tools": [
                    {
                        "name": "create_session",
                        "description": "Create a new terminal session",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "name": {"type": "string", "description": "Session name (optional)"},
                                "rows": {"type": "integer", "description": "Terminal rows (default: 24)"},
                                "cols": {"type": "integer", "description": "Terminal columns (default: 80)"}
                            }
                        }
                    },
                    {
                        "name": "run_command",
                        "description": "Execute a command in a terminal session",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "command": {"type": "string", "description": "Command to execute"},
                                "session": {"type": "string", "description": "Session name (optional, uses active session)"}
                            },
                            "required": ["command"]
                        }
                    },
                    {
                        "name": "get_output",
                        "description": "Get output from a terminal session",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "session": {"type": "string", "description": "Session name (optional, uses active session)"},
                                "wait_ms": {"type": "integer", "description": "Wait time in milliseconds before reading (default: 100)"}
                            }
                        }
                    },
                    {
                        "name": "list_sessions",
                        "description": "List all active terminal sessions",
                        "inputSchema": {"type": "object", "properties": {}}
                    },
                    {
                        "name": "close_session",
                        "description": "Close a terminal session",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "session": {"type": "string", "description": "Session name to close"}
                            },
                            "required": ["session"]
                        }
                    },
                    {
                        "name": "switch_session",
                        "description": "Switch the active session",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "session": {"type": "string", "description": "Session name to activate"}
                            },
                            "required": ["session"]
                        }
                    },
                    {
                        "name": "send_input",
                        "description": "Send raw input to a terminal session (for interactive commands)",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "input": {"type": "string", "description": "Input to send"},
                                "session": {"type": "string", "description": "Session name (optional)"}
                            },
                            "required": ["input"]
                        }
                    },
                    {
                        "name": "send_control",
                        "description": "Send a control signal to a terminal session",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "signal": {"type": "string", "enum": ["ctrl-c", "ctrl-d", "ctrl-z"], "description": "Control signal to send"},
                                "session": {"type": "string", "description": "Session name (optional)"}
                            },
                            "required": ["signal"]
                        }
                    }
                ]
            })),

            "tools/call" => {
                let params = params.ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: "Missing params".to_string(),
                    data: None,
                })?;

                let tool_name = params.get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| JsonRpcError {
                        code: -32602,
                        message: "Missing tool name".to_string(),
                        data: None,
                    })?;

                let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

                self.call_tool(tool_name, arguments)
            }

            _ => Err(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", method),
                data: None,
            }),
        }
    }

    fn call_tool(&self, name: &str, args: Value) -> Result<Value, JsonRpcError> {
        match name {
            "create_session" => self.create_session(args),
            "run_command" => self.run_command(args),
            "get_output" => self.get_output(args),
            "list_sessions" => self.list_sessions(),
            "close_session" => self.close_session(args),
            "switch_session" => self.switch_session(args),
            "send_input" => self.send_input(args),
            "send_control" => self.send_control(args),
            _ => Err(JsonRpcError {
                code: -32602,
                message: format!("Unknown tool: {}", name),
                data: None,
            }),
        }
    }

    fn create_session(&self, args: Value) -> Result<Value, JsonRpcError> {
        let name = args.get("name")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| format!("session-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap()));

        let rows = args.get("rows").and_then(|v| v.as_u64()).unwrap_or(24) as u16;
        let cols = args.get("cols").and_then(|v| v.as_u64()).unwrap_or(80) as u16;

        let pty_id = self.pty_manager.create_session(rows, cols)
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to create session: {}", e),
                data: None,
            })?;

        let session_info = SessionInfo {
            id: pty_id,
            name: name.clone(),
            rows,
            cols,
            created_at: std::time::Instant::now(),
        };

        {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.insert(name.clone(), session_info);
        }

        // Set as active if first session
        {
            let mut active = self.active_session.lock().unwrap();
            if active.is_none() {
                *active = Some(name.clone());
            }
        }

        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Created session '{}' ({}x{})", name, cols, rows)
            }]
        }))
    }

    fn run_command(&self, args: Value) -> Result<Value, JsonRpcError> {
        let command = args.get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing command".to_string(),
                data: None,
            })?;

        let session_name = args.get("session")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| self.active_session.lock().unwrap().clone())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "No active session. Create one first.".to_string(),
                data: None,
            })?;

        let pty_id = {
            let sessions = self.sessions.lock().unwrap();
            sessions.get(&session_name)
                .map(|s| s.id)
                .ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: format!("Session '{}' not found", session_name),
                    data: None,
                })?
        };

        // Send command with newline
        let command_with_newline = format!("{}\n", command);
        self.pty_manager.write(&pty_id, command_with_newline.as_bytes())
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to write to session: {}", e),
                data: None,
            })?;

        // Wait a bit for command to execute
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Read output
        let output = self.pty_manager.read(&pty_id)
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to read from session: {}", e),
                data: None,
            })?;

        let output_str = String::from_utf8_lossy(&output);
        let clean_output = strip_ansi_codes(&output_str);

        Ok(json!({
            "content": [{
                "type": "text",
                "text": clean_output
            }]
        }))
    }

    fn get_output(&self, args: Value) -> Result<Value, JsonRpcError> {
        let session_name = args.get("session")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| self.active_session.lock().unwrap().clone())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "No active session. Create one first.".to_string(),
                data: None,
            })?;

        let wait_ms = args.get("wait_ms").and_then(|v| v.as_u64()).unwrap_or(100);

        let pty_id = {
            let sessions = self.sessions.lock().unwrap();
            sessions.get(&session_name)
                .map(|s| s.id)
                .ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: format!("Session '{}' not found", session_name),
                    data: None,
                })?
        };

        // Wait for output
        std::thread::sleep(std::time::Duration::from_millis(wait_ms));

        let output = self.pty_manager.read(&pty_id)
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to read from session: {}", e),
                data: None,
            })?;

        let output_str = String::from_utf8_lossy(&output);
        let clean_output = strip_ansi_codes(&output_str);

        Ok(json!({
            "content": [{
                "type": "text",
                "text": clean_output
            }]
        }))
    }

    fn list_sessions(&self) -> Result<Value, JsonRpcError> {
        let sessions = self.sessions.lock().unwrap();
        let active = self.active_session.lock().unwrap();

        let session_list: Vec<Value> = sessions.iter().map(|(name, info)| {
            let is_active = active.as_ref() == Some(name);
            json!({
                "name": name,
                "cols": info.cols,
                "rows": info.rows,
                "active": is_active
            })
        }).collect();

        Ok(json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&session_list).unwrap()
            }]
        }))
    }

    fn close_session(&self, args: Value) -> Result<Value, JsonRpcError> {
        let session_name = args.get("session")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing session name".to_string(),
                data: None,
            })?;

        let pty_id = {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.remove(session_name)
                .map(|s| s.id)
                .ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: format!("Session '{}' not found", session_name),
                    data: None,
                })?
        };

        let _ = self.pty_manager.close_session(&pty_id);

        // Clear active session if it was the closed one
        {
            let mut active = self.active_session.lock().unwrap();
            if active.as_ref() == Some(&session_name.to_string()) {
                *active = self.sessions.lock().unwrap().keys().next().cloned();
            }
        }

        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Closed session '{}'", session_name)
            }]
        }))
    }

    fn switch_session(&self, args: Value) -> Result<Value, JsonRpcError> {
        let session_name = args.get("session")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing session name".to_string(),
                data: None,
            })?;

        // Verify session exists
        {
            let sessions = self.sessions.lock().unwrap();
            if !sessions.contains_key(session_name) {
                return Err(JsonRpcError {
                    code: -32602,
                    message: format!("Session '{}' not found", session_name),
                    data: None,
                });
            }
        }

        {
            let mut active = self.active_session.lock().unwrap();
            *active = Some(session_name.to_string());
        }

        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Switched to session '{}'", session_name)
            }]
        }))
    }

    fn send_input(&self, args: Value) -> Result<Value, JsonRpcError> {
        let input = args.get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing input".to_string(),
                data: None,
            })?;

        let session_name = args.get("session")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| self.active_session.lock().unwrap().clone())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "No active session".to_string(),
                data: None,
            })?;

        let pty_id = {
            let sessions = self.sessions.lock().unwrap();
            sessions.get(&session_name)
                .map(|s| s.id)
                .ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: format!("Session '{}' not found", session_name),
                    data: None,
                })?
        };

        self.pty_manager.write(&pty_id, input.as_bytes())
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to write to session: {}", e),
                data: None,
            })?;

        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Sent {} bytes to '{}'", input.len(), session_name)
            }]
        }))
    }

    fn send_control(&self, args: Value) -> Result<Value, JsonRpcError> {
        let signal = args.get("signal")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing signal".to_string(),
                data: None,
            })?;

        let session_name = args.get("session")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| self.active_session.lock().unwrap().clone())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "No active session".to_string(),
                data: None,
            })?;

        let pty_id = {
            let sessions = self.sessions.lock().unwrap();
            sessions.get(&session_name)
                .map(|s| s.id)
                .ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: format!("Session '{}' not found", session_name),
                    data: None,
                })?
        };

        let bytes: &[u8] = match signal {
            "ctrl-c" => &[0x03],
            "ctrl-d" => &[0x04],
            "ctrl-z" => &[0x1a],
            _ => return Err(JsonRpcError {
                code: -32602,
                message: format!("Unknown signal: {}", signal),
                data: None,
            }),
        };

        self.pty_manager.write(&pty_id, bytes)
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to send signal: {}", e),
                data: None,
            })?;

        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Sent {} to '{}'", signal, session_name)
            }]
        }))
    }
}

impl Default for StandaloneMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_rpc_parse() {
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#;
        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.method, "initialize");
    }

    #[test]
    fn test_mcp_command_serialize() {
        let cmd = McpCommand::CreateTab { name: Some("test".to_string()) };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("CreateTab"));
    }
}
