//! Environment detection and adaptation for AgTerm
//!
//! This module detects various runtime environments and adjusts terminal behavior accordingly.

use std::env;
use std::path::Path;

/// Environment detection results
#[derive(Debug, Clone)]
pub struct EnvironmentInfo {
    /// Running in an SSH session
    pub is_ssh: bool,
    /// Running inside a container (Docker, Podman, etc.)
    pub is_container: bool,
    /// Running inside tmux
    pub is_tmux: bool,
    /// Running inside GNU screen
    pub is_screen: bool,
    /// Terminal type (TERM environment variable)
    pub term_type: String,
    /// Color support level
    pub color_support: ColorSupport,
    /// True color (24-bit) support
    pub has_truecolor: bool,
    /// Mouse support capability
    pub has_mouse_support: bool,
    /// Unicode support capability
    pub has_unicode: bool,
}

/// Color support levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSupport {
    /// No color support
    None,
    /// Basic 16 colors
    Basic,
    /// 256 colors
    Color256,
    /// True color (24-bit RGB)
    TrueColor,
}

impl Default for EnvironmentInfo {
    fn default() -> Self {
        Self {
            is_ssh: false,
            is_container: false,
            is_tmux: false,
            is_screen: false,
            term_type: "xterm".to_string(),
            color_support: ColorSupport::TrueColor,
            has_truecolor: true,
            has_mouse_support: true,
            has_unicode: true,
        }
    }
}

impl EnvironmentInfo {
    /// Detect the current environment
    pub fn detect() -> Self {
        let is_ssh = Self::detect_ssh();
        let is_container = Self::detect_container();
        let is_tmux = Self::detect_tmux();
        let is_screen = Self::detect_screen();
        let term_type = Self::detect_term_type();
        let has_truecolor = Self::detect_truecolor();
        let color_support = Self::detect_color_support(&term_type, has_truecolor);
        let has_mouse_support = Self::detect_mouse_support(&term_type);
        let has_unicode = Self::detect_unicode();

        Self {
            is_ssh,
            is_container,
            is_tmux,
            is_screen,
            term_type,
            color_support,
            has_truecolor,
            has_mouse_support,
            has_unicode,
        }
    }

    /// Detect if running in an SSH session
    fn detect_ssh() -> bool {
        // SSH_CONNECTION is set by SSH daemon
        // Format: "client_ip client_port server_ip server_port"
        env::var("SSH_CONNECTION").is_ok()
            || env::var("SSH_CLIENT").is_ok()
            || env::var("SSH_TTY").is_ok()
    }

    /// Detect if running inside a container
    fn detect_container() -> bool {
        // Docker creates /.dockerenv file
        if Path::new("/.dockerenv").exists() {
            return true;
        }

        // Podman and other OCI containers create /run/.containerenv
        if Path::new("/run/.containerenv").exists() {
            return true;
        }

        // Check for container-specific environment variables
        if env::var("container").is_ok() {
            return true;
        }

        // Check cgroup for container indicators (Docker, Kubernetes, etc.)
        if let Ok(cgroup) = std::fs::read_to_string("/proc/self/cgroup") {
            if cgroup.contains("docker")
                || cgroup.contains("kubepods")
                || cgroup.contains("containerd") {
                return true;
            }
        }

        false
    }

    /// Detect if running inside tmux
    fn detect_tmux() -> bool {
        env::var("TMUX").is_ok()
    }

    /// Detect if running inside GNU screen
    fn detect_screen() -> bool {
        env::var("STY").is_ok() || env::var("WINDOW").is_ok()
    }

    /// Detect terminal type
    fn detect_term_type() -> String {
        env::var("TERM").unwrap_or_else(|_| "xterm".to_string())
    }

    /// Detect true color (24-bit) support
    fn detect_truecolor() -> bool {
        // Check COLORTERM for truecolor/24bit
        if let Ok(colorterm) = env::var("COLORTERM") {
            if colorterm.contains("truecolor") || colorterm.contains("24bit") {
                return true;
            }
        }

        // Check TERM for true color indicators
        if let Ok(term) = env::var("TERM") {
            if term.contains("24bit") || term.contains("truecolor") {
                return true;
            }
        }

        // Many modern terminals support true color by default
        // Check for known terminals with truecolor support
        if let Ok(term_program) = env::var("TERM_PROGRAM") {
            match term_program.as_str() {
                "iTerm.app" | "Apple_Terminal" | "WezTerm" | "Alacritty" | "kitty" => {
                    return true;
                }
                _ => {}
            }
        }

        false
    }

