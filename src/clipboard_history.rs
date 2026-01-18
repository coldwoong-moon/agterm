//! Clipboard history management with pin and search functionality
//!
//! Provides clipboard history tracking with features like:
//! - Content type detection (text, path, url)
//! - Pinned items for frequently used content
//! - Duplicate removal
//! - File persistence
//! - Search and filtering

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

/// Type of clipboard content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClipboardType {
    /// Plain text
    Text,
    /// File path (absolute or relative)
    Path,
    /// URL (http, https, ftp, etc.)
    Url,
    /// Email address
    Email,
    /// Code snippet (detected by common patterns)
    Code,
}

impl ClipboardType {
    /// Detect content type from text
    pub fn detect(content: &str) -> Self {
        let trimmed = content.trim();

        // Check for URL
        if trimmed.starts_with("http://")
            || trimmed.starts_with("https://")
            || trimmed.starts_with("ftp://")
            || trimmed.starts_with("file://") {
            return ClipboardType::Url;
        }

        // Check for email
        if trimmed.contains('@') && trimmed.contains('.') && !trimmed.contains(' ') {
            let parts: Vec<&str> = trimmed.split('@').collect();
            if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
                return ClipboardType::Email;
            }
        }

        // Check for file path (Unix or Windows style)
        if trimmed.starts_with('/')
            || trimmed.starts_with("~/")
            || (trimmed.len() > 2 && trimmed.chars().nth(1) == Some(':')) {
            return ClipboardType::Path;
        }

        // Check for code patterns
        let code_indicators = [
            "fn ", "func ", "function ", "def ", "class ", "interface ",
            "import ", "from ", "use ", "#include",
            "const ", "let ", "var ", "public ", "private ",
            "{", "}", "=>", "->", "//", "/*", "*/",
        ];

        if code_indicators.iter().any(|&indicator| trimmed.contains(indicator)) {
            return ClipboardType::Code;
        }

        ClipboardType::Text
    }
}

/// Represents a single clipboard entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClipboardEntry {
    /// The clipboard content
    pub content: String,
    /// When the content was copied
    pub timestamp: DateTime<Utc>,
    /// Source identifier (e.g., terminal ID, application name)
    pub source: Option<String>,
    /// Type of content
    pub content_type: ClipboardType,
    /// Whether this entry is pinned
    pub pinned: bool,
    /// Optional label/tag for the entry
    pub label: Option<String>,
}

impl ClipboardEntry {
    /// Create a new clipboard entry
    pub fn new(content: String, source: Option<String>) -> Self {
        let content_type = ClipboardType::detect(&content);
        Self {
            content,
            timestamp: Utc::now(),
            source,
            content_type,
            pinned: false,
            label: None,
        }
    }

    /// Create a pinned entry
    pub fn pinned(content: String, label: Option<String>) -> Self {
        let content_type = ClipboardType::detect(&content);
        Self {
            content,
            timestamp: Utc::now(),
            source: None,
            content_type,
            pinned: true,
            label,
        }
    }

    /// Get a preview of the content (first line or truncated)
    pub fn preview(&self, max_len: usize) -> String {
        let first_line = self.content.lines().next().unwrap_or("");
        if first_line.len() > max_len {
            format!("{}...", &first_line[..max_len])
        } else {
            first_line.to_string()
        }
    }

    /// Check if content matches query (case-insensitive)
    pub fn matches(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.content.to_lowercase().contains(&query_lower)
            || self.label.as_ref().map_or(false, |l| l.to_lowercase().contains(&query_lower))
            || self.source.as_ref().map_or(false, |s| s.to_lowercase().contains(&query_lower))
    }
}

