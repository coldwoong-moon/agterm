//! Offline fallback handler for MCP
//!
//! Provides local suggestions and caching when MCP server is unavailable.

use super::response::McpResponse;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Offline mode handler
#[derive(Debug, Clone)]
pub struct OfflineHandler {
    /// Response cache
    cache: HashMap<String, CachedResponse>,
    /// Whether offline mode is active
    offline_mode: bool,
}

/// Cached MCP response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResponse {
    /// The cached response
    pub response: McpResponse,
    /// When it was cached (Unix timestamp)
    pub timestamp: u64,
    /// Time-to-live in seconds
    pub ttl_seconds: u64,
}

impl OfflineHandler {
    /// Create a new offline handler
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            offline_mode: false,
        }
    }

    /// Check if currently in offline mode
    pub fn is_offline(&self) -> bool {
        self.offline_mode
    }

    /// Set offline mode
    pub fn set_offline(&mut self, offline: bool) {
        self.offline_mode = offline;
    }

    /// Cache a response
    pub fn cache_response(&mut self, query: String, response: McpResponse, ttl_seconds: u64) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.cache.insert(
            query,
            CachedResponse {
                response,
                timestamp,
                ttl_seconds,
            },
        );
    }

    /// Get cached response if available and not expired
    pub fn get_cached(&self, query: &str) -> Option<McpResponse> {
        let cached = self.cache.get(query)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check if expired (>= for immediate expiration when TTL is 0)
        if now - cached.timestamp >= cached.ttl_seconds {
            return None;
        }

        Some(cached.response.clone())
    }

    /// Clear expired cache entries
    pub fn cleanup_cache(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.cache.retain(|_, cached| {
            now - cached.timestamp < cached.ttl_seconds
        });
    }

    /// Get local suggestions based on keywords
    pub fn local_suggestions(&self, query: &str) -> Option<McpResponse> {
        let query_lower = query.to_lowercase();

        // Git-related queries
        if query_lower.contains("git") {
            return Some(self.git_suggestions(&query_lower));
        }

        // Cargo/Rust queries
        if query_lower.contains("cargo") || query_lower.contains("rust") {
            return Some(self.cargo_suggestions(&query_lower));
        }

        // npm/node queries
        if query_lower.contains("npm") || query_lower.contains("node") {
            return Some(self.npm_suggestions(&query_lower));
        }

        // File operations
        if query_lower.contains("file") || query_lower.contains("directory") {
            return Some(self.file_suggestions(&query_lower));
        }

        // Process management
        if query_lower.contains("process") || query_lower.contains("kill") {
            return Some(self.process_suggestions(&query_lower));
        }

        // Network operations
        if query_lower.contains("network") || query_lower.contains("port") {
            return Some(self.network_suggestions(&query_lower));
        }

        // Generic help
        None
    }

    fn git_suggestions(&self, query: &str) -> McpResponse {
        let mut response = if query.contains("status") {
            McpResponse::new("Check git repository status.".to_string())
        } else if query.contains("commit") {
            McpResponse::new("Commit changes to the repository.".to_string())
        } else if query.contains("push") {
            McpResponse::new("Push commits to remote repository.".to_string())
        } else if query.contains("pull") {
            McpResponse::new("Pull changes from remote repository.".to_string())
        } else if query.contains("branch") {
            McpResponse::new("Manage git branches.".to_string())
        } else if query.contains("log") {
            McpResponse::new("View commit history.".to_string())
        } else if query.contains("diff") {
            McpResponse::new("Show changes between commits.".to_string())
        } else {
            McpResponse::new("Git version control operations.".to_string())
        };

        response.set_metadata("source".to_string(), "offline".to_string());
        response.set_metadata("category".to_string(), "git".to_string());
        response
    }

    fn cargo_suggestions(&self, query: &str) -> McpResponse {
        let mut response = if query.contains("build") {
            McpResponse::new("Build Rust project with Cargo.".to_string())
        } else if query.contains("test") {
            McpResponse::new("Run tests for Rust project.".to_string())
        } else if query.contains("run") {
            McpResponse::new("Build and run Rust project.".to_string())
        } else if query.contains("check") {
            McpResponse::new("Check Rust project for errors without building.".to_string())
        } else if query.contains("clippy") {
            McpResponse::new("Run Clippy linter for Rust code.".to_string())
        } else if query.contains("fmt") || query.contains("format") {
            McpResponse::new("Format Rust code with rustfmt.".to_string())
        } else {
            McpResponse::new("Cargo package manager for Rust.".to_string())
        };

        response.set_metadata("source".to_string(), "offline".to_string());
        response.set_metadata("category".to_string(), "cargo".to_string());
        response
    }

    fn npm_suggestions(&self, query: &str) -> McpResponse {
        let mut response = if query.contains("install") {
            McpResponse::new("Install Node.js dependencies.".to_string())
        } else if query.contains("run") {
            McpResponse::new("Run npm script from package.json.".to_string())
        } else if query.contains("test") {
            McpResponse::new("Run tests for Node.js project.".to_string())
        } else if query.contains("build") {
            McpResponse::new("Build Node.js project.".to_string())
        } else if query.contains("start") {
            McpResponse::new("Start Node.js application.".to_string())
        } else {
            McpResponse::new("Node.js package manager.".to_string())
        };

        response.set_metadata("source".to_string(), "offline".to_string());
        response.set_metadata("category".to_string(), "npm".to_string());
        response
    }

    fn file_suggestions(&self, query: &str) -> McpResponse {
        let mut response = if query.contains("list") || query.contains("ls") {
            McpResponse::new("List files and directories.".to_string())
        } else if query.contains("create") || query.contains("mkdir") {
            McpResponse::new("Create new directory.".to_string())
        } else if query.contains("remove") || query.contains("delete") {
            McpResponse::new("Remove files or directories.".to_string())
        } else if query.contains("copy") || query.contains("cp") {
            McpResponse::new("Copy files or directories.".to_string())
        } else if query.contains("move") || query.contains("mv") {
            McpResponse::new("Move or rename files.".to_string())
        } else if query.contains("search") || query.contains("find") {
            McpResponse::new("Search for files.".to_string())
        } else {
            McpResponse::new("File system operations.".to_string())
        };

        response.set_metadata("source".to_string(), "offline".to_string());
        response.set_metadata("category".to_string(), "file".to_string());
        response
    }

    fn process_suggestions(&self, query: &str) -> McpResponse {
        let mut response = if query.contains("list") || query.contains("ps") {
            McpResponse::new("List running processes.".to_string())
        } else if query.contains("kill") {
            McpResponse::new("Terminate a process by PID.".to_string())
        } else if query.contains("top") {
            McpResponse::new("Display system resource usage.".to_string())
        } else {
            McpResponse::new("Process management operations.".to_string())
        };

        response.set_metadata("source".to_string(), "offline".to_string());
        response.set_metadata("category".to_string(), "process".to_string());
        response
    }

    fn network_suggestions(&self, query: &str) -> McpResponse {
        let mut response = if query.contains("port") {
            McpResponse::new("Check network ports and connections.".to_string())
        } else if query.contains("ping") {
            McpResponse::new("Test network connectivity.".to_string())
        } else if query.contains("curl") || query.contains("wget") {
            McpResponse::new("Download or fetch data from URL.".to_string())
        } else {
            McpResponse::new("Network operations.".to_string())
        };

        response.set_metadata("source".to_string(), "offline".to_string());
        response.set_metadata("category".to_string(), "network".to_string());
        response
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Clear all cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for OfflineHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CachedResponse {
    /// Check if cached response is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        now - self.timestamp > self.ttl_seconds
    }

    /// Get age of cached response in seconds
    pub fn age_seconds(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        now - self.timestamp
    }

    /// Get remaining TTL in seconds
    pub fn remaining_ttl(&self) -> u64 {
        let age = self.age_seconds();
        self.ttl_seconds.saturating_sub(age)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_offline_handler_creation() {
        let handler = OfflineHandler::new();

        assert!(!handler.is_offline());
        assert_eq!(handler.cache_size(), 0);
    }

    #[test]
    fn test_set_offline() {
        let mut handler = OfflineHandler::new();

        handler.set_offline(true);
        assert!(handler.is_offline());

        handler.set_offline(false);
        assert!(!handler.is_offline());
    }

    #[test]
    fn test_cache_response() {
        let mut handler = OfflineHandler::new();
        let response = McpResponse::new("test response".to_string());

        handler.cache_response("test query".to_string(), response, 3600);

        assert_eq!(handler.cache_size(), 1);

        let cached = handler.get_cached("test query");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().content, "test response");
    }

    #[test]
    fn test_cache_expiration() {
        let mut handler = OfflineHandler::new();
        let response = McpResponse::new("test".to_string());

        // Cache with 1 second TTL
        handler.cache_response("query".to_string(), response, 1);

        // Should return None after expiration
        std::thread::sleep(Duration::from_secs(2));
        let cached = handler.get_cached("query");
        assert!(cached.is_none());
    }

    #[test]
    fn test_cache_cleanup() {
        let mut handler = OfflineHandler::new();

        // Add some entries with 1 second TTL
        for i in 0..5 {
            let response = McpResponse::new(format!("response {}", i));
            handler.cache_response(format!("query{}", i), response, 1);
        }

        assert_eq!(handler.cache_size(), 5);

        std::thread::sleep(Duration::from_secs(2));
        handler.cleanup_cache();

        // All should be cleaned up
        assert_eq!(handler.cache_size(), 0);
    }

    #[test]
    fn test_clear_cache() {
        let mut handler = OfflineHandler::new();

        for i in 0..10 {
            let response = McpResponse::new(format!("response {}", i));
            handler.cache_response(format!("query{}", i), response, 3600);
        }

        assert_eq!(handler.cache_size(), 10);

        handler.clear_cache();
        assert_eq!(handler.cache_size(), 0);
    }

    #[test]
    fn test_git_suggestions() {
        let handler = OfflineHandler::new();

        let status_response = handler.local_suggestions("git status");
        assert!(status_response.is_some());
        let resp = status_response.unwrap();
        assert!(resp.content.contains("status"));
        assert_eq!(resp.get_metadata("category"), Some(&"git".to_string()));

        let commit_response = handler.local_suggestions("git commit");
        assert!(commit_response.is_some());
        assert!(commit_response.unwrap().content.contains("Commit"));
    }

    #[test]
    fn test_cargo_suggestions() {
        let handler = OfflineHandler::new();

        let build_response = handler.local_suggestions("cargo build");
        assert!(build_response.is_some());
        let resp = build_response.unwrap();
        assert!(resp.content.contains("Build"));
        assert_eq!(resp.get_metadata("category"), Some(&"cargo".to_string()));

        let test_response = handler.local_suggestions("cargo test");
        assert!(test_response.is_some());
        assert!(test_response.unwrap().content.contains("test"));
    }

    #[test]
    fn test_npm_suggestions() {
        let handler = OfflineHandler::new();

        let install_response = handler.local_suggestions("npm install");
        assert!(install_response.is_some());
        let resp = install_response.unwrap();
        assert!(resp.content.contains("Install"));
        assert_eq!(resp.get_metadata("category"), Some(&"npm".to_string()));
    }

    #[test]
    fn test_file_suggestions() {
        let handler = OfflineHandler::new();

        let list_response = handler.local_suggestions("list files");
        assert!(list_response.is_some());
        let resp = list_response.unwrap();
        assert!(resp.content.contains("List"));
        assert_eq!(resp.get_metadata("category"), Some(&"file".to_string()));
    }

    #[test]
    fn test_process_suggestions() {
        let handler = OfflineHandler::new();

        let kill_response = handler.local_suggestions("kill process");
        assert!(kill_response.is_some());
        let resp = kill_response.unwrap();
        assert!(resp.content.contains("process"));
        assert_eq!(resp.get_metadata("category"), Some(&"process".to_string()));
    }

    #[test]
    fn test_network_suggestions() {
        let handler = OfflineHandler::new();

        let port_response = handler.local_suggestions("check port");
        assert!(port_response.is_some());
        let resp = port_response.unwrap();
        assert!(resp.content.contains("port"));
        assert_eq!(resp.get_metadata("category"), Some(&"network".to_string()));
    }

    #[test]
    fn test_no_suggestions() {
        let handler = OfflineHandler::new();

        let response = handler.local_suggestions("random unknown query");
        assert!(response.is_none());
    }

    #[test]
    fn test_cached_response_expiration() {
        let response = McpResponse::new("test".to_string());
        let cached = CachedResponse {
            response,
            timestamp: 1000,
            ttl_seconds: 100,
        };

        // With current time much later, should be expired
        assert!(cached.is_expired());
    }

    #[test]
    fn test_cached_response_age() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let response = McpResponse::new("test".to_string());
        let cached = CachedResponse {
            response,
            timestamp: now - 50,
            ttl_seconds: 100,
        };

        let age = cached.age_seconds();
        assert!(age >= 50 && age <= 52); // Allow small variance
    }

    #[test]
    fn test_cached_response_remaining_ttl() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let response = McpResponse::new("test".to_string());
        let cached = CachedResponse {
            response,
            timestamp: now - 30,
            ttl_seconds: 100,
        };

        let remaining = cached.remaining_ttl();
        assert!(remaining >= 68 && remaining <= 71); // ~70 seconds remaining
    }

    #[test]
    fn test_cache_with_different_queries() {
        let mut handler = OfflineHandler::new();

        let response1 = McpResponse::new("response 1".to_string());
        let response2 = McpResponse::new("response 2".to_string());

        handler.cache_response("query1".to_string(), response1, 3600);
        handler.cache_response("query2".to_string(), response2, 3600);

        assert_eq!(handler.cache_size(), 2);

        let cached1 = handler.get_cached("query1");
        let cached2 = handler.get_cached("query2");

        assert!(cached1.is_some());
        assert!(cached2.is_some());
        assert_eq!(cached1.unwrap().content, "response 1");
        assert_eq!(cached2.unwrap().content, "response 2");
    }

    #[test]
    fn test_cache_overwrite() {
        let mut handler = OfflineHandler::new();

        let response1 = McpResponse::new("first".to_string());
        let response2 = McpResponse::new("second".to_string());

        handler.cache_response("query".to_string(), response1, 3600);
        handler.cache_response("query".to_string(), response2, 3600);

        assert_eq!(handler.cache_size(), 1);

        let cached = handler.get_cached("query");
        assert_eq!(cached.unwrap().content, "second");
    }

    #[test]
    fn test_offline_metadata() {
        let handler = OfflineHandler::new();

        let response = handler.local_suggestions("git status").unwrap();

        assert_eq!(response.get_metadata("source"), Some(&"offline".to_string()));
        assert_eq!(response.get_metadata("category"), Some(&"git".to_string()));
    }
}
