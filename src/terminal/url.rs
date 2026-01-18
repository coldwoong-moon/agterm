//! URL and file path detection and opening
//!
//! Provides functionality for detecting URLs and file paths in terminal output
//! and opening them in the system's default application.

use std::sync::LazyLock;
use regex::Regex;

/// URL pattern regex - matches http://, https?://, and file:// URLs
pub static URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)https?://[^\s<>"'`)\]]+|file://[^\s<>"'`)\]]+"#).unwrap()
});

/// File path pattern regex - matches absolute paths, home-relative paths, and relative paths
pub static FILE_PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|[\s:])(/[^\s:]+|~[^\s:]+|\.{1,2}/[^\s:]+)").unwrap()
});

/// Type of detected link
#[derive(Debug, Clone, PartialEq)]
pub enum LinkType {
    /// HTTP(S) or file:// URL
    Url(String),
    /// File system path
    FilePath(String),
}

/// Find a link at the specified column position in a line of text
///
/// # Arguments
/// * `text` - The line of text to search
/// * `col` - The column position to check
///
/// # Returns
/// * `Some(LinkType)` if a link was found at the position
/// * `None` if no link was found
pub fn find_link_at(text: &str, col: usize) -> Option<LinkType> {
    // Check URLs first (higher priority)
    for m in URL_REGEX.find_iter(text) {
        if col >= m.start() && col < m.end() {
            return Some(LinkType::Url(m.as_str().to_string()));
        }
    }

    // Check file paths
    for m in FILE_PATH_REGEX.find_iter(text) {
        if col >= m.start() && col < m.end() {
            let path = m.as_str().trim();
            return Some(LinkType::FilePath(path.to_string()));
        }
    }

    None
}

/// Open a link in the system's default application
///
/// # Arguments
/// * `link` - The link to open
///
/// # Returns
/// * `Ok(())` if the link was opened successfully
/// * `Err(String)` with error message if opening failed
pub fn open_link(link: &LinkType) -> Result<(), String> {
    match link {
        LinkType::Url(url) => {
            open::that(url).map_err(|e| format!("Failed to open URL: {e}"))
        }
        LinkType::FilePath(path) => {
            // Expand tilde to home directory
            let expanded = if path.starts_with('~') {
                if let Some(home) = dirs::home_dir() {
                    path.replacen('~', home.to_str().unwrap_or("~"), 1)
                } else {
                    path.clone()
                }
            } else {
                path.clone()
            };

            open::that(&expanded).map_err(|e| format!("Failed to open file path: {e}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_detection_http() {
        let text = "Check out http://example.com for more info";
        let link = find_link_at(text, 15);
        assert_eq!(link, Some(LinkType::Url("http://example.com".to_string())));
    }

    #[test]
    fn test_url_detection_https() {
        let text = "Visit https://github.com/user/repo for the code";
        let link = find_link_at(text, 10);
        assert_eq!(link, Some(LinkType::Url("https://github.com/user/repo".to_string())));
    }

    #[test]
    fn test_url_detection_file() {
        let text = "See file:///home/user/document.txt";
        let link = find_link_at(text, 10);
        assert_eq!(link, Some(LinkType::Url("file:///home/user/document.txt".to_string())));
    }

    #[test]
    fn test_file_path_detection_absolute() {
        let text = "Error in /usr/local/bin/app.sh:42";
        let link = find_link_at(text, 12);
        assert_eq!(link, Some(LinkType::FilePath("/usr/local/bin/app.sh".to_string())));
    }

    #[test]
    fn test_file_path_detection_home() {
        let text = "Config at ~/config.toml";
        let link = find_link_at(text, 12);
        assert_eq!(link, Some(LinkType::FilePath("~/config.toml".to_string())));
    }

    #[test]
    fn test_file_path_detection_relative() {
        let text = "See ./src/main.rs for details";
        let link = find_link_at(text, 7);
        assert_eq!(link, Some(LinkType::FilePath("./src/main.rs".to_string())));
    }

    #[test]
    fn test_no_link_at_position() {
        let text = "Just some regular text";
        let link = find_link_at(text, 5);
        assert_eq!(link, None);
    }

    #[test]
    fn test_url_with_query_params() {
        let text = "API: https://api.example.com/v1/users?id=123&sort=asc";
        let link = find_link_at(text, 10);
        assert_eq!(link, Some(LinkType::Url("https://api.example.com/v1/users?id=123&sort=asc".to_string())));
    }

    #[test]
    fn test_multiple_urls_in_line() {
        let text = "See http://example.com and https://github.com";

        // First URL
        let link1 = find_link_at(text, 6);
        assert_eq!(link1, Some(LinkType::Url("http://example.com".to_string())));

        // Second URL
        let link2 = find_link_at(text, 30);
        assert_eq!(link2, Some(LinkType::Url("https://github.com".to_string())));
    }

    #[test]
    fn test_url_at_start_of_line() {
        let text = "https://example.com is the site";
        let link = find_link_at(text, 0);
        assert_eq!(link, Some(LinkType::Url("https://example.com".to_string())));
    }

    #[test]
    fn test_url_at_end_of_line() {
        let text = "Visit https://example.com";
        let link = find_link_at(text, 24);
        assert_eq!(link, Some(LinkType::Url("https://example.com".to_string())));
    }
}
