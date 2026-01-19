//! Agent Registry Module
//!
//! Manages MCP agent configurations and connection states.

use floem::peniko::Color;
use floem::prelude::*;
use std::collections::HashMap;

/// Configuration for an MCP agent
#[derive(Debug, Clone, PartialEq)]
pub struct AgentConfig {
    /// Unique identifier for the agent
    pub id: String,
    /// Display name
    pub name: String,
    /// Command to execute
    pub command: String,
    /// Command arguments
    pub args: Vec<String>,
    /// Optional icon identifier
    pub icon: Option<String>,
    /// Agent brand color
    pub color: Color,
    /// List of agent capabilities
    pub capabilities: Vec<String>,
}

/// Connection state for an agent
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    /// Agent is not connected
    Disconnected,
    /// Agent is in the process of connecting
    Connecting,
    /// Agent is successfully connected
    Connected,
    /// Agent encountered an error
    Error(String),
}

impl ConnectionState {
    /// Check if the agent is connected
    pub fn is_connected(&self) -> bool {
        matches!(self, ConnectionState::Connected)
    }

    /// Check if the agent is in an error state
    pub fn is_error(&self) -> bool {
        matches!(self, ConnectionState::Error(_))
    }

    /// Get error message if in error state
    pub fn error_message(&self) -> Option<&str> {
        if let ConnectionState::Error(msg) = self {
            Some(msg)
        } else {
            None
        }
    }
}

/// Agent registry managing all available agents
#[derive(Clone)]
pub struct AgentRegistry {
    /// List of all configured agents
    pub agents: RwSignal<Vec<AgentConfig>>,
    /// Connection states for each agent (keyed by agent ID)
    pub active_connections: RwSignal<HashMap<String, ConnectionState>>,
}

impl AgentRegistry {
    /// Create a new agent registry with default agents
    pub fn new() -> Self {
        let default_agents = Self::load_defaults();
        let connections = HashMap::new();

        Self {
            agents: RwSignal::new(default_agents),
            active_connections: RwSignal::new(connections),
        }
    }

    /// Load default agent configurations
    pub fn load_defaults() -> Vec<AgentConfig> {
        vec![
            AgentConfig {
                id: "claude_code".to_string(),
                name: "Claude Code".to_string(),
                command: "claude".to_string(),
                args: vec!["--mcp-server".to_string()],
                icon: Some("claude".to_string()),
                color: Color::rgb8(124, 58, 237), // #7C3AED (purple)
                capabilities: vec![
                    "code_editing".to_string(),
                    "file_operations".to_string(),
                    "terminal_access".to_string(),
                    "reasoning".to_string(),
                ],
            },
            AgentConfig {
                id: "gemini_cli".to_string(),
                name: "Gemini CLI".to_string(),
                command: "gemini".to_string(),
                args: vec!["mcp".to_string()],
                icon: Some("gemini".to_string()),
                color: Color::rgb8(66, 133, 244), // #4285F4 (blue)
                capabilities: vec![
                    "multimodal".to_string(),
                    "code_analysis".to_string(),
                    "vision".to_string(),
                ],
            },
            AgentConfig {
                id: "openai_codex".to_string(),
                name: "OpenAI Codex".to_string(),
                command: "openai".to_string(),
                args: vec!["--mcp".to_string()],
                icon: Some("openai".to_string()),
                color: Color::rgb8(16, 163, 127), // #10A37F (teal)
                capabilities: vec![
                    "code_generation".to_string(),
                    "code_completion".to_string(),
                    "documentation".to_string(),
                ],
            },
            AgentConfig {
                id: "qwen_code".to_string(),
                name: "Qwen Code".to_string(),
                command: "qwen".to_string(),
                args: vec!["mcp".to_string()],
                icon: Some("qwen".to_string()),
                color: Color::rgb8(99, 102, 241), // #6366F1 (indigo)
                capabilities: vec![
                    "code_understanding".to_string(),
                    "multilingual".to_string(),
                    "fast_inference".to_string(),
                ],
            },
        ]
    }

    /// Add a custom agent to the registry
    pub fn add_custom(&self, config: AgentConfig) {
        let config_id = config.id.clone();

        self.agents.update(|agents| {
            // Check if agent with same ID already exists
            if let Some(pos) = agents.iter().position(|a| a.id == config.id) {
                // Replace existing agent
                agents[pos] = config;
            } else {
                // Add new agent
                agents.push(config);
            }
        });

        // Initialize connection state to Disconnected
        self.active_connections.update(|connections| {
            connections.entry(config_id)
                .or_insert(ConnectionState::Disconnected);
        });
    }

    /// Get agent configuration by ID
    pub fn get_agent(&self, id: &str) -> Option<AgentConfig> {
        self.agents.with(|agents| {
            agents.iter().find(|a| a.id == id).cloned()
        })
    }

