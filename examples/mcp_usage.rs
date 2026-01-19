//! MCP Client Usage Example
//!
//! This example demonstrates how to use the MCP client modules in AgTerm.
//!
//! To run this example:
//! ```bash
//! cargo run --example mcp_usage
//! ```

use agterm::mcp::{
    McpClient, McpConfig, ServerConfig, ServerProfile, ServerType,
    TransportConfig, RetryConfig, ToolCall,
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AgTerm MCP Client Example ===\n");

    // Example 1: Load configuration from default location
    println!("1. Loading MCP configuration...");
    match McpConfig::load_or_default() {
        Ok(config) => {
            println!("   ✓ Loaded configuration");
            println!("   - Number of servers: {}", config.servers.len());
            println!("   - Default server: {:?}", config.default_server);
            println!("   - Offline fallback: {}", config.settings.offline_fallback);
        }
        Err(e) => {
            println!("   ✗ Failed to load config: {}", e);
        }
    }

    // Example 2: Create a custom configuration
    println!("\n2. Creating custom configuration...");
    let mut config = McpConfig::default();

    // Add a custom HTTP server
    let custom_profile = ServerProfile {
        name: "ollama".to_string(),
        description: "Local Ollama LLM Server".to_string(),
        config: ServerConfig {
            name: "ollama".to_string(),
            server_type: ServerType::LocalLLM,
            transport: TransportConfig::Http {
                url: "http://localhost:11434".to_string(),
                headers: HashMap::new(),
            },
            timeout_ms: 60_000,
            retry: RetryConfig::default(),
            metadata: HashMap::new(),
        },
        enabled: true,
        auto_connect: false,
    };

    config.upsert_server(custom_profile);
    println!("   ✓ Added custom server: ollama");

    // Example 3: Save configuration
    println!("\n3. Saving configuration...");
    // Uncomment to actually save:
    // match config.save_to_file("./mcp_config.toml") {
    //     Ok(_) => println!("   ✓ Configuration saved to ./mcp_config.toml"),
    //     Err(e) => println!("   ✗ Failed to save: {}", e),
    // }
    println!("   (Skipped for example)");

    // Example 4: Create and connect a client
    println!("\n4. Creating MCP client...");
    let profile = ServerProfile {
        name: "example".to_string(),
        description: "Example MCP Server".to_string(),
        config: ServerConfig {
            name: "example".to_string(),
            server_type: ServerType::Custom,
            transport: TransportConfig::Http {
                url: "http://localhost:8080".to_string(),
                headers: HashMap::new(),
            },
            timeout_ms: 30_000,
            retry: RetryConfig::default(),
            metadata: HashMap::new(),
        },
        enabled: true,
        auto_connect: false,
    };

    let mut client = McpClient::new(&profile);
    println!("   ✓ Client created");

    // Example 5: Connect to server (placeholder implementation)
    println!("\n5. Connecting to server...");
    match client.connect().await {
        Ok(_) => {
            println!("   ✓ Connected successfully");
            println!("   - Connection status: {}", if client.is_connected() { "Connected" } else { "Disconnected" });
        }
        Err(e) => {
            println!("   ✗ Connection failed: {}", e);
            return Ok(());
        }
    }

    // Example 6: Discover server capabilities
    println!("\n6. Discovering server capabilities...");
    match client.discover_capabilities().await {
        Ok(capabilities) => {
            println!("   ✓ Capabilities discovered");
            println!("   - Tools: {}", capabilities.tools.len());
            println!("   - Prompts: {}", capabilities.prompts.len());
            println!("   - Resources: {}", capabilities.resources.len());

            // List tools
            if !capabilities.tools.is_empty() {
                println!("\n   Available tools:");
                for tool in &capabilities.tools {
                    println!("     - {} : {}", tool.name, tool.description);
                }
            }
        }
        Err(e) => {
            println!("   ✗ Discovery failed: {}", e);
        }
    }

    // Example 7: Send a message
    println!("\n7. Sending message...");
    match client.send_message("Hello, how can you help?").await {
        Ok(response) => {
            println!("   ✓ Message sent successfully");
            if let Some(result) = response.result {
                println!("   - Response: {:?}", result);
            }
        }
        Err(e) => {
            println!("   ✗ Failed to send message: {}", e);
        }
    }

    // Example 8: Call a tool
    println!("\n8. Calling a tool...");
    let tool_call = ToolCall {
        name: "execute_command".to_string(),
        arguments: serde_json::json!({
            "command": "echo 'Hello from MCP!'"
        }),
    };

    match client.call_tool(&tool_call).await {
        Ok(result) => {
            println!("   ✓ Tool called successfully");
            println!("   - Error: {}", result.is_error);
            println!("   - Content items: {}", result.content.len());
        }
        Err(e) => {
            println!("   ✗ Tool call failed: {}", e);
        }
    }

    // Example 9: Disconnect
    println!("\n9. Disconnecting...");
    match client.disconnect().await {
        Ok(_) => {
            println!("   ✓ Disconnected successfully");
        }
        Err(e) => {
            println!("   ✗ Disconnect failed: {}", e);
        }
    }

    println!("\n=== Example Complete ===");
    Ok(())
}
