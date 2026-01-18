//! Terminal title management
//!
//! Handles dynamic window/tab title updates from OSC sequences and shell integration.

use std::path::Path;

/// Terminal title information from OSC sequences and shell integration
#[derive(Debug, Clone, Default)]
pub struct TitleInfo {
    /// OSC 0/2: Window title set by application
    pub window_title: Option<String>,
    /// OSC 1: Icon title (typically shorter version)
    pub icon_title: Option<String>,
    /// Current command being executed (from shell integration)
    pub current_command: Option<String>,
    /// Current working directory (from OSC 7)
    pub cwd: Option<String>,
}

impl TitleInfo {
    /// Create a new empty TitleInfo
    pub fn new() -> Self {
        Self::default()
    }

    /// Handle OSC (Operating System Command) sequences for title management
    ///
    /// # OSC Sequences
    /// - OSC 0 ; text ST - Set window title and icon name
    /// - OSC 1 ; text ST - Set icon name
    /// - OSC 2 ; text ST - Set window title
    /// - OSC 7 ; file://host/path ST - Set current working directory
    pub fn handle_osc(&mut self, command: u32, data: &str) {
        match command {
            0 => {
                // Set both window and icon title
                let title = data.to_string();
                self.window_title = Some(title.clone());
                self.icon_title = Some(title);
            }
            1 => {
                // Set icon title only
                self.icon_title = Some(data.to_string());
            }
            2 => {
                // Set window title only
                self.window_title = Some(data.to_string());
            }
            7 => {
                // Set current working directory (file://host/path format)
                if let Some(path) = parse_file_url(data) {
                    self.cwd = Some(path);
                }
            }
            _ => {
                // Unknown OSC command, ignore
            }
        }
    }

    /// Update the current command being executed
    ///
    /// This is typically called by shell integration scripts that notify
    /// the terminal before/after command execution.
    pub fn set_current_command(&mut self, command: Option<String>) {
        self.current_command = command;
    }

    /// Get the display title with priority order and optional truncation
    ///
    /// Priority order:
    /// 1. window_title (explicitly set by application via OSC 2/0)
    /// 2. current_command (shell integration showing active command)
    /// 3. default title (fallback)
    ///
    /// # Arguments
    /// * `default` - Default title if no other title is set
    /// * `max_length` - Maximum length of title (None for no limit)
    ///
    /// # Returns
    /// The title to display, truncated if necessary
    pub fn display_title(&self, default: &str, max_length: Option<usize>) -> String {
        let title = self
            .window_title
            .as_ref()
            .or(self.current_command.as_ref())
            .map(|s| s.as_str())
            .unwrap_or(default);

        // Truncate if necessary
        if let Some(max_len) = max_length {
            if title.len() > max_len {
                format!("{}...", &title[..max_len.saturating_sub(3)])
            } else {
                title.to_string()
            }
        } else {
            title.to_string()
        }
    }

    /// Get a short title suitable for tabs or icon names
    ///
    /// Uses icon_title if set, otherwise extracts filename from cwd or command.
    pub fn short_title(&self, default: &str) -> String {
        if let Some(icon) = &self.icon_title {
            return icon.clone();
        }

        // Try to extract something short from current command
        if let Some(cmd) = &self.current_command {
            // Get first word (command name)
            if let Some(cmd_name) = cmd.split_whitespace().next() {
                return cmd_name.to_string();
            }
        }

        // Try to extract directory name from cwd
        if let Some(cwd) = &self.cwd {
            if let Some(dir_name) = Path::new(cwd).file_name() {
                if let Some(name) = dir_name.to_str() {
                    return name.to_string();
                }
            }
        }

        default.to_string()
    }

    /// Get the current working directory
    pub fn cwd(&self) -> Option<&str> {
        self.cwd.as_deref()
    }

    /// Get the window title
    pub fn window_title(&self) -> Option<&str> {
        self.window_title.as_deref()
    }

    /// Get the icon title
    pub fn icon_title(&self) -> Option<&str> {
        self.icon_title.as_deref()
    }

    /// Get the current command
    pub fn current_command(&self) -> Option<&str> {
        self.current_command.as_deref()
    }

    /// Clear all title information
    pub fn clear(&mut self) {
        self.window_title = None;
        self.icon_title = None;
        self.current_command = None;
        self.cwd = None;
    }
}