    /// Get connection state for an agent
    pub fn get_connection_state(&self, id: &str) -> ConnectionState {
        self.active_connections.with(|connections| {
            connections.get(id)
                .cloned()
                .unwrap_or(ConnectionState::Disconnected)
        })
    }

    /// Set connection state for an agent
    pub fn set_connection_state(&self, id: &str, state: ConnectionState) {
        self.active_connections.update(|connections| {
            connections.insert(id.to_string(), state);
        });
    }

    /// Remove an agent from the registry
    pub fn remove_agent(&self, id: &str) -> bool {
        let mut removed = false;

        self.agents.update(|agents| {
            if let Some(pos) = agents.iter().position(|a| a.id == id) {
                agents.remove(pos);
                removed = true;
            }
        });

        if removed {
            self.active_connections.update(|connections| {
                connections.remove(id);
            });
        }

        removed
    }

    /// Get all agent IDs
    pub fn get_agent_ids(&self) -> Vec<String> {
        self.agents.with(|agents| {
            agents.iter().map(|a| a.id.clone()).collect()
        })
    }

    /// Check if an agent is connected
    pub fn is_connected(&self, id: &str) -> bool {
        self.get_connection_state(id).is_connected()
    }

    /// Get count of connected agents
    pub fn connected_count(&self) -> usize {
        self.active_connections.with(|connections| {
            connections.values()
                .filter(|state| state.is_connected())
                .count()
        })
    }

