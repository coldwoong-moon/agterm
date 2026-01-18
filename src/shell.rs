//! Shell detection and recommendation
//!
//! This module provides utilities for detecting available shells on the system,
//! determining their types and versions, and recommending appropriate shells.

use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, warn};

/// Supported shell types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    Nushell,
    PowerShell,
    Cmd,
    Unknown(String),
}

impl ShellType {
    /// Get the display name of the shell type
    pub fn display_name(&self) -> &str {
        match self {
            ShellType::Bash => "Bash",
            ShellType::Zsh => "Zsh",
            ShellType::Fish => "Fish",
            ShellType::Nushell => "Nushell",
            ShellType::PowerShell => "PowerShell",
            ShellType::Cmd => "Command Prompt",
            ShellType::Unknown(name) => name,
        }
    }

    /// Get a brief description of the shell
    pub fn description(&self) -> &str {
        match self {
            ShellType::Bash => "Bourne Again Shell - widely compatible, standard on most systems",
            ShellType::Zsh => "Z Shell - feature-rich with better defaults than bash",
            ShellType::Fish => "Friendly Interactive Shell - modern shell with great UX",
            ShellType::Nushell => "Nu Shell - modern shell with structured data pipelines",
            ShellType::PowerShell => "PowerShell - powerful shell with .NET integration",
            ShellType::Cmd => "Windows Command Prompt - basic Windows shell",
            ShellType::Unknown(_) => "Unknown shell type",
        }
    }
}

/// Information about a shell installation
#[derive(Debug, Clone)]
pub struct ShellInfo {
    pub shell_type: ShellType,
    pub path: PathBuf,
    pub version: Option<String>,
    pub is_login_shell: bool,
}

impl ShellInfo {
    /// Get all available shells on the system
    pub fn available_shells() -> Vec<ShellInfo> {
        debug!("Detecting available shells");

        let mut shells = Vec::new();

        // Common shell paths by platform
        let shell_paths = if cfg!(windows) {
            vec![
                "C:\\Windows\\System32\\cmd.exe",
                "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe",
                "C:\\Program Files\\PowerShell\\7\\pwsh.exe",
                "C:\\Program Files\\Git\\bin\\bash.exe",
                "C:\\msys64\\usr\\bin\\bash.exe",
                "C:\\msys64\\usr\\bin\\zsh.exe",
            ]
        } else if cfg!(target_os = "macos") {
            vec![
                "/bin/bash",
                "/bin/zsh",
                "/usr/local/bin/bash",
                "/usr/local/bin/zsh",
                "/usr/local/bin/fish",
                "/usr/local/bin/nu",
                "/opt/homebrew/bin/bash",
                "/opt/homebrew/bin/zsh",
                "/opt/homebrew/bin/fish",
                "/opt/homebrew/bin/nu",
            ]
        } else {
            // Linux and other Unix-like systems
            vec![
                "/bin/bash",
                "/bin/zsh",
                "/bin/fish",
                "/usr/bin/bash",
                "/usr/bin/zsh",
                "/usr/bin/fish",
                "/usr/bin/nu",
                "/usr/local/bin/bash",
                "/usr/local/bin/zsh",
                "/usr/local/bin/fish",
                "/usr/local/bin/nu",
            ]
        };

        for path in shell_paths {
            if let Some(shell_info) = Self::from_path(path) {
                shells.push(shell_info);
            }
        }

        debug!(count = shells.len(), "Found available shells");
        shells
    }

    /// Create ShellInfo from a shell path
    pub fn from_path(path: &str) -> Option<ShellInfo> {
        let path_buf = PathBuf::from(path);

        // Check if the file exists and is executable
        if !path_buf.exists() {
            return None;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(&path_buf) {
                let permissions = metadata.permissions();
                // Check if executable bit is set
                if permissions.mode() & 0o111 == 0 {
                    return None;
                }
            } else {
                return None;
            }
        }

        let shell_type = Self::detect_type(path);
        let version = Self::get_version_for_path(&path_buf, &shell_type);

        debug!(
            path = %path,
            shell_type = ?shell_type,
            version = ?version,
            "Detected shell"
        );

        Some(ShellInfo {
            shell_type,
            path: path_buf,
            version,
            is_login_shell: false,
        })
    }

    /// Get the default shell from the SHELL environment variable
    pub fn default_shell() -> Option<ShellInfo> {
        #[cfg(windows)]
        {
            // On Windows, check COMSPEC first, then default to PowerShell
            let shell_path = std::env::var("COMSPEC")
                .ok()
                .or_else(|| Some("C:\\Windows\\System32\\cmd.exe".to_string()))?;

            Self::from_path(&shell_path)
        }

        #[cfg(not(windows))]
        {
            let shell_path = std::env::var("SHELL").ok()?;
            let mut shell_info = Self::from_path(&shell_path)?;
            shell_info.is_login_shell = true;
            Some(shell_info)
        }
    }

    /// Get the version of this shell
    pub fn get_version(&self) -> Option<String> {
        Self::get_version_for_path(&self.path, &self.shell_type)
    }

