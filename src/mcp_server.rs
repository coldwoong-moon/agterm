//! MCP Server for AgTerm
//!
//! This module implements an MCP (Model Context Protocol) server that allows
//! AI agents like Claude Code to control AgTerm terminals programmatically.
//!
//! ## Usage Modes
//!
//! ```bash
//! # MCP server with GUI (default)
//! agterm --mcp-server
//!
//! # MCP server only (headless, no GUI)
//! agterm --mcp-server --headless
//! ```
//!
//! ## Claude Code Integration
//!
//! Add to `.mcp.json` in your project:
//! ```json
//! {
//!   "mcpServers": {
//!     "agterm": {
//!       "command": "/path/to/agterm",
//!       "args": ["--mcp-server"]
//!     }
//!   }
//! }
//! ```
//!
//! ## Available MCP Tools
//!
//! ### Session Management
//! - `create_session`: Create a new terminal session with optional name, rows, cols
//! - `list_sessions`: List all active terminal sessions
//! - `close_session`: Close a terminal session by name
//! - `switch_session`: Switch the active session
//! - `resize_session`: Resize a terminal session (rows, cols)
//!
//! ### Command Execution
//! - `run_command`: Execute a command in a session
//!   - `wait: true` (default): Wait for output and return it
//!   - `wait: false`: Async execution, return immediately
//!   - `wait_ms`: Custom wait time in milliseconds (default: 300)
//! - `send_input`: Send raw input to a session (for interactive commands)
//! - `send_control`: Send control signals (ctrl-c, ctrl-d, ctrl-z)
//!
//! ### Output Retrieval
//! - `get_output`: Get output from a session with optional wait_ms
//!
//! ### Environment & Directory
//! - `get_cwd`: Get current working directory of a session
//! - `set_cwd`: Change working directory of a session
//! - `set_env`: Set environment variable in a session
//!
//! ### History
//! - `get_history`: Get command history from a session (with optional limit)
//!
//! ## Example Workflow
//!
//! ```text
//! 1. create_session(name: "build")
//! 2. run_command(command: "cargo build", wait: false)  // async
//! 3. get_output(wait_ms: 5000)  // check progress
//! 4. run_command(command: "cargo test")  // sync, wait for result
//! 5. close_session(session: "build")
//! ```

use crate::terminal::pty::{PtyId, PtyManager};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Strip ANSI escape codes and clean terminal output
fn strip_ansi_codes(input: &str) -> String {
    // Comprehensive ANSI/terminal escape sequence patterns
    let ansi_regex = Regex::new(concat!(
        r"\x1b\[[0-9;?]*[a-zA-Z~]|",           // CSI sequences
        r"\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)|", // OSC sequences
        r"\x1bP[^\x1b]*\x1b\\|",               // DCS (Device Control String)
        r"\x1b[()][AB012]|",                   // Character set selection
        r"\x1b[>=]|",                          // Application/Normal keypad
        r"\x1b[78]|",                          // Save/Restore cursor
        r"\x1b[DME]|",                         // Line control
        r"P\+q[0-9a-fA-F]+\\|",                // Escaped DCS responses
        r"\x1b\[[\x30-\x3f]*[\x20-\x2f]*[\x40-\x7e]"  // More CSI sequences
    )).unwrap();

    let result = ansi_regex.replace_all(input, "");

    // Clean up control characters
    let cleaned: String = result.chars()
        .filter(|c| *c == '\n' || *c == ' ' || (*c >= ' ' && !c.is_control()))
        .collect();

    // Patterns to filter out
    let noise_patterns = [
        "warning: fish could not",
        "This is often due to",
        "See 'help terminal-compatibility'",
        "man fish-terminal-compatibility",
        "This fish process will no longer",
        "Welcome to fish",
        "Type help for instructions",
        "friendly interactive shell",
    ];

    // Remove noise lines and shell prompts
    let lines: Vec<&str> = cleaned.lines()
        .map(|line| line.trim_end())
        .filter(|line| {
            if line.is_empty() {
                return false;
            }
            // Filter noise patterns
            for pattern in &noise_patterns {
                if line.contains(pattern) {
                    return false;
                }
            }
            // Filter lines that are just special chars (%, ⏎, etc.)
            if line.chars().all(|c| c == '%' || c == '⏎' || c.is_whitespace()) {
                return false;
            }
            // Filter out shell prompt lines (various formats)
            // zsh: user@host path %
            // bash: user@host:path$
            // fish: user@host path (branch)>
            if line.contains("@") && (
                line.ends_with(" %") ||
                line.ends_with(">") ||
                line.ends_with("$") ||
                line.contains(" % ") ||
                line.contains(")>")
            ) {
                return false;
            }
            true
        })
        .collect();

    lines.join("\n")
}

