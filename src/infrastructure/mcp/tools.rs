//! MCP Tool Helpers
//!
//! Helper types and functions for working with MCP tools.

use rmcp::model::{CallToolResult, RawContent, Tool};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Tool information with server context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    /// Server name that provides this tool
    pub server: String,
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: Option<String>,
    /// Input schema (JSON Schema)
    pub input_schema: Option<Value>,
}

impl ToolInfo {
    /// Create from rmcp Tool with server name
    pub fn from_tool(server: impl Into<String>, tool: &Tool) -> Self {
        Self {
            server: server.into(),
            name: tool.name.to_string(),
            description: tool
                .description
                .as_ref()
                .map(std::string::ToString::to_string),
            input_schema: Some(serde_json::to_value(&tool.input_schema).unwrap_or(Value::Null)),
        }
    }

    /// Get fully qualified name (`server:tool_name`)
    #[must_use]
    pub fn qualified_name(&self) -> String {
        format!("{}:{}", self.server, self.name)
    }
}

/// Tool call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    /// Server name
    pub server: String,
    /// Tool name
    pub tool: String,
    /// Arguments
    #[serde(default)]
    pub arguments: HashMap<String, Value>,
}

impl ToolCallRequest {
    /// Create a new tool call request
    pub fn new(server: impl Into<String>, tool: impl Into<String>) -> Self {
        Self {
            server: server.into(),
            tool: tool.into(),
            arguments: HashMap::new(),
        }
    }

    /// Add an argument
    pub fn with_arg(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.arguments.insert(key.into(), value.into());
        self
    }

    /// Set arguments from JSON value
    #[must_use]
    pub fn with_args_json(mut self, args: Value) -> Self {
        if let Some(obj) = args.as_object() {
            self.arguments = obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        }
        self
    }

    /// Convert arguments to `serde_json::Map`
    #[must_use]
    pub fn arguments_as_map(&self) -> Option<serde_json::Map<String, Value>> {
        if self.arguments.is_empty() {
            None
        } else {
            let map: serde_json::Map<String, Value> = self
                .arguments
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            Some(map)
        }
    }
}

/// Tool call result wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResponse {
    /// Whether the call succeeded
    pub success: bool,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Text content from result
    pub text: Vec<String>,
    /// Raw content items
    #[serde(skip)]
    pub content: Vec<ContentItem>,
}

/// Simplified content item
#[derive(Debug, Clone)]
pub enum ContentItem {
    Text(String),
    Image { data: String, mime_type: String },
    Resource { uri: String, text: Option<String> },
    Audio { data: String, mime_type: String },
}

impl ToolCallResponse {
    /// Create from rmcp `CallToolResult`
    #[must_use]
    pub fn from_result(result: &CallToolResult) -> Self {
        let success = !result.is_error.unwrap_or(false);
        let mut text = Vec::new();
        let mut content = Vec::new();

        for item in &result.content {
            // Access the raw content via Deref
            match &item.raw {
                RawContent::Text(t) => {
                    text.push(t.text.clone());
                    content.push(ContentItem::Text(t.text.clone()));
                }
                RawContent::Image(img) => {
                    content.push(ContentItem::Image {
                        data: img.data.clone(),
                        mime_type: img.mime_type.clone(),
                    });
                }
                RawContent::Resource(res) => {
                    let uri = match &res.resource {
                        rmcp::model::ResourceContents::TextResourceContents {
                            uri, text, ..
                        } => {
                            content.push(ContentItem::Resource {
                                uri: uri.clone(),
                                text: Some(text.clone()),
                            });
                            uri.clone()
                        }
                        rmcp::model::ResourceContents::BlobResourceContents { uri, .. } => {
                            content.push(ContentItem::Resource {
                                uri: uri.clone(),
                                text: None,
                            });
                            uri.clone()
                        }
                    };
                    let _ = uri; // Suppress unused warning
                }
                RawContent::Audio(audio) => {
                    content.push(ContentItem::Audio {
                        data: audio.data.clone(),
                        mime_type: audio.mime_type.clone(),
                    });
                }
                RawContent::ResourceLink(res) => {
                    content.push(ContentItem::Resource {
                        uri: res.uri.clone(),
                        text: None,
                    });
                }
            }
        }

        Self {
            success,
            error: None,
            text,
            content,
        }
    }

    /// Create an error response
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            error: Some(message.into()),
            text: Vec::new(),
            content: Vec::new(),
        }
    }

    /// Get combined text content
    #[must_use]
    pub fn text_content(&self) -> String {
        self.text.join("\n")
    }

    /// Check if there's any text content
    #[must_use]
    pub fn has_text(&self) -> bool {
        !self.text.is_empty()
    }
}

/// Parse tool arguments from various formats
pub fn parse_tool_arguments(input: &str) -> Result<HashMap<String, Value>, serde_json::Error> {
    // Try parsing as JSON object
    if let Ok(value) = serde_json::from_str::<Value>(input) {
        if let Some(obj) = value.as_object() {
            return Ok(obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
        }
    }

    // If it's a single value, try to parse as JSON
    if let Ok(value) = serde_json::from_str::<Value>(input) {
        let mut map = HashMap::new();
        map.insert("input".to_string(), value);
        return Ok(map);
    }

    // Fall back to treating it as a plain string
    let mut map = HashMap::new();
    map.insert("input".to_string(), Value::String(input.to_string()));
    Ok(map)
}

/// Format tool result for display
#[must_use]
pub fn format_tool_result(response: &ToolCallResponse) -> String {
    let mut output = String::new();

    if !response.success {
        if let Some(ref err) = response.error {
            output.push_str(&format!("Error: {err}\n"));
        }
    }

    for (i, item) in response.content.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }

        match item {
            ContentItem::Text(text) => {
                output.push_str(text);
            }
            ContentItem::Image { mime_type, .. } => {
                output.push_str(&format!("[Image: {mime_type}]"));
            }
            ContentItem::Resource { uri, text } => {
                output.push_str(&format!("[Resource: {uri}]"));
                if let Some(t) = text {
                    output.push_str(&format!("\n{t}"));
                }
            }
            ContentItem::Audio { mime_type, .. } => {
                output.push_str(&format!("[Audio: {mime_type}]"));
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_info_qualified_name() {
        let info = ToolInfo {
            server: "git-server".to_string(),
            name: "git_status".to_string(),
            description: Some("Get git status".to_string()),
            input_schema: None,
        };

        assert_eq!(info.qualified_name(), "git-server:git_status");
    }

    #[test]
    fn test_tool_call_request() {
        let request = ToolCallRequest::new("server", "tool")
            .with_arg("path", "/tmp")
            .with_arg("verbose", true);

        assert_eq!(request.server, "server");
        assert_eq!(request.tool, "tool");
        assert_eq!(request.arguments.len(), 2);
    }

    #[test]
    fn test_parse_arguments_json() {
        let args = parse_tool_arguments(r#"{"key": "value", "num": 42}"#).unwrap();

        assert_eq!(
            args.get("key").unwrap(),
            &Value::String("value".to_string())
        );
        assert_eq!(args.get("num").unwrap(), &Value::Number(42.into()));
    }

    #[test]
    fn test_parse_arguments_plain_string() {
        let args = parse_tool_arguments("hello world").unwrap();

        assert_eq!(
            args.get("input").unwrap(),
            &Value::String("hello world".to_string())
        );
    }

    #[test]
    fn test_error_response() {
        let response = ToolCallResponse::error("Something went wrong");

        assert!(!response.success);
        assert_eq!(response.error, Some("Something went wrong".to_string()));
    }
}