    /// Reset all connection states to Disconnected
    pub fn reset_connections(&self) {
        self.active_connections.update(|connections| {
            for state in connections.values_mut() {
                *state = ConnectionState::Disconnected;
            }
        });
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_defaults() {
        let agents = AgentRegistry::load_defaults();
        assert_eq!(agents.len(), 4);

        // Check Claude Code
        let claude = agents.iter().find(|a| a.id == "claude_code").unwrap();
        assert_eq!(claude.name, "Claude Code");
        assert_eq!(claude.command, "claude");
        assert_eq!(claude.args, vec!["--mcp-server"]);
        assert_eq!(claude.color, Color::rgb8(124, 58, 237));

        // Check Gemini CLI
        let gemini = agents.iter().find(|a| a.id == "gemini_cli").unwrap();
        assert_eq!(gemini.name, "Gemini CLI");
        assert_eq!(gemini.command, "gemini");
        assert_eq!(gemini.args, vec!["mcp"]);
        assert_eq!(gemini.color, Color::rgb8(66, 133, 244));

        // Check OpenAI Codex
        let openai = agents.iter().find(|a| a.id == "openai_codex").unwrap();
        assert_eq!(openai.name, "OpenAI Codex");
        assert_eq!(openai.command, "openai");
        assert_eq!(openai.args, vec!["--mcp"]);
        assert_eq!(openai.color, Color::rgb8(16, 163, 127));

        // Check Qwen Code
        let qwen = agents.iter().find(|a| a.id == "qwen_code").unwrap();
        assert_eq!(qwen.name, "Qwen Code");
        assert_eq!(qwen.command, "qwen");
        assert_eq!(qwen.args, vec!["mcp"]);
        assert_eq!(qwen.color, Color::rgb8(99, 102, 241));
    }

    #[test]
    fn test_registry_new() {
        let registry = AgentRegistry::new();
        let agents = registry.agents.get();
        assert_eq!(agents.len(), 4);

        let connections = registry.active_connections.get();
        assert!(connections.is_empty());
    }

    #[test]
    fn test_add_custom_agent() {
        let registry = AgentRegistry::new();

        let custom = AgentConfig {
            id: "custom_agent".to_string(),
            name: "Custom Agent".to_string(),
            command: "custom".to_string(),
            args: vec!["--flag".to_string()],
            icon: None,
            color: Color::rgb8(255, 0, 0),
            capabilities: vec!["custom".to_string()],
        };

        registry.add_custom(custom.clone());

        let agents = registry.agents.get();
        assert_eq!(agents.len(), 5);

        let found = registry.get_agent("custom_agent");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Custom Agent");

        // Check connection state initialized to Disconnected
        let state = registry.get_connection_state("custom_agent");
        assert_eq!(state, ConnectionState::Disconnected);
    }

    #[test]
    fn test_get_agent() {
        let registry = AgentRegistry::new();

        let claude = registry.get_agent("claude_code");
        assert!(claude.is_some());
        assert_eq!(claude.unwrap().name, "Claude Code");

        let nonexistent = registry.get_agent("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_connection_state() {
        let registry = AgentRegistry::new();

        // Initial state
        let state = registry.get_connection_state("claude_code");
        assert_eq!(state, ConnectionState::Disconnected);

        // Set connecting
        registry.set_connection_state("claude_code", ConnectionState::Connecting);
        let state = registry.get_connection_state("claude_code");
        assert_eq!(state, ConnectionState::Connecting);

        // Set connected
        registry.set_connection_state("claude_code", ConnectionState::Connected);
        let state = registry.get_connection_state("claude_code");
        assert_eq!(state, ConnectionState::Connected);
        assert!(registry.is_connected("claude_code"));

        // Set error
        registry.set_connection_state("claude_code", ConnectionState::Error("Test error".to_string()));
        let state = registry.get_connection_state("claude_code");
        assert!(state.is_error());
        assert_eq!(state.error_message(), Some("Test error"));
    }

    #[test]
    fn test_remove_agent() {
        let registry = AgentRegistry::new();

        // Add custom agent
        let custom = AgentConfig {
            id: "custom".to_string(),
            name: "Custom".to_string(),
            command: "custom".to_string(),
            args: vec![],
            icon: None,
            color: Color::rgb8(0, 0, 0),
            capabilities: vec![],
        };
        registry.add_custom(custom);

        // Set connection state
        registry.set_connection_state("custom", ConnectionState::Connected);

        // Remove agent
        let removed = registry.remove_agent("custom");
        assert!(removed);

        // Verify removed
        assert!(registry.get_agent("custom").is_none());
        assert_eq!(registry.get_connection_state("custom"), ConnectionState::Disconnected);

        // Try removing nonexistent
        let removed = registry.remove_agent("nonexistent");
        assert!(!removed);
    }

    #[test]
    fn test_get_agent_ids() {
        let registry = AgentRegistry::new();
        let ids = registry.get_agent_ids();
        assert_eq!(ids.len(), 4);
        assert!(ids.contains(&"claude_code".to_string()));
        assert!(ids.contains(&"gemini_cli".to_string()));
        assert!(ids.contains(&"openai_codex".to_string()));
        assert!(ids.contains(&"qwen_code".to_string()));
    }

    #[test]
    fn test_connected_count() {
        let registry = AgentRegistry::new();

        assert_eq!(registry.connected_count(), 0);

        registry.set_connection_state("claude_code", ConnectionState::Connected);
        assert_eq!(registry.connected_count(), 1);

        registry.set_connection_state("gemini_cli", ConnectionState::Connected);
        assert_eq!(registry.connected_count(), 2);

        registry.set_connection_state("claude_code", ConnectionState::Disconnected);
        assert_eq!(registry.connected_count(), 1);
    }

    #[test]
    fn test_reset_connections() {
        let registry = AgentRegistry::new();

        // Set some connection states
        registry.set_connection_state("claude_code", ConnectionState::Connected);
        registry.set_connection_state("gemini_cli", ConnectionState::Connecting);
        registry.set_connection_state("openai_codex", ConnectionState::Error("Test".to_string()));

        assert_eq!(registry.connected_count(), 1);

        // Reset all
        registry.reset_connections();

        assert_eq!(registry.connected_count(), 0);
        assert_eq!(registry.get_connection_state("claude_code"), ConnectionState::Disconnected);
        assert_eq!(registry.get_connection_state("gemini_cli"), ConnectionState::Disconnected);
        assert_eq!(registry.get_connection_state("openai_codex"), ConnectionState::Disconnected);
    }

    #[test]
    fn test_connection_state_methods() {
        let state = ConnectionState::Connected;
        assert!(state.is_connected());
        assert!(!state.is_error());
        assert_eq!(state.error_message(), None);

        let state = ConnectionState::Error("Test error".to_string());
        assert!(!state.is_connected());
        assert!(state.is_error());
        assert_eq!(state.error_message(), Some("Test error"));

        let state = ConnectionState::Disconnected;
        assert!(!state.is_connected());
        assert!(!state.is_error());
        assert_eq!(state.error_message(), None);
    }

    #[test]
    fn test_add_custom_replaces_existing() {
        let registry = AgentRegistry::new();

        // Add custom agent with same ID as existing
        let custom = AgentConfig {
            id: "claude_code".to_string(),
            name: "Modified Claude".to_string(),
            command: "claude-modified".to_string(),
            args: vec!["--new-flag".to_string()],
            icon: None,
            color: Color::rgb8(255, 255, 255),
            capabilities: vec!["modified".to_string()],
        };

        registry.add_custom(custom);

        // Should still have 4 agents
        assert_eq!(registry.agents.get().len(), 4);

        // Verify replacement
        let agent = registry.get_agent("claude_code").unwrap();
        assert_eq!(agent.name, "Modified Claude");
        assert_eq!(agent.command, "claude-modified");
    }

    #[test]
    fn test_agent_config_capabilities() {
        let agents = AgentRegistry::load_defaults();

        let claude = agents.iter().find(|a| a.id == "claude_code").unwrap();
        assert!(claude.capabilities.contains(&"code_editing".to_string()));
        assert!(claude.capabilities.contains(&"reasoning".to_string()));

        let gemini = agents.iter().find(|a| a.id == "gemini_cli").unwrap();
        assert!(gemini.capabilities.contains(&"multimodal".to_string()));
        assert!(gemini.capabilities.contains(&"vision".to_string()));
    }
}
