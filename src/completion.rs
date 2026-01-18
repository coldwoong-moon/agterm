//! Auto-completion engine for AgTerm
//!
//! Provides intelligent completion suggestions for:
//! - Commands (from PATH)
//! - Files and directories
//! - Command history

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionItem {
    pub text: String,
    pub kind: CompletionKind,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Command,   // Executable/command
    File,      // File
    Directory, // Directory
    History,   // From history
    Alias,     // Shell alias
}

impl CompletionKind {
    /// Get display prefix for the completion kind
    pub fn prefix(&self) -> &str {
        match self {
            CompletionKind::Command => "cmd",
            CompletionKind::File => "file",
            CompletionKind::Directory => "dir",
            CompletionKind::History => "hist",
            CompletionKind::Alias => "alias",
        }
    }
}

pub struct CompletionEngine {
    history: Vec<String>,
    path_commands: HashSet<String>,
    max_history: usize,
}

impl CompletionEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            history: Vec::new(),
            path_commands: HashSet::new(),
            max_history: 1000,
        };
        engine.load_path_commands();
        engine
    }

    /// Load commands from PATH environment variable
    pub fn load_path_commands(&mut self) {
        self.path_commands.clear();

        let path_var = match env::var("PATH") {
            Ok(p) => p,
            Err(_) => return,
        };

        let separator = if cfg!(windows) { ';' } else { ':' };

        for dir_str in path_var.split(separator) {
            let dir = Path::new(dir_str);
            if !dir.is_dir() {
                continue;
            }

            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        // Check if it's executable
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 {
                                if let Some(name) = entry.file_name().to_str() {
                                    self.path_commands.insert(name.to_string());
                                }
                            }
                        }

                        #[cfg(windows)]
                        {
                            if metadata.is_file() {
                                if let Some(name) = entry.file_name().to_str() {
                                    // On Windows, check for .exe, .bat, .cmd extensions
                                    if name.ends_with(".exe")
                                        || name.ends_with(".bat")
                                        || name.ends_with(".cmd")
                                    {
                                        // Strip extension for completion
                                        let name_no_ext =
                                            name.rsplit_once('.').map(|(n, _)| n).unwrap_or(name);
                                        self.path_commands.insert(name_no_ext.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        tracing::debug!(
            "Loaded {} commands from PATH",
            self.path_commands.len()
        );
    }

    /// Add a command to history
    pub fn add_to_history(&mut self, command: &str) {
        let command = command.trim();
        if command.is_empty() {
            return;
        }

        // Remove if already exists (to move to front)
        self.history.retain(|h| h != command);

        // Add to front
        self.history.insert(0, command.to_string());

        // Trim to max size
        if self.history.len() > self.max_history {
            self.history.truncate(self.max_history);
        }
    }

    /// Generate completion suggestions for the given input
    pub fn complete(&self, input: &str, cwd: &str) -> Vec<CompletionItem> {
        if input.is_empty() {
            return self.recent_history(10);
        }

        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.len() <= 1 {
            // Command completion
            self.complete_command(input)
        } else {
            // Argument completion (files/directories)
            let last = parts.last().unwrap_or(&"");
            self.complete_path(last, cwd)
        }
    }

    /// Complete command names
    fn complete_command(&self, prefix: &str) -> Vec<CompletionItem> {
        let mut items = Vec::new();
        let prefix_lower = prefix.to_lowercase();

        // First, check history for matching commands
        for hist in &self.history {
            let hist_cmd = hist.split_whitespace().next().unwrap_or("");
            if hist_cmd.to_lowercase().starts_with(&prefix_lower) {
                items.push(CompletionItem {
                    text: hist.clone(),
                    kind: CompletionKind::History,
                    description: Some("from history".to_string()),
                });

                if items.len() >= 5 {
                    break;
                }
            }
        }

        // Then add matching PATH commands
        let mut path_matches: Vec<_> = self
            .path_commands
            .iter()
            .filter(|cmd| cmd.to_lowercase().starts_with(&prefix_lower))
            .cloned()
            .collect();

        path_matches.sort();

        for cmd in path_matches.into_iter().take(20) {
            items.push(CompletionItem {
                text: cmd,
                kind: CompletionKind::Command,
                description: None,
            });
        }

        items
    }

    /// Complete file paths
    fn complete_path(&self, prefix: &str, cwd: &str) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        // Determine the directory to search and the filename prefix
        let (search_dir, file_prefix) = if prefix.contains('/') || prefix.contains('\\') {
            let path = PathBuf::from(prefix);
            let dir = if prefix.ends_with('/') || prefix.ends_with('\\') {
                path.clone()
            } else {
                path.parent().unwrap_or(Path::new("")).to_path_buf()
            };
            let file = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            (dir, file)
        } else {
            (PathBuf::from(cwd), prefix.to_string())
        };

        // Resolve relative paths
        let search_dir = if search_dir.is_relative() {
            PathBuf::from(cwd).join(search_dir)
        } else {
            search_dir
        };

        // Read directory entries
        if let Ok(entries) = fs::read_dir(&search_dir) {
            let mut matches: Vec<_> = entries
                .flatten()
                .filter_map(|entry| {
                    let file_name = entry.file_name();
                    let name = file_name.to_str()?;

                    // Filter by prefix
                    if !name.to_lowercase().starts_with(&file_prefix.to_lowercase()) {
                        return None;
                    }

                    let metadata = entry.metadata().ok()?;
                    let kind = if metadata.is_dir() {
                        CompletionKind::Directory
                    } else {
                        CompletionKind::File
                    };

                    // Get full path relative to original prefix
                    let full_text = if prefix.contains('/') || prefix.contains('\\') {
                        let parent = PathBuf::from(prefix)
                            .parent()
                            .unwrap_or(Path::new(""))
                            .to_path_buf();
                        let mut full = parent.join(name);
                        if kind == CompletionKind::Directory {
                            full.push("");
                        }
                        full.to_string_lossy().to_string()
                    } else if kind == CompletionKind::Directory {
                        format!("{}/", name)
                    } else {
                        name.to_string()
                    };

                    Some(CompletionItem {
                        text: full_text,
                        kind,
                        description: None,
                    })
                })
                .collect();

            // Sort: directories first, then files, alphabetically within each group
            matches.sort_by(|a, b| {
                match (a.kind, b.kind) {
                    (CompletionKind::Directory, CompletionKind::File) => std::cmp::Ordering::Less,
                    (CompletionKind::File, CompletionKind::Directory) => {
                        std::cmp::Ordering::Greater
                    }
                    _ => a.text.cmp(&b.text),
                }
            });

            items.extend(matches.into_iter().take(50));
        }

        items
    }

    /// Get recent history items
    fn recent_history(&self, limit: usize) -> Vec<CompletionItem> {
        self.history
            .iter()
            .take(limit)
            .map(|cmd| CompletionItem {
                text: cmd.clone(),
                kind: CompletionKind::History,
                description: Some("recent".to_string()),
            })
            .collect()
    }

    /// Get history for search/filtering
    pub fn get_history(&self) -> &[String] {
        &self.history
    }

    /// Clear all history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

impl Default for CompletionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_engine_new() {
        let engine = CompletionEngine::new();
        assert!(engine.path_commands.len() > 0, "Should load PATH commands");
    }

    #[test]
    fn test_add_to_history() {
        let mut engine = CompletionEngine::new();
        engine.add_to_history("ls -la");
        engine.add_to_history("cd /tmp");
        engine.add_to_history("pwd");

        assert_eq!(engine.history.len(), 3);
        assert_eq!(engine.history[0], "pwd");
        assert_eq!(engine.history[1], "cd /tmp");
        assert_eq!(engine.history[2], "ls -la");
    }

    #[test]
    fn test_history_deduplication() {
        let mut engine = CompletionEngine::new();
        engine.add_to_history("ls");
        engine.add_to_history("pwd");
        engine.add_to_history("ls");

        assert_eq!(engine.history.len(), 2);
        assert_eq!(engine.history[0], "ls");
        assert_eq!(engine.history[1], "pwd");
    }

    #[test]
    fn test_complete_command() {
        let mut engine = CompletionEngine::new();
        engine.add_to_history("git status");
        engine.add_to_history("git log");

        let completions = engine.complete("git", "/tmp");
        assert!(
            completions.iter().any(|c| c.text.starts_with("git")),
            "Should include git completions"
        );
    }

    #[test]
    fn test_recent_history() {
        let mut engine = CompletionEngine::new();
        for i in 0..15 {
            engine.add_to_history(&format!("command{}", i));
        }

        let recent = engine.recent_history(10);
        assert_eq!(recent.len(), 10);
        assert_eq!(recent[0].text, "command14");
        assert_eq!(recent[0].kind, CompletionKind::History);
    }

    #[test]
    fn test_empty_input_returns_history() {
        let mut engine = CompletionEngine::new();
        engine.add_to_history("test command");

        let completions = engine.complete("", "/tmp");
        assert!(
            completions.iter().any(|c| c.kind == CompletionKind::History),
            "Empty input should return history"
        );
    }

    #[test]
    fn test_completion_kind_prefix() {
        assert_eq!(CompletionKind::Command.prefix(), "cmd");
        assert_eq!(CompletionKind::File.prefix(), "file");
        assert_eq!(CompletionKind::Directory.prefix(), "dir");
        assert_eq!(CompletionKind::History.prefix(), "hist");
        assert_eq!(CompletionKind::Alias.prefix(), "alias");
    }
}
