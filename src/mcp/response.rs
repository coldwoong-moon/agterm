//! MCP response processing and command extraction
//!
//! Handles parsing and processing of MCP responses, including command extraction
//! and tool call result handling.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Processed MCP response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    /// Main content of the response
    pub content: String,
    /// Suggested command to execute (if any)
    pub suggested_command: Option<String>,
    /// Tool calls made during response
    pub tool_calls: Vec<ToolCallResult>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Result of a tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    /// Name of the tool that was called
    pub name: String,
    /// Result of the tool call (if successful)
    pub result: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
}

impl McpResponse {
    /// Create a new MCP response
    pub fn new(content: String) -> Self {
        Self {
            content,
            suggested_command: None,
            tool_calls: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Extract command from response content
    ///
    /// Looks for common patterns like:
    /// - Code blocks with shell/bash markers
    /// - Lines starting with $
    /// - Explicit "Run: command" patterns
    pub fn extract_command(&self) -> Option<String> {
        if let Some(ref cmd) = self.suggested_command {
            return Some(cmd.clone());
        }

        // Try to extract from content
        Self::extract_command_from_text(&self.content)
    }

    fn extract_command_from_text(text: &str) -> Option<String> {
        // Pattern 1: Code blocks with shell/bash markers
        if let Some(cmd) = Self::extract_from_code_block(text) {
            return Some(cmd);
        }

        // Pattern 2: Lines starting with $ or #
        if let Some(cmd) = Self::extract_from_prompt_line(text) {
            return Some(cmd);
        }

        // Pattern 3: Explicit "Run: command" pattern
        if let Some(cmd) = Self::extract_from_run_pattern(text) {
            return Some(cmd);
        }

        // Pattern 4: Single command line in backticks
        if let Some(cmd) = Self::extract_from_backticks(text) {
            return Some(cmd);
        }

        None
    }

    fn extract_from_code_block(text: &str) -> Option<String> {
        // Look for ```bash or ```shell code blocks
        let patterns = ["```bash\n", "```shell\n", "```sh\n", "```zsh\n"];

        for pattern in &patterns {
            if let Some(start) = text.find(pattern) {
                let content_start = start + pattern.len();
                if let Some(end) = text[content_start..].find("\n```") {
                    let command = text[content_start..content_start + end].trim();
                    if !command.is_empty() {
                        return Some(command.to_string());
                    }
                }
            }
        }

        // Try generic code blocks
        if let Some(start) = text.find("```\n") {
            let content_start = start + 4;
            if let Some(end) = text[content_start..].find("\n```") {
                let command = text[content_start..content_start + end].trim();
                // Check if it looks like a shell command
                if Self::looks_like_shell_command(command) {
                    return Some(command.to_string());
                }
            }
        }

        None
    }

    fn extract_from_prompt_line(text: &str) -> Option<String> {
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("$ ") {
                return Some(trimmed[2..].trim().to_string());
            }
            if trimmed.starts_with("# ") && Self::looks_like_shell_command(&trimmed[2..]) {
                return Some(trimmed[2..].trim().to_string());
            }
        }
        None
    }

    fn extract_from_run_pattern(text: &str) -> Option<String> {
        let patterns = ["Run: ", "Execute: ", "Command: ", "Try: "];

        for pattern in &patterns {
            if let Some(pos) = text.find(pattern) {
                let start = pos + pattern.len();
                let rest = &text[start..];

                // Take until newline or end of string
                let end = rest.find('\n').unwrap_or(rest.len());
                let command = rest[..end].trim();

                if !command.is_empty() {
                    return Some(command.to_string());
                }
            }
        }
        None
    }

    fn extract_from_backticks(text: &str) -> Option<String> {
        // Look for single command in backticks
        if let Some(start) = text.find('`') {
            if let Some(end) = text[start + 1..].find('`') {
                let command = text[start + 1..start + 1 + end].trim();
                if Self::looks_like_shell_command(command) && !command.contains('\n') {
                    return Some(command.to_string());
                }
            }
        }
        None
    }

    fn looks_like_shell_command(text: &str) -> bool {
        let common_commands = [
            "ls", "cd", "pwd", "cat", "echo", "grep", "find", "git", "cargo", "npm", "yarn",
            "python", "rustc", "make", "docker", "kubectl", "ssh", "curl", "wget", "tar",
            "zip", "unzip", "mv", "cp", "rm", "mkdir", "touch", "chmod", "chown", "ps",
            "kill", "top", "df", "du", "free", "uname", "which", "whereis", "man", "vim",
            "nano", "emacs", "code",
        ];

        let first_word = text.split_whitespace().next().unwrap_or("");

        // Check if starts with common command
        if common_commands.iter().any(|&cmd| first_word == cmd) {
            return true;
        }

        // Check if starts with path or variable
        if first_word.starts_with("./")
            || first_word.starts_with("../")
            || first_word.starts_with('/')
            || first_word.starts_with('$')
        {
            return true;
        }

        false
    }

    /// Convert from rmcp response (placeholder for actual rmcp integration)
    pub fn from_mcp_response(content: String) -> Self {
        let mut response = Self::new(content.clone());
        response.suggested_command = Self::extract_command_from_text(&content);
        response
    }

    /// Add a tool call result
    pub fn add_tool_call(&mut self, name: String, result: Option<String>, error: Option<String>) {
        self.tool_calls.push(ToolCallResult { name, result, error });
    }

    /// Set metadata value
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Check if response has errors
    pub fn has_errors(&self) -> bool {
        self.tool_calls.iter().any(|call| call.error.is_some())
    }

    /// Get all errors
    pub fn get_errors(&self) -> Vec<String> {
        self.tool_calls
            .iter()
            .filter_map(|call| call.error.clone())
            .collect()
    }
}

impl Default for McpResponse {
    fn default() -> Self {
        Self::new(String::new())
    }
}

impl ToolCallResult {
    /// Create a successful tool call result
    pub fn success(name: String, result: String) -> Self {
        Self {
            name,
            result: Some(result),
            error: None,
        }
    }

