//! Multi-Agent Manager Module
//!
//! This module manages multiple AI agent connections for AgTerm, supporting
//! concurrent agent sessions with a primary agent designation.
//!
//! # Overview
//!
//! The Multi-Agent Manager enables:
//! - Connection management for multiple AI agents
//! - Primary agent designation for default operations
//! - Activity tracking and connection limits
//! - Agent chaining for sequential operations (future)
//!
//! # Key Concepts
//!
//! ## AgentConnection
//! Represents a single agent's connection state with metadata such as:
//! - Agent ID for identification
//! - Connection timestamp
//! - Tool count (number of available tools)
//! - Last activity timestamp (reactive)
//!
//! ## MultiAgentManager
//! Central coordinator for all agent connections, enforcing:
//! - Maximum concurrent connection limits (default: 5)
//! - Primary agent tracking
//! - Connection lifecycle management
//!
//! ## AgentChain (Future)
//! Enables sequential agent operations where:
//! - Output from one agent feeds into the next
//! - Terminal input can be injected at any step
//! - Complex workflows can be orchestrated
//!
//! # Example Usage
//!
//! ```ignore
//! use agterm::floem_app::multi_agent::MultiAgentManager;
//!
//! let manager = MultiAgentManager::new();
//!
//! // Connect agents
//! manager.add_connection("agent-1".to_string(), 10);
//! manager.add_connection("agent-2".to_string(), 5);
//!
//! // Set primary
//! manager.set_primary("agent-1".to_string());
//!
//! // Check status
//! if manager.is_connected("agent-1") {
//!     println!("Agent 1 is connected");
//! }
//!
//! // Update activity
//! manager.update_activity("agent-1");
//!
//! // Cleanup
//! manager.disconnect_all();
//! ```

use std::collections::HashMap;
use floem::reactive::{RwSignal, SignalGet, SignalUpdate, SignalWith};
use chrono::{DateTime, Utc};

/// Represents a connected AI agent with metadata
///
/// Each agent connection tracks:
/// - Unique identifier
/// - Connection timestamp (UTC)
/// - Number of available tools
/// - Last activity timestamp (reactive for UI updates)
#[derive(Clone, Debug)]
pub struct AgentConnection {
    /// Unique identifier for the agent
    pub agent_id: String,

    /// Timestamp when the agent connected (UTC)
    pub connected_at: DateTime<Utc>,

    /// Number of tools available to this agent
    pub tools_count: usize,

    /// Last activity timestamp (reactive for UI updates)
    pub last_activity: RwSignal<DateTime<Utc>>,
}

impl AgentConnection {
    /// Create a new agent connection
    pub fn new(agent_id: String, tools_count: usize) -> Self {
        let now = Utc::now();
        Self {
            agent_id,
            connected_at: now,
            tools_count,
            last_activity: RwSignal::new(now),
        }
    }
}

/// Central manager for multiple AI agent connections
///
/// Manages the lifecycle and coordination of multiple agent connections,
/// enforcing connection limits and tracking the primary agent.
///
/// # Constraints
/// - Maximum concurrent connections (default: 5)
/// - Primary agent must be from connected agents
/// - Connection IDs must be unique
#[derive(Clone)]
pub struct MultiAgentManager {
    /// Map of agent_id -> AgentConnection
    pub connections: RwSignal<HashMap<String, AgentConnection>>,

    /// Currently designated primary agent (None if no agents connected)
    pub primary_agent: RwSignal<Option<String>>,

    /// Maximum allowed concurrent connections
    pub max_connections: usize,
}

impl MultiAgentManager {
    /// Create a new MultiAgentManager with default settings
    ///
    /// Default configuration:
    /// - max_connections: 5
    /// - No connections
    /// - No primary agent
    pub fn new() -> Self {
        Self::with_max_connections(5)
    }

