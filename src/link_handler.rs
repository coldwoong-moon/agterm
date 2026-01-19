//! Terminal Link Handler
//!
//! Provides comprehensive link detection and handling capabilities for terminal output.
//! Supports multiple link types including URLs, file paths, email addresses, IP addresses,
//! and custom regex patterns.
//!
//! This module extends the basic URL detection in `terminal::url` with additional
//! link types and customizable action handlers.

use std::sync::Arc;
use regex::Regex;
use once_cell::sync::Lazy;

/// Type of detected link
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LinkType {
    /// Web URL (http, https)
    Url,
    /// File system path (absolute/relative)
    FilePath,
    /// Email address
    Email,
    /// IP address with optional port
    IpAddress,
    /// Custom user-defined pattern
    Custom(String),
}

impl LinkType {
    /// Get a human-readable name for the link type
    pub fn name(&self) -> &str {
        match self {
            LinkType::Url => "URL",
            LinkType::FilePath => "File Path",
            LinkType::Email => "Email",
            LinkType::IpAddress => "IP Address",
            LinkType::Custom(name) => name,
        }
    }
}

/// A detected link with its location in text
#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    /// Type of the link
    pub link_type: LinkType,
    /// The actual link text
    pub text: String,
    /// Start position in the text (byte offset)
    pub start: usize,
    /// End position in the text (byte offset)
    pub end: usize,
}

impl Link {
    /// Create a new link
    pub fn new(link_type: LinkType, text: String, start: usize, end: usize) -> Self {
        Self {
            link_type,
            text,
            start,
            end,
        }
    }

    /// Get the length of the link text
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Check if the link is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if a position is within this link's range
    pub fn contains_position(&self, pos: usize) -> bool {
        pos >= self.start && pos < self.end
    }
}

/// Link pattern with regex and type
struct LinkPattern {
    regex: Regex,
    link_type: LinkType,
}

/// Link detector that finds links in text
pub struct LinkDetector {
    patterns: Vec<LinkPattern>,
}

/// Default regex patterns for common link types
static URL_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)\b(?:https?|ftp|file)://[^\s<>"'\]\)]+"#).unwrap()
});

static EMAIL_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b").unwrap()
});

static IP_PATTERN: Lazy<Regex> = Lazy::new(|| {
    // Matches IPv4 addresses with optional port (e.g., 192.168.1.1:8080)
    // and IPv6 addresses in brackets with optional port (e.g., [::1]:8080)
    Regex::new(r"(?:\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)(?::\d{1,5})?\b|\[(?:[0-9a-fA-F]{0,4}:){2,7}[0-9a-fA-F]{0,4}\](?::\d{1,5})?)").unwrap()
});

static FILE_PATH_PATTERN: Lazy<Regex> = Lazy::new(|| {
    // Matches absolute paths, home-relative paths, and relative paths
    // Enhanced to better detect file paths with extensions
    // Requires at least one non-slash character after initial slash to avoid matching "//"
    Regex::new(r"(?:^|[\s:])(/[^\s:;,\]\)/][^\s:;,\]\)]*|~[^\s:;,\]\)]+|\.{1,2}/[^\s:;,\]\)]+)").unwrap()
});

impl LinkDetector {
    /// Create a new link detector with default patterns
    pub fn new() -> Self {
        Self {
            patterns: vec![
                LinkPattern {
                    regex: URL_PATTERN.clone(),
                    link_type: LinkType::Url,
                },
                LinkPattern {
                    regex: EMAIL_PATTERN.clone(),
                    link_type: LinkType::Email,
                },
                LinkPattern {
                    regex: IP_PATTERN.clone(),
                    link_type: LinkType::IpAddress,
                },
                LinkPattern {
                    regex: FILE_PATH_PATTERN.clone(),
                    link_type: LinkType::FilePath,
                },
            ],
        }
    }

    /// Create a link detector with only specified link types
    pub fn with_types(types: &[LinkType]) -> Self {
        let mut detector = Self {
            patterns: Vec::new(),
        };

        for link_type in types {
            match link_type {
                LinkType::Url => detector.patterns.push(LinkPattern {
                    regex: URL_PATTERN.clone(),
                    link_type: LinkType::Url,
                }),
                LinkType::Email => detector.patterns.push(LinkPattern {
                    regex: EMAIL_PATTERN.clone(),
                    link_type: LinkType::Email,
                }),
                LinkType::IpAddress => detector.patterns.push(LinkPattern {
                    regex: IP_PATTERN.clone(),
                    link_type: LinkType::IpAddress,
                }),
                LinkType::FilePath => detector.patterns.push(LinkPattern {
                    regex: FILE_PATH_PATTERN.clone(),
                    link_type: LinkType::FilePath,
                }),
                LinkType::Custom(_) => {
                    // Custom patterns must be added via add_custom_pattern
                }
            }
        }

        detector
    }

