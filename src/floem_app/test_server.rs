//! UI Automation Test Server
//!
//! Provides a TCP server for UI automation testing.
//! When started with `--test-server`, the app listens on port 19999 for test commands.
//!
//! ## Protocol
//!
//! JSON-RPC 2.0 over TCP (newline-delimited JSON)
//!
//! ## Available Commands
//!
//! ### State Queries
//! - `get_state` - Get full UI state snapshot
//! - `get_tabs` - Get tab list
//! - `get_active_tab` - Get active tab info
//! - `get_panes` - Get panes in active tab
//!
//! ### UI Actions
//! - `create_tab` - Create new tab
//! - `close_tab` - Close tab by index
//! - `switch_tab` - Switch to tab by index
//! - `split_pane` - Split current pane
//! - `send_keys` - Send keystrokes to focused pane
//! - `set_theme` - Change theme
//!
//! ### Assertions
//! - `assert_tab_count` - Assert number of tabs
//! - `assert_active_tab` - Assert active tab index
//! - `wait_for` - Wait for condition with timeout

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use floem::reactive::{SignalGet, SignalUpdate};

use super::state::AppState;

/// Default test server port
pub const TEST_SERVER_PORT: u16 = 19999;

/// Test command request
#[derive(Debug, Deserialize)]
struct TestRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

/// Test command response
#[derive(Debug, Serialize)]
struct TestResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<TestError>,
}

/// Test error
#[derive(Debug, Serialize)]
struct TestError {
    code: i32,
    message: String,
}

/// UI state snapshot (serializable)
#[derive(Debug, Serialize, Deserialize)]
pub struct UiStateSnapshot {
    pub tabs: Vec<TabSnapshot>,
    pub active_tab_index: usize,
    pub font_size: f32,
    pub theme_name: String,
}

/// Tab snapshot
#[derive(Debug, Serialize, Deserialize)]
pub struct TabSnapshot {
    pub id: String,
    pub title: String,
    pub is_active: bool,
    pub pane_count: usize,
}

/// Pane snapshot
#[derive(Debug, Serialize, Deserialize)]
pub struct PaneSnapshot {
    pub id: String,
    pub has_pty: bool,
    pub is_focused: bool,
}

/// Test server state
pub struct TestServer {
    app_state: Arc<Mutex<Option<AppState>>>,
    running: Arc<Mutex<bool>>,
}