    /// Create a new MultiAgentManager with custom max connections
    pub fn with_max_connections(max_connections: usize) -> Self {
        tracing::info!("Initializing MultiAgentManager with max_connections={}", max_connections);
        Self {
            connections: RwSignal::new(HashMap::new()),
            primary_agent: RwSignal::new(None),
            max_connections,
        }
    }

    /// Add a new agent connection
    ///
    /// # Arguments
    /// * `agent_id` - Unique identifier for the agent
    /// * `tools_count` - Number of tools available to the agent
    ///
    /// # Returns
    /// * `Ok(())` - Connection added successfully
    /// * `Err(String)` - Error message if connection failed
    ///
    /// # Errors
    /// - Connection limit reached
    /// - Agent ID already exists
    pub fn add_connection(&self, agent_id: String, tools_count: usize) -> Result<(), String> {
        // Check if already connected
        if self.connections.with(|conns| conns.contains_key(&agent_id)) {
            tracing::warn!("Agent '{}' is already connected", agent_id);
            return Err(format!("Agent '{}' is already connected", agent_id));
        }

        // Check connection limit
        if self.connections.with(|conns| conns.len()) >= self.max_connections {
            let count = self.connections.with(|conns| conns.len());
            tracing::warn!(
                "Connection limit reached ({}/{})",
                count,
                self.max_connections
            );
            return Err(format!(
                "Maximum connections ({}) reached",
                self.max_connections
            ));
        }

        // Add new connection
        let connection = AgentConnection::new(agent_id.clone(), tools_count);
        self.connections.update(|conns| {
            conns.insert(agent_id.clone(), connection);
        });

        let conn_count = self.connections.with(|conns| conns.len());
        tracing::info!(
            "Agent '{}' connected with {} tools ({}/{})",
            agent_id,
            tools_count,
            conn_count,
            self.max_connections
        );

        // If this is the first connection, set as primary
        if conn_count == 1 {
            self.set_primary(agent_id.clone())?;
            tracing::info!("Agent '{}' automatically set as primary (first connection)", agent_id);
        }

        Ok(())
    }

    /// Remove an agent connection
    ///
    /// # Arguments
    /// * `agent_id` - Identifier of the agent to disconnect
    ///
    /// # Returns
    /// * `true` - Agent was connected and removed
    /// * `false` - Agent was not connected
    ///
    /// # Side Effects
    /// - If removed agent was primary, primary is cleared
    /// - If other agents remain, the first one becomes primary
    pub fn remove_connection(&self, agent_id: &str) -> bool {
        // Check if agent exists
        let exists = self.connections.with(|conns| conns.contains_key(agent_id));
        if !exists {
            tracing::debug!("Attempted to disconnect non-existent agent '{}'", agent_id);
            return false;
        }

        // Remove the connection
        self.connections.update(|conns| {
            conns.remove(agent_id);
        });

        let remaining_count = self.connections.with(|conns| conns.len());
        tracing::info!("Agent '{}' disconnected ({} remaining)", agent_id, remaining_count);

        // Clear primary if this was the primary agent
        let was_primary = self.primary_agent.with(|p| {
            p.as_ref().map(|id| id == agent_id).unwrap_or(false)
        });

        if was_primary {
            // Try to set a new primary from remaining connections
            let remaining_agents = self.get_connected_agents();
            if let Some(new_primary) = remaining_agents.first() {
                let _ = self.set_primary(new_primary.clone());
                tracing::info!("Agent '{}' promoted to primary after '{}' disconnected", new_primary, agent_id);
            } else {
                self.primary_agent.set(None);
                tracing::info!("No primary agent (all agents disconnected)");
            }
        }

        true
    }

    /// Set the primary agent
    ///
    /// # Arguments
    /// * `agent_id` - Identifier of the agent to set as primary
    ///
    /// # Returns
    /// * `Ok(())` - Primary agent set successfully
    /// * `Err(String)` - Error message if agent is not connected
    ///
    /// # Errors
    /// - Agent ID is not in connected agents
    pub fn set_primary(&self, agent_id: String) -> Result<(), String> {
        // Verify agent is connected
        if !self.is_connected(&agent_id) {
            tracing::warn!("Cannot set primary: agent '{}' is not connected", agent_id);
            return Err(format!("Agent '{}' is not connected", agent_id));
        }

        self.primary_agent.set(Some(agent_id.clone()));
        tracing::info!("Agent '{}' set as primary", agent_id);
        Ok(())
    }