    /// Create a failed tool call result
    pub fn error(name: String, error: String) -> Self {
        Self {
            name,
            result: None,
            error: Some(error),
        }
    }

    /// Check if tool call was successful
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }

    /// Check if tool call failed
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_creation() {
        let response = McpResponse::new("Test content".to_string());

        assert_eq!(response.content, "Test content");
        assert!(response.suggested_command.is_none());
        assert_eq!(response.tool_calls.len(), 0);
        assert_eq!(response.metadata.len(), 0);
    }

    #[test]
    fn test_extract_command_from_bash_block() {
        let text = r#"
You can run this command:

```bash
cargo build --release
```
"#;
        let response = McpResponse::new(text.to_string());
        let command = response.extract_command();

        assert_eq!(command, Some("cargo build --release".to_string()));
    }

    #[test]
    fn test_extract_command_from_shell_block() {
        let text = r#"
Try this:

```shell
git status
```
"#;
        let response = McpResponse::new(text.to_string());
        let command = response.extract_command();

        assert_eq!(command, Some("git status".to_string()));
    }

    #[test]
    fn test_extract_command_from_prompt() {
        let text = "Run this command:\n$ ls -la\nTo see all files.";
        let response = McpResponse::new(text.to_string());
        let command = response.extract_command();

        assert_eq!(command, Some("ls -la".to_string()));
    }

    #[test]
    fn test_extract_command_from_run_pattern() {
        let text = "Run: cargo test\nThis will run all tests.";
        let response = McpResponse::new(text.to_string());
        let command = response.extract_command();

        assert_eq!(command, Some("cargo test".to_string()));
    }

    #[test]
    fn test_extract_command_from_backticks() {
        let text = "You can use `git commit -m \"message\"` to commit.";
        let response = McpResponse::new(text.to_string());
        let command = response.extract_command();

        assert_eq!(command, Some("git commit -m \"message\"".to_string()));
    }

    #[test]
    fn test_extract_command_with_explicit_suggestion() {
        let mut response = McpResponse::new("Some content".to_string());
        response.suggested_command = Some("npm install".to_string());

        let command = response.extract_command();
        assert_eq!(command, Some("npm install".to_string()));
    }

    #[test]
    fn test_no_command_extraction() {
        let text = "This is just regular text without any commands.";
        let response = McpResponse::new(text.to_string());
        let command = response.extract_command();

        assert_eq!(command, None);
    }

