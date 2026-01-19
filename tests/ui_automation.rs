//! UI Automation Integration Tests
//!
//! These tests spawn the AgTerm GUI with `--test-server` flag and
//! control it via TCP to verify UI behavior.
//!
//! # Running Tests
//!
//! ```bash
//! cargo test --test ui_automation
//! ```
//!
//! # Test Server Protocol
//!
//! The test server listens on port 19999 and accepts JSON-RPC 2.0 commands.

use serde_json::json;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

/// Test server port
const TEST_PORT: u16 = 19999;

/// Helper struct to manage GUI process lifecycle
struct GuiProcess {
    child: Child,
}

impl GuiProcess {
    /// Spawn the GUI with test server enabled
    fn spawn() -> Result<Self, String> {
        let child = Command::new("cargo")
            .args(["run", "--", "--test-server"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn GUI: {}", e))?;

        // Wait for server to be ready
        thread::sleep(Duration::from_secs(3));

        Ok(Self { child })
    }
}

impl Drop for GuiProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

/// Test client for connecting to the test server
struct TestClient {
    stream: TcpStream,
    id_counter: u64,
}

impl TestClient {
    fn connect() -> Result<Self, String> {
        // Try connecting with retries
        for attempt in 1..=10 {
            match TcpStream::connect(format!("127.0.0.1:{}", TEST_PORT)) {
                Ok(stream) => {
                    stream
                        .set_read_timeout(Some(Duration::from_secs(5)))
                        .map_err(|e| e.to_string())?;
                    return Ok(Self {
                        stream,
                        id_counter: 0,
                    });
                }
                Err(_) if attempt < 10 => {
                    thread::sleep(Duration::from_millis(500));
                }
                Err(e) => return Err(format!("Failed to connect after 10 attempts: {}", e)),
            }
        }
        Err("Failed to connect".to_string())
    }

    fn request(&mut self, method: &str, params: Option<serde_json::Value>) -> Result<serde_json::Value, String> {
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

        let response: serde_json::Value =
            serde_json::from_str(&response_line).map_err(|e| e.to_string())?;

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
}

// ============================================================================
// Unit Tests (no GUI required)
// ============================================================================

#[test]
fn test_json_rpc_request_format() {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "get_state",
        "params": null,
    });

    assert_eq!(request["method"], "get_state");
    assert_eq!(request["jsonrpc"], "2.0");
}

#[test]
fn test_tab_snapshot_serialization() {
    let snapshot = json!({
        "id": "test-id",
        "title": "Test Tab",
        "is_active": true,
        "pane_count": 1,
    });

    assert_eq!(snapshot["title"], "Test Tab");
    assert_eq!(snapshot["pane_count"], 1);
}

// ============================================================================
// Integration Tests (require --test-server flag implementation)
// ============================================================================

/// Test: Connect to test server and ping
///
/// This test requires the GUI to be running with --test-server flag.
/// Run manually with: cargo run -- --test-server &
#[test]
#[ignore = "Requires GUI with --test-server flag"]
fn test_ping() {
    let mut client = TestClient::connect().expect("Failed to connect");
    let result = client.request("ping", None).expect("Ping failed");
    assert!(result.get("pong").and_then(|v| v.as_bool()).unwrap_or(false));
}

/// Test: Get initial UI state
#[test]
#[ignore = "Requires GUI with --test-server flag"]
fn test_get_state() {
    let mut client = TestClient::connect().expect("Failed to connect");
    let result = client.request("get_state", None).expect("get_state failed");

    // Should have at least one tab
    let tabs = result.get("tabs").and_then(|v| v.as_array());
    assert!(tabs.is_some());
    assert!(!tabs.unwrap().is_empty());

    // Should have font_size
    let font_size = result.get("font_size").and_then(|v| v.as_f64());
    assert!(font_size.is_some());
}

/// Test: Create and close tab
#[test]
#[ignore = "Requires GUI with --test-server flag"]
fn test_create_and_close_tab() {
    let mut client = TestClient::connect().expect("Failed to connect");

    // Get initial tab count
    let state = client.request("get_state", None).expect("get_state failed");
    let initial_count = state
        .get("tabs")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    // Create new tab
    let result = client.request("create_tab", None).expect("create_tab failed");
    assert!(result.get("success").and_then(|v| v.as_bool()).unwrap_or(false));

    // Verify tab count increased
    let state = client.request("get_state", None).expect("get_state failed");
    let new_count = state
        .get("tabs")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    assert_eq!(new_count, initial_count + 1);

    // Close the new tab
    let result = client
        .request("close_tab", Some(json!({"index": new_count - 1})))
        .expect("close_tab failed");
    assert!(result.get("success").and_then(|v| v.as_bool()).unwrap_or(false));

    // Verify tab count restored
    let state = client.request("get_state", None).expect("get_state failed");
    let final_count = state
        .get("tabs")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    assert_eq!(final_count, initial_count);
}

/// Test: Switch tabs
#[test]
#[ignore = "Requires GUI with --test-server flag"]
fn test_switch_tab() {
    let mut client = TestClient::connect().expect("Failed to connect");

    // Create a second tab
    client.request("create_tab", None).expect("create_tab failed");

    // Switch to first tab
    let result = client
        .request("switch_tab", Some(json!({"index": 0})))
        .expect("switch_tab failed");
    assert!(result.get("success").and_then(|v| v.as_bool()).unwrap_or(false));

    // Verify active tab
    client
        .request("assert_active_tab", Some(json!({"index": 0})))
        .expect("assert_active_tab failed");

    // Cleanup - close the extra tab
    let _ = client.request("close_tab", Some(json!({"index": 1})));
}

/// Test: Send keys to terminal
#[test]
#[ignore = "Requires GUI with --test-server flag"]
fn test_send_keys() {
    let mut client = TestClient::connect().expect("Failed to connect");

    // Send some keys
    let result = client
        .request("send_keys", Some(json!({"keys": "echo hello\n"})))
        .expect("send_keys failed");
    assert!(result.get("success").and_then(|v| v.as_bool()).unwrap_or(false));

    // Wait for command to execute
    thread::sleep(Duration::from_millis(500));
}

/// Test: Change theme
#[test]
#[ignore = "Requires GUI with --test-server flag"]
fn test_set_theme() {
    let mut client = TestClient::connect().expect("Failed to connect");

    // Change to light theme
    let result = client
        .request("set_theme", Some(json!({"name": "Ghostty Light"})))
        .expect("set_theme failed");
    assert!(result.get("success").and_then(|v| v.as_bool()).unwrap_or(false));

    // Verify theme changed
    let state = client.request("get_state", None).expect("get_state failed");
    let theme_name = state.get("theme_name").and_then(|v| v.as_str());
    assert_eq!(theme_name, Some("Ghostty Light"));

    // Restore dark theme
    let _ = client.request("set_theme", Some(json!({"name": "Ghostty Dark"})));
}

/// Test: Assertions
#[test]
#[ignore = "Requires GUI with --test-server flag"]
fn test_assertions() {
    let mut client = TestClient::connect().expect("Failed to connect");

    // Get current tab count
    let state = client.request("get_state", None).expect("get_state failed");
    let tab_count = state
        .get("tabs")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    // Assert tab count (should pass)
    client
        .request("assert_tab_count", Some(json!({"count": tab_count})))
        .expect("assert_tab_count should pass");

    // Assert wrong tab count (should fail)
    let result = client.request("assert_tab_count", Some(json!({"count": 999})));
    assert!(result.is_err());
}

// ============================================================================
// Full Integration Test (spawns GUI)
// ============================================================================

/// Full integration test that spawns the GUI
///
/// This test is ignored by default because it requires a display.
/// Run with: cargo test --test ui_automation test_full_integration -- --ignored
#[test]
#[ignore = "Requires display and spawns GUI process"]
fn test_full_integration() {
    // Spawn GUI with test server
    let _gui = GuiProcess::spawn().expect("Failed to spawn GUI");

    // Wait for GUI to initialize
    thread::sleep(Duration::from_secs(3));

    // Connect to test server
    let mut client = TestClient::connect().expect("Failed to connect to test server");

    // Run test sequence
    client.request("ping", None).expect("Ping failed");

    let state = client.request("get_state", None).expect("get_state failed");
    println!("Initial state: {}", serde_json::to_string_pretty(&state).unwrap());

    // Create tab
    client.request("create_tab", None).expect("create_tab failed");

    // Switch tabs
    client
        .request("switch_tab", Some(json!({"index": 0})))
        .expect("switch_tab failed");

    // Send keys
    client
        .request("send_keys", Some(json!({"keys": "ls\n"})))
        .expect("send_keys failed");

    thread::sleep(Duration::from_secs(1));

    // Final state
    let state = client.request("get_state", None).expect("get_state failed");
    println!("Final state: {}", serde_json::to_string_pretty(&state).unwrap());
}