/// Parse file:// URL to extract path
///
/// Supports formats:
/// - file://hostname/path
/// - file:///path (empty hostname = localhost)
///
/// # Examples
/// - file:///Users/name/project -> Some("/Users/name/project")
/// - file://localhost/tmp -> Some("/tmp")
/// - invalid URL -> None
fn parse_file_url(url: &str) -> Option<String> {
    let url = url.trim();

    // Must start with file://
    if !url.starts_with("file://") {
        return None;
    }

    // Remove file:// prefix
    let without_scheme = &url[7..];

    // Find the first slash after hostname
    // file:///path has empty hostname
    // file://hostname/path has hostname
    if let Some(path_start) = without_scheme.find('/') {
        let path = &without_scheme[path_start..];

        // URL decode the path
        match urlencoding::decode(path) {
            Ok(decoded) => Some(decoded.to_string()),
            Err(_) => Some(path.to_string()), // Fallback to non-decoded
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_info_new() {
        let info = TitleInfo::new();
        assert!(info.window_title.is_none());
        assert!(info.icon_title.is_none());
        assert!(info.current_command.is_none());
        assert!(info.cwd.is_none());
    }

    #[test]
    fn test_handle_osc_window_title() {
        let mut info = TitleInfo::new();
        info.handle_osc(2, "My Window");

        assert_eq!(info.window_title, Some("My Window".to_string()));
        assert!(info.icon_title.is_none());
    }

    #[test]
    fn test_handle_osc_icon_title() {
        let mut info = TitleInfo::new();
        info.handle_osc(1, "Icon");

        assert_eq!(info.icon_title, Some("Icon".to_string()));
        assert!(info.window_title.is_none());
    }

    #[test]
    fn test_handle_osc_both_titles() {
        let mut info = TitleInfo::new();
        info.handle_osc(0, "Both");

        assert_eq!(info.window_title, Some("Both".to_string()));
        assert_eq!(info.icon_title, Some("Both".to_string()));
    }

    #[test]
    fn test_handle_osc_cwd() {
        let mut info = TitleInfo::new();
        info.handle_osc(7, "file:///Users/test/project");

        assert_eq!(info.cwd, Some("/Users/test/project".to_string()));
    }

    #[test]
    fn test_display_title_priority() {
        let mut info = TitleInfo::new();

        // Default
        assert_eq!(info.display_title("Terminal", None), "Terminal");

        // Current command
        info.set_current_command(Some("vim".to_string()));
        assert_eq!(info.display_title("Terminal", None), "vim");

        // Window title takes priority
        info.handle_osc(2, "My Editor");
        assert_eq!(info.display_title("Terminal", None), "My Editor");
    }

    #[test]
    fn test_display_title_truncation() {
        let mut info = TitleInfo::new();
        info.handle_osc(2, "This is a very long title that should be truncated");

        let truncated = info.display_title("Terminal", Some(20));
        assert!(truncated.len() <= 20);
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_short_title_from_icon() {
        let mut info = TitleInfo::new();
        info.handle_osc(1, "Short");

        assert_eq!(info.short_title("Default"), "Short");
    }

    #[test]
    fn test_short_title_from_command() {
        let mut info = TitleInfo::new();
        info.set_current_command(Some("vim file.txt".to_string()));

        assert_eq!(info.short_title("Default"), "vim");
    }

    #[test]
    fn test_short_title_from_cwd() {
        let mut info = TitleInfo::new();
        info.handle_osc(7, "file:///Users/test/myproject");

        assert_eq!(info.short_title("Default"), "myproject");
    }

    #[test]
    fn test_parse_file_url_localhost() {
        assert_eq!(
            parse_file_url("file:///Users/test/project"),
            Some("/Users/test/project".to_string())
        );
    }

    #[test]
    fn test_parse_file_url_with_hostname() {
        assert_eq!(
            parse_file_url("file://localhost/tmp"),
            Some("/tmp".to_string())
        );
    }

    #[test]
    fn test_parse_file_url_with_spaces() {
        assert_eq!(
            parse_file_url("file:///path/with%20spaces"),
            Some("/path/with spaces".to_string())
        );
    }

    #[test]
    fn test_parse_file_url_invalid() {
        assert_eq!(parse_file_url("not a url"), None);
        assert_eq!(parse_file_url("http://example.com"), None);
        assert_eq!(parse_file_url("file://"), None);
    }

    #[test]
    fn test_clear() {
        let mut info = TitleInfo::new();
        info.handle_osc(0, "Title");
        info.set_current_command(Some("cmd".to_string()));
        info.handle_osc(7, "file:///path");

        info.clear();

        assert!(info.window_title.is_none());
        assert!(info.icon_title.is_none());
        assert!(info.current_command.is_none());
        assert!(info.cwd.is_none());
    }
}