    #[test]
    fn test_looks_like_shell_command() {
        assert!(McpResponse::looks_like_shell_command("ls -la"));
        assert!(McpResponse::looks_like_shell_command("git status"));
        assert!(McpResponse::looks_like_shell_command("cargo build"));
        assert!(McpResponse::looks_like_shell_command("./script.sh"));
        assert!(McpResponse::looks_like_shell_command("/usr/bin/python"));
        assert!(McpResponse::looks_like_shell_command("$HOME/bin/tool"));

        assert!(!McpResponse::looks_like_shell_command("hello world"));
        assert!(!McpResponse::looks_like_shell_command("some random text"));
    }

    #[test]
    fn test_add_tool_call() {
        let mut response = McpResponse::default();

        response.add_tool_call(
            "file_read".to_string(),
            Some("file contents".to_string()),
            None,
        );

        assert_eq!(response.tool_calls.len(), 1);
        assert_eq!(response.tool_calls[0].name, "file_read");
        assert_eq!(response.tool_calls[0].result, Some("file contents".to_string()));
        assert!(response.tool_calls[0].error.is_none());
    }

    #[test]
    fn test_tool_call_success() {
        let call = ToolCallResult::success("test".to_string(), "ok".to_string());

        assert!(call.is_success());
        assert!(!call.is_error());
        assert_eq!(call.result, Some("ok".to_string()));
        assert!(call.error.is_none());
    }

    #[test]
    fn test_tool_call_error() {
        let call = ToolCallResult::error("test".to_string(), "failed".to_string());

        assert!(!call.is_success());
        assert!(call.is_error());
        assert!(call.result.is_none());
        assert_eq!(call.error, Some("failed".to_string()));
    }

    #[test]
    fn test_has_errors() {
        let mut response = McpResponse::default();

        assert!(!response.has_errors());

        response.add_tool_call("tool1".to_string(), Some("ok".to_string()), None);
        assert!(!response.has_errors());

        response.add_tool_call("tool2".to_string(), None, Some("error".to_string()));
        assert!(response.has_errors());
    }

    #[test]
    fn test_get_errors() {
        let mut response = McpResponse::default();

        response.add_tool_call("tool1".to_string(), Some("ok".to_string()), None);
        response.add_tool_call("tool2".to_string(), None, Some("error1".to_string()));
        response.add_tool_call("tool3".to_string(), None, Some("error2".to_string()));

        let errors = response.get_errors();
        assert_eq!(errors.len(), 2);
        assert!(errors.contains(&"error1".to_string()));
        assert!(errors.contains(&"error2".to_string()));
    }

    #[test]
    fn test_metadata() {
        let mut response = McpResponse::default();

        response.set_metadata("key1".to_string(), "value1".to_string());
        response.set_metadata("key2".to_string(), "value2".to_string());

        assert_eq!(response.get_metadata("key1"), Some(&"value1".to_string()));
        assert_eq!(response.get_metadata("key2"), Some(&"value2".to_string()));
        assert_eq!(response.get_metadata("key3"), None);
    }

    #[test]
    fn test_from_mcp_response() {
        let content = r#"
You should run:

```bash
cargo fmt
```
"#;
        let response = McpResponse::from_mcp_response(content.to_string());

        assert_eq!(response.content, content);
        assert_eq!(response.suggested_command, Some("cargo fmt".to_string()));
    }

    #[test]
    fn test_extract_multiline_command_from_block() {
        let text = r#"
Run these commands:

```bash
cd project
cargo build
cargo test
```
"#;
        let response = McpResponse::new(text.to_string());
        let command = response.extract_command();

        assert!(command.is_some());
        let cmd = command.unwrap();
        assert!(cmd.contains("cd project"));
        assert!(cmd.contains("cargo build"));
        assert!(cmd.contains("cargo test"));
    }

    #[test]
    fn test_serialization() {
        let mut response = McpResponse::new("test".to_string());
        response.suggested_command = Some("ls".to_string());
        response.add_tool_call("tool".to_string(), Some("result".to_string()), None);
        response.set_metadata("key".to_string(), "value".to_string());

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: McpResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.content, "test");
        assert_eq!(deserialized.suggested_command, Some("ls".to_string()));
        assert_eq!(deserialized.tool_calls.len(), 1);
        assert_eq!(
            deserialized.get_metadata("key"),
            Some(&"value".to_string())
        );
    }
}