    /// Get the current primary agent ID
    ///
    /// # Returns
    /// * `Some(String)` - ID of the primary agent
    /// * `None` - No primary agent set (no agents connected)
    pub fn get_primary(&self) -> Option<String> {
        self.primary_agent.get()
    }

    /// Check if an agent is currently connected
    ///
    /// # Arguments
    /// * `agent_id` - Identifier of the agent to check
    ///
    /// # Returns
    /// * `true` - Agent is connected
    /// * `false` - Agent is not connected
    pub fn is_connected(&self, agent_id: &str) -> bool {
        self.connections.with(|conns| conns.contains_key(agent_id))
    }

    /// Get list of all connected agent IDs
    ///
    /// # Returns
    /// Vector of agent IDs in arbitrary order
    pub fn get_connected_agents(&self) -> Vec<String> {
        self.connections.with(|conns| {
            conns.keys().cloned().collect()
        })
    }

    /// Update the last activity timestamp for an agent
    ///
    /// # Arguments
    /// * `agent_id` - Identifier of the agent to update
    ///
    /// # Returns
    /// * `true` - Activity timestamp updated
    /// * `false` - Agent not found
    pub fn update_activity(&self, agent_id: &str) -> bool {
        self.connections.with(|conns| {
            if let Some(conn) = conns.get(agent_id) {
                conn.last_activity.set(Utc::now());
                tracing::trace!("Updated activity timestamp for agent '{}'", agent_id);
                true
            } else {
                tracing::debug!("Cannot update activity: agent '{}' not found", agent_id);
                false
            }
        })
    }

    /// Disconnect all agents
    ///
    /// Clears all connections and resets primary agent.
    /// Use this for application shutdown or full reset.
    pub fn disconnect_all(&self) {
        let count = self.connections.with(|conns| conns.len());
        self.connections.update(|conns| conns.clear());
        self.primary_agent.set(None);
        tracing::info!("Disconnected all agents (count: {})", count);
    }

    /// Get connection count
    ///
    /// # Returns
    /// Number of currently connected agents
    pub fn connection_count(&self) -> usize {
        self.connections.with(|conns| conns.len())
    }

    /// Check if connection limit is reached
    ///
    /// # Returns
    /// * `true` - At maximum capacity
    /// * `false` - Can accept more connections
    pub fn is_full(&self) -> bool {
        self.connection_count() >= self.max_connections
    }

    /// Get connection details for a specific agent
    ///
    /// # Arguments
    /// * `agent_id` - Identifier of the agent
    ///
    /// # Returns
    /// * `Some(AgentConnection)` - Connection details
    /// * `None` - Agent not found
    pub fn get_connection(&self, agent_id: &str) -> Option<AgentConnection> {
        self.connections.with(|conns| conns.get(agent_id).cloned())
    }
}

impl Default for MultiAgentManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Agent Chain - Future Feature
// ============================================================================

/// Input source for a chain step
///
/// Defines where the input data comes from for this step in the chain.
#[derive(Clone, Debug)]
pub enum ChainInput {
    /// Static JSON value provided at chain definition time
    Static(serde_json::Value),

    /// Output from the previous step in the chain
    FromPrevious,

    /// Live input from the terminal
    FromTerminal,
}

/// A single step in an agent chain
///
/// Represents one operation in a sequential agent workflow.
#[derive(Clone, Debug)]
pub struct ChainStep {
    /// Agent to execute this step
    pub agent_id: String,

    /// Tool to invoke on the agent
    pub tool_name: String,

    /// Input source for this step
    pub input: ChainInput,
}