    /// Add a custom link pattern
    ///
    /// # Arguments
    /// * `name` - Name for this custom link type
    /// * `pattern` - Regular expression pattern to match
    ///
    /// # Returns
    /// * `Ok(())` if pattern was added successfully
    /// * `Err(String)` if pattern is invalid
    pub fn add_custom_pattern(&mut self, name: String, pattern: &str) -> Result<(), String> {
        let regex = Regex::new(pattern)
            .map_err(|e| format!("Invalid regex pattern: {e}"))?;

        self.patterns.push(LinkPattern {
            regex,
            link_type: LinkType::Custom(name),
        });

        Ok(())
    }

    /// Detect all links in the given text
    ///
    /// # Arguments
    /// * `text` - The text to scan for links
    ///
    /// # Returns
    /// A vector of detected links, sorted by start position
    pub fn detect_links(&self, text: &str) -> Vec<Link> {
        let mut links = Vec::new();

        for pattern in &self.patterns {
            for mat in pattern.regex.find_iter(text) {
                let mut link_text = mat.as_str().to_string();
                let mut start = mat.start();

                // For file paths, trim leading whitespace/colon
                if matches!(pattern.link_type, LinkType::FilePath) {
                    link_text = link_text.trim_start_matches(|c: char| c.is_whitespace() || c == ':').to_string();
                    start = mat.start() + (mat.as_str().len() - link_text.len());
                }

                let end = start + link_text.len();

                links.push(Link::new(
                    pattern.link_type.clone(),
                    link_text,
                    start,
                    end,
                ));
            }
        }

        // Sort by start position and remove overlapping links (keep first match)
        links.sort_by_key(|link| link.start);
        Self::remove_overlapping_links(links)
    }

    /// Find a link at the specified position
    ///
    /// # Arguments
    /// * `text` - The text to search
    /// * `pos` - The byte position to check
    ///
    /// # Returns
    /// The link at the position, if any
    pub fn find_link_at(&self, text: &str, pos: usize) -> Option<Link> {
        self.detect_links(text)
            .into_iter()
            .find(|link| link.contains_position(pos))
    }

    /// Remove overlapping links, keeping the first match
    fn remove_overlapping_links(links: Vec<Link>) -> Vec<Link> {
        let mut result = Vec::new();
        let mut last_end = 0;

        for link in links {
            if link.start >= last_end {
                last_end = link.end;
                result.push(link);
            }
        }

        result
    }
}

impl Default for LinkDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Action to perform when a link is activated
#[derive(Clone)]
#[allow(clippy::type_complexity)]
pub enum LinkAction {
    /// Open the link in the system's default application
    OpenDefault,
    /// Copy the link to clipboard
    CopyToClipboard,
    /// Execute a custom command with the link as argument
    Command(String),
    /// Custom callback (Arc-wrapped for cloning)
    Custom(Arc<dyn Fn(&Link) -> Result<(), String> + Send + Sync>),
}

impl std::fmt::Debug for LinkAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkAction::OpenDefault => write!(f, "OpenDefault"),
            LinkAction::CopyToClipboard => write!(f, "CopyToClipboard"),
            LinkAction::Command(cmd) => f.debug_tuple("Command").field(cmd).finish(),
            LinkAction::Custom(_) => write!(f, "Custom(<function>)"),
        }
    }
}

/// Link handler that manages link detection and actions
pub struct LinkHandler {
    detector: LinkDetector,
    default_actions: std::collections::HashMap<LinkType, LinkAction>,
}

impl LinkHandler {
    /// Create a new link handler with default detector and actions
    pub fn new() -> Self {
        let mut handler = Self {
            detector: LinkDetector::new(),
            default_actions: std::collections::HashMap::new(),
        };

        // Set up default actions
        handler.set_default_action(LinkType::Url, LinkAction::OpenDefault);
        handler.set_default_action(LinkType::FilePath, LinkAction::OpenDefault);
        handler.set_default_action(LinkType::Email, LinkAction::OpenDefault);
        handler.set_default_action(LinkType::IpAddress, LinkAction::CopyToClipboard);

        handler
    }