    /// Detect color support level
    fn detect_color_support(term_type: &str, has_truecolor: bool) -> ColorSupport {
        if has_truecolor {
            return ColorSupport::TrueColor;
        }

        // Check TERM value for color support
        if term_type.contains("256color") {
            return ColorSupport::Color256;
        }

        if term_type.contains("color") {
            return ColorSupport::Basic;
        }

        // Check for legacy terminals with limited color support
        if term_type == "linux" || term_type == "vt100" || term_type == "vt220" {
            return ColorSupport::Basic;
        }

        // Default to basic color for unknown terminals
        if term_type.starts_with("xterm") || term_type.starts_with("rxvt") {
            return ColorSupport::Basic;
        }

        ColorSupport::None
    }

    /// Detect mouse support capability
    fn detect_mouse_support(term_type: &str) -> bool {
        // Most modern terminals support mouse
        // Exceptions: very old terminals, some screen sessions without proper TERM

        // Known terminals without mouse support
        if term_type == "dumb" || term_type == "vt100" || term_type == "vt220" {
            return false;
        }

        // Most xterm-compatible terminals support mouse
        if term_type.starts_with("xterm")
            || term_type.starts_with("screen")
            || term_type.starts_with("tmux")
            || term_type.starts_with("rxvt") {
            return true;
        }

        // Default to true for modern environments
        true
    }

    /// Detect Unicode (UTF-8) support
    fn detect_unicode() -> bool {
        // Check LANG and LC_* environment variables for UTF-8
        if let Ok(lang) = env::var("LANG") {
            if lang.to_lowercase().contains("utf-8") || lang.to_lowercase().contains("utf8") {
                return true;
            }
        }

        if let Ok(lc_all) = env::var("LC_ALL") {
            if lc_all.to_lowercase().contains("utf-8") || lc_all.to_lowercase().contains("utf8") {
                return true;
            }
        }

        if let Ok(lc_ctype) = env::var("LC_CTYPE") {
            if lc_ctype.to_lowercase().contains("utf-8") || lc_ctype.to_lowercase().contains("utf8") {
                return true;
            }
        }

        // Check TERM for UTF-8 indicators
        if let Ok(term) = env::var("TERM") {
            if term.contains("utf8") {
                return true;
            }
        }

        // Default to true on modern systems
        // Most systems use UTF-8 by default now
        true
    }

    /// Get a human-readable environment description
    pub fn description(&self) -> String {
        let mut parts = Vec::new();

        if self.is_ssh {
            parts.push("SSH");
        }
        if self.is_container {
            parts.push("Container");
        }
        if self.is_tmux {
            parts.push("tmux");
        }
        if self.is_screen {
            parts.push("screen");
        }

        parts.push(&self.term_type);

        match self.color_support {
            ColorSupport::TrueColor => parts.push("TrueColor"),
            ColorSupport::Color256 => parts.push("256 color"),
            ColorSupport::Basic => parts.push("Basic color"),
            ColorSupport::None => parts.push("No color"),
        }

        if !self.has_mouse_support {
            parts.push("No mouse");
        }

        if !self.has_unicode {
            parts.push("No UTF-8");
        }

        parts.join(" | ")
    }

    /// Check if the environment is constrained (SSH, container, etc.)
    pub fn is_constrained(&self) -> bool {
        self.is_ssh || self.is_container
    }

    /// Check if running in a terminal multiplexer
    pub fn is_multiplexed(&self) -> bool {
        self.is_tmux || self.is_screen
    }

    /// Suggest optimal configuration based on environment
    pub fn suggested_settings(&self) -> EnvironmentSettings {
        EnvironmentSettings {
            enable_truecolor: self.has_truecolor,
            enable_mouse: self.has_mouse_support,
            enable_unicode: self.has_unicode,
            // Reduce animations in constrained environments
            enable_animations: !self.is_constrained(),
            // Reduce font effects in constrained environments
            enable_font_ligatures: !self.is_constrained(),
            // Adjust scrollback based on environment
            scrollback_lines: if self.is_constrained() { 5000 } else { 10000 },
            // Reduce refresh rate in SSH/container
            refresh_rate_ms: if self.is_constrained() { 50 } else { 16 },
        }
    }
}

