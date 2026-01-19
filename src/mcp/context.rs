//! Terminal context provider for MCP integration
//!
//! Provides rich contextual information about the terminal state to MCP clients.

use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

/// Terminal context for MCP protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalContext {
    /// Current working directory
    pub cwd: PathBuf,
    /// Shell type (bash, zsh, fish, etc.)
    pub shell: String,
    /// Recent terminal output (last N lines)
    pub recent_output: Vec<String>,
    /// Current user input
    pub current_input: String,
    /// Environment variables
    pub environment: ContextEnvironment,
    /// Git repository information (if in a git repo)
    pub git_info: Option<GitInfo>,
    /// Running processes
    pub processes: Vec<ProcessInfo>,
    /// Terminal dimensions
    pub dimensions: TerminalDimensions,
    /// Current selection (if any)
    pub selection: Option<String>,
}

/// Environment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEnvironment {
    /// Current user
    pub user: String,
    /// Home directory
    pub home: PathBuf,
    /// Language/locale
    pub lang: String,
    /// Terminal type
    pub term: String,
    /// Editor preference
    pub editor: Option<String>,
    /// Shell path
    pub shell: PathBuf,
}

/// Git repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    /// Current branch name
    pub branch: String,
    /// Whether there are uncommitted changes
    pub is_dirty: bool,
    /// Remote URL (if configured)
    pub remote_url: Option<String>,
    /// Ahead/behind remote
    pub ahead_behind: Option<(usize, usize)>,
}

/// Process information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Process name
    pub name: String,
    /// Full command line
    pub command: String,
}

/// Terminal dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalDimensions {
    /// Number of columns
    pub cols: u16,
    /// Number of rows
    pub rows: u16,
}

impl TerminalContext {
    /// Create a new terminal context
    pub fn new(cwd: PathBuf, shell: String) -> Self {
        Self {
            cwd,
            shell,
            recent_output: Vec::new(),
            current_input: String::new(),
            environment: ContextEnvironment::detect(),
            git_info: None,
            processes: Vec::new(),
            dimensions: TerminalDimensions { cols: 80, rows: 24 },
            selection: None,
        }
    }

    /// Convert context to a formatted string for MCP messages
    pub fn to_context_string(&self) -> String {
        let mut context = String::new();

        // Working directory
        context.push_str(&format!("Working Directory: {}\n", self.cwd.display()));

        // Shell
        context.push_str(&format!("Shell: {}\n", self.shell));

        // Git info
        if let Some(ref git) = self.git_info {
            context.push_str(&format!("Git Branch: {}\n", git.branch));
            if git.is_dirty {
                context.push_str("Git Status: Uncommitted changes\n");
            }
            if let Some(ref remote) = git.remote_url {
                context.push_str(&format!("Git Remote: {}\n", remote));
            }
            if let Some((ahead, behind)) = git.ahead_behind {
                if ahead > 0 || behind > 0 {
                    context.push_str(&format!(
                        "Git Sync: {} ahead, {} behind\n",
                        ahead, behind
                    ));
                }
            }
        }

        // Environment
        context.push_str(&format!("User: {}\n", self.environment.user));
        context.push_str(&format!("Home: {}\n", self.environment.home.display()));
        if let Some(ref editor) = self.environment.editor {
            context.push_str(&format!("Editor: {}\n", editor));
        }

        // Terminal dimensions
        context.push_str(&format!(
            "Terminal Size: {}x{}\n",
            self.dimensions.cols, self.dimensions.rows
        ));

        // Current input
        if !self.current_input.is_empty() {
            context.push_str(&format!("Current Input: {}\n", self.current_input));
        }

        // Selection
        if let Some(ref sel) = self.selection {
            context.push_str(&format!("Selection: {}\n", sel));
        }

        // Recent output (last 10 lines)
        if !self.recent_output.is_empty() {
            context.push_str("\nRecent Output:\n");
            let start = self.recent_output.len().saturating_sub(10);
            for line in &self.recent_output[start..] {
                context.push_str(&format!("  {}\n", line));
            }
        }

        // Running processes
        if !self.processes.is_empty() {
            context.push_str(&format!("\nRunning Processes: {}\n", self.processes.len()));
            for proc in self.processes.iter().take(5) {
                context.push_str(&format!("  {} ({}): {}\n", proc.pid, proc.name, proc.command));
            }
        }

        context
    }