/// Remove echoed command from output
fn remove_command_echo(output: &str, command: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();
    let cmd_trimmed = command.trim();

    // Filter out lines that match the command (echoed input)
    let filtered: Vec<&str> = lines.into_iter()
        .filter(|line| {
            let line_trimmed = line.trim();
            // Skip if line exactly matches command or contains it as echo
            if line_trimmed == cmd_trimmed {
                return false;
            }
            // Skip if line starts with the command (partial echo)
            if line_trimmed.starts_with(cmd_trimmed) && line_trimmed.len() == cmd_trimmed.len() {
                return false;
            }
            true
        })
        .collect();

    filtered.join("\n")
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

/// Command history entry
#[derive(Debug, Clone, Serialize)]
struct HistoryEntry {
    command: String,
    timestamp: u64,
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
    history: Vec<HistoryEntry>,
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
                                "session": {"type": "string", "description": "Session name (optional, uses active session)"},
                                "wait": {"type": "boolean", "description": "Wait for output (default: true). Set false for async execution."},
                                "wait_ms": {"type": "integer", "description": "Wait time in milliseconds (default: 300)"}
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
                    },
                    {
                        "name": "resize_session",
                        "description": "Resize a terminal session",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "rows": {"type": "integer", "description": "New number of rows"},
                                "cols": {"type": "integer", "description": "New number of columns"},
                                "session": {"type": "string", "description": "Session name (optional)"}
                            },
                            "required": ["rows", "cols"]
                        }
                    },
                    {
                        "name": "get_cwd",
                        "description": "Get current working directory of a session",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "session": {"type": "string", "description": "Session name (optional)"}
                            }
                        }
                    },
                    {
                        "name": "set_cwd",
                        "description": "Change working directory of a session",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "path": {"type": "string", "description": "Directory path to change to"},
                                "session": {"type": "string", "description": "Session name (optional)"}
                            },
                            "required": ["path"]
                        }
                    },
                    {
                        "name": "set_env",
                        "description": "Set environment variable in a session",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "name": {"type": "string", "description": "Environment variable name"},
                                "value": {"type": "string", "description": "Environment variable value"},
                                "session": {"type": "string", "description": "Session name (optional)"}
                            },
                            "required": ["name", "value"]
                        }
                    },
                    {
                        "name": "get_history",
                        "description": "Get command history from a session",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "session": {"type": "string", "description": "Session name (optional)"},
                                "limit": {"type": "integer", "description": "Maximum number of entries to return (default: 50)"}
                            }
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
            "resize_session" => self.resize_session(args),
            "get_cwd" => self.get_cwd(args),
            "set_cwd" => self.set_cwd(args),
            "set_env" => self.set_env(args),
            "get_history" => self.get_history(args),
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
            history: Vec::new(),
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

        // Parse wait options
        let should_wait = args.get("wait").and_then(|v| v.as_bool()).unwrap_or(true);
        let wait_ms = args.get("wait_ms").and_then(|v| v.as_u64()).unwrap_or(300);

        let pty_id = {
            let mut sessions = self.sessions.lock().unwrap();
            let session = sessions.get_mut(&session_name)
                .ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: format!("Session '{}' not found", session_name),
                    data: None,
                })?;

            // Record command in history
            session.history.push(HistoryEntry {
                command: command.to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            });

            session.id
        };

        // Send command with newline
        let command_with_newline = format!("{}\n", command);
        self.pty_manager.write(&pty_id, command_with_newline.as_bytes())
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to write to session: {}", e),
                data: None,
            })?;

        // If async execution requested, return immediately
        if !should_wait {
            return Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Command '{}' sent to session '{}'. Use get_output to retrieve results.", command, session_name)
                }]
            }));
        }

        // Wait for command to execute
        std::thread::sleep(std::time::Duration::from_millis(wait_ms));

        // Read output
        let output = self.pty_manager.read(&pty_id)
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to read from session: {}", e),
                data: None,
            })?;

        let output_str = String::from_utf8_lossy(&output);
        let clean_output = strip_ansi_codes(&output_str);

        // Remove echoed command from output
        let final_output = remove_command_echo(&clean_output, command);

        Ok(json!({
            "content": [{
                "type": "text",
                "text": final_output
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

    fn resize_session(&self, args: Value) -> Result<Value, JsonRpcError> {
        let rows = args.get("rows")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing rows".to_string(),
                data: None,
            })? as u16;

        let cols = args.get("cols")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing cols".to_string(),
                data: None,
            })? as u16;

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
            let mut sessions = self.sessions.lock().unwrap();
            let session = sessions.get_mut(&session_name)
                .ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: format!("Session '{}' not found", session_name),
                    data: None,
                })?;
            // Update stored dimensions
            session.rows = rows;
            session.cols = cols;
            session.id
        };

        self.pty_manager.resize(&pty_id, rows, cols)
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to resize session: {}", e),
                data: None,
            })?;

        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Resized session '{}' to {}x{}", session_name, cols, rows)
            }]
        }))
    }

    fn get_cwd(&self, args: Value) -> Result<Value, JsonRpcError> {
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

        // Execute pwd command and get output
        self.pty_manager.write(&pty_id, b"pwd\n")
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to execute pwd: {}", e),
                data: None,
            })?;

        std::thread::sleep(std::time::Duration::from_millis(100));

        let output = self.pty_manager.read(&pty_id)
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to read output: {}", e),
                data: None,
            })?;

        let output_str = String::from_utf8_lossy(&output);
        let clean_output = strip_ansi_codes(&output_str);
        let cwd = remove_command_echo(&clean_output, "pwd")
            .lines()
            .find(|line| line.starts_with('/'))
            .unwrap_or("")
            .trim()
            .to_string();

        Ok(json!({
            "content": [{
                "type": "text",
                "text": cwd
            }]
        }))
    }

    fn set_cwd(&self, args: Value) -> Result<Value, JsonRpcError> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing path".to_string(),
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

        // Execute cd command
        let cd_cmd = format!("cd {} && pwd\n", path);
        self.pty_manager.write(&pty_id, cd_cmd.as_bytes())
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to change directory: {}", e),
                data: None,
            })?;

        std::thread::sleep(std::time::Duration::from_millis(150));

        let output = self.pty_manager.read(&pty_id)
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to read output: {}", e),
                data: None,
            })?;

        let output_str = String::from_utf8_lossy(&output);
        let clean_output = strip_ansi_codes(&output_str);

        // Check if cd succeeded by looking for the path in output
        let new_cwd = clean_output
            .lines()
            .find(|line| line.starts_with('/'))
            .unwrap_or("")
            .trim()
            .to_string();

        if new_cwd.is_empty() {
            return Err(JsonRpcError {
                code: -32603,
                message: format!("Failed to change to directory: {}", path),
                data: None,
            });
        }

        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Changed directory to: {}", new_cwd)
            }]
        }))
    }

    fn set_env(&self, args: Value) -> Result<Value, JsonRpcError> {
        let name = args.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing environment variable name".to_string(),
                data: None,
            })?;

        let value = args.get("value")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing environment variable value".to_string(),
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

        // Export environment variable (works in bash, zsh, fish)
        let export_cmd = format!("export {}='{}'\n", name, value.replace('\'', "'\\''"));
        self.pty_manager.write(&pty_id, export_cmd.as_bytes())
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Failed to set environment variable: {}", e),
                data: None,
            })?;

        std::thread::sleep(std::time::Duration::from_millis(50));

        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Set {}={}", name, value)
            }]
        }))
    }

    fn get_history(&self, args: Value) -> Result<Value, JsonRpcError> {
        let session_name = args.get("session")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| self.active_session.lock().unwrap().clone())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "No active session".to_string(),
                data: None,
            })?;

        let limit = args.get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(50) as usize;

        let history = {
            let sessions = self.sessions.lock().unwrap();
            let session = sessions.get(&session_name)
                .ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: format!("Session '{}' not found", session_name),
                    data: None,
                })?;

            let start = if session.history.len() > limit {
                session.history.len() - limit
            } else {
                0
            };
            session.history[start..].to_vec()
        };

        Ok(json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&history).unwrap_or_else(|_| "[]".to_string())
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
    fn test_json_rpc_with_params() {
        let json = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"run_command","arguments":{"command":"ls"}}}"#;
        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.method, "tools/call");
        assert!(request.params.is_some());
        let params = request.params.unwrap();
        assert_eq!(params.get("name").unwrap(), "run_command");
    }

    #[test]
    fn test_mcp_command_serialize() {
        let cmd = McpCommand::CreateTab { name: Some("test".to_string()) };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("CreateTab"));
    }

    #[test]
    fn test_strip_ansi_codes() {
        // Basic CSI sequence
        let input = "\x1b[32mgreen\x1b[0m";
        let output = strip_ansi_codes(input);
        assert_eq!(output, "green");

        // OSC sequence (title)
        let input = "\x1b]0;Terminal Title\x07text";
        let output = strip_ansi_codes(input);
        assert_eq!(output, "text");

        // Multiple sequences
        let input = "\x1b[1m\x1b[34mbold blue\x1b[0m normal";
        let output = strip_ansi_codes(input);
        assert_eq!(output, "bold blue normal");
    }

    #[test]
    fn test_remove_command_echo() {
        let output = "ls\nfile1.txt\nfile2.txt";
        let result = remove_command_echo(output, "ls");
        assert_eq!(result, "file1.txt\nfile2.txt");

        // Command not in output
        let output = "file1.txt\nfile2.txt";
        let result = remove_command_echo(output, "ls");
        assert_eq!(result, "file1.txt\nfile2.txt");
    }

    #[test]
    fn test_history_entry_serialization() {
        let entry = HistoryEntry {
            command: "echo hello".to_string(),
            timestamp: 1234567890,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("echo hello"));
        assert!(json.contains("1234567890"));
    }

    #[test]
    fn test_standalone_server_initialization() {
        let server = StandaloneMcpServer::new();
        let sessions = server.sessions.lock().unwrap();
        assert!(sessions.is_empty());
        let active = server.active_session.lock().unwrap();
        assert!(active.is_none());
    }

    #[test]
    fn test_json_rpc_response_serialization() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            result: Some(json!({"status": "ok"})),
            error: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"result\""));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_json_rpc_error_response() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            result: None,
            error: Some(JsonRpcError {
                code: -32600,
                message: "Invalid request".to_string(),
                data: None,
            }),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"error\""));
        assert!(json.contains("-32600"));
    }
}