impl ChainStep {
    /// Create a new chain step with static input
    pub fn with_static(agent_id: String, tool_name: String, input: serde_json::Value) -> Self {
        Self {
            agent_id,
            tool_name,
            input: ChainInput::Static(input),
        }
    }

    /// Create a new chain step with input from previous step
    pub fn with_previous(agent_id: String, tool_name: String) -> Self {
        Self {
            agent_id,
            tool_name,
            input: ChainInput::FromPrevious,
        }
    }

    /// Create a new chain step with terminal input
    pub fn with_terminal(agent_id: String, tool_name: String) -> Self {
        Self {
            agent_id,
            tool_name,
            input: ChainInput::FromTerminal,
        }
    }
}

/// Agent chain for sequential operations
///
/// Enables orchestrating multiple agents in a pipeline where
/// the output of one step feeds into the next.
///
/// # Example
/// ```ignore
/// let chain = AgentChain::new()
///     .add_step(ChainStep::with_terminal("analyzer".to_string(), "analyze".to_string()))
///     .add_step(ChainStep::with_previous("formatter".to_string(), "format".to_string()))
///     .add_step(ChainStep::with_previous("validator".to_string(), "validate".to_string()));
/// ```
#[derive(Clone, Debug, Default)]
pub struct AgentChain {
    /// Ordered list of steps to execute
    pub steps: Vec<ChainStep>,
}