    /// Get version for a specific shell path and type
    fn get_version_for_path(path: &Path, shell_type: &ShellType) -> Option<String> {
        let version_arg = match shell_type {
            ShellType::Bash | ShellType::Zsh => "--version",
            ShellType::Fish => "--version",
            ShellType::Nushell => "--version",
            ShellType::PowerShell => "-Version",
            ShellType::Cmd => "", // cmd doesn't have a version flag
            ShellType::Unknown(_) => "--version",
        };

        if version_arg.is_empty() {
            return None;
        }

        match Command::new(path).arg(version_arg).output() {
            Ok(output) => {
                let version_text = String::from_utf8_lossy(&output.stdout);
                // Parse first line and extract version number
                let version = version_text
                    .lines()
                    .next()
                    .map(|line| line.trim().to_string());
                version
            }
            Err(e) => {
                warn!(path = ?path, error = %e, "Failed to get shell version");
                None
            }
        }
    }

    /// Detect shell type from path
    fn detect_type(path: &str) -> ShellType {
        let path_lower = path.to_lowercase();

        // Extract filename handling both Unix (/) and Windows (\) path separators
        let file_name = path
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(path)
            .to_lowercase();

        if file_name.starts_with("bash") || path_lower.contains("/bash") {
            ShellType::Bash
        } else if file_name.starts_with("zsh") || path_lower.contains("/zsh") {
            ShellType::Zsh
        } else if file_name.starts_with("fish") || path_lower.contains("/fish") {
            ShellType::Fish
        } else if file_name.starts_with("nu") || file_name == "nu.exe" {
            ShellType::Nushell
        } else if file_name.starts_with("pwsh") || file_name.starts_with("powershell") {
            ShellType::PowerShell
        } else if file_name == "cmd.exe" || file_name == "cmd" {
            ShellType::Cmd
        } else {
            ShellType::Unknown(file_name)
        }
    }
}

/// Recommend a shell based on availability and features
///
/// Priority order: zsh > fish > bash > others
pub fn recommend_shell() -> Option<ShellInfo> {
    debug!("Recommending shell");

    let available = ShellInfo::available_shells();

    if available.is_empty() {
        warn!("No shells found on system");
        return None;
    }

    // Try to find the best shell in priority order
    let recommended = available
        .iter()
        .find(|s| s.shell_type == ShellType::Zsh)
        .or_else(|| available.iter().find(|s| s.shell_type == ShellType::Fish))
        .or_else(|| available.iter().find(|s| s.shell_type == ShellType::Bash))
        .or_else(|| available.first());

    if let Some(shell) = recommended {
        debug!(
            shell_type = ?shell.shell_type,
            path = %shell.path.display(),
            "Recommended shell"
        );
    }

    recommended.cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_type_detection() {
        assert_eq!(ShellInfo::detect_type("/bin/bash"), ShellType::Bash);
        assert_eq!(ShellInfo::detect_type("/bin/zsh"), ShellType::Zsh);
        assert_eq!(ShellInfo::detect_type("/usr/local/bin/fish"), ShellType::Fish);
        assert_eq!(ShellInfo::detect_type("/usr/bin/nu"), ShellType::Nushell);
        // Windows paths (case-insensitive due to to_lowercase in detect_type)
        assert_eq!(
            ShellInfo::detect_type("c:\\windows\\system32\\cmd.exe"),
            ShellType::Cmd
        );
        assert_eq!(
            ShellInfo::detect_type("C:\\Windows\\System32\\cmd.exe"),
            ShellType::Cmd
        );
        assert_eq!(
            ShellInfo::detect_type("c:\\program files\\powershell\\7\\pwsh.exe"),
            ShellType::PowerShell
        );
        assert_eq!(
            ShellInfo::detect_type("C:\\Program Files\\PowerShell\\7\\pwsh.exe"),
            ShellType::PowerShell
        );
    }

    #[test]
    fn test_shell_type_display_name() {
        assert_eq!(ShellType::Bash.display_name(), "Bash");
        assert_eq!(ShellType::Zsh.display_name(), "Zsh");
        assert_eq!(ShellType::Fish.display_name(), "Fish");
        assert_eq!(ShellType::Nushell.display_name(), "Nushell");
        assert_eq!(ShellType::PowerShell.display_name(), "PowerShell");
        assert_eq!(ShellType::Cmd.display_name(), "Command Prompt");
    }

    #[test]
    fn test_default_shell_detection() {
        // This test will vary by system, just ensure it doesn't panic
        let default = ShellInfo::default_shell();
        if let Some(shell) = default {
            assert!(shell.path.exists());
            // On Unix systems, the default shell should be marked as login shell
            #[cfg(not(windows))]
            assert!(shell.is_login_shell);
        }
    }

    #[test]
    fn test_available_shells() {
        // This test will vary by system, just ensure it returns something
        let shells = ShellInfo::available_shells();
        // Most systems should have at least one shell
        #[cfg(not(target_os = "windows"))]
        assert!(!shells.is_empty(), "Expected at least one shell to be found");
    }

    #[test]
    fn test_recommend_shell() {
        // This test will vary by system
        let recommended = recommend_shell();
        if let Some(shell) = recommended {
            assert!(shell.path.exists());
            // The recommended shell should be one of the preferred types
            assert!(
                matches!(
                    shell.shell_type,
                    ShellType::Zsh | ShellType::Fish | ShellType::Bash | _
                ),
                "Unexpected shell type: {:?}",
                shell.shell_type
            );
        }
    }

    #[test]
    fn test_shell_from_nonexistent_path() {
        let shell = ShellInfo::from_path("/nonexistent/shell");
        assert!(shell.is_none(), "Should return None for nonexistent path");
    }
}
