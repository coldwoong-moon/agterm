//! OSC 8 Hyperlink Protocol Support
//!
//! Implements the OSC 8 hyperlink protocol for clickable URLs in terminal output.
//! Format: `\x1b]8;[params];[url]\x07` or `\x1b]8;[params];[url]\x1b\\`
//!
//! Reference: https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda

use std::sync::Arc;

/// OSC 8 hyperlink with optional ID for grouping
///
/// The hyperlink can be associated with multiple cells, and cells with the same
/// ID are considered part of the same logical hyperlink (e.g., a URL split across lines).
#[derive(Debug, Clone, PartialEq)]
pub struct Hyperlink {
    /// Optional link ID for grouping multiple cells into one logical hyperlink
    pub id: Option<String>,
    /// The target URL (uses Arc for memory efficiency via string interning)
    pub url: Arc<String>,
}

impl Hyperlink {
    /// Create a new hyperlink with URL and optional ID
    pub fn new(url: String, id: Option<String>) -> Self {
        Self {
            id,
            url: Arc::new(url),
        }
    }

    /// Parse OSC 8 sequence data
    ///
    /// Format: `params;url` where params can contain `id=value`
    ///
    /// # Examples
    /// - `";https://example.com"` - URL without ID
    /// - `"id=abc123;https://example.com"` - URL with ID
    /// - `";"` - Empty URL (terminates hyperlink)
    ///
    /// # Returns
    /// - `Some(Hyperlink)` if valid URL is present
    /// - `None` if URL is empty (hyperlink termination)
    pub fn parse_osc8(data: &str) -> Option<Self> {
        // Split into params and URL: "params;url"
        let parts: Vec<&str> = data.splitn(2, ';').collect();
        if parts.len() != 2 {
            return None;
        }

        let params = parts[0];
        let url = parts[1];

        // Empty URL means terminate hyperlink
        if url.is_empty() {
            return None;
        }

        // Parse ID from params (format: "key=value:key=value")
        let id = Self::parse_id_from_params(params);

        Some(Self::new(url.to_string(), id))
    }

    /// Parse ID parameter from OSC 8 params string
    ///
    /// Params format: "key=value:key=value:..." or "key=value"
    fn parse_id_from_params(params: &str) -> Option<String> {
        if params.is_empty() {
            return None;
        }

        // Split by ':' and look for "id=..." entries
        for param in params.split(':') {
            if let Some(value) = param.strip_prefix("id=") {
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }

        None
    }

    /// Get the URL as a string reference
    pub fn url(&self) -> &str {
        &self.url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_url() {
        let result = Hyperlink::parse_osc8(";https://example.com");
        assert!(result.is_some());
        let link = result.unwrap();
        assert_eq!(link.url(), "https://example.com");
        assert_eq!(link.id, None);
    }

    #[test]
    fn test_parse_url_with_id() {
        let result = Hyperlink::parse_osc8("id=test123;https://example.com");
        assert!(result.is_some());
        let link = result.unwrap();
        assert_eq!(link.url(), "https://example.com");
        assert_eq!(link.id, Some("test123".to_string()));
    }

    #[test]
    fn test_parse_url_with_multiple_params() {
        let result = Hyperlink::parse_osc8("id=abc:foo=bar;https://example.com");
        assert!(result.is_some());
        let link = result.unwrap();
        assert_eq!(link.url(), "https://example.com");
        assert_eq!(link.id, Some("abc".to_string()));
    }

    #[test]
    fn test_parse_empty_url_terminates() {
        let result = Hyperlink::parse_osc8(";");
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_invalid_format() {
        let result = Hyperlink::parse_osc8("no-semicolon");
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_url_with_special_chars() {
        let result = Hyperlink::parse_osc8(";https://example.com/path?query=value&other=123");
        assert!(result.is_some());
        let link = result.unwrap();
        assert_eq!(link.url(), "https://example.com/path?query=value&other=123");
    }

    #[test]
    fn test_parse_file_url() {
        let result = Hyperlink::parse_osc8(";file:///home/user/document.txt");
        assert!(result.is_some());
        let link = result.unwrap();
        assert_eq!(link.url(), "file:///home/user/document.txt");
    }
}