impl AgentChain {
    /// Create a new empty agent chain
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
        }
    }

    /// Add a step to the chain
    pub fn add_step(mut self, step: ChainStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Get the number of steps in the chain
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Check if the chain is empty
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Validate the chain
    ///
    /// # Arguments
    /// * `manager` - MultiAgentManager to verify agent IDs
    ///
    /// # Returns
    /// * `Ok(())` - Chain is valid
    /// * `Err(String)` - Validation error message
    ///
    /// # Validation Rules
    /// - All agent IDs must exist in connected agents
    /// - FromPrevious input cannot be used in first step
    pub fn validate(&self, manager: &MultiAgentManager) -> Result<(), String> {
        if self.steps.is_empty() {
            return Err("Chain cannot be empty".to_string());
        }

        for (idx, step) in self.steps.iter().enumerate() {
            // Check if agent is connected
            if !manager.is_connected(&step.agent_id) {
                return Err(format!(
                    "Step {}: Agent '{}' is not connected",
                    idx, step.agent_id
                ));
            }

            // First step cannot use FromPrevious
            if idx == 0 && matches!(step.input, ChainInput::FromPrevious) {
                return Err("First step cannot use FromPrevious input".to_string());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = MultiAgentManager::new();
        assert_eq!(manager.max_connections, 5);
        assert_eq!(manager.connection_count(), 0);
        assert!(manager.get_primary().is_none());
    }

    #[test]
    fn test_add_connection() {
        let manager = MultiAgentManager::new();

        // Add first connection
        assert!(manager.add_connection("agent-1".to_string(), 10).is_ok());
        assert_eq!(manager.connection_count(), 1);
        assert!(manager.is_connected("agent-1"));

        // First agent should be primary
        assert_eq!(manager.get_primary(), Some("agent-1".to_string()));
    }

    #[test]
    fn test_duplicate_connection() {
        let manager = MultiAgentManager::new();

        manager.add_connection("agent-1".to_string(), 10).unwrap();
        let result = manager.add_connection("agent-1".to_string(), 5);

        assert!(result.is_err());
        assert_eq!(manager.connection_count(), 1);
    }

    #[test]
    fn test_connection_limit() {
        let manager = MultiAgentManager::with_max_connections(2);

        assert!(manager.add_connection("agent-1".to_string(), 10).is_ok());
        assert!(manager.add_connection("agent-2".to_string(), 5).is_ok());
        assert!(manager.is_full());

        let result = manager.add_connection("agent-3".to_string(), 3);
        assert!(result.is_err());
        assert_eq!(manager.connection_count(), 2);
    }

    #[test]
    fn test_remove_connection() {
        let manager = MultiAgentManager::new();

        manager.add_connection("agent-1".to_string(), 10).unwrap();
        manager.add_connection("agent-2".to_string(), 5).unwrap();

        assert!(manager.remove_connection("agent-1"));
        assert_eq!(manager.connection_count(), 1);
        assert!(!manager.is_connected("agent-1"));
        assert!(manager.is_connected("agent-2"));
    }

    #[test]
    fn test_primary_agent_management() {
        let manager = MultiAgentManager::new();

        manager.add_connection("agent-1".to_string(), 10).unwrap();
        manager.add_connection("agent-2".to_string(), 5).unwrap();

        // First agent is primary
        assert_eq!(manager.get_primary(), Some("agent-1".to_string()));

        // Change primary
        assert!(manager.set_primary("agent-2".to_string()).is_ok());
        assert_eq!(manager.get_primary(), Some("agent-2".to_string()));

        // Cannot set non-existent agent as primary
        let result = manager.set_primary("agent-3".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_primary_promotion_on_disconnect() {
        let manager = MultiAgentManager::new();

        manager.add_connection("agent-1".to_string(), 10).unwrap();
        manager.add_connection("agent-2".to_string(), 5).unwrap();

        // agent-1 is primary
        assert_eq!(manager.get_primary(), Some("agent-1".to_string()));

        // Remove primary agent
        manager.remove_connection("agent-1");

        // agent-2 should be promoted
        assert_eq!(manager.get_primary(), Some("agent-2".to_string()));
    }

    #[test]
    fn test_disconnect_all() {
        let manager = MultiAgentManager::new();

        manager.add_connection("agent-1".to_string(), 10).unwrap();
        manager.add_connection("agent-2".to_string(), 5).unwrap();

        manager.disconnect_all();

        assert_eq!(manager.connection_count(), 0);
        assert!(manager.get_primary().is_none());
    }

    #[test]
    fn test_update_activity() {
        let manager = MultiAgentManager::new();

        manager.add_connection("agent-1".to_string(), 10).unwrap();

        let conn1 = manager.get_connection("agent-1").unwrap();
        let initial_activity = conn1.last_activity.get();

        std::thread::sleep(std::time::Duration::from_millis(10));

        assert!(manager.update_activity("agent-1"));

        let conn2 = manager.get_connection("agent-1").unwrap();
        let updated_activity = conn2.last_activity.get();

        assert!(updated_activity > initial_activity);
    }

    #[test]
    fn test_chain_creation() {
        let chain = AgentChain::new()
            .add_step(ChainStep::with_terminal("agent-1".to_string(), "read".to_string()))
            .add_step(ChainStep::with_previous("agent-2".to_string(), "process".to_string()));

        assert_eq!(chain.len(), 2);
        assert!(!chain.is_empty());
    }

    #[test]
    fn test_chain_validation() {
        let manager = MultiAgentManager::new();
        manager.add_connection("agent-1".to_string(), 10).unwrap();
        manager.add_connection("agent-2".to_string(), 5).unwrap();

        // Valid chain
        let valid_chain = AgentChain::new()
            .add_step(ChainStep::with_terminal("agent-1".to_string(), "read".to_string()))
            .add_step(ChainStep::with_previous("agent-2".to_string(), "process".to_string()));

        assert!(valid_chain.validate(&manager).is_ok());

        // Invalid: FromPrevious in first step
        let invalid_chain = AgentChain::new()
            .add_step(ChainStep::with_previous("agent-1".to_string(), "read".to_string()));

        assert!(invalid_chain.validate(&manager).is_err());

        // Invalid: Non-existent agent
        let invalid_chain2 = AgentChain::new()
            .add_step(ChainStep::with_terminal("agent-3".to_string(), "read".to_string()));

        assert!(invalid_chain2.validate(&manager).is_err());
    }
}