    /// Create a link handler with a custom detector
    pub fn with_detector(detector: LinkDetector) -> Self {
        Self {
            detector,
            default_actions: std::collections::HashMap::new(),
        }
    }

    /// Set the default action for a link type
    pub fn set_default_action(&mut self, link_type: LinkType, action: LinkAction) {
        self.default_actions.insert(link_type, action);
    }

    /// Get the detector
    pub fn detector(&self) -> &LinkDetector {
        &self.detector
    }

    /// Get a mutable reference to the detector
    pub fn detector_mut(&mut self) -> &mut LinkDetector {
        &mut self.detector
    }

    /// Handle a link activation with default action
    ///
    /// # Arguments
    /// * `link` - The link to handle
    ///
    /// # Returns
    /// * `Ok(())` if the link was handled successfully
    /// * `Err(String)` with error message if handling failed
    pub fn handle_link(&self, link: &Link) -> Result<(), String> {
        let action = self.default_actions.get(&link.link_type)
            .ok_or_else(|| format!("No default action for link type: {}", link.link_type.name()))?;

        self.execute_action(link, action)
    }

    /// Handle a link with a specific action
    ///
    /// # Arguments
    /// * `link` - The link to handle
    /// * `action` - The action to perform
    ///
    /// # Returns
    /// * `Ok(())` if the link was handled successfully
    /// * `Err(String)` with error message if handling failed
    pub fn handle_link_with_action(&self, link: &Link, action: &LinkAction) -> Result<(), String> {
        self.execute_action(link, action)
    }

    /// Execute a link action
    fn execute_action(&self, link: &Link, action: &LinkAction) -> Result<(), String> {
        match action {
            LinkAction::OpenDefault => self.open_link(link),
            LinkAction::CopyToClipboard => self.copy_to_clipboard(link),
            LinkAction::Command(cmd) => self.execute_command(link, cmd),
            LinkAction::Custom(callback) => callback(link),
        }
    }

    /// Open a link in the system's default application
    fn open_link(&self, link: &Link) -> Result<(), String> {
        let target = match link.link_type {
            LinkType::Url => link.text.clone(),
            LinkType::FilePath => {
                // Expand tilde to home directory
                if link.text.starts_with('~') {
                    if let Some(home) = dirs::home_dir() {
                        link.text.replacen('~', home.to_str().unwrap_or("~"), 1)
                    } else {
                        link.text.clone()
                    }
                } else {
                    link.text.clone()
                }
            }
            LinkType::Email => format!("mailto:{}", link.text),
            LinkType::IpAddress => {
                // Try to open as HTTP URL if it looks like a web address
                if link.text.contains(':') {
                    format!("http://{}", link.text)
                } else {
                    return Err("Cannot open IP address without port".to_string());
                }
            }
            LinkType::Custom(_) => link.text.clone(),
        };

        open::that(&target)
            .map_err(|e| format!("Failed to open {}: {}", link.link_type.name(), e))
    }

    /// Copy link text to clipboard
    fn copy_to_clipboard(&self, link: &Link) -> Result<(), String> {
        use arboard::Clipboard;

        let mut clipboard = Clipboard::new()
            .map_err(|e| format!("Failed to access clipboard: {e}"))?;

        clipboard.set_text(&link.text)
            .map_err(|e| format!("Failed to copy to clipboard: {e}"))
    }

    /// Execute a command with the link text as argument
    fn execute_command(&self, link: &Link, command: &str) -> Result<(), String> {
        let full_command = command.replace("{}", &link.text);

        std::process::Command::new("sh")
            .arg("-c")
            .arg(&full_command)
            .spawn()
            .map_err(|e| format!("Failed to execute command: {e}"))?;

        Ok(())
    }
}

