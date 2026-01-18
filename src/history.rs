//! Command history management with reverse search functionality
//!
//! Provides Ctrl+R style reverse-i-search for command history,
//! with support for file persistence and search state management.

use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

/// Represents a single command in history
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoryEntry {
    /// The command text
    pub command: String,
    /// Unix timestamp when command was executed
    pub timestamp: Option<i64>,
    /// Working directory when command was executed
    pub cwd: Option<String>,
}

impl HistoryEntry {
    /// Create a new history entry
    pub fn new(command: String, cwd: Option<String>) -> Self {
        Self {
            command,
            timestamp: Some(chrono::Utc::now().timestamp()),
            cwd,
        }
    }

    /// Create an entry without timestamp
    pub fn simple(command: String) -> Self {
        Self {
            command,
            timestamp: None,
            cwd: None,
        }
    }
}

/// Manages command history with search functionality
#[derive(Debug, Clone)]
pub struct HistoryManager {
    /// All history entries (oldest first)
    entries: Vec<HistoryEntry>,
    /// Path to history file
    file_path: Option<PathBuf>,
    /// Maximum number of entries to keep
    max_size: usize,
    /// Whether reverse search is active
    search_mode: bool,
    /// Current search query
    search_query: String,
    /// Current match index (into filtered results)
    search_index: Option<usize>,
    /// Cached search results (indices into entries vec)
    search_results: Vec<usize>,
    /// Whether to ignore duplicate consecutive entries
    ignore_duplicates: bool,
    /// Whether to ignore commands starting with space
    ignore_space: bool,
}