    /// Detect git repository information
    pub fn detect_git_info(&mut self) {
        self.git_info = Self::try_detect_git(&self.cwd);
    }

    fn try_detect_git(cwd: &PathBuf) -> Option<GitInfo> {
        use std::process::Command;

        // Check if we're in a git repository
        let output = Command::new("git")
            .args(&["rev-parse", "--is-inside-work-tree"])
            .current_dir(cwd)
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        // Get branch name
        let branch_output = Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(cwd)
            .output()
            .ok()?;

        let branch = String::from_utf8_lossy(&branch_output.stdout)
            .trim()
            .to_string();

        // Check if dirty
        let status_output = Command::new("git")
            .args(&["status", "--porcelain"])
            .current_dir(cwd)
            .output()
            .ok()?;

        let is_dirty = !status_output.stdout.is_empty();

        // Get remote URL
        let remote_output = Command::new("git")
            .args(&["config", "--get", "remote.origin.url"])
            .current_dir(cwd)
            .output()
            .ok();

        let remote_url = remote_output.and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        });

        // Get ahead/behind info
        let ahead_behind = Self::get_ahead_behind(cwd);

        Some(GitInfo {
            branch,
            is_dirty,
            remote_url,
            ahead_behind,
        })
    }

    fn get_ahead_behind(cwd: &PathBuf) -> Option<(usize, usize)> {
        use std::process::Command;

        let output = Command::new("git")
            .args(&["rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
            .current_dir(cwd)
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = output_str.trim().split_whitespace().collect();

        if parts.len() == 2 {
            let ahead = parts[0].parse().ok()?;
            let behind = parts[1].parse().ok()?;
            Some((ahead, behind))
        } else {
            None
        }
    }

    /// Add a line to recent output
    pub fn add_output_line(&mut self, line: String) {
        self.recent_output.push(line);
        // Keep only last 100 lines
        if self.recent_output.len() > 100 {
            self.recent_output.remove(0);
        }
    }

    /// Update current input
    pub fn update_input(&mut self, input: String) {
        self.current_input = input;
    }

    /// Update terminal dimensions
    pub fn update_dimensions(&mut self, cols: u16, rows: u16) {
        self.dimensions = TerminalDimensions { cols, rows };
    }

    /// Set selection text
    pub fn set_selection(&mut self, selection: Option<String>) {
        self.selection = selection;
    }
}

impl ContextEnvironment {
    /// Detect environment information from system
    pub fn detect() -> Self {
        let user = env::var("USER")
            .or_else(|_| env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());

        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));

        let lang = env::var("LANG").unwrap_or_else(|_| "C".to_string());

        let term = env::var("TERM").unwrap_or_else(|_| "xterm-256color".to_string());

        let editor = env::var("EDITOR").ok().or_else(|| env::var("VISUAL").ok());

        let shell = env::var("SHELL")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/bin/sh"));

        Self {
            user,
            home,
            lang,
            term,
            editor,
            shell,
        }
    }
}