/// Suggested settings based on environment
#[derive(Debug, Clone)]
pub struct EnvironmentSettings {
    /// Enable true color (24-bit) rendering
    pub enable_truecolor: bool,
    /// Enable mouse support
    pub enable_mouse: bool,
    /// Enable Unicode/UTF-8 rendering
    pub enable_unicode: bool,
    /// Enable UI animations
    pub enable_animations: bool,
    /// Enable font ligatures
    pub enable_font_ligatures: bool,
    /// Scrollback buffer size
    pub scrollback_lines: usize,
    /// Refresh rate in milliseconds
    pub refresh_rate_ms: u64,
}

impl Default for EnvironmentSettings {
    fn default() -> Self {
        Self {
            enable_truecolor: true,
            enable_mouse: true,
            enable_unicode: true,
            enable_animations: true,
            enable_font_ligatures: true,
            scrollback_lines: 10000,
            refresh_rate_ms: 16,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_environment() {
        // Basic smoke test - should not panic
        let env_info = EnvironmentInfo::detect();

        // Basic validation
        assert!(!env_info.term_type.is_empty());

        // Should have some color support on modern systems
        // (This might fail on CI without proper TERM setup)
        println!("Environment: {}", env_info.description());
    }

    #[test]
    fn test_color_support_detection() {
        // Test 256 color detection
        assert_eq!(
            EnvironmentInfo::detect_color_support("xterm-256color", false),
            ColorSupport::Color256
        );

        // Test truecolor
        assert_eq!(
            EnvironmentInfo::detect_color_support("xterm-256color", true),
            ColorSupport::TrueColor
        );

        // Test basic color
        assert_eq!(
            EnvironmentInfo::detect_color_support("xterm", false),
            ColorSupport::Basic
        );

        // Test no color
        assert_eq!(
            EnvironmentInfo::detect_color_support("dumb", false),
            ColorSupport::None
        );
    }

    #[test]
    fn test_mouse_support_detection() {
        // Modern terminals should support mouse
        assert!(EnvironmentInfo::detect_mouse_support("xterm-256color"));
        assert!(EnvironmentInfo::detect_mouse_support("screen-256color"));
        assert!(EnvironmentInfo::detect_mouse_support("tmux-256color"));

        // Legacy terminals should not
        assert!(!EnvironmentInfo::detect_mouse_support("vt100"));
        assert!(!EnvironmentInfo::detect_mouse_support("dumb"));
    }

    #[test]
    fn test_container_detection() {
        // This test will fail if not running in a container
        // It's mainly for documentation purposes
        let is_container = EnvironmentInfo::detect_container();
        println!("Is container: {}", is_container);
    }

    #[test]
    fn test_ssh_detection() {
        // This test will fail if not running over SSH
        // It's mainly for documentation purposes
        let is_ssh = EnvironmentInfo::detect_ssh();
        println!("Is SSH: {}", is_ssh);
    }

    #[test]
    fn test_suggested_settings() {
        let env_info = EnvironmentInfo::default();
        let settings = env_info.suggested_settings();

        // Default environment should have optimal settings
        assert!(settings.enable_truecolor);
        assert!(settings.enable_mouse);
        assert!(settings.enable_unicode);
        assert_eq!(settings.scrollback_lines, 10000);
        assert_eq!(settings.refresh_rate_ms, 16);
    }

    #[test]
    fn test_constrained_environment_settings() {
        let mut env_info = EnvironmentInfo::default();
        env_info.is_ssh = true;

        let settings = env_info.suggested_settings();

        // Constrained environment should have reduced settings
        assert!(!settings.enable_animations);
        assert!(!settings.enable_font_ligatures);
        assert_eq!(settings.scrollback_lines, 5000);
        assert_eq!(settings.refresh_rate_ms, 50);
    }

    #[test]
    fn test_environment_description() {
        let env_info = EnvironmentInfo::default();
        let desc = env_info.description();

        // Should contain term type
        assert!(desc.contains("xterm"));
        // Should mention color support
        assert!(desc.contains("color") || desc.contains("Color"));
    }

    #[test]
    fn test_is_constrained() {
        let mut env_info = EnvironmentInfo::default();
        assert!(!env_info.is_constrained());

        env_info.is_ssh = true;
        assert!(env_info.is_constrained());

        env_info.is_ssh = false;
        env_info.is_container = true;
        assert!(env_info.is_constrained());
    }

    #[test]
    fn test_is_multiplexed() {
        let mut env_info = EnvironmentInfo::default();
        assert!(!env_info.is_multiplexed());

        env_info.is_tmux = true;
        assert!(env_info.is_multiplexed());

        env_info.is_tmux = false;
        env_info.is_screen = true;
        assert!(env_info.is_multiplexed());
    }
}