impl Default for LinkHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_creation() {
        let link = Link::new(LinkType::Url, "https://example.com".to_string(), 0, 19);
        assert_eq!(link.link_type, LinkType::Url);
        assert_eq!(link.text, "https://example.com");
        assert_eq!(link.start, 0);
        assert_eq!(link.end, 19);
        assert_eq!(link.len(), 19);
        assert!(!link.is_empty());
    }

    #[test]
    fn test_link_contains_position() {
        let link = Link::new(LinkType::Url, "https://example.com".to_string(), 10, 29);
        assert!(!link.contains_position(9));
        assert!(link.contains_position(10));
        assert!(link.contains_position(20));
        assert!(link.contains_position(28));
        assert!(!link.contains_position(29));
    }

    #[test]
    fn test_detect_url() {
        let detector = LinkDetector::new();
        let text = "Check out https://example.com for more info";
        let links = detector.detect_links(text);

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].link_type, LinkType::Url);
        assert_eq!(links[0].text, "https://example.com");
        assert_eq!(links[0].start, 10);
    }

    #[test]
    fn test_detect_email() {
        let detector = LinkDetector::new();
        let text = "Contact us at support@example.com for help";
        let links = detector.detect_links(text);

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].link_type, LinkType::Email);
        assert_eq!(links[0].text, "support@example.com");
    }

    #[test]
    fn test_detect_ip_address() {
        let detector = LinkDetector::new();

        // IPv4 without port
        let text1 = "Connect to 192.168.1.1 for admin";
        let links1 = detector.detect_links(text1);
        assert_eq!(links1.len(), 1);
        assert_eq!(links1[0].link_type, LinkType::IpAddress);
        assert_eq!(links1[0].text, "192.168.1.1");

        // IPv4 with port
        let text2 = "Server at 192.168.1.1:8080";
        let links2 = detector.detect_links(text2);
        assert_eq!(links2.len(), 1);
        assert_eq!(links2[0].link_type, LinkType::IpAddress);
        assert_eq!(links2[0].text, "192.168.1.1:8080");

        // IPv6
        let text3 = "Connect to [::1]:8080";
        let links3 = detector.detect_links(text3);
        assert_eq!(links3.len(), 1);
        assert_eq!(links3[0].link_type, LinkType::IpAddress);
        assert_eq!(links3[0].text, "[::1]:8080");
    }

    #[test]
    fn test_detect_file_path() {
        let detector = LinkDetector::new();

        // Absolute path
        let text1 = "Error in /usr/local/bin/app.sh:42";
        let links1 = detector.detect_links(text1);
        assert_eq!(links1.len(), 1);
        assert_eq!(links1[0].link_type, LinkType::FilePath);
        assert_eq!(links1[0].text, "/usr/local/bin/app.sh");

        // Home-relative path
        let text2 = "Config at ~/config.toml";
        let links2 = detector.detect_links(text2);
        assert_eq!(links2.len(), 1);
        assert_eq!(links2[0].link_type, LinkType::FilePath);
        assert_eq!(links2[0].text, "~/config.toml");

        // Relative path
        let text3 = "See ./src/main.rs for details";
        let links3 = detector.detect_links(text3);
        assert_eq!(links3.len(), 1);
        assert_eq!(links3[0].link_type, LinkType::FilePath);
        assert_eq!(links3[0].text, "./src/main.rs");
    }

    #[test]
    fn test_detect_multiple_links() {
        let detector = LinkDetector::new();
        let text = "Visit https://example.com or email support@example.com or check /var/log/app.log";
        let links = detector.detect_links(text);

        assert_eq!(links.len(), 3);
        assert_eq!(links[0].link_type, LinkType::Url);
        assert_eq!(links[1].link_type, LinkType::Email);
        assert_eq!(links[2].link_type, LinkType::FilePath);
    }

    #[test]
    fn test_find_link_at_position() {
        let detector = LinkDetector::new();
        let text = "Visit https://example.com for more info";

        // Position in URL
        let link = detector.find_link_at(text, 15);
        assert!(link.is_some());
        let link = link.unwrap();
        assert_eq!(link.link_type, LinkType::Url);
        assert_eq!(link.text, "https://example.com");

        // Position outside URL
        let no_link = detector.find_link_at(text, 0);
        assert!(no_link.is_none());
    }

    #[test]
    fn test_custom_pattern() {
        let mut detector = LinkDetector::new();

        // Add custom pattern for issue references (e.g., #123)
        detector.add_custom_pattern(
            "Issue".to_string(),
            r"#\d+"
        ).unwrap();

        let text = "Fixed in commit #123 and #456";
        let links = detector.detect_links(text);

        let custom_links: Vec<_> = links.iter()
            .filter(|l| matches!(l.link_type, LinkType::Custom(_)))
            .collect();

        assert_eq!(custom_links.len(), 2);
        assert_eq!(custom_links[0].text, "#123");
        assert_eq!(custom_links[1].text, "#456");
    }

    #[test]
    fn test_detector_with_specific_types() {
        let detector = LinkDetector::with_types(&[LinkType::Url, LinkType::Email]);
        let text = "Email support@example.com or visit https://example.com or check /var/log";
        let links = detector.detect_links(text);

        // Should only detect URL and Email, not file path
        assert_eq!(links.len(), 2);
        assert!(links.iter().any(|l| l.link_type == LinkType::Email));
        assert!(links.iter().any(|l| l.link_type == LinkType::Url));
        assert!(!links.iter().any(|l| l.link_type == LinkType::FilePath));
    }

    #[test]
    fn test_overlapping_links() {
        let mut detector = LinkDetector::new();

        // Add a custom pattern that might overlap with URL
        detector.add_custom_pattern(
            "Word".to_string(),
            r"\bhttps\b"
        ).unwrap();

        let text = "Visit https://example.com";
        let links = detector.detect_links(text);

        // Should only keep the first match (URL pattern), not the overlapping custom pattern
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].link_type, LinkType::Url);
    }

    #[test]
    fn test_link_type_name() {
        assert_eq!(LinkType::Url.name(), "URL");
        assert_eq!(LinkType::FilePath.name(), "File Path");
        assert_eq!(LinkType::Email.name(), "Email");
        assert_eq!(LinkType::IpAddress.name(), "IP Address");
        assert_eq!(LinkType::Custom("Issue".to_string()).name(), "Issue");
    }

    #[test]
    fn test_link_handler_creation() {
        let handler = LinkHandler::new();
        assert!(!handler.default_actions.is_empty());
    }

    #[test]
    fn test_url_with_query_params() {
        let detector = LinkDetector::new();
        let text = "API: https://api.example.com/v1/users?id=123&sort=asc";
        let links = detector.detect_links(text);

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].link_type, LinkType::Url);
        assert_eq!(links[0].text, "https://api.example.com/v1/users?id=123&sort=asc");
    }

    #[test]
    fn test_url_with_fragment() {
        let detector = LinkDetector::new();
        let text = "See https://example.com/docs#section-1";
        let links = detector.detect_links(text);

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].link_type, LinkType::Url);
        assert_eq!(links[0].text, "https://example.com/docs#section-1");
    }

    #[test]
    fn test_file_protocol_url() {
        let detector = LinkDetector::new();
        let text = "Open file:///home/user/document.txt";
        let links = detector.detect_links(text);

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].link_type, LinkType::Url);
        assert_eq!(links[0].text, "file:///home/user/document.txt");
    }

    #[test]
    fn test_ftp_url() {
        let detector = LinkDetector::new();
        let text = "Download from ftp://ftp.example.com/file.zip";
        let links = detector.detect_links(text);

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].link_type, LinkType::Url);
        assert_eq!(links[0].text, "ftp://ftp.example.com/file.zip");
    }

    #[test]
    fn test_email_with_plus() {
        let detector = LinkDetector::new();
        let text = "Email user+tag@example.com";
        let links = detector.detect_links(text);

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].link_type, LinkType::Email);
        assert_eq!(links[0].text, "user+tag@example.com");
    }

    #[test]
    fn test_no_false_positives() {
        let detector = LinkDetector::new();

        // Should not match incomplete URLs
        let text1 = "http:// is incomplete";
        let links1 = detector.detect_links(text1);
        assert_eq!(links1.len(), 0);

        // Should not match @ without valid email
        let text2 = "Price @ $10";
        let links2 = detector.detect_links(text2);
        assert_eq!(links2.len(), 0);
    }

    #[test]
    fn test_multiple_urls_in_line() {
        let detector = LinkDetector::new();
        let text = "See http://example.com and https://github.com and ftp://ftp.example.com";
        let links = detector.detect_links(text);

        assert_eq!(links.len(), 3);
        assert_eq!(links[0].text, "http://example.com");
        assert_eq!(links[1].text, "https://github.com");
        assert_eq!(links[2].text, "ftp://ftp.example.com");
    }

    #[test]
    fn test_url_at_line_boundaries() {
        let detector = LinkDetector::new();

        // URL at start
        let text1 = "https://example.com is the site";
        let links1 = detector.detect_links(text1);
        assert_eq!(links1.len(), 1);
        assert_eq!(links1[0].start, 0);

        // URL at end
        let text2 = "Visit https://example.com";
        let links2 = detector.detect_links(text2);
        assert_eq!(links2.len(), 1);
    }
}