impl Default for TerminalContext {
    fn default() -> Self {
        Self::new(
            env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let cwd = PathBuf::from("/tmp");
        let shell = "bash".to_string();
        let context = TerminalContext::new(cwd.clone(), shell.clone());

        assert_eq!(context.cwd, cwd);
        assert_eq!(context.shell, shell);
        assert_eq!(context.recent_output.len(), 0);
        assert_eq!(context.dimensions.cols, 80);
        assert_eq!(context.dimensions.rows, 24);
    }

    #[test]
    fn test_add_output_line() {
        let mut context = TerminalContext::default();

        context.add_output_line("line 1".to_string());
        context.add_output_line("line 2".to_string());

        assert_eq!(context.recent_output.len(), 2);
        assert_eq!(context.recent_output[0], "line 1");
        assert_eq!(context.recent_output[1], "line 2");
    }

    #[test]
    fn test_output_line_limit() {
        let mut context = TerminalContext::default();

        // Add 150 lines
        for i in 0..150 {
            context.add_output_line(format!("line {}", i));
        }

        // Should keep only last 100
        assert_eq!(context.recent_output.len(), 100);
        assert_eq!(context.recent_output[0], "line 50");
        assert_eq!(context.recent_output[99], "line 149");
    }

    #[test]
    fn test_update_input() {
        let mut context = TerminalContext::default();
        context.update_input("ls -la".to_string());

        assert_eq!(context.current_input, "ls -la");
    }

    #[test]
    fn test_update_dimensions() {
        let mut context = TerminalContext::default();
        context.update_dimensions(120, 40);

        assert_eq!(context.dimensions.cols, 120);
        assert_eq!(context.dimensions.rows, 40);
    }

    #[test]
    fn test_set_selection() {
        let mut context = TerminalContext::default();
        context.set_selection(Some("selected text".to_string()));

        assert!(context.selection.is_some());
        assert_eq!(context.selection.unwrap(), "selected text");
    }

    #[test]
    fn test_environment_detect() {
        let env = ContextEnvironment::detect();

        // Basic checks - should have some values
        assert!(!env.user.is_empty());
        assert!(env.home.exists() || env.home == PathBuf::from("/"));
        assert!(!env.term.is_empty());
    }

    #[test]
    fn test_context_string_format() {
        let mut context = TerminalContext::new(
            PathBuf::from("/home/user/project"),
            "zsh".to_string(),
        );

        context.update_input("git status".to_string());
        context.add_output_line("On branch main".to_string());

        let context_str = context.to_context_string();

        assert!(context_str.contains("Working Directory: /home/user/project"));
        assert!(context_str.contains("Shell: zsh"));
        assert!(context_str.contains("Current Input: git status"));
        assert!(context_str.contains("Recent Output:"));
        assert!(context_str.contains("On branch main"));
    }

    #[test]
    fn test_git_info_serialization() {
        let git_info = GitInfo {
            branch: "main".to_string(),
            is_dirty: true,
            remote_url: Some("https://github.com/user/repo.git".to_string()),
            ahead_behind: Some((2, 1)),
        };

        let json = serde_json::to_string(&git_info).unwrap();
        let deserialized: GitInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.branch, "main");
        assert!(deserialized.is_dirty);
        assert_eq!(
            deserialized.remote_url,
            Some("https://github.com/user/repo.git".to_string())
        );
        assert_eq!(deserialized.ahead_behind, Some((2, 1)));
    }

    #[test]
    fn test_context_with_git_info() {
        let mut context = TerminalContext::default();
        context.git_info = Some(GitInfo {
            branch: "feature/test".to_string(),
            is_dirty: true,
            remote_url: Some("git@github.com:user/repo.git".to_string()),
            ahead_behind: Some((3, 0)),
        });

        let context_str = context.to_context_string();

        assert!(context_str.contains("Git Branch: feature/test"));
        assert!(context_str.contains("Git Status: Uncommitted changes"));
        assert!(context_str.contains("Git Remote: git@github.com:user/repo.git"));
        assert!(context_str.contains("Git Sync: 3 ahead, 0 behind"));
    }

    #[test]
    fn test_process_info() {
        let process = ProcessInfo {
            pid: 12345,
            name: "rust-analyzer".to_string(),
            command: "rust-analyzer server".to_string(),
        };

        assert_eq!(process.pid, 12345);
        assert_eq!(process.name, "rust-analyzer");
        assert_eq!(process.command, "rust-analyzer server");
    }

    #[test]
    fn test_context_with_processes() {
        let mut context = TerminalContext::default();
        context.processes.push(ProcessInfo {
            pid: 1001,
            name: "vim".to_string(),
            command: "vim main.rs".to_string(),
        });
        context.processes.push(ProcessInfo {
            pid: 1002,
            name: "cargo".to_string(),
            command: "cargo build".to_string(),
        });

        let context_str = context.to_context_string();

        assert!(context_str.contains("Running Processes: 2"));
        assert!(context_str.contains("1001 (vim): vim main.rs"));
        assert!(context_str.contains("1002 (cargo): cargo build"));
    }
}