impl TestServer {
    /// Create a new test server
    pub fn new() -> Self {
        Self {
            app_state: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Set the app state (called from main thread after initialization)
    pub fn set_app_state(&self, state: AppState) {
        let mut app_state = self.app_state.lock().unwrap();
        *app_state = Some(state);
    }

    /// Start the test server in a background thread
    pub fn start(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", TEST_SERVER_PORT))?;
        listener.set_nonblocking(false)?;

        tracing::info!("Test server listening on port {}", TEST_SERVER_PORT);

        let app_state = Arc::clone(&self.app_state);
        let running = Arc::clone(&self.running);

        *running.lock().unwrap() = true;

        thread::spawn(move || {
            for stream in listener.incoming() {
                if !*running.lock().unwrap() {
                    break;
                }

                match stream {
                    Ok(stream) => {
                        let app_state = Arc::clone(&app_state);
                        thread::spawn(move || {
                            if let Err(e) = handle_client(stream, app_state) {
                                tracing::error!("Test client error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Test server accept error: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the test server
    pub fn stop(&self) {
        *self.running.lock().unwrap() = false;
    }
}

/// Handle a test client connection
fn handle_client(
    mut stream: TcpStream,
    app_state: Arc<Mutex<Option<AppState>>>,
) -> std::io::Result<()> {
    let reader = BufReader::new(stream.try_clone()?);

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<TestRequest>(&line) {
            Ok(request) => process_request(request, &app_state),
            Err(e) => TestResponse {
                jsonrpc: "2.0".to_string(),
                id: None,
                result: None,
                error: Some(TestError {
                    code: -32700,
                    message: format!("Parse error: {}", e),
                }),
            },
        };

        let response_json = serde_json::to_string(&response)?;
        writeln!(stream, "{}", response_json)?;
        stream.flush()?;
    }

    Ok(())
}

/// Process a test request
fn process_request(
    request: TestRequest,
    app_state: &Arc<Mutex<Option<AppState>>>,
) -> TestResponse {
    let state_guard = app_state.lock().unwrap();
    let state = match state_guard.as_ref() {
        Some(s) => s,
        None => {
            return TestResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(TestError {
                    code: -32603,
                    message: "App state not initialized".to_string(),
                }),
            };
        }
    };

    let params = request.params.unwrap_or(json!({}));

    let result = match request.method.as_str() {
        // State queries
        "get_state" => get_state(state),
        "get_tabs" => get_tabs(state),
        "get_active_tab" => get_active_tab(state),
        "get_panes" => get_panes(state),

        // UI actions
        "create_tab" => create_tab(state),
        "close_tab" => close_tab(state, &params),
        "switch_tab" => switch_tab(state, &params),
        "send_keys" => send_keys(state, &params),
        "set_theme" => set_theme(state, &params),
        "set_font_size" => set_font_size(state, &params),

        // Assertions
        "assert_tab_count" => assert_tab_count(state, &params),
        "assert_active_tab" => assert_active_tab(state, &params),
        "ping" => Ok(json!({"pong": true})),

        _ => Err(TestError {
            code: -32601,
            message: format!("Method not found: {}", request.method),
        }),
    };

    match result {
        Ok(value) => TestResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(value),
            error: None,
        },
        Err(error) => TestResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(error),
        },
    }
}

// ============================================================================
// State Query Handlers
// ============================================================================

fn get_state(state: &AppState) -> Result<Value, TestError> {
    let tabs = state.tabs.get();
    let active_tab = state.active_tab.get();
    let font_size = state.font_size.get();
    let theme = state.theme.get();

    let tab_snapshots: Vec<TabSnapshot> = tabs
        .iter()
        .map(|tab| {
            let tree = tab.pane_tree.get();
            TabSnapshot {
                id: tab.id.to_string(),
                title: tab.title.get(),
                is_active: tab.is_active.get(),
                pane_count: tree.get_all_leaf_ids().len(),
            }
        })
        .collect();

    Ok(json!(UiStateSnapshot {
        tabs: tab_snapshots,
        active_tab_index: active_tab,
        font_size,
        theme_name: theme.name().to_string(),
    }))
}

fn get_tabs(state: &AppState) -> Result<Value, TestError> {
    let tabs = state.tabs.get();

    let tab_list: Vec<Value> = tabs
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            json!({
                "index": i,
                "id": tab.id.to_string(),
                "title": tab.title.get(),
                "is_active": tab.is_active.get(),
            })
        })
        .collect();

    Ok(json!({"tabs": tab_list, "count": tab_list.len()}))
}

fn get_active_tab(state: &AppState) -> Result<Value, TestError> {
    let tabs = state.tabs.get();
    let active_index = state.active_tab.get();

    if let Some(tab) = tabs.get(active_index) {
        let tree = tab.pane_tree.get();
        Ok(json!({
            "index": active_index,
            "id": tab.id.to_string(),
            "title": tab.title.get(),
            "pane_count": tree.get_all_leaf_ids().len(),
        }))
    } else {
        Err(TestError {
            code: -32602,
            message: "No active tab".to_string(),
        })
    }
}

fn get_panes(state: &AppState) -> Result<Value, TestError> {
    let tabs = state.tabs.get();
    let active_index = state.active_tab.get();

    if let Some(tab) = tabs.get(active_index) {
        let tree = tab.pane_tree.get();
        let pane_ids = tree.get_all_leaf_ids();
        let focused_id = tree.get_focused_leaf().map(|(id, _)| id);

        let panes: Vec<Value> = pane_ids
            .iter()
            .map(|id| {
                json!({
                    "id": id.to_string(),
                    "is_focused": Some(*id) == focused_id,
                })
            })
            .collect();

        Ok(json!({"panes": panes, "count": panes.len()}))
    } else {
        Err(TestError {
            code: -32602,
            message: "No active tab".to_string(),
        })
    }
}

// ============================================================================
// UI Action Handlers
// ============================================================================

fn create_tab(state: &AppState) -> Result<Value, TestError> {
    let new_tab = super::state::Tab::new("New Tab", &state.pty_manager);
    let tab_id = new_tab.id.to_string();

    state.tabs.update(|tabs| {
        // Deactivate all existing tabs
        for tab in tabs.iter() {
            tab.is_active.set(false);
        }
        // Add and activate new tab
        new_tab.is_active.set(true);
        tabs.push(new_tab);
    });

    let new_index = state.tabs.get().len() - 1;
    state.active_tab.set(new_index);

    Ok(json!({
        "success": true,
        "tab_id": tab_id,
        "index": new_index,
    }))
}

fn close_tab(state: &AppState, params: &Value) -> Result<Value, TestError> {
    let index = params
        .get("index")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);

    let tabs = state.tabs.get();
    let target_index = index.unwrap_or_else(|| state.active_tab.get());

    if target_index >= tabs.len() {
        return Err(TestError {
            code: -32602,
            message: format!("Invalid tab index: {}", target_index),
        });
    }

    if tabs.len() <= 1 {
        return Err(TestError {
            code: -32602,
            message: "Cannot close last tab".to_string(),
        });
    }

    // Cleanup PTY sessions
    tabs[target_index].cleanup(&state.pty_manager);

    drop(tabs);

    state.tabs.update(|tabs| {
        tabs.remove(target_index);
    });

    // Adjust active tab if needed
    let new_len = state.tabs.get().len();
    let active = state.active_tab.get();
    if active >= new_len {
        state.active_tab.set(new_len - 1);
    }

    Ok(json!({"success": true, "closed_index": target_index}))
}

fn switch_tab(state: &AppState, params: &Value) -> Result<Value, TestError> {
    let index = params
        .get("index")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| TestError {
            code: -32602,
            message: "Missing 'index' parameter".to_string(),
        })? as usize;

    let tabs = state.tabs.get();

    if index >= tabs.len() {
        return Err(TestError {
            code: -32602,
            message: format!("Invalid tab index: {}", index),
        });
    }

    // Update active states
    for (i, tab) in tabs.iter().enumerate() {
        tab.is_active.set(i == index);
    }

    drop(tabs);
    state.active_tab.set(index);

    Ok(json!({"success": true, "active_index": index}))
}

fn send_keys(state: &AppState, params: &Value) -> Result<Value, TestError> {
    let keys = params
        .get("keys")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TestError {
            code: -32602,
            message: "Missing 'keys' parameter".to_string(),
        })?;

    let tabs = state.tabs.get();
    let active_index = state.active_tab.get();

    if let Some(tab) = tabs.get(active_index) {
        if let Some(session_id) = tab.get_focused_pty_session() {
            state
                .pty_manager
                .write(&session_id, keys.as_bytes())
                .map_err(|e| TestError {
                    code: -32603,
                    message: format!("Failed to send keys: {}", e),
                })?;

            Ok(json!({"success": true, "sent": keys}))
        } else {
            Err(TestError {
                code: -32603,
                message: "No PTY session in focused pane".to_string(),
            })
        }
    } else {
        Err(TestError {
            code: -32602,
            message: "No active tab".to_string(),
        })
    }
}

fn set_theme(state: &AppState, params: &Value) -> Result<Value, TestError> {
    let theme_name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TestError {
            code: -32602,
            message: "Missing 'name' parameter".to_string(),
        })?;

    // Try to find the theme
    let theme = super::theme::Theme::from_name_opt(theme_name)
        .ok_or_else(|| TestError {
            code: -32602,
            message: format!("Theme not found: {}. Available: Ghostty Dark, Ghostty Light", theme_name),
        })?;

    state.theme.set(theme);

    Ok(json!({"success": true, "theme": theme.name()}))
}

fn set_font_size(state: &AppState, params: &Value) -> Result<Value, TestError> {
    let size = params
        .get("size")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| TestError {
            code: -32602,
            message: "Missing 'size' parameter".to_string(),
        })? as f32;

    if size < 6.0 || size > 72.0 {
        return Err(TestError {
            code: -32602,
            message: "Font size must be between 6 and 72".to_string(),
        });
    }

    state.font_size.set(size);

    Ok(json!({"success": true, "font_size": size}))
}

// ============================================================================
// Assertion Handlers
// ============================================================================

fn assert_tab_count(state: &AppState, params: &Value) -> Result<Value, TestError> {
    let expected = params
        .get("count")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| TestError {
            code: -32602,
            message: "Missing 'count' parameter".to_string(),
        })? as usize;