impl HistoryManager {
    /// Create a new history manager
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: Vec::new(),
            file_path: None,
            max_size,
            search_mode: false,
            search_query: String::new(),
            search_index: None,
            search_results: Vec::new(),
            ignore_duplicates: true,
            ignore_space: true,
        }
    }

    /// Create with configuration options
    pub fn with_config(
        max_size: usize,
        ignore_duplicates: bool,
        ignore_space: bool,
    ) -> Self {
        Self {
            entries: Vec::new(),
            file_path: None,
            max_size,
            search_mode: false,
            search_query: String::new(),
            search_index: None,
            search_results: Vec::new(),
            ignore_duplicates,
            ignore_space,
        }
    }

    /// Load history from file
    pub fn load_from_file(&mut self, path: PathBuf) -> std::io::Result<()> {
        if !path.exists() {
            // Create parent directory if needed
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            // Create empty file
            File::create(&path)?;
        }

        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        self.entries.clear();
        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if !line.is_empty() {
                // Try to parse as JSON first (for rich format)
                if let Ok(entry) = serde_json::from_str::<HistoryEntry>(line) {
                    self.entries.push(entry);
                } else {
                    // Fall back to plain text format
                    self.entries.push(HistoryEntry::simple(line.to_string()));
                }
            }
        }

        self.file_path = Some(path);
        tracing::info!("Loaded {} history entries", self.entries.len());
        Ok(())
    }

    /// Save history to file
    pub fn save_to_file(&self) -> std::io::Result<()> {
        if let Some(path) = &self.file_path {
            // Create parent directory if needed
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)?;

            // Write most recent entries (up to max_size)
            let start = if self.entries.len() > self.max_size {
                self.entries.len() - self.max_size
            } else {
                0
            };

            for entry in &self.entries[start..] {
                // Write as JSON for rich format
                let json = serde_json::to_string(entry).unwrap_or_else(|_| {
                    // Fallback to plain text if JSON fails
                    entry.command.clone()
                });
                writeln!(file, "{json}")?;
            }

            tracing::debug!("Saved {} history entries to {:?}", self.entries.len(), path);
            Ok(())
        } else {
            Ok(())
        }
    }

    /// Add a command to history
    pub fn add(&mut self, command: String, cwd: Option<String>) {
        // Ignore empty commands
        if command.trim().is_empty() {
            return;
        }

        // Ignore commands starting with space if configured
        if self.ignore_space && command.starts_with(' ') {
            return;
        }

        // Ignore duplicate consecutive entries if configured
        if self.ignore_duplicates {
            if let Some(last) = self.entries.last() {
                if last.command == command {
                    return;
                }
            }
        }

        let entry = HistoryEntry::new(command, cwd);
        self.entries.push(entry);

        // Trim to max size
        if self.entries.len() > self.max_size {
            self.entries.remove(0);
        }
    }

    /// Start reverse search mode
    pub fn start_reverse_search(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
        self.search_index = None;
        self.search_results.clear();
        tracing::debug!("Started reverse search mode");
    }

    /// Update search query and find matches
    pub fn update_search(&mut self, query: &str) {
        self.search_query = query.to_string();
        self.search_results.clear();

        if query.is_empty() {
            self.search_index = None;
            return;
        }

        // Search backwards through history
        let query_lower = query.to_lowercase();
        for (idx, entry) in self.entries.iter().enumerate().rev() {
            if entry.command.to_lowercase().contains(&query_lower) {
                self.search_results.push(idx);
            }
        }

        // Set index to first match (most recent)
        self.search_index = if self.search_results.is_empty() {
            None
        } else {
            Some(0)
        };

        tracing::debug!(
            "Search updated: query='{}', {} matches",
            query,
            self.search_results.len()
        );
    }

    /// Get current search match
    pub fn current_match(&self) -> Option<&HistoryEntry> {
        if let Some(idx) = self.search_index {
            let entry_idx = self.search_results.get(idx)?;
            self.entries.get(*entry_idx)
        } else {
            None
        }
    }

    /// Move to next match (older in history)
    pub fn next_match(&mut self) -> bool {
        if self.search_results.is_empty() {
            return false;
        }

        if let Some(idx) = self.search_index {
            if idx + 1 < self.search_results.len() {
                self.search_index = Some(idx + 1);
                tracing::debug!("Moved to next match: {}/{}", idx + 2, self.search_results.len());
                return true;
            }
        }
        false
    }

    /// Move to previous match (newer in history)
    pub fn prev_match(&mut self) -> bool {
        if self.search_results.is_empty() {
            return false;
        }

        if let Some(idx) = self.search_index {
            if idx > 0 {
                self.search_index = Some(idx - 1);
                tracing::debug!("Moved to prev match: {}/{}", idx, self.search_results.len());
                return true;
            }
        }
        false
    }

    /// End search and return selected command
    pub fn end_search(&mut self) -> Option<String> {
        self.search_mode = false;
        let result = self.current_match().map(|e| e.command.clone());
        self.search_query.clear();
        self.search_index = None;
        self.search_results.clear();
        tracing::debug!("Ended search mode, selected: {:?}", result);
        result
    }

    /// Cancel search without returning command
    pub fn cancel_search(&mut self) {
        self.search_mode = false;
        self.search_query.clear();
        self.search_index = None;
        self.search_results.clear();
        tracing::debug!("Cancelled search mode");
    }

    /// Check if in search mode
    pub fn is_searching(&self) -> bool {
        self.search_mode
    }

    /// Get current search query
    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    /// Get number of search results
    pub fn search_result_count(&self) -> usize {
        self.search_results.len()
    }

    /// Get current search position (1-based)
    pub fn search_position(&self) -> Option<usize> {
        self.search_index.map(|idx| idx + 1)
    }

    /// Get most recent n entries
    pub fn recent(&self, n: usize) -> Vec<&HistoryEntry> {
        let start = if self.entries.len() > n {
            self.entries.len() - n
        } else {
            0
        };
        self.entries[start..].iter().rev().collect()
    }

    /// Get all entries
    pub fn all(&self) -> &[HistoryEntry] {
        &self.entries
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.entries.clear();
        self.search_query.clear();
        self.search_index = None;
        self.search_results.clear();
    }

    /// Get total number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_add_and_retrieve() {
        let mut history = HistoryManager::new(100);

        history.add("ls -la".to_string(), Some("/home".to_string()));
        history.add("cd /tmp".to_string(), Some("/home".to_string()));
        history.add("pwd".to_string(), Some("/tmp".to_string()));

        assert_eq!(history.len(), 3);
        let recent = history.recent(3);
        assert_eq!(recent[0].command, "pwd");
        assert_eq!(recent[1].command, "cd /tmp");
        assert_eq!(recent[2].command, "ls -la");
    }

    #[test]
    fn test_ignore_duplicates() {
        let mut history = HistoryManager::new(100);

        history.add("ls".to_string(), None);
        history.add("ls".to_string(), None);
        history.add("pwd".to_string(), None);
        history.add("pwd".to_string(), None);

        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_ignore_space() {
        let mut history = HistoryManager::new(100);

        history.add(" secret command".to_string(), None);
        history.add("normal command".to_string(), None);

        assert_eq!(history.len(), 1);
        assert_eq!(history.recent(1)[0].command, "normal command");
    }

    #[test]
    fn test_max_size() {
        let mut history = HistoryManager::new(3);

        history.add("cmd1".to_string(), None);
        history.add("cmd2".to_string(), None);
        history.add("cmd3".to_string(), None);
        history.add("cmd4".to_string(), None);

        assert_eq!(history.len(), 3);
        assert_eq!(history.recent(3)[2].command, "cmd2");
        assert_eq!(history.recent(3)[0].command, "cmd4");
    }

    #[test]
    fn test_reverse_search() {
        let mut history = HistoryManager::new(100);

        history.add("git status".to_string(), None);
        history.add("git commit -m 'test'".to_string(), None);
        history.add("git push".to_string(), None);
        history.add("ls -la".to_string(), None);

        history.start_reverse_search();
        assert!(history.is_searching());

        history.update_search("git");
        assert_eq!(history.search_result_count(), 3);

        let current = history.current_match().unwrap();
        assert_eq!(current.command, "git push"); // Most recent

        history.next_match();
        let current = history.current_match().unwrap();
        assert_eq!(current.command, "git commit -m 'test'");

        history.next_match();
        let current = history.current_match().unwrap();
        assert_eq!(current.command, "git status");

        let selected = history.end_search();
        assert_eq!(selected, Some("git status".to_string()));
        assert!(!history.is_searching());
    }

    #[test]
    fn test_search_no_matches() {
        let mut history = HistoryManager::new(100);

        history.add("ls".to_string(), None);
        history.add("pwd".to_string(), None);

        history.start_reverse_search();
        history.update_search("nonexistent");

        assert_eq!(history.search_result_count(), 0);
        assert!(history.current_match().is_none());
    }

    #[test]
    fn test_search_case_insensitive() {
        let mut history = HistoryManager::new(100);

        history.add("Git Status".to_string(), None);

        history.start_reverse_search();
        history.update_search("git");

        assert_eq!(history.search_result_count(), 1);
        assert_eq!(history.current_match().unwrap().command, "Git Status");
    }

    #[test]
    fn test_file_persistence() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        // Write some test data
        writeln!(temp_file, r#"{{"command":"test1","timestamp":null,"cwd":null}}"#).unwrap();
        writeln!(temp_file, r#"{{"command":"test2","timestamp":null,"cwd":null}}"#).unwrap();
        writeln!(temp_file, "plain text command").unwrap();
        temp_file.flush().unwrap();

        let mut history = HistoryManager::new(100);
        history.load_from_file(path.clone()).unwrap();

        assert_eq!(history.len(), 3);
        assert_eq!(history.all()[0].command, "test1");
        assert_eq!(history.all()[1].command, "test2");
        assert_eq!(history.all()[2].command, "plain text command");

        // Add more and save
        history.add("new command".to_string(), None);
        history.save_to_file().unwrap();

        // Load again to verify
        let mut history2 = HistoryManager::new(100);
        history2.load_from_file(path).unwrap();
        assert_eq!(history2.len(), 4);
        assert_eq!(history2.all()[3].command, "new command");
    }

    #[test]
    fn test_cancel_search() {
        let mut history = HistoryManager::new(100);
        history.add("test".to_string(), None);

        history.start_reverse_search();
        history.update_search("test");
        assert!(history.is_searching());

        history.cancel_search();
        assert!(!history.is_searching());
        assert_eq!(history.search_query(), "");
    }
}
