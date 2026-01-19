use thiserror::Error;

/// Errors that can occur in MCP operations
#[derive(Error, Debug, Clone)]
pub enum McpError {
    /// Connection-related errors
    #[error("Connection error: {0}")]
    Connection(String),

    /// Transport-level errors
    #[error("Transport error: {0}")]
    Transport(String),

    /// Request construction or sending errors
    #[error("Request error: {0}")]
    Request(String),

    /// Response parsing or validation errors
    #[error("Response error: {0}")]
    Response(String),

    /// Tool call execution errors
    #[error("Tool call error: {0}")]
    ToolCall(String),

    /// Timeout errors
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Server not found errors
    #[error("Server not found: {0}")]
    ServerNotFound(String),

    /// Offline/network errors
    #[error("Offline: {0}")]
    Offline(String),

    /// Rate limit errors
    #[error("Rate limited: {0}")]
    RateLimited(String),
}

impl McpError {
    /// Create a connection error
    pub fn connection<S: Into<String>>(msg: S) -> Self {
        Self::Connection(msg.into())
    }

    /// Create a transport error
    pub fn transport<S: Into<String>>(msg: S) -> Self {
        Self::Transport(msg.into())
    }

    /// Create a request error
    pub fn request<S: Into<String>>(msg: S) -> Self {
        Self::Request(msg.into())
    }

    /// Create a response error
    pub fn response<S: Into<String>>(msg: S) -> Self {
        Self::Response(msg.into())
    }

    /// Create a tool call error
    pub fn tool_call<S: Into<String>>(msg: S) -> Self {
        Self::ToolCall(msg.into())
    }

    /// Create a timeout error
    pub fn timeout<S: Into<String>>(msg: S) -> Self {
        Self::Timeout(msg.into())
    }

    /// Create a config error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::Config(msg.into())
    }

    /// Create a server not found error
    pub fn server_not_found<S: Into<String>>(msg: S) -> Self {
        Self::ServerNotFound(msg.into())
    }

    /// Create an offline error
    pub fn offline<S: Into<String>>(msg: S) -> Self {
        Self::Offline(msg.into())
    }

    /// Create a rate limited error
    pub fn rate_limited<S: Into<String>>(msg: S) -> Self {
        Self::RateLimited(msg.into())
    }
}

/// Result type for MCP operations
pub type McpResult<T> = Result<T, McpError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = McpError::connection("failed to connect");
        assert_eq!(err.to_string(), "Connection error: failed to connect");

        let err = McpError::transport("transport failed");
        assert_eq!(err.to_string(), "Transport error: transport failed");

        let err = McpError::request("invalid request");
        assert_eq!(err.to_string(), "Request error: invalid request");

        let err = McpError::response("invalid response");
        assert_eq!(err.to_string(), "Response error: invalid response");

        let err = McpError::tool_call("tool execution failed");
        assert_eq!(err.to_string(), "Tool call error: tool execution failed");

        let err = McpError::timeout("operation timed out");
        assert_eq!(err.to_string(), "Operation timed out: operation timed out");

        let err = McpError::config("invalid config");
        assert_eq!(err.to_string(), "Configuration error: invalid config");

        let err = McpError::server_not_found("server xyz");
        assert_eq!(err.to_string(), "Server not found: server xyz");

        let err = McpError::offline("no network");
        assert_eq!(err.to_string(), "Offline: no network");

        let err = McpError::rate_limited("too many requests");
        assert_eq!(err.to_string(), "Rate limited: too many requests");
    }

    #[test]
    fn test_error_clone() {
        let err = McpError::connection("test");
        let cloned = err.clone();
        assert_eq!(err.to_string(), cloned.to_string());
    }

    #[test]
    fn test_result_type() {
        let ok_result: McpResult<i32> = Ok(42);
        assert_eq!(ok_result.unwrap(), 42);

        let err_result: McpResult<i32> = Err(McpError::connection("failed"));
        assert!(err_result.is_err());
    }
}