    let actual = state.tabs.get().len();

    if actual == expected {
        Ok(json!({"success": true, "count": actual}))
    } else {
        Err(TestError {
            code: -32000,
            message: format!("Tab count assertion failed: expected {}, got {}", expected, actual),
        })
    }
}

fn assert_active_tab(state: &AppState, params: &Value) -> Result<Value, TestError> {
    let expected = params
        .get("index")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| TestError {
            code: -32602,
            message: "Missing 'index' parameter".to_string(),
        })? as usize;

    let actual = state.active_tab.get();

    if actual == expected {
        Ok(json!({"success": true, "index": actual}))
    } else {
        Err(TestError {
            code: -32000,
            message: format!(
                "Active tab assertion failed: expected {}, got {}",
                expected, actual
            ),
        })
    }
}

// ============================================================================
// Test Client (for integration tests)
// ============================================================================

/// Test client for connecting to the test server
pub struct TestClient {
    stream: TcpStream,
    id_counter: u64,
}

impl TestClient {
    /// Connect to the test server
    pub fn connect() -> std::io::Result<Self> {
        Self::connect_to(TEST_SERVER_PORT)
    }

    /// Connect to a specific port
    pub fn connect_to(port: u16) -> std::io::Result<Self> {
        let stream = TcpStream::connect(format!("127.0.0.1:{}", port))?;
        stream.set_read_timeout(Some(std::time::Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(std::time::Duration::from_secs(5)))?;

        Ok(Self {
            stream,
            id_counter: 0,
        })
    }

    /// Send a request and get response
    pub fn request(&mut self, method: &str, params: Option<Value>) -> Result<Value, String> {
        self.id_counter += 1;

        let request = json!({
            "jsonrpc": "2.0",
            "id": self.id_counter,
            "method": method,
            "params": params,
        });

        writeln!(self.stream, "{}", request.to_string()).map_err(|e| e.to_string())?;
        self.stream.flush().map_err(|e| e.to_string())?;

        let mut reader = BufReader::new(self.stream.try_clone().map_err(|e| e.to_string())?);
        let mut response_line = String::new();
        reader
            .read_line(&mut response_line)
            .map_err(|e| e.to_string())?;

        let response: Value = serde_json::from_str(&response_line).map_err(|e| e.to_string())?;

        if let Some(error) = response.get("error") {
            Err(error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
                .to_string())
        } else {
            Ok(response.get("result").cloned().unwrap_or(json!(null)))
        }
    }

    // Convenience methods
    pub fn ping(&mut self) -> Result<(), String> {
        self.request("ping", None)?;
        Ok(())
    }

    pub fn get_state(&mut self) -> Result<UiStateSnapshot, String> {
        let result = self.request("get_state", None)?;
        serde_json::from_value(result).map_err(|e| e.to_string())
    }

    pub fn create_tab(&mut self) -> Result<Value, String> {
        self.request("create_tab", None)
    }

    pub fn close_tab(&mut self, index: Option<usize>) -> Result<Value, String> {
        self.request("close_tab", Some(json!({"index": index})))
    }

    pub fn switch_tab(&mut self, index: usize) -> Result<Value, String> {
        self.request("switch_tab", Some(json!({"index": index})))
    }

    pub fn send_keys(&mut self, keys: &str) -> Result<Value, String> {
        self.request("send_keys", Some(json!({"keys": keys})))
    }

    pub fn assert_tab_count(&mut self, count: usize) -> Result<(), String> {
        self.request("assert_tab_count", Some(json!({"count": count})))?;
        Ok(())
    }

    pub fn assert_active_tab(&mut self, index: usize) -> Result<(), String> {
        self.request("assert_active_tab", Some(json!({"index": index})))?;
        Ok(())
    }
}