/// Manages clipboard history with search and pin functionality
#[derive(Debug, Clone)]
pub struct ClipboardHistory {
    /// All clipboard entries (oldest first)
    entries: Vec<ClipboardEntry>,
    /// Path to history file
    file_path: Option<PathBuf>,
    /// Maximum number of entries to keep (excluding pinned)
    max_size: usize,
    /// Whether to remove duplicate entries
    deduplicate: bool,
    /// Minimum content length to track
    min_length: usize,
    /// Maximum content length to track
    max_length: usize,
}

impl ClipboardHistory {
    /// Create a new clipboard history manager
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: Vec::new(),
            file_path: None,
            max_size,
            deduplicate: true,
            min_length: 1,
            max_length: 1_000_000, // 1MB
        }
    }

    /// Create with configuration options
    pub fn with_config(
        max_size: usize,
        deduplicate: bool,
        min_length: usize,
        max_length: usize,
    ) -> Self {
        Self {
            entries: Vec::new(),
            file_path: None,
            max_size,
            deduplicate,
            min_length,
            max_length,
        }
    }

    /// Load history from file
    pub fn load_from_file(&mut self, path: PathBuf) -> std::io::Result<()> {
        if !path.exists() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            File::create(&path)?;
        }

        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        self.entries.clear();
        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if !line.is_empty() {
                if let Ok(entry) = serde_json::from_str::<ClipboardEntry>(line) {
                    self.entries.push(entry);
                }
            }
        }

        self.file_path = Some(path);
        tracing::info!("Loaded {} clipboard entries", self.entries.len());
        Ok(())
    }

    /// Save history to file
    pub fn save_to_file(&self) -> std::io::Result<()> {
        if let Some(path) = &self.file_path {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)?;

            // Always save pinned entries
            let pinned_entries: Vec<&ClipboardEntry> = self.entries.iter()
                .filter(|e| e.pinned)
                .collect();

            // Save most recent non-pinned entries
            let unpinned_entries: Vec<&ClipboardEntry> = self.entries.iter()
                .filter(|e| !e.pinned)
                .collect();

            let start = if unpinned_entries.len() > self.max_size {
                unpinned_entries.len() - self.max_size
            } else {
                0
            };

            // Write pinned entries first
            for entry in pinned_entries {
                if let Ok(json) = serde_json::to_string(entry) {
                    writeln!(file, "{}", json)?;
                }
            }

            // Write recent unpinned entries
            for entry in &unpinned_entries[start..] {
                if let Ok(json) = serde_json::to_string(entry) {
                    writeln!(file, "{}", json)?;
                }
            }

            tracing::debug!("Saved {} clipboard entries to {:?}", self.entries.len(), path);
            Ok(())
        } else {
            Ok(())
        }
    }

    /// Add content to clipboard history
    pub fn add(&mut self, content: String, source: Option<String>) -> bool {
        // Validate content length
        if content.len() < self.min_length || content.len() > self.max_length {
            return false;
        }

        // Ignore whitespace-only content
        if content.trim().is_empty() {
            return false;
        }

        // Check for duplicates if enabled
        if self.deduplicate {
            // Remove existing duplicate (keep most recent)
            self.entries.retain(|e| e.content != content || e.pinned);
        }

        let entry = ClipboardEntry::new(content, source);
        self.entries.push(entry);

        // Trim to max size (excluding pinned entries)
        self.trim_to_size();

        true
    }

    /// Trim unpinned entries to max size
    fn trim_to_size(&mut self) {
        let unpinned_count = self.entries.iter().filter(|e| !e.pinned).count();

        if unpinned_count > self.max_size {
            let to_remove = unpinned_count - self.max_size;
            let mut removed = 0;

            self.entries.retain(|e| {
                if e.pinned || removed >= to_remove {
                    true
                } else {
                    removed += 1;
                    false
                }
            });
        }
    }

    /// Pin an entry by index
    pub fn pin(&mut self, index: usize, label: Option<String>) -> bool {
        if let Some(entry) = self.entries.get_mut(index) {
            entry.pinned = true;
            entry.label = label;
            true
        } else {
            false
        }
    }

    /// Unpin an entry by index
    pub fn unpin(&mut self, index: usize) -> bool {
        if let Some(entry) = self.entries.get_mut(index) {
            entry.pinned = false;
            true
        } else {
            false
        }
    }

    /// Toggle pin status of an entry
    pub fn toggle_pin(&mut self, index: usize, label: Option<String>) -> bool {
        if let Some(entry) = self.entries.get_mut(index) {
            entry.pinned = !entry.pinned;
            if entry.pinned {
                entry.label = label;
            }
            true
        } else {
            false
        }
    }

    /// Get all pinned entries
    pub fn pinned(&self) -> Vec<&ClipboardEntry> {
        self.entries.iter().filter(|e| e.pinned).collect()
    }

    /// Get all unpinned entries
    pub fn unpinned(&self) -> Vec<&ClipboardEntry> {
        self.entries.iter().filter(|e| !e.pinned).collect()
    }

    /// Search for entries matching query
    pub fn search(&self, query: &str) -> Vec<&ClipboardEntry> {
        if query.is_empty() {
            return self.entries.iter().collect();
        }

        self.entries.iter()
            .filter(|e| e.matches(query))
            .collect()
    }

    /// Filter entries by content type
    pub fn filter_by_type(&self, content_type: ClipboardType) -> Vec<&ClipboardEntry> {
        self.entries.iter()
            .filter(|e| e.content_type == content_type)
            .collect()
    }

    /// Get most recent n entries
    pub fn recent(&self, n: usize) -> Vec<&ClipboardEntry> {
        let start = if self.entries.len() > n {
            self.entries.len() - n
        } else {
            0
        };
        self.entries[start..].iter().rev().collect()
    }

    /// Get entry by index
    pub fn get(&self, index: usize) -> Option<&ClipboardEntry> {
        self.entries.get(index)
    }

    /// Get mutable entry by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut ClipboardEntry> {
        self.entries.get_mut(index)
    }

    /// Get all entries
    pub fn all(&self) -> &[ClipboardEntry] {
        &self.entries
    }

    /// Remove entry by index
    pub fn remove(&mut self, index: usize) -> Option<ClipboardEntry> {
        if index < self.entries.len() {
            Some(self.entries.remove(index))
        } else {
            None
        }
    }

    /// Clear all unpinned entries
    pub fn clear_unpinned(&mut self) {
        self.entries.retain(|e| e.pinned);
    }

    /// Clear all entries (including pinned)
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get total number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get number of pinned entries
    pub fn pinned_count(&self) -> usize {
        self.entries.iter().filter(|e| e.pinned).count()
    }

    /// Get unique content hashes for duplicate detection
    pub fn unique_contents(&self) -> HashSet<String> {
        self.entries.iter()
            .map(|e| e.content.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_content_type_detection() {
        assert_eq!(ClipboardType::detect("https://example.com"), ClipboardType::Url);
        assert_eq!(ClipboardType::detect("http://test.org"), ClipboardType::Url);
        assert_eq!(ClipboardType::detect("user@example.com"), ClipboardType::Email);
        assert_eq!(ClipboardType::detect("/home/user/file.txt"), ClipboardType::Path);
        assert_eq!(ClipboardType::detect("~/documents"), ClipboardType::Path);
        assert_eq!(ClipboardType::detect("C:\\Windows\\System32"), ClipboardType::Path);
        assert_eq!(ClipboardType::detect("fn main() {"), ClipboardType::Code);
        assert_eq!(ClipboardType::detect("function test() {}"), ClipboardType::Code);
        assert_eq!(ClipboardType::detect("plain text"), ClipboardType::Text);
    }

    #[test]
    fn test_add_and_retrieve() {
        let mut history = ClipboardHistory::new(100);

        assert!(history.add("First item".to_string(), Some("terminal-1".to_string())));
        assert!(history.add("Second item".to_string(), Some("terminal-2".to_string())));
        assert!(history.add("Third item".to_string(), None));

        assert_eq!(history.len(), 3);

        let recent = history.recent(3);
        assert_eq!(recent[0].content, "Third item");
        assert_eq!(recent[1].content, "Second item");
        assert_eq!(recent[2].content, "First item");
    }

    #[test]
    fn test_deduplication() {
        let mut history = ClipboardHistory::new(100);

        history.add("duplicate".to_string(), None);
        history.add("unique".to_string(), None);
        history.add("duplicate".to_string(), None);

        assert_eq!(history.len(), 2);
        assert_eq!(history.recent(2)[0].content, "duplicate");
        assert_eq!(history.recent(2)[1].content, "unique");
    }

    #[test]
    fn test_no_deduplication() {
        let mut history = ClipboardHistory::with_config(100, false, 1, 1_000_000);

        history.add("duplicate".to_string(), None);
        history.add("duplicate".to_string(), None);

        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_length_limits() {
        let mut history = ClipboardHistory::with_config(100, true, 5, 20);

        assert!(!history.add("abc".to_string(), None)); // Too short
        assert!(history.add("valid".to_string(), None)); // Valid
        assert!(!history.add("a".repeat(100), None)); // Too long
        assert!(!history.add("   ".to_string(), None)); // Whitespace only

        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_max_size() {
        let mut history = ClipboardHistory::new(3);

        history.add("item1".to_string(), None);
        history.add("item2".to_string(), None);
        history.add("item3".to_string(), None);
        history.add("item4".to_string(), None);

        assert_eq!(history.len(), 3);
        let all = history.all();
        assert_eq!(all[0].content, "item2");
        assert_eq!(all[1].content, "item3");
        assert_eq!(all[2].content, "item4");
    }

    #[test]
    fn test_pin_functionality() {
        let mut history = ClipboardHistory::new(100);

        history.add("unpinned1".to_string(), None);
        history.add("to_pin".to_string(), None);
        history.add("unpinned2".to_string(), None);

        assert!(history.pin(1, Some("Important".to_string())));

        let pinned = history.pinned();
        assert_eq!(pinned.len(), 1);
        assert_eq!(pinned[0].content, "to_pin");
        assert_eq!(pinned[0].label, Some("Important".to_string()));

        assert!(history.unpin(1));
        assert_eq!(history.pinned().len(), 0);
    }

    #[test]
    fn test_pin_survives_trimming() {
        let mut history = ClipboardHistory::new(2);

        history.add("item1".to_string(), None);
        history.add("item2".to_string(), None);
        history.pin(0, Some("Keep".to_string())); // Pin first item

        history.add("item3".to_string(), None);
        history.add("item4".to_string(), None);

        // Should have: item1(pinned), item3, item4
        assert_eq!(history.len(), 3);
        assert_eq!(history.pinned_count(), 1);

        let all = history.all();
        assert_eq!(all[0].content, "item1");
        assert!(all[0].pinned);
    }

    #[test]
    fn test_toggle_pin() {
        let mut history = ClipboardHistory::new(100);

        history.add("item".to_string(), None);

        assert!(!history.get(0).unwrap().pinned);
        history.toggle_pin(0, Some("Label".to_string()));
        assert!(history.get(0).unwrap().pinned);
        history.toggle_pin(0, None);
        assert!(!history.get(0).unwrap().pinned);
    }

    #[test]
    fn test_search() {
        let mut history = ClipboardHistory::new(100);

        history.add("git status".to_string(), None);
        history.add("git commit -m 'test'".to_string(), None);
        history.add("ls -la".to_string(), None);
        history.add("git push".to_string(), None);

        let results = history.search("git");
        assert_eq!(results.len(), 3);

        let results = history.search("commit");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "git commit -m 'test'");

        let results = history.search("nonexistent");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_by_label() {
        let mut history = ClipboardHistory::new(100);

        history.add("content1".to_string(), None);
        history.add("content2".to_string(), None);
        history.pin(0, Some("important".to_string()));

        let results = history.search("important");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "content1");
    }

    #[test]
    fn test_filter_by_type() {
        let mut history = ClipboardHistory::new(100);

        history.add("https://example.com".to_string(), None);
        history.add("plain text".to_string(), None);
        history.add("http://test.org".to_string(), None);
        history.add("/home/user/file".to_string(), None);

        let urls = history.filter_by_type(ClipboardType::Url);
        assert_eq!(urls.len(), 2);

        let paths = history.filter_by_type(ClipboardType::Path);
        assert_eq!(paths.len(), 1);

        let text = history.filter_by_type(ClipboardType::Text);
        assert_eq!(text.len(), 1);
    }

    #[test]
    fn test_entry_preview() {
        let entry = ClipboardEntry::new("Short text".to_string(), None);
        assert_eq!(entry.preview(20), "Short text");

        let long = "This is a very long text that should be truncated";
        let entry = ClipboardEntry::new(long.to_string(), None);
        assert_eq!(entry.preview(20), "This is a very long ...");

        let multiline = "First line\nSecond line\nThird line";
        let entry = ClipboardEntry::new(multiline.to_string(), None);
        assert_eq!(entry.preview(50), "First line");
    }

    #[test]
    fn test_clear_operations() {
        let mut history = ClipboardHistory::new(100);

        history.add("item1".to_string(), None);
        history.add("item2".to_string(), None);
        history.add("item3".to_string(), None);
        history.pin(1, Some("Keep".to_string()));

        history.clear_unpinned();
        assert_eq!(history.len(), 1);
        assert_eq!(history.get(0).unwrap().content, "item2");

        history.clear();
        assert!(history.is_empty());
    }

    #[test]
    fn test_remove_entry() {
        let mut history = ClipboardHistory::new(100);

        history.add("item1".to_string(), None);
        history.add("item2".to_string(), None);
        history.add("item3".to_string(), None);

        let removed = history.remove(1);
        assert_eq!(removed.unwrap().content, "item2");
        assert_eq!(history.len(), 2);

        let none = history.remove(10);
        assert!(none.is_none());
    }

    #[test]
    fn test_file_persistence() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        // Create initial history
        let mut history = ClipboardHistory::new(100);
        history.load_from_file(path.clone()).unwrap();

        history.add("entry1".to_string(), Some("source1".to_string()));
        history.add("entry2".to_string(), None);
        history.pin(0, Some("Pinned".to_string()));

        history.save_to_file().unwrap();

        // Load in new instance
        let mut history2 = ClipboardHistory::new(100);
        history2.load_from_file(path).unwrap();

        assert_eq!(history2.len(), 2);
        assert_eq!(history2.get(0).unwrap().content, "entry1");
        assert!(history2.get(0).unwrap().pinned);
        assert_eq!(history2.get(0).unwrap().label, Some("Pinned".to_string()));
        assert_eq!(history2.get(1).unwrap().content, "entry2");
    }

    #[test]
    fn test_pinned_entries_in_persistence() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        let mut history = ClipboardHistory::new(2); // Small max size
        history.load_from_file(path.clone()).unwrap();

        history.add("item1".to_string(), None);
        history.add("item2".to_string(), None);
        history.pin(0, Some("Important".to_string()));
        history.add("item3".to_string(), None);
        history.add("item4".to_string(), None);
        history.add("item5".to_string(), None);

        history.save_to_file().unwrap();

        // Load and verify pinned entry is preserved
        let mut history2 = ClipboardHistory::new(2);
        history2.load_from_file(path).unwrap();

        assert!(history2.len() <= 3); // 1 pinned + 2 unpinned max
        assert_eq!(history2.pinned_count(), 1);

        let pinned = history2.pinned();
        assert_eq!(pinned[0].content, "item1");
    }
}
